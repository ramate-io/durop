//! Bounded catalog UI with an explicit Unicode and IME text editor.
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
use std::{
    collections::BTreeMap,
    io::Read,
    io::{self, BufRead},
    sync::{Mutex, mpsc},
};
const COLUMNS: usize = 4;
const ROWS: usize = 3;
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Field {
    Search,
    Pack,
    Address,
    Secret,
}
#[derive(Default, Debug, Clone)]
struct Editor {
    text: String,
    cursor: usize,
    preedit: String,
    select_all: bool,
}
impl Editor {
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
    fn left(&mut self) {
        self.cursor = self.text[..self.cursor]
            .char_indices()
            .next_back()
            .map_or(0, |(i, _)| i);
        self.select_all = false;
    }
    fn right(&mut self) {
        self.cursor = self.text[self.cursor..]
            .chars()
            .next()
            .map_or(self.cursor, |c| self.cursor + c.len_utf8());
        self.select_all = false;
    }
    fn label(&self) -> String {
        format!(
            "{}|{}{}",
            &self.text[..self.cursor],
            self.preedit,
            &self.text[self.cursor..]
        )
    }
}
#[derive(Resource, Default)]
struct Model {
    query: Editor,
    pack: Editor,
    focus: Option<Field>,
    first_row: usize,
    signature: String,
    root: Option<Entity>,
    address: Editor,
    secret: Editor,
    network_initialized: bool,
    saved_preferences: Option<crate::preferences::Preferences>,
}
#[derive(Component, Clone)]
enum UiAction {
    Runtime(Action),
    Focus(Field),
    Page(i32),
    Network(crate::network_app::NetworkAction),
}
#[derive(Resource, Default)]
struct Icons(BTreeMap<String, Option<Handle<Image>>>);
const MAX_ICON_BYTES: usize = 1024 * 1024;
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
fn read_icon(path: &std::path::Path) -> Result<Vec<u8>, std::io::Error> {
    let mut bytes = Vec::new();
    std::fs::File::open(path)?
        .take(MAX_ICON_BYTES as u64 + 1)
        .read_to_end(&mut bytes)?;
    Ok(bytes)
}
#[derive(Resource)]
struct Terminal(Mutex<mpsc::Receiver<String>>);
#[derive(Resource)]
struct PanelCamera(Entity);
pub struct PanelPlugin;
impl Plugin for PanelPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Model>()
            .init_resource::<Icons>()
            .add_systems(Startup, setup)
            .add_systems(
                Update,
                (buttons, text_input, scroll, terminal)
                    .chain()
                    .before(crate::app::actions),
            )
            .add_systems(
                Update,
                render
                    .after(crate::app::media_bridge)
                    .before(crate::overlay::sync_overlay),
            );
    }
}
fn setup(
    mut commands: Commands,
    primary: Single<Entity, With<PrimaryWindow>>,
    proxy: Res<bevy::winit::EventLoopProxyWrapper>,
    runtime: Res<Runtime>,
    mut model: ResMut<Model>,
    mut controls: ResMut<crate::overlay::PendingControls>,
) {
    let camera = commands
        .spawn((
            Camera2d,
            RenderTarget::Window(WindowRef::Entity(*primary)),
            RenderLayers::layer(PANEL_LAYER),
        ))
        .id();
    commands.insert_resource(PanelCamera(camera));
    model.pack.text = runtime.pack_input.clone();
    model.pack.cursor = model.pack.text.len();
    let preferences = crate::preferences::load();
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
fn visible_indices(clips: &[Clip], query: &str, row: usize) -> Vec<usize> {
    clips
        .iter()
        .enumerate()
        .filter(|(_, c)| matching(c, query))
        .skip(row.saturating_mul(COLUMNS))
        .take(COLUMNS * ROWS)
        .map(|(i, _)| i)
        .collect()
}
fn buttons(
    mut model: ResMut<Model>,
    buttons: Query<(&Interaction, &UiAction), Changed<Interaction>>,
    mut actions: ResMut<Actions>,
    mut network: ResMut<crate::network_app::NetworkState>,
) {
    for (interaction, action) in &buttons {
        if *interaction != Interaction::Pressed {
            continue;
        }
        match action {
            UiAction::Runtime(a) => {
                model.focus = None;
                actions.0.push_back(a.clone());
            }
            UiAction::Focus(field) => model.focus = Some(*field),
            UiAction::Page(d) => {
                model.first_row = if *d < 0 {
                    model.first_row.saturating_sub(ROWS)
                } else {
                    model.first_row + ROWS
                }
            }
            UiAction::Network(action) => {
                network.address = model.address.text.clone();
                if matches!(action, crate::network_app::NetworkAction::Connect) {
                    network.secret = std::mem::take(&mut model.secret.text);
                    model.secret.cursor = 0;
                }
                network.actions.push_back(action.clone());
                model.focus = None;
            }
        }
    }
}
#[expect(
    clippy::too_many_arguments,
    reason = "Native clipboard access keeps the editor on Bevy's main thread"
)]
fn text_input(
    mut keys: MessageReader<KeyboardInput>,
    mut ime: MessageReader<Ime>,
    primary: Single<(Entity, &mut Window), With<PrimaryWindow>>,
    mut model: ResMut<Model>,
    runtime: Res<Runtime>,
    held: Res<ButtonInput<KeyCode>>,
    mut actions: ResMut<Actions>,
    _main: bevy::ecs::system::NonSendMarker,
) {
    let (entity, mut window) = primary.into_inner();
    window.ime_enabled = model.focus.is_some();
    if !window.focused {
        keys.clear();
        ime.clear();
        return;
    }
    let mut committed = false;
    for event in ime.read() {
        let Some(field) = model.focus else {
            continue;
        };
        let edit = match field {
            Field::Search => &mut model.query,
            Field::Pack => &mut model.pack,
            Field::Address => &mut model.address,
            Field::Secret => &mut model.secret,
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
    for key in keys.read() {
        if key.window != entity || key.state != ButtonState::Pressed {
            continue;
        }
        if let Some(field) = model.focus {
            let modified = held.pressed(KeyCode::SuperLeft)
                || held.pressed(KeyCode::SuperRight)
                || held.pressed(KeyCode::ControlLeft)
                || held.pressed(KeyCode::ControlRight);
            let edit = match field {
                Field::Search => &mut model.query,
                Field::Pack => &mut model.pack,
                Field::Address => &mut model.address,
                Field::Secret => &mut model.secret,
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
                    if field == Field::Pack {
                        actions.0.push_back(Action::LoadPack(edit.text.clone()));
                    }
                    model.focus = None;
                }
                _ if modified && key.key_code == KeyCode::KeyA => edit.select_all = true,
                #[cfg(target_os = "macos")]
                _ if modified && key.key_code == KeyCode::KeyV => {
                    if let Some(text) = crate::app_macos::paste_text() {
                        edit.insert(if field == Field::Secret {
                            text.trim()
                        } else {
                            &text
                        });
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
        } else if !key.repeat {
            match key.logical_key {
                Key::Escape => actions.0.push_back(Action::StopAll),
                Key::Space if runtime.destination == Destination::Local => {
                    if let Some(index) =
                        visible_indices(&runtime.catalog.clips, &model.query.text, model.first_row)
                            .first()
                    {
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
fn scroll(
    mut wheel: MessageReader<MouseWheel>,
    primary: Single<Entity, With<PrimaryWindow>>,
    mut model: ResMut<Model>,
) {
    for event in wheel.read() {
        if event.window == *primary && event.y != 0.0 {
            model.first_row = if event.y > 0.0 {
                model.first_row.saturating_sub(1)
            } else {
                model.first_row + 1
            };
        }
    }
}
fn button(parent: &mut ChildSpawnerCommands, label: impl Into<String>, action: UiAction) {
    parent
        .spawn((
            Button,
            action,
            Node {
                padding: UiRect::axes(px(10), px(6)),
                ..default()
            },
            BackgroundColor(Color::srgb(0.16, 0.19, 0.23)),
        ))
        .with_child((
            Text::new(label),
            TextFont {
                font_size: FontSize::Px(14.),
                ..default()
            },
        ));
}
fn render(
    mut commands: Commands,
    mut model: ResMut<Model>,
    runtime: Res<Runtime>,
    camera: Res<PanelCamera>,
    mut icons: ResMut<Icons>,
    mut images: ResMut<Assets<Image>>,
    network: Res<crate::network_app::NetworkState>,
) {
    if !model.network_initialized {
        model.address.text = network.address.clone();
        model.address.cursor = model.address.text.len();
        model.network_initialized = true;
    }
    let remote = runtime.destination == Destination::Remote;
    let preferences = crate::preferences::Preferences {
        pack: runtime.pack_input.clone(),
        query: model.query.text.clone(),
        display: runtime.display,
    };
    if model.saved_preferences.as_ref() != Some(&preferences) {
        let _ = crate::preferences::save(&preferences);
        model.saved_preferences = Some(preferences);
    }
    let snapshot = if remote {
        network.snapshot.as_ref().unwrap_or(&runtime.snapshot)
    } else {
        &runtime.snapshot
    };
    let signature = format!(
        "{}|{}|{:?}|{}|{}|{}|{}|{:?}|{}|{}|{}|{}|{}|{}|{}",
        model.query.label(),
        model.pack.label(),
        model.focus,
        model.first_row,
        snapshot.sequence,
        runtime.status,
        runtime.catalog.revision,
        runtime.destination,
        runtime.enabled,
        network.status,
        network.connected,
        network.hosting,
        model.address.label(),
        model.secret.text.len(),
        network.remote_icons.len()
    );
    if signature == model.signature {
        return;
    }
    model.signature = signature;
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
                        metadata_path: Default::default(),
                        icon_path: Default::default(),
                        asset_path: Default::default(),
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
    let total = clips
        .iter()
        .filter(|c| matching(c, &model.query.text))
        .count();
    let max_row = total.div_ceil(COLUMNS).saturating_sub(ROWS);
    model.first_row = model.first_row.min(max_row);
    if let Some(root) = model.root.take() {
        commands.entity(root).despawn();
    }
    let root = commands
        .spawn((
            Node {
                width: percent(100),
                height: percent(100),
                padding: UiRect::all(px(14)),
                flex_direction: FlexDirection::Column,
                row_gap: px(8),
                overflow: Overflow::clip(),
                ..default()
            },
            BackgroundColor(Color::srgb(0.055, 0.065, 0.08)),
            UiTargetCamera(camera.0),
        ))
        .id();
    model.root = Some(root);
    commands.entity(root).with_children(|parent|{
        parent.spawn((Text::new(format!("DUROP | {} | {} | display {}",if runtime.destination==Destination::Local{"This Mac"}else{"Remote host"},if remote{if network.connected{"connected"}else{"offline"}}else if runtime.enabled{"host running"}else{"host stopped"},runtime.display.map_or("primary".into(),|index|index.to_string()))),TextFont{font_size:FontSize::Px(22.),..default()}));
        parent.spawn(Node{column_gap:px(6),flex_wrap:FlexWrap::Wrap,..default()}).with_children(|row|{for(label,action)in[("This Mac",Action::Destination(Destination::Local)),("Remote host",Action::Destination(Destination::Remote)),(if runtime.enabled{"Stop host"}else{"Start host"},Action::ToggleHost),("STOP ALL",Action::StopAll),("Display >",Action::NextDisplay),("Hide",Action::Hide),("Quit",Action::Quit)]{button(row,label,UiAction::Runtime(action));}});
        parent.spawn(Node{column_gap:px(6),..default()}).with_children(|row|{button(row,format!("Pack: {}",model.pack.label()),UiAction::Focus(Field::Pack));button(row,"Load / refresh",UiAction::Runtime(Action::LoadPack(model.pack.text.clone())));});
        button(parent,format!("Search title / tags / category: {}",model.query.label()),UiAction::Focus(Field::Search));
        parent.spawn(Node{column_gap:px(5),flex_wrap:FlexWrap::Wrap,..default()}).with_children(|row|{
            button(row,format!("Tailnet address: {}",model.address.label()),UiAction::Focus(Field::Address));
            button(row,format!("Pairing secret: {}", "*".repeat(model.secret.text.chars().count())),UiAction::Focus(Field::Secret));
            for (label,action) in [("Connect",crate::network_app::NetworkAction::Connect),("Disconnect",crate::network_app::NetworkAction::Disconnect),("Enable remote host",crate::network_app::NetworkAction::EnableHost),("Revoke remote",crate::network_app::NetworkAction::DisableHost)]{button(row,label,UiAction::Network(action));}
        });
        parent.spawn((Text::new(&network.status),TextFont{font_size:FontSize::Px(12.),..default()}));
        if let Some(secret)=&network.pairing_secret {parent.spawn((Text::new(format!("Pairing secret (private): {secret}")),TextFont{font_size:FontSize::Px(12.),..default()}));}
        parent.spawn((Text::new(&runtime.status),TextFont{font_size:FontSize::Px(13.),..default()},TextColor(Color::srgb(0.85,0.8,0.6))));
        parent.spawn(Node{display:Display::Grid,grid_template_columns:RepeatedGridTrack::fr(COLUMNS as u16,1.),column_gap:px(8),row_gap:px(8),..default()}).with_children(|grid|{for index in visible_indices(clips,&model.query.text,model.first_row){let clip=&clips[index];let readiness=snapshot.clips.iter().find(|c|c.clip_id==clip.identity).map(|c|format!("{:?}",c.readiness)).unwrap_or_else(||"Unchecked".into());let state=snapshot.instances.iter().rev().find(|i|i.clip_id==clip.identity).map(|i|format!("{:?}",i.state)).unwrap_or_else(||"Idle".into());let error=runtime.errors.get(&clip.identity).cloned().unwrap_or_default();
            let key=format!("{}:{}:{}:{}",remote,snapshot.catalog_revision,clip.identity,remote&&network.remote_icons.contains_key(&clip.identity));
            if !icons.0.contains_key(&key){
                let bytes=if remote{network.remote_icons.get(&clip.identity).cloned()}else{read_icon(&clip.icon_path).ok()};
                let handle=bytes.and_then(|bytes|icon_image(&bytes).ok()).map(|image|images.add(image));
                if icons.0.len()>=128{let old=icons.0.keys().next().cloned().unwrap();if let Some(Some(handle))=icons.0.remove(&old){images.remove(handle.id());}}
                icons.0.insert(key.clone(),handle);
            }
            let icon=icons.0.get(&key).cloned().flatten();
            grid.spawn((Button,UiAction::Runtime(Action::Play(clip.identity.clone())),Node{height:px(102),padding:UiRect::all(px(7)),flex_direction:FlexDirection::Column,row_gap:px(3),overflow:Overflow::clip(),..default()},BackgroundColor(Color::srgb(0.12,0.14,0.17)))).with_children(|tile|{if let Some(icon)=icon{tile.spawn((ImageNode::new(icon),Node{width:px(32),height:px(32),..default()}));}tile.spawn((Text::new(&clip.metadata.title),TextFont{font_size:FontSize::Px(15.),..default()}));tile.spawn((Text::new(format!("{} | {readiness} | {state}",std::path::Path::new(&clip.metadata.asset).extension().unwrap_or_default().to_string_lossy())),TextFont{font_size:FontSize::Px(11.),..default()}));if !error.is_empty(){tile.spawn((Text::new(error),TextFont{font_size:FontSize::Px(11.),..default()},TextColor(Color::srgb(1.,0.4,0.35))));}});
        }});
        parent.spawn(Node{column_gap:px(6),..default()}).with_children(|row|{button(row,"Previous",UiAction::Page(-1));button(row,"Next",UiAction::Page(1));row.spawn((Text::new(format!("{total} matches | row {} | mouse wheel scrolls",model.first_row+1)),TextFont{font_size:FontSize::Px(13.),..default()}));});
        parent.spawn(Node{column_gap:px(5),flex_wrap:FlexWrap::Wrap,..default()}).with_children(|row|{for instance in snapshot.instances.iter().filter(|i|!i.state.is_terminal()).take(8){button(row,format!("Stop {} | {:?}",instance.clip_id,instance.state),UiAction::Runtime(Action::Stop(instance.instance_id.clone())));}});
        parent.spawn((Text::new("Search: Unicode / IME, arrows, Backspace, Ctrl/Cmd+A. Esc stops all outside fields. Terminal: panel | stop | quit. Native fullscreen overlay is unsupported on this Mac."),TextFont{font_size:FontSize::Px(11.),..default()}));
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    fn config() -> crate::app::Config {
        crate::app::Config::parse(["--mode".into(), "controller".into()]).unwrap()
    }
    #[test]
    fn thumbnails_reject_large_payloads_and_oversized_ihdr_before_decode() {
        let mut bytes = vec![0; 24];
        bytes[..8].copy_from_slice(b"\x89PNG\r\n\x1a\n");
        bytes[12..16].copy_from_slice(b"IHDR");
        bytes[16..20].copy_from_slice(&100_000u32.to_be_bytes());
        bytes[20..24].copy_from_slice(&100_000u32.to_be_bytes());
        assert_eq!(
            icon_image(&bytes).unwrap_err(),
            "thumbnail dimensions exceed 512x512"
        );
        assert!(icon_image(&vec![0; MAX_ICON_BYTES + 1]).is_err());
        let icon =
            std::fs::read(crate::app::resource_root().join("sample-packs/podcast-pack/gif.png"))
                .unwrap();
        assert!(icon_image(&icon).is_ok());
    }
    fn input_app() -> (App, Entity) {
        let mut app = App::new();
        app.insert_resource(Runtime::new(&config()).unwrap())
            .init_resource::<Model>()
            .init_resource::<Actions>()
            .init_resource::<ButtonInput<KeyCode>>()
            .add_message::<KeyboardInput>()
            .add_message::<Ime>()
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
    #[test]
    fn ten_thousand_catalog_items_render_only_visible_rows_and_search_all_fields() {
        let catalog = crate::catalog::load_local(
            &crate::app::resource_root().join("sample-packs/podcast-pack"),
        )
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
            visible_indices(&clips, "sound music", 100).len(),
            COLUMNS * ROWS
        );
        assert_eq!(visible_indices(&clips, "sound music", 100)[0], 400);
        assert_eq!(visible_indices(&clips, "Title 9999", 0), vec![9999]);
        assert!(visible_indices(&clips, "absent", 0).is_empty());
        assert_eq!(visible_indices(&clips, "", usize::MAX).len(), 0);
    }
}
