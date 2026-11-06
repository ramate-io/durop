# Reactions: screen classics and real cues

30 offline clips: **12 animated GIFs, 12 MP3 sounds and 6 H.264 MP4 videos**,
about 19 MiB including 320-pixel PNG thumbnails. Includes Obama's mic drop,
Keaton's falling-house stunt, Chaplin, Feline Follies, the Spinning Dancer,
real applause/laughter/crickets/drums, and six historic NASA callouts. This is a
curated collection of reusable classics and reaction cues; no popularity ranking
is implied. The MP4 excerpts are silent because their original sources are silent.
The DOE nuclear-test clips depict an actual nuclear flash/cloud.

All six MP4s have one H.264 video stream and **zero audio streams**, verified by
stream inspection. For audible cues, select one of the twelve sound entries.
Durop's production AVFoundation path also plays audio embedded in videos, using
the same player and clock as the video; this pack's silent excerpts cannot test
that capability. The separate `fixtures/media/flash-beep.mp4` contains an actual
AAC soundtrack for A/V acceptance. Native command-line probes are intentionally
muted and do not establish speaker output.

Each item has a distinct title, tags, real media and an adjacent thumbnail.
Visual excerpts from the same source occupy separate nonoverlapping intervals.
Very short original animations repeat within their bounded playback duration.
Playback does not download anything, run this script or require FFmpeg.

## Credits and redistribution

**Media are not covered by the repository's MIT license.** Each output and its
PNG thumbnail retain the source terms listed below. Keep this README, SOURCES.lock
and LICENSES directory with redistributed copies. Our resizes, excerpts, GIF/MP4/
MP3 conversions and thumbnails are adaptations; sharealike adaptations are
provided under the same CC BY-SA version as their source. No endorsement is
claimed. Public-domain notices describe the source's stated status; consult the
linked source for its jurisdiction and basis, particularly early US films.

- [SOURCES.lock](SOURCES.lock) records each source URL, source SHA-256, author,
  license/notice URL, original duration, selected interval and transformation.
- [SHA256SUMS](SHA256SUMS) pins all output media, metadata and thumbnails.
- [CC BY-SA 4.0](LICENSES/CC-BY-SA-4.0.txt),
  [CC BY-SA 3.0](LICENSES/CC-BY-SA-3.0.txt) and
  [CC0 1.0](LICENSES/CC0-1.0.txt) full legal texts accompany the assets.

The mic-drop adaptation is distributed under **CC BY-SA 4.0**, credited to
Jdlrobson and The White House, as explicitly identified in the Commons structured
license metadata. The underlying official White House footage is a US government
work; this does not remove the adaptation's attribution/sharealike notice.

NASA audio comes from the agency's [Historical Sounds](https://www.nasa.gov/historical-sounds/)
collection and its linked [media-use guidelines](https://www.nasa.gov/nasa-brand-center/images-and-media/).
NASA-produced content generally is not copyrighted in the US, subject to the
stated third-party-content exceptions. These are mission recordings, not music.
Credit NASA; do not suggest NASA endorses a product or person. NASA insignia,
logotypes and identifiers have separate legal protections; no NASA logos are
supplied as pack branding. Recognizable people's privacy/publicity rights and
commercial advertising restrictions remain applicable; the media policy is not a
blanket trademark or endorsement license.

| Clip | Author / credit | Source and terms |
| --- | --- | --- |
| Obama: mic drop | The White House; GIF adaptation by Jdlrobson | [Source](https://commons.wikimedia.org/wiki/File:Barack_Obama_drops_the_mic.gif) · [CC BY-SA 4.0 (GIF adaptation); underlying White House footage is a US government work](https://creativecommons.org/licenses/by-sa/4.0/) |
| Chaplin: hat tip (illustration) | Akumiszcza (source photo); Nevit Dilmen (image/GIF adaptation) | [Source](https://commons.wikimedia.org/wiki/File:Charlie_Chaplin.gif) · [Public domain](https://commons.wikimedia.org/wiki/File:Charlie_Chaplin.gif) |
| Keaton: house falls around him | Joseph Schenck and Buster Keaton (production), Charles Reisner (direction). | [Source](https://commons.wikimedia.org/wiki/File:GIF_of_Buster_Keaton_in_%22Steamboat_Bill_Jr%22_1928.gif) · [Public domain](https://commons.wikimedia.org/wiki/File:GIF_of_Buster_Keaton_in_"Steamboat_Bill_Jr"_1928.gif) |
| Chaplin: sharing with the dog | Charlie Chaplin | [Source](https://commons.wikimedia.org/wiki/File:Dog%27s_Life.gif) · [Public domain](https://commons.wikimedia.org/wiki/File:Dog's_Life.gif) |
| Feline Follies: tantrum | Pat Sullivan | [Source](https://commons.wikimedia.org/wiki/File:Animated_GIF_from_the_1919_Feline_folies_by_Pat_Sullivan.gif) · [Public domain](https://commons.wikimedia.org/wiki/File:Animated_GIF_from_the_1919_Feline_folies_by_Pat_Sullivan.gif) |
| Feline Follies: not amused | Pat Sullivan | [Source](https://commons.wikimedia.org/wiki/File:Animated_GIF_from_the_1919_Feline_folies_by_Pat_Sullivan.gif) · [Public domain](https://commons.wikimedia.org/wiki/File:Animated_GIF_from_the_1919_Feline_folies_by_Pat_Sullivan.gif) |
| Cat: that look | Amrutha Unni | [Source](https://commons.wikimedia.org/wiki/File:Cat_funny_gif.gif) · [CC BY-SA 4.0](https://creativecommons.org/licenses/by-sa/4.0) |
| Spinning Dancer: which way? | Nobuyuki Kayahara | [Source](https://commons.wikimedia.org/wiki/File:Spinning_Dancer.gif) · [CC BY-SA 3.0](https://creativecommons.org/licenses/by-sa/3.0) |
| Fred Ott: the sneeze (1894) | W.K.-L. Dickson | [Source](https://commons.wikimedia.org/wiki/File:Fred_Ott_Sneeze_L.gif) · [Public domain](https://commons.wikimedia.org/wiki/File:Fred_Ott_Sneeze_L.gif) |
| A knowing wink | Gebruiker:Dmbekker | [Source](https://commons.wikimedia.org/wiki/File:Animated_winking_Smiley_colored.gif) · [Public domain](https://commons.wikimedia.org/wiki/File:Animated_winking_Smiley_colored.gif) |
| Nuclear test: flash | USDE | [Source](https://commons.wikimedia.org/wiki/File:GreenhouseDog.gif) · [Public domain](https://commons.wikimedia.org/wiki/File:GreenhouseDog.gif) |
| And they are off! | Eadweard Muybridge | [Source](https://commons.wikimedia.org/wiki/File:Muybridge_race_horse_animated.gif) · [Public domain](https://commons.wikimedia.org/wiki/File:Muybridge_race_horse_animated.gif) |
| Applause and cheers | starlite | [Source](https://commons.wikimedia.org/wiki/File:Clapping_hurray.ogg) · [Public domain](https://commons.wikimedia.org/wiki/File:Clapping_hurray.ogg) |
| Rimshot beat | Pannage | [Source](https://commons.wikimedia.org/wiki/File:Rimshot_loop_120bpm.ogg) · [CC BY-SA 4.0](https://creativecommons.org/licenses/by-sa/4.0) |
| Bursting out laughing | ezwa | [Source](https://commons.wikimedia.org/wiki/File:Bursting_out_laughing.ogg) · [Public domain](https://commons.wikimedia.org/wiki/File:Bursting_out_laughing.ogg) |
| Crickets: awkward silence | DrTrumpet | [Source](https://commons.wikimedia.org/wiki/File:Audio_H%C3%B6rbild_Grillenzirpen_-_nachts_um_3_im_F%C3%B6hrenwald_M%C3%B6dling.ogg) · [CC BY-SA 4.0](https://creativecommons.org/licenses/by-sa/4.0) |
| Drum roll, please | Iwan Sounds and DIY | [Source](https://commons.wikimedia.org/wiki/File:Drum_Roll_Intro.ogg) · [CC0](http://creativecommons.org/publicdomain/zero/1.0/deed.en) |
| Bluebird train horn | NIRVAAN222222 | [Source](https://commons.wikimedia.org/wiki/File:The_horn_of_a_Bluebird_railcar.ogg) · [CC0](http://creativecommons.org/publicdomain/zero/1.0/deed.en) |
| Houston, we have had a problem | NASA; recorded mission crew and Mission Control | [Source](https://www.nasa.gov/historical-sounds/) · [US government work; NASA media-use guidelines](https://www.nasa.gov/nasa-brand-center/images-and-media/) |
| The Eagle has landed | NASA; recorded mission crew and Mission Control | [Source](https://www.nasa.gov/historical-sounds/) · [US government work; NASA media-use guidelines](https://www.nasa.gov/nasa-brand-center/images-and-media/) |
| One small step | NASA; recorded mission crew and Mission Control | [Source](https://www.nasa.gov/historical-sounds/) · [US government work; NASA media-use guidelines](https://www.nasa.gov/nasa-brand-center/images-and-media/) |
| Apollo 11: liftoff | NASA; recorded mission crew and Mission Control | [Source](https://www.nasa.gov/historical-sounds/) · [US government work; NASA media-use guidelines](https://www.nasa.gov/nasa-brand-center/images-and-media/) |
| Mercury: zero G | NASA; recorded mission crew and Mission Control | [Source](https://www.nasa.gov/historical-sounds/) · [US government work; NASA media-use guidelines](https://www.nasa.gov/nasa-brand-center/images-and-media/) |
| Godspeed, John Glenn | NASA; recorded mission crew and Mission Control | [Source](https://www.nasa.gov/historical-sounds/) · [US government work; NASA media-use guidelines](https://www.nasa.gov/nasa-brand-center/images-and-media/) |
| Keaton: hello there | Buster Keaton | [Source](https://commons.wikimedia.org/wiki/File:Buster_Keaton_as_a_bellboy_in_the_March_18_1918_movie_The_Bell_Boy.gif) · [Public domain](https://commons.wikimedia.org/wiki/File:Buster_Keaton_as_a_bellboy_in_the_March_18_1918_movie_The_Bell_Boy.gif) |
| Keaton: the deadpan look | Buster Keaton | [Source](https://commons.wikimedia.org/wiki/File:Buster_Keaton_as_a_bellboy_in_the_March_18_1918_movie_The_Bell_Boy.gif) · [Public domain](https://commons.wikimedia.org/wiki/File:Buster_Keaton_as_a_bellboy_in_the_March_18_1918_movie_The_Bell_Boy.gif) |
| Keaton: sudden exit | Buster Keaton | [Source](https://commons.wikimedia.org/wiki/File:Buster_Keaton_as_a_bellboy_in_the_March_18_1918_movie_The_Bell_Boy.gif) · [Public domain](https://commons.wikimedia.org/wiki/File:Buster_Keaton_as_a_bellboy_in_the_March_18_1918_movie_The_Bell_Boy.gif) |
| Feline Follies: startled | Pat Sullivan | [Source](https://commons.wikimedia.org/wiki/File:Animated_GIF_from_the_1919_Feline_folies_by_Pat_Sullivan.gif) · [Public domain](https://commons.wikimedia.org/wiki/File:Animated_GIF_from_the_1919_Feline_folies_by_Pat_Sullivan.gif) |
| Feline Follies: cat escapes | Pat Sullivan | [Source](https://commons.wikimedia.org/wiki/File:Animated_GIF_from_the_1919_Feline_folies_by_Pat_Sullivan.gif) · [Public domain](https://commons.wikimedia.org/wiki/File:Animated_GIF_from_the_1919_Feline_folies_by_Pat_Sullivan.gif) |
| Nuclear test: mushroom cloud | USDE | [Source](https://commons.wikimedia.org/wiki/File:GreenhouseDog.gif) · [Public domain](https://commons.wikimedia.org/wiki/File:GreenhouseDog.gif) |

## Rebuild and check

The checked-in pack is ready to play. Its builder requires Python 3.9+ and an
FFmpeg build supporting libx264/libmp3lame/GIF. The recorded outputs were built
with FFmpeg 7.1 (task-local imageio-ffmpeg 0.6.0 tool; not bundled in the app).
All conversion inputs are individual short GIF/audio/video files; no full film
is fetched. Original source files and the tool binary are not checked in.

```sh
# Offline CI: output hashes, unique assets, provenance, PNG sizes and counts.
python3 sample-packs/reactions/pack_tool.py verify --metadata-only
# Offline full media decode; point FFMPEG at your installed executable if needed.
python3 sample-packs/reactions/pack_tool.py verify --ffmpeg /path/to/ffmpeg
# Explicitly download hash-pinned source files (max 32 MiB each) and rebuild.
python3 sample-packs/reactions/pack_tool.py build --download --ffmpeg /path/to/ffmpeg
```

Acquisition fails if a source changes, is unavailable or exceeds its limit.
Downloads use a temporary external cache, never this pack directory. The builder
uses finite inputs for GIF palette generation and bounds each conversion to 90s.
Different FFmpeg versions may produce different output hashes; review new output
and provenance before accepting regenerated SHA256SUMS.

Validated locally: all 30 clips load through `durop_catalog::Catalog::load`,
all 30 media files decode fully in FFmpeg, all output hashes match, and both
full and metadata-only pack checks pass. Structural catalog validation still
reports decoder readiness as unchecked; these checks do not establish audible
hardware output, AVFoundation presentation or visual compositing in the app.
