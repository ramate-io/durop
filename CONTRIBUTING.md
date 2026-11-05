# Contributing

Use the commands in the README before submitting changes. CI runs formatting, locked checking, tests and a native build on macOS and Linux. Keep Cargo.lock and flake.lock committed when dependencies change. Shell entry does not install hooks or change Git configuration; there is no required hook setup.

The first runtime target is Apple Silicon macOS. Linux CI checks portable code and does not validate macOS window behavior. This initial window uses Bevy's windowing features only; rendering, audio, video, networking and infrastructure dependencies should be added when their work requires them. No media backend has been selected.

The shell includes the X11 libraries needed by the chosen Bevy window backend on Linux. Apple builds need the platform SDK and developer tools as described in the README. Do not add machine-specific SDK or Homebrew paths.

Bootstrap validation: on Apple Silicon macOS 27.0.1 with Rust 1.95.0, the Nix shell passed formatting, locked check, test, clippy and build. The [merged bootstrap CI run](https://github.com/ramate-io/durop/actions/runs/41737039260) passed formatting, locked check, tests and native builds on ubuntu-24.04 and macos-14. CI does not launch or validate macOS window behavior.

## Reference provenance

The pinned nixpkgs and rust-overlay revisions and the idea of a Rust toolchain file were informed by Maybraid at commit `0d25b9777709253f572d22f74e227c630abf7353`. Maybraid's `LICENSE.md` is a source-available license, not an MIT license. No Maybraid source or documentation was copied; this implementation is independently written and Durop retains its MIT license. The shell deliberately has no automatic Git setup, game tooling, or deployment target overrides.

## Infrastructure checks

Use `nix develop .#infrastructure` for the smaller infrastructure-only shell, then follow the dependency sync, binding generation, pure tests, synthesis and validation commands in [the Tailscale runbook](infra/cdktn/tailscale/README.md). The default shell has the same locked tools. Commit `uv.lock`, `package-lock.json`, generated `imports/` and `.terraform.lock.hcl` when upgrading infrastructure dependencies; the existing `flake.lock` fixes Python, uv, Node and OpenTofu. Infrastructure CI runs on both supported platforms with no deployment credentials and no plan/apply. Configuration, state, plans and credentials are private operator data and must stay outside Git. Live policy import, apply and two-Mac network acceptance require a separately reviewed operator action.
