pub mod catalog;
pub mod commands;
pub mod host;
pub mod media;
mod overlay;
mod panel;
pub mod transport;
mod app;

use bevy::{prelude::*, window::ExitCondition};

fn main() {
    let arguments: Vec<_> = std::env::args_os().collect();
    if arguments
        .get(1)
        .is_some_and(|argument| argument == "--media-probe")
    {
        let Some(path) = arguments.get(2) else {
            eprintln!("usage: durop --media-probe PATH [--cancel]");
            std::process::exit(2);
        };
        let result = if arguments
            .get(3)
            .is_some_and(|argument| argument == "--cancel")
        {
            media::probe_cancel(std::path::Path::new(path))
        } else {
            media::probe(std::path::Path::new(path))
        };
        match result {
            Ok(report) => println!("{report:#?}"),
            Err(message) => {
                eprintln!("media probe failed: {message}");
                std::process::exit(1);
            }
        }
        return;
    }
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args == ["--self-check"] {
        let root = app::resource_root();
        match catalog::load_local(&root.join("sample-packs/podcast-pack")) {
            Ok(catalog) => { println!("resources={} clips={} revision={}", root.display(), catalog.clips.len(), catalog.revision); return; }
            Err(error) => { eprintln!("resource self-check failed: {error}"); std::process::exit(1); }
        }
    }
    let config=match app::Config::parse(args){Ok(config)=>config,Err(message)=>{eprintln!("{message}");std::process::exit(2);}};
    let runtime=match app::Runtime::new(&config){Ok(runtime)=>runtime,Err(message)=>{eprintln!("Pack load failed: {message}");std::process::exit(1);}};
    App::new()
        .insert_resource(runtime)
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Durop".into(),
                resolution: (1040, 800).into(),
                visible: config.mode != app::Mode::Host,
                focused: config.mode != app::Mode::Host,
                ..default()
            }),
            close_when_requested: false,
            exit_condition: ExitCondition::DontExit,
            ..default()
        }).set(AssetPlugin{file_path:app::resource_root().join("assets").to_string_lossy().into_owned(),..default()}))
        .add_plugins((overlay::OverlayPlugin, media::MediaPlugin, app::RuntimePlugin, panel::PanelPlugin))
        .run();
}
