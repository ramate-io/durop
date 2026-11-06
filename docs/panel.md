# Controller and application modes

Run `cargo run -p durop --bin durop --locked` inside `nix develop`. The controller
and overlay share one process. The default [reaction catalog](../sample-packs/reactions)
contains 30 offline GIFs, sounds and videos with attribution and redistribution terms.

## Collection and configuration

The top row contains **Host**, **Catalog** and search. At compact widths, search
moves below the two selectors. Cards show real PNG previews, titles, media kinds
and playback controls; the responsive grid renders only visible rows plus a partial
row. Wheel/trackpad scrolling and the draggable scrollbar reach the whole collection.
Wheel input accumulates distance before advancing a row, so tiny trackpad events
do not each jump a full row; bursts are bounded and navigation discards residual motion.
Search matches every word across title, tags and category. Thumbnails are bounded
before decode (one MiB, 512×512), with a bounded 128-entry cache including failures.
Remote icons additionally satisfy the transport's 32 KiB limit.

**Host → Native** selects the primary or an enumerated display. **Host → Tailscale**
accepts the remote host's explicit Tailscale IP:port and Durop pairing token. This
is a Durop session credential, not a Tailscale API or enrollment key. Input is masked
and cleared after submission; credentials never enter preferences, URLs or command
arguments. **Share this Mac…** in Native settings configures a local listener;
the runtime verifies the address and generates its own credential. **Revoke remote**
closes that access. Keep the displayed host credential out of recordings/screenshots.
Durop does not enroll machines or alter tailnet policy.

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

Text entry supports Unicode/IME composition, arrows, Home/End, Backspace and
Ctrl/Cmd+A replacement. Cmd/Ctrl+V pastes on macOS. Escape closes a modal/menu first,
then leaves a focused editor, and otherwise stops playback. Space outside editing
plays the first visible local tile. Bundled fonts have limited script coverage;
missing glyphs do not discard the underlying search text.

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

Tests cover real keyboard/IME and wheel messages, draft validation/cancellation,
modal input isolation, controller-only routing, a 10,000-item virtualized catalog,
thumbnail bounds and preference migration. The redesign was also inspected in a
native development preview: grid, popup stacking, long titles, masked validation,
cancel, catalog loading, scrollbar reachability and active card controls. A 720×520
preview verified compact cards and form scrolling with persistent Apply/Cancel.
The window has a 400×520 minimum; desktop defaults remain 1160×820. Native
muted probes completed for GIF, MP3 and MP4; all 30 assets passed full FFmpeg decode.
These checks do not establish audible output, recorded A/V sync, fullscreen or
two-Mac acceptance; those remain in the [operator acceptance](acceptance.md).
