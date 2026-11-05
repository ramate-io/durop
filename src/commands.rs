//! Shared local/remote protocol. Acceptance is distinct from backend playback.
use serde::{Deserialize, Serialize};

pub const PROTOCOL_VERSION: u32 = 1;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Envelope {
    pub protocol_version: u32,
    pub session_id: String,
    pub command_id: String,
    pub catalog_revision: String,
    pub command: PlaybackCommand,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum PlaybackCommand {
    Play { clip_id: String },
    Stop { instance_id: String },
    StopAll,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum CommandOutcome {
    Accepted { instance_id: Option<String> },
    Rejected { reason: Rejection },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Rejection {
    UnsupportedVersion,
    WrongSession,
    StaleCatalog,
    InvalidCommand,
    UnknownClip,
    UnknownInstance,
    Busy,
    Superseded,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InstanceState {
    Preparing,
    Playing,
    Stopping,
    Finished,
    Stopped,
    Failed,
}
impl InstanceState {
    pub fn is_terminal(self) -> bool {
        matches!(self, Self::Finished | Self::Stopped | Self::Failed)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InstanceSnapshot {
    pub instance_id: String,
    pub clip_id: String,
    pub generation: u64,
    pub state: InstanceState,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Readiness {
    Unchecked,
    Ready,
    Failed,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClipStatus {
    pub clip_id: String,
    pub readiness: Readiness,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HostSnapshot {
    pub session_id: String,
    pub catalog_revision: String,
    pub sequence: u64,
    pub clips: Vec<ClipStatus>,
    pub instances: Vec<InstanceSnapshot>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum EventKind {
    Accepted {
        command_id: String,
    },
    Rejected {
        command_id: String,
        reason: Rejection,
    },
    Preparing,
    Started,
    Finished,
    Stopped,
    Failed {
        message: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HostEvent {
    pub sequence: u64,
    pub instance_id: Option<String>,
    pub event: EventKind,
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn strict_wire_envelope_roundtrips_without_native_data() {
        let json = r#"{"protocolVersion":1,"sessionId":"session","commandId":"command","catalogRevision":"revision","command":{"type":"play","clip_id":"pack/clip"}}"#;
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
}
