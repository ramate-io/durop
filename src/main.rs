pub mod catalog;
pub mod commands;
pub mod host;
pub mod media;
mod overlay;
mod panel;
pub mod transport;

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
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Durop — overlay proof".into(),
                resolution: (620, 300).into(),
                ..default()
            }),
            close_when_requested: false,
            exit_condition: ExitCondition::DontExit,
            ..default()
        }))
        .add_plugins((overlay::OverlayPlugin, panel::PanelPlugin))
        .run();
}
