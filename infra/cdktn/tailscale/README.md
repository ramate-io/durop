# Tailscale infrastructure

Python CDK Terrain is the authoritative deployment source. It manages the complete policy of an **existing** tailnet, plus MagicDNS only when `magic_dns` is explicitly configured. OpenTofu owns state and deployment. No cloud playback server is created.

## Toolchain and offline checks

From the repository root, `nix develop .#infrastructure` enters a smaller shell; the default shell includes the same tools. The existing `flake.lock` pins Python **3.12.13**, Node **24.15.0**, uv **0.11.16**, and OpenTofu **1.11.8**. `uv.lock` pins CDK Terrain **0.25.0**, constructs **10.6.0**, and transitive Python dependencies. `package-lock.json` pins genuine `cdktn-cli` **0.25.0** and its dependencies; its Node requirement is >=22.23.4. The official provider is **tailscale/tailscale 0.29.2**, with signed checksums for Linux x86_64 and Apple Silicon. No archived CDKTF package is used.

```sh
cd infra/cdktn/tailscale
uv sync --locked
npm ci --ignore-scripts
TERRAFORM_BINARY_NAME=tofu npm exec -- cdktn get
uv run --locked --no-sync python -m unittest discover -s tests -v
uv run --locked --no-sync python main.py synth --config network.config.example.json
uv run --locked --no-sync python main.py validate durop-network --config network.config.example.json
```

Bindings in `imports/` were generated with these tools; regenerate on provider/tool upgrades, review the diff, and commit them. Generation uses OpenTofu's registry and executable (`TERRAFORM_BINARY_NAME=tofu`), not Terraform. Synthesis after setup is deterministic and offline, requires no credentials and makes no Tailscale API calls. Validation initializes the locked provider (a download on a cold cache), then validates its schema without provider authentication. The CLI defaults to synthesis. `cdktf.json` is the filename required by Terrain.

Linux qualification passed dependency sync, binding generation, eleven pure tests, synthesis, and generated OpenTofu validation. Native Apple Silicon qualification on macOS 27.0.1 passed the same documented sequence on 2026-11-04 with the versions above, without deployment credentials. Binding regeneration left the committed bindings and provider lockfile unchanged; the tests verified deterministic synthesis and preservation of the complete supplied policy. [Infrastructure CI](https://github.com/ramate-io/durop/actions/runs/41780246089) also passed on macos-14 and ubuntu-24.04 at `b13f670`. No live policy read, import, plan, apply or enrollment was performed; live acceptance remains pending.

The qualification Mac had neither the Tailscale CLI nor the app in `/Applications`, and its process environment had no deployment credentials or explicit tailnet target; no private network configuration was present. These checks establish tooling readiness only. Deployment still requires an administrator's complete policy export/backup, private configuration and scoped credentials, durable state, and enrollment of both Macs before the reviewed live steps and two-network TCP 48732 acceptance below.

OpenTofu 1.11.8 does not expose all newer schema features to Terrain; this provider's policy/DNS resources use none of the omitted ephemeral/write-only/identity features. In the managed Linux qualification environment only, jsii's cache trust check required `JSII_RUNTIME_PACKAGE_CACHE=disabled` because `/home` belonged to another UID; the native Mac qualification and CI did not need that workaround.

## Prepare the whole policy before deployment

`network.config.example.json` is for a dedicated test tailnet only. **Never use it to replace an existing tailnet policy.** The `tailscale_acl` resource owns the entire policy. This code preserves every supplied policy section, including unrelated grants, legacy ACLs, SSH, tests, tags and posture rules. It does not fetch or merge the live policy during synthesis.

Before any import or apply, an administrator must export the complete current policy from the Tailscale admin console's Access controls editor and save the original HuJSON in a private backup outside Git. Save a normalized JSON copy as well (using the editor's JSON export/formatting capability or a reviewed HuJSON parser; `json.loads` cannot parse comments). Create ignored `network.config.json` containing `{"policy": <the complete exported JSON object>}`. Retain all existing sections. Add only reviewed durop groups/host/grants and policy tests; replace example email identities and address with enrolled devices. Keep personal laptops owned by their users. A narrow TCP 48732 grant does not negate any existing broader allow.

The sample has authorized-controller accept and unauthorized-identity deny tests. Replace both identities with real test users and retain those tests in the full policy. Review other rules that could permit the unauthorized identity. The local validator checks settings shapes, group identities, host IP/CIDR addresses, grant selectors and port ranges; the Tailscale policy engine remains responsible for semantic policy validation during plan/apply.

Leave `magic_dns` absent unless this stack has explicitly taken ownership. Setting it to `true` or `false` owns that whole preference and requires import/review of the existing setting too. Otherwise use an existing DNS name or the device's Tailscale IP.

## Credentials and persistent state

An administrator creates a dedicated OAuth trust credential separately. For policy reads/validation use `policy_file:read`; for modification use `policy_file`. Current Tailscale scope dependencies also require `devices:posture_attributes` and `devices:core:read`. Add `dns:read` for DNS reads or `dns` for DNS modifications only when MagicDNS is owned. Confirm the current scope dependencies in [Tailscale's trust credential reference](https://tailscale.com/kb/1623/trust-credentials#scopes). Device enrollment/auth-key administration is a separate credential. Federated identity is supported through the provider's documented environment variables when configured separately; PR CI does not receive deployment credentials or OIDC permissions.

Supply `TAILSCALE_OAUTH_CLIENT_ID`, `TAILSCALE_OAUTH_CLIENT_SECRET`, and explicit `TAILSCALE_TAILNET` using your secret manager/environment. Do not put their values in source, command history, generated JSON, or durop session tokens. Plan and apply require an explicit tailnet target. Do not use `TAILSCALE_BASE_URL` for normal deployments.

The default state directory is `${XDG_STATE_HOME:-$HOME/.local/state}/durop/tailscale`. Use a different private, persistent absolute `--state-dir` for each tailnet. The wrapper creates it with mode 700 and uses umask 077 for files. State, backups, the saved plan and metadata remain there when `cdktf.out` is removed. The backend stores an absolute state path. Put this directory on durable encrypted storage with independent backups. Do not put it inside the checkout or generated output; the wrapper rejects those paths. Local file locks protect one operator on one machine. Multiple operators must first implement and migrate to a shared locking backend; do not share/copy this local backend between active operators. Keep the same state path when changing checkouts.

## Import, review, and apply

The following commands are **explicit live operator actions**, not part of tests, shell entry, or session startup. Set up the private config/credentials and choose the persistent state directory first. Take exclusive operational ownership; do not run raw OpenTofu commands concurrently with the wrapper.

```sh
# Initialize and validate the synthesized stack with its durable local backend.
uv run --locked --no-sync python main.py validate durop-network
# Import records existing ownership in state; it does not authorize policy replacement.
tofu -chdir=cdktf.out/stacks/durop-network import tailscale_acl.policy acl
# Only when magic_dns is explicitly present/owned:
# tofu -chdir=cdktf.out/stacks/durop-network import tailscale_dns_preferences.dns dns_preferences
uv run --locked --no-sync python main.py plan durop-network
# Inspect the exact saved plan; it may contain private policy data.
tofu -chdir=cdktf.out/stacks/durop-network show "$HOME/.local/state/durop/tailscale/reviewed.tfplan"
# Only after whole-tailnet review:
uv run --locked --no-sync python main.py apply durop-network
uv run --locked --no-sync python main.py plan durop-network
```

If XDG_STATE_HOME or `--state-dir` is used, adjust the `show` path accordingly and use the same CLI state/config/output options for each invocation. Both imported resource IDs above are arbitrary per the pinned provider's documentation. `overwrite_existing_content=false` requires importing an existing policy before managing it. `prevent_destroy=true` and `reset_acl_on_destroy=false` protect against accidental resource removal/reset; the wrapper has no destroy command.

Review the complete policy diff, deletions, tests, resource addresses, tailnet and state path. The final plan after apply should report no changes. No plan/apply was run during development of this change, so a clean second live plan is an operator acceptance step. Verify authorized TCP 48732 reachability and unauthorized rejection on the actual tailnet.

Apply uses only the saved plan. Metadata hashes source and generated configuration, complete private config, provider/Python/npm locks, generated bindings, backend path and provider target environment, plus the plan bytes. Missing or changed inputs cause refusal before OpenTofu runs. A failed plan invalidates the earlier plan; an attempted apply consumes its review metadata even on failure. OpenTofu additionally rejects plans whose state changed. Run and review a new plan after any change/failure. Never edit the saved plan or metadata, or run a fresh direct `tofu apply` to bypass this check.

## Rollback and recovery

Before each live change, privately back up the current exported policy and persistent state (including OpenTofu's `.backup` files) with a timestamp. Restrict backup permissions like state. To reverse policy changes, restore the backed-up **complete policy** into private configuration, retaining resource ownership and the current state, then synthesize, plan, review and apply a new saved plan. Do not restore old state as a policy rollback: state does not restore remote resources. If access is lost, a tailnet administrator uses the admin console to restore the policy backup; then refresh through a new plan and reconcile configuration before applying again.

If generated output is lost, regenerate and initialize against the same durable backend; do not re-import resources already present in state. If state is lost, stop deployment, restore the latest state backup, and reconcile with a plan. If no valid state backup exists, export the live complete policy again and carefully import into a new backend before proposing changes. Never delete state to make a diff disappear. Retiring infrastructure is a separate reviewed whole-tailnet change, requiring explicit removal of lifecycle protection and a separately reviewed destruction plan; ending a recording never destroys policy or logs devices out.

## Laptop enrollment and sessions

Install the supported Tailscale macOS app and authorize its privileged network integration separately on both Macs. Each intended user logs into the existing tailnet interactively; use persistent user-owned enrollment. The Nix tools do not install the network extension or enroll laptops. Diagnose from each enrolled Mac:

```sh
tailscale up
tailscale ip -4
tailscale status --json
tailscale ping recording-mac
tailscale netcheck
```

Treat status output as private. Confirm both networks and the host TCP port, then start durop's local/host/controller modes when implemented. Each recording uses existing infrastructure; it needs no apply. Ending a durop session revokes application access and closes its listener, while Tailscale enrollment persists. Ephemeral CI node enrollment is out of scope and requires its own narrowly scoped credentials and cleanup.
