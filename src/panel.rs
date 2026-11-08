//! Native controller panel with bounded virtualization and Unicode/IME editing.

pub(crate) mod configuration;

mod placement;

mod input;

#[cfg(test)]
mod tests;
mod theme;
mod view;

use crate::commands::{VisualFit, VisualPlacement};
use crate::{
    app::{Action, Actions, Destination, Mode, Runtime},
    catalog::Clip,
    overlay::PANEL_LAYER,
};
use bevy::{
    camera::{RenderTarget, visibility::RenderLayers},
    input::{
        keyboard::{Key, KeyboardInput},
        mouse::MouseWheel,
    },
    input_focus::{
        FocusCause, FocusedInput, InputFocus,
        tab_navigation::{TabGroup, TabIndex, TabNavigationPlugin},
    },
    picking::hover::Hovered,
    prelude::*,
    text::{EditableText, EditableTextFilter, EditableTextSystems, TextCursorStyle},
    ui::{InteractionDisabled, Pressed},
    ui_widgets::{Activate, ActivateOnPress, Button, TextInput},
    window::{PrimaryWindow, WindowRef},
};
use configuration::{
    CatalogProvider, ConfigurationContext, ConfigurationDraft, ConfigurationIntent, DisplayChoice,
    HostProvider, Provider,
};
use placement::{PlacementModal, PlacementPreset, PlacementScope};
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
    /// Visual placement percentage in the active isolated draft.
    Placement(usize),
}

/// Draft mirror whose live Bevy input survives surrounding view refreshes.
#[derive(Default, Clone)]
struct Editor {
    /// Committed content only; IME preedit belongs to the engine editor.
    text: String,
    /// Persistent input entity, retired when this draft is replaced or closed.
    entity: Option<Entity>,
    /// Masked secret fields support replacement without exposing plaintext widgets.
    selected: bool,
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
    /// Experimental visual placement form, mutually exclusive with provider forms.
    placement_modal: Option<PlacementModal>,
    /// Exact destination lifetime owning the bounded session adjustments.
    placement_scope: Option<PlacementScope>,
    /// At most 256 clip adjustments; catalog metadata is never modified.
    placements: BTreeMap<String, VisualPlacement>,
    /// Oldest-applied order for bounded override eviction.
    placement_order: std::collections::VecDeque<String>,
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
}

/// Attach a semantic action to a rendered button.
#[derive(Component, Clone)]
enum UiAction {
    /// Queue an application lifecycle/playback intent.
    Runtime(Action),

    /// Play with the current session adjustment.
    Play(String),
    /// Open the isolated visual-only placement draft.
    EditPlacement(String),
    /// Reset the draft to catalog values without applying it.
    ResetPlacement,
    /// Set a common display position in the draft without applying or sending it.
    PresetPlacement(PlacementPreset),
    /// Nudge one visual percentage by one point for pointer-only adjustment.
    StepPlacement(usize, i16),
    /// Apply placement to active occurrences, optionally sending another play.
    ApplyPlacement(bool),

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
    /// Select a source form without changing that source's permission.
    SelectSource(SourceSelection),
    /// Return from an uncommitted new-source draft without creating a controller.
    CancelSource,
    /// Expand one optional host setting inside the selected remote source.
    SourceDetails(SourceDetails),
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
    Sources(SourcesDraft),
}

/// The selected form is independent of each source's enabled permission.
#[derive(Clone, PartialEq, Eq)]
enum SourceSelection {
    /// The singleton source for this Mac's buttons and shortcuts.
    Local,
    /// One already configured remote source, identified without its secret.
    Remote(String),
    /// Source-type chooser opened by the empty plus slot.
    ChooseType,
    /// Empty Tailscale controller form that emits no action until Add.
    NewRemote,
}

/// Only one optional shared-host setting is expanded at a time.
#[derive(Clone, Copy, Default, PartialEq, Eq)]
enum SourceDetails {
    /// Show the selected source's ordinary controls only.
    #[default]
    None,
    /// Configure the shared inbound listener without changing To.
    Listener,
    /// Explicitly edit the host's protected Tailscale invitation credential.
    Credentials,
}

/// Track one explicit Add until the runtime supplies the created source identity.
struct PendingSource {
    /// Validated name used only to recognize the result of this Add.
    name: String,
    /// Existing identities prevent a duplicate name from selecting an older source.
    known_ids: Vec<String>,
}

/// Panel-only navigation and uncommitted add ownership; never grants source permission.
struct SourcesDraft {
    /// One currently displayed source or uncommitted provider form.
    selection: SourceSelection,
    /// Source to restore when the plus/type/new-source flow is canceled.
    previous: Option<SourceSelection>,
    /// Optional host detail opened explicitly by the user.
    details: SourceDetails,
    /// A queued Add awaiting a concrete runtime-owned source.
    pending: Option<PendingSource>,
}

impl Default for SourcesDraft {
    fn default() -> Self {
        Self {
            selection: SourceSelection::Local,
            previous: None,
            details: SourceDetails::None,
            pending: None,
        }
    }
}

impl Modal {
    /// Expose provider identity only for transactional configuration forms.
    fn provider(&self) -> Option<Provider> {
        match &self.draft {
            ModalDraft::Configuration(draft) => Some(draft.provider),
            ModalDraft::Sources(_) => None,
        }
    }

    /// Navigate source forms without queuing permission or creation actions.
    fn select_source(&mut self, selection: SourceSelection) {
        let ModalDraft::Sources(sources) = &mut self.draft else {
            return;
        };
        if matches!(selection, SourceSelection::ChooseType)
            && matches!(
                sources.selection,
                SourceSelection::Local | SourceSelection::Remote(_)
            )
        {
            sources.previous = Some(sources.selection.clone());
        } else if matches!(
            selection,
            SourceSelection::Local | SourceSelection::Remote(_)
        ) {
            sources.previous = None;
        }
        sources.selection = selection;
        sources.details = SourceDetails::None;
        sources.pending = None;
        self.editors[0] = Editor::default();
        self.editors[1] = Editor::default();
        self.error = None;
        self.scroll = 0.;
    }

    /// Cancel only the uncommitted plus flow and discard its sensitive or name edits.
    fn cancel_source(&mut self) {
        let ModalDraft::Sources(sources) = &mut self.draft else {
            return;
        };
        let selection = sources.previous.take().unwrap_or(SourceSelection::Local);
        self.select_source(selection);
    }

    /// Reveal a host setting explicitly; closing or switching always discards a key draft.
    fn show_source_details(&mut self, details: SourceDetails) {
        let ModalDraft::Sources(sources) = &mut self.draft else {
            return;
        };
        if !matches!(sources.selection, SourceSelection::Remote(_)) {
            return;
        }
        sources.details = if sources.details == details {
            SourceDetails::None
        } else {
            details
        };
        self.editors[1] = Editor::default();
        self.error = None;
        self.scroll = 0.;
    }

    /// Only visible source editors can own keyboard or IME input.
    fn source_fields(&self) -> Vec<usize> {
        let ModalDraft::Sources(sources) = &self.draft else {
            return vec![];
        };
        match (&sources.selection, sources.details) {
            (SourceSelection::NewRemote, _) if sources.pending.is_none() => vec![0],
            (SourceSelection::Remote(_), SourceDetails::Listener) => vec![2],
            (SourceSelection::Remote(_), SourceDetails::Credentials) => vec![1],
            _ => vec![],
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
    /// Reserve complete toolbar rows before estimating the first collection viewport.
    fn toolbar_height(width: f32) -> f32 {
        if width < 534. {
            228.
        } else if width < 840. {
            148.
        } else {
            68.
        }
    }

    /// Fit large previews while allowing one-column compact windows and six-column wide windows.
    fn for_window(width: f32, height: f32) -> Self {
        let available = (width - 78.).max(180.);
        let columns = ((available + 20.) / 250.).floor().clamp(1., 6.) as usize;
        let toolbar_extra = Self::toolbar_height(width) - 68.;
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

/// Card-level feedback follows confirmed playback and descendant pointer hover.
#[derive(Component)]
struct CardFeedback {
    /// True while at least one occurrence is actually playing at the selected destination.
    playing: bool,
}

impl CardFeedback {
    /// Choose a warm surface independently of pointer position so playback stays visible.
    fn background(&self) -> Color {
        if self.playing {
            theme::PLAYING_PAPER
        } else {
            theme::PAPER
        }
    }

    /// Shadows change paint only; no size or position changes can disturb scrolling or text.
    fn shadow(&self, hovered: bool) -> BoxShadow {
        if self.playing {
            BoxShadow::new(
                theme::PLAYING_GLOW,
                px(0),
                px(0),
                px(1),
                px(if hovered { 12 } else { 8 }),
            )
        } else if hovered {
            BoxShadow::new(theme::HOVER_SHADOW, px(0), px(3), px(0), px(10))
        } else {
            BoxShadow::default()
        }
    }
}

impl Model {
    /// Whether either isolated form owns input and scrolling.
    fn has_modal(&self) -> bool {
        self.modal.is_some() || self.placement_modal.is_some()
    }

    /// Access the scroll position of the isolated form currently owning input.
    fn modal_scroll(&mut self) -> Option<&mut f32> {
        if let Some(modal) = &mut self.modal {
            Some(&mut modal.scroll)
        } else {
            self.placement_modal.as_mut().map(|modal| &mut modal.scroll)
        }
    }

    /// Block all background actions while a modal owns interaction, retaining the safety stop.
    fn allows(&self, action: &UiAction) -> bool {
        !self.has_modal()
            || matches!(
                action,
                UiAction::Apply
                    | UiAction::ApplyPlacement(_)
                    | UiAction::ResetPlacement
                    | UiAction::PresetPlacement(_)
                    | UiAction::StepPlacement(_, _)
                    | UiAction::Cancel
                    | UiAction::Cycle(_)
                    | UiAction::Network(_)
                    | UiAction::EnableListener
                    | UiAction::AddController
                    | UiAction::SaveAdminSecret
                    | UiAction::SelectSource(_)
                    | UiAction::CancelSource
                    | UiAction::SourceDetails(_)
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
        self.placement_modal = None;

        self.revision = self.revision.wrapping_add(1);

        let draft = ConfigurationDraft::new(provider, context);
        let editors = draft
            .fields
            .iter()
            .map(|field| Editor {
                text: field.value.clone(),
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
        self.placement_modal = None;
        self.revision = self.revision.wrapping_add(1);
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
        self.placement_modal = None;
        self.revision = self.revision.wrapping_add(1);
        self.modal = Some(Modal {
            draft: ModalDraft::Sources(SourcesDraft::default()),
            editors: vec![
                Editor::default(),
                Editor::default(),
                Editor {
                    text: listen_address.to_owned(),
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

    /// Select the result of an explicit Add and recover if a selected source was revoked.
    fn reconcile_sources(&mut self, network: &crate::network_app::NetworkState) {
        if let Some(modal) = &mut self.modal
            && matches!(modal.draft, ModalDraft::Sources(_))
            && modal
                .editors
                .get(2)
                .is_some_and(|editor| editor.text.is_empty() && editor.entity.is_none())
        {
            let endpoint = network.listener_endpoint();
            if !endpoint.is_empty() {
                modal.editors[2] = Editor {
                    text: endpoint.to_owned(),
                    ..default()
                };
            }
        }
        if let Some(modal) = &mut self.modal
            && matches!(
                &modal.draft,
                ModalDraft::Sources(sources) if sources.details == SourceDetails::Listener
            )
            && !network.hosting
            && network.status.starts_with("Remote host unavailable:")
        {
            modal.error = Some(network.status.clone());
        }
        let Some(modal) = &mut self.modal else {
            return;
        };
        let ModalDraft::Sources(sources) = &mut modal.draft else {
            return;
        };
        if let Some(pending) = &sources.pending {
            if let Some(source) = network.controllers.iter().find(|source| {
                source.name == pending.name && !pending.known_ids.contains(&source.id)
            }) {
                let selection = SourceSelection::Remote(source.id.clone());
                modal.select_source(selection);
            } else if !network.actions.iter().any(|action| {
                matches!(action,
                crate::network_app::NetworkAction::AddController {name} if name == &pending.name)
            }) {
                sources.pending = None;
                modal.error = Some(network.status.clone());
            }
        } else if matches!(&sources.selection, SourceSelection::Remote(id)
            if !network.controllers.iter().any(|source| &source.id == id))
        {
            modal.select_source(SourceSelection::Local);
        }
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
            Field::Placement(index) => self.placement_modal.as_mut()?.editors.get_mut(index),
            Field::Configuration(index) => {
                let modal = self.modal.as_mut()?;
                if let ModalDraft::Configuration(draft) = &modal.draft {
                    let field = draft.fields.get(index)?;
                    if field.read_only || !field.choices.is_empty() {
                        return None;
                    }
                } else if !modal.source_fields().contains(&index) {
                    return None;
                }
                modal.editors.get_mut(index)
            }
        }
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
            Has<Pressed>,
            &bevy::ui::RelativeCursorPosition,
            &ComputedNode,
        ),
        With<Scrollbar>,
    >,
    mut bodies: Query<&mut ScrollPosition, With<CatalogBody>>,
    mouse: Res<ButtonInput<MouseButton>>,
) {
    if !mouse.pressed(MouseButton::Left) || model.has_modal() {
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
        if interaction && !model.dragging {
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
fn hover(mut widgets: Query<(&Hovered, Has<Pressed>, &HoverColors, &mut BackgroundColor)>) {
    for (hovered, pressed, colors, mut background) in &mut widgets {
        let color = if hovered.get() || pressed {
            colors.hovered
        } else {
            colors.resting
        };
        if background.0 != color {
            background.0 = color;
        }
    }
}

/// Paint whole-card hover feedback, including previews, selectable titles and controls.
fn card_feedback(model: Res<Model>, mut cards: Query<(&Hovered, &CardFeedback, &mut BoxShadow)>) {
    for (hovered, feedback, mut shadow) in &mut cards {
        let desired = feedback.shadow(hovered.get() && !model.has_modal());
        if *shadow != desired {
            *shadow = desired;
        }
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
        app.add_plugins(TabNavigationPlugin)
            .add_observer(input::activate)
            .add_observer(input::keyboard)
            .init_resource::<input::PendingActions>()
            .init_resource::<input::OrderedKeyboard>()
            .init_resource::<Model>()
            .init_resource::<Icons>()
            .init_resource::<Fonts>()
            .add_systems(Startup, setup)
            .add_systems(
                PreUpdate,
                input::dispatch_keyboard
                    .after(bevy::input::InputSystems)
                    .before(bevy::input_focus::InputFocusSystems::Dispatch),
            )
            .add_systems(
                Update,
                (
                    scroll,
                    scrollbar,
                    scroll_thumb,
                    hover,
                    card_feedback,
                    terminal,
                )
                    .chain()
                    .before(crate::app::actions),
            )
            .add_systems(
                PostUpdate,
                (
                    input::synchronize,
                    buttons,
                    input::focus_style,
                    input::request_redraw,
                )
                    .chain()
                    .after(EditableTextSystems),
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
    mut pending: ResMut<input::PendingActions>,
    mut focus: ResMut<InputFocus>,
    mut actions: ResMut<Actions>,
    mut network: ResMut<crate::network_app::NetworkState>,
    runtime: Res<Runtime>,
    monitors: Query<(Entity, &bevy::window::Monitor)>,
) {
    model.sync_placements(PlacementScope::current(&runtime, &network));
    for action in &std::mem::take(&mut pending.0) {
        if !model.allows(action) {
            continue;
        }
        model.revision = model.revision.wrapping_add(1);
        match action {
            UiAction::Play(clip_id) => {
                model.focus = None;
                actions.0.push_back(model.play_action(clip_id));
            }
            UiAction::EditPlacement(clip_id) => model.open_placement(clip_id, &runtime, &network),
            UiAction::ResetPlacement => {
                if let Some(modal) = &mut model.placement_modal {
                    modal.reset();
                }
                model.focus = None;
            }
            UiAction::PresetPlacement(preset) => {
                if let Some(modal) = &mut model.placement_modal {
                    preset.apply(modal);
                }
                model.focus = None;
            }
            UiAction::StepPlacement(index, direction) => {
                if let Some(modal) = &mut model.placement_modal {
                    modal.step(*index, *direction);
                }
                model.focus = None;
            }
            UiAction::ApplyPlacement(send) => {
                let snapshot = if runtime.destination == Destination::Remote {
                    network.snapshot.as_ref()
                } else {
                    Some(&runtime.snapshot)
                };
                if let Some(snapshot) = snapshot {
                    actions.0.extend(model.apply_placement(snapshot, *send));
                }
            }
            UiAction::Runtime(action) => {
                model.focus = None;
                model.dropdown = None;
                actions.0.push_back(action.clone());
            }
            UiAction::Dropdown(Dropdown::From) => {
                if network.listen_address.is_empty() && network.discovered_listener.is_none() {
                    network.discover_listener();
                }
                model.open_sources(network.listener_endpoint());
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
                        Some(Provider::Host(HostProvider::Native)) => Some(
                            "This launch is controller-only. Restart Durop in local mode to play on this Mac.",
                        ),
                        Some(Provider::Catalog(_)) => Some(
                            "Catalog is supplied by the remote host. Restart Durop in local mode to use local catalog sources.",
                        ),
                        Some(Provider::Host(
                            HostProvider::Tailscale | HostProvider::TailscaleListener,
                        ))
                        | None => None,
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
                            model.open_sources(network.listener_endpoint());
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
            UiAction::SelectSource(selection) => {
                if let Some(modal) = &mut model.modal {
                    modal.select_source(selection.clone());
                }
                model.focus = None;
            }
            UiAction::CancelSource => {
                if let Some(modal) = &mut model.modal {
                    modal.cancel_source();
                }
                model.focus = None;
            }
            UiAction::SourceDetails(details) => {
                if let Some(modal) = &mut model.modal {
                    modal.show_source_details(*details);
                }
                model.focus = None;
            }
            UiAction::Network(action) => network.actions.push_back(action.clone()),
            UiAction::EnableListener => {
                let Some(modal) = &mut model.modal else {
                    continue;
                };
                if !modal.source_fields().contains(&2) {
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
                let ModalDraft::Sources(sources) = &mut modal.draft else {
                    continue;
                };
                if sources.selection != SourceSelection::NewRemote || sources.pending.is_some() {
                    continue;
                }
                if network.controllers.len() >= 16 {
                    modal.error = Some("At most 16 remote sources are supported. Revoke a source before adding another.".into());
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
                sources.pending = Some(PendingSource {
                    name: name.to_owned(),
                    known_ids: network
                        .controllers
                        .iter()
                        .map(|source| source.id.clone())
                        .collect(),
                });
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
                if !modal.source_fields().contains(&1) {
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

        if let Some(field) = model.focus {
            if let Some(entity) = model.editor(field).and_then(|editor| editor.entity) {
                focus.set(entity, FocusCause::Pressed);
            }
        } else {
            focus.clear();
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
    if let Some(scroll) = model.modal_scroll() {
        for (node, mut position) in &mut bodies {
            let maximum =
                ((node.content_size().y - node.size().y) * node.inverse_scale_factor()).max(0.);
            *scroll = scroll.clamp(0., maximum);
            position.y = *scroll;
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
        if let Some(scroll) = model.modal_scroll() {
            for (node, mut position) in &mut bodies {
                let maximum =
                    ((node.content_size().y - node.size().y) * node.inverse_scale_factor()).max(0.);
                *scroll = (position.y - delta).clamp(0., maximum);
                position.y = *scroll;
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
