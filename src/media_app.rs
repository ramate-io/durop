//! Bevy scheduling adapter for the framework-independent media service.

use bevy::prelude::*;
use durop_media::MediaService;

/// Install the native main-thread owner and poll it during application updates.
pub struct MediaPlugin;

impl Plugin for MediaPlugin {
    fn build(&self, app: &mut App) {
        app.insert_non_send(MediaService::new());
        app.add_systems(Update, Self::update);
    }
}

impl MediaPlugin {
    /// Keep native control objects on the main thread while workers decode frames.
    fn update(mut service: NonSendMut<MediaService>) {
        service.update();
    }
}
