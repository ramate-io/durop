#!/bin/sh
# Run from a normal native terminal, never through nix develop.
set -eu
exec /usr/bin/python3 "$(dirname "$0")/package.py" "$@"
