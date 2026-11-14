#!/usr/bin/env python3
"""Index source facts without distributing original Java code or assets."""
import argparse
import hashlib
import json
import re
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
REVISION = '795369c5a03c7cf39307398a432a5d644698e1df'


def without_comments(text):
    # Keep strings and newlines, so evidence line numbers remain exact.
    token = r'"(?:\\.|[^"\\])*"|\'(?:\\.|[^\'\\])*\'|/\*[\s\S]*?\*/|//[^\n]*'
    return re.sub(token, lambda m: re.sub(r'[^\n]', ' ', m[0])
                  if m[0].startswith(('//', '/*')) else m[0], text)


def arguments(text, start):
    """Split Java call arguments; respect nested calls, arrays and literals."""
    stack = []
    quote = None
    escaped = False
    result = []
    begin = start + 1
    for i in range(start + 1, len(text)):
        char = text[i]
        if quote:
            if escaped:
                escaped = False
            elif char == '\\':
                escaped = True
            elif char == quote:
                quote = None
        elif char in ('"', "'"):
            quote = char
        elif char in '([{':
            stack.append(char)
        elif char in ')]}':
            if not stack:
                result.append(text[begin:i].strip())
                return result, i
            stack.pop()
        elif char == ',' and not stack:
            result.append(text[begin:i].strip())
            begin = i + 1
    raise ValueError('Unclosed Java call')


def stack_fact(expression):
    found = re.match(r'new\s+ItemStack\s*\(', expression)
    if found:
        values, _ = arguments(expression, found.end() - 1)
        return {'reference': values[0], 'count': values[1] if len(values) > 1 else '1',
                'metadata': values[2] if len(values) > 2 else '0'}
    if re.fullmatch(r'[\w.]+', expression):
        return {'reference': expression}
    return {'manual_review_required': True}


def inventory(source, root=ROOT):
    index = json.loads((root / 'docs/ORIGINAL_SOURCE_INDEX.json').read_text())
    if index['revision'] != REVISION or index['tree_truncated']:
        raise ValueError('Untrusted source index revision/tree')
    contents = {}
    for entry in index['files']:
        data = (source / entry['path']).read_bytes()
        blob = hashlib.sha1(f'blob {len(data)}\0'.encode() + data).hexdigest()
        if blob != entry['git_blob'] or len(data) != entry['bytes']:
            raise ValueError(f'Original source mismatch: {entry["path"]}')
        contents[entry['path']] = data.decode('utf-8')
    return describe_sources(contents)


def describe_sources(contents):
    """Extract facts; callers separately verify Git source or published JARs."""
    classes = []
    objects = []
    recipes = []
    interfaces = []
    for path, raw in sorted(contents.items()):
        if not path.endswith('.java'):
            continue
        text = without_comments(raw)
        parts = Path(path).parts
        module = parts[1] if parts[0] == 'modules' else parts[0]
        name = Path(path).stem
        category = 'Частично реализована в оригинале' if module in ('logic', 'storage', 'mobile') else 'Реализована в исходниках оригинала'
        classes.append({'class': name, 'module': module, 'path': path,
                        'public_methods': sorted(set(re.findall(
                            r'(?:public|protected)\s+(?:static\s+|final\s+|synchronized\s+)*[\w<>\[\], ?]+\s+(\w+)\s*\([^;{}]*\)\s*\{', text))),
                        'constants': [{'name': m[0], 'value': m[1].strip()}
                                      for m in re.findall(r'public\s+static\s+final\s+(?:int|String)\s+(\w+)\s*=\s*([^;]+);', text)],
                        'todo_lines': [i for i, line in enumerate(raw.splitlines(), 1) if 'TODO' in line]})
        for match in re.finditer(r'@PC_FieldObject\(\s*clazz\s*=\s*(\w+)\.class[^)]*\)\s*public\s+static\s+(\w+)\s+(\w+)', text):
            clazz, kind, field = match.groups()
            objects.append({'module': module, 'class': clazz, 'field': field, 'kind': kind,
                            'original_category': category,
                            'registration': {'path': path, 'line': text.count('\n', 0, match.start()) + 1}})
        if any(part in ('gui', 'container', 'gres') for part in parts) or 'Gui' in name or 'Container' in name:
            interfaces.append({'module': module, 'class': name, 'path': path})
        for match in re.finditer(r'(GameRegistry\.add(?:ShapelessRecipe|Recipe|Smelting)|new\s+PC_3DRecipe)\s*\(', text):
            args, _ = arguments(text, match.end() - 1)
            is_3d = 'PC_3DRecipe' in match[1]
            mode = list(re.finditer(r'PC_GlobalVariables\.(mediumRecipes|legacyRecipes)', text[:match.start()]))
            patterns = [json.loads(m) for m in re.findall(r'"(?:\\.|[^"\\])*"', ','.join(args[1:]))]
            ingredients = []
            # Facts only, never export Java method bodies. Complex/custom/3D
            # handlers stay linked to source and explicitly need manual review.
            for ingredient in re.finditer(r"'([^'])'\s*,\s*", ','.join(args[1:])):
                rest = ','.join(args[1:])[ingredient.end():]
                values, _ = arguments('(' + rest + ')', 0)
                ingredients.append({'symbol': ingredient[1], **stack_fact(values[0])})
            recipes.append({'module': module, 'kind': '3D' if is_3d else match[1].split('.')[-1],
                            'mode': mode[-1][1] if mode else 'unconditional_or_custom',
                            'output_or_handler': stack_fact(args[0]), 'patterns': patterns,
                            'ingredients': ingredients, 'manual_review_required': is_3d or not ingredients,
                            'source': {'path': path, 'line': text.count('\n', 0, match.start()) + 1}})
    return {'repository': 'PowerCraft/2.0', 'revision': REVISION,
            'scope': 'Все 427 Java-файлов и 18 build/README файлов проверены по Git blob ID. Это статический реестр, не запуск Forge и не доказательство отсутствия ошибок оригинала. Политика категории 2 для Logic/Storage/Mobile следует предупреждению автора README.',
            'verified_source_files': len(contents), 'java_classes': classes,
            'registered_objects': objects, 'recipe_declarations': recipes, 'interfaces': interfaces,
            'planned_or_removed': ['Weasel: отсутствует, планировался rewrite', 'CheckPoints: удалён',
                                   'Integrations: только build.gradle, не подключён', 'Энергетика: запланирована',
                                   'Legacy Logic/Weasel: планы', 'ItemStorage: прежнее имя Storage, не отдельный модуль']}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--source', type=Path, required=True)
    args = parser.parse_args()
    data = inventory(args.source)
    out = ROOT / 'docs/ORIGINAL_MECHANICS.json'
    out.write_text(json.dumps(data, ensure_ascii=False, indent=2) + '\n')
    print(json.dumps({'verified_files': data['verified_source_files'], 'java_classes': len(data['java_classes']),
                      'registered_objects': len(data['registered_objects']),
                      'recipe_declarations': len(data['recipe_declarations']),
                      'interfaces': len(data['interfaces']), 'output': str(out)}, ensure_ascii=False))


if __name__ == '__main__':
    main()
