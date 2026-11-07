"""Literal dotenv loading; inherited environment values always win."""
from __future__ import annotations

import os
from pathlib import Path
import re


def load_env(path: Path, *, required: bool = False) -> None:
    """Load literal single-line assignments without expansion or execution.

    Quotes delimit values. Reject malformed files without echoing secret contents.
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
