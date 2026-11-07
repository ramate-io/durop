"""Pure dotenv tests: parsing is data, independent of CDK tools and uv."""

import importlib.util
import os
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch


REPO = Path(__file__).resolve().parents[2]
SPEC = importlib.util.spec_from_file_location('durop_env', REPO / 'infra/cdktn/tailscale/env.py')
env = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(env)


class EnvTests(unittest.TestCase):
    """Keep configuration parsing and credential boundaries independent of CDK tools."""

    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.base = Path(self.temp.name)
        environment = patch.dict(os.environ, {}, clear=True)
        environment.start()
        self.addCleanup(environment.stop)

    def test_dotenv_is_literal_and_existing_environment_wins(self):
        path = self.base / 'private.env'
        sentinel = self.base / 'executed'
        path.write_text(f'''# private data\nexport EXISTING=file\nSECRET="spaces # literal" # comment\nCOMMAND=$(touch {sentinel})\nBACKTICKS=`touch {sentinel}`\nVARIABLE=${{SECRET}}\nPLAIN=value # comment\n''')
        os.environ['EXISTING'] = 'environment'
        env.load_env(path)
        self.assertEqual(os.environ['EXISTING'], 'environment')
        self.assertEqual(os.environ['SECRET'], 'spaces # literal')
        self.assertEqual(os.environ['VARIABLE'], '${SECRET}')
        self.assertEqual(os.environ['PLAIN'], 'value')
        self.assertFalse(sentinel.exists())

    def test_bad_file_is_atomic_and_does_not_echo_secrets(self):
        path = self.base / 'bad.env'
        path.write_text('FIRST=secret\nnot a secret assignment\n')
        with self.assertRaisesRegex(ValueError, r':2: expected NAME=value') as error:
            env.load_env(path)
        self.assertNotIn('secret', str(error.exception))
        self.assertNotIn('FIRST', os.environ)


if __name__ == '__main__':
    unittest.main()
