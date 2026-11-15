#!/usr/bin/env python3
"""Assemble a Windows test kit; game validation is recorded separately."""
import hashlib
import json
import zipfile
import tomllib
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


def write_zip(out, files):
    out.parent.mkdir(parents=True, exist_ok=True)
    with zipfile.ZipFile(out, 'w') as archive:
        for name, data in sorted(files.items()):
            info = zipfile.ZipInfo(name, (2026, 1, 1, 0, 0, 0))
            info.compress_type = zipfile.ZIP_DEFLATED
            info.external_attr = 0o100644 << 16
            archive.writestr(info, data)
    out.chmod(0o644)
    return out


def pin_native(root=ROOT):
    version = tomllib.loads((root / 'Cargo.toml').read_text())['package']['version']
    filename = f'powercraft2-{version}.hnmod'
    files = read_zip(root / 'native/target' / filename)
    manifest = json.loads(files['horizon-native.json'])
    compat = json.loads((root / 'native/powercraft2/compatibility.json').read_text())
    if (manifest['horizon'] != compat['horizon_revision']
            or manifest['mixins'] != compat['horizon_mixins_revision']
            or manifest['version'] != version):
        raise ValueError('Native package revision mismatch')
    files['horizon-native.json'] = (json.dumps(pinned_manifest(manifest), sort_keys=True, indent=2) + '\n').encode()
    return write_zip(root / 'dist' / filename, files)


def prepare(root=ROOT):
    dist = root / 'dist'
    compat = json.loads((root / 'native/powercraft2/compatibility.json').read_text())
    version = tomllib.loads((root / 'Cargo.toml').read_text())['package']['version']
    native_path = dist / f'powercraft2-{version}.hnmod'
    content_version = tomllib.loads((root / 'content/horizon-mod.toml').read_text())['version']
    content_path = dist / f'powercraft2-{content_version}-content.hmod'
    native = read_zip(native_path)
    content = read_zip(content_path)
    manifest = json.loads(native['horizon-native.json'])
    expected = {'id': 'powercraft2', 'version': version, 'game': compat['game'],
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
    report = 'docs/VERIFICATION.json' if (root / 'docs/VERIFICATION.json').exists() else 'docs/SMOOTH_MOVEMENT_VERIFICATION.json' if (root / 'docs/SMOOTH_MOVEMENT_VERIFICATION.json').exists() else 'docs/WINDOWS11_VERIFICATION.json'
    for source, name in [('docs/WINDOWS11.md', 'INSTALL-WINDOWS11.md'),
                         ('docs/PORTING_PROGRESS.md' if (root / 'docs/PORTING_PROGRESS.md').exists() else 'docs/FEATURE_AUDIT.md', 'FEATURES.md'),
                         (report, 'VERIFICATION.json')]:
        files[name] = (root / source).read_bytes()
    changes = root / 'CHANGELOG.md'
    if not changes.exists():
        changes = root / 'docs/SMOOTH_MOVEMENT.md'
    if changes.exists():
        files['CHANGES.md'] = changes.read_bytes()
    for name in ['TESTING.md', 'PORTING_PROGRESS.md', 'ORIGINAL_REGISTRY.md',
                 'ORIGINAL_MECHANICS.json', 'ORIGINAL_SOURCE_INDEX.json',
                 'BASELINE.md', 'BASELINE_DATA.json', 'COMPATIBILITY.md', 'LEGAL.md',
                 'STORAGE_TRANSFER.md', 'PUBLISHED_RELEASE.md', 'PUBLISHED_AUDIT.json',
                 'CORE.md', 'DECO.md', 'MACHINES.md', 'MECHANICS_AUDIT.md', 'AGENTS_STATUS.md',
                 'PORTING_COMPARISON.json', 'DETECTOR.md', 'SPEEDY_VERIFICATION.json']:
        document = root / 'docs' / name
        if document.exists():
            files[name] = document.read_bytes()
    if report == 'docs/VERIFICATION.json':
        report_data = json.loads((root / report).read_text())
        logs = (root / report_data.get('logs_directory', 'target/verification-transport')).resolve()
        if not logs.is_relative_to((root / 'target').resolve()):
            raise ValueError('Verification logs must stay inside project target')
        for log in sorted(logs.glob('*')):
            if log.is_file() and log.suffix in ('.json', '.log'):
                files['logs/' + log.name] = log.read_bytes()
    elif report.endswith('SMOOTH_MOVEMENT_VERIFICATION.json'):
        for name in ['smooth-project-check.log', 'horizon-smooth-check.log',
                     'smooth-native-validation.json', 'smooth-content-validation.json']:
            log = root / 'target' / name
            if log.exists():
                files['logs/' + name] = log.read_bytes()
    else:
        logs = root / 'target/verification-windows11'
        for log in sorted(logs.glob('*.log')):
            files['logs/' + log.name] = log.read_bytes()
    files['NOT-GAME-VALIDATED.txt'] = (
        'Это набор для экспериментальной проверки, не проверенный игровой релиз.\n'
        'Фактические результаты Rust/интеграции указаны в VERIFICATION.json.\n'
        'Сначала прочитайте INSTALL-WINDOWS11.md и VERIFICATION.json.\n'
        'packages/*.hnmod создан официальным native.py package, затем закреплён строго.\n'
        'Удалено только необязательное hooks из дескриптора; Rust-payload не меняется.\n'
        'native-pinned/ — распакованный вход Launcher для того же точного commit.\n'
    ).encode()
    hashes = [f'{hashlib.sha256(data).hexdigest()}  {name}' for name, data in sorted(files.items())]
    files['SHA256SUMS.txt'] = ('\n'.join(hashes) + '\n').encode()
    out = dist / 'powercraft2-windows11-test-kit.zip'
    return write_zip(out, files)


if __name__ == '__main__':
    import argparse
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--native-only', action='store_true', help='pin official native.py package output for distribution')
    args = parser.parse_args()
    out = pin_native() if args.native_only else prepare()
    print(json.dumps({'path': str(out), 'size': out.stat().st_size,
                      'sha256': hashlib.sha256(out.read_bytes()).hexdigest(),
                      'official_validation': 'see VERIFICATION.json; not run by this packager',
                      'windows_execution': 'not tested'}, indent=2))
