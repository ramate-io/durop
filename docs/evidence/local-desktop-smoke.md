# Local desktop smoke: panel and bounded GIF loop

This is a short, local GUI smoke record, not final acceptance. It records direct
observations separately from unresolved behavior. The user is handling final
desktop and two-Mac acceptance.

## Run identity

| Item | Value |
|---|---|
| Date | November 4, 2026 |
| System | Mac16,11, M4 Pro, macOS 27.0.1 |
| Display | MSI, 1920 × 1080 |
| App | Native package at `dist/native-proof-final`; package binary revision `558c12a`, containing panel revision `1c843f0` |
| Temporary loop pack | A copied pack under `/tmp/durop-panel-acceptance/DuropSmoke.app/Contents/Resources/`; not the repository pack |
| Recording | `/tmp/durop-panel-acceptance/native-gif-loop.mov`, 12 seconds, 1920 × 1080, nominal encoded rate 57.337 fps |

The temporary pack's original path was restored from preferences when the native
app launched. Package relocation did not cause it to switch to the bundled
sample pack in this run. Follow-up `30187a1` normalizes bundled defaults across app relocation and adds a
regression test. That follow-up still needs a new native GUI retest.

## Observed

- The existing application window could be read quickly. Launching or acquiring
  the app through CUA was inconsistent and took approximately 8–21 minutes in
  some attempts.
- The panel showed four sample tiles. Searching for `gif` reduced the results to
  one tile. GIF and video tiles reached `Ready` and later `Finished` states.
- A looping GIF showed `Playing` and its per-instance Stop button. The app's
  visible ASCII chrome rendered as expected.
- A reviewed frame from the local whole-display recording showed the colored GIF
  at the bottom right with the desktop visible through its transparent area.
  The recording was reviewed locally; its path is given above.

## Not established

- The source GIF is only 4 × 4 pixels and was magnified. This is a basic visual
  compositing observation, not a fringe, orientation, or image-quality benchmark.
  The 57.337 fps figure is the recording's nominal encoding rate, not Durop or
  game frame-rate evidence.
- Cmd+A and Ctrl+A in the search field appended `a` instead of selecting the
  field contents in this GUI run, despite the earlier unit fix in `1c843f0`.
  A source-based hypothesis is that winit can deliver a flags-only
  `ModifiersChanged` event without physical keyboard input; raw events were not
  captured during this run. A raw-event fix is underway in PR 20 and needs a new
  native GUI retest.
- Repeated StopAll pointer attempts and Escape attempts during the active GIF did
  not produce an observed `Stopped` state. `Finished` appeared after the GIF's
  timer elapsed; that does not prove StopAll or Escape succeeded. CUA/native
  window input routing may have affected the attempts. Host and real-loopback
  tests pass, but they do not establish GUI StopAll behavior.
- A UI Quit attempt changed the PID observed by a subsequent accessibility
  query, which relaunched the app. Clean UI quit was therefore not verified. The
  task-owned earlier smoke process (PID 99492) and native package process (PID
  2419) were terminated by the root agent. Those process cleanups were not UI
  Quit acceptance. The bounded GIF had no audio and ended.
- The recording contains private background windows. Keep the recording and
  extracted frames local; do not copy or upload them.

No claim is made here for audible output, system-audio capture, A/V
synchronization, game focus or performance, fullscreen support, a 30-minute
session, or two-Mac operation. Native fullscreen remains unsupported based on
the separate evidence in [overlay proof](../overlay-proof.md). All new fixes
require another GUI acceptance run before being recorded as passed.
