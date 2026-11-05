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

const FILE_LIMIT: usize = 16 * 1024 * 1024;
const EVENT_LIMIT: usize = 4096;
const FOOTER_RESERVE: usize = 1024;
const TRACE_LIMIT: usize = 256;
static RECORDER: OnceLock<Recorder> = OnceLock::new();

#[derive(Clone, Copy)]
pub enum Origin {
    Local,
    Remote,
}
impl Origin {
    fn label(self) -> &'static str {
        match self {
            Self::Local => "local",
            Self::Remote => "remote_host_handoff",
        }
    }
}
#[derive(Default)]
struct Counters {
    dropped: AtomicU64,
    evicted: AtomicU64,
    writer_errors: AtomicU64,
    capped: AtomicBool,
}
enum WriterCommand {
    Event(Vec<u8>),
    Stop(mpsc::Sender<bool>),
}
#[derive(Clone)]
struct EventFactory {
    sender: SyncSender<WriterCommand>,
    counters: Arc<Counters>,
}
struct EventBuffer {
    factory: EventFactory,
    bytes: Vec<u8>,
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
struct CommandTrace {
    session: String,
    command: String,
    start: Instant,
    id: u64,
    origin: Origin,
}
struct InstanceTrace {
    start: Instant,
    id: u64,
    origin: Origin,
    first_frame: bool,
}
#[derive(Default)]
struct TraceState {
    commands: VecDeque<CommandTrace>,
    instances: BTreeMap<String, InstanceTrace>,
}
struct Recorder {
    dispatch: tracing::Dispatch,
    state: Mutex<TraceState>,
    next: AtomicU64,
    counters: Arc<Counters>,
    enabled: Arc<AtomicBool>,
}

/// Hold through App::run. Drop attempts a final summary/flush for at most two seconds,
/// after rendering has ended. A blocked disk never blocks the render thread's writes.
pub struct DiagnosticsGuard {
    sender: Option<SyncSender<WriterCommand>>,
    writer: Option<JoinHandle<()>>,
    enabled: Arc<AtomicBool>,
}
impl DiagnosticsGuard {
    pub fn finish(mut self) -> bool {
        self.finish_inner()
    }
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

pub fn start(path: &Path) -> io::Result<DiagnosticsGuard> {
    if RECORDER.get().is_some() {
        return Err(io::Error::new(
            io::ErrorKind::AlreadyExists,
            "diagnostics already configured",
        ));
    }
    let (recorder, guard) = create(path, FILE_LIMIT)?;
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
        .spawn(move || write_events(file, receiver, writer_counters, limit))?;
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
fn recorder() -> Option<&'static Recorder> {
    RECORDER
        .get()
        .filter(|recorder| recorder.enabled.load(Ordering::Acquire))
}
fn micros(elapsed: Duration) -> u64 {
    u64::try_from(elapsed.as_micros()).unwrap_or(u64::MAX)
}
impl Recorder {
    fn emit(&self, stage: &'static str, id: u64, origin: Origin, elapsed: Duration) {
        tracing::dispatcher::with_default(
            &self.dispatch,
            || tracing::info!(target: "durop_timing", stage, trace_id = id, origin = origin.label(), elapsed_us = micros(elapsed)),
        );
    }
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
pub fn submitted(envelope: &Envelope, origin: Origin) {
    if let Some(recorder) = recorder() {
        recorder.submitted(envelope, origin);
    }
}
pub fn acknowledged(session: &str, command: &str, outcome: &CommandOutcome) {
    if let Some(recorder) = recorder() {
        recorder.acknowledged(session, command, outcome);
    }
}
pub fn observe_events(events: &[HostEvent]) {
    if let Some(recorder) = recorder() {
        recorder.observe_events(events);
    }
}
pub fn first_frame_submitted(instance: &str) {
    if let Some(recorder) = recorder() {
        recorder.first_frame_submitted(instance);
    }
}
/// Controller-local roundtrip only. Never subtract clocks from different machines.
pub fn controller_ack(elapsed: Duration) {
    if let Some(recorder) = recorder() {
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
        let (recorder, guard) = create(&path, 4096).unwrap();
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
        assert!(create(&path, FILE_LIMIT).is_err());
        std::fs::remove_file(path).unwrap();
    }
    #[test]
    fn timings_correlate_without_logging_private_identifiers_and_maps_are_bounded() {
        let path = path();
        let (recorder, guard) = create(&path, FILE_LIMIT).unwrap();
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
