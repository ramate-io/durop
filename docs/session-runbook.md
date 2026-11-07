# One-app session runbook

Use the packaged `Durop.app` described in [packaging](packaging.md). Both Macs
run the same app; there is no separate host daemon or browser controller.
Native fullscreen overlay presentation was absent in the measured TextEdit Space
and remains unsupported in that configuration. Start acceptance on an ordinary
desktop; do not infer fullscreen-game support from desktop compositing.

## Local playback

1. Launch Durop. The bundled reaction catalog provides GIFs, sound bites and
   video clips. Enable local controls in **From**, then select a local display
   in **To** and choose **Apply** to start local playback authority.
2. Search titles/tags/categories, then click a clip. Its state/readiness and any
   individual failure appear in the panel. Change displays through **To**.
   Folder names supply initial categories; identity is independent of folder moves.
3. Use per-instance Stop or **STOP ALL**. Escape first cancels an open modal or
   closes a menu, then exits a focused field, and otherwise stops all. Do not rely on a
   global keyboard hook.
4. Hide or close the panel only when its reopen path is available. The same
   process continues hosting; reopen from the Dock or attached terminal `panel`.
   `stop` and `quit` are optional terminal safety controls. No second executable
   is required to keep hosting or reopen the panel. If no reopen path exists,
   the panel stays available.

To select a local pack, choose **Catalog → Local directory**, enter its path and
press **Apply**. Use an absolute
path when launching from Finder. Directory packs are copied into the user cache,
validated and pinned; stopping/reloading starts a new catalog session rather
than mutating a live pin. A failed preparation retains the previous cached
snapshot. Current UI reload stops the session before attempting a replacement.

For Git packs, choose **Catalog → Git repository** and provide the HTTPS URL,
literal ref and optional pack folder. The Durop example uses
`https://github.com/ramate-io/durop.git`, `main`, and `sample-packs/reactions`.
Launch arguments also accept `--pack https://HOST/ORG/PACK.git#REF:PACK/FOLDER`
or explicitly selected `git+file:///absolute/repository#REF`. Existing Git/helper setup is
required for refresh; local/cached playback is independent of Git. Supply a
branch, tag or exact commit, not revision expressions. No URL credentials or
remote Git LFS materialization are supported. See [pack format](packs.md).

## Host and remote controller

Start with the [Tailnet quickstart](tailnet-quickstart.md) for OAuth, `uv run
python main.py ensure`, the Tailscale macOS app and the complete Durop pairing flow.

Install and authorize Tailscale separately on both Macs. Keep each device owned
by its user. Guests can stay in their own tailnets and accept a share of the
playback device; same-tailnet users with existing policy access need no share.
Durop discovers the standard installed macOS app executable even from Finder.
See the [infrastructure reference](../infra/cdktn/tailscale/README.md) for the
Nix CLI's macOS daemon caveat. Durop does not enroll machines or apply policy.
Do not expose its listener through a wildcard/LAN address or public forwarding.

On the playback Mac, choose its local display in **To** and load the intended
pack in **Catalog**. In **From**, choose **+ → Tailscale controller**, enter a name,
and **Add controller**. Select that source and open **Listener settings**, enter
this Mac's Tailscale IP and port (for example `100.x.y.z:48732`), and choose
**Enable remote listener**. The listener
verifies that the address belongs to the authenticated local Tailscale node.
Local controls and multiple remote controller sources can remain enabled together.

Buttons across the top of **From** select one source's form without changing
which sources are enabled. Use **+** to add more sources; each receives a separate
runtime credential. If the recipient needs access to this device, configure
**Invitation credentials**, then choose **Create network invitation** in their form;
this uses the host's user-owned `TAILSCALE_API_KEY` from its environment or secure
UI storage. The invitation shares this one device and does not enroll the user
in the host's tailnet. Network policy must admit shared guests on TCP 48732,
such as the sample's narrow `autogroup:shared` grant. See the official
[sharing guide](https://tailscale.com/docs/features/sharing).

Choose **Copy invitation link + pairing details**, or **Copy Durop pairing
details** when no new share is needed. Send these privately to that controller;
keep the clipboard and pairing panel out of recordings. Infrastructure OAuth
credentials and the host API key never belong in these pairing details.

On the controller Mac, accept the optional device share while signed into the
recipient's own account. In Durop's **To**, choose the remote host, enter its
address and that controller's Durop pairing token, then **Apply**. The masked
secret is cleared after submission. The remote host supplies catalog metadata,
icons and playback state; media stays on the host. Commands target its current
session/revision and receive acknowledgements/state updates. Stale sessions are
rejected; disconnected commands are not replayed after reconnect.

Optional launch configuration, with the app closed first:

```sh
/Applications/Durop.app/Contents/MacOS/durop --mode host --listen-tailscale 100.x.y.z:48732
/Applications/Durop.app/Contents/MacOS/durop --mode controller --connect ws://100.x.y.z:48732/control
```

For development, launch a controller with:

```sh
nix develop --command cargo run -p durop --bin durop --locked -- --mode controller
```

Controller launch prefills the destination; open **To**, enter its token and
choose **Apply** explicitly. Controller mode starts without a local pack. Host
mode uses the same app/window with playback authority; hiding its panel does
not terminate the process. Pairing material never belongs in command arguments.

On an individual controller card, **Disable** suspends permission and preserves
its token for re-enabling; **Revoke controller** removes that source and its
credential. Revoke and add it again to rotate its token. Restarting the host app
also discards runtime controller credentials. **Disable remote listener** closes
all remote access while retaining individual tokens for re-enabling during this
app run. Durop revocation does not remove an accepted Tailscale share; remove
that separately in Tailscale administration. **STOP ALL** stops playback
immediately. Disconnect and quit when the recording ends; quitting tears down
playback and networking without changing Tailscale enrollment or policy.

## Record and qualify

Use a capture tool that records the whole selected display and system output
without a virtual audio driver. For example OBS's macOS Screen Capture source
supports system audio on supported macOS/OBS versions; follow its
[official desktop-audio guide](https://obsproject.com/kb/macos-desktop-audio-capture-guide).
Grant the capture application's requested screen/system-audio permission in
System Settings. Check that audio reaches both the monitor output and recording;
a muted decode probe establishes neither.

Record the flash/beep video fixture and inspect the saved result to measure A/V
synchronization. Include transparent GIF/alpha video over a safe desktop,
restart, bounded loop, StopAll, automatic clearing and an unfocused trigger.
The optional `Resources/sample-packs/media-proof` pack includes an asymmetric
orientation clip: green-dominant tall top-left marker, blue-dominant wide
bottom-right marker and partially transparent outer edges. Record that clip to
establish visible orientation/fringe quality; CPU readback alone does not prove
Bevy's displayed orientation. Avoid sharing private windows, pairing secrets or
unrelated account/device details.

The user will perform the two-Mac acceptance: installed package without the
development environment, local media and system audio, remote authentication,
revision handling, StopAll/revoke/reconnect, direct versus relay transport,
network interruption, selected-display behavior, and safe session shutdown.
Physical controller/game/mixed-monitor/fullscreen coverage must be recorded
separately. Current native decode probes and loopback transport tests do not
substitute for that evidence.

## Troubleshooting

- **Missing media/icons:** run the installed binary with `--self-check` from an
  arbitrary directory. Keep the complete `.app`; moving only its executable
  loses Resources. Do not set Bevy/Cargo asset variables to make a broken bundle
  appear healthy.
- **Pack load error:** read the panel's focused error. Unsupported/corrupt media
  fail individually. Unresolved LFS pointers need local materialization before
  preparation. Missing external Git affects refresh, not cached local playback.
- **Cannot connect:** verify the exact Tailscale IP, service/CLI availability,
  listener port, ACL and session secret. Tailnet ping alone does not demonstrate
  the Durop port or credential handshake. Never paste full private status JSON
  into shared diagnostics.
- **Overlay absent in fullscreen:** this is an observed unsupported mode; use a
  normal desktop/windowed target. Do not disable system protections or assume a
  higher window level fixes it.
- **Device loss/no sound:** stop the affected clip, verify the default output
  and retry; retain the individual backend error. Physical device-loss recovery
  and audible A/V synchronization remain acceptance work.
