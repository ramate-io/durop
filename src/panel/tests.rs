//! Regression coverage for input routing, modal isolation and bounded collection geometry.

use super::*;
use bevy::input::ButtonState;
use bevy::window::Ime;

/// Feed legacy routing fixtures through a one-shot intent queue; engine activation has separate integration coverage.
fn queue_test_activations(
    widgets: Query<&UiAction, Added<Pressed>>,
    mut pending: ResMut<input::PendingActions>,
) {
    pending.0.extend(widgets.iter().cloned());
}

/// Start a controller-only runtime without loading a pack from user preferences.
fn config() -> crate::app::Config {
    crate::app::Config::parse(["--mode".into(), "controller".into()]).unwrap()
}

/// Reject hostile image dimensions before allocating decoded thumbnail pixels.
#[test]
fn thumbnails_reject_large_payloads_and_oversized_ihdr_before_decode() {
    let mut bytes = vec![0; 24];
    bytes[..8].copy_from_slice(b"\x89PNG\r\n\x1a\n");
    bytes[12..16].copy_from_slice(b"IHDR");
    bytes[16..20].copy_from_slice(&100_000u32.to_be_bytes());
    bytes[20..24].copy_from_slice(&100_000u32.to_be_bytes());
    assert_eq!(
        Icons::icon_image(&bytes).unwrap_err(),
        "thumbnail dimensions exceed 512x512"
    );
    assert!(Icons::icon_image(&vec![0; MAX_ICON_BYTES + 1]).is_err());
    let icon = std::fs::read(crate::app::resource_root().join("sample-packs/podcast-pack/gif.png"))
        .unwrap();
    assert!(Icons::icon_image(&icon).is_ok());
}

/// The headless edit harness has no renderer visibility propagation; expose its UI nodes for tab traversal.
fn mark_test_nodes_visible(mut nodes: Query<&mut InheritedVisibility>) {
    for mut visible in &mut nodes {
        *visible = InheritedVisibility::VISIBLE;
    }
}

/// Construct the production text widgets and engine dispatch/edit lifecycle without a native window.
fn input_app() -> (App, Entity) {
    let mut app = App::new();
    let runtime = Runtime::new(&config()).unwrap();
    let network = crate::network_app::NetworkState::for_ui_test(runtime.snapshot.clone());
    let preferences = crate::preferences::Preferences {
        pack: crate::preferences::Preferences::stored_pack(
            &runtime.pack_input,
            &crate::app::Config::bundled_pack(),
        ),
        query: String::new(),
        display: runtime.display,
    };
    app.add_plugins((
        bevy::input::InputPlugin,
        bevy::input_focus::InputFocusPlugin,
        bevy::input_focus::InputDispatchPlugin,
        bevy::ui_widgets::TextInputPlugin,
        TabNavigationPlugin,
    ))
    .insert_resource(runtime)
    .insert_resource(network)
    .insert_resource(Model {
        saved_preferences: Some(preferences),
        ..default()
    })
    .init_resource::<Actions>()
    .init_resource::<input::PendingActions>()
    .init_resource::<input::OrderedKeyboard>()
    .init_resource::<bevy::clipboard::Clipboard>()
    .init_resource::<bevy::text::FontCx>()
    .init_resource::<bevy::text::LayoutCx>()
    .init_resource::<Time<bevy::time::Real>>()
    .init_resource::<bevy::picking::events::PointerState>()
    .init_resource::<UiScale>()
    .init_resource::<Icons>()
    .init_resource::<Fonts>()
    .init_resource::<Assets<Image>>()
    .init_resource::<Assets<Font>>()
    .register_required_components::<EditableText, bevy::ui::widget::EditableTextContentSizeState>()
    .add_message::<Ime>()
    .add_message::<bevy::window::RequestRedraw>()
    .add_message::<bevy::picking::events::PointerRelease>()
    .add_message::<bevy::input::gamepad::GamepadButtonChangedEvent>()
    .add_observer(input::activate)
    .add_observer(input::keyboard)
    .add_systems(
        PreUpdate,
        input::dispatch_keyboard
            .after(bevy::input::InputSystems)
            .before(bevy::input_focus::InputFocusSystems::Dispatch),
    )
    .add_systems(Update, (view::render, mark_test_nodes_visible).chain())
    .add_systems(
        PostUpdate,
        bevy::text::apply_text_edits.in_set(EditableTextSystems),
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
    );
    let mono = Font::from_bytes(
        std::fs::read(crate::app::resource_root().join("assets/fonts/IBMPlexMono-Regular.ttf"))
            .unwrap(),
    );
    let display = Font::from_bytes(
        std::fs::read(crate::app::resource_root().join("assets/fonts/ExpletusSans-Regular.ttf"))
            .unwrap(),
    );
    {
        let mut context = app.world_mut().resource_mut::<bevy::text::FontCx>();
        context.collection.register_fonts(mono.data.clone(), None);
        context.set_sans_serif_family("IBM Plex Mono").unwrap();
        context.set_monospace_family("IBM Plex Mono").unwrap();
    }
    let mono = app.world_mut().resource_mut::<Assets<Font>>().add(mono);
    let display = app.world_mut().resource_mut::<Assets<Font>>().add(display);
    app.insert_resource(Fonts { mono, display });
    let window = app
        .world_mut()
        .spawn((
            Window {
                focused: true,
                ..default()
            },
            PrimaryWindow,
        ))
        .id();
    let camera = app.world_mut().spawn(Camera2d).id();
    app.insert_resource(PanelCamera(camera));
    app.update();
    let search = app.world().resource::<Model>().query.entity.unwrap();
    app.world_mut()
        .resource_mut::<InputFocus>()
        .set(search, FocusCause::Pressed);
    (app, window)
}

/// Submit one pressed keyboard message through the same path as native input.
fn key(app: &mut App, window: Entity, logical_key: Key, key_code: KeyCode, text: Option<&str>) {
    app.world_mut().write_message(KeyboardInput {
        key_code,
        logical_key,
        state: ButtonState::Pressed,
        text: text.map(Into::into),
        repeat: false,
        window,
    });
}

/// Editing must consume shortcut-looking keys until focus is explicitly released.
#[test]
fn actual_keyboard_messages_edit_search_and_suppress_drop_shortcuts() {
    let (mut app, window) = input_app();
    key(
        &mut app,
        window,
        Key::Character("q".into()),
        KeyCode::KeyQ,
        Some("q"),
    );
    key(&mut app, window, Key::Space, KeyCode::Space, Some(" "));
    app.update();
    assert_eq!(app.world().resource::<Model>().query.text, "q ");
    assert!(app.world().resource::<Actions>().0.is_empty());
    key(&mut app, window, Key::Escape, KeyCode::Escape, None);
    app.update();
    assert!(app.world().resource::<Actions>().0.is_empty()); // first Escape ends search
    key(&mut app, window, Key::Escape, KeyCode::Escape, None);
    app.update();
    assert!(matches!(
        app.world().resource::<Actions>().0.front(),
        Some(Action::StopAll)
    ));
}

/// Engine character selection and replacement preserve Unicode text rather than inserting command letters.
#[test]
fn engine_selection_replaces_unicode_search_and_suppresses_shortcuts() {
    use bevy::text::TextEdit;
    let (mut app, window) = input_app();
    let input = app.world().resource::<Model>().query.entity.unwrap();
    app.world_mut()
        .get_mut::<EditableText>(input)
        .unwrap()
        .queue_edit(TextEdit::Insert("Aé🌈".into()));
    app.update();
    key(&mut app, window, Key::Shift, KeyCode::ShiftLeft, None);
    key(&mut app, window, Key::ArrowLeft, KeyCode::ArrowLeft, None);
    app.update();
    app.world_mut().write_message(KeyboardInput {
        key_code: KeyCode::ShiftLeft,
        logical_key: Key::Shift,
        state: ButtonState::Released,
        text: None,
        repeat: false,
        window,
    });
    key(
        &mut app,
        window,
        Key::Character("日本".into()),
        KeyCode::KeyN,
        Some("日本"),
    );
    app.update();
    assert_eq!(app.world().resource::<Model>().query.text, "Aé日本");
    assert!(app.world().resource::<Actions>().0.is_empty());
}

/// Button admission requests cancellation without claiming native teardown finished.
#[test]
fn pressed_stop_all_button_routes_to_shared_host_and_truthful_request_status() {
    // Exercise UI routing and host policy; native teardown is separately acknowledged.
    let mut runtime = Runtime::new(&config()).unwrap();
    let catalog = std::sync::Arc::new(
        crate::catalog::load_local(&crate::app::resource_root().join("sample-packs/podcast-pack"))
            .unwrap(),
    );
    runtime.catalog = catalog.clone();
    runtime.host = crate::host::Host::new("ui-test".into(), catalog.clone());
    runtime.destination = Destination::Local;
    let envelope = crate::commands::Envelope {
        protocol_version: crate::commands::PROTOCOL_VERSION,
        session_id: "ui-test".into(),
        command_id: "play".into(),
        catalog_revision: catalog.revision.clone(),
        command: crate::commands::PlaybackCommand::Play {
            clip_id: catalog.clips[0].identity.clone(),
        },
    };
    runtime.host.accept(envelope);
    let token = runtime
        .host
        .drain_effects()
        .into_iter()
        .find_map(|effect| {
            if let crate::host::BackendEffect::Prepare { token, .. } = effect {
                Some(token)
            } else {
                None
            }
        })
        .unwrap();
    for event in [
        crate::host::BackendEventKind::Prepared,
        crate::host::BackendEventKind::Started,
    ] {
        runtime.host.backend_event(crate::host::BackendEvent {
            token: token.clone(),
            event,
        });
    }
    runtime.snapshot = runtime.host.snapshot();
    let network = crate::network_app::NetworkState::for_ui_test(runtime.snapshot.clone());
    let mut app = App::new();
    app.insert_resource(runtime)
        .insert_resource(network)
        .init_resource::<Model>()
        .init_resource::<Actions>()
        .init_resource::<input::PendingActions>()
        .init_resource::<input::OrderedKeyboard>()
        .init_resource::<InputFocus>()
        .add_systems(Update, (queue_test_activations, buttons).chain());
    app.world_mut()
        .spawn((Pressed, UiAction::Runtime(Action::StopAll)));
    app.update();
    assert!(matches!(
        app.world_mut().resource_mut::<Actions>().0.pop_front(),
        Some(Action::StopAll)
    ));
    app.world_mut()
        .resource_scope(|world, mut runtime: Mut<Runtime>| {
            let mut network = world.resource_mut::<crate::network_app::NetworkState>();
            runtime.request_stop_all(&mut network);
            assert!(runtime.status.starts_with("Stop all requested"));
            network.accept_for_ui_test(&mut runtime.host);
            assert_eq!(
                runtime.host.snapshot().instances[0].state,
                crate::commands::InstanceState::Stopping
            );
            assert!(
                runtime
                    .host
                    .drain_effects()
                    .iter()
                    .any(|effect| matches!(effect, crate::host::BackendEffect::Stop { .. }))
            );
        });
}

/// IME composition is separate from committed text and never duplicates typed input.
#[test]
fn actual_ime_messages_keep_preedit_uncommitted_and_commit_only_once() {
    let (mut app, window) = input_app();
    app.world_mut().write_message(Ime::Preedit {
        window,
        value: "に".into(),
        cursor: Some((3, 3)),
    });
    app.update();
    assert_eq!(app.world().resource::<Model>().query.text, "");
    let input = app.world().resource::<Model>().query.entity.unwrap();
    assert!(
        app.world()
            .get::<EditableText>(input)
            .unwrap()
            .is_composing()
    );
    app.world_mut().write_message(Ime::Commit {
        window,
        value: "日本".into(),
    });
    app.update();
    assert_eq!(app.world().resource::<Model>().query.text, "日本");
    key(&mut app, window, Key::Backspace, KeyCode::Backspace, None);
    app.update();
    assert_eq!(app.world().resource::<Model>().query.text, "日");
    assert!(app.world().resource::<Actions>().0.is_empty());
}

/// Virtualized rendering stays bounded while filtering the complete source catalog.
#[test]
fn ten_thousand_catalog_items_render_only_visible_rows_and_search_all_fields() {
    let catalog =
        crate::catalog::load_local(&crate::app::resource_root().join("sample-packs/podcast-pack"))
            .unwrap();
    let clips: Vec<_> = (0..10_000)
        .map(|i| {
            let mut clip = catalog.clips[0].clone();
            clip.identity = format!("large/{i}");
            clip.metadata.title = format!("Title {i}");
            clip.metadata.tags = vec!["sound".into()];
            clip.category = "Music".into();
            clip
        })
        .collect();
    let mut model = Model {
        matches: clips.len(),
        viewport_height: 450.,
        ..default()
    };
    model.layout.rows = 2;
    model.query.text = "sound music".into();
    model.scroll = 100. * model.row_stride() + 0.5;
    assert_eq!(model.visible(&clips).len(), model.layout.columns * 4);
    assert_eq!(model.visible(&clips)[0], 400);
    model.scroll = model.max_scroll();
    assert_eq!(model.visible(&clips).last(), Some(&9999));
    model.scroll = 0.;
    model.query.text = "Title 9999".into();
    assert_eq!(model.visible(&clips), vec![9999]);
    model.query.text = "absent".into();
    assert!(model.visible(&clips).is_empty());
}

/// Build provider inputs without touching native monitors, credentials or disk state.
fn provider_context() -> ConfigurationContext {
    ConfigurationContext {
        displays: vec![DisplayChoice {
            index: 0,
            label: "Studio display".into(),
        }],
        current_display: None,
        current_endpoint: "100.100.0.10:48732".into(),
        local_pack: "/tmp/catalog".into(),
        bundled_path: crate::app::Config::bundled_pack(),
        git_url: "https://github.com/ramate-io/durop.git".into(),
        git_reference: "main".into(),
        git_directory: "sample-packs/reactions".into(),
    }
}

/// Escape closes an active draft before reaching global playback shortcuts.
#[test]
fn escape_cancels_provider_modal_before_requesting_any_playback_stop() {
    let (mut app, window) = input_app();
    app.world_mut()
        .resource_mut::<Model>()
        .open(Provider::Host(HostProvider::Tailscale), &provider_context());
    key(&mut app, window, Key::Escape, KeyCode::Escape, None);
    app.update();
    assert!(app.world().resource::<Model>().modal.is_none());
    assert!(app.world().resource::<Actions>().0.is_empty());
    key(&mut app, window, Key::Escape, KeyCode::Escape, None);
    app.update();
    assert!(matches!(
        app.world().resource::<Actions>().0.front(),
        Some(Action::StopAll)
    ));
}

/// A modal blocks background play and discards its credential when canceled.
#[test]
fn provider_modal_blocks_background_play_but_preserves_emergency_stop() {
    let runtime = Runtime::new(&config()).unwrap();
    let network = crate::network_app::NetworkState::for_ui_test(runtime.snapshot.clone());
    let mut model = Model::default();
    model.open(Provider::Host(HostProvider::Tailscale), &provider_context());
    model.modal.as_mut().unwrap().editors[1]
        .text
        .push_str(&"a".repeat(64));
    let mut app = App::new();
    app.insert_resource(runtime)
        .insert_resource(network)
        .insert_resource(model)
        .init_resource::<Actions>()
        .init_resource::<input::PendingActions>()
        .init_resource::<InputFocus>()
        .add_systems(Update, (queue_test_activations, buttons).chain());
    app.world_mut().spawn((
        Pressed,
        UiAction::Runtime(Action::Play("background/clip".into())),
    ));
    app.world_mut()
        .spawn((Pressed, UiAction::Runtime(Action::StopAll)));
    app.update();
    let actions = &app.world().resource::<Actions>().0;
    assert_eq!(actions.len(), 1);
    assert!(matches!(actions.front(), Some(Action::StopAll)));
    let mut model = app.world_mut().resource_mut::<Model>();
    model.close();
    model.open(Provider::Host(HostProvider::Tailscale), &provider_context());
    assert!(model.modal.as_ref().unwrap().editors[1].text.is_empty());
}

/// Validation failures stay editable; successful apply consumes the private draft.
#[test]
fn apply_retains_invalid_edits_and_consumes_only_a_valid_provider_draft() {
    let context = provider_context();
    let mut model = Model::default();
    model.open(Provider::Host(HostProvider::Tailscale), &context);
    assert!(model.apply(&context).is_none());
    assert!(model.modal.as_ref().unwrap().error.is_some());
    model.modal.as_mut().unwrap().editors[1]
        .text
        .push_str(&"b".repeat(64));
    let intent = model.apply(&context).expect("validated pairing intent");
    assert!(matches!(intent, ConfigurationIntent::Tailscale { .. }));
    assert!(model.modal.is_none());
    assert!(model.focus.is_none());
}

/// Responsive geometry preserves bounded rows and lets the scrollbar reach both ends.
#[test]
fn responsive_grid_and_scrollbar_keep_large_catalogs_bounded_and_reachable() {
    assert_eq!(GridLayout::for_window(1160., 820.).columns, 4);
    assert_eq!(GridLayout::for_window(400., 600.).columns, 1);
    assert_eq!(GridLayout::for_window(1800., 1000.).columns, 6);
    let mut model = Model {
        matches: 10_000,
        ..default()
    };
    model.scroll = model.max_scroll();
    assert!(model.first_row() > 2000);
    let (top, height) = model.thumb_geometry(500.);
    assert_eq!(top + height, 500.);
    assert_eq!(height, 24.);
    model.matches = 0;
    assert_eq!(model.max_scroll(), 0.);
    assert_eq!(model.thumb_geometry(500.), (0., 500.));
}

/// A controller-only launch cannot silently become an empty native playback host.
#[test]
fn native_provider_apply_in_controller_mode_retains_actionable_modal_error() {
    let runtime = Runtime::new(&config()).unwrap();
    let network = crate::network_app::NetworkState::for_ui_test(runtime.snapshot.clone());
    let mut model = Model::default();
    model.open(Provider::Host(HostProvider::Native), &provider_context());
    let mut app = App::new();
    app.insert_resource(runtime)
        .insert_resource(network)
        .insert_resource(model)
        .init_resource::<Actions>()
        .init_resource::<input::PendingActions>()
        .init_resource::<InputFocus>()
        .add_systems(Update, (queue_test_activations, buttons).chain());
    app.world_mut().spawn((Pressed, UiAction::Apply));
    app.update();
    assert!(app.world().resource::<Actions>().0.is_empty());
    assert!(
        app.world()
            .resource::<Model>()
            .modal
            .as_ref()
            .unwrap()
            .error
            .as_ref()
            .unwrap()
            .contains("controller-only")
    );
}

/// Pointer capture survives rebuilt tracks, and a modal releases it before scrolling.
#[test]
fn scrollbar_drag_tracks_pointer_and_modal_cancels_capture() {
    let mut app = App::new();
    let mut mouse = ButtonInput::<MouseButton>::default();
    mouse.press(MouseButton::Left);
    app.insert_resource(mouse)
        .insert_resource(Model {
            matches: 10_000,
            ..default()
        })
        .add_systems(Update, scrollbar);
    let track = app
        .world_mut()
        .spawn((
            Scrollbar,
            ComputedNode {
                size: Vec2::new(8., 500.),
                inverse_scale_factor: 1.,
                ..default()
            },
            Pressed,
            bevy::ui::RelativeCursorPosition {
                normalized: Some(Vec2::new(0., 0.5)),
                ..default()
            },
        ))
        .id();
    app.update();
    let model = app.world().resource::<Model>();
    assert!(model.dragging);
    assert_eq!(model.scroll, model.max_scroll());
    app.world_mut().entity_mut(track).insert((
        (),
        bevy::ui::RelativeCursorPosition {
            normalized: Some(Vec2::new(0., -0.5)),
            ..default()
        },
    ));
    app.update();
    assert_eq!(app.world().resource::<Model>().scroll, 0.);
    app.world_mut()
        .resource_mut::<Model>()
        .open(Provider::Host(HostProvider::Native), &provider_context());
    app.world_mut().entity_mut(track).insert((
        Pressed,
        bevy::ui::RelativeCursorPosition {
            normalized: Some(Vec2::new(0., 0.5)),
            ..default()
        },
    ));
    app.update();
    let model = app.world().resource::<Model>();
    assert!(!model.dragging);
    assert_eq!(model.scroll, 0.);
}

/// Catalog selection cannot switch a controller-only runtime into local playback.
#[test]
fn controller_catalog_apply_preserves_remote_destination_and_draft() {
    for provider in [
        CatalogProvider::Bundled,
        CatalogProvider::LocalDirectory,
        CatalogProvider::GitHttps,
    ] {
        let runtime = Runtime::new(&config()).unwrap();
        let network = crate::network_app::NetworkState::for_ui_test(runtime.snapshot.clone());
        let mut model = Model::default();
        model.open(Provider::Catalog(provider), &provider_context());
        let mut app = App::new();
        app.insert_resource(runtime)
            .insert_resource(network)
            .insert_resource(model)
            .init_resource::<Actions>()
            .init_resource::<input::PendingActions>()
            .init_resource::<InputFocus>()
            .add_systems(Update, (queue_test_activations, buttons).chain());
        app.world_mut().spawn((Pressed, UiAction::Apply));
        app.update();
        assert!(app.world().resource::<Actions>().0.is_empty());
        let model = app.world().resource::<Model>();
        let modal = model
            .modal
            .as_ref()
            .expect("blocked apply retains the draft");
        assert_eq!(modal.provider(), Some(Provider::Catalog(provider)));
        assert!(modal.error.as_ref().unwrap().contains("remote host"));
    }
}

/// Scrolling a short form reaches its bounded body without moving the clip collection.
#[test]
fn modal_wheel_scroll_is_clamped_and_does_not_move_catalog_rows() {
    for source_controls in [false, true] {
        let mut app = App::new();
        let mut model = Model {
            scroll: 903.,
            matches: 10_000,
            ..default()
        };
        if source_controls {
            model.open_sources("100.101.102.103:7010");
        } else {
            model.open(
                Provider::Catalog(CatalogProvider::GitHttps),
                &provider_context(),
            );
        }
        app.insert_resource(model)
            .add_message::<MouseWheel>()
            .add_systems(Update, scroll);
        let window = app
            .world_mut()
            .spawn((PrimaryWindow, Window::default()))
            .id();
        let body = app
            .world_mut()
            .spawn((
                ModalBody,
                ComputedNode {
                    size: Vec2::new(500., 100.),
                    content_size: Vec2::new(500., 500.),
                    inverse_scale_factor: 1.,
                    ..default()
                },
                ScrollPosition::default(),
            ))
            .id();
        app.world_mut().write_message(MouseWheel {
            unit: bevy::input::mouse::MouseScrollUnit::Pixel,
            phase: bevy::input::touch::TouchPhase::Moved,
            x: 0.,
            y: -800.,
            window,
        });
        app.update();
        assert_eq!(app.world().get::<ScrollPosition>(body).unwrap().y, 400.);
        {
            let model = app.world().resource::<Model>();
            assert_eq!(model.scroll, 903.);
            assert_eq!(model.modal.as_ref().unwrap().scroll, 400.);
        }
        // Rebuilding source controls after asynchronous updates must retain the user's position.
        app.world_mut().resource_mut::<Model>().revision += 1;
        app.update();
        assert_eq!(app.world().get::<ScrollPosition>(body).unwrap().y, 400.);
        // A source removal or viewport resize shrinks content and clamps stale travel.
        app.world_mut()
            .get_mut::<ComputedNode>(body)
            .unwrap()
            .content_size
            .y = 150.;
        app.update();
        assert_eq!(app.world().get::<ScrollPosition>(body).unwrap().y, 50.);
        assert_eq!(app.world().resource::<Model>().scroll, 903.);
    }
}

/// A complete compact card retains its two-line title and action row below the preview.
#[test]
fn short_window_cards_keep_complete_controls_and_scroll_small_viewports() {
    for (width, height) in [(720., 520.), (400., 520.), (1160., 520.), (1160., 820.)] {
        let layout = GridLayout::for_window(width, height);
        assert!((134. ..=282.).contains(&layout.card_height));
        assert!(layout.card_width > 0.);
        assert!(layout.preview_height >= 24.);
        // Title 44px + controls 20px + padding/border 26px + two 10px gaps.
        assert_eq!(
            layout.preview_height + 44. + 20. + 26. + 20.,
            layout.card_height
        );
        assert_eq!(layout.rows, 1);
    }
    assert_eq!(GridLayout::for_window(1160., 820.).preview_height, 172.);
    assert_eq!(GridLayout::for_window(720., 520.).preview_height, 34.);
}

/// Build the real wheel-routing system with measured viewport geometry and no renderer.
fn wheel_app(offset: f32) -> (App, Entity, Entity) {
    let mut app = App::new();
    app.insert_resource(Model {
        scroll: offset,
        matches: 10_000,
        ..default()
    })
    .add_message::<MouseWheel>()
    .add_systems(Update, scroll);
    let window = app
        .world_mut()
        .spawn((PrimaryWindow, Window::default()))
        .id();
    let body = app
        .world_mut()
        .spawn((
            CatalogBody,
            ComputedNode {
                size: Vec2::new(1000., 450.),
                inverse_scale_factor: 1.,
                ..default()
            },
            ScrollPosition(Vec2::new(0., offset)),
        ))
        .id();
    (app, window, body)
}

/// Send proportional wheel input through the same message path used by the desktop.
fn wheel_input(app: &mut App, window: Entity, unit: bevy::input::mouse::MouseScrollUnit, y: f32) {
    app.world_mut().write_message(MouseWheel {
        unit,
        phase: bevy::input::touch::TouchPhase::Moved,
        x: 0.,
        y,
        window,
    });
}

/// Every fractional trackpad event moves the native viewport without changing virtual row origin.
#[test]
fn tiny_trackpad_deltas_scroll_immediately_in_fractional_pixels() {
    let (mut app, window, body) = wheel_app(0.);
    for frame in 1..=8 {
        wheel_input(
            &mut app,
            window,
            bevy::input::mouse::MouseScrollUnit::Pixel,
            -0.25,
        );
        app.update();
        assert_eq!(
            app.world().get::<ScrollPosition>(body).unwrap().y,
            frame as f32 * 0.25
        );
        assert_eq!(app.world().resource::<Model>().first_row(), 0);
    }
}

/// Line input has a fixed logical distance while direction changes retain every pixel.
#[test]
fn line_wheels_and_reversed_trackpad_travel_remain_continuous() {
    let (mut app, window, body) = wheel_app(500.);
    for (unit, delta, expected) in [
        (bevy::input::mouse::MouseScrollUnit::Line, -2., 580.),
        (bevy::input::mouse::MouseScrollUnit::Pixel, 0.5, 579.5),
        (bevy::input::mouse::MouseScrollUnit::Pixel, -0.25, 579.75),
    ] {
        wheel_input(&mut app, window, unit, delta);
        app.update();
        assert_eq!(app.world().get::<ScrollPosition>(body).unwrap().y, expected);
    }
}

/// Coalesced and OS momentum deltas have no frame cap, inertia or deferred overshoot.
#[test]
fn wheel_bursts_apply_all_travel_and_clamp_only_at_content_edges() {
    let (mut app, window, body) = wheel_app(0.);
    for _ in 0..100 {
        wheel_input(
            &mut app,
            window,
            bevy::input::mouse::MouseScrollUnit::Pixel,
            -1000.,
        );
    }
    app.update();
    assert_eq!(app.world().get::<ScrollPosition>(body).unwrap().y, 100_000.);
    wheel_input(
        &mut app,
        window,
        bevy::input::mouse::MouseScrollUnit::Pixel,
        -1_000_000.,
    );
    app.update();
    let maximum = app.world().resource::<Model>().max_scroll();
    assert_eq!(app.world().get::<ScrollPosition>(body).unwrap().y, maximum);
    wheel_input(
        &mut app,
        window,
        bevy::input::mouse::MouseScrollUnit::Pixel,
        0.5,
    );
    app.update();
    assert_eq!(
        app.world().get::<ScrollPosition>(body).unwrap().y,
        maximum - 0.5
    );
    app.update();
    assert_eq!(
        app.world().get::<ScrollPosition>(body).unwrap().y,
        maximum - 0.5
    );
}

/// Logical viewport measurements govern bounds and changes in filtering clamp immediately.
#[test]
fn measured_viewport_and_filter_changes_clamp_native_scroll_bounds() {
    let (mut app, _, body) = wheel_app(1_000_000.);
    app.world_mut().entity_mut(body).insert(ComputedNode {
        size: Vec2::new(2000., 1067.),
        inverse_scale_factor: 0.5,
        ..default()
    });
    app.update();
    let model = app.world().resource::<Model>();
    assert_eq!(model.viewport_height, 533.5);
    assert_eq!(model.max_scroll(), model.content_height() - 533.5);
    assert_eq!(
        app.world().get::<ScrollPosition>(body).unwrap().y,
        model.max_scroll()
    );
    app.world_mut().resource_mut::<Model>().matches = 1;
    app.update();
    assert_eq!(app.world().get::<ScrollPosition>(body).unwrap().y, 0.);
    app.world_mut().resource_mut::<Model>().matches = 10_000;
    app.update();
    assert_eq!(app.world().get::<ScrollPosition>(body).unwrap().y, 0.);
}

/// Search starts at the first result while a provider dialog preserves catalog position.
#[test]
fn search_resets_pixel_offset_and_modal_navigation_preserves_it() {
    let (mut app, window) = input_app();
    app.world_mut().resource_mut::<Model>().scroll = 123.5;
    key(
        &mut app,
        window,
        Key::Character("x".into()),
        KeyCode::KeyX,
        Some("x"),
    );
    app.update();
    assert_eq!(app.world().resource::<Model>().scroll, 0.);
    let mut model = app.world_mut().resource_mut::<Model>();
    model.scroll = 123.5;
    model.open(
        Provider::Catalog(CatalogProvider::GitHttps),
        &provider_context(),
    );
    assert_eq!(model.scroll, 123.5);
}

/// Retina normalization applies equally to catalog and modal pixel deltas; lines stay logical.
#[test]
fn trackpad_offsets_use_logical_pixels_on_scaled_windows() {
    let (mut app, window, body) = wheel_app(0.);
    app.world_mut()
        .get_mut::<Window>(window)
        .unwrap()
        .resolution
        .set_scale_factor_override(Some(2.));
    wheel_input(
        &mut app,
        window,
        bevy::input::mouse::MouseScrollUnit::Pixel,
        -0.5,
    );
    app.update();
    assert_eq!(app.world().get::<ScrollPosition>(body).unwrap().y, 0.25);
    wheel_input(
        &mut app,
        window,
        bevy::input::mouse::MouseScrollUnit::Line,
        -1.,
    );
    app.update();
    assert_eq!(app.world().get::<ScrollPosition>(body).unwrap().y, 40.25);
    app.world_mut().resource_mut::<Model>().open(
        Provider::Catalog(CatalogProvider::GitHttps),
        &provider_context(),
    );
    let modal = app
        .world_mut()
        .spawn((
            ModalBody,
            ComputedNode {
                size: Vec2::new(1000., 200.),
                content_size: Vec2::new(1000., 1000.),
                inverse_scale_factor: 0.5,
                ..default()
            },
            ScrollPosition::default(),
        ))
        .id();
    wheel_input(
        &mut app,
        window,
        bevy::input::mouse::MouseScrollUnit::Pixel,
        -1.,
    );
    app.update();
    assert_eq!(app.world().get::<ScrollPosition>(modal).unwrap().y, 0.5);
    assert_eq!(app.world().get::<ScrollPosition>(body).unwrap().y, 40.25);
}

/// Secondary-window and non-finite input never changes the primary collection.
#[test]
fn wheel_ignores_other_windows_and_nonfinite_offsets() {
    let (mut app, window, body) = wheel_app(123.5);
    let secondary = app.world_mut().spawn(Window::default()).id();
    for (target, delta) in [
        (secondary, -200.),
        (window, f32::INFINITY),
        (window, f32::NAN),
    ] {
        wheel_input(
            &mut app,
            target,
            bevy::input::mouse::MouseScrollUnit::Pixel,
            delta,
        );
    }
    app.update();
    assert_eq!(app.world().get::<ScrollPosition>(body).unwrap().y, 123.5);
}

/// Grabbing an existing thumb retains its fractional offset until the pointer actually moves.
#[test]
fn scrollbar_grab_preserves_offset_and_tracks_fractional_motion() {
    let (mut app, _, body) = wheel_app(1000.5);
    let mut mouse = ButtonInput::<MouseButton>::default();
    mouse.press(MouseButton::Left);
    app.insert_resource(mouse)
        .add_systems(Update, scrollbar.after(scroll));
    app.update();
    let (top, _) = app.world().resource::<Model>().thumb_geometry(500.);
    let pointer = top + 5.;
    let track = app
        .world_mut()
        .spawn((
            Scrollbar,
            Pressed,
            ComputedNode {
                size: Vec2::new(8., 1000.),
                inverse_scale_factor: 0.5,
                ..default()
            },
            bevy::ui::RelativeCursorPosition {
                normalized: Some(Vec2::new(0., pointer / 500. - 0.5)),
                ..default()
            },
        ))
        .id();
    app.update();
    let grabbed = app.world().get::<ScrollPosition>(body).unwrap().y;
    assert!((grabbed - 1000.5).abs() < 0.05);
    app.world_mut()
        .entity_mut(track)
        .insert(bevy::ui::RelativeCursorPosition {
            normalized: Some(Vec2::new(0., (pointer + 0.25) / 500. - 0.5)),
            ..default()
        });
    app.update();
    let moved = app.world().get::<ScrollPosition>(body).unwrap().y;
    assert!(moved > grabbed);
    assert!(moved.fract() > 0.);
}

/// Create the real mixed-media catalog with matching local host/session identities.
fn placement_runtime() -> Runtime {
    let mut runtime = Runtime::new(&config()).unwrap();
    runtime.catalog = std::sync::Arc::new(
        crate::catalog::load_local(&crate::app::resource_root().join("sample-packs/podcast-pack"))
            .unwrap(),
    );
    runtime.host = crate::host::Host::new("placement-ui".into(), runtime.catalog.clone());
    runtime.snapshot = runtime.host.snapshot();
    runtime.destination = Destination::Local;
    runtime
}

/// Select a real visual clip rather than assuming fixture ordering.
fn visual_clip(runtime: &Runtime) -> &Clip {
    runtime
        .catalog
        .clips
        .iter()
        .find(|clip| crate::host::Host::default_placement(clip).is_some())
        .unwrap()
}

/// Strict decimal parsing preserves basis points and rejects non-finite or out-of-range input.
#[test]
fn placement_percentages_are_finite_bounded_and_exact() {
    for (text, expected) in [("0.01", 1), ("2.5", 250), ("33.33", 3333), ("100", 10000)] {
        assert_eq!(PlacementModal::percentage(text, true), Some(expected));
    }
    assert_eq!(PlacementModal::percentage("0", false), Some(0));
    for text in [
        "0", "-1", "NaN", "inf", "1e1", "100.01", "0.001", "", "1.2.3", "65535",
    ] {
        assert_eq!(PlacementModal::percentage(text, true), None, "{text}");
    }
}

/// Apply changes all live copies of a clip, excludes stopping/history, and remembers future plays.
#[test]
fn placement_apply_updates_live_occurrences_without_playing_another_copy() {
    use crate::commands::{InstanceSnapshot, InstanceState};
    let mut runtime = placement_runtime();
    let clip = visual_clip(&runtime).clone();
    let network = crate::network_app::NetworkState::for_ui_test(runtime.snapshot.clone());
    let mut model = Model::default();
    model.sync_placements(PlacementScope::current(&runtime, &network));
    for (index, state) in [
        InstanceState::Preparing,
        InstanceState::Playing,
        InstanceState::Stopping,
        InstanceState::Finished,
    ]
    .into_iter()
    .enumerate()
    {
        runtime.snapshot.instances.push(InstanceSnapshot {
            instance_id: format!("copy-{index}"),
            clip_id: clip.identity.clone(),
            generation: index as u64,
            state,
            placement: None,
        });
    }
    runtime.snapshot.instances.push(InstanceSnapshot {
        instance_id: "other".into(),
        clip_id: "different/clip".into(),
        generation: 9,
        state: InstanceState::Playing,
        placement: None,
    });
    model.open_placement(&clip.identity, &runtime, &network);
    model.placement_modal.as_mut().unwrap().editors[0].text = "33.33".into();
    let actions = model.apply_placement(&runtime.snapshot, false);
    assert_eq!(actions.len(), 2);
    for (index, action) in actions.iter().enumerate() {
        assert!(
            matches!(action, Action::SetVisual { instance_id, placement: Some(value) } if instance_id == &format!("copy-{index}") && value.width_bps == 3333)
        );
    }
    assert!(model.placement_modal.is_none());
    assert!(
        matches!(model.play_action(&clip.identity), Action::PlayVisual { placement, .. } if placement.width_bps == 3333)
    );
}

/// Invalid drafts emit nothing; Cancel also discards a pending Reset without touching remembered values.
#[test]
fn placement_validation_and_cancel_never_mutate_committed_values() {
    let runtime = placement_runtime();
    let clip = visual_clip(&runtime);
    let defaults = crate::host::Host::default_placement(clip).unwrap();
    let changed = VisualPlacement {
        width_bps: 2345,
        ..defaults
    };
    let network = crate::network_app::NetworkState::for_ui_test(runtime.snapshot.clone());
    let mut model = Model::default();
    model.placements.insert(clip.identity.clone(), changed);
    model.open_placement(&clip.identity, &runtime, &network);
    model.placement_modal.as_mut().unwrap().editors[2].text = "NaN".into();
    assert!(model.apply_placement(&runtime.snapshot, true).is_empty());
    assert!(model.placement_modal.as_ref().unwrap().error.is_some());
    assert_eq!(model.placements[&clip.identity], changed);
    model.placement_modal.as_mut().unwrap().reset();
    model.close();
    assert_eq!(model.placements[&clip.identity], changed);
}

/// Reset is explicit: apply clears active overrides, and Send uses the original catalog playback path.
#[test]
fn placement_reset_and_send_restore_catalog_defaults() {
    use crate::commands::{InstanceSnapshot, InstanceState};
    let mut runtime = placement_runtime();
    let clip = visual_clip(&runtime).clone();
    let defaults = crate::host::Host::default_placement(&clip).unwrap();
    let changed = VisualPlacement {
        width_bps: 2345,
        ..defaults
    };
    runtime.snapshot.instances.push(InstanceSnapshot {
        instance_id: "live".into(),
        clip_id: clip.identity.clone(),
        generation: 1,
        state: InstanceState::Playing,
        placement: Some(changed),
    });
    let network = crate::network_app::NetworkState::for_ui_test(runtime.snapshot.clone());
    let mut model = Model::default();
    model.placements.insert(clip.identity.clone(), changed);
    model.open_placement(&clip.identity, &runtime, &network);
    model.placement_modal.as_mut().unwrap().reset();
    let actions = model.apply_placement(&runtime.snapshot, true);
    assert!(
        matches!(&actions[0], Action::SetVisual { instance_id, placement: None } if instance_id == "live")
    );
    assert!(matches!(&actions[1], Action::Play(id) if id == &clip.identity));
    assert!(model.placements.is_empty());
    assert_eq!(crate::host::Host::default_placement(&clip), Some(defaults));
}

/// A normal Send commits the edited percentages and appends exactly one adjusted play.
#[test]
fn placement_send_uses_the_validated_draft() {
    let runtime = placement_runtime();
    let clip = visual_clip(&runtime);
    let network = crate::network_app::NetworkState::for_ui_test(runtime.snapshot.clone());
    let mut model = Model::default();
    model.open_placement(&clip.identity, &runtime, &network);
    model.placement_modal.as_mut().unwrap().editors[1].text = "12.34".into();
    let actions = model.apply_placement(&runtime.snapshot, true);
    assert!(
        matches!(actions.as_slice(), [Action::PlayVisual { clip_id, placement }] if clip_id == &clip.identity && placement.x_bps == 1234)
    );
}

/// Active authoritative state wins over a controller's stale remembered draft; audio has no editor.
#[test]
fn placement_editor_uses_authoritative_active_state_and_excludes_audio() {
    use crate::commands::{InstanceSnapshot, InstanceState};
    let mut runtime = placement_runtime();
    let clip = visual_clip(&runtime).clone();
    let defaults = crate::host::Host::default_placement(&clip).unwrap();
    let current = VisualPlacement {
        width_bps: 7654,
        ..defaults
    };
    let stale = VisualPlacement {
        width_bps: 1234,
        ..defaults
    };
    runtime.snapshot.instances.push(InstanceSnapshot {
        instance_id: "other-controller".into(),
        clip_id: clip.identity.clone(),
        generation: 1,
        state: InstanceState::Playing,
        placement: Some(current),
    });
    let network = crate::network_app::NetworkState::for_ui_test(runtime.snapshot.clone());
    let mut model = Model::default();
    model.placements.insert(clip.identity.clone(), stale);
    model.open_placement(&clip.identity, &runtime, &network);
    assert_eq!(
        model.placement_modal.as_ref().unwrap().placement().unwrap(),
        current
    );
    assert!(!model.allows(&UiAction::Play(clip.identity.clone())));
    assert!(model.allows(&UiAction::Runtime(Action::StopAll)));
    model.close();
    let audio = runtime
        .catalog
        .clips
        .iter()
        .find(|clip| crate::host::Host::default_placement(clip).is_none())
        .unwrap();
    model.open_placement(&audio.identity, &runtime, &network);
    assert!(model.placement_modal.is_none());
}

/// Destination, host lifetime, revision and disconnection each invalidate session-only overrides.
#[test]
fn placement_adjustments_never_cross_destination_session_or_revision() {
    let runtime = placement_runtime();
    let clip = visual_clip(&runtime);
    let defaults = crate::host::Host::default_placement(clip).unwrap();
    let original = PlacementScope::from_snapshot(false, &runtime.snapshot);
    for scope in [
        Some(PlacementScope {
            remote: true,
            ..original.clone()
        }),
        Some(PlacementScope {
            session: "next-session".into(),
            ..original.clone()
        }),
        Some(PlacementScope {
            revision: "next-revision".into(),
            ..original.clone()
        }),
        None,
    ] {
        let mut model = Model::default();
        model.sync_placements(Some(original.clone()));
        model.placements.insert(clip.identity.clone(), defaults);
        model.placement_order.push_back(clip.identity.clone());
        model.placement_modal = Some(PlacementModal::new(
            clip.identity.clone(),
            clip.metadata.title.clone(),
            defaults,
            defaults,
        ));
        model.focus = Some(Field::Placement(0));
        model.sync_placements(scope);
        assert!(model.placements.is_empty());
        assert!(model.placement_order.is_empty());
        assert!(model.placement_modal.is_none());
        assert!(model.focus.is_none());
    }
}

/// Large local catalogs cannot grow the experimental remembered adjustments without bound.
#[test]
fn placement_override_storage_evicts_the_oldest_applied_clip() {
    let runtime = placement_runtime();
    let clip = visual_clip(&runtime);
    let defaults = crate::host::Host::default_placement(clip).unwrap();
    let mut model = Model::default();
    for index in 0..257 {
        model.placement_modal = Some(PlacementModal::new(
            format!("pack/clip-{index}"),
            "Clip".into(),
            defaults,
            defaults,
        ));
        model.apply_placement(&runtime.snapshot, false);
    }
    assert_eq!(model.placements.len(), 256);
    assert_eq!(model.placement_order.len(), 256);
    assert!(!model.placements.contains_key("pack/clip-0"));
    assert!(model.placements.contains_key("pack/clip-256"));
}

/// Remote editing uses host-supplied placement defaults instead of reconstructing local catalog metadata.
#[test]
fn placement_remote_defaults_and_reset_come_from_the_host_projection() {
    let mut runtime = placement_runtime();
    runtime.destination = Destination::Remote;
    let remote_defaults = VisualPlacement {
        width_bps: 4321,
        x_bps: 2345,
        y_bps: 6789,
    };
    let mut network = crate::network_app::NetworkState::for_ui_test(runtime.snapshot.clone());
    network.connected = true;
    network.snapshot = Some(runtime.snapshot.clone());
    network.remote_catalog = Some(crate::network_app::RemoteCatalogView {
        pack_id: "remote".into(),
        title: "Host pack".into(),
        revision: runtime.snapshot.catalog_revision.clone(),
        clips: vec![crate::network_app::RemoteClip {
            clip_id: "remote/visual".into(),
            title: "Remote visual".into(),
            tags: Vec::new(),
            category: "reactions".into(),
            media_type: "gif".into(),
            default_placement: Some(remote_defaults),
        }],
    });
    let mut model = Model::default();
    model.sync_placements(PlacementScope::current(&runtime, &network));
    model.open_placement("remote/visual", &runtime, &network);
    let modal = model.placement_modal.as_mut().unwrap();
    assert_eq!(modal.placement().unwrap(), remote_defaults);
    modal.step(0, 1);
    modal.reset();
    assert_eq!(modal.placement().unwrap(), remote_defaults);
    assert!(
        matches!(model.apply_placement(network.snapshot.as_ref().unwrap(), true).as_slice(), [Action::Play(id)] if id == "remote/visual")
    );
}

/// Keyboard edits remain isolated, and pointer nudges clamp to the same validated percentage range.
#[test]
fn placement_fields_route_keyboard_input_and_nudges_preserve_bounds() {
    let (mut app, window) = input_app();
    let runtime = placement_runtime();
    let scope = PlacementScope::from_snapshot(false, &runtime.snapshot);
    let network = crate::network_app::NetworkState::for_ui_test(runtime.snapshot.clone());
    app.insert_resource(runtime).insert_resource(network);
    let defaults = VisualPlacement {
        width_bps: 1,
        x_bps: 0,
        y_bps: 10000,
    };
    {
        let mut model = app.world_mut().resource_mut::<Model>();
        model.sync_placements(Some(scope));
        model.placement_modal = Some(PlacementModal::new(
            "test/clip".into(),
            "Test clip".into(),
            defaults,
            defaults,
        ));
        model.revision = model.revision.wrapping_add(1);
    }
    app.update();
    let editor = app
        .world()
        .resource::<Model>()
        .placement_modal
        .as_ref()
        .unwrap()
        .editors[0]
        .entity
        .unwrap();
    assert_eq!(app.world().resource::<InputFocus>().get(), Some(editor));
    assert!(app.world().get::<TextCursorStyle>(editor).is_some());
    app.world_mut()
        .get_mut::<EditableText>(editor)
        .unwrap()
        .queue_edit(bevy::text::TextEdit::SelectAll);
    key(
        &mut app,
        window,
        Key::Character("33.33".into()),
        KeyCode::Digit3,
        Some("33.33"),
    );
    app.update();
    app.world_mut().resource_mut::<Runtime>().snapshot.sequence += 1;
    app.world_mut()
        .resource_mut::<crate::network_app::NetworkState>()
        .status = "Snapshot changed while editing placement".into();
    app.update();
    assert_eq!(app.world().resource::<InputFocus>().get(), Some(editor));
    assert_eq!(
        app.world()
            .resource::<Model>()
            .placement_modal
            .as_ref()
            .unwrap()
            .editors[0]
            .entity,
        Some(editor)
    );
    {
        let mut model = app.world_mut().resource_mut::<Model>();
        let modal = model.placement_modal.as_mut().unwrap();
        assert_eq!(modal.placement().unwrap().width_bps, 3333);
        modal.reset();
        modal.step(0, -1);
        modal.step(1, -1);
        modal.step(2, 1);
        assert_eq!(modal.placement().unwrap(), defaults);
    }
    app.update();
    let editors = app
        .world()
        .resource::<Model>()
        .placement_modal
        .as_ref()
        .unwrap()
        .editors
        .iter()
        .map(|editor| editor.entity.unwrap())
        .collect::<Vec<_>>();
    assert_ne!(editors[0], editor);
    assert!(app.world().get_entity(editor).is_err());
    assert_eq!(
        app.world()
            .get::<EditableText>(editors[0])
            .unwrap()
            .value()
            .to_string(),
        "0.01"
    );
    key(&mut app, window, Key::Tab, KeyCode::Tab, None);
    app.update();
    assert_eq!(app.world().resource::<InputFocus>().get(), Some(editors[1]));
    assert!(app.world().resource::<Actions>().0.is_empty());
    key(&mut app, window, Key::Escape, KeyCode::Escape, None);
    app.update();
    assert!(app.world().resource::<Model>().placement_modal.is_none());
    assert!(app.world().resource::<Actions>().0.is_empty());
}

/// Sources summarize simultaneous permission instead of presenting an exclusive selector.
#[test]
fn from_summary_includes_local_and_multiple_controllers() {
    assert_eq!(Model::source_summary(true, 2), "Local + 2 controllers");
    assert_eq!(Model::source_summary(true, 0), "Local + 0 controllers");
    assert_eq!(Model::source_summary(false, 1), "1 controller");
}

/// Source forms use ephemeral editor state and never produce a provider destination intent.
#[test]
fn from_drafts_cancel_secrets_and_keep_to_selection_independent() {
    let mut model = Model::default();
    model.open_sources("100.101.102.103:7010");
    let modal = model.modal.as_mut().unwrap();
    modal.select_source(SourceSelection::Remote("alice".into()));
    modal.show_source_details(SourceDetails::Credentials);
    model
        .editor(Field::Configuration(1))
        .unwrap()
        .text
        .push_str("private-admin-key");
    assert!(model.apply(&provider_context()).is_none());
    assert!(model.modal.is_some());
    assert!(!model.allows(&UiAction::Runtime(Action::Play("pack/clip".into()))));
    assert!(model.allows(&UiAction::Runtime(Action::StopAll)));
    model.close();
    model.open_sources("100.101.102.103:7010");
    assert!(model.modal.as_ref().unwrap().editors[1].text.is_empty());
    assert_eq!(
        model.modal.as_ref().unwrap().editors[2].text,
        "100.101.102.103:7010"
    );
}

/// Exercise source buttons without dispatching the independently owned network executor.
fn sources_app() -> App {
    let runtime = Runtime::new(&config()).unwrap();
    let mut network = crate::network_app::NetworkState::for_ui_test(runtime.snapshot.clone());
    network.controllers = ["alice", "bob"]
        .into_iter()
        .map(|id| crate::network_app::ControllerSource {
            id: id.into(),
            name: id.into(),
            enabled: true,
            pairing_secret: None,
            invitation_url: None,
            invitation_status: "No invitation".into(),
        })
        .collect();
    let mut model = Model::default();
    model.open_sources("100.101.102.103:7010");
    let mut app = App::new();
    app.insert_resource(runtime)
        .insert_resource(network)
        .insert_resource(model)
        .init_resource::<Actions>()
        .init_resource::<input::PendingActions>()
        .init_resource::<InputFocus>()
        .add_systems(Update, (queue_test_activations, buttons).chain());
    app
}

/// Disabling one controller never emits a destination change or revokes its peers/local source.
#[test]
fn source_buttons_enqueue_independent_permissions_without_changing_to() {
    use crate::network_app::NetworkAction;
    let mut app = sources_app();
    app.world_mut().spawn((
        Pressed,
        UiAction::Network(NetworkAction::SetControllerEnabled {
            id: "alice".into(),
            enabled: false,
        }),
    ));
    app.world_mut().spawn((
        Pressed,
        UiAction::Network(NetworkAction::SetControllerEnabled {
            id: "bob".into(),
            enabled: true,
        }),
    ));
    app.update();
    let actions = &app
        .world()
        .resource::<crate::network_app::NetworkState>()
        .actions;
    assert_eq!(actions.len(), 2);
    assert!(actions.iter().any(|action| matches!(action, NetworkAction::SetControllerEnabled {id, enabled: false} if id == "alice")));
    assert!(actions.iter().any(|action| matches!(action, NetworkAction::SetControllerEnabled {id, enabled: true} if id == "bob")));
    assert!(app.world().resource::<Actions>().0.is_empty());
    assert_eq!(
        app.world().resource::<Runtime>().destination,
        Destination::Remote
    );
}

/// Explicit save transfers a key once and clears the masked draft before asynchronous storage.
#[test]
fn secure_save_consumes_only_the_admin_secret_draft() {
    use crate::network_app::NetworkAction;
    let mut app = sources_app();
    {
        let mut model = app.world_mut().resource_mut::<Model>();
        let modal = model.modal.as_mut().unwrap();
        modal.select_source(SourceSelection::Remote("alice".into()));
        modal.show_source_details(SourceDetails::Credentials);
    }
    app.world_mut()
        .resource_mut::<Model>()
        .modal
        .as_mut()
        .unwrap()
        .editors[1]
        .text
        .push_str("  secret-key  ");
    app.world_mut().spawn((Pressed, UiAction::SaveAdminSecret));
    app.update();
    assert!(
        matches!(app.world().resource::<crate::network_app::NetworkState>().actions.front(), Some(NetworkAction::SaveAdminSecret {secret}) if secret == "secret-key")
    );
    assert!(
        app.world()
            .resource::<Model>()
            .modal
            .as_ref()
            .unwrap()
            .editors[1]
            .text
            .is_empty()
    );
    assert!(app.world().resource::<Actions>().0.is_empty());
}

/// Inbound listener editing must never replace the independent outbound host endpoint.
#[test]
fn listener_apply_keeps_outbound_to_address() {
    use crate::network_app::NetworkAction;
    let mut app = sources_app();
    {
        let mut model = app.world_mut().resource_mut::<Model>();
        let modal = model.modal.as_mut().unwrap();
        modal.select_source(SourceSelection::Remote("alice".into()));
        modal.show_source_details(SourceDetails::Listener);
    }
    app.world_mut()
        .resource_mut::<crate::network_app::NetworkState>()
        .address = "ws://100.111.112.113:7010/control".into();
    app.world_mut().spawn((Pressed, UiAction::EnableListener));
    app.update();
    let network = app.world().resource::<crate::network_app::NetworkState>();
    assert_eq!(network.address, "ws://100.111.112.113:7010/control");
    assert_eq!(network.listen_address, "100.101.102.103:7010");
    assert!(matches!(
        network.actions.front(),
        Some(NetworkAction::EnableHost)
    ));
    assert!(app.world().resource::<Actions>().0.is_empty());
}

#[test]
fn listener_accepts_tailnet_ip_without_an_explicit_port() {
    let mut app = sources_app();
    {
        let mut model = app.world_mut().resource_mut::<Model>();
        let modal = model.modal.as_mut().unwrap();
        modal.select_source(SourceSelection::Remote("alice".into()));
        modal.show_source_details(SourceDetails::Listener);
        modal.editors[2].text = "100.101.102.103".into();
    }
    app.world_mut().spawn((Pressed, UiAction::EnableListener));
    app.update();
    let network = app.world().resource::<crate::network_app::NetworkState>();
    assert_eq!(network.listen_address, "100.101.102.103:48732");
    assert!(matches!(
        network.actions.front(),
        Some(crate::network_app::NetworkAction::EnableHost)
    ));
}

/// Press one transient source-navigation button without leaving stale synthetic interactions.
fn source_press(app: &mut App, action: UiAction) {
    let button = app.world_mut().spawn((Pressed, action)).id();
    app.update();
    app.world_mut().despawn(button);
}

/// Selecting source forms leaves all independent permissions and the destination untouched.
#[test]
fn selecting_configured_sources_never_changes_permissions_or_to() {
    let mut app = sources_app();
    for selection in [
        SourceSelection::Remote("alice".into()),
        SourceSelection::Remote("bob".into()),
        SourceSelection::Local,
    ] {
        source_press(&mut app, UiAction::SelectSource(selection.clone()));
        let model = app.world().resource::<Model>();
        let ModalDraft::Sources(sources) = &model.modal.as_ref().unwrap().draft else {
            panic!("source form");
        };
        assert!(sources.selection == selection);
        let network = app.world().resource::<crate::network_app::NetworkState>();
        assert!(network.local_enabled);
        assert!(network.controllers.iter().all(|source| source.enabled));
        assert!(network.actions.is_empty());
        assert!(app.world().resource::<Actions>().0.is_empty());
        assert_eq!(
            app.world().resource::<Runtime>().destination,
            Destination::Remote
        );
    }
}

/// Plus and provider selection are reversible drafts; Cancel restores the configured source.
#[test]
fn plus_type_switch_and_cancel_create_no_controller() {
    let mut app = sources_app();
    source_press(
        &mut app,
        UiAction::SelectSource(SourceSelection::Remote("bob".into())),
    );
    source_press(
        &mut app,
        UiAction::SelectSource(SourceSelection::ChooseType),
    );
    source_press(&mut app, UiAction::SelectSource(SourceSelection::NewRemote));
    app.world_mut()
        .resource_mut::<Model>()
        .editor(Field::Configuration(0))
        .unwrap()
        .text
        .push_str("Uncommitted controller");
    source_press(
        &mut app,
        UiAction::SelectSource(SourceSelection::ChooseType),
    );
    source_press(&mut app, UiAction::SelectSource(SourceSelection::NewRemote));
    assert!(
        app.world()
            .resource::<Model>()
            .modal
            .as_ref()
            .unwrap()
            .editors[0]
            .text
            .is_empty()
    );
    source_press(&mut app, UiAction::CancelSource);
    let model = app.world().resource::<Model>();
    let ModalDraft::Sources(sources) = &model.modal.as_ref().unwrap().draft else {
        panic!("source form");
    };
    assert!(sources.selection == SourceSelection::Remote("bob".into()));
    let network = app.world().resource::<crate::network_app::NetworkState>();
    assert_eq!(network.controllers.len(), 2);
    assert!(network.actions.is_empty());
}

/// Hidden credentials are discarded on Back and source selection without saving or leaking focus.
#[test]
fn leaving_invitation_credentials_discards_secret_and_hidden_input_focus() {
    let mut app = sources_app();
    source_press(
        &mut app,
        UiAction::SelectSource(SourceSelection::Remote("alice".into())),
    );
    source_press(
        &mut app,
        UiAction::SourceDetails(SourceDetails::Credentials),
    );
    app.world_mut()
        .resource_mut::<Model>()
        .editor(Field::Configuration(1))
        .unwrap()
        .text
        .push_str("private-admin-key");
    source_press(
        &mut app,
        UiAction::SourceDetails(SourceDetails::Credentials),
    );
    assert!(
        app.world_mut()
            .resource_mut::<Model>()
            .editor(Field::Configuration(1))
            .is_none()
    );
    source_press(
        &mut app,
        UiAction::SourceDetails(SourceDetails::Credentials),
    );
    assert!(
        app.world()
            .resource::<Model>()
            .modal
            .as_ref()
            .unwrap()
            .editors[1]
            .text
            .is_empty()
    );
    app.world_mut()
        .resource_mut::<Model>()
        .editor(Field::Configuration(1))
        .unwrap()
        .text
        .push_str("second-key");
    source_press(&mut app, UiAction::SelectSource(SourceSelection::Local));
    let model = app.world().resource::<Model>();
    assert!(model.focus.is_none());
    assert!(model.modal.as_ref().unwrap().editors[1].text.is_empty());
    assert!(
        app.world()
            .resource::<crate::network_app::NetworkState>()
            .actions
            .is_empty()
    );
}

/// An explicit Add selects its returned identity even if an older source has the same name.
#[test]
fn add_selects_only_new_controller_after_runtime_result() {
    use crate::network_app::{ControllerSource, NetworkAction, NetworkState};
    let mut app = sources_app();
    source_press(
        &mut app,
        UiAction::SelectSource(SourceSelection::ChooseType),
    );
    source_press(&mut app, UiAction::SelectSource(SourceSelection::NewRemote));
    app.world_mut()
        .resource_mut::<Model>()
        .editor(Field::Configuration(0))
        .unwrap()
        .text
        .push_str("alice");
    source_press(&mut app, UiAction::AddController);
    {
        let network = app.world().resource::<NetworkState>();
        assert_eq!(network.controllers.len(), 2);
        assert!(
            matches!(network.actions.front(), Some(NetworkAction::AddController {name}) if name == "alice")
        );
    }
    app.world_mut()
        .resource_mut::<NetworkState>()
        .controllers
        .push(ControllerSource {
            id: "new-alice".into(),
            name: "alice".into(),
            enabled: true,
            pairing_secret: None,
            invitation_url: None,
            invitation_status: "Ready".into(),
        });
    app.world_mut()
        .resource_scope(|world, mut model: Mut<Model>| {
            model.reconcile_sources(world.resource::<NetworkState>());
        });
    let model = app.world().resource::<Model>();
    let ModalDraft::Sources(sources) = &model.modal.as_ref().unwrap().draft else {
        panic!("source form");
    };
    assert!(sources.selection == SourceSelection::Remote("new-alice".into()));
    assert!(sources.pending.is_none());
    assert_eq!(
        app.world().resource::<Runtime>().destination,
        Destination::Remote
    );
}

/// Controller-only launches can configure an inbound listener for remote-to-remote routing.
#[test]
fn controller_launch_listener_provider_apply_preserves_remote_destination() {
    let mut app = sources_app();
    app.world_mut().resource_mut::<Model>().open(
        Provider::Host(HostProvider::TailscaleListener),
        &provider_context(),
    );
    source_press(&mut app, UiAction::Apply);
    assert!(matches!(
        app.world()
            .resource::<crate::network_app::NetworkState>()
            .actions
            .front(),
        Some(crate::network_app::NetworkAction::EnableHost)
    ));
    assert!(app.world().resource::<Actions>().0.is_empty());
    assert_eq!(
        app.world().resource::<Runtime>().destination,
        Destination::Remote
    );
}

/// Status, catalog filtering and snapshot updates must preserve the live selection and caret entity.
#[test]
fn actual_view_refresh_retains_engine_selection_focus_and_composition() {
    use bevy::text::TextEdit;
    let (mut app, window) = input_app();
    let input = app.world().resource::<Model>().query.entity.unwrap();
    app.world_mut()
        .get_mut::<EditableText>(input)
        .unwrap()
        .queue_edit(TextEdit::Insert("reaction".into()));
    app.update();
    app.world_mut()
        .get_mut::<EditableText>(input)
        .unwrap()
        .queue_edit(TextEdit::SelectAll);
    app.update();
    let selection = app
        .world()
        .get::<EditableText>(input)
        .unwrap()
        .editor
        .raw_selection()
        .text_range();
    assert_eq!(selection, 0..8);
    let root = app.world().resource::<Model>().root.unwrap();
    {
        let mut runtime = app.world_mut().resource_mut::<Runtime>();
        runtime.status = "Playback changed during editing".into();
        runtime.snapshot.sequence += 1;
    }
    app.world_mut()
        .resource_mut::<crate::network_app::NetworkState>()
        .status = "Remote status changed".into();
    app.update();
    assert_eq!(app.world().resource::<Model>().root, Some(root));
    assert_eq!(app.world().resource::<Model>().query.entity, Some(input));
    assert_eq!(app.world().resource::<InputFocus>().get(), Some(input));
    assert_eq!(
        app.world()
            .get::<EditableText>(input)
            .unwrap()
            .editor
            .raw_selection()
            .text_range(),
        selection
    );
    assert!(app.world().get::<TextCursorStyle>(input).is_some());
    assert_eq!(
        app.world().get::<TextLayout>(input).unwrap().linebreak,
        LineBreak::NoWrap
    );
    key(
        &mut app,
        window,
        Key::Character("日本".into()),
        KeyCode::KeyN,
        Some("日本"),
    );
    app.update();
    assert_eq!(app.world().resource::<Model>().query.text, "日本");
    app.world_mut().write_message(Ime::Preedit {
        window,
        value: "に".into(),
        cursor: Some((3, 3)),
    });
    app.update();
    app.world_mut().resource_mut::<Runtime>().status = "Still editing".into();
    app.update();
    assert!(
        app.world()
            .get::<EditableText>(input)
            .unwrap()
            .is_composing()
    );
    assert_eq!(app.world().resource::<Model>().query.text, "日本");
}

/// Opening a modal enters its engine tab group, which traps focus and masks protected values.
#[test]
fn rendered_modal_initial_focus_and_tab_stay_inside_visible_fields() {
    let (mut app, window) = input_app();
    app.world_mut()
        .resource_mut::<Model>()
        .open(Provider::Host(HostProvider::Tailscale), &provider_context());
    app.update();
    let model = app.world().resource::<Model>();
    let search = model.query.entity.unwrap();
    let modal = model.modal.as_ref().unwrap();
    let endpoint = modal.editors[0].entity.unwrap();
    let secret = modal.editors[1].entity.unwrap();
    assert_eq!(app.world().resource::<InputFocus>().get(), Some(endpoint));
    assert!(app.world().get::<InteractionDisabled>(search).is_some());
    assert!(app.world().get::<EditableText>(secret).is_none());
    assert!(app.world().get::<Text>(secret).unwrap().0.is_empty());
    key(&mut app, window, Key::Tab, KeyCode::Tab, None);
    app.update();
    assert_eq!(app.world().resource::<InputFocus>().get(), Some(secret));
    key(
        &mut app,
        window,
        Key::Character("private-token".into()),
        KeyCode::KeyP,
        Some("private-token"),
    );
    app.update();
    assert_eq!(app.world().get::<Text>(secret).unwrap().0, "•".repeat(13));
    key(&mut app, window, Key::Tab, KeyCode::Tab, None);
    app.update();
    assert_eq!(app.world().resource::<InputFocus>().get(), Some(endpoint));
    key(&mut app, window, Key::Shift, KeyCode::ShiftRight, None);
    key(&mut app, window, Key::Tab, KeyCode::Tab, None);
    app.world_mut().write_message(KeyboardInput {
        window,
        key_code: KeyCode::ShiftRight,
        logical_key: Key::Shift,
        state: ButtonState::Released,
        text: None,
        repeat: false,
    });
    app.update();
    assert_eq!(app.world().resource::<InputFocus>().get(), Some(secret));
    assert!(app.world().resource::<Actions>().0.is_empty());
}

/// Retiring a hidden source field clears its entity handle so revisiting constructs a usable editor.
#[test]
fn rendered_source_listener_can_close_and_reopen_without_dead_input_handles() {
    let (mut app, window) = input_app();
    let mut fixture = sources_app();
    let network = fixture
        .world_mut()
        .remove_resource::<crate::network_app::NetworkState>()
        .unwrap();
    app.insert_resource(network);
    {
        let mut model = app.world_mut().resource_mut::<Model>();
        model.open_sources("100.101.102.103:48732");
        let modal = model.modal.as_mut().unwrap();
        modal.select_source(SourceSelection::Remote("alice".into()));
        modal.show_source_details(SourceDetails::Listener);
    }
    app.update();
    let original = app
        .world()
        .resource::<Model>()
        .modal
        .as_ref()
        .unwrap()
        .editors[2]
        .entity
        .unwrap();
    {
        let mut model = app.world_mut().resource_mut::<Model>();
        model
            .modal
            .as_mut()
            .unwrap()
            .show_source_details(SourceDetails::Listener);
        model.revision += 1;
    }
    app.update();
    assert!(app.world().get_entity(original).is_err());
    assert!(
        app.world()
            .resource::<Model>()
            .modal
            .as_ref()
            .unwrap()
            .editors[2]
            .entity
            .is_none()
    );
    {
        let mut model = app.world_mut().resource_mut::<Model>();
        model
            .modal
            .as_mut()
            .unwrap()
            .show_source_details(SourceDetails::Listener);
        model.revision += 1;
    }
    app.update();
    let reopened = app
        .world()
        .resource::<Model>()
        .modal
        .as_ref()
        .unwrap()
        .editors[2]
        .entity
        .unwrap();
    assert_ne!(original, reopened);
    assert_eq!(app.world().resource::<InputFocus>().get(), Some(reopened));
    key(&mut app, window, Key::Backspace, KeyCode::Backspace, None);
    app.update();
    assert_eq!(
        app.world()
            .resource::<Model>()
            .modal
            .as_ref()
            .unwrap()
            .editors[2]
            .text,
        "100.101.102.103:4873"
    );
    {
        let mut text = app.world_mut().get_mut::<EditableText>(reopened).unwrap();
        text.queue_edit(bevy::text::TextEdit::SelectAll);
        text.queue_edit(bevy::text::TextEdit::Backspace);
    }
    app.update();
    app.world_mut()
        .resource_mut::<crate::network_app::NetworkState>()
        .status = "Discovery changed while editing".into();
    app.update();
    let model = app.world().resource::<Model>();
    let editor = &model.modal.as_ref().unwrap().editors[2];
    assert!(editor.text.is_empty());
    assert_eq!(editor.entity, Some(reopened));
    assert_eq!(app.world().resource::<InputFocus>().get(), Some(reopened));
}

/// A same-frame final edit and Apply must commit engine text while an unrelated status region refreshes.
#[test]
fn queued_final_engine_edit_and_apply_survive_transient_view_refresh() {
    use bevy::text::TextEdit;
    let (mut app, _) = input_app();
    app.world_mut().resource_mut::<Runtime>().destination = Destination::Local;
    app.world_mut().resource_mut::<Runtime>().mode = Mode::Local;
    app.world_mut().resource_mut::<Model>().open(
        Provider::Catalog(CatalogProvider::GitHttps),
        &provider_context(),
    );
    app.update();
    let input = app
        .world()
        .resource::<Model>()
        .modal
        .as_ref()
        .unwrap()
        .editors[0]
        .entity
        .unwrap();
    let button = app
        .world_mut()
        .query::<(Entity, &UiAction)>()
        .iter(app.world())
        .find_map(|(entity, action)| matches!(action, UiAction::Apply).then_some(entity))
        .unwrap();
    {
        let mut text = app.world_mut().get_mut::<EditableText>(input).unwrap();
        text.queue_edit(TextEdit::SelectAll);
        text.queue_edit(TextEdit::Insert("https://example.com/final.git".into()));
    }
    app.world_mut().trigger(Activate { entity: button });
    app.world_mut().resource_mut::<Runtime>().status = "Refresh before text commit".into();
    app.update();
    assert!(app.world().get_entity(button).is_ok());
    assert!(app.world().resource::<Model>().modal.is_none());
    assert!(app.world().resource::<Actions>().0.iter().any(|action| matches!(action, Action::LoadPack(source) if source.starts_with("https://example.com/final.git#"))));
    app.update();
    assert!(app.world().get_entity(input).is_err());
    assert!(app.world().get_entity(button).is_err());
}

/// Fast modifier chords must reach Bevy editing with their event-time flags, even in one update.
#[test]
fn ordered_command_down_a_command_up_selects_instead_of_typing_a() {
    use bevy::text::TextEdit;
    let (mut app, window) = input_app();
    let input = app.world().resource::<Model>().query.entity.unwrap();
    app.world_mut()
        .get_mut::<EditableText>(input)
        .unwrap()
        .queue_edit(TextEdit::Insert("gif".into()));
    app.update();
    let (modifier, code) = if cfg!(target_os = "macos") {
        (Key::Super, KeyCode::SuperLeft)
    } else {
        (Key::Control, KeyCode::ControlLeft)
    };
    key(&mut app, window, modifier.clone(), code, None);
    key(
        &mut app,
        window,
        Key::Character("a".into()),
        KeyCode::KeyA,
        Some("a"),
    );
    app.world_mut().write_message(KeyboardInput {
        window,
        key_code: code,
        logical_key: modifier,
        state: ButtonState::Released,
        text: None,
        repeat: false,
    });
    app.update();
    assert_eq!(app.world().resource::<Model>().query.text, "gif");
    assert_eq!(
        app.world()
            .get::<EditableText>(input)
            .unwrap()
            .editor
            .raw_selection()
            .text_range(),
        0..3
    );
    key(
        &mut app,
        window,
        Key::Character("日本".into()),
        KeyCode::KeyN,
        Some("日本"),
    );
    app.update();
    assert_eq!(app.world().resource::<Model>().query.text, "日本");
    assert!(app.world().resource::<Actions>().0.is_empty());
}

/// Measure production UI at the ordinary one-physical-pixel-per-logical-pixel desktop scale.
fn enable_headless_layout(app: &mut App, width: u32, height: u32) {
    enable_headless_layout_at_scale(app, width, height, 1.);
}

/// Run real Bevy text measurement and layout at an explicit desktop/Retina render scale.
fn enable_headless_layout_at_scale(app: &mut App, width: u32, height: u32, scale: f32) {
    use bevy::app::{HierarchyPropagatePlugin, PropagateSet};
    use bevy::ui::{ComputedUiRenderTargetInfo, ComputedUiTargetCamera, UiSystems};
    app.add_plugins((
        HierarchyPropagatePlugin::<ComputedUiTargetCamera>::new(PostUpdate),
        HierarchyPropagatePlugin::<ComputedUiRenderTargetInfo>::new(PostUpdate),
    ))
    .init_resource::<bevy::ui::ui_surface::UiSurface>()
    .init_resource::<bevy::text::RemSize>()
    .init_resource::<bevy::text::TextPipeline>()
    .init_resource::<bevy::text::ScaleCx>()
    .init_resource::<bevy::text::FontAtlasSet>()
    .configure_sets(
        PostUpdate,
        (
            UiSystems::Prepare,
            UiSystems::Propagate,
            UiSystems::Content,
            UiSystems::Layout,
            EditableTextSystems,
            UiSystems::PostLayout,
        )
            .chain(),
    )
    .configure_sets(
        PostUpdate,
        PropagateSet::<ComputedUiTargetCamera>::default().in_set(UiSystems::Propagate),
    )
    .configure_sets(
        PostUpdate,
        PropagateSet::<ComputedUiRenderTargetInfo>::default().in_set(UiSystems::Propagate),
    )
    .add_systems(
        PostUpdate,
        bevy::ui::update::propagate_ui_target_cameras.in_set(UiSystems::Prepare),
    )
    .add_systems(
        PostUpdate,
        (
            bevy::text::load_font_assets_into_font_collection,
            bevy::text::detect_text_needs_rerender,
            bevy::ui::sync_font_size_to_em_size,
            bevy::ui::widget::measure_text_system,
            bevy::ui::widget::update_editable_text_content_size,
            bevy::ui::widget::update_editable_text_styles,
        )
            .chain()
            .in_set(UiSystems::Content),
    )
    .add_systems(
        PostUpdate,
        bevy::ui::ui_layout_system.in_set(UiSystems::Layout),
    )
    .add_systems(
        PostUpdate,
        bevy::ui::widget::sync_editable_text_viewports
            .after(UiSystems::Layout)
            .before(EditableTextSystems),
    )
    .add_systems(
        PostUpdate,
        (
            bevy::ui::widget::text_system,
            bevy::ui::widget::update_editable_text_layout,
        )
            .chain()
            .in_set(UiSystems::PostLayout)
            .before(bevy::input_focus::InputFocusSystems::FocusChangeEvents),
    )
    .add_systems(Update, scroll.before(view::render));
    let physical_size = UVec2::new(
        (width as f32 * scale) as u32,
        (height as f32 * scale) as u32,
    );
    let camera = app.world().resource::<PanelCamera>().0;
    app.world_mut().entity_mut(camera).insert(Camera {
        computed: bevy::camera::ComputedCameraValues {
            target_info: Some(bevy::camera::RenderTargetInfo {
                physical_size,
                scale_factor: scale,
            }),
            ..default()
        },
        viewport: Some(bevy::camera::Viewport {
            physical_size,
            ..default()
        }),
        ..default()
    });
    let mut windows = app
        .world_mut()
        .query_filtered::<&mut Window, With<PrimaryWindow>>();
    for mut window in windows.iter_mut(app.world_mut()) {
        window.resolution = bevy::window::WindowResolution::new(physical_size.x, physical_size.y)
            .with_scale_factor_override(scale);
    }
}

/// Full layout must bound the catalog and keep modal geometry inside desktop and compact windows.
#[test]
fn actual_bevy_layout_bounds_collection_and_modal_across_filter_refreshes() {
    for (width, height) in [(1160, 820), (720, 520)] {
        let (mut app, window) = input_app();
        enable_headless_layout(&mut app, width, height);
        {
            let mut runtime = app.world_mut().resource_mut::<Runtime>();
            runtime.destination = Destination::Local;
            runtime.mode = Mode::Local;
            runtime.catalog = std::sync::Arc::new(
                crate::catalog::load_local(
                    &crate::app::resource_root().join("sample-packs/reactions"),
                )
                .unwrap(),
            );
            runtime.status = "Layout fixture".into();
        }
        app.update();
        app.update();
        let root = app.world().resource::<Model>().root.unwrap();
        let size = app.world().get::<ComputedNode>(root).unwrap().size();
        assert!(
            size.y <= height as f32 + 1.,
            "root {size:?} exceeds viewport {width}x{height}"
        );
        let mut bodies = app
            .world_mut()
            .query_filtered::<&ComputedNode, With<CatalogBody>>();
        let body = bodies.single(app.world()).unwrap();
        assert!(body.size().y > 0. && body.size().y < height as f32);
        assert!(body.content_size().y > body.size().y);
        key(
            &mut app,
            window,
            Key::Character("Holly".into()),
            KeyCode::KeyH,
            Some("Holly"),
        );
        app.update();
        app.update();
        let root = app.world().resource::<Model>().root.unwrap();
        assert!(app.world().get::<ComputedNode>(root).unwrap().size().y <= height as f32 + 1.);
        app.world_mut().resource_mut::<Model>().open(
            Provider::Catalog(CatalogProvider::GitHttps),
            &provider_context(),
        );
        app.update();
        let mut groups = app
            .world_mut()
            .query::<(&TabGroup, &ComputedNode, &UiGlobalTransform)>();
        let (_, node, transform) = groups
            .iter(app.world())
            .find(|(group, _, _)| group.modal)
            .unwrap();
        let center = transform.translation;
        let half = node.size() * 0.5;
        assert!(node.size().y > 100. && node.size().x > 100.);
        assert!(
            center.y - half.y >= -1. && center.y + half.y <= height as f32 + 1.,
            "modal center {center:?} size {:?} outside {width}x{height}",
            node.size()
        );
        let mut modal_bodies = app
            .world_mut()
            .query_filtered::<&ComputedNode, With<ModalBody>>();
        assert!(modal_bodies.single(app.world()).unwrap().size().y > 0.);
    }
}

/// Bevy 0.20 stores native windows in thread-local storage; flag-only Command must still reach editing.
#[test]
fn native_flag_only_modifiers_use_thread_local_windows_and_reset_on_focus_loss() {
    let (mut app, window) = input_app();
    let native = winit::window::WindowId::from(918_273);
    bevy::winit::WINIT_WINDOWS.with(|windows| {
        windows.borrow_mut().entity_to_winit.insert(window, native);
    });
    app.add_message::<bevy::winit::RawWinitWindowEvent>();
    app.world_mut()
        .write_message(bevy::winit::RawWinitWindowEvent {
            window_id: native,
            event: winit::event::WindowEvent::ModifiersChanged(
                winit::keyboard::ModifiersState::SUPER.into(),
            ),
        });
    app.update();
    assert!(
        app.world()
            .resource::<ButtonInput<Key>>()
            .pressed(Key::Super)
    );
    app.world_mut().get_mut::<Window>(window).unwrap().focused = false;
    app.world_mut()
        .write_message(bevy::winit::RawWinitWindowEvent {
            window_id: native,
            event: winit::event::WindowEvent::Focused(false),
        });
    key(
        &mut app,
        window,
        Key::Character("a".into()),
        KeyCode::KeyA,
        Some("a"),
    );
    app.update();
    app.world_mut().get_mut::<Window>(window).unwrap().focused = true;
    app.update();
    assert!(app.world().resource::<Model>().query.text.is_empty());
    assert!(
        !app.world()
            .resource::<ButtonInput<Key>>()
            .pressed(Key::Super)
    );
    bevy::winit::WINIT_WINDOWS.with(|windows| {
        windows.borrow_mut().entity_to_winit.remove(&window);
    });
}

/// Text committed after Update must request another frame in the native low-power event loop.
#[test]
fn committed_engine_edit_requests_a_followup_render_without_pointer_motion() {
    let (mut app, window) = input_app();
    app.update();
    app.world_mut()
        .resource_mut::<Messages<bevy::window::RequestRedraw>>()
        .clear();
    key(
        &mut app,
        window,
        Key::Character("filter".into()),
        KeyCode::KeyF,
        Some("filter"),
    );
    app.update();
    assert_eq!(app.world().resource::<Model>().query.text, "filter");
    assert!(
        !app.world()
            .resource::<Messages<bevy::window::RequestRedraw>>()
            .is_empty()
    );
}

/// Pointer presses must focus the padded input and drive Bevy selection/caret placement.
#[test]
fn engine_pointer_press_selects_text_and_places_caret_inside_padded_field() {
    use bevy::picking::{
        backend::HitData,
        events::{Pointer, PointerPress},
        pointer::{Location, PointerButton, PointerId},
    };
    use bevy::text::TextEdit;
    let (mut app, window) = input_app();
    app.add_plugins(bevy::input_focus::pointer_focus::PointerFocusPlugin);
    enable_headless_layout(&mut app, 1160, 820);
    let input = app.world().resource::<Model>().query.entity.unwrap();
    app.world_mut()
        .get_mut::<EditableText>(input)
        .unwrap()
        .queue_edit(TextEdit::Insert("hello".into()));
    app.update();
    app.update();
    let node = app.world().get::<ComputedNode>(input).unwrap();
    let transform = app.world().get::<UiGlobalTransform>(input).unwrap();
    let position = transform
        .affine()
        .transform_point2(node.content_box().min + Vec2::splat(1.));
    let camera = app.world().resource::<PanelCamera>().0;
    let event = |count| PointerPress {
        entity: input,
        pointer: Pointer::new(
            PointerId::Mouse,
            Location {
                target: bevy::camera::NormalizedRenderTarget::Window(
                    WindowRef::Entity(window).normalize(Some(window)).unwrap(),
                ),
                position,
            },
        ),
        button: PointerButton::Primary,
        hit: HitData::new(camera, 0., None, None),
        count,
    };
    app.world_mut().resource_mut::<InputFocus>().clear();
    app.update();
    app.world_mut().trigger(event(3));
    app.update();
    assert_eq!(app.world().resource::<InputFocus>().get(), Some(input));
    assert_eq!(
        app.world()
            .get::<EditableText>(input)
            .unwrap()
            .editor
            .raw_selection()
            .text_range(),
        0..5
    );
    app.world_mut().trigger(event(1));
    app.update();
    assert_eq!(
        app.world()
            .get::<EditableText>(input)
            .unwrap()
            .editor
            .raw_selection()
            .text_range(),
        0..0
    );
    key(
        &mut app,
        window,
        Key::Character("X".into()),
        KeyCode::KeyX,
        Some("X"),
    );
    app.update();
    assert_eq!(app.world().resource::<Model>().query.text, "Xhello");
}

/// Read-only selectable labels consume text keys and leave playback shortcuts for unfocused input.
#[test]
fn readonly_engine_text_selection_suppresses_playback_and_remains_immutable() {
    use bevy::text::{TextEdit, TextReadWriteMode};
    let (mut app, window) = input_app();
    app.world_mut().resource_mut::<Runtime>().destination = Destination::Local;
    let label = app
        .world_mut()
        .spawn((
            TextInput,
            EditableText::new("Status text"),
            TextCursorStyle::default(),
            TextReadWriteMode::ReadOnly,
            TabIndex(-1),
        ))
        .id();
    app.world_mut()
        .resource_mut::<InputFocus>()
        .set(label, FocusCause::Pressed);
    app.world_mut()
        .get_mut::<EditableText>(label)
        .unwrap()
        .queue_edit(TextEdit::SelectAll);
    app.update();
    assert_eq!(
        app.world()
            .get::<EditableText>(label)
            .unwrap()
            .editor
            .raw_selection()
            .text_range(),
        0..11
    );
    key(&mut app, window, Key::Space, KeyCode::Space, Some(" "));
    key(
        &mut app,
        window,
        Key::Character("x".into()),
        KeyCode::KeyX,
        Some("x"),
    );
    app.update();
    assert_eq!(
        app.world()
            .get::<EditableText>(label)
            .unwrap()
            .value()
            .to_string(),
        "Status text"
    );
    assert!(app.world().resource::<Actions>().0.is_empty());
    key(&mut app, window, Key::Escape, KeyCode::Escape, None);
    app.update();
    assert!(app.world().resource::<Actions>().0.is_empty());
    assert!(app.world().resource::<InputFocus>().get().is_none());
}

/// Unchanged selectable modal labels retain their engine selection and retire when their context closes.
#[test]
fn production_readonly_modal_selection_survives_refresh_and_retires_on_close() {
    use bevy::text::{TextEdit, TextReadWriteMode};
    let (mut app, _) = input_app();
    app.world_mut().resource_mut::<Model>().open(
        Provider::Catalog(CatalogProvider::GitHttps),
        &provider_context(),
    );
    app.update();
    let label = app
        .world_mut()
        .query::<(Entity, &EditableText, &TextReadWriteMode)>()
        .iter(app.world())
        .find_map(|(entity, text, mode)| {
            (*mode == TextReadWriteMode::ReadOnly && text.value() == "Git repository")
                .then_some(entity)
        })
        .unwrap();
    app.world_mut()
        .resource_mut::<InputFocus>()
        .set(label, FocusCause::Pressed);
    app.world_mut()
        .get_mut::<EditableText>(label)
        .unwrap()
        .queue_edit(TextEdit::SelectAll);
    app.update();
    app.world_mut()
        .resource_mut::<crate::network_app::NetworkState>()
        .status = "Network update while selecting label".into();
    app.update();
    assert_eq!(app.world().resource::<InputFocus>().get(), Some(label));
    assert_eq!(
        app.world()
            .get::<EditableText>(label)
            .unwrap()
            .editor
            .raw_selection()
            .text_range(),
        0..14
    );
    app.world_mut().resource_mut::<Model>().close();
    app.update();
    assert!(app.world().get_entity(label).is_err());
    assert_ne!(app.world().resource::<InputFocus>().get(), Some(label));
}

/// Selecting a real rendered title must stop the pointer before the enclosing Play button activates.
#[test]
fn production_card_title_selection_does_not_activate_card_play() {
    use bevy::picking::{
        backend::HitData,
        events::{Pointer, PointerPress},
        pointer::{Location, PointerButton, PointerId},
    };
    use bevy::text::TextReadWriteMode;
    let (mut app, window) = input_app();
    app.add_plugins(bevy::ui_widgets::ButtonPlugin);
    enable_headless_layout(&mut app, 1160, 820);
    let (identity, title) = {
        let mut runtime = app.world_mut().resource_mut::<Runtime>();
        runtime.destination = Destination::Local;
        runtime.catalog = std::sync::Arc::new(
            crate::catalog::load_local(&crate::app::resource_root().join("sample-packs/reactions"))
                .unwrap(),
        );
        runtime.status = "Card input fixture".into();
        (
            runtime.catalog.clips[0].identity.clone(),
            runtime.catalog.clips[0].metadata.title.clone(),
        )
    };
    app.update();
    app.update();
    app.update();
    let label = app
        .world_mut()
        .query::<(Entity, &EditableText, &TextReadWriteMode)>()
        .iter(app.world())
        .find_map(|(entity, text, mode)| {
            (*mode == TextReadWriteMode::ReadOnly && text.value().to_string() == title)
                .then_some(entity)
        })
        .unwrap();
    let card = app
        .world_mut()
        .query::<(Entity, &UiAction)>()
        .iter(app.world())
        .find_map(|(entity, action)| {
            matches!(action, UiAction::Play(id) if id == &identity).then_some(entity)
        })
        .unwrap();
    let camera = app.world().resource::<PanelCamera>().0;
    let event = |entity| PointerPress {
        entity,
        pointer: Pointer::new(
            PointerId::Mouse,
            Location {
                target: bevy::camera::NormalizedRenderTarget::Window(
                    WindowRef::Entity(window).normalize(Some(window)).unwrap(),
                ),
                position: Vec2::ZERO,
            },
        ),
        button: PointerButton::Primary,
        hit: HitData::new(camera, 0., None, None),
        count: 1,
    };
    assert!(app.world().get::<EditableText>(label).is_some());
    assert!(
        app.world()
            .get::<bevy::text::TextReadWriteMode>(label)
            .is_some()
    );
    assert!(app.world().get::<ComputedNode>(label).is_some());
    assert!(
        app.world()
            .get::<bevy::ui::ComputedUiRenderTargetInfo>(label)
            .is_some()
    );
    assert!(app.world().get::<UiGlobalTransform>(label).is_some());
    assert!(
        app.world().resource::<Actions>().0.is_empty(),
        "Fixture already had actions"
    );
    assert!(
        app.world().resource::<input::PendingActions>().0.is_empty(),
        "Fixture already had queued actions"
    );
    app.world_mut().trigger(event(label));
    app.update();
    assert!(app.world().resource::<Actions>().0.is_empty());
    // The explicit preview/Play target still activates and survives the pending refresh.
    app.world_mut().trigger(event(card));
    app.update();
    assert!(
        app.world()
            .resource::<Actions>()
            .0
            .iter()
            .any(|action| matches!(action, Action::Play(id) if id == &identity))
    );
}

/// Another window's modifier transitions must not alter this panel's ordered editor state.
#[test]
fn secondary_window_modifier_does_not_change_primary_text_insertion() {
    let (mut app, primary) = input_app();
    let secondary = app.world_mut().spawn(Window::default()).id();
    key(
        &mut app,
        primary,
        Key::Character("gif".into()),
        KeyCode::KeyG,
        Some("gif"),
    );
    app.update();
    key(
        &mut app,
        secondary,
        Key::Control,
        KeyCode::ControlLeft,
        None,
    );
    key(
        &mut app,
        primary,
        Key::Character("a".into()),
        KeyCode::KeyA,
        Some("a"),
    );
    app.update();
    assert_eq!(app.world().resource::<Model>().query.text, "gifa");
    key(
        &mut app,
        primary,
        Key::Character("b".into()),
        KeyCode::KeyB,
        Some("b"),
    );
    app.update();
    assert_eq!(app.world().resource::<Model>().query.text, "gifab");
    assert!(app.world().resource::<Actions>().0.is_empty());
}

/// Both the final appended secret glyph and the final deletion must schedule a settling render.
#[test]
fn secret_append_and_backspace_update_revision_and_request_redraw() {
    let (mut app, window) = input_app();
    app.world_mut()
        .resource_mut::<Model>()
        .open(Provider::Host(HostProvider::Tailscale), &provider_context());
    app.update();
    let secret = app
        .world()
        .resource::<Model>()
        .modal
        .as_ref()
        .unwrap()
        .editors[1]
        .entity
        .unwrap();
    app.world_mut()
        .resource_mut::<InputFocus>()
        .set(secret, FocusCause::Pressed);
    app.update();
    app.update();
    app.world_mut()
        .resource_mut::<Messages<bevy::window::RequestRedraw>>()
        .clear();
    let initial_revision = app.world().resource::<Model>().revision;
    key(
        &mut app,
        window,
        Key::Character("x".into()),
        KeyCode::KeyX,
        Some("x"),
    );
    app.update();
    assert!(app.world().resource::<Model>().revision > initial_revision);
    assert_eq!(app.world().get::<Text>(secret).unwrap().0, "•");
    assert!(
        !app.world()
            .resource::<Messages<bevy::window::RequestRedraw>>()
            .is_empty()
    );
    app.update();
    app.world_mut()
        .resource_mut::<Messages<bevy::window::RequestRedraw>>()
        .clear();
    let inserted_revision = app.world().resource::<Model>().revision;
    key(&mut app, window, Key::Backspace, KeyCode::Backspace, None);
    app.update();
    assert!(app.world().resource::<Model>().revision > inserted_revision);
    assert!(app.world().get::<Text>(secret).unwrap().0.is_empty());
    assert!(
        !app.world()
            .resource::<Messages<bevy::window::RequestRedraw>>()
            .is_empty()
    );
    assert!(app.world().get::<EditableText>(secret).is_none());
}

/// Capture drawn glyph geometry by displayed value, independently of rebuilt entity identities.
#[derive(Clone, Debug, PartialEq)]
struct MeasuredTextGeometry {
    /// Physical center on the primary render target.
    center: Vec2,
    /// Actual allocated text node extent after Taffy and glyph measurement.
    size: Vec2,
    /// The render extraction target must remain available on every frame.
    camera: Option<Entity>,
    /// Hierarchical visibility participates in actual glyph extraction.
    visible: bool,
    /// Physical text bounds after shaping, independent of the enclosing node's allocation.
    layout_size: Vec2,
    /// Every local glyph position catches shifting text even when its node stays stationary.
    glyph_positions: Vec<Vec2>,
}

impl MeasuredTextGeometry {
    /// Sort duplicate labels by position so equivalent trees compare deterministically.
    fn compare(left: &Self, right: &Self) -> std::cmp::Ordering {
        left.center
            .y
            .total_cmp(&right.center.y)
            .then(left.center.x.total_cmp(&right.center.x))
    }

    /// Read all nonempty displayed text after the production measurement/layout schedules finish.
    fn capture(world: &mut World) -> BTreeMap<String, Vec<Self>> {
        let mut result: BTreeMap<String, Vec<Self>> = BTreeMap::new();
        for (text, node, transform, layout, camera, visibility) in world
            .query::<(
                &Text,
                &ComputedNode,
                &UiGlobalTransform,
                &bevy::text::TextLayoutInfo,
                Option<&bevy::ui::ComputedUiTargetCamera>,
                &InheritedVisibility,
            )>()
            .iter(world)
        {
            if !text.0.is_empty() {
                result.entry(text.0.clone()).or_default().push(Self {
                    center: transform.translation,
                    size: node.size(),
                    camera: camera.and_then(bevy::ui::ComputedUiTargetCamera::get),
                    visible: visibility.get(),
                    layout_size: layout.size,
                    glyph_positions: layout.glyphs.iter().map(|glyph| glyph.position).collect(),
                });
            }
        }
        for copies in result.values_mut() {
            copies.sort_by(Self::compare);
        }
        result
    }

    /// Every retained displayed value must keep its measured glyph geometry on the idle frame.
    fn assert_stable(
        before: &BTreeMap<String, Vec<Self>>,
        after: &BTreeMap<String, Vec<Self>>,
        transition: &str,
    ) {
        assert_eq!(
            before.keys().collect::<Vec<_>>(),
            after.keys().collect::<Vec<_>>(),
            "{transition}: unchanged displayed text disappeared or appeared late"
        );
        for (text, geometry) in before {
            let next = after
                .get(text)
                .expect("identical key sets were checked above");
            assert_eq!(
                geometry, next,
                "{transition}: unchanged text {text:?} moved or resized between rebuild and idle frames"
            );
        }
    }
}

/// Capture region anchors and unchanged chrome children to ensure edits preserve ownership.
fn retained_region_identity(world: &mut World) -> BTreeMap<Entity, (Option<Entity>, Vec<Entity>)> {
    world
        .query::<(
            Entity,
            &view::RegionKind,
            Option<&ChildOf>,
            Option<&Children>,
        )>()
        .iter(world)
        .map(|(entity, kind, parent, children)| {
            let retained_children = if *kind == view::RegionKind::Collection {
                vec![]
            } else {
                children
                    .map(|children| children.iter().collect())
                    .unwrap_or_default()
            };
            (entity, (parent.map(ChildOf::parent), retained_children))
        })
        .collect()
}

/// Typing and virtual-row replacement must render measured text in place on their first frame.
#[test]
fn measured_text_geometry_is_stable_on_typing_and_virtual_scroll_rebuild_frames() {
    for scale in [1., 2.] {
        let (mut app, window) = input_app();
        enable_headless_layout_at_scale(&mut app, 1160, 820, scale);
        {
            let mut runtime = app.world_mut().resource_mut::<Runtime>();
            runtime.destination = Destination::Local;
            runtime.mode = Mode::Local;
            runtime.catalog = std::sync::Arc::new(
                crate::catalog::load_local(
                    &crate::app::resource_root().join("sample-packs/reactions"),
                )
                .unwrap(),
            );
            runtime.status = "Measured layout fixture".into();
        }
        for _ in 0..5 {
            app.update();
        }
        let initial = MeasuredTextGeometry::capture(app.world_mut());
        let region_identity = retained_region_identity(app.world_mut());
        assert!(!region_identity.is_empty());
        assert!(initial["durop"].iter().all(|geometry| geometry.size.x > 0.
            && geometry.size.y > 0.
            && !geometry.glyph_positions.is_empty()));
        // Whitespace changes the real search draft while preserving the complete catalog result set.
        key(&mut app, window, Key::Space, KeyCode::Space, Some(" "));
        app.update();
        let event_frame = MeasuredTextGeometry::capture(app.world_mut());
        assert_eq!(
            region_identity,
            retained_region_identity(app.world_mut()),
            "typing event replaced chrome ownership"
        );
        MeasuredTextGeometry::assert_stable(&initial, &event_frame, "typing event frame");
        app.update();
        let typed = MeasuredTextGeometry::capture(app.world_mut());
        assert_eq!(
            region_identity,
            retained_region_identity(app.world_mut()),
            "typing refresh replaced chrome ownership"
        );
        MeasuredTextGeometry::assert_stable(&initial, &typed, "typing rebuild frame");
        for _ in 0..3 {
            app.update();
            let typed_idle = MeasuredTextGeometry::capture(app.world_mut());
            MeasuredTextGeometry::assert_stable(&typed, &typed_idle, "typing rebuild to idle");
        }
        let stride = app.world().resource::<Model>().row_stride();
        app.world_mut().write_message(MouseWheel {
            window,
            unit: bevy::input::mouse::MouseScrollUnit::Pixel,
            x: 0.,
            phase: bevy::input::touch::TouchPhase::Moved,
            y: -(stride + 8.) * scale,
        });
        app.update();
        assert!(app.world().resource::<Model>().first_row() > 0);
        let scrolled = MeasuredTextGeometry::capture(app.world_mut());
        assert_eq!(
            region_identity,
            retained_region_identity(app.world_mut()),
            "virtual scroll replaced chrome ownership"
        );
        for _ in 0..3 {
            app.update();
            let scrolled_idle = MeasuredTextGeometry::capture(app.world_mut());
            MeasuredTextGeometry::assert_stable(&scrolled, &scrolled_idle, "virtualized scrolling");
        }
        app.world_mut().resource_mut::<Model>().open(
            Provider::Catalog(CatalogProvider::GitHttps),
            &provider_context(),
        );
        for _ in 0..4 {
            app.update();
        }
        let modal_geometry = MeasuredTextGeometry::capture(app.world_mut());
        let modal_identity = retained_region_identity(app.world_mut());
        key(
            &mut app,
            window,
            Key::Character("x".into()),
            KeyCode::KeyX,
            Some("x"),
        );
        for _ in 0..3 {
            app.update();
            assert_eq!(
                modal_identity,
                retained_region_identity(app.world_mut()),
                "ordinary draft editing replaced modal ownership"
            );
            let modal_frame = MeasuredTextGeometry::capture(app.world_mut());
            MeasuredTextGeometry::assert_stable(
                &modal_geometry,
                &modal_frame,
                "modal draft typing",
            );
        }
    }
}

/// Real glyph metrics must not stretch the toolbar or its single-line search input.
#[test]
fn measured_toolbar_and_search_fit_responsive_rows() {
    for width in [1160, 840, 839, 720, 534, 533, 400] {
        let (mut app, _) = input_app();
        enable_headless_layout(&mut app, width, 820);
        for _ in 0..5 {
            app.update();
        }
        let search = app.world().resource::<Model>().query.entity.unwrap();
        let search_node = app.world().get::<ComputedNode>(search).unwrap();
        assert_eq!(
            search_node.size().y * search_node.inverse_scale_factor(),
            24.,
            "search height at width {width}"
        );
        let search_container = app.world().get::<ChildOf>(search).unwrap().parent();
        let toolbar = app
            .world()
            .get::<ChildOf>(search_container)
            .unwrap()
            .parent();
        let toolbar_node = app.world().get::<ComputedNode>(toolbar).unwrap();
        assert_eq!(
            toolbar_node.size().y * toolbar_node.inverse_scale_factor(),
            GridLayout::toolbar_height(width as f32),
            "toolbar height at width {width}"
        );
    }
}

/// Compact source settings must scroll their final Back action fully into the content viewport.
#[test]
fn measured_compact_source_details_reach_final_action_without_clipping() {
    for details in [SourceDetails::Listener, SourceDetails::Credentials] {
        let (mut app, window) = input_app();
        enable_headless_layout(&mut app, 720, 520);
        let mut source_fixture = sources_app();
        app.insert_resource(
            source_fixture
                .world_mut()
                .remove_resource::<crate::network_app::NetworkState>()
                .unwrap(),
        );
        {
            let mut model = app.world_mut().resource_mut::<Model>();
            model.open_sources("100.101.102.103:48732");
            let modal = model.modal.as_mut().unwrap();
            modal.select_source(SourceSelection::Remote("alice".into()));
            modal.show_source_details(details);
        }
        for _ in 0..5 {
            app.update();
        }
        app.world_mut().write_message(MouseWheel {
            window,
            unit: bevy::input::mouse::MouseScrollUnit::Pixel,
            x: 0.,
            y: -10000.,
            phase: bevy::input::touch::TouchPhase::Moved,
        });
        for _ in 0..3 {
            app.update();
        }
        let body = app
            .world_mut()
            .query_filtered::<Entity, With<ModalBody>>()
            .single(app.world())
            .unwrap();
        let body_node = app.world().get::<ComputedNode>(body).unwrap();
        let body_transform = app.world().get::<UiGlobalTransform>(body).unwrap();
        let viewport_top = body_transform
            .affine()
            .transform_point2(body_node.content_box().min)
            .y;
        let viewport_bottom = body_transform
            .affine()
            .transform_point2(body_node.content_box().max)
            .y;
        let back_label = app
            .world_mut()
            .query::<(Entity, &Text)>()
            .iter(app.world())
            .find_map(|(entity, text)| (text.0 == "Back to controller").then_some(entity))
            .unwrap();
        let back = app.world().get::<ChildOf>(back_label).unwrap().parent();
        let back_node = app.world().get::<ComputedNode>(back).unwrap();
        let back_transform = app.world().get::<UiGlobalTransform>(back).unwrap();
        let back_top = back_transform.translation.y - back_node.size().y * 0.5;
        let back_bottom = back_transform.translation.y + back_node.size().y * 0.5;
        assert!(
            back_top >= viewport_top - 1. && back_bottom <= viewport_bottom + 1.,
            "final source action [{back_top}, {back_bottom}] clipped by viewport [{viewport_top}, {viewport_bottom}]"
        );
        let form = app.world().get::<Children>(body).unwrap()[0];
        let form_node = app.world().get::<ComputedNode>(form).unwrap();
        let form_transform = app.world().get::<UiGlobalTransform>(form).unwrap();
        let form_top = form_transform.translation.y - form_node.size().y * 0.5;
        assert!(
            back_bottom - form_top <= form_node.size().y + 1.,
            "scroll content omits the final source action"
        );
    }
}

/// Reusing an unchanged count must append it after the replacement heading in its stable region.
#[test]
fn measured_collection_heading_changes_preserve_left_right_order_and_camera() {
    let (mut app, _) = input_app();
    enable_headless_layout(&mut app, 1160, 820);
    app.world_mut().resource_mut::<Runtime>().destination = Destination::Local;
    for _ in 0..5 {
        app.update();
    }
    let count = app
        .world_mut()
        .query::<(Entity, &Text)>()
        .iter(app.world())
        .find_map(|(entity, text)| (text.0 == "0 REACTIONS").then_some(entity))
        .unwrap();
    for (destination, heading) in [
        (Destination::Remote, "REMOTE COLLECTION"),
        (Destination::Local, "YOUR COLLECTION"),
        (Destination::Remote, "REMOTE COLLECTION"),
    ] {
        app.world_mut().resource_mut::<Runtime>().destination = destination;
        app.update();
        let frame = MeasuredTextGeometry::capture(app.world_mut());
        assert!(
            frame[heading][0].center.x < frame["0 REACTIONS"][0].center.x,
            "count moved before the replacement heading"
        );
        assert!(frame[heading][0].camera.is_some());
        assert!(frame["0 REACTIONS"][0].camera.is_some());
        assert!(app.world().get::<Text>(count).is_some());
        app.update();
        let idle = MeasuredTextGeometry::capture(app.world_mut());
        assert_eq!(
            frame[heading], idle[heading],
            "replacement heading changed geometry or camera on the idle frame"
        );
        assert_eq!(
            frame["0 REACTIONS"], idle["0 REACTIONS"],
            "retained count changed geometry or camera on the idle frame"
        );
    }
}

/// Measured compact placement forms keep native fields scrollable and every footer action available.
#[test]
fn measured_compact_placement_form_keeps_footer_and_last_content_visible() {
    let (mut app, window) = input_app();
    enable_headless_layout(&mut app, 720, 520);
    let runtime = placement_runtime();
    let clip_id = visual_clip(&runtime).identity.clone();
    let network = crate::network_app::NetworkState::for_ui_test(runtime.snapshot.clone());
    app.insert_resource(runtime).insert_resource(network);
    app.world_mut()
        .resource_scope(|world, mut model: Mut<Model>| {
            let runtime = world.resource::<Runtime>();
            let network = world.resource::<crate::network_app::NetworkState>();
            model.sync_placements(PlacementScope::current(runtime, network));
            model.open_placement(&clip_id, runtime, network);
            model.placement_modal.as_mut().unwrap().title =
                "A long placement title that wraps without hiding Apply or Send".into();
        });
    for _ in 0..5 {
        app.update();
    }
    for after_scroll in [false, true] {
        if after_scroll {
            wheel_input(
                &mut app,
                window,
                bevy::input::mouse::MouseScrollUnit::Pixel,
                -10000.,
            );
            for _ in 0..3 {
                app.update();
            }
        }
        let mut dialogs = app
            .world_mut()
            .query::<(&TabGroup, &ComputedNode, &UiGlobalTransform)>();
        let (_, dialog_node, dialog_transform) = dialogs
            .iter(app.world())
            .find(|(group, _, _)| group.modal)
            .unwrap();
        let dialog_top = dialog_transform.translation.y - dialog_node.size().y * 0.5;
        let dialog_bottom = dialog_transform.translation.y + dialog_node.size().y * 0.5;
        assert!(
            dialog_top >= 23. && dialog_bottom <= 497.,
            "placement dialog escaped the compact viewport"
        );
        let footer_buttons: Vec<_> = app
            .world_mut()
            .query::<(&UiAction, &ComputedNode, &UiGlobalTransform)>()
            .iter(app.world())
            .filter(|(action, _, _)| {
                matches!(
                    action,
                    UiAction::ApplyPlacement(_) | UiAction::ResetPlacement
                )
            })
            .map(|(_, node, transform)| {
                (
                    transform.translation.y - node.size().y * 0.5,
                    transform.translation.y + node.size().y * 0.5,
                )
            })
            .collect();
        assert_eq!(footer_buttons.len(), 3);
        assert!(
            footer_buttons
                .iter()
                .all(|(top, bottom)| *top >= dialog_top && *bottom <= dialog_bottom),
            "placement footer action was clipped"
        );
        let body = app
            .world_mut()
            .query_filtered::<Entity, With<ModalBody>>()
            .single(app.world())
            .unwrap();
        let body_node = app.world().get::<ComputedNode>(body).unwrap();
        assert!(body_node.size().y > 0.);
        if after_scroll {
            let body_transform = app.world().get::<UiGlobalTransform>(body).unwrap();
            let content_bottom = body_transform
                .affine()
                .transform_point2(body_node.content_box().max)
                .y;
            let hint = app
                .world_mut()
                .query::<(&Text, &ComputedNode, &UiGlobalTransform)>()
                .iter(app.world())
                .find(|(text, _, _)| text.0.starts_with("Apply updates active plays"))
                .map(|(_, node, transform)| (node.size().y, transform.translation.y))
                .unwrap();
            assert!(
                hint.1 + hint.0 * 0.5 <= content_bottom + 1.,
                "final placement help text remains clipped at maximum scroll"
            );
        }
    }
}
