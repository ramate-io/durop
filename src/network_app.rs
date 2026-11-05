//! Bounded application orchestration for native and authenticated Tailscale controllers.
//!
//! Native admission stays in process. Remote admission is available only after a
//! connecting backend consumes the authenticated snapshot and becomes connected.
//! Host policy, cancellation guards, and StopAll epochs remain in the shared ingress.
use crate::{
    app::Runtime,
    catalog::Catalog,
    commands::{CommandOutcome, Envelope, HostEvent, HostSnapshot, PlaybackCommand, Rejection},
    host::{CommandInbox, QueueError, command_channel},
    transport::{
        self, ControllerClient, ControllerUpdate, HostBridge, RemoteCatalog, RemoteCommand,
        RemoteSession, SessionConfig, SessionCredential,
    },
};
use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet, VecDeque},
    io::Read,
    net::SocketAddr,
    sync::{Arc, Mutex},
    time::Duration,
};
use tokio::sync::{broadcast, mpsc, watch};

mod controller;

use controller::{
    Connected, Connecting, ControllerAdmissionError, ControllerBackend, ControllerMailbox,
    ControllerRequest, NativeControllerBackend, TailscaleControllerBackend,
};

/// Explicit initial connection/listener choices supplied by application configuration.
#[derive(Resource, Default)]
pub struct NetworkConfig {
    /// Selected tailnet IP:port or exact ws://IP:port/control target; never a credential.
    pub address: Option<String>,
    /// Start remote hosting only when the user explicitly requested a listener.
    pub enable_host: bool,
}

/// UI intentions consumed once; disconnected button presses are discarded.
#[derive(Clone)]
pub enum NetworkAction {
    /// Verify this Mac's selected tailnet address and enable a new credential.
    EnableHost,
    /// Revoke the current listener and all controller access.
    DisableHost,
    /// Consume the protected input credential for one explicit connection attempt.
    Connect,
    /// Drop the remote controller and all pending connection work.
    Disconnect,
    /// Admit one online playback command through the Tailscale backend.
    RemoteCommand(PlaybackCommand),
}

/// Controller-safe clip metadata, with no native asset paths or playback objects.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RemoteClip {
    /// Canonical catalog clip identity used by the host policy path.
    pub clip_id: String,
    /// Bounded display title from the validated pack.
    pub title: String,
    /// Bounded search tags for the panel.
    pub tags: Vec<String>,
    /// Relative catalog category for navigation, never an executable filesystem path.
    pub category: String,
    /// Validated panel category: audio, video, or gif.
    pub media_type: String,
}

/// Bounded metadata projection for browsing one pinned authenticated catalog.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RemoteCatalogView {
    /// Public pack identity from its manifest.
    pub pack_id: String,
    /// Public manifest title.
    pub title: String,
    /// Pinned revision checked by the same authoritative host as local requests.
    pub revision: String,
    /// At most 256 bounded clip projections; icons use the authenticated allowlist.
    pub clips: Vec<RemoteClip>,
}

impl RemoteCatalogView {
    fn validate(&self) -> bool {
        let mut ids = BTreeSet::new();
        self.clips.len() <= 256
            && self.title.len() <= 256
            && self.pack_id.len() <= 256
            && self.revision.len() <= 128
            && self.clips.iter().all(|clip| {
                !clip.clip_id.is_empty()
                    && clip.clip_id.len() <= 256
                    && ids.insert(&clip.clip_id)
                    && clip.title.len() <= 256
                    && clip.category.len() <= 128
                    && clip.tags.len() <= 32
                    && clip.tags.iter().all(|tag| tag.len() <= 64)
                    && ["audio", "video", "gif"].contains(&clip.media_type.as_str())
            })
    }
}

/// Background results carry a generation so an ended session cannot update a new one.
enum Update {
    Remote(RemoteCommand),
    HostEnabled(u64, RemoteSession),
    HostEnded(u64),
    HostFailed(u64, String),
    Connected(u64, HostSnapshot, RemoteCatalogView),
    Snapshot(u64, HostSnapshot),
    Icon(u64, String, Vec<u8>),
    Outcome(u64, CommandOutcome),
    Disconnected(u64, String),
}

/// Keep the reply ticket and admission guard attached to the same pending request.
type GuardedRequest = Arc<Mutex<Option<RemoteCommand>>>;

/// UI-facing state plus the sole Tokio runtime and private controller lifecycle owners.
#[derive(Resource)]
pub struct NetworkState {
    /// Selected explicit host address, retained as editable UI input.
    pub address: String,
    /// Explicit pairing input only. Taken and cleared when Connect is pressed.
    pub secret: String,
    /// Human-readable connection/admission status, not proof of media presentation.
    pub status: String,
    /// Latest authoritative remote snapshot, absent while disconnected.
    pub snapshot: Option<HostSnapshot>,
    /// Validated remote metadata, cleared whenever the connection ends.
    pub remote_catalog: Option<RemoteCatalogView>,
    /// Bounded PNG bytes keyed only by known remote clip identities.
    pub remote_icons: BTreeMap<String, Vec<u8>>,
    /// UI connection view; actual command permission resides in the connected typestate.
    pub connected: bool,
    /// Whether an enabled local listener currently grants controller access.
    pub hosting: bool,
    /// Host-only pairing display value, cleared on revocation and never persisted.
    pub pairing_secret: Option<String>,
    /// One-shot UI intentions; each update processes at most 64 and discards excess.
    pub actions: VecDeque<NetworkAction>,
    runtime: tokio::runtime::Runtime,
    local: NativeControllerBackend,
    inbox: CommandInbox,
    remote_job: tokio::task::JoinHandle<()>,
    bridge: HostBridge,
    snapshots: watch::Sender<HostSnapshot>,
    updates: mpsc::Receiver<Update>,
    update_tx: mpsc::Sender<Update>,
    wake: Arc<dyn Fn() + Send + Sync>,
    session: Option<Arc<RemoteSession>>,
    host_job: Option<tokio::task::JoinHandle<()>>,
    host_monitor: Option<tokio::task::JoinHandle<()>>,
    client_job: Option<tokio::task::JoinHandle<()>>,
    connecting: Option<TailscaleControllerBackend<Connecting>>,
    controller: Option<TailscaleControllerBackend<Connected>>,
    host_generation: u64,
    client_generation: u64,
    ticket: u64,
    pending: BTreeMap<u64, GuardedRequest>,
}

impl NetworkState {
    #[cfg(test)]
    /// Construct the same bounded orchestration without a native window event loop.
    pub(crate) fn for_ui_test(snapshot: HostSnapshot) -> Self {
        NetworkState::new(snapshot, &NetworkConfig::default(), Arc::new(|| {}))
    }

    #[cfg(test)]
    /// Pump the authoritative queue for deterministic UI command-routing fixtures.
    pub(crate) fn accept_for_ui_test(&mut self, host: &mut crate::host::Host) {
        let session = host.session_id().to_owned();
        self.accept_pending(host, &session);
    }

    /// Admit through the native ControllerBackend; success means queued, not started.
    pub fn queue_local(&mut self, envelope: Envelope) -> Result<(), QueueError> {
        self.local
            .submit(envelope.clone())
            .map_err(|error| match error {
                ControllerAdmissionError::Busy => QueueError::Busy,
                ControllerAdmissionError::Unavailable | ControllerAdmissionError::StaleSession => {
                    QueueError::Closed
                }
            })?;
        crate::diagnostics::submitted(&envelope, crate::diagnostics::Origin::Local);
        Ok(())
    }

    /// Publish state before events and revoke a listener whose host session changed.
    pub fn publish_local(&mut self, snapshot: HostSnapshot, events: &[HostEvent]) {
        if self
            .session
            .as_ref()
            .is_some_and(|session| session.session_id != snapshot.session_id)
        {
            self.disable_host();
        }
        self.snapshots.send_replace(snapshot);
        if self.hosting {
            for event in events {
                let _ = self.bridge.events.send(event.clone());
            }
        }
    }

    /// Revoke permission synchronously before canceling background listener ownership.
    fn disable_host(&mut self) {
        self.host_generation = self.host_generation.wrapping_add(1);
        if let Some(job) = self.host_job.take() {
            job.abort();
        }
        if let Some(job) = self.host_monitor.take() {
            job.abort();
        }
        if let Some(session) = self.session.take() {
            session.revoke();
        }
        self.hosting = false;
        self.pairing_secret = None;
    }

    /// Consume ready/connecting states and cancel the socket before clearing the UI view.
    fn disconnect(&mut self) {
        self.client_generation = self.client_generation.wrapping_add(1);
        self.connecting = None;
        if let Some(controller) = self.controller.take() {
            let _ = controller.disconnect();
        }
        if let Some(job) = self.client_job.take() {
            job.abort();
        }
        self.connected = false;
        self.snapshot = None;
        self.remote_catalog = None;
        self.remote_icons.clear();
        self.actions
            .retain(|action| !matches!(action, NetworkAction::RemoteCommand(_)));
    }

    /// Build the trusted metadata projection off the UI thread, then verify and bind.
    fn enable_host(&mut self, catalog: Arc<Catalog>) {
        self.disable_host();
        let Ok(address) = NetworkConfig::parse_address(&self.address) else {
            self.status = "Choose this Mac's Tailscale IP and port".into();
            return;
        };
        let generation = self.host_generation;
        let bridge = self.bridge.clone();
        let updates = self.update_tx.clone();
        let wake = self.wake.clone();
        self.status = "Verifying local Tailscale address…".into();
        self.host_job = Some(self.runtime.spawn(async move {
            let result =
                tokio::task::spawn_blocking(move || RemoteCatalogView::project(&catalog)).await;
            let result = match result {
                Ok(Ok(catalog)) => transport::enable(
                    SessionConfig {
                        address,
                        allowed_origins: vec![],
                    },
                    bridge,
                    catalog,
                )
                .await
                .map_err(|error| error.to_string()),
                Ok(Err(error)) => Err(error),
                Err(_) => Err("Catalog projection failed".into()),
            };
            let update = match result {
                Ok(session) => Update::HostEnabled(generation, session),
                Err(error) => Update::HostFailed(generation, error),
            };
            if updates.send(update).await.is_ok() {
                wake();
            }
        }));
    }

    /// Create a non-admitting connecting typestate for one explicit protected attempt.
    fn connect(&mut self) {
        self.disconnect();
        let secret = std::mem::take(&mut self.secret);
        let (Ok(address), Ok(credential)) = (
            NetworkConfig::parse_address(&self.address),
            SessionCredential::from_secret(secret),
        ) else {
            self.status = "Enter a Tailscale IP:port and pairing credential".into();
            return;
        };
        let generation = self.client_generation;
        let namespace = bevy::asset::uuid::Uuid::new_v4().to_string();
        let (connecting, mailbox) = TailscaleControllerBackend::new().begin_connect(namespace);
        self.connecting = Some(connecting);
        self.status = "Connecting to selected host…".into();
        let updates = self.update_tx.clone();
        let wake = self.wake.clone();
        self.client_job = Some(self.runtime.spawn(controller(
            generation, address, credential, mailbox, updates, wake,
        )));
    }

    /// Preserve the transport lease and reply ticket through the native shared ingress.
    fn admit_remote(&mut self, request: RemoteCommand) {
        if !request.is_active() {
            return;
        }
        self.ticket = self
            .ticket
            .checked_add(1)
            .expect("network ticket exhausted");
        let ticket = self.ticket;
        let envelope = request.envelope.clone();
        let request = Arc::new(Mutex::new(Some(request)));
        let active = request.clone();
        let guard = Arc::new(move || {
            active
                .lock()
                .unwrap()
                .as_ref()
                .is_some_and(RemoteCommand::is_active)
        });
        match self.local.submit_guarded(envelope.clone(), ticket, guard) {
            Ok(()) => {
                crate::diagnostics::submitted(&envelope, crate::diagnostics::Origin::Remote);
                self.pending.insert(ticket, request);
            }
            Err(_) => {
                if let Some(request) = request.lock().unwrap().take() {
                    let _ = request.reply.send(CommandOutcome::Rejected {
                        reason: Rejection::Busy,
                    });
                }
            }
        }
    }
}

impl NetworkState {
    /// Pump canonical host policy and route outcomes back to their original controllers.
    fn accept_pending(
        &mut self,
        host: &mut crate::host::Host,
        session: &str,
    ) -> Vec<CommandOutcome> {
        let mut local = Vec::new();
        for receipt in self.inbox.drain(host) {
            crate::diagnostics::acknowledged(session, &receipt.command_id, &receipt.outcome);
            if let Some(ticket) = receipt.ticket {
                if let Some(request) = self
                    .pending
                    .remove(&ticket)
                    .and_then(|request| request.lock().unwrap().take())
                {
                    let _ = request.reply.send(receipt.outcome);
                }
            } else if let CommandOutcome::Rejected { reason } = receipt.outcome {
                local.push(CommandOutcome::Rejected { reason });
            }
        }
        self.pending.retain(|_, request| {
            request
                .lock()
                .unwrap()
                .as_ref()
                .is_some_and(RemoteCommand::is_active)
        });
        local
    }
}

impl Drop for NetworkState {
    fn drop(&mut self) {
        self.remote_job.abort();
        self.disable_host();
        self.disconnect();
    }
}

/// Install bounded controller orchestration between UI intentions and media effects.
pub struct NetworkPlugin;
impl Plugin for NetworkPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<NetworkConfig>()
            .add_systems(Startup, setup)
            .add_systems(
                Update,
                dispatch
                    .after(crate::app::actions)
                    .before(crate::app::media_bridge),
            );
    }
}

/// Bind UI wake notifications after Bevy has installed its native event-loop proxy.
fn setup(
    mut commands: Commands,
    runtime: Res<Runtime>,
    config: Res<NetworkConfig>,
    proxy: Res<bevy::winit::EventLoopProxyWrapper>,
) {
    let proxy = (**proxy).clone();
    let wake: Arc<dyn Fn() + Send + Sync> = Arc::new(move || {
        let _ = proxy.send_event(bevy::winit::WinitUserEvent::WakeUp);
    });
    commands.insert_resource(NetworkState::new(runtime.host.snapshot(), &config, wake));
}

impl NetworkState {
    /// Own one runtime and all bounded channels for both controller implementations.
    fn new(
        initial_snapshot: HostSnapshot,
        config: &NetworkConfig,
        wake: Arc<dyn Fn() + Send + Sync>,
    ) -> NetworkState {
        let host_wake = wake.clone();
        let (local, inbox) = command_channel(move || host_wake());
        let (normal, mut remote) = mpsc::channel(128);
        let (priority, mut emergency) = mpsc::channel(8);
        let (snapshots, snapshot) = watch::channel(initial_snapshot);
        let (events, _) = broadcast::channel(256);
        let (update_tx, updates) = mpsc::channel(32);
        let async_runtime = tokio::runtime::Builder::new_multi_thread()
            .worker_threads(2)
            .enable_all()
            .build()
            .expect("Tokio runtime");
        let forwarded = update_tx.clone();
        let forward_wake = wake.clone();
        let remote_job = async_runtime.spawn(async move {
        loop {
            tokio::select! {
                biased;
                request = emergency.recv() => {
                    let Some(request) = request else { break; };
                    // Admit already queued normal work before the stop so the shared
                    // ingress epoch barrier supersedes it, rather than replaying later.
                    while let Ok(normal) = remote.try_recv() { if !notify(&forwarded, &forward_wake, Update::Remote(normal)).await { return; } }
                    if !notify(&forwarded, &forward_wake, Update::Remote(request)).await { break; }
                }
                request = remote.recv() => { let Some(request) = request else { break; }; if !notify(&forwarded, &forward_wake, Update::Remote(request)).await { break; } }
            }
        }
    });
        NetworkState {
            address: config.address.clone().unwrap_or_default(),
            secret: String::new(),
            status: "Remote access disabled".into(),
            snapshot: None,
            remote_catalog: None,
            remote_icons: BTreeMap::new(),
            connected: false,
            hosting: false,
            pairing_secret: None,
            actions: if config.enable_host {
                VecDeque::from([NetworkAction::EnableHost])
            } else {
                VecDeque::new()
            },
            runtime: async_runtime,
            local: NativeControllerBackend::new(local),
            inbox,
            remote_job,
            bridge: HostBridge {
                commands: normal,
                emergency: priority,
                snapshot,
                events,
            },
            snapshots,
            updates,
            update_tx,
            wake,
            session: None,
            host_job: None,
            host_monitor: None,
            client_job: None,
            connecting: None,
            controller: None,
            host_generation: 0,
            client_generation: 0,
            ticket: 0,
            pending: BTreeMap::new(),
        }
    }
}

/// Admit bounded UI intentions, apply generation-checked results, then pump host policy.
fn dispatch(mut network: ResMut<NetworkState>, mut runtime: ResMut<Runtime>) {
    if !runtime.enabled
        || network.session.as_ref().is_some_and(|session| {
            !session.is_active() || session.session_id != runtime.snapshot.session_id
        })
    {
        if network.hosting {
            network.status = "Remote hosting ended".into();
        }
        network.disable_host();
    }
    for _ in 0..64 {
        let Some(action) = network.actions.pop_front() else {
            break;
        };
        match action {
            NetworkAction::EnableHost => {
                if runtime.enabled {
                    network.snapshots.send_replace(runtime.host.snapshot());
                    network.enable_host(runtime.catalog.clone());
                } else {
                    network.status = "Start the local host first".into();
                }
            }
            NetworkAction::DisableHost => {
                network.disable_host();
                network.status = "Remote access disabled".into();
            }
            NetworkAction::Connect => network.connect(),
            NetworkAction::Disconnect => {
                network.disconnect();
                network.status = "Controller disconnected".into();
            }
            NetworkAction::RemoteCommand(command) => {
                if !network.connected {
                    network.status = "Remote unavailable; command not sent".into();
                    continue;
                }
                if network
                    .controller
                    .as_mut()
                    .is_none_or(|controller| controller.submit_command(command).is_err())
                {
                    network.status = "Remote busy or disconnected; command not queued".into();
                }
            }
        }
    }
    if !network.actions.is_empty() {
        network.actions.clear();
        network.status = "Too many actions; excess requests discarded".into();
    }
    for _ in 0..32 {
        let Ok(update) = network.updates.try_recv() else {
            break;
        };
        match update {
            Update::Remote(request) => network.admit_remote(request),
            Update::HostEnabled(generation, session)
                if generation == network.host_generation
                    && runtime.enabled
                    && session.session_id == runtime.snapshot.session_id =>
            {
                network.pairing_secret = Some(session.credential.bearer().into());
                let session = Arc::new(session);
                let monitored = session.clone();
                let updates = network.update_tx.clone();
                let wake = network.wake.clone();
                network.host_monitor = Some(network.runtime.spawn(async move {
                    while monitored.is_active() {
                        tokio::time::sleep(Duration::from_millis(500)).await;
                    }
                    let _ = notify(&updates, &wake, Update::HostEnded(generation)).await;
                }));
                network.session = Some(session);
                network.hosting = true;
                network.status =
                    "Remote hosting enabled; share pairing credential privately".into();
            }
            Update::HostFailed(generation, error) if generation == network.host_generation => {
                network.status = format!("Remote host unavailable: {error}")
            }
            Update::HostEnded(generation) if generation == network.host_generation => {
                network.disable_host();
                network.status = "Remote hosting ended; Tailscale address unavailable".into();
            }
            Update::Connected(generation, snapshot, catalog)
                if generation == network.client_generation =>
            {
                let Some(connecting) = network.connecting.take() else {
                    continue;
                };
                network.controller = Some(connecting.connected(&snapshot));
                network.snapshot = Some(snapshot);
                network.remote_catalog = Some(catalog);
                network.connected = true;
                network.status = "Remote host connected".into();
            }
            Update::Snapshot(generation, snapshot) if generation == network.client_generation => {
                network.snapshot = Some(snapshot)
            }
            Update::Icon(generation, id, png) if generation == network.client_generation => {
                network.remote_icons.insert(id, png);
            }
            Update::Outcome(generation, outcome) if generation == network.client_generation => {
                network.status = match outcome {
                    CommandOutcome::Accepted { .. } => {
                        "Remote command accepted; awaiting playback status".into()
                    }
                    CommandOutcome::Rejected { reason } => {
                        format!("Remote command rejected: {reason:?}")
                    }
                }
            }
            Update::Disconnected(generation, error) if generation == network.client_generation => {
                network.disconnect();
                network.status = format!("Remote disconnected: {error}");
            }
            _ => {}
        }
    }
    let session_id = runtime.snapshot.session_id.clone();
    for outcome in network.accept_pending(&mut runtime.host, &session_id) {
        if let CommandOutcome::Rejected { reason } = outcome {
            runtime.status = format!("Command rejected: {reason:?}");
        }
    }
}

impl NetworkConfig {
    /// Accept only an explicit socket address or the protocol's exact WebSocket target.
    fn parse_address(address: &str) -> Result<SocketAddr, ()> {
        let address = if let Some(url) = address.strip_prefix("ws://") {
            url.strip_suffix("/control").ok_or(())?
        } else {
            address
        };
        address.parse().map_err(|_| ())
    }
}

impl RemoteCatalogView {
    /// Read trusted local icons off the UI thread and build a path-free wire projection.
    fn project(catalog: &Catalog) -> Result<RemoteCatalog, String> {
        let mut icons = BTreeMap::new();
        let mut clips = Vec::new();
        for clip in &catalog.clips {
            let media_type = match clip
                .asset_path
                .extension()
                .and_then(|value| value.to_str())
                .map(str::to_ascii_lowercase)
                .as_deref()
            {
                Some("gif") => "gif",
                Some("wav" | "mp3" | "m4a" | "aac" | "aiff" | "flac") => "audio",
                _ => "video",
            };
            clips.push(RemoteClip {
                clip_id: clip.identity.clone(),
                title: clip.metadata.title.clone(),
                tags: clip.metadata.tags.clone(),
                category: clip.category.to_string_lossy().into_owned(),
                media_type: media_type.into(),
            });
            let mut png = Vec::new();
            std::fs::File::open(&clip.icon_path)
                .map_err(|_| "Catalog icon unavailable")?
                .take(32 * 1024 + 1)
                .read_to_end(&mut png)
                .map_err(|_| "Catalog icon unavailable")?;
            icons.insert(clip.identity.clone(), png);
        }
        let view = RemoteCatalogView {
            pack_id: catalog.manifest.pack_id.clone(),
            title: catalog.manifest.title.clone(),
            revision: catalog.revision.clone(),
            clips,
        };
        if !view.validate() {
            return Err("Catalog exceeds remote metadata bounds".into());
        }
        RemoteCatalog::new(
            serde_json::to_value(view).map_err(|_| "Catalog serialization failed")?,
            icons,
        )
        .map_err(|error| error.to_string())
    }
}

/// Publish one bounded background result and explicitly wake an otherwise idle UI.
async fn notify(
    updates: &mpsc::Sender<Update>,
    wake: &Arc<dyn Fn() + Send + Sync>,
    update: Update,
) -> bool {
    if updates.send(update).await.is_err() {
        return false;
    }
    wake();
    true
}

/// Own the remote socket task and report its terminal state to the matching UI generation.
async fn controller(
    generation: u64,
    address: SocketAddr,
    credential: SessionCredential,
    mut mailbox: ControllerMailbox,
    updates: mpsc::Sender<Update>,
    wake: Arc<dyn Fn() + Send + Sync>,
) {
    let result = controller_session(
        generation,
        address,
        credential,
        &mut mailbox.normal,
        &mut mailbox.priority,
        &updates,
        &wake,
    )
    .await;
    if let Err(error) = result {
        let _ = notify(&updates, &wake, Update::Disconnected(generation, error)).await;
    }
}

/// Authenticate the selected target before any controller backend becomes ready.
async fn controller_session(
    generation: u64,
    address: SocketAddr,
    credential: SessionCredential,
    normal: &mut mpsc::Receiver<ControllerRequest>,
    priority: &mut mpsc::Receiver<ControllerRequest>,
    updates: &mpsc::Sender<Update>,
    wake: &Arc<dyn Fn() + Send + Sync>,
) -> Result<(), String> {
    let client = ControllerClient::connect(address, credential)
        .await
        .map_err(|error| error.to_string())?;
    controller_connected(generation, client, normal, priority, updates, wake).await
}

/// Service one ready socket with StopAll precedence, bounded browsing, and no retries.
async fn controller_connected(
    generation: u64,
    mut client: ControllerClient,
    normal: &mut mpsc::Receiver<ControllerRequest>,
    priority: &mut mpsc::Receiver<ControllerRequest>,
    updates: &mpsc::Sender<Update>,
    wake: &Arc<dyn Fn() + Send + Sync>,
) -> Result<(), String> {
    client.observe_acknowledgments(crate::diagnostics::controller_ack);
    let catalog: RemoteCatalogView =
        serde_json::from_value(client.catalog().await.map_err(|error| error.to_string())?)
            .map_err(|_| "Invalid host catalog")?;
    if !catalog.validate() || catalog.revision != client.current_snapshot().catalog_revision {
        return Err("Invalid host catalog revision or bounds".into());
    }
    if !notify(
        updates,
        wake,
        Update::Connected(
            generation,
            client.current_snapshot().clone(),
            catalog.clone(),
        ),
    )
    .await
    {
        return Ok(());
    }
    let mut icons = catalog.clips.into_iter();
    let mut tick = tokio::time::interval(Duration::from_millis(100));
    tick.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
    let mut refresh = false;
    let mut cutoff = 0;
    loop {
        let command = tokio::select! {
            biased;
            command = priority.recv() => command,
            command = normal.recv() => command,
            update = client.next_update() => {
                match update.map_err(|error| error.to_string())? { ControllerUpdate::Event(_) => refresh = true, ControllerUpdate::Snapshot(snapshot) => { if !notify(updates, wake, Update::Snapshot(generation, snapshot)).await { break; } } }
                continue;
            }
            _ = tick.tick() => {
                if refresh {
                    let snapshot = client.refresh_snapshot().await.map_err(|error| error.to_string())?; refresh = false;
                    if !notify(updates, wake, Update::Snapshot(generation, snapshot)).await { break; }
                } else if let Some(clip) = icons.next() {
                    match client.icon(&clip.clip_id).await { Ok(png) => { if !notify(updates, wake, Update::Icon(generation, clip.clip_id, png)).await { break; } }, Err(transport::ControllerError::UnknownIcon) => {}, Err(error) => return Err(error.to_string()) }
                }
                continue;
            }
        };
        let Some(request) = command else {
            break;
        };
        if !request.is_active() {
            continue;
        }
        if request.epoch < cutoff {
            if !notify(
                updates,
                wake,
                Update::Outcome(
                    generation,
                    CommandOutcome::Rejected {
                        reason: Rejection::Superseded,
                    },
                ),
            )
            .await
            {
                break;
            }
            continue;
        }
        let stop = matches!(request.envelope.command, PlaybackCommand::StopAll);
        let outcome = client.submit_envelope(request.envelope).await;
        match outcome {
            Ok(outcome) => {
                if stop && matches!(outcome, CommandOutcome::Accepted { .. }) {
                    cutoff = cutoff.max(request.epoch);
                }
                refresh = true;
                if !notify(updates, wake, Update::Outcome(generation, outcome)).await {
                    break;
                }
            }
            Err(transport::ControllerError::Busy) => {
                if !notify(
                    updates,
                    wake,
                    Update::Outcome(
                        generation,
                        CommandOutcome::Rejected {
                            reason: Rejection::Busy,
                        },
                    ),
                )
                .await
                {
                    break;
                }
            }
            Err(error) => return Err(error.to_string()),
        }
    }
    client.disconnect().await;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    #[test]
    #[ignore = "explicit local timing capture; set DUROP_ACCEPTANCE_OUTPUT to a new private JSONL path"]
    fn capture_same_mac_policy_and_loopback_acknowledgments() {
        let path =
            std::env::var_os("DUROP_ACCEPTANCE_OUTPUT").expect("set DUROP_ACCEPTANCE_OUTPUT");
        let samples = std::env::var("DUROP_ACCEPTANCE_SAMPLES")
            .ok()
            .and_then(|value| value.parse::<usize>().ok())
            .unwrap_or(30);
        assert!((1..=300).contains(&samples));
        let guard = crate::diagnostics::start(std::path::Path::new(&path)).unwrap();
        let catalog = Arc::new(
            crate::catalog::load_local(
                &std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                    .join("sample-packs/podcast-pack"),
            )
            .unwrap(),
        );
        let mut host = crate::host::Host::new("local-acceptance-probe".into(), catalog.clone());
        let mut network =
            NetworkState::new(host.snapshot(), &NetworkConfig::default(), Arc::new(|| {}));
        for sample in 0..samples {
            network
                .queue_local(Envelope {
                    protocol_version: crate::commands::PROTOCOL_VERSION,
                    session_id: "local-acceptance-probe".into(),
                    command_id: format!("local-{sample}"),
                    catalog_revision: catalog.revision.clone(),
                    command: PlaybackCommand::StopAll,
                })
                .unwrap();
            assert!(
                network
                    .accept_pending(&mut host, "local-acceptance-probe")
                    .is_empty()
            );
        }
        network.publish_local(host.snapshot(), &[]);
        let session = network
            .runtime
            .block_on(transport::enable_test_loopback(
                network.bridge.clone(),
                RemoteCatalogView::project(&catalog).unwrap(),
            ))
            .unwrap();
        let address = session.address;
        let secret = SessionCredential::from_secret(session.credential.bearer().into()).unwrap();
        let revision = catalog.revision.clone();
        let (done, completed) = std::sync::mpsc::sync_channel(1);
        let task = network.runtime.spawn(async move {
            let mut client = ControllerClient::connect_loopback(address, secret)
                .await
                .unwrap();
            client.observe_acknowledgments(crate::diagnostics::controller_ack);
            for sample in 0..samples {
                let outcome = client
                    .command(
                        format!("remote-{sample}"),
                        revision.clone(),
                        PlaybackCommand::StopAll,
                    )
                    .await
                    .unwrap();
                assert!(matches!(outcome, CommandOutcome::Accepted { .. }));
                tokio::time::sleep(Duration::from_millis(80)).await;
            }
            client.disconnect().await;
            done.send(()).unwrap();
        });
        let deadline = std::time::Instant::now() + Duration::from_secs(40);
        loop {
            while let Ok(update) = network.updates.try_recv() {
                if let Update::Remote(request) = update {
                    network.admit_remote(request);
                }
            }
            network.accept_pending(&mut host, "local-acceptance-probe");
            if completed.try_recv().is_ok() {
                break;
            }
            assert!(std::time::Instant::now() < deadline);
            std::thread::sleep(Duration::from_millis(1));
        }
        network.runtime.block_on(task).unwrap();
        network.runtime.block_on(session.close());
        assert!(guard.finish());
        println!(
            "Captured {samples} local policy acknowledgments and {samples} real-loopback command round trips; no media presentation or audible output measured"
        );
    }

    #[test]
    fn real_client_routes_through_guarded_shared_host_ingress() {
        let catalog = Arc::new(
            crate::catalog::load_local(
                &std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                    .join("sample-packs/podcast-pack"),
            )
            .unwrap(),
        );
        let mut host = crate::host::Host::new("network-test".into(), catalog.clone());
        let wakes = Arc::new(AtomicUsize::new(0));
        let count = wakes.clone();
        let mut network = NetworkState::new(
            host.snapshot(),
            &NetworkConfig::default(),
            Arc::new(move || {
                count.fetch_add(1, Ordering::SeqCst);
            }),
        );
        let session = network
            .runtime
            .block_on(transport::enable_test_loopback(
                network.bridge.clone(),
                RemoteCatalogView::project(&catalog).unwrap(),
            ))
            .unwrap();
        let address = session.address;
        let secret = SessionCredential::from_secret(session.credential.bearer().into()).unwrap();
        let clip_id = catalog.clips[0].identity.clone();
        let revision = catalog.revision.clone();
        let (done, completed) = std::sync::mpsc::sync_channel(1);
        let task = network.runtime.spawn(async move {
            let mut client = ControllerClient::connect_loopback(address, secret)
                .await
                .unwrap();
            let metadata = client.catalog().await.unwrap();
            assert!(metadata.get("root").is_none());
            assert!(metadata["clips"][0].get("assetPath").is_none());
            assert!(!client.icon(&clip_id).await.unwrap().is_empty());
            let accepted = client
                .command(
                    "remote-play".into(),
                    revision.clone(),
                    PlaybackCommand::Play {
                        clip_id: clip_id.clone(),
                    },
                )
                .await
                .unwrap();
            assert!(matches!(accepted, CommandOutcome::Accepted { .. }));
            let duplicate = client
                .command(
                    "remote-play".into(),
                    revision.clone(),
                    PlaybackCommand::Play { clip_id },
                )
                .await
                .unwrap();
            assert_eq!(accepted, duplicate);
            let stopped = client
                .command("remote-stop".into(), revision, PlaybackCommand::StopAll)
                .await
                .unwrap();
            assert!(matches!(stopped, CommandOutcome::Accepted { .. }));
            client.disconnect().await;
            done.send(()).unwrap();
        });
        let deadline = std::time::Instant::now() + Duration::from_secs(10);
        loop {
            while let Ok(update) = network.updates.try_recv() {
                if let Update::Remote(request) = update {
                    network.admit_remote(request);
                }
            }
            network.accept_pending(&mut host, "network-test");
            let events = host.drain_events();
            network.publish_local(host.snapshot(), &events);
            if completed.try_recv().is_ok() {
                break;
            }
            assert!(
                std::time::Instant::now() < deadline,
                "remote shared-host roundtrip timed out"
            );
            std::thread::sleep(Duration::from_millis(1));
        }
        network.runtime.block_on(task).unwrap();
        assert!(
            wakes.load(Ordering::SeqCst) > 0,
            "remote work must wake an idle application"
        );
        assert_eq!(
            host.snapshot().instances.len(),
            1,
            "retry must not create another occurrence"
        );
        network.runtime.block_on(session.close());
    }

    #[test]
    fn controller_stop_supersedes_unsent_online_actions_without_replaying_them() {
        let catalog = Arc::new(
            crate::catalog::load_local(
                &std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                    .join("sample-packs/podcast-pack"),
            )
            .unwrap(),
        );
        let mut host = crate::host::Host::new("network-test".into(), catalog.clone());
        let mut network =
            NetworkState::new(host.snapshot(), &NetworkConfig::default(), Arc::new(|| {}));
        let session = network
            .runtime
            .block_on(transport::enable_test_loopback(
                network.bridge.clone(),
                RemoteCatalogView::project(&catalog).unwrap(),
            ))
            .unwrap();
        let address = session.address;
        let secret = SessionCredential::from_secret(session.credential.bearer().into()).unwrap();
        let (connecting, mut mailbox) =
            TailscaleControllerBackend::new().begin_connect("actor-test".into());
        let mut backend = connecting.connected(&host.snapshot());
        backend
            .submit_command(PlaybackCommand::Play {
                clip_id: catalog.clips[0].identity.clone(),
            })
            .unwrap();
        backend.submit_command(PlaybackCommand::StopAll).unwrap();
        backend
            .submit_command(PlaybackCommand::Play {
                clip_id: catalog.clips[0].identity.clone(),
            })
            .unwrap();
        let updates = network.update_tx.clone();
        let wake = network.wake.clone();
        let actor = network.runtime.spawn(async move {
            let client = ControllerClient::connect_loopback(address, secret)
                .await
                .unwrap();
            controller_connected(
                1,
                client,
                &mut mailbox.normal,
                &mut mailbox.priority,
                &updates,
                &wake,
            )
            .await
            .unwrap();
        });
        let deadline = std::time::Instant::now() + Duration::from_secs(10);
        let mut accepted = 0;
        let mut superseded = 0;
        while accepted != 2 || superseded != 1 {
            while let Ok(update) = network.updates.try_recv() {
                match update {
                    Update::Remote(request) => network.admit_remote(request),
                    Update::Outcome(_, CommandOutcome::Accepted { .. }) => accepted += 1,
                    Update::Outcome(
                        _,
                        CommandOutcome::Rejected {
                            reason: Rejection::Superseded,
                        },
                    ) => superseded += 1,
                    _ => {}
                }
            }
            network.accept_pending(&mut host, "network-test");
            let events = host.drain_events();
            network.publish_local(host.snapshot(), &events);
            assert!(
                std::time::Instant::now() < deadline,
                "controller actor did not complete priority requests"
            );
            std::thread::sleep(Duration::from_millis(1));
        }
        drop(backend);
        network.runtime.block_on(actor).unwrap();
        assert_eq!(host.snapshot().instances.len(), 1);
        assert_eq!(
            host.snapshot().instances[0].state,
            crate::commands::InstanceState::Preparing
        );
        network.runtime.block_on(session.close());
    }

    #[test]
    fn address_and_catalog_projection_cannot_serve_arbitrary_paths() {
        assert_eq!(
            NetworkConfig::parse_address("ws://100.100.0.10:48732/control").unwrap(),
            "100.100.0.10:48732".parse().unwrap()
        );
        for invalid in [
            "ws://secret@100.100.0.10:48732/control",
            "ws://100.100.0.10:48732/control?token=x",
            "ws://example.com:48732/control",
            "http://100.100.0.10:48732/control",
        ] {
            assert!(NetworkConfig::parse_address(invalid).is_err());
        }
        let view = RemoteCatalogView {
            pack_id: "pack".into(),
            title: "Pack".into(),
            revision: "revision".into(),
            clips: vec![RemoteClip {
                clip_id: "pack/clip".into(),
                title: "Clip".into(),
                tags: vec![],
                category: "audio".into(),
                media_type: "audio".into(),
            }],
        };
        assert!(view.validate());
        let mut duplicates = view.clone();
        duplicates.clips.push(view.clips[0].clone());
        assert!(!duplicates.validate());
    }
}
