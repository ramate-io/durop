# Drop packs, version 1

`durop.json` contains `schemaVersion: 1`, `packId`, and `title`. Every other
`.json` file recursively discovered outside `.git` is clip metadata. Metadata
paths sort lexically, and parent folders form initial categories. IDs use ASCII
letters, digits, hyphens and underscores. Identity is `packId/id`; moving a clip
folder does not change it. Duplicate clip IDs are errors. Titles must be nonempty.
Unknown fields and unknown schema versions are errors.

Each clip requires `id`, `title`, `asset`, and an adjacent, same-basename `.png`.
PNG icons are supplied by pack authors. Asset paths are relative to the metadata
folder, without `..`, absolute paths or backslashes. Canonical paths (including
symlinks) must remain in the pack root; directory cycles/aliases are rejected.
All JSON files must be metadata; put provenance in Markdown, not JSON.

`tags` defaults to `[]`. `playback` defaults to `retrigger: restart`, `loop: false`,
with a 10000 ms duration limit. Retrigger values are `restart`, `ignore-if-playing`, `stack`.
`maxDurationMs`, when supplied, is an integer from 1 through 600000 milliseconds (10 minutes). Loops are always bounded.
`visual` is optional (including for audio); when present it requires `anchor`
(`top-left`, `top-right`, `bottom-left`, `bottom-right`, `center`),
`widthFraction` (> 0 and <= 1 of overlay width), and `fit` (`contain`, `cover`).
For a fullscreen transition, use `anchor: center`, `widthFraction: 1`, and
`fit: cover`. This fills the entire display rectangle on landscape, portrait,
and ultrawide monitors, uniformly scaling and cropping overflow as needed.
GIF and native video use the same placement rule. Smaller catalog cards retain
their square fitting area; `contain` preserves the complete frame inside it.
These are immutable playback defaults. The experimental controller
[placement editor](panel.md#experimental-visual-placement) sends per-occurrence
overrides without modifying this metadata.

The v1 format policy accepts WAV, MP3, OGG, FLAC, M4A, AAC, MP4, MOV, WebM and GIF
extensions, case-insensitively. This is an authoring allowlist, not a promise of
codec support. Catalog validation checks nonempty media and basic PNG headers;
`DecoderReadiness::Unchecked` explicitly requires future backend probing before
playback. Corrupt media and a PNG with a valid header may still fail decoding.
Unresolved Git LFS pointers are rejected; materialize local assets first. Remote
Git LFS fetching is not supported. Source media is copied unchanged; generated
playback variants belong in a separate derived cache.

JSON Schemas in `schemas/` describe authoring shapes; the Rust loader also checks
filesystem containment, unique IDs, assets and icons. `examples/sample-pack` is
an independently authored, redistributable example with asset provenance.

## Preparing and pinning cached snapshots

The application calls `CatalogCache::prepare` on a worker with an explicitly
selected `PackSource`: a local directory, a local Git repository/ref, or HTTPS
repository/ref, optionally selecting a pack subdirectory. The controller's
**Catalog** menu configures these sources. `refresh`
prepares and validates first, then changes the active `Arc<Catalog>` in one
assignment. `pin` retains the old revision and its asset paths for a live session.
Preparation alone makes an update available without changing the active session.
Prepared assets are ordinary local files and need no network access at trigger
time. The application owns the cache directory and must retain pinned snapshots;
this version deliberately performs no garbage collection.

Git fetch resolves a branch/tag/commit to an exact commit. The Git CLI is invoked
through argument arrays, with hooks disabled, no interactive prompts, and
protocols restricted to HTTPS (plus file only for explicit local repositories).
Objects are exported directly via `ls-tree`/`cat-file`: there is no checkout,
filter/smudge, LFS fetch, submodule recursion, or execution of pack content.
Git symlinks and gitlinks/submodules are rejected; local directory symlinks are
supported only within the root. HTTPS URLs cannot contain credentials, queries,
fragments or escaped bytes. Existing Git credential helpers may service HTTPS
requests; manifests and errors never persist supplied tokens or Git stderr.
References must be literal branch/tag names or commit IDs, without revision
expressions. Errors omit remote diagnostics to avoid credential disclosure.

For a pack nested in a repository, use `GitCatalogSource::in_directory` or
the controller's **Git repository → Pack folder** field. The launch syntax is
`--pack https://github.com/ramate-io/durop.git#main:sample-packs/reactions`.
Only the selected subtree is exported; the immutable revision still identifies
the full commit. Relative paths cannot contain empty, dot, parent, or `.git`
components. Cache identities include the selected directory, so two packs in
one commit cannot collide.

Cache directory keys are SHA-256 over length-delimited source identity, immutable
revision and `catalog-v1` derived-format version. Local directory revisions are
SHA-256 content digests of copied manifest, metadata, icons and referenced media;
they are not Git commits and include uncommitted changes. Local source directories
must be quiescent while preparing; the copied result is revalidated. Keep the
application-owned cache outside local source packs. Git identities use canonical
local repository paths or the supplied HTTPS URL. Equivalent URL spellings may
create separate cache entries. Metadata, icons and original media are staged on
the same filesystem, validated, and atomically renamed into the snapshot cache.
Failed preparation removes staging and leaves the active snapshot unchanged.

Limits: each file/Git output is at most 256 MiB, a prepared pack at most 1 GiB,
local discovery at most 10000 directories/metadata files and depth 64. Each Git
operation has a 120 second timeout covering the process and output reader. On
Linux/macOS, timeout cleanup kills the isolated Git process group, including
helpers that inherit stdout; a helper that deliberately detaches from that group
is outside this cleanup guarantee. These conservative limits are format/runtime
policy, not decoder capability. Future UI integration should present preparation
errors before enabling a session. Crash recovery can load a previously prepared
snapshot using `load_local` on its cache directory, but revision selection and
persisted active-session configuration are left to application integration.

## Native acceptance checks

In addition to the README's locked checks, run:

```sh
nix develop --command python3 scripts/catalog-https-acceptance.py
nix develop --command cargo test -p durop-catalog --locked cache_tests::https_public_export_acceptance -- --ignored --exact
```

The first command requires host Python 3 and OpenSSL (not supplied by the current
development shell). It creates a temporary, certificate-verified localhost HTTPS
smart-Git server and root sample pack, with an isolated Git configuration and
synthetic credential helper. It checks authenticated fetching, unchanged media,
exact commit revisions, prepared updates, pinned sessions, failed validation,
unavailable remotes and offline cached loading. Markers verify that pack scripts,
a configured smudge filter and Git hooks do not execute. The helper records only
operation names; the fixture never reads account credentials or changes user Git
configuration. Temporary certificates, helper and repository are removed on exit.

The second command exports an exact public Durop commit over ordinary GitHub
HTTPS and validates its nested sample pack. Durop's repository root is not itself
a pack; full HTTPS root-pack cache activation is covered by the localhost fixture.
These two acceptance tests are ignored by default to keep ordinary unit tests
independent of network access and host fixture tools.

On Apple Silicon macOS 27.0.1, the twelve regular catalog tests and both HTTPS
acceptance paths passed through `nix develop`. Authentication coverage exercises
Git's helper protocol with synthetic credentials, not a provider-specific private
repository or an OS keychain. Decoder probing, playback and host/session integration
remain deferred; offline acceptance verifies cached file access, not playback.
