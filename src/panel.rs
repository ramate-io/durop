//! Native controller panel with bounded virtualization and Unicode/IME editing.

pub(crate) mod configuration;
#[cfg(test)]
mod tests;
mod theme;
mod view;

use crate::{
    app::{Action, Actions, Destination, Mode, Runtime},
    catalog::Clip,
    overlay::PANEL_LAYER,
};
use bevy::{
    camera::{RenderTarget, visibility::RenderLayers},
    input::{
        ButtonState,
        keyboard::{Key, KeyboardInput},
        mouse::MouseWheel,
    },
    prelude::*,
    window::{Ime, PrimaryWindow, WindowRef},
};
use configuration::{
    CatalogProvider, ConfigurationContext, ConfigurationDraft, ConfigurationIntent, DisplayChoice,
    HostProvider, Provider,
};
use std::{
    collections::BTreeMap,
    io::Read,
    io::{self, BufRead},
    sync::{Mutex, mpsc},
};
use theme::Fonts;

/// Reference desktop grid width used by contract tests.
#[cfg(test)]
const COLUMNS: usize = 4;

/// Reference visible row bound used by contract tests.
#[cfg(test)]
const ROWS: usize = 3;

/// Identify the editor receiving committed keyboard and IME input.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Field {
    /// Catalog text filter.
    Search,
    /// Provider-specific editor inside the active configuration dialog.
    Configuration(usize),
}

/// Bounded Unicode editor with a byte-boundary cursor and separate IME preedit.
#[derive(Default, Clone)]
struct Editor {
    /// Committed UTF-8 content.
    text: String,
    /// Byte index guaranteed to lie on a character boundary.
    cursor: usize,
    /// IME composition displayed but not yet committed.
    preedit: String,
    /// Whether insertion/deletion replaces the full committed value.
    select_all: bool,
}

impl Editor {
    /// Replace a selection or insert printable text while preserving the character limit.
    fn insert(&mut self, value: &str) {
        if self.select_all {
            self.text.clear();
            self.cursor = 0;
            self.select_all = false;
        }
        let value: String = value
            .chars()
            .filter(|c| !c.is_control())
            .take(4096usize.saturating_sub(self.text.chars().count()))
            .collect();
        self.text.insert_str(self.cursor, &value);
        self.cursor += value.len();
    }

    /// Delete the selection or the preceding complete Unicode scalar.
    fn backspace(&mut self) {
        if self.select_all {
            self.text.clear();
            self.cursor = 0;
            self.select_all = false;
            return;
        }
        if let Some((i, _)) = self.text[..self.cursor].char_indices().next_back() {
            self.text.drain(i..self.cursor);
            self.cursor = i;
        }
    }

    /// Move the cursor to the preceding UTF-8 character boundary.
    fn left(&mut self) {
        self.cursor = self.text[..self.cursor]
            .char_indices()
            .next_back()
            .map_or(0, |(i, _)| i);
        self.select_all = false;
    }

    /// Move the cursor to the next UTF-8 character boundary.
    fn right(&mut self) {
        self.cursor = self.text[self.cursor..]
            .chars()
            .next()
            .map_or(self.cursor, |c| self.cursor + c.len_utf8());
        self.select_all = false;
    }

    /// Compose display text with the cursor and uncommitted IME preedit.
    fn label(&self) -> String {
        format!(
            "{}|{}{}",
            &self.text[..self.cursor],
            self.preedit,
            &self.text[self.cursor..]
        )
    }
}

/// Panel-only editing, pagination and render invalidation state.
#[derive(Resource, Default)]
struct Model {
    /// Search-field editor.
    query: Editor,
    /// Editor currently owning text input.
    focus: Option<Field>,
    /// First visible row after filtering.
    first_row: usize,
    /// Last render-state fingerprint used to avoid unnecessary tree rebuilds.
    signature: String,
    /// Current panel widget subtree.
    root: Option<Entity>,
    /// Open provider selector; closed before a configuration dialog appears.
    dropdown: Option<Dropdown>,
    /// Pending provider settings; applying is the only path to side effects.
    modal: Option<Modal>,
    /// Current responsive grid geometry and scroll bounds.
    layout: GridLayout,
    /// Number of filtered clips used to clamp scrolling and scrollbar position.
    matches: usize,
    /// Invalidate the view without including credential text in a fingerprint.
    revision: u64,
    /// Whether scrollbar dragging has captured the pointer.
    dragging: bool,
    /// Last persisted benign state, avoiding unchanged disk writes.
    saved_preferences: Option<crate::preferences::Preferences>,
    /// Modifier state reconstructed from ordered keyboard events.
    modifiers: u8,
    /// Native modifier flags used when physical key messages are absent.
    raw_modifiers: u8,
}

/// Attach a semantic action to a rendered button.
#[derive(Component, Clone)]
enum UiAction {
    /// Queue an application lifecycle/playback intent.
    Runtime(Action),
    /// Move text focus to one editor.
    Focus(Field),
    /// Open or close a provider menu.
    Dropdown(Dropdown),
    /// Open a new isolated configuration draft.
    Configure(Provider),
    /// Discard the draft and its ephemeral credential.
    Cancel,
    /// Validate and apply the active provider draft.
    Apply,
    /// Advance a finite choice field without accepting arbitrary display indices.
    Cycle(usize),
    /// Revoke the selected remote connection immediately.
    Disconnect,
    /// Revoke remote access to the local host without hiding the controller.
    RevokeHost,
}

/// Header menu whose choices configure a host or catalog provider.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Dropdown {
    /// Native or authenticated Tailscale control.
    Host,
    /// Bundled, directory or Git catalog sources.
    Catalog,
}

/// Isolated form edits; dropping this state discards ephemeral credentials.
struct Modal {
    /// Provider-owned validation schema and uncommitted values.
    draft: ConfigurationDraft,
    /// Unicode edit state corresponding one-to-one with draft fields.
    editors: Vec<Editor>,
    /// Validation feedback shown without reflecting credential values.
    error: Option<String>,
    /// Logical offset of the bounded form body, preserved across field edits.
    scroll: f32,
}

/// Responsive geometry keeps rendered entities bounded independently of catalog size.
#[derive(Clone, Copy, Debug)]
struct GridLayout {
    /// Visible columns based on available logical width.
    columns: usize,
    /// Complete visible rows based on available logical height.
    rows: usize,
    /// Logical width of each card after gaps and scrollbar reservation.
    card_width: f32,
    /// Fixed media-card height keeps row virtualization predictable.
    card_height: f32,
}

impl Default for GridLayout {
    fn default() -> Self {
        Self::for_window(1160., 820.)
    }
}

impl GridLayout {
    /// Fit large previews while allowing one-column compact windows and six-column wide windows.
    fn for_window(width: f32, height: f32) -> Self {
        let available = (width - 78.).max(180.);
        let columns = ((available + 20.) / 250.).floor().clamp(1., 6.) as usize;
        let toolbar_extra = if width < 760. { 80. } else { 0. };
        let rows = ((height - 276. - toolbar_extra) / 302.)
            .floor()
            .clamp(1., 6.) as usize;
        Self {
            columns,
            rows,
            card_width: (available - 20. * (columns - 1) as f32) / columns as f32,
            card_height: 282.,
        }
    }
}

/// Scrollbar track captures pointer coordinates without creating a node per catalog item.
#[derive(Component)]
struct Scrollbar;

/// Scrollable configuration body keeps its heading and action footer available.
#[derive(Component)]
struct ModalBody;

/// Reusable hover colors avoid rebuilding the tree while the pointer moves.
#[derive(Component)]
struct HoverColors {
    /// Paper color when the pointer is elsewhere.
    resting: Color,
    /// Accent used for hovered and pressed surfaces.
    hovered: Color,
}

impl Model {
    /// Block all background actions while a modal owns interaction, retaining the safety stop.
    fn allows(&self, action: &UiAction) -> bool {
        self.modal.is_none()
            || matches!(
                action,
                UiAction::Apply
                    | UiAction::Cancel
                    | UiAction::Cycle(_)
                    | UiAction::Focus(Field::Configuration(_))
                    | UiAction::RevokeHost
                    | UiAction::Configure(Provider::Host(HostProvider::TailscaleListener))
                    | UiAction::Runtime(Action::StopAll | Action::Stop(_))
            )
    }

    /// Build provider context from live monitor identities and the current committed application state.
    fn configuration_context(
        runtime: &Runtime,
        network: &crate::network_app::NetworkState,
        monitors: &Query<(Entity, &bevy::window::Monitor)>,
    ) -> ConfigurationContext {
        let mut monitors: Vec<_> = monitors.iter().collect();
        monitors.sort_by_key(|(entity, _)| entity.to_bits());
        let (git_url, git_reference, git_directory) = if runtime.pack_input.starts_with("https://")
        {
            let (url, revision) = runtime
                .pack_input
                .rsplit_once('#')
                .unwrap_or((&runtime.pack_input, "main"));
            let (reference, directory) = revision.split_once(':').unwrap_or((revision, ""));
            (url.to_owned(), reference.to_owned(), directory.to_owned())
        } else {
            (
                "https://github.com/ramate-io/durop.git".into(),
                "main".into(),
                "sample-packs/reactions".into(),
            )
        };
        ConfigurationContext {
            displays: monitors
                .iter()
                .enumerate()
                .map(|(index, (_, monitor))| DisplayChoice {
                    index,
                    label: format!(
                        "{} · {} × {}",
                        monitor.name.as_deref().unwrap_or("Display"),
                        monitor.physical_width,
                        monitor.physical_height
                    ),
                })
                .collect(),
            current_display: runtime.display,
            current_endpoint: network.address.clone(),
            local_pack: runtime.pack_input.clone(),
            bundled_path: crate::app::Config::bundled_pack(),
            git_url,
            git_reference,
            git_directory,
        }
    }

    /// Begin a new provider draft without changing playback or network ownership.
    fn open(&mut self, provider: Provider, context: &ConfigurationContext) {
        let draft = ConfigurationDraft::new(provider, context);
        let editors = draft
            .fields
            .iter()
            .map(|field| Editor {
                text: field.value.clone(),
                cursor: field.value.len(),
                ..default()
            })
            .collect();
        self.modal = Some(Modal {
            draft,
            editors,
            error: None,
            scroll: 0.,
        });
        self.dropdown = None;
        self.focus = None;
        self.dragging = false;
    }

    /// Discard modal state and transient menus; no provider intent is emitted.
    fn close(&mut self) {
        if let Some(modal) = self.modal.take() {
            modal.draft.cancel();
        }
        self.dropdown = None;
        self.focus = None;
    }

    /// Validate editable values before consuming the draft into a typed application intent.
    fn apply(&mut self, context: &ConfigurationContext) -> Option<ConfigurationIntent> {
        let modal = self.modal.as_mut()?;
        for (field, editor) in modal.draft.fields.iter_mut().zip(&modal.editors) {
            field.value.clone_from(&editor.text);
        }
        if let Err(error) = modal.draft.validate(context) {
            modal.error = Some(error.message.into());
            return None;
        }
        let modal = self.modal.take()?;
        self.focus = None;
        modal.draft.apply(context).ok()
    }

    /// Retrieve only an editable field; choices and immutable catalog paths do not accept text.
    fn editor(&mut self, field: Field) -> Option<&mut Editor> {
        match field {
            Field::Search => Some(&mut self.query),
            Field::Configuration(index) => {
                let modal = self.modal.as_mut()?;
                let field = modal.draft.fields.get(index)?;
                if field.read_only || !field.choices.is_empty() {
                    return None;
                }
                modal.editors.get_mut(index)
            }
        }
    }

    /// Trim pasted credentials while leaving ordinary user-entered field values exact.
    #[cfg(target_os = "macos")]
    fn is_secret(&self, field: Field) -> bool {
        match field {
            Field::Search => false,
            Field::Configuration(index) => self
                .modal
                .as_ref()
                .and_then(|modal| modal.draft.fields.get(index))
                .is_some_and(|field| field.secret),
        }
    }

    /// Cycle keyboard focus through editable modal fields without leaking input to the catalog.
    fn focus_next(&mut self) {
        let Some(modal) = &self.modal else {
            return;
        };
        let fields: Vec<_> = modal
            .draft
            .fields
            .iter()
            .enumerate()
            .filter(|(_, field)| !field.read_only && field.choices.is_empty())
            .map(|(index, _)| index)
            .collect();
        if fields.is_empty() {
            return;
        }
        let next = match self.focus {
            Some(Field::Configuration(index)) => fields
                .iter()
                .position(|candidate| *candidate == index)
                .map_or(0, |current| (current + 1) % fields.len()),
            _ => 0,
        };
        self.focus = Some(Field::Configuration(fields[next]));
    }

    /// Clamp the first virtual row against the complete filtered result count.
    fn max_row(&self) -> usize {
        self.matches
            .div_ceil(self.layout.columns)
            .saturating_sub(self.layout.rows)
    }

    /// Include one clipped preview row beyond complete rows without making the last row unreachable.
    fn visible(&self, clips: &[Clip]) -> Vec<usize> {
        clips
            .iter()
            .enumerate()
            .filter(|(_, clip)| Self::matching(clip, &self.query.text))
            .skip(self.first_row.saturating_mul(self.layout.columns))
            .take(self.layout.columns * (self.layout.rows + 1))
            .map(|(index, _)| index)
            .collect()
    }

    /// Map a track-relative pointer coordinate to the nearest available row.
    fn seek_scrollbar(&mut self, fraction: f32) {
        let total = self.matches.div_ceil(self.layout.columns).max(1);
        let thumb = (self.layout.rows as f32 / total as f32).min(1.);
        let progress = ((fraction - thumb * 0.5) / (1. - thumb).max(f32::EPSILON)).clamp(0., 1.);
        self.first_row = (progress * self.max_row() as f32).round() as usize;
    }
}

/// Drag the virtual scrollbar, retaining capture across subtree rebuilds and pointer exits.
fn scrollbar(
    mut model: ResMut<Model>,
    tracks: Query<(&Interaction, &bevy::ui::RelativeCursorPosition), With<Scrollbar>>,
    mouse: Res<ButtonInput<MouseButton>>,
) {
    if !mouse.pressed(MouseButton::Left) || model.modal.is_some() {
        model.dragging = false;
        return;
    }
    for (interaction, cursor) in &tracks {
        if *interaction == Interaction::Pressed {
            model.dragging = true;
        }
        if model.dragging
            && let Some(position) = cursor.normalized
        {
            model.seek_scrollbar(position.y + 0.5);
        }
    }
}

/// Apply subtle feedback to interactive paper surfaces without triggering layout churn.
fn hover(
    mut widgets: Query<(&Interaction, &HoverColors, &mut BackgroundColor), Changed<Interaction>>,
) {
    for (interaction, colors, mut background) in &mut widgets {
        background.0 = if *interaction == Interaction::None {
            colors.resting
        } else {
            colors.hovered
        };
    }
}

/// Cache decoded thumbnail handles and failed loads by clip identity.
#[derive(Resource, Default)]
struct Icons(BTreeMap<String, Option<Handle<Image>>>);

/// Maximum encoded thumbnail size read or accepted for decoding.
const MAX_ICON_BYTES: usize = 1024 * 1024;
impl Icons {
    /// Validate bounded PNG dimensions before allowing Bevy to decode a thumbnail.
    fn icon_image(bytes: &[u8]) -> Result<Image, String> {
        if bytes.len() > MAX_ICON_BYTES
            || bytes.len() < 24
            || !bytes.starts_with(b"\x89PNG\r\n\x1a\n")
            || &bytes[12..16] != b"IHDR"
        {
            return Err("invalid or oversized PNG thumbnail".into());
        }
        let width = u32::from_be_bytes(bytes[16..20].try_into().unwrap());
        let height = u32::from_be_bytes(bytes[20..24].try_into().unwrap());
        if width == 0 || height == 0 || width > 512 || height > 512 {
            return Err("thumbnail dimensions exceed 512x512".into());
        }
        Image::from_buffer(
            bytes,
            bevy::image::ImageType::Extension("png"),
            bevy::image::CompressedImageFormats::NONE,
            true,
            bevy::image::ImageSampler::nearest(),
            bevy::asset::RenderAssetUsages::default(),
        )
        .map_err(|_| "thumbnail decode failed".into())
    }

    /// Bound the bytes read from a local thumbnail before decoding.
    fn read_icon(path: &std::path::Path) -> Result<Vec<u8>, std::io::Error> {
        let mut bytes = Vec::new();
        std::fs::File::open(path)?
            .take(MAX_ICON_BYTES as u64 + 1)
            .read_to_end(&mut bytes)?;
        Ok(bytes)
    }
}

/// Receive optional stdin commands without blocking the UI thread.
#[derive(Resource)]
struct Terminal(Mutex<mpsc::Receiver<String>>);

/// Identify the camera rendering only the catalog panel layer.
#[derive(Resource)]
struct PanelCamera(Entity);

/// Install the bounded panel UI and its input systems.
pub struct PanelPlugin;
impl Plugin for PanelPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Model>()
            .init_resource::<Icons>()
            .init_resource::<Fonts>()
            .add_systems(Startup, setup)
            .add_systems(
                Update,
                (buttons, text_input, scroll, scrollbar, hover, terminal)
                    .chain()
                    .before(crate::app::actions),
            )
            .add_systems(
                Update,
                view::render
                    .after(crate::app::media_bridge)
                    .before(crate::overlay::sync_overlay),
            );
    }
}

/// Create the panel camera and restore benign preferences before the first render.
#[expect(
    clippy::too_many_arguments,
    reason = "Startup binds the panel camera, assets, preferences and native display controls"
)]
fn setup(
    mut commands: Commands,
    primary: Single<Entity, With<PrimaryWindow>>,
    proxy: Res<bevy::winit::EventLoopProxyWrapper>,
    runtime: Res<Runtime>,
    mut model: ResMut<Model>,
    mut controls: ResMut<crate::overlay::PendingControls>,
    assets: Res<AssetServer>,
    mut fonts: ResMut<Fonts>,
) {
    *fonts = Fonts::load(&assets);
    let camera = commands
        .spawn((
            Camera2d,
            RenderTarget::Window(WindowRef::Entity(*primary)),
            RenderLayers::layer(PANEL_LAYER),
        ))
        .id();
    commands.insert_resource(PanelCamera(camera));
    let preferences = crate::preferences::Preferences::load();
    model.query.text = preferences.query;
    model.query.cursor = model.query.text.len();
    if let Some(index) = runtime.display {
        controls.0.push(crate::overlay::Control::Display(index));
    }
    let (tx, rx) = mpsc::sync_channel(32);
    commands.insert_resource(Terminal(Mutex::new(rx)));
    let proxy = (**proxy).clone();
    std::thread::spawn(move || {
        for line in io::stdin().lock().lines() {
            let Ok(line) = line else {
                break;
            };
            if tx.send(line).is_err() {
                return;
            }
            if proxy
                .send_event(bevy::winit::WinitUserEvent::WakeUp)
                .is_err()
            {
                return;
            }
        }
        let _ = tx.send("terminal-closed".into());
        let _ = proxy.send_event(bevy::winit::WinitUserEvent::WakeUp);
    });
    eprintln!("Safety terminal: stop | panel | hide | play PACK/CLIP | quit");
}

/// Translate available stdin lines into application actions without blocking.
fn terminal(
    terminal: Res<Terminal>,
    mut actions: ResMut<Actions>,
    mut controls: ResMut<crate::overlay::PendingControls>,
    mut runtime: ResMut<Runtime>,
) {
    for line in terminal.0.lock().unwrap().try_iter().take(32) {
        match line.trim() {
            "stop" => actions.0.push_back(Action::StopAll),
            "panel" => actions.0.push_back(Action::Show),
            "hide" => actions.0.push_back(Action::Hide),
            "quit" => actions.0.push_back(Action::Quit),
            "terminal-closed" => {
                runtime.terminal_active = false;
                if runtime.mode != Mode::Host {
                    controls.0.push(crate::overlay::Control::TerminalClosed);
                }
            }
            "displays" => controls.0.push(crate::overlay::Control::Displays),
            s if s.starts_with("play ") => actions.0.push_back(Action::Play(s[5..].into())),
            s if s.starts_with("display ") => {
                if let Ok(i) = s[8..].parse() {
                    controls.0.push(crate::overlay::Control::Display(i));
                }
            }
            _ => {
                eprintln!("Use stop | panel | hide | play PACK/CLIP | displays | display N | quit")
            }
        }
    }
}

impl Model {
    /// Require every search term to match a clip title, category or tag.
    fn matching(clip: &Clip, query: &str) -> bool {
        query.to_lowercase().split_whitespace().all(|part| {
            clip.metadata.title.to_lowercase().contains(part)
                || clip
                    .category
                    .to_string_lossy()
                    .to_lowercase()
                    .contains(part)
                || clip
                    .metadata
                    .tags
                    .iter()
                    .any(|tag| tag.to_lowercase().contains(part))
        })
    }

    /// Select only the current page of matching catalog entries for rendering.
    #[cfg(test)]
    fn visible_indices(clips: &[Clip], query: &str, row: usize) -> Vec<usize> {
        clips
            .iter()
            .enumerate()
            .filter(|(_, c)| Model::matching(c, query))
            .skip(row.saturating_mul(COLUMNS))
            .take(COLUMNS * ROWS)
            .map(|(i, _)| i)
            .collect()
    }
}

/// Dispatch one pointer press; rebuilding widgets must not repeat a held click.
fn buttons(
    mut model: ResMut<Model>,
    buttons: Query<(&Interaction, &UiAction), Changed<Interaction>>,
    mut actions: ResMut<Actions>,
    mut network: ResMut<crate::network_app::NetworkState>,
    runtime: Res<Runtime>,
    monitors: Query<(Entity, &bevy::window::Monitor)>,
    mouse: Option<Res<ButtonInput<MouseButton>>>,
) {
    if mouse
        .as_ref()
        .is_some_and(|mouse| !mouse.just_pressed(MouseButton::Left))
    {
        return;
    }
    for (interaction, action) in &buttons {
        if *interaction != Interaction::Pressed || !model.allows(action) {
            continue;
        }
        model.revision = model.revision.wrapping_add(1);
        match action {
            UiAction::Runtime(action) => {
                model.focus = None;
                model.dropdown = None;
                actions.0.push_back(action.clone());
            }
            UiAction::Focus(field) => {
                model.focus = Some(*field);
                model.dropdown = None;
            }
            UiAction::Dropdown(menu) => {
                model.dropdown = if model.dropdown == Some(*menu) {
                    None
                } else {
                    Some(*menu)
                };
                model.focus = None;
            }
            UiAction::Configure(provider) => {
                let context = Model::configuration_context(&runtime, &network, &monitors);
                model.open(*provider, &context);
            }
            UiAction::Cancel => model.close(),
            UiAction::Cycle(index) => {
                if let Some(modal) = &mut model.modal {
                    let field = &modal.draft.fields[*index];
                    let editor = &mut modal.editors[*index];
                    if !field.choices.is_empty() {
                        let selected = field
                            .choices
                            .iter()
                            .position(|choice| choice.value == editor.text)
                            .unwrap_or(0);
                        editor.text = field.choices[(selected + 1) % field.choices.len()]
                            .value
                            .clone();
                        editor.cursor = editor.text.len();
                    }
                }
            }
            UiAction::Apply => {
                if runtime.mode == Mode::Controller
                    && let Some(modal) = &mut model.modal
                {
                    let restriction = match modal.draft.provider {
                        Provider::Host(HostProvider::Native | HostProvider::TailscaleListener) => {
                            Some(
                                "This launch is controller-only. Restart Durop in local mode to play or share this Mac.",
                            )
                        }
                        Provider::Catalog(_) => Some(
                            "Catalog is supplied by the remote host. Restart Durop in local mode to use local catalog sources.",
                        ),
                        Provider::Host(HostProvider::Tailscale) => None,
                    };
                    if let Some(message) = restriction {
                        modal.error = Some(message.into());
                        continue;
                    }
                }
                let context = Model::configuration_context(&runtime, &network, &monitors);
                if let Some(intent) = model.apply(&context) {
                    let source = intent.source_value();
                    match intent {
                        ConfigurationIntent::Native { display } => {
                            actions.0.push_back(Action::Destination(Destination::Local));
                            actions.0.push_back(Action::SelectDisplay(display));
                            if !runtime.enabled {
                                actions.0.push_back(Action::ToggleHost);
                            }
                        }
                        ConfigurationIntent::Tailscale {
                            endpoint,
                            credential,
                        } => {
                            network.address = format!("ws://{endpoint}/control");
                            network.secret = credential.bearer().to_owned();
                            network
                                .actions
                                .push_back(crate::network_app::NetworkAction::Connect);
                            actions
                                .0
                                .push_back(Action::Destination(Destination::Remote));
                        }
                        ConfigurationIntent::TailscaleListener { endpoint } => {
                            network.address = endpoint.to_string();
                            network
                                .actions
                                .push_back(crate::network_app::NetworkAction::EnableHost);
                            model.open(Provider::Host(HostProvider::Native), &context);
                        }
                        ConfigurationIntent::Bundled { path }
                        | ConfigurationIntent::LocalDirectory { path } => {
                            actions.0.push_back(Action::Destination(Destination::Local));
                            actions
                                .0
                                .push_back(Action::LoadPack(path.to_string_lossy().into_owned()));
                        }
                        ConfigurationIntent::GitHttps { .. } => {
                            actions.0.push_back(Action::Destination(Destination::Local));
                            actions.0.push_back(Action::LoadPack(
                                source.expect("catalog intent has a source"),
                            ));
                        }
                    }
                }
            }
            UiAction::RevokeHost => {
                network
                    .actions
                    .push_back(crate::network_app::NetworkAction::DisableHost);
            }
            UiAction::Disconnect => {
                network
                    .actions
                    .push_back(crate::network_app::NetworkAction::Disconnect);
            }
        }
    }
}

/// Retain raw-window modifier ordering alongside translated key events.
enum OrderedInput {
    /// Native command/control flags at this point in the event sequence.
    Modifiers(u8),
    /// Keyboard message paired with the flags active at its position.
    Keyboard(KeyboardInput),
}

/// Merge keyboard events with raw modifier changes in their original event order.
fn ordered_keyboard(
    events: impl IntoIterator<Item = OrderedInput>,
    modifiers: &mut u8,
) -> Vec<(KeyboardInput, bool)> {
    let mut keys = Vec::new();
    for event in events {
        match event {
            OrderedInput::Modifiers(value) => *modifiers = value,
            OrderedInput::Keyboard(key) => keys.push((key, *modifiers != 0)),
        }
    }
    keys
}

/// Route keyboard, clipboard and IME input to the focused editor before shortcuts.
#[expect(
    clippy::too_many_arguments,
    reason = "The editor combines ordered raw modifier flags, Bevy IME and window state"
)]
fn text_input(
    mut keys: MessageReader<KeyboardInput>,
    mut ime: MessageReader<Ime>,
    primary: Single<(Entity, &mut Window), With<PrimaryWindow>>,
    mut model: ResMut<Model>,
    runtime: Res<Runtime>,
    mut actions: ResMut<Actions>,
    _main: bevy::ecs::system::NonSendMarker,
    mut raw: MessageReader<bevy::winit::RawWinitWindowEvent>,
    native: Option<NonSend<bevy::winit::WinitWindows>>,
) {
    let (entity, mut window) = primary.into_inner();
    window.ime_enabled = model.focus.is_some();
    if !window.focused {
        model.modifiers = 0;
        model.raw_modifiers = 0;
        keys.clear();
        ime.clear();
        raw.clear();
        return;
    }
    let mut committed = false;
    for event in ime.read() {
        model.revision = model.revision.wrapping_add(1);
        let Some(field) = model.focus else {
            continue;
        };
        let Some(edit) = model.editor(field) else {
            continue;
        };
        match event {
            Ime::Preedit { window, value, .. } if *window == entity => edit.preedit = value.clone(),
            Ime::Commit { window, value } if *window == entity => {
                edit.preedit.clear();
                edit.insert(value);
                committed = true;
            }
            Ime::Disabled { window } if *window == entity => edit.preedit.clear(),
            _ => (),
        }
    }
    let native_id = native
        .as_ref()
        .and_then(|windows| windows.entity_to_winit.get(&entity))
        .copied();
    let mut raw_steps = Vec::new();
    for event in raw.read() {
        if Some(event.window_id) != native_id {
            continue;
        }
        match &event.event {
            winit::event::WindowEvent::ModifiersChanged(modifiers) => {
                let flags = modifiers.state();
                raw_steps.push(OrderedInput::Modifiers(
                    u8::from(flags.super_key()) | u8::from(flags.control_key()) << 2,
                ));
            }
            winit::event::WindowEvent::KeyboardInput { event, .. } => {
                raw_steps.push(OrderedInput::Keyboard(
                    bevy::winit::converters::convert_keyboard_input(event, entity),
                ))
            }
            winit::event::WindowEvent::Focused(false) => raw_steps.push(OrderedInput::Modifiers(0)),
            _ => (),
        }
    }
    let ordered = ordered_keyboard(raw_steps, &mut model.raw_modifiers);
    let input: Vec<_> = if ordered.is_empty() {
        keys.read().cloned().map(|key| (key, None)).collect()
    } else {
        keys.clear();
        ordered
            .into_iter()
            .map(|(key, modified)| (key, Some(modified)))
            .collect()
    };
    for (key, raw_modified) in input {
        if key.window != entity {
            continue;
        }
        let modifier = match key.key_code {
            KeyCode::SuperLeft => 1,
            KeyCode::SuperRight => 2,
            KeyCode::ControlLeft => 4,
            KeyCode::ControlRight => 8,
            _ => 0,
        };
        if modifier != 0 {
            if key.state == ButtonState::Pressed {
                model.modifiers |= modifier;
            } else {
                model.modifiers &= !modifier;
            }
        }
        if key.state != ButtonState::Pressed {
            continue;
        }
        model.revision = model.revision.wrapping_add(1);
        if key.logical_key == Key::Escape && (model.modal.is_some() || model.dropdown.is_some()) {
            model.close();
            continue;
        }
        if key.logical_key == Key::Tab && model.modal.is_some() {
            model.focus_next();
            continue;
        }
        if let Some(field) = model.focus {
            // Preserve modifier ordering even if key-down and key-up arrive in
            // one update, when ButtonInput already contains the final state.
            let modified = raw_modified.unwrap_or(model.modifiers != 0);
            #[cfg(target_os = "macos")]
            let secret = model.is_secret(field);
            let Some(edit) = model.editor(field) else {
                continue;
            };
            if key.logical_key == Key::Escape {
                edit.preedit.clear();
                model.focus = None;
                continue;
            }
            if !edit.preedit.is_empty() {
                continue;
            }
            match key.logical_key {
                Key::Backspace => edit.backspace(),
                Key::ArrowLeft => edit.left(),
                Key::ArrowRight => edit.right(),
                Key::Home => edit.cursor = 0,
                Key::End => edit.cursor = edit.text.len(),
                Key::Enter => {
                    model.focus = None;
                }
                _ if modified && key.key_code == KeyCode::KeyA => edit.select_all = true,
                #[cfg(target_os = "macos")]
                _ if modified && key.key_code == KeyCode::KeyV => {
                    if let Some(text) = crate::app_macos::paste_text() {
                        edit.insert(if secret { text.trim() } else { &text });
                    }
                }
                _ if !modified && !committed => {
                    if let Some(text) = &key.text {
                        edit.insert(text);
                    }
                }
                _ => (),
            }
            model.first_row = 0;
        } else if !key.repeat && !raw_modified.unwrap_or(model.modifiers != 0) {
            match key.logical_key {
                Key::Escape => actions.0.push_back(Action::StopAll),
                Key::Space
                    if runtime.destination == Destination::Local
                        && model.modal.is_none()
                        && model.dropdown.is_none() =>
                {
                    if let Some(index) = model.visible(&runtime.catalog.clips).first() {
                        actions.0.push_back(Action::Play(
                            runtime.catalog.clips[*index].identity.clone(),
                        ));
                    }
                }
                _ => (),
            }
        }
    }
}

/// Route wheel input to the active form body or bounded collection rows.
fn scroll(
    mut wheel: MessageReader<MouseWheel>,
    primary: Single<Entity, With<PrimaryWindow>>,
    mut model: ResMut<Model>,
    mut bodies: Query<(&ComputedNode, &mut ScrollPosition), With<ModalBody>>,
) {
    for event in wheel.read() {
        if event.window != *primary || event.y == 0.0 {
            continue;
        }
        if let Some(modal) = &mut model.modal {
            for (node, mut position) in &mut bodies {
                let maximum =
                    ((node.content_size().y - node.size().y) * node.inverse_scale_factor()).max(0.);
                let delta = match event.unit {
                    bevy::input::mouse::MouseScrollUnit::Line => event.y * 32.,
                    bevy::input::mouse::MouseScrollUnit::Pixel => event.y,
                };
                modal.scroll = (position.y - delta).clamp(0., maximum);
                position.y = modal.scroll;
            }
        } else {
            model.dropdown = None;
            model.first_row = if event.y > 0.0 {
                model.first_row.saturating_sub(1)
            } else {
                (model.first_row + 1).min(model.max_row())
            };
        }
    }
}
