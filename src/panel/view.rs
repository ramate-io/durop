//! Presentation of the controller collection, provider menus and isolated configuration form.

use super::theme::*;
use super::*;
use bevy::picking::Pickable;
use bevy::text::TextReadWriteMode;
use bevy::ui::RelativeCursorPosition;
use std::{cell::RefCell, collections::HashMap};

/// Style and content identify reusable labels without retaining obsolete display values.
#[derive(Clone, PartialEq, Eq, Hash)]
struct SelectableKey {
    /// Exact displayed value; secrets are never passed to display labels.
    text: String,
    /// Distinguishes the bundled display and interface faces.
    display: bool,
    /// Bitwise logical font size avoids rounding collisions.
    size: u32,
    /// Bitwise sRGBA channels distinguish labels with different contrast.
    color: [u32; 4],
}

/// Keep selection state when an unchanged label moves into the refreshed hierarchy.
#[derive(Clone, Copy)]
struct SelectableText {
    /// A neutral wrapper retains both measured text and its selection overlay.
    root: Entity,
    /// A coincident read-only engine editor owns selection and clipboard behavior.
    selection: Entity,
}

/// Reuse unchanged labels once per render and retire every unused display value.
#[derive(Default)]
pub(super) struct SelectableTexts {
    /// Labels in the previous region hierarchy and available for direct reparenting.
    available: HashMap<SelectableKey, Vec<SelectableText>>,
    /// Labels attached to the current hierarchy, including duplicate values.
    used: HashMap<SelectableKey, Vec<SelectableText>>,
}

impl SelectableTexts {
    /// Stage unchanged labels for direct reparenting within their persistent region.
    fn prepare(&mut self) {
        self.available = std::mem::take(&mut self.used);
        // Reuse identical labels in render order so focus stays on the same occurrence.
        for labels in self.available.values_mut() {
            labels.reverse();
        }
    }

    /// Drop stale labels and release focus if the selected display value disappeared.
    fn finish(&mut self, commands: &mut Commands, focus: &mut InputFocus) {
        for label in self.available.drain().flat_map(|(_, labels)| labels) {
            if focus.get() == Some(label.selection) {
                focus.clear();
            }
            commands.entity(label.root).despawn();
        }
    }
}

/// Construct small reusable paper controls without coupling styling to provider policy.
struct View<'a> {
    /// Shared bundled fonts; widgets only clone lightweight asset handles.
    fonts: &'a Fonts,

    /// Interior mutability lets nested child builders share the current label cache.
    texts: RefCell<&'a mut SelectableTexts>,
}

impl View<'_> {
    /// Add selectable interface text with the same intrinsic layout as ordinary text.
    fn label(
        &self,
        parent: &mut ChildSpawnerCommands,
        text: impl Into<String>,
        size: f32,
        color: Color,
    ) {
        self.selectable(parent, text.into(), size, color, false);
    }

    /// Add selectable display typography for headings and card titles.
    fn title(
        &self,
        parent: &mut ChildSpawnerCommands,
        text: impl Into<String>,
        size: f32,
        color: Color,
    ) {
        self.selectable(parent, text.into(), size, color, true);
    }

    /// Overlay selection on measured text; engine editors otherwise have no intrinsic width.
    fn selectable(
        &self,
        parent: &mut ChildSpawnerCommands,
        text: String,
        size: f32,
        color: Color,
        display: bool,
    ) {
        let key = SelectableKey {
            text,
            display,
            size: size.to_bits(),
            color: color.to_srgba().to_f32_array().map(f32::to_bits),
        };
        let mut texts = self.texts.borrow_mut();
        let retained = texts.available.get_mut(&key).and_then(Vec::pop);
        let label = if let Some(label) = retained {
            let parent_entity = parent.target_entity();
            parent
                .commands()
                .entity(parent_entity)
                .insert_child(usize::MAX, label.root);
            label
        } else {
            let font = if display {
                self.fonts.display(size)
            } else {
                self.fonts.mono(size)
            };
            let root = parent
                .spawn(Node {
                    // Preserve intrinsic minimums; zero minimums with percentage maxima
                    // collapse measured label wrappers to zero width in Taffy.
                    flex_shrink: if display { 1. } else { 0. },
                    ..default()
                })
                .id();
            // Separate text roots prevent the transparent editor from joining the measured
            // label's span/style hierarchy. Ordinary Text remains the sole layout content.
            let measured = parent
                .commands()
                .spawn((
                    Text::new(&key.text),
                    font.clone(),
                    TextColor(color),
                    Node::default(),
                    Pickable::IGNORE,
                ))
                .id();
            parent.commands().entity(root).add_child(measured);
            let selection = parent
                .commands()
                .spawn((
                    TextInput,
                    EditableText {
                        visible_lines: None,
                        allow_newlines: true,
                        cursor_margin: Vec2::ZERO,
                        cursor_width: 0.,
                        ..EditableText::new(&key.text)
                    },
                    TextReadWriteMode::ReadOnly,
                    font,
                    // The measured label supplies ordinary glyphs. Only selection is painted here.
                    TextColor(Color::NONE),
                    TextCursorStyle {
                        selection_color: PURPLE.with_alpha(0.28),
                        // Bevy uses this color for read-only selections even while focused.
                        unfocused_selection_color: PURPLE.with_alpha(0.28),
                        selected_text_color: None,
                        ..default()
                    },
                    TabIndex(-1),
                    Node {
                        position_type: PositionType::Absolute,
                        left: px(0),
                        top: px(0),
                        width: percent(100),
                        height: percent(100),
                        ..default()
                    },
                ))
                .id();
            parent.commands().entity(root).add_child(selection);
            SelectableText { root, selection }
        };
        texts.used.entry(key).or_default().push(label);
    }

    /// Keep action labels in their parent's hit target instead of intercepting selection drags.
    fn action_label(
        &self,
        parent: &mut ChildSpawnerCommands,
        text: impl Into<String>,
        size: f32,
        color: Color,
    ) {
        parent.spawn((
            Text::new(text),
            self.fonts.mono(size),
            TextColor(color),
            Node {
                flex_shrink: 0.,
                ..default()
            },
        ));
    }

    /// Separate logical form sections without adding a border to every individual field.
    fn rule(&self, parent: &mut ChildSpawnerCommands) {
        parent.spawn((
            Node {
                width: percent(100),
                height: px(1),
                min_height: px(1),
                flex_shrink: 0.,
                ..default()
            },
            BackgroundColor(RULE),
            Pickable::IGNORE,
        ));
    }

    /// Attach the persistent engine editor to its freshly rendered field slot.
    fn input(&self, parent: &mut ChildSpawnerCommands, editor: &Editor) {
        if let Some(entity) = editor.entity {
            let parent_entity = parent.target_entity();
            parent.commands().entity(parent_entity).add_child(entity);
        }
    }

    /// Add a button with a comfortable hit target and bounded hover feedback.
    fn button(
        &self,
        parent: &mut ChildSpawnerCommands,
        label: impl Into<String>,
        action: UiAction,
        primary: bool,
    ) {
        let (background, foreground, hovered) = if primary {
            (INK, PAPER, PURPLE)
        } else {
            (PAPER, INK, BACKGROUND)
        };
        parent
            .spawn((
                Button,
                ActivateOnPress,
                Hovered::default(),
                action,
                Node {
                    min_height: px(38),
                    padding: UiRect::axes(px(16), px(9)),
                    border: UiRect::all(px(1)),
                    border_radius: BorderRadius::all(px(5)),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    ..default()
                },
                BackgroundColor(background),
                BorderColor::all(if primary { INK } else { RULE }),
                HoverColors {
                    resting: background,
                    hovered,
                },
            ))
            .with_children(|button| self.action_label(button, label, 12., foreground));
    }

    /// Render a provider selector with its popup anchored directly below the control.
    fn selector(
        &self,
        parent: &mut ChildSpawnerCommands,
        label: &str,
        value: &str,
        menu: Dropdown,
        open: bool,
        width: f32,
    ) {
        parent
            .spawn(Node {
                width: px(width),
                flex_shrink: 1.,
                min_width: px(150),
                ..default()
            })
            .with_children(|container| {
                container
                    .spawn((
                        Button,
                        ActivateOnPress,
                        Hovered::default(),
                        UiAction::Dropdown(menu),
                        Node {
                            width: percent(100),
                            height: px(68),
                            min_height: px(68),
                            max_height: px(68),
                            padding: UiRect::axes(px(16), px(11)),
                            border: UiRect::all(px(1)),
                            border_radius: BorderRadius::all(px(6)),
                            flex_direction: FlexDirection::Column,
                            row_gap: px(6),
                            justify_content: JustifyContent::Center,
                            ..default()
                        },
                        BackgroundColor(PAPER),
                        BorderColor::all(if open { PURPLE } else { RULE }),
                        HoverColors {
                            resting: PAPER,
                            hovered: BACKGROUND,
                        },
                    ))
                    .with_children(|button| {
                        self.action_label(button, label, 10., MUTED);
                        button
                            .spawn(Node {
                                width: percent(100),
                                justify_content: JustifyContent::SpaceBetween,
                                align_items: AlignItems::Center,
                                column_gap: px(8),
                                ..default()
                            })
                            .with_children(|row| {
                                self.action_label(row, value, 13., INK);
                                self.action_label(row, if open { "−" } else { "+" }, 17., PURPLE);
                            });
                    });
                if open {
                    container
                        .spawn((
                            Node {
                                position_type: PositionType::Absolute,
                                top: px(76),
                                left: if menu == Dropdown::To { px(0) } else { auto() },
                                right: if menu == Dropdown::Catalog {
                                    px(0)
                                } else {
                                    auto()
                                },
                                width: px(width.max(260.)),
                                padding: UiRect::all(px(8)),
                                border: UiRect::all(px(1)),
                                border_radius: BorderRadius::all(px(6)),
                                flex_direction: FlexDirection::Column,
                                row_gap: px(4),
                                ..default()
                            },
                            BackgroundColor(PAPER),
                            BorderColor::all(RULE),
                            GlobalZIndex(30),
                            Pickable::default(),
                        ))
                        .with_children(|popup| {
                            let entries = match menu {
                                Dropdown::From => vec![],
                                Dropdown::To => vec![
                                    (
                                        "This Mac · choose display",
                                        Provider::Host(HostProvider::Native),
                                    ),
                                    (
                                        "Remote host · IP + pairing token",
                                        Provider::Host(HostProvider::Tailscale),
                                    ),
                                ],
                                Dropdown::Catalog => vec![
                                    (
                                        "Durop reactions",
                                        Provider::Catalog(CatalogProvider::Bundled),
                                    ),
                                    (
                                        "Local directory…",
                                        Provider::Catalog(CatalogProvider::LocalDirectory),
                                    ),
                                    (
                                        "Git repository…",
                                        Provider::Catalog(CatalogProvider::GitHttps),
                                    ),
                                ],
                            };
                            for (label, provider) in entries {
                                self.button(popup, label, UiAction::Configure(provider), false);
                            }
                        });
                }
            });
    }

    /// Show an actual clip image at preserved aspect ratio inside a large preview well.
    #[expect(
        clippy::too_many_arguments,
        reason = "One bounded card combines layout with decoded preview and playback state"
    )]
    fn card(
        &self,
        parent: &mut ChildSpawnerCommands,
        clip: &Clip,
        image: Option<Handle<Image>>,
        dimensions: Option<Vec2>,
        layout: GridLayout,
        status: &str,
        stop: Option<&str>,
        playing: bool,
        editable: bool,
    ) {
        let feedback = CardFeedback { playing };
        parent
            .spawn((
                Node {
                    width: px(layout.card_width),
                    height: px(layout.card_height),
                    padding: UiRect::all(px(12)),
                    border: UiRect::all(px(1)),
                    border_radius: BorderRadius::all(px(8)),
                    flex_direction: FlexDirection::Column,
                    row_gap: px(10),
                    overflow: Overflow::clip(),
                    ..default()
                },
                BackgroundColor(feedback.background()),
                BorderColor::all(if playing { PLAYING_RULE } else { RULE }),
                feedback.shadow(false),
                feedback,
                Hovered::default(),
            ))
            .with_children(|card| {
                card.spawn((
                    Button,
                    ActivateOnPress,
                    Hovered::default(),
                    UiAction::Play(clip.identity.clone()),
                    Node {
                        width: percent(100),
                        height: px(layout.preview_height),
                        min_height: px(layout.preview_height),
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::Center,
                        border_radius: BorderRadius::all(px(4)),
                        overflow: Overflow::clip(),
                        ..default()
                    },
                    BackgroundColor(BACKGROUND),
                ))
                .with_children(|preview| {
                    if let Some(image) = image {
                        let dimensions = dimensions.unwrap_or(Vec2::splat(1.));
                        let scale = ((layout.card_width - 26.) / dimensions.x)
                            .min(layout.preview_height / dimensions.y);
                        preview.spawn((
                            ImageNode::new(image),
                            Node {
                                width: px(dimensions.x * scale),
                                height: px(dimensions.y * scale),
                                flex_shrink: 0.,
                                flex_direction: FlexDirection::Column,
                                ..default()
                            },
                        ));
                    } else {
                        self.action_label(preview, "PREVIEW UNAVAILABLE", 10., MUTED);
                    }
                });
                card.spawn(Node {
                    width: percent(100),
                    height: px(44),
                    min_height: px(44),
                    flex_shrink: 0.,
                    overflow: Overflow::clip(),
                    ..default()
                })
                .with_children(|title| {
                    self.title(title, &clip.metadata.title, 17., INK);
                });
                let kind = match std::path::Path::new(&clip.metadata.asset)
                    .extension()
                    .and_then(|extension| extension.to_str())
                {
                    Some("gif") => "GIF",
                    Some("wav" | "mp3" | "m4a" | "aac" | "flac" | "ogg") => "AUDIO",
                    _ => "VIDEO",
                };
                card.spawn(Node {
                    width: percent(100),
                    justify_content: JustifyContent::SpaceBetween,
                    ..default()
                })
                .with_children(|row| {
                    self.action_label(row, kind, 9., MUTED);
                    if let Some(instance) = stop {
                        row.spawn((
                            Button,
                            ActivateOnPress,
                            Hovered::default(),
                            UiAction::Runtime(Action::Stop(instance.to_owned())),
                            Node {
                                padding: UiRect::axes(px(6), px(1)),
                                border_radius: BorderRadius::all(px(3)),
                                ..default()
                            },
                            BackgroundColor(PURPLE),
                            HoverColors {
                                resting: PURPLE,
                                hovered: INK,
                            },
                        ))
                        .with_children(|stop| self.action_label(stop, "STOP", 9., PAPER));
                    }
                    if editable {
                        self.card_control(
                            row,
                            "Edit",
                            UiAction::EditPlacement(clip.identity.clone()),
                            false,
                        );
                    }
                    self.card_control(row, status, UiAction::Play(clip.identity.clone()), true);
                });
            });
    }

    /// Keep card actions distinct and compact enough for narrow collection columns.
    fn card_control(
        &self,
        parent: &mut ChildSpawnerCommands,
        label: &str,
        action: UiAction,
        primary: bool,
    ) {
        parent
            .spawn((
                Button,
                ActivateOnPress,
                Hovered::default(),
                action,
                Node {
                    padding: UiRect::axes(px(6), px(2)),
                    border_radius: BorderRadius::all(px(3)),
                    ..default()
                },
                BackgroundColor(if primary { PURPLE } else { BACKGROUND }),
                HoverColors {
                    resting: if primary { PURPLE } else { BACKGROUND },
                    hovered: RULE,
                },
            ))
            .with_children(|button| {
                self.action_label(button, label, 9., if primary { PAPER } else { INK })
            });
    }

    /// Render editable settings and validation feedback inside the bounded form body.
    fn modal_fields(
        &self,
        dialog: &mut ChildSpawnerCommands,
        modal: &Modal,
        focus: Option<Field>,
        network: &crate::network_app::NetworkState,
        runtime: &Runtime,
    ) {
        let ModalDraft::Configuration(draft) = &modal.draft else {
            self.sources_fields(dialog, modal, focus, network);
            return;
        };
        for (index, field) in draft.fields.iter().enumerate() {
            dialog
                .spawn(Node {
                    flex_direction: FlexDirection::Column,
                    row_gap: px(8),
                    ..default()
                })
                .with_children(|row| {
                    self.label(row, field.label, 11., MUTED);
                    let editor = &modal.editors[index];
                    let selected = focus == Some(Field::Configuration(index));
                    let value = if !field.choices.is_empty() {
                        field
                            .choices
                            .iter()
                            .find(|choice| choice.value == editor.text)
                            .map(|choice| format!("{}  +", choice.label))
                            .unwrap_or_else(|| editor.text.clone())
                    } else {
                        editor.text.clone()
                    };
                    let node = Node {
                        width: percent(100),
                        min_height: px(48),
                        padding: UiRect::axes(px(14), px(13)),
                        border: UiRect::all(px(1)),
                        border_radius: BorderRadius::all(px(5)),
                        overflow: Overflow::clip(),
                        ..default()
                    };
                    if field.read_only {
                        row.spawn((node, BackgroundColor(BACKGROUND), BorderColor::all(RULE)))
                            .with_children(|input| self.label(input, value, 12., MUTED));
                    } else if field.choices.is_empty() {
                        self.input(row, editor);
                    } else {
                        let action = UiAction::Cycle(index);
                        row.spawn((
                            Button,
                            ActivateOnPress,
                            Hovered::default(),
                            action,
                            node,
                            BackgroundColor(BACKGROUND),
                            BorderColor::all(if selected { PURPLE } else { RULE }),
                            HoverColors {
                                resting: BACKGROUND,
                                hovered: PAPER,
                            },
                        ))
                        .with_children(|input| self.action_label(input, value, 12., INK));
                    }
                    self.label(row, field.hint, 10., MUTED);
                });
        }
        if matches!(draft.provider, Provider::Catalog(_))
            && runtime.destination == Destination::Remote
        {
            self.label(dialog, "Catalog is supplied by the remote host. Choose To → This Mac before changing local catalog sources.", 12., PURPLE);
        }
        if draft.provider == Provider::Host(HostProvider::Native)
            && runtime.mode == Mode::Controller
        {
            self.label(
                dialog,
                "This launch is controller-only. Restart in local mode to play on this Mac.",
                12.,
                PURPLE,
            );
        }
        if let Some(error) = &modal.error {
            self.label(dialog, error, 12., PURPLE);
        }
    }

    /// Render a source editor using the same keyboard, IME and masked-input behavior as providers.
    fn source_editor(
        &self,
        parent: &mut ChildSpawnerCommands,
        modal: &Modal,
        focus: Option<Field>,
        index: usize,
        label: &str,
        secret: bool,
    ) {
        self.label(parent, label, 11., MUTED);
        let _ = (focus, secret);
        self.input(parent, &modal.editors[index]);
    }

    /// Render one configured source's controls; selecting it never changes permission.
    fn controller_source(
        &self,
        dialog: &mut ChildSpawnerCommands,
        source: &crate::network_app::ControllerSource,
        network: &crate::network_app::NetworkState,
    ) {
        use crate::network_app::NetworkAction;
        self.label(dialog, "TAILSCALE CONTROLLER", 10., PURPLE);
        self.label(dialog, &source.name, 16., INK);
        self.label(dialog, &source.invitation_status, 11., MUTED);
        dialog
            .spawn(Node {
                column_gap: px(8),
                row_gap: px(8),
                flex_wrap: FlexWrap::Wrap,
                ..default()
            })
            .with_children(|row| {
                self.button(
                    row,
                    if source.enabled {
                        "Enabled · Disable"
                    } else {
                        "Disabled · Enable"
                    },
                    UiAction::Network(NetworkAction::SetControllerEnabled {
                        id: source.id.clone(),
                        enabled: !source.enabled,
                    }),
                    false,
                );
                self.button(
                    row,
                    "Revoke controller",
                    UiAction::Network(NetworkAction::RevokeController {
                        id: source.id.clone(),
                    }),
                    false,
                );
            });
        if network.hosting && source.pairing_secret.is_some() {
            self.button(
                dialog,
                if source.invitation_url.is_some() {
                    "Copy invitation link + pairing details"
                } else {
                    "Copy Durop pairing details"
                },
                UiAction::Network(NetworkAction::CopyControllerHandoff {
                    id: source.id.clone(),
                }),
                false,
            );
        } else if !network.hosting {
            self.label(
                dialog,
                "Enable the shared listener to receive remote commands and copy pairing details.",
                10.,
                MUTED,
            );
        }
        if network.hosting {
            dialog
                .spawn(Node {
                    column_gap: px(8),
                    row_gap: px(8),
                    flex_wrap: FlexWrap::Wrap,
                    ..default()
                })
                .with_children(|row| {
                    if network.admin_secret_available {
                        self.button(
                            row,
                            if source.invitation_url.is_some() {
                                "Replace network invitation"
                            } else {
                                "Create network invitation"
                            },
                            UiAction::Network(NetworkAction::CreateInvitation {
                                id: source.id.clone(),
                            }),
                            false,
                        );
                    }
                    if source.invitation_url.is_some() {
                        self.button(
                            row,
                            "Revoke invitation",
                            UiAction::Network(NetworkAction::RevokeInvitation {
                                id: source.id.clone(),
                            }),
                            false,
                        );
                        self.button(
                            row,
                            "Refresh status",
                            UiAction::Network(NetworkAction::RefreshInvitations),
                            false,
                        );
                    }
                });
        }
        self.label(dialog, "Durop pairing and Tailscale device access are separate. Revoking this controller ends Durop permission; accepted device access may also need removal in Tailscale.", 10., MUTED);
    }

    /// Source chips indicate every permission independently of the selected form.
    fn source_selector(
        &self,
        dialog: &mut ChildSpawnerCommands,
        sources: &SourcesDraft,
        network: &crate::network_app::NetworkState,
    ) {
        dialog
            .spawn(Node {
                width: percent(100),
                column_gap: px(8),
                row_gap: px(8),
                flex_wrap: FlexWrap::Wrap,
                ..default()
            })
            .with_children(|row| {
                self.button(
                    row,
                    format!(
                        "Local · {}",
                        if network.local_enabled { "On" } else { "Off" }
                    ),
                    UiAction::SelectSource(SourceSelection::Local),
                    sources.selection == SourceSelection::Local,
                );
                for source in &network.controllers {
                    self.button(
                        row,
                        format!(
                            "{} · {}",
                            source.name,
                            if source.enabled { "On" } else { "Off" }
                        ),
                        UiAction::SelectSource(SourceSelection::Remote(source.id.clone())),
                        sources.selection == SourceSelection::Remote(source.id.clone()),
                    );
                }
            });
    }

    /// Reveal shared listener or API-key settings only from an existing Tailscale source's form.
    fn source_details(
        &self,
        dialog: &mut ChildSpawnerCommands,
        modal: &Modal,
        focus: Option<Field>,
        sources: &SourcesDraft,
        network: &crate::network_app::NetworkState,
    ) {
        use crate::network_app::NetworkAction;
        dialog
            .spawn(Node {
                column_gap: px(8),
                row_gap: px(8),
                flex_wrap: FlexWrap::Wrap,
                ..default()
            })
            .with_children(|row| {
                self.button(
                    row,
                    "Listener settings",
                    UiAction::SourceDetails(SourceDetails::Listener),
                    sources.details == SourceDetails::Listener,
                );
                self.button(
                    row,
                    "Invitation credentials",
                    UiAction::SourceDetails(SourceDetails::Credentials),
                    sources.details == SourceDetails::Credentials,
                );
            });
        match sources.details {
            SourceDetails::None => (),
            SourceDetails::Listener => {
                self.rule(dialog);
                self.label(
                    dialog,
                    "LISTENER · SHARED BY TAILSCALE SOURCES",
                    10.,
                    PURPLE,
                );
                self.source_editor(
                    dialog,
                    modal,
                    focus,
                    2,
                    "This Mac's Tailscale IP:port",
                    false,
                );
                self.label(dialog, network.listener_hint(), 10., MUTED);
                self.button(
                    dialog,
                    if network.hosting {
                        "Apply listener address"
                    } else {
                        "Enable remote listener"
                    },
                    UiAction::EnableListener,
                    false,
                );
                if network.hosting {
                    self.label(
                        dialog,
                        format!("Listening at {}", network.listen_address),
                        11.,
                        GREEN,
                    );
                    self.button(
                        dialog,
                        "Disable remote listener",
                        UiAction::Network(NetworkAction::DisableHost),
                        false,
                    );
                }
                self.label(
                    dialog,
                    "The selected To destination stays independent of this listener.",
                    10.,
                    MUTED,
                );
                self.button(
                    dialog,
                    "Back to controller",
                    UiAction::SourceDetails(SourceDetails::Listener),
                    false,
                );
            }
            SourceDetails::Credentials => {
                self.rule(dialog);
                self.label(
                    dialog,
                    "INVITATION CREDENTIALS · SHARED BY TAILSCALE SOURCES",
                    10.,
                    PURPLE,
                );
                self.label(
                    dialog,
                    &network.admin_status,
                    11.,
                    if network.admin_secret_available {
                        GREEN
                    } else {
                        MUTED
                    },
                );
                self.source_editor(dialog, modal, focus, 1, "Host Tailscale API key", true);
                self.label(dialog, "Use TAILSCALE_API_KEY or save a key in this Mac's secure store. Keys are never included in pairing details.", 10., MUTED);
                dialog
                    .spawn(Node {
                        column_gap: px(8),
                        row_gap: px(8),
                        flex_wrap: FlexWrap::Wrap,
                        ..default()
                    })
                    .with_children(|row| {
                        self.button(row, "Save key securely", UiAction::SaveAdminSecret, false);
                        self.button(
                            row,
                            "Remove saved key",
                            UiAction::Network(NetworkAction::RemoveAdminSecret),
                            false,
                        );
                    });
                self.button(
                    dialog,
                    "Back to controller",
                    UiAction::SourceDetails(SourceDetails::Credentials),
                    false,
                );
            }
        }
    }

    /// Render just the selected provider form beneath the source selector.
    fn sources_fields(
        &self,
        dialog: &mut ChildSpawnerCommands,
        modal: &Modal,
        focus: Option<Field>,
        network: &crate::network_app::NetworkState,
    ) {
        use crate::network_app::NetworkAction;
        let ModalDraft::Sources(sources) = &modal.draft else {
            return;
        };
        self.source_selector(dialog, sources, network);
        self.rule(dialog);
        match &sources.selection {
            SourceSelection::Local => {
                self.label(dialog, "LOCAL CONTROLS", 10., PURPLE);
                self.label(dialog, "Buttons and shortcuts on this Mac.", 12., INK);
                self.button(
                    dialog,
                    if network.local_enabled {
                        "Enabled · Disable"
                    } else {
                        "Disabled · Enable"
                    },
                    UiAction::Network(NetworkAction::SetLocalEnabled(!network.local_enabled)),
                    false,
                );
                self.label(dialog, "Local controls and remote controllers can stay enabled together. Commands use the selected To destination.", 10., MUTED);
            }
            SourceSelection::Remote(id) => {
                if let Some(source) = network.controllers.iter().find(|source| &source.id == id) {
                    self.controller_source(dialog, source, network);
                    self.rule(dialog);
                    self.source_details(dialog, modal, focus, sources, network);
                }
            }
            SourceSelection::ChooseType => {
                self.label(dialog, "ADD A SOURCE", 10., PURPLE);
                self.button(
                    dialog,
                    "Local controls · already configured",
                    UiAction::SelectSource(SourceSelection::Local),
                    false,
                );
                self.button(
                    dialog,
                    "Tailscale controller…",
                    UiAction::SelectSource(SourceSelection::NewRemote),
                    false,
                );
                self.button(dialog, "Cancel", UiAction::CancelSource, false);
            }
            SourceSelection::NewRemote => {
                self.label(dialog, "NEW TAILSCALE CONTROLLER", 10., PURPLE);
                if let Some(pending) = &sources.pending {
                    self.label(dialog, format!("Adding {}…", pending.name), 12., MUTED);
                } else {
                    self.source_editor(dialog, modal, focus, 0, "Controller name", false);
                    self.label(dialog, "Add creates an individual Durop pairing token. It leaves your other sources enabled.", 10., MUTED);
                    self.button(dialog, "Add controller", UiAction::AddController, true);
                }
                dialog
                    .spawn(Node {
                        column_gap: px(8),
                        row_gap: px(8),
                        flex_wrap: FlexWrap::Wrap,
                        ..default()
                    })
                    .with_children(|row| {
                        self.button(
                            row,
                            "Back",
                            UiAction::SelectSource(SourceSelection::ChooseType),
                            false,
                        );
                        self.button(row, "Cancel", UiAction::CancelSource, false);
                    });
            }
        }
        if let Some(error) = &modal.error {
            self.label(dialog, error, 12., PURPLE);
        }
    }

    /// Render native percentage editors inside the same bounded, focus-trapped modal contract.
    fn placement_modal(&self, root: &mut ChildSpawnerCommands, modal: &PlacementModal) {
        root.spawn((
            Node {
                position_type: PositionType::Absolute,
                left: px(0), top: px(0), width: percent(100), height: percent(100),
                align_items: AlignItems::Center, justify_content: JustifyContent::Center,
                padding: UiRect::all(px(24)), ..default()
            },
            BackgroundColor(Color::srgba(0.102, 0.102, 0.102, 0.48)),
            GlobalZIndex(100), Pickable::default(),
        )).with_children(|backdrop| {
            // The cancel target is a sibling, so editor and control events never cancel the form.
            backdrop.spawn((Button, ActivateOnPress, UiAction::Cancel, Node {
                position_type: PositionType::Absolute, width: percent(100), height: percent(100), ..default()
            }));
            backdrop.spawn((GlobalZIndex(101), TabGroup::modal(), Node {
                width: px(650), max_width: percent(100), max_height: percent(100),
                padding: UiRect::all(px(24)), border: UiRect::all(px(1)),
                border_radius: BorderRadius::all(px(10)), flex_direction: FlexDirection::Column,
                row_gap: px(16), ..default()
            }, BackgroundColor(PAPER), BorderColor::all(RULE), Pickable::default()))
            .with_children(|dialog| {
                self.label(dialog, "VISUAL PLACEMENT · EXPERIMENTAL", 10., PURPLE);
                self.title(dialog, format!("Edit {}", modal.title), 26., INK);
                self.label(dialog, &modal.clip_id, 10., MUTED);
                self.rule(dialog);
                dialog.spawn((ModalBody, ScrollPosition(Vec2::new(0., modal.scroll)), Node {
                    width: percent(100), min_height: px(0), flex_direction: FlexDirection::Column,
                    overflow: Overflow::scroll_y(), ..default()
                })).with_children(|body| {
                    body.spawn(Node {
                        width: percent(100), flex_direction: FlexDirection::Column,
                        row_gap: px(14), flex_shrink: 0., ..default()
                    }).with_children(|fields| {
                        self.label(fields, "Percentages use the host display. Position keeps media on screen.", 11., MUTED);
                        self.label(fields, match modal.fit {
                            VisualFit::Catalog => "Fitting follows the catalog. Fullscreen fills the display with cropping.",
                            VisualFit::Cover => "Cover fitting: fills the placement area with centered cropping. Reset restores catalog fitting.",
                        }, 10., MUTED);
                        fields.spawn(Node {
                            width: percent(100), column_gap: px(8), row_gap: px(8),
                            flex_wrap: FlexWrap::Wrap, ..default()
                        }).with_children(|presets| {
                            for (label, preset) in [
                                ("Fullscreen", PlacementPreset::Fullscreen),
                                ("Top left", PlacementPreset::TopLeft),
                                ("Top right", PlacementPreset::TopRight),
                                ("Bottom left", PlacementPreset::BottomLeft),
                                ("Bottom right", PlacementPreset::BottomRight),
                            ] {
                                self.button(presets, label, UiAction::PresetPlacement(preset), false);
                            }
                        });
                        for (index, label) in ["Scale (% of host width)", "Horizontal (% left → right)", "Vertical (% top → bottom)"].into_iter().enumerate() {
                            self.label(fields, label, 11., MUTED);
                            fields.spawn(Node {
                                width: percent(100), min_width: px(0), column_gap: px(8),
                                align_items: AlignItems::Center, ..default()
                            }).with_children(|row| {
                                self.button(row, "−", UiAction::StepPlacement(index, -1), false);
                                self.input(row, &modal.editors[index]);
                                self.button(row, "+", UiAction::StepPlacement(index, 1), false);
                            });
                        }
                        self.label(fields, "Apply updates active plays and remembers this session. Send also requests playback.", 11., MUTED);
                        if modal.reset && modal.placement().ok() == Some(modal.defaults) {
                            self.label(fields, "Catalog defaults selected. Apply or Send to restore them.", 11., GREEN);
                        }
                        if let Some(error) = &modal.error { self.label(fields, error, 11., PURPLE); }
                    });
                });
                dialog.spawn((Node {
                    width: percent(100), column_gap: px(8), row_gap: px(8),
                    border: UiRect::top(px(1)), padding: UiRect::top(px(16)),
                    flex_wrap: FlexWrap::Wrap, flex_shrink: 0., ..default()
                }, BorderColor::all(RULE))).with_children(|footer| {
                    self.button(footer, "Reset to defaults", UiAction::ResetPlacement, false);
                    self.button(footer, "Cancel", UiAction::Cancel, false);
                    self.button(footer, "Apply", UiAction::ApplyPlacement(false), false);
                    self.button(footer, "Send →", UiAction::ApplyPlacement(true), true);
                    self.button(footer, "STOP ALL", UiAction::Runtime(Action::StopAll), false);
                });
            });
        });
    }

    /// Render provider-specific fields over a focus-blocking, dimmed collection.
    fn modal(
        &self,
        root: &mut ChildSpawnerCommands,
        modal: &Modal,
        focus: Option<Field>,
        network: &crate::network_app::NetworkState,
        runtime: &Runtime,
    ) {
        let (title, subtitle) = match modal.provider() {
            None => (
                "From",
                "Select a source to edit. Sources can stay enabled together.",
            ),
            Some(Provider::Host(HostProvider::Native)) => (
                "To this Mac",
                "Play on this Mac. Choose where your reactions appear.",
            ),
            Some(Provider::Host(HostProvider::Tailscale)) => (
                Provider::Host(HostProvider::Tailscale).label(),
                "Connect to an authenticated host on your private network.",
            ),
            Some(Provider::Host(HostProvider::TailscaleListener)) => (
                "Share this Mac",
                "Allow another Durop controller to pair with this host over Tailscale.",
            ),
            Some(Provider::Catalog(CatalogProvider::Bundled)) => (
                "Durop reactions",
                "A collection that travels with your app. Ready to play offline.",
            ),
            Some(Provider::Catalog(CatalogProvider::LocalDirectory)) => {
                ("Local catalog", "Load a pack from a directory on this Mac.")
            }
            Some(Provider::Catalog(CatalogProvider::GitHttps)) => (
                "Git repository",
                "Pin a catalog from a repository, branch or commit.",
            ),
        };
        root.spawn((
            Node {
                position_type: PositionType::Absolute,
                left: px(0),
                top: px(0),
                width: percent(100),
                height: percent(100),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                padding: UiRect::all(px(24)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.102, 0.102, 0.102, 0.48)),
            GlobalZIndex(100),
            Pickable::default(),
        ))
        .with_children(|backdrop| {
            backdrop.spawn((
                Button,
                ActivateOnPress,
                UiAction::Cancel,
                Node { position_type: PositionType::Absolute, width: percent(100), height: percent(100), ..default() },
            ));
            backdrop
                .spawn((
                    GlobalZIndex(101),
                    TabGroup::modal(),
                    Node {
                        width: px(650),
                        max_width: percent(100),
                        max_height: percent(100),
                        padding: UiRect::all(px(32)),
                        border: UiRect::all(px(1)),
                        border_radius: BorderRadius::all(px(10)),
                        flex_direction: FlexDirection::Column,
                        row_gap: px(22),
                        ..default()
                    },
                    BackgroundColor(PAPER),
                    BorderColor::all(RULE),
                    Pickable::default(),
                ))
                .with_children(|dialog| {
                    dialog
                        .spawn(Node {
                            flex_shrink: 0.,
                            justify_content: JustifyContent::SpaceBetween,
                            align_items: AlignItems::Center,
                            ..default()
                        })
                        .with_children(|heading| {
                            self.title(heading, title, 31., INK);
                            self.button(heading, "Close ×", UiAction::Cancel, false);
                        });
                    self.label(dialog, subtitle, 12., MUTED);
                    // Keep decoration outside the viewport so its size is the exact
                    // scrollable content height used when clamping the last action into view.
                    self.rule(dialog);
                    dialog
                        .spawn((
                            ModalBody,
                            ScrollPosition(Vec2::new(0., modal.scroll)),
                            Node {
                                width: percent(100),
                                min_height: px(0),
                                flex_direction: FlexDirection::Column,
                                overflow: Overflow::scroll_y(),
                                ..default()
                            },
                        ))
                        .with_children(|body| {
                            body.spawn(Node {
                                width: percent(100),
                                flex_direction: FlexDirection::Column,
                                row_gap: px(22),
                                flex_shrink: 0.,
                                ..default()
                            })
                            .with_children(|dialog| {
                                self.modal_fields(dialog, modal, focus, network, runtime);
                            });
                        });
                    dialog
                        .spawn(Node {
                            width: percent(100),
                            flex_shrink: 0.,
                            min_height: px(38),
                            border: UiRect::top(px(1)),
                            padding: UiRect::top(px(16)),
                            justify_content: JustifyContent::SpaceBetween,
                            align_items: AlignItems::Center,
                            margin: UiRect::top(px(6)),
                            ..default()
                        })
                        .insert(BorderColor::all(RULE))
                        .with_children(|footer| {
                            self.button(
                                footer,
                                "STOP ALL",
                                UiAction::Runtime(Action::StopAll),
                                false,
                            );
                            footer
                                .spawn(Node {
                                    column_gap: px(10),
                                    ..default()
                                })
                                .with_children(|actions| {
                                    if modal.provider().is_some() {
                                        self.button(actions, "Cancel", UiAction::Cancel, false);
                                        self.button(actions, "Apply →", UiAction::Apply, true);
                                    } else {
                                        let adding = matches!(&modal.draft, ModalDraft::Sources(sources)
                                            if matches!(sources.selection, SourceSelection::ChooseType | SourceSelection::NewRemote));
                                        self.button(actions, "+", UiAction::SelectSource(SourceSelection::ChooseType), adding);
                                        self.button(actions, "Done", UiAction::Cancel, true);
                                    }
                                });
                        });
                });
        });
    }
}

/// Stable ownership boundaries keep unrelated UI text attached while another region changes.
#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum RegionKind {
    /// Application name and optional tagline.
    Brand,
    /// Provider selectors and the persistent search editor.
    Header,
    /// Collection heading and match count.
    Section,
    /// Virtualized collection rows inside the persistent viewport.
    Collection,
    /// Application status and emergency controls.
    Footer,
    /// Independently refreshed modal hierarchy.
    Modal,
}

/// One persistent parent and the selectable text owned by its replaceable children.
#[derive(Default)]
struct RenderRegion {
    /// Parent remains attached to the live UI camera across content refreshes.
    root: Option<Entity>,
    /// Only presentation state used by this region participates in invalidation.
    signature: Option<String>,
    /// Selection survives a region refresh by moving labels before old parents retire.
    texts: SelectableTexts,
}

impl RenderRegion {
    /// Construct the region under an already retained root.
    fn spawn(commands: &mut Commands, parent: Entity, kind: RegionKind, node: Node) -> Self {
        let root = commands.spawn((kind, node)).id();
        commands.entity(parent).add_child(root);
        Self {
            root: Some(root),
            ..default()
        }
    }

    /// Report structural changes without treating text edits in another region as invalidation.
    fn changed(&self, signature: &str) -> bool {
        self.signature.as_deref() != Some(signature)
    }

    /// Reparent reusable leaves before despawning old scaffolds; the parent never loses its camera.
    fn refresh(
        &mut self,
        commands: &mut Commands,
        children: &Query<&Children>,
        fonts: &Fonts,
        focus: &mut InputFocus,
        signature: String,
        fill: impl FnOnce(&View, &mut ChildSpawnerCommands),
    ) {
        if !self.changed(&signature) {
            return;
        }
        let root = self
            .root
            .expect("render regions are initialized before refresh");
        let old_children: Vec<_> = children
            .get(root)
            .map(|children| children.iter().collect())
            .unwrap_or_default();
        self.texts.prepare();
        // Direct child labels are retired by their cache, rather than by scaffold teardown.
        let owned_labels: Vec<_> = self
            .texts
            .available
            .values()
            .flatten()
            .map(|label| label.root)
            .collect();
        {
            let view = View {
                fonts,
                texts: RefCell::new(&mut self.texts),
            };
            commands
                .entity(root)
                .with_children(|parent| fill(&view, parent));
        }
        self.texts.finish(commands, focus);
        for child in old_children {
            if !owned_labels.contains(&child) {
                commands.entity(child).despawn();
            }
        }
        self.signature = Some(signature);
    }
}

/// Persistent chrome, viewport and modal parents; each cache owns only its region's labels.
#[derive(Default)]
pub(super) struct RenderRegions {
    /// Brand typography never participates in query or scroll invalidation.
    brand: RenderRegion,
    /// Provider controls own the search editor's parent.
    header: RenderRegion,
    /// Match-count labels can change without changing toolbar geometry.
    section: RenderRegion,
    /// Only virtualized rows are replaced; viewport and scrollbar stay attached.
    collection: RenderRegion,
    /// Playback/network statuses do not rebuild input widgets.
    footer: RenderRegion,
    /// Ordinary draft edits leave the modal hierarchy intact.
    modal: RenderRegion,
    /// Thumb availability color follows collection extent without replacing its entity.
    thumb: Option<Entity>,
}

impl RenderRegions {
    /// Fixed toolbar bounds prevent the search editor from stretching its row.
    fn header_node(height: f32) -> Node {
        Node {
            width: percent(100),
            height: px(height),
            min_height: px(height),
            max_height: px(height),
            flex_shrink: 0.,
            column_gap: px(14),
            row_gap: px(12),
            flex_wrap: FlexWrap::Wrap,
            align_items: AlignItems::Stretch,
            ..default()
        }
    }

    /// Build the stable layout once; future updates replace only the owning region's children.
    fn initialize(
        &mut self,
        commands: &mut Commands,
        root: Entity,
        model: &Model,
        toolbar_height: f32,
    ) {
        self.brand = RenderRegion::spawn(
            commands,
            root,
            RegionKind::Brand,
            Node {
                height: px(32),
                min_height: px(32),
                justify_content: JustifyContent::SpaceBetween,
                align_items: AlignItems::Center,
                ..default()
            },
        );
        self.header = RenderRegion::spawn(
            commands,
            root,
            RegionKind::Header,
            Self::header_node(toolbar_height),
        );
        self.section = RenderRegion::spawn(
            commands,
            root,
            RegionKind::Section,
            Node {
                width: percent(100),
                height: px(24),
                min_height: px(24),
                margin: UiRect::top(px(4)),
                justify_content: JustifyContent::SpaceBetween,
                align_items: AlignItems::Center,
                ..default()
            },
        );
        let collection = commands
            .spawn(Node {
                width: percent(100),
                flex_grow: 1.,
                flex_basis: px(0),
                min_height: px(0),
                column_gap: px(14),
                overflow: Overflow::clip(),
                ..default()
            })
            .id();
        commands.entity(root).add_child(collection);
        let body = commands
            .spawn((
                CatalogBody,
                ScrollPosition(Vec2::new(0., model.scroll)),
                Node {
                    flex_grow: 1.,
                    min_width: px(0),
                    min_height: px(0),
                    flex_direction: FlexDirection::Column,
                    overflow: Overflow::scroll_y(),
                    ..default()
                },
            ))
            .id();
        commands.entity(collection).add_child(body);
        self.collection = RenderRegion::spawn(
            commands,
            body,
            RegionKind::Collection,
            Node {
                width: percent(100),
                flex_shrink: 0.,
                flex_direction: FlexDirection::Column,
                ..default()
            },
        );
        let track = commands
            .spawn((
                Button,
                Hovered::default(),
                Scrollbar,
                RelativeCursorPosition::default(),
                Node {
                    width: px(8),
                    min_width: px(8),
                    height: percent(100),
                    border_radius: BorderRadius::all(px(4)),
                    ..default()
                },
                BackgroundColor(YELLOW.with_alpha(0.22)),
                Pickable::default(),
            ))
            .id();
        commands.entity(collection).add_child(track);
        let (top, height) = model.thumb_geometry(model.viewport_height());
        let thumb = commands
            .spawn((
                ScrollThumb,
                Node {
                    position_type: PositionType::Absolute,
                    top: px(top),
                    height: px(height),
                    width: percent(100),
                    border_radius: BorderRadius::all(px(4)),
                    ..default()
                },
                BackgroundColor(RULE),
                Pickable::IGNORE,
            ))
            .id();
        commands.entity(track).add_child(thumb);
        self.thumb = Some(thumb);
        self.footer = RenderRegion::spawn(
            commands,
            root,
            RegionKind::Footer,
            Node {
                min_height: px(40),
                width: percent(100),
                padding: UiRect::top(px(12)),
                border: UiRect::top(px(1)),
                justify_content: JustifyContent::SpaceBetween,
                align_items: AlignItems::Center,
                column_gap: px(12),
                ..default()
            },
        );
        commands
            .entity(self.footer.root.unwrap())
            .insert(BorderColor::all(RULE));
        self.modal = RenderRegion::spawn(
            commands,
            root,
            RegionKind::Modal,
            Node {
                position_type: PositionType::Absolute,
                left: px(0),
                top: px(0),
                width: percent(100),
                height: percent(100),
                ..default()
            },
        );
        commands
            .entity(self.modal.root.unwrap())
            .insert(Pickable::IGNORE);
    }

    /// Fingerprint only modal structure, finite choices and displayed statuses, excluding secret values.
    fn modal_signature(
        model: &Model,
        runtime: &Runtime,
        network: &crate::network_app::NetworkState,
    ) -> String {
        if let Some(modal) = &model.placement_modal {
            // Native editor identities change only when reset/step replaces a draft field.
            return format!(
                "placement|{:?}|{:?}|{:?}|{:?}|{}|{:?}",
                modal.clip_id,
                modal.title,
                modal
                    .editors
                    .iter()
                    .map(|editor| editor.entity)
                    .collect::<Vec<_>>(),
                modal.error,
                modal.reset && modal.placement().ok() == Some(modal.defaults),
                modal.fit
            );
        }
        let Some(modal) = &model.modal else {
            return "closed".into();
        };
        let fields: Vec<_> = modal.editors.iter().map(|editor| editor.entity).collect();
        let structure = match &modal.draft {
            ModalDraft::Configuration(draft) => format!(
                "{:?}|{:?}",
                draft.provider,
                draft
                    .fields
                    .iter()
                    .enumerate()
                    .filter(|(_, field)| !field.choices.is_empty())
                    .map(|(index, _)| (index, &modal.editors[index].text))
                    .collect::<Vec<_>>()
            ),
            ModalDraft::Sources(sources) => {
                let selection = match &sources.selection {
                    SourceSelection::Local => "local".into(),
                    SourceSelection::Remote(id) => format!("remote:{id}"),
                    SourceSelection::ChooseType => "choose".into(),
                    SourceSelection::NewRemote => "new".into(),
                };
                format!(
                    "{selection}|{}|{:?}|{}|{}|{}|{}|{:?}",
                    sources.details as u8,
                    sources.pending.as_ref().map(|pending| &pending.name),
                    network.local_enabled,
                    network.hosting,
                    network.listen_address,
                    network.listener_hint(),
                    network
                        .controllers
                        .iter()
                        .map(|source| (
                            &source.id,
                            &source.name,
                            source.enabled,
                            source.pairing_secret.is_some(),
                            source.invitation_url.is_some(),
                            &source.invitation_status
                        ))
                        .collect::<Vec<_>>()
                )
            }
        };
        format!(
            "{structure}|{fields:?}|{:?}|{:?}|{:?}|{}|{}",
            modal.error,
            runtime.destination,
            runtime.mode,
            network.admin_status,
            network.admin_secret_available
        )
    }
}

/// Refresh independent UI regions while preserving the live hierarchy around input and scrolling.
#[expect(
    clippy::too_many_arguments,
    reason = "The view combines read-only application state with bounded region caches"
)]
pub(super) fn render(
    mut commands: Commands,
    mut model: ResMut<Model>,
    runtime: Res<Runtime>,
    camera: Res<PanelCamera>,
    mut icons: ResMut<Icons>,
    mut images: ResMut<Assets<Image>>,
    network: Res<crate::network_app::NetworkState>,
    fonts: Res<Fonts>,
    primary: Single<&Window, With<PrimaryWindow>>,
    inputs: Query<Entity, With<input::InputField>>,
    children: Query<&Children>,
    mut focus: ResMut<InputFocus>,
    mut regions: Local<RenderRegions>,
) {
    model.sync_placements(PlacementScope::current(&runtime, &network));

    model.reconcile_sources(&network);

    let remote = runtime.destination == Destination::Remote;
    let compact = primary.width() < 840.;
    // Three minimum-width selectors need another row below a 534-pixel window.
    let toolbar_height = GridLayout::toolbar_height(primary.width());
    let provider_width = ((primary.width() - 84.) / 3.).max(150.);
    let preferences = crate::preferences::Preferences {
        pack: crate::preferences::Preferences::stored_pack(
            &runtime.pack_input,
            &crate::app::Config::bundled_pack(),
        ),
        query: model.query.text.clone(),
        display: runtime.display,
    };
    if model.saved_preferences.as_ref() != Some(&preferences) {
        if !cfg!(test) {
            let _ = preferences.save();
        }
        model.saved_preferences = Some(preferences);
    }
    let snapshot = if remote {
        network.snapshot.as_ref().unwrap_or(&runtime.snapshot)
    } else {
        &runtime.snapshot
    };
    let remote_clips: Vec<Clip> = if remote {
        network
            .remote_catalog
            .as_ref()
            .map(|catalog| {
                catalog
                    .clips
                    .iter()
                    .map(|clip| Clip {
                        identity: clip.clip_id.clone(),
                        category: clip.category.clone().into(),
                        metadata: crate::catalog::ClipMetadata {
                            id: clip.clip_id.clone(),
                            title: clip.title.clone(),
                            asset: format!("clip.{}", clip.media_type),
                            tags: clip.tags.clone(),
                            playback: crate::catalog::Playback::default(),
                            visual: None,
                        },
                        metadata_path: default(),
                        icon_path: default(),
                        asset_path: default(),
                        decoder_readiness: crate::catalog::DecoderReadiness::Unchecked,
                    })
                    .collect()
            })
            .unwrap_or_default()
    } else {
        Vec::new()
    };
    let clips = if remote {
        remote_clips.as_slice()
    } else {
        runtime.catalog.clips.as_slice()
    };
    model.layout = GridLayout::for_window(primary.width(), primary.height());
    model.matches = clips
        .iter()
        .filter(|clip| Model::matching(clip, &model.query.text))
        .count();
    model.layout.rows = (model.viewport_height() / model.row_stride())
        .ceil()
        .max(1.) as usize;
    model.scroll = model.scroll.clamp(0., model.max_scroll());

    let brand_signature = format!("{}", primary.width() >= 620.);
    let header_signature = |model: &Model| {
        format!(
            "{:?}|{}|{}|{}|{}|{}|{:?}",
            model.dropdown,
            primary.width(),
            remote,
            runtime.pack_input,
            Model::source_summary(
                network.local_enabled,
                network
                    .controllers
                    .iter()
                    .filter(|source| source.enabled)
                    .count()
            ),
            model.focus == Some(Field::Search),
            model.query.entity
        )
    };
    let section_signature = format!("{remote}|{}", model.matches);
    let collection_signature = format!(
        "{:?}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}|{:?}",
        model.query.text,
        model.first_row(),
        model.layout.columns,
        model.layout.rows,
        model.layout.card_width,
        model.layout.card_height,
        model.viewport_height(),
        snapshot.sequence,
        snapshot.catalog_revision,
        runtime.catalog.revision,
        network.remote_icons.len(),
        runtime.loading,
        runtime.errors.keys().collect::<Vec<_>>()
    );
    let collection_signature = format!("{collection_signature}|{remote}|{}", network.connected);
    let footer_signature = format!(
        "{remote}|{}|{}|{}",
        runtime.status, network.status, network.connected
    );
    let modal_signature = RenderRegions::modal_signature(&model, &runtime, &network);
    let signature = format!(
        "{:?}",
        (
            &brand_signature,
            header_signature(&model),
            &section_signature,
            &collection_signature,
            &footer_signature,
            &modal_signature
        )
    );
    if signature == model.signature {
        return;
    }
    // The root is never replaced for text edits, scroll travel or asynchronous status changes.
    if model.root.is_none() {
        let root = commands
            .spawn((
                Node {
                    width: percent(100),
                    height: percent(100),
                    min_height: px(0),
                    max_height: percent(100),
                    padding: UiRect::all(px(28)),
                    flex_direction: FlexDirection::Column,
                    row_gap: px(18),
                    overflow: Overflow::clip(),
                    ..default()
                },
                BackgroundColor(BACKGROUND),
                UiTargetCamera(camera.0),
                TabGroup::default(),
            ))
            .id();
        model.root = Some(root);
        regions.initialize(&mut commands, root, &model, toolbar_height);
    }
    input::prepare(&mut commands, &mut model, &inputs, &fonts, &mut focus);
    let header_signature = header_signature(&model);
    let modal_signature = RenderRegions::modal_signature(&model, &runtime, &network);
    if regions.header.changed(&header_signature) {
        commands
            .entity(regions.header.root.unwrap())
            .insert(RenderRegions::header_node(toolbar_height));
    }
    let extent = model.content_height().max(model.viewport_height());
    if regions.collection.changed(&collection_signature) {
        commands
            .entity(regions.collection.root.unwrap())
            .insert(Node {
                width: percent(100),
                height: px(extent),
                min_height: px(extent),
                flex_shrink: 0.,
                flex_direction: FlexDirection::Column,
                ..default()
            });

        commands
            .entity(regions.thumb.unwrap())
            .insert(BackgroundColor(if model.max_scroll() == 0. {
                RULE
            } else {
                PURPLE
            }));
    }
    regions.brand.refresh(
        &mut commands,
        &children,
        &fonts,
        &mut focus,
        brand_signature.clone(),
        |view, brand| {
            view.title(brand, "durop", 29., INK);
            if primary.width() >= 620. {
                view.label(brand, "A LITTLE EXPRESSION, ON CUE.", 10., MUTED);
            }
        },
    );
    regions.header.refresh(
        &mut commands,
        &children,
        &fonts,
        &mut focus,
        header_signature.clone(),
        |view, header| {
            view.selector(
                header,
                "FROM",
                &Model::source_summary(
                    network.local_enabled,
                    network
                        .controllers
                        .iter()
                        .filter(|source| source.enabled)
                        .count(),
                ),
                Dropdown::From,
                false,
                if compact { provider_width } else { 230. },
            );
            view.selector(
                header,
                "TO",
                if remote { "Remote host" } else { "This Mac" },
                Dropdown::To,
                model.dropdown == Some(Dropdown::To),
                if compact { provider_width } else { 170. },
            );
            let catalog_label = if remote {
                "Remote catalog"
            } else if runtime.pack_input.starts_with("https://") {
                "Git repository"
            } else if runtime.pack_input == crate::app::Config::bundled_pack().to_string_lossy() {
                "Durop reactions"
            } else {
                "Local directory"
            };
            view.selector(
                header,
                "CATALOG",
                catalog_label,
                Dropdown::Catalog,
                model.dropdown == Some(Dropdown::Catalog),
                if compact { provider_width } else { 210. },
            );
            header
                .spawn((
                    Node {
                        flex_grow: 1.,
                        width: if compact { percent(100) } else { auto() },
                        height: px(68),
                        min_height: px(68),
                        max_height: px(68),
                        min_width: px(130),
                        padding: UiRect::axes(px(16), px(11)),
                        border: UiRect::all(px(1)),
                        border_radius: BorderRadius::all(px(6)),
                        flex_direction: FlexDirection::Column,
                        row_gap: px(6),
                        justify_content: JustifyContent::Center,
                        overflow: Overflow::clip(),
                        ..default()
                    },
                    BackgroundColor(PAPER),
                    BorderColor::all(if model.focus == Some(Field::Search) {
                        PURPLE
                    } else {
                        RULE
                    }),
                    HoverColors {
                        resting: PAPER,
                        hovered: BACKGROUND,
                    },
                ))
                .with_children(|search| {
                    view.label(search, "SEARCH", 10., MUTED);
                    view.input(search, &model.query);
                });
        },
    );
    regions.section.refresh(
        &mut commands,
        &children,
        &fonts,
        &mut focus,
        section_signature.clone(),
        |view, section| {
            view.label(
                section,
                if remote {
                    "REMOTE COLLECTION"
                } else {
                    "YOUR COLLECTION"
                },
                10.,
                PURPLE,
            );
            view.label(section, format!("{} REACTIONS", model.matches), 10., MUTED);
        },
    );
    regions.collection.refresh(
        &mut commands,
        &children,
        &fonts,
        &mut focus,
        collection_signature.clone(),
        |view, grid| {
            let visible = model.visible(clips);
            if visible.is_empty() {
                grid.spawn(Node {
                    width: percent(100),
                    flex_grow: 1.,
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    row_gap: px(14),
                    ..default()
                })
                .with_children(|empty| {
                    view.title(
                        empty,
                        if runtime.loading {
                            "Gathering your collection…"
                        } else if remote && !network.connected {
                            "Connect your host"
                        } else {
                            "Nothing here, yet."
                        },
                        28.,
                        INK,
                    );
                    view.label(
                        empty,
                        if remote && !network.connected {
                            "Choose To → Remote host to enter its IP and Durop pairing token."
                        } else {
                            "Try another search or choose a catalog above."
                        },
                        12.,
                        MUTED,
                    );
                });
            }
            for (row_index, indices) in visible.chunks(model.layout.columns).enumerate() {
                grid.spawn(Node {
                    position_type: PositionType::Absolute,
                    top: px((model.first_row() + row_index) as f32 * model.row_stride()),
                    width: percent(100),
                    height: px(model.layout.card_height),
                    column_gap: px(20),
                    min_height: px(model.layout.card_height),
                    ..default()
                })
                .with_children(|row| {
                    for index in indices {
                        let clip = &clips[*index];
                        let key = format!(
                            "{}:{}:{}:{}",
                            remote,
                            snapshot.catalog_revision,
                            clip.identity,
                            remote && network.remote_icons.contains_key(&clip.identity)
                        );
                        if !icons.0.contains_key(&key) {
                            let bytes = if remote {
                                network.remote_icons.get(&clip.identity).cloned()
                            } else {
                                Icons::read_icon(&clip.icon_path).ok()
                            };
                            let handle = bytes
                                .and_then(|bytes| Icons::icon_image(&bytes).ok())
                                .map(|image| images.add(image));
                            if icons.0.len() >= 128 {
                                let old = icons.0.keys().next().cloned().unwrap();
                                if let Some(Some(handle)) = icons.0.remove(&old) {
                                    images.remove(handle.id());
                                }
                            }
                            icons.0.insert(key.clone(), handle);
                        }
                        let image = icons.0.get(&key).cloned().flatten();
                        let dimensions = image
                            .as_ref()
                            .and_then(|handle| images.get(handle))
                            .map(|image| Vec2::new(image.width() as f32, image.height() as f32));
                        let active = snapshot.instances.iter().rev().find(|instance| {
                            instance.clip_id == clip.identity && !instance.state.is_terminal()
                        });
                        let status = if runtime.errors.contains_key(&clip.identity) {
                            "ERROR"
                        } else {
                            "PLAY →"
                        };
                        view.card(
                            row,
                            clip,
                            image,
                            dimensions,
                            model.layout,
                            status,
                            active.map(|instance| instance.instance_id.as_str()),
                            (!remote || network.connected)
                                && snapshot.instances.iter().any(|instance| {
                                    instance.clip_id == clip.identity
                                        && instance.state == crate::commands::InstanceState::Playing
                                }),
                            if remote {
                                network
                                    .remote_catalog
                                    .as_ref()
                                    .and_then(|catalog| {
                                        catalog.clips.iter().find(|remote_clip| {
                                            remote_clip.clip_id == clip.identity
                                        })
                                    })
                                    .is_some_and(|remote_clip| {
                                        remote_clip.default_placement.is_some()
                                    })
                            } else {
                                crate::host::Host::default_placement(clip).is_some()
                            },
                        );
                    }
                });
            }
        },
    );
    regions.footer.refresh(
        &mut commands,
        &children,
        &fonts,
        &mut focus,
        footer_signature.clone(),
        |view, footer| {
            footer
                .spawn(Node {
                    flex_direction: FlexDirection::Column,
                    row_gap: px(3),
                    flex_grow: 1.,
                    overflow: Overflow::clip(),
                    ..default()
                })
                .with_children(|status| {
                    view.label(
                        status,
                        if remote {
                            &network.status
                        } else {
                            &runtime.status
                        },
                        10.,
                        if remote && !network.connected {
                            MUTED
                        } else {
                            GREEN
                        },
                    );
                    view.label(
                        status,
                        if remote {
                            "ESC TO STOP · CLICK A CARD TO PLAY".to_owned()
                        } else {
                            format!("{} · ESC TO STOP", network.status)
                        },
                        9.,
                        MUTED,
                    );
                });
            if remote && network.connected {
                view.button(footer, "Disconnect", UiAction::Disconnect, false);
            }
            view.button(footer, "STOP ALL", UiAction::Runtime(Action::StopAll), true);
        },
    );
    regions.modal.refresh(
        &mut commands,
        &children,
        &fonts,
        &mut focus,
        modal_signature.clone(),
        |view, parent| {
            if let Some(modal) = &model.modal {
                view.modal(parent, modal, model.focus, &network, &runtime);
            }
            if let Some(modal) = &model.placement_modal {
                view.placement_modal(parent, modal);
            }
        },
    );
    model.signature = format!(
        "{:?}",
        (
            &brand_signature,
            &header_signature,
            &section_signature,
            &collection_signature,
            &footer_signature,
            &modal_signature
        )
    );
}
