//! Durop application assembly: one executable over independent domain libraries.

#![deny(missing_docs)]

mod app;

#[cfg(target_os = "macos")]
mod app_macos;

/// Validated local and Git catalog sources, independent of presentation.
pub extern crate durop_catalog as catalog;

/// Shared wire commands and host observations.
pub extern crate durop_protocol as commands;

mod diagnostics;

/// Serialized playback policy and bounded command ingress.
pub extern crate durop_host as host;

/// Framework-independent GIF and native media workers.
pub extern crate durop_media as media;

mod media_app;
mod network_app;
pub mod overlay;
mod panel;
mod preferences;

/// Authenticated Tailscale sessions and controller protocol.
pub extern crate durop_transport as transport;

use bevy::{prelude::*, window::ExitCondition};

/// Parse startup mode, assemble the application, and flush diagnostics on exit.
fn main() {
    if std::env::args().nth(1).as_deref() == Some("--version") {
        println!("Durop {}", env!("CARGO_PKG_VERSION"));
        return;
    }
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
            media::ProbeReport::cancel(std::path::Path::new(path))
        } else {
            media::ProbeReport::play(std::path::Path::new(path))
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
        match catalog::load_local(&app::Config::bundled_pack()) {
            Ok(catalog) => {
                println!(
                    "resources={} clips={} revision={}",
                    root.display(),
                    catalog.clips.len(),
                    catalog.revision
                );
                return;
            }
            Err(error) => {
                eprintln!("resource self-check failed: {error}");
                std::process::exit(1);
            }
        }
    }
    let config = match app::Config::parse(args) {
        Ok(config) => config,
        Err(message) => {
            eprintln!("{message}");
            std::process::exit(2);
        }
    };
    let diagnostic_guard = config
        .diagnostics
        .as_ref()
        .map(|path| diagnostics::start(path))
        .transpose()
        .unwrap_or_else(|error| {
            eprintln!("Cannot start protected diagnostics: {error}");
            std::process::exit(2);
        });
    let runtime = match app::Runtime::new(&config) {
        Ok(runtime) => runtime,
        Err(message) => {
            eprintln!("Pack load failed: {message}");
            std::process::exit(1);
        }
    };
    App::new()
        .insert_resource(runtime)
        .insert_resource(network_app::NetworkConfig {
            address: config.listen.clone().or_else(|| config.connect.clone()),
            enable_host: config.listen.is_some(),
        })
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Durop".into(),
                        resolution: (1160, 820).into(),
                        visible: config.mode != app::Mode::Host,
                        focused: config.mode != app::Mode::Host,
                        ..default()
                    }),
                    close_when_requested: false,
                    exit_condition: ExitCondition::DontExit,
                    ..default()
                })
                .set(AssetPlugin {
                    file_path: app::resource_root()
                        .join("assets")
                        .to_string_lossy()
                        .into_owned(),
                    ..default()
                }),
        )
        .add_plugins((
            overlay::OverlayPlugin,
            media_app::MediaPlugin,
            app::RuntimePlugin,
            network_app::NetworkPlugin,
            panel::PanelPlugin,
        ))
        .run();
    if let Some(guard) = diagnostic_guard
        && !guard.finish()
    {
        eprintln!("Diagnostics did not finish cleanly; treat the file as incomplete");
    }
}
