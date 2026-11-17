# Workspace architecture

Durop has one runnable binary and five libraries. The root package is the default
workspace member, so both `cargo run --locked` and the explicit
`cargo run -p durop --bin durop --locked` launch the application from the repository
root. Use `--workspace` for checks and tests so library tests are included.

| Package | Responsibility | Internal dependencies |
| --- | --- | --- |
| `durop-protocol` | Serializable commands, outcomes, snapshots and events | None |
| `durop-catalog` | Pack validation, immutable cache revisions and source adapters | None |
| `durop-host` | Authoritative admission, deduplication and playback lifecycle | Protocol, catalog |
| `durop-media` | Bounded GIF/AVFoundation decoding and generation-tagged output | None |
| `durop-transport` | Authenticated Tailscale sessions and WebSocket protocol | Protocol |
| `durop` | Bevy UI, application coordination, native window policy and diagnostics | All five libraries |

The application pins Bevy 0.20.1 with granular rendering and window features.
`bevy_ui_widgets` supplies text input behavior and input focus, `ui_picking`
connects pointer input to widgets, and `system_clipboard` enables operating-system
copy/paste. `DefaultPlugins` registers their plugins; text fields pair
`EditableText` state with the `TextInput` widget.

Libraries must not import application modules or Bevy. The application translates
host effects to media requests and returns generation-tagged backend events. The
media library exposes a main-thread service for native progress; `src/media_app.rs`
adapts it to Bevy. The transport reports optional acknowledgment timings through an
injected observer, keeping diagnostics ownership in the application.

## Interchangeable behavior

Catalog sources share the `CatalogSource` contract. Directory and Git adapters
materialize a source into cache-owned staging paths; common validation and
publication policy stays in the cache. Source selection remains configuration,
while source behavior is dispatched through the trait. Catalog revision pinning
prevents a refresh from mutating resources used by active playback.

Application commands share the `ControllerBackend` admission contract. The native
adapter submits to the in-process host queue; the Tailscale adapter submits through
an authenticated connection with bounded normal and priority queues. Admission
does not mean the host accepted the command or that playback started. Session,
revision and generation checks still apply, and a disconnected controller never
queues actions for later replay. “Native” means local control, not an unauthenticated
LAN socket.

The panel separates **From** (independently enabled local and remote inputs),
**To** (one local or remote playback destination), and **Catalog**. Each remote
source has a host-issued credential and revocable lease. Source-scoped command
identities prevent two controllers from colliding in the host's deduplication
history. Disabling one source invalidates queued work without changing the other
sources. Local input has its own revocable guard; application shutdown can still
stop playback when local controls are disabled.

A relay advertises the selected destination's session, catalog and snapshots,
and returns its actual admission outcomes. It never falls back to local playback
when disconnected. Changing destinations closes the old input session and rebuilds
the explicitly enabled listener against the new destination; controllers must
reconnect. Direct self-connections are rejected. Guarded forwarding cancels a
revoked source's still-pending command through an authenticated per-connection
cancellation barrier, preserving other sources. Cancellation cannot undo an
operation the destination already accepted. Both relay and destination should run
this version to support cancellation acknowledgments.

`InvitationProvider` and `SecretStore` live in `durop-transport`, with no Bevy
dependency. The application runs their HTTPS/Keychain work off the UI thread.
Infrastructure OAuth credentials configure policy; a host's user-owned API token
creates device-sharing invitations; per-controller Durop credentials authorize
commands. These capabilities are distinct. The host token comes from its process
environment or explicit secure storage and never enters a controller handoff.
Controller credentials and source lists are session-only and are not preferences.

Typestate makes lifecycle boundaries explicit: a materialized catalog must validate
before it can publish, and a disconnected/connecting remote controller cannot use
the connected controller's admission implementation. State transitions consume the
prior stage. External I/O, connection loss and asynchronous cancellation still
require runtime checks; typestate cannot promise a network connection stays alive.

Media decoder selection remains private to `durop-media`; callers use `MediaWorker`
and the same prepare/start/stop/event contract for GIF and AVFoundation. Native
objects stay behind the media boundary. Platform window callbacks belong in the
application because they implement presentation and focus policy.

## Ownership and tests

Keep construction, validation and transformations on their owning types. Bevy
systems and foreign callbacks may remain free functions because their signatures
are integration boundaries. Document public API contracts and private invariants,
including cancellation, limits and thread affinity. Separate declarations and
implementations with blank lines.

Run a focused library suite with `cargo test -p durop-catalog --locked` or
`cargo test -p durop-transport --locked`. Run all suites with
`cargo test --workspace --locked`. The default-off transport `test-support` feature
exposes loopback helpers to application test dependencies; ordinary app/package
builds do not enable it. Tests using these helpers do not establish real tailnet or
two-Mac acceptance.

Native packaging builds `target/native-package/aarch64-apple-darwin/release/durop`
and places that executable in the same app bundle. A regular `cargo build --release`
uses `target/release/durop`.
Application resources resolve from the source root in development and from the
bundle's `Contents/Resources` after packaging. Library fixtures resolve relative
to their crate's location; runtime library APIs receive explicit resource paths.

## macOS overlay surface

The normal controller remains a winit window in the single Durop process.
`overlay_macos.rs` creates a separate nonactivating `NSPanel` and plain `NSView`
on the main thread. Before Bevy's first surface extraction, it redirects only
the overlay's `RawHandleWrapper` (and holder) to that view. The original winit
window stays hidden and supplies render dimensions/event-loop ownership. No
initialized object's class changes; no winit class methods are swizzled and no
winit view is reparented.

A retaining association on the original native window owns the panel for as long
as Bevy's retained winit handle can back an in-flight GPU surface. A non-send
resource owns main-thread presentation state; its teardown orders the panel out
without closing a view that may still be used by the renderer. This bridge depends
on Bevy 0.20.1/winit 0.30.13 surface creation and raw-handle ownership. Review it
on upgrades: redirection must precede the first extraction, not replace a live
surface afterward. The application supports one overlay entity/surface.

The native display ID selects `NSScreen.frame`, including the Dock/menu region.
The panel refuses key/main status, ignores mouse events, and orders in without
activating the app at the public status-window level (above the Dock). All-Spaces
and fullscreen-auxiliary behavior applies to this genuine panel. Empty playback
still clears transparently; invalid/missing geometry hides the panel and disables
the camera. Actual qualifications and input-testing limits are in
[overlay proof](overlay-proof.md).
