# Media backend proof (#3)

The POC selects macOS AVFoundation for audio/video and the Rust `gif` decoder
for GIF timelines. The measured native prototype decoded H264/AAC and HEVC with
alpha, so there is no current requirement to introduce GStreamer and its plugin
packaging. This is an internal backend boundary in the existing Rust executable;
Swift scripts generate/probe fixtures only and are not runtime dependencies.

`Cargo.lock` pins Bevy 0.19.1, objc2 0.6.4, Apple framework bindings 0.3.2 and
`gif` 0.14.2. AVFoundation/CoreMedia/CoreVideo/CoreImage/CoreGraphics are system
frameworks. The Rust bindings and `gif` use MIT/Zlib/Apache-compatible dependency
licenses recorded in their crates; distributors must retain applicable notices.
System codecs ship with macOS; this repository bundles no codec implementation
or GStreamer runtime. Cross-platform AV playback is not implemented: Linux
supports the portable GIF boundary and returns a per-clip unsupported error for
native audio/video.

## Threading and clock

Install `media::MediaPlugin` exactly once before preparing media. Its NonSend
native owner keeps typed AVPlayer/AVPlayerItem/VideoOutput handles on the main
thread, as required by the bindings. It only submits controls and polls status
and due output; AVFoundation performs asynchronous decoding. No synchronous
readiness wait or CPU color conversion runs in this system. Local cached files
are the only sources accepted by this boundary.

Each `MediaWorker` owns one host instance. `prepare(MediaSpec)` accepts strictly
increasing nonzero generations, followed by `start(generation)` or
`stop(generation)`. All events and frames carry the generation. A stopped or
replaced accepted prepare emits a terminal acknowledgement even if canceled
before decoding. `Started` waits for advancing backend time or a due frame,
rather than reporting submission of `play` as playback. Native stop is
acknowledged after pause/item detachment; if the main control owner does not
respond within one second, the worker reports failure instead of claiming stop.

A single AVPlayer owns a video's audio and media time. VideoOutput is pulled at
that player's current media position, so frames are due in the same clock as
audio. The soundtrack is never started through a second player. POC audio uses
AVPlayer's system-default output route; muted probes do not establish audibility.
Native item errors propagate individually, including reported decoder/output
errors; physical output-device loss and default-device changes remain untested.
A maximum duration of ten minutes bounds a session, including loops. Loop seeks
wait for the native position to return to zero before resuming.

Production preparation passes `volume: 1.0` from `src/app.rs`; the native owner
applies that value with `AVPlayer::setVolume` before starting the same player.
The command-line `--media-probe` deliberately passes `volume: 0.0`. Extracting
video frames does not require a separate audio player or Bevy audio component.

The six MP4s in `sample-packs/reactions/videos` contain one H.264 video stream
and **no audio stream**. Their silent source excerpts cannot demonstrate video
soundtrack playback. For an actual combined A/V input, `fixtures/media/flash-beep.mp4`
contains H.264 video and AAC mono audio at 44.1 kHz. Its soundtrack decoded to
nonzero samples in the November 5, 2026 check (mean −26.3 dB, peak −11.7 dB).
The muted native probe prepared, advanced and completed that combined asset;
audible hardware output and recorded A/V synchronization still require separate
acceptance. Launch with `--pack fixtures/media` to select the flash/beep video
in the application when performing that check.

Immutable CIImage values (explicitly Send/Sync in the binding) retain due native
pixel buffers and cross a latest-only slot to the worker. Core Image renders
explicit sRGB RGBA8 into one of at most three reusable CPU buffers. The worker
unpremultiplies pixels before publishing `AlphaMode::Straight`, matching Bevy's
ordinary sprite blending. Source row stride is handled by Core Image; output
stride is explicitly width×4. This is a CPU upload boundary, not shared Metal
textures or a zero-copy claim. Core Image may use acceleration internally.

GIF timelines preserve local frame offsets, transparent pixels, logical
background and disposal-to-background/restore-previous semantics. The fixture
regression checks the composed pixels and centisecond timing. Zero-delay GIF
frames use an explicit 100 ms fallback. GIF has its own monotonic clock because
it has no audio. Its authored infinite-loop flag cannot bypass `max_duration`;
only the host's bounded `looping` option repeats the timeline.

The host must keep the Bevy update loop running while any instance is preparing
or playing, combining `media::needs_updates()` with its own instance/animation
policy. This services native control requests; it is not permissible to stop
updating while a native worker waits for readiness. The application consumes
`latest_frame()` for reusable Bevy Image uploads, `position()` and
`drain_events()` through its existing host/media integration.

## Bounds and tests

Command queues hold four commands per worker; at most eight accepted generations
may have unconsumed terminal events. Preparation backpressure preserves terminal
acknowledgements instead of dropping them. The event bound is 24 (Prepared,
Started, terminal for each outstanding generation). Stop/cancellation bypass the
command queue. Native control requests and live sessions are capped at eight;
the host must also cap simultaneous MediaWorkers.

Files are limited to 256 MiB, frame dimensions to 4096×4096, one RGBA frame to
64 MiB, native conversion buffers to three, GIF snapshots to 128 MiB/1000 frames,
and native readiness to eight seconds. GIF composition additionally needs its
canvas, optional restore snapshot and decoder storage. Frames are latest-only;
obsolete due frames are replaced, and retained consumers exhaust the three-slot
native pool by causing frame drops rather than more allocations. Limits are
per worker, not a measured total-process RSS guarantee.

Native tests cover GIF pixels/disposal/timing, preparation without autostart,
stop/clear, rejected stale frames, acknowledgement of canceled queued generations,
bounded looping, and a stalled consumer retaining only the latest frame.
Formatting, locked check/test/build and strict all-target clippy pass on Apple
Silicon macOS 27.0.1. Native controls and codec behavior require a logged-in Mac;
portable tests alone do not establish playback.

## Reproduce and measured results

```sh
nix develop --command cargo build --locked
./target/debug/durop --media-probe fixtures/media/flash-beep.mp4
./target/debug/durop --media-probe fixtures/media/alpha-hevc.mov
./target/debug/durop --media-probe fixtures/media/flash-beep.wav
./target/debug/durop --media-probe fixtures/media/disposal.gif
./target/debug/durop --media-probe fixtures/media/flash-beep.mp4 --cancel
./target/debug/durop --media-probe fixtures/media/corrupt.mp4
```

The final command is expected to exit 1 with an individual error. These are
muted, bounded probes of the actual Rust main-owner/worker path, without Bevy
window/rendering initialization. They pump the main run loop for native status
updates. `--cancel` waits until a native request exists, then stops before
readiness and checks its acknowledgement. The simpler Swift prototype can be
repeated with `swift scripts/media/probe.swift PATH`; its measurements differ
because it does not exercise the Rust color-conversion bridge.

On November 4, 2026, a Mac mini M4 Pro (12 CPU/16 GPU cores, 48 GB), macOS 27.0.1,
produced the following individual Rust samples; they are not percentiles:

| Fixture | Ready | Start request → first converted frame | Frames | Result |
| --- | ---: | ---: | ---: | --- |
| H264/AAC MP4, 128×128/30 fps/2 s | 296.34 ms | 62.88 ms | 60 | Position reached 2 s; alpha 255 |
| HEVC-with-alpha MOV, same size/rate/duration | 241.83 ms | 13.75 ms | 60 | Position reached 2 s; alpha 0–255 |
| PCM WAV, 48 kHz mono/2 s | 234.45 ms | No video | 0 | Position reached 2 s |
| Transparent GIF, 4×4/0.7 s | 4.88 ms | 3.93 ms | 4 | Position reached 0.703 s; alpha 0–255 |
| Cancel native preparation | No readiness | No frame | 0 | Stopped acknowledged in 10.49 ms |
| Corrupt MP4 | Error | No frame | 0 | `Cannot Open`, exit 1; no process crash |

Readiness and first-converted-frame latency are separate. Neither measures a
frame actually presented by Bevy or sound reaching an output device.
`terminal_cleanup_ms` measures post-terminal probe cleanup and must not be
reported as playback stop latency; only `cancellation_ack_ms` measures the
explicit cancellation acknowledgement in this probe.

One additional H264/AAC process-cost sample (`/usr/bin/time -l`, muted) took
2.34 seconds wall time, 0.19 seconds user CPU and 0.09 seconds system CPU, with
68,829,184 bytes maximum RSS and 90,506,272 bytes peak memory footprint. This
includes startup and two seconds of decoding/conversion, excludes the cost of
external macOS codec services and Bevy rendering, and is not a sustained CPU/GPU
benchmark. GPU usage was not measured.

The first Rust alpha run exposed a real problem: the default native output
format could not be represented by CIImage, whose older binding marked the
returned image nonnull. The backend now explicitly requests BGRA and treats a
nil native image as a per-clip error. Both H264 and alpha probes passed after
that correction. A sandboxed native encoder also reported no available encoder;
the same fixture generation passed with native codec-service access, so that
was an environment restriction, not a codec qualification failure.

Audible output, whole-screen/system-audio flash/beep synchronization, actual
Bevy frame presentation/reusable texture behavior, unfocused playback,
physical device-loss recovery, sustained CPU/GPU cost and renderer-stall RSS
remain device acceptance work. No playback, lip-sync, zero-copy or fullscreen
game-support claim follows from these muted decode probes.
