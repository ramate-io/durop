//! Serialized host authority; native resources stay behind generation-tagged effects.
#![deny(missing_docs)]

use durop_catalog::{Anchor, Catalog, Clip, Retrigger};
use durop_protocol::{
    ClipStatus, CommandOutcome, Envelope, EventKind, HostEvent, HostSnapshot, InstanceSnapshot,
    InstanceState, PROTOCOL_VERSION, PlaybackCommand, Readiness, Rejection, VisualPlacement,
};
use std::{
    collections::{BTreeMap, VecDeque},
    sync::Arc,
};

/// Maximum active occurrences, including workers still stopping.
pub const CONCURRENCY_CAP: usize = 8;
/// Maximum remembered command outcomes per host lifetime.
pub const DEDUPE_CAP: usize = 1024;
/// Maximum terminal occurrences retained for reconnect snapshots.
pub const HISTORY_CAP: usize = 256;
/// Maximum undrained host events retained in memory.
pub const EVENT_CAP: usize = 256;

/// Occurrence and generation guard carried across the backend boundary.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BackendToken {
    /// Occurrence whose resources the backend owns.
    pub instance_id: String,
    /// Generation that must still match when the backend reports completion.
    pub generation: u64,
}

/// Deferred native actions emitted only after serialized host policy decisions.
#[derive(Debug, Clone)]
pub enum BackendEffect {
    /// Prepare immutable clip resources without starting playback.
    Prepare {
        /// Guard identifying the occurrence and generation for this action.
        token: BackendToken,
        /// Pinned catalog metadata and local resources to prepare.
        clip: Box<Clip>,
        /// Explicit occurrence override, or none to use the catalog default.
        placement: Option<VisualPlacement>,
    },
    /// Change presentation on the next frame without replacing decoder resources.
    SetVisual {
        /// Guard identifying the occurrence and generation for this action.
        token: BackendToken,
        /// Explicit occurrence override, or none to restore the catalog default.
        placement: Option<VisualPlacement>,
    },
    /// Start an already prepared occurrence.
    Start {
        /// Guard identifying the occurrence and generation for this action.
        token: BackendToken,
    },
    /// Cancel work and acknowledge only after native cleanup.
    Stop {
        /// Guard identifying the occurrence and generation for this action.
        token: BackendToken,
    },
}

/// Generation-tagged backend observation returned to the host authority.
#[derive(Debug, Clone)]
pub struct BackendEvent {
    /// Occurrence guard copied from the initiating backend effect.
    pub token: BackendToken,
    /// Observed native lifecycle transition.
    pub event: BackendEventKind,
}

/// Backend acknowledgements accepted by the serialized host state machine.
#[derive(Debug, Clone)]
pub enum BackendEventKind {
    /// Resources are ready; the host may now request playback start.
    Prepared,
    /// Playback start was acknowledged by the backend.
    Started,
    /// Playback completed and its resources have been released.
    Finished,
    /// Requested cancellation completed and released resources.
    Stopped,
    /// Preparation or playback ended unsuccessfully.
    Failed {
        /// Backend diagnostic associated with this terminal failure.
        message: String,
    },
}

/// Host-private lifecycle bookkeeping for one active occurrence.
struct Instance {
    /// Observable identity, generation and lifecycle state; only the host mutates it.
    snapshot: InstanceSnapshot,
    /// Whether matching backend preparation was acknowledged before start admission.
    prepared: bool,
    /// Older restarting occurrences whose cleanup must finish before this one starts.
    blockers: Vec<String>,
    /// Prevents emitting duplicate Start effects while awaiting acknowledgement.
    start_requested: bool,
}

impl Instance {
    /// Capture the occurrence guard attached to outgoing native work.
    fn token(&self) -> BackendToken {
        BackendToken {
            instance_id: self.snapshot.instance_id.clone(),
            generation: self.snapshot.generation,
        }
    }
}

/// Single serialized policy authority over a pinned catalog and playback session.
pub struct Host {
    /// Fresh lifetime identity used to reject envelopes from older hosts.
    session_id: String,
    /// Immutable revision pinned for every occurrence in this session.
    catalog: Arc<Catalog>,
    /// Active occurrences, including stopping workers that still consume capacity.
    instances: BTreeMap<String, Instance>,
    /// Terminal snapshots retained in completion order up to HISTORY_CAP.
    history: VecDeque<InstanceSnapshot>,
    /// Latest preparation result keyed by stable catalog clip identity.
    readiness: BTreeMap<String, Readiness>,
    /// Remembered command decisions; replay returns these without new effects.
    dedupe: BTreeMap<String, CommandOutcome>,
    /// Insertion order used to evict the oldest remembered command at DEDUPE_CAP.
    dedupe_order: VecDeque<String>,
    /// Ordered observations retained up to EVENT_CAP until the application drains them.
    events: VecDeque<HostEvent>,
    /// Deferred backend actions; admission bounds pending work before adding more.
    effects: VecDeque<BackendEffect>,
    /// Monotonic event sequence, independent of controller command identifiers.
    sequence: u64,
    /// Session-local occurrence counter; identities never reuse an earlier value.
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

    /// Resolve immutable catalog placement for GIF/video clips; audio has no placement.
    /// Missing visual metadata uses a centered fit box at 30% of display width.
    /// Fractional authoring widths are rounded to the nearest valid basis point.
    pub fn default_placement(clip: &Clip) -> Option<VisualPlacement> {
        let extension = clip.asset_path.extension()?.to_str()?.to_ascii_lowercase();
        if !["gif", "mp4", "mov", "webm"].contains(&extension.as_str()) {
            return None;
        }
        let Some(visual) = &clip.metadata.visual else {
            return Some(VisualPlacement {
                width_bps: 3000,
                x_bps: 5000,
                y_bps: 5000,
                fit: durop_protocol::VisualFit::Catalog,
            });
        };
        let (x_bps, y_bps) = match visual.anchor {
            Anchor::TopLeft => (0, 0),
            Anchor::TopRight => (10_000, 0),
            Anchor::BottomLeft => (0, 10_000),
            Anchor::BottomRight => (10_000, 10_000),
            Anchor::Center => (5000, 5000),
        };
        Some(VisualPlacement {
            width_bps: (visual.width_fraction * 10_000.0)
                .round()
                .clamp(1.0, 10_000.0) as u16,
            x_bps,
            y_bps,
            fit: durop_protocol::VisualFit::Catalog,
        })
    }

    /// Validate, deduplicate and apply one controller intention.
    pub fn accept(&mut self, envelope: Envelope) -> CommandOutcome {
        self.accept_inner(envelope, false)
    }

    /// Apply admission ordering while retaining outcomes for idempotent retries.
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

    /// Resolve a command against bounded active/history state before emitting effects.
    fn decide(&mut self, command: &PlaybackCommand) -> Result<Option<String>, Rejection> {
        match command {
            PlaybackCommand::Play { clip_id } | PlaybackCommand::PlayVisual { clip_id, .. } => {
                let clip = self
                    .catalog
                    .clips
                    .iter()
                    .find(|c| &c.identity == clip_id)
                    .ok_or(Rejection::UnknownClip)?;
                if let PlaybackCommand::PlayVisual { placement, .. } = command
                    && (!placement.is_valid() || Self::default_placement(clip).is_none())
                {
                    return Err(Rejection::InvalidCommand);
                }
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
            PlaybackCommand::SetVisual {
                instance_id,
                placement,
            } => {
                if placement.is_some_and(|value| !value.is_valid()) {
                    return Err(Rejection::InvalidCommand);
                }
                let instance = self.instances.get(instance_id).ok_or_else(|| {
                    if self
                        .history
                        .iter()
                        .any(|value| value.instance_id == *instance_id)
                    {
                        Rejection::Superseded
                    } else {
                        Rejection::UnknownInstance
                    }
                })?;
                if !matches!(
                    instance.snapshot.state,
                    InstanceState::Preparing | InstanceState::Playing
                ) {
                    return Err(Rejection::Superseded);
                }
                let clip = self
                    .catalog
                    .clips
                    .iter()
                    .find(|clip| clip.identity == instance.snapshot.clip_id)
                    .unwrap();
                if Self::default_placement(clip).is_none() {
                    return Err(Rejection::InvalidCommand);
                }
                // Coalescing keeps one unsent presentation change per generation.
                if self.effects.len() >= 64 && !self.effects.iter().any(|effect| {
                    matches!(effect, BackendEffect::SetVisual { token, .. } if *token == instance.token())
                }) {
                    return Err(Rejection::Busy);
                }
                Ok(Some(instance_id.clone()))
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

    /// Apply an accepted policy decision and enqueue generation-guarded effects.
    fn perform(&mut self, command: PlaybackCommand, instance_id: Option<String>) {
        let placement = match &command {
            PlaybackCommand::PlayVisual { placement, .. } => Some(*placement),
            _ => None,
        };
        match command {
            PlaybackCommand::Play { clip_id } | PlaybackCommand::PlayVisual { clip_id, .. } => {
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
                        placement,
                    },
                    prepared: false,
                    blockers,
                    start_requested: false,
                };
                self.effects.push_back(BackendEffect::Prepare {
                    token: instance.token(),
                    clip: Box::new(clip),
                    placement,
                });
                self.instances.insert(instance_id.clone(), instance);
                self.emit(Some(instance_id.clone()), EventKind::Preparing);
                if placement.is_some() {
                    self.emit(Some(instance_id), EventKind::VisualChanged { placement });
                }
            }
            PlaybackCommand::SetVisual {
                instance_id,
                placement,
            } => {
                let instance = self.instances.get_mut(&instance_id).unwrap();
                instance.snapshot.placement = placement;
                let token = instance.token();
                self.effects.retain(|effect| {
                    !matches!(effect,
                        BackendEffect::SetVisual { token: queued, .. } if queued == &token
                    )
                });
                self.effects
                    .push_back(BackendEffect::SetVisual { token, placement });
                self.emit(Some(instance_id), EventKind::VisualChanged { placement });
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

    /// Request cancellation once while retaining the occurrence until backend cleanup.
    fn stop(&mut self, id: &str) {
        if let Some(instance) = self.instances.get_mut(id)
            && instance.snapshot.state != InstanceState::Stopping
        {
            instance.snapshot.state = InstanceState::Stopping;
            let token = instance.token();
            // Cancel unsent work too. Every stop still gets a teardown acknowledgement.
            self.effects.retain(|effect| match effect {
                BackendEffect::Prepare { token: t, .. }
                | BackendEffect::SetVisual { token: t, .. }
                | BackendEffect::Start { token: t } => t != &token,
                BackendEffect::Stop { .. } => true,
            });
            self.effects.push_back(BackendEffect::Stop { token });
        }
    }

    /// Callback tokens must match the worker's original occurrence and generation.
    /// Advance lifecycle only when the backend occurrence and generation still match.
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

    /// Move a terminal occurrence into bounded history and unblock restart dependents.
    fn terminal(&mut self, id: &str, state: InstanceState, event: EventKind) {
        let Some(mut instance) = self.instances.remove(id) else {
            return;
        };
        let token = instance.token();
        self.effects.retain(|effect| match effect {
            BackendEffect::Prepare { token: t, .. }
            | BackendEffect::SetVisual { token: t, .. }
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

    /// Record an ordered rejection event without issuing backend work.
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

    /// Append a monotonically sequenced observation, retaining the bounded event tail.
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

    /// Return a bounded recovery view of readiness, active instances and history.
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

    /// Return the fresh identity shared by this host and its controller envelopes.
    pub fn session_id(&self) -> &str {
        &self.session_id
    }

    /// Return the latest emitted event sequence.
    pub fn sequence(&self) -> u64 {
        self.sequence
    }

    /// Consume ordered events accumulated since the previous application update.
    pub fn drain_events(&mut self) -> Vec<HostEvent> {
        self.events.drain(..).collect()
    }

    /// Consume native actions to execute outside serialized policy handling.
    pub fn drain_effects(&mut self) -> Vec<BackendEffect> {
        self.effects.drain(..).collect()
    }
}

/// Bounded shared ingress. The wake callback must notify the application event loop; it
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

/// One admitted ingress item with stop-ordering and optional remote liveness.
struct Queued {
    /// Validated-at-consumption command; queue admission itself grants no host authority.
    envelope: Envelope,
    /// Stop ordering at admission; earlier ordinary work is superseded by a newer barrier.
    epoch: u64,
    /// Optional remote reply correlation, absent for local fire-and-observe submissions.
    ticket: Option<u64>,
    /// Optional connection lease checked immediately before the host applies the command.
    active: Option<Arc<dyn Fn() -> bool + Send + Sync>>,
}

/// Mutex-protected sender state keeps stop epochs ordered with queue admission.
struct Submit {
    /// Bounded ordinary lane; saturation cannot consume emergency capacity.
    normal_tx: tokio::sync::mpsc::Sender<Queued>,
    /// Reserved bounded lane for StopAll requests.
    priority_tx: tokio::sync::mpsc::Sender<Queued>,
    /// Last successfully queued stop barrier, advanced atomically with admission.
    epoch: u64,
}

/// Cloneable bounded ingress shared by local and authenticated remote controllers.
#[derive(Clone)]
pub struct CommandSender {
    /// Shared lock orders queue insertion and stop-epoch changes across all producers.
    inner: Arc<std::sync::Mutex<Submit>>,
    /// Nonblocking application wake callback invoked only after successful admission.
    wake: Arc<dyn Fn() + Send + Sync>,
}

/// Queue admission failure; no playback policy has been applied yet.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QueueError {
    /// The selected bounded queue has no remaining capacity.
    Busy,
    /// The sole host receiver has been dropped.
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

    /// Atomically admit a command and advance the stop epoch only after queue success.
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

/// Single host-side receiver with separate emergency and ordinary queues.
pub struct CommandInbox {
    /// Sole consumer of ordinary controller intentions.
    normal_rx: tokio::sync::mpsc::Receiver<Queued>,
    /// Emergency lane drained before ordinary commands on every update.
    priority_rx: tokio::sync::mpsc::Receiver<Queued>,
    /// Newest effective stop barrier; older ordinary commands are rejected as superseded.
    cutoff: u64,
    /// Requests another application update when bounded draining leaves pending work.
    wake: Arc<dyn Fn() + Send + Sync>,
}

/// Host outcome correlated with its command and optional remote reply ticket.
#[derive(Debug)]
pub struct CommandReceipt {
    /// Controller command identity used for matching acknowledgements.
    pub command_id: String,
    /// Serialized host decision for the queued operation.
    pub outcome: CommandOutcome,
    /// Remote reply ticket; absent for local fire-and-observe commands.
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

    /// Whether another bounded drain should wake the application loop.
    pub fn has_pending(&self) -> bool {
        !self.priority_rx.is_empty() || !self.normal_rx.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use durop_catalog::{ClipMetadata, DecoderReadiness, PackManifest, Playback};
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
            protocol_version: PROTOCOL_VERSION,
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
                BackendEffect::Prepare { token, clip, .. } => {
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
                _ => request.protocol_version = PROTOCOL_VERSION + 1,
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

    /// A visual fixture keeps authoring metadata immutable across occurrence edits.
    fn visual_host(policy: Retrigger) -> Host {
        let mut host = host(policy);
        Arc::make_mut(&mut host.catalog).clips[0].asset_path = "clip.gif".into();
        host
    }

    /// Asymmetric explicit Cover detects loss of fitting intent as well as coordinate swaps.
    fn placement() -> VisualPlacement {
        VisualPlacement {
            width_bps: 4200,
            x_bps: 1250,
            y_bps: 8750,
            fit: durop_protocol::VisualFit::Cover,
        }
    }

    /// Submit one occurrence-only visual request through the usual envelope guards.
    fn play_visual(id: &str) -> Envelope {
        envelope(
            id,
            PlaybackCommand::PlayVisual {
                clip_id: "pack/clip".into(),
                placement: placement(),
            },
        )
    }

    /// Submit one edit or reset without changing the occurrence identity.
    fn set_visual(id: &str, token: &BackendToken, placement: Option<VisualPlacement>) -> Envelope {
        envelope(
            id,
            PlaybackCommand::SetVisual {
                instance_id: token.instance_id.clone(),
                placement,
            },
        )
    }

    #[test]
    fn defaults_map_visual_metadata_and_reject_audio_without_mutating_catalog() {
        use durop_catalog::{Fit, Visual};
        let host = visual_host(Retrigger::Stack);
        let mut clip = host.catalog.clips[0].clone();
        assert_eq!(
            Host::default_placement(&clip),
            Some(VisualPlacement {
                width_bps: 3000,
                x_bps: 5000,
                y_bps: 5000,
                fit: durop_protocol::VisualFit::Catalog,
            })
        );
        for (anchor, x_bps, y_bps) in [
            (Anchor::TopLeft, 0, 0),
            (Anchor::TopRight, 10_000, 0),
            (Anchor::BottomLeft, 0, 10_000),
            (Anchor::BottomRight, 10_000, 10_000),
            (Anchor::Center, 5000, 5000),
        ] {
            clip.metadata.visual = Some(Visual {
                anchor,
                width_fraction: 0.4567,
                fit: Fit::Cover,
            });
            assert_eq!(
                Host::default_placement(&clip),
                Some(VisualPlacement {
                    width_bps: 4567,
                    x_bps,
                    y_bps,
                    fit: durop_protocol::VisualFit::Catalog,
                })
            );
        }
        for extension in ["gif", "GIF", "mp4", "mov", "webm"] {
            clip.asset_path = format!("clip.{extension}").into();
            assert!(Host::default_placement(&clip).is_some());
        }
        clip.asset_path = "clip.wav".into();
        assert_eq!(Host::default_placement(&clip), None);
        assert!(host.catalog.clips[0].metadata.visual.is_none());
    }

    #[test]
    fn visual_edit_preparing_playing_and_reset_keep_decoder_generation() {
        let mut host = visual_host(Retrigger::Stack);
        let token = prepared_token(&mut host, "play");
        let edit = set_visual("edit", &token, Some(placement()));
        let outcome = host.accept(edit.clone());
        let sequence = host.sequence;
        assert_eq!(host.accept(edit), outcome);
        assert_eq!(host.sequence, sequence);
        assert_eq!(host.instances.len(), 1);
        assert_eq!(host.snapshot().instances[0].generation, token.generation);
        assert_eq!(host.snapshot().instances[0].placement, Some(placement()));
        assert!(
            matches!(host.drain_effects().as_slice(), [BackendEffect::SetVisual { token: actual, placement: Some(value) }] if actual == &token && *value == placement())
        );
        assert!(host.drain_events().iter().any(|event| matches!(event.event, EventKind::VisualChanged { placement: Some(value) } if value == placement())));
        start(&mut host, &token);
        host.accept(set_visual("reset", &token, None));
        assert_eq!(host.snapshot().instances[0].placement, None);
        assert_eq!(host.snapshot().instances[0].state, InstanceState::Playing);
        assert!(
            matches!(host.drain_effects().as_slice(), [BackendEffect::SetVisual { token: actual, placement: None }] if actual == &token)
        );
        assert!(host.catalog.clips[0].metadata.visual.is_none());
    }

    #[test]
    fn visual_play_obeys_ignore_stack_and_restart_policies() {
        let mut ignore = visual_host(Retrigger::IgnoreIfPlaying);
        let first = ignore.accept(play_visual("one"));
        assert_eq!(ignore.accept(play("two")), first);
        assert_eq!(ignore.snapshot().instances[0].placement, Some(placement()));
        assert_eq!(ignore.drain_effects().len(), 1);
        let mut stack = visual_host(Retrigger::Stack);
        assert_ne!(
            stack.accept(play_visual("one")),
            stack.accept(play_visual("two"))
        );
        assert_eq!(stack.instances.len(), 2);
        let mut restart = visual_host(Retrigger::Restart);
        let old = prepared_token(&mut restart, "old");
        restart.accept(play_visual("new"));
        let effects = restart.drain_effects();
        assert!(
            matches!(effects.as_slice(), [BackendEffect::Stop { token }, BackendEffect::Prepare { placement: Some(value), .. }] if token == &old && *value == placement())
        );
        assert_eq!(
            restart.instances[&old.instance_id].snapshot.state,
            InstanceState::Stopping
        );
    }

    #[test]
    fn visual_commands_reject_audio_invalid_bounds_and_stale_context() {
        let mut audio = host(Retrigger::Stack);
        assert_eq!(
            audio.accept(play_visual("audio")),
            CommandOutcome::Rejected {
                reason: Rejection::InvalidCommand
            }
        );
        let audio_token = prepared_token(&mut audio, "audio-default");
        assert_eq!(
            audio.accept(set_visual("audio-edit", &audio_token, None)),
            CommandOutcome::Rejected {
                reason: Rejection::InvalidCommand
            }
        );
        let mut host = visual_host(Retrigger::Stack);
        for (index, value) in [
            VisualPlacement {
                width_bps: 0,
                ..placement()
            },
            VisualPlacement {
                width_bps: 10_001,
                ..placement()
            },
            VisualPlacement {
                x_bps: 10_001,
                ..placement()
            },
            VisualPlacement {
                y_bps: 10_001,
                ..placement()
            },
        ]
        .into_iter()
        .enumerate()
        {
            let request = envelope(
                &format!("bad-play-{index}"),
                PlaybackCommand::PlayVisual {
                    clip_id: "pack/clip".into(),
                    placement: value,
                },
            );
            assert_eq!(
                host.accept(request),
                CommandOutcome::Rejected {
                    reason: Rejection::InvalidCommand
                }
            );
        }
        let token = prepared_token(&mut host, "valid");
        assert_eq!(
            host.accept(set_visual(
                "invalid-edit",
                &token,
                Some(VisualPlacement {
                    width_bps: 0,
                    ..placement()
                })
            )),
            CommandOutcome::Rejected {
                reason: Rejection::InvalidCommand
            }
        );
        let missing = BackendToken {
            instance_id: "session:missing".into(),
            generation: 999,
        };
        assert_eq!(
            host.accept(set_visual("missing-edit", &missing, None)),
            CommandOutcome::Rejected {
                reason: Rejection::UnknownInstance
            }
        );
        for (id, reason) in [
            ("session", Rejection::WrongSession),
            ("revision", Rejection::StaleCatalog),
        ] {
            let mut request = set_visual(id, &token, Some(placement()));
            if id == "session" {
                request.session_id = "old".into();
            } else {
                request.catalog_revision = "old".into();
            }
            assert_eq!(host.accept(request), CommandOutcome::Rejected { reason });
        }
        assert_eq!(host.snapshot().instances[0].placement, None);
        assert!(host.drain_effects().is_empty());
    }

    #[test]
    fn visual_effects_coalesce_and_stop_discards_queued_edits() {
        let mut host = visual_host(Retrigger::Stack);
        let token = prepared_token(&mut host, "play");
        for index in 0..100 {
            assert!(matches!(
                host.accept(set_visual(
                    &format!("edit-{index}"),
                    &token,
                    Some(VisualPlacement {
                        x_bps: index,
                        ..placement()
                    })
                )),
                CommandOutcome::Accepted { .. }
            ));
        }
        assert_eq!(host.effects.len(), 1);
        host.accept(envelope("stop", PlaybackCommand::StopAll));
        assert!(
            matches!(host.drain_effects().as_slice(), [BackendEffect::Stop { token: actual }] if actual == &token)
        );
        assert_eq!(
            host.accept(set_visual("late", &token, Some(placement()))),
            CommandOutcome::Rejected {
                reason: Rejection::Superseded
            }
        );
        signal(&mut host, &token, BackendEventKind::Prepared);
        assert!(host.drain_effects().is_empty());
        signal(&mut host, &token, BackendEventKind::Stopped);
        assert_eq!(
            host.accept(set_visual("terminal", &token, None)),
            CommandOutcome::Rejected {
                reason: Rejection::Superseded
            }
        );
        assert!(host.drain_effects().is_empty());
    }

    #[test]
    fn stop_all_supersedes_queued_visual_play_and_edits() {
        let mut host = visual_host(Retrigger::Stack);
        let token = prepared_token(&mut host, "existing");
        let (sender, mut inbox) = command_channel(|| {});
        sender.try_send(play_visual("queued-play")).unwrap();
        sender
            .try_send(set_visual("queued-edit", &token, Some(placement())))
            .unwrap();
        sender
            .try_send(envelope("barrier", PlaybackCommand::StopAll))
            .unwrap();
        let receipts = inbox.drain(&mut host);
        assert_eq!(receipts.len(), 3);
        assert!(receipts[1..].iter().all(|receipt| receipt.outcome
            == CommandOutcome::Rejected {
                reason: Rejection::Superseded
            }));
        assert_eq!(host.instances.len(), 1);
        assert_eq!(host.snapshot().instances[0].state, InstanceState::Stopping);
        assert_eq!(host.snapshot().instances[0].placement, None);
        assert!(
            matches!(host.drain_effects().as_slice(), [BackendEffect::Stop { token: actual }] if actual == &token)
        );
    }

    #[test]
    fn natural_completion_discards_pending_visual_effects_and_retains_override_history() {
        let mut host = visual_host(Retrigger::Stack);
        let token = prepared_token(&mut host, "play");
        start(&mut host, &token);
        host.accept(set_visual("edit", &token, Some(placement())));
        signal(&mut host, &token, BackendEventKind::Finished);
        assert!(host.drain_effects().is_empty());
        assert_eq!(host.snapshot().instances[0].state, InstanceState::Finished);
        assert_eq!(host.snapshot().instances[0].placement, Some(placement()));
        assert_eq!(
            host.accept(set_visual("terminal-edit", &token, None)),
            CommandOutcome::Rejected {
                reason: Rejection::Superseded
            }
        );
    }
}
