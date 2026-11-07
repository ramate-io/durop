//! Explicit, revocable, tailnet-only remote sessions, outside Bevy's frame loop.
//!
//! Local/native controllers use the application's in-process host ingress. This
//! module provides only the authenticated Tailscale transport and bounded wire client.
#![deny(missing_docs)]

use durop_protocol::{
    CommandOutcome, Envelope, HostEvent, HostSnapshot, PROTOCOL_VERSION, PlaybackCommand,
};
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{
    collections::BTreeMap,
    net::{IpAddr, SocketAddr},
    path::{Path, PathBuf},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::Duration,
};
use subtle::ConstantTimeEq;
use tokio::{
    io::AsyncReadExt,
    net::{TcpListener, TcpStream},
    sync::{Semaphore, broadcast, mpsc, oneshot, watch},
    task::{JoinHandle, JoinSet},
    time::{interval, timeout},
};
use tokio_tungstenite::{
    accept_hdr_async_with_config,
    tungstenite::{
        handshake::server::{ErrorResponse, Request, Response},
        http::StatusCode,
        protocol::{Message, WebSocketConfig},
    },
};

// Includes unauthenticated upgrades, so stalled handshakes cannot exhaust task capacity.
const MAX_CONNECTIONS: usize = 4;
// Requests are small commands/identities; full media is never accepted on this channel.
const MAX_INPUT: usize = 16 * 1024;
/// TCP port used when a tailnet IP is supplied without an explicit port.
pub const DEFAULT_LISTEN_PORT: u16 = 48732;

/// Compressed PNG size accepted in the remote catalog; JSON encodes each byte as a number.
pub const MAX_ICON_PNG: usize = 160 * 1024;
// JSON arrays expand PNG bytes about four times; keep snapshots and one icon in one frame.
const MAX_OUTPUT: usize = 768 * 1024;
const _: () = assert!(MAX_OUTPUT >= MAX_ICON_PNG * 4 + 1024);
// Each stage has its own deadline; an idle or backpressured peer cannot pin a task.
const HANDSHAKE_TIMEOUT: Duration = Duration::from_secs(3);
const IO_TIMEOUT: Duration = Duration::from_secs(5);
const COMMAND_TIMEOUT: Duration = Duration::from_secs(2);
const IDLE_TIMEOUT: Duration = Duration::from_secs(60);
// Control frames count too, so ping traffic cannot bypass the authenticated rate limit.
const REQUESTS_PER_SECOND: usize = 20;
// Negotiate the version before accepting any application payload.
const SUBPROTOCOL: &str = "durop.v1";

/// Binding verification mode; only isolated tests may omit tailnet verification.
enum BindingGuard {
    // Recheck both OS assignment and Tailscale self identity throughout the session.
    Tailnet,
    #[cfg(any(test, feature = "test-support"))]
    // Explicit default-off fixture support; production never chooses a fallback binding.
    Loopback,
}

/// Failure to validate, enable, or maintain a remote session.
#[derive(Debug)]
pub enum TransportError {
    /// The selected address, origin list, credential, or catalog exceeds policy bounds.
    InvalidConfiguration,
    /// The trusted catalog projection exceeded icon or metadata wire bounds.
    InvalidCatalog,
    /// The configured address or installed Tailscale CLI cannot verify a running tailnet.
    TailnetUnavailable,
    /// An operating-system socket or interface operation failed.
    Io(std::io::Error),
    /// The operating system could not provide secure credential randomness.
    Random,
}

impl From<std::io::Error> for TransportError {
    fn from(error: std::io::Error) -> Self {
        Self::Io(error)
    }
}

impl std::fmt::Display for TransportError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidConfiguration => f.write_str("invalid remote session configuration"),
            Self::InvalidCatalog => f.write_str("remote catalog preview exceeds sharing bounds"),
            Self::TailnetUnavailable => f.write_str("configured Tailscale address or CLI is unavailable; install/authorize Tailscale and verify this Mac's tailnet address (the macOS app's bundled CLI is supported)"),
            Self::Io(error) => write!(f, "transport I/O: {error}"),
            Self::Random => f.write_str("secure randomness unavailable"),
        }
    }
}

impl std::error::Error for TransportError {}

/// Only the enable/pairing UI should read this value. Debug never prints credentials.
pub struct SessionCredential(String);

impl std::fmt::Debug for SessionCredential {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("SessionCredential([REDACTED])")
    }
}

impl SessionCredential {
    /// Accept an explicitly entered pairing secret. Do not persist or log the input.
    pub fn from_secret(secret: String) -> Result<Self, TransportError> {
        if secret.len() != 64 || !secret.bytes().all(|byte| byte.is_ascii_hexdigit()) {
            return Err(TransportError::InvalidConfiguration);
        }
        Ok(Self(secret))
    }

    /// Reveal the pairing secret only for an explicit authenticated upgrade or pairing UI.
    pub fn bearer(&self) -> &str {
        &self.0
    }
}

/// Encode fresh operating-system randomness for the private credential constructor.
/// The returned bytes are sensitive even though this temporary value is a String.
fn random_hex<const N: usize>() -> Result<String, TransportError> {
    let mut bytes = [0; N];
    getrandom::fill(&mut bytes).map_err(|_| TransportError::Random)?;
    Ok(bytes.iter().map(|byte| format!("{byte:02x}")).collect())
}

/// Exact origins only. An empty list permits native clients with no Origin.
pub struct SessionConfig {
    /// Exact assigned Tailscale address and nonzero listener port; never a wildcard.
    pub address: SocketAddr,
    /// At most eight exact web origins; native clients may omit Origin entirely.
    pub allowed_origins: Vec<String>,
}

impl SessionConfig {
    // Validate configuration before opening a socket or invoking the Tailscale CLI.
    fn validate(&self) -> Result<(), TransportError> {
        if self.address.port() == 0
            || !is_tailnet_ip(self.address.ip())
            || self.allowed_origins.len() > 8
            || self.allowed_origins.iter().any(|origin| {
                origin.len() > 256
                    || !(origin.starts_with("http://") || origin.starts_with("https://"))
                    || origin.contains(['\r', '\n', '*', '?', '#'])
                    || origin.ends_with('/')
            })
        {
            return Err(TransportError::InvalidConfiguration);
        }
        Ok(())
    }
}

/// In-memory catalog projection. The trusted loader supplies validated PNGs; no path
/// is ever accepted from a controller, nor is the filesystem read by this adapter.
#[derive(Clone)]
pub struct RemoteCatalog {
    // Already-projected public metadata; native catalog paths must not enter this value.
    metadata: Value,
    // Immutable exact-ID allowlist; lookup never interprets a controller string as a path.
    icons: BTreeMap<String, Vec<u8>>,
}

/// Bound decoded PNG allocation as well as compressed wire size.
pub fn valid_icon_png(png: &[u8]) -> bool {
    png.len() >= 33
        && png.len() <= MAX_ICON_PNG
        && png.starts_with(b"\x89PNG\r\n\x1a\n")
        && png[8..12] == 13u32.to_be_bytes()
        && &png[12..16] == b"IHDR"
        && [
            u32::from_be_bytes(png[16..20].try_into().unwrap()),
            u32::from_be_bytes(png[20..24].try_into().unwrap()),
        ]
        .iter()
        .all(|dimension| (1..=512).contains(dimension))
}

impl RemoteCatalog {
    /// Construct a bounded, controller-safe metadata projection and known PNG icon map.
    /// The trusted catalog loader supplies the bytes; the wire never accepts asset paths.
    pub fn new(metadata: Value, icons: BTreeMap<String, Vec<u8>>) -> Result<Self, TransportError> {
        if serde_json::to_vec(&metadata)
            .map_err(|_| TransportError::InvalidCatalog)?
            .len()
            > 64 * 1024
            || icons.len() > 256
            || icons
                .iter()
                .any(|(id, png)| id.is_empty() || id.len() > 256 || !valid_icon_png(png))
        {
            return Err(TransportError::InvalidCatalog);
        }
        Ok(Self { metadata, icons })
    }
}

/// Drain with try_recv on the host thread. Check is_active immediately before
/// Host::accept, skip stale requests, then reply. StopAll uses a separate bounded lane.
pub struct RemoteCommand {
    /// Host policy fields and command. Registry sessions namespace command IDs by
    /// authenticated source; legacy sessions preserve the original command ID.
    pub envelope: Envelope,
    /// One reply to the originating connection; closure also cancels pending admission.
    pub reply: oneshot::Sender<CommandOutcome>,
    /// Host-assigned identity for registry authentication; legacy sessions return None.
    pub source_id: Option<String>,
    // Revoking this controller invalidates queued commands independently of the session.
    source: Arc<AtomicBool>,
    // Dropping the originating socket task invalidates requests already in the host queue.
    connection: Arc<AtomicBool>,
    // Ending host permission invalidates every connection's pending requests together.
    session: Arc<AtomicBool>,
}

impl RemoteCommand {
    /// Whether source/connection/session permission and the reply receiver remain live.
    /// The host must check this immediately before accepting the command.
    pub fn is_active(&self) -> bool {
        self.source.load(Ordering::Acquire)
            && self.connection.load(Ordering::Acquire)
            && self.session.load(Ordering::Acquire)
            && !self.reply.is_closed()
    }
}

/// Host-owned bounded ingress plus authoritative state/event publication.
#[derive(Clone)]
pub struct HostBridge {
    /// Ordinary requests; a full queue produces an explicit busy reply.
    pub commands: mpsc::Sender<RemoteCommand>,
    /// Separate StopAll lane that cannot be starved by ordinary queue saturation.
    pub emergency: mpsc::Sender<RemoteCommand>,
    /// Latest complete host state, published before corresponding events.
    pub snapshot: watch::Receiver<HostSnapshot>,
    /// Sequenced events; lag causes a replacement snapshot rather than inferred state.
    pub events: broadcast::Sender<HostEvent>,
}

/// Drop/revoke closes the listener and connections. Enabling again generates a new
/// credential. The host supplies its current session identity; deduplication stays there.
pub struct RemoteSession {
    /// The exact interface address and port bound by this session.
    pub address: SocketAddr,
    /// Host-owned recording session identity used for command validation/deduplication.
    pub session_id: String,
    /// Legacy pairing credential; redacted by Debug. Registry-only sessions reject
    /// this value and require a credential issued by `SourceAccessRegistry`.
    pub credential: SessionCredential,
    // Synchronous permission check shared with connections and application status views.
    active: Arc<AtomicBool>,
    // Wakes tasks blocked on socket I/O when the owner explicitly ends access.
    revoke: watch::Sender<bool>,
    // Owns the listener and joins/aborts all bounded connection tasks during shutdown.
    task: JoinHandle<()>,
}

impl RemoteSession {
    /// Report current listener permission, including background verification failure.
    pub fn is_active(&self) -> bool {
        self.active.load(Ordering::Acquire)
    }

    /// Immediately cancel permission and signal listener/connection shutdown.
    pub fn revoke(&self) {
        self.active.store(false, Ordering::Release);
        self.revoke.send_replace(true);
    }

    /// Revoke access and wait for the bounded listener/connection tasks to finish.
    pub async fn close(mut self) {
        self.revoke();
        let _ = (&mut self.task).await;
    }
}

impl Drop for RemoteSession {
    fn drop(&mut self) {
        self.revoke();
    }
}

/// Enabling is explicit. Production accepts only an assigned IP corroborated by
/// `tailscale status --json`; wildcard, LAN, and loopback addresses are rejected.
pub async fn enable(
    config: SessionConfig,
    bridge: HostBridge,
    catalog: RemoteCatalog,
) -> Result<RemoteSession, TransportError> {
    config.validate()?;
    verify_tailnet_address(config.address.ip()).await?;
    let listener = TcpListener::bind(config.address).await?;
    start(
        listener,
        config.allowed_origins,
        bridge,
        catalog,
        BindingGuard::Tailnet,
    )
}

/// Enable a listener accepting only credentials issued by the supplied registry.
/// The compatibility `RemoteSession::credential` is deliberately rejected here.
/// Each authenticated source gets independent command identity and revocation guards.
pub async fn enable_with_registry(
    config: SessionConfig,
    bridge: HostBridge,
    catalog: RemoteCatalog,
    registry: SourceAccessRegistry,
) -> Result<RemoteSession, TransportError> {
    config.validate()?;
    verify_tailnet_address(config.address.ip()).await?;
    let listener = TcpListener::bind(config.address).await?;
    start_with_access(
        listener,
        config.allowed_origins,
        bridge,
        catalog,
        BindingGuard::Tailnet,
        Some(registry),
    )
}

#[cfg(any(test, feature = "test-support"))]
/// Enable an authenticated loopback fixture only in explicit test-support builds.
pub async fn enable_test_loopback(
    bridge: HostBridge,
    catalog: RemoteCatalog,
) -> Result<RemoteSession, TransportError> {
    let listener = TcpListener::bind("127.0.0.1:0").await?;
    start(listener, vec![], bridge, catalog, BindingGuard::Loopback)
}

#[cfg(any(test, feature = "test-support"))]
/// Enable a registry-only loopback fixture under the explicit test-support feature.
pub async fn enable_test_loopback_with_registry(
    bridge: HostBridge,
    catalog: RemoteCatalog,
    registry: SourceAccessRegistry,
) -> Result<RemoteSession, TransportError> {
    let listener = TcpListener::bind("127.0.0.1:0").await?;
    start_with_access(
        listener,
        vec![],
        bridge,
        catalog,
        BindingGuard::Loopback,
        Some(registry),
    )
}

/// Attach lifecycle ownership to an already verified listener and fresh credential.
/// The drop guard also revokes permission if the task is aborted before its first poll.
fn start(
    listener: TcpListener,
    origins: Vec<String>,
    bridge: HostBridge,
    catalog: RemoteCatalog,
    guard: BindingGuard,
) -> Result<RemoteSession, TransportError> {
    start_with_access(listener, origins, bridge, catalog, guard, None)
}

/// Construct either legacy single-secret or registry-only listener authentication.
fn start_with_access(
    listener: TcpListener,
    origins: Vec<String>,
    bridge: HostBridge,
    catalog: RemoteCatalog,
    guard: BindingGuard,
    registry: Option<SourceAccessRegistry>,
) -> Result<RemoteSession, TransportError> {
    let address = listener.local_addr()?;
    let session_id = bridge.snapshot.borrow().session_id.clone();
    if session_id.is_empty() || session_id.len() > 128 {
        return Err(TransportError::InvalidConfiguration);
    }
    let credential = SessionCredential(random_hex::<32>()?);
    let expected_authorization = format!("Bearer {}", credential.bearer());
    let active = Arc::new(AtomicBool::new(true));
    let (revoke, mut revoked) = watch::channel(false);
    let state = Arc::new(ConnectionState {
        expected_authorization,
        registry,
        origins,
        active: active.clone(),
        bridge,
        catalog,
        session_id: session_id.clone(),
    });
    let lifetime = ConnectionLifetime(active.clone());
    let task = tokio::spawn(async move {
        let _lifetime = lifetime;
        let mut connections = JoinSet::new();
        let slots = Arc::new(Semaphore::new(MAX_CONNECTIONS));
        let mut check = interval(Duration::from_secs(2));
        loop {
            tokio::select! {
                biased;
                _ = revoked.changed() => break,
                _ = check.tick(), if matches!(guard, BindingGuard::Tailnet) => {
                    if verify_tailnet_address(address.ip()).await.is_err() {
                        break;
                    }
                }
                accepted = listener.accept() => {
                    let Ok((tcp, _peer)) = accepted else {
                        break;
                    };
                    let Ok(permit) = slots.clone().try_acquire_owned() else {
                        drop(tcp);
                        continue;
                    };
                    let state = state.clone();
                    let revoked = revoked.clone();
                    connections.spawn(async move {
                        let _permit = permit;
                        connection(tcp, state, revoked).await;
                    });
                }
                Some(_) = connections.join_next(), if !connections.is_empty() => {}
            }
        }
        state.active.store(false, Ordering::Release);
        connections.abort_all();
        while connections.join_next().await.is_some() {}
    });
    Ok(RemoteSession {
        address,
        session_id,
        credential,
        active,
        revoke,
        task,
    })
}

/// Immutable session policy shared by its bounded authenticated connection tasks.
struct ConnectionState {
    // Compared in constant time during upgrade; never included in responses or Debug output.
    expected_authorization: String,
    // When present, legacy authorization is rejected; identity comes only from the secret.
    registry: Option<SourceAccessRegistry>,
    // Exact browser origins, independent of tailnet membership and bearer permission.
    origins: Vec<String>,
    // Shared session permission; revocation is checked again before command admission.
    active: Arc<AtomicBool>,
    // Host-owned queues and authoritative publication, with no playback objects here.
    bridge: HostBridge,
    // Pinned metadata/icons cannot change beneath an authenticated connection.
    catalog: RemoteCatalog,
    // Binds this credential to one host recording identity.
    session_id: String,
}

/// Fail closed when a listener or connection task exits, including cancellation.
struct ConnectionLifetime(Arc<AtomicBool>);

impl Drop for ConnectionLifetime {
    fn drop(&mut self) {
        self.0.store(false, Ordering::Release);
    }
}

/// One in-flight command reply; the socket remains readable during this wait.
struct PendingReply {
    // Echoed only to the same authenticated requester to correlate its outcome.
    command_id: String,
    // Closing this receiver cancels admission via RemoteCommand::is_active.
    received: oneshot::Receiver<CommandOutcome>,
    // Absolute deadline survives intervening socket messages and host events.
    deadline: tokio::time::Instant,
}

/// Strict allowlisted controller requests; no arbitrary files or external URLs.
#[derive(Deserialize)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
enum ClientMessage {
    // Carries canonical policy data; the host still validates revision and clip identity.
    Command { envelope: Envelope },
    // Cancellation affects only the same authenticated socket's outstanding request.
    Cancel { command_id: String },
    // Reconnect/gap recovery always asks for complete authoritative state.
    Snapshot,
    // Browsing stays on the authenticated socket rather than a second HTTP server.
    Catalog,
    // Exact catalog key only, with no filesystem-path interpretation.
    Icon { clip_id: String },
}

impl ClientMessage {
    /// Validate unit-variant keys explicitly before Serde interprets the request.
    fn parse(text: &str) -> Option<Self> {
        let value: Value = serde_json::from_str(text).ok()?;
        let object = value.as_object()?;
        // Serde's internally tagged unit variants otherwise accept extra fields.
        let allowed: &[&str] = match object.get("type")?.as_str()? {
            "snapshot" | "catalog" => &["type"],
            "icon" => &["type", "clip_id"],
            "command" => &["type", "envelope"],
            "cancel" => &["type", "command_id"],
            _ => return None,
        };
        if object.len() != allowed.len()
            || object.keys().any(|key| !allowed.contains(&key.as_str()))
        {
            return None;
        }
        serde_json::from_value(value).ok()
    }
}

/// Owned-by-host response shapes serialized only after upgrade authentication.
#[derive(Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum ServerMessage<'a> {
    // Complete state replaces missing events; consumers use its sequence boundary.
    Snapshot {
        snapshot: &'a HostSnapshot,
    },
    // Pre-sanitized metadata borrowed from the immutable session projection.
    Catalog {
        catalog: &'a Value,
    },
    // Known PNG bytes only; compressed size and IHDR dimensions were checked at construction.
    Icon {
        clip_id: &'a str,
        png: &'a [u8],
    },
    // Correlation is per authenticated connection, not inferred from playback events.
    Outcome {
        command_id: &'a str,
        outcome: CommandOutcome,
    },
    // A distinct barrier lets a client drain a racing abandoned outcome without ambiguity.
    Canceled {
        command_id: &'a str,
    },
    // Sequenced host transitions do not imply physical presentation.
    Event {
        event: &'a HostEvent,
    },
    // Fixed protocol codes avoid exposing internal paths or implementation errors.
    Error {
        code: &'a str,
    },
}

// Require path, bearer, origin, version, and live permission before accepting the upgrade.
// tungstenite's public Callback signature fixes this response type.
#[allow(clippy::result_large_err)]
fn handshake(
    request: &Request,
    mut response: Response,
    state: &ConnectionState,
    lease: &mut Option<access::AccessLease>,
) -> Result<Response, ErrorResponse> {
    let authorized = request.headers().get_all("authorization").iter().count() == 1
        && request
            .headers()
            .get("authorization")
            .map(|value| {
                if let Some(registry) = &state.registry {
                    *lease = registry.authenticate(value.as_bytes());
                    lease.is_some()
                } else {
                    bool::from(
                        value
                            .as_bytes()
                            .ct_eq(state.expected_authorization.as_bytes()),
                    )
                }
            })
            .unwrap_or(false);
    let origin = match request.headers().get_all("origin").iter().count() {
        0 => true,
        1 => request
            .headers()
            .get("origin")
            .and_then(|v| v.to_str().ok())
            .is_some_and(|origin| state.origins.iter().any(|allowed| allowed == origin)),
        _ => false,
    };
    let protocol = request
        .headers()
        .get_all("sec-websocket-protocol")
        .iter()
        .count()
        == 1
        && request
            .headers()
            .get("sec-websocket-protocol")
            .and_then(|v| v.to_str().ok())
            == Some(SUBPROTOCOL);
    if request.uri().path_and_query().map(|value| value.as_str()) != Some("/control")
        || !authorized
        || !origin
        || !protocol
        || !state.active.load(Ordering::Acquire)
    {
        let mut denied = ErrorResponse::new(Some("remote session denied".into()));
        *denied.status_mut() = StatusCode::FORBIDDEN;
        return Err(denied);
    }
    response.headers_mut().insert(
        "sec-websocket-protocol",
        SUBPROTOCOL.parse().expect("constant HTTP header"),
    );
    Ok(response)
}

/// Keep one bounded socket readable while awaiting its single host reply.
/// Revocation and EOF cancel queued work; previously accepted playback belongs to the host.
async fn connection(
    tcp: TcpStream,
    state: Arc<ConnectionState>,
    mut revoked: watch::Receiver<bool>,
) {
    let live = Arc::new(AtomicBool::new(true));
    let _lifetime = ConnectionLifetime(live.clone());
    let config = WebSocketConfig::default()
        .max_message_size(Some(MAX_INPUT))
        .max_frame_size(Some(MAX_INPUT))
        .write_buffer_size(0)
        .max_write_buffer_size(MAX_OUTPUT + 1024);
    // Callback requires an unboxed ErrorResponse (an upstream API constraint).
    let mut lease = None;
    #[allow(clippy::result_large_err)]
    let upgraded = timeout(
        HANDSHAKE_TIMEOUT,
        accept_hdr_async_with_config(
            tcp,
            |request: &Request, response| handshake(request, response, &state, &mut lease),
            Some(config),
        ),
    )
    .await;
    let Ok(Ok(mut ws)) = upgraded else {
        return;
    };
    let source = lease
        .as_ref()
        .map(|lease| lease.active.clone())
        .unwrap_or_else(|| Arc::new(AtomicBool::new(true)));
    let mut source_revoked = lease.as_ref().map(|lease| lease.revoked.subscribe());
    if !source.load(Ordering::Acquire) {
        return;
    }
    let mut events = state.bridge.events.subscribe();
    let mut snapshots = state.bridge.snapshot.clone();
    let snapshot = snapshots.borrow_and_update().clone();
    if !send(
        &mut ws,
        &ServerMessage::Snapshot {
            snapshot: &snapshot,
        },
    )
    .await
    {
        return;
    }
    let mut rate_started = tokio::time::Instant::now();
    let mut requests = 0;
    let mut pending: Option<PendingReply> = None;
    let mut idle_deadline = tokio::time::Instant::now() + IDLE_TIMEOUT;
    loop {
        tokio::select! {
            biased;
            _ = revoked.changed() => break,
            _ = async {
                match source_revoked.as_mut() {
                    Some(revoked) => { let _ = revoked.changed().await; },
                    None => std::future::pending::<()>().await,
                }
            } => break,
            _ = tokio::time::sleep_until(idle_deadline) => break,
            incoming = ws.next() => {
                let Some(Ok(message)) = incoming else { break; };
                idle_deadline = tokio::time::Instant::now() + IDLE_TIMEOUT;
                if rate_started.elapsed() >= Duration::from_secs(1) {
                    rate_started = tokio::time::Instant::now();
                    requests = 0;
                }
                requests += 1;
                if requests > REQUESTS_PER_SECOND || !state.active.load(Ordering::Acquire) || !source.load(Ordering::Acquire) { break; }
                let Message::Text(text) = message else {
                    match message {
                        Message::Ping(bytes) => {
                            if !matches!(timeout(IO_TIMEOUT, ws.send(Message::Pong(bytes))).await, Ok(Ok(()))) {
                                break;
                            }
                        }
                        Message::Pong(_) => {},
                        _ => break,
                    }
                    continue;
                };
                let Some(request) = ClientMessage::parse(&text) else { break; };
                let response = match request {
                    ClientMessage::Cancel { command_id } => {
                        if command_id.is_empty() || command_id.len() > 128 { break; }
                        if pending.as_ref().is_some_and(|pending| pending.command_id == command_id) {
                            // Dropping the reply receiver synchronously invalidates host admission.
                            pending = None;
                        }
                        send(&mut ws, &ServerMessage::Canceled { command_id: &command_id }).await
                    }
                    ClientMessage::Snapshot => { let snapshot = state.bridge.snapshot.borrow().clone(); send(&mut ws, &ServerMessage::Snapshot { snapshot: &snapshot }).await },
                    ClientMessage::Catalog => send(&mut ws, &ServerMessage::Catalog { catalog: &state.catalog.metadata }).await,
                    ClientMessage::Icon { clip_id } => match state.catalog.icons.get(&clip_id) { Some(png) => send(&mut ws, &ServerMessage::Icon { clip_id: &clip_id, png }).await, None => send(&mut ws, &ServerMessage::Error { code: "unknown_icon" }).await },
                    ClientMessage::Command { mut envelope } => {
                        // Revision, IDs, deduplication, and playback remain authoritative in Host.
                        if envelope.session_id != state.session_id || envelope.protocol_version != PROTOCOL_VERSION { break; }
                        let command_id = envelope.command_id.clone();
                        if command_id.is_empty() || command_id.len() > 128 { break; }
                        if let Some(lease) = &lease { envelope.command_id = lease.command_id(&command_id); }
                        let emergency = matches!(envelope.command, PlaybackCommand::StopAll);
                        if pending.is_some() && !emergency {
                            if !send(&mut ws, &ServerMessage::Error { code: "busy" }).await { break; }
                            continue;
                        }
                        // StopAll cancels this connection's pending normal request, too.
                        if emergency {
                            pending = None;
                        }
                        let (reply, received) = oneshot::channel();
                        let command = RemoteCommand { envelope, reply, source_id: lease.as_ref().map(|lease| lease.source_id.clone()), source: source.clone(), connection: live.clone(), session: state.active.clone() };
                        let queue = if emergency { &state.bridge.emergency } else { &state.bridge.commands };
                        if queue.try_send(command).is_err() {
                            send(&mut ws, &ServerMessage::Error { code: "busy" }).await
                        } else {
                            pending = Some(PendingReply {
                                command_id,
                                received,
                                deadline: tokio::time::Instant::now() + COMMAND_TIMEOUT,
                            });
                            true
                        }
                    }
                };
                if !response { break; }
            }
            outcome = async {
                let Some(pending) = pending.as_mut() else { return std::future::pending().await; };
                tokio::time::timeout_at(pending.deadline, &mut pending.received).await
            }, if pending.is_some() => {
                let Some(pending) = pending.take() else { break; };
                match outcome {
                    Ok(Ok(outcome)) => {
                        if !send(&mut ws, &ServerMessage::Outcome { command_id: &pending.command_id, outcome }).await {
                            break;
                        }
                    }
                    _ => break,
                }
            }
            changed = snapshots.changed() => {
                if changed.is_err() { break; }
                let snapshot = snapshots.borrow_and_update().clone();
                if snapshot.session_id != state.session_id { break; }
                if !send(&mut ws, &ServerMessage::Snapshot { snapshot: &snapshot }).await { break; }
            }
            event = events.recv() => {
                let delivered = match event {
                    Ok(event) => send(&mut ws, &ServerMessage::Event { event: &event }).await,
                    Err(broadcast::error::RecvError::Lagged(_)) => { let snapshot = state.bridge.snapshot.borrow().clone(); send(&mut ws, &ServerMessage::Snapshot { snapshot: &snapshot }).await },
                    Err(broadcast::error::RecvError::Closed) => false,
                };
                if !delivered { break; }
            }
        }
    }
    live.store(false, Ordering::Release);
    let _ = timeout(IO_TIMEOUT, ws.close(None)).await;
}

/// Bound serialization and backpressure independently of incoming request limits.
async fn send(
    ws: &mut tokio_tungstenite::WebSocketStream<TcpStream>,
    message: &ServerMessage<'_>,
) -> bool {
    let Ok(text) = serde_json::to_string(message) else {
        return false;
    };
    if text.len() > MAX_OUTPUT {
        return false;
    }
    matches!(
        timeout(IO_TIMEOUT, ws.send(Message::Text(text.into()))).await,
        Ok(Ok(()))
    )
}

/// Check address-family eligibility only; local ownership needs the separate OS/CLI check.
fn is_tailnet_ip(ip: IpAddr) -> bool {
    match ip {
        IpAddr::V4(ip) => {
            let octets = ip.octets();
            octets[0] == 100 && (64..128).contains(&octets[1])
        }
        IpAddr::V6(ip) => ip.segments()[..3] == [0xfd7a, 0x115c, 0xa1e0],
    }
}

/// Fail closed unless both the OS interface and running Tailscale self identity agree.
async fn verify_tailnet_address(ip: IpAddr) -> Result<(), TransportError> {
    if !is_tailnet_ip(ip)
        || !if_addrs::get_if_addrs()
            .map_err(TransportError::Io)?
            .iter()
            .any(|interface| interface.ip() == ip)
    {
        return Err(TransportError::TailnetUnavailable);
    }
    let status = tailscale_output(&["status", "--json"]).await?;
    let parsed: Value =
        serde_json::from_slice(&status).map_err(|_| TransportError::TailnetUnavailable)?;
    let own = parsed
        .pointer("/Self/TailscaleIPs")
        .and_then(Value::as_array)
        .is_some_and(|addresses| {
            addresses
                .iter()
                .any(|address| address.as_str().and_then(|s| s.parse::<IpAddr>().ok()) == Some(ip))
        });
    if parsed.get("BackendState").and_then(Value::as_str) != Some("Running") || !own {
        return Err(TransportError::TailnetUnavailable);
    }
    Ok(())
}

/// Choose this node's assigned tailnet IP from CLI status and OS interfaces.
/// Prefers IPv4 when both families are present so the listener form stays copyable.
fn select_local_tailnet_ip(status: &Value, interfaces: &[IpAddr]) -> Option<IpAddr> {
    if status.get("BackendState").and_then(Value::as_str) != Some("Running") {
        return None;
    }
    let mut v4 = None;
    let mut v6 = None;
    for address in status.pointer("/Self/TailscaleIPs")?.as_array()? {
        let Ok(ip) = address.as_str()?.parse::<IpAddr>() else {
            continue;
        };
        if !is_tailnet_ip(ip) || !interfaces.contains(&ip) {
            continue;
        }
        match ip {
            IpAddr::V4(_) if v4.is_none() => v4 = Some(ip),
            IpAddr::V6(_) if v6.is_none() => v6 = Some(ip),
            _ => {}
        }
    }
    v4.or(v6)
}

/// Discover this Mac's assigned tailnet address for the default Durop listener port.
/// Requires a running Tailscale node whose CGNAT or fd7a address is on a local interface.
pub async fn local_listener_address() -> Result<SocketAddr, TransportError> {
    let interfaces: Vec<IpAddr> = if_addrs::get_if_addrs()
        .map_err(TransportError::Io)?
        .into_iter()
        .map(|interface| interface.ip())
        .collect();
    let parsed: Value = serde_json::from_slice(&tailscale_output(&["status", "--json"]).await?)
        .map_err(|_| TransportError::TailnetUnavailable)?;
    let ip =
        select_local_tailnet_ip(&parsed, &interfaces).ok_or(TransportError::TailnetUnavailable)?;
    Ok(SocketAddr::new(ip, DEFAULT_LISTEN_PORT))
}

/// Resolve an installed executable without invoking shell aliases or installation prompts.
fn find_tailscale(search: &std::ffi::OsStr, fallbacks: &[PathBuf]) -> Option<PathBuf> {
    std::env::split_paths(search)
        .map(|directory| directory.join("tailscale"))
        .chain(fallbacks.iter().cloned())
        .find(|path| is_executable_file(path))
}

/// Validate a known candidate directly, without introducing a current-directory search.
fn is_executable_file(path: &Path) -> bool {
    if !path.is_file() {
        return false;
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        path.metadata()
            .is_ok_and(|metadata| metadata.permissions().mode() & 0o111 != 0)
    }
    #[cfg(not(unix))]
    {
        true
    }
}

/// Use argument arrays and force the macOS app's CLI mode without opening its UI.
fn tailscale_command(executable: &Path, arguments: &[&str]) -> tokio::process::Command {
    let mut command = tokio::process::Command::new(executable);
    // The official macOS app executable doubles as its CLI. Finder's process
    // environment lacks terminal heuristics; force CLI mode to avoid opening UI.
    // https://tailscale.com/docs/reference/tailscale-cli?tab=macos
    command.args(arguments).env("TAILSCALE_BE_CLI", "1");
    command
}

/// Read at most 256 KiB within two seconds; cancellation kills the child process.
/// Callers supply only read-only status, netcheck, or ping arguments.
async fn tailscale_output(arguments: &[&str]) -> Result<Vec<u8>, TransportError> {
    let search = std::env::var_os("PATH").unwrap_or_else(|| "/usr/bin:/bin".into());
    #[cfg(target_os = "macos")]
    let fallbacks = [
        PathBuf::from("/Applications/Tailscale.app/Contents/MacOS/Tailscale"),
        PathBuf::from("/usr/local/bin/tailscale"),
    ];
    #[cfg(not(target_os = "macos"))]
    let fallbacks = [];
    let executable =
        find_tailscale(&search, &fallbacks).ok_or(TransportError::TailnetUnavailable)?;
    let mut candidates = vec![executable];
    for fallback in &fallbacks {
        if !candidates.contains(fallback) && is_executable_file(fallback) {
            candidates.push(fallback.clone());
        }
    }
    // A Nix CLI may target an absent open-source daemon while the GUI app is enrolled.
    // Try only known executable candidates; all calls remain bounded and read-only.
    for executable in candidates {
        if let Ok(output) = tailscale_output_from(&executable, arguments).await {
            return Ok(output);
        }
    }
    Err(TransportError::TailnetUnavailable)
}

/// Bound one read-only CLI attempt and suppress subprocess error text that may contain identities.
async fn tailscale_output_from(
    executable: &Path,
    arguments: &[&str],
) -> Result<Vec<u8>, TransportError> {
    use std::process::Stdio;
    let mut child = tailscale_command(executable, arguments)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .kill_on_drop(true)
        .spawn()
        .map_err(|_| TransportError::TailnetUnavailable)?;
    let mut output = Vec::new();
    let stdout = child
        .stdout
        .take()
        .ok_or(TransportError::TailnetUnavailable)?;
    let result = timeout(Duration::from_secs(2), async {
        stdout.take(256 * 1024 + 1).read_to_end(&mut output).await?;
        let status = child.wait().await?;
        Ok::<_, std::io::Error>(status.success() && output.len() <= 256 * 1024)
    })
    .await;
    if !matches!(result, Ok(Ok(true))) {
        return Err(TransportError::TailnetUnavailable);
    }
    Ok(output)
}

/// Discover the current local Tailscale device ID through bounded read-only CLI status.
/// No device/account data beyond the identifier is returned. A stopped daemon fails closed.
pub async fn local_device_id() -> Result<String, TransportError> {
    let status: Value = serde_json::from_slice(&tailscale_output(&["status", "--json"]).await?)
        .map_err(|_| TransportError::TailnetUnavailable)?;
    if status.get("BackendState").and_then(Value::as_str) != Some("Running") {
        return Err(TransportError::TailnetUnavailable);
    }
    status
        .pointer("/Self/ID")
        .and_then(Value::as_str)
        .filter(|id| {
            !id.is_empty()
                && id.len() <= 128
                && id
                    .bytes()
                    .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_'))
        })
        .map(str::to_owned)
        .ok_or(TransportError::TailnetUnavailable)
}

/// Read-only, bounded report; never invokes enrollment or policy changes. No raw
/// status JSON is returned because it can include unrelated account/device details.
pub async fn diagnostics(peer: Option<IpAddr>) -> Result<Value, TransportError> {
    if peer.is_some_and(|ip| !is_tailnet_ip(ip)) {
        return Err(TransportError::InvalidConfiguration);
    }
    let status: Value = serde_json::from_slice(&tailscale_output(&["status", "--json"]).await?)
        .map_err(|_| TransportError::TailnetUnavailable)?;
    let netcheck = tailscale_output(&["netcheck", "--format=json"]).await;
    let ping = if let Some(ip) = peer {
        tailscale_output(&["ping", "--c=1", "--timeout=1s", &ip.to_string()])
            .await
            .ok()
            .map(|bytes| String::from_utf8_lossy(&bytes).into_owned())
    } else {
        None
    };
    Ok(
        serde_json::json!({"backendState": status.get("BackendState"), "selfAddresses": status.pointer("/Self/TailscaleIPs"), "netcheck": netcheck.ok().and_then(|bytes| serde_json::from_slice::<Value>(&bytes).ok()), "ping": ping}),
    )
}

mod access;

/// Tailscale invitation and host-only administration credential contracts.
pub mod invitations;

mod client;

#[cfg(test)]
mod tests;

pub use access::{SourceAccess, SourceAccessRegistry};

pub use client::{ControllerClient, ControllerError, ControllerUpdate};

#[cfg(all(test, unix))]
mod cli_path_tests {
    use super::*;
    use std::os::unix::fs::PermissionsExt;
    #[test]
    fn finder_fallback_and_cli_mode_do_not_need_shell_aliases() {
        struct Temp(PathBuf);
        impl Drop for Temp {
            fn drop(&mut self) {
                let _ = std::fs::remove_dir_all(&self.0);
            }
        }
        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let directory = Temp(
            std::env::temp_dir().join(format!("durop-ts-cli-{}-{unique}", std::process::id())),
        );
        std::fs::create_dir(&directory.0).unwrap();
        let search = std::env::join_paths([directory.0.join("missing")]).unwrap();
        let app = directory.0.join("Tailscale");
        std::fs::write(
            &app,
            b"this file must never execute in a path-resolution test",
        )
        .unwrap();
        std::fs::set_permissions(&app, std::fs::Permissions::from_mode(0o755)).unwrap();
        assert_eq!(
            find_tailscale(&search, std::slice::from_ref(&app)),
            Some(app.clone())
        );
        let command = tailscale_command(&app, &["status", "--json"]);
        assert!(
            command
                .as_std()
                .get_envs()
                .any(|(key, value)| key == "TAILSCALE_BE_CLI"
                    && value == Some(std::ffi::OsStr::new("1")))
        );
        let preferred = directory.0.join("tailscale");
        std::fs::copy(&app, &preferred).unwrap();
        let search = std::env::join_paths([&directory.0]).unwrap();
        assert_eq!(find_tailscale(&search, &[app]), Some(preferred));
    }
}
