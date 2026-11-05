import importlib.util
import json
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest

SCRIPT = Path(__file__).parents[1] / "conventional_commits.py"
SPEC = importlib.util.spec_from_file_location("conventional_commits", SCRIPT)
commits = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(commits)


class ConventionalHeaders(unittest.TestCase):
    def test_valid_subjects_support_scopes_breaking_changes_and_case(self):
        for header in (
            "feat: add remote controls", "fix(panel): retain keyboard focus",
            "ci(workflows): check pull request titles", "docs: describe local sessions",
            "feat(protocol)!: remove the legacy envelope", "fix!: reject stale credentials",
            "test(host/state): cover concurrent stops", "refactor(network_app): simplify handoff",
            "ReVeRt: restore the previous behavior", "docs: clarify café labels",
        ):
            with self.subTest(header=header):
                commits.validate_header(header)

    def test_invalid_subjects_are_rejected(self):
        for header in (
            "", "Add remote controls", "feature: add remote controls", "fix: ",
            "fix(): empty scope", "fix(scope with spaces): invalid scope",
            "fix(panel)! : misplaced colon", "fix:  extra separator space",
            " fix: leading whitespace", "fix: trailing whitespace ",
            "fix: newline\nbody", "fix: tab\there", "fix: control\x1b",
            "Merge pull request #1", "docs: " + "x" * 195,
        ):
            with self.subTest(header=header), self.assertRaises(ValueError):
                commits.validate_header(header)

    def test_length_boundary(self):
        commits.validate_header("docs: " + "x" * 194)

    def test_event_json_treats_shell_metacharacters_as_data(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            effect = root / "must-not-exist"
            title = f"ci: preserve $(touch {effect}) literally"
            event = root / "event.json"
            event.write_text(json.dumps({"pull_request": {"title": title}}))
            result = subprocess.run([sys.executable, str(SCRIPT), "--event", str(event)], capture_output=True, text=True, check=False)
            self.assertEqual(result.returncode, 0, result.stderr)
            self.assertFalse(effect.exists())
            self.assertNotIn(title, result.stdout)

    def test_malformed_or_missing_event_title_fails_closed(self):
        with tempfile.TemporaryDirectory() as directory:
            event = Path(directory) / "event.json"
            for payload in ("not json", "[]", "{}", '{"pull_request":{"title":null}}'):
                event.write_text(payload)
                with self.subTest(payload=payload), self.assertRaises(ValueError):
                    commits.event_title(event)
            event.write_bytes(b" " * (commits.MAX_EVENT_BYTES + 1))
            with self.assertRaises(ValueError):
                commits.event_title(event)


if __name__ == "__main__":
    unittest.main()
