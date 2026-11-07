# Tailnet quickstart

Durop runs on your Macs. A tailnet administrator prepares network access once;
the host plays media and invites controllers; controllers send commands. Normal
sessions need no infrastructure apply. The [infrastructure runbook](../infra/cdktn/tailscale/README.md)
covers complete-policy ownership, credentials, durable state and recovery.

## Administrator: prepare the existing tailnet

1. Create a dedicated OAuth trust credential in the
   [Tailscale admin console](https://login.tailscale.com/admin) with `policy_file`,
   `devices:posture_attributes` and `devices:core:read`. Copy `.env.example` to
   ignored `.env` at the checkout root. Set `TAILSCALE_OAUTH_CLIENT_ID`,
   `TAILSCALE_OAUTH_CLIENT_SECRET` and explicit `TAILSCALE_TAILNET`. Keep these
   credentials on the administrator's machine.
2. Install and sign into the [Tailscale macOS app](https://tailscale.com/docs/install/mac)
   on each host and controller Mac; approve devices when required. Keep laptops
   owned by their users; guests can remain signed into their own tailnets.
3. If intended remotes cannot already reach the playback Mac on TCP **48732**,
   let `ensure` pull the live ACL, name the playback host, grant
   `autogroup:shared` that port, and save a reviewed plan. Same-tailnet users
   who should reach the host without a device share can be added with repeatable
   `--member` emails; guests stay in their own tailnets and accept a share of
   this Mac. Optional `--deny` adds a policy test that must fail. Existing
   broader grants still apply. Skip this stack if you do not want CDKTN to own
   policy and 48732 already works.

From `nix develop`, with the playback Mac's Tailscale IPv4 from the app:

```sh
cd infra/cdktn/tailscale
uv sync --locked
uv run --locked --no-sync python main.py ensure --host HOST_TAILSCALE_IP
# After reviewing the printed complete policy diff, tests, tailnet and backend:
uv run --locked --no-sync python main.py apply
uv run --locked --no-sync python main.py plan
```

`ensure` writes ignored `network.config.json`, imports the singleton ACL and
prints the saved plan. It preserves unrelated rules and refuses to overwrite a
local config that differs; pass `--replace-config` to rebuild from the live ACL.
The example config is only for a dedicated test tailnet. Keep `magic_dns` absent
unless deliberately taking ownership of that preference. Repeat `ensure` without
`--host` to re-plan from the saved file. Optional `--member you@example.com` and
`--deny other@example.com` add same-tailnet allow/deny tests.

The final plan should report no changes. State and saved plans use a private,
persistent directory outside the checkout. Use the same `--state-dir` for every
command and a separate directory per tailnet. The CLI automatically reads
root `.env` as literal data; existing environment variables win. Use
`--env-file /private/path.env` or `--no-env` to override that behavior. Provisioning
does not enroll devices or generate Durop session pairing credentials.

## Host: choose controls, display and catalog

Start Durop on the playback Mac:

```sh
nix develop --command cargo run -p durop --bin durop --locked
```

1. In **To**, choose the local display. In **Catalog**, choose the clips to play.
2. In **From**, choose **+ → Tailscale controller**, enter a name, and **Add controller**.
   Buttons across the top select which source to edit; local and remote sources
   stay independently enabled. In the new source's **Listener settings**, Durop
   prefills this Mac's Tailscale IPv4 and TCP 48732 when the app CLI can see the
   enrolled node. Confirm or correct the address, then **Enable remote listener**.
   Durop verifies the address belongs to the local enrolled node. Listener settings
   are shared by all Tailscale sources. MagicDNS names are not accepted.
3. If that user needs network access, open **Invitation credentials** to configure
   the host token, then choose **Create network invitation** in their source form.
   This shares only this host device with that user; it does not
   make them a member of the host's tailnet. The host needs its own user-owned
   `TAILSCALE_API_KEY` from its process environment or the UI's secure credential
   storage. Root `.env` is read by the infrastructure manager only; it does not
   export variables into Durop.
4. Choose **Copy invitation link + pairing details**, or **Copy Durop pairing
   details** when no invitation is needed. Send these privately to that controller.
   Each named controller has a separate Durop token.

Keep invitation and pairing credentials out of recordings. The administrator's
OAuth policy credential, the host's invitation API token, Tailscale device login,
and Durop session pairing are separate permissions. The infrastructure manager
excludes `TAILSCALE_API_KEY` from its provider environment.

## Controller: connect to the host

Accept the host's device-share invitation when needed, install Tailscale and
sign into your own account. Shared guests can stay in their own tailnets; users
already authorized in the host's tailnet need no share. Run Durop in controller mode:

```sh
nix develop --command cargo run -p durop --bin durop --locked -- --mode controller
```

In **To**, choose the remote host, enter its Tailscale address and **Durop session
pairing credential**, then connect. Do not enter a Tailscale API key or OAuth
secret in the pairing field. The remote host supplies the catalog and playback
state; media plays on the host's display. Try a clip, its Stop action and
**STOP ALL**. On the host use the controller card's **Disable** or **Revoke
controller** and verify that controller's commands stop. Disable preserves its
token for re-enabling; revocation removes it. Revoke and add the controller again,
or restart the host app and add it, to issue a new token. **Disable remote
listener** stops all remote access but retains individual tokens for re-enabling
during this app run. Removing Durop permission does not remove accepted Tailscale
device sharing; manage that separately in Tailscale administration.

The narrow shared-user grant permits accepted sharing recipients to reach the
Durop port on this host. It does not share other devices or replace the separate
Durop authentication step. Both sides' custom network policies can affect access.
See Tailscale's [device sharing guide](https://tailscale.com/docs/features/sharing)
and [inviting versus sharing](https://tailscale.com/docs/reference/inviting-vs-sharing).

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
