//! Warm paper, ink and restrained accent colors for the native controller.

use bevy::prelude::*;

/// Background paper surrounding cards and provider controls.
pub(super) const BACKGROUND: Color = Color::srgb(0.957, 0.933, 0.886);

/// Lighter paper used for cards and the configuration surface.
pub(super) const PAPER: Color = Color::srgb(0.984, 0.969, 0.925);

/// Primary text and high-contrast controls.
pub(super) const INK: Color = Color::srgb(0.102, 0.102, 0.102);

/// Secondary editorial labels and explanatory text.
pub(super) const MUTED: Color = Color::srgb(0.365, 0.341, 0.290);

/// Fine borders separating paper surfaces.
pub(super) const RULE: Color = Color::srgb(0.733, 0.714, 0.671);

/// Selected provider, focus, and playback accents.
pub(super) const PURPLE: Color = Color::srgb(0.431, 0.345, 0.514);

/// Confirmed connectivity and local readiness.
pub(super) const GREEN: Color = Color::srgb(0.239, 0.400, 0.322);

/// Warm hover accent without adding another dominant color.
pub(super) const YELLOW: Color = Color::srgb(0.925, 0.765, 0.408);

/// Bundled open-license display and monospaced interface font handles.
#[derive(Resource, Default)]
pub(super) struct Fonts {
    /// Expressive display face used sparingly for titles.
    pub display: Handle<Font>,

    /// Readable mono face for controls, metadata and fields.
    pub mono: Handle<Font>,
}

impl Fonts {
    /// Resolve fonts through the same resource root as packaged application assets.
    pub fn load(assets: &AssetServer) -> Self {
        Self {
            display: assets.load("fonts/ExpletusSans-Regular.ttf"),
            mono: assets.load("fonts/IBMPlexMono-Regular.ttf"),
        }
    }

    /// Construct ordinary interface typography at the requested logical size.
    pub fn mono(&self, size: f32) -> TextFont {
        TextFont {
            font: FontSource::Handle(self.mono.clone()),
            font_size: FontSize::Px(size),
            ..default()
        }
    }

    /// Construct a display title with the bundled editorial face.
    pub fn display(&self, size: f32) -> TextFont {
        TextFont {
            font: FontSource::Handle(self.display.clone()),
            font_size: FontSize::Px(size),
            ..default()
        }
    }
}
