#!/usr/bin/env python3
"""Export committed sources, Git history and existing installable packages."""
import hashlib
import io
import json
import subprocess
import tempfile
import zipfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
REPOSITORY = 'https://github.com/LutiyTektonik/power_craft_2_rust.git'


def git(*args):
    return subprocess.check_output(['git', *args], cwd=ROOT)


def main():
    if git('status', '--porcelain', '--untracked-files=normal').strip():
        raise ValueError('Сначала сохраните все изменения в Git; экспортируется только HEAD.')
    verification = json.loads((ROOT / 'docs/VERIFICATION.json').read_text())
    release = verification['content_version']
    files = {}
    with zipfile.ZipFile(io.BytesIO(git('archive', '--format=zip', 'HEAD'))) as source:
        for entry in source.infolist():
            if not entry.is_dir():
                files['source/' + entry.filename] = (source.read(entry), entry.external_attr)
    artifacts = [
        *(Path(artifact['path']).name for artifact in verification['artifacts']),
        'powercraft2-0.2.0.hnmod', 'powercraft2-0.2.0-content.hmod',
        'powercraft2-windows11-test-kit.zip',
    ]
    for name in artifacts:
        path = ROOT / 'dist' / name
        data = path.read_bytes()
        with zipfile.ZipFile(io.BytesIO(data)) as archive:
            if archive.testzip() is not None:
                raise ValueError(f'Повреждён архив: {name}')
        files['artifacts/' + name] = (data, 0o100644 << 16)
    for artifact in verification['artifacts']:
        name = Path(artifact['path']).name
        data = files['artifacts/' + name][0]
        if hashlib.sha256(data).hexdigest() != artifact['sha256']:
            raise ValueError(f'Пакет отличается от проверенного: {name}')
    with tempfile.TemporaryDirectory(prefix='pc2-transfer-') as temporary:
        bundle = Path(temporary) / 'project.bundle'
        git('bundle', 'create', str(bundle), '--branches', '--tags')
        git('bundle', 'verify', str(bundle))
        files['project.bundle'] = (bundle.read_bytes(), 0o100644 << 16)
    branch = git('branch', '--show-current').decode().strip()
    if not branch:
        raise ValueError('Для переноса переключитесь на рабочую ветку, не detached HEAD.')
    restore = f'''Полное восстановление в Bash / WSL2 Ubuntu 24.04:
git clone --branch {branch} ./project.bundle powercraft2-horizon
cd powercraft2-horizon
git remote set-url origin {REPOSITORY}
git fetch origin --tags

Без сети fetch можно пропустить. Прочитайте README.md, AGENTS.md и
docs/LOCAL_DEVELOPMENT.md. Настройте среду через scripts/bootstrap.sh --install,
source scripts/env.sh; затем scripts/check.sh. Авторизация переносится отдельно.
source/ — снимок, project.bundle — история. Не копируйте source поверх клона.
artifacts/ — установочные пары {release} и проверенной 0.2.0, Windows test kit.
Все файлы сверяются по TRANSFER_MANIFEST.json.
'''
    files['RESTORE.txt'] = (restore.encode(), 0o100644 << 16)
    manifest = {
        'repository': REPOSITORY,
        'branch': branch,
        'commit': git('rev-parse', 'HEAD').decode().strip(),
        'refs': git('show-ref', '--heads', '--tags').decode().splitlines(),
        'game_version': release,
        'files': {name: hashlib.sha256(data).hexdigest()
                  for name, (data, _) in sorted(files.items())},
    }
    files['TRANSFER_MANIFEST.json'] = (
        (json.dumps(manifest, ensure_ascii=False, indent=2) + '\n').encode(),
        0o100644 << 16,
    )
    output = ROOT / 'dist/powercraft2-local-transfer.zip'
    with zipfile.ZipFile(output, 'w', compression=zipfile.ZIP_DEFLATED) as archive:
        for name, (data, mode) in sorted(files.items()):
            info = zipfile.ZipInfo(name, (2026, 1, 1, 0, 0, 0))
            info.external_attr = mode
            info.compress_type = zipfile.ZIP_DEFLATED
            archive.writestr(info, data)
    print(json.dumps({'path': str(output), 'bytes': output.stat().st_size,
                      'sha256': hashlib.sha256(output.read_bytes()).hexdigest(),
                      'commit': manifest['commit'], 'files': len(files)}, indent=2))


if __name__ == '__main__':
    try:
        main()
    except (OSError, ValueError, KeyError, zipfile.BadZipFile,
            subprocess.CalledProcessError) as error:
        raise SystemExit(str(error)) from error
