# Tailnet quickstart

Durop runs on your Macs. A tailnet administrator prepares network access once;
the host plays media and invites controllers; controllers send commands. Normal
sessions need no infrastructure apply. The [infrastructure runbook](../infra/cdktn/tailscale/README.md)
covers complete-policy ownership, credentials, durable state and recovery.

## Administrator: prepare the existing tailnet

1. Select your existing tailnet in the [Tailscale admin console](https://login.tailscale.com/admin).
   Install and sign into the [Tailscale macOS app](https://tailscale.com/docs/install/mac)
   on each Mac; approve devices when required. Keep laptops owned by their users.
2. Ensure policy permits intended controllers to reach the playback Mac on TCP
   **48732**, with authorized and unauthorized policy tests. Existing broader
   grants still apply. If policy already allows this, skip provisioning.
3. For policy changes, export and privately back up the **complete** current
   policy first. Create `infra/cdktn/tailscale/network.config.json` containing
   `{"policy": <complete ordinary JSON policy object>}`; preserve unrelated rules.
   The example config is only for a dedicated test tailnet. Keep `magic_dns`
   absent unless deliberately taking ownership of that preference.
4. Copy `.env.example` to ignored `.env` at the checkout root. Set the administrator's
   `TAILSCALE_OAUTH_CLIENT_ID`, `TAILSCALE_OAUTH_CLIENT_SECRET` and explicit
   `TAILSCALE_TAILNET`. Keep these OAuth credentials on the administrator's machine.

From the checkout root:

```sh
nix develop .#infrastructure
python infra/manage.py setup durop-network
python infra/manage.py validate durop-network
# First adoption only: records the existing policy and configured DNS in state.
python infra/manage.py import durop-network
python infra/manage.py plan durop-network
python infra/manage.py show durop-network
# After reviewing the complete policy diff, tests, tailnet and backend:
python infra/manage.py apply durop-network
python infra/manage.py plan durop-network
```

The final plan should report no changes. State and saved plans use a private,
persistent directory outside the checkout. Use the same `--state-dir` for every
command and a separate directory per tailnet. The manager automatically reads
root `.env` as literal data; existing environment variables win. Use
`--env-file /private/path.env` or `--no-env` to override that behavior. Provisioning
does not enroll devices or generate Durop session pairing credentials.

## Host: choose controls, display and catalog

Start Durop on the playback Mac:

```sh
nix develop --command cargo run -p durop --bin durop --locked
```

1. In **To**, choose the local display. In **Catalog**, choose the clips to play.
2. In **From**, enable local controls and/or remote controllers. Both can remain
   enabled together. Enable remote access using this Mac's Tailscale address and
   TCP 48732; Durop verifies the address belongs to the local enrolled node.
3. Use **From → Remote controllers** to invite an intended user. The host needs
   its own user-owned `TAILSCALE_API_KEY`, supplied in the host process environment
   or through the UI's secure credential storage. A repository `.env` is loaded
   by the infrastructure manager only; it does not export variables into Durop.
   An invitation joins the network; share the separately generated Durop session
   pairing credential privately with that controller.

Keep invitation and pairing credentials out of recordings. The administrator's
OAuth policy credential, the host's invitation API token, Tailscale device login,
and Durop session pairing are separate permissions. The infrastructure manager
excludes `TAILSCALE_API_KEY` from its provider environment.

## Controller: connect to the host

Accept the host's invitation when needed, install Tailscale and sign into the
same tailnet. Run Durop in controller mode:

```sh
nix develop --command cargo run -p durop --bin durop --locked -- --mode controller
```

In **To**, choose the remote host, enter its Tailscale address and **Durop session
pairing credential**, then connect. Do not enter a Tailscale API key or OAuth
secret in the pairing field. The remote host supplies the catalog and playback
state; media plays on the host's display. Try a clip, its Stop action and
**STOP ALL**. On the host revoke remote access and verify commands stop; enabling
sharing again requires a fresh pairing credential.

For connectivity checks, both Nix shells include `tailscale`. On macOS that
open-source CLI may target a different daemon from the installed GUI app. Use
the GUI app's bundled CLI when checking its enrollment:

```sh
TAILSCALE_BE_CLI=1 /Applications/Tailscale.app/Contents/MacOS/Tailscale status
TAILSCALE_BE_CLI=1 /Applications/Tailscale.app/Contents/MacOS/Tailscale ip -4
TAILSCALE_BE_CLI=1 /Applications/Tailscale.app/Contents/MacOS/Tailscale ping HOST_TAILSCALE_IP
```

Replace `HOST_TAILSCALE_IP` with the playback Mac's address. See the official
[CLI reference](https://tailscale.com/docs/reference/tailscale-cli) and
[macOS variants](https://tailscale.com/docs/concepts/macos-variants). A successful
ping checks the peer, not the TCP grant or Durop authentication. No router port
forwarding, Serve or Funnel is required. Ending Durop sharing closes its listener;
Tailscale enrollment persists. Desktop, recording and two-Mac qualification are
covered in the [session runbook](session-runbook.md); native fullscreen overlays
remain unsupported. This guide is not evidence of live provisioning or two-Mac acceptance.
