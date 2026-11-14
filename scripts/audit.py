#!/usr/bin/env python3
"""Structural/source audit; does not type-check Rust or run Horizon."""
import argparse
import hashlib
import json
import re
import sys
import tomllib
from pathlib import Path

from textures import textures

ROOT = Path(__file__).resolve().parents[1]


class AuditError(ValueError):
    pass


def require(condition, message):
    if not condition:
        raise AuditError(message)


def blob(data):
    return hashlib.sha1(f'blob {len(data)}\0'.encode() + data).hexdigest()


def read_json(path):
    def unique(pairs):
        result = {}
        for key, value in pairs:
            require(key not in result, f'{path}: duplicate key {key}')
            result[key] = value
        return result
    return json.loads(path.read_text(), object_pairs_hook=unique)


def validate(root=ROOT, sources=None):
    lock = read_json(root / 'upstream.lock.json')
    compat = read_json(root / 'native/powercraft2/compatibility.json')
    bundle = root / 'native/powercraft2/mixins/powercraft2'
    manifest = tomllib.loads((bundle / 'horizon-mixin.toml').read_text())
    cargo = tomllib.loads((root / 'Cargo.toml').read_text())
    content = root / 'content'
    cm = tomllib.loads((content / 'horizon-mod.toml').read_text())
    require(re.fullmatch(r'[a-z0-9_]{1,64}', cm['id']), 'invalid mod id')
    require(cm['id'] == manifest['mod']['id'] == cargo['package']['name'] == 'powercraft2', 'id mismatch')
    require(manifest['mod']['version'] == cargo['package']['version'], 'native version mismatch')
    require(cm['version'] == cargo['package']['version'], 'content/native release version mismatch')
    require(cm['side'] == 'both' and cm['api'] == '0.2', 'content must target both sides/API 0.2')
    require(cm['capabilities'] == ['registries'], 'unexpected capabilities')
    for key in ('horizon_revision', 'horizon_mixins_revision'):
        require(re.fullmatch(r'[0-9a-f]{40}', compat[key]), f'invalid pin {key}')
    revision = compat['horizon_revision']
    require(revision == manifest['compatibility']['horizon_revision'] == lock['horizon']['revision'], 'Horizon pin mismatch')
    require(compat['horizon_mixins_revision'] == lock['mixins']['revision'], 'Mixins pin mismatch')
    require(compat['game'] == lock['horizon']['game'] == '26.3', 'game mismatch')
    require(compat['hooks'] == 381 <= lock['horizon']['hooks_available'], 'hook API mismatch')
    module, = manifest['module']
    require(module['name'] == 'powercraft2_transport', 'foreign module name')
    require(module['file'] == 'crates/horizon-client/src/game.rs', 'unexpected destination')
    require((bundle / module['source']).resolve() == (root / cargo['lib']['path']).resolve(), 'runtime source drift')
    evidence = read_json(root / 'docs/API_EVIDENCE.json')
    require(evidence['horizon_revision'] == revision, 'stale API evidence')
    available = {h['name']: h for h in evidence['hooks']}
    expected = {'tick', 'entity_tick_listened', 'entity_tick_post', 'state_for_placement'}
    require({i['hook'] for i in manifest['injection']} == expected, 'unexpected hooks')
    require(len(manifest['injection']) == len(expected), 'duplicate hook')
    for injection in manifest['injection']:
        require(injection['kind'] == 'HEAD' and 'target' not in injection, 'only named HEAD hooks permitted')
        require(injection['hook_version'] == available[injection['hook']]['version'], 'hook version mismatch')
        require((bundle / injection['helper']).is_file(), 'missing helper')
    for path in content.rglob('*'):
        if path.relative_to(content).parts[0] == 'target':
            continue  # Official cargo horizon-mod build output, never input.
        require(not path.is_symlink(), f'symlink: {path}')
        if not path.is_file() or path == content / 'horizon-mod.toml':
            continue
        parts = path.relative_to(content).parts
        require(parts[0] in ('assets', 'data') and parts[1] == 'powercraft2', f'foreign namespace: {path}')
        if path.suffix == '.json':
            read_json(path)
    block = read_json(content / 'data/powercraft2/horizon/block/conveyor.json')
    require(block['template'] == 'minecraft:iron_block' and block['item'] is True, 'block registration changed')
    facing = block['state_properties']['facing']
    directions = {'north', 'east', 'south', 'west'}
    require(facing['type'] == 'enum' and set(facing['values']) == directions, 'invalid facing schema')
    require(len(facing['values']) == 4 and block['default_state']['facing'] in directions, 'invalid facing default')
    shape = [[0, 0, 0, 1, 0.25, 1]]
    require(block['shapes']['default'] == block['collision_shapes']['default'] == shape, 'surface mismatch')
    variants = read_json(content / 'assets/powercraft2/blockstates/conveyor.json')['variants']
    require(set(variants) == {f'facing={d}' for d in directions}, 'missing block model variant')
    for direction, angle in [('north', 0), ('east', 90), ('south', 180), ('west', 270)]:
        variant = variants[f'facing={direction}']
        require(variant['model'] == 'powercraft2:block/conveyor' and variant.get('y', 0) == angle, 'wrong model rotation')
    model = read_json(content / 'assets/powercraft2/models/block/conveyor.json')
    require(model['elements'][0]['to'] == [16, 4, 16], 'visual/collision mismatch')
    recipe = read_json(content / 'data/powercraft2/recipe/conveyor.json')
    require(recipe['result'] == {'id': 'powercraft2:conveyor', 'count': 4}, 'recipe result mismatch')
    for name, data in textures().items():
        require((content / 'assets/powercraft2/textures/block' / name).read_bytes() == data, 'texture provenance mismatch')
    for name in ('speedy_conveyor', 'brake_conveyor'):
        definition = read_json(content / f'data/powercraft2/horizon/block/{name}.json')
        require(definition['item'] is True and definition['template'] == block['template'], 'variant registration mismatch')
        require(definition['state_properties']['facing'] == facing, 'variant facing mismatch')
        require(definition['shapes']['default'] == definition['collision_shapes']['default'] == shape, 'variant surface mismatch')
        require(definition['default_state']['facing'] == 'north', 'variant default mismatch')
        states = read_json(content / f'assets/powercraft2/blockstates/{name}.json')['variants']
        if name == 'brake_conveyor':
            require(definition['state_properties']['powered'] == {'type': 'boolean'}, 'invalid brake powered schema')
            require(definition['default_state']['powered'] == 'false', 'invalid brake default power')
            expected_states = {f'facing={d},powered={power}' for d in directions for power in ('false', 'true')}
        else:
            expected_states = {f'facing={d}' for d in directions}
        require(set(states) == expected_states, 'missing variant blockstate')
        for state, variant in states.items():
            direction = state.split(',')[0].split('=')[1]
            suffix = '_powered' if 'powered=true' in state else ''
            require(variant['model'] == f'powercraft2:block/{name}{suffix}', 'wrong variant model')
            require(variant.get('y', 0) == {'north': 0, 'east': 90, 'south': 180, 'west': 270}[direction], 'wrong variant rotation')
            model_path = content / f'assets/powercraft2/models/block/{name}{suffix}.json'
            require(read_json(model_path)['elements'][0]['to'] == [16, 4, 16], 'variant visual/collision mismatch')
        recipe = read_json(content / f'data/powercraft2/recipe/{name}.json')
        require(recipe['result']['id'] == f'powercraft2:{name}', 'variant recipe mismatch')
        loot = read_json(content / f'data/powercraft2/loot_table/blocks/{name}.json')
        require(loot['pools'][0]['entries'][0]['name'] == f'powercraft2:{name}', 'variant loot mismatch')
    tab = read_json(content / 'data/powercraft2/horizon/creative_tab/transport.json')
    require(tab['items'] == ['powercraft2:conveyor', 'powercraft2:speedy_conveyor', 'powercraft2:brake_conveyor'], 'transport tab mismatch')
    provenance = read_json(root / 'docs/TOOL_PROVENANCE.json')
    require(provenance['revision'] == lock['mods']['revision'], 'tools pin mismatch')
    for file in provenance['files']:
        require(blob((root / file['path']).read_bytes()) == file['git_blob'], f'modified official tool: {file["path"]}')
    if sources is not None:
        for file in evidence['source_files']:
            require(blob((sources / file['path']).read_bytes()) == file['git_blob'], f'source mismatch: {file["path"]}')
    return {'structural_audit': 'passed', 'named_hooks': sorted(expected),
            'pinned_source_files_checked': len(evidence['source_files']) if sources is not None else 0,
            'rust_compilation': 'not checked', 'in_game': 'not checked'}


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--sources', type=Path, help='pristine Horizon export at the pin; verify Git blob hashes')
    args = parser.parse_args()
    try:
        print(json.dumps(validate(sources=args.sources), indent=2))
    except (AuditError, KeyError, ValueError, OSError) as error:
        print(f'Audit failed: {error}', file=sys.stderr)
        sys.exit(1)
