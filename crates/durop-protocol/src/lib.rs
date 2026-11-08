//! Shared local/remote protocol. Acceptance is distinct from backend playback.
#![deny(missing_docs)]

use serde::{Deserialize, Serialize};

/// Wire schema version accepted by the host and every controller.
pub const PROTOCOL_VERSION: u32 = 2;

/// Authenticated-session command envelope; contains identifiers, never native paths.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Envelope {
    /// Wire schema version used to decode this command.
    pub protocol_version: u32,
    /// Fresh host session identity; prevents replay across host lifetimes.
    pub session_id: String,
    /// Controller-generated deduplication key within the session.
    pub command_id: String,
    /// Immutable catalog revision the controller used to choose this command.
    pub catalog_revision: String,
    /// Requested policy operation; playback acceptance remains host-owned.
    pub command: PlaybackCommand,
}

/// Fitting policy for an occurrence-only visual placement override.
/// Catalog delegates to immutable clip metadata; Cover explicitly fills the fitting area
/// with centered cropping and never stretches the source image.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VisualFit {
    /// Retain the catalog's contain/cover policy, including the default contain behavior.
    #[default]
    Catalog,
    /// Fill the placement area with aspect-preserving centered cropping.
    Cover,
}

/// Display-relative visual fit box and position, expressed in basis points.
/// Width is a fraction of the host display width. Positions are fractions of the
/// available horizontal/vertical travel, keeping the fitted visual on screen.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct VisualPlacement {
    /// Fit-box width: 1 through 10,000, where 10,000 is the full display width.
    pub width_bps: u16,
    /// Horizontal travel: 0 is the left edge and 10,000 is the right edge.
    pub x_bps: u16,
    /// Vertical travel: 0 is the top edge and 10,000 is the bottom edge.
    pub y_bps: u16,
    /// Explicit fitting intent; omitted fields from earlier version-2 payloads use catalog policy.
    #[serde(default)]
    pub fit: VisualFit,
}

impl VisualPlacement {
    /// Reject zero width and values exceeding one full display or travel range.
    pub fn is_valid(self) -> bool {
        (1..=10_000).contains(&self.width_bps) && self.x_bps <= 10_000 && self.y_bps <= 10_000
    }
}

/// Portable controller intentions, independent of transport and rendering.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum PlaybackCommand {
    /// Request playback of a catalog clip by its stable identity.
    Play {
        /// Stable catalog identity to play.
        clip_id: String,
    },
    /// Play a visual clip with an explicit occurrence-only placement override.
    PlayVisual {
        /// Stable catalog identity to play.
        clip_id: String,
        /// Valid display-relative override; audio clips reject this command.
        placement: VisualPlacement,
    },
    /// Update an active visual occurrence without restarting its decoder.
    SetVisual {
        /// Preparing or playing occurrence whose placement will change.
        instance_id: String,
        /// Explicit override, or none to restore the catalog placement.
        placement: Option<VisualPlacement>,
    },
    /// Stop the named occurrence; repeated stops are idempotent.
    Stop {
        /// Playback occurrence to cancel.
        instance_id: String,
    },
    /// Stop every occurrence and supersede older queued play requests.
    StopAll,
}

impl<'de> Deserialize<'de> for PlaybackCommand {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        // Serde's internally tagged unit variant otherwise ignores extra fields.
        // Decode StopAll as an empty struct variant while retaining the public API.
        #[derive(Deserialize)]
        #[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
        enum StrictCommand {
            /// Request playback of a catalog clip by its stable identity.
            Play {
                /// Stable catalog identity to play.
                clip_id: String,
            },
            /// Decode a visual play request with a bounded override.
            PlayVisual {
                clip_id: String,
                placement: VisualPlacement,
            },
            /// Decode an occurrence placement update or reset.
            SetVisual {
                instance_id: String,
                placement: Option<VisualPlacement>,
            },
            /// Stop the named occurrence; repeated stops are idempotent.
            Stop {
                /// Playback occurrence to cancel.
                instance_id: String,
            },
            StopAll {},
        }
        Ok(match StrictCommand::deserialize(deserializer)? {
            StrictCommand::Play { clip_id } => Self::Play { clip_id },
            StrictCommand::PlayVisual { clip_id, placement } => {
                Self::PlayVisual { clip_id, placement }
            }
            StrictCommand::SetVisual {
                instance_id,
                placement,
            } => Self::SetVisual {
                instance_id,
                placement,
            },
            StrictCommand::Stop { instance_id } => Self::Stop { instance_id },
            StrictCommand::StopAll {} => Self::StopAll,
        })
    }
}

/// Immediate host decision, distinct from later backend lifecycle events.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum CommandOutcome {
    /// Host accepted the command, optionally identifying its occurrence.
    Accepted {
        /// Accepted occurrence, or none for a session-wide command.
        instance_id: Option<String>,
    },
    /// Host rejected the command without applying a playback effect.
    Rejected {
        /// Host policy reason for rejection.
        reason: Rejection,
    },
}

/// Stable rejection reasons that controllers can display without native details.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Rejection {
    /// The envelope uses an unsupported wire schema version.
    UnsupportedVersion,
    /// The envelope belongs to another host lifetime.
    WrongSession,
    /// The selected catalog revision no longer matches the host.
    StaleCatalog,
    /// The command has invalid identifier or payload bounds.
    InvalidCommand,
    /// The requested clip is absent from the pinned catalog.
    UnknownClip,
    /// The occurrence is absent from active state and the bounded terminal history.
    UnknownInstance,
    /// Bounded host resources cannot admit the operation yet.
    Busy,
    /// A newer stop or cancellation invalidated the queued operation.
    Superseded,
}

/// Observable lifecycle of one playback occurrence.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InstanceState {
    /// Resources are being prepared; playback has not begun.
    Preparing,
    /// The backend has acknowledged playback start.
    Playing,
    /// Cancellation is requested; backend resources are not yet released.
    Stopping,
    /// Playback reached its natural terminal boundary.
    Finished,
    /// Cancellation completed and backend resources were released.
    Stopped,
    /// Preparation or playback failed with a terminal outcome.
    Failed,
}

impl InstanceState {
    /// Whether the occurrence can leave active state and enter bounded history.
    pub fn is_terminal(self) -> bool {
        matches!(self, Self::Finished | Self::Stopped | Self::Failed)
    }
}

/// Transport-safe state of one generation-tagged playback occurrence.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InstanceSnapshot {
    /// Host-generated identity of a single playback occurrence.
    pub instance_id: String,
    /// Stable pack/clip identity in the pinned catalog.
    pub clip_id: String,
    /// Monotonic backend generation used to reject stale notifications.
    pub generation: u64,
    /// Most recently acknowledged host lifecycle state.
    pub state: InstanceState,
    /// Explicit occurrence override; none uses the immutable catalog default.
    #[serde(default)]
    pub placement: Option<VisualPlacement>,
}

/// Cached result of attempting to prepare a catalog clip.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Readiness {
    /// No decoder preparation has been observed in this host lifetime.
    Unchecked,
    /// At least one preparation completed successfully.
    Ready,
    /// Preparation or playback failed with a terminal outcome.
    Failed,
}

/// Decoder readiness associated with a catalog identity.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClipStatus {
    /// Stable pack/clip identity in the pinned catalog.
    pub clip_id: String,
    /// Latest known preparation result for this clip.
    pub readiness: Readiness,
}

/// Complete bounded state used when connecting or recovering from an event gap.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HostSnapshot {
    /// Fresh host session identity; prevents replay across host lifetimes.
    pub session_id: String,
    /// Immutable catalog revision pinned by this host session.
    pub catalog_revision: String,
    /// Monotonic host event sequence at this observation.
    pub sequence: u64,
    /// Readiness of all clips in the pinned catalog.
    pub clips: Vec<ClipStatus>,
    /// Active occurrences followed by bounded terminal history.
    pub instances: Vec<InstanceSnapshot>,
}

/// Ordered host observations; command acceptance does not imply playback start.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum EventKind {
    /// A controller command passed host admission.
    Accepted {
        /// Deduplication identity of the command whose outcome is reported.
        command_id: String,
    },
    /// A controller command failed host admission.
    Rejected {
        /// Deduplication identity of the command whose outcome is reported.
        command_id: String,
        /// Stable reason the host declined this command.
        reason: Rejection,
    },
    /// Resources are being prepared; playback has not begun.
    Preparing,
    /// An active occurrence's explicit placement override changed or was reset.
    VisualChanged {
        /// Explicit override, or none when the catalog default was restored.
        placement: Option<VisualPlacement>,
    },
    /// The backend acknowledged actual playback start.
    Started,
    /// Playback reached its natural terminal boundary.
    Finished,
    /// Cancellation completed and backend resources were released.
    Stopped,
    /// The backend reported a terminal playback or preparation error.
    Failed {
        /// Human-readable failure detail with no protocol control meaning.
        message: String,
    },
}

/// One ordered event; sequence gaps require a fresh host snapshot.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HostEvent {
    /// Monotonic host event sequence at this observation.
    pub sequence: u64,
    /// Affected occurrence, or none for session-wide/command-only events.
    pub instance_id: Option<String>,
    /// Host observation carried by this event.
    pub event: EventKind,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strict_wire_envelope_roundtrips_without_native_data() {
        let json = r#"{"protocolVersion":2,"sessionId":"session","commandId":"command","catalogRevision":"revision","command":{"type":"play","clip_id":"pack/clip"}}"#;
        let envelope: Envelope = serde_json::from_str(json).unwrap();
        assert_eq!(
            envelope.command,
            PlaybackCommand::Play {
                clip_id: "pack/clip".into()
            }
        );
        assert_eq!(
            serde_json::from_value::<Envelope>(serde_json::to_value(&envelope).unwrap()).unwrap(),
            envelope
        );
        let mut value: serde_json::Value = serde_json::from_str(json).unwrap();
        value["assetPath"] = "/private/override.wav".into();
        assert!(serde_json::from_value::<Envelope>(value).is_err());
        let invalid = json.replace(
            "\"clip_id\":\"pack/clip\"",
            "\"clip_id\":\"pack/clip\",\"volume\":900",
        );
        assert!(serde_json::from_str::<Envelope>(&invalid).is_err());
    }

    #[test]
    fn stop_all_rejects_unknown_payload_fields() {
        assert_eq!(
            serde_json::from_str::<PlaybackCommand>(r#"{"type":"stop_all"}"#).unwrap(),
            PlaybackCommand::StopAll
        );
        assert!(
            serde_json::from_str::<PlaybackCommand>(
                r#"{"type":"stop_all","assetPath":"override"}"#
            )
            .is_err()
        );
        assert_eq!(
            serde_json::to_string(&PlaybackCommand::StopAll).unwrap(),
            r#"{"type":"stop_all"}"#
        );
    }

    #[test]
    fn visual_commands_events_and_snapshots_roundtrip() {
        let placement = VisualPlacement {
            width_bps: 4200,
            x_bps: 0,
            y_bps: 10_000,
            fit: VisualFit::Cover,
        };
        for command in [
            PlaybackCommand::PlayVisual {
                clip_id: "pack/clip".into(),
                placement,
            },
            PlaybackCommand::SetVisual {
                instance_id: "session:1".into(),
                placement: Some(placement),
            },
            PlaybackCommand::SetVisual {
                instance_id: "session:1".into(),
                placement: None,
            },
        ] {
            let encoded = serde_json::to_value(&command).unwrap();
            assert_eq!(
                serde_json::from_value::<PlaybackCommand>(encoded).unwrap(),
                command
            );
        }
        let event = EventKind::VisualChanged {
            placement: Some(placement),
        };
        assert_eq!(
            serde_json::from_value::<EventKind>(serde_json::to_value(&event).unwrap()).unwrap(),
            event
        );
        let legacy =
            r#"{"instanceId":"session:1","clipId":"pack/clip","generation":1,"state":"playing"}"#;
        let mut snapshot: InstanceSnapshot = serde_json::from_str(legacy).unwrap();
        assert_eq!(snapshot.placement, None);
        snapshot.placement = Some(placement);
        assert_eq!(
            serde_json::from_value::<InstanceSnapshot>(serde_json::to_value(&snapshot).unwrap())
                .unwrap(),
            snapshot
        );
    }

    #[test]
    fn visual_fit_defaults_for_earlier_payloads_and_accepts_only_explicit_known_policies() {
        let legacy = serde_json::json!({"widthBps":10000,"xBps":5000,"yBps":5000});
        let placement: VisualPlacement = serde_json::from_value(legacy.clone()).unwrap();
        assert_eq!(placement.fit, VisualFit::Catalog);
        let command =
            serde_json::json!({"type":"play_visual","clip_id":"pack/clip","placement":legacy});
        assert!(
            matches!(serde_json::from_value::<PlaybackCommand>(command).unwrap(),
            PlaybackCommand::PlayVisual { placement, .. } if placement.fit == VisualFit::Catalog)
        );
        let mut cover = serde_json::to_value(placement).unwrap();
        cover["fit"] = serde_json::json!("cover");
        let decoded: VisualPlacement = serde_json::from_value(cover.clone()).unwrap();
        assert_eq!(decoded.fit, VisualFit::Cover);
        assert_eq!(serde_json::to_value(decoded).unwrap(), cover);
        cover["fit"] = serde_json::json!("stretch");
        assert!(serde_json::from_value::<VisualPlacement>(cover).is_err());
    }

    #[test]
    fn visual_bounds_and_unknown_fields_are_explicit() {
        for width_bps in [1, 10_000] {
            assert!(
                VisualPlacement {
                    width_bps,
                    x_bps: 0,
                    y_bps: 10_000,
                    fit: VisualFit::Catalog,
                }
                .is_valid()
            );
        }
        for placement in [
            VisualPlacement {
                width_bps: 0,
                x_bps: 0,
                y_bps: 0,
                fit: VisualFit::Catalog,
            },
            VisualPlacement {
                width_bps: 10_001,
                x_bps: 0,
                y_bps: 0,
                fit: VisualFit::Catalog,
            },
            VisualPlacement {
                width_bps: 1,
                x_bps: 10_001,
                y_bps: 0,
                fit: VisualFit::Catalog,
            },
            VisualPlacement {
                width_bps: 1,
                x_bps: 0,
                y_bps: u16::MAX,
                fit: VisualFit::Catalog,
            },
        ] {
            assert!(!placement.is_valid());
        }
        let command = serde_json::json!({"type":"play_visual", "clip_id":"pack/clip", "placement":{"widthBps":3000,"xBps":5000,"yBps":5000,"crop":"cover"}});
        assert!(serde_json::from_value::<PlaybackCommand>(command).is_err());
        assert!(
            serde_json::from_value::<VisualPlacement>(
                serde_json::json!({"widthBps":-1,"xBps":0,"yBps":0})
            )
            .is_err()
        );
        assert!(
            serde_json::from_value::<VisualPlacement>(
                serde_json::json!({"widthBps":65536,"xBps":0,"yBps":0})
            )
            .is_err()
        );
        assert!(serde_json::from_value::<PlaybackCommand>(serde_json::json!({"type":"set_visual","instance_id":"session:1","placement":null,"assetPath":"override"})).is_err());
    }
}
