//! Opt-in, bounded, privacy-preserving observation points. These are not output proofs.
//! Formatting and try_send are bounded on the caller; filesystem I/O runs on one thread.
use crate::commands::{CommandOutcome, Envelope, EventKind, HostEvent};
use std::{
    collections::{BTreeMap, VecDeque},
    fs::{File, OpenOptions},
    io::{self, Write},
    path::Path,
    sync::{
        Arc, Mutex, OnceLock,
        atomic::{AtomicBool, AtomicU64, Ordering},
        mpsc::{self, SyncSender},
    },
    thread::JoinHandle,
    time::{Duration, Instant},
};
use tracing_subscriber::fmt::MakeWriter;

/// Maximum capture size, including the reserved completion footer.
const FILE_LIMIT: usize = 16 * 1024 * 1024;

/// Maximum serialized size of one complete JSON event.
const EVENT_LIMIT: usize = 4096;

/// Reserve enough space for the final loss/completion counters.
const FOOTER_RESERVE: usize = 1024;

/// Maximum live command and instance correlations retained in memory.
const TRACE_LIMIT: usize = 256;

/// Process-wide opt-in recorder; caller observation points become no-ops when absent.
static RECORDER: OnceLock<Recorder> = OnceLock::new();

/// Identify the command ingress without storing private addresses or credentials.
#[derive(Clone, Copy)]
pub enum Origin {
    /// Command submitted by this application’s local UI.
    Local,
    /// Command handed off from an authenticated remote connection.
    Remote,
}

impl Origin {
    /// Return the stable public label used in aggregate timing records.
    fn label(self) -> &'static str {
        match self {
            Self::Local => "local",
            Self::Remote => "remote_host_handoff",
        }
    }
}

/// Collect loss and disk errors without blocking timing observation points.
#[derive(Default)]
struct Counters {
    /// Whole records discarded by size, capacity or capture bounds.
    dropped: AtomicU64,
    /// Correlations removed to preserve the fixed trace budget.
    evicted: AtomicU64,
    /// Disk-worker errors encountered during capture.
    writer_errors: AtomicU64,
    /// Whether the file budget has been exhausted.
    capped: AtomicBool,
}

/// Bounded disk-worker messages containing whole events or a flush request.
enum WriterCommand {
    /// Write a complete serialized record if space remains.
    Event(Vec<u8>),
    /// Flush the final summary and return a completion acknowledgment.
    Stop(mpsc::Sender<bool>),
}

/// Create per-event buffers sharing one bounded writer queue.
#[derive(Clone)]
struct EventFactory {
    /// Nonblocking ingress to the bounded disk-worker queue.
    sender: SyncSender<WriterCommand>,
    /// Shared capture loss/error accounting.
    counters: Arc<Counters>,
}

/// Accumulate one JSON event and discard it whole if size or queue bounds are exceeded.
struct EventBuffer {
    /// Queue and counters inherited from the recorder.
    factory: EventFactory,
    /// Serialized bytes for this one event.
    bytes: Vec<u8>,
    /// Reject the entire event if any write exceeds its byte limit.
    overflowed: bool,
}

impl<'a> MakeWriter<'a> for EventFactory {
    type Writer = EventBuffer;
    fn make_writer(&'a self) -> Self::Writer {
        EventBuffer {
            factory: self.clone(),
            bytes: Vec::with_capacity(512),
            overflowed: false,
        }
    }
}

impl Write for EventBuffer {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        if self.bytes.len().saturating_add(bytes.len()) > EVENT_LIMIT {
            self.overflowed = true;
        }
        if !self.overflowed {
            self.bytes.extend_from_slice(bytes);
        }
        Ok(bytes.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl Drop for EventBuffer {
    fn drop(&mut self) {
        if self.overflowed
            || self.factory.counters.capped.load(Ordering::Relaxed)
            || self
                .factory
                .sender
                .try_send(WriterCommand::Event(std::mem::take(&mut self.bytes)))
                .is_err()
        {
            self.factory
                .counters
                .dropped
                .fetch_add(1, Ordering::Relaxed);
        }
    }
}

/// Correlate a submitted command privately until its acknowledgment arrives.
struct CommandTrace {
    /// Private correlation session; never emitted verbatim.
    session: String,
    /// Private correlation command; never emitted verbatim.
    command: String,
    /// Same-process monotonic submission time.
    start: Instant,
    /// Public numeric trace identifier.
    id: u64,
    /// Ingress category for aggregate timing.
    origin: Origin,
}

/// Track accepted playback until its first frame and terminal callback.
struct InstanceTrace {
    /// Original command submission time on this host.
    start: Instant,
    /// Public numeric trace identifier inherited from the command.
    id: u64,
    /// Original ingress category.
    origin: Origin,
    /// Whether the first render submission has already been recorded.
    first_frame: bool,
}

/// Bound the in-memory correlations used to derive aggregate timings.
#[derive(Default)]
struct TraceState {
    /// Bounded pending acknowledgments ordered by submission.
    commands: VecDeque<CommandTrace>,
    /// Bounded accepted playback correlations.
    instances: BTreeMap<String, InstanceTrace>,
}

/// Own the tracing dispatcher, bounded correlation state and loss accounting.
struct Recorder {
    /// Private tracing subscriber for the capture file.
    dispatch: tracing::Dispatch,
    /// Protected bounded correlation maps.
    state: Mutex<TraceState>,
    /// Next public numeric trace identifier.
    next: AtomicU64,
    /// Shared loss and disk-error accounting.
    counters: Arc<Counters>,
    /// Disable observation before requesting final flush.
    enabled: Arc<AtomicBool>,
}

/// Hold through App::run. Drop attempts a final summary/flush for at most two seconds,
/// after rendering has ended. A blocked disk never blocks the render thread's writes.
pub struct DiagnosticsGuard {
    /// Shutdown route to the disk worker, consumed once.
    sender: Option<SyncSender<WriterCommand>>,
    /// Worker handle detached rather than blocking on an unhealthy filesystem.
    writer: Option<JoinHandle<()>>,
    /// Shared switch preventing new records during shutdown.
    enabled: Arc<AtomicBool>,
}

impl DiagnosticsGuard {
    /// Disable further capture and request a bounded final summary/flush.
    pub fn finish(mut self) -> bool {
        self.finish_inner()
    }

    /// Attempt idempotent shutdown within two seconds without joining a potentially blocked disk thread.
    fn finish_inner(&mut self) -> bool {
        self.enabled.store(false, Ordering::Release);
        let Some(sender) = self.sender.take() else {
            return true;
        };
        let (ack, complete) = mpsc::channel();
        let deadline = Instant::now() + Duration::from_secs(2);
        let mut command = WriterCommand::Stop(ack);
        loop {
            match sender.try_send(command) {
                Ok(()) => break,
                Err(mpsc::TrySendError::Full(returned)) if Instant::now() < deadline => {
                    command = returned;
                    std::thread::sleep(Duration::from_millis(1));
                }
                _ => return false,
            }
        }
        let finished = complete
            .recv_timeout(deadline.saturating_duration_since(Instant::now()))
            .unwrap_or(false);
        // Do not join: even closing a file may stall on an unhealthy filesystem.
        // The worker has acknowledged the final write; detaching keeps shutdown bounded.
        if finished {
            self.writer.take();
        }
        finished
    }
}

impl Drop for DiagnosticsGuard {
    fn drop(&mut self) {
        let _ = self.finish_inner();
    }
}

/// Create a new private capture file and enable process-wide observation once.
pub fn start(path: &Path) -> io::Result<DiagnosticsGuard> {
    if RECORDER.get().is_some() {
        return Err(io::Error::new(
            io::ErrorKind::AlreadyExists,
            "diagnostics already configured",
        ));
    }
    let (recorder, guard) = Recorder::create(path, FILE_LIMIT)?;
    RECORDER.set(recorder).map_err(|_| {
        io::Error::new(
            io::ErrorKind::AlreadyExists,
            "diagnostics already configured",
        )
    })?;
    let recorder = RECORDER.get().expect("just initialized recorder");
    tracing::dispatcher::with_default(
        &recorder.dispatch,
        || tracing::info!(target: "durop_timing", stage = "diagnostics_start", schema_version = 1, file_limit_bytes = FILE_LIMIT, event_limit_bytes = EVENT_LIMIT),
    );
    Ok(guard)
}

impl Recorder {
    /// Build the bounded recorder and disk worker for a newly created capture file.
    fn create(path: &Path, limit: usize) -> io::Result<(Recorder, DiagnosticsGuard)> {
        let mut options = OpenOptions::new();
        options.write(true).create_new(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            options.mode(0o600);
        }
        let file = options.open(path)?;
        let counters = Arc::new(Counters::default());
        let writer_counters = counters.clone();
        let (sender, receiver) = mpsc::sync_channel(256);
        let writer = std::thread::Builder::new()
            .name("durop-diagnostics".into())
            .spawn(move || Recorder::write_events(file, receiver, writer_counters, limit))?;
        let subscriber = tracing_subscriber::fmt()
            .json()
            .with_current_span(false)
            .with_span_list(false)
            .with_ansi(false)
            .with_max_level(tracing::Level::INFO)
            .with_writer(EventFactory {
                sender: sender.clone(),
                counters: counters.clone(),
            })
            .finish();
        let enabled = Arc::new(AtomicBool::new(true));
        Ok((
            Recorder {
                dispatch: tracing::Dispatch::new(subscriber),
                state: Mutex::new(TraceState::default()),
                next: AtomicU64::new(1),
                counters,
                enabled: enabled.clone(),
            },
            DiagnosticsGuard {
                sender: Some(sender),
                writer: Some(writer),
                enabled,
            },
        ))
    }

    /// Write complete events up to the file cap and acknowledge a final summary.
    fn write_events(
        mut file: File,
        receiver: mpsc::Receiver<WriterCommand>,
        counters: Arc<Counters>,
        limit: usize,
    ) {
        let mut written = 0;
        for command in receiver {
            match command {
                WriterCommand::Event(bytes) => {
                    if written + bytes.len() > limit.saturating_sub(FOOTER_RESERVE) {
                        counters.capped.store(true, Ordering::Relaxed);
                        counters.dropped.fetch_add(1, Ordering::Relaxed);
                        continue;
                    }
                    if file.write_all(&bytes).is_err() {
                        counters.writer_errors.fetch_add(1, Ordering::Relaxed);
                        counters.capped.store(true, Ordering::Relaxed);
                    } else {
                        written += bytes.len();
                    }
                }
                WriterCommand::Stop(ack) => {
                    let mut footer = serde_json::to_vec(&serde_json::json!({"target":"durop_timing","fields":{"stage":"diagnostics_summary","dropped_events":counters.dropped.load(Ordering::Relaxed),"evicted_traces":counters.evicted.load(Ordering::Relaxed),"writer_errors":counters.writer_errors.load(Ordering::Relaxed),"file_capped":counters.capped.load(Ordering::Relaxed),"bytes_before_summary":written}})).unwrap_or_default();
                    footer.push(b'\n');
                    let ok = written + footer.len() <= limit
                        && file.write_all(&footer).and_then(|_| file.flush()).is_ok();
                    let _ = ack.send(ok);
                    break;
                }
            }
        }
    }

    /// Return the configured recorder only while capture remains enabled.
    fn active() -> Option<&'static Recorder> {
        RECORDER
            .get()
            .filter(|recorder| recorder.enabled.load(Ordering::Acquire))
    }
}

/// Convert elapsed time to saturated microseconds without narrowing overflow.
fn micros(elapsed: Duration) -> u64 {
    u64::try_from(elapsed.as_micros()).unwrap_or(u64::MAX)
}

impl Recorder {
    /// Emit an aggregate timing stage without exposing its private correlation identifiers.
    fn emit(&self, stage: &'static str, id: u64, origin: Origin, elapsed: Duration) {
        tracing::dispatcher::with_default(
            &self.dispatch,
            || tracing::info!(target: "durop_timing", stage, trace_id = id, origin = origin.label(), elapsed_us = micros(elapsed)),
        );
    }

    /// Start bounded private correlation for a submitted command when capture is enabled.
    fn submitted(&self, envelope: &Envelope, origin: Origin) {
        if envelope.session_id.len() > 128 || envelope.command_id.len() > 128 {
            return;
        }
        let mut state = self.state.lock().unwrap();
        if state.commands.len() == TRACE_LIMIT {
            state.commands.pop_front();
            self.counters.evicted.fetch_add(1, Ordering::Relaxed);
        }
        state.commands.push_back(CommandTrace {
            session: envelope.session_id.clone(),
            command: envelope.command_id.clone(),
            start: Instant::now(),
            id: self.next.fetch_add(1, Ordering::Relaxed),
            origin,
        });
    }

    /// Record acceptance or rejection timing and transfer accepted playback correlation.
    fn acknowledged(&self, session: &str, command: &str, outcome: &CommandOutcome) {
        let mut state = self.state.lock().unwrap();
        let Some(position) = state
            .commands
            .iter()
            .position(|trace| trace.session == session && trace.command == command)
        else {
            return;
        };
        let trace = state.commands.remove(position).expect("located command");
        self.emit(
            if matches!(outcome, CommandOutcome::Accepted { .. }) {
                "command_acknowledged"
            } else {
                "command_rejected"
            },
            trace.id,
            trace.origin,
            trace.start.elapsed(),
        );
        if let CommandOutcome::Accepted {
            instance_id: Some(instance),
        } = outcome
        {
            if state.instances.len() == TRACE_LIMIT && !state.instances.contains_key(instance) {
                if let Some(oldest) = state.instances.keys().next().cloned() {
                    state.instances.remove(&oldest);
                }
                self.counters.evicted.fetch_add(1, Ordering::Relaxed);
            }
            state
                .instances
                .entry(instance.clone())
                .or_insert(InstanceTrace {
                    start: trace.start,
                    id: trace.id,
                    origin: trace.origin,
                    first_frame: false,
                });
        }
    }

    /// Record backend lifecycle observations and retire terminal instance correlations.
    fn observe_events(&self, events: &[HostEvent]) {
        let mut state = self.state.lock().unwrap();
        for event in events {
            let Some(instance) = event.instance_id.as_ref() else {
                continue;
            };
            let terminal = matches!(
                event.event,
                EventKind::Finished | EventKind::Stopped | EventKind::Failed { .. }
            );
            if let Some(trace) = state.instances.get(instance) {
                let stage = match event.event {
                    EventKind::Started => Some("backend_started_callback_observed"),
                    EventKind::Finished => Some("finished_callback_observed"),
                    EventKind::Stopped => Some("stopped_callback_observed"),
                    EventKind::Failed { .. } => Some("failed_callback_observed"),
                    _ => None,
                };
                if let Some(stage) = stage {
                    self.emit(stage, trace.id, trace.origin, trace.start.elapsed());
                }
            }
            if terminal {
                state.instances.remove(instance);
            }
        }
    }

    /// Record only the first render submission for an accepted playback instance.
    fn first_frame_submitted(&self, instance: &str) {
        let mut state = self.state.lock().unwrap();
        if let Some(trace) = state.instances.get_mut(instance)
            && !trace.first_frame
        {
            trace.first_frame = true;
            self.emit(
                "first_frame_submitted",
                trace.id,
                trace.origin,
                trace.start.elapsed(),
            );
        }
    }
}

/// Start bounded private correlation for a submitted command when capture is enabled.
pub fn submitted(envelope: &Envelope, origin: Origin) {
    if let Some(recorder) = Recorder::active() {
        recorder.submitted(envelope, origin);
    }
}

/// Record acceptance or rejection timing and transfer accepted playback correlation.
pub fn acknowledged(session: &str, command: &str, outcome: &CommandOutcome) {
    if let Some(recorder) = Recorder::active() {
        recorder.acknowledged(session, command, outcome);
    }
}

/// Record backend lifecycle observations and retire terminal instance correlations.
pub fn observe_events(events: &[HostEvent]) {
    if let Some(recorder) = Recorder::active() {
        recorder.observe_events(events);
    }
}

/// Record only the first render submission for an accepted playback instance.
pub fn first_frame_submitted(instance: &str) {
    if let Some(recorder) = Recorder::active() {
        recorder.first_frame_submitted(instance);
    }
}

/// Controller-local roundtrip only. Never subtract clocks from different machines.
pub fn controller_ack(elapsed: Duration) {
    if let Some(recorder) = Recorder::active() {
        tracing::dispatcher::with_default(
            &recorder.dispatch,
            || tracing::info!(target: "durop_timing", stage = "controller_ack_roundtrip", trace_id = recorder.next.fetch_add(1, Ordering::Relaxed), origin = "controller", elapsed_us = micros(elapsed)),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn path() -> std::path::PathBuf {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        std::env::temp_dir().join(format!(
            "durop-diagnostics-{}-{}.jsonl",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ))
    }
    #[test]
    fn saturated_writer_queue_discards_whole_events_without_waiting() {
        let counters = Arc::new(Counters::default());
        let (sender, receiver) = mpsc::sync_channel(1);
        let factory = EventFactory {
            sender,
            counters: counters.clone(),
        };
        for _ in 0..2 {
            let mut event = factory.make_writer();
            event.write_all(b"{\"stage\":\"test\"}\n").unwrap();
        }
        assert_eq!(counters.dropped.load(Ordering::Relaxed), 1);
        assert!(
            matches!(receiver.try_recv(), Ok(WriterCommand::Event(bytes)) if bytes == b"{\"stage\":\"test\"}\n")
        );
        {
            let mut event = factory.make_writer();
            event.write_all(&vec![b'x'; EVENT_LIMIT + 1]).unwrap();
        }
        assert_eq!(counters.dropped.load(Ordering::Relaxed), 2);
        assert!(receiver.try_recv().is_err());
    }

    #[test]
    fn whole_event_cap_and_private_file_never_write_partial_json() {
        let path = path();
        let (recorder, guard) = Recorder::create(&path, 4096).unwrap();
        for _ in 0..100 {
            recorder.emit(
                "test_observation",
                1,
                Origin::Local,
                Duration::from_micros(12),
            );
        }
        assert!(guard.finish());
        let bytes = std::fs::read(&path).unwrap();
        assert!(bytes.len() <= 4096);
        let rows: Vec<serde_json::Value> = std::str::from_utf8(&bytes)
            .unwrap()
            .lines()
            .map(|line| serde_json::from_str(line).unwrap())
            .collect();
        assert_eq!(
            rows.last().unwrap()["fields"]["stage"],
            "diagnostics_summary"
        );
        assert!(
            rows.last().unwrap()["fields"]["dropped_events"]
                .as_u64()
                .unwrap()
                > 0
        );
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            assert_eq!(
                std::fs::metadata(&path).unwrap().permissions().mode() & 0o777,
                0o600
            );
        }
        assert!(Recorder::create(&path, FILE_LIMIT).is_err());
        std::fs::remove_file(path).unwrap();
    }
    #[test]
    fn timings_correlate_without_logging_private_identifiers_and_maps_are_bounded() {
        let path = path();
        let (recorder, guard) = Recorder::create(&path, FILE_LIMIT).unwrap();
        let envelope = Envelope {
            protocol_version: 1,
            session_id: "PRIVATE_SESSION".into(),
            command_id: "PRIVATE_COMMAND".into(),
            catalog_revision: "PRIVATE_REPOSITORY".into(),
            command: crate::commands::PlaybackCommand::Play {
                clip_id: "PRIVATE_CLIP".into(),
            },
        };
        recorder.submitted(&envelope, Origin::Remote);
        recorder.acknowledged(
            &envelope.session_id,
            &envelope.command_id,
            &CommandOutcome::Accepted {
                instance_id: Some("PRIVATE_INSTANCE".into()),
            },
        );
        recorder.first_frame_submitted("PRIVATE_INSTANCE");
        recorder.first_frame_submitted("PRIVATE_INSTANCE");
        for _ in 0..300 {
            recorder.submitted(&envelope, Origin::Local);
        }
        assert_eq!(recorder.state.lock().unwrap().commands.len(), TRACE_LIMIT);
        assert!(guard.finish());
        let text = std::fs::read_to_string(&path).unwrap();
        assert!(!text.contains("PRIVATE_"));
        assert_eq!(text.matches("first_frame_submitted").count(), 1);
        std::fs::remove_file(path).unwrap();
    }
}
