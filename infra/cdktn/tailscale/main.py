"""Explicit lifecycle commands. Apply consumes exactly one reviewed saved plan."""
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

from network import synthesize
from settings import Settings

ROOT = Path(__file__).resolve().parent
STACK = 'durop-network'


def digest(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def tofu(stack_dir: Path, *args: str) -> None:
    subprocess.run(['tofu', f'-chdir={stack_dir}', *args], check=True)


def fingerprint(config: Path, state_dir: Path, stack_dir: Path) -> str:
    paths = [ROOT / n for n in ('main.py', 'network.py', 'settings.py', 'pyproject.toml',
             'uv.lock', 'package.json', 'package-lock.json', 'cdktf.json', '.terraform.lock.hcl', '.python-version', '.node-version')]
    paths += sorted((ROOT / 'imports').rglob('*.py')) + sorted((ROOT / 'imports').rglob('*.tgz'))
    paths += sorted((ROOT / 'imports').glob('*.json'))
    values = {str(p.relative_to(ROOT)): digest(p) for p in paths}
    values.update({name: digest(ROOT.parents[2] / name) for name in ('flake.nix', 'flake.lock')})
    values.update(config=digest(config), state=str(state_dir),
                  generated=digest(stack_dir / 'cdk.tf.json'),
                  provider_target=hashlib.sha256(json.dumps({name: os.environ.get(name, '') for name in ('TAILSCALE_TAILNET', 'TAILSCALE_BASE_URL', 'TAILSCALE_OAUTH_CLIENT_ID')}, sort_keys=True).encode()).hexdigest())
    return hashlib.sha256(json.dumps(values, sort_keys=True).encode()).hexdigest()


@contextmanager
def operator_lock(state_dir: Path):
    state_dir.mkdir(parents=True, exist_ok=True, mode=0o700)
    if state_dir.stat().st_mode & 0o077:
        raise ValueError('state directory must be private (chmod 700)')
    with (state_dir / 'operator.lock').open('a') as handle:
        fcntl.flock(handle, fcntl.LOCK_EX | fcntl.LOCK_NB)
        yield


def execute(command: str, config: Path, state_dir: Path, output: Path) -> None:
    state_dir, output, config = state_dir.expanduser().resolve(), output.resolve(), config.resolve()
    repo = ROOT.parents[2]
    if state_dir == repo or repo in state_dir.parents or state_dir == output or output in state_dir.parents:
        raise ValueError('state/plans must live outside the checkout and generated output')
    if command in {'plan', 'apply'} and not os.environ.get('TAILSCALE_TAILNET', '').strip():
        raise ValueError('plan/apply require an explicit TAILSCALE_TAILNET target')
    settings = Settings.load(config, state_dir)
    if command == 'synth':
        synthesize(settings, str(output))
        return
    with operator_lock(state_dir):
        plan = state_dir / 'reviewed.tfplan'
        metadata = state_dir / 'reviewed.json'
        # Missing plan is refused before synthesis or any OpenTofu subprocess.
        if command == 'apply' and (not plan.is_file() or not metadata.is_file()):
            raise ValueError('no reviewed plan; run plan and review it first')
        synthesize(settings, str(output))
        stack_dir = output / 'stacks' / STACK
        shutil.copyfile(ROOT / '.terraform.lock.hcl', stack_dir / '.terraform.lock.hcl')
        current = fingerprint(config, state_dir, stack_dir)
        if command == 'apply':
            recorded = json.loads(metadata.read_text())
            if recorded != {'fingerprint': current, 'plan_sha256': digest(plan)}:
                raise ValueError('stale or modified plan; create and review a new plan')
        if command == 'plan':
            # Invalidate any earlier reviewed plan before an operation that may fail.
            metadata.unlink(missing_ok=True)
            plan.unlink(missing_ok=True)
        tofu(stack_dir, 'init', '-input=false', '-lockfile=readonly')
        tofu(stack_dir, 'validate')
        if command == 'plan':
            tofu(stack_dir, 'plan', '-input=false', f'-out={plan}')
            metadata.write_text(json.dumps({'fingerprint': current, 'plan_sha256': digest(plan)}, sort_keys=True) + '\n')
        elif command == 'apply':
            # Invalidate before apply: interrupted/failed applies also require a fresh review.
            metadata.unlink()
            tofu(stack_dir, 'apply', '-input=false', str(plan))
            plan.unlink()


def main() -> int:
    os.umask(0o077)
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('command', nargs='?', choices=['synth', 'validate', 'plan', 'apply'], default='synth')
    parser.add_argument('stack', nargs='?', choices=[STACK])
    parser.add_argument('--config', type=Path, default=ROOT / 'network.config.json')
    parser.add_argument('--state-dir', type=Path, default=Path(os.environ.get('XDG_STATE_HOME', str(Path.home() / '.local/state'))) / 'durop/tailscale')
    parser.add_argument('--output', type=Path, default=ROOT / 'cdktf.out')
    args = parser.parse_args()
    if args.command != 'synth' and args.stack is None:
        parser.error('validate/plan/apply require explicit stack durop-network')
    try:
        execute(args.command, args.config, args.state_dir, args.output)
    except (ValueError, OSError, subprocess.CalledProcessError) as error:
        print(f'{args.command} failed: {error}', file=sys.stderr)
        return 1
    return 0


if __name__ == '__main__':
    raise SystemExit(main())
