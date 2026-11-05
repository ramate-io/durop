//! Bounded media worker boundary. Native decoder objects never leave the worker.
//!
//! One worker belongs to one host instance; the host bounds simultaneous workers.
//! Generations tag every event/frame so a stopped or restarted decoder cannot
//! publish into its replacement. Frames use a latest-only slot, not a FIFO.

use std::{
    collections::{BTreeSet, VecDeque},
    path::PathBuf,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
        mpsc,
    },
    thread,
    time::{Duration, Instant},
};
mod gif;
#[cfg(target_os = "macos")]
mod native;

const MAX_DIMENSION: u32 = 4096;
const MAX_FRAME_BYTES: usize = 64 * 1024 * 1024;
const MAX_DURATION: Duration = Duration::from_secs(600);
const PREPARE_TIMEOUT: Duration = Duration::from_secs(8);

#[derive(Debug, Clone)]
pub struct MediaSpec {
    pub path: PathBuf,
    pub generation: u64,
    pub max_duration: Duration,
    pub looping: bool,
    pub volume: f32,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AlphaMode {
    Straight,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorSpace {
    Srgb,
}
#[derive(Debug, Clone)]
pub struct DecodedFrame {
    pub generation: u64,
    pub presentation_ns: u64,
    pub width: u32,
    pub height: u32,
    pub row_stride: usize,
    pub rgba: Arc<[u8]>,
    pub alpha: AlphaMode,
    pub color_space: ColorSpace,
}
#[derive(Debug, Clone, PartialEq)]
pub enum MediaEventKind {
    Prepared { duration: Duration, has_video: bool },
    Started,
    Finished,
    Stopped,
    Failed { message: String },
}
#[derive(Debug, Clone, PartialEq)]
pub struct MediaEvent {
    pub generation: u64,
    pub kind: MediaEventKind,
}
#[derive(Default)]
struct Shared {
    outstanding: BTreeSet<u64>,
    events: VecDeque<MediaEvent>,
    frame: Option<DecodedFrame>,
    position: Duration,
}
struct Selection {
    generation: u64,
    stopped: bool,
}
struct Control {
    selection: Mutex<Selection>,
    shutdown: AtomicBool,
}
impl Control {
    fn cancelled(&self, generation: u64) -> bool {
        let selection = self.selection.lock().unwrap();
        self.shutdown.load(Ordering::Acquire)
            || selection.stopped
            || selection.generation != generation
    }
}
enum Command {
    Prepare(MediaSpec),
    Start(u64),
}
/// Commands are bounded; stop/cancellation bypass the queue and cannot be blocked
/// by a stalled renderer. Dropping a worker requests teardown without joining on
/// the render thread; Stopped is emitted only after native objects are released.
pub struct MediaWorker {
    sender: mpsc::SyncSender<Command>,
    control: Arc<Control>,
    shared: Arc<Mutex<Shared>>,
}
impl MediaWorker {
    pub fn spawn() -> Self {
        let (sender, receiver) = mpsc::sync_channel(4);
        let control = Arc::new(Control {
            selection: Mutex::new(Selection {
                generation: 0,
                stopped: true,
            }),
            shutdown: AtomicBool::new(false),
        });
        let shared = Arc::new(Mutex::new(Shared::default()));
        let worker_control = control.clone();
        let worker_shared = shared.clone();
        thread::Builder::new()
            .name("durop-media".into())
            .spawn(move || {
                worker(receiver, worker_control, worker_shared);
            })
            .expect("media worker thread");
        Self {
            sender,
            control,
            shared,
        }
    }
    pub fn prepare(&self, spec: MediaSpec) -> Result<(), String> {
        if spec.generation == 0
            || spec.max_duration.is_zero()
            || spec.max_duration > MAX_DURATION
            || !spec.volume.is_finite()
            || !(0.0..=1.0).contains(&spec.volume)
        {
            return Err("invalid generation, duration or volume".into());
        }
        // The receiver checks the same short-held mutex, so command acceptance
        // and cancellation publication are coherent without a waiting spin.
        let mut selection = self.control.selection.lock().unwrap();
        if spec.generation <= selection.generation {
            return Err("generation must increase for each prepare on a worker".into());
        }
        let generation = spec.generation;
        let mut shared = self.shared.lock().unwrap();
        if shared.outstanding.len() >= 8 {
            return Err(
                "drain terminal events before accepting more than eight generations".into(),
            );
        }
        self.sender
            .try_send(Command::Prepare(spec))
            .map_err(|_| "media command queue full or worker closed")?;
        shared.outstanding.insert(generation);
        *selection = Selection {
            generation,
            stopped: false,
        };
        Ok(())
    }
    pub fn start(&self, generation: u64) -> Result<(), String> {
        self.sender
            .try_send(Command::Start(generation))
            .map_err(|_| "media command queue full or worker closed".into())
    }
    pub fn stop(&self, generation: u64) {
        let mut selection = self.control.selection.lock().unwrap();
        if selection.generation == generation {
            selection.stopped = true;
            drop(selection);
            self.shared.lock().unwrap().frame = None;
        }
    }
    pub fn drain_events(&self) -> Vec<MediaEvent> {
        let mut shared = self.shared.lock().unwrap();
        let events: Vec<_> = shared.events.drain(..).collect();
        for event in &events {
            if matches!(
                event.kind,
                MediaEventKind::Finished | MediaEventKind::Stopped | MediaEventKind::Failed { .. }
            ) {
                shared.outstanding.remove(&event.generation);
            }
        }
        events
    }
    pub fn latest_frame(&self) -> Option<DecodedFrame> {
        self.shared.lock().unwrap().frame.take()
    }
    pub fn position(&self) -> Duration {
        self.shared.lock().unwrap().position
    }
}
impl Drop for MediaWorker {
    fn drop(&mut self) {
        self.control.shutdown.store(true, Ordering::Release);
    }
}
fn event(shared: &Mutex<Shared>, generation: u64, kind: MediaEventKind) {
    let mut shared = shared.lock().unwrap();
    // At most eight accepted, unconsumed generations, each with Prepared,
    // Started and one terminal event. Backpressure occurs at prepare(), so no
    // accepted lifecycle loses its terminal acknowledgement to a full queue.
    assert!(shared.events.len() < 24, "media event lifecycle bound");
    shared.events.push_back(MediaEvent { generation, kind });
}
fn frame_size(width: u32, height: u32) -> Result<usize, String> {
    if width == 0 || height == 0 || width > MAX_DIMENSION || height > MAX_DIMENSION {
        return Err("media dimensions must be between 1 and 4096".into());
    }
    let size = width as usize * height as usize * 4;
    if size > MAX_FRAME_BYTES {
        return Err("decoded frame exceeds 64 MiB".into());
    }
    Ok(size)
}
fn publish(shared: &Mutex<Shared>, control: &Control, frame: DecodedFrame) {
    // Same lock order as prepare: selection, then shared. Cancellation cannot
    // interleave between the generation check and publication.
    let selection = control.selection.lock().unwrap();
    if control.shutdown.load(Ordering::Acquire)
        || selection.stopped
        || selection.generation != frame.generation
    {
        return;
    }
    shared.lock().unwrap().frame = Some(frame);
}

enum PreparationError {
    Cancelled,
    Failed(String),
}
impl From<String> for PreparationError {
    fn from(message: String) -> Self {
        Self::Failed(message)
    }
}
impl From<&str> for PreparationError {
    fn from(message: &str) -> Self {
        Self::Failed(message.into())
    }
}
enum Decoder {
    Gif(gif::Timeline),
    #[cfg(target_os = "macos")]
    Native(native::Player),
}
impl Decoder {
    fn prepare(spec: &MediaSpec, control: &Control) -> Result<Self, PreparationError> {
        let metadata = std::fs::metadata(&spec.path).map_err(|e| e.to_string())?;
        if !metadata.is_file() || metadata.len() > 256 * 1024 * 1024 {
            return Err("media file must be regular and at most 256 MiB".into());
        }
        if spec
            .path
            .extension()
            .is_some_and(|x| x.eq_ignore_ascii_case("gif"))
        {
            return gif::Timeline::prepare(&spec.path, control, spec.generation)
                .map(Self::Gif)
                .map_err(|message| {
                    if control.cancelled(spec.generation) {
                        PreparationError::Cancelled
                    } else {
                        PreparationError::Failed(message)
                    }
                });
        }
        #[cfg(target_os = "macos")]
        {
            native::Player::prepare(spec, control).map(Self::Native)
        }
        #[cfg(not(target_os = "macos"))]
        {
            Err("AVFoundation playback is available only on macOS; GIF decoding is portable".into())
        }
    }
    fn stop(&mut self) -> Result<(), String> {
        match self {
            Self::Gif(_) => Ok(()),
            #[cfg(target_os = "macos")]
            Self::Native(x) => x.stop(),
        }
    }
    fn duration(&self) -> Duration {
        match self {
            Self::Gif(x) => x.duration(),
            #[cfg(target_os = "macos")]
            Self::Native(x) => x.duration(),
        }
    }
    fn has_video(&self) -> bool {
        match self {
            Self::Gif(_) => true,
            #[cfg(target_os = "macos")]
            Self::Native(x) => x.has_video(),
        }
    }
    fn start(&mut self) {
        match self {
            Self::Gif(x) => x.start(),
            #[cfg(target_os = "macos")]
            Self::Native(x) => x.start(),
        }
    }
    fn tick(&mut self, generation: u64) -> Result<(Duration, Option<DecodedFrame>), String> {
        match self {
            Self::Gif(x) => x.tick(generation),
            #[cfg(target_os = "macos")]
            Self::Native(x) => x.tick(generation),
        }
    }
    fn restart(&mut self) -> Result<(), String> {
        match self {
            Self::Gif(x) => {
                x.start();
                Ok(())
            }
            #[cfg(target_os = "macos")]
            Self::Native(x) => x.restart(),
        }
    }
}
fn worker(receiver: mpsc::Receiver<Command>, control: Arc<Control>, shared: Arc<Mutex<Shared>>) {
    let mut decoder: Option<Decoder> = None;
    let mut spec: Option<MediaSpec> = None;
    let mut playing: Option<Instant> = None;
    let mut announced = false;
    loop {
        if control.shutdown.load(Ordering::Acquire) {
            break;
        }
        if let Some(current) = &spec
            && control.cancelled(current.generation)
        {
            terminate(
                &mut decoder,
                &shared,
                current.generation,
                MediaEventKind::Stopped,
            );
            playing = None;
            shared.lock().unwrap().frame = None;
            spec = None;
        }
        match receiver.recv_timeout(Duration::from_millis(5)) {
            Ok(Command::Prepare(next)) => {
                if let Some(current) = &spec {
                    terminate(
                        &mut decoder,
                        &shared,
                        current.generation,
                        MediaEventKind::Stopped,
                    );
                    spec = None;
                }
                playing = None;
                if control.cancelled(next.generation) {
                    event(&shared, next.generation, MediaEventKind::Stopped);
                    continue;
                }
                {
                    let mut s = shared.lock().unwrap();
                    s.frame = None;
                    s.position = Duration::ZERO;
                }
                match Decoder::prepare(&next, &control) {
                    Ok(prepared) => {
                        if control.cancelled(next.generation) {
                            let mut cancelled = Some(prepared);
                            terminate(
                                &mut cancelled,
                                &shared,
                                next.generation,
                                MediaEventKind::Stopped,
                            );
                            continue;
                        }
                        event(
                            &shared,
                            next.generation,
                            MediaEventKind::Prepared {
                                duration: prepared.duration(),
                                has_video: prepared.has_video(),
                            },
                        );
                        decoder = Some(prepared);
                        spec = Some(next);
                    }
                    Err(message) => {
                        event(
                            &shared,
                            next.generation,
                            match message {
                                PreparationError::Cancelled => MediaEventKind::Stopped,
                                PreparationError::Failed(message) => {
                                    MediaEventKind::Failed { message }
                                }
                            },
                        );
                        spec = None;
                    }
                }
            }
            Ok(Command::Start(generation)) => {
                if let Some(current) = &spec
                    && current.generation == generation
                    && !control.cancelled(generation)
                    && let Some(decoder) = decoder.as_mut()
                    && playing.is_none()
                {
                    decoder.start();
                    playing = Some(Instant::now());
                    announced = false;
                }
            }
            Err(mpsc::RecvTimeoutError::Disconnected) => break,
            Err(mpsc::RecvTimeoutError::Timeout) => (),
        }
        if let (Some(current), Some(started), Some(active)) = (&spec, playing, decoder.as_mut()) {
            #[cfg(target_os = "macos")]
            let result = objc2::rc::autoreleasepool(|_| active.tick(current.generation));
            #[cfg(not(target_os = "macos"))]
            let result = active.tick(current.generation);
            match result {
                Ok((position, frame)) => {
                    if !announced && (position > Duration::ZERO || frame.is_some()) {
                        event(&shared, current.generation, MediaEventKind::Started);
                        announced = true;
                    }
                    shared.lock().unwrap().position = position;
                    if let Some(frame) = frame {
                        publish(&shared, &control, frame);
                    }
                    if started.elapsed() >= current.max_duration || position >= active.duration() {
                        if current.looping && started.elapsed() < current.max_duration {
                            if let Err(message) = active.restart() {
                                terminate(
                                    &mut decoder,
                                    &shared,
                                    current.generation,
                                    MediaEventKind::Failed { message },
                                );
                                spec = None;
                                playing = None;
                            }
                        } else {
                            terminate(
                                &mut decoder,
                                &shared,
                                current.generation,
                                if announced {
                                    MediaEventKind::Finished
                                } else {
                                    MediaEventKind::Failed{message:"playback did not produce advancing time or frames before its deadline".into()}
                                },
                            );
                            spec = None;
                            playing = None;
                            shared.lock().unwrap().frame = None;
                        }
                    }
                }
                Err(message) => {
                    terminate(
                        &mut decoder,
                        &shared,
                        current.generation,
                        MediaEventKind::Failed { message },
                    );
                    spec = None;
                    playing = None;
                    shared.lock().unwrap().frame = None;
                }
            }
        }
    }
    if let Some(current) = spec {
        terminate(
            &mut decoder,
            &shared,
            current.generation,
            MediaEventKind::Stopped,
        );
    }
}
fn terminate(
    decoder: &mut Option<Decoder>,
    shared: &Mutex<Shared>,
    generation: u64,
    kind: MediaEventKind,
) {
    let result = decoder.as_mut().map_or(Ok(()), Decoder::stop);
    decoder.take();
    event(
        shared,
        generation,
        match result {
            Ok(()) => kind,
            Err(message) => MediaEventKind::Failed { message },
        },
    );
}

/// Install once in the app. AVPlayer's control objects stay main-thread owned;
/// decode work is internal to AVFoundation, and color conversion runs on workers.
pub struct MediaPlugin;
impl bevy::prelude::Plugin for MediaPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        #[cfg(target_os = "macos")]
        native::install(app);
        #[cfg(not(target_os = "macos"))]
        let _ = app;
    }
}
pub fn needs_updates() -> bool {
    #[cfg(target_os = "macos")]
    {
        native::needs_updates()
    }
    #[cfg(not(target_os = "macos"))]
    {
        false
    }
}

#[derive(Debug)]
pub struct ProbeReport {
    pub ready_ms: f64,
    pub first_frame_after_start_ms: Option<f64>,
    pub terminal_cleanup_ms: f64,
    pub cancellation_ack_ms: Option<f64>,
    pub frames: usize,
    pub dimensions: Option<(u32, u32)>,
    pub alpha_range: Option<(u8, u8)>,
    pub final_position: Duration,
}
/// Bounded, muted command-line proof of the real Rust owner/worker bridge.
/// Invoke once from main before constructing the regular application.
pub fn probe(path: &std::path::Path) -> Result<ProbeReport, String> {
    probe_mode(path, false)
}
pub fn probe_cancel(path: &std::path::Path) -> Result<ProbeReport, String> {
    probe_mode(path, true)
}
fn probe_mode(path: &std::path::Path, cancel_preparation: bool) -> Result<ProbeReport, String> {
    #[cfg(target_os = "macos")]
    assert!(
        objc2::MainThreadMarker::new().is_some(),
        "media probe must run on main"
    );
    let mut app = bevy::prelude::App::new();
    app.add_plugins(MediaPlugin);
    let worker = MediaWorker::spawn();
    let beginning = Instant::now();
    worker.prepare(MediaSpec {
        path: path.canonicalize().map_err(|e| e.to_string())?,
        generation: 1,
        max_duration: Duration::from_secs(4),
        looping: false,
        volume: 0.0,
    })?;
    let mut report = ProbeReport {
        ready_ms: 0.0,
        first_frame_after_start_ms: None,
        terminal_cleanup_ms: 0.0,
        cancellation_ack_ms: None,
        frames: 0,
        dimensions: None,
        alpha_range: None,
        final_position: Duration::ZERO,
    };
    let mut start = None;
    let mut terminal = None;
    let mut cancel_started = None;
    while beginning.elapsed() < Duration::from_secs(12) {
        if cancel_preparation && cancel_started.is_none() && needs_updates() {
            cancel_started = Some(Instant::now());
            worker.stop(1);
        }
        app.update();
        for event in worker.drain_events() {
            match event.kind {
                MediaEventKind::Prepared { .. } => {
                    report.ready_ms = beginning.elapsed().as_secs_f64() * 1000.0;
                    if !cancel_preparation {
                        start = Some(Instant::now());
                        worker.start(1)?;
                    }
                }
                MediaEventKind::Finished => terminal = Some(Ok(())),
                MediaEventKind::Stopped if cancel_preparation => {
                    report.cancellation_ack_ms =
                        cancel_started.map(|start| start.elapsed().as_secs_f64() * 1000.0);
                    terminal = Some(Ok(()));
                }
                MediaEventKind::Failed { message } => terminal = Some(Err(message)),
                _ => (),
            }
        }
        if let Some(frame) = worker.latest_frame() {
            report.frames += 1;
            report.dimensions = Some((frame.width, frame.height));
            if report.first_frame_after_start_ms.is_none() {
                report.first_frame_after_start_ms =
                    start.map(|t| t.elapsed().as_secs_f64() * 1000.0);
            }
            let (mut low, mut high) = report.alpha_range.unwrap_or((255, 0));
            for pixel in frame.rgba.chunks_exact(4) {
                low = low.min(pixel[3]);
                high = high.max(pixel[3]);
            }
            report.alpha_range = Some((low, high));
        }
        report.final_position = worker.position();
        if terminal.is_some() {
            break;
        }
        #[cfg(target_os = "macos")]
        objc2_foundation::NSRunLoop::currentRunLoop().runUntilDate(
            &objc2_foundation::NSDate::dateWithTimeIntervalSinceNow(0.003),
        );
        #[cfg(not(target_os = "macos"))]
        thread::sleep(Duration::from_millis(3));
    }
    let stopping = Instant::now();
    worker.stop(1);
    drop(worker);
    while needs_updates() && stopping.elapsed() < Duration::from_secs(2) {
        app.update();
        thread::sleep(Duration::from_millis(2));
    }
    report.terminal_cleanup_ms = stopping.elapsed().as_secs_f64() * 1000.0;
    terminal.unwrap_or(Err("media probe exceeded twelve seconds".into()))?;
    Ok(report)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn fixture(generation: u64) -> MediaSpec {
        MediaSpec {
            path: PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("fixtures/media/disposal.gif"),
            generation,
            max_duration: Duration::from_millis(120),
            looping: true,
            volume: 0.0,
        }
    }
    fn wait(worker: &MediaWorker, predicate: impl Fn(&MediaEventKind) -> bool) -> Vec<MediaEvent> {
        let deadline = Instant::now() + Duration::from_secs(2);
        let mut result = Vec::new();
        loop {
            result.extend(worker.drain_events());
            if result.iter().any(|event| predicate(&event.kind)) {
                return result;
            }
            assert!(Instant::now() < deadline, "events timed out: {result:?}");
            thread::sleep(Duration::from_millis(2));
        }
    }
    #[test]
    fn prepare_does_not_start_stop_clears_and_rejects_stale_frames() {
        let worker = MediaWorker::spawn();
        worker.prepare(fixture(1)).unwrap();
        let prepared = wait(&worker, |event| {
            matches!(event, MediaEventKind::Prepared { .. })
        });
        assert!(
            !prepared
                .iter()
                .any(|event| event.kind == MediaEventKind::Started)
        );
        assert!(worker.latest_frame().is_none());
        worker.start(1).unwrap();
        wait(&worker, |event| *event == MediaEventKind::Started);
        let stopping = Instant::now();
        worker.stop(1);
        wait(&worker, |event| *event == MediaEventKind::Stopped);
        assert!(stopping.elapsed() < Duration::from_millis(250));
        assert!(worker.latest_frame().is_none());
        let stale = DecodedFrame {
            generation: 1,
            presentation_ns: 0,
            width: 1,
            height: 1,
            row_stride: 4,
            rgba: vec![255; 4].into(),
            alpha: AlphaMode::Straight,
            color_space: ColorSpace::Srgb,
        };
        publish(&worker.shared, &worker.control, stale);
        assert!(worker.latest_frame().is_none());
        assert!(worker.prepare(fixture(1)).is_err());
    }
    #[test]
    fn cancelled_queued_generations_each_acknowledge_and_final_generation_completes() {
        let worker = MediaWorker::spawn();
        let mut accepted = Vec::new();
        for generation in 1..=5 {
            if worker.prepare(fixture(generation)).is_ok() {
                accepted.push(generation);
                worker.stop(generation);
            }
        }
        let deadline = Instant::now() + Duration::from_secs(2);
        let mut terminals = BTreeSet::new();
        while terminals.len() < accepted.len() {
            for event in worker.drain_events() {
                if event.kind == MediaEventKind::Stopped {
                    terminals.insert(event.generation);
                }
            }
            assert!(
                Instant::now() < deadline,
                "missing acknowledgements: {accepted:?}, got {terminals:?}"
            );
            thread::sleep(Duration::from_millis(2));
        }
        worker.prepare(fixture(10)).unwrap();
        wait(&worker, |event| {
            matches!(event, MediaEventKind::Prepared { .. })
        });
        worker.start(10).unwrap();
        wait(&worker, |event| *event == MediaEventKind::Finished);
        assert!(worker.latest_frame().is_none());
    }
    #[test]
    fn stalled_consumer_has_one_frame_and_bounded_events() {
        let worker = MediaWorker::spawn();
        let mut spec = fixture(1);
        spec.max_duration = Duration::from_millis(450);
        worker.prepare(spec).unwrap();
        wait(&worker, |event| {
            matches!(event, MediaEventKind::Prepared { .. })
        });
        worker.start(1).unwrap();
        thread::sleep(Duration::from_millis(350));
        let shared = worker.shared.lock().unwrap();
        assert!(
            shared
                .frame
                .as_ref()
                .is_some_and(|frame| frame.rgba.len() == 64)
        );
        assert!(shared.events.len() <= 3);
        drop(shared);
        worker.stop(1);
        wait(&worker, |event| *event == MediaEventKind::Stopped);
    }
}
