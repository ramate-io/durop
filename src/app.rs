//! Application modes and the real host/media bridge. No decoder work runs in UI code.
use crate::{
    catalog::{Anchor, Catalog, CatalogCache, Clip, Fit, PackSource},
    commands::{Envelope, PROTOCOL_VERSION, PlaybackCommand},
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Local,
    Host,
    Controller,
}
#[derive(Debug, Clone)]
pub struct Config {
    pub mode: Mode,
    pub pack: PathBuf,
    pub display: Option<usize>,
    pub connect: Option<String>,
    pub listen: Option<String>,
    pub diagnostics: Option<PathBuf>,
}
impl Config {
    pub fn parse(args: impl IntoIterator<Item = String>) -> Result<Self, String> {
        let preferences = crate::preferences::load();
        let mut config = Self {
            mode: Mode::Local,
            pack: crate::preferences::resolved_pack(
                &preferences.pack,
                &resource_root().join("sample-packs/podcast-pack"),
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
                "--mode" => config.mode=match value()?.as_str(){"local"=>Mode::Local,"host"=>Mode::Host,"controller"=>Mode::Controller,_=>return Err("mode must be local, host or controller".into())},
                "--pack"=>config.pack=value()?.into(),
                "--display"=>config.display=Some(value()?.parse().map_err(|_|"display must be a nonnegative index")?),
                "--connect"=>config.connect=Some(value()?),
                "--listen-tailscale"=>config.listen=Some(value()?),
                "--diagnostics"=>config.diagnostics=Some(value()?.into()),
                "--help"|"-h"=>return Err("durop [--mode local|host|controller] [--pack DIRECTORY] [--display INDEX] [--connect ws://HOST:48732/control] [--listen-tailscale IP:PORT] [--diagnostics NEW_JSONL_FILE]".into()),
                _=>return Err(format!("unknown option {arg}")),
            }
        }
        Ok(config)
    }
}
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
        Ok(PackSource::HttpsGit {
            url: url.into(),
            reference: reference.into(),
        })
    } else {
        Ok(PackSource::LocalDirectory(value.into()))
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Destination {
    Local,
    Remote,
}
#[derive(Debug, Clone)]
pub enum Action {
    Play(String),
    Stop(String),
    StopAll,
    ToggleHost,
    Hide,
    Show,
    Quit,
    LoadPack(String),
    Destination(Destination),
    NextDisplay,
}
#[derive(Resource, Default)]
pub struct Actions(pub VecDeque<Action>);
#[derive(Resource, Default)]
pub struct ReopenSupport(pub bool);
struct Playback {
    token: BackendToken,
    worker: MediaWorker,
    clip: Clip,
    image: Option<Handle<Image>>,
    sprite: Option<Entity>,
    failure: Option<String>,
    dimensions: UVec2,
}
#[derive(Resource)]
pub struct Runtime {
    pub host: Host,
    pub snapshot: crate::commands::HostSnapshot,
    pub catalog: Arc<Catalog>,
    pub mode: Mode,
    pub destination: Destination,
    pub enabled: bool,
    pub status: String,
    pub errors: BTreeMap<String, String>,
    pub pack_input: String,
    pub loading: bool,
    pub terminal_active: bool,
    pub display: Option<usize>,
    workers: BTreeMap<String, Playback>,
    next_command: u64,
    quitting: bool,
    snapshot_dirty: bool,
    load_rx: Mutex<mpsc::Receiver<Result<Arc<Catalog>, String>>>,
    load_tx: mpsc::Sender<Result<Arc<Catalog>, String>>,
}
fn private_root() -> PathBuf {
    crate::preferences::cache_dir()
}
fn session_id() -> String {
    // UUID randomness comes from the OS, never from controller input.
    bevy::asset::uuid::Uuid::new_v4().to_string()
}
impl Runtime {
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
            pack_source(&config.pack.to_string_lossy()).and_then(|source| {
                CatalogCache::new(private_root().join("packs"))
                    .and_then(|cache| cache.prepare(&source))
                    .map_err(|e| e.to_string())
            })
        };
        let failure = result.as_ref().err().cloned();
        let catalog = result.unwrap_or_else(|_| empty());
        let (load_tx, load_rx) = mpsc::channel();
        let host = Host::new(session_id(), catalog.clone());
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
    pub fn busy(&self) -> bool {
        !self.workers.is_empty() || self.quitting
    }
}

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
                if runtime.destination != Destination::Local {
                    network
                        .actions
                        .push_back(crate::network_app::NetworkAction::RemoteCommand(
                            PlaybackCommand::Play { clip_id },
                        ));
                } else if !runtime.enabled {
                    runtime.status = "Start the local host before playing".into();
                } else {
                    runtime.submit(PlaybackCommand::Play { clip_id }, &mut network);
                }
            }
            Action::Stop(instance_id) => {
                if runtime.destination == Destination::Remote {
                    network
                        .actions
                        .push_back(crate::network_app::NetworkAction::RemoteCommand(
                            PlaybackCommand::Stop { instance_id },
                        ));
                } else {
                    runtime.submit(PlaybackCommand::Stop { instance_id }, &mut network);
                }
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
            Action::NextDisplay => {
                let count = monitors.iter().count();
                if count > 0 {
                    let next = runtime.display.map_or(0, |i| (i + 1) % count);
                    runtime.display = Some(next);
                    controls.0.push(crate::overlay::Control::Display(next));
                }
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
                    let result = pack_source(&path).and_then(|source| {
                        CatalogCache::new(private_root().join("packs"))
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
                    runtime.host = Host::new(session_id(), catalog.clone());
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
                let handle = images.add(frame_image(&frame));
                crate::diagnostics::first_frame_submitted(&playback.token.instance_id);
                let (size, position) = placement(
                    &playback.clip,
                    Vec2::new(frame.width as f32, frame.height as f32),
                    Vec2::new(window.width(), window.height()),
                );
                let mut sprite = Sprite::from_image(handle.clone());
                sprite.custom_size = Some(size);
                sprite.rect = frame_crop(
                    &playback.clip,
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
            let (size, position) = placement(
                &playback.clip,
                playback.dimensions.as_vec2(),
                Vec2::new(window.width(), window.height()),
            );
            sprite.custom_size = Some(size);
            sprite.rect = frame_crop(
                &playback.clip,
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
                BackendEffect::Prepare { token, clip } => {
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
    overlay.busy = runtime.busy() || crate::media::needs_updates();
    if runtime.quitting && runtime.workers.is_empty() {
        exit.write(AppExit::Success);
    }
}
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
pub fn placement(clip: &Clip, frame: Vec2, viewport: Vec2) -> (Vec2, Vec2) {
    let fraction = clip
        .metadata
        .visual
        .as_ref()
        .map_or(0.3, |v| v.width_fraction as f32);
    let target = Vec2::splat((viewport.x * fraction).min(viewport.y));
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
    let offset = (viewport - size) * 0.5;
    let position = match clip.metadata.visual.as_ref().map(|v| &v.anchor) {
        Some(Anchor::TopLeft) => Vec2::new(-offset.x, offset.y),
        Some(Anchor::TopRight) => offset,
        Some(Anchor::BottomLeft) => -offset,
        Some(Anchor::BottomRight) => Vec2::new(offset.x, -offset.y),
        _ => Vec2::ZERO,
    };
    (size, position)
}
fn frame_crop(clip: &Clip, frame: Vec2, viewport: Vec2) -> Option<Rect> {
    let visual = clip.metadata.visual.as_ref()?;
    if !matches!(visual.fit, Fit::Cover) {
        return None;
    }
    let target = Vec2::splat((viewport.x * visual.width_fraction as f32).min(viewport.y));
    let scale = (target / frame).max_element();
    Some(Rect::from_center_size(frame * 0.5, target / scale))
}

#[cfg(test)]
mod tests {
    use super::*;
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
        assert!(
            matches!(pack_source("https://example.com/pack.git#release").unwrap(),PackSource::HttpsGit{url,reference}if url=="https://example.com/pack.git"&&reference=="release")
        );
        assert!(
            matches!(pack_source("git+file:///tmp/pack#main").unwrap(),PackSource::LocalGit{path,reference}if path==std::path::Path::new("/tmp/pack")&&reference=="main")
        );
        assert!(pack_source("git+file:///tmp/pack").is_err());
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
        let (size, position) = placement(&clip, Vec2::new(200., 100.), Vec2::new(800., 600.));
        assert_eq!(size, Vec2::new(200., 100.));
        assert_eq!(position, Vec2::new(300., -250.));
        assert!(frame_crop(&clip, Vec2::new(200., 100.), Vec2::new(800., 600.)).is_none());
        clip.metadata.visual.as_mut().unwrap().fit = Fit::Cover;
        let (size, position) = placement(&clip, Vec2::new(200., 100.), Vec2::new(800., 600.));
        assert_eq!(size, Vec2::splat(200.));
        assert_eq!(position, Vec2::new(300., -200.));
        let crop = frame_crop(&clip, Vec2::new(200., 100.), Vec2::new(800., 600.)).unwrap();
        assert_eq!(crop.min, Vec2::new(50., 0.));
        assert_eq!(crop.max, Vec2::new(150., 100.));
    }
}
