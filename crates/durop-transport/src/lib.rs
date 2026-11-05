//! Explicit, revocable, tailnet-only remote sessions, outside Bevy's frame loop.
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

const MAX_CONNECTIONS: usize = 4;
const MAX_INPUT: usize = 16 * 1024;
const MAX_OUTPUT: usize = 256 * 1024;
const HANDSHAKE_TIMEOUT: Duration = Duration::from_secs(3);
const IO_TIMEOUT: Duration = Duration::from_secs(5);
const COMMAND_TIMEOUT: Duration = Duration::from_secs(2);
const IDLE_TIMEOUT: Duration = Duration::from_secs(60);
const REQUESTS_PER_SECOND: usize = 20;
const SUBPROTOCOL: &str = "durop.v1";
enum BindingGuard {
    Tailnet,
    #[cfg(any(test, feature = "test-support"))]
    Loopback,
}

#[derive(Debug)]
pub enum TransportError {
    InvalidConfiguration,
    TailnetUnavailable,
    Io(std::io::Error),
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
    pub fn bearer(&self) -> &str {
        &self.0
    }
}
fn random_hex<const N: usize>() -> Result<String, TransportError> {
    let mut bytes = [0; N];
    getrandom::fill(&mut bytes).map_err(|_| TransportError::Random)?;
    Ok(bytes.iter().map(|byte| format!("{byte:02x}")).collect())
}

/// Exact origins only. An empty list permits native clients with no Origin.
pub struct SessionConfig {
    pub address: SocketAddr,
    pub allowed_origins: Vec<String>,
}
impl SessionConfig {
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
    metadata: Value,
    icons: BTreeMap<String, Vec<u8>>,
}
/// Bound decoded PNG allocation as well as compressed wire size.
pub fn valid_icon_png(png: &[u8]) -> bool {
    png.len() >= 33
        && png.len() <= 32 * 1024
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
    pub fn new(metadata: Value, icons: BTreeMap<String, Vec<u8>>) -> Result<Self, TransportError> {
        if serde_json::to_vec(&metadata)
            .map_err(|_| TransportError::InvalidConfiguration)?
            .len()
            > 64 * 1024
            || icons.len() > 256
            || icons
                .iter()
                .any(|(id, png)| id.is_empty() || id.len() > 256 || !valid_icon_png(png))
        {
            return Err(TransportError::InvalidConfiguration);
        }
        Ok(Self { metadata, icons })
    }
}

/// Drain with try_recv on the host thread. Check is_active immediately before
/// Host::accept, skip stale requests, then reply. StopAll uses a separate bounded lane.
pub struct RemoteCommand {
    pub envelope: Envelope,
    pub reply: oneshot::Sender<CommandOutcome>,
    connection: Arc<AtomicBool>,
    session: Arc<AtomicBool>,
}
impl RemoteCommand {
    pub fn is_active(&self) -> bool {
        self.connection.load(Ordering::Acquire)
            && self.session.load(Ordering::Acquire)
            && !self.reply.is_closed()
    }
}
#[derive(Clone)]
pub struct HostBridge {
    pub commands: mpsc::Sender<RemoteCommand>,
    pub emergency: mpsc::Sender<RemoteCommand>,
    pub snapshot: watch::Receiver<HostSnapshot>,
    pub events: broadcast::Sender<HostEvent>,
}

/// Drop/revoke closes the listener and connections. Enabling again generates a new
/// credential. The host supplies its current session identity; deduplication stays there.
pub struct RemoteSession {
    pub address: SocketAddr,
    pub session_id: String,
    pub credential: SessionCredential,
    active: Arc<AtomicBool>,
    revoke: watch::Sender<bool>,
    task: JoinHandle<()>,
}
impl RemoteSession {
    pub fn is_active(&self) -> bool {
        self.active.load(Ordering::Acquire)
    }
    pub fn revoke(&self) {
        self.active.store(false, Ordering::Release);
        self.revoke.send_replace(true);
    }
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

/// Start an authenticated loopback session for cross-crate regression tests only.
#[cfg(any(test, feature = "test-support"))]
pub async fn enable_test_loopback(
    bridge: HostBridge,
    catalog: RemoteCatalog,
) -> Result<RemoteSession, TransportError> {
    let listener = TcpListener::bind("127.0.0.1:0").await?;
    start(listener, vec![], bridge, catalog, BindingGuard::Loopback)
}

fn start(
    listener: TcpListener,
    origins: Vec<String>,
    bridge: HostBridge,
    catalog: RemoteCatalog,
    guard: BindingGuard,
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
                _ = check.tick(), if matches!(guard, BindingGuard::Tailnet) => { if verify_tailnet_address(address.ip()).await.is_err() { break; } }
                accepted = listener.accept() => {
                    let Ok((tcp, _peer)) = accepted else { break; };
                    let Ok(permit) = slots.clone().try_acquire_owned() else { drop(tcp); continue; };
                    let state = state.clone();
                    let revoked = revoked.clone();
                    connections.spawn(async move { let _permit = permit; connection(tcp, state, revoked).await; });
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

struct ConnectionState {
    expected_authorization: String,
    origins: Vec<String>,
    active: Arc<AtomicBool>,
    bridge: HostBridge,
    catalog: RemoteCatalog,
    session_id: String,
}
struct ConnectionLifetime(Arc<AtomicBool>);
impl Drop for ConnectionLifetime {
    fn drop(&mut self) {
        self.0.store(false, Ordering::Release);
    }
}
struct PendingReply {
    command_id: String,
    received: oneshot::Receiver<CommandOutcome>,
    deadline: tokio::time::Instant,
}

#[derive(Deserialize)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
enum ClientMessage {
    Command { envelope: Envelope },
    Snapshot,
    Catalog,
    Icon { clip_id: String },
}
fn parse_client(text: &str) -> Option<ClientMessage> {
    let value: Value = serde_json::from_str(text).ok()?;
    let object = value.as_object()?;
    // Serde's internally tagged unit variants otherwise accept extra fields.
    let allowed: &[&str] = match object.get("type")?.as_str()? {
        "snapshot" | "catalog" => &["type"],
        "icon" => &["type", "clip_id"],
        "command" => &["type", "envelope"],
        _ => return None,
    };
    if object.len() != allowed.len() || object.keys().any(|key| !allowed.contains(&key.as_str())) {
        return None;
    }
    serde_json::from_value(value).ok()
}
#[derive(Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum ServerMessage<'a> {
    Snapshot {
        snapshot: &'a HostSnapshot,
    },
    Catalog {
        catalog: &'a Value,
    },
    Icon {
        clip_id: &'a str,
        png: &'a [u8],
    },
    Outcome {
        command_id: &'a str,
        outcome: CommandOutcome,
    },
    Event {
        event: &'a HostEvent,
    },
    Error {
        code: &'a str,
    },
}

// tungstenite's public Callback signature fixes this response type.
#[allow(clippy::result_large_err)]
fn handshake(
    request: &Request,
    mut response: Response,
    state: &ConnectionState,
) -> Result<Response, ErrorResponse> {
    let authorized = request.headers().get_all("authorization").iter().count() == 1
        && request
            .headers()
            .get("authorization")
            .map(|value| {
                bool::from(
                    value
                        .as_bytes()
                        .ct_eq(state.expected_authorization.as_bytes()),
                )
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
    #[allow(clippy::result_large_err)]
    let upgraded = timeout(
        HANDSHAKE_TIMEOUT,
        accept_hdr_async_with_config(
            tcp,
            |request: &Request, response| handshake(request, response, &state),
            Some(config),
        ),
    )
    .await;
    let Ok(Ok(mut ws)) = upgraded else {
        return;
    };
    let mut events = state.bridge.events.subscribe();
    let snapshot = state.bridge.snapshot.borrow().clone();
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
            _ = tokio::time::sleep_until(idle_deadline) => break,
            incoming = ws.next() => {
                let Some(Ok(message)) = incoming else { break; };
                idle_deadline = tokio::time::Instant::now() + IDLE_TIMEOUT;
                if rate_started.elapsed() >= Duration::from_secs(1) { rate_started = tokio::time::Instant::now(); requests = 0; }
                requests += 1;
                if requests > REQUESTS_PER_SECOND || !state.active.load(Ordering::Acquire) { break; }
                let Message::Text(text) = message else {
                    match message { Message::Ping(bytes) => { if !matches!(timeout(IO_TIMEOUT, ws.send(Message::Pong(bytes))).await, Ok(Ok(()))) { break; } }, Message::Pong(_) => {}, _ => break }
                    continue;
                };
                let Some(request) = parse_client(&text) else { break; };
                let response = match request {
                    ClientMessage::Snapshot => { let snapshot = state.bridge.snapshot.borrow().clone(); send(&mut ws, &ServerMessage::Snapshot { snapshot: &snapshot }).await },
                    ClientMessage::Catalog => send(&mut ws, &ServerMessage::Catalog { catalog: &state.catalog.metadata }).await,
                    ClientMessage::Icon { clip_id } => match state.catalog.icons.get(&clip_id) { Some(png) => send(&mut ws, &ServerMessage::Icon { clip_id: &clip_id, png }).await, None => send(&mut ws, &ServerMessage::Error { code: "unknown_icon" }).await },
                    ClientMessage::Command { envelope } => {
                        // Revision, IDs, deduplication, and playback remain authoritative in Host.
                        if envelope.session_id != state.session_id || envelope.protocol_version != PROTOCOL_VERSION { break; }
                        let command_id = envelope.command_id.clone();
                        let emergency = matches!(envelope.command, PlaybackCommand::StopAll);
                        if pending.is_some() && !emergency {
                            if !send(&mut ws, &ServerMessage::Error { code: "busy" }).await { break; }
                            continue;
                        }
                        // StopAll cancels this connection's pending normal request, too.
                        if emergency { pending = None; }
                        let (reply, received) = oneshot::channel();
                        let command = RemoteCommand { envelope, reply, connection: live.clone(), session: state.active.clone() };
                        let queue = if emergency { &state.bridge.emergency } else { &state.bridge.commands };
                        if queue.try_send(command).is_err() { send(&mut ws, &ServerMessage::Error { code: "busy" }).await }
                        else { pending = Some(PendingReply { command_id, received, deadline: tokio::time::Instant::now() + COMMAND_TIMEOUT }); true }
                    }
                };
                if !response { break; }
            }
            outcome = async {
                let Some(pending) = pending.as_mut() else { return std::future::pending().await; };
                tokio::time::timeout_at(pending.deadline, &mut pending.received).await
            }, if pending.is_some() => {
                let Some(pending) = pending.take() else { break; };
                match outcome { Ok(Ok(outcome)) => { if !send(&mut ws, &ServerMessage::Outcome { command_id: &pending.command_id, outcome }).await { break; } }, _ => break }
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
fn is_tailnet_ip(ip: IpAddr) -> bool {
    match ip {
        IpAddr::V4(ip) => {
            let octets = ip.octets();
            octets[0] == 100 && (64..128).contains(&octets[1])
        }
        IpAddr::V6(ip) => ip.segments()[..3] == [0xfd7a, 0x115c, 0xa1e0],
    }
}
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
fn find_tailscale(search: &std::ffi::OsStr, fallbacks: &[PathBuf]) -> Option<PathBuf> {
    std::env::split_paths(search)
        .map(|directory| directory.join("tailscale"))
        .chain(fallbacks.iter().cloned())
        .find(|path| {
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
        })
}
fn tailscale_command(executable: &Path, arguments: &[&str]) -> tokio::process::Command {
    let mut command = tokio::process::Command::new(executable);
    // The official macOS app executable doubles as its CLI. Finder's process
    // environment lacks terminal heuristics; force CLI mode to avoid opening UI.
    // https://tailscale.com/docs/reference/tailscale-cli?tab=macos
    command.args(arguments).env("TAILSCALE_BE_CLI", "1");
    command
}
async fn tailscale_output(arguments: &[&str]) -> Result<Vec<u8>, TransportError> {
    use std::process::Stdio;
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
    let mut child = tailscale_command(&executable, arguments)
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

mod client;
#[cfg(test)]
mod tests;
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
