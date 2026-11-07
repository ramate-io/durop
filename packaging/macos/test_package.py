"""Artifact version and DMG naming used by native packaging."""
import importlib.util
import os
from pathlib import Path
import subprocess
import sys
import unittest

SCRIPT = Path(__file__).with_name("package.py")
spec = importlib.util.spec_from_file_location("bundle_package", SCRIPT)
packager = importlib.util.module_from_spec(spec)
spec.loader.exec_module(packager)


class ArtifactVersionTests(unittest.TestCase):
    def test_local_builds_use_the_cargo_package_version(self):
        self.assertEqual(packager.resolve_artifact_version("0.1.0"), "0.1.0")

    def test_ci_builds_append_the_short_commit(self):
        self.assertEqual(
            packager.resolve_artifact_version("0.1.0", sha="abcdef1234567890"),
            "0.1.0-abcdef1",
        )

    def test_published_release_tags_drop_a_leading_v(self):
        self.assertEqual(
            packager.resolve_artifact_version(
                "0.1.5", event_name="release", ref_name="v0.1.5-playtest-001", sha="deadbeef",
            ),
            "0.1.5-playtest-001",
        )

    def test_release_refs_and_unprefixed_tags_are_accepted(self):
        self.assertEqual(
            packager.release_tag_version("refs/tags/v0.1.0-playtest-001"),
            "0.1.0-playtest-001",
        )
        self.assertEqual(packager.release_tag_version("0.1.0-playtest-002"), "0.1.0-playtest-002")

    def test_explicit_override_wins_over_github_context(self):
        self.assertEqual(
            packager.resolve_artifact_version(
                "0.1.0", override="0.1.0-playtest-001", event_name="release",
                ref_name="v9.9.9", sha="ffffff0",
            ),
            "0.1.0-playtest-001",
        )

    def test_unsafe_artifact_versions_are_rejected(self):
        with self.assertRaisesRegex(ValueError, "safe filename"):
            packager.resolve_artifact_version("0.1.0", override="0.1.0/evil")
        with self.assertRaisesRegex(ValueError, "safe filename"):
            packager.resolve_artifact_version("0.1.0", override="../0.1.0")

    def test_dmg_name_includes_the_release_tag_not_only_the_package_version(self):
        artifact = packager.resolve_artifact_version(
            "0.1.5", event_name="release", ref_name="v0.1.5-playtest-001",
        )
        self.assertEqual(packager.distributable_name(artifact), "Durop-0.1.5-playtest-001-macos-arm64.dmg")
        self.assertNotEqual(packager.distributable_name("0.1.5"), packager.distributable_name(artifact))

    def test_print_version_without_github_matches_cargo(self):
        env = {key: value for key, value in os.environ.items() if not key.startswith("GITHUB_") and key != "VERSION"}
        output = subprocess.check_output([sys.executable, str(SCRIPT), "--print-version"], env=env, text=True)
        self.assertEqual(output.strip(), packager.cargo_package_version())

    def test_print_version_reads_github_release_context(self):
        env = {key: value for key, value in os.environ.items() if not key.startswith("GITHUB_") and key != "VERSION"}
        env.update(GITHUB_EVENT_NAME="release", GITHUB_REF_NAME="v0.1.5-playtest-001", GITHUB_SHA="abc1234")
        output = subprocess.check_output([sys.executable, str(SCRIPT), "--print-version"], env=env, text=True)
        self.assertEqual(output.strip(), "0.1.5-playtest-001")

    def test_busy_hdiutil_output_is_retried(self):
        self.assertTrue(packager.dmg_create_is_busy("hdiutil: create failed - Resource busy"))
        self.assertFalse(packager.dmg_create_is_busy("hdiutil: create failed - image not recognized"))
