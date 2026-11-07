"""Pure CLI tests: dotenv is data, invocation is independent of working directory."""

import importlib.util
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import types
import unittest
from unittest.mock import patch


REPO = Path(__file__).resolve().parents[2]
SPEC = importlib.util.spec_from_file_location('durop_manage', REPO / 'infra/manage.py')
manager = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(manager)


class ManagerTests(unittest.TestCase):
    """Keep configuration parsing and credential boundaries independent of CDK tools."""

    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.base = Path(self.temp.name)
        environment = patch.dict(os.environ, {}, clear=True)
        environment.start()
        self.addCleanup(environment.stop)

    def test_dotenv_is_literal_and_existing_environment_wins(self):
        env = self.base / 'private.env'
        sentinel = self.base / 'executed'
        env.write_text(f'''# private data\nexport EXISTING=file\nSECRET="spaces # literal" # comment\nCOMMAND=$(touch {sentinel})\nBACKTICKS=`touch {sentinel}`\nVARIABLE=${{SECRET}}\nPLAIN=value # comment\n''')
        os.environ['EXISTING'] = 'environment'
        manager.load_env(env)
        self.assertEqual(os.environ['EXISTING'], 'environment')
        self.assertEqual(os.environ['SECRET'], 'spaces # literal')
        self.assertEqual(os.environ['VARIABLE'], '${SECRET}')
        self.assertEqual(os.environ['PLAIN'], 'value')
        self.assertFalse(sentinel.exists())

    def test_bad_file_is_atomic_and_does_not_echo_secrets(self):
        env = self.base / 'bad.env'
        env.write_text('FIRST=secret\nnot a secret assignment\n')
        with self.assertRaisesRegex(ValueError, r':2: expected NAME=value') as error:
            manager.load_env(env)
        self.assertNotIn('secret', str(error.exception))
        self.assertNotIn('FIRST', os.environ)

    def test_env_options_defaults_and_host_token_isolation(self):
        (self.base / '.env').write_text('TAILSCALE_TAILNET=from-root\nTAILSCALE_API_KEY=host-only\nXDG_STATE_HOME=/private/state\n')
        alternative = self.base / 'alternative.env'
        alternative.write_text('TAILSCALE_TAILNET=alternative\n')
        calls = []
        fake = types.SimpleNamespace(execute=lambda *args: calls.append((args, dict(os.environ))))
        with patch.object(manager, 'REPO', self.base), patch.dict(sys.modules, {'main': fake}), \
                patch.object(sys, 'prefix', str(manager.PROJECT / '.venv')):
            self.assertEqual(manager.main(['synth', manager.STACK]), 0)
            self.assertEqual(calls[-1][1]['TAILSCALE_TAILNET'], 'from-root')
            self.assertEqual(calls[-1][0][2], Path('/private/state/durop/tailscale'))
            self.assertNotIn('TAILSCALE_API_KEY', calls[-1][1])
            os.environ.clear()
            self.assertEqual(manager.main(['synth', manager.STACK, '--env-file', str(alternative)]), 0)
            self.assertEqual(calls[-1][1]['TAILSCALE_TAILNET'], 'alternative')
            os.environ.clear()
            self.assertEqual(manager.main(['synth', manager.STACK, '--no-env']), 0)
            self.assertNotIn('TAILSCALE_TAILNET', calls[-1][1])

    def test_arbitrary_working_directory_and_help(self):
        result = subprocess.run([sys.executable, str(REPO / 'infra/manage.py'), '--help'],
                                cwd=self.base, check=True, capture_output=True, text=True)
        self.assertIn('durop-network', result.stdout)
        self.assertIn('--no-env', result.stdout)
        self.assertIn('ensure', result.stdout)
        self.assertIn('--host', result.stdout)

    def test_overlay_flags_only_valid_with_ensure(self):
        self.assertEqual(manager.main(['synth', manager.STACK, '--no-env', '--host', '100.64.0.1']), 1)

    def test_missing_venv_runs_setup_before_reexec(self):
        project = self.base / 'tailscale'
        project.mkdir()
        runs = []

        def fake_run(argv, **kwargs):
            runs.append(argv)
            return subprocess.CompletedProcess(argv, 0)

        with patch.object(manager, 'PROJECT', project), patch.object(manager, 'setup') as setup, \
                patch('subprocess.run', side_effect=fake_run), \
                patch.object(sys, 'prefix', str(self.base / 'other-python')):
            self.assertEqual(manager.main(['status', manager.STACK, '--no-env']), 0)
        setup.assert_called_once()
        self.assertEqual(runs[0][:3], ['uv', 'run', '--project'])


if __name__ == '__main__':
    unittest.main()
