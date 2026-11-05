//! Single-display overlay proof. No playback or network policy lives here.
use bevy::{
    camera::{RenderTarget, visibility::RenderLayers},
    ecs::system::NonSendMarker,
    prelude::*,
    window::{
        CompositeAlphaMode, CursorOptions, Monitor, PrimaryMonitor, PrimaryWindow,
        WindowCloseRequested, WindowLevel, WindowPosition, WindowRef,
    },
    winit::{UpdateMode, WinitMonitors, WinitSettings},
};
use std::{
    sync::{Mutex, mpsc::Receiver},
    time::{Duration, Instant},
};

#[cfg(target_os = "macos")]
#[path = "overlay_macos.rs"]
mod macos;

/// Render layer reserved for the control panel.
pub const PANEL_LAYER: usize = 1;

/// Render layer reserved for transparent playback and proof fixtures.
pub(crate) const OVERLAY_LAYER: usize = 2;

/// Summarize playback visibility and progress needs for the window adapter.
#[derive(Resource, Default)]
pub(crate) struct PlaybackOverlay {
    /// Whether any playback sprite requires a visible overlay.
    pub visible: bool,
    /// Whether playback or native callbacks need continued update ticks.
    pub busy: bool,
}

/// Mark the transparent click-through playback window.
#[derive(Component)]
pub struct OverlayWindow;

/// Mark the camera that renders only the overlay layer.
#[derive(Component)]
pub(crate) struct OverlayCamera;

/// Mark the built-in compositing proof sprite.
#[derive(Component)]
pub(crate) struct Fixture;

/// Mark a window whose platform-specific overlay policy has been installed.
#[derive(Component)]
pub(super) struct NativeReady;

/// Collect window/display intents for the next ordered update.
#[derive(Resource, Default)]
pub struct PendingControls(
    /// Intents drained in order by the overlay control system.
    pub Vec<Control>,
);

/// Receive optional proof controls without blocking the render thread.
#[derive(Resource)]
pub struct Controls(
    /// Protected receiver polled without blocking the render thread.
    pub Mutex<Receiver<Control>>,
);

/// Window and display commands independent of playback policy.
#[derive(Debug, PartialEq, Eq)]
pub enum Control {
    /// Start the timed compositing proof.
    Show,
    /// Show the proof until explicitly stopped.
    Hold,
    /// Hide the proof without changing host playback policy.
    Stop,
    /// Restore the panel window.
    ShowPanel,
    /// Hide the panel if a safe control path remains.
    HidePanel,
    /// Cycle to the next currently connected display.
    NextDisplay,
    /// Print the current display inventory.
    Displays,
    /// Select a display by its current index.
    Display(usize),
    /// Request proof-mode exit.
    Quit,
    /// Record loss of the stdin control path.
    TerminalClosed,
}

impl Control {
    /// Parse the exact proof-terminal grammar; reject unknown or malformed commands.
    pub fn parse(line: &str) -> Option<Self> {
        Some(match line.trim() {
            "show" => Self::Show,
            "hold" => Self::Hold,
            "stop" => Self::Stop,
            "panel" => Self::ShowPanel,
            "hide" => Self::HidePanel,
            "next" => Self::NextDisplay,
            "displays" => Self::Displays,
            "quit" => Self::Quit,
            s => {
                let index = s.strip_prefix("display ")?.parse::<usize>().ok()?;
                Self::Display(index)
            }
        })
    }
}

/// Track the selected monitor, proof animation and terminal availability.
#[derive(Resource, Default)]
pub(crate) struct OverlayState {
    /// Selected monitor entity, cleared when unplugged.
    target: Option<Entity>,
    /// Whether the compositing proof was requested.
    visible: bool,
    /// Start time of the optional proof animation.
    animation: Option<Instant>,
    /// Whether stdin can no longer restore a hidden panel.
    terminal_closed: bool,
}

/// Install display selection, transparent rendering and platform window policy.
pub struct OverlayPlugin;
impl Plugin for OverlayPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PendingControls>()
            .init_resource::<PlaybackOverlay>()
            .init_resource::<OverlayState>()
            .insert_resource(idle_settings())
            .add_systems(Startup, setup)
            .add_systems(PreUpdate, refresh_monitor_geometry)
            .add_systems(
                Update,
                (read_controls, apply_controls, sync_overlay).chain(),
            );
        #[cfg(target_os = "macos")]
        app.add_systems(PreUpdate, macos::configure);
        #[cfg(not(target_os = "macos"))]
        app.add_systems(PreUpdate, mark_ready);
    }
}

/// Sleep until input while the overlay has no animation or playback work.
fn idle_settings() -> WinitSettings {
    WinitSettings {
        focused_mode: UpdateMode::reactive_low_power(Duration::MAX),
        unfocused_mode: UpdateMode::reactive_low_power(Duration::MAX),
    }
}

/// Create a hidden nonactivating overlay and its isolated camera/fixture.
fn setup(mut commands: Commands, assets: Res<AssetServer>) {
    let window = commands
        .spawn((
            OverlayWindow,
            Window {
                title: "Durop overlay".into(),
                transparent: true,
                decorations: false,
                focused: false,
                visible: false,
                resizable: false,
                has_shadow: false,
                window_level: WindowLevel::AlwaysOnTop,
                #[cfg(target_os = "macos")]
                composite_alpha_mode: CompositeAlphaMode::PostMultiplied,
                #[cfg(not(target_os = "macos"))]
                composite_alpha_mode: CompositeAlphaMode::PreMultiplied,
                ..default()
            },
            CursorOptions {
                hit_test: false,
                ..default()
            },
        ))
        .id();
    commands.spawn((
        Camera2d,
        Camera {
            clear_color: Color::NONE.into(),
            is_active: false,
            ..default()
        },
        RenderTarget::Window(WindowRef::Entity(window)),
        RenderLayers::layer(OVERLAY_LAYER),
        OverlayCamera,
    ));
    commands.spawn((
        Sprite::from_image(assets.load("overlay-proof.png")),
        RenderLayers::layer(OVERLAY_LAYER),
        Fixture,
    ));
}

/// Complete the portable window setup when no AppKit policy is needed.
#[cfg(not(target_os = "macos"))]
#[expect(
    clippy::type_complexity,
    reason = "Bevy query filters document the native readiness boundary"
)]
fn mark_ready(
    mut commands: Commands,
    windows: Query<
        Entity,
        (
            With<OverlayWindow>,
            Without<NativeReady>,
            With<bevy::window::RawHandleWrapper>,
        ),
    >,
) {
    for window in &windows {
        commands.entity(window).insert(NativeReady);
    }
}

// Bevy 0.19 creates Monitor components once, but does not refresh their
// geometry for an existing native monitor. Refresh from the same backend handles.
/// Refresh physical monitor bounds after display configuration changes.
fn refresh_monitor_geometry(
    backend: Res<WinitMonitors>,
    mut monitors: Query<(Entity, &mut Monitor)>,
    _main_thread: NonSendMarker,
) {
    for (entity, mut monitor) in &mut monitors {
        if let Some(native) = backend.find_entity(entity) {
            let size = native.size();
            let position = native.position();
            if monitor.physical_width != size.width
                || monitor.physical_height != size.height
                || monitor.physical_position != IVec2::new(position.x, position.y)
                || monitor.scale_factor != native.scale_factor()
            {
                monitor.physical_width = size.width;
                monitor.physical_height = size.height;
                monitor.physical_position = IVec2::new(position.x, position.y);
                monitor.scale_factor = native.scale_factor();
            }
        }
    }
}

/// Drain available terminal controls into the ordered application queue.
fn read_controls(controls: Option<Res<Controls>>, mut pending: ResMut<PendingControls>) {
    if let Some(controls) = controls {
        pending.0.extend(
            controls
                .0
                .lock()
                .expect("local control receiver poisoned")
                .try_iter(),
        );
    }
}

/// Apply panel visibility and monitor choices without hiding the only available stop control.
#[expect(
    clippy::too_many_arguments,
    reason = "Bevy declares independent window and control resources as system parameters"
)]
pub(crate) fn apply_controls(
    mut pending: ResMut<PendingControls>,
    mut state: ResMut<OverlayState>,
    mut panel: Single<&mut Window, (With<PrimaryWindow>, Without<OverlayWindow>)>,
    monitors: Query<(Entity, &Monitor)>,
    mut close: MessageReader<WindowCloseRequested>,
    primary: Single<Entity, With<PrimaryWindow>>,
    mut exit: MessageWriter<AppExit>,
    reopen: Option<Res<crate::app::ReopenSupport>>,
) {
    let mut displays: Vec<_> = monitors.iter().collect();
    displays.sort_by_key(|(entity, _)| entity.to_bits());
    for request in close.read() {
        state.visible = false;
        state.animation = None;
        if request.window == *primary {
            panel.visible =
                state.terminal_closed && !reopen.as_ref().is_some_and(|support| support.0);
        }
    }
    for control in pending.0.drain(..) {
        match control {
            Control::Show | Control::Hold => {
                state.visible = true;
                state.animation = (control == Control::Show).then(Instant::now);
            }
            Control::Stop => {
                state.visible = false;
                state.animation = None;
            }
            Control::ShowPanel => {
                panel.visible = true;
                panel.focused = true;
            }
            Control::HidePanel => {
                if state.terminal_closed {
                    eprintln!("Keep the panel open: terminal safety controls are unavailable.");
                } else {
                    panel.visible = false;
                }
            }
            Control::TerminalClosed => {
                state.terminal_closed = true;
                panel.visible = true;
            }
            Control::Quit => {
                exit.write(AppExit::Success);
            }
            Control::Displays => {
                for (i, (entity, monitor)) in displays.iter().enumerate() {
                    eprintln!(
                        "display {i}: {:?} {}x{} scale {} {:?}{}",
                        monitor.name,
                        monitor.physical_width,
                        monitor.physical_height,
                        monitor.scale_factor,
                        monitor.physical_position,
                        if Some(*entity) == state.target {
                            " [selected]"
                        } else {
                            ""
                        }
                    );
                }
            }
            Control::Display(index) => {
                if let Some((entity, _)) = displays.get(index) {
                    state.target = Some(*entity);
                } else {
                    eprintln!("Display index {index} unavailable; use displays.");
                }
            }
            Control::NextDisplay => {
                if !displays.is_empty() {
                    let next = displays
                        .iter()
                        .position(|(entity, _)| Some(*entity) == state.target)
                        .map_or(0, |i| (i + 1) % displays.len());
                    state.target = Some(displays[next].0);
                }
            }
        }
    }
}

/// Reconcile playback/proof visibility, monitor changes and event-loop wake behavior.
#[expect(
    clippy::too_many_arguments,
    reason = "Overlay synchronization reads native readiness and display/render resources together"
)]
pub(crate) fn sync_overlay(
    mut state: ResMut<OverlayState>,
    monitors: Query<(Entity, &Monitor, Has<PrimaryMonitor>)>,
    mut window: Single<(&mut Window, Has<NativeReady>), With<OverlayWindow>>,
    mut camera: Single<&mut Camera, With<OverlayCamera>>,
    mut fixture: Single<(&Sprite, &mut Transform, &mut Visibility), With<Fixture>>,
    playback: Option<Res<PlaybackOverlay>>,
    assets: Res<AssetServer>,
    mut settings: ResMut<WinitSettings>,
) {
    if state.target.is_none() {
        state.target = monitors
            .iter()
            .find(|(_, _, primary)| *primary)
            .or_else(|| monitors.iter().next())
            .map(|(entity, _, _)| entity);
    }
    let selected = state.target.and_then(|target| monitors.get(target).ok());
    if selected.is_none() {
        // Never silently move an active drop to a different display after unplugging.
        state.target = None;
        state.visible = false;
        state.animation = None;
    }
    if let Some((_, monitor, _)) = selected {
        let bounds = display_bounds(monitor.physical_size(), monitor.scale_factor);
        if let Some((size, scale)) = bounds {
            let position = WindowPosition::At(monitor.physical_position);
            if window.0.position != position {
                window.0.position = position;
            }
            if window.0.resolution.physical_width() != size.x
                || window.0.resolution.physical_height() != size.y
                || window.0.resolution.scale_factor_override() != Some(scale)
            {
                window.0.resolution.set_scale_factor_override(Some(scale));
                window.0.resolution.set_physical_resolution(size.x, size.y);
            }
        } else {
            state.visible = false;
            state.animation = None;
        }
    }
    if let Some(start) = state.animation {
        let elapsed = start.elapsed().as_secs_f32();
        if elapsed >= 5.0 {
            state.visible = false;
            state.animation = None;
        } else {
            fixture.1.scale = Vec3::splat(1.0 + 0.1 * (elapsed * 5.0).sin());
        }
    } else {
        fixture.1.scale = Vec3::ONE;
    }
    let playback_visible = playback.as_ref().is_some_and(|p| p.visible);
    *fixture.2 = if state.visible {
        Visibility::Visible
    } else {
        Visibility::Hidden
    };
    let visible = (state.visible || playback_visible) && selected.is_some() && window.1;
    if window.0.visible != visible {
        window.0.visible = visible;
    }
    if camera.is_active != visible {
        camera.is_active = visible;
    }
    let loading = matches!(
        assets.load_state(&fixture.0.image),
        bevy::asset::LoadState::Loading | bevy::asset::LoadState::NotLoaded
    );
    let active = loading
        || (visible && state.animation.is_some())
        || playback.as_ref().is_some_and(|p| p.busy);
    let mode = if active {
        WinitSettings::continuous()
    } else {
        idle_settings()
    };
    if settings.focused_mode != mode.focused_mode || settings.unfocused_mode != mode.unfocused_mode
    {
        *settings = mode;
    }
}

/// Reject invalid monitor scale/size and preserve physical pixels in logical geometry.
fn display_bounds(size: UVec2, scale: f64) -> Option<(UVec2, f32)> {
    (size.x > 0 && size.y > 0 && scale.is_finite() && scale > 0.0 && scale <= f32::MAX as f64)
        .then_some((size, scale as f32))
}
#[cfg(test)]
mod tests {
    use super::*;
    fn control_app() -> (App, Entity) {
        let mut app = App::new();
        app.init_resource::<PendingControls>()
            .init_resource::<OverlayState>()
            .add_message::<WindowCloseRequested>()
            .add_message::<AppExit>()
            .add_systems(Update, apply_controls);
        let panel = app
            .world_mut()
            .spawn((Window::default(), PrimaryWindow))
            .id();
        (app, panel)
    }
    fn send(app: &mut App, control: Control) {
        app.world_mut()
            .resource_mut::<PendingControls>()
            .0
            .push(control);
        app.update();
    }
    #[test]
    fn closing_panel_stops_without_quitting_and_terminal_can_reopen() {
        let (mut app, panel) = control_app();
        send(&mut app, Control::Hold);
        assert!(app.world().resource::<OverlayState>().visible);
        app.world_mut()
            .write_message(WindowCloseRequested { window: panel });
        app.update();
        assert!(!app.world().resource::<OverlayState>().visible);
        assert!(!app.world().get::<Window>(panel).unwrap().visible);
        assert!(app.should_exit().is_none());
        send(&mut app, Control::ShowPanel);
        assert!(app.world().get::<Window>(panel).unwrap().visible);
        send(&mut app, Control::Quit);
        assert!(app.should_exit().is_some());
    }
    #[test]
    fn missing_terminal_keeps_panel_available_for_stop() {
        let (mut app, panel) = control_app();
        send(&mut app, Control::HidePanel);
        assert!(!app.world().get::<Window>(panel).unwrap().visible);
        send(&mut app, Control::TerminalClosed);
        send(&mut app, Control::HidePanel);
        assert!(app.world().get::<Window>(panel).unwrap().visible);
        app.world_mut()
            .write_message(WindowCloseRequested { window: panel });
        app.update();
        assert!(app.world().get::<Window>(panel).unwrap().visible);
    }
    #[test]
    fn unplugging_selected_display_clears_instead_of_migrating_drop() {
        let mut app = App::new();
        app.add_plugins((
            MinimalPlugins,
            bevy::asset::AssetPlugin::default(),
            bevy::image::ImagePlugin::default(),
        ))
        .init_resource::<OverlayState>()
        .insert_resource(idle_settings())
        .add_systems(Update, sync_overlay);
        let monitor = || Monitor {
            name: None,
            physical_width: 3024,
            physical_height: 1964,
            physical_position: IVec2::new(-3024, 0),
            scale_factor: 2.0,
            refresh_rate_millihertz: None,
            video_modes: Vec::new(),
        };
        let selected = app.world_mut().spawn((monitor(), PrimaryMonitor)).id();
        let remaining = app.world_mut().spawn(monitor()).id();
        let window = app
            .world_mut()
            .spawn((Window::default(), OverlayWindow, NativeReady))
            .id();
        let camera = app
            .world_mut()
            .spawn((Camera::default(), OverlayCamera))
            .id();
        let image = app
            .world()
            .resource::<AssetServer>()
            .load("overlay-proof.png");
        app.world_mut().spawn((Sprite::from_image(image), Fixture));
        {
            let mut state = app.world_mut().resource_mut::<OverlayState>();
            state.target = Some(selected);
            state.visible = true;
        }
        app.update();
        let bounds = app.world().get::<Window>(window).unwrap();
        assert!(bounds.visible);
        assert_eq!(bounds.resolution.physical_width(), 3024);
        assert_eq!(bounds.resolution.width(), 1512.0);
        assert_eq!(bounds.position, WindowPosition::At(IVec2::new(-3024, 0)));
        app.world_mut().despawn(selected);
        app.update();
        assert!(!app.world().get::<Window>(window).unwrap().visible);
        assert!(!app.world().get::<Camera>(camera).unwrap().is_active);
        app.update();
        let state = app.world().resource::<OverlayState>();
        assert_eq!(state.target, Some(remaining));
        assert!(!state.visible);
    }
    #[test]
    fn retina_bounds_preserve_physical_pixels() {
        let (size, scale) = display_bounds(UVec2::new(3024, 1964), 2.0).unwrap();
        assert_eq!(size.as_vec2() / scale, Vec2::new(1512.0, 982.0));
        assert!(display_bounds(UVec2::ZERO, 2.0).is_none());
        assert!(display_bounds(UVec2::ONE, f64::NAN).is_none());
        assert!(display_bounds(UVec2::ONE, 0.0).is_none());
    }
    #[test]
    fn terminal_commands_are_exact_and_bounded() {
        assert_eq!(Control::parse(" display 2 "), Some(Control::Display(2)));
        assert_eq!(Control::parse("stop"), Some(Control::Stop));
        for bad in ["display -1", "display 1 extra", "show ; quit", ""] {
            assert_eq!(Control::parse(bad), None);
        }
    }
}
