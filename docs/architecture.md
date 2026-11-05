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
