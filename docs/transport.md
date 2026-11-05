# Remote-session adapter

This is the independent issue #8 transport component. Application mode controls,
the native host dispatcher, and two-Mac acceptance remain integration work. It does
not start a listener by being compiled or by opening the bootstrap window.

The host explicitly calls `transport::enable` on its one long-lived Tokio runtime,
providing `SessionConfig`, `HostBridge`, and `RemoteCatalog`. Keep the returned
`RemoteSession` only while remote access is enabled. `revoke()` immediately marks
queued requests inactive and schedules connection/listener shutdown; `close().await`
waits for shutdown. Dropping the handle also revokes. A new enable generates a new
256-bit OS-random credential. Show `credential.bearer()` only in the explicit host
pairing UI; never put it in a URL, command-line argument, file, analytics, or log.
The credential's Debug representation is redacted.

Production configuration requires an exact configured Tailscale IPv4 or IPv6 address
and a nonzero port (the infrastructure example uses TCP 48732). Enable requires the
address on a local operating-system interface and in `tailscale status --json`'s
running self identity. Each session checks again every two seconds. Failure revokes
the session, aborts its tasks, and closes the listener. A slow status command times
out after two seconds. No wildcard, loopback, LAN, MagicDNS resolution, interface
fallback, enrollment, or infrastructure apply is performed. The loopback bypass is
compiled only for isolated tests. Keep the `tailscale` CLI on the application's PATH;
the standalone macOS Tailscale app may require its documented CLI setup.

The POC uses `ws://TAILSCALE_IP:48732/control`: confidentiality and peer-network
authentication depend on Tailscale's encrypted network, while the independent
session credential grants application access. A public listener is unsupported.
Install and authorize Tailscale separately. Follow the [network runbook](../infra/cdktn/tailscale/README.md)
for policy backup/import, reviewed deployment, installation/login, and the narrow
controller-to-host port grant. Tailnet membership alone grants no durop access.
Ending a durop session leaves Tailscale enrollment and policy intact.

## Wire protocol

Use native WebSocket clients that can supply `Authorization: Bearer CREDENTIAL`
and `Sec-WebSocket-Protocol: durop.v1`. The server requires exactly one of each and
echoes `durop.v1`. The only accepted request target is `/control`, with no query.
Native clients may omit Origin; if present it must exactly match one configured
origin. Browser JavaScript cannot set this Authorization header and is not supported
as a controller by this adapter. Do not work around that with URL tokens.

Every connection first receives `{"type":"snapshot","snapshot":...}` using
the canonical issue #5 `HostSnapshot`. Requests are JSON tagged by `type`:

```json
{"type":"command","envelope":{"protocolVersion":1,"sessionId":"host-session","commandId":"unique-request","catalogRevision":"resolved-commit","command":{"type":"play","clip_id":"podcast/applause"}}}
```

Other requests are `{"type":"snapshot"}`, `{"type":"catalog"}`, and
`{"type":"icon","clip_id":"podcast/applause"}`. Commands use the same
canonical `Envelope` and `PlaybackCommand` as local traffic, including `stop` with
`instance_id`, and `stop_all`. Unsupported protocol/session identity terminates the
connection before queueing. The host rejects revision mismatches, invalid IDs,
unknown clips/instances, and policy violations before playback, and deduplicates
session/command IDs. An accepted reply is not a report of audible playback.

Replies are `outcome` (with `command_id` and canonical `outcome`), `catalog` (with
`catalog`), or `icon` (with `clip_id` and `png`, a JSON array of byte values).
An unknown icon yields `{"type":"error","code":"unknown_icon"}`. Queue
saturation or another in-flight normal command yields `busy`; do not retry blindly
with a new command ID. Host events are `{"type":"event","event":...}`.
Track canonical event sequence numbers, discard events at/below the latest snapshot
sequence, and request a snapshot after a gap. A lagging server-side subscriber sends
a new snapshot rather than inferring missing state.

The trusted catalog loader supplies only controller-safe metadata and an immutable
map of known clip IDs to validated PNG bytes. No remote filesystem path or asset URL
is supported. There are at most 256 icons, each at most 32 KiB, and metadata is at
most 64 KiB. Full playback assets and Git credentials never belong in that map.

## Bounds and host wiring

At most four connections (including unauthenticated upgrades) exist per session.
An upgrade has three seconds; input frames/messages are at most 16 KiB; output is
at most 256 KiB and has a five-second send deadline. Idle connections close after
60 seconds. Every incoming message, including ping/pong, consumes one of 20 requests
per one-second window. Malformed JSON, unknown fields, binary messages, oversized
frames, and excess rates terminate the connection. Credentials are fixed length.

`HostBridge` carries bounded normal and emergency `mpsc` senders, a watch receiver
for the current typed host snapshot, and a bounded broadcast sender for typed host
events. Network tasks use `try_send`, never block Bevy, and hold at most one pending
command per connection. They continue reading the socket while awaiting host replies,
so disconnect invalidates pending work. Host replies time out after two seconds.

The native dispatcher must check `RemoteCommand::is_active()` immediately before
the shared host policy path and skip stale requests. This includes disconnected,
revoked, superseded, and timed-out requests. Drain the emergency lane first; a newly
accepted StopAll must invalidate normal requests already queued across all controllers
(use the host ingress barrier). A same-connection StopAll also closes that connection's
pending normal reply. Return the canonical host outcome through `reply`, publish
host events and fresh snapshots, and keep their sizes within the output bound.

Never persist an adapter queue or replay controller button presses on reconnection.
Reconnection gets the current snapshot. Already accepted playback can finish after
network loss. Re-enabling a recording session should create a new host session ID as
well as a fresh credential, preventing reuse of old command identities.

`diagnostics(peer)` performs only bounded, read-only `tailscale status --json`,
`netcheck --format=json`, and optional one-packet `ping` to a tailnet IP. It returns
selected self fields, connectivity information, and direct/relay ping text; it omits
raw account/device status. It never runs login, up, down, policy, plan, or apply.
Diagnostics are host-local, not remotely callable. They do not log session secrets.

## Validation and remaining acceptance

Run `nix develop --command cargo test --locked transport::tests`, followed by the
repository's formatting, locked check/test, and clippy commands. Real loopback sockets
exercise successful authentication, rejection of credentials/origin/path/version,
bounded connections and rates, malformed/oversized messages, catalog/icon scoping,
reconnect snapshots, priority StopAll, disconnect cancellation, and revocation. These
tests never bind to a real Tailscale interface or change network configuration.

Two separately enrolled Macs on different networks still must prove authorized
catalog/icon browse, play/stop/status, concurrent local/remote policy, wrong revisions,
duplicate retries, and denied-identity access to the configured TCP port. Record
direct versus relayed transport and controller-send, acknowledgment, and actual
backend-start timings. A Tailscale ping alone proves neither port reachability nor
durop authorization. No such live acceptance has been performed by these tests.
