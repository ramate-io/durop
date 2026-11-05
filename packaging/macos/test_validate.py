"""Native loader regression: inspect the complete bundle, including hidden dylibs."""
import importlib.util
from pathlib import Path
import platform
import shutil
import subprocess
import tempfile
import unittest

spec = importlib.util.spec_from_file_location("bundle_validate", Path(__file__).with_name("validate.py"))
validator = importlib.util.module_from_spec(spec)
spec.loader.exec_module(validator)


class ParserTests(unittest.TestCase):
    def test_loader_environment_and_path_escape_are_rejected(self):
        with self.assertRaisesRegex(ValueError, "environment"):
            validator.load_commands("cmd LC_DYLD_ENVIRONMENT\nname DYLD_LIBRARY_PATH=/tmp (offset 12)")
        self.assertFalse(validator.is_system("/System/Library/../../etc/forbidden.dylib"))
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory).resolve()
            executable = root / "Contents/MacOS/durop"
            with self.assertRaisesRegex(ValueError, "escapes"):
                validator.expand("@executable_path/../../../outside", executable, executable, root)


@unittest.skipUnless(platform.system() == "Darwin" and shutil.which("clang"), "requires native Apple linker")
class NativeLoaderTests(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory(prefix="durop-validator-")
        self.root = Path(self.temporary.name)
        self.app = self.root / "Durop.app"
        self.frameworks = self.app / "Contents/Frameworks"
        self.frameworks.mkdir(parents=True)
        self.binary = self.app / "Contents/MacOS/durop"
        self.binary.parent.mkdir()
        self.source = self.root / "library.c"
        self.source.write_text("int answer(void) { return 42; }\n")
        self.library = self.frameworks / "libanswer.dylib"
        self.compile(["-dynamiclib", self.source, "-Wl,-install_name,@rpath/libanswer.dylib", "-o", self.library])
        main = self.root / "main.c"
        main.write_text("int answer(void); int main(void) { return answer() == 42 ? 0 : 1; }\n")
        self.compile([main, "-L" + str(self.frameworks), "-lanswer", "-Wl,-rpath,@executable_path/../Frameworks", "-o", self.binary])

    def tearDown(self):
        self.temporary.cleanup()

    def compile(self, arguments):
        subprocess.run(["/usr/bin/clang", "-arch", "arm64", "-mmacosx-version-min=14.0"] + list(map(str, arguments)), check=True, capture_output=True)

    def test_bundled_dylib_resolves_without_developer_install_name(self):
        report = validator.audit_machos(self.app, "14.0")
        self.assertEqual(report["external_non_system_dependencies"], 0)
        self.assertEqual(len(report["mach_o_files"]), 2)
        self.library.unlink()
        with self.assertRaisesRegex(ValueError, "unresolved bundled"):
            validator.audit_machos(self.app, "14.0")

    def test_unreferenced_plugin_is_audited_too(self):
        hidden = self.app / "Contents/Resources/plugins/hidden.dylib"
        hidden.parent.mkdir(parents=True)
        self.compile(["-dynamiclib", self.source, "-Wl,-install_name,/nix/store/forbidden/libhidden.dylib", "-o", hidden])
        with self.assertRaisesRegex(ValueError, "developer path"):
            validator.audit_machos(self.app, "14.0")


if __name__ == "__main__":
    unittest.main()
