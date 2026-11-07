#!/usr/bin/env python3
"""Manage Durop's existing tailnet with locked tools and reviewed saved plans."""

from __future__ import annotations

import argparse
import os
from pathlib import Path
import re
import subprocess
import sys


# Resolve every default from this checkout, regardless of the caller's directory.
REPO = Path(__file__).resolve().parent.parent
PROJECT = REPO / 'infra/cdktn/tailscale'
STACK = 'durop-network'
COMMANDS = ('setup', 'synth', 'validate', 'import', 'plan', 'show', 'apply', 'status', 'ensure')


def load_env(path: Path, *, required: bool = False) -> None:
    """Load literal single-line assignments; inherited values always win.

    Quotes delimit values but never enable expansion, escapes or shell execution.
    Reject malformed files without exposing their potentially secret contents.
    """
    if not path.is_file() and not required:
        return
    values = {}
    for number, line in enumerate(path.read_text().splitlines(), 1):
        line = line.strip()
        if not line or line.startswith('#'):
            continue
        match = re.fullmatch(r'(?:export\s+)?([A-Za-z_][A-Za-z0-9_]*)\s*=\s*(.*)', line)
        if not match:
            raise ValueError(f'{path}:{number}: expected NAME=value')
        name, value = match.groups()
        if value.startswith(('"', "'")):
            quote = value[0]
            end = value.find(quote, 1)
            if end < 0 or (value[end + 1:].strip() and not value[end + 1:].lstrip().startswith('#')):
                raise ValueError(f'{path}:{number}: invalid quoted value')
            value = value[1:end]
        else:
            value = re.split(r'\s+#', value, maxsplit=1)[0].rstrip()
        values[name] = value
    for name, value in values.items():
        os.environ.setdefault(name, value)


def setup() -> None:
    """Install locked dependencies and regenerate the committed provider bindings."""
    subprocess.run(['uv', 'sync', '--locked', '--project', str(PROJECT)], check=True)
    subprocess.run(['npm', 'ci', '--ignore-scripts'], cwd=PROJECT, check=True)
    subprocess.run(['npm', 'exec', '--', 'cdktn', 'get'], cwd=PROJECT,
                   env={**os.environ, 'TERRAFORM_BINARY_NAME': 'tofu'}, check=True)


def main(argv: list[str] | None = None) -> int:
    """Parse one public CLI and enter the locked Python runtime when needed."""
    os.umask(0o077)
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('command', choices=COMMANDS)
    parser.add_argument('stack', choices=[STACK])
    env = parser.add_mutually_exclusive_group()
    env.add_argument('--env-file', type=Path, help='literal dotenv file; defaults to checkout root .env')
    env.add_argument('--no-env', action='store_true', help='do not load a dotenv file')
    parser.add_argument('--config', type=Path, default=PROJECT / 'network.config.json')
    parser.add_argument('--state-dir', type=Path, help='private persistent backend outside the checkout')
    parser.add_argument('--output', type=Path, default=PROJECT / 'cdktf.out')
    parser.add_argument('--host', help='playback Mac Tailscale IP or prefix for hosts.durop-host')
    parser.add_argument('--controller', action='append', help='repeatable email for group:durop-controllers')
    parser.add_argument('--unauthorized', help='email that policy tests must deny on TCP 48732')
    parser.add_argument('--replace-config', action='store_true',
                        help='overwrite network.config.json when ensure merges the live ACL')
    args = parser.parse_args(argv)
    try:
        if not args.no_env:
            load_env(args.env_file.expanduser().resolve() if args.env_file else REPO / '.env',
                     required=args.env_file is not None)
        # The host's invitation token must never become provisioner authentication.
        os.environ.pop('TAILSCALE_API_KEY', None)
        if args.command != 'ensure' and (args.host or args.controller or args.unauthorized or args.replace_config):
            raise ValueError('host/controller flags are only valid with ensure')
        if args.command == 'setup':
            setup()
        elif Path(sys.prefix).resolve() != (PROJECT / '.venv').resolve():
            if os.environ.get('_DUROP_INFRA_RUNTIME'):
                raise ValueError('locked runtime missing; run setup durop-network first')
            if not (PROJECT / '.venv' / 'bin' / 'python').is_file():
                setup()
            arguments = sys.argv[1:] if argv is None else argv
            return subprocess.run(['uv', 'run', '--project', str(PROJECT), '--locked', '--no-sync',
                                   'python', str(Path(__file__).resolve()), *arguments],
                                  env={**os.environ, '_DUROP_INFRA_RUNTIME': '1'}, check=True).returncode
        else:
            sys.path.insert(0, str(PROJECT))
            from main import execute
            from policy import overlay_from_flags
            state = args.state_dir or Path(os.environ.get('XDG_STATE_HOME', str(Path.home() / '.local/state'))) / 'durop/tailscale'
            if args.command == 'ensure':
                execute(args.command, args.config, state, args.output,
                        overlay=overlay_from_flags(args.host, args.controller, args.unauthorized,
                                                   args.replace_config))
            else:
                execute(args.command, args.config, state, args.output)
    except (ValueError, OSError, subprocess.CalledProcessError) as error:
        print(f'{args.command} failed: {error}', file=sys.stderr)
        return 1
    return 0


if __name__ == '__main__':
    raise SystemExit(main())
