#!/usr/bin/env python3
"""Read pinned published JARs as ZIP; never execute Java or export original art."""
import argparse
import hashlib
import importlib.util
import json
import zipfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
spec = importlib.util.spec_from_file_location('original_facts', ROOT / 'scripts/inventory-original.py')
facts = importlib.util.module_from_spec(spec)
spec.loader.exec_module(facts)


def audit(directory, current_source=None, root=ROOT):
    index = json.loads((root / 'docs/PUBLISHED_FILES.json').read_text())
    contents = {}
    files = []
    archives = {}
    if current_source is not None:
        facts.inventory(current_source, root)  # Verify all 445 upstream files first.
        current = {str(p).split('/src/main/java/')[-1]: p.read_bytes()
                   for p in current_source.rglob('*.java')}
    else:
        current = None
    equal = 0
    different = []
    for file in index['files']:
        path = directory / file['path']
        data = path.read_bytes()
        blob = hashlib.sha1(f'blob {len(data)}\0'.encode() + data).hexdigest()
        if blob != file['git_blob'] or len(data) != file['bytes']:
            raise ValueError(f'Published JAR mismatch: {file["path"]}')
        with zipfile.ZipFile(path) as archive:
            if archive.testzip() is not None:
                raise ValueError(f'Published JAR CRC failed: {file["path"]}')
            names = archive.namelist()
            java = [n for n in names if n.endswith('.java')]
            classes = [n for n in names if n.endswith('.class')]
            module = Path(file['path']).parts[2]
            for name in classes:
                if archive.read(name)[:4] != b'\xca\xfe\xba\xbe':
                    raise ValueError(f'Invalid class file: {name}')
            if java:
                archives[module] = file['path']
            for name in java:
                data_java = archive.read(name)
                prefix = f'{module}/src/main/java' if module in ('api', 'powercraft') else f'modules/{module}/src/main/java'
                contents[f'{prefix}/{name}'] = data_java.decode('utf-8-sig')
                if current is not None:
                    if current.get(name) == data_java:
                        equal += 1
                    else:
                        different.append(name)
            files.append({**file, 'sha256': hashlib.sha256(data).hexdigest(),
                          'java_files': len(java), 'class_files': len(classes),
                          'compiled_classes': classes})
    registry = facts.describe_sources(contents)
    registry.update(repository=index['repository'], revision=index['revision'],
                    source_archives=archives,
                    scope='Статический реестр опубликованных source-JAR 0.1.0B. Logical path после /src/main/java/ — путь внутри указанного source_archives[module]. Все JAR проверены по Git blob ID; код не запускался. Наличие реализации/публикации не доказывает игровую исправность.')
    report = {'maven_revision': index['revision'], 'files': files,
              'comparison': {'total_java': len(contents), 'byte_identical': equal if current is not None else None,
                             'different': different if current is not None else None},
              'not_executed': 'Архивы прочитаны как ZIP; Forge/Java-код не запускался, оригинальные ресурсы в мод не включены.'}
    return report, registry


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--published', type=Path, required=True)
    parser.add_argument('--source', type=Path)
    args = parser.parse_args()
    report, registry = audit(args.published, args.source)
    for name, value in [('PUBLISHED_AUDIT.json', report), ('PUBLISHED_MECHANICS.json', registry)]:
        (ROOT / 'docs' / name).write_text(json.dumps(value, ensure_ascii=False, indent=2) + '\n')
    print(json.dumps({'jars': len(report['files']), 'source_files': len(registry['java_classes']),
                      'recipe_declarations': len(registry['recipe_declarations']),
                      'registered_objects': len(registry['registered_objects']),
                      'byte_identical_to_git': report['comparison']['byte_identical']}, ensure_ascii=False))


if __name__ == '__main__':
    main()
