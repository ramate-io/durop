//! Bounded application adapter: one runtime, shared guarded host ingress, no offline replay.
use crate::{
    app::Runtime,
    catalog::Catalog,
    commands::{CommandOutcome, Envelope, HostEvent, HostSnapshot, PlaybackCommand, Rejection},
    host::{CommandInbox, CommandSender, QueueError, command_channel},
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

#[derive(Resource, Default)]
pub struct NetworkConfig {
    pub address: Option<String>,
    pub enable_host: bool,
}
#[derive(Clone)]
pub enum NetworkAction {
    EnableHost,
    DisableHost,
    Connect,
    Disconnect,
    RemoteCommand(PlaybackCommand),
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RemoteClip {
    pub clip_id: String,
    pub title: String,
    pub tags: Vec<String>,
    pub category: String,
    pub media_type: String,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RemoteCatalogView {
    pub pack_id: String,
    pub title: String,
    pub revision: String,
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
type GuardedRequest = Arc<Mutex<Option<RemoteCommand>>>;
#[derive(Debug)]
struct ControllerRequest {
    command: PlaybackCommand,
    epoch: u64,
}
#[derive(Resource)]
pub struct NetworkState {
    pub address: String,
    /// Explicit pairing input only. Taken and cleared when Connect is pressed.
    pub secret: String,
    pub status: String,
    pub snapshot: Option<HostSnapshot>,
    pub remote_catalog: Option<RemoteCatalogView>,
    pub remote_icons: BTreeMap<String, Vec<u8>>,
    pub connected: bool,
    pub hosting: bool,
    pub pairing_secret: Option<String>,
    pub actions: VecDeque<NetworkAction>,
    runtime: tokio::runtime::Runtime,
    local: CommandSender,
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
    client: Option<mpsc::Sender<ControllerRequest>>,
    client_priority: Option<mpsc::Sender<ControllerRequest>>,
    host_generation: u64,
    client_generation: u64,
    client_epoch: u64,
    ticket: u64,
    pending: BTreeMap<u64, GuardedRequest>,
}
impl NetworkState {
    pub fn queue_local(&self, envelope: Envelope) -> Result<(), QueueError> {
        self.local.try_send(envelope)
    }
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
    fn disconnect(&mut self) {
        self.client_generation = self.client_generation.wrapping_add(1);
        self.client = None;
        self.client_priority = None;
        self.client_epoch = 0;
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
    fn enable_host(&mut self, catalog: Arc<Catalog>) {
        self.disable_host();
        let Ok(address) = parse_address(&self.address) else {
            self.status = "Choose this Mac's Tailscale IP and port".into();
            return;
        };
        let generation = self.host_generation;
        let bridge = self.bridge.clone();
        let updates = self.update_tx.clone();
        let wake = self.wake.clone();
        self.status = "Verifying local Tailscale address…".into();
        self.host_job = Some(self.runtime.spawn(async move {
            let result = tokio::task::spawn_blocking(move || catalog_projection(&catalog)).await;
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
    fn connect(&mut self) {
        self.disconnect();
        let secret = std::mem::take(&mut self.secret);
        let (Ok(address), Ok(credential)) = (
            parse_address(&self.address),
            SessionCredential::from_secret(secret),
        ) else {
            self.status = "Enter a Tailscale IP:port and pairing credential".into();
            return;
        };
        let generation = self.client_generation;
        let (normal, normal_rx) = mpsc::channel(16);
        let (priority, priority_rx) = mpsc::channel(8);
        self.client = Some(normal);
        self.client_priority = Some(priority);
        self.status = "Connecting to selected host…".into();
        let updates = self.update_tx.clone();
        let wake = self.wake.clone();
        self.client_job = Some(self.runtime.spawn(controller(
            generation,
            address,
            credential,
            normal_rx,
            priority_rx,
            updates,
            wake,
        )));
    }
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
        match self.local.try_send_guarded(envelope, ticket, guard) {
            Ok(()) => {
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
    fn accept_pending(&mut self, host: &mut crate::host::Host) -> Vec<CommandOutcome> {
        let mut local = Vec::new();
        for receipt in self.inbox.drain(host) {
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
    commands.insert_resource(make_state(runtime.host.snapshot(), &config, wake));
}

fn make_state(
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
        local,
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
        client: None,
        client_priority: None,
        host_generation: 0,
        client_generation: 0,
        client_epoch: 0,
        ticket: 0,
        pending: BTreeMap::new(),
    }
}

fn dispatch(mut network: ResMut<NetworkState>, mut runtime: ResMut<Runtime>) {
    if !runtime.enabled
        || network.session.as_ref().is_some_and(|session| {
            !session.is_active() || session.session_id != runtime.host.snapshot().session_id
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
                let stop = matches!(command, PlaybackCommand::StopAll);
                let sender = if stop {
                    network.client_priority.clone()
                } else {
                    network.client.clone()
                };
                let epoch = if stop {
                    network
                        .client_epoch
                        .checked_add(1)
                        .expect("controller epoch exhausted")
                } else {
                    network.client_epoch
                };
                if sender.as_ref().is_none_or(|sender| {
                    sender
                        .try_send(ControllerRequest { command, epoch })
                        .is_err()
                }) {
                    network.status = "Remote busy or disconnected; command not queued".into();
                } else if stop {
                    network.client_epoch = epoch;
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
                    && session.session_id == runtime.host.snapshot().session_id =>
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
    for outcome in network.accept_pending(&mut runtime.host) {
        if let CommandOutcome::Rejected { reason } = outcome {
            runtime.status = format!("Command rejected: {reason:?}");
        }
    }
}

fn parse_address(address: &str) -> Result<SocketAddr, ()> {
    let address = if let Some(url) = address.strip_prefix("ws://") {
        url.strip_suffix("/control").ok_or(())?
    } else {
        address
    };
    address.parse().map_err(|_| ())
}
fn catalog_projection(catalog: &Catalog) -> Result<RemoteCatalog, String> {
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
async fn controller(
    generation: u64,
    address: SocketAddr,
    credential: SessionCredential,
    mut normal: mpsc::Receiver<ControllerRequest>,
    mut priority: mpsc::Receiver<ControllerRequest>,
    updates: mpsc::Sender<Update>,
    wake: Arc<dyn Fn() + Send + Sync>,
) {
    let result = controller_session(
        generation,
        address,
        credential,
        &mut normal,
        &mut priority,
        &updates,
        &wake,
    )
    .await;
    if let Err(error) = result {
        let _ = notify(&updates, &wake, Update::Disconnected(generation, error)).await;
    }
}
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
async fn controller_connected(
    generation: u64,
    mut client: ControllerClient,
    normal: &mut mpsc::Receiver<ControllerRequest>,
    priority: &mut mpsc::Receiver<ControllerRequest>,
    updates: &mpsc::Sender<Update>,
    wake: &Arc<dyn Fn() + Send + Sync>,
) -> Result<(), String> {
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
    let identity = bevy::asset::uuid::Uuid::new_v4().to_string();
    let mut command_id: u64 = 0;
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
        let stop = matches!(request.command, PlaybackCommand::StopAll);
        command_id = command_id
            .checked_add(1)
            .ok_or("Command sequence exhausted")?;
        let outcome = client
            .command(
                format!("{identity}-{command_id}"),
                client.current_snapshot().catalog_revision.clone(),
                request.command,
            )
            .await;
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
        let mut network = make_state(
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
                catalog_projection(&catalog).unwrap(),
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
            network.accept_pending(&mut host);
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
        let mut network = make_state(host.snapshot(), &NetworkConfig::default(), Arc::new(|| {}));
        let session = network
            .runtime
            .block_on(transport::enable_test_loopback(
                network.bridge.clone(),
                catalog_projection(&catalog).unwrap(),
            ))
            .unwrap();
        let address = session.address;
        let secret = SessionCredential::from_secret(session.credential.bearer().into()).unwrap();
        let (normal, mut normal_rx) = mpsc::channel(16);
        let (priority, mut priority_rx) = mpsc::channel(8);
        normal
            .try_send(ControllerRequest {
                command: PlaybackCommand::Play {
                    clip_id: catalog.clips[0].identity.clone(),
                },
                epoch: 0,
            })
            .unwrap();
        priority
            .try_send(ControllerRequest {
                command: PlaybackCommand::StopAll,
                epoch: 1,
            })
            .unwrap();
        normal
            .try_send(ControllerRequest {
                command: PlaybackCommand::Play {
                    clip_id: catalog.clips[0].identity.clone(),
                },
                epoch: 1,
            })
            .unwrap();
        let updates = network.update_tx.clone();
        let wake = network.wake.clone();
        let actor = network.runtime.spawn(async move {
            let client = ControllerClient::connect_loopback(address, secret)
                .await
                .unwrap();
            controller_connected(1, client, &mut normal_rx, &mut priority_rx, &updates, &wake)
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
            network.accept_pending(&mut host);
            let events = host.drain_events();
            network.publish_local(host.snapshot(), &events);
            assert!(
                std::time::Instant::now() < deadline,
                "controller actor did not complete priority requests"
            );
            std::thread::sleep(Duration::from_millis(1));
        }
        drop(normal);
        drop(priority);
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
            parse_address("ws://100.100.0.10:48732/control").unwrap(),
            "100.100.0.10:48732".parse().unwrap()
        );
        for invalid in [
            "ws://secret@100.100.0.10:48732/control",
            "ws://100.100.0.10:48732/control?token=x",
            "ws://example.com:48732/control",
            "http://100.100.0.10:48732/control",
        ] {
            assert!(parse_address(invalid).is_err());
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
