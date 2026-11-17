# Overlay proof (#2)

Run from the checkout with `nix develop --command cargo run --locked` on a
logged-in Apple Silicon Mac. The terminal is the local safety control surface;
keep it attached. The application includes media, networking and the normal controller UI.
One Bevy App owns the normal panel and overlay; macOS renders the latter through
a separate nonactivating native panel backed by a hidden winit event window. Each
camera targets its own window and render layer. The PNG has an opaque pink square
with a transparent center and margin. Its source dimensions are 256 × 256 pixels;
it appears at 256 logical points before the optional pulse animation.

## Local controls

| Panel key | Terminal command | Behavior |
| --- | --- | --- |
| Space | `show` | Pulse the fixture for five seconds, including while unfocused |
| — | `hold` | Keep a static fixture visible until stopped |
| Escape | `stop` | Clear playback/fixture to transparency |
| D | `next` | Cycle the selected display |
| — | `displays` | Print current display indexes, pixel bounds, and scales |
| — | `display N` | Select an index from the latest display listing |
| H | `hide` | Hide panel while retaining terminal controls |
| — | `panel` | Reopen/focus the panel |
| Q | `quit` | Exit the process |

Closing the panel stops the overlay and hides the panel, retaining both window
entities. Terminal input remains usable while the panel is hidden. If stdin ends
or fails, the panel reappears and further hide/close requests keep it visible;
its stop and quit keys remain available. Run interactively for hidden-panel tests.
The terminal queue is bounded to 32 commands and wakes the same Bevy event loop;
there is no second application process or global keyboard hook.

The initial target is the primary monitor (or the first reported monitor). A
removed target clears the active overlay; it never transfers an active fixture
to another screen. A later trigger can choose a remaining screen. Display indexes
are transient: run `displays` again after plugging/unplugging. Window placement
uses physical desktop coordinates and physical dimensions, with the monitor's
scale factor for logical sprite size. The pinned Bevy version does not refresh
existing Monitor geometry, so this proof refreshes it from its native backend
handles on window/event-loop updates.

Idle rendering uses indefinite reactive low-power waits in both focus states.
Once native setup and valid display geometry are ready, the click-through overlay
window and camera remain available between plays. An empty scene renders a
transparent clear instead of hiding with an old drawable that could reappear on
the next play. This does not request continuous idle frames; static fixtures
also use reactive waits. Missing or invalid displays disable the surface.
Fixture loading and the five-second animation temporarily
use continuous updates in both focus states. This is redraw policy, not an idle
power measurement; verify actual redraw/CPU behavior on the Mac.

## macOS boundary

`src/overlay_macos.rs` creates a genuine borderless, nonactivating `NSPanel`
with a plain render `NSView`. It redirects Bevy's overlay surface handle before
first extraction, retaining the hidden winit window for ownership, dimensions
and event-loop updates. The normal controller window is unchanged. See the
[ownership contract](architecture.md#macos-overlay-surface).

The panel's subclass is registered **before allocation**. It refuses key/main
window status and prevents AppKit from constraining its frame to the visible
work area. No initialized window changes class; no winit class is swizzled.
The panel ignores mouse events, stays visible when the app deactivates, and uses
`orderFrontRegardless` without activation or key-window calls. `NSScreen.frame`
is selected by native display ID, with AppKit's logical coordinates. The public
status-window level is above the Dock (observed 25 vs. 20), without changing
Dock preferences or application-wide presentation options. Transparent pixels
naturally reveal underlying content; opaque pixels can cover the Dock/menu bar.

`CanJoinAllSpaces | Stationary | IgnoresCycle | FullScreenAuxiliary` now apply
to this native panel. They were insufficient on the previous winit NSWindow.
The implementation preserves transparent clear and post-multiplied surface alpha.
API references: [NSPanel](https://developer.apple.com/documentation/appkit/nspanel),
[fullscreen auxiliary](https://developer.apple.com/documentation/appkit/nswindow/collectionbehavior-swift.struct/fullscreenauxiliary),
[window levels](https://developer.apple.com/documentation/coregraphics/cgwindowlevel).

These contracts are not a promise about every game, exclusive display mode or
macOS version. Native evidence below qualifies one environment. Linux tests cannot
prove AppKit compositing/focus. Recheck surface creation/ownership on Bevy/winit
upgrades, and physical display removal on the actual hardware.

## Manual acceptance procedure (Mac required)

Record the macOS version, Mac/GPU, display arrangement, physical resolutions,
scaling, game/window modes, capture application, and Git commit with the results.
Run the README formatting/check/test/clippy/build commands first.

1. Launch once. Verify one `durop` process owns the normal input-taking controller,
   hidden winit overlay owner and separate transparent native overlay panel. Use `displays`, then `display N`, then `hold`. Confirm only the pink
   fixture appears on the selected screen and panel text never appears there.
2. Check both the center hole/margins and solid square: desktop pixels must show
   through transparent areas without a background rectangle. Click, drag, type,
   and scroll in another app underneath both opaque and transparent regions.
   Test controller input in the chosen game while the overlay is visible.
3. Test an unfocused trigger without switching to Durop at trigger time. For
   example enter the development shell and build first. Start the binary with
   the following pipeline, wait until its panel is ready, then press Enter in the
   terminal. The panel hides; switch to the target app within five seconds and
   observe its input/focus throughout the pulse:

   ```sh
   nix develop
   cargo build --locked
   {
     printf 'Wait for the Durop panel, then press Enter: ' >&2
     IFS= read -r ready
     printf 'hide\n'
     sleep 5
     printf 'show\n'
     sleep 8
     printf 'panel\n'
     cat
   } | BEVY_ASSET_ROOT="$PWD" ./target/debug/durop
   ```

   Run this from the checkout root. `BEVY_ASSET_ROOT` points to the directory
   containing `assets/`, so direct execution finds `assets/overlay-proof.png`.
   Unlike `cargo run`, direct execution does not supply `CARGO_MANIFEST_DIR`;
   without an explicit asset root, Bevy searches beside the executable instead.
   Use your configured target directory for the binary path if customized.
   No compilation takes place during the timed test. The terminal accepts
   commands after the delayed test. Repeat on a normal
   desktop, a borderless game, and native fullscreen in another Space. Test
   Mission Control and switching Spaces. Record each outcome; do not infer one
   mode's support from another. If the overlay is absent or steals focus, mark
   that mode unsupported and keep the issue open.
4. Trigger `show` repeatedly while unfocused. Verify smooth five-second pulses
   and automatic disappearance. Test `hold`, `stop`, `hide`, `panel`, and closing
   the panel while a drop is active. Stop from the terminal while hidden. End
   stdin and verify the panel remains available. `quit` must end the process.
5. Test mixed Retina/non-Retina screens, negative monitor coordinates, resolution
   and scaling changes, rearrangement, and unplugging the selected display while
   showing a fixture. Confirm placement/sizing follows the selected screen and
   unplugging stops the drop without moving it onto another display or crashing.
6. Make a whole-display recording with the selected display. Include the panel,
   hide it, trigger the PNG, demonstrate input through the pink region, wait for
   automatic clearing, and stop/reopen via terminal. Inspect the saved recording
   for correct alpha/no rectangle. Save a brief recording and attach it to the PR.
7. With Instruments/Activity Monitor, compare empty, static `hold`, animated
   `show`, and the panel hidden with an empty overlay. Empty/static states should settle without
   continuous redraws; `show` should continue unfocused and return to idle.

## Historical validation record (previous NSWindow implementation)

Cloud Linux (Rust 1.95.0) passed `cargo fmt --all -- --check`,
`cargo check --locked`, `cargo test --locked` (five tests),
`cargo clippy --all-targets --locked -- -D warnings`, and `cargo build --locked`.
`git diff --check` also passed. A Linux window smoke was unavailable: the host
had no DISPLAY/WAYLAND_DISPLAY, X11 socket, Xvfb, or Vulkan ICD. No Linux
window/rendering runtime result is claimed. Tests cover command parsing, Retina bounds,
panel close/reopen, terminal loss, and display-removal clearing.

The adapter also passed Apple Silicon cross-check and cross-clippy:

```sh
CARGO_FEATURE_NO_NEON=1 cargo check --locked --target aarch64-apple-darwin
CARGO_FEATURE_NO_NEON=1 cargo clippy --all-targets --locked --target aarch64-apple-darwin -- -D warnings
```

This temporary environment switch disables BLAKE3's NEON C build for the check.
Without it, the cloud host's Linux C compiler rejects Apple `-arch` and
`-mmacosx-version-min` arguments. No build workaround or SDK path is committed.
The cross-check validates Rust types; it does not establish a native linked build.

On November 4, 2026, native validation used macOS 27.0.1 (26A434) on a
Mac mini (Mac16,11), M4 Pro with 12 CPU cores, 16 GPU cores and 48 GB RAM.
The only reported display was an MSI MP161 E2, 1920 × 1080 at 60 Hz,
scale 1, origin (0, 0). Commit `3c78e54` passed native formatting, locked
check/test/build and strict clippy through `nix develop`, without the cross-check's
`CARGO_FEATURE_NO_NEON` workaround. A panel screenshot confirmed rendered text,
and Activity Monitor showed one `durop` PID.

The first `hold` crashed in AppKit's `NSDP_getComputedPropertyValue` via
`_realMakeFirstResponder` and `orderFront:`. Both the raw executable and a
temporary `.app` wrapper failed, with macOS IPS crash reports. The wrapper under
`/tmp` contained an identical binary, a plist and a resource/assets link for
desktop automation; it was not production packaging.

The fix preserves the initialized window's ISA and dispatches focus policy by
instance marker. Native checks include six passing tests, including Objective-C
dispatch coverage for marked overlays, ordinary windows and inherited methods.
This test uses NSObject probes, not real NSWindows. After desktop access returned,
the `6929aef` build survived `hold`. Its normal-desktop static recording showed
the magenta fixture with transparent center and margins. The temporary wrapper
needed an `assets` link beside its executable; a link only in `Resources` did
not load the fixture. The normal-desktop recording also showed a pulse and
automatic clearing. Terminal `hide`, `stop`, `panel`, and panel close/reopen were
responsive; `quit` exited with status 0. A process CPU spot check read 0.0%,
which does not establish redraw counts or idle power.

The [reviewed normal-desktop clip](evidence/normal-desktop-6929aef.mov) is a
32-second excerpt (source seconds 8–40) of `normal-input-6929aef.mov`.
Its SHA-256 is
`3184100e23192ac79dea767f2ceb25dcf0bf129a2d71d0e4eb82e6d3f09b4a9a`.
It documents desktop compositing and pulse/clearing only. Full source recordings
remain local because they include private desktop content.

In a separate native-fullscreen TextEdit Space, representative frames at 3, 10,
20 and 35 seconds of a 45-second recording showed no fixture despite `show`
then `hold`. Text entry reached TextEdit, but absence of the overlay does not
establish input passthrough or retained focus: the targeted automation could
inject text or reactivate its chosen app. A second 35-second recording tested
the public
[`canJoinAllApplications`](https://developer.apple.com/documentation/appkit/nswindow/collectionbehavior-swift.struct/canjoinallapplications)
flag on the same native-fullscreen TextEdit Space. Frames at 3, 10, 20 and 30
seconds again showed no fixture; an independent NSWorkspace query reported
`com.apple.TextEdit` frontmost during `hold`. The ineffective candidate flag was
removed. Native-fullscreen presentation was unsupported with that NSWindow implementation.
The new NSPanel result below supersedes this limitation for the tested TextEdit
configuration; the failed experiment remains useful evidence against flags alone.

Beyond normal-desktop compositing, pulse/clearing and local safety controls,
click/keyboard/controller passthrough, focus retention under an unfocused
trigger, broader fullscreen/Spaces support, mixed displays, physical monitor
removal, precise animation timing and idle-power acceptance remain pending.
Game/controller cases were not exercised; only one non-Retina display was
available to this run. Compilation and
logic checks do not satisfy those runtime acceptance items. Record the remaining
device results before declaring #2 complete.

## Native panel validation (2026-11-11)

Environment: macOS **27.0.1 (26A434)**; Mac mini **Mac16,11**, M4 Pro,
12 CPU/16 GPU cores, 48 GB RAM; one **MSI MP161 E2, 1920 × 1080 at 60 Hz**,
scale 1, origin (0,0). Target application: system TextEdit, normal desktop,
maximized window and native fullscreen Space. No game/controller or second
monitor was available to this run.

Source: `codex/macos-native-overlay` based on `04a01f1`. The exercised debug
executable SHA-256 was
`011f082eeac9467fce85203c1d60f0ad04fa4003aa378c0236af844279775d1b`.
A temporary `.app` wrapper supplied identity/resources for desktop automation;
it ran the one Durop binary and was not distribution-package validation.
The committed [probe pack](../fixtures/native-overlay) is the exact GIF/metadata
used, with an opaque magenta border and transparent center. Reproduce with:

```sh
nix develop --command cargo run --locked -- --pack fixtures/native-overlay
# Attached terminal:
hide
play native-proof/border
stop
play native-proof/border
displays
display 0
panel
quit
```

Trigger through the attached terminal/remote controller while TextEdit remains
frontmost. Inspect whole-display screenshots, not TextEdit-only screenshots:
app-window captures omit the overlay. The fixture loops for at most ten minutes.
Screenshots are retained locally, excluded from the published branch because
they contain desktop content. The published window audits contain only the
selected display geometry and TextEdit/Durop/Dock window metadata, without
document titles or contents. Rendering observations below were made from the
local screenshots; the audit text alone does not prove rendered pixels.

| Case | Observed result |
| --- | --- |
| Normal desktop | Opaque border reaches every screen edge; transparent center shows underlying windows. Raw desktop captures remain local because unrelated windows were visible. |
| Maximized TextEdit and Dock | Locally reviewed active/cleared screenshots: the opaque bottom border covers the Dock, which reappears after stop. Full 1920 × 1080 panel at level 25; Dock level 20. |
| TextEdit native fullscreen Space | Locally reviewed active/cleared screenshots: actual border rendering, transparent center and clearing. The fullscreen document occupies the separate Space, without desktop window chrome. |
| Unfocused terminal trigger | `NSWorkspace.frontmostApplication` reported TextEdit immediately before and after the fullscreen play command, without intervening UI input. [Window audit](evidence/native-panel-2026-11-11/fullscreen-windows.txt) and [desktop audit](evidence/native-panel-2026-11-11/desktop-windows.txt) preserve post-trigger observations. These are samples, not continuous focus telemetry. |
| Enter/exit fullscreen | Entered fullscreen while Durop was running; played there, stopped/replayed, then exited fullscreen during active playback. The border remained rendered on the desktop afterward. |
| Repeated playback and clearing | Terminal stop removed the border; a second play rendered it again while TextEdit stayed fullscreen. |
| Keyboard/mouse automation | Targeted click/text entry reached TextEdit during playback. Targeted automation can bypass ordinary hit testing or reactivate its target, so this does **not** establish physical mouse/keyboard/controller passthrough. |
| Panel and shutdown | Terminal `panel` reopened the normal controller after hidden playback; accessibility reported its standard window/buttons. Terminal `quit` exited status 0 with no crash. |
| Monitor selection | `displays` reported the one monitor, and `display 0` was accepted. No physical multi-monitor or hotplug claim. |

The Control-Left/Right automation did not produce an observable Space change;
independent switching between existing Spaces remains **unverified**. Dock
accessibility timed out, so **Dock-click reopening and normal panel interaction**
still need a manual regression check, despite terminal reopening succeeding.
Desktop automation also intermittently reported a locked session/stalled; results
above use the subsequently unlocked, observable TextEdit session. No Dock settings,
macOS protections or user activation policies were changed.

Unverified: physical click/drag/scroll and keyboard passthrough, real game or
controller input, exclusive fullscreen, other apps/macOS versions, multiple
Spaces switching, mixed DPI/negative-position monitors, hotplug, recording/video
smoothness, long-running resource use, and production `.app` signing/packaging.
For an unqualified game, use its windowed/maximized mode as the best verified
presentation fallback, and perform the physical-input checks before relying on it.

Native workspace fmt/check/test/clippy (`-D warnings`), rustdoc (`-D warnings`),
build and Python checks passed. Rust: **199 passed, 3 explicit probes ignored**;
Python: **14 passed**. The updated display-removal and stop/replay tests verify
that the macOS winit owner stays hidden while the camera remains active, and
invalid geometry disables presentation. These tests do not substitute for
actual native screenshots or the outstanding input/hardware checks.
