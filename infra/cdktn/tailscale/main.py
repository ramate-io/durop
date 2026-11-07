"""Durop tailnet CLI. uv run --locked --no-sync python main.py <command>."""
from __future__ import annotations

import argparse
from contextlib import contextmanager
import fcntl
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys

from env import load_env
from network import synthesize
from policy import DuropOverlay, config_document, fetch_live_policy, merge_durop, overlay_from_flags
from settings import Settings

ROOT = Path(__file__).resolve().parent
REPO = ROOT.parents[2]
STACK = 'durop-network'
COMMANDS = ('setup', 'synth', 'validate', 'import', 'plan', 'show', 'apply', 'status', 'ensure')


def digest(path: Path) -> str:
    """Hash private inputs without including their contents in metadata."""
    return hashlib.sha256(path.read_bytes()).hexdigest()


def tofu(stack_dir: Path, *args: str) -> None:
    """Run OpenTofu directly with an argument vector and the selected backend."""
    subprocess.run(['tofu', f'-chdir={stack_dir}', *args], check=True)


def fingerprint(config: Path, state_dir: Path, stack_dir: Path) -> str:
    """Bind a saved plan to all authored, generated, locked and target inputs."""
    paths = [ROOT / n for n in ('main.py', 'network.py', 'settings.py', 'policy.py', 'env.py', 'pyproject.toml',
             'uv.lock', 'package.json', 'package-lock.json', 'cdktf.json', '.terraform.lock.hcl', '.python-version', '.node-version')]
    paths += sorted((ROOT / 'imports').rglob('*.py')) + sorted((ROOT / 'imports').rglob('*.tgz'))
    paths += sorted((ROOT / 'imports').glob('*.json'))
    values = {str(p.relative_to(ROOT)): digest(p) for p in paths}
    values.update({name: digest(REPO / name) for name in ('flake.nix', 'flake.lock')})
    values.update(config=digest(config), state=str(state_dir),
                  generated=digest(stack_dir / 'cdk.tf.json'),
                  provider_target=hashlib.sha256(json.dumps({name: os.environ.get(name, '') for name in ('TAILSCALE_TAILNET', 'TAILSCALE_BASE_URL', 'TAILSCALE_OAUTH_CLIENT_ID')}, sort_keys=True).encode()).hexdigest())
    return hashlib.sha256(json.dumps(values, sort_keys=True).encode()).hexdigest()


@contextmanager
def operator_lock(state_dir: Path):
    """Serialize operations on a private local backend, including imports and reads."""
    state_dir.mkdir(parents=True, exist_ok=True, mode=0o700)
    if state_dir.stat().st_mode & 0o077:
        raise ValueError('state directory must be private (chmod 700)')
    with (state_dir / 'operator.lock').open('a') as handle:
        fcntl.flock(handle, fcntl.LOCK_EX | fcntl.LOCK_NB)
        yield


def setup() -> None:
    """Install locked dependencies and regenerate the committed provider bindings."""
    subprocess.run(['uv', 'sync', '--locked', '--project', str(ROOT)], check=True)
    subprocess.run(['npm', 'ci', '--ignore-scripts'], cwd=ROOT, check=True)
    subprocess.run(['npm', 'exec', '--', 'cdktn', 'get'], cwd=ROOT,
                   env={**os.environ, 'TERRAFORM_BINARY_NAME': 'tofu'}, check=True)


def execute(command: str, config: Path, state_dir: Path, output: Path,
            overlay: DuropOverlay | None = None) -> None:
    """Execute a single stack lifecycle step without replacing reviewed apply inputs."""
    state_dir, output, config = state_dir.expanduser().resolve(), output.resolve(), config.resolve()
    if state_dir == REPO or REPO in state_dir.parents or state_dir == output or output in state_dir.parents:
        raise ValueError('state/plans must live outside the checkout and generated output')
    if command not in {'synth', 'validate', 'import', 'plan', 'show', 'apply', 'status', 'ensure'}:
        raise ValueError('unknown lifecycle command')
    if overlay is not None and command != 'ensure':
        raise ValueError('host/member flags are only valid with ensure')
    if command in {'import', 'plan', 'apply', 'ensure'}:
        for name in ('TAILSCALE_TAILNET', 'TAILSCALE_OAUTH_CLIENT_ID', 'TAILSCALE_OAUTH_CLIENT_SECRET'):
            if not os.environ.get(name, '').strip():
                raise ValueError(f'{command} requires explicit {name}')
    if command == 'synth':
        settings = Settings.load(config, state_dir)
        synthesize(settings, str(output))
        return
    with operator_lock(state_dir):
        plan = state_dir / 'reviewed.tfplan'
        metadata = state_dir / 'reviewed.json'
        if command in {'plan', 'import', 'ensure'}:
            # Invalidate before config parsing/synthesis: failures also require fresh review.
            metadata.unlink(missing_ok=True)
            plan.unlink(missing_ok=True)
        # Missing plan is refused before synthesis or any OpenTofu subprocess.
        if command in {'apply', 'show'} and (not plan.is_file() or not metadata.is_file()):
            raise ValueError('no reviewed plan; run plan and review it first')
        if command == 'ensure':
            _write_ensured_config(config, state_dir, overlay)
        settings = Settings.load(config, state_dir)
        synthesize(settings, str(output))
        stack_dir = output / 'stacks' / STACK
        shutil.copyfile(ROOT / '.terraform.lock.hcl', stack_dir / '.terraform.lock.hcl')
        current = fingerprint(config, state_dir, stack_dir)
        if command in {'apply', 'show'}:
            recorded = json.loads(metadata.read_text())
            if recorded != {'fingerprint': current, 'plan_sha256': digest(plan)}:
                raise ValueError('stale or modified plan; create and review a new plan')
        tofu(stack_dir, 'init', '-input=false', '-lockfile=readonly')
        tofu(stack_dir, 'validate')
        if command in {'import', 'ensure'}:
            _import_unowned(stack_dir, state_dir, settings)
        if command == 'status':
            print(f'Stack: {STACK}\nState: {state_dir / "terraform.tfstate"}')
            if (state_dir / 'terraform.tfstate').is_file():
                tofu(stack_dir, 'state', 'list')
            else:
                print('No resources imported into this backend.')
            print(f'Saved plan: {"present; run show to verify" if metadata.is_file() and plan.is_file() else "none"}')
        elif command == 'show':
            tofu(stack_dir, 'show', str(plan))
        elif command in {'plan', 'ensure'}:
            tofu(stack_dir, 'plan', '-input=false', f'-out={plan}')
            metadata.write_text(json.dumps({'fingerprint': current, 'plan_sha256': digest(plan)}, sort_keys=True) + '\n')
            if command == 'ensure':
                tofu(stack_dir, 'show', str(plan))
        elif command == 'apply':
            # Invalidate before apply: interrupted/failed applies also require a fresh review.
            metadata.unlink()
            tofu(stack_dir, 'apply', '-input=false', str(plan))
            plan.unlink()


def _write_ensured_config(config: Path, state_dir: Path, overlay: DuropOverlay | None) -> None:
    """Materialize complete desired state from the live ACL when merging, else require a file."""
    if overlay is None:
        if not config.is_file():
            raise ValueError('ensure requires --host, or an existing configuration')
        return
    live = fetch_live_policy()
    (state_dir / 'fetched-acl.json').write_text(json.dumps(live, sort_keys=True, indent=2) + '\n')
    desired = config_document(merge_durop(live, overlay))
    Settings.parse(json.loads(desired), state_dir)
    if config.is_file() and config.read_text() != desired and not overlay.replace_config:
        raise ValueError('configuration exists and differs from the merged live ACL; pass --replace-config to overwrite')
    config.write_text(desired)


def _import_unowned(stack_dir: Path, state_dir: Path, settings: Settings) -> None:
    """Inspect backend ownership before importing; never re-import tracked resources."""
    state = state_dir / 'terraform.tfstate'
    owned = set()
    if state.is_file():
        result = subprocess.run(['tofu', f'-chdir={stack_dir}', 'state', 'list'],
                                check=True, capture_output=True, text=True)
        owned = set(result.stdout.splitlines())
    resources = [('tailscale_acl.policy', 'acl')]
    if settings.magic_dns is not None:
        resources.append(('tailscale_dns_preferences.dns', 'dns_preferences'))
    for address, resource_id in resources:
        if address not in owned:
            tofu(stack_dir, 'import', '-input=false', address, resource_id)


def main(argv: list[str] | None = None) -> int:
    """Parse one public CLI; run under `uv run --locked --no-sync` like other Terrain stacks."""
    os.umask(0o077)
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('command', choices=COMMANDS)
    env = parser.add_mutually_exclusive_group()
    env.add_argument('--env-file', type=Path, help='literal dotenv file; defaults to checkout root .env')
    env.add_argument('--no-env', action='store_true', help='do not load a dotenv file')
    parser.add_argument('--config', type=Path, default=ROOT / 'network.config.json')
    parser.add_argument('--state-dir', type=Path, help='private persistent backend outside the checkout')
    parser.add_argument('--output', type=Path, default=ROOT / 'cdktf.out')
    parser.add_argument('--host', help='playback Mac Tailscale IP or prefix for hosts.durop-host')
    parser.add_argument('--member', action='append',
                        help='optional same-tailnet user email allowed on TCP 48732; guests use device sharing')
    parser.add_argument('--deny', help='optional email that policy tests must reject on TCP 48732')
    parser.add_argument('--replace-config', action='store_true',
                        help='overwrite network.config.json when ensure merges the live ACL')
    args = parser.parse_args(argv)
    try:
        if not args.no_env:
            load_env(args.env_file.expanduser().resolve() if args.env_file else REPO / '.env',
                     required=args.env_file is not None)
        # The host's invitation token must never become provisioner authentication.
        os.environ.pop('TAILSCALE_API_KEY', None)
        if args.command != 'ensure' and (args.host or args.member or args.deny or args.replace_config):
            raise ValueError('host/member flags are only valid with ensure')
        if args.command == 'setup':
            setup()
            return 0
        state = args.state_dir or Path(os.environ.get('XDG_STATE_HOME', str(Path.home() / '.local/state'))) / 'durop/tailscale'
        overlay = overlay_from_flags(args.host, args.member, args.deny, args.replace_config) if args.command == 'ensure' else None
        execute(args.command, args.config, state, args.output, overlay=overlay)
    except (ValueError, OSError, subprocess.CalledProcessError) as error:
        print(f'{args.command} failed: {error}', file=sys.stderr)
        return 1
    return 0


if __name__ == '__main__':
    raise SystemExit(main())
