import io
import json
import os
from pathlib import Path
from unittest.mock import patch
from urllib.error import HTTPError
import tempfile
import unittest

from policy import (
    DuropOverlay,
    config_document,
    fetch_live_policy,
    merge_durop,
    overlay_from_flags,
)
from settings import Settings


class FakeResponse:
    """Minimal context-managed HTTP body for urlopen patches."""

    def __init__(self, body: bytes):
        self._body = body

    def read(self):
        return self._body

    def __enter__(self):
        return self

    def __exit__(self, *args):
        return False


class PolicyTests(unittest.TestCase):
    def setUp(self):
        self.overlay = DuropOverlay(host='100.64.1.1', controllers=('host@example.com',),
                                    unauthorized='guest@example.com')

    def test_overlay_flags_require_the_complete_set(self):
        self.assertIsNone(overlay_from_flags(None, None, None, False))
        with self.assertRaisesRegex(ValueError, 'ensure requires --host'):
            overlay_from_flags('100.64.1.1', None, None, False)
        built = overlay_from_flags('100.64.1.1', ['a@b.co'], 'c@d.co', True)
        self.assertEqual(built.host, '100.64.1.1')
        self.assertTrue(built.replace_config)

    def test_merge_preserves_unrelated_rules_and_is_idempotent(self):
        live = {
            'groups': {'group:admins': ['ops@example.com']},
            'ssh': [{'action': 'check', 'src': ['autogroup:member'], 'dst': ['autogroup:self'],
                     'users': ['autogroup:nonroot']}],
            'acls': [{'action': 'accept', 'src': ['*'], 'dst': ['*:*']}],
        }
        first = merge_durop(live, self.overlay)
        second = merge_durop(first, self.overlay)
        self.assertEqual(first, second)
        self.assertEqual(first['groups']['group:admins'], ['ops@example.com'])
        self.assertEqual(first['ssh'], live['ssh'])
        self.assertEqual(first['acls'], live['acls'])
        self.assertEqual(first['hosts']['durop-host'], '100.64.1.1')
        self.assertEqual(first['groups']['group:durop-controllers'], ['host@example.com'])
        grants = {(tuple(grant['src']), tuple(grant['dst']), tuple(grant['ip'])) for grant in first['grants']}
        self.assertEqual(grants, {
            (('group:durop-controllers',), ('durop-host',), ('tcp:48732',)),
            (('autogroup:shared',), ('durop-host',), ('tcp:48732',)),
        })
        with tempfile.TemporaryDirectory() as tmp:
            Settings.parse(json.loads(config_document(first)), Path(tmp))

    def test_conflicting_host_alias_is_refused(self):
        with self.assertRaisesRegex(ValueError, 'different address'):
            merge_durop({'hosts': {'durop-host': '100.64.9.9'}}, self.overlay)

    def test_fetch_uses_oauth_then_acl_json_without_echoing_bodies(self):
        responses = [
            FakeResponse(b'{"access_token":"secret-token"}'),
            FakeResponse(b'{"groups":{}}'),
        ]

        def fake_open(request, timeout):
            self.assertEqual(timeout, 30)
            if request.full_url.endswith('/oauth/token'):
                self.assertEqual(request.get_method(), 'POST')
                return responses.pop(0)
            self.assertIn('Authorization', request.headers)
            self.assertNotIn('secret-token', request.full_url)
            self.assertEqual(request.get_method(), 'GET')
            return responses.pop(0)

        with patch.dict(os.environ, {'TAILSCALE_TAILNET': 'example.ts.net',
                                     'TAILSCALE_OAUTH_CLIENT_ID': 'id',
                                     'TAILSCALE_OAUTH_CLIENT_SECRET': 'secret'}):
            with patch('policy.urlopen', side_effect=fake_open):
                self.assertEqual(fetch_live_policy(), {'groups': {}})

    def test_fetch_failures_do_not_include_response_bodies(self):
        error = HTTPError('https://api.tailscale.com/api/v2/oauth/token', 401, 'denied', None,
                          io.BytesIO(b'super-secret'))
        with patch.dict(os.environ, {'TAILSCALE_TAILNET': 'example.ts.net',
                                     'TAILSCALE_OAUTH_CLIENT_ID': 'id',
                                     'TAILSCALE_OAUTH_CLIENT_SECRET': 'secret'}):
            with patch('policy.urlopen', side_effect=error):
                with self.assertRaisesRegex(ValueError, 'Tailscale API request failed') as raised:
                    fetch_live_policy()
        self.assertNotIn('super-secret', str(raised.exception))
        self.assertNotIn('secret', str(raised.exception))


if __name__ == '__main__':
    unittest.main()
