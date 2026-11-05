#!/usr/bin/env python3
"""Validate a Conventional Commit header or a PR title from GitHub's event JSON."""
import argparse
import json
from pathlib import Path
import re
import sys

TYPES = frozenset({"feat", "fix", "build", "chore", "ci", "docs", "style", "refactor", "perf", "test", "revert"})
MAX_HEADER_LENGTH = 200
MAX_EVENT_BYTES = 4 * 1024 * 1024
HEADER = re.compile(r"(?P<type>[A-Za-z]+)(?:\([A-Za-z0-9][A-Za-z0-9_./-]*\))?!?: (?P<description>\S(?:.*\S)?)")


def validate_header(header):
    """Validate only the single subject line; never interpret its contents as code."""
    if not isinstance(header, str) or not header or len(header) > MAX_HEADER_LENGTH:
        raise ValueError("header must contain 1..200 characters")
    if not header.isprintable() or header != header.strip():
        raise ValueError("header must be one printable line without surrounding whitespace")
    match = HEADER.fullmatch(header)
    if match is None:
        raise ValueError("expected 'type(scope)!: description'; scope and ! are optional")
    if match["type"].lower() not in TYPES:
        raise ValueError("unsupported type; use " + ", ".join(sorted(TYPES)))


def event_title(path):
    with Path(path).open("rb") as stream:
        payload = stream.read(MAX_EVENT_BYTES + 1)
    if len(payload) > MAX_EVENT_BYTES:
        raise ValueError("event JSON exceeds the 4 MiB validation bound")
    event = json.loads(payload)
    if not isinstance(event, dict) or not isinstance(event.get("pull_request"), dict):
        raise ValueError("event JSON must contain a pull_request object")
    title = event["pull_request"].get("title")
    if not isinstance(title, str):
        raise ValueError("event JSON must contain a string pull_request.title")
    return title


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    source = parser.add_mutually_exclusive_group(required=True)
    source.add_argument("--header", help="proposed commit subject or PR title")
    source.add_argument("--event", type=Path, help="GitHub pull_request event JSON file")
    args = parser.parse_args(argv)
    try:
        validate_header(event_title(args.event) if args.event is not None else args.header)
    except (OSError, ValueError) as error:
        print(f"Invalid Conventional Commit header: {error}", file=sys.stderr)
        return 1
    # Do not echo untrusted titles into terminal output or annotations.
    print("Conventional Commit header is valid.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
