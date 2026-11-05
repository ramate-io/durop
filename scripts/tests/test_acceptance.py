import importlib.util
import json
from pathlib import Path
import tempfile
import subprocess
import sys
import types
import unittest

SPEC = importlib.util.spec_from_file_location("acceptance", Path(__file__).parents[1] / "acceptance.py")
acceptance = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(acceptance)

class Statistics(unittest.TestCase):
    def test_report_isolates_durop_from_recording_overhead(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            for label, value in (("baseline", 10), ("recording", 20), ("idle", 21)):
                (root / f"{label}.csv").write_text(f"frame_ms\n{value}\n")
            args = types.SimpleNamespace(timing=None, frames=[f"{label}={root / (label + '.csv')}" for label in ("baseline", "recording", "idle")], resources=None, av_offsets=None, metadata=None, out=root / "report.json")
            acceptance.report(args)
            result = json.loads(args.out.read_text())
            self.assertEqual(result["gameP95ComparisonBaseline"], "recording")
            self.assertTrue(result["gameP95Comparison"]["idle"]["withinProposedLimit"])
            self.assertEqual(result["gameFrameMilliseconds"]["baseline"]["p95"], 10)

    def test_frame_targets_handle_missing_zero_and_exact_boundary(self):
        self.assertIsNone(acceptance.frame_comparison(None, 20, 5)["withinProposedLimit"])
        self.assertIsNone(acceptance.frame_comparison(0, 20, 5)["withinProposedLimit"])
        self.assertTrue(acceptance.frame_comparison(20, 21, 5)["withinProposedLimit"])
        self.assertFalse(acceptance.frame_comparison(20, 21.1, 5)["withinProposedLimit"])

    def test_child_output_and_time_are_bounded(self):
        self.assertEqual(acceptance.run([sys.executable, "-c", "print('ok')"]), "ok")
        with self.assertRaises(ValueError):
            acceptance.run([sys.executable, "-c", "print('x'*300000)"])
        with self.assertRaises(subprocess.TimeoutExpired):
            acceptance.run([sys.executable, "-c", "import time;time.sleep(2)"], timeout=.05)

    def test_nearest_rank_counts_and_empty_data(self):
        self.assertEqual(acceptance.summary([]), {"count": 0, "p50": None, "p95": None, "max": None})
        self.assertEqual(acceptance.summary(list(range(1, 101))), {"count": 100, "p50": 50, "p95": 95, "max": 100})
        self.assertEqual(acceptance.summary([8, 1, 3, 2]), {"count": 4, "p50": 2, "p95": 8, "max": 8})
        with self.assertRaises(ValueError):
            acceptance.summary([float("nan")])

    def test_incomplete_capture_and_private_fields_are_not_silently_accepted(self):
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "timing.jsonl"
            rows = [{"target": "durop_timing", "fields": {"stage": "diagnostics_start", "schema_version": 1}}, {"target": "durop_timing", "fields": {"stage": "first_frame_submitted", "origin": "local", "trace_id": 1, "elapsed_us": 1200}}]
            path.write_text("\n".join(json.dumps(row) for row in rows))
            report = acceptance.timed_rows(path)
            self.assertFalse(report["complete"])
            self.assertFalse(report["lossless"])
            self.assertEqual(report["metrics"][0]["milliseconds"]["p95"], 1.2)
            rows[1]["fields"]["token"] = "private"
            path.write_text("\n".join(json.dumps(row) for row in rows))
            with self.assertRaises(ValueError):
                acceptance.timed_rows(path)

    def test_complete_capture_reports_loss_and_uses_only_same_host_intervals(self):
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "timing.jsonl"
            rows = [{"target": "durop_timing", "fields": {"stage": "diagnostics_start", "schema_version": 1}}, {"target": "durop_timing", "fields": {"stage": "controller_ack_roundtrip", "origin": "controller", "trace_id": 1, "elapsed_us": 125000}}, {"target": "durop_timing", "fields": {"stage": "diagnostics_summary", "dropped_events": 1, "evicted_traces": 0, "writer_errors": 0, "file_capped": False}}]
            path.write_text("\n".join(json.dumps(row) for row in rows))
            report = acceptance.timed_rows(path)
            self.assertTrue(report["complete"])
            self.assertFalse(report["lossless"])
            self.assertEqual(report["metrics"][0]["milliseconds"]["p95"], 125)

    def test_signed_av_offsets_remain_signed_and_absolute_target_is_separate(self):
        rows = [{"offset_ms": "-80"}, {"offset_ms": "20"}]
        self.assertEqual(acceptance.numeric(rows, "offset_ms", signed=True), [-80, 20])
        with self.assertRaises(ValueError):
            acceptance.numeric(rows, "offset_ms")

if __name__ == "__main__":
    unittest.main()
