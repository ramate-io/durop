# Local panel and application modes

Run `durop` for the local panel and overlay in one process. The bundled `sample-packs/podcast-pack` includes generated WAV, MP4, alpha HEVC and GIF examples. Click a tile to submit Play; acceptance, preparation and actual Started state are distinct. Individual Stop buttons and STOP ALL control the same authoritative host as remote controllers. No preview is substituted for a selected remote destination.

The grid renders twelve tiles at a time. Mouse-wheel scrolling and Previous/Next change the visible rows. Search matches all words across title, tags and category, without decoding thumbnail video. PNG thumbnail input is bounded before decode (one MiB, 512×512); both successful and failed thumbnail loads share a bounded 128-entry cache. Remote icons also satisfy transport's stricter 32 KiB limit.

Click the search or pack field to edit. Text entry supports Unicode characters, IME preedit/commit, arrow navigation, Home/End, Backspace and Ctrl/Cmd+A replacement. Composition remains separate from committed search text. Enter or Escape ends editing; Escape outside an editor stops playback. Space outside an editor plays the first visible local tile. Search typing never invokes playback shortcuts. The default font can show missing glyphs for scripts outside its coverage; editing/search retain the Unicode text. Broader font coverage remains a visible limitation. On macOS Cmd/Ctrl+V pastes at the focused editor; pairing input remains masked and is cleared after Connect.

Enter a pack directory and select Load / refresh. HTTPS Git packs use `https://HOST/REPOSITORY.git#REF`; local Git packs use `git+file://ABSOLUTE/PATH#REF`. Refresh stops the current host and revokes remote access before preparing a new immutable catalog. A failed load leaves an actionable error and the host disabled; it does not replace the current pinned catalog with partial content. Reloading starts a new host session. Decoder failures appear next to their affected tile.

Select This Mac or Remote host intentionally. Remote mode displays the remote catalog and authoritative snapshot after Connect; disconnected commands fail without a local fallback or offline replay. Type the Tailscale address and session pairing credential, then Connect. The secret is masked in its input field and cleared when submitted. It is never stored in preferences or put in a URL/process argument. Enable remote host explicitly to create the listener and a fresh pairing credential; Revoke remote immediately closes that access. The displayed host pairing secret is private: exclude it from recordings/screenshots intended for sharing.

Examples:

```sh
durop --mode local --pack ./sample-packs/podcast-pack --display 0
durop --mode host --pack ./sample-packs/podcast-pack --listen-tailscale 100.64.0.10:48732
durop --mode controller --connect ws://100.64.0.10:48732/control
durop --self-check
durop --version
```

Host mode starts with its primary panel hidden and does not take input. Controller mode needs no local pack and cannot start a local host. The display button cycles the selected display; the selection is remembered locally. Native fullscreen Spaces did not show the overlay on the qualification Mac, so use a normal desktop/windowed or borderless application. Display and system-audio capture are separate recording-software setup steps.

Hide/close preserves a deliberately enabled host. On macOS an explicit Dock reopen shows the existing panel, including in host mode, without a second process. Terminal commands `panel`, `hide`, `stop`, `play PACK/CLIP`, `displays`, `display N` and `quit` provide a second control path without global keyboard interception. If neither a native reopen hook nor terminal control is available, the panel stays available for stopping. Quit issues StopAll and waits for media-resource teardown acknowledgements before exiting.

On macOS benign preferences live at `~/Library/Application Support/io.ramate.durop/preferences.json` and immutable pack snapshots at `~/Library/Caches/io.ramate.durop/packs`. Other platforms use `XDG_CONFIG_HOME`/`XDG_CACHE_HOME` with `.config`/`.cache` fallbacks and the `io.ramate.durop` subdirectory. Preferences contain only pack source, query and display, use a bounded atomic JSON write, and never restore remote access or a remote destination. The bundled default pack is stored as a logical reference and resolved against the current app's Resources after relocation or upgrade; legacy absolute bundle-default paths migrate automatically. Explicit external pack paths remain unchanged. Bad preferences do not prevent the panel from starting. Shared pack metadata remains unchanged.

The executable resolves `Contents/Resources/assets` and `Contents/Resources/sample-packs/podcast-pack` in a macOS bundle, independent of working directory. Development runs use the checkout resources. `--self-check` validates the bundled catalog, media and icons without launching a window, networking or audio.

Automated checks cover real KeyboardInput/IME messages, Unicode editing, the twelve-tile viewport over 10,000 entries, thumbnail bounds, preference corruption and the host/media/transport contracts. Native UI, actual playback composition and recorded audio synchronization still require desktop acceptance; unit tests do not establish recording correctness. Two-Mac acceptance is performed by the operator.
