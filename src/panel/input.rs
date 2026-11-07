//! Engine-backed editing, persistent entity ownership and protected secret entry.

use super::*;

/// Semantic identity shared by the draft mirror and its persistent input entity.
#[derive(Component, Clone, Copy)]
pub(super) struct InputField(pub(super) Field);

/// A masked fallback because Bevy 0.20 does not implement password rendering.
#[derive(Component)]
pub(super) struct SecretInput;

/// Activation intents outlive transient button entities until engine text edits are committed.
#[derive(Resource, Default)]
pub(super) struct PendingActions(pub(super) Vec<UiAction>);

/// Capture the semantic action before surrounding widgets can be refreshed.
pub(super) fn activate(
    on: On<Activate>,
    mut pending: ResMut<PendingActions>,
    actions: Query<&UiAction>,
) {
    if let Ok(action) = actions.get(on.entity) {
        pending.0.push(action.clone());
    }
}

/// Construct or retain an input without replacing Parley's cursor, selection or composition.
fn prepare_editor(
    commands: &mut Commands,
    editor: &mut Editor,
    field: Field,
    secret: bool,
    fonts: &Fonts,
) -> Entity {
    if let Some(entity) = editor.entity {
        // A refreshed region reparents this editor before retiring its previous parent.
        return entity;
    }
    // Toolbar search shares the outer selector surface; modal editors own their field frame.
    let search = field == Field::Search;
    let height = if search { 24. } else { 46. };
    let mut input = commands.spawn((
        InputField(field),
        Node {
            width: percent(100),
            min_width: px(0),
            // Explicit border-box bounds avoid TextInputMeasure adding padding to its
            // minimum content height, which otherwise expands this editor by 28 pixels.
            height: px(height),
            min_height: px(height),
            max_height: px(height),
            padding: if search {
                UiRect::ZERO
            } else {
                UiRect::axes(px(14), px(13))
            },
            border: UiRect::all(px(if search { 0. } else { 1. })),
            border_radius: BorderRadius::all(px(5)),
            overflow: Overflow::clip(),
            ..default()
        },
        fonts.mono(13.),
        TextColor(theme::INK),
        TextLayout::no_wrap(),
        BackgroundColor(if search {
            theme::PAPER
        } else {
            theme::BACKGROUND
        }),
        BorderColor::all(theme::RULE),
        TabIndex(match field {
            Field::Search => 0,
            Field::Configuration(index) => index as i32,
        }),
    ));
    if secret {
        input.insert((
            SecretInput,
            Text::new("•".repeat(editor.text.chars().count())),
        ));
    } else {
        input.insert((
            TextInput,
            EditableText {
                max_characters: Some(4096),
                allow_newlines: false,
                ..EditableText::new(&editor.text)
            },
            EditableTextFilter::new(|c| !c.is_control()),
            // Extraction needs this component for both the caret and selection highlight.
            TextCursorStyle::default(),
        ));
    }
    let entity = input.id();
    editor.entity = Some(entity);
    entity
}

/// Retain live fields for direct reparenting and retire fields whose owning draft no longer exists.
pub(super) fn prepare(
    commands: &mut Commands,
    model: &mut Model,
    existing: &Query<Entity, With<InputField>>,
    fonts: &Fonts,
    focus: &mut InputFocus,
) {
    let mut retained = vec![prepare_editor(
        commands,
        &mut model.query,
        Field::Search,
        false,
        fonts,
    )];
    if let Some(modal) = &mut model.modal {
        let fields: Vec<_> = match &modal.draft {
            ModalDraft::Configuration(draft) => draft
                .fields
                .iter()
                .enumerate()
                .filter(|(_, field)| !field.read_only && field.choices.is_empty())
                .map(|(index, field)| (index, field.secret))
                .collect(),
            ModalDraft::Sources(_) => modal
                .source_fields()
                .into_iter()
                .map(|index| (index, index == 1))
                .collect(),
        };
        for (index, editor) in modal.editors.iter_mut().enumerate() {
            if !fields.iter().any(|(visible, _)| *visible == index) {
                editor.entity = None;
            }
        }
        let mut first_new = None;
        for (index, secret) in fields {
            let is_new = modal.editors[index].entity.is_none();
            let entity = prepare_editor(
                commands,
                &mut modal.editors[index],
                Field::Configuration(index),
                secret,
                fonts,
            );
            if is_new && first_new.is_none() {
                first_new = Some(entity);
            }
            retained.push(entity);
        }
        if let Some(entity) = first_new
            && !focus
                .get()
                .is_some_and(|entity| retained[1..].contains(&entity))
        {
            focus.set(entity, FocusCause::Navigated);
        }
    }
    for entity in existing {
        if !retained.contains(&entity) {
            // Detach first, so retirement has one owner rather than racing parent teardown.
            commands.entity(entity).remove::<ChildOf>().despawn();
            if focus.get() == Some(entity) {
                focus.clear();
            }
        }
    }
    let search = model.query.entity.unwrap();
    if model.modal.is_some() {
        commands
            .entity(search)
            .insert((InteractionDisabled, TabIndex(-1)));
        if focus.get() == Some(search) {
            focus.clear();
        }
    } else {
        commands
            .entity(search)
            .remove::<InteractionDisabled>()
            .insert(TabIndex(0));
    }
}

/// Mirror committed engine text after edits apply; preedit never enters saved preferences/actions.
pub(super) fn synchronize(
    mut model: ResMut<Model>,
    inputs: Query<(Entity, &InputField, &EditableText)>,
    focus: Res<InputFocus>,
    fields: Query<&InputField>,
) {
    for (entity, field, text) in &inputs {
        let Some(editor) = model.editor(field.0) else {
            continue;
        };
        if editor.entity != Some(entity) {
            continue;
        }
        let value = text.value().to_string();
        if editor.text != value {
            editor.text = value;
            if field.0 == Field::Search {
                model.scroll = 0.;
            }
            model.revision = model.revision.wrapping_add(1);
        }
    }
    model.focus = focus
        .get()
        .and_then(|entity| fields.get(entity).ok())
        .map(|field| field.0);
}

/// Reflect focus and masked values directly, without invalidating surrounding layout.
pub(super) fn focus_style(
    focus: Res<InputFocus>,
    model: Res<Model>,
    mut fields: Query<(Entity, &InputField, &mut BorderColor, Option<&mut Text>)>,
) {
    for (entity, field, mut border, label) in &mut fields {
        border.set_all(if focus.get() == Some(entity) {
            theme::PURPLE
        } else {
            theme::RULE
        });
        if let (Field::Configuration(index), Some(mut label), Some(modal)) =
            (field.0, label, model.modal.as_ref())
            && let Some(editor) = modal.editors.get(index)
        {
            label.0 = "•".repeat(editor.text.chars().count());
        }
    }
}

/// Route shortcuts using the original event target even when engine editing clears InputFocus.
/// Secret fields deliberately accept bounded append/replacement only and never export plaintext.
#[expect(
    clippy::too_many_arguments,
    reason = "Focused input bridges native modifiers, clipboard, protected drafts and playback shortcuts"
)]
pub(super) fn keyboard(
    mut on: On<FocusedInput<KeyboardInput>>,
    fields: Query<(&InputField, Has<SecretInput>)>,
    text_targets: Query<(), With<EditableText>>,
    keys: Res<ButtonInput<Key>>,
    mut clipboard: ResMut<bevy::clipboard::Clipboard>,
    mut model: ResMut<Model>,
    mut focus: ResMut<InputFocus>,
    runtime: Res<Runtime>,
    mut actions: ResMut<Actions>,
) {
    // A bubbling observer can run on ancestors too. Each physical event is handled once.
    if on.focused_entity != on.original_event_target() || !on.input.state.is_pressed() {
        return;
    }
    let field = fields.get(on.original_event_target()).ok();
    let is_text = field.is_some() || text_targets.contains(on.original_event_target());
    let command = keys.pressed(if cfg!(target_os = "macos") {
        Key::Super
    } else {
        Key::Control
    });
    if on.input.logical_key == Key::Escape {
        if model.modal.is_some() || model.dropdown.is_some() {
            model.close();
            model.revision = model.revision.wrapping_add(1);
            focus.clear();
        } else if !is_text {
            actions.0.push_back(Action::StopAll);
        }
        return;
    }
    if let Some((field, true)) = field {
        if on.input.logical_key == Key::Tab {
            return;
        }
        on.propagate(false);
        let Some(editor) = model.editor(field.0) else {
            return;
        };
        let mut insert = None;
        let mut changed = false;
        match &on.input.logical_key {
            Key::Character(value) if command && value.eq_ignore_ascii_case("a") => {
                editor.selected = true
            }
            Key::Character(value) if command && value.eq_ignore_ascii_case("v") => {
                insert = clipboard
                    .fetch_text()
                    .poll_result()
                    .and_then(Result::ok)
                    .map(|value| value.trim().to_owned());
            }
            Key::Backspace | Key::Delete => {
                if editor.selected {
                    changed = !editor.text.is_empty();
                    editor.text.clear();
                    editor.selected = false;
                } else {
                    changed = editor.text.pop().is_some();
                }
            }
            Key::Enter => focus.clear(),
            _ if !command => insert = on.input.text.as_ref().map(ToString::to_string),
            _ => (),
        }
        if let Some(value) = insert {
            if editor.selected {
                changed |= !editor.text.is_empty();
                editor.text.clear();
                editor.selected = false;
            }
            let remaining = 4096usize.saturating_sub(editor.text.chars().count());
            for character in value.chars().filter(|c| !c.is_control()).take(remaining) {
                editor.text.push(character);
                changed = true;
            }
        }
        if changed {
            // Wake a settling frame without putting credential bytes in a render fingerprint.
            model.revision = model.revision.wrapping_add(1);
        }
        return;
    }
    if is_text || command || on.input.repeat || model.modal.is_some() || model.dropdown.is_some() {
        return;
    }
    if on.input.logical_key == Key::Space
        && runtime.destination == Destination::Local
        && let Some(index) = model.visible(&runtime.catalog.clips).first()
    {
        actions
            .0
            .push_back(Action::Play(runtime.catalog.clips[*index].identity.clone()));
    }
}

/// Native modifiers can change without physical key messages; retain their event ordering.
#[derive(Resource, Default)]
pub(super) struct OrderedKeyboard {
    /// Independent cursor leaves native window messages available to the window backend.
    raw: bevy::ecs::message::MessageCursor<bevy::winit::RawWinitWindowEvent>,
    /// Super, Control, Alt and Shift flags at the latest native event.
    modifiers: u8,
    /// Synthetic/portable keyboard events use their ordered physical modifier transitions.
    physical: ButtonInput<KeyCode>,
}

impl OrderedKeyboard {
    /// Read raw modifier changes and translated key events for the panel window.
    fn native_events(&mut self, world: &World, window: Entity) -> Vec<(KeyboardInput, u8)> {
        let native_id = bevy::winit::WINIT_WINDOWS
            .with(|windows| windows.borrow().entity_to_winit.get(&window).copied());
        let Some(messages) = world.get_resource::<Messages<bevy::winit::RawWinitWindowEvent>>()
        else {
            return vec![];
        };
        let mut events = Vec::new();
        for event in self.raw.read(messages) {
            if Some(event.window_id) != native_id {
                continue;
            }
            match &event.event {
                winit::event::WindowEvent::ModifiersChanged(modifiers) => {
                    let flags = modifiers.state();
                    self.modifiers = u8::from(flags.super_key())
                        | u8::from(flags.control_key()) << 1
                        | u8::from(flags.alt_key()) << 2
                        | u8::from(flags.shift_key()) << 3;
                }
                winit::event::WindowEvent::KeyboardInput {
                    event,
                    is_synthetic: false,
                    ..
                } => {
                    events.push((
                        bevy::winit::converters::convert_keyboard_input(event, window),
                        self.modifiers,
                    ));
                }
                winit::event::WindowEvent::Focused(false) => self.modifiers = 0,
                _ => (),
            }
        }
        events
    }

    /// Reconstruct portable modifier state at each key, including down/key/up in one frame.
    fn portable_events(&mut self, events: Vec<KeyboardInput>) -> Vec<(KeyboardInput, u8)> {
        events
            .into_iter()
            .map(|event| {
                if event.state.is_pressed() {
                    self.physical.press(event.key_code);
                } else {
                    self.physical.release(event.key_code);
                }
                let flags = [
                    (KeyCode::SuperLeft, KeyCode::SuperRight),
                    (KeyCode::ControlLeft, KeyCode::ControlRight),
                    (KeyCode::AltLeft, KeyCode::AltRight),
                    (KeyCode::ShiftLeft, KeyCode::ShiftRight),
                ]
                .into_iter()
                .enumerate()
                .fold(0, |flags, (index, (left, right))| {
                    flags
                        | u8::from(self.physical.pressed(left) || self.physical.pressed(right))
                            << index
                });
                (event, flags)
            })
            .collect()
    }
    /// Set both logical and physical modifier views for one synchronous engine event.
    fn apply_modifiers(world: &mut World, modifiers: u8) {
        for (index, (key, left, right)) in [
            (Key::Super, KeyCode::SuperLeft, KeyCode::SuperRight),
            (Key::Control, KeyCode::ControlLeft, KeyCode::ControlRight),
            (Key::Alt, KeyCode::AltLeft, KeyCode::AltRight),
            (Key::Shift, KeyCode::ShiftLeft, KeyCode::ShiftRight),
        ]
        .into_iter()
        .enumerate()
        {
            world.resource_mut::<ButtonInput<KeyCode>>().release(left);
            world.resource_mut::<ButtonInput<KeyCode>>().release(right);
            if modifiers & (1 << index) != 0 {
                world.resource_mut::<ButtonInput<Key>>().press(key);
                world.resource_mut::<ButtonInput<KeyCode>>().press(left);
            } else {
                world.resource_mut::<ButtonInput<Key>>().release(key);
            }
        }
    }
}

/// Dispatch the engine's focused keyboard events synchronously with their native modifier state.
/// Bevy's default dispatcher sees only the final frame modifier state; macOS may report Command
/// solely through `ModifiersChanged`. Application keyboard messages are drained after InputSystems
/// consumed them, preventing duplicate engine dispatch; mouse/gamepad dispatch remains native.
pub(super) fn dispatch_keyboard(world: &mut World) {
    let Some((window, focused)) = world
        .query_filtered::<(Entity, &Window), With<PrimaryWindow>>()
        .iter(world)
        .next()
        .map(|(entity, window)| (entity, window.focused))
    else {
        return;
    };
    let messages: Vec<_> = world
        .resource_mut::<Messages<KeyboardInput>>()
        .drain()
        .filter(|event| event.window == window)
        .collect();
    let (ordered, native_modifiers) =
        world.resource_scope(|world, mut ordered: Mut<OrderedKeyboard>| {
            let native = ordered.native_events(world, window);
            if !focused {
                ordered.modifiers = 0;
                ordered.physical.release_all();
                return (vec![], None);
            }
            let native_modifiers = bevy::winit::WINIT_WINDOWS
                .with(|windows| windows.borrow().entity_to_winit.contains_key(&window))
                .then_some(ordered.modifiers);
            let events = if native.is_empty() {
                ordered.portable_events(messages)
            } else {
                native
            };
            (events, native_modifiers)
        });
    if !focused {
        return;
    }
    let logical = world.resource::<ButtonInput<Key>>().clone();
    let physical = world.resource::<ButtonInput<KeyCode>>().clone();
    for (event, modifiers) in ordered {
        if event.window != window {
            continue;
        }
        OrderedKeyboard::apply_modifiers(world, modifiers);
        let target = world
            .resource::<InputFocus>()
            .get()
            .filter(|entity| world.entities().contains(*entity))
            .unwrap_or(window);
        world.trigger(FocusedInput::new(target, event, window));
        world.flush();
    }
    *world.resource_mut::<ButtonInput<Key>>() = logical;
    *world.resource_mut::<ButtonInput<KeyCode>>() = physical;
    if let Some(modifiers) = native_modifiers {
        OrderedKeyboard::apply_modifiers(world, modifiers);
    }
}

/// Wake the low-power native loop when late text edits or modal actions need an Update render.
pub(super) fn request_redraw(
    model: Res<Model>,
    mut previous: Local<Option<(u64, Option<Field>)>>,
    catalogs: Query<&ComputedNode, With<CatalogBody>>,
    mut redraw: MessageWriter<bevy::window::RequestRedraw>,
) {
    let current = (model.revision, model.focus);
    // Initial layout and resizing are measured after Update. Wake exactly until the next
    // Update has copied the actual viewport bounds into the virtualized collection model.
    let viewport_changed = catalogs.iter().any(|node| {
        let height = node.size().y * node.inverse_scale_factor();
        height > 0. && (height - model.viewport_height).abs() > 0.01
    });
    if previous.as_ref() != Some(&current) || viewport_changed {
        *previous = Some(current);
        redraw.write(bevy::window::RequestRedraw);
    }
}
