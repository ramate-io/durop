//! Bounded catalog UI with an explicit Unicode and IME text editor.
use crate::{app::{Action,Actions,Destination,Mode,Runtime},catalog::Clip,overlay::PANEL_LAYER};
use bevy::{prelude::*,camera::{RenderTarget,visibility::RenderLayers},window::{PrimaryWindow,WindowRef,Ime},input::{ButtonState,keyboard::{KeyboardInput,Key},mouse::MouseWheel}};
use std::{collections::BTreeMap,io::{self,BufRead},sync::{Mutex,mpsc}};
const COLUMNS:usize=4;
const ROWS:usize=3;
#[derive(Debug,Clone,Copy,PartialEq,Eq)]enum Field{Search,Pack}
#[derive(Default,Debug,Clone)]struct Editor{text:String,cursor:usize,preedit:String,select_all:bool}
impl Editor{
    fn insert(&mut self,value:&str){
        if self.select_all{self.text.clear();self.cursor=0;self.select_all=false;}
        let value:String=value.chars().filter(|c|!c.is_control()).take(4096usize.saturating_sub(self.text.chars().count())).collect();
        self.text.insert_str(self.cursor,&value);self.cursor+=value.len();
    }
    fn backspace(&mut self){if self.select_all{self.text.clear();self.cursor=0;self.select_all=false;return;}if let Some((i,_))=self.text[..self.cursor].char_indices().next_back(){self.text.drain(i..self.cursor);self.cursor=i;}}
    fn left(&mut self){self.cursor=self.text[..self.cursor].char_indices().next_back().map_or(0,|(i,_)|i);self.select_all=false;}
    fn right(&mut self){self.cursor=self.text[self.cursor..].chars().next().map_or(self.cursor,|c|self.cursor+c.len_utf8());self.select_all=false;}
    fn label(&self)->String{format!("{}▏{}{}",&self.text[..self.cursor],self.preedit,&self.text[self.cursor..])}
}
#[derive(Resource,Default)]struct Model{query:Editor,pack:Editor,focus:Option<Field>,first_row:usize,signature:String,root:Option<Entity>}
#[derive(Component,Clone)]enum UiAction{Runtime(Action),Focus(Field),Page(i32)}
#[derive(Resource,Default)]struct Icons(BTreeMap<String,Handle<Image>>);
#[derive(Resource)]struct Terminal(Mutex<mpsc::Receiver<String>>);
#[derive(Resource)]struct PanelCamera(Entity);
pub struct PanelPlugin;
impl Plugin for PanelPlugin{fn build(&self,app:&mut App){app.init_resource::<Model>().init_resource::<Icons>().add_systems(Startup,setup).add_systems(Update,(buttons,text_input,scroll,terminal,render).chain().before(crate::overlay::sync_overlay));}}
fn setup(mut commands:Commands,primary:Single<Entity,With<PrimaryWindow>>,proxy:Res<bevy::winit::EventLoopProxyWrapper>,runtime:Res<Runtime>,mut model:ResMut<Model>){
    let camera=commands.spawn((Camera2d,RenderTarget::Window(WindowRef::Entity(*primary)),RenderLayers::layer(PANEL_LAYER))).id();commands.insert_resource(PanelCamera(camera));
    model.pack.text=runtime.pack_input.clone();model.pack.cursor=model.pack.text.len();
    let(tx,rx)=mpsc::sync_channel(32);commands.insert_resource(Terminal(Mutex::new(rx)));let proxy=(**proxy).clone();
    std::thread::spawn(move||{for line in io::stdin().lock().lines(){let Ok(line)=line else{break;};if tx.send(line).is_err(){return;}if proxy.send_event(bevy::winit::WinitUserEvent::WakeUp).is_err(){return;}}let _=tx.send("terminal-closed".into());let _=proxy.send_event(bevy::winit::WinitUserEvent::WakeUp);});
    eprintln!("Safety terminal: stop | panel | hide | play PACK/CLIP | quit");
}
fn terminal(terminal:Res<Terminal>,mut actions:ResMut<Actions>,mut controls:ResMut<crate::overlay::PendingControls>,runtime:Res<Runtime>){
    for line in terminal.0.lock().unwrap().try_iter().take(32){match line.trim(){
        "stop"=>actions.0.push_back(Action::StopAll),"panel"=>actions.0.push_back(Action::Show),"hide"=>actions.0.push_back(Action::Hide),"quit"=>actions.0.push_back(Action::Quit),
        "terminal-closed"=>if runtime.mode!=Mode::Host{controls.0.push(crate::overlay::Control::TerminalClosed);},
        "displays"=>controls.0.push(crate::overlay::Control::Displays),
        s if s.starts_with("play ")=>actions.0.push_back(Action::Play(s[5..].into())),
        s if s.starts_with("display ")=>if let Ok(i)=s[8..].parse(){controls.0.push(crate::overlay::Control::Display(i));},
        _=>eprintln!("Use stop | panel | hide | play PACK/CLIP | displays | display N | quit"),
    }}
}
fn matching(clip:&Clip,query:&str)->bool{query.to_lowercase().split_whitespace().all(|part|clip.metadata.title.to_lowercase().contains(part)||clip.category.to_string_lossy().to_lowercase().contains(part)||clip.metadata.tags.iter().any(|tag|tag.to_lowercase().contains(part)))}
fn visible_indices(clips:&[Clip],query:&str,row:usize)->Vec<usize>{clips.iter().enumerate().filter(|(_,c)|matching(c,query)).skip(row.saturating_mul(COLUMNS)).take(COLUMNS*ROWS).map(|(i,_)|i).collect()}
fn buttons(mut model:ResMut<Model>,buttons:Query<(&Interaction,&UiAction),Changed<Interaction>>,mut actions:ResMut<Actions>){for(interaction,action)in&buttons{if *interaction!=Interaction::Pressed{continue;}match action{UiAction::Runtime(a)=>{model.focus=None;actions.0.push_back(a.clone());},UiAction::Focus(field)=>model.focus=Some(*field),UiAction::Page(d)=>model.first_row=if *d<0{model.first_row.saturating_sub(ROWS)}else{model.first_row+ROWS},}}}
fn text_input(mut keys:MessageReader<KeyboardInput>,mut ime:MessageReader<Ime>,primary:Single<(Entity,&mut Window),With<PrimaryWindow>>,mut model:ResMut<Model>,runtime:Res<Runtime>,held:Res<ButtonInput<KeyCode>>,mut actions:ResMut<Actions>){
    let(entity,mut window)=primary.into_inner();window.ime_enabled=model.focus.is_some();if !window.focused{keys.clear();ime.clear();return;}
    let mut committed=false;
    for event in ime.read(){let Some(field)=model.focus else{continue;};let edit=match field{Field::Search=>&mut model.query,Field::Pack=>&mut model.pack};match event{
        Ime::Preedit{window,value,..}if *window==entity=>edit.preedit=value.clone(),Ime::Commit{window,value}if *window==entity=>{edit.preedit.clear();edit.insert(value);committed=true;},Ime::Disabled{window}if *window==entity=>edit.preedit.clear(),_=>(),}}
    for key in keys.read(){if key.window!=entity||key.state!=ButtonState::Pressed{continue;}
        if let Some(field)=model.focus{
            let modified=held.pressed(KeyCode::SuperLeft)||held.pressed(KeyCode::SuperRight)||held.pressed(KeyCode::ControlLeft)||held.pressed(KeyCode::ControlRight);
            let edit=match field{Field::Search=>&mut model.query,Field::Pack=>&mut model.pack};
            if key.logical_key==Key::Escape{edit.preedit.clear();model.focus=None;continue;}if !edit.preedit.is_empty(){continue;}
            match key.logical_key{Key::Backspace=>edit.backspace(),Key::ArrowLeft=>edit.left(),Key::ArrowRight=>edit.right(),Key::Home=>edit.cursor=0,Key::End=>edit.cursor=edit.text.len(),Key::Enter=>{if field==Field::Pack{actions.0.push_back(Action::LoadPack(edit.text.clone()));}model.focus=None;},_ if modified&&key.key_code==KeyCode::KeyA=>edit.select_all=true,_ if !modified&&!committed=>if let Some(text)=&key.text{edit.insert(text);},_=>(),}model.first_row=0;
        }else if !key.repeat{match key.logical_key{Key::Escape=>actions.0.push_back(Action::StopAll),Key::Space=>if let Some(index)=visible_indices(&runtime.catalog.clips,&model.query.text,model.first_row).first(){actions.0.push_back(Action::Play(runtime.catalog.clips[*index].identity.clone()));},_=>(),}}
    }
}
fn scroll(mut wheel:MessageReader<MouseWheel>,primary:Single<Entity,With<PrimaryWindow>>,mut model:ResMut<Model>){for event in wheel.read(){if event.window==*primary&&event.y!=0.0{model.first_row=if event.y>0.0{model.first_row.saturating_sub(1)}else{model.first_row+1};}}}
fn button(parent:&mut ChildSpawnerCommands,label:impl Into<String>,action:UiAction){parent.spawn((Button,action,Node{padding:UiRect::axes(px(10),px(6)),..default()},BackgroundColor(Color::srgb(0.16,0.19,0.23)))).with_child((Text::new(label),TextFont{font_size:FontSize::Px(14.),..default()}));}
fn render(mut commands:Commands,mut model:ResMut<Model>,runtime:Res<Runtime>,camera:Res<PanelCamera>,mut icons:ResMut<Icons>,mut images:ResMut<Assets<Image>>){
    if runtime.mode==Mode::Host{return;}let snapshot=runtime.host.snapshot();let total=runtime.catalog.clips.iter().filter(|c|matching(c,&model.query.text)).count();let max_row=total.div_ceil(COLUMNS).saturating_sub(ROWS);model.first_row=model.first_row.min(max_row);
    let signature=format!("{}|{}|{:?}|{}|{}|{}|{}|{:?}|{}",model.query.label(),model.pack.label(),model.focus,model.first_row,snapshot.sequence,runtime.status,runtime.catalog.revision,runtime.destination,runtime.enabled);if signature==model.signature{return;}model.signature=signature;
    if let Some(root)=model.root.take(){commands.entity(root).despawn();}
    let root=commands.spawn((Node{width:percent(100),height:percent(100),padding:UiRect::all(px(14)),flex_direction:FlexDirection::Column,row_gap:px(8),overflow:Overflow::clip(),..default()},BackgroundColor(Color::srgb(0.055,0.065,0.08)),UiTargetCamera(camera.0))).id();model.root=Some(root);
    commands.entity(root).with_children(|parent|{
        parent.spawn((Text::new(format!("DUROP · {} · {}",if runtime.destination==Destination::Local{"This Mac"}else{"Remote host"},if runtime.enabled{"host running"}else{"host stopped"})),TextFont{font_size:FontSize::Px(22.),..default()}));
        parent.spawn(Node{column_gap:px(6),flex_wrap:FlexWrap::Wrap,..default()}).with_children(|row|{for(label,action)in[("This Mac",Action::Destination(Destination::Local)),("Remote host",Action::Destination(Destination::Remote)),(if runtime.enabled{"Stop host"}else{"Start host"},Action::ToggleHost),("STOP ALL",Action::StopAll),("Display →",Action::NextDisplay),("Hide",Action::Hide),("Quit",Action::Quit)]{button(row,label,UiAction::Runtime(action));}});
        parent.spawn(Node{column_gap:px(6),..default()}).with_children(|row|{button(row,format!("Pack: {}",model.pack.label()),UiAction::Focus(Field::Pack));button(row,"Load / refresh",UiAction::Runtime(Action::LoadPack(model.pack.text.clone())));});
        button(parent,format!("Search title / tags / category: {}",model.query.label()),UiAction::Focus(Field::Search));
        parent.spawn((Text::new(&runtime.status),TextFont{font_size:FontSize::Px(13.),..default()},TextColor(Color::srgb(0.85,0.8,0.6))));
        parent.spawn(Node{display:Display::Grid,grid_template_columns:RepeatedGridTrack::fr(COLUMNS as u16,1.),column_gap:px(8),row_gap:px(8),..default()}).with_children(|grid|{for index in visible_indices(&runtime.catalog.clips,&model.query.text,model.first_row){let clip=&runtime.catalog.clips[index];let readiness=snapshot.clips.iter().find(|c|c.clip_id==clip.identity).map(|c|format!("{:?}",c.readiness)).unwrap_or_else(||"Unchecked".into());let state=snapshot.instances.iter().rev().find(|i|i.clip_id==clip.identity).map(|i|format!("{:?}",i.state)).unwrap_or_else(||"Idle".into());let error=runtime.errors.get(&clip.identity).cloned().unwrap_or_default();
            let key=format!("{}:{}",runtime.catalog.revision,clip.identity);
            if !icons.0.contains_key(&key)&&let Ok(bytes)=std::fs::read(&clip.icon_path)&&let Ok(image)=Image::from_buffer(&bytes,bevy::image::ImageType::Extension("png"),bevy::image::CompressedImageFormats::NONE,true,bevy::image::ImageSampler::nearest(),bevy::asset::RenderAssetUsages::default()){
                if icons.0.len()>=128{let old=icons.0.keys().next().cloned().unwrap();if let Some(handle)=icons.0.remove(&old){images.remove(handle.id());}}icons.0.insert(key.clone(),images.add(image));}
            let icon=icons.0.get(&key).cloned();
            grid.spawn((Button,UiAction::Runtime(Action::Play(clip.identity.clone())),Node{height:px(112),padding:UiRect::all(px(7)),flex_direction:FlexDirection::Column,row_gap:px(3),overflow:Overflow::clip(),..default()},BackgroundColor(Color::srgb(0.12,0.14,0.17)))).with_children(|tile|{if let Some(icon)=icon{tile.spawn((ImageNode::new(icon),Node{width:px(40),height:px(40),..default()}));}tile.spawn((Text::new(&clip.metadata.title),TextFont{font_size:FontSize::Px(15.),..default()}));tile.spawn((Text::new(format!("{} · {readiness} · {state}",clip.asset_path.extension().unwrap_or_default().to_string_lossy())),TextFont{font_size:FontSize::Px(11.),..default()}));if !error.is_empty(){tile.spawn((Text::new(error),TextFont{font_size:FontSize::Px(11.),..default()},TextColor(Color::srgb(1.,0.4,0.35))));}});
        }});
        parent.spawn(Node{column_gap:px(6),..default()}).with_children(|row|{button(row,"↑ Previous",UiAction::Page(-1));button(row,"↓ Next",UiAction::Page(1));row.spawn((Text::new(format!("{total} matches · row {} · mouse wheel scrolls",model.first_row+1)),TextFont{font_size:FontSize::Px(13.),..default()}));});
        parent.spawn(Node{column_gap:px(5),flex_wrap:FlexWrap::Wrap,..default()}).with_children(|row|{for instance in snapshot.instances.iter().filter(|i|!i.state.is_terminal()).take(8){button(row,format!("Stop {} · {:?}",instance.clip_id,instance.state),UiAction::Runtime(Action::Stop(instance.instance_id.clone())));}});
        parent.spawn((Text::new("Search: Unicode / IME, arrows, Backspace, Ctrl/Cmd+A. Esc stops all outside fields. Terminal: panel | stop | quit. Native fullscreen overlay is unsupported on this Mac."),TextFont{font_size:FontSize::Px(11.),..default()}));
    });
}
