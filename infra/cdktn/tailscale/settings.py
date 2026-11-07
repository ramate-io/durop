"""Validated public settings; the policy is a complete document, never a fragment."""
from dataclasses import dataclass
import ipaddress
import json
from pathlib import Path
import re
from typing import Any


@dataclass(frozen=True)
class Settings:
    policy: dict[str, Any]
    state_dir: Path
    magic_dns: bool | None = None

    @classmethod
    def load(cls, path: Path, state_dir: Path) -> 'Settings':
        """Read and validate a complete configuration file."""
        return cls.parse(json.loads(path.read_text()), state_dir)

    @classmethod
    def parse(cls, data: Any, state_dir: Path) -> 'Settings':
        """Validate a complete configuration object already loaded as JSON."""
        if not isinstance(data, dict):
            raise ValueError('configuration must be an object')
        if set(data) - {'policy', 'magic_dns'} or not isinstance(data.get('policy'), dict):
            raise ValueError('configuration requires a complete policy object and optional magic_dns')
        policy = data['policy']
        if not isinstance(policy.get('groups', {}), dict) or not isinstance(policy.get('hosts', {}), dict):
            raise ValueError('groups and hosts must be objects')
        if not isinstance(policy.get('grants', []), list):
            raise ValueError('grants must be an array')
        for group, members in policy.get('groups', {}).items():
            if not re.fullmatch(r'group:[a-zA-Z0-9_-]+', group) or not isinstance(members, list):
                raise ValueError('invalid group')
            for member in members:
                if not isinstance(member, str) or not re.fullmatch(r'[^\s@]+@[^\s@]+\.[^\s@]+', member):
                    raise ValueError('group members must be email identities')
        for host, address in policy.get('hosts', {}).items():
            if not re.fullmatch(r'[a-zA-Z0-9_-]+', host):
                raise ValueError('invalid host alias')
            if not isinstance(address, str):
                raise ValueError('host address must be a string')
            ipaddress.ip_network(address, strict=False)
        def selector(value: str) -> bool:
            if value == '*' or value in policy.get('hosts', {}):
                return True
            if re.fullmatch(r'(group|tag|autogroup|ipset):[a-zA-Z0-9_.@*/-]+', value):
                return True
            if re.fullmatch(r'[^\s@]+@[^\s@]+\.[^\s@]+', value):
                return True
            try:
                ipaddress.ip_network(value, strict=False)
                return True
            except ValueError:
                return False

        for grant in policy.get('grants', []):
            if not isinstance(grant, dict) or not isinstance(grant.get('ip', []), list):
                raise ValueError('grant must be an object with an ip array when present')
            for field in ('src', 'dst'):
                if not isinstance(grant.get(field), list) or not grant[field] or not all(isinstance(x, str) and selector(x) for x in grant[field]):
                    raise ValueError(f'grant requires nonempty {field} selectors')
            for port in grant.get('ip', []):
                if not isinstance(port, str):
                    raise ValueError('protocol/port must be a string')
                if port == '*':
                    continue
                match = re.fullmatch(r'(tcp|udp|sctp|icmp|icmpv6|igmp|[0-9]+):(\*|\d+(?:-\d+)?)', port)
                if not match:
                    raise ValueError('invalid protocol/port')
                if match[2] != '*':
                    values = [int(x) for x in match[2].split('-')]
                    if not all(1 <= x <= 65535 for x in values) or values != sorted(values):
                        raise ValueError('port out of range')
        dns = data.get('magic_dns')
        if dns is not None and not isinstance(dns, bool):
            raise ValueError('magic_dns must be a boolean when explicitly owned')
        state_dir = state_dir.expanduser().resolve()
        return cls(policy=policy, state_dir=state_dir, magic_dns=dns)
