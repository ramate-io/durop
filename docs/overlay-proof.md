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
before allowing it to become visible. A no-ivar subclass of winit's existing
window class refuses key/main-window status and maps `makeKeyAndOrderFront:` to
`orderFront:`. This preserves winit's other behavior and leaves the panel's class
unchanged. It also disables mouse handling and shadows and requests all Spaces,
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
   } | ./target/debug/durop
   ```

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
`git diff --check` also passed. Tests cover command parsing, Retina bounds,
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

Cloud Linux compilation and logic checks do not satisfy the runtime acceptance
items above. No Mac runtime, fullscreen/Spaces, keyboard/controller focus,
physical monitor removal, GPU compositing, recording, or idle-power result is
claimed by this document. Record the actual device results and recording in the
PR before declaring #2 complete.
