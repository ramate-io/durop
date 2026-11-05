//! AVPlayer controls stay on AppKit's main thread; immutable CIImage values
//! cross to the worker for expensive color conversion and CPU frame copies.
use super::*;
use objc2::{
    AnyThread, ClassType, MainThreadMarker, MainThreadOnly, msg_send, rc::Retained,
    runtime::AnyObject,
};
use objc2_av_foundation::{AVPlayer, AVPlayerItem, AVPlayerItemStatus, AVPlayerItemVideoOutput};
use objc2_core_graphics::{CGColorSpace, kCGColorSpaceSRGB};
use objc2_core_image::{CIContext, CIImage, kCIFormatRGBA8};
use objc2_core_media::CMTime;
use objc2_foundation::{NSDictionary, NSNumber, NSString, NSURL};
use std::{ptr::NonNull, sync::OnceLock};

/// Immutable Core Image frame transferred from main-thread controls to a decoder worker.
struct NativeImage {
    /// Retained immutable image whose pixels are copied only on the worker.
    image: Retained<CIImage>,
    /// Presentation timestamp in nanoseconds from the native video output.
    timestamp: u64,
}

/// Shared worker/main-thread mailbox containing only flags, values and immutable images.
#[derive(Default)]
struct State {
    /// One-shot native readiness result with duration and video capability.
    ready: Option<Result<(Duration, bool), String>>,
    /// Most recent main-thread observation of the player media clock.
    position: Duration,
    /// Latest immutable image, replacing earlier frames not consumed by the worker.
    image: Option<NativeImage>,
    /// Worker request to begin playback after preparation.
    start: bool,
    /// Worker request to seek to the beginning for another loop.
    restart: bool,
    /// Cancellation flag observed before native readiness and during playback.
    stop: bool,
    /// Acknowledgement set only after main-thread native resources are released.
    stopped: bool,
    /// Native playback failure to surface on the next worker tick.
    error: Option<String>,
}

/// One bounded native preparation request handed to the main-thread owner.
struct Request {
    /// Immutable worker preparation settings.
    spec: MediaSpec,
    /// Generation-specific shared flags and value-only observations.
    state: Arc<Mutex<State>>,
}

/// Process-wide bounded handoff installed exactly once by the main-thread service.
static REQUESTS: OnceLock<mpsc::SyncSender<Request>> = OnceLock::new();
/// Main-thread-only AVFoundation resources for one live playback generation.
struct Session {
    /// Playback control object confined to the native owner thread.
    player: Retained<AVPlayer>,
    /// Prepared asset item whose readiness and terminal status are polled.
    item: Retained<AVPlayerItem>,
    /// Video output used to obtain immutable pixel-buffer-backed images.
    output: Retained<AVPlayerItemVideoOutput>,
    /// Generation-specific shared flags and value-only observations.
    state: Arc<Mutex<State>>,
    /// Whether playback start was already submitted to AVPlayer.
    started: bool,
    /// Deadline origin for an in-progress looping seek.
    seeking: Option<Instant>,
}

/// Exclusive owner of non-Send AVPlayer resources and pending preparation requests.
pub(super) struct NativeOwner {
    /// Bounded queue of worker preparation requests.
    receiver: mpsc::Receiver<Request>,
    /// At most eight live main-thread native sessions.
    sessions: Vec<Session>,
}

impl NativeOwner {
    /// Install the bounded native request queue on the application's main thread.
    pub(super) fn new() -> Self {
        assert!(
            MainThreadMarker::new().is_some(),
            "native media owner must be created on main"
        );
        let (sender, receiver) = mpsc::sync_channel(8);
        assert!(
            REQUESTS.set(sender).is_ok(),
            "install MediaService only once"
        );
        Self {
            receiver,
            sessions: Vec::new(),
        }
    }
}

/// True while controls need regular main-thread updates. The host combines this
/// with its animation update policy; preparation must not sleep indefinitely.
pub(super) fn needs_updates() -> bool {
    ACTIVE.load(Ordering::Acquire) > 0
}

/// Pending plus live sessions; prevents the application sleeping during preparation.
static ACTIVE: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
impl NativeOwner {
    /// Poll native controls without blocking decoding or copying frame pixels.
    pub(super) fn update(&mut self) {
        let owner = self;
        let mtm = MainThreadMarker::new().expect("native media controls on main thread");
        // SAFETY: All AVPlayer objects are created, read and released on this thread.
        // AVFoundation performs decoding asynchronously; no readiness wait or CPU
        // pixel conversion is performed inside this system.
        unsafe {
            while let Ok(request) = owner.receiver.try_recv() {
                if request.state.lock().unwrap().stop {
                    request.state.lock().unwrap().stopped = true;
                    ACTIVE.fetch_sub(1, Ordering::AcqRel);
                    continue;
                }
                if owner.sessions.len() >= 8 {
                    {
                        let mut state = request.state.lock().unwrap();
                        state.ready = Some(Err("native session limit is eight".into()));
                        state.stopped = true;
                    }
                    ACTIVE.fetch_sub(1, Ordering::AcqRel);
                    continue;
                }
                let Some(path) = request.spec.path.to_str() else {
                    {
                        let mut state = request.state.lock().unwrap();
                        state.ready = Some(Err("media path must be UTF-8".into()));
                        state.stopped = true;
                    }
                    ACTIVE.fetch_sub(1, Ordering::AcqRel);
                    continue;
                };
                let url = NSURL::fileURLWithPath(&NSString::from_str(path));
                let item = AVPlayerItem::initWithURL(AVPlayerItem::alloc(mtm), &url);
                let key = NSString::from_str(
                    &objc2_core_video::kCVPixelBufferPixelFormatTypeKey.to_string(),
                );
                let format = NSNumber::new_u32(objc2_core_video::kCVPixelFormatType_32BGRA);
                let attributes: Retained<NSDictionary<NSString, AnyObject>> =
                    NSDictionary::from_slices(&[&*key], &[&*format as &AnyObject]);
                let output = AVPlayerItemVideoOutput::initWithPixelBufferAttributes(
                    AVPlayerItemVideoOutput::alloc(),
                    Some(&attributes),
                );
                item.addOutput(&output);
                let player = AVPlayer::initWithPlayerItem(AVPlayer::alloc(mtm), Some(&item));
                player.setVolume(request.spec.volume);
                player.setAutomaticallyWaitsToMinimizeStalling(false);
                owner.sessions.push(Session {
                    player,
                    item,
                    output,
                    state: request.state,
                    started: false,
                    seeking: None,
                });
            }
            owner.sessions.retain_mut(|session| {
                let mut state = session.state.lock().unwrap();
                if state.stop {
                    session.player.pause();
                    session.player.replaceCurrentItemWithPlayerItem(None);
                    state.image = None;
                    state.stopped = true;
                    ACTIVE.fetch_sub(1, Ordering::AcqRel);
                    return false;
                }
                if session.item.status() == AVPlayerItemStatus::Failed {
                    let message = session.item.error().map_or_else(
                        || {
                            "AVPlayer item failed (including output-device or decoder errors)"
                                .into()
                        },
                        |e| e.localizedDescription().to_string(),
                    );
                    state.ready = Some(Err(message.clone()));
                    state.error = Some(message);
                    session.player.pause();
                    session.player.replaceCurrentItemWithPlayerItem(None);
                    state.stopped = true;
                    ACTIVE.fetch_sub(1, Ordering::AcqRel);
                    return false;
                }
                if state.ready.is_none() && session.item.status() == AVPlayerItemStatus::ReadyToPlay
                {
                    let seconds = session.item.duration().seconds();
                    let size = session.item.presentationSize();
                    state.ready = Some(if seconds.is_finite() && seconds > 0.0 {
                        Ok((
                            Duration::from_secs_f64(seconds),
                            size.width > 0.0 && size.height > 0.0,
                        ))
                    } else {
                        Err("media duration must be finite and positive".into())
                    });
                }
                if state.restart {
                    session.player.pause();
                    session.player.seekToTime(CMTime::new(0, 1));
                    session.seeking = Some(Instant::now());
                    state.restart = false;
                    state.position = Duration::ZERO;
                    state.image = None;
                    return true;
                }
                if let Some(seeking) = session.seeking {
                    let seconds = session.player.currentTime().seconds();
                    if seconds.is_finite() && (0.0..=0.05).contains(&seconds) {
                        session.seeking = None;
                        session.player.play();
                    } else {
                        if seeking.elapsed() > Duration::from_secs(1) {
                            state.error = Some("native loop seek timed out".into());
                        }
                        return true;
                    }
                }
                if state.start && !session.started {
                    session.player.play();
                    session.started = true;
                }
                if session.started {
                    let time = session.player.currentTime();
                    let seconds = time.seconds();
                    if seconds.is_finite() && seconds >= 0.0 {
                        state.position = Duration::from_secs_f64(seconds);
                    }
                    if session.output.hasNewPixelBufferForItemTime(time) {
                        let mut display = CMTime::new(0, 1);
                        if let Some(buffer) = session
                            .output
                            .copyPixelBufferForItemTime_itemTimeForDisplay(time, &mut display)
                        {
                            // CIImage is immutable and explicitly Send+Sync in the
                            // binding. It retains its CVPixelBuffer dependency; no
                            // unsound Send wrapper around AVPlayer/native handles.
                            // Some unsupported native pixel formats return nil even
                            // though older SDK annotations mark this nonnull.
                            let image: Option<Retained<CIImage>> =
                                msg_send![CIImage::class(),imageWithCVPixelBuffer: &*buffer];
                            let Some(image) = image else {
                                state.error = Some(
                                    "native pixel buffer cannot be converted by Core Image".into(),
                                );
                                return true;
                            };
                            let pts = display.seconds();
                            state.image = Some(NativeImage {
                                image,
                                timestamp: if pts.is_finite() && pts >= 0.0 {
                                    (pts * 1e9) as u64
                                } else {
                                    0
                                },
                            });
                        }
                    }
                }
                true
            });
        }
        // Count pending requests as well as live sessions to avoid a lost wake.
    }
}

/// Worker-side handle that owns pixel conversion while main-thread controls own AVPlayer.
pub(super) struct Player {
    /// Generation-specific shared flags and value-only observations.
    state: Arc<Mutex<State>>,
    /// Validated native stream duration.
    duration: Duration,
    /// Whether the prepared stream exposes a video track.
    has_video: bool,
    /// Worker-owned color conversion context for tightly packed sRGB frames.
    context: Retained<CIContext>,
    /// Small reusable frame pool; buffers are reused only after renderer release.
    buffers: Vec<Arc<[u8]>>,
}

impl Player {
    /// Request native resources and poll readiness with cancellation and a fixed deadline.
    pub(super) fn prepare(spec: &MediaSpec, control: &Control) -> Result<Self, PreparationError> {
        let state = Arc::new(Mutex::new(State::default()));
        let sender = REQUESTS
            .get()
            .ok_or("install MediaService before preparing native media")?;
        ACTIVE.fetch_add(1, Ordering::AcqRel);
        if sender
            .try_send(Request {
                spec: spec.clone(),
                state: state.clone(),
            })
            .is_err()
        {
            ACTIVE.fetch_sub(1, Ordering::AcqRel);
            return Err("native preparation queue full".into());
        }
        let started = Instant::now();
        let prepared = loop {
            if control.cancelled(spec.generation) || started.elapsed() > PREPARE_TIMEOUT {
                State::stop_and_wait(&state)?;
                return Err(if control.cancelled(spec.generation) {
                    PreparationError::Cancelled
                } else {
                    PreparationError::Failed("native preparation exceeded eight seconds".into())
                });
            }
            if let Some(ready) = state.lock().unwrap().ready.clone() {
                break ready;
            }
            thread::sleep(Duration::from_millis(5));
        };
        let (duration, has_video) = match prepared {
            Ok(x) => x,
            Err(e) => {
                State::stop_and_wait(&state)?;
                return Err(e.into());
            }
        };
        // SAFETY: CIContext is Send+Sync; this context is used only by this worker.
        let context = unsafe { CIContext::contextWithOptions(None) };
        Ok(Self {
            state,
            duration,
            has_video,
            context,
            buffers: Vec::new(),
        })
    }

    /// Ask the main-thread owner to release resources and await a bounded acknowledgement.
    pub(super) fn stop(&mut self) -> Result<(), String> {
        State::stop_and_wait(&self.state)
    }

    /// Return the validated media duration reported at preparation.
    pub(super) fn duration(&self) -> Duration {
        self.duration
    }

    /// Report whether this stream can produce decoded frames.
    pub(super) fn has_video(&self) -> bool {
        self.has_video
    }

    /// Signal playback start without touching main-thread control objects.
    pub(super) fn start(&mut self) {
        self.state.lock().unwrap().start = true;
    }

    /// Request a bounded asynchronous seek before the next playback loop.
    pub(super) fn restart(&mut self) -> Result<(), String> {
        let mut s = self.state.lock().unwrap();
        s.restart = true;
        s.position = Duration::ZERO;
        Ok(())
    }

    /// Read current position and convert the latest native image into immutable RGBA pixels.
    pub(super) fn tick(
        &mut self,
        generation: u64,
    ) -> Result<(Duration, Option<DecodedFrame>), String> {
        let (position, image) = {
            let mut state = self.state.lock().unwrap();
            if let Some(error) = &state.error {
                return Err(error.clone());
            }
            (state.position, state.image.take())
        };
        let Some(native) = image else {
            return Ok((position, None));
        };
        // SAFETY: immutable CIImage and the private CIContext are valid for the
        // call; destination allocation, stride and bounds are checked below.
        unsafe {
            let bounds = native.image.extent();
            let (width, height) = (bounds.size.width as u32, bounds.size.height as u32);
            let size = DecodedFrame::byte_len(width, height)?;
            if self.buffers.first().is_some_and(|x| x.len() != size) {
                self.buffers.clear();
            }
            let index =
                if let Some(index) = self.buffers.iter().position(|x| Arc::strong_count(x) == 1) {
                    index
                } else if self.buffers.len() < 3 {
                    self.buffers.push(vec![0u8; size].into());
                    self.buffers.len() - 1
                } else {
                    return Ok((position, None));
                }; // retained/stalled consumers drop frames
            let bytes = Arc::get_mut(&mut self.buffers[index]).unwrap();
            let color = CGColorSpace::with_name(Some(kCGColorSpaceSRGB))
                .ok_or("sRGB color space unavailable")?;
            self.context
                .render_toBitmap_rowBytes_bounds_format_colorSpace(
                    &native.image,
                    NonNull::new(bytes.as_mut_ptr().cast()).unwrap(),
                    (width as usize * 4) as isize,
                    bounds,
                    kCIFormatRGBA8,
                    Some(&color),
                );
            // Core Image RGBA8 bitmap output is premultiplied. Bevy's normal
            // texture/sprite path expects straight alpha; normalize explicitly.
            for pixel in bytes.chunks_exact_mut(4) {
                let a = u32::from(pixel[3]);
                if a == 0 {
                    pixel[..3].fill(0);
                } else if a < 255 {
                    for channel in &mut pixel[..3] {
                        *channel = ((u32::from(*channel) * 255 + a / 2) / a).min(255) as u8;
                    }
                }
            }
            Ok((
                position,
                Some(DecodedFrame {
                    generation,
                    presentation_ns: native.timestamp,
                    width,
                    height,
                    row_stride: width as usize * 4,
                    rgba: self.buffers[index].clone(),
                    alpha: AlphaMode::Straight,
                    color_space: ColorSpace::Srgb,
                }),
            ))
        }
    }
}

impl Drop for Player {
    fn drop(&mut self) {
        self.state.lock().unwrap().stop = true;
    }
}

impl State {
    /// Request cleanup and bound the worker wait for its main-thread acknowledgement.
    fn stop_and_wait(state: &Mutex<State>) -> Result<(), String> {
        state.lock().unwrap().stop = true;
        let deadline = Instant::now() + Duration::from_secs(1);
        while !state.lock().unwrap().stopped {
            if Instant::now() >= deadline {
                return Err(
                    "native control owner did not acknowledge stop within one second".into(),
                );
            }
            thread::sleep(Duration::from_millis(2));
        }
        Ok(())
    }
}
