"""Internal lifecycle operations for infra/manage.py; apply consumes one saved plan."""
from contextlib import contextmanager
import fcntl
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess

from network import synthesize
from settings import Settings

ROOT = Path(__file__).resolve().parent
STACK = 'durop-network'


def digest(path: Path) -> str:
    """Hash private inputs without including their contents in metadata."""
    return hashlib.sha256(path.read_bytes()).hexdigest()


def tofu(stack_dir: Path, *args: str) -> None:
    """Run OpenTofu directly with an argument vector and the selected backend."""
    subprocess.run(['tofu', f'-chdir={stack_dir}', *args], check=True)


def fingerprint(config: Path, state_dir: Path, stack_dir: Path) -> str:
    """Bind a saved plan to all authored, generated, locked and target inputs."""
    paths = [ROOT / n for n in ('main.py', 'network.py', 'settings.py', 'pyproject.toml',
             'uv.lock', 'package.json', 'package-lock.json', 'cdktf.json', '.terraform.lock.hcl', '.python-version', '.node-version')]
    paths += sorted((ROOT / 'imports').rglob('*.py')) + sorted((ROOT / 'imports').rglob('*.tgz'))
    paths += sorted((ROOT / 'imports').glob('*.json'))
    values = {str(p.relative_to(ROOT)): digest(p) for p in paths}
    values.update({name: digest(ROOT.parents[2] / name) for name in ('flake.nix', 'flake.lock')})
    values['manager'] = digest(ROOT.parents[1] / 'manage.py')
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


def execute(command: str, config: Path, state_dir: Path, output: Path) -> None:
    """Execute a single stack lifecycle step without replacing reviewed apply inputs."""
    state_dir, output, config = state_dir.expanduser().resolve(), output.resolve(), config.resolve()
    repo = ROOT.parents[2]
    if state_dir == repo or repo in state_dir.parents or state_dir == output or output in state_dir.parents:
        raise ValueError('state/plans must live outside the checkout and generated output')
    if command not in {'synth', 'validate', 'import', 'plan', 'show', 'apply', 'status'}:
        raise ValueError('unknown lifecycle command')
    if command in {'import', 'plan', 'apply'}:
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
        if command in {'plan', 'import'}:
            # Invalidate before config parsing/synthesis: failures also require fresh review.
            metadata.unlink(missing_ok=True)
            plan.unlink(missing_ok=True)
        # Missing plan is refused before synthesis or any OpenTofu subprocess.
        if command in {'apply', 'show'} and (not plan.is_file() or not metadata.is_file()):
            raise ValueError('no reviewed plan; run plan and review it first')
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
        if command == 'import':
            # Inspect backend ownership before importing; never re-import tracked resources.
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
        elif command == 'status':
            print(f'Stack: {STACK}\nState: {state_dir / "terraform.tfstate"}')
            if (state_dir / 'terraform.tfstate').is_file():
                tofu(stack_dir, 'state', 'list')
            else:
                print('No resources imported into this backend.')
            print(f'Saved plan: {"present; run show to verify" if metadata.is_file() and plan.is_file() else "none"}')
        elif command == 'show':
            tofu(stack_dir, 'show', str(plan))
        elif command == 'plan':
            tofu(stack_dir, 'plan', '-input=false', f'-out={plan}')
            metadata.write_text(json.dumps({'fingerprint': current, 'plan_sha256': digest(plan)}, sort_keys=True) + '\n')
        elif command == 'apply':
            # Invalidate before apply: interrupted/failed applies also require a fresh review.
            metadata.unlink()
            tofu(stack_dir, 'apply', '-input=false', str(plan))
            plan.unlink()
