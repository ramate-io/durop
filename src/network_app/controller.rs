//! Application controller admission over one canonical command envelope.
//!
//! Native means the local, in-process host queue. Tailscale means a previously
//! authenticated remote session; it never provides a direct/public TCP mode.
//! Admission is bounded and does not claim host acceptance or physical playback.

use crate::{
    commands::{Envelope, HostSnapshot, PROTOCOL_VERSION, PlaybackCommand},
    host::{CommandSender, QueueError},
};
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
use tokio::sync::mpsc;

/// Why a controller could not admit a request without waiting.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ControllerAdmissionError {
    /// The bounded normal or emergency lane is full.
    Busy,
    /// The controller has no live command receiver.
    Unavailable,
    /// The request belongs to a different protocol or authenticated session.
    StaleSession,
}

impl From<QueueError> for ControllerAdmissionError {
    fn from(error: QueueError) -> Self {
        match error {
            QueueError::Busy => Self::Busy,
            QueueError::Closed => Self::Unavailable,
        }
    }
}

/// Admit a canonical request immediately, with explicit backpressure.
///
/// Implementations preserve command identity and policy fields. Success means
/// queued work only; the authoritative host still validates and deduplicates it.
pub(crate) trait ControllerBackend {
    /// Submit once on this backend; never retain work for a future connection.
    fn submit(&mut self, envelope: Envelope) -> Result<(), ControllerAdmissionError>;
}

/// Native controller backed exclusively by the shared in-process host ingress.
pub(crate) struct NativeControllerBackend {
    sender: CommandSender,
}

impl NativeControllerBackend {
    /// Attach to the same bounded policy queue used by authenticated remote work.
    pub(crate) fn new(sender: CommandSender) -> Self {
        Self { sender }
    }

    /// Admit host-side remote work with its cancellation guard and reply ticket.
    /// The canonical host ingress owns the StopAll barrier and final guard check.
    pub(super) fn submit_guarded(
        &self,
        envelope: Envelope,
        ticket: u64,
        active: Arc<dyn Fn() -> bool + Send + Sync>,
    ) -> Result<(), QueueError> {
        self.sender.try_send_guarded(envelope, ticket, active)
    }
}

impl ControllerBackend for NativeControllerBackend {
    fn submit(&mut self, envelope: Envelope) -> Result<(), ControllerAdmissionError> {
        self.sender.try_send(envelope).map_err(Into::into)
    }
}

/// Typestate with no connection and no channels capable of retaining commands.
pub(crate) struct Disconnected;

/// Typestate waiting for the transport's authenticated initial snapshot.
/// It deliberately does not implement [`ControllerBackend`].
pub(crate) struct Connecting {
    normal: mpsc::Sender<ControllerRequest>,
    priority: mpsc::Sender<ControllerRequest>,
    namespace: String,
    live: Arc<AtomicBool>,
}

impl Drop for Connecting {
    fn drop(&mut self) {
        self.live.store(false, Ordering::Release);
    }
}

/// Typestate bound to one authenticated host identity and command receiver.
/// A subsequent socket failure is still a runtime condition, reported by admission
/// or the transport actor; it never reconnects or replays requests automatically.
pub(crate) struct Connected {
    channels: Connecting,
    session_id: String,
    catalog_revision: String,
    next_command: u64,
    epoch: u64,
}

/// Tailscale controller whose API follows its connection lifecycle at compile time.
pub(crate) struct TailscaleControllerBackend<State> {
    state: State,
}

/// One online request and its emergency-stop ordering epoch.
#[derive(Debug)]
pub(super) struct ControllerRequest {
    /// The exact canonical identity and policy request to send over the socket.
    pub(super) envelope: Envelope,
    /// Earlier normal requests are superseded by an accepted later StopAll.
    pub(super) epoch: u64,
    live: Arc<AtomicBool>,
}

impl ControllerRequest {
    /// Check immediately before handing queued work to the authenticated socket.
    pub(super) fn is_active(&self) -> bool {
        self.live.load(Ordering::Acquire)
    }
}

/// Bounded receiver lanes owned by exactly one authenticated connection task.
pub(super) struct ControllerMailbox {
    /// Ordinary online requests, with no disconnected persistence.
    pub(super) normal: mpsc::Receiver<ControllerRequest>,
    /// StopAll requests, serviced before ordinary queued work.
    pub(super) priority: mpsc::Receiver<ControllerRequest>,
}

impl TailscaleControllerBackend<Disconnected> {
    /// Construct a disconnected backend; it cannot submit commands.
    pub(crate) fn new() -> Self {
        Self {
            state: Disconnected,
        }
    }

    /// Begin one explicit connection attempt with a fresh command-ID namespace.
    /// Only `Connecting::connected`, after an authenticated snapshot, enables admission.
    pub(super) fn begin_connect(
        self,
        namespace: String,
    ) -> (TailscaleControllerBackend<Connecting>, ControllerMailbox) {
        let (normal, normal_rx) = mpsc::channel(16);
        let (priority, priority_rx) = mpsc::channel(8);
        (
            TailscaleControllerBackend {
                state: Connecting {
                    normal,
                    priority,
                    namespace,
                    live: Arc::new(AtomicBool::new(true)),
                },
            },
            ControllerMailbox {
                normal: normal_rx,
                priority: priority_rx,
            },
        )
    }
}

impl TailscaleControllerBackend<Connecting> {
    /// Bind admission to the identity supplied by the authenticated initial snapshot.
    pub(super) fn connected(
        self,
        snapshot: &HostSnapshot,
    ) -> TailscaleControllerBackend<Connected> {
        TailscaleControllerBackend {
            state: Connected {
                channels: self.state,
                session_id: snapshot.session_id.clone(),
                catalog_revision: snapshot.catalog_revision.clone(),
                next_command: 0,
                epoch: 0,
            },
        }
    }
}

impl TailscaleControllerBackend<Connected> {
    /// Create a fresh online command identity and admit it through the shared trait.
    pub(super) fn submit_command(
        &mut self,
        command: PlaybackCommand,
    ) -> Result<(), ControllerAdmissionError> {
        self.state.next_command = self
            .state
            .next_command
            .checked_add(1)
            .expect("controller command sequence exhausted");
        let envelope = Envelope {
            protocol_version: PROTOCOL_VERSION,
            session_id: self.state.session_id.clone(),
            command_id: format!(
                "{}-{}",
                self.state.channels.namespace, self.state.next_command
            ),
            catalog_revision: self.state.catalog_revision.clone(),
            command,
        };
        self.submit(envelope)
    }

    /// End admission and revoke the lease on already queued work before dropping lanes.
    pub(super) fn disconnect(self) -> TailscaleControllerBackend<Disconnected> {
        TailscaleControllerBackend::new()
    }
}

impl ControllerBackend for TailscaleControllerBackend<Connected> {
    fn submit(&mut self, envelope: Envelope) -> Result<(), ControllerAdmissionError> {
        if !self.state.channels.live.load(Ordering::Acquire)
            || self.state.channels.normal.is_closed()
            || self.state.channels.priority.is_closed()
        {
            return Err(ControllerAdmissionError::Unavailable);
        }
        if envelope.protocol_version != PROTOCOL_VERSION
            || envelope.session_id != self.state.session_id
        {
            return Err(ControllerAdmissionError::StaleSession);
        }
        let stop = matches!(envelope.command, PlaybackCommand::StopAll);
        let epoch = if stop {
            self.state
                .epoch
                .checked_add(1)
                .expect("controller epoch exhausted")
        } else {
            self.state.epoch
        };
        let sender = if stop {
            &self.state.channels.priority
        } else {
            &self.state.channels.normal
        };
        sender
            .try_send(ControllerRequest {
                envelope,
                epoch,
                live: self.state.channels.live.clone(),
            })
            .map_err(|error| match error {
                mpsc::error::TrySendError::Full(_) => ControllerAdmissionError::Busy,
                mpsc::error::TrySendError::Closed(_) => ControllerAdmissionError::Unavailable,
            })?;
        if stop {
            self.state.epoch = epoch;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::host::command_channel;

    fn request(id: &str, command: PlaybackCommand) -> Envelope {
        Envelope {
            protocol_version: PROTOCOL_VERSION,
            session_id: "session".into(),
            command_id: id.into(),
            catalog_revision: "revision".into(),
            command,
        }
    }

    fn snapshot() -> HostSnapshot {
        HostSnapshot {
            session_id: "session".into(),
            catalog_revision: "revision".into(),
            sequence: 0,
            clips: vec![],
            instances: vec![],
        }
    }

    /// A test backend exercises the same admission contract without network I/O.
    struct FakeBackend {
        normal: usize,
        priority: usize,
    }

    impl ControllerBackend for FakeBackend {
        fn submit(&mut self, envelope: Envelope) -> Result<(), ControllerAdmissionError> {
            let remaining = if matches!(envelope.command, PlaybackCommand::StopAll) {
                &mut self.priority
            } else {
                &mut self.normal
            };
            if *remaining == 0 {
                return Err(ControllerAdmissionError::Busy);
            }
            *remaining -= 1;
            Ok(())
        }
    }

    fn bounded_admission_contract(backend: &mut dyn ControllerBackend) {
        let mut admitted = 0;
        loop {
            match backend.submit(request(
                &format!("normal-{admitted}"),
                PlaybackCommand::Play {
                    clip_id: "pack/clip".into(),
                },
            )) {
                Ok(()) => {
                    admitted += 1;
                    assert!(admitted <= 128, "ordinary admission must be bounded");
                }
                Err(ControllerAdmissionError::Busy) => break,
                Err(error) => panic!("unexpected admission result: {error:?}"),
            }
        }
        assert!(admitted > 0);
        assert_eq!(
            backend.submit(request("stop", PlaybackCommand::StopAll)),
            Ok(())
        );
    }

    #[test]
    fn native_remote_and_fake_share_bounded_admission_and_stop_priority() {
        let (sender, _inbox) = command_channel(|| {});
        bounded_admission_contract(&mut NativeControllerBackend::new(sender));
        let (connecting, _mailbox) = TailscaleControllerBackend::new().begin_connect("test".into());
        bounded_admission_contract(&mut connecting.connected(&snapshot()));
        bounded_admission_contract(&mut FakeBackend {
            normal: 2,
            priority: 1,
        });
    }

    #[test]
    fn only_connected_remote_typestate_implements_admission() {
        fn ready<T: ControllerBackend>() {}
        ready::<NativeControllerBackend>();
        ready::<TailscaleControllerBackend<Connected>>();

        // If either non-ready state gains ControllerBackend, these calls become
        // ambiguous and fail compilation, enforcing the negative trait assertions.
        trait NotReady<Marker> {}
        impl<T> NotReady<()> for T {}
        impl<T: ControllerBackend> NotReady<u8> for T {}
        fn not_ready<T: NotReady<Marker>, Marker>() {}
        not_ready::<TailscaleControllerBackend<Disconnected>, _>();
        not_ready::<TailscaleControllerBackend<Connecting>, _>();
    }

    #[test]
    fn remote_epoch_session_and_disconnect_are_not_replayed() {
        let (connecting, mut mailbox) =
            TailscaleControllerBackend::new().begin_connect("test".into());
        let mut connected = connecting.connected(&snapshot());
        connected
            .submit(request(
                "before",
                PlaybackCommand::Play {
                    clip_id: "pack/clip".into(),
                },
            ))
            .unwrap();
        connected
            .submit(request("stop", PlaybackCommand::StopAll))
            .unwrap();
        connected
            .submit(request(
                "after",
                PlaybackCommand::Play {
                    clip_id: "pack/clip".into(),
                },
            ))
            .unwrap();
        assert_eq!(mailbox.normal.try_recv().unwrap().epoch, 0);
        assert_eq!(mailbox.priority.try_recv().unwrap().epoch, 1);
        assert_eq!(mailbox.normal.try_recv().unwrap().epoch, 1);
        let mut stale = request("stale", PlaybackCommand::StopAll);
        stale.session_id = "old".into();
        assert_eq!(
            connected.submit(stale),
            Err(ControllerAdmissionError::StaleSession)
        );
        connected
            .submit(request("canceled", PlaybackCommand::StopAll))
            .unwrap();
        let disconnected = connected.disconnect();
        assert!(!mailbox.priority.try_recv().unwrap().is_active());
        assert!(mailbox.normal.is_closed());
        assert!(mailbox.priority.is_closed());
        let (_new_attempt, new_mailbox) = disconnected.begin_connect("new".into());
        assert!(new_mailbox.normal.is_empty());
        assert!(new_mailbox.priority.is_empty());
    }
}
