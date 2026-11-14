#!/usr/bin/env python3
"""Assemble an explicitly unvalidated Windows test kit from existing source packages."""
import hashlib
import json
import zipfile
from pathlib import Path, PurePosixPath

ROOT = Path(__file__).resolve().parents[1]


def read_zip(path):
    files = {}
    with zipfile.ZipFile(path) as archive:
        if archive.testzip() is not None:
            raise ValueError(f'ZIP CRC failed: {path}')
        for entry in archive.infolist():
            name = entry.filename
            parts = PurePosixPath(name).parts
            if (entry.is_dir() or name.startswith('/') or '\\' in name or ':' in name
                    or '..' in parts or name in files):
                raise ValueError(f'Invalid archive entry: {name}')
            files[name] = archive.read(entry)
    return files


def pinned_manifest(manifest):
    # In the real Launcher, `hooks` means Horizon is only a tested-on commit.
    # Its absence makes `horizon` a strict pin; retain the Mixins pin as well.
    result = dict(manifest)
    result.pop('hooks', None)
    return result


def prepare(root=ROOT):
    dist = root / 'dist'
    compat = json.loads((root / 'native/powercraft2/compatibility.json').read_text())
    native_path = dist / 'powercraft2-0.1.0.hnmod'
    content_path = dist / 'powercraft2-0.1.0-content.hmod'
    native = read_zip(native_path)
    content = read_zip(content_path)
    manifest = json.loads(native['horizon-native.json'])
    expected = {'id': 'powercraft2', 'version': '0.1.0', 'game': compat['game'],
                'horizon': compat['horizon_revision'], 'mixins': compat['horizon_mixins_revision']}
    if any(manifest.get(k) != v for k, v in expected.items()):
        raise ValueError('Native archive pin or identity mismatch')
    bundle = root / 'native/powercraft2/mixins/powercraft2'
    expected_names = {'horizon-native.json', 'mixins/powercraft2/LICENSE'}
    expected_names.update(f'mixins/powercraft2/{p.name}' for p in bundle.iterdir() if p.is_file())
    if set(native) != expected_names:
        raise ValueError('Unexpected native payload')
    for name, data in native.items():
        if name == 'horizon-native.json':
            continue
        source = root / 'native/powercraft2/LICENSE' if name.endswith('/LICENSE') else bundle / PurePosixPath(name).name
        if data != source.read_bytes():
            raise ValueError(f'Stale source in native archive: {name}')
    for name, data in content.items():
        if data != (root / 'content' / name).read_bytes():
            raise ValueError(f'Stale content in archive: {name}')
    files = {f'packages/{native_path.name}': native_path.read_bytes(),
             f'packages/{content_path.name}': content_path.read_bytes()}
    for name, data in native.items():
        if name == 'horizon-native.json':
            data = (json.dumps(pinned_manifest(manifest), sort_keys=True, indent=2) + '\n').encode()
        files['native-pinned/' + name] = data
    for source, name in [('docs/WINDOWS11.md', 'INSTALL-WINDOWS11.md'),
                         ('docs/FEATURE_AUDIT.md', 'FEATURES.md'),
                         ('docs/WINDOWS11_VERIFICATION.json', 'VERIFICATION.json')]:
        files[name] = (root / source).read_bytes()
    logs = root / 'target/verification-windows11'
    for log in sorted(logs.glob('*.log')):
        files['logs/' + log.name] = log.read_bytes()
    files['NOT-GAME-VALIDATED.txt'] = (
        'Это набор для экспериментальной проверки, не проверенный игровой релиз.\n'
        'Rust, официальный валидатор и запуск Windows заблокированы в Cloud.\n'
        'Сначала прочитайте INSTALL-WINDOWS11.md и VERIFICATION.json.\n'
        'packages/*.hnmod создан официальным native.py package; контент упакован нашим скриптом.\n'
        'native-pinned/ сохраняет исходники без изменения, но убирает необязательный hooks\n'
        'из тестового дескриптора, чтобы Launcher использовал точный Horizon commit.\n'
    ).encode()
    hashes = [f'{hashlib.sha256(data).hexdigest()}  {name}' for name, data in sorted(files.items())]
    files['SHA256SUMS.txt'] = ('\n'.join(hashes) + '\n').encode()
    out = dist / 'powercraft2-windows11-test-kit.zip'
    with zipfile.ZipFile(out, 'w') as archive:
        for name, data in sorted(files.items()):
            info = zipfile.ZipInfo(name, (2026, 1, 1, 0, 0, 0))
            info.compress_type = zipfile.ZIP_DEFLATED
            info.external_attr = 0o100644 << 16
            archive.writestr(info, data)
    out.chmod(0o644)
    return out


if __name__ == '__main__':
    out = prepare()
    print(json.dumps({'path': str(out), 'size': out.stat().st_size,
                      'sha256': hashlib.sha256(out.read_bytes()).hexdigest(),
                      'official_validation': 'not performed', 'windows_execution': 'not tested'}, indent=2))
