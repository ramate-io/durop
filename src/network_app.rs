//! Bounded application orchestration for native and authenticated Tailscale controllers.
//!
//! Native admission stays in process. Remote admission is available only after a
//! connecting backend consumes the authenticated snapshot and becomes connected.
//! Host policy, cancellation guards, and StopAll epochs remain in the shared ingress.
use crate::{
    app::Runtime,
    catalog::Catalog,
    commands::{
        CommandOutcome, Envelope, HostEvent, HostSnapshot, PlaybackCommand, Rejection,
        VisualPlacement,
    },
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
    net::{IpAddr, SocketAddr},
    sync::{Arc, Mutex},
    time::Duration,
};
use tokio::sync::{broadcast, mpsc, watch};

mod controller;

mod sources;

#[cfg(test)]
mod routing_tests;

pub use sources::ControllerSource;

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
    /// Independent listener address; outbound connection settings never overwrite it.
    pub listen_address: Option<String>,
}

/// UI intentions consumed once; disconnected button presses are discarded.
#[derive(Clone)]
pub enum NetworkAction {
    /// Independently admit or reject locally generated playback commands.
    SetLocalEnabled(bool),
    /// Issue a separate session credential for a named input source.
    AddController { name: String },
    /// Change one source's permission without affecting other sources.
    SetControllerEnabled { id: String, enabled: bool },
    /// Permanently revoke one source's session credential.
    RevokeController { id: String },
    /// Create a host-device sharing invitation using host-owned administration credentials.
    CreateInvitation { id: String },
    /// Revoke a pending Tailscale invitation independently of Durop pairing.
    RevokeInvitation { id: String },
    /// Reconcile pending and accepted invitations with Tailscale.
    RefreshInvitations,
    /// Explicitly save a masked UI input through the platform secret store.
    SaveAdminSecret { secret: String },
    /// Remove the saved administrative credential; environment overrides remain explicit.
    RemoveAdminSecret,
    /// Copy the selected controller's private connection instructions to the clipboard.
    CopyControllerHandoff { id: String },
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
    /// Host-resolved visual default; absent for audio clips or older projections.
    #[serde(default)]
    pub default_placement: Option<VisualPlacement>,
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
    // Reject oversized/duplicate projections before icons or panel resources are allocated.
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
                    && clip
                        .default_placement
                        .is_none_or(|placement| clip.media_type != "audio" && placement.is_valid())
            })
    }
}

/// Background results carry a generation so an ended session cannot update a new one.
enum Update {
    // Secret-store and Tailscale API results are processed only on the application thread.
    Sources(sources::SourceUpdate),
    // A live transport request still needs the shared host ingress guard and policy.
    Remote(RemoteCommand),
    // Ownership arrives only after catalog projection, address verification, and binding.
    HostEnabled(u64, RemoteSession),
    // Wake the idle UI when the listener's verification or permission ends.
    HostEnded(u64),
    // Report only the current enable attempt; stale generations are ignored by dispatch.
    HostFailed(u64, String),
    // One-shot OS/CLI lookup of this Mac's assigned tailnet listener address.
    ListenerDiscovered(Option<String>),
    // The authenticated initial snapshot authorizes the Connecting -> Connected transition.
    Connected(u64, HostSnapshot, RemoteCatalogView),
    // Complete remote state supersedes any previously displayed event-derived state.
    Snapshot(u64, HostSnapshot),
    // Bounded bytes fetched for an exact ID from the validated remote catalog.
    Icon(u64, String, Vec<u8>),
    // Every preview has been fetched or explicitly reported unavailable.
    CatalogReady(u64),
    // Admission/host outcome, distinct from backend Started or physical output.
    Outcome(u64, CommandOutcome),
    // Clear controller state and pending UI intentions for this generation only.
    Disconnected(u64, String),
}

/// Keep the reply ticket and admission guard attached to the same pending request.
type GuardedRequest = Arc<Mutex<Option<RemoteCommand>>>;

/// UI-facing state plus the sole Tokio runtime and private controller lifecycle owners.
#[derive(Resource)]
pub struct NetworkState {
    /// Local commands are an independent source, never an alternative to remote sources.
    pub local_enabled: bool,
    // Toggle-off cancels native work admitted earlier in the same UI frame.
    local_permission: Arc<std::sync::atomic::AtomicBool>,
    /// Address to receive commands on, independent of the selected destination.
    pub listen_address: String,
    /// Assigned tailnet listener discovered from this Mac's Tailscale status, if any.
    pub discovered_listener: Option<String>,
    // True after the one-shot OS/CLI lookup finishes, including failure.
    listener_lookup_done: bool,
    /// Independently authorized controller inputs for the current application session.
    pub controllers: Vec<ControllerSource>,
    /// Redacted administrative capability and secure-storage status.
    pub admin_status: String,
    /// Whether a validated host-side API credential is available for invitations.
    pub admin_secret_available: bool,
    // Shared authentication leases invalidate already queued commands on revocation.
    sources: transport::SourceAccessRegistry,
    // Host-side secrets and asynchronous invitation ownership stay out of UI models.
    administration: sources::Administration,
    // Track selected routing independently of connection availability; never fall back locally.
    destination: crate::app::Destination,
    // Preserve the explicit From listener choice while switching To sessions.
    receive_requested: bool,
    // A relay's immutable catalog is published only after all available previews arrive.
    remote_catalog_ready: bool,
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
    /// One-shot UI intentions; each update processes at most 64 and discards excess.
    pub actions: VecDeque<NetworkAction>,
    // One long-lived executor for sockets/CLI work; never created by a button handler.
    runtime: tokio::runtime::Runtime,
    // Native controller and host-side remote requests converge on this in-process queue.
    local: NativeControllerBackend,
    // Sole policy pump; drains emergency work first and enforces canonical stop barriers.
    inbox: CommandInbox,
    // Forwards authenticated ingress into bounded UI updates and wakes an idle event loop.
    remote_job: tokio::task::JoinHandle<()>,
    // Listener-facing channel endpoints; no native playback objects cross this boundary.
    bridge: HostBridge,
    // Publish the complete local state before broadcasting its associated events.
    snapshots: watch::Sender<HostSnapshot>,
    // Main-thread receiver for generation-tagged background results.
    updates: mpsc::Receiver<Update>,
    // Cloned into background tasks; bounded capacity limits queued UI work.
    update_tx: mpsc::Sender<Update>,
    // Explicit event-loop wake shared by queue ingress and background result producers.
    wake: Arc<dyn Fn() + Send + Sync>,
    // Owning handle revokes all listener access when disabled or replaced.
    session: Option<Arc<RemoteSession>>,
    // Cancelable enable operation, including trusted icon reads off the UI thread.
    host_job: Option<tokio::task::JoinHandle<()>>,
    // Turns background listener termination into an idle-UI status update.
    host_monitor: Option<tokio::task::JoinHandle<()>>,
    // Owns the remote socket; aborting it discards the old connection's pending work.
    client_job: Option<tokio::task::JoinHandle<()>>,
    // Authentication in progress: this state has no ControllerBackend implementation.
    connecting: Option<TailscaleControllerBackend<Connecting>>,
    // Only this ready typestate can admit remote commands from the UI.
    controller: Option<TailscaleControllerBackend<Connected>>,
    // Invalidates results from an earlier host enable/disable operation.
    host_generation: u64,
    // Invalidates results from an earlier remote connection attempt.
    client_generation: u64,
    // Correlates each remote queue admission independently of user command IDs.
    ticket: u64,
    // Bounded by ingress capacity; preserves each request's live guard and reply sender.
    pending: BTreeMap<u64, GuardedRequest>,
}

impl NetworkState {
    #[cfg(test)]
    /// Construct the same bounded orchestration without a native window event loop.
    pub(crate) fn for_ui_test(snapshot: HostSnapshot) -> Self {
        let mut network = NetworkState::new(snapshot, &NetworkConfig::default(), Arc::new(|| {}));
        network.listener_lookup_done = true;
        network
    }

    /// Listener field value: explicit address, else the discovered tailnet IP:port.
    pub fn listener_endpoint(&self) -> &str {
        if !self.listen_address.is_empty() {
            &self.listen_address
        } else {
            self.discovered_listener.as_deref().unwrap_or("")
        }
    }

    /// Explain the discovered or required listener address without repeating user input.
    pub fn listener_hint(&self) -> String {
        if let Some(address) = &self.discovered_listener {
            format!("Detected {address}. MagicDNS names are not accepted.")
        } else if self.listener_lookup_done {
            "Tailscale did not report this Mac's address. Enter the 100.x IP and port 48732 from the Tailscale app.".into()
        } else {
            "Looking up this Mac's Tailscale address…".into()
        }
    }

    /// Read-only status/CLI lookup; never enrolls a device or changes tailnet policy.
    pub fn discover_listener(&self) {
        let updates = self.update_tx.clone();
        let wake = self.wake.clone();
        self.runtime.spawn(async move {
            let found = transport::local_listener_address()
                .await
                .ok()
                .map(|address| address.to_string());
            notify(&updates, &wake, Update::ListenerDiscovered(found)).await;
        });
    }

    #[cfg(test)]
    /// Pump the authoritative queue for deterministic UI command-routing fixtures.
    pub(crate) fn accept_for_ui_test(&mut self, host: &mut crate::host::Host) {
        let session = host.session_id().to_owned();
        self.accept_pending(host, &session);
    }

    /// Admit through the native ControllerBackend; success means queued, not started.
    pub fn queue_local(&mut self, envelope: Envelope) -> Result<(), QueueError> {
        if !self.local_enabled {
            return Err(QueueError::Closed);
        }
        let live = self.local_permission.clone();
        self.local.submit_guarded(
            envelope.clone(),
            0,
            Arc::new(move || live.load(std::sync::atomic::Ordering::Acquire)),
        )?;
        crate::diagnostics::submitted(&envelope, crate::diagnostics::Origin::Local);
        Ok(())
    }

    /// Application cleanup can stop local media even when local controls are disabled.
    pub(crate) fn queue_system(&mut self, envelope: Envelope) -> Result<(), QueueError> {
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
        if self.destination != crate::app::Destination::Local {
            return;
        }
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
        self.remote_catalog_ready = false;
        if self.destination == crate::app::Destination::Remote {
            self.disable_host();
        }
        self.actions
            .retain(|action| !matches!(action, NetworkAction::RemoteCommand(_)));
    }

    /// Build the trusted metadata projection off the UI thread, then verify and bind.
    fn enable_host(&mut self, catalog: Arc<Catalog>) {
        if self.destination == crate::app::Destination::Remote && !self.remote_catalog_ready {
            self.status =
                "Waiting for the To destination's catalog before enabling remote inputs".into();
            return;
        }
        self.disable_host();
        let Ok(address) = NetworkConfig::parse_address(&self.listen_address) else {
            self.receive_requested = false;
            self.status = "Choose this Mac's Tailscale IP and port".into();
            return;
        };
        let generation = self.host_generation;
        let bridge = self.bridge.clone();
        let updates = self.update_tx.clone();
        let wake = self.wake.clone();
        let registry = self.sources.clone();
        let remote_catalog = if self.destination == crate::app::Destination::Remote {
            let Some(view) = self.remote_catalog.as_ref() else {
                self.status = "Connect the To destination before sharing controls".into();
                return;
            };
            match RemoteCatalog::new(
                serde_json::to_value(view).expect("validated catalog is serializable"),
                self.remote_icons.clone(),
            ) {
                Ok(catalog) => Some(catalog),
                Err(_) => {
                    self.status = "Wait for destination catalog previews before sharing".into();
                    return;
                }
            }
        } else {
            None
        };
        self.status = "Verifying local Tailscale address…".into();
        self.host_job = Some(self.runtime.spawn(async move {
            let result = tokio::task::spawn_blocking(move || {
                remote_catalog.map_or_else(|| RemoteCatalogView::project(&catalog), Ok)
            })
            .await;
            let result = match result {
                Ok(Ok(catalog)) => transport::enable_with_registry(
                    SessionConfig {
                        address,
                        allowed_origins: vec![],
                    },
                    bridge,
                    catalog,
                    registry,
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
        if let Ok(target) = NetworkConfig::parse_address(&self.address)
            && (self
                .session
                .as_ref()
                .is_some_and(|session| session.address == target)
                || NetworkConfig::parse_address(&self.listen_address).ok() == Some(target))
        {
            self.secret.clear();
            self.status = "To cannot point back to this Mac's From listener".into();
            return;
        }
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
        if self.destination == crate::app::Destination::Remote {
            if let Some(controller) = self.controller.as_mut() {
                controller.forward(request);
            } else {
                let _ = request.reply.send(CommandOutcome::Rejected {
                    reason: Rejection::Busy,
                });
            }
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
        let receipts = self.inbox.drain(host);
        // A controller may request a snapshot immediately after its outcome. Publish
        // host authority before releasing replies, independent of the render bridge.
        if !receipts.is_empty() {
            self.publish_local(host.snapshot(), &[]);
        }
        for receipt in receipts {
            crate::diagnostics::acknowledged(session, &receipt.command_id, &receipt.outcome);
            if let Some(ticket) = receipt.ticket.filter(|ticket| *ticket != 0) {
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
    let network = NetworkState::new(runtime.host.snapshot(), &config, wake);
    network.discover_listener();
    commands.insert_resource(network);
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
                        let Some(request) = request else {
                            break;
                        };
                        // Preserve admission order so the canonical StopAll epoch
                        // supersedes ordinary requests that were already queued.
                        while let Ok(normal) = remote.try_recv() {
                            if !notify(&forwarded, &forward_wake, Update::Remote(normal)).await {
                                return;
                            }
                        }
                        if !notify(&forwarded, &forward_wake, Update::Remote(request)).await {
                            break;
                        }
                    }
                    request = remote.recv() => {
                        let Some(request) = request else {
                            break;
                        };
                        if !notify(&forwarded, &forward_wake, Update::Remote(request)).await {
                            break;
                        }
                    }
                }
            }
        });
        NetworkState {
            local_enabled: true,
            local_permission: Arc::new(std::sync::atomic::AtomicBool::new(true)),
            listen_address: config.listen_address.clone().unwrap_or_default(),
            discovered_listener: None,
            listener_lookup_done: false,
            controllers: Vec::new(),
            admin_status: "Configure an API token to create invitations".into(),
            admin_secret_available: false,
            sources: transport::SourceAccessRegistry::default(),
            administration: sources::Administration::default(),
            destination: crate::app::Destination::Local,
            receive_requested: config.enable_host,
            remote_catalog_ready: false,
            address: config.address.clone().unwrap_or_default(),
            secret: String::new(),
            status: "Remote access disabled".into(),
            snapshot: None,
            remote_catalog: None,
            remote_icons: BTreeMap::new(),
            connected: false,
            hosting: false,
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
    network.initialize_administration();
    if network.destination != runtime.destination {
        network.disable_host();
        network.destination = runtime.destination;
    }
    let expected_session = if runtime.destination == crate::app::Destination::Remote {
        network
            .snapshot
            .as_ref()
            .map(|snapshot| snapshot.session_id.clone())
    } else if runtime.enabled {
        Some(runtime.snapshot.session_id.clone())
    } else {
        None
    };
    if expected_session.is_none()
        || network.session.as_ref().is_some_and(|session| {
            !session.is_active() || Some(&session.session_id) != expected_session.as_ref()
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
                network.receive_requested = true;
                let selected = if runtime.destination == crate::app::Destination::Remote {
                    network.snapshot.clone()
                } else {
                    runtime.enabled.then(|| runtime.host.snapshot())
                };
                if let Some(snapshot) = selected {
                    network.snapshots.send_replace(snapshot);
                    network.enable_host(runtime.catalog.clone());
                } else {
                    network.status = "Start the local host first".into();
                }
            }
            NetworkAction::DisableHost => {
                network.receive_requested = false;
                network.disable_host();
                network.status = "Remote access disabled".into();
            }
            NetworkAction::Connect => network.connect(),
            NetworkAction::Disconnect => {
                network.disconnect();
                network.status = "Controller disconnected".into();
            }
            NetworkAction::RemoteCommand(command) => {
                if !network.local_enabled {
                    network.status = "Local controls are disabled in From".into();
                    continue;
                }
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
            action => network.source_action(action),
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
            Update::Sources(update) => network.source_update(update),
            Update::HostEnabled(generation, session)
                if generation == network.host_generation
                    && Some(&session.session_id) == expected_session.as_ref() =>
            {
                network.host_job = None;
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
                network.status = "Remote inputs enabled; add controllers in From".into();
            }
            Update::HostFailed(generation, error) if generation == network.host_generation => {
                network.host_job = None;
                network.receive_requested = false;
                network.status = format!("Remote host unavailable: {error}")
            }
            Update::ListenerDiscovered(found) => {
                network.listener_lookup_done = true;
                if let Some(address) = found {
                    network.discovered_listener = Some(address.clone());
                    if network.listen_address.is_empty() {
                        network.listen_address = address;
                    }
                }
            }
            Update::HostEnded(generation) if generation == network.host_generation => {
                network.receive_requested = false;
                network.disable_host();
                network.status = "Remote hosting ended; Tailscale address unavailable".into();
            }
            Update::Connected(generation, snapshot, catalog)
                if generation == network.client_generation =>
            {
                if snapshot.session_id == runtime.snapshot.session_id {
                    network.disconnect();
                    network.status = "Cannot route controls back into this host".into();
                    continue;
                }
                let Some(connecting) = network.connecting.take() else {
                    continue;
                };
                network.controller = Some(connecting.connected(&snapshot));
                if runtime.destination == crate::app::Destination::Remote {
                    network.snapshots.send_replace(snapshot.clone());
                }
                network.snapshot = Some(snapshot);
                network.remote_catalog = Some(catalog);
                network.connected = true;
                network.status = "Remote host connected".into();
            }
            Update::Snapshot(generation, snapshot) if generation == network.client_generation => {
                if runtime.destination == crate::app::Destination::Remote {
                    network.snapshots.send_replace(snapshot.clone());
                }
                network.snapshot = Some(snapshot)
            }
            Update::Icon(generation, id, png) if generation == network.client_generation => {
                network.remote_icons.insert(id, png);
            }
            Update::CatalogReady(generation) if generation == network.client_generation => {
                network.remote_catalog_ready = true;
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
    if network.receive_requested && !network.hosting && network.host_job.is_none() {
        let selected = if runtime.destination == crate::app::Destination::Remote {
            network
                .remote_catalog_ready
                .then(|| network.snapshot.clone())
                .flatten()
        } else {
            runtime.enabled.then(|| runtime.host.snapshot())
        };
        if let Some(snapshot) = selected {
            network.snapshots.send_replace(snapshot);
            network.enable_host(runtime.catalog.clone());
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
        if let Ok(endpoint) = address.parse() {
            return Ok(endpoint);
        }
        address
            .parse::<IpAddr>()
            .map(|ip| SocketAddr::new(ip, crate::transport::DEFAULT_LISTEN_PORT))
            .map_err(|_| ())
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
                default_placement: crate::host::Host::default_placement(clip),
            });
            let mut png = Vec::new();
            std::fs::File::open(&clip.icon_path)
                .map_err(|_| format!("Catalog icon unavailable for {}", clip.identity))?
                .take((crate::transport::MAX_ICON_PNG + 1) as u64)
                .read_to_end(&mut png)
                .map_err(|_| format!("Catalog icon unavailable for {}", clip.identity))?;
            if png.len() > crate::transport::MAX_ICON_PNG {
                return Err(format!(
                    "Clip {} icon is {} KiB; remote sharing allows at most {} KiB",
                    clip.identity,
                    png.len().div_ceil(1024),
                    crate::transport::MAX_ICON_PNG / 1024
                ));
            }
            if !crate::transport::valid_icon_png(&png) {
                return Err(format!(
                    "Clip {} icon is not a bounded PNG preview",
                    clip.identity
                ));
            }
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
    let mut catalog_ready = false;
    let mut cutoff = 0;
    loop {
        let command = tokio::select! {
            biased;
            command = priority.recv() => command,
            command = normal.recv() => command,
            update = client.next_update() => {
                match update.map_err(|error| error.to_string())? {
                    ControllerUpdate::Event(_) => refresh = true,
                    ControllerUpdate::Snapshot(snapshot) => {
                        if !notify(updates, wake, Update::Snapshot(generation, snapshot)).await {
                            break;
                        }
                    }
                }
                continue;
            }
            _ = tick.tick() => {
                if refresh {
                    let snapshot = client.refresh_snapshot().await.map_err(|error| error.to_string())?;
                    refresh = false;
                    if !notify(updates, wake, Update::Snapshot(generation, snapshot)).await {
                        break;
                    }
                } else if let Some(clip) = icons.next() {
                    match client.icon(&clip.clip_id).await {
                        Ok(png) => {
                            if !notify(updates, wake, Update::Icon(generation, clip.clip_id, png)).await {
                                break;
                            }
                        }
                        Err(transport::ControllerError::UnknownIcon) => {},
                        Err(error) => return Err(error.to_string()),
                    }
                } else if !catalog_ready {
                    catalog_ready = true;
                    if !notify(updates, wake, Update::CatalogReady(generation)).await {
                        break;
                    }
                }
                continue;
            }
        };
        let Some(mut request) = command else {
            break;
        };
        if !request.is_active() {
            continue;
        }
        if request.epoch < cutoff {
            request.reply(CommandOutcome::Rejected {
                reason: Rejection::Superseded,
            });
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
        let outcome = client
            .submit_guarded_envelope(request.envelope.clone(), &|| request.is_active())
            .await;
        match outcome {
            Ok(Some(outcome)) => {
                request.reply(outcome.clone());
                if stop && matches!(outcome, CommandOutcome::Accepted { .. }) {
                    cutoff = cutoff.max(request.epoch);
                }
                refresh = true;
                if !notify(updates, wake, Update::Outcome(generation, outcome)).await {
                    break;
                }
            }
            Ok(None) => continue,
            Err(transport::ControllerError::Busy) => {
                request.reply(CommandOutcome::Rejected {
                    reason: Rejection::Busy,
                });
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
        let clip = catalog
            .clips
            .iter()
            .find(|clip| crate::host::Host::default_placement(clip).is_some())
            .unwrap();
        let clip_id = clip.identity.clone();
        let default_placement = crate::host::Host::default_placement(clip).unwrap();
        let placement = VisualPlacement {
            width_bps: 4200,
            x_bps: 1234,
            y_bps: 8765,
        };
        let expected_defaults: BTreeMap<_, _> = catalog
            .clips
            .iter()
            .map(|clip| {
                (
                    clip.identity.clone(),
                    crate::host::Host::default_placement(clip),
                )
            })
            .collect();
        let revision = catalog.revision.clone();
        let published = network.bridge.snapshot.clone();
        let task = network.runtime.spawn(async move {
            let mut client = ControllerClient::connect_loopback(address, secret)
                .await
                .unwrap();
            let metadata = client.catalog().await.unwrap();
            assert!(metadata.get("root").is_none());
            assert!(metadata["clips"][0].get("assetPath").is_none());
            let view: RemoteCatalogView = serde_json::from_value(metadata).unwrap();
            assert!(view.validate());
            for clip in view.clips {
                assert_eq!(clip.default_placement, expected_defaults[&clip.clip_id]);
            }
            assert!(!client.icon(&clip_id).await.unwrap().is_empty());
            let expected = Arc::new(Mutex::new(Some(placement)));
            let acknowledged = expected.clone();
            client.observe_acknowledgments(move |_| {
                assert_eq!(
                    published.borrow().instances[0].placement,
                    *acknowledged.lock().unwrap(),
                    "authoritative placement must be published before its outcome"
                );
            });
            let accepted = client
                .command(
                    "remote-play".into(),
                    revision.clone(),
                    PlaybackCommand::PlayVisual {
                        clip_id: clip_id.clone(),
                        placement,
                    },
                )
                .await
                .unwrap();
            assert!(matches!(accepted, CommandOutcome::Accepted { .. }));
            let duplicate = client
                .command(
                    "remote-play".into(),
                    revision.clone(),
                    PlaybackCommand::PlayVisual { clip_id, placement },
                )
                .await
                .unwrap();
            assert_eq!(accepted, duplicate);
            let CommandOutcome::Accepted {
                instance_id: Some(instance_id),
            } = &accepted
            else {
                panic!("missing visual occurrence");
            };
            let snapshot = client.refresh_snapshot().await.unwrap();
            assert_eq!(snapshot.instances[0].placement, Some(placement));
            for (index, next) in [Some(default_placement), None].into_iter().enumerate() {
                *expected.lock().unwrap() = next;
                let outcome = client
                    .command(
                        format!("remote-placement-{index}"),
                        revision.clone(),
                        PlaybackCommand::SetVisual {
                            instance_id: instance_id.clone(),
                            placement: next,
                        },
                    )
                    .await
                    .unwrap();
                assert!(matches!(outcome, CommandOutcome::Accepted { .. }));
                let snapshot = client.refresh_snapshot().await.unwrap();
                assert_eq!(snapshot.instances[0].placement, next);
            }
            let stopped = client
                .command("remote-stop".into(), revision, PlaybackCommand::StopAll)
                .await
                .unwrap();
            assert!(matches!(stopped, CommandOutcome::Accepted { .. }));
            client.disconnect().await;
        });
        let deadline = std::time::Instant::now() + Duration::from_secs(10);
        loop {
            while let Ok(update) = network.updates.try_recv() {
                if let Update::Remote(request) = update {
                    network.admit_remote(request);
                }
            }
            network.accept_pending(&mut host, "network-test");
            // No render-bridge publication: ingress must publish the acknowledged
            // state itself before the client can immediately refresh it.
            if task.is_finished() {
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
        assert_eq!(
            NetworkConfig::parse_address("100.75.38.86").unwrap(),
            "100.75.38.86:48732".parse().unwrap()
        );
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
                default_placement: None,
            }],
        };
        assert!(view.validate());
        let mut missing_default = serde_json::to_value(&view).unwrap();
        missing_default["clips"][0]
            .as_object_mut()
            .unwrap()
            .remove("defaultPlacement");
        assert!(
            serde_json::from_value::<RemoteCatalogView>(missing_default)
                .unwrap()
                .clips[0]
                .default_placement
                .is_none()
        );
        let mut duplicates = view.clone();
        duplicates.clips.push(view.clips[0].clone());
        assert!(!duplicates.validate());
        let mut visual = view.clone();
        visual.clips[0].media_type = "gif".into();
        visual.clips[0].default_placement = Some(VisualPlacement {
            width_bps: 3000,
            x_bps: 5000,
            y_bps: 5000,
        });
        assert!(visual.validate());
        let serialized = serde_json::to_value(&visual).unwrap();
        assert_eq!(serialized["clips"][0]["defaultPlacement"]["widthBps"], 3000);
        assert!(
            serde_json::from_value::<RemoteCatalogView>(serialized)
                .unwrap()
                .validate()
        );
        visual.clips[0].default_placement.as_mut().unwrap().x_bps = 10_001;
        assert!(!visual.validate());
        visual.clips[0].default_placement.as_mut().unwrap().x_bps = 5000;
        visual.clips[0].media_type = "audio".into();
        assert!(!visual.validate());
    }

    #[test]
    fn native_visual_commands_use_the_shared_host_path() {
        let catalog = Arc::new(
            crate::catalog::load_local(
                &crate::app::resource_root().join("sample-packs/podcast-pack"),
            )
            .unwrap(),
        );
        let mut host = crate::host::Host::new("native-visual".into(), catalog.clone());
        let mut network =
            NetworkState::new(host.snapshot(), &NetworkConfig::default(), Arc::new(|| {}));
        let clip = catalog
            .clips
            .iter()
            .find(|clip| crate::host::Host::default_placement(clip).is_some())
            .unwrap();
        let placement = VisualPlacement {
            width_bps: 4200,
            x_bps: 1234,
            y_bps: 8765,
        };
        let envelope = |id: &str, command| Envelope {
            protocol_version: crate::commands::PROTOCOL_VERSION,
            session_id: "native-visual".into(),
            command_id: id.into(),
            catalog_revision: catalog.revision.clone(),
            command,
        };
        network
            .queue_local(envelope(
                "play",
                PlaybackCommand::PlayVisual {
                    clip_id: clip.identity.clone(),
                    placement,
                },
            ))
            .unwrap();
        assert!(
            network
                .accept_pending(&mut host, "native-visual")
                .is_empty()
        );
        let snapshot = host.snapshot();
        assert_eq!(snapshot.instances[0].placement, Some(placement));
        let instance_id = snapshot.instances[0].instance_id.clone();
        for (index, next) in [
            Some(VisualPlacement {
                width_bps: 8000,
                ..placement
            }),
            None,
        ]
        .into_iter()
        .enumerate()
        {
            network
                .queue_local(envelope(
                    &format!("set-{index}"),
                    PlaybackCommand::SetVisual {
                        instance_id: instance_id.clone(),
                        placement: next,
                    },
                ))
                .unwrap();
            assert!(
                network
                    .accept_pending(&mut host, "native-visual")
                    .is_empty()
            );
            assert_eq!(host.snapshot().instances[0].placement, next);
        }
    }

    #[test]
    fn bundled_reactions_project_within_remote_icon_bounds() {
        let catalog = crate::catalog::load_local(
            &std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("sample-packs/reactions"),
        )
        .unwrap();
        RemoteCatalogView::project(&catalog).unwrap();
    }
}
