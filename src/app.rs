//! Application modes and the real host/media bridge. No decoder work runs in UI code.
use crate::{catalog::{self, Catalog, CatalogCache, Clip, PackSource, Anchor, Fit}, commands::{Envelope, PlaybackCommand, CommandOutcome, InstanceState, PROTOCOL_VERSION}, host::{Host, BackendEffect, BackendEvent, BackendEventKind, BackendToken}, media::{MediaWorker, MediaSpec, MediaEventKind}, overlay::{OverlayWindow, PlaybackOverlay, OVERLAY_LAYER}};
use bevy::{prelude::*, asset::RenderAssetUsages, camera::visibility::RenderLayers, render::render_resource::{Extent3d, TextureDimension, TextureFormat}, window::PrimaryWindow};
use serde::{Serialize, Deserialize};
use std::{collections::{BTreeMap, VecDeque}, path::PathBuf, sync::{Arc, Mutex, mpsc}, time::Duration};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode { Local, Host, Controller }
#[derive(Debug, Clone)]
pub struct Config { pub mode: Mode, pub pack: PathBuf, pub display: Option<usize>, pub connect: Option<String> }
impl Config {
    pub fn parse(args: impl IntoIterator<Item=String>) -> Result<Self,String> {
        let mut config=Self{mode:Mode::Local,pack:"examples/podcast-pack".into(),display:None,connect:None};
        let mut args=args.into_iter();
        while let Some(arg)=args.next() {
            let mut value=|| args.next().ok_or_else(||format!("{arg} needs a value"));
            match arg.as_str() {
                "--mode" => config.mode=match value()?.as_str(){"local"=>Mode::Local,"host"=>Mode::Host,"controller"=>Mode::Controller,_=>return Err("mode must be local, host or controller".into())},
                "--pack"=>config.pack=value()?.into(),
                "--display"=>config.display=Some(value()?.parse().map_err(|_|"display must be a nonnegative index")?),
                "--connect"=>config.connect=Some(value()?),
                "--help"|"-h"=>return Err("durop [--mode local|host|controller] [--pack DIRECTORY] [--display INDEX] [--connect ws://HOST:48732/control]".into()),
                _=>return Err(format!("unknown option {arg}")),
            }
        }
        Ok(config)
    }
}
#[derive(Debug,Clone,Copy,PartialEq,Eq)]
pub enum Destination { Local, Remote }
#[derive(Debug,Clone)]
pub enum Action { Play(String), Stop(String), StopAll, ToggleHost, Hide, Show, Quit, LoadPack(String), Destination(Destination), NextDisplay }
#[derive(Resource,Default)]
pub struct Actions(pub VecDeque<Action>);
struct Playback {
    token: BackendToken, worker: MediaWorker, clip: Clip,
    image: Option<Handle<Image>>, sprite: Option<Entity>,
}
#[derive(Resource)]
pub struct Runtime {
    pub host: Host,
    pub catalog: Arc<Catalog>,
    pub mode: Mode,
    pub destination: Destination,
    pub enabled: bool,
    pub status: String,
    pub errors: BTreeMap<String,String>,
    pub pack_input: String,
    pub loading: bool,
    workers: BTreeMap<String,Playback>,
    next_command: u64,
    quitting: bool,
    load_rx: Mutex<mpsc::Receiver<Result<Arc<Catalog>,String>>>,
    load_tx: mpsc::Sender<Result<Arc<Catalog>,String>>,
}
fn private_root() -> PathBuf {
    std::env::var_os("XDG_STATE_HOME").map(PathBuf::from).unwrap_or_else(||PathBuf::from(std::env::var_os("HOME").unwrap_or_else(||".".into())).join(".local/state")).join("durop")
}
fn session_id() -> String {
    // UUID randomness comes from the OS, never from controller input.
    bevy::asset::uuid::Uuid::new_v4().to_string()
}
impl Runtime {
    pub fn new(config: &Config) -> Result<Self,String> {
        let cache=CatalogCache::new(private_root().join("packs")).map_err(|e|e.to_string())?;
        let catalog=cache.prepare(&PackSource::LocalDirectory(config.pack.clone())).map_err(|e|e.to_string())?;
        let (load_tx,load_rx)=mpsc::channel();
        Ok(Self {host:Host::new(session_id(),catalog.clone()),catalog,mode:config.mode,destination:if config.mode==Mode::Controller {Destination::Remote}else{Destination::Local},enabled:config.mode!=Mode::Controller,status:if config.mode==Mode::Controller{"Remote host unavailable: connect through transport when configured".into()}else{"This Mac · host running".into()},errors:BTreeMap::new(),pack_input:config.pack.to_string_lossy().into_owned(),loading:false,workers:BTreeMap::new(),next_command:0,quitting:false,load_rx:Mutex::new(load_rx),load_tx})
    }
    fn submit(&mut self, command: PlaybackCommand) {
        self.next_command+=1;
        let envelope=Envelope{protocol_version:PROTOCOL_VERSION,session_id:self.host.snapshot().session_id,command_id:format!("local-{}",self.next_command),catalog_revision:self.catalog.revision.clone(),command};
        if let CommandOutcome::Rejected{reason}=self.host.accept(envelope) { self.status=format!("Rejected: {reason:?}"); }
    }
    pub fn busy(&self)->bool { !self.workers.is_empty() || self.loading || self.quitting }
}

pub struct RuntimePlugin;
impl Plugin for RuntimePlugin {
    fn build(&self,app:&mut App) {
        app.init_resource::<Actions>().add_systems(Update,(actions,media_bridge).chain().before(crate::overlay::sync_overlay));
    }
}
fn actions(mut runtime:ResMut<Runtime>,mut actions:ResMut<Actions>,mut controls:ResMut<crate::overlay::PendingControls>,mut primary:Single<&mut Window,(With<PrimaryWindow>,Without<OverlayWindow>)>,proxy:Res<bevy::winit::EventLoopProxyWrapper>) {
    for action in actions.0.drain(..) {
        match action {
            Action::Play(clip_id)=>{
                if runtime.destination!=Destination::Local {runtime.status="Remote host unavailable; no command sent".into();}
                else if !runtime.enabled {runtime.status="Start the local host before playing".into();}
                else {runtime.submit(PlaybackCommand::Play{clip_id});}
            }
            Action::Stop(instance_id)=>runtime.submit(PlaybackCommand::Stop{instance_id}),
            Action::StopAll=>runtime.submit(PlaybackCommand::StopAll),
            Action::ToggleHost=>{
                if runtime.mode==Mode::Controller {runtime.status="Controller mode cannot start a local host".into();continue;}
                runtime.enabled=!runtime.enabled;
                if !runtime.enabled {runtime.submit(PlaybackCommand::StopAll);}
                runtime.status=if runtime.enabled {"This Mac · host running"}else{"This Mac · host stopped"}.into();
            }
            Action::Hide=>primary.visible=false,
            Action::Show=>{primary.visible=true;primary.focused=true;},
            Action::Quit=>{runtime.quitting=true;runtime.enabled=false;runtime.submit(PlaybackCommand::StopAll);},
            Action::Destination(destination)=>{runtime.destination=destination;runtime.status=if destination==Destination::Local {"Destination: This Mac"}else{"Destination: Remote host · unavailable; no commands sent"}.into();},
            Action::NextDisplay=>controls.0.push(crate::overlay::Control::NextDisplay),
            Action::LoadPack(path)=>{
                if runtime.loading {runtime.status="Pack load already in progress".into();continue;}
                runtime.submit(PlaybackCommand::StopAll);runtime.enabled=false;runtime.loading=true;runtime.pack_input=path.clone();
                let tx=runtime.load_tx.clone();let proxy=(**proxy).clone();
                std::thread::spawn(move||{
                    let result=CatalogCache::new(private_root().join("packs")).and_then(|cache|cache.prepare(&PackSource::LocalDirectory(path.into()))).map_err(|e|e.to_string());
                    let _=tx.send(result);let _=proxy.send_event(bevy::winit::WinitUserEvent::WakeUp);
                });
            }
        }
    }
    // Publish a replacement only after the old host has released every worker.
    if runtime.loading && runtime.workers.is_empty() {
        let result=runtime.load_rx.lock().unwrap().try_recv().ok();
        if let Some(result)=result {
            runtime.loading=false;
            match result {
                Ok(catalog)=>{runtime.host=Host::new(session_id(),catalog.clone());runtime.catalog=catalog;runtime.errors.clear();runtime.enabled=runtime.mode!=Mode::Controller;runtime.status="Pack loaded; catalog pinned for this session".into();},
                Err(message)=>{runtime.status=format!("Pack load failed: {message}");},
            }
        }
    }
}

fn media_bridge(mut commands:Commands,mut runtime:ResMut<Runtime>,mut images:ResMut<Assets<Image>>,mut sprites:Query<(&mut Sprite,&mut Transform)>,window:Single<&Window,With<OverlayWindow>>,mut overlay:ResMut<PlaybackOverlay>,mut exit:MessageWriter<AppExit>) {
    let mut callbacks=Vec::new();
    let mut remove=Vec::new();
    for (id,playback) in &mut runtime.workers {
        for event in playback.worker.drain_events() {
            if event.generation!=playback.token.generation {continue;}
            let kind=match event.kind {
                MediaEventKind::Prepared{..}=>BackendEventKind::Prepared,
                MediaEventKind::Started=>BackendEventKind::Started,
                MediaEventKind::Finished=>{remove.push(id.clone());BackendEventKind::Finished},
                MediaEventKind::Stopped=>{remove.push(id.clone());BackendEventKind::Stopped},
                MediaEventKind::Failed{message}=>{remove.push(id.clone());BackendEventKind::Failed{message}},
            };
            callbacks.push(BackendEvent{token:playback.token.clone(),event:kind});
        }
        if let Some(frame)=playback.worker.latest_frame() {
            if frame.generation!=playback.token.generation {continue;}
            let image=frame_image(&frame);
            if let Some(handle)=&playback.image {
                if let Some(existing)=images.get_mut(handle) { *existing=image; }
            } else {
                let handle=images.add(image);
                playback.sprite=Some(commands.spawn((Sprite::from_image(handle.clone()),RenderLayers::layer(OVERLAY_LAYER),Transform::default())).id());
                playback.image=Some(handle);
            }
            if let Some(entity)=playback.sprite && let Ok((mut sprite,mut transform))=sprites.get_mut(entity) {
                let (size,position)=placement(&playback.clip,Vec2::new(frame.width as f32,frame.height as f32),Vec2::new(window.width(),window.height()));
                sprite.custom_size=Some(size);*transform=Transform::from_xyz(position.x,position.y,0.0);
            }
        }
    }
    for callback in callbacks {runtime.host.backend_event(callback);}
    for id in remove { if let Some(playback)=runtime.workers.remove(&id) { if let Some(entity)=playback.sprite {commands.entity(entity).despawn();} if let Some(image)=playback.image {images.remove(image.id());} } }
    // Bounded policy effects; native decoder control remains in MediaPlugin.
    for _ in 0..3 {
        let effects=runtime.host.drain_effects(); if effects.is_empty(){break;}
        for effect in effects {
            match effect {
                BackendEffect::Prepare{token,clip}=>{
                    let worker=MediaWorker::spawn();
                    let result=worker.prepare(MediaSpec{path:clip.asset_path.clone(),generation:token.generation,max_duration:Duration::from_millis(clip.metadata.playback.max_duration_ms),looping:clip.metadata.playback.looping,volume:1.0});
                    match result {
                        Ok(())=>{runtime.workers.insert(token.instance_id.clone(),Playback{token,worker,clip:*clip,image:None,sprite:None});},
                        Err(message)=>runtime.host.backend_event(BackendEvent{token,event:BackendEventKind::Failed{message}}),
                    }
                }
                BackendEffect::Start{token}=>{if let Some(playback)=runtime.workers.get(&token.instance_id) && let Err(message)=playback.worker.start(token.generation) {playback.worker.stop(token.generation);runtime.host.backend_event(BackendEvent{token,event:BackendEventKind::Failed{message}});}},
                BackendEffect::Stop{token}=>{
                    if let Some(playback)=runtime.workers.get(&token.instance_id) {playback.worker.stop(token.generation);}
                    else {runtime.host.backend_event(BackendEvent{token,event:BackendEventKind::Stopped});}
                }
            }
        }
    }
    for event in runtime.host.drain_events() {
        if let crate::commands::EventKind::Failed{message}=event.event {
            if let Some(instance)=runtime.host.snapshot().instances.iter().find(|i|Some(&i.instance_id)==event.instance_id.as_ref()) {runtime.errors.insert(instance.clip_id.clone(),message.clone());}
            runtime.status=format!("Playback failed: {message}");
        }
    }
    overlay.visible=runtime.workers.values().any(|p|p.sprite.is_some());
    overlay.busy=runtime.busy() || crate::media::needs_updates();
    if runtime.quitting && runtime.workers.is_empty() {exit.write(AppExit::Success);}
}
fn frame_image(frame:&crate::media::DecodedFrame)->Image {
    // Decoder contract is tightly packed straight-alpha sRGB RGBA. A handle and
    // texture identity are reused per occurrence, with one latest frame retained.
    Image::new(Extent3d{width:frame.width,height:frame.height,depth_or_array_layers:1},TextureDimension::D2,frame.rgba.to_vec(),TextureFormat::Rgba8UnormSrgb,RenderAssetUsages::default())
}
pub fn placement(clip:&Clip,frame:Vec2,viewport:Vec2)->(Vec2,Vec2) {
    let fraction=clip.metadata.visual.as_ref().map_or(0.3,|v|v.width_fraction as f32);
    let target=Vec2::new(viewport.x*fraction,viewport.y*fraction);
    let fit=clip.metadata.visual.as_ref().map(|v|&v.fit);
    let ratios=target/frame;
    let scale=if matches!(fit,Some(Fit::Cover)){ratios.max_element()}else{ratios.min_element()};
    let size=frame*scale;
    let offset=(viewport-size)*0.5;
    let position=match clip.metadata.visual.as_ref().map(|v|&v.anchor) {
        Some(Anchor::TopLeft)=>Vec2::new(-offset.x,offset.y),Some(Anchor::TopRight)=>offset,
        Some(Anchor::BottomLeft)=>-offset,Some(Anchor::BottomRight)=>Vec2::new(offset.x,-offset.y),_=>Vec2::ZERO,
    };
    (size,position)
}
