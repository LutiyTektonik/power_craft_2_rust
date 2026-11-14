#!/usr/bin/env python3
"""Package content using the documented zip layout; official validation is separate."""
import hashlib
import json
import zipfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]


def package_content(root=ROOT):
    import tomllib
    source = root / 'content'
    manifest = tomllib.loads((source / 'horizon-mod.toml').read_text())
    out = root / 'dist' / f'{manifest["id"]}-{manifest["version"]}-content.hmod'
    out.parent.mkdir(exist_ok=True)
    paths = [source / 'horizon-mod.toml']
    for folder in ('assets', 'data'):
        paths.extend(p for p in (source / folder).rglob('*') if p.is_file())
    with zipfile.ZipFile(out, 'w') as archive:
        for path in sorted(paths):
            if path.is_symlink():
                raise ValueError(f'symlink refused: {path}')
            info = zipfile.ZipInfo(path.relative_to(source).as_posix(), (2026, 1, 1, 0, 0, 0))
            info.compress_type = zipfile.ZIP_DEFLATED
            info.external_attr = 0o100644 << 16
            archive.writestr(info, path.read_bytes())
    return out


if __name__ == '__main__':
    out = package_content()
    print(json.dumps({'file': str(out), 'size': out.stat().st_size,
                      'sha256': hashlib.sha256(out.read_bytes()).hexdigest(),
                      'official_validation': 'not performed by this packager'}, indent=2))
