# Tailscale infrastructure reference

Start with the role-based [Tailnet quickstart](../../../docs/tailnet-quickstart.md).
`infra/manage.py` is the single infrastructure CLI. Typed CDK Terrain constructs
manage the complete policy of an **existing** tailnet, plus MagicDNS only when
explicitly configured. OpenTofu deploys with persistent local state. Nothing here
creates a cloud playback server, enrolls a Mac, invites users or generates Durop
session pairing credentials.

## Toolchain and commands

From the checkout root:

```sh
nix develop .#infrastructure
python infra/manage.py setup durop-network
python -m unittest discover -s scripts/tests -v
uv --directory infra/cdktn/tailscale run --locked --no-sync python -m unittest discover -s tests -v
python infra/manage.py synth durop-network --no-env --config infra/cdktn/tailscale/network.config.example.json
python infra/manage.py validate durop-network --no-env --config infra/cdktn/tailscale/network.config.example.json
```

The manager is independent of the current directory. Defaults resolve against its
own checkout; explicitly supplied relative paths resolve against the caller's
working directory. Commands require the explicit `durop-network` stack. `setup`
syncs locked Python/npm dependencies and regenerates bindings; it may download
dependencies, but performs no Tailscale operations. Other commands enter the
locked uv runtime automatically unless already running in the project's virtual
environment. If that environment is missing, they run `setup` first. Runtime
re-entry uses `--no-sync`.

| Command | Effect |
| --- | --- |
| `setup` | Install locked dependencies and regenerate provider bindings. |
| `synth` | Generate deterministic OpenTofu JSON, with no API calls. |
| `validate` | Initialize locked provider/backend and validate schema; no deployment. |
| `import` | Adopt existing policy, and DNS only when configured; skip addresses already in state. |
| `plan` | Save a plan and fingerprints; invalidate any previous review first. |
| `show` | Verify fingerprints and display the exact saved plan. |
| `apply` | Consume the saved review and apply only that plan. |
| `status` | Show backend location, tracked addresses and saved-plan presence. |
| `ensure` | Fetch the live ACL, merge Durop grants when flags are passed, import, plan and show. |

`import`, `plan`, `apply` and `ensure` are live operator actions and require explicit OAuth
credentials and tailnet. No test or shell entry runs them against Tailscale.
`validate`, `show` and `status` initialize OpenTofu and may download the locked
provider; status inspects local ownership, not live connectivity or policy drift.
The wrapper has no destroy command.

The committed `flake.lock` pins Python 3.12.13, Node 24.15.0, uv 0.11.16 and
OpenTofu 1.11.8. Both Nix shells also include the open-source Tailscale CLI.
`uv.lock` pins CDK Terrain 0.25.0 and constructs 10.6.0; `package-lock.json`
pins `cdktn-cli` 0.25.0. The official provider is `tailscale/tailscale` 0.29.2,
with signed checksums for Linux x86_64 and Apple Silicon. No archived CDKTF
package is used; `cdktf.json` is Terrain's required filename. Regeneration uses
`TERRAFORM_BINARY_NAME=tofu`; review and commit changes to `imports/`, provider
lock and Python/npm locks together. Preserve `flake.lock` on toolchain changes.

CI runs setup, pure tests, deterministic synthesis and schema validation on Linux
and macOS, without deployment credentials or plan/apply. These checks do not
establish enrollment, policy deployment or two-Mac acceptance. Earlier tooling
qualification passed on both platforms; no live plan/apply was performed.

## Complete policy and optional DNS ownership

`ensure` downloads the live ACL as ordinary JSON (`Accept: application/json`),
writes a private `fetched-acl.json` next to state, merges Durop host/group/port
grants and tests, and saves ignored `network.config.json`. Pass `--host`,
`--controller` (repeatable) and `--unauthorized`. Synthesis still never fetches
or merges; only this operator command talks to the ACL API before generating
OpenTofu JSON. Unrelated grants, ACLs, SSH rules, tests, tags and posture rules
are preserved. A conflicting `durop-host` address is refused. An existing local
config that differs from the merge is refused unless `--replace-config` is set.
The example configuration is suitable only for a dedicated test tailnet;
replacing a real policy with it would remove unrelated access. Existing broad
grants remain effective. Skip `ensure`/`apply` when you do not want CDKTN to own
policy and TCP 48732 already works.

The dedicated sample includes both a same-tailnet controller group and a narrow
`autogroup:shared` → `durop-host` / `tcp:48732` grant. The latter covers users who
accepted a share of this device; the group alone does not admit shared guests.
It grants only the Durop port on the named host and does not share additional
devices. Review any broader rules in both tailnets and test with a real recipient.
See Tailscale's [sharing/access-control guidance](https://tailscale.com/docs/features/sharing).

The `tailscale_acl` resource owns the whole policy. Synthesis preserves every
supplied policy section; it never fetches or merges the live document. Local
validation checks shapes, identities, IP/CIDR addresses, selectors and ports;
Tailscale's policy engine performs semantic validation during live operations.

Leave `magic_dns` absent unless taking ownership of that complete preference.
Either `true` or `false` enables ownership and requires import of the existing
setting. Use the host's existing DNS name or Tailscale IP otherwise. Import uses
`tailscale_acl.policy` with provider ID `acl`, and, only when configured,
`tailscale_dns_preferences.dns` with ID `dns_preferences`; those IDs are arbitrary
per the pinned provider's import contract. Existing addresses are skipped;
failed state inspection aborts rather than assuming a resource is unmanaged.
`overwrite_existing_content=false` requires policy adoption before replacement;
`prevent_destroy=true` and `reset_acl_on_destroy=false` guard accidental removal.

## Credentials and dotenv

Create a dedicated administrator OAuth trust credential for the provisioner:
`TAILSCALE_OAUTH_CLIENT_ID`, `TAILSCALE_OAUTH_CLIENT_SECRET` and explicit
`TAILSCALE_TAILNET`. For modification use `policy_file`,
`devices:posture_attributes` and `devices:core:read`; read-only policy scopes are
`policy_file:read`, `devices:posture_attributes:read` and `devices:core:read`.
Add DNS permission only when owning DNS. Verify current dependencies in the
[trust credential reference](https://tailscale.com/docs/reference/trust-credentials#scopes).
This manager uses OAuth client credentials; it does not provision trust credentials
or depend on host API tokens. Do not use `TAILSCALE_BASE_URL` for ordinary deployments.

The root `.env` is loaded automatically as data; the invoking environment wins,
even when its value is empty. `--env-file /private/path.env` requires that file;
`--no-env` disables loading. The parser accepts optional `export`, blank/comment
lines and single-line quoted or unquoted `NAME=value` assignments. It performs
no command substitution, variable expansion, escape decoding or shell evaluation.
Malformed files fail without echoing their contents. Copy `.env.example` as a
starting point and keep secret files outside Git. Avoid secrets in arguments,
recorded terminal input or generated policy JSON.

The host's user-owned `TAILSCALE_API_KEY` is solely for its device-share invitation UI, supplied
through the host's environment or secure UI storage. The manager removes it
before invoking dependencies or the provider to avoid conflicting authentication.
It never substitutes that token for OAuth credentials. Device enrollment uses
Tailscale login; Durop pairing uses a separate runtime credential per named
controller. Disabling a source or the listener retains that token until revoked
or the host app exits. Neither is produced by infrastructure provisioning.

## Persistent state, review and apply

The default backend is `${XDG_STATE_HOME:-$HOME/.local/state}/durop/tailscale`.
Use `--state-dir /private/persistent/path` consistently and a separate directory
per tailnet. State and plans must live outside the checkout and generated output;
the manager rejects unsafe paths. It creates state directories with mode 700,
requires private existing directories and uses umask 077. The backend path is
absolute; deleting `cdktf.out` never deletes state or plans. Use durable encrypted
storage and independent private backups.

One nonblocking file lock serializes all backend operations, including imports
and reads, on one machine. Do not run raw OpenTofu concurrently. Multiple
operators need a separately implemented shared locking backend and migration;
do not copy/share this local backend between active operators.

Before a live change, retain `fetched-acl.json`, the authored configuration and
state, including OpenTofu `.backup` files. Then run the quickstart's
ensure/apply sequence (or import/plan/show/apply on an existing file).
Review the full diff, deletions, policy tests, resource addresses, target tailnet
and backend path. A final plan after apply should show no changes; verify intended
TCP 48732 access and rejection on real devices separately.

Apply accepts only the saved plan. Metadata binds its bytes to the manager and
lifecycle code, authored policy configuration, generated configuration, provider,
Python/npm/Nix locks, generated bindings, backend path and provider target.
Missing, changed or corrupted review inputs fail before OpenTofu runs. `show`
checks the same fingerprint. A new plan or import invalidates the old review;
an attempted apply consumes review metadata even if OpenTofu fails or is
interrupted. OpenTofu also rejects stale state. After a change/failure, create,
show and review a new plan. Never edit review files or bypass this with raw apply.

## Recovery and macOS diagnostics

To roll back policy, restore the backed-up **complete configuration**, retain
current state/ownership, then plan, show, review and apply. Restoring old state
does not restore remote policy. If access is lost, an administrator restores the
policy in the admin console, then reconciles configuration through a fresh plan.
If generated output is lost, regenerate against the same durable backend without
re-importing. If state is lost, stop deployment and restore its latest backup;
without a valid backup, export the live policy again and carefully adopt it into
a new backend. Never delete state to hide a diff. Retirement requires a separate
reviewed policy change; ending a recording never destroys infrastructure.

The Nix CLI does not install a macOS network extension or enroll the machine.
Tailscale's [GUI variants and open-source daemon](https://tailscale.com/docs/concepts/macos-variants)
are distinct installations. A Nix `tailscale` can look for the open-source
`tailscaled` socket rather than the running app's daemon. For a GUI enrollment,
use its bundled CLI explicitly rather than starting a competing daemon:

```sh
TAILSCALE_BE_CLI=1 /Applications/Tailscale.app/Contents/MacOS/Tailscale status
TAILSCALE_BE_CLI=1 /Applications/Tailscale.app/Contents/MacOS/Tailscale ip -4
TAILSCALE_BE_CLI=1 /Applications/Tailscale.app/Contents/MacOS/Tailscale ping HOST_TAILSCALE_IP
```

See the official [CLI reference](https://tailscale.com/docs/reference/tailscale-cli)
for installation-specific paths and socket options. Keep status output private.
A ping alone does not prove TCP reachability or Durop authentication. Use the
quickstart's **From**, **To** and **Catalog** flows for sessions; no infrastructure
apply is required per recording. Actual desktop, audio, recording and two-network
acceptance remain operator checks in the [session runbook](../../../docs/session-runbook.md).
