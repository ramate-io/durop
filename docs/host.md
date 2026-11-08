# Playback authority

`durop-protocol` owns the version 2 local/remote wire contract. `durop-host` owns policy decisions and a catalog snapshot pinned with `Arc<Catalog>` for the session. Create a fresh session ID for every new host; ending a recording must stop all workers before retiring that host. A new catalog revision takes effect in a new session, never underneath active playback.

An envelope contains `protocolVersion`, `sessionId`, `commandId`, `catalogRevision`, and `command`. Commands use a `type` tag: `play` with `clip_id`, `stop` with `instance_id`, or `stop_all`. Clip IDs are stable catalog identities such as `pack/clip`. Callers cannot override asset paths, playback limits or looping. Unknown envelope/command fields fail deserialization.

The experimental `play_visual` command includes `clip_id` and `placement`;
`set_visual` includes `instance_id` and optional `placement` (null restores catalog
defaults). Placement uses integer basis points: `widthBps` is 1–10000 and
`xBps`/`yBps` are 0–10000, where 10000 means 100%. `fit` is `catalog` (also the
default when omitted) or `cover`; cover fills the fitting area with centered
cropping while preserving aspect ratio. Explicit width scales a display-proportioned
fitting box; position interpolates between the
corresponding display edges while keeping the rendered media within them. These
commands apply only to GIF/video clips. Active snapshots retain the explicit
override, and `visual_changed` events carry live updates for remote recovery.
Defaults remain in the immutable catalog; a null override uses them.

`Host::accept` validates and deduplicates commands before creating occurrences. `Accepted` means policy acceptance, not audible/visible playback. Each occurrence has a host-generated ID and generation and moves through preparing, playing, stopping, and one terminal state. Asset readiness is independent: the snapshot reports the latest preparation result for each clip. Only a matching backend `Started` callback after preparation/start permission changes the occurrence to playing. Stop requests report stopping until native teardown is acknowledged. Repeated terminal callbacks produce no new transition.

The cap is eight concurrent native workers, including preparing and stopping workers. Stack creates independent occurrences. Ignore-if-playing returns the existing preparing/playing occurrence without restarting it. Restart cancels older same-clip occurrences, prepares the replacement, and withholds its start until prior workers acknowledge resource release. At the worker cap restart is rejected Busy; callers must retry with a new command ID after a worker releases. There is no playback queue.

The dedupe window retains the last 1024 accepted/rejected outcomes within the current session. A retry within that window returns its original outcome, including Busy, without emitting another acceptance or executing another effect. Reusing an ID for another payload still returns the original outcome. Clients must use new IDs for intentional new requests and must not retry commands after the dedupe window or across sessions. Old-session requests are rejected. Instance terminal history retains 256 occurrences; targeting an older retired ID returns UnknownInstance.

Events have monotonically increasing sequence numbers. The host retains at most 256 undrained events; a missing sequence requires fetching `Host::snapshot`, not guessing playback state. Snapshots include revision, sequence, clip readiness, active instances and bounded terminal history, without native handles or asset paths. They describe backend-confirmed state, not audio routing/capture acceptance.

## Shared ingress and Bevy integration

`command_channel(wake)` returns cloneable `CommandSender` and single-owner `CommandInbox`, using bounded Tokio channels with 128 ordinary slots and eight dedicated StopAll slots. `try_send` returns Busy or Closed on admission failure. Queue admission is distinct from `Host::accept`; `drain` returns command receipts. A saturated ordinary lane cannot block StopAll. If the emergency lane itself is saturated it reports Busy; already pending StopAll commands retain priority.

Drain at most eight emergency and 32 ordinary requests per Bevy update. New accepted StopAll requests supersede older queued commands so buffered Play requests cannot resurrect playback immediately after emergency stop. Duplicate or invalid StopAll requests do not advance that cancellation barrier. The wake callback must send a nonblocking event to Bevy/winit; it runs after every admitted command and again when the per-update budget leaves work pending. Do not construct a Tokio runtime per command. The application owns one runtime for asynchronous IO; decoding must not block Bevy's update thread. Main-thread-only native controls belong in a Bevy `NonSend` resource, with native asynchronous decoding or safely transferable conversion work offthread. An authenticated transport adapter must preserve this same cancellation barrier when forwarding its own request/reply queues, and discard revoked/disconnected requests before calling the host.

## Backend adapter contract

Drain `BackendEffect`s after command processing and callbacks. A `Prepare` carries the immutable catalog `Clip` plus `BackendToken { instance_id, generation }` and the optional placement override. The backend uses the pinned asset path, looping/max duration and visual metadata. Native resources remain behind the adapter; one backend occurrence per host instance supports stack without requiring native objects to implement `Send`. Adapters communicate generation-tagged `Prepared`, `Started`, `Finished`, `Stopped`, or `Failed` events through `Host::backend_event`.

`SetVisual` carries an occurrence token and optional placement override. The
application changes its rendering geometry without seeking, restarting or
replacing the decoder. The host accepts it only for preparing/playing visual
occurrences and coalesces pending placement effects. Stop and terminal cleanup
remove those pending effects. The renderer processes callbacks and policy effects
before updating or creating sprites, so a reset queued before a frame is presented
cannot briefly display the previous geometry.

`Prepared` proves only asset/backend readiness. It requests a separate `Start` effect when policy allows. A `Start` performs actual backend start; its later `Started` acknowledgement marks playing. `Stop` must cancel preparation, audio, decoder, texture/frame publishing and all occurrence effects together, then report `Stopped`. It must also acknowledge an unknown/not-yet-prepared token, because cancellation can supersede an unsent Prepare. `Finished` and `Failed` callbacks must mean the worker has released occurrence resources. Stop racing completion/failure produces one Stopped terminal result. Late callbacks with a stale generation or stopped/terminal ID are ignored.

Pending effects coalesce per token: stop removes unsent prepare/start, and terminal release removes all pending token effects. With eight active occurrences there are at most twenty-four pending effects; they must be drained promptly. No effect is silently dropped for a live token. Failed preparation keeps clip readiness separate from occurrence termination. Error messages are bounded to 512 characters; adapters must omit credentials and private device information.

This change tests the real deterministic host policy with a scripted backend boundary. It does not claim local audible/visible runtime acceptance. The native media worker from issue #3 and the Bevy application bridge/panel from issue #6 must execute these effects and feed real callbacks before that acceptance is complete.
