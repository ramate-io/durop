# Overlay proof (#2)

Run from the checkout with `nix develop --command cargo run --locked` on a
logged-in Apple Silicon Mac. The terminal is the local safety control surface;
keep it attached. This proof adds no media decoder, networking, or production UI.
One Bevy App owns the normal panel and a hidden borderless overlay window. Each
camera targets its own window and render layer. The PNG has an opaque pink square
with a transparent center and margin. Its source dimensions are 256 × 256 pixels;
it appears at 256 logical points before the optional pulse animation.

## Local controls

| Panel key | Terminal command | Behavior |
| --- | --- | --- |
| Space | `show` | Pulse the fixture for five seconds, including while unfocused |
| — | `hold` | Keep a static fixture visible until stopped |
| Escape | `stop` | Hide overlay and disable its camera |
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
The overlay camera is disabled and its window hidden when empty. Static fixtures
also use reactive waits. Fixture loading and the five-second animation temporarily
use continuous updates in both focus states. This is redraw policy, not an idle
power measurement; verify actual redraw/CPU behavior on the Mac.

## macOS boundary

`src/overlay_macos.rs` configures only the overlay, on the AppKit main thread,
before allowing it to become visible. Three method adapters on `WinitWindow`
check a per-instance associated marker: overlays refuse key/main-window status
and map `makeKeyAndOrderFront:` to `orderFront:`; unmarked panel windows forward
to the original implementations. Inherited methods are overridden only on
`WinitWindow`, leaving `NSWindow` itself unchanged. The adapter preserves every
window's original class (ISA): replacing the class of an initialized AppKit
window caused a computed-property assertion on macOS 27.0.1. A native Objective-C
dispatch regression checks marked/unmarked instances, inherited method isolation
and unchanged classes; desktop acceptance must additionally exercise real windows.
It also disables mouse handling and shadows and requests all Spaces,
stationary/fullscreen-auxiliary behavior. The overlay uses a transparent clear
color and macOS post-multiplied surface alpha.

These flags do not prove support for native fullscreen games, exclusive display
modes, or every Spaces configuration. The adapter is coupled to winit 0.30.13's
NSWindow policy: review it when upgrading Bevy/winit. Linux checks cannot prove
macOS focus, compositing, controller routing, capture, or fullscreen behavior.
The backend itself documents a possible monitor-unplug panic on some platforms;
the application handles removal once the backend reports it, but cannot catch a
backend event-loop crash. Record any reproduction as unsupported.

## Manual acceptance procedure (Mac required)

Record the macOS version, Mac/GPU, display arrangement, physical resolutions,
scaling, game/window modes, capture application, and Git commit with the results.
Run the README formatting/check/test/clippy/build commands first.

1. Launch once. Verify two application windows belong to the same `durop` process
   in Activity Monitor, with a normal input-taking panel and initially hidden
   overlay. Use `displays`, then `display N`, then `hold`. Confirm only the pink
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
   `show`, and both windows hidden. Empty/static states should settle without
   continuous redraws; `show` should continue unfocused and return to idle.

## Validation record

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
removed. Native-fullscreen presentation is unsupported in this observed
configuration; the retained implementation is the `6929aef` ISA fix and original
collection behavior flags.

Beyond normal-desktop compositing, pulse/clearing and local safety controls,
click/keyboard/controller passthrough, focus retention under an unfocused
trigger, broader fullscreen/Spaces support, mixed displays, physical monitor
removal, precise animation timing and idle-power acceptance remain pending.
Game/controller and mixed-monitor hardware were unavailable. Compilation and
logic checks do not satisfy those runtime acceptance items. Record the remaining
device results before declaring #2 complete.
