//! Presentation of the controller collection, provider menus and isolated configuration form.

use super::theme::*;
use super::*;
use bevy::ui::{FocusPolicy, RelativeCursorPosition};

/// Construct small reusable paper controls without coupling styling to provider policy.
struct View<'a> {
    /// Shared bundled fonts; widgets only clone lightweight asset handles.
    fonts: &'a Fonts,
}

impl View<'_> {
    /// Add a monospaced label with explicit contrast against paper surfaces.
    fn label(
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
            .with_children(|button| self.label(button, label, 12., foreground));
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
                        UiAction::Dropdown(menu),
                        Node {
                            width: percent(100),
                            min_height: px(68),
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
                        self.label(button, label, 10., MUTED);
                        button
                            .spawn(Node {
                                width: percent(100),
                                justify_content: JustifyContent::SpaceBetween,
                                align_items: AlignItems::Center,
                                column_gap: px(8),
                                ..default()
                            })
                            .with_children(|row| {
                                self.label(row, value, 13., INK);
                                self.label(row, if open { "−" } else { "+" }, 17., PURPLE);
                            });
                    });
                if open {
                    container
                        .spawn((
                            Node {
                                position_type: PositionType::Absolute,
                                top: px(76),
                                left: if menu == Dropdown::Host {
                                    px(0)
                                } else {
                                    auto()
                                },
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
                            FocusPolicy::Block,
                        ))
                        .with_children(|popup| {
                            let entries = match menu {
                                Dropdown::Host => vec![
                                    ("Native · this Mac", Provider::Host(HostProvider::Native)),
                                    (
                                        "Tailscale · remote host",
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
    ) {
        parent
            .spawn((
                Button,
                UiAction::Runtime(Action::Play(clip.identity.clone())),
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
                BackgroundColor(PAPER),
                BorderColor::all(RULE),
                HoverColors {
                    resting: PAPER,
                    hovered: Color::srgb(0.969, 0.941, 0.871),
                },
            ))
            .with_children(|card| {
                card.spawn((
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
                                ..default()
                            },
                        ));
                    } else {
                        self.label(preview, "PREVIEW UNAVAILABLE", 10., MUTED);
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
                    title.spawn((
                        Text::new(&clip.metadata.title),
                        self.fonts.display(17.),
                        TextColor(INK),
                    ));
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
                    self.label(row, kind, 9., MUTED);
                    if let Some(instance) = stop {
                        row.spawn((
                            Button,
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
                        .with_children(|stop| self.label(stop, "STOP", 9., PAPER));
                    } else {
                        self.label(row, status, 9., PURPLE);
                    }
                });
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
        for (index, field) in modal.draft.fields.iter().enumerate() {
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
                    let value = if field.secret {
                        if editor.text.is_empty() {
                            "Enter pairing secret".into()
                        } else {
                            "•".repeat(editor.text.chars().count())
                        }
                    } else if !field.choices.is_empty() {
                        field
                            .choices
                            .iter()
                            .find(|choice| choice.value == editor.text)
                            .map(|choice| format!("{}  +", choice.label))
                            .unwrap_or_else(|| editor.text.clone())
                    } else if selected {
                        editor.label()
                    } else if editor.text.is_empty() {
                        "Enter a value…".into()
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
                    } else {
                        let action = if field.choices.is_empty() {
                            UiAction::Focus(Field::Configuration(index))
                        } else {
                            UiAction::Cycle(index)
                        };
                        row.spawn((
                            Button,
                            action,
                            node,
                            BackgroundColor(BACKGROUND),
                            BorderColor::all(if selected { PURPLE } else { RULE }),
                            HoverColors {
                                resting: BACKGROUND,
                                hovered: PAPER,
                            },
                        ))
                        .with_children(|input| self.label(input, value, 12., INK));
                    }
                    self.label(row, field.hint, 10., MUTED);
                });
        }
        if matches!(modal.draft.provider, Provider::Catalog(_)) && runtime.mode == Mode::Controller
        {
            self.label(dialog, "Catalog is supplied by the remote host. Restart in local mode to use local catalog sources.", 12., PURPLE);
        }
        if modal.draft.provider == Provider::Host(HostProvider::Native)
            && runtime.mode == Mode::Controller
        {
            self.label(
                dialog,
                "This launch is controller-only. Restart in local mode to play or share this Mac.",
                12.,
                PURPLE,
            );
        }
        if modal.draft.provider == Provider::Host(HostProvider::Native)
            && runtime.mode != Mode::Controller
        {
            dialog
                .spawn(Node {
                    flex_direction: FlexDirection::Column,
                    row_gap: px(10),
                    ..default()
                })
                .with_children(|sharing| {
                    self.label(sharing, "REMOTE ACCESS", 10., MUTED);
                    self.label(sharing, &network.status, 10., MUTED);
                    if network.hosting {
                        self.label(
                            sharing,
                            "Sharing is enabled. Give this secret only to your controller.",
                            10.,
                            GREEN,
                        );
                        if let Some(secret) = &network.pairing_secret {
                            self.label(sharing, secret, 11., INK);
                        }
                        self.button(sharing, "Revoke remote access", UiAction::RevokeHost, false);
                    } else {
                        self.button(
                            sharing,
                            "Share this Mac…",
                            UiAction::Configure(Provider::Host(HostProvider::TailscaleListener)),
                            false,
                        );
                    }
                });
        }
        if let Some(error) = &modal.error {
            self.label(dialog, error, 12., PURPLE);
        }
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
        let (title, subtitle) = match modal.draft.provider {
            Provider::Host(HostProvider::Native) => (
                "Native host",
                "Play on this Mac. Choose where your reactions appear.",
            ),
            Provider::Host(HostProvider::Tailscale) => (
                modal.draft.provider.label(),
                "Connect to an authenticated host on your private network.",
            ),
            Provider::Host(HostProvider::TailscaleListener) => (
                "Share this Mac",
                "Allow another Durop controller to pair with this host over Tailscale.",
            ),
            Provider::Catalog(CatalogProvider::Bundled) => (
                "Durop reactions",
                "A collection that travels with your app. Ready to play offline.",
            ),
            Provider::Catalog(CatalogProvider::LocalDirectory) => {
                ("Local catalog", "Load a pack from a directory on this Mac.")
            }
            Provider::Catalog(CatalogProvider::GitHttps) => (
                "Git repository",
                "Pin a catalog from a repository, branch or commit.",
            ),
        };
        root.spawn((
            Button,
            UiAction::Cancel,
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
            FocusPolicy::Block,
        ))
        .with_children(|backdrop| {
            backdrop
                .spawn((
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
                    FocusPolicy::Block,
                ))
                .with_children(|dialog| {
                    dialog
                        .spawn(Node {
                            justify_content: JustifyContent::SpaceBetween,
                            align_items: AlignItems::Center,
                            ..default()
                        })
                        .with_children(|heading| {
                            heading.spawn((
                                Text::new(title),
                                self.fonts.display(31.),
                                TextColor(INK),
                            ));
                            self.button(heading, "Close ×", UiAction::Cancel, false);
                        });
                    self.label(dialog, subtitle, 12., MUTED);
                    dialog
                        .spawn((
                            ModalBody,
                            ScrollPosition(Vec2::new(0., modal.scroll)),
                            Node {
                                width: percent(100),
                                min_height: px(0),
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
                            justify_content: JustifyContent::SpaceBetween,
                            align_items: AlignItems::Center,
                            margin: UiRect::top(px(6)),
                            ..default()
                        })
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
                                    self.button(actions, "Cancel", UiAction::Cancel, false);
                                    self.button(actions, "Apply →", UiAction::Apply, true);
                                });
                        });
                });
        });
    }
}

/// Rebuild only when visible state changes, keeping secret values out of invalidation fingerprints.
#[expect(
    clippy::too_many_arguments,
    reason = "The view combines read-only application state with its bounded caches"
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
) {
    let remote = runtime.destination == Destination::Remote;
    let compact = primary.width() < 760.;
    let provider_width = ((primary.width() - 70.) * 0.5).max(150.);
    let preferences = crate::preferences::Preferences {
        pack: crate::preferences::Preferences::stored_pack(
            &runtime.pack_input,
            &crate::app::Config::bundled_pack(),
        ),
        query: model.query.text.clone(),
        display: runtime.display,
    };
    if model.saved_preferences.as_ref() != Some(&preferences) {
        let _ = preferences.save();
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
    model.first_row = model.first_row.min(model.max_row());
    let signature = format!(
        "{}|{:?}|{:?}|{}|{}|{}|{}|{}|{:?}|{}|{}|{}|{}|{}|{}|{}|{}",
        model.revision,
        model.focus,
        model.dropdown,
        model.first_row,
        snapshot.sequence,
        runtime.status,
        runtime.catalog.revision,
        model.matches,
        runtime.destination,
        runtime.enabled,
        network.status,
        network.connected,
        network.remote_icons.len(),
        primary.width(),
        primary.height(),
        model.query.label(),
        network.hosting
    );
    if signature == model.signature {
        return;
    }
    model.signature = signature;
    if let Some(root) = model.root.take() {
        commands.entity(root).despawn();
    }
    let root = commands
        .spawn((
            Node {
                width: percent(100),
                height: percent(100),
                padding: UiRect::all(px(28)),
                flex_direction: FlexDirection::Column,
                row_gap: px(18),
                overflow: Overflow::clip(),
                ..default()
            },
            BackgroundColor(BACKGROUND),
            UiTargetCamera(camera.0),
        ))
        .id();
    model.root = Some(root);
    let view = View { fonts: &fonts };
    commands.entity(root).with_children(|parent| {
        parent
            .spawn(Node {
                height: px(32),
                min_height: px(32),
                justify_content: JustifyContent::SpaceBetween,
                align_items: AlignItems::Center,
                ..default()
            })
            .with_children(|brand| {
                brand.spawn((Text::new("durop"), fonts.display(29.), TextColor(INK)));
                if primary.width() >= 620. {
                    view.label(brand, "A LITTLE EXPRESSION, ON CUE.", 10., MUTED);
                }
            });
        parent
            .spawn(Node {
                width: percent(100),
                min_height: px(if compact { 148. } else { 68. }),
                column_gap: px(14),
                row_gap: px(12),
                flex_wrap: FlexWrap::Wrap,
                align_items: AlignItems::Stretch,
                ..default()
            })
            .with_children(|header| {
                view.selector(
                    header,
                    "HOST",
                    if remote { "Tailscale" } else { "Native" },
                    Dropdown::Host,
                    model.dropdown == Some(Dropdown::Host),
                    if compact { provider_width } else { 220. },
                );
                let catalog_label = if remote {
                    "Remote catalog"
                } else if runtime.pack_input.starts_with("https://") {
                    "Git repository"
                } else if runtime.pack_input == crate::app::Config::bundled_pack().to_string_lossy()
                {
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
                    if compact { provider_width } else { 300. },
                );
                header
                    .spawn((
                        Button,
                        UiAction::Focus(Field::Search),
                        Node {
                            flex_grow: 1.,
                            width: if compact { percent(100) } else { auto() },
                            min_height: px(68),
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
                        view.label(
                            search,
                            if model.focus == Some(Field::Search) {
                                model.query.label()
                            } else if model.query.text.is_empty() {
                                "Find a reaction…".into()
                            } else {
                                model.query.text.clone()
                            },
                            13.,
                            if model.query.text.is_empty() {
                                MUTED
                            } else {
                                INK
                            },
                        );
                    });
            });
        parent
            .spawn(Node {
                width: percent(100),
                height: px(24),
                min_height: px(24),
                margin: UiRect::top(px(4)),
                justify_content: JustifyContent::SpaceBetween,
                align_items: AlignItems::Center,
                ..default()
            })
            .with_children(|section| {
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
            });
        parent
            .spawn(Node {
                width: percent(100),
                flex_grow: 1.,
                min_height: px(0),
                column_gap: px(14),
                overflow: Overflow::clip(),
                ..default()
            })
            .with_children(|collection| {
                collection
                    .spawn(Node {
                        flex_grow: 1.,
                        flex_direction: FlexDirection::Column,
                        row_gap: px(20),
                        overflow: Overflow::clip(),
                        ..default()
                    })
                    .with_children(|grid| {
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
                                empty.spawn((
                                    Text::new(if runtime.loading {
                                        "Gathering your collection…"
                                    } else if remote && !network.connected {
                                        "Connect your host"
                                    } else {
                                        "Nothing here, yet."
                                    }),
                                    fonts.display(28.),
                                    TextColor(INK),
                                ));
                                view.label(
                                    empty,
                                    if remote && !network.connected {
                                        "Choose Tailscale above to pair with a remote host."
                                    } else {
                                        "Try another search or choose a catalog above."
                                    },
                                    12.,
                                    MUTED,
                                );
                            });
                        }
                        for indices in visible.chunks(model.layout.columns) {
                            grid.spawn(Node {
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
                                        .map(|image| {
                                            Vec2::new(image.width() as f32, image.height() as f32)
                                        });
                                    let active = snapshot.instances.iter().rev().find(|instance| {
                                        instance.clip_id == clip.identity
                                            && !instance.state.is_terminal()
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
                                    );
                                }
                            });
                        }
                    });
                collection
                    .spawn((
                        Button,
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
                        FocusPolicy::Block,
                    ))
                    .with_children(|track| {
                        let total_rows = model.matches.div_ceil(model.layout.columns).max(1);
                        let fraction = (model.layout.rows as f32 / total_rows as f32).min(1.);
                        track.spawn((
                            Node {
                                position_type: PositionType::Absolute,
                                top: percent(100. * model.first_row as f32 / total_rows as f32),
                                width: percent(100),
                                height: percent(100. * fraction),
                                min_height: px(24),
                                border_radius: BorderRadius::all(px(4)),
                                ..default()
                            },
                            BackgroundColor(if model.max_row() == 0 { RULE } else { PURPLE }),
                            FocusPolicy::Pass,
                        ));
                    });
            });
        parent
            .spawn(Node {
                min_height: px(40),
                width: percent(100),
                padding: UiRect::top(px(12)),
                border: UiRect::top(px(1)),
                justify_content: JustifyContent::SpaceBetween,
                align_items: AlignItems::Center,
                column_gap: px(12),
                ..default()
            })
            .insert(BorderColor::all(RULE))
            .with_children(|footer| {
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
            });
        if let Some(modal) = &model.modal {
            view.modal(parent, modal, model.focus, &network, &runtime);
        }
    });
}
