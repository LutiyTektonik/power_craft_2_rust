#!/usr/bin/env python3
"""Structural/source audit; does not type-check Rust or run Horizon."""
import argparse
import hashlib
import json
import re
import sys
import tomllib
from pathlib import Path

from textures import item_textures, textures

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
    modules = manifest['module']
    expected_modules = {'transport', 'separator', 'ejector', 'elevator', 'splitter',
                        'player_transport', 'deco', 'xp_bank', 'xp_bank_logic'}
    require(len(modules) == len(expected_modules)
            and {module['id'] for module in modules} == expected_modules, 'unexpected native modules')
    for module in modules:
        require(module['name'] == f"powercraft2_{module['id']}"
                and module['file'] == 'crates/horizon-client/src/game.rs'
                and (bundle / module['source']).is_file(), 'invalid native module')
    transport = next(module for module in modules if module['id'] == 'transport')
    require((bundle / transport['source']).resolve() == (root / cargo['lib']['path']).resolve(),
            'runtime source drift')
    evidence = read_json(root / 'docs/API_EVIDENCE.json')
    require(evidence['horizon_revision'] == revision, 'stale API evidence')
    available = {h['name']: h for h in evidence['hooks']}
    expected = {'tick', 'entity_tick_listened', 'entity_tick_post', 'state_for_placement',
                'block_entity', 'block_entity_ticks', 'block_entity_tick', 'block_signal', 'is_signal_source',
                'block_entity_container', 'screen', 'menu_slot_rules', 'menu_button',
                'screen_click', 'screen_draw', 'update_shape', 'player_tick', 'loot_function', 'loot_drops'}
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
    for name in ('speedy_conveyor', 'brake_conveyor', 'detector_conveyor', 'redirector_conveyor', 'separator_conveyor', 'ejector_conveyor'):
        definition = read_json(content / f'data/powercraft2/horizon/block/{name}.json')
        inherits_facing = name in ('separator_conveyor', 'ejector_conveyor')
        template = 'minecraft:barrel' if inherits_facing else block['template']
        require(definition['item'] is True and definition['template'] == template, 'variant registration mismatch')
        if inherits_facing:
            require('facing' not in definition.get('state_properties', {}),
                    'container belt inherits facing from the barrel template')
        else:
            require(definition['state_properties']['facing'] == facing, 'variant facing mismatch')
        require(definition['shapes']['default'] == definition['collision_shapes']['default'] == shape, 'variant surface mismatch')
        require(definition['default_state']['facing'] == 'north', 'variant default mismatch')
        states = read_json(content / f'assets/powercraft2/blockstates/{name}.json')['variants']
        if name in ('brake_conveyor', 'detector_conveyor', 'redirector_conveyor'):
            power_name = name.removesuffix('_conveyor')
            require(definition['state_properties']['powered'] == {'type': 'boolean'}, f'invalid {power_name} powered schema')
            require(definition['default_state']['powered'] == 'false', f'invalid {power_name} default power')
            expected_states = {f'facing={d},powered={power}' for d in directions for power in ('false', 'true')}
        else:
            expected_states = {f'facing={d}' for d in directions}
        if inherits_facing:
            expected_states |= {'facing=up', 'facing=down'}
        require(set(states) == expected_states, 'missing variant blockstate')
        for state, variant in states.items():
            direction = state.split(',')[0].split('=')[1]
            suffix = '_powered' if 'powered=true' in state else ''
            require(variant['model'] == f'powercraft2:block/{name}{suffix}', 'wrong variant model')
            require(variant.get('y', 0) == {'north': 0, 'east': 90, 'south': 180, 'west': 270, 'up': 0, 'down': 0}[direction], 'wrong variant rotation')
            model_path = content / f'assets/powercraft2/models/block/{name}{suffix}.json'
            require(read_json(model_path)['elements'][0]['to'] == [16, 4, 16], 'variant visual/collision mismatch')
        recipe = read_json(content / f'data/powercraft2/recipe/{name}.json')
        require(recipe['result']['id'] == f'powercraft2:{name}', 'variant recipe mismatch')
        if name == 'redirector_conveyor':
            require(recipe['pattern'] == ['C', 'R']
                    and recipe['key'] == {'C': 'powercraft2:conveyor', 'R': 'minecraft:redstone'}
                    and recipe['result']['count'] == 1, 'published redirector recipe mismatch')
        if name == 'separator_conveyor':
            require(recipe['pattern'] == ['D', 'C', 'R']
                    and recipe['key'] == {'D': 'minecraft:diamond', 'C': 'powercraft2:conveyor',
                                          'R': 'minecraft:redstone'}
                    and recipe['result']['count'] == 1, 'published separator recipe mismatch')
        if name == 'ejector_conveyor':
            require(definition['state_properties'] == {'powered': {'type': 'boolean'}}
                    and definition['default_state']['powered'] == 'false',
                    'invalid ejector powered schema')
            require(recipe['pattern'] == ['B', 'C', 'R']
                    and recipe['key'] == {'B': 'minecraft:bow', 'C': 'powercraft2:conveyor',
                                          'R': 'minecraft:redstone'}
                    and recipe['result']['count'] == 1, 'published ejector recipe mismatch')
        loot = read_json(content / f'data/powercraft2/loot_table/blocks/{name}.json')
        require(loot['pools'][0]['entries'][0]['name'] == f'powercraft2:{name}', 'variant loot mismatch')
    for direction in ('up', 'down'):
        name = f'elevator_{direction}'
        identifier = f'powercraft2:{name}'
        definition = read_json(content / f'data/powercraft2/horizon/block/{name}.json')
        require(definition['item'] is True and definition['template'] == block['template'],
                'elevator registration mismatch')
        schema = {'powered': {'type': 'boolean'}}
        defaults = {'powered': 'false'}
        collision = {'default': []}
        variants = {'': {'model': f'powercraft2:block/{name}'}}
        key = {'C': 'powercraft2:conveyor', 'G': 'minecraft:gold_ingot'}
        middle = 'C C'
        if direction == 'down':
            schema['base'] = {'type': 'boolean'}
            defaults['base'] = 'true'
            collision.update(properties=['base'], values={'true': shape})
            variants = {'base=false': {'model': f'powercraft2:block/{name}'},
                        'base=true': {'model': f'powercraft2:block/{name}_base'}}
            key['R'] = 'minecraft:redstone'
            middle = 'CRC'
        require(definition['state_properties'] == schema and definition['default_state'] == defaults,
                'elevator state mismatch')
        require(definition['collision_shapes'] == collision, 'elevator collision mismatch')
        actual = read_json(content / f'assets/powercraft2/blockstates/{name}.json')['variants']
        require(actual == variants, 'elevator blockstate mismatch')
        for state in actual.values():
            model = state['model'].removeprefix('powercraft2:')
            require((content / f'assets/powercraft2/models/{model}.json').is_file(),
                    'missing elevator model')
        recipe = read_json(content / f'data/powercraft2/recipe/{name}.json')
        require(recipe['pattern'] == ['CGC', middle, 'CGC'] and recipe['key'] == key
                and recipe['result'] == {'id': identifier, 'count': 6}, 'published elevator recipe mismatch')
        loot = read_json(content / f'data/powercraft2/loot_table/blocks/{name}.json')
        require(loot['pools'][0]['entries'][0]['name'] == identifier, 'wrong elevator drop')
    tab = read_json(content / 'data/powercraft2/horizon/creative_tab/transport.json')
    require(tab['items'] == ['powercraft2:conveyor', 'powercraft2:speedy_conveyor', 'powercraft2:brake_conveyor', 'powercraft2:detector_conveyor', 'powercraft2:redirector_conveyor', 'powercraft2:separator_conveyor', 'powercraft2:ejector_conveyor', 'powercraft2:elevator_up', 'powercraft2:elevator_down', 'powercraft2:splitter', 'powercraft2:sticky_boots'], 'transport tab mismatch')
    menu = read_json(content / 'data/powercraft2/horizon/menu/separator_conveyor.json')
    require(menu == {'slots': 18}, 'separator menu must have two banks of nine slots')
    menu = read_json(content / 'data/powercraft2/horizon/menu/ejector_conveyor.json')
    require(menu == {'slots': 1, 'data_slots': 4}, 'ejector menu settings mismatch')
    splitter_block = read_json(content / 'data/powercraft2/horizon/block/splitter.json')
    require(splitter_block['template'] == 'minecraft:barrel' and splitter_block['item'] is True
            and splitter_block['collision_shapes'] == {'default': []}
            and 'state_properties' not in splitter_block, 'splitter block registration mismatch')
    splitter_model = 'powercraft2:block/splitter'
    variants = read_json(content / 'assets/powercraft2/blockstates/splitter.json')['variants']
    require(variants == {'': {'model': splitter_model}}, 'splitter block model mismatch')
    item = read_json(content / 'assets/powercraft2/items/splitter.json')
    require(item['model']['model'] == splitter_model, 'splitter item model mismatch')
    model = read_json(content / 'assets/powercraft2/models/block/splitter.json')
    require(len(model['elements']) == len(splitter_block['shapes']['default']) == 10,
            'splitter shape mismatch')
    recipe = read_json(content / 'data/powercraft2/recipe/splitter.json')
    require(recipe['pattern'] == [' U ', 'CSC', ' D ']
            and recipe['key'] == {'U': 'powercraft2:elevator_up', 'C': 'powercraft2:conveyor',
                                  'S': 'powercraft2:separator_conveyor', 'D': 'powercraft2:elevator_down'}
            and recipe['result'] == {'id': 'powercraft2:splitter', 'count': 1},
            'published splitter recipe mismatch')
    loot = read_json(content / 'data/powercraft2/loot_table/blocks/splitter.json')
    require(loot['pools'][0]['entries'][0]['name'] == 'powercraft2:splitter', 'wrong splitter drop')
    menu = read_json(content / 'data/powercraft2/horizon/menu/splitter.json')
    require(menu == {'slots': 30}, 'splitter menu must have six banks of five slots')
    boots = read_json(content / 'data/powercraft2/horizon/item/sticky_boots.json')
    require(boots == {'template': 'minecraft:iron_boots'}, 'sticky boots equipment mismatch')
    recipe = read_json(content / 'data/powercraft2/recipe/sticky_boots.json')
    require(recipe['pattern'] == ['B', 'S', 'B']
            and recipe['key'] == {'B': 'minecraft:iron_boots', 'S': 'minecraft:slime_ball'}
            and recipe['result'] == {'id': 'powercraft2:sticky_boots', 'count': 1},
            'published sticky boots recipe mismatch')
    boots_model = 'powercraft2:item/sticky_boots'
    item = read_json(content / 'assets/powercraft2/items/sticky_boots.json')
    model = read_json(content / 'assets/powercraft2/models/item/sticky_boots.json')
    require(item['model']['model'] == boots_model and model['textures']['layer0'] == boots_model,
            'sticky boots model mismatch')
    for name, data in item_textures().items():
        require((content / 'assets/powercraft2/textures/item' / name).read_bytes() == data,
                'item texture provenance mismatch')
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
