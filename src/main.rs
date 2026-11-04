pub mod catalog;
mod commands;
mod host;
mod media;
mod overlay;
mod panel;
mod transport;

use bevy::prelude::*;
use bevy::window::WindowPlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Durop".into(),
                ..default()
            }),
            ..default()
        }))
        .run();
}
