#!/usr/bin/env python3
"""Fetch pinned official tooling for local use; never redistribute it in this repository."""
import argparse
import hashlib
import json
import sys
import urllib.request
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]


def blob(data):
    return hashlib.sha1(f'blob {len(data)}\0'.encode() + data).hexdigest()


def ensure(download=False):
    provenance = json.loads((ROOT / 'docs/TOOL_PROVENANCE.json').read_text())
    revision = provenance['revision']
    for file in provenance['files']:
        relative = file['path']
        if relative not in ('native/tools/native.py', 'native/tools/game.py'):
            raise ValueError('unexpected external tool path')
        path = ROOT / relative
        if path.is_symlink():
            raise ValueError(f'symlink refused: {relative}')
        if path.is_file():
            if blob(path.read_bytes()) != file['git_blob']:
                raise ValueError(f'modified external tool: {relative}')
            continue
        if not download:
            raise ValueError('Нет официальных инструментов: python3 scripts/fetch-tools.py --download')
        url = f'https://raw.githubusercontent.com/horizon-rust/horizon-mods/{revision}/{relative}'
        request = urllib.request.Request(url, headers={'User-Agent': 'powercraft2-bootstrap'})
        with urllib.request.urlopen(request, timeout=20) as response:
            data = response.read(256 * 1024 + 1)
        if len(data) > 256 * 1024 or blob(data) != file['git_blob']:
            raise ValueError(f'upstream identity mismatch: {relative}')
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_bytes(data)


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--download', action='store_true')
    args = parser.parse_args()
    try:
        ensure(args.download)
        print('Официальные инструменты: точные Git blob ID подтверждены.')
    except (OSError, ValueError) as error:
        print(str(error), file=sys.stderr)
        sys.exit(1)
