# Durop

Durop is an early Rust/Bevy application for controlling and playing media drops. This bootstrap opens one empty window; playback, networking, the panel, and overlay are future work.

## Development

Install Nix with flakes enabled. On macOS, install Xcode or the Command Line Tools and select them with `xcode-select`; the SDK and linker are needed for native builds. From a checkout:

```sh
nix develop
cargo run --locked
cargo fmt --all -- --check
cargo check --locked
cargo test --locked
cargo clippy --all-targets --locked -- -D warnings
```

Alternatively prefix each command with `nix develop --command`. Rust and Bevy are pinned in `rust-toolchain.toml` and `Cargo.lock`. The shell includes Git, jq and OpenTofu; Python, uv and Node infrastructure tooling is deferred to issue #7. Installing the Tailscale macOS application and its network extension is a separate step.

The development shell targets Apple Silicon macOS and x86_64 Linux CI. See [CONTRIBUTING.md](CONTRIBUTING.md) for validation status. Nix supplies development tools, not a distributable macOS application; packaging, signing and notarization are future work.
