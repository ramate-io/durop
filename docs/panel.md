# Controller and application modes

Run `cargo run -p durop --bin durop --locked` inside `nix develop`. The controller
and overlay share one process. The default [reaction catalog](../sample-packs/reactions)
contains 31 offline GIFs, sounds and videos with attribution and redistribution terms.

## Collection and configuration

The top row contains **From**, **To**, **Catalog** and search. At compact widths, search
moves below the selectors. Cards show real PNG previews, titles, media kinds
and playback controls; the responsive grid renders only visible rows plus a partial
row. Wheel/trackpad scrolling and the draggable scrollbar reach the whole collection.
The desktop toolbar is 68 logical pixels tall; compact windows wrap complete control
rows. Search and collection updates preserve the surrounding controls and their text
selection. Form separators sit outside the scrolling viewport so the last action can
scroll fully into view.
The collection uses Bevy UI’s `ScrollPosition` and vertical scroll overflow.
Trackpad movement updates a continuous logical-pixel offset, including fractional
deltas and OS-provided momentum; there are no row thresholds or snapping. Mouse
wheel lines convert to pixels, and the scrollbar follows the same continuous offset.
Search matches every word across title, tags and category. Thumbnails are bounded
before decode (one MiB, 512×512), with a bounded 128-entry cache including failures.
Remote icons additionally satisfy the transport's 160 KiB limit.

**From** independently enables local controls and authenticated remote controllers.
Selecting a source opens its controls without changing its permission. **To → This Mac**
selects the primary or an enumerated display. **To → Remote host**
accepts the remote host's explicit Tailscale IP:port and Durop pairing token. This
is a Durop session credential, not a Tailscale API or enrollment key. Input is masked
and cleared after submission; credentials never enter preferences, URLs or command
arguments. The selected remote source’s **Listener** settings configure a local listener;
the runtime verifies the address and generates its own credential. **Revoke remote**
closes that access. Keep the displayed host credential out of recordings/screenshots.
Durop does not enroll machines or alter tailnet policy. Protected host invitation
credentials are loaded only after adding a remote controller or enabling a valid
Tailscale listener, or when explicitly saving/removing a token. Ordinary startup, opening From, local playback and connecting to a remote host
do not access Keychain. An explicit `--listen-tailscale` launch starts host sharing
and may request credentials.

**Catalog** offers the bundled pack, an absolute local directory, or an HTTPS Git
repository with a literal ref and optional pack folder. The repository example is
`https://github.com/ramate-io/durop.git`, ref `main`, folder `sample-packs/reactions`.
Refresh stops playback and revokes remote access before preparing an immutable
catalog. Failure retains the previous pin, leaves hosting disabled and shows an
error; successful replacement begins a new session. See [pack sources](packs.md).

Each modal edits a draft. **Apply** validates before emitting an action; **Cancel**,
close, backdrop or Escape discard edits. The form body scrolls in short windows
while its heading and action footer remain visible. Background playback controls
are blocked while configuring, but **STOP ALL** remains available. Controller-only
launches reject local host/catalog Apply with an explanation because their catalog
belongs to the remote host.

Click a card to submit Play. Acceptance, preparation and actual Started state remain
distinct. A card's Stop button targets its latest active occurrence; **STOP ALL**
stops every occurrence. Remote mode displays the remote catalog and authoritative
snapshot; disconnected commands have no local fallback or offline replay.

Headings, hints, status messages and displayed values can be selected and copied
with the mouse and Command/Control+C. Action button labels keep their click behavior.
Selecting a read-only label does not edit its value or run playback shortcuts.
Horizontal rules separate the modal heading, form sections and action footer.

Search and ordinary form fields use Bevy 0.20’s `TextInput` and `EditableText` with
an engine-rendered blinking caret and selection highlight. Click to place the caret;
drag or Shift+arrow to select text, and double-click to select a word. Command+A/C/X/V
on macOS (Control on other desktops) select, copy, cut and paste through the system
clipboard. Unicode/IME composition, word navigation, Home/End and Delete/Backspace
use the engine editor. Inputs are bounded to 4096 characters and remain on one line;
long values scroll horizontally. Their entities, selection and uncommitted IME
composition survive search filtering, playback snapshots and network status refreshes.
A small event-order adapter supplies native modifier flags to Bevy before each key,
including Command press/key/release bursts within one frame.

Bevy 0.20 does not provide password masking. Pairing tokens and host API keys therefore
use a separate masked field: type or paste a value, Command/Control+A to replace it,
and Backspace/Delete to remove it. Secret fields never expose plaintext through a
rendered text input and do not copy or cut credentials to the clipboard. They do not
provide caret placement, partial selection or IME composition. Secrets are cleared
on submission or when their draft is discarded.

Opening an editable form focuses its first visible field. Tab and Shift+Tab cycle
within that form; hidden fields and background search cannot receive editing. Escape
closes a modal/menu first, then leaves a focused editor, and otherwise stops playback.
Space outside editing plays the first visible local tile. Bundled fonts have limited
script coverage; missing glyphs do not discard the underlying search text.

## Experimental visual placement

GIF and video cards include **Edit**. The placement dialog changes the selected
clip's scale and horizontal/vertical position for the destination host display.
Audio cards have no placement controls.

- **Scale** is the fitting area's width as a percentage of the host display
  width, greater than 0% and at most 100%. Explicit adjustments scale both
  display dimensions together. Ordinary adjustments retain the catalog's fitting
  rule; Fullscreen selects cover fitting. Reset restores the original catalog
  fitting area and rule.
- **Horizontal** runs from 0% at the left edge to 100% at the right edge.
- **Vertical** runs from 0% at the top edge to 100% at the bottom edge.

Position percentages interpolate over the available space around the rendered
media, keeping it on-screen: 50% centers it and 100% aligns its far edge. Values
refer to the host's selected display, not the controller window, and accept two
decimal places.

**Fullscreen** sets scale to 100%, centers the media and selects cover fitting,
so the entire display is filled even when the catalog uses contain. The image
keeps its aspect ratio; excess edges are cropped symmetrically. **Top left**,
**Top right**, **Bottom left** and **Bottom right** align the media to that corner
while retaining the current scale and fitting rule. These shortcuts
only edit the draft until you choose Apply or Send.

**Apply** remembers the adjustment for this controller session and updates active
occurrences of this clip without restarting their decoder or soundtrack. **Send**
applies the adjustment and requests playback. **Reset to defaults** restores the
catalog's scale, anchor and fitting rule in the draft; Apply or Send commits that
reset. The renderer consumes placement changes before presenting frames, including
the first frame of a preparing occurrence.
**Cancel** discards the draft. New host sessions, destinations or catalog revisions
clear remembered overrides; this experiment does not rewrite pack metadata or
persist placement preferences to disk.

Local and Tailscale controllers use the same host command policy. Invalid values,
stale catalogs and finished/stopping occurrences cannot be updated. The wire
protocol is version 2 (`durop.v2`); run the same build on host and controller.

## Launch and persistence

```sh
durop --mode local --pack ./sample-packs/reactions --display 0
durop --mode host --pack ./sample-packs/reactions --listen-tailscale 100.64.0.10:48732
durop --mode controller --connect ws://100.64.0.10:48732/control
durop --self-check
```

Host mode starts with its panel hidden. Controller mode needs no local pack and
cannot start a local host. Native fullscreen Spaces remain unsupported on the
qualification Mac; use a normal desktop/windowed or borderless application.

Dock reopen and terminal `panel`, `hide`, `stop`, `play PACK/CLIP`, `displays`,
`display N` and `quit` provide controls without global keyboard interception. If
neither Dock reopen nor terminal control is available, the panel stays available.
Quit requests StopAll and waits for media-resource teardown.

Benign macOS preferences live at
`~/Library/Application Support/io.ramate.durop/preferences.json`; immutable pack
snapshots use `~/Library/Caches/io.ramate.durop/packs`. Other platforms use
`XDG_CONFIG_HOME`/`XDG_CACHE_HOME` with `.config`/`.cache` fallbacks. Preferences
contain only pack source, query and display. They never restore remote access or
a remote destination. Bundled pack tokens follow app relocation; legacy bundle
paths migrate automatically. Explicit external paths remain unchanged.

Bundles resolve assets and sample packs under `Contents/Resources`, independently
of the working directory; development uses checkout resources. `--self-check`
validates the bundled catalog without opening windows, networking or audio.

## Design and verification

The warm paper/ink palette and display typography are inspired by
[Symbolic Software](https://symbolic.software/). Expletus Sans and IBM Plex Mono
are bundled with their [OFL notices](../assets/fonts/README.md); no proprietary
Berkeley Mono font or Symbolic artwork is copied.

Tests cover the actual Bevy keyboard/IME edit lifecycle, selection and focus across
view refreshes, final-edit-plus-Apply ordering, modal Tab isolation, actual pointer
caret placement/selection, native modifier/focus-loss ordering, low-power redraws, hidden editor
retirement/reopening, real wheel messages, draft validation/cancellation,
modal input isolation, controller-only routing, a 10,000-item virtualized catalog,
thumbnail bounds and preference migration. The redesign was also inspected in a
native development preview: grid, popup stacking, long titles, masked validation,
cancel, catalog loading, scrollbar reachability and active card controls. A 720×520
preview verified compact cards and form scrolling with persistent Apply/Cancel.
The continuous-scroll revision was checked in the native preview: small gestures
moved cards by 34px then 4px, crossing a virtual-row boundary retained the partial
row offset, and the final row and filtered empty state remained reachable. Track
clicks worked; automated native thumb dragging was inconclusive, while fractional
dragging and grab-point preservation pass system tests.
The window has a 400×520 minimum; desktop defaults remain 1160×820. Native
muted probes completed for GIF, MP3 and MP4; all 31 assets passed full FFmpeg decode.
These checks do not establish audible output, recorded A/V sync, fullscreen or
two-Mac acceptance; those remain in the [operator acceptance](acceptance.md).

The Bevy 0.20 migration was checked separately in the native macOS preview:
From/source draft opening and cancellation, visible bounded dialogs, continuous
8-pixel scroll movement and scrollbar track navigation, visible caret/selection,
Command+A/C/X/V, and copying both a card title and a wrapped form paragraph into
search. Long titles wrap and section rules remain visible. Desktop and compact
viewport bounds also run through Bevy layout tests; native IME candidate-window
and standalone distribution acceptance remain separate from these checks.

Regression checks use the bundled fonts and Bevy's text measurement and glyph layout,
not just empty layout boxes. They inspect each frame's text geometry and render-camera
assignment through search edits and virtual-row changes. This catches the transient
camera loss caused by detaching retained text before replacing its ancestors.
The updated native preview confirmed the 68-pixel toolbar, 46-pixel form fields,
repeated search edits and selection, and fixed surrounding controls across multiple
virtual-row scroll boundaries. Draft and Git form footer actions remained fully visible.
