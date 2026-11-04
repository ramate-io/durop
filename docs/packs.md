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
