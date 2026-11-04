"""Offline CDK Terrain synthesis; lifecycle commands follow in the wrapper."""
import argparse
from pathlib import Path
from network import synthesize
from settings import Settings

ROOT = Path(__file__).resolve().parent

if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('command', nargs='?', choices=['synth'], default='synth')
    parser.add_argument('--config', type=Path, default=ROOT / 'network.config.json')
    parser.add_argument('--state-dir', type=Path, default=Path.home() / '.local/state/durop/tailscale')
    parser.add_argument('--output', type=Path, default=ROOT / 'cdktf.out')
    args = parser.parse_args()
    synthesize(Settings.load(args.config, args.state_dir), str(args.output))
