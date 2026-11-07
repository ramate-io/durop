"""Fetch the live tailnet ACL and merge Durop's host/port grants into a complete document."""
from __future__ import annotations

from copy import deepcopy
from dataclasses import dataclass
import json
import os
from typing import Any
from urllib.error import HTTPError, URLError
from urllib.parse import quote, urlencode
from urllib.request import Request, urlopen

HOST_ALIAS = 'durop-host'
CONTROLLER_GROUP = 'group:durop-controllers'
PORT = 'tcp:48732'
ACCEPT = f'{HOST_ALIAS}:48732'
DEFAULT_API = 'https://api.tailscale.com'


@dataclass(frozen=True)
class DuropOverlay:
    """Operator-supplied identities for the Durop host, controllers and deny test."""

    host: str
    controllers: tuple[str, ...]
    unauthorized: str
    replace_config: bool = False


def overlay_from_flags(
    host: str | None,
    controllers: list[str] | None,
    unauthorized: str | None,
    replace_config: bool,
) -> DuropOverlay | None:
    """Build an overlay when any merge flag is present; require the complete set."""
    controllers = list(controllers or [])
    if not host and not controllers and not unauthorized and not replace_config:
        return None
    if not host or not controllers or not unauthorized:
        raise ValueError('ensure requires --host, --controller and --unauthorized to merge Durop grants')
    return DuropOverlay(host=host, controllers=tuple(controllers),
                        unauthorized=unauthorized, replace_config=replace_config)


def fetch_live_policy() -> dict[str, Any]:
    """Download the tailnet ACL as ordinary JSON using the provisioner OAuth credential."""
    tailnet = os.environ.get('TAILSCALE_TAILNET', '').strip()
    client_id = os.environ.get('TAILSCALE_OAUTH_CLIENT_ID', '').strip()
    client_secret = os.environ.get('TAILSCALE_OAUTH_CLIENT_SECRET', '').strip()
    if not tailnet or not client_id or not client_secret:
        raise ValueError('ensure requires explicit TAILSCALE_TAILNET, TAILSCALE_OAUTH_CLIENT_ID and TAILSCALE_OAUTH_CLIENT_SECRET')
    origin = os.environ.get('TAILSCALE_BASE_URL', DEFAULT_API).strip().rstrip('/') or DEFAULT_API
    token = _access_token(origin, client_id, client_secret)
    payload = _api_json(f'{origin}/api/v2/tailnet/{quote(tailnet, safe="")}/acl',
                        {'Authorization': f'Bearer {token}', 'Accept': 'application/json'})
    if isinstance(payload, str):
        payload = json.loads(payload)
    if not isinstance(payload, dict):
        raise ValueError('Tailscale ACL response must be a JSON object')
    return payload


def merge_durop(policy: dict[str, Any], overlay: DuropOverlay) -> dict[str, Any]:
    """Return a complete policy with Durop grants inserted; preserve unrelated sections.

    Existing `durop-host` or grants are left in place when they already match.
    A conflicting host address is refused rather than overwritten.
    """
    merged = deepcopy(policy)
    groups = dict(merged.get('groups') or {})
    hosts = dict(merged.get('hosts') or {})
    grants = list(merged.get('grants') or [])
    tests = list(merged.get('tests') or [])
    current = hosts.get(HOST_ALIAS)
    if current is not None and current != overlay.host:
        raise ValueError('hosts.durop-host already maps to a different address')
    hosts[HOST_ALIAS] = overlay.host
    members = list(groups.get(CONTROLLER_GROUP) or [])
    for email in overlay.controllers:
        if email not in members:
            members.append(email)
    groups[CONTROLLER_GROUP] = members
    for source in ([CONTROLLER_GROUP], ['autogroup:shared']):
        grant = {'src': source, 'dst': [HOST_ALIAS], 'ip': [PORT]}
        if not any(_same_grant(existing, grant) for existing in grants):
            grants.append(grant)
    for email in overlay.controllers:
        test = {'src': email, 'accept': [ACCEPT]}
        if not any(_same_mapping(existing, test) for existing in tests):
            tests.append(test)
    deny = {'src': overlay.unauthorized, 'deny': [ACCEPT]}
    if not any(_same_mapping(existing, deny) for existing in tests):
        tests.append(deny)
    merged['groups'] = groups
    merged['hosts'] = hosts
    merged['grants'] = grants
    merged['tests'] = tests
    return merged


def config_document(policy: dict[str, Any]) -> str:
    """Serialize owned configuration without taking MagicDNS unless the operator adds it."""
    return json.dumps({'policy': policy}, sort_keys=True, indent=2) + '\n'


def _same_grant(left: Any, right: dict[str, Any]) -> bool:
    # Exact src/dst/ip match so a broader existing grant does not hide a missing Durop rule.
    return isinstance(left, dict) and left.get('src') == right['src'] and left.get('dst') == right['dst'] and left.get('ip') == right['ip']


def _same_mapping(left: Any, right: dict[str, Any]) -> bool:
    return isinstance(left, dict) and all(left.get(key) == value for key, value in right.items())


def _access_token(origin: str, client_id: str, client_secret: str) -> str:
    payload = _api_json(f'{origin}/api/v2/oauth/token',
                        {'Content-Type': 'application/x-www-form-urlencoded'},
                        urlencode({'client_id': client_id, 'client_secret': client_secret,
                                   'grant_type': 'client_credentials'}).encode())
    token = payload.get('access_token') if isinstance(payload, dict) else None
    if not isinstance(token, str) or not token:
        raise ValueError('Tailscale OAuth response did not include an access token')
    return token


def _api_json(url: str, headers: dict[str, str], data: bytes | None = None) -> Any:
    """POST or GET JSON without echoing response bodies that may contain credentials."""
    request = Request(url, data=data, headers=headers, method='POST' if data is not None else 'GET')
    try:
        with urlopen(request, timeout=30) as response:
            body = response.read()
    except HTTPError:
        raise ValueError('Tailscale API request failed') from None
    except URLError:
        raise ValueError('Tailscale API is unreachable') from None
    try:
        return json.loads(body)
    except json.JSONDecodeError:
        raise ValueError('Tailscale API returned non-JSON; export requires Accept: application/json') from None
