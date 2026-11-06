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
    /// Logical pixel offset mirrored into the native UI scroll viewport.
    scroll: f32,
    /// Measured logical height of the collection viewport; zero until its first layout.
    viewport_height: f32,
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
    /// Logical pointer distance from the thumb top, retained throughout a drag.
    drag_offset: f32,
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
    /// Enqueue one source operation without performing I/O on the UI thread.
    Network(crate::network_app::NetworkAction),
    /// Validate and bind this Mac's listener address from the source draft.
    EnableListener,
    /// Issue a separately revocable pairing token for the named controller.
    AddController,
    /// Transfer the masked draft to the secure credential store asynchronously.
    SaveAdminSecret,
}

/// Header menu whose choices configure a host or catalog provider.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Dropdown {
    /// Independently enabled local and authenticated remote controllers.
    From,
    /// This Mac's display or an authenticated remote output host.
    To,
    /// Bundled, directory or Git catalog sources.
    Catalog,
}

/// Isolated form edits; dropping this state discards ephemeral credentials.
struct Modal {
    /// Transactional provider schema or independently applied source controls.
    draft: ModalDraft,
    /// Provider fields, or source name/API key/listener address in that order.
    editors: Vec<Editor>,
    /// Validation feedback shown without reflecting credential values.
    error: Option<String>,
    /// Logical offset of the bounded form body, preserved across field edits.
    scroll: f32,
}

/// Distinguish a transactional provider form from live, independently applied source controls.
enum ModalDraft {
    /// Provider edits become effective only after a validated Apply.
    Configuration(ConfigurationDraft),
    /// Source controls enqueue individual operations; editors hold name, API key and listener address.
    Sources,
}

impl Modal {
    /// Expose provider identity only for transactional configuration forms.
    fn provider(&self) -> Option<Provider> {
        match &self.draft {
            ModalDraft::Configuration(draft) => Some(draft.provider),
            ModalDraft::Sources => None,
        }
    }
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
    /// Card height fits a complete title and controls into compact viewports.
    card_height: f32,
    /// Preview shrinks before the title and controls lose space.
    preview_height: f32,
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
        let toolbar_extra = if width < 820. { 80. } else { 0. };
        let viewport = (height - 296. - toolbar_extra).max(134.);
        let card_height = viewport.min(282.);
        let rows = ((viewport + 20.) / (card_height + 20.))
            .floor()
            .clamp(1., 6.) as usize;
        Self {
            columns,
            rows,
            card_width: (available - 20. * (columns - 1) as f32) / columns as f32,
            card_height,
            preview_height: card_height - 110.,
        }
    }
}

/// Native scroll viewport containing only the visible catalog rows and full-height spacing.
#[derive(Component)]
struct CatalogBody;

/// Thumb geometry is updated directly during scrolling without rebuilding the widget tree.
#[derive(Component)]
struct ScrollThumb;

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
                    | UiAction::Network(_)
                    | UiAction::EnableListener
                    | UiAction::AddController
                    | UiAction::SaveAdminSecret
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
            draft: ModalDraft::Configuration(draft),
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
        if let Some(Modal {
            draft: ModalDraft::Configuration(draft),
            ..
        }) = self.modal.take()
        {
            draft.cancel();
        }
        self.dropdown = None;
        self.focus = None;
    }

    /// Open live source controls with fresh, ephemeral name and API-key editors.
    fn open_sources(&mut self, listen_address: &str) {
        self.modal = Some(Modal {
            draft: ModalDraft::Sources,
            editors: vec![
                Editor::default(),
                Editor::default(),
                Editor {
                    text: listen_address.to_owned(),
                    cursor: listen_address.len(),
                    ..default()
                },
            ],
            error: None,
            scroll: 0.,
        });
        self.dropdown = None;
        self.focus = None;
        self.dragging = false;
    }

    /// Summarize enabled sources without implying they are mutually exclusive or connected.
    fn source_summary(local_enabled: bool, remote_count: usize) -> String {
        let controllers = format!(
            "{remote_count} controller{}",
            if remote_count == 1 { "" } else { "s" }
        );
        if local_enabled {
            format!("Local + {controllers}")
        } else {
            controllers
        }
    }

    /// Validate editable values before consuming the draft into a typed application intent.
    fn apply(&mut self, context: &ConfigurationContext) -> Option<ConfigurationIntent> {
        let modal = self.modal.as_mut()?;
        let ModalDraft::Configuration(draft) = &mut modal.draft else {
            return None;
        };
        for (field, editor) in draft.fields.iter_mut().zip(&modal.editors) {
            field.value.clone_from(&editor.text);
        }
        if let Err(error) = draft.validate(context) {
            modal.error = Some(error.message.into());
            return None;
        }
        let Modal {
            draft: ModalDraft::Configuration(draft),
            ..
        } = self.modal.take()?
        else {
            return None;
        };
        self.focus = None;
        draft.apply(context).ok()
    }

    /// Retrieve only an editable field; choices and immutable catalog paths do not accept text.
    fn editor(&mut self, field: Field) -> Option<&mut Editor> {
        match field {
            Field::Search => Some(&mut self.query),
            Field::Configuration(index) => {
                let modal = self.modal.as_mut()?;
                if let ModalDraft::Configuration(draft) = &modal.draft {
                    let field = draft.fields.get(index)?;
                    if field.read_only || !field.choices.is_empty() {
                        return None;
                    }
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
            Field::Configuration(index) => {
                self.modal.as_ref().is_some_and(|modal| match &modal.draft {
                    ModalDraft::Configuration(draft) => {
                        draft.fields.get(index).is_some_and(|field| field.secret)
                    }
                    ModalDraft::Sources => index == 1,
                })
            }
        }
    }

    /// Cycle keyboard focus through editable modal fields without leaking input to the catalog.
    fn focus_next(&mut self) {
        let Some(modal) = &self.modal else {
            return;
        };
        let fields: Vec<_> = match &modal.draft {
            ModalDraft::Configuration(draft) => draft
                .fields
                .iter()
                .enumerate()
                .filter(|(_, field)| !field.read_only && field.choices.is_empty())
                .map(|(index, _)| index)
                .collect(),
            ModalDraft::Sources => vec![0, 2, 1],
        };
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

    /// Exact content extent includes the inter-row gaps, with no trailing gap.
    fn content_height(&self) -> f32 {
        let rows = self.matches.div_ceil(self.layout.columns);
        rows as f32 * self.row_stride() - if rows == 0 { 0. } else { 20. }
    }

    /// Logical distance between successive row origins.
    fn row_stride(&self) -> f32 {
        self.layout.card_height + 20.
    }

    /// Use measured viewport geometry once Bevy has performed layout.
    fn viewport_height(&self) -> f32 {
        if self.viewport_height > 0. {
            self.viewport_height
        } else {
            self.layout.rows as f32 * self.row_stride() - 20.
        }
    }

    /// Scroll bounds depend on actual viewport size rather than complete visible rows.
    fn max_scroll(&self) -> f32 {
        (self.content_height() - self.viewport_height()).max(0.)
    }

    /// Row origin changes invalidate virtualized content, while fractional travel does not.
    fn first_row(&self) -> usize {
        (self.scroll / self.row_stride()).floor() as usize
    }

    /// Render intersecting rows plus one overscan row, independently of catalog size.
    fn visible(&self, clips: &[Clip]) -> Vec<usize> {
        clips
            .iter()
            .enumerate()
            .filter(|(_, clip)| Self::matching(clip, &self.query.text))
            .skip(self.first_row().saturating_mul(self.layout.columns))
            .take(self.layout.columns * (self.layout.rows + 2))
            .map(|(index, _)| index)
            .collect()
    }

    /// Return thumb top and extent in logical track pixels, including its minimum hit size.
    fn thumb_geometry(&self, track_height: f32) -> (f32, f32) {
        let height = (track_height * self.viewport_height() / self.content_height().max(1.))
            .max(24.)
            .min(track_height);
        let progress = (self.scroll / self.max_scroll().max(f32::EPSILON)).clamp(0., 1.);
        (progress * (track_height - height), height)
    }
}

/// Drag continuously, preserving the initial grab point and capture across row rebuilds.
fn scrollbar(
    mut model: ResMut<Model>,
    tracks: Query<
        (
            &Interaction,
            &bevy::ui::RelativeCursorPosition,
            &ComputedNode,
        ),
        With<Scrollbar>,
    >,
    mut bodies: Query<&mut ScrollPosition, With<CatalogBody>>,
    mouse: Res<ButtonInput<MouseButton>>,
) {
    if !mouse.pressed(MouseButton::Left) || model.modal.is_some() {
        model.dragging = false;
        return;
    }
    for (interaction, cursor, node) in &tracks {
        let Some(position) = cursor.normalized else {
            continue;
        };
        let track_height = node.size().y * node.inverse_scale_factor();
        if track_height <= 0. {
            continue;
        }
        let pointer = (position.y + 0.5) * track_height;
        let (top, height) = model.thumb_geometry(track_height);
        if *interaction == Interaction::Pressed && !model.dragging {
            model.dragging = true;
            model.drag_offset = if (top..=top + height).contains(&pointer) {
                pointer - top
            } else {
                height * 0.5
            };
        }
        if model.dragging {
            let progress = ((pointer - model.drag_offset)
                / (track_height - height).max(f32::EPSILON))
            .clamp(0., 1.);
            model.scroll = progress * model.max_scroll();
            for mut body in &mut bodies {
                body.y = model.scroll;
            }
        }
    }
}

/// Reflect every pixel of native scrolling in the scrollbar without invalidating the view.
fn scroll_thumb(
    model: Res<Model>,
    tracks: Query<&ComputedNode, With<Scrollbar>>,
    mut thumbs: Query<&mut Node, With<ScrollThumb>>,
) {
    for track in &tracks {
        let (top, height) = model.thumb_geometry(track.size().y * track.inverse_scale_factor());
        for mut thumb in &mut thumbs {
            thumb.top = px(top);
            thumb.height = px(height);
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
                (
                    buttons,
                    text_input,
                    scroll,
                    scrollbar,
                    scroll_thumb,
                    hover,
                    terminal,
                )
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
            UiAction::Dropdown(Dropdown::From) => {
                model.open_sources(&network.listen_address);
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
                let mut context = Model::configuration_context(&runtime, &network, &monitors);
                if *provider == Provider::Host(HostProvider::TailscaleListener) {
                    context.current_endpoint.clone_from(&network.listen_address);
                }
                model.open(*provider, &context);
            }
            UiAction::Cancel => model.close(),
            UiAction::Cycle(index) => {
                if let Some(modal) = &mut model.modal
                    && let ModalDraft::Configuration(draft) = &modal.draft
                {
                    let field = &draft.fields[*index];
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
                if runtime.destination == Destination::Remote
                    && let Some(modal) = &mut model.modal
                    && matches!(modal.provider(), Some(Provider::Catalog(_)))
                {
                    modal.error = Some("Catalog is supplied by the remote host. Choose To → This Mac before changing local catalog sources.".into());
                    continue;
                }
                if runtime.mode == Mode::Controller
                    && let Some(modal) = &mut model.modal
                {
                    let restriction = match modal.provider() {
                        Some(Provider::Host(
                            HostProvider::Native | HostProvider::TailscaleListener,
                        )) => Some(
                            "This launch is controller-only. Restart Durop in local mode to play or share this Mac.",
                        ),
                        Some(Provider::Catalog(_)) => Some(
                            "Catalog is supplied by the remote host. Restart Durop in local mode to use local catalog sources.",
                        ),
                        Some(Provider::Host(HostProvider::Tailscale)) | None => None,
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
                            network.listen_address = endpoint.to_string();
                            network
                                .actions
                                .push_back(crate::network_app::NetworkAction::EnableHost);
                            model.open_sources(&network.listen_address);
                        }
                        ConfigurationIntent::Bundled { path }
                        | ConfigurationIntent::LocalDirectory { path } => {
                            actions
                                .0
                                .push_back(Action::LoadPack(path.to_string_lossy().into_owned()));
                        }
                        ConfigurationIntent::GitHttps { .. } => {
                            actions.0.push_back(Action::LoadPack(
                                source.expect("catalog intent has a source"),
                            ));
                        }
                    }
                }
            }
            UiAction::Network(action) => network.actions.push_back(action.clone()),
            UiAction::EnableListener => {
                let Some(modal) = &mut model.modal else {
                    continue;
                };
                if !matches!(modal.draft, ModalDraft::Sources) {
                    continue;
                }
                let mut context = Model::configuration_context(&runtime, &network, &monitors);
                context.current_endpoint.clone_from(&modal.editors[2].text);
                let draft = ConfigurationDraft::new(
                    Provider::Host(HostProvider::TailscaleListener),
                    &context,
                );
                match draft.apply(&context) {
                    Ok(ConfigurationIntent::TailscaleListener { endpoint }) => {
                        network.listen_address = endpoint.to_string();
                        network
                            .actions
                            .push_back(crate::network_app::NetworkAction::EnableHost);
                        modal.error = None;
                    }
                    Err(error) => modal.error = Some(error.message.into()),
                    _ => unreachable!("listener schema returns a listener intent"),
                }
            }
            UiAction::AddController => {
                let Some(modal) = &mut model.modal else {
                    continue;
                };
                if !matches!(modal.draft, ModalDraft::Sources) {
                    continue;
                }
                let name = modal.editors[0].text.trim();
                if name.is_empty() || name.len() > 80 || name.chars().any(char::is_control) {
                    modal.error = Some(
                        "Give this controller a name of 1–80 bytes without control characters."
                            .into(),
                    );
                    continue;
                }
                network
                    .actions
                    .push_back(crate::network_app::NetworkAction::AddController {
                        name: name.to_owned(),
                    });
                modal.editors[0] = Editor::default();
                modal.error = None;
                model.focus = None;
            }
            UiAction::SaveAdminSecret => {
                let Some(modal) = &mut model.modal else {
                    continue;
                };
                if !matches!(modal.draft, ModalDraft::Sources) {
                    continue;
                }
                let secret = modal.editors[1].text.trim();
                if secret.is_empty() || secret.len() > 4096 || secret.chars().any(char::is_control)
                {
                    modal.error =
                        Some("Enter a Tailscale API key without control characters.".into());
                    continue;
                }
                network
                    .actions
                    .push_back(crate::network_app::NetworkAction::SaveAdminSecret {
                        secret: secret.to_owned(),
                    });
                modal.editors[1] = Editor::default();
                modal.error = None;
                model.focus = None;
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
                if field == Field::Search {
                    model.scroll = 0.;
                }
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
            if field == Field::Search {
                model.scroll = 0.;
            }
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

/// Apply unmodified OS wheel/momentum travel to native, logical-pixel scroll positions.
#[expect(
    clippy::type_complexity,
    reason = "Catalog and modal scroll queries exclude each other to permit independent mutable positions"
)]
fn scroll(
    mut wheel: MessageReader<MouseWheel>,
    primary: Single<(Entity, &Window), With<PrimaryWindow>>,
    mut model: ResMut<Model>,
    mut bodies: Query<
        (&ComputedNode, &mut ScrollPosition),
        (With<ModalBody>, Without<CatalogBody>),
    >,
    mut catalogs: Query<
        (&ComputedNode, &mut ScrollPosition),
        (With<CatalogBody>, Without<ModalBody>),
    >,
) {
    let (primary, window) = primary.into_inner();
    // Winit forwards physical pixel deltas, including fractional OS momentum on Retina.
    let scale_factor = window.scale_factor().max(f32::EPSILON);
    for (node, mut position) in &mut catalogs {
        let height = node.size().y * node.inverse_scale_factor();
        if height <= 0. {
            continue;
        }
        model.viewport_height = height;
        model.layout.rows = (model.viewport_height() / model.row_stride())
            .ceil()
            .max(1.) as usize;
        model.scroll = model.scroll.clamp(0., model.max_scroll());
        position.y = model.scroll;
    }
    // Clamp modal bounds on resize even when no wheel message arrives.
    if let Some(modal) = &mut model.modal {
        for (node, mut position) in &mut bodies {
            let maximum =
                ((node.content_size().y - node.size().y) * node.inverse_scale_factor()).max(0.);
            modal.scroll = modal.scroll.clamp(0., maximum);
            position.y = modal.scroll;
        }
    }
    for event in wheel.read() {
        if event.window != primary || !event.y.is_finite() {
            continue;
        }
        let delta = match event.unit {
            bevy::input::mouse::MouseScrollUnit::Line => event.y * 40.,
            bevy::input::mouse::MouseScrollUnit::Pixel => event.y / scale_factor,
        };
        if let Some(modal) = &mut model.modal {
            for (node, mut position) in &mut bodies {
                let maximum =
                    ((node.content_size().y - node.size().y) * node.inverse_scale_factor()).max(0.);
                modal.scroll = (position.y - delta).clamp(0., maximum);
                position.y = modal.scroll;
            }
        } else if delta != 0. {
            model.dropdown = None;
            model.scroll = (model.scroll - delta).clamp(0., model.max_scroll());
            for (_, mut position) in &mut catalogs {
                position.y = model.scroll;
            }
        }
    }
}
