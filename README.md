# Durop

Durop plays local media drops through a transparent macOS overlay, with a searchable
pack catalog and optional authenticated remote control over Tailscale.

The POC targets Apple Silicon macOS 14+. Normal desktop playback is supported;
native fullscreen overlays remain unsupported. See the [acceptance record](docs/acceptance.md)
for remaining audio, recording, input-focus and two-Mac qualification.

## Run

Install Nix with flakes enabled and select Xcode or Command Line Tools with
`xcode-select`. From this repository's root:

```sh
nix develop
cargo run -p durop --bin durop --locked
```

This opens the local catalog with bundled samples. Select **This Mac**, choose a
display, and click a clip; **STOP ALL** stops playback. `cargo run --locked` also
works from the workspace root. Without entering the shell first:

```sh
nix develop --command cargo run -p durop --bin durop --locked
```

For a standalone `.app`, follow [packaging](docs/packaging.md). For arguments,
pack sources, host/controller modes and pairing, see the [session guide](docs/session-runbook.md).

## Work

| Track | Open work |
| --- | --- |
| [Upcoming Events](https://github.com/ramate-io/durop/issues?q=is%3Aissue%20state%3Aopen%20label%3Aevent%20label%3Apriority%3Ahigh%2Cpriority%3Aurgent) | Dated demos and acceptance milestones. |
| [Release Candidates](https://github.com/ramate-io/durop/issues?q=is%3Aissue%20state%3Aopen%20label%3Arelease-candidate%20label%3Apriority%3Ahigh%2Cpriority%3Aurgent) | Usable versions delivering those events. |
| [Features & Bugs](https://github.com/ramate-io/durop/issues?q=is%3Aissue%20state%3Aopen%20label%3Afeature%2Cbug%2Cproposal%20label%3Apriority%3Ahigh%2Cpriority%3Aurgent) | High-priority implementation, fixes and research. |

The [planning standard](docs/planning.md) links features and bugs to release
candidates, and candidates to events. [Contributing](CONTRIBUTING.md) covers
workspace checks, symbol documentation and Conventional Commits.

## Organization

- [`src/`](src/): the `durop` executable, UI and application adapters.
- [`crates/`](crates/): protocol, catalog, host, media and transport libraries; see [architecture](docs/architecture.md).
- [`docs/`](docs/): operator guides and measured acceptance evidence.
- [`infra/`](infra/): optional Tailscale administration; see the [runbook](infra/cdktn/tailscale/README.md).

Durop is [MIT licensed](LICENSE).
