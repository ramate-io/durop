# Independently authored media fixtures

All signals, pixels and fixture generators in this directory and `scripts/media/`
were authored for Durop. The fixture content is dedicated to the public domain
under CC0 1.0; the generators remain covered by Durop's MIT license. No third-party
media was copied. These fixtures may be redistributed with Durop.

This directory is also a schema-v1 pack (`packId: media-proof`) for host/panel
integration. The four metadata entries select audio, GIF, A/V and alpha video;
each is bounded to four seconds. Their PNG icons reuse Durop's independently
authored `assets/overlay-proof.png` fixture. Corrupt and intermediate files have
no catalog entries and are used only by explicit probe commands.

| File | Contents |
| --- | --- |
| `flash-beep.wav` | Two seconds, mono signed 16-bit PCM at 48 kHz; 880 Hz beeps of 80 ms starting at 0.5 and 1.5 seconds |
| `silent-h264.mov` | Intermediate 128×128, 30 fps, two-second H264 video with three-frame flashes starting at 0.5 and 1.5 seconds |
| `flash-beep.mp4` | H264 video and AAC audio exported together from the preceding two authored tracks |
| `alpha-hevc.mov` | Two-second HEVC-with-alpha video of a magenta ring with a transparent center/margins and timed flashes |
| `disposal.gif` | Four-frame 4×4 RGBA timeline with transparent pixels, local offsets, delays 100/200/300/100 ms, disposal-to-background and restore-previous |
| `corrupt.mp4` | Deliberate plain-text invalid container; must fail individually |

Regenerate the WAV/GIF/corrupt inputs with `python3 scripts/media/generate.py`.
On a Mac with Xcode Command Line Tools, generate the videos with:

```sh
swift -module-cache-path /tmp/durop-media-swift-cache scripts/media/generate-video.swift fixtures/media
```

Native encoders need access to the macOS codec services; sandbox-denied encoder
errors do not establish unsupported codecs. Encoding output need not be byte
identical across macOS releases. Track times permit a future whole-screen plus
system-audio recording to measure flash/beep synchronization; the fixture alone
does not demonstrate that synchronization.
