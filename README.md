# Durop

Durop is a Rust/Bevy proof of concept for local media drops and authenticated
remote control. One application contains the host, controller, catalog panel and
transparent overlay. Audio/video use AVFoundation on macOS; transparent GIFs use
a bounded composited timeline. Packs provide author-supplied PNG icons and media.

The first runtime target is Apple Silicon macOS. Normal-desktop compositing and
native decoding have measured evidence; native fullscreen overlay presentation
was absent in the tested TextEdit Space and remains unsupported there. Audible
output, recorded A/V synchronization, games/controllers/mixed displays and
installed two-Mac operation still require device acceptance. See the
[media decision](docs/media-backend.md) and [overlay evidence](docs/overlay-proof.md).

## Use the app

The [native package guide](docs/packaging.md) builds one ad-hoc-signed `.app` and
ZIP with bundled samples, notices and recursive loader validation. Its declared
POC minimum is macOS 14.0 on Apple Silicon; actual runtime evidence currently
comes from macOS 27.0.1. A developer toolchain is not required for bundled/local
playback. Git pack refresh needs usable external Git; remote control needs
separately installed/configured Tailscale. No release has been published by the
packaging workflow.

Launch the app, select **This Mac**, choose a display and click a clip. **STOP
ALL** is the safety control. Use **Remote host** plus an explicit tailnet address
and a private session credential for another Mac. Read the
[session runbook](docs/session-runbook.md) for local/host/controller modes,
pack configuration, recording with system audio, shutdown and troubleshooting.
The user performs the second-Mac acceptance.

## Development

Install Nix with flakes enabled. On macOS, select Xcode or Command Line Tools
with `xcode-select`; the Apple SDK/linker is required for native builds.
From a checkout:

```sh
nix develop
cargo run --locked
cargo fmt --all -- --check
cargo check --locked
cargo test --locked
cargo clippy --all-targets --locked -- -D warnings
cargo build --locked
```

Rust, Bevy and dependencies are pinned through `rust-toolchain.toml`, Cargo.lock
and flake.lock. Use `nix develop --command COMMAND` for individual checks. Nix
supplies developer/admin tools, not the distributable runtime. Native packaging
uses a separate clean Cargo/Apple-SDK path. No hooks or Git configuration are
installed automatically. See [CONTRIBUTING.md](CONTRIBUTING.md).

The default and smaller `nix develop .#infrastructure` shells include the locked
Python/uv/Node/OpenTofu tooling used by
[the Tailscale infrastructure runbook](infra/cdktn/tailscale/README.md). Those
admin tools are not bundled in Durop. No live tailnet policy is changed by
application builds or CI.

- [Versioned packs, cache and credentials](docs/packs.md)
- [Host state and command policy](docs/host.md)
- [Transport, pairing and network diagnostics](docs/transport.md)
- [Native media fixtures and reproduction](fixtures/media/README.md)
