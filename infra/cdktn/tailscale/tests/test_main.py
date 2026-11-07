import json
from pathlib import Path
import shutil
import subprocess
import tempfile
import unittest
from unittest.mock import patch

import main
import importlib.util


# Exercise the public manager against the actual saved-plan lifecycle implementation.
manager_spec = importlib.util.spec_from_file_location('durop_manage', main.ROOT.parents[1] / 'manage.py')
manager = importlib.util.module_from_spec(manager_spec)
manager_spec.loader.exec_module(manager)


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
        target = patch.dict('os.environ', {'TAILSCALE_TAILNET': 'test-tailnet',
                            'TAILSCALE_OAUTH_CLIENT_ID': 'test-client',
                            'TAILSCALE_OAUTH_CLIENT_SECRET': 'test-secret'})
        target.start()
        self.addCleanup(target.stop)

    def fake_run(self, argv, *, check, **kwargs):
        self.assertTrue(check)
        self.calls.append(argv)
        if 'plan' in argv:
            Path(next(x.removeprefix('-out=') for x in argv if x.startswith('-out='))).write_bytes(b'review me')
        return subprocess.CompletedProcess(argv, 0, stdout='tailscale_acl.policy\n')

    def execute(self, command):
        with patch('main.subprocess.run', side_effect=self.fake_run):
            main.execute(command, self.config, self.state, self.out)

    def execute_overlay(self, overlay):
        with patch('main.subprocess.run', side_effect=self.fake_run):
            main.execute('ensure', self.config, self.state, self.out, overlay=overlay)

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
        for changed in [main.ROOT / 'network.py', main.ROOT / '.terraform.lock.hcl',
                        main.ROOT.parents[1] / 'manage.py']:
            def modified(path):
                return 'changed' if path == changed else original(path)
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

    def test_public_manager_review_and_import_lifecycle(self):
        arguments = ['durop-network', '--no-env', '--config', str(self.config),
                     '--state-dir', str(self.state), '--output', str(self.out)]
        with patch('main.subprocess.run', side_effect=self.fake_run):
            self.assertEqual(manager.main(['plan', *arguments]), 0)
            self.assertEqual(manager.main(['show', *arguments]), 0)
            self.assertEqual(self.calls[-1][-2:], ['show', str(self.state / 'reviewed.tfplan')])
            self.assertEqual(manager.main(['apply', *arguments]), 0)
            self.calls.clear()
            self.assertEqual(manager.main(['apply', *arguments]), 1)
            self.assertEqual(self.calls, [])

    def test_import_skips_owned_policy_and_imports_configured_dns(self):
        self.execute('plan')
        self.calls.clear()
        (self.state / 'terraform.tfstate').write_text('{}')
        data = json.loads(self.config.read_text())
        data['magic_dns'] = True
        self.config.write_text(json.dumps(data))
        self.execute('import')
        imports = [call for call in self.calls if 'import' in call]
        self.assertEqual([call[-2:] for call in imports],
                         [['tailscale_dns_preferences.dns', 'dns_preferences']])
        self.assertFalse((self.state / 'reviewed.json').exists())
        self.assertFalse((self.state / 'reviewed.tfplan').exists())

    def test_fresh_import_only_adopts_policy_when_dns_unowned(self):
        self.execute('import')
        imports = [call for call in self.calls if 'import' in call]
        self.assertEqual([call[-2:] for call in imports], [['tailscale_acl.policy', 'acl']])

    def test_show_refuses_modified_plan_without_subprocess(self):
        self.execute('plan')
        self.calls.clear()
        (self.state / 'reviewed.tfplan').write_bytes(b'changed')
        with self.assertRaisesRegex(ValueError, 'stale or modified plan'):
            self.execute('show')
        self.assertEqual(self.calls, [])

    def test_live_commands_require_oauth_provisioner(self):
        with patch.dict('os.environ', {'TAILSCALE_OAUTH_CLIENT_SECRET': ''}):
            for command in ('import', 'plan', 'apply', 'ensure'):
                with self.assertRaisesRegex(ValueError, 'TAILSCALE_OAUTH_CLIENT_SECRET'):
                    self.execute(command)
        self.assertEqual(self.calls, [])

    def test_ensure_merges_live_acl_then_imports_plans_and_shows(self):
        overlay = main.DuropOverlay(host='100.64.1.1', controllers=('host@example.com',),
                                    unauthorized='guest@example.com')
        live = {'ssh': [{'action': 'check', 'src': ['autogroup:member'], 'dst': ['autogroup:self'],
                         'users': ['autogroup:nonroot']}]}
        self.config.unlink()
        with patch('main.fetch_live_policy', return_value=live):
            self.execute_overlay(overlay)
        data = json.loads(self.config.read_text())
        self.assertEqual(data['policy']['hosts']['durop-host'], '100.64.1.1')
        self.assertEqual(data['policy']['ssh'], live['ssh'])
        self.assertNotIn('magic_dns', data)
        self.assertEqual(json.loads((self.state / 'fetched-acl.json').read_text()), live)
        prefix = ['tofu', f'-chdir={self.out}/stacks/durop-network']
        self.assertEqual(self.calls[0], prefix + ['init', '-input=false', '-lockfile=readonly'])
        self.assertEqual(self.calls[1], prefix + ['validate'])
        imports = [call for call in self.calls if 'import' in call]
        self.assertEqual([call[-2:] for call in imports], [['tailscale_acl.policy', 'acl']])
        self.assertEqual(self.calls[-2], prefix + ['plan', '-input=false', f'-out={self.state}/reviewed.tfplan'])
        self.assertEqual(self.calls[-1], prefix + ['show', str(self.state / 'reviewed.tfplan')])
        self.assertTrue((self.state / 'reviewed.json').exists())

    def test_ensure_without_overlay_uses_existing_config(self):
        with patch('main.fetch_live_policy', side_effect=AssertionError('must not fetch')):
            self.execute('ensure')
        imports = [call for call in self.calls if 'import' in call]
        self.assertEqual([call[-2:] for call in imports], [['tailscale_acl.policy', 'acl']])
        self.assertEqual(self.calls[-1][-2:], ['show', str(self.state / 'reviewed.tfplan')])

    def test_ensure_refuses_to_clobber_existing_config(self):
        overlay = main.DuropOverlay(host='100.64.1.1', controllers=('host@example.com',),
                                    unauthorized='guest@example.com')
        original = self.config.read_text()
        with patch('main.fetch_live_policy', return_value={}):
            with self.assertRaisesRegex(ValueError, 'replace-config'):
                self.execute_overlay(overlay)
        self.assertEqual(self.config.read_text(), original)
        self.assertEqual(self.calls, [])

    def test_ensure_replace_config_overwrites_local_file(self):
        overlay = main.DuropOverlay(host='100.64.1.1', controllers=('host@example.com',),
                                    unauthorized='guest@example.com', replace_config=True)
        with patch('main.fetch_live_policy', return_value={}):
            self.execute_overlay(overlay)
        self.assertEqual(json.loads(self.config.read_text())['policy']['hosts']['durop-host'], '100.64.1.1')

    def test_invalid_config_invalidates_previous_plan(self):
        self.execute('plan')
        self.calls.clear()
        self.config.write_text('not JSON')
        with self.assertRaises(ValueError):
            self.execute('plan')
        self.assertFalse((self.state / 'reviewed.json').exists())
        self.assertFalse((self.state / 'reviewed.tfplan').exists())
        self.assertEqual(self.calls, [])


if __name__ == '__main__':
    unittest.main()
