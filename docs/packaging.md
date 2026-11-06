# Native Apple Silicon package (#9)

The package is one `Durop.app` containing local playback, host and controller
modes. The distributable is built with native Rust and the selected Apple SDK,
independently of Nix. AVFoundation and the other linked Apple frameworks are
system components; no Swift probe, separate host executable, media plugin
installation or developer toolchain is required for bundled/local playback.

The POC declares **macOS 14.0 and Apple Silicon** as its conservative supported
floor, matching the oldest native CI runner. Both the compiler deployment target
and Info.plist use 14.0; the validator checks every Mach-O minimum and requires
the executable's target to match. Individual AV APIs being older does not prove
compatibility of the entire Bevy/winit/native dependency closure. Actual device
runtime evidence is currently macOS 27.0.1 on an M4 Pro; macOS 14 and a second
Mac remain user acceptance checks.

## Build and inspect

From a normal macOS terminal with Xcode/Command Line Tools selected and the
`rust-toolchain.toml` Rust channel installed:

```sh
packaging/macos/package.sh --output dist/local-proof
python3 packaging/macos/validate.py dist/local-proof/Durop.app
python3 -m unittest discover -s packaging/macos -p 'test_*.py' -v
```

Use a fresh output directory; the script refuses to replace an existing package.
`--cargo /absolute/path/to/native/cargo` selects a task-local native installation
without changing global Rust configuration. Nix/Homebrew Cargo and Nix shell
entry are rejected for this distribution path. The script starts Cargo with a
small clean environment, the SDK resolved by `xcrun`, a dedicated
`target/native-package` build directory and a locked native arm64 release build.
Cargo's existing native build cache is reused, while every package invocation
still runs the locked build and mandatory bundle/resource checks.

If a native rustup launcher was installed through Homebrew, select its actual
official toolchain executable with
`--cargo "$(rustup which --toolchain 1.95.0 cargo)"`. The packaging CI does this
explicitly; the launcher/installer's location must not be mistaken for the
compiler/Cargo used to build the distributable.

The output directory contains the app, a versioned ZIP, its SHA-256 checksum,
`BUILD-INFO.json` and `VALIDATION.json`. The app contains:

```text
Contents/
  Info.plist
  MacOS/durop
  Resources/
    Durop.icns
    assets/
    sample-packs/reactions/
    sample-packs/podcast-pack/
    sample-packs/media-proof/
    notices/
    BUILD-INFO.json
```

The icon repackages Durop's independently authored overlay PNG. Version comes
from Cargo.toml; the build record stores the exact source revision, compiler
channel, SDK and minimum OS. Identity is `io.ramate.durop`. Build artifacts must
not contain secrets or private configuration.

The default reactions catalog is copied with its provenance and license records.
The bundled Expletus Sans and IBM Plex Mono fonts retain their OFL texts under
`assets/fonts/`; media and fonts do not inherit the application's MIT license.

The recursive validator enumerates every Mach-O, including unreferenced plugins
and nested libraries. It checks arm64 architecture, deployment versions,
`LC_RPATH`, install names, loader environment overrides and dependency resolution.
Only Apple system paths or resolved paths within the app are permitted;
Nix/Homebrew, `/usr/local`, user/developer and temporary external paths fail.
Apple shared-cache libraries need not be present as ordinary filesystem files.
Regression tests build native test dylibs and prove that missing dependencies
and a hidden plugin with a forbidden install name are rejected.

## Signing, launch and resources

POC outputs are **ad-hoc signed, not Developer ID signed or notarized**. The
script applies the ad-hoc signature, verifies it, then validates resources from
`/private/tmp` with Cargo/Bevy asset variables absent and an empty temporary HOME.
`--self-check` validates bundled catalog/icons/assets without GUI, networking,
audio, cache writes or Git. This establishes location-independent resource
lookup, not Finder/GPU playback acceptance. It uses `Contents/Resources`, not the
working directory or source checkout.

Copy `Durop.app` into Applications and launch it normally. Gatekeeper may require
the operator's normal Open confirmation for an unnotarized downloaded app; do
not disable Gatekeeper globally. No paid signing account is a POC prerequisite.
Screen recording/system-audio capture permissions belong to the chosen capture
application. Tailscale's network extension enrollment is a separate installation
and approval, not bundled setup performed by Durop.

Caches live in `~/Library/Caches/io.ramate.durop/packs`; nonsecret preferences
live in `~/Library/Application Support/io.ramate.durop/preferences.json`.
Neither belongs inside the app. Updating/replacing the bundle preserves those
user directories. Pack selection/query/display may persist; pairing secrets,
remote destination and hosting intent do not. Retaining caches is necessary
while sessions pin snapshots; this POC has no automatic garbage collection.

## External tools and notices

Local directories, bundled samples and already cached media need no Git.
**Git refresh requires usable external Git on PATH** and optional trusted Git
credential helpers. Apple `/usr/bin/git` can be an installation stub on a clean
Mac; the catalog checks the selected tool without invoking that stub or prompting
to install developer tools. Missing Git is actionable, not an automatic CLT
requirement. This app does not bundle Apple/Nix/Homebrew Git. Tokens must never
be embedded in URLs or manifests. Remote control additionally requires the
separately configured Tailscale service. CLI lookup respects PATH and, on macOS,
falls back to the installed `/Applications/Tailscale.app/Contents/MacOS/Tailscale`
and `/usr/local/bin/tailscale`. `TAILSCALE_BE_CLI=1` forces command-line mode,
preventing Finder's environment from opening the Tailscale GUI accidentally.
These read-only status checks do not enroll machines or change policy. The
resolution follows [Tailscale's official CLI documentation](https://tailscale.com/docs/reference/tailscale-cli?tab=macos).

`Resources/notices` includes the project license, exact locked native-target
dependency metadata, distributed license/notice files, package author attributions,
labelled canonical SPDX texts for crates omitting shared license files, and the
upstream objc2 Apple-SDK licensing explanation, and the native Rust standard
library's copyright/license notices. License-text provenance and
SHA-256 values are checked in under `packaging/licenses`. Sample packs retain
their own asset provenance files. Do not silently add a non-system media/Git
runtime: its recursive closure, redistribution notices and source obligations
must be handled before distribution.

The workflow `.github/workflows/package.yml` builds native macOS artifacts for
pull requests and manual dispatch; it uploads the ZIP/checksum/build/validation
records. It does not create a release or publish artifacts outside CI. Release
publication, Developer ID signing and notarization require a separate explicit
operator action. The high-level development/native-build separation was reviewed
from Maybraid's packaging README; no Maybraid source or documentation was copied.

See [the session runbook](session-runbook.md) for local/host/controller operation
and outstanding second-Mac acceptance.

## Native validation record

On November 4, 2026, the native path built the assembled app on an M4 Pro Mac
mini running macOS 27.0.1, using official Rust 1.95.0 and Apple SDK 15.5, outside
Nix. The first complete bundle passed ad-hoc signing/strict verification,
recursive arm64/Mach-O validation (21 Apple/system dependencies, zero external
non-system dependencies), and the clean-environment `--self-check` from `/tmp`,
which reported the bundled Resources directory and four sample clips. The ZIP
was approximately 21 MiB. Dependency notices covered 366 locked native-target
packages plus the Rust standard library. Three native validator regressions and
the runtime Git/Tailscale discovery tests passed.

This records build, loader, signature and resource evidence. It does not claim
Finder/GPU/audio playback, the oldest declared OS, a development-free second Mac,
Gatekeeper acceptance, or live Tailscale connectivity. Final artifact hashes and
source revision are supplied in each output's BUILD-INFO/VALIDATION/checksum files.

CI uses the supported `macos-15` arm64 runner; GitHub [retired macOS 14 runners](https://github.blog/changelog/2026-10-01-github-actions-macos-14-runner-image-retirement/) on November 2, 2026. The app deployment floor remains macOS 14, validated from Mach-O load commands. Building on macOS 15 does not substitute for runtime acceptance on macOS 14.
