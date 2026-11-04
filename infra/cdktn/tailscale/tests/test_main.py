import json
from pathlib import Path
import shutil
import subprocess
import tempfile
import unittest
from unittest.mock import patch

import main


class WrapperTests(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.addCleanup(self.tmp.cleanup)
        # macOS /tmp aliases /private/tmp; production canonicalizes paths.
        self.base = Path(self.tmp.name).resolve()
        self.config = self.base / 'config.json'
        shutil.copyfile(main.ROOT / 'network.config.example.json', self.config)
        self.state = self.base / 'persistent state'
        self.out = self.base / 'generated output'
        self.calls = []
        target = patch.dict('os.environ', {'TAILSCALE_TAILNET': 'test-tailnet'})
        target.start()
        self.addCleanup(target.stop)

    def fake_run(self, argv, *, check):
        self.assertTrue(check)
        self.calls.append(argv)
        if 'plan' in argv:
            Path(next(x.removeprefix('-out=') for x in argv if x.startswith('-out='))).write_bytes(b'review me')

    def execute(self, command):
        with patch('main.subprocess.run', side_effect=self.fake_run):
            main.execute(command, self.config, self.state, self.out)

    def test_argument_vectors_and_durability(self):
        self.execute('plan')
        prefix = ['tofu', f'-chdir={self.out}/stacks/durop-network']
        self.assertEqual(self.calls, [prefix + ['init', '-input=false', '-lockfile=readonly'], prefix + ['validate'], prefix + ['plan', '-input=false', f'-out={self.state}/reviewed.tfplan']])
        self.assertEqual((self.out / 'stacks/durop-network/.terraform.lock.hcl').read_bytes(), (main.ROOT / '.terraform.lock.hcl').read_bytes())
        (self.state / 'terraform.tfstate').write_text('durable')
        shutil.rmtree(self.out)
        self.assertTrue((self.state / 'reviewed.tfplan').exists())
        self.execute('apply')
        self.assertEqual(self.calls[-1], prefix + ['apply', '-input=false', str(self.state / 'reviewed.tfplan')])
        self.assertEqual((self.state / 'terraform.tfstate').read_text(), 'durable')
        self.assertFalse((self.state / 'reviewed.json').exists())

    def test_missing_plan_runs_no_commands(self):
        with self.assertRaisesRegex(ValueError, 'no reviewed plan'):
            self.execute('apply')
        self.assertEqual(self.calls, [])

    def test_changed_config_plan_and_tailnet_refused(self):
        for kind in ['config', 'plan', 'tailnet']:
            with self.subTest(kind=kind):
                self.execute('plan')
                self.calls.clear()
                if kind == 'config':
                    self.config.write_text(self.config.read_text() + '\n')
                elif kind == 'plan':
                    (self.state / 'reviewed.tfplan').write_bytes(b'tampered')
                with patch.dict('os.environ', {'TAILSCALE_TAILNET': 'changed'} if kind == 'tailnet' else {}):
                    with self.assertRaisesRegex(ValueError, 'stale or modified plan'):
                        self.execute('apply')
                self.assertEqual(self.calls, [])

    def test_source_and_provider_lock_fingerprints(self):
        self.execute('plan')
        self.calls.clear()
        original = main.digest
        for changed in ['network.py', '.terraform.lock.hcl']:
            def modified(path):
                return 'changed' if path == main.ROOT / changed else original(path)
            with patch('main.digest', side_effect=modified):
                with self.assertRaisesRegex(ValueError, 'stale or modified plan'):
                    self.execute('apply')
        self.assertEqual(self.calls, [])

    def test_failed_apply_consumes_review(self):
        self.execute('plan')
        def failing_apply(argv, *, check):
            if 'apply' in argv:
                raise subprocess.CalledProcessError(1, argv)
        with patch('main.subprocess.run', side_effect=failing_apply):
            with self.assertRaises(subprocess.CalledProcessError):
                main.execute('apply', self.config, self.state, self.out)
        self.assertFalse((self.state / 'reviewed.json').exists())
        with self.assertRaisesRegex(ValueError, 'no reviewed plan'):
            self.execute('apply')

    def test_subprocess_failure_invalidates_plan(self):
        self.execute('plan')
        with patch('main.subprocess.run', side_effect=subprocess.CalledProcessError(1, ['tofu'])):
            with self.assertRaises(subprocess.CalledProcessError):
                main.execute('plan', self.config, self.state, self.out)
        self.assertFalse((self.state / 'reviewed.json').exists())
        with self.assertRaisesRegex(ValueError, 'no reviewed plan'):
            self.execute('apply')

    def test_reject_state_in_checkout_or_generated_output(self):
        for state in [main.ROOT / 'state', self.out / 'state']:
            with self.assertRaisesRegex(ValueError, 'outside'):
                main.execute('synth', self.config, state, self.out)


if __name__ == '__main__':
    unittest.main()
