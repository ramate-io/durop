//! Deliberately small local proof controls, replaced by the production panel later.
use crate::overlay::{Control, Controls, PANEL_LAYER};
use bevy::{
    camera::{RenderTarget, visibility::RenderLayers},
    prelude::*,
    window::{PrimaryWindow, WindowRef},
    winit::{EventLoopProxyWrapper, WinitUserEvent},
};
use std::{
    io::{self, BufRead},
    sync::{Mutex, mpsc},
};

pub struct PanelPlugin;
impl Plugin for PanelPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup)
            .add_systems(Update, keyboard.before(crate::overlay::apply_controls));
    }
}

fn setup(
    mut commands: Commands,
    primary: Single<Entity, With<PrimaryWindow>>,
    proxy: Res<EventLoopProxyWrapper>,
) {
    commands.spawn((
        Camera2d,
        RenderTarget::Window(WindowRef::Entity(*primary)),
        RenderLayers::layer(PANEL_LAYER),
    ));
    commands.spawn((
        Text2d::new("DUROP / OVERLAY PROOF\n\nSpace: animated PNG   Esc: stop\nD: next display   H: hide panel   Q: quit\n\nClose: stop and hide (terminal must remain connected).\nTerminal: show | hold | stop | panel | hide\n          displays | display N | quit"),
        TextFont { font_size: FontSize::Px(16.0), ..default() },
        RenderLayers::layer(PANEL_LAYER),
    ));
    let (sender, receiver) = mpsc::sync_channel(32);
    commands.insert_resource(Controls(Mutex::new(receiver)));
    let proxy = (**proxy).clone();
    std::thread::spawn(move || {
        for line in io::stdin().lock().lines() {
            let Ok(line) = line else { break };
            match Control::parse(&line) {
                Some(control) => {
                    if sender.send(control).is_err() {
                        break;
                    }
                    if proxy.send_event(WinitUserEvent::WakeUp).is_err() {
                        break;
                    }
                }
                None => eprintln!(
                    "Commands: show | hold | stop | panel | hide | displays | display N | quit"
                ),
            }
        }
        let _ = sender.send(Control::TerminalClosed);
        let _ = proxy.send_event(WinitUserEvent::WakeUp);
    });
    eprintln!(
        "Local safety controls: type stop, panel, or quit in this terminal even when the panel is hidden."
    );
}

fn keyboard(keys: Res<ButtonInput<KeyCode>>, mut actions: ResMut<crate::overlay::PendingControls>) {
    for (key, control) in [
        (KeyCode::Space, Control::Show),
        (KeyCode::Escape, Control::Stop),
        (KeyCode::KeyD, Control::NextDisplay),
        (KeyCode::KeyH, Control::HidePanel),
        (KeyCode::KeyQ, Control::Quit),
    ] {
        if keys.just_pressed(key) {
            actions.0.push(control);
        }
    }
}
