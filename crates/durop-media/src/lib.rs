//! Bounded media worker boundary. Native decoder objects never leave the worker.
//!
//! One worker belongs to one host instance; the host bounds simultaneous workers.
//! Generations tag every event/frame so a stopped or restarted decoder cannot
//! publish into its replacement. Frames use a latest-only slot, not a FIFO.

#![deny(missing_docs)]

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

/// Largest accepted decoded dimension, bounding frame allocations.
const MAX_DIMENSION: u32 = 4096;
/// Per-frame allocation ceiling shared by portable and native decoders.
const MAX_FRAME_BYTES: usize = 64 * 1024 * 1024;
/// Hard playback lifetime ceiling independent of authored clip metadata.
const MAX_DURATION: Duration = Duration::from_secs(600);
/// Maximum wait for asynchronous AVFoundation readiness.
#[cfg(target_os = "macos")]
const PREPARE_TIMEOUT: Duration = Duration::from_secs(8);

/// Immutable preparation request for one generation of a playback worker.
#[derive(Debug, Clone)]
pub struct MediaSpec {
    /// Validated local media resource selected by the catalog.
    pub path: PathBuf,
    /// Monotonic generation used to discard stale frames and events.
    pub generation: u64,
    /// Caller-selected lifetime cap, bounded again by the decoder.
    pub max_duration: Duration,
    /// Whether playback restarts until the lifetime cap or cancellation.
    pub looping: bool,
    /// Normalized output gain in the inclusive range zero to one.
    pub volume: f32,
}

/// Pixel alpha representation required by the renderer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AlphaMode {
    /// RGB channels are unassociated with alpha and require straight-alpha blending.
    Straight,
}

/// Color encoding of decoded CPU frame pixels.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorSpace {
    /// Standard sRGB color encoding after native color conversion.
    Srgb,
}

/// Immutable tightly packed CPU frame safe to move between worker and renderer.
#[derive(Debug, Clone)]
pub struct DecodedFrame {
    /// Monotonic generation used to discard stale frames and events.
    pub generation: u64,
    /// Media presentation timestamp in nanoseconds, not a cross-machine clock.
    pub presentation_ns: u64,
    /// Decoded horizontal pixel count.
    pub width: u32,
    /// Decoded vertical pixel count.
    pub height: u32,
    /// Byte distance between adjacent RGBA rows.
    pub row_stride: usize,
    /// Shared immutable RGBA8 pixels; older unread frames are replaced.
    pub rgba: Arc<[u8]>,
    /// Alpha representation used by the uploaded texture.
    pub alpha: AlphaMode,
    /// Encoding to preserve when creating the renderer texture.
    pub color_space: ColorSpace,
}

/// Worker lifecycle acknowledgements; terminal events follow native cleanup.
#[derive(Debug, Clone, PartialEq)]
pub enum MediaEventKind {
    /// Preparation completed and reported the immutable stream capabilities.
    Prepared {
        /// Validated source duration, before the caller lifetime cap.
        duration: Duration,
        /// Whether the stream can produce RGBA frames.
        has_video: bool,
    },
    /// Playback has begun after an explicit start command.
    Started,
    /// The natural end or configured lifetime limit completed playback.
    Finished,
    /// Cancellation released native resources for this generation.
    Stopped,
    /// Preparation, decoding or cleanup terminated with an error.
    Failed {
        /// Decoder diagnostic describing this terminal failure.
        message: String,
    },
}

/// Lifecycle acknowledgement tagged to its initiating playback generation.
#[derive(Debug, Clone, PartialEq)]
pub struct MediaEvent {
    /// Monotonic generation used to discard stale frames and events.
    pub generation: u64,
    /// Transition observed by the decoder worker.
    pub kind: MediaEventKind,
}

/// Bounded producer/consumer mailbox: lifecycle events and one latest frame.
#[derive(Default)]
struct Shared {
    /// Accepted generations retained until their terminal acknowledgement is consumed.
    outstanding: BTreeSet<u64>,
    /// At most Prepared, Started and one terminal event per accepted generation.
    events: VecDeque<MediaEvent>,
    /// One newest unconsumed frame; publication replaces older frames.
    frame: Option<DecodedFrame>,
    /// Last media position observed by the worker.
    position: Duration,
}

/// Current generation and cancellation flag guarded by one publication lock.
struct Selection {
    /// Current monotonic playback generation.
    generation: u64,
    /// Whether current generation has received an out-of-band stop.
    stopped: bool,
}

/// Out-of-band cancellation that cannot be blocked by a full command queue.
struct Control {
    /// Serializes generation changes and frame publication with cancellation.
    selection: Mutex<Selection>,
    /// Requests teardown when the owning worker handle is dropped.
    shutdown: AtomicBool,
}

impl Control {
    /// Reject work after shutdown, explicit stop or generation replacement.
    fn cancelled(&self, generation: u64) -> bool {
        let selection = self.selection.lock().unwrap();
        self.shutdown.load(Ordering::Acquire)
            || selection.stopped
            || selection.generation != generation
    }
}

/// Bounded ordinary worker commands; cancellation bypasses this channel.
enum Command {
    Prepare(MediaSpec),
    Start(u64),
}

/// Commands are bounded; stop/cancellation bypass the queue and cannot be blocked
/// by a stalled renderer. Dropping a worker requests teardown without joining on
/// the render thread; Stopped is emitted only after native objects are released.
pub struct MediaWorker {
    /// Bounded channel for ordinary prepare/start operations.
    sender: mpsc::SyncSender<Command>,
    /// Shared generation guard and cancellation path independent of queue capacity.
    control: Arc<Control>,
    /// Lifecycle and latest-frame mailbox consumed by the application.
    shared: Arc<Mutex<Shared>>,
}

impl MediaWorker {
    /// Create one worker with bounded preparation/start queues and a latest-frame slot.
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
                Self::run(receiver, worker_control, worker_shared);
            })
            .expect("media worker thread");
        Self {
            sender,
            control,
            shared,
        }
    }

    /// Admit a newer generation while bounding unconsumed lifecycle acknowledgements.
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

    /// Request playback only for the currently selected prepared generation.
    pub fn start(&self, generation: u64) -> Result<(), String> {
        self.sender
            .try_send(Command::Start(generation))
            .map_err(|_| "media command queue full or worker closed".into())
    }

    /// Cancel a matching generation immediately and clear its pending frame.
    pub fn stop(&self, generation: u64) {
        let mut selection = self.control.selection.lock().unwrap();
        if selection.generation == generation {
            selection.stopped = true;
            drop(selection);
            self.shared.lock().unwrap().frame = None;
        }
    }

    /// Consume lifecycle events and release terminal generation admission slots.
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

    /// Take the newest frame exactly once, dropping any unread predecessors.
    pub fn latest_frame(&self) -> Option<DecodedFrame> {
        self.shared.lock().unwrap().frame.take()
    }

    /// Read the most recently observed media position for diagnostics.
    pub fn position(&self) -> Duration {
        self.shared.lock().unwrap().position
    }
}

impl Drop for MediaWorker {
    fn drop(&mut self) {
        self.control.shutdown.store(true, Ordering::Release);
    }
}

impl Shared {
    /// Append one lifecycle event while enforcing the accepted-generation queue bound.
    fn event(shared: &Mutex<Shared>, generation: u64, kind: MediaEventKind) {
        let mut shared = shared.lock().unwrap();
        // At most eight accepted, unconsumed generations, each with Prepared,
        // Started and one terminal event. Backpressure occurs at prepare(), so no
        // accepted lifecycle loses its terminal acknowledgement to a full queue.
        assert!(shared.events.len() < 24, "media event lifecycle bound");
        shared.events.push_back(MediaEvent { generation, kind });
    }
}

impl DecodedFrame {
    /// Validate dimensions and return a bounded tightly packed RGBA allocation size.
    fn byte_len(width: u32, height: u32) -> Result<usize, String> {
        if width == 0 || height == 0 || width > MAX_DIMENSION || height > MAX_DIMENSION {
            return Err("media dimensions must be between 1 and 4096".into());
        }
        let size = width as usize * height as usize * 4;
        if size > MAX_FRAME_BYTES {
            return Err("decoded frame exceeds 64 MiB".into());
        }
        Ok(size)
    }
}

impl Control {
    /// Publish only while this generation remains live under the cancellation lock.
    fn publish(&self, shared: &Mutex<Shared>, frame: DecodedFrame) {
        // Same lock order as prepare: selection, then shared. Cancellation cannot
        // interleave between the generation check and publication.
        let selection = self.selection.lock().unwrap();
        if self.shutdown.load(Ordering::Acquire)
            || selection.stopped
            || selection.generation != frame.generation
        {
            return;
        }
        shared.lock().unwrap().frame = Some(frame);
    }
}

/// Distinguish deliberate cancellation from a decoder failure during preparation.
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

/// Selected concrete decoder; native objects remain confined to their owner thread.
enum Decoder {
    Gif(gif::Timeline),
    #[cfg(target_os = "macos")]
    Native(native::Player),
}

impl Decoder {
    /// Validate file bounds and select portable GIF or native stream decoding.
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

    /// Release backend resources before the caller emits a terminal acknowledgement.
    fn stop(&mut self) -> Result<(), String> {
        match self {
            Self::Gif(_) => Ok(()),
            #[cfg(target_os = "macos")]
            Self::Native(x) => x.stop(),
        }
    }

    /// Read the validated source duration independent of worker lifetime limits.
    fn duration(&self) -> Duration {
        match self {
            Self::Gif(x) => x.duration(),
            #[cfg(target_os = "macos")]
            Self::Native(x) => x.duration(),
        }
    }

    /// Report whether this decoder can produce RGBA frames.
    fn has_video(&self) -> bool {
        match self {
            Self::Gif(_) => true,
            #[cfg(target_os = "macos")]
            Self::Native(x) => x.has_video(),
        }
    }

    /// Begin the selected decoder after explicit host start admission.
    fn start(&mut self) {
        match self {
            Self::Gif(x) => x.start(),
            #[cfg(target_os = "macos")]
            Self::Native(x) => x.start(),
        }
    }

    /// Observe media time and at most one newest generation-tagged frame.
    fn tick(&mut self, generation: u64) -> Result<(Duration, Option<DecodedFrame>), String> {
        match self {
            Self::Gif(x) => x.tick(generation),
            #[cfg(target_os = "macos")]
            Self::Native(x) => x.tick(generation),
        }
    }

    /// Restart source playback while retaining the worker overall lifetime cap.
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

impl MediaWorker {
    /// Own decoder transitions on one worker thread and serialize lifecycle acknowledgement.
    fn run(receiver: mpsc::Receiver<Command>, control: Arc<Control>, shared: Arc<Mutex<Shared>>) {
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
                Decoder::terminate(
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
                        Decoder::terminate(
                            &mut decoder,
                            &shared,
                            current.generation,
                            MediaEventKind::Stopped,
                        );
                        spec = None;
                    }
                    playing = None;
                    if control.cancelled(next.generation) {
                        Shared::event(&shared, next.generation, MediaEventKind::Stopped);
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
                                Decoder::terminate(
                                    &mut cancelled,
                                    &shared,
                                    next.generation,
                                    MediaEventKind::Stopped,
                                );
                                continue;
                            }
                            Shared::event(
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
                            Shared::event(
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
            if let (Some(current), Some(started), Some(active)) = (&spec, playing, decoder.as_mut())
            {
                #[cfg(target_os = "macos")]
                let result = objc2::rc::autoreleasepool(|_| active.tick(current.generation));
                #[cfg(not(target_os = "macos"))]
                let result = active.tick(current.generation);
                match result {
                    Ok((position, frame)) => {
                        if !announced && (position > Duration::ZERO || frame.is_some()) {
                            Shared::event(&shared, current.generation, MediaEventKind::Started);
                            announced = true;
                        }
                        shared.lock().unwrap().position = position;
                        if let Some(frame) = frame {
                            control.publish(&shared, frame);
                        }
                        if started.elapsed() >= current.max_duration
                            || position >= active.duration()
                        {
                            if current.looping && started.elapsed() < current.max_duration {
                                if let Err(message) = active.restart() {
                                    Decoder::terminate(
                                        &mut decoder,
                                        &shared,
                                        current.generation,
                                        MediaEventKind::Failed { message },
                                    );
                                    spec = None;
                                    playing = None;
                                }
                            } else {
                                Decoder::terminate(
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
                        Decoder::terminate(
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
            Decoder::terminate(
                &mut decoder,
                &shared,
                current.generation,
                MediaEventKind::Stopped,
            );
        }
    }
}

impl Decoder {
    /// Stop and drop resources before publishing the terminal generation acknowledgement.
    fn terminate(
        decoder: &mut Option<Decoder>,
        shared: &Mutex<Shared>,
        generation: u64,
        kind: MediaEventKind,
    ) {
        let result = decoder.as_mut().map_or(Ok(()), Decoder::stop);
        decoder.take();
        Shared::event(
            shared,
            generation,
            match result {
                Ok(()) => kind,
                Err(message) => MediaEventKind::Failed { message },
            },
        );
    }
}

/// Main-thread owner of platform playback controls, independent of an application framework.
///
/// Construct once and call [`Self::update`] regularly while [`Self::needs_updates`]
/// is true. The native owner is deliberately non-Send: AVPlayer objects must be
/// created, polled and released on the main thread.
pub struct MediaService {
    #[cfg(target_os = "macos")]
    owner: native::NativeOwner,
    _main_thread: std::marker::PhantomData<std::rc::Rc<()>>,
}

impl MediaService {
    /// Install the bounded main-thread service before preparing native media.
    pub fn new() -> Self {
        Self {
            #[cfg(target_os = "macos")]
            owner: native::NativeOwner::new(),
            _main_thread: std::marker::PhantomData,
        }
    }

    /// Poll platform controls once without waiting for readiness or decode work.
    pub fn update(&mut self) {
        #[cfg(target_os = "macos")]
        self.owner.update();
    }

    /// Whether native preparation or playback needs another main-thread tick.
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
}

impl Default for MediaService {
    fn default() -> Self {
        Self::new()
    }
}

/// Measured local timings from a bounded, muted native playback probe.
#[derive(Debug)]
pub struct ProbeReport {
    /// Milliseconds from admission to decoder readiness.
    pub ready_ms: f64,
    /// Milliseconds from start request to first available frame, when video exists.
    pub first_frame_after_start_ms: Option<f64>,
    /// Milliseconds spent draining native cleanup after the probe terminates.
    pub terminal_cleanup_ms: f64,
    /// Milliseconds from deliberate preparation cancellation to acknowledgement.
    pub cancellation_ack_ms: Option<f64>,
    /// Number of distinct latest frames observed by the probe.
    pub frames: usize,
    /// Dimensions of the most recently observed video frame.
    pub dimensions: Option<(u32, u32)>,
    /// Minimum and maximum alpha byte observed across captured frames.
    pub alpha_range: Option<(u8, u8)>,
    /// Last media position reported before cleanup.
    pub final_position: Duration,
}

impl ProbeReport {
    /// Measure muted playback once on the main thread before constructing the application.
    pub fn play(path: &std::path::Path) -> Result<ProbeReport, String> {
        Self::run(path, false)
    }
}

impl ProbeReport {
    /// Measure cancellation while asynchronous native preparation is pending.
    pub fn cancel(path: &std::path::Path) -> Result<ProbeReport, String> {
        Self::run(path, true)
    }
}

impl ProbeReport {
    /// Drive the framework-independent native service through bounded probe and cleanup deadlines.
    fn run(path: &std::path::Path, cancel_preparation: bool) -> Result<ProbeReport, String> {
        #[cfg(target_os = "macos")]
        assert!(
            objc2::MainThreadMarker::new().is_some(),
            "media probe must run on main"
        );
        let mut service = MediaService::new();
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
            if cancel_preparation && cancel_started.is_none() && MediaService::needs_updates() {
                cancel_started = Some(Instant::now());
                worker.stop(1);
            }
            service.update();
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
        while MediaService::needs_updates() && stopping.elapsed() < Duration::from_secs(2) {
            service.update();
            thread::sleep(Duration::from_millis(2));
        }
        report.terminal_cleanup_ms = stopping.elapsed().as_secs_f64() * 1000.0;
        terminal.unwrap_or(Err("media probe exceeded twelve seconds".into()))?;
        Ok(report)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn fixture(generation: u64) -> MediaSpec {
        MediaSpec {
            path: PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("../../fixtures/media/disposal.gif"),
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
        worker.control.publish(&worker.shared, stale);
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
        // Exercise the producer/consumer mailbox without a playback deadline:
        // the real-worker tests above cover start, completion and stop. Sleeping
        // inside a short clip's lifetime races valid terminal frame cleanup.
        let (sender, _receiver) = mpsc::sync_channel(4);
        let worker = MediaWorker {
            sender,
            control: Arc::new(Control {
                selection: Mutex::new(Selection {
                    generation: 1,
                    stopped: false,
                }),
                shutdown: AtomicBool::new(false),
            }),
            shared: Arc::new(Mutex::new(Shared {
                outstanding: BTreeSet::from([1]),
                ..Default::default()
            })),
        };
        Shared::event(
            &worker.shared,
            1,
            MediaEventKind::Prepared {
                duration: Duration::from_secs(1),
                has_video: true,
            },
        );
        Shared::event(&worker.shared, 1, MediaEventKind::Started);

        let mut buffers = Vec::new();
        for sequence in 1..=1024 {
            let rgba: Arc<[u8]> = vec![(sequence % 251) as u8; 64].into();
            buffers.push(Arc::downgrade(&rgba));
            worker.control.publish(
                &worker.shared,
                DecodedFrame {
                    generation: 1,
                    presentation_ns: sequence,
                    width: 4,
                    height: 4,
                    row_stride: 16,
                    rgba,
                    alpha: AlphaMode::Straight,
                    color_space: ColorSpace::Srgb,
                },
            );
            // No frames or events are consumed during this producer burst.
            assert_eq!(worker.shared.lock().unwrap().events.len(), 2);
        }
        assert!(
            buffers[..1023]
                .iter()
                .all(|buffer| buffer.strong_count() == 0)
        );
        assert_eq!(buffers[1023].strong_count(), 1);

        let latest = worker.latest_frame().unwrap();
        assert_eq!(latest.generation, 1);
        assert_eq!(latest.presentation_ns, 1024);
        assert_eq!((latest.width, latest.height, latest.row_stride), (4, 4, 16));
        assert_eq!(&*latest.rgba, &[(1024 % 251) as u8; 64]);
        assert!(worker.latest_frame().is_none());
        assert_eq!(
            worker.drain_events(),
            [
                MediaEvent {
                    generation: 1,
                    kind: MediaEventKind::Prepared {
                        duration: Duration::from_secs(1),
                        has_video: true,
                    },
                },
                MediaEvent {
                    generation: 1,
                    kind: MediaEventKind::Started,
                },
            ]
        );
        drop(latest);
        assert!(buffers.iter().all(|buffer| buffer.strong_count() == 0));
    }
}
