# Acceptance and timing capture

The user performs the final gameplay, recording, and two-Mac acceptance. The tools
here collect reproducible local observations and leave missing evidence explicit.
Passing unit tests, a decoder probe, an image asset submission, or a Tailscale ping
does not demonstrate audible output, displayed pixels, recording quality, or usable
game input. See [packaging](packaging.md), [the panel](panel.md),
[media](media-backend.md), and [transport](transport.md) for the application/setup contracts.

## Available local evidence

At source `78e643506f4478bdc32fdab463fe5cd9e07869cd`, the explicit policy/loopback probe
ran on Mac16,11, 48 GiB RAM, arm64, macOS 27.0.1. It used a debug test executable,
an empty host receiving StopAll, 30 samples per origin, and a 1 ms simulated main-loop
poll. The diagnostic capture was complete with zero dropped events, evicted traces, or writer
errors. [Raw JSONL](evidence/local-loopback-78e6435.jsonl) and the
[aggregate report](evidence/local-loopback-78e6435.json) include source/fixture and
test-executable identity. The fixture is loaded for the catalog; no decoder is started.

| Observation | Count | p50 ms | p95 ms | Max ms |
|---|---:|---:|---:|---:|
| Local queue to policy acknowledgment | 30 | 0.001 | 0.005 | 0.014 |
| Remote host handoff to policy acknowledgment | 30 | 0.054 | 0.113 | 0.151 |
| Same-Mac authenticated loopback controller round trip | 30 | 1.981 | 2.624 | 2.827 |

This probe measures neither panel click latency nor media output, game performance,
A/V offset, recording, sustained resource behavior, or a second network. Those entries
remain unmeasured in the report and belong to the user's final acceptance run.

## Private capture directory and artifact identity

Python 3 uses only its standard library. Create a new directory outside Git:

```sh
python3 scripts/acceptance.py init /tmp/durop-run --app /Applications/Durop.app
```

The directory is mode 0700; output files use 0600 and `create_new` semantics. Existing
files are never overwritten. `environment.json` records model, RAM, OS and architecture,
the source checkout revision separately from the packaged app revision, and the SHA256
of the actual signed executable. For an app bundle, source/version/SDK provenance comes
from `Contents/Resources/BUILD-INFO.json`; its pre-signing executable hash is not treated
as the signed artifact hash. Without a bundle, fill the app revision/version explicitly.

Fill in the exact catalog snapshot/pack commit, media dimensions/frame rates, display
resolutions/scaling, game revision/settings, and recording resolution/FPS/system-audio
settings. The template has no fabricated defaults or passed scenarios. Use neutral run
labels such as `cold`, `warm`, `remote`, `baseline`, `recording`, `idle`, and `active`.

## Opt-in bounded application observations

Start the same application executable with a new diagnostics file:

```sh
/Applications/Durop.app/Contents/MacOS/durop \
  --diagnostics /tmp/durop-run/warm.jsonl
```

No pairing credential belongs in this command. Enter it in the protected controller
field after selecting the intended host. Normal application logging stays independent:
the timing recorder uses a scoped `tracing` dispatch and does not replace Bevy's global
subscriber. Timing events contain numeric trace IDs, a fixed stage/origin label, and
monotonic elapsed microseconds. They exclude tokens, repository URLs, user command IDs,
clip IDs/titles/contents, playback paths, and raw protocol bodies.

Each event is at most 4096 bytes. The caller only formats a bounded record and performs
`try_send` into a 256-record queue; a dedicated thread performs filesystem I/O. The
16 MiB file limit preserves complete JSONL events and reserves a final loss summary.
Correlation maps hold at most 256 pending commands and 256 instances. A full queue,
full file, or evicted trace is counted. App shutdown waits at most two seconds for a
summary/flush, after the render loop exits. Missing summaries or nonzero losses make
the report incomplete/lossy rather than silently successful. Force-killing the process
can lose queued records. Profiling overhead itself must be included in comparisons.

| Stage | Actual observation |
|---|---|
| `command_acknowledged` | Queue admission to shared host policy outcome consumption |
| `command_rejected` | Queue admission to rejection consumption |
| `backend_started_callback_observed` | Queue admission to application consumption of backend Started |
| `first_frame_submitted` | Queue admission to the first decoded frame added as a Bevy image asset |
| `finished_callback_observed`, `stopped_callback_observed`, `failed_callback_observed` | Application consumption of the named backend callback |
| `controller_ack_roundtrip` | One controller's monotonic send-to-correlated-reply interval, before snapshot recovery |

`local` starts at successful in-process queue admission. `remote_host_handoff` starts
when the host adapter admits an authenticated remote request into the shared queue;
it excludes controller/network transit. Controller round trips are measured on the
controller alone. Never subtract wall-clock timestamps from separate Macs to invent
one-way latency. A reply may be a rejection; examine the rejection count as well.
Timing rows cover admitted commands with observed outcomes. Admission failures,
pre-admission rejection, or a command canceled by disconnect may have no timing row.
Retain total button-attempt/failure counts in the operator evidence; these percentiles
alone do not establish a zero-failure rate.

Image asset submission precedes Bevy render preparation/GPU presentation. Backend
Started precedes any independently verified audible samples/displayed pixels. Audio
clips do not produce an image submission metric. Use the recorded flash/beep fixture
for actual A/V offset and physical output timing.

For cold runs, record a fresh app/first decoder preparation and describe OS file-cache
state as uncontrolled. For warm runs, repeat the same clip after a prior completion;
record that definition rather than claiming an uncached disk test. Capture enough
samples to make p95 useful, retaining count/max and all failed/rejected attempts.

## Same-Mac protocol measurement without network exposure

An explicit ignored test exercises the real native client, authenticated loopback
WebSocket, idle-waking adapter, and shared host policy. Loopback enable/connect helpers
exist only under `cfg(test)`; production still rejects loopback/wildcard/LAN targets.
It sends StopAll against an empty host to measure policy acknowledgments, not playback.

```sh
DUROP_ACCEPTANCE_OUTPUT=/tmp/durop-run/loopback.jsonl \
  nix develop --command cargo test --locked \
  network_app::tests::capture_same_mac_policy_and_loopback_acknowledgments \
  -- --ignored --exact --nocapture
```

The default is 30 local samples plus 30 paced loopback samples, with a 1 ms simulated
main-loop poll. `DUROP_ACCEPTANCE_SAMPLES` accepts 1..300. This measures neither UI
click latency nor media output and is not a substitute for the two-network run.

## Gameplay, recording, and resources

Use the same game scene/settings/display and recorder settings for four distinct runs:
Maybraid alone (`baseline`), Maybraid plus recording (`recording`), both plus Durop idle
(`idle`), and both with a representative active drop (`active`). Keep capture duration
and sampling method identical. Export individual game frame intervals to CSV with a
`frame_ms` column. An average FPS number is insufficient.

Use Instruments for CPU/GPU investigation and actual frame pacing; Apple documents
the [Game Performance and Metal system trace workflow](https://developer.apple.com/documentation/xcode/analyzing-the-performance-of-your-metal-app).
Inspect the installed templates/help before capture:

```sh
xcrun xctrace list templates
xcrun xctrace help record
xcrun xctrace record --template 'Time Profiler' --attach PID \
  --time-limit 30s --output /tmp/durop-run/cpu.trace
```

Use an appropriate Game Performance/Metal System Trace template in Instruments to
attribute GPU and scheduling changes; process CPU alone is not GPU load. Keep raw
Instruments traces private because they can include unrelated process details.

The lightweight process sampler records only RSS and CPU for an explicitly selected
PID, not command lines or unrelated processes:

```sh
python3 scripts/acceptance.py sample --pid PID --seconds 1800 \
  --interval 1 --out /tmp/durop-run/active-resources.csv
```

Perform at least 30 minutes of mixed playback in the whole-display recording, including
repeated restart/stop cycles, every supported fixture, rapid overlap, StopAll, panel
hide/reopen, and a display change/unplug. Check cleanup after terminal callbacks and
after Quit. A short flat RSS trace does not establish 30-minute stability. Decoder
implementation caps are structural bounds, not measured queue-occupancy traces.

## Two Macs and safe diagnostic capture

Follow the packaged setup and network runbook. The authorized controller must browse
cached icons, trigger/stop, and see truthful status. Verify the configured TCP port and
session authentication from allowed and denied identities; a ping alone is insufficient.
Interrupt a controller connection while playback is active, then reconnect deliberately.
The host may finish accepted playback; no queued/offline button presses may fire later.
Revocation must disconnect existing access and reject the old credential.

After Tailscale is separately installed/authorized, optional read-only sanitized capture:

```sh
python3 scripts/acceptance.py network --peer 100.100.0.10 \
  --out /tmp/durop-run/network.json
```

If needed, `--tailscale` selects the installed CLI executable. This runs bounded
status/netcheck/one-packet ping only. The output retains backend/connectivity flags and
direct/relay classification while dropping machine names, user details, addresses, and
raw CLI text. Direct connectivity is not required for correctness. No enrollment,
network policy, provisioning, login, or infrastructure apply is part of this capture.

## Aggregate without inventing evidence

```sh
python3 scripts/acceptance.py report \
  --metadata /tmp/durop-run/environment.json \
  --timing warm=/tmp/durop-run/warm.jsonl \
  --timing remote=/tmp/durop-run/controller.jsonl \
  --frames baseline=/tmp/durop-run/baseline-frames.csv \
  --frames recording=/tmp/durop-run/recording-frames.csv \
  --frames idle=/tmp/durop-run/idle-frames.csv \
  --frames active=/tmp/durop-run/active-frames.csv \
  --resources active=/tmp/durop-run/active-resources.csv \
  --av-offsets /tmp/durop-run/av-offsets.csv \
  --out /tmp/durop-run/report.json
```

Supply only files actually captured; omit absent arguments. A/V CSV uses signed
`offset_ms` values measured from the flash/beep recording, with the sign convention
recorded in the environment notes. The report gives signed and absolute distributions.
All percentiles use nearest rank (`ceil(p*n)-1`), and empty observations remain null
with count zero. Durop's frame-time comparisons require a positive `recording` baseline
p95, isolating its effect from recorder overhead; the game-alone baseline remains a
separate distribution. The proposed
review limits remain warm physical output p95 100 ms, controller acknowledgment p95
250 ms, absolute A/V offset 50 ms, idle game p95 increase 5%, and active increase 10%.
Submission/callback metrics do not establish the physical-output target. Failed targets
must become an explicit follow-up or scope decision, not be removed from the sample.

Update each scenario's `unmeasured` status only after observing its evidence. Attach
artifact hashes/revisions, counts, recordings, frame CSVs, resource CSVs and limitations.
Review public artifacts for unrelated recorder/system information before sharing.

Validation commands for this harness:

```sh
python3 -m unittest discover -s scripts/tests -v
nix develop --command cargo test --locked diagnostics::tests
```
