#!/usr/bin/env python3
"""Local, bounded measurement capture and honest aggregation; never enrolls a network."""
import argparse
import csv
import datetime
import hashlib
import ipaddress
import json
import math
import os
from pathlib import Path
import platform
import re
import selectors
import shutil
import subprocess
import sys
import time

ROOT = Path(__file__).resolve().parents[1]
STAGES = {
    "command_acknowledged": "Shared host policy outcome observed; not audible/visible output",
    "command_rejected": "Shared host rejection observed",
    "backend_started_callback_observed": "Backend Started callback consumed by application; not proof of output",
    "first_frame_submitted": "First decoded frame added as a Bevy image asset; not proof of GPU presentation",
    "finished_callback_observed": "Backend Finished callback consumed",
    "stopped_callback_observed": "Backend Stopped callback consumed",
    "failed_callback_observed": "Backend Failed callback consumed",
    "controller_ack_roundtrip": "Controller-local send through correlated host reply; excludes later snapshot recovery",
}
ORIGINS = {"local", "remote_host_handoff", "controller"}
SCENARIOS = ["local_panel_host", "remote_cohost", "audio_system_recording", "transparent_gif", "video_audio", "alpha_video", "fullscreen_game_input", "rapid_overlap_stop_all", "disconnect_reconnect_no_replay", "idle_repeated_display_change", "thirty_minute_mixed_recording", "quit_releases_resources"]

def run(args, timeout=5):
    process = subprocess.Popen(args, stdin=subprocess.DEVNULL, stdout=subprocess.PIPE, stderr=subprocess.DEVNULL)
    deadline = time.monotonic() + timeout
    output = bytearray()
    try:
        with selectors.DefaultSelector() as selector:
            selector.register(process.stdout, selectors.EVENT_READ)
            while selector.get_map():
                remaining = deadline - time.monotonic()
                if remaining <= 0:
                    raise subprocess.TimeoutExpired(args, timeout)
                for key, _ in selector.select(remaining):
                    chunk = os.read(key.fd, min(65536, 256 * 1024 + 1 - len(output)))
                    if not chunk:
                        selector.unregister(key.fd)
                    output.extend(chunk)
                    if len(output) > 256 * 1024:
                        raise ValueError("diagnostic output exceeds bound")
            result = process.wait(timeout=max(.001, deadline - time.monotonic()))
            if result:
                raise subprocess.CalledProcessError(result, args)
    finally:
        if process.poll() is None:
            process.kill()
            process.wait(timeout=2)
        process.stdout.close()
    return output.decode("utf-8", errors="replace").strip()

def optional(args):
    try:
        return run(args)
    except (OSError, subprocess.SubprocessError, ValueError):
        return None

def private_file(path, mode="w"):
    return os.fdopen(os.open(path, os.O_CREAT | os.O_EXCL | os.O_WRONLY, 0o600), mode, newline="" if mode == "w" else None)

def write_json(path, value):
    with private_file(path) as stream:
        json.dump(value, stream, indent=2, sort_keys=True, allow_nan=False)
        stream.write("\n")

def summary(values):
    data = sorted(values)
    if any(not math.isfinite(value) for value in data):
        raise ValueError("non-finite measurement")
    if not data:
        return {"count": 0, "p50": None, "p95": None, "max": None}
    return {"count": len(data), "p50": data[max(0, math.ceil(len(data) * .50) - 1)], "p95": data[max(0, math.ceil(len(data) * .95) - 1)], "max": data[-1]}

def frame_comparison(baseline, observed, limit):
    change = 100 * (observed / baseline - 1) if observed is not None and baseline is not None and baseline > 0 else None
    within = change <= limit or math.isclose(change, limit, abs_tol=1e-9) if change is not None else None
    return {"p95IncreasePercent": change, "proposedLimitPercent": limit, "withinProposedLimit": within}

def timed_rows(path):
    if path.stat().st_size > 16 * 1024 * 1024:
        raise ValueError("timing file exceeds application bound")
    metrics, header, footer = {}, None, None
    with path.open() as stream:
        for line in stream:
            if len(line.encode()) > 4096:
                raise ValueError("timing row exceeds bound")
            row = json.loads(line)
            if footer is not None:
                raise ValueError("timing data after final summary")
            if row.get("target") != "durop_timing" or not isinstance(row.get("fields"), dict):
                raise ValueError("unexpected diagnostic target or schema")
            fields = row["fields"]
            stage = fields.get("stage")
            if stage == "diagnostics_start":
                if header is not None or fields.get("schema_version") != 1 or set(fields) - {"stage", "schema_version", "file_limit_bytes", "event_limit_bytes"}:
                    raise ValueError("unsupported or duplicate diagnostics header")
                header = fields
            elif stage == "diagnostics_summary":
                if set(fields) - {"stage", "dropped_events", "evicted_traces", "writer_errors", "file_capped", "bytes_before_summary"}:
                    raise ValueError("unexpected summary fields")
                footer = fields
            else:
                if stage not in STAGES or fields.get("origin") not in ORIGINS or set(fields) - {"stage", "origin", "trace_id", "elapsed_us"}:
                    raise ValueError("unexpected timing fields")
                elapsed = fields.get("elapsed_us")
                if type(elapsed) is not int or elapsed < 0 or elapsed > 2**64 - 1:
                    raise ValueError("invalid monotonic interval")
                metrics.setdefault((stage, fields["origin"]), []).append(elapsed / 1000)
    complete = header is not None and footer is not None
    lossless = complete and all(footer.get(key) == 0 for key in ("dropped_events", "evicted_traces", "writer_errors")) and footer.get("file_capped") is False
    return {"complete": complete, "lossless": lossless, "summary": footer, "metrics": [{"stage": stage, "origin": origin, "observation": STAGES[stage], "milliseconds": summary(values)} for (stage, origin), values in sorted(metrics.items())]}

def csv_rows(path):
    if path.stat().st_size > 8 * 1024 * 1024:
        raise ValueError("measurement CSV exceeds bound")
    with path.open(newline="") as stream:
        return list(csv.DictReader(stream))

def numeric(rows, column, signed=False):
    values = [float(row[column]) for row in rows]
    if any(not math.isfinite(value) or (value < 0 and not signed) for value in values):
        raise ValueError("invalid measurement value")
    return values

def named(inputs):
    result = {}
    for item in inputs or []:
        label, separator, path = item.partition("=")
        if not separator or not re.fullmatch(r"[a-z0-9_-]{1,32}", label) or label in result:
            raise ValueError("use unique short labels: label=FILE")
        result[label] = Path(path)
    return result

def environment(args):
    directory = args.directory
    directory.mkdir(mode=0o700, parents=True, exist_ok=False)
    memory = optional(["sysctl", "-n", "hw.memsize"]) if sys.platform == "darwin" else None
    build_info = None
    binary_hash = None
    if args.app:
        build_path = args.app / "Contents/Resources/BUILD-INFO.json"
        if build_path.exists():
            build_info = json.loads(build_path.read_text())
        binary = args.app / "Contents/MacOS/durop" if args.app.suffix == ".app" else args.app
        digest = hashlib.sha256()
        with binary.open("rb") as stream:
            for chunk in iter(lambda: stream.read(65536), b""):
                digest.update(chunk)
        binary_hash = digest.hexdigest()
    metadata = {"createdUtc": datetime.datetime.now(datetime.timezone.utc).isoformat(), "machine": {"model": optional(["sysctl", "-n", "hw.model"]) if sys.platform == "darwin" else platform.machine(), "ramBytes": int(memory) if memory else None, "architecture": platform.machine(), "os": platform.system(), "osVersion": optional(["sw_vers", "-productVersion"]) if sys.platform == "darwin" else platform.release()}, "sourceCheckoutRevision": optional(["git", "-C", str(ROOT), "rev-parse", "HEAD"]), "appRevision": build_info.get("source_revision") if build_info else None, "appVersion": build_info.get("version") if build_info else None, "appBinarySha256": binary_hash, "appBuildInfo": build_info, "mediaBackend": "AVFoundation (system OS version) and gif crate (Cargo.lock)", "packRevision": None, "mediaDimensionsAndFrameRate": None, "displays": None, "recordingSettings": None, "gameRevisionAndSettings": None, "scenarios": {scenario: {"status": "unmeasured", "evidence": []} for scenario in SCENARIOS}}
    write_json(directory / "environment.json", metadata)
    with private_file(directory / "av-offsets.example.csv") as stream:
        stream.write("offset_ms\n")
    with private_file(directory / "frame-times.example.csv") as stream:
        stream.write("frame_ms\n")
    print("Created private acceptance directory and unmeasured evidence template")

def sample(args):
    if args.pid <= 0 or not 1 <= args.seconds <= 7200 or not .1 <= args.interval <= 60:
        raise ValueError("require positive PID, duration 1..7200 seconds, interval .1..60 seconds")
    started = time.monotonic()
    count = 0
    with private_file(args.out) as stream:
        writer = csv.writer(stream)
        writer.writerow(["elapsed_s", "rss_kib", "cpu_percent"])
        try:
            while time.monotonic() - started < args.seconds:
                try:
                    fields = run(["ps", "-p", str(args.pid), "-o", "rss=,%cpu="], timeout=2).split()
                except subprocess.CalledProcessError:
                    break
                if len(fields) != 2:
                    break
                rss, cpu = (float(value) for value in fields)
                if not all(math.isfinite(value) and value >= 0 for value in (rss, cpu)):
                    raise ValueError("invalid process sample")
                writer.writerow([round(time.monotonic() - started, 6), rss, cpu])
                stream.flush()
                count += 1
                time.sleep(args.interval)
        except KeyboardInterrupt:
            pass
    print(json.dumps({"samples": count, "observedSeconds": round(time.monotonic() - started, 3), "observation": "ps process RSS and CPU; no GPU, output, or long-term stability claim"}))

def report(args):
    timings = {label: timed_rows(path) for label, path in named(args.timing).items()}
    frames = {label: summary(numeric(csv_rows(path), "frame_ms")) for label, path in named(args.frames).items()}
    resources = {}
    for label, path in named(args.resources).items():
        rows = csv_rows(path)
        rss, cpu, elapsed = numeric(rows, "rss_kib"), numeric(rows, "cpu_percent"), numeric(rows, "elapsed_s")
        resources[label] = {"count": len(rows), "observedSeconds": elapsed[-1] - elapsed[0] if elapsed else 0, "rssKiB": summary(rss), "rssStartKiB": rss[0] if rss else None, "rssEndKiB": rss[-1] if rss else None, "rssDeltaKiB": rss[-1] - rss[0] if rss else None, "cpuPercent": summary(cpu), "observation": "Process sampling; neither GPU load nor proof of bounded memory beyond this interval"}
    comparisons = {}
    baseline = frames.get("baseline", {}).get("p95")
    for label, target in (("idle", 5), ("active", 10)):
        observed = frames.get(label, {}).get("p95")
        comparisons[label] = frame_comparison(baseline, observed, target)
    av = None
    if args.av_offsets:
        offsets = numeric(csv_rows(args.av_offsets), "offset_ms", signed=True)
        absolute = summary([abs(value) for value in offsets])
        av = {"signedMilliseconds": summary(offsets), "absoluteMilliseconds": absolute, "absoluteMaxWithin50ms": absolute["max"] <= 50 if absolute["max"] is not None else None, "observation": "Operator measured flash/beep recording offsets; not inferred from app callbacks"}
    result = {"percentileMethod": "nearest rank: sorted[ceil(p*n)-1]", "environment": json.loads(args.metadata.read_text()) if args.metadata else None, "timings": timings, "gameFrameMilliseconds": frames, "gameP95Comparison": comparisons, "processResources": resources, "audioVideoOffsets": av, "unmeasuredByThisTool": ["audibility", "displayed pixels", "game input usability", "GPU utilization", "network one-way latency", "recording quality", "two-Mac authorization"], "proposedPhysicalTargetsNotProvenBySubmissionMetrics": {"warmLocalOutputP95ms": 100, "controllerAcknowledgmentP95ms": 250, "absoluteAVOffsetMs": 50}, "physicalOutputClaimsFromCallbacks": False}
    write_json(args.out, result)
    print("Wrote aggregate report; missing measurements remain null/unmeasured")

def network(args):
    binary = args.tailscale or shutil.which("tailscale")
    if not binary:
        raise ValueError("choose installed Tailscale CLI with --tailscale")
    status = json.loads(run([binary, "status", "--json"]))
    check = json.loads(run([binary, "netcheck", "--format=json"]))
    result = {"backendState": status.get("BackendState"), "netcheck": {key: check.get(key) for key in ("UDP", "IPv4", "IPv6", "MappingVariesByDestIP", "PreferredDERP")}, "connection": "unmeasured", "pingMilliseconds": None}
    if args.peer:
        peer = ipaddress.ip_address(args.peer)
        if not (peer in ipaddress.ip_network("100.64.0.0/10") or peer in ipaddress.ip_network("fd7a:115c:a1e0::/48")):
            raise ValueError("peer must be a Tailscale address")
        output = run([binary, "ping", "--c=1", "--timeout=1s", str(peer)])
        result["connection"] = "relay" if "DERP" in output else ("direct" if "via" in output else "unmeasured")
        elapsed = re.search(r"\bin (\d+(?:\.\d+)?)ms\b", output)
        if elapsed:
            result["pingMilliseconds"] = float(elapsed[1])
    write_json(args.out, result)

def main():
    parser = argparse.ArgumentParser(description=__doc__)
    commands = parser.add_subparsers(dest="command", required=True)
    init = commands.add_parser("init"); init.add_argument("directory", type=Path); init.add_argument("--app", type=Path); init.set_defaults(function=environment)
    capture = commands.add_parser("sample"); capture.add_argument("--pid", type=int, required=True); capture.add_argument("--seconds", type=float, default=1800); capture.add_argument("--interval", type=float, default=1); capture.add_argument("--out", type=Path, required=True); capture.set_defaults(function=sample)
    aggregate = commands.add_parser("report"); aggregate.add_argument("--timing", action="append"); aggregate.add_argument("--frames", action="append"); aggregate.add_argument("--resources", action="append"); aggregate.add_argument("--av-offsets", type=Path); aggregate.add_argument("--metadata", type=Path); aggregate.add_argument("--out", type=Path, required=True); aggregate.set_defaults(function=report)
    diagnose = commands.add_parser("network"); diagnose.add_argument("--tailscale"); diagnose.add_argument("--peer"); diagnose.add_argument("--out", type=Path, required=True); diagnose.set_defaults(function=network)
    args = parser.parse_args()
    try:
        args.function(args)
    except (OSError, ValueError, KeyError, subprocess.SubprocessError) as error:
        parser.exit(2, f"Acceptance capture failed: {error}\n")

if __name__ == "__main__":
    main()
