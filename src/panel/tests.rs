//! Regression coverage for input routing, modal isolation and bounded collection geometry.

use super::*;

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

/// Construct real input message routing with a focused synthetic primary window.
fn input_app() -> (App, Entity) {
    let mut app = App::new();
    app.insert_resource(Runtime::new(&config()).unwrap())
        .init_resource::<Model>()
        .init_resource::<Actions>()
        .init_resource::<ButtonInput<KeyCode>>()
        .add_message::<KeyboardInput>()
        .add_message::<Ime>()
        .add_message::<bevy::winit::RawWinitWindowEvent>()
        .add_systems(Update, text_input);
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
    app.world_mut().resource_mut::<Model>().focus = Some(Field::Search);
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

/// Within-frame modifier ordering must preserve select-all semantics.
#[test]
fn modifier_down_a_modifier_up_in_one_update_selects_instead_of_typing_a() {
    let (mut app, window) = input_app();
    {
        let mut model = app.world_mut().resource_mut::<Model>();
        model.query.insert("gif");
    }
    key(&mut app, window, Key::Super, KeyCode::SuperLeft, None);
    key(
        &mut app,
        window,
        Key::Character("a".into()),
        KeyCode::KeyA,
        Some("a"),
    );
    app.world_mut().write_message(KeyboardInput {
        key_code: KeyCode::SuperLeft,
        logical_key: Key::Super,
        state: ButtonState::Released,
        text: None,
        repeat: false,
        window,
    });
    app.update();
    assert_eq!(app.world().resource::<Model>().query.text, "gif");
    assert!(app.world().resource::<Model>().query.select_all);
    assert_eq!(app.world().resource::<Model>().modifiers, 0);
    key(
        &mut app,
        window,
        Key::Character("video".into()),
        KeyCode::KeyV,
        Some("video"),
    );
    app.update();
    assert_eq!(app.world().resource::<Model>().query.text, "video");
    assert!(app.world().resource::<Actions>().0.is_empty());
}

/// Native modifier flags must work even without a physical modifier key event.
#[test]
fn flag_only_modifiers_apply_to_the_ordered_key_without_physical_modifier_events() {
    let entity = Entity::PLACEHOLDER;
    let key = |text: &str, code| KeyboardInput {
        window: entity,
        key_code: code,
        logical_key: Key::Character(text.into()),
        text: Some(text.into()),
        state: ButtonState::Pressed,
        repeat: false,
    };
    let mut flags = 0;
    let keys = ordered_keyboard(
        [
            OrderedInput::Modifiers(1),
            OrderedInput::Keyboard(key("a", KeyCode::KeyA)),
            OrderedInput::Modifiers(0),
            OrderedInput::Keyboard(key("g", KeyCode::KeyG)),
        ],
        &mut flags,
    );
    assert_eq!(keys.len(), 2);
    assert!(keys[0].1);
    assert!(!keys[1].1);
    assert_eq!(flags, 0);
    let mut query = Editor::default();
    query.insert("video");
    for (key, modified) in keys {
        if modified && key.key_code == KeyCode::KeyA {
            query.select_all = true;
        } else if !modified {
            query.insert(key.text.as_deref().unwrap());
        }
    }
    assert_eq!(query.text, "g");
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
        .add_systems(Update, buttons);
    app.world_mut()
        .spawn((Interaction::Pressed, UiAction::Runtime(Action::StopAll)));
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
    key(
        &mut app,
        window,
        Key::Character("n".into()),
        KeyCode::KeyN,
        Some("n"),
    );
    app.update();
    assert_eq!(app.world().resource::<Model>().query.text, "");
    assert_eq!(app.world().resource::<Model>().query.preedit, "に");
    app.world_mut().write_message(Ime::Commit {
        window,
        value: "日本".into(),
    });
    key(
        &mut app,
        window,
        Key::Character("日本".into()),
        KeyCode::KeyN,
        Some("日本"),
    );
    app.update();
    assert_eq!(app.world().resource::<Model>().query.text, "日本");
    key(&mut app, window, Key::Backspace, KeyCode::Backspace, None);
    app.update();
    assert_eq!(app.world().resource::<Model>().query.text, "日");
    assert!(app.world().resource::<Actions>().0.is_empty());
}

/// Editing and selection must not split UTF-8 scalar boundaries.
#[test]
fn unicode_cursor_selection_and_backspace_stay_on_character_boundaries() {
    let mut editor = Editor::default();
    editor.insert("Aé🌈");
    editor.left();
    editor.backspace();
    assert_eq!(editor.text, "A🌈");
    editor.right();
    editor.backspace();
    assert_eq!(editor.text, "A");
    editor.select_all = true;
    editor.insert("日本");
    assert_eq!(editor.text, "日本");
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
    model.modal.as_mut().unwrap().editors[1].insert(&"a".repeat(64));
    let mut app = App::new();
    app.insert_resource(runtime)
        .insert_resource(network)
        .insert_resource(model)
        .init_resource::<Actions>()
        .add_systems(Update, buttons);
    app.world_mut().spawn((
        Interaction::Pressed,
        UiAction::Runtime(Action::Play("background/clip".into())),
    ));
    app.world_mut()
        .spawn((Interaction::Pressed, UiAction::Runtime(Action::StopAll)));
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
    model.modal.as_mut().unwrap().editors[1].insert(&"b".repeat(64));
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
        .add_systems(Update, buttons);
    app.world_mut()
        .spawn((Interaction::Pressed, UiAction::Apply));
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
            Interaction::Pressed,
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
        Interaction::None,
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
        Interaction::Pressed,
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
            .add_systems(Update, buttons);
        app.world_mut()
            .spawn((Interaction::Pressed, UiAction::Apply));
        app.update();
        assert!(app.world().resource::<Actions>().0.is_empty());
        let model = app.world().resource::<Model>();
        let modal = model
            .modal
            .as_ref()
            .expect("blocked apply retains the draft");
        assert_eq!(modal.draft.provider, Provider::Catalog(provider));
        assert!(modal.error.as_ref().unwrap().contains("remote host"));
    }
}

/// Scrolling a short form reaches its bounded body without moving the clip collection.
#[test]
fn modal_wheel_scroll_is_clamped_and_does_not_move_catalog_rows() {
    let mut app = App::new();
    let mut model = Model {
        scroll: 903.,
        matches: 10_000,
        ..default()
    };
    model.open(
        Provider::Catalog(CatalogProvider::GitHttps),
        &provider_context(),
    );
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
    let model = app.world().resource::<Model>();
    assert_eq!(model.scroll, 903.);
    assert_eq!(model.modal.as_ref().unwrap().scroll, 400.);
}

/// A complete compact card retains its two-line title and action row below the preview.
#[test]
fn short_window_cards_fit_titles_and_controls_inside_the_collection() {
    for (width, height) in [(720., 520.), (400., 520.), (1160., 520.), (1160., 820.)] {
        let layout = GridLayout::for_window(width, height);
        let toolbar_extra = if width < 760. { 80. } else { 0. };
        let available_height = height - 296. - toolbar_extra;
        assert!(layout.card_height <= available_height);
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
            Interaction::Pressed,
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
    let defaults = VisualPlacement {
        width_bps: 1,
        x_bps: 0,
        y_bps: 10000,
    };
    {
        let mut model = app.world_mut().resource_mut::<Model>();
        model.placement_modal = Some(PlacementModal::new(
            "test/clip".into(),
            "Test clip".into(),
            defaults,
            defaults,
        ));
        model.focus = Some(Field::Placement(0));
        model.placement_modal.as_mut().unwrap().editors[0].select_all = true;
    }
    key(
        &mut app,
        window,
        Key::Character("33.33".into()),
        KeyCode::Digit3,
        Some("33.33"),
    );
    app.update();
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
    assert!(app.world().resource::<Actions>().0.is_empty());
    key(&mut app, window, Key::Escape, KeyCode::Escape, None);
    app.update();
    assert!(app.world().resource::<Model>().placement_modal.is_none());
    assert!(app.world().resource::<Actions>().0.is_empty());
}
