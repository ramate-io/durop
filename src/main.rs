pub mod catalog;
mod commands;
mod host;
mod media;
mod overlay;
mod panel;
mod transport;

use bevy::{prelude::*, window::ExitCondition};

fn main() {
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
