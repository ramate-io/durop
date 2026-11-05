//! Serialized host authority; native resources stay behind generation-tagged effects.
use crate::{
    catalog::{Catalog, Clip, Retrigger},
    commands::{
        ClipStatus, CommandOutcome, Envelope, EventKind, HostEvent, HostSnapshot, InstanceSnapshot,
        InstanceState, PROTOCOL_VERSION, PlaybackCommand, Readiness, Rejection,
    },
};
use std::{
    collections::{BTreeMap, VecDeque},
    sync::Arc,
};

pub const CONCURRENCY_CAP: usize = 8;
pub const DEDUPE_CAP: usize = 1024;
pub const HISTORY_CAP: usize = 256;
pub const EVENT_CAP: usize = 256;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BackendToken {
    pub instance_id: String,
    pub generation: u64,
}
#[derive(Debug, Clone)]
pub enum BackendEffect {
    Prepare {
        token: BackendToken,
        clip: Box<Clip>,
    },
    Start {
        token: BackendToken,
    },
    Stop {
        token: BackendToken,
    },
}
#[derive(Debug, Clone)]
pub struct BackendEvent {
    pub token: BackendToken,
    pub event: BackendEventKind,
}
#[derive(Debug, Clone)]
pub enum BackendEventKind {
    Prepared,
    Started,
    Finished,
    Stopped,
    Failed { message: String },
}

struct Instance {
    snapshot: InstanceSnapshot,
    prepared: bool,
    blockers: Vec<String>,
    start_requested: bool,
}
impl Instance {
    fn token(&self) -> BackendToken {
        BackendToken {
            instance_id: self.snapshot.instance_id.clone(),
            generation: self.snapshot.generation,
        }
    }
}

pub struct Host {
    session_id: String,
    catalog: Arc<Catalog>,
    instances: BTreeMap<String, Instance>,
    history: VecDeque<InstanceSnapshot>,
    readiness: BTreeMap<String, Readiness>,
    dedupe: BTreeMap<String, CommandOutcome>,
    dedupe_order: VecDeque<String>,
    events: VecDeque<HostEvent>,
    effects: VecDeque<BackendEffect>,
    sequence: u64,
    next_instance: u64,
}

impl Host {
    /// The session ID must be fresh for every session and supplied by the owner.
    /// Pin the catalog for this host's entire lifetime; refresh creates a new host/session.
    pub fn new(session_id: String, catalog: Arc<Catalog>) -> Self {
        assert!(!session_id.is_empty() && session_id.len() <= 128);
        let readiness = catalog
            .clips
            .iter()
            .map(|c| (c.identity.clone(), Readiness::Unchecked))
            .collect();
        Self {
            session_id,
            catalog,
            instances: BTreeMap::new(),
            history: VecDeque::new(),
            readiness,
            dedupe: BTreeMap::new(),
            dedupe_order: VecDeque::new(),
            events: VecDeque::new(),
            effects: VecDeque::new(),
            sequence: 0,
            next_instance: 0,
        }
    }

    pub fn accept(&mut self, envelope: Envelope) -> CommandOutcome {
        self.accept_inner(envelope, false)
    }

    fn accept_inner(&mut self, envelope: Envelope, superseded: bool) -> CommandOutcome {
        let reason = if envelope.protocol_version != PROTOCOL_VERSION {
            Some(Rejection::UnsupportedVersion)
        } else if envelope.session_id != self.session_id {
            Some(Rejection::WrongSession)
        } else if envelope.command_id.is_empty() || envelope.command_id.len() > 128 {
            Some(Rejection::InvalidCommand)
        } else {
            None
        };
        if let Some(reason) = reason {
            return self.rejected(&envelope.command_id, reason);
        }
        if let Some(outcome) = self.dedupe.get(&envelope.command_id) {
            return outcome.clone();
        }
        let outcome = if superseded {
            self.rejected(&envelope.command_id, Rejection::Superseded)
        } else if envelope.catalog_revision != self.catalog.revision {
            self.rejected(&envelope.command_id, Rejection::StaleCatalog)
        } else {
            // Record acceptance before lifecycle events; rejected requests emit no acceptance.
            match self.decide(&envelope.command) {
                Ok(instance_id) => {
                    self.emit(
                        instance_id.clone(),
                        EventKind::Accepted {
                            command_id: envelope.command_id.clone(),
                        },
                    );
                    self.perform(envelope.command, instance_id.clone());
                    CommandOutcome::Accepted { instance_id }
                }
                Err(reason) => self.rejected(&envelope.command_id, reason),
            }
        };
        if self.dedupe_order.len() == DEDUPE_CAP {
            self.dedupe.remove(&self.dedupe_order.pop_front().unwrap());
        }
        self.dedupe_order.push_back(envelope.command_id.clone());
        self.dedupe.insert(envelope.command_id, outcome.clone());
        outcome
    }

    fn decide(&mut self, command: &PlaybackCommand) -> Result<Option<String>, Rejection> {
        match command {
            PlaybackCommand::Play { clip_id } => {
                let clip = self
                    .catalog
                    .clips
                    .iter()
                    .find(|c| &c.identity == clip_id)
                    .ok_or(Rejection::UnknownClip)?;
                let existing = self.instances.values().find(|i| {
                    i.snapshot.clip_id == *clip_id && i.snapshot.state != InstanceState::Stopping
                });
                if matches!(clip.metadata.playback.retrigger, Retrigger::IgnoreIfPlaying)
                    && let Some(existing) = existing
                {
                    return Ok(Some(existing.snapshot.instance_id.clone()));
                }
                // Count stopping workers too: do not exceed native-resource bounds during restart.
                if self.instances.len() >= CONCURRENCY_CAP || self.effects.len() >= 64 {
                    return Err(Rejection::Busy);
                }
                self.next_instance = self
                    .next_instance
                    .checked_add(1)
                    .expect("instance sequence exhausted");
                Ok(Some(format!("{}:{}", self.session_id, self.next_instance)))
            }
            PlaybackCommand::Stop { instance_id } => {
                if self.instances.contains_key(instance_id)
                    || self.history.iter().any(|i| i.instance_id == *instance_id)
                {
                    Ok(Some(instance_id.clone()))
                } else {
                    Err(Rejection::UnknownInstance)
                }
            }
            PlaybackCommand::StopAll => Ok(None),
        }
    }

    fn perform(&mut self, command: PlaybackCommand, instance_id: Option<String>) {
        match command {
            PlaybackCommand::Play { clip_id } => {
                let instance_id = instance_id.unwrap();
                if self.instances.contains_key(&instance_id) {
                    return;
                }
                let clip = self
                    .catalog
                    .clips
                    .iter()
                    .find(|c| c.identity == clip_id)
                    .unwrap()
                    .clone();
                let blockers: Vec<_> =
                    if matches!(clip.metadata.playback.retrigger, Retrigger::Restart) {
                        self.instances
                            .values()
                            .filter(|i| i.snapshot.clip_id == clip_id)
                            .map(|i| i.snapshot.instance_id.clone())
                            .collect()
                    } else {
                        Vec::new()
                    };
                for id in &blockers {
                    self.stop(id);
                }
                let instance = Instance {
                    snapshot: InstanceSnapshot {
                        instance_id: instance_id.clone(),
                        clip_id,
                        generation: self.next_instance,
                        state: InstanceState::Preparing,
                    },
                    prepared: false,
                    blockers,
                    start_requested: false,
                };
                self.effects.push_back(BackendEffect::Prepare {
                    token: instance.token(),
                    clip: Box::new(clip),
                });
                self.instances.insert(instance_id.clone(), instance);
                self.emit(Some(instance_id), EventKind::Preparing);
            }
            PlaybackCommand::Stop { instance_id } => self.stop(&instance_id),
            PlaybackCommand::StopAll => {
                let ids: Vec<_> = self.instances.keys().cloned().collect();
                for id in ids {
                    self.stop(&id);
                }
            }
        }
    }

    fn stop(&mut self, id: &str) {
        if let Some(instance) = self.instances.get_mut(id)
            && instance.snapshot.state != InstanceState::Stopping
        {
            instance.snapshot.state = InstanceState::Stopping;
            let token = instance.token();
            // Cancel unsent work too. Every stop still gets a teardown acknowledgement.
            self.effects.retain(|effect| match effect {
                BackendEffect::Prepare { token: t, .. } | BackendEffect::Start { token: t } => {
                    t != &token
                }
                BackendEffect::Stop { .. } => true,
            });
            self.effects.push_back(BackendEffect::Stop { token });
        }
    }

    /// Callback tokens must match the worker's original occurrence and generation.
    pub fn backend_event(&mut self, event: BackendEvent) {
        let Some(instance) = self.instances.get_mut(&event.token.instance_id) else {
            return;
        };
        if instance.token() != event.token {
            return;
        }
        let id = event.token.instance_id;
        let state = instance.snapshot.state;
        match event.event {
            BackendEventKind::Prepared
                if state == InstanceState::Preparing && !instance.prepared =>
            {
                instance.prepared = true;
                self.readiness
                    .insert(instance.snapshot.clip_id.clone(), Readiness::Ready);
                if instance.blockers.is_empty() {
                    instance.start_requested = true;
                    self.effects.push_back(BackendEffect::Start {
                        token: instance.token(),
                    });
                }
            }
            BackendEventKind::Started
                if state == InstanceState::Preparing && instance.start_requested =>
            {
                instance.snapshot.state = InstanceState::Playing;
                self.emit(Some(id), EventKind::Started);
            }
            BackendEventKind::Finished if state == InstanceState::Playing => {
                self.terminal(&id, InstanceState::Finished, EventKind::Finished)
            }
            BackendEventKind::Stopped if state == InstanceState::Stopping => {
                self.terminal(&id, InstanceState::Stopped, EventKind::Stopped)
            }
            BackendEventKind::Failed { message } if state != InstanceState::Stopping => {
                if state == InstanceState::Preparing {
                    self.readiness
                        .insert(instance.snapshot.clip_id.clone(), Readiness::Failed);
                }
                self.terminal(
                    &id,
                    InstanceState::Failed,
                    EventKind::Failed {
                        message: message.chars().take(512).collect(),
                    },
                );
            }
            // A stop racing normal completion still acknowledges teardown exactly once.
            BackendEventKind::Finished | BackendEventKind::Failed { .. }
                if state == InstanceState::Stopping =>
            {
                self.terminal(&id, InstanceState::Stopped, EventKind::Stopped)
            }
            _ => (),
        }
    }

    fn terminal(&mut self, id: &str, state: InstanceState, event: EventKind) {
        let Some(mut instance) = self.instances.remove(id) else {
            return;
        };
        let token = instance.token();
        self.effects.retain(|effect| match effect {
            BackendEffect::Prepare { token: t, .. }
            | BackendEffect::Start { token: t }
            | BackendEffect::Stop { token: t } => t != &token,
        });
        instance.snapshot.state = state;
        if self.history.len() == HISTORY_CAP {
            self.history.pop_front();
        }
        self.history.push_back(instance.snapshot);
        self.emit(Some(id.to_owned()), event);
        for waiting in self.instances.values_mut() {
            waiting.blockers.retain(|blocker| blocker != id);
            if waiting.snapshot.state == InstanceState::Preparing
                && waiting.prepared
                && !waiting.start_requested
                && waiting.blockers.is_empty()
            {
                waiting.start_requested = true;
                self.effects.push_back(BackendEffect::Start {
                    token: waiting.token(),
                });
            }
        }
    }

    fn rejected(&mut self, command_id: &str, reason: Rejection) -> CommandOutcome {
        self.emit(
            None,
            EventKind::Rejected {
                command_id: command_id.chars().take(128).collect(),
                reason: reason.clone(),
            },
        );
        CommandOutcome::Rejected { reason }
    }
    fn emit(&mut self, instance_id: Option<String>, event: EventKind) {
        self.sequence = self
            .sequence
            .checked_add(1)
            .expect("event sequence exhausted");
        if self.events.len() == EVENT_CAP {
            self.events.pop_front();
        }
        self.events.push_back(HostEvent {
            sequence: self.sequence,
            instance_id,
            event,
        });
    }
    pub fn snapshot(&self) -> HostSnapshot {
        HostSnapshot {
            session_id: self.session_id.clone(),
            catalog_revision: self.catalog.revision.clone(),
            sequence: self.sequence,
            clips: self
                .readiness
                .iter()
                .map(|(clip_id, readiness)| ClipStatus {
                    clip_id: clip_id.clone(),
                    readiness: readiness.clone(),
                })
                .collect(),
            instances: self
                .history
                .iter()
                .cloned()
                .chain(self.instances.values().map(|i| i.snapshot.clone()))
                .collect(),
        }
    }
    pub fn drain_events(&mut self) -> Vec<HostEvent> {
        self.events.drain(..).collect()
    }
    pub fn drain_effects(&mut self) -> Vec<BackendEffect> {
        self.effects.drain(..).collect()
    }
}

/// Bounded shared ingress. The wake callback must notify Bevy's event loop; it
/// must not perform playback or block. One receiver serializes all policy work.
pub fn command_channel(wake: impl Fn() + Send + Sync + 'static) -> (CommandSender, CommandInbox) {
    let (normal_tx, normal_rx) = tokio::sync::mpsc::channel(128);
    let (priority_tx, priority_rx) = tokio::sync::mpsc::channel(8);
    let wake: Arc<dyn Fn() + Send + Sync> = Arc::new(wake);
    (
        CommandSender {
            inner: Arc::new(std::sync::Mutex::new(Submit {
                normal_tx,
                priority_tx,
                epoch: 0,
            })),
            wake: wake.clone(),
        },
        CommandInbox {
            normal_rx,
            priority_rx,
            cutoff: 0,
            wake,
        },
    )
}

struct Queued {
    envelope: Envelope,
    epoch: u64,
    ticket: Option<u64>,
    active: Option<Arc<dyn Fn() -> bool + Send + Sync>>,
}
struct Submit {
    normal_tx: tokio::sync::mpsc::Sender<Queued>,
    priority_tx: tokio::sync::mpsc::Sender<Queued>,
    epoch: u64,
}
#[derive(Clone)]
pub struct CommandSender {
    inner: Arc<std::sync::Mutex<Submit>>,
    wake: Arc<dyn Fn() + Send + Sync>,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QueueError {
    Busy,
    Closed,
}
impl CommandSender {
    /// Queue admission is not host acceptance. Failed admission always returns
    /// Busy/Closed; it never silently drops a controller command.
    pub fn try_send(&self, envelope: Envelope) -> Result<(), QueueError> {
        self.send(envelope, None, None)
    }
    /// Remote admission must still be live immediately before host acceptance.
    pub fn try_send_guarded(
        &self,
        envelope: Envelope,
        ticket: u64,
        active: Arc<dyn Fn() -> bool + Send + Sync>,
    ) -> Result<(), QueueError> {
        self.send(envelope, Some(ticket), Some(active))
    }
    fn send(
        &self,
        envelope: Envelope,
        ticket: Option<u64>,
        active: Option<Arc<dyn Fn() -> bool + Send + Sync>>,
    ) -> Result<(), QueueError> {
        let mut submit = self.inner.lock().unwrap();
        let priority = matches!(envelope.command, PlaybackCommand::StopAll);
        let epoch = if priority {
            submit.epoch.checked_add(1).expect("queue epoch exhausted")
        } else {
            submit.epoch
        };
        let tx = if priority {
            &submit.priority_tx
        } else {
            &submit.normal_tx
        };
        tx.try_send(Queued {
            envelope,
            epoch,
            ticket,
            active,
        })
        .map_err(|e| match e {
            tokio::sync::mpsc::error::TrySendError::Full(_) => QueueError::Busy,
            tokio::sync::mpsc::error::TrySendError::Closed(_) => QueueError::Closed,
        })?;
        if priority {
            submit.epoch = epoch;
        }
        drop(submit);
        (self.wake)();
        Ok(())
    }
}
pub struct CommandInbox {
    normal_rx: tokio::sync::mpsc::Receiver<Queued>,
    priority_rx: tokio::sync::mpsc::Receiver<Queued>,
    cutoff: u64,
    wake: Arc<dyn Fn() + Send + Sync>,
}
#[derive(Debug)]
pub struct CommandReceipt {
    pub command_id: String,
    pub outcome: CommandOutcome,
    pub ticket: Option<u64>,
}
impl CommandInbox {
    /// At most 8 emergency requests then 32 ordinary requests per update.
    /// A new accepted StopAll supersedes older queued commands, but a replayed
    /// StopAll does not stop occurrences created after its original execution.
    pub fn drain(&mut self, host: &mut Host) -> Vec<CommandReceipt> {
        let mut receipts = Vec::new();
        for _ in 0..8 {
            let Ok(queued) = self.priority_rx.try_recv() else {
                break;
            };
            let sequence = host.sequence;
            let command_id = queued.envelope.command_id.clone();
            let live = queued.active.as_ref().is_none_or(|active| active());
            let outcome = if live {
                host.accept(queued.envelope)
            } else {
                CommandOutcome::Rejected {
                    reason: Rejection::Superseded,
                }
            };
            if matches!(outcome, CommandOutcome::Accepted { .. }) && host.sequence != sequence {
                self.cutoff = self.cutoff.max(queued.epoch);
            }
            receipts.push(CommandReceipt {
                command_id,
                outcome,
                ticket: queued.ticket,
            });
        }
        for _ in 0..32 {
            let Ok(queued) = self.normal_rx.try_recv() else {
                break;
            };
            let command_id = queued.envelope.command_id.clone();
            let outcome = if queued.active.as_ref().is_none_or(|active| active()) {
                host.accept_inner(queued.envelope, queued.epoch < self.cutoff)
            } else {
                CommandOutcome::Rejected {
                    reason: Rejection::Superseded,
                }
            };
            receipts.push(CommandReceipt {
                command_id,
                outcome,
                ticket: queued.ticket,
            });
        }
        if self.has_pending() {
            (self.wake)();
        }
        receipts
    }
    pub fn has_pending(&self) -> bool {
        !self.priority_rx.is_empty() || !self.normal_rx.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::catalog::{ClipMetadata, DecoderReadiness, PackManifest, Playback};
    use std::sync::atomic::{AtomicUsize, Ordering};

    fn host(policy: Retrigger) -> Host {
        let clip = Clip {
            identity: "pack/clip".into(),
            category: "category".into(),
            metadata: ClipMetadata {
                id: "clip".into(),
                title: "Clip".into(),
                asset: "clip.wav".into(),
                tags: vec![],
                playback: Playback {
                    retrigger: policy,
                    looping: false,
                    max_duration_ms: 1200,
                },
                visual: None,
            },
            metadata_path: "clip.json".into(),
            icon_path: "clip.png".into(),
            asset_path: "clip.wav".into(),
            decoder_readiness: DecoderReadiness::Unchecked,
        };
        Host::new(
            "session".into(),
            Arc::new(Catalog {
                root: ".".into(),
                manifest: PackManifest {
                    schema_version: 1,
                    pack_id: "pack".into(),
                    title: "Pack".into(),
                },
                clips: vec![clip],
                revision: "revision".into(),
            }),
        )
    }
    fn envelope(id: &str, command: PlaybackCommand) -> Envelope {
        Envelope {
            protocol_version: 1,
            session_id: "session".into(),
            command_id: id.into(),
            catalog_revision: "revision".into(),
            command,
        }
    }
    fn play(id: &str) -> Envelope {
        envelope(
            id,
            PlaybackCommand::Play {
                clip_id: "pack/clip".into(),
            },
        )
    }
    fn prepared_token(host: &mut Host, id: &str) -> BackendToken {
        assert!(matches!(
            host.accept(play(id)),
            CommandOutcome::Accepted { .. }
        ));
        host.drain_effects()
            .into_iter()
            .find_map(|e| match e {
                BackendEffect::Prepare { token, clip } => {
                    assert_eq!(clip.metadata.playback.max_duration_ms, 1200);
                    Some(token)
                }
                _ => None,
            })
            .unwrap()
    }
    fn signal(host: &mut Host, token: &BackendToken, event: BackendEventKind) {
        host.backend_event(BackendEvent {
            token: token.clone(),
            event,
        });
    }
    fn start(host: &mut Host, token: &BackendToken) {
        signal(host, token, BackendEventKind::Prepared);
        assert!(matches!(
            host.drain_effects().as_slice(),
            [BackendEffect::Start { .. }]
        ));
        signal(host, token, BackendEventKind::Started);
    }

    #[test]
    fn acceptance_and_asset_readiness_never_claim_started_playback() {
        let mut host = host(Retrigger::Stack);
        let token = prepared_token(&mut host, "one");
        let snapshot = host.snapshot();
        assert_eq!(snapshot.instances[0].state, InstanceState::Preparing);
        assert_eq!(snapshot.clips[0].readiness, Readiness::Unchecked);
        assert!(matches!(
            host.drain_events().as_slice(),
            [
                HostEvent {
                    sequence: 1,
                    event: EventKind::Accepted { .. },
                    ..
                },
                HostEvent {
                    sequence: 2,
                    event: EventKind::Preparing,
                    ..
                }
            ]
        ));
        signal(&mut host, &token, BackendEventKind::Started); // unprepared callback is invalid
        assert_eq!(host.snapshot().instances[0].state, InstanceState::Preparing);
        signal(&mut host, &token, BackendEventKind::Prepared);
        signal(&mut host, &token, BackendEventKind::Prepared);
        assert_eq!(host.drain_effects().len(), 1);
        assert_eq!(host.snapshot().clips[0].readiness, Readiness::Ready);
        assert_eq!(host.snapshot().instances[0].state, InstanceState::Preparing);
        signal(&mut host, &token, BackendEventKind::Started);
        signal(&mut host, &token, BackendEventKind::Started);
        assert_eq!(host.drain_events().len(), 1);
        assert_eq!(host.snapshot().instances[0].state, InstanceState::Playing);
    }

    #[test]
    fn duplicate_outcomes_and_session_revision_version_validation() {
        let mut host = host(Retrigger::Stack);
        let outcome = host.accept(play("one"));
        let sequence = host.snapshot().sequence;
        assert_eq!(host.accept(play("one")), outcome);
        assert_eq!(host.snapshot().sequence, sequence);
        assert_eq!(host.instances.len(), 1);
        for reason in [
            Rejection::WrongSession,
            Rejection::StaleCatalog,
            Rejection::UnsupportedVersion,
        ] {
            let mut request = play("invalid");
            match reason {
                Rejection::WrongSession => request.session_id = "old".into(),
                Rejection::StaleCatalog => request.catalog_revision = "old".into(),
                _ => request.protocol_version = 2,
            }
            assert_eq!(host.accept(request), CommandOutcome::Rejected { reason });
        }
        assert_eq!(host.instances.len(), 1);
        assert_eq!(host.drain_effects().len(), 1);
    }

    #[test]
    fn retrigger_ignore_returns_same_occurrence_and_stack_creates_distinct_ones() {
        let mut ignore = host(Retrigger::IgnoreIfPlaying);
        let first = ignore.accept(play("one"));
        assert_eq!(ignore.accept(play("two")), first);
        assert_eq!(ignore.instances.len(), 1);
        assert_eq!(ignore.drain_effects().len(), 1);
        let mut stack = host(Retrigger::Stack);
        assert_ne!(stack.accept(play("one")), stack.accept(play("two")));
        assert_eq!(stack.instances.len(), 2);
    }

    #[test]
    fn restart_invalidates_preparation_and_waits_for_prior_resource_release() {
        let mut host = host(Retrigger::Restart);
        let old = prepared_token(&mut host, "old");
        let new = prepared_token(&mut host, "new");
        assert_eq!(
            host.instances[&old.instance_id].snapshot.state,
            InstanceState::Stopping
        );
        signal(&mut host, &old, BackendEventKind::Prepared);
        signal(&mut host, &old, BackendEventKind::Started);
        signal(&mut host, &new, BackendEventKind::Prepared);
        signal(&mut host, &new, BackendEventKind::Started);
        assert!(host.drain_effects().is_empty());
        assert_eq!(
            host.instances[&new.instance_id].snapshot.state,
            InstanceState::Preparing
        );
        signal(&mut host, &old, BackendEventKind::Stopped);
        assert!(
            matches!(host.drain_effects().as_slice(), [BackendEffect::Start { token }] if token == &new)
        );
        signal(&mut host, &new, BackendEventKind::Started);
        assert_eq!(
            host.instances[&new.instance_id].snapshot.state,
            InstanceState::Playing
        );
    }

    #[test]
    fn cap_counts_preparing_playing_and_stopping_workers() {
        let mut host = host(Retrigger::Stack);
        let tokens: Vec<_> = (0..CONCURRENCY_CAP)
            .map(|n| prepared_token(&mut host, &n.to_string()))
            .collect();
        assert_eq!(
            host.accept(play("excess")),
            CommandOutcome::Rejected {
                reason: Rejection::Busy
            }
        );
        host.accept(envelope("stop", PlaybackCommand::StopAll));
        assert_eq!(
            host.accept(play("still-excess")),
            CommandOutcome::Rejected {
                reason: Rejection::Busy
            }
        );
        signal(&mut host, &tokens[0], BackendEventKind::Stopped);
        assert!(matches!(
            host.accept(play("released")),
            CommandOutcome::Accepted { .. }
        ));
        assert_eq!(host.instances.len(), CONCURRENCY_CAP);
    }

    #[test]
    fn stop_during_prepare_stale_tokens_and_terminal_callbacks_are_idempotent() {
        let mut host = host(Retrigger::Restart);
        let token = prepared_token(&mut host, "play");
        let mut stale = token.clone();
        stale.generation += 1;
        signal(&mut host, &stale, BackendEventKind::Prepared);
        assert!(host.drain_effects().is_empty());
        for id in ["stop-one", "stop-two"] {
            host.accept(envelope(
                id,
                PlaybackCommand::Stop {
                    instance_id: token.instance_id.clone(),
                },
            ));
        }
        assert_eq!(host.drain_effects().len(), 1);
        assert_eq!(
            host.instances[&token.instance_id].snapshot.state,
            InstanceState::Stopping
        );
        signal(&mut host, &token, BackendEventKind::Prepared);
        signal(&mut host, &token, BackendEventKind::Started);
        signal(&mut host, &token, BackendEventKind::Stopped);
        let sequence = host.sequence;
        for event in [
            BackendEventKind::Stopped,
            BackendEventKind::Finished,
            BackendEventKind::Failed {
                message: "late".into(),
            },
            BackendEventKind::Prepared,
        ] {
            signal(&mut host, &token, event);
        }
        assert_eq!(host.sequence, sequence);
        assert_eq!(host.history.len(), 1);
        assert_eq!(host.history[0].state, InstanceState::Stopped);
        assert!(host.drain_effects().is_empty());
        assert!(matches!(
            host.accept(envelope(
                "stop-terminal",
                PlaybackCommand::Stop {
                    instance_id: token.instance_id
                }
            )),
            CommandOutcome::Accepted { .. }
        ));
    }

    #[test]
    fn normal_completion_and_prepare_failure_each_emit_one_terminal_event() {
        let mut host = host(Retrigger::Stack);
        let token = prepared_token(&mut host, "play");
        start(&mut host, &token);
        signal(&mut host, &token, BackendEventKind::Finished);
        signal(
            &mut host,
            &token,
            BackendEventKind::Failed {
                message: "late".into(),
            },
        );
        assert_eq!(host.history.len(), 1);
        assert_eq!(host.history[0].state, InstanceState::Finished);
        let failed = prepared_token(&mut host, "failed");
        signal(
            &mut host,
            &failed,
            BackendEventKind::Failed {
                message: "decoder failed".into(),
            },
        );
        signal(&mut host, &failed, BackendEventKind::Prepared);
        assert_eq!(host.snapshot().clips[0].readiness, Readiness::Failed);
        assert_eq!(host.history[1].state, InstanceState::Failed);
        assert!(host.drain_effects().is_empty());
    }

    #[test]
    fn saturated_normal_ingress_cannot_block_stop_all_and_old_plays_are_superseded() {
        let wakes = Arc::new(AtomicUsize::new(0));
        let w = wakes.clone();
        let (tx, mut rx) = command_channel(move || {
            w.fetch_add(1, Ordering::SeqCst);
        });
        let mut host = host(Retrigger::Stack);
        let token = prepared_token(&mut host, "active");
        for n in 0..128 {
            tx.try_send(play(&format!("queued-{n}"))).unwrap();
        }
        assert_eq!(tx.try_send(play("overflow")), Err(QueueError::Busy));
        tx.try_send(envelope("stop", PlaybackCommand::StopAll))
            .unwrap();
        let receipts = rx.drain(&mut host);
        assert_eq!(receipts.len(), 33);
        assert_eq!(receipts[0].command_id, "stop");
        assert!(receipts[1..].iter().all(|r| r.outcome
            == CommandOutcome::Rejected {
                reason: Rejection::Superseded
            }));
        assert_eq!(host.instances.len(), 1);
        assert_eq!(
            host.instances[&token.instance_id].snapshot.state,
            InstanceState::Stopping
        );
        assert_eq!(wakes.load(Ordering::SeqCst), 130); // admitted requests + budget continuation
        while rx.has_pending() {
            rx.drain(&mut host);
        }
        assert_eq!(host.instances.len(), 1);
        drop(rx);
        assert_eq!(tx.try_send(play("closed")), Err(QueueError::Closed));
    }

    #[test]
    fn duplicate_or_wrong_session_stop_all_does_not_supersede_new_commands() {
        let (tx, mut rx) = command_channel(|| {});
        let mut host = host(Retrigger::Stack);
        tx.try_send(envelope("stop", PlaybackCommand::StopAll))
            .unwrap();
        rx.drain(&mut host);
        tx.try_send(play("new")).unwrap();
        tx.try_send(envelope("stop", PlaybackCommand::StopAll))
            .unwrap();
        assert!(
            rx.drain(&mut host)
                .iter()
                .all(|r| matches!(r.outcome, CommandOutcome::Accepted { .. }))
        );
        tx.try_send(play("newer")).unwrap();
        let mut wrong = envelope("wrong-stop", PlaybackCommand::StopAll);
        wrong.session_id = "old-session".into();
        tx.try_send(wrong).unwrap();
        rx.drain(&mut host);
        assert_eq!(host.instances.len(), 2);
    }

    #[test]
    fn guarded_remote_stop_barriers_respect_liveness_and_distinct_receipts() {
        let mut host = host(Retrigger::Stack);
        let (tx, mut rx) = command_channel(|| {});
        tx.try_send(play("local-before-dead-stop")).unwrap();
        tx.try_send_guarded(
            envelope("dead-remote-stop", PlaybackCommand::StopAll),
            11,
            Arc::new(|| false),
        )
        .unwrap();
        let receipts = rx.drain(&mut host);
        assert!(matches!(
            receipts[0].outcome,
            CommandOutcome::Rejected {
                reason: Rejection::Superseded
            }
        ));
        assert_eq!(receipts[0].ticket, Some(11));
        assert!(matches!(
            receipts[1].outcome,
            CommandOutcome::Accepted { .. }
        ));
        tx.try_send(play("local-before-live-stop")).unwrap();
        tx.try_send_guarded(
            envelope("live-stop", PlaybackCommand::StopAll),
            12,
            Arc::new(|| true),
        )
        .unwrap();
        let receipts = rx.drain(&mut host);
        assert!(matches!(
            receipts[1].outcome,
            CommandOutcome::Rejected {
                reason: Rejection::Superseded
            }
        ));
        tx.try_send(play("local-after-stop")).unwrap();
        tx.try_send_guarded(
            envelope("live-stop", PlaybackCommand::StopAll),
            13,
            Arc::new(|| true),
        )
        .unwrap();
        let receipts = rx.drain(&mut host);
        assert_eq!(receipts[0].ticket, Some(13));
        assert!(matches!(
            receipts[1].outcome,
            CommandOutcome::Accepted { .. }
        ));
    }

    #[test]
    fn histories_dedupe_effects_and_events_stay_bounded() {
        let mut host = host(Retrigger::Stack);
        for n in 0..1300 {
            host.accept(play(&n.to_string()));
            let token = host.instances.values().next().unwrap().token();
            signal(
                &mut host,
                &token,
                BackendEventKind::Failed {
                    message: "failure".into(),
                },
            );
            assert!(host.effects.is_empty());
        }
        assert_eq!(host.dedupe.len(), DEDUPE_CAP);
        assert_eq!(host.dedupe_order.len(), DEDUPE_CAP);
        assert_eq!(host.history.len(), HISTORY_CAP);
        assert_eq!(host.events.len(), EVENT_CAP);
        assert_eq!(host.sequence, 3900);
        assert_eq!(host.drain_events()[0].sequence, 3900 - EVENT_CAP as u64 + 1);
        assert!(!host.dedupe.contains_key("0"));
    }
}
