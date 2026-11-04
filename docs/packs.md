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
repository/ref. No panel or transport integration is included yet. `refresh`
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
