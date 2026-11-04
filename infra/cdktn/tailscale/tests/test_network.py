import json
from pathlib import Path
import tempfile
import unittest

from network import synthesize
from settings import Settings

ROOT = Path(__file__).resolve().parents[1]


class NetworkTests(unittest.TestCase):
    def test_deterministic_complete_policy_and_backend(self):
        with tempfile.TemporaryDirectory() as tmp:
            base = Path(tmp)
            data = json.loads((ROOT / 'network.config.example.json').read_text())
            data['policy']['ssh'] = [{'action': 'check', 'src': ['autogroup:member'], 'dst': ['autogroup:self'], 'users': ['autogroup:nonroot']}]
            config = base / 'config.json'
            config.write_text(json.dumps(data))
            settings = Settings.load(config, base / 'persistent')
            synthesize(settings, str(base / 'out'))
            generated = base / 'out/stacks/durop-network/cdk.tf.json'
            first = generated.read_bytes()
            synthesize(settings, str(base / 'out'))
            self.assertEqual(first, generated.read_bytes())
            result = json.loads(first)
            self.assertEqual(json.loads(result['resource']['tailscale_acl']['policy']['acl']), data['policy'])
            self.assertEqual(result['terraform']['backend']['local']['path'], str(base / 'persistent/terraform.tfstate'))
            self.assertEqual(result['terraform']['required_providers']['tailscale']['version'], '0.29.2')
            self.assertIn('tailscale', result['provider'])
            self.assertNotIn('tailscale_dns_preferences', result['resource'])
            self.assertFalse((base / 'persistent').exists())

    def test_invalid_root(self):
        with tempfile.TemporaryDirectory() as tmp:
            config = Path(tmp) / 'config.json'
            for data in [None, [], 'bad', 1, {'policy': []}]:
                config.write_text(json.dumps(data))
                with self.assertRaises(ValueError):
                    Settings.load(config, Path(tmp) / 'state')

    def test_invalid_settings(self):
        with tempfile.TemporaryDirectory() as tmp:
            config = Path(tmp) / 'config.json'
            for policy in [{'groups': []}, {'hosts': []}, {'grants': {}}, {'grants': [None]}, {'grants': [{'ip': 'tcp:12'}]}, {'hosts': {'host': 'not-an-address'}}, {'groups': {'group:durop': ['bad identity']}}, {'grants': [{'src': ['*'], 'dst': ['host'], 'ip': ['tcp:99999']}]}]:
                with self.subTest(policy=policy):
                    config.write_text(json.dumps({'policy': policy}))
                    with self.assertRaises(ValueError):
                        Settings.load(config, Path(tmp) / 'state')


if __name__ == '__main__':
    unittest.main()
