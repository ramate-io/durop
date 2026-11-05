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
        protocol_version: 1,
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
    assert_eq!(
        Model::visible_indices(&clips, "sound music", 100).len(),
        COLUMNS * ROWS
    );
    assert_eq!(Model::visible_indices(&clips, "sound music", 100)[0], 400);
    assert_eq!(Model::visible_indices(&clips, "Title 9999", 0), vec![9999]);
    assert!(Model::visible_indices(&clips, "absent", 0).is_empty());
    assert_eq!(Model::visible_indices(&clips, "", usize::MAX).len(), 0);
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
    model.seek_scrollbar(1.);
    assert_eq!(model.first_row, model.max_row());
    assert!(model.first_row > 2000);
    model.seek_scrollbar(0.);
    assert_eq!(model.first_row, 0);
    model.matches = 0;
    model.seek_scrollbar(1.);
    assert_eq!(model.first_row, 0);
    assert_eq!(model.max_row(), 0);
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
    assert_eq!(model.first_row, model.max_row());
    app.world_mut().entity_mut(track).insert((
        Interaction::None,
        bevy::ui::RelativeCursorPosition {
            normalized: Some(Vec2::new(0., -0.5)),
            ..default()
        },
    ));
    app.update();
    assert_eq!(app.world().resource::<Model>().first_row, 0);
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
    assert_eq!(model.first_row, 0);
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
        first_row: 3,
        ..default()
    };
    model.open(
        Provider::Catalog(CatalogProvider::GitHttps),
        &provider_context(),
    );
    app.insert_resource(model)
        .add_message::<MouseWheel>()
        .add_systems(Update, scroll);
    let window = app.world_mut().spawn(PrimaryWindow).id();
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
    assert_eq!(model.first_row, 3);
    assert_eq!(model.modal.as_ref().unwrap().scroll, 400.);
}
