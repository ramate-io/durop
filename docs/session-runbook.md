# One-app session runbook

Use the packaged `Durop.app` described in [packaging](packaging.md). Both Macs
run the same app; there is no separate host daemon or browser controller.
Native fullscreen overlay presentation was absent in the measured TextEdit Space
and remains unsupported in that configuration. Start acceptance on an ordinary
desktop; do not infer fullscreen-game support from desktop compositing.

## Local playback

1. Launch Durop. The default bundled podcast pack provides audio, transparent
   GIF, video-with-audio and alpha-video fixtures. Select **This Mac** and ensure
   the local host is running.
2. Search titles/tags/categories, then click a clip. Its state/readiness and any
   individual failure appear in the panel. Choose the target with **Display →**.
   Folder names supply initial categories; identity is independent of folder moves.
3. Use per-instance Stop or **STOP ALL**. Escape stops all when focus is outside
   a text field; while editing, Escape first exits the field. Do not rely on a
   global keyboard hook.
4. Hide or close the panel only when its reopen path is available. The same
   process continues hosting; reopen from the Dock or attached terminal `panel`.
   `stop` and `quit` are optional terminal safety controls. No second executable
   is required to keep hosting or reopen the panel. If no reopen path exists,
   the panel stays available.

To select a local pack, edit **Pack** and press **Load / refresh**. Use an absolute
path when launching from Finder. Directory packs are copied into the user cache,
validated and pinned; stopping/reloading starts a new catalog session rather
than mutating a live pin. A failed preparation retains the previous cached
snapshot. Current UI reload stops the session before attempting a replacement.

For Git packs, the field accepts `https://HOST/ORG/PACK.git#REF` or explicitly
selected `git+file:///absolute/repository#REF`. Existing Git/helper setup is
required for refresh; local/cached playback is independent of Git. Supply a
branch, tag or exact commit, not revision expressions. No URL credentials or
remote Git LFS materialization are supported. See [pack format](packs.md).

## Host and remote controller

Install and authorize Tailscale separately on both Macs. Durop uses its CLI for bounded read-only verification and discovers the
standard installed macOS app executable even from Finder. Follow [the Tailscale runbook](../infra/cdktn/tailscale/README.md),
including its standalone macOS setup. Verify the configured identity/tags
and narrow ACL policy. Durop does not enroll machines or apply tailnet policy.
Do not expose the listener on a wildcard/LAN interface or through public
forwarding.

On the playback Mac, select **This Mac**, load the intended pack and start its
local host. Enter that Mac's explicit Tailscale IP and port (for example
`100.x.y.z:48732`) in **Tailnet address**, then choose **Enable remote host**.
The listener verifies that the address belongs to the authenticated local
Tailscale node. The private pairing secret is displayed for this session; share
it privately with the intended controller and keep it out of recordings/logs.

On the controller Mac, choose **Remote host**, enter the host's address and the
pairing secret, then **Connect**. The secret field is masked and cleared after
submission. Remote catalog metadata and icons are supplied by the host; media
stays on the host. Commands target that host's current session/revision and
receive acknowledgements/state updates. A stale catalog/session is rejected.
Disconnected commands are not replayed after reconnect.

Optional launch configuration, with the app closed first:

```sh
/Applications/Durop.app/Contents/MacOS/durop --mode host --listen-tailscale 100.x.y.z:48732
/Applications/Durop.app/Contents/MacOS/durop --mode controller --connect ws://100.x.y.z:48732/control
```

The controller launch only prefills the destination; enter its secret in the UI
and Connect explicitly. Controller mode starts without a local pack. Host mode
uses the same app/window with playback authority; hiding its panel does not
terminate the process. Pairing material is never placed in command arguments.

Use **Revoke remote** to stop listener access and invalidate its session
credential. **STOP ALL** is the immediate playback safety action; stop hosting,
Disconnect and Quit when the recording session ends. Quit requests playback
teardown and disables networking. Ending Durop does not remove Tailscale
enrollment or modify tailnet policy.

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
The generated orientation fixture must be asymmetric before claiming row
orientation correctness. Avoid sharing private windows, pairing secrets or
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
