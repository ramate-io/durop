# Contributing

Use the [README run commands](README.md#run) to launch the app. From the repository root inside `nix develop`, validate every workspace member:

```sh
cargo fmt --all -- --check
cargo check --workspace --locked
cargo test --workspace --locked
cargo clippy --workspace --all-targets --locked -- -D warnings
RUSTDOCFLAGS='-D warnings' cargo doc --workspace --no-deps --locked
cargo build --workspace --locked
python3 -m unittest discover -s scripts/tests -v
```

CI runs the Rust checks on macOS and Linux. Keep Cargo.lock and flake.lock committed when dependencies change. Shell entry does not install hooks or change Git configuration; there is no required hook setup.

The first runtime target is Apple Silicon macOS. One executable assembles separate protocol, catalog, host, media and transport crates. Follow the [dependency boundaries](docs/architecture.md); application/UI dependencies stay out of those libraries. Linux CI checks portable code and does not validate native AppKit/AVFoundation behavior. Audio/video controls and system codecs are qualified on macOS; GIF and state/transport logic have portable tests.

## Planning and Rust style

Follow the [events → release candidates → features/bugs standard](docs/planning.md), using the issue templates and one priority label per issue. Research uses `proposal`; link its result to implementation work when ready.

Document public types, fields, variants, traits and methods with rustdoc that explains behavior, ownership, failure and cancellation contracts where relevant. Give private symbols a short purpose or invariant comment; preserve safety explanations at native boundaries. Leave a blank line between declarations, functions and implementation blocks. Do not replace useful explanations with comments that merely repeat a symbol's name.

Prefer methods and constructors on the type that owns the behavior. Free functions remain appropriate for Bevy systems, foreign callbacks and small utilities with no natural owner. Introduce traits at real interchangeable boundaries, use them from production call sites, and test alternate implementations against the same contract. Use typestate to restrict meaningful lifecycle operations to valid stages, keeping runtime checks for asynchronous or external failure. Avoid types or traits that only rename one helper.

The shell includes the X11 libraries needed by the chosen Bevy window backend on Linux. Apple builds need the platform SDK and developer tools as described in the README. Do not add machine-specific SDK or Homebrew paths.

Initial bootstrap validation: on Apple Silicon macOS 27.0.1 with Rust 1.95.0, the Nix shell passed formatting, locked check, test, clippy and build. The [merged bootstrap CI run](https://github.com/ramate-io/durop/actions/runs/41737039260) passed formatting, locked check, tests and native builds on ubuntu-24.04 and macos-14. CI does not launch or validate macOS window behavior.

For the current application, run the README checks and the focused native probes described in `docs/media-backend.md`. Native distribution uses the independent Cargo/Apple-SDK path in `docs/packaging.md`, with an explicit macOS 14 deployment floor, ad-hoc signing, recursive Mach-O validation and a read-only arbitrary-directory resource check. Native packaging CI artifacts do not establish Finder, GPU, audible output, recorded A/V synchronization or two-Mac acceptance. Keep measured native evidence and remaining physical-device gaps explicit; the user handles second-Mac qualification.

## Commit and pull request titles

Every new non-merge commit subject and every PR title must follow
[Conventional Commits 1.0.0](https://www.conventionalcommits.org/en/v1.0.0/):

```text
type(scope)!: description
```

The scope and breaking-change `!` are optional. Durop supports `feat`, `fix`,
`build`, `chore`, `ci`, `docs`, `style`, `refactor`, `perf`, `test`, and `revert`.
Types are matched without regard to case; use lowercase consistently. A scope
names an area such as `panel`, `transport`, or `packaging`; it starts with a letter
or digit and may contain letters, digits, `_`, `.`, `/`, or `-`. Use one printable
line of at most 200 characters, exactly one space after the colon, a nonempty
description, and no leading or trailing whitespace. Examples:

```text
feat(panel): add catalog search
fix(transport): reject stale session credentials
ci: validate pull request titles
docs(packaging): clarify native setup
feat(protocol)!: remove the legacy command envelope
```

Bodies and footers may explain the change; document breaking behavior in the body
and, where appropriate, a `BREAKING CHANGE:` footer. The header check does not
validate bodies, footers, or whether the chosen type matches the implementation.

The **Conventional Commit PR title** CI job validates the current PR title when
the PR is opened, edited, synchronized, or reopened. Titles are suitable squash
commit subjects: if using squash merge, preserve a valid Conventional Commit
subject when completing the merge. This policy does not change enabled merge
methods. Autogenerated merge commit subjects are exempt.

The existing nonconventional history is grandfathered. Do not rewrite it to
satisfy this policy. CI checks PR titles only; it does not scan historical or
individual branch commits, so authors and reviewers remain responsible for new
non-merge commit subjects. No Git configuration or hooks are installed.

Validate a proposed commit subject or PR title locally with Python 3:

```sh
python3 scripts/conventional_commits.py --header 'fix(panel): retain keyboard focus'
python3 -m unittest discover -s scripts/tests -p 'test_conventional_commits.py' -v
```

## Reference provenance

The pinned nixpkgs and rust-overlay revisions, Rust toolchain file, crate boundaries and issue-planning conventions were informed by Maybraid at commit `0d25b9777709253f572d22f74e227c630abf7353`. Maybraid's `LICENSE.md` is a source-available license, not an MIT license. No Maybraid source or documentation was copied; this implementation is independently written and Durop retains its MIT license. The shell deliberately has no automatic Git setup, game tooling, or deployment target overrides.

## Infrastructure checks

Use `nix develop` for Rust and infrastructure tools, then follow the uv sync, binding generation, pure tests, synthesis and validation commands in [the Tailscale runbook](infra/cdktn/tailscale/README.md). Commit `uv.lock`, `package-lock.json`, generated `imports/` and `.terraform.lock.hcl` when upgrading infrastructure dependencies; the existing `flake.lock` fixes Python, uv, Node and OpenTofu. Infrastructure CI runs on both supported platforms with no deployment credentials or plan/apply. Configuration, state, plans and credentials are private operator data and must stay outside Git. Live policy import, apply and two-Mac network acceptance require a separately reviewed operator action.
