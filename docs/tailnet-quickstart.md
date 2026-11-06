# Tailnet setup and Durop pairing

There are two interfaces: the **Tailscale admin console** manages your private
network; the **Durop controller UI** selects and controls a playback Mac. Durop
has no hosted web portal. CDK Terrain (`cdktn`) configures an existing tailnet's
policy through OpenTofu; it does not create your Tailscale account, install the
Mac app, enroll devices or start Durop.

This is an operator guide, not a deployment record. No live enrollment, policy
import, plan or apply was performed while preparing it. Two-Mac acceptance
remains for the operator.

## 1. Create or select the tailnet in the admin console

Sign in to the [Tailscale admin console](https://login.tailscale.com/admin).
Create a tailnet through Tailscale's signup flow if you do not already have one;
otherwise select the intended existing tailnet. An administrator invites the
controller user if needed; ensure they join this tailnet rather than a separate
personal network. See [user invitations](https://tailscale.com/docs/features/sharing/how-to/invite-any-user).

On both Macs install the supported [Tailscale macOS app](https://tailscale.com/docs/install/mac),
open it, authorize the requested network/system integration and log into that
same tailnet. Keep these laptops persistently user-owned. In the admin console,
check **Machines** for both devices and approve them if device approval is enabled.
See [device enrollment and approval](https://tailscale.com/docs/features/access-control/device-management/how-to/set-up).

Record the playback Mac's Tailscale IPv4 address. Use the CLI already installed
with the app; if `tailscale` is on PATH, run:

```sh
tailscale ip -4
tailscale status
tailscale ping HOST_TAILSCALE_IP
```

Replace `HOST_TAILSCALE_IP` with the actual playback Mac address. If PATH has no
CLI, the standard standalone installation can be invoked without changing PATH:

```sh
TAILSCALE_BE_CLI=1 /Applications/Tailscale.app/Contents/MacOS/Tailscale ip -4
TAILSCALE_BE_CLI=1 /Applications/Tailscale.app/Contents/MacOS/Tailscale status
```

Durop also discovers standard macOS Tailscale installations when launched from
Finder. Keep device/status output private. Ping checks the peer, not Durop's TCP
port or application authentication.

## 2. Prepare and review policy with CDK Terrain

A tailnet administrator operates this step. If reviewed existing policy already
permits the intended controller to reach the host on TCP **48732**, normal
Durop sessions need no infrastructure apply.

For policy changes, first follow the [infrastructure runbook](../infra/cdktn/tailscale/README.md)
for full-policy backup, durable private state and rollback. In **Access controls**,
export the complete current policy and save its original HuJSON outside Git.
Convert a copy with a reviewed HuJSON parser to ordinary JSON. Create ignored
`infra/cdktn/tailscale/network.config.json` containing `{"policy": <complete JSON object>}`.
Retain unrelated policy sections. Add the intended user group, host address,
TCP 48732 grant and authorized/unauthorized tests from the example, replacing
all example identities and addresses. The example alone is only for a dedicated
test tailnet. Existing broader grants can still allow other users; a narrow
Durop grant does not override them. Leave `magic_dns` absent unless deliberately
managing that preference too.

From the repository root, enter the pinned shell and prepare the tools:

```sh
nix develop .#infrastructure
cd infra/cdktn/tailscale
uv sync --locked
npm ci --ignore-scripts
TERRAFORM_BINARY_NAME=tofu npm exec -- cdktn get
uv run --locked --no-sync python -m unittest discover -s tests -v
uv run --locked --no-sync python main.py synth --config network.config.example.json
uv run --locked --no-sync python main.py validate durop-network --config network.config.example.json
```

These example checks do not deploy. Setup/validation can download dependencies;
synthesis makes no Tailscale API calls. Now load the deployment credential and
explicit `TAILSCALE_TAILNET` through your secret manager, as described below.
The following commands use **private `network.config.json`**, not the example.
They are live operator actions. Use one operator and the same durable state path
throughout; this example uses the default path (adjust `show` if XDG_STATE_HOME
or `--state-dir` changes it):

```sh
uv run --locked --no-sync python main.py validate durop-network
# First adoption only: import existing policy into this backend.
tofu -chdir=cdktf.out/stacks/durop-network import tailscale_acl.policy acl
uv run --locked --no-sync python main.py plan durop-network
tofu -chdir=cdktf.out/stacks/durop-network show "$HOME/.local/state/durop/tailscale/reviewed.tfplan"
# Only after reviewing the entire policy diff, tests, target and state path:
uv run --locked --no-sync python main.py apply durop-network
uv run --locked --no-sync python main.py plan durop-network
```

Do not re-import already managed resources. MagicDNS ownership additionally needs
its separate import from the infrastructure runbook. Confirm the final plan has
no changes; verify authorized access and unauthorized rejection. Apply consumes
the reviewed saved plan; changes/failures require a new plan and review. Never
bypass that check with direct `tofu apply` or replace policy with the example.

## 3. Keep the three credential roles separate

| Credential | Used by | Purpose |
| --- | --- | --- |
| Tailscale OAuth trust credential / API access token | Infrastructure operator/provider | Manage tailnet policy through the API; never enter it into Durop. |
| Tailscale login or device enrollment auth key | Tailscale app | Join a device to the tailnet. Interactive login suffices for these Macs; CDK Terrain does not enroll them. |
| Durop pairing token / session secret | Durop **Host → Tailscale** form | Authenticate to this running Durop host only; generated by **Share this Mac…**. |

This repository documents OAuth using `TAILSCALE_OAUTH_CLIENT_ID`,
`TAILSCALE_OAUTH_CLIENT_SECRET` and `TAILSCALE_TAILNET`. Supply values through a
secret manager/environment, not source, command arguments or recorded terminal
input. For policy modification, current scopes are `policy_file`,
`devices:posture_attributes` and `devices:core:read`; read-only policy access uses
`policy_file:read`, `devices:posture_attributes:read` and `devices:core:read`.
Add DNS permissions only when managing DNS. Confirm the
[current trust-credential scopes](https://tailscale.com/docs/reference/trust-credentials)
before creating the credential. A general API token is not a Durop pairing token.

## 4. Pair through Durop's native UI

Install/run the same Durop app on both Macs. Start with an ordinary desktop;
native fullscreen overlays remain unsupported. On the **playback Mac**:

1. Select the desired catalog. The bundled **Durop reactions** needs no Git/network.
2. Choose **Host → Native**, choose its display, then **Apply**. This enables local
   playback authority. Controller-only launches cannot play/share locally.
3. Reopen **Host → Native**, select **Share this Mac…**, enter this Mac's explicit
   Tailscale address and port, such as `100.x.y.z:48732`, then **Apply**.
4. Confirm sharing is enabled. Give the displayed session pairing secret privately
   to the intended controller. Keep this modal out of recordings/screenshots.

Durop verifies that the bind address belongs to the running local Tailscale node;
it does not fall back to a LAN, wildcard or public listener. No router port
forwarding, Tailscale Serve or Funnel is needed.

On the **controller Mac**, choose **Host → Tailscale**, enter the host's address
and its **Durop pairing token**, then **Apply**. The masked token is ephemeral,
not saved in preferences. Catalog/icons and playback state now come from the
host; media stays on that Mac. The local catalog picker does not configure the
remote host. Use the host's UI to change its catalog, then pair again if revoked.

Try one clip, per-instance Stop and **STOP ALL**, then revoke remote access in
the host's Native settings and confirm the controller can no longer command it.
Re-enable sharing for a fresh credential and pair explicitly. Disconnected clicks
are not queued for replay. A successful ping alone does not establish this flow;
check the host listener, TCP policy and application token separately if pairing
fails. Do not weaken binding/authentication to diagnose it.

When finished, stop playback, revoke remote access, disconnect the controller
and quit Durop. Tailscale device enrollment persists; no policy teardown/apply
is needed for each recording. Use the [session runbook](session-runbook.md) for
recording, reconnect and two-Mac acceptance, and the infrastructure runbook for
policy rollback/recovery.
