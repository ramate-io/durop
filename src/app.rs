//! Application modes and the real host/media bridge. No decoder work runs in UI code.
use crate::{
    catalog::{Catalog, CatalogCache, Clip, Fit, PackSource},
    commands::{Envelope, PROTOCOL_VERSION, PlaybackCommand, VisualPlacement},
    host::{BackendEffect, BackendEvent, BackendEventKind, BackendToken, Host},
    media::{MediaEventKind, MediaSpec, MediaWorker},
    overlay::{OVERLAY_LAYER, OverlayWindow, PlaybackOverlay},
};
use bevy::{
    asset::RenderAssetUsages,
    camera::visibility::RenderLayers,
    prelude::*,
    render::render_resource::{Extent3d, TextureDimension, TextureFormat},
    window::PrimaryWindow,
};
use std::{
    collections::{BTreeMap, VecDeque},
    path::PathBuf,
    sync::{Arc, Mutex, mpsc},
    time::Duration,
};

/// Select which application roles are active at startup.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    /// Run the panel and local playback host.
    Local,
    /// Run the local host with the panel initially hidden.
    Host,
    /// Control another host without loading a local media pack.
    Controller,
}

/// Launch configuration resolved from benign preferences and explicit command-line arguments.
#[derive(Debug, Clone)]
pub struct Config {
    /// Roles enabled at startup.
    pub mode: Mode,
    /// Local pack path or encoded Git source.
    pub pack: PathBuf,
    /// Preferred overlay monitor index.
    pub display: Option<usize>,
    /// Explicit remote WebSocket endpoint; no credential is embedded.
    pub connect: Option<String>,
    /// Explicit local Tailscale address for the host listener.
    pub listen: Option<String>,
    /// New capture file to create, if timing capture is requested.
    pub diagnostics: Option<PathBuf>,
}

impl Config {
    /// Resolve the offline reaction catalog shipped with this executable.
    pub fn bundled_pack() -> PathBuf {
        resource_root().join("sample-packs/reactions")
    }

    /// Parse startup options without accepting session credentials on the command line.
    pub fn parse(args: impl IntoIterator<Item = String>) -> Result<Self, String> {
        let preferences = crate::preferences::Preferences::load();
        let mut config = Self {
            mode: Mode::Local,
            pack: crate::preferences::Preferences::resolved_pack(
                &preferences.pack,
                &Self::bundled_pack(),
            ),
            display: preferences.display,
            connect: None,
            listen: None,
            diagnostics: None,
        };
        let mut args = args.into_iter();
        while let Some(arg) = args.next() {
            let mut value = || args.next().ok_or_else(|| format!("{arg} needs a value"));
            match arg.as_str() {
                "--mode" => {
                    config.mode = match value()?.as_str() {
                        "local" => Mode::Local,
                        "host" => Mode::Host,
                        "controller" => Mode::Controller,
                        _ => return Err("mode must be local, host or controller".into()),
                    };
                }
                "--pack" => config.pack = value()?.into(),
                "--display" => {
                    config.display = Some(value()?.parse().map_err(|_| "display must be a nonnegative index")?);
                }
                "--connect" => config.connect = Some(value()?),
                "--listen-tailscale" => config.listen = Some(value()?),
                "--diagnostics" => config.diagnostics = Some(value()?.into()),
                "--help" | "-h" => return Err("durop [--mode local|host|controller] [--pack DIRECTORY] [--display INDEX] [--connect ws://HOST:48732/control] [--listen-tailscale IP:PORT] [--diagnostics NEW_JSONL_FILE]".into()),
                _ => return Err(format!("unknown option {arg}")),
            }
        }
        Ok(config)
    }
}

/// Locate packaged Resources beside the executable, or the source root during development.
pub fn resource_root() -> PathBuf {
    if let Ok(exe) = std::env::current_exe()
        && let Some(directory) = exe.parent()
        && directory.file_name().is_some_and(|name| name == "MacOS")
        && let Some(contents) = directory.parent()
    {
        return contents.join("Resources");
    }
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

impl Config {
    /// Interpret a local directory or an explicit Git source and reference.
    fn pack_source(value: &str) -> Result<PackSource, String> {
        if let Some(value) = value.strip_prefix("git+file://") {
            let (path, reference) = value
                .rsplit_once('#')
                .ok_or("local Git pack requires git+file://PATH#REF")?;
            Ok(PackSource::LocalGit {
                path: path.into(),
                reference: reference.into(),
            })
        } else if value.starts_with("https://") {
            let (url, reference) = value.rsplit_once('#').unwrap_or((value, "HEAD"));
            if let Some((reference, directory)) = reference.split_once(':') {
                return Ok(PackSource::HttpsGitDirectory {
                    url: url.into(),
                    reference: reference.into(),
                    directory: directory.into(),
                });
            }
            Ok(PackSource::HttpsGit {
                url: url.into(),
                reference: reference.into(),
            })
        } else {
            Ok(PackSource::LocalDirectory(value.into()))
        }
    }
}

/// Choose the host that receives newly requested playback.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Destination {
    /// Send new playback to this process’s host.
    Local,
    /// Send new playback to the authenticated remote host.
    Remote,
}

/// User intent routed through the shared application command path.
#[derive(Debug, Clone)]
pub enum Action {
    /// Request playback of a stable clip identifier.
    Play(String),
    /// Request visual playback with a bounded per-occurrence placement override.
    PlayVisual {
        /// Stable catalog identity admitted by the selected host.
        clip_id: String,
        /// Position and fit-box width in normalized host-display coordinates.
        placement: VisualPlacement,
    },
    /// Change or reset one live visual occurrence without restarting its decoder.
    SetVisual {
        /// Host occurrence identity, including occurrences still preparing.
        instance_id: String,
        /// Explicit override, or None to restore the catalog default.
        placement: Option<VisualPlacement>,
    },
    /// Stop one host occurrence.
    Stop(String),
    /// Stop all local playback and the selected remote host.
    StopAll,
    /// Enable or disable remote access to the local host.
    ToggleHost,
    /// Hide the panel while preserving a usable stop/reopen path.
    Hide,
    /// Show the control panel.
    Show,
    /// Stop playback and finish orderly application shutdown.
    Quit,
    /// Prepare a source asynchronously before replacing the active catalog.
    LoadPack(String),
    /// Choose the destination for subsequent playback.
    Destination(Destination),
    /// Choose a specific monitor, or restore the primary-display default.
    SelectDisplay(Option<usize>),
}

/// Ordered pending user actions collected from the panel and native callbacks.
#[derive(Resource, Default)]
pub struct Actions(pub VecDeque<Action>);

/// Track whether the native Dock callback can restore a hidden panel.
#[derive(Resource, Default)]
pub struct ReopenSupport(pub bool);

/// Own one occurrence’s decoder, render resources and generation token.
struct Playback {
    /// Host generation identity used to reject stale callbacks.
    token: BackendToken,
    /// Bounded decoder worker for this occurrence.
    worker: MediaWorker,
    /// Immutable metadata and file paths pinned for this playback.
    clip: Clip,
    /// Explicit occurrence override; None resolves the immutable catalog default.
    placement: Option<VisualPlacement>,
    /// Texture handle reused as decoded frames arrive.
    image: Option<Handle<Image>>,
    /// Overlay entity presenting the current texture.
    sprite: Option<Entity>,
    /// Deferred preparation/decode failure to report to the host.
    failure: Option<String>,
    /// Most recently published frame dimensions.
    dimensions: UVec2,
}

/// Application adapter joining the authoritative host, immutable catalog and per-occurrence media workers.
#[derive(Resource)]
pub struct Runtime {
    /// Authoritative local command and lifecycle state.
    pub host: Host,
    /// Last published local host view.
    pub snapshot: crate::commands::HostSnapshot,
    /// Pinned active catalog shared with the host.
    pub catalog: Arc<Catalog>,
    /// Startup role selection.
    pub mode: Mode,
    /// Target host for newly requested playback.
    pub destination: Destination,
    /// Whether the local pack is usable for playback.
    pub enabled: bool,
    /// Current user-facing application status.
    pub status: String,
    /// Last playback errors keyed by clip identity.
    pub errors: BTreeMap<String, String>,
    /// Editable source string displayed in the panel.
    pub pack_input: String,
    /// Whether a pack refresh is outstanding.
    pub loading: bool,
    /// Whether stdin can provide a fallback control path.
    pub terminal_active: bool,
    /// Selected monitor index persisted as a benign preference.
    pub display: Option<usize>,
    /// Live occurrences and their owned decoder/render resources.
    workers: BTreeMap<String, Playback>,
    /// Monotonic local command identifier suffix.
    next_command: u64,
    /// Whether exit is waiting for all playback resources to stop.
    quitting: bool,
    /// Whether host changes require a new published snapshot.
    snapshot_dirty: bool,
    /// Receive completed asynchronous catalog preparations.
    load_rx: Mutex<mpsc::Receiver<Result<Arc<Catalog>, String>>>,
    /// Publish prepared catalogs from the background loader.
    load_tx: mpsc::Sender<Result<Arc<Catalog>, String>>,
}

impl Runtime {
    /// Generate an unpredictable local host session identifier from operating-system randomness.
    fn session_id() -> String {
        // UUID randomness comes from the OS, never from controller input.
        bevy::asset::uuid::Uuid::new_v4().to_string()
    }
}

impl Runtime {
    /// Initialize the requested role; retain catalog errors as recoverable panel state.
    pub fn new(config: &Config) -> Result<Self, String> {
        let empty = || {
            Arc::new(Catalog {
                root: PathBuf::new(),
                manifest: crate::catalog::PackManifest {
                    schema_version: 1,
                    pack_id: "remote".into(),
                    title: "Remote host".into(),
                },
                clips: vec![],
                revision: "unconnected".into(),
            })
        };
        let result = if config.mode == Mode::Controller {
            Ok(empty())
        } else {
            Config::pack_source(&config.pack.to_string_lossy()).and_then(|source| {
                CatalogCache::new(crate::preferences::Preferences::cache_dir().join("packs"))
                    .and_then(|cache| cache.prepare(&source))
                    .map_err(|e| e.to_string())
            })
        };
        let failure = result.as_ref().err().cloned();
        let catalog = result.unwrap_or_else(|_| empty());
        let (load_tx, load_rx) = mpsc::channel();
        let host = Host::new(Runtime::session_id(), catalog.clone());
        let snapshot = host.snapshot();
        Ok(Self {
            host,
            snapshot,
            catalog,
            mode: config.mode,
            destination: if config.mode == Mode::Controller {
                Destination::Remote
            } else {
                Destination::Local
            },
            enabled: config.mode != Mode::Controller && failure.is_none(),
            status: if let Some(message) = failure {
                format!("Pack load failed: {message}; choose a pack and Load / refresh")
            } else if config.mode == Mode::Controller {
                "Remote host unavailable: connect through transport when configured".into()
            } else {
                "This Mac | host running".into()
            },
            errors: BTreeMap::new(),
            pack_input: config.pack.to_string_lossy().into_owned(),
            loading: false,
            terminal_active: true,
            display: config.display,
            workers: BTreeMap::new(),
            next_command: 0,
            quitting: false,
            snapshot_dirty: true,
            load_rx: Mutex::new(load_rx),
            load_tx,
        })
    }

    /// Wrap a local command with the active session/revision and attempt bounded admission.
    fn submit(&mut self, command: PlaybackCommand, network: &mut crate::network_app::NetworkState) {
        self.next_command += 1;
        let envelope = Envelope {
            protocol_version: PROTOCOL_VERSION,
            session_id: self.snapshot.session_id.clone(),
            command_id: format!("local-{}", self.next_command),
            catalog_revision: self.catalog.revision.clone(),
            command,
        };
        if let Err(reason) = network.queue_local(envelope) {
            self.status = format!("Queue admission failed: {reason:?}");
        }
    }

    /// Route every occurrence command through the selected controller admission path.
    fn submit_selected(
        &mut self,
        command: PlaybackCommand,
        network: &mut crate::network_app::NetworkState,
    ) {
        if self.destination == Destination::Remote {
            network
                .actions
                .push_back(crate::network_app::NetworkAction::RemoteCommand(command));
        } else if !self.enabled
            && matches!(
                command,
                PlaybackCommand::Play { .. } | PlaybackCommand::PlayVisual { .. }
            )
        {
            self.status = "Start the local host before playing".into();
        } else {
            self.submit(command, network);
        }
    }

    /// Request local cleanup and, when selected, a remote stop without claiming completion.
    pub(crate) fn request_stop_all(&mut self, network: &mut crate::network_app::NetworkState) {
        self.status = "Stop all requested; waiting for playback resources to stop".into();
        if self.destination == Destination::Remote {
            network
                .actions
                .push_back(crate::network_app::NetworkAction::RemoteCommand(
                    PlaybackCommand::StopAll,
                ));
        }
        self.submit(PlaybackCommand::StopAll, network);
    }

    /// Keep the event loop active while playback or orderly shutdown needs progress.
    pub fn busy(&self) -> bool {
        !self.workers.is_empty() || self.quitting
    }
}

/// Schedule user actions, media callbacks and native reopen support in dependency order.
pub struct RuntimePlugin;
impl Plugin for RuntimePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Actions>()
            .init_resource::<ReopenSupport>()
            .add_systems(
                Update,
                (actions.before(crate::overlay::apply_controls), media_bridge)
                    .chain()
                    .before(crate::overlay::sync_overlay),
            );
        #[cfg(target_os = "macos")]
        app.add_systems(Startup, crate::app_macos::install)
            .add_systems(Update, crate::app_macos::poll.before(actions));
    }
}

/// Apply queued user intents and asynchronous catalog results on the application thread.
#[expect(
    clippy::too_many_arguments,
    reason = "Bevy declares independent ECS resources as system parameters"
)]
pub(crate) fn actions(
    mut runtime: ResMut<Runtime>,
    mut actions: ResMut<Actions>,
    mut controls: ResMut<crate::overlay::PendingControls>,
    mut primary: Single<&mut Window, (With<PrimaryWindow>, Without<OverlayWindow>)>,
    proxy: Res<bevy::winit::EventLoopProxyWrapper>,
    mut network: ResMut<crate::network_app::NetworkState>,
    reopen: Res<ReopenSupport>,
    monitors: Query<&bevy::window::Monitor>,
) {
    for action in actions.0.drain(..) {
        match action {
            Action::Play(clip_id) => {
                runtime.submit_selected(PlaybackCommand::Play { clip_id }, &mut network);
            }
            Action::PlayVisual { clip_id, placement } => {
                runtime.submit_selected(
                    PlaybackCommand::PlayVisual { clip_id, placement },
                    &mut network,
                );
            }
            Action::SetVisual {
                instance_id,
                placement,
            } => {
                runtime.submit_selected(
                    PlaybackCommand::SetVisual {
                        instance_id,
                        placement,
                    },
                    &mut network,
                );
            }
            Action::Stop(instance_id) => {
                runtime.submit_selected(PlaybackCommand::Stop { instance_id }, &mut network);
            }
            Action::StopAll => runtime.request_stop_all(&mut network),
            Action::ToggleHost => {
                if runtime.mode == Mode::Controller {
                    runtime.status = "Controller mode cannot start a local host".into();
                    continue;
                }
                if runtime.catalog.clips.is_empty() {
                    runtime.status = "Load a valid catalog before starting the local host".into();
                    continue;
                }
                runtime.enabled = !runtime.enabled;
                if !runtime.enabled {
                    runtime.submit(PlaybackCommand::StopAll, &mut network);
                    network
                        .actions
                        .push_back(crate::network_app::NetworkAction::DisableHost);
                }
                runtime.status = if runtime.enabled {
                    "This Mac | host running"
                } else {
                    "This Mac | host stopped"
                }
                .into();
            }
            Action::Hide => {
                if runtime.terminal_active || reopen.0 {
                    primary.visible = false;
                } else {
                    runtime.status =
                        "Keep panel open: neither Dock reopen nor terminal controls are available"
                            .into();
                }
            }
            Action::Show => {
                primary.visible = true;
                primary.focused = true;
            }
            Action::Quit => {
                runtime.quitting = true;
                runtime.enabled = false;
                runtime.submit(PlaybackCommand::StopAll, &mut network);
                network
                    .actions
                    .push_back(crate::network_app::NetworkAction::DisableHost);
                network
                    .actions
                    .push_back(crate::network_app::NetworkAction::Disconnect);
            }
            Action::Destination(destination) => {
                runtime.destination = destination;
                runtime.status = if destination == Destination::Local {
                    "Destination: This Mac"
                } else {
                    "Destination: Remote host | unavailable; no commands sent"
                }
                .into();
            }
            Action::SelectDisplay(display) => {
                if display.is_some_and(|index| index >= monitors.iter().count()) {
                    runtime.status = "Selected display is no longer available".into();
                    continue;
                }
                runtime.display = display;
                controls.0.push(display.map_or(
                    crate::overlay::Control::PrimaryDisplay,
                    crate::overlay::Control::Display,
                ));
            }
            Action::LoadPack(path) => {
                if runtime.loading {
                    runtime.status = "Pack load already in progress".into();
                    continue;
                }
                runtime.submit(PlaybackCommand::StopAll, &mut network);
                network
                    .actions
                    .push_back(crate::network_app::NetworkAction::DisableHost);
                runtime.enabled = false;
                runtime.loading = true;
                runtime.status = "Loading pack; playback stopped and remote access revoked".into();
                runtime.pack_input = path.clone();
                let tx = runtime.load_tx.clone();
                let proxy = (**proxy).clone();
                std::thread::spawn(move || {
                    let result = Config::pack_source(&path).and_then(|source| {
                        CatalogCache::new(
                            crate::preferences::Preferences::cache_dir().join("packs"),
                        )
                        .and_then(|cache| cache.prepare(&source))
                        .map_err(|e| e.to_string())
                    });
                    let _ = tx.send(result);
                    let _ = proxy.send_event(bevy::winit::WinitUserEvent::WakeUp);
                });
            }
        }
    }
    // Publish a replacement only after the old host has released every worker.
    if runtime.loading && runtime.workers.is_empty() {
        let result = runtime.load_rx.lock().unwrap().try_recv().ok();
        if let Some(result) = result {
            runtime.loading = false;
            match result {
                Ok(catalog) => {
                    runtime.host = Host::new(Runtime::session_id(), catalog.clone());
                    runtime.snapshot = runtime.host.snapshot();
                    runtime.snapshot_dirty = true;
                    runtime.catalog = catalog;
                    runtime.errors.clear();
                    runtime.enabled = runtime.mode != Mode::Controller;
                    runtime.status = "Pack loaded; catalog pinned for this session".into();
                }
                Err(message) => {
                    runtime.status = format!("Pack load failed: {message}");
                }
            }
        }
    }
}

/// Translate host effects to decoder work and acknowledge backend lifecycle transitions.
#[expect(
    clippy::too_many_arguments,
    reason = "The bridge reads host, media and render resources in one serialized update"
)]
pub(crate) fn media_bridge(
    mut commands: Commands,
    mut runtime: ResMut<Runtime>,
    mut images: ResMut<Assets<Image>>,
    mut sprites: Query<(&mut Sprite, &mut Transform)>,
    window: Single<&Window, With<OverlayWindow>>,
    mut overlay: ResMut<PlaybackOverlay>,
    mut exit: MessageWriter<AppExit>,
    mut network: ResMut<crate::network_app::NetworkState>,
) {
    let mut callbacks = Vec::new();
    let mut remove = Vec::new();
    for (id, playback) in &mut runtime.workers {
        for event in playback.worker.drain_events() {
            if event.generation != playback.token.generation {
                continue;
            }
            let kind = match event.kind {
                MediaEventKind::Prepared { .. } => BackendEventKind::Prepared,
                MediaEventKind::Started => BackendEventKind::Started,
                MediaEventKind::Finished => {
                    remove.push(id.clone());
                    BackendEventKind::Finished
                }
                MediaEventKind::Stopped => {
                    remove.push(id.clone());
                    match playback.failure.take() {
                        Some(message) => BackendEventKind::Failed { message },
                        None => BackendEventKind::Stopped,
                    }
                }
                MediaEventKind::Failed { message } => {
                    remove.push(id.clone());
                    BackendEventKind::Failed { message }
                }
            };
            callbacks.push(BackendEvent {
                token: playback.token.clone(),
                event: kind,
            });
        }
        if let Some(frame) = playback.worker.latest_frame() {
            if frame.generation != playback.token.generation {
                continue;
            }
            playback.dimensions = UVec2::new(frame.width, frame.height);
            if let Some(handle) = &playback.image {
                if let Some(mut existing) = images.get_mut(handle) {
                    let size = Extent3d {
                        width: frame.width,
                        height: frame.height,
                        depth_or_array_layers: 1,
                    };
                    if existing.texture_descriptor.size != size {
                        existing.resize(size);
                    }
                    let data = existing.data.get_or_insert_with(Vec::new);
                    data.clear();
                    data.extend_from_slice(&frame.rgba);
                }
            } else {
                let handle = images.add(Playback::frame_image(&frame));
                crate::diagnostics::first_frame_submitted(&playback.token.instance_id);
                let (size, position) = Playback::placement(
                    &playback.clip,
                    playback.placement,
                    Vec2::new(frame.width as f32, frame.height as f32),
                    Vec2::new(window.width(), window.height()),
                );
                let mut sprite = Sprite::from_image(handle.clone());
                sprite.custom_size = Some(size);
                sprite.rect = Playback::frame_crop(
                    &playback.clip,
                    playback.placement,
                    playback.dimensions.as_vec2(),
                    Vec2::new(window.width(), window.height()),
                );
                playback.sprite = Some(
                    commands
                        .spawn((
                            sprite,
                            RenderLayers::layer(OVERLAY_LAYER),
                            Transform::from_xyz(position.x, position.y, 0.0),
                        ))
                        .id(),
                );
                playback.image = Some(handle);
            }
        }
        if let Some(entity) = playback.sprite
            && let Ok((mut sprite, mut transform)) = sprites.get_mut(entity)
        {
            let (size, position) = Playback::placement(
                &playback.clip,
                playback.placement,
                playback.dimensions.as_vec2(),
                Vec2::new(window.width(), window.height()),
            );
            sprite.custom_size = Some(size);
            sprite.rect = Playback::frame_crop(
                &playback.clip,
                playback.placement,
                playback.dimensions.as_vec2(),
                Vec2::new(window.width(), window.height()),
            );
            *transform = Transform::from_xyz(position.x, position.y, 0.0);
        }
    }
    let callbacks_seen = !callbacks.is_empty();
    for callback in callbacks {
        runtime.host.backend_event(callback);
    }
    for id in remove {
        if let Some(playback) = runtime.workers.remove(&id) {
            if let Some(entity) = playback.sprite {
                commands.entity(entity).despawn();
            }
            if let Some(image) = playback.image {
                images.remove(image.id());
            }
        }
    }
    // Bounded policy effects; native decoder control remains in MediaPlugin.
    for _ in 0..3 {
        let effects = runtime.host.drain_effects();
        if effects.is_empty() {
            break;
        }
        for effect in effects {
            match effect {
                BackendEffect::Prepare {
                    token,
                    clip,
                    placement,
                } => {
                    let worker = MediaWorker::spawn();
                    let result = worker.prepare(MediaSpec {
                        path: clip.asset_path.clone(),
                        generation: token.generation,
                        max_duration: Duration::from_millis(clip.metadata.playback.max_duration_ms),
                        looping: clip.metadata.playback.looping,
                        volume: 1.0,
                    });
                    match result {
                        Ok(()) => {
                            runtime.workers.insert(
                                token.instance_id.clone(),
                                Playback {
                                    token,
                                    worker,
                                    clip: *clip,
                                    placement,
                                    image: None,
                                    sprite: None,
                                    failure: None,
                                    dimensions: UVec2::ZERO,
                                },
                            );
                        }
                        Err(message) => runtime.host.backend_event(BackendEvent {
                            token,
                            event: BackendEventKind::Failed { message },
                        }),
                    }
                }
                BackendEffect::SetVisual { token, placement } => {
                    if let Some(playback) = runtime.workers.get_mut(&token.instance_id) {
                        playback.set_visual(&token, placement);
                    }
                }
                BackendEffect::Start { token } => {
                    if let Some(playback) = runtime.workers.get_mut(&token.instance_id)
                        && let Err(message) = playback.worker.start(token.generation)
                    {
                        playback.failure = Some(message);
                        playback.worker.stop(token.generation);
                    }
                }
                BackendEffect::Stop { token } => {
                    if let Some(playback) = runtime.workers.get_mut(&token.instance_id) {
                        playback.worker.stop(token.generation);
                        if let Some(entity) = playback.sprite.take() {
                            commands.entity(entity).despawn();
                        }
                        if let Some(image) = playback.image.take() {
                            images.remove(image.id());
                        }
                    } else {
                        runtime.host.backend_event(BackendEvent {
                            token,
                            event: BackendEventKind::Stopped,
                        });
                    }
                }
            }
        }
    }
    let events = runtime.host.drain_events();
    crate::diagnostics::observe_events(&events);
    if runtime.snapshot_dirty || callbacks_seen || !events.is_empty() {
        runtime.snapshot = runtime.host.snapshot();
        runtime.snapshot_dirty = false;
        network.publish_local(runtime.snapshot.clone(), &events);
    }
    for event in events {
        if matches!(event.event, crate::commands::EventKind::Stopped)
            && runtime
                .snapshot
                .instances
                .iter()
                .all(|instance| instance.state.is_terminal())
        {
            runtime.status = "Playback stopped".into();
        }
        if let crate::commands::EventKind::Failed { message } = event.event {
            let clip_id = runtime
                .snapshot
                .instances
                .iter()
                .find(|i| Some(&i.instance_id) == event.instance_id.as_ref())
                .map(|instance| instance.clip_id.clone());
            if let Some(clip_id) = clip_id {
                runtime.errors.insert(clip_id, message.clone());
            }
            runtime.status = format!("Playback failed: {message}");
        }
    }
    if runtime.status == "Stop all requested; waiting for playback resources to stop"
        && runtime
            .snapshot
            .instances
            .iter()
            .all(|instance| instance.state.is_terminal())
    {
        runtime.status = "No active playback".into();
    }
    overlay.visible = runtime.workers.values().any(|p| p.sprite.is_some());
    overlay.busy = runtime.busy() || crate::media::MediaService::needs_updates();
    if runtime.quitting && runtime.workers.is_empty() {
        exit.write(AppExit::Success);
    }
}

impl Playback {
    /// Convert a tightly packed decoder frame into a straight-alpha sRGB Bevy texture.
    fn frame_image(frame: &crate::media::DecodedFrame) -> Image {
        // Decoder contract is tightly packed straight-alpha sRGB RGBA. A handle and
        // texture identity are reused per occurrence, with one latest frame retained.
        Image::new(
            Extent3d {
                width: frame.width,
                height: frame.height,
                depth_or_array_layers: 1,
            },
            TextureDimension::D2,
            frame.rgba.to_vec(),
            TextureFormat::Rgba8UnormSrgb,
            RenderAssetUsages::default(),
        )
    }

    /// Apply a live override only to the worker generation that the host authorized.
    fn set_visual(&mut self, token: &BackendToken, placement: Option<VisualPlacement>) {
        if self.token.instance_id == token.instance_id && self.token.generation == token.generation
        {
            self.placement = placement;
        }
    }

    /// Retain exact catalog sizing when no override is active, sharing the box with crop math.
    fn fit_target(clip: &Clip, placement: Option<VisualPlacement>, viewport: Vec2) -> Vec2 {
        let fraction = placement.map_or_else(
            || {
                clip.metadata
                    .visual
                    .as_ref()
                    .map_or(0.3, |visual| visual.width_fraction as f32)
            },
            |placement| f32::from(placement.width_bps) / 10_000.0,
        );
        Vec2::splat((viewport.x * fraction).min(viewport.y))
    }

    /// Fit a clip within the current overlay monitor and keep normalized positions on screen.
    fn placement(
        clip: &Clip,
        placement: Option<VisualPlacement>,
        frame: Vec2,
        viewport: Vec2,
    ) -> (Vec2, Vec2) {
        let target = Self::fit_target(clip, placement, viewport);
        let placement = placement.or_else(|| Host::default_placement(clip));
        let fit = clip.metadata.visual.as_ref().map(|v| &v.fit);
        let ratios = target / frame;
        let scale = if matches!(fit, Some(Fit::Cover)) {
            ratios.max_element()
        } else {
            ratios.min_element()
        };
        let size = if matches!(fit, Some(Fit::Cover)) {
            target
        } else {
            frame * scale
        };
        let travel = viewport - size;
        let normalized = placement.map_or(Vec2::splat(0.5), |p| {
            Vec2::new(f32::from(p.x_bps), f32::from(p.y_bps)) / 10_000.0
        });
        let position = travel * (normalized - Vec2::splat(0.5)) * Vec2::new(1.0, -1.0);
        (size, position)
    }

    /// Compute a centered source rectangle using the same effective width as placement.
    fn frame_crop(
        clip: &Clip,
        placement: Option<VisualPlacement>,
        frame: Vec2,
        viewport: Vec2,
    ) -> Option<Rect> {
        let visual = clip.metadata.visual.as_ref()?;
        if !matches!(visual.fit, Fit::Cover) {
            return None;
        }
        let target = Self::fit_target(clip, placement, viewport);
        let scale = (target / frame).max_element();
        Some(Rect::from_center_size(frame * 0.5, target / scale))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::catalog::Anchor;

    #[test]
    fn bundled_reactions_are_complete_and_cover_all_three_media_kinds() {
        let catalog = crate::catalog::load_local(&Config::bundled_pack()).unwrap();
        assert_eq!(catalog.clips.len(), 31);
        let kinds: std::collections::BTreeSet<_> = catalog
            .clips
            .iter()
            .map(|clip| clip.asset_path.extension().unwrap().to_str().unwrap())
            .collect();
        assert_eq!(kinds, ["gif", "mp3", "mp4"].into_iter().collect());
        assert!(
            catalog
                .clips
                .iter()
                .all(|clip| clip.metadata.tags.len() >= 2)
        );
    }

    #[test]
    fn modes_and_explicit_cli_targets_do_not_take_credentials() {
        let config = Config::parse(
            [
                "--mode",
                "host",
                "--pack",
                "/chosen/pack",
                "--display",
                "2",
                "--listen-tailscale",
                "100.64.0.10:48732",
            ]
            .map(str::to_owned),
        )
        .unwrap();
        assert_eq!(config.mode, Mode::Host);
        assert_eq!(config.pack, PathBuf::from("/chosen/pack"));
        assert_eq!(config.display, Some(2));
        assert_eq!(config.listen.as_deref(), Some("100.64.0.10:48732"));
        for args in [
            vec!["--mode", "unknown"],
            vec!["--display", "-1"],
            vec!["--credential", "secret"],
        ] {
            assert!(Config::parse(args.into_iter().map(str::to_owned)).is_err());
        }
    }
    #[test]
    fn controller_starts_without_reading_a_local_pack() {
        let config = Config::parse(
            [
                "--mode",
                "controller",
                "--pack",
                "/does-not-exist",
                "--connect",
                "ws://100.64.0.10:48732/control",
            ]
            .map(str::to_owned),
        )
        .unwrap();
        let runtime = Runtime::new(&config).unwrap();
        assert!(runtime.catalog.clips.is_empty());
        assert!(!runtime.enabled);
        assert_eq!(runtime.destination, Destination::Remote);
        assert!(runtime.workers.is_empty());
    }
    #[test]
    fn git_pack_selection_preserves_explicit_reference() {
        assert!(matches!(
            Config::pack_source("https://github.com/ramate-io/durop.git#main:sample-packs/reactions").unwrap(),
            PackSource::HttpsGitDirectory { url, reference, directory }
                if url == "https://github.com/ramate-io/durop.git" && reference == "main"
                    && directory == std::path::Path::new("sample-packs/reactions")
        ));
        assert!(
            matches!(Config::pack_source("https://example.com/pack.git#release").unwrap(),PackSource::HttpsGit{url,reference}if url=="https://example.com/pack.git"&&reference=="release")
        );
        assert!(
            matches!(Config::pack_source("git+file:///tmp/pack#main").unwrap(),PackSource::LocalGit{path,reference}if path==std::path::Path::new("/tmp/pack")&&reference=="main")
        );
        assert!(Config::pack_source("git+file:///tmp/pack").is_err());
    }
    #[test]
    fn placement_bounds_width_and_cover_crops_without_stretching() {
        let catalog =
            crate::catalog::load_local(&resource_root().join("sample-packs/podcast-pack")).unwrap();
        let mut clip = catalog
            .clips
            .iter()
            .find(|c| c.metadata.visual.is_some())
            .unwrap()
            .clone();
        let visual = clip.metadata.visual.as_mut().unwrap();
        visual.width_fraction = 0.25;
        visual.anchor = Anchor::BottomRight;
        visual.fit = Fit::Contain;
        let (size, position) =
            Playback::placement(&clip, None, Vec2::new(200., 100.), Vec2::new(800., 600.));
        assert_eq!(size, Vec2::new(200., 100.));
        assert_eq!(position, Vec2::new(300., -250.));
        assert!(
            Playback::frame_crop(&clip, None, Vec2::new(200., 100.), Vec2::new(800., 600.))
                .is_none()
        );
        clip.metadata.visual.as_mut().unwrap().fit = Fit::Cover;
        let (size, position) =
            Playback::placement(&clip, None, Vec2::new(200., 100.), Vec2::new(800., 600.));
        assert_eq!(size, Vec2::splat(200.));
        assert_eq!(position, Vec2::new(300., -200.));
        let crop = Playback::frame_crop(&clip, None, Vec2::new(200., 100.), Vec2::new(800., 600.))
            .unwrap();
        assert_eq!(crop.min, Vec2::new(50., 0.));
        assert_eq!(crop.max, Vec2::new(150., 100.));
    }

    #[test]
    fn placement_overrides_preserve_aspect_and_bounds_on_each_monitor_shape() {
        let catalog =
            crate::catalog::load_local(&resource_root().join("sample-packs/podcast-pack")).unwrap();
        let mut clip = catalog
            .clips
            .iter()
            .find(|clip| clip.metadata.visual.is_some())
            .unwrap()
            .clone();
        for fit in [Fit::Contain, Fit::Cover] {
            clip.metadata.visual.as_mut().unwrap().fit = fit.clone();
            for viewport in [Vec2::new(800., 600.), Vec2::new(600., 1000.)] {
                for frame in [
                    Vec2::new(1920., 1080.),
                    Vec2::new(100., 300.),
                    Vec2::splat(256.),
                ] {
                    for width_bps in [1, 2500, 10_000] {
                        for (x_bps, y_bps) in [
                            (0, 0),
                            (10_000, 0),
                            (0, 10_000),
                            (10_000, 10_000),
                            (5000, 5000),
                        ] {
                            let placement = Some(VisualPlacement {
                                width_bps,
                                x_bps,
                                y_bps,
                            });
                            let (size, center) =
                                Playback::placement(&clip, placement, frame, viewport);
                            assert!(size.x <= viewport.x + 0.001 && size.y <= viewport.y + 0.001);
                            assert!(center.x.abs() + size.x * 0.5 <= viewport.x * 0.5 + 0.001);
                            assert!(center.y.abs() + size.y * 0.5 <= viewport.y * 0.5 + 0.001);
                            let source = Playback::frame_crop(&clip, placement, frame, viewport)
                                .map_or(frame, |rect| rect.size());
                            assert!((size.x / size.y - source.x / source.y).abs() < 0.0001);
                            if x_bps == 0 {
                                assert!((center.x - size.x * 0.5 + viewport.x * 0.5).abs() < 0.001);
                            }
                            if y_bps == 0 {
                                assert!((center.y + size.y * 0.5 - viewport.y * 0.5).abs() < 0.001);
                            }
                            if x_bps == 10_000 {
                                assert!((center.x + size.x * 0.5 - viewport.x * 0.5).abs() < 0.001);
                            }
                            if y_bps == 10_000 {
                                assert!((center.y - size.y * 0.5 + viewport.y * 0.5).abs() < 0.001);
                            }
                            if x_bps == 5000 && y_bps == 5000 {
                                assert_eq!(center, Vec2::ZERO);
                            }
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn live_placement_preserves_worker_and_rejects_stale_generation() {
        let catalog =
            crate::catalog::load_local(&resource_root().join("sample-packs/podcast-pack")).unwrap();
        let clip = catalog
            .clips
            .iter()
            .find(|clip| clip.metadata.visual.is_some())
            .unwrap()
            .clone();
        let token = BackendToken {
            instance_id: "visual".into(),
            generation: 7,
        };
        let mut playback = Playback {
            token: token.clone(),
            worker: MediaWorker::spawn(),
            clip,
            placement: None,
            image: None,
            sprite: None,
            failure: None,
            dimensions: UVec2::new(200, 100),
        };
        let worker = &playback.worker as *const MediaWorker;
        let placement = VisualPlacement {
            width_bps: 4000,
            x_bps: 0,
            y_bps: 10_000,
        };
        let mut stale = token.clone();
        stale.generation -= 1;
        playback.set_visual(&stale, Some(placement));
        assert_eq!(playback.placement, None);
        playback.set_visual(&token, Some(placement));
        assert_eq!(playback.placement, Some(placement));
        assert_eq!(playback.dimensions, UVec2::new(200, 100));
        assert_eq!(&playback.worker as *const MediaWorker, worker);
        assert!(playback.worker.drain_events().is_empty());
        playback.set_visual(&token, None);
        assert_eq!(playback.placement, None);
    }
}
