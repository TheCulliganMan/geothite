#!/usr/bin/env python3
"""Read-only source, native-art, roster and full-parent riverfront audit.

This establishes bounded publication and static access. Actual services,
story gates, fishing, battles, animation and Continue need controller evidence.
No packs or source assets are written by this tool.
"""
import argparse
import base64
from collections import deque
from copy import deepcopy
import hashlib
import json
from pathlib import Path

from render_reference import SourceArt, load_pack

ROOT = Path(__file__).resolve().parents[3]
TOWN = 'MnGrandRapids'
ALIAS = 'mn_grand_rapids_riverfront'
FISHER = 'MN_EXTRA_GR_BANK_FISHER'
WON = 'EVENT_' + FISHER
LABELS = {FISHER + suffix for suffix in ['', 'Seen', 'Won', 'Lost', 'After', 'Declined']}


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def blocks(source):
    assert source['dimensions_metatiles'] == [24, 24]
    assert len(source['rows']) == 24
    assert all(len(row.split()) == 24 for row in source['rows'])
    return [int(word, 16) for row in source['rows'] for word in row.split()]


def check(parent, child, before, after, placement):
    old = parent['data']['maps'][TOWN]
    town = child['data']['maps'][TOWN]
    assert old['blocks'] == blocks(before)
    assert town['blocks'] == blocks(after)
    assert town['attributes'] == dict(old['attributes'], tileset_name=ALIAS)
    assert town['objects'][:-1] == old['objects']
    actor = town['objects'][-1]
    assert (actor['x'], actor['y'], actor['sprite'], actor['script']) == (25, 36, 'SPRITE_FISHER', FISHER)
    assert actor['spritemovedata'] == 'SPRITEMOVEDATA_STANDING_DOWN'
    assert actor['radius'] == 0 and actor['object_type'] == 'OBJECTTYPE_SCRIPT'
    assert town['events'] == old['events']
    assert town['map_event_section_commands'][:-1] == old['map_event_section_commands']
    assert town['map_event_section_commands'][-1]['args'] == [
        '25', '36', 'SPRITE_FISHER', 'SPRITEMOVEDATA_STANDING_DOWN', '0', '0',
        '-1', '-1', '0', 'OBJECTTYPE_SCRIPT', '0', FISHER, '-1']
    assert town['scripts'].keys() - old['scripts'].keys() == LABELS
    for label, commands in old['scripts'].items():
        assert town['scripts'][label] == commands, label
    for suffix, field in [('Seen', 'before_lines'), ('Won', 'after_lines'), ('Lost', 'loss_lines')]:
        lines = [json.loads(c['args'][0]) for c in town['scripts'][FISHER + suffix]
                 if c['command'] in {'text', 'line', 'para', 'cont'}]
        assert lines == placement[field]
        assert all(len(line) <= 18 for line in lines)
    assert town['scripts'][FISHER + 'Declined'] == [
        {'command': 'closetext', 'args': []}, {'command': 'end', 'args': []}]
    assert [c['command'] for c in town['scripts'][FISHER]] == [
        'faceplayer', 'checkevent', 'iftrue', 'opentext', 'writetext', 'yesorno',
        'iffalse', 'closetext', 'winlosstext', 'loadtrainer', 'startbattle',
        'reloadmapafterbattle', 'setevent', 'end']
    rebuilt = deepcopy(old)
    for key in ['attributes', 'blocks', 'objects', 'scripts', 'map_event_section_commands']:
        rebuilt[key] = town[key]
    parsed_fields = [
        'script_control_commands', 'script_map_commands', 'script_text_commands',
        'script_flag_commands', 'scripted_trainer_battles', 'script_text_bodies',
        'script_object_commands', 'script_runtime_commands']
    for key in parsed_fields:
        prior, current = old[key], town[key]
        if isinstance(prior, dict):
            assert {k: v for k, v in current.items() if k not in LABELS} == prior
        else:
            # The native parser can reorder whole programs. Within each program,
            # the source label, command index and every operand must stay exact.
            order = lambda v: (v['source_script'], v.get('command_index', -1))
            assert sorted((v for v in current if v['source_script'] not in LABELS), key=order) == sorted(prior, key=order), key
        rebuilt[key] = current
    assert rebuilt == town, 'unexpected native town field change'

    data = dict(parent['data'])
    for key, name, value in [
        ('maps', TOWN, town), ('map_attributes', TOWN, town['attributes']),
        ('map_blocks', old['attributes']['blocks_label'],
         base64.b64encode(bytes(town['blocks'])).decode('ascii')),
        ('npcs', TOWN, town['objects'])]:
        data[key] = dict(data[key], **{name: value})
    scripts = dict(data['map_scripts'])
    for label in LABELS:
        assert label not in scripts
        scripts[label] = town['scripts'][label]
    scripts[old['attributes']['map_events_label']] = [
        {'command': c['command'], 'args': c['args']} for c in town['map_event_section_commands']]
    data['map_scripts'] = scripts
    constants = deepcopy(data['story_event_script_constants'])
    new_constants = child['data']['story_event_script_constants']['global']
    assert new_constants.keys() - constants['global'].keys() == {FISHER, WON}
    assert new_constants[FISHER] == 1
    assert new_constants[WON] not in constants['global'].values()
    constants['global'].update({name: new_constants[name] for name in [FISHER, WON]})
    data['story_event_script_constants'] = constants
    trainers = deepcopy(data['trainers'])
    trainer = child['data']['trainers']['trainers'][FISHER]
    assert FISHER not in trainers['trainers']
    assert trainer['name'] == placement['display_name'] and trainer['trainer_class'] == 'FISHER'
    assert [(p['species'], p['level']) for p in trainer['party']] == [
        (p['species'], p['level']) for p in placement['party']]
    trainers['trainers'][FISHER] = trainer
    data['trainers'] = trainers

    files = child['runtime_files']
    additions = {f'data/tilesets/{ALIAS}{suffix}' for suffix in
                 ['.json', '_metatiles.bin', '_palette_map.json']}
    additions.update(f'gfx/tilesets/{ALIAS}{suffix}' for suffix in ['.2bpp', '.png'])
    assert files.keys() - parent['runtime_files'].keys() == additions
    assert all(files[k] == v for k, v in parent['runtime_files'].items())
    alias = child['data']['tilesets'][ALIAS]
    original = data['tilesets']['johto_modern']
    assert json.loads(bytes(files[f'data/tilesets/{ALIAS}.json'])) == alias['collision']
    assert json.loads(bytes(files[f'data/tilesets/{ALIAS}_palette_map.json'])) == alias['palette_map']
    assert {k: v for k, v in alias['collision'].items() if int(k, 16) < 128} == original['collision']
    assert len(alias['collision']) == 131
    a, b = SourceArt(parent), SourceArt(child)
    for time in ['morn', 'day', 'nite']:
        for actual, expected in zip(b.blocks(ALIAS, time)[:128], a.blocks('johto_modern', time)):
            assert actual.tobytes() == expected.tobytes(), 'inherited native block pixels changed'
        for recipe in after['derived_blocks']:
            destination, source = int(recipe['id'], 16), int(recipe['source_block'], 16)
            assert b.blocks(ALIAS, time)[destination].tobytes() == a.blocks('port', time)[source].tobytes()
            assert alias['collision'][recipe['id']] == parent['data']['tilesets']['port']['collision'][recipe['source_block']]
    layout = bytes(files[f'data/tilesets/{ALIAS}_metatiles.bin'])
    assert layout[:2048] == bytes(parent['runtime_files']['data/tilesets/johto_modern_metatiles.bin'])
    port_layout = bytes(parent['runtime_files']['data/tilesets/port_metatiles.bin'])
    for recipe in after['derived_blocks']:
        destination, source = int(recipe['id'], 16), int(recipe['source_block'], 16)
        for i in range(16):
            if port_layout[source * 16 + i] == 0x14:
                assert layout[destination * 16 + i] == 0x14, 'static water imported at pier'
    data['tilesets'] = dict(data['tilesets'], **{ALIAS: alias})
    field_moves = deepcopy(data['field_moves'])
    field_moves['cut']['replacements'][ALIAS] = field_moves['cut']['replacements']['johto_modern']
    data['field_moves'] = field_moves
    assert data == child['data'], ('unapproved content outside the town, art or optional trainer',
                                   [key for key in data.keys() | child['data'].keys()
                                    if data.get(key) != child['data'].get(key)])
    report = dict(parent['report'])
    report['manifests'] = [*report['manifests'],
                           'minnesota-grand-rapids-riverfront-native-v1-expanded-city-art',
                           'minnesota-grand-rapids-riverfront-v1']
    assert child['report'] == report
    for key in parent.keys() | child.keys():
        if key not in {'identity', 'data', 'runtime_files', 'report'}:
            assert child.get(key) == parent.get(key), key

    curation = json.loads((ROOT / 'planning/minnesota/pokemon/roster-curation.json').read_text())
    roster = json.loads((ROOT / 'planning/minnesota/pokemon/roster.json').read_text())
    retired = {f['id'] for f in curation['retired_families']}
    active = {m['name'].upper() for f in roster['families'] if f['id'] not in retired for m in f['members']}
    relevant = {p['species'] for p in trainer['party']}
    for rod in data['fishing']['groups']['FISHGROUP_MN_SHORE']['rod_tables'].values():
        for slot in rod['slots']:
            if slot['species']:
                relevant.add(slot['species'])
            elif slot['time_group']:
                for timed in data['fishing']['time_groups'][slot['time_group']].values():
                    if isinstance(timed, dict) and timed.get('species'):
                        relevant.add(timed['species'])
    pending = list(relevant)
    while pending:
        species = pending.pop()
        assert species in active, species
        for edge in data['evolutions'].get(species, []):
            target = edge['species']
            if target not in relevant:
                relevant.add(target)
                pending.append(target)

    kinds = {(x, y): alias['collision'][f'{town["blocks"][y//2*24+x//2]:02x}'][y % 2 * 2 + x % 2]
             for y in range(48) for x in range(48)}
    bodies = {(o['x'], o['y']) for o in town['objects']}
    floor = {p for p, kind in kinds.items() if kind in {'FLOOR', 'DOOR'}} - bodies
    reached, queue = {(6, 24)}, deque([(6, 24)])
    while queue:
        x, y = queue.popleft()
        for point in [(x-1, y), (x+1, y), (x, y-1), (x, y+1)]:
            if point in floor and point not in reached:
                reached.add(point)
                queue.append(point)
    assert floor - reached == {(x, y) for x in [0, 1, 46, 47] for y in [24, 25]}
    assert all(tuple(v['tile']) in reached for v in after['review_contract']['viewpoints'])
    assert all(kinds[x, y] not in {'WATER', 'WATERFALL'} for x, y in kinds
               if x in [0, 47] or y in [0, 47])
    assert kinds[26, 38] == 'FLOOR' and kinds[26, 39] == 'WATER'
    return dict(unchanged_maps=len(data['maps'])-1, reachable_ordinary_tiles=len(reached),
                original_guard_arrival_pockets=8, original_warps_preserved=len(town['events']['warps']),
                native_art_times_compared=3, additional_native_blocks=3,
                regional_species_and_evolution_closure=sorted(relevant),
                audio_and_all_original_runtime_files_exact=True)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ['parent', 'child', 'source', 'output']:
        parser.add_argument('--' + name, type=Path, required=True)
    args = parser.parse_args()
    assert not args.output.exists(), 'retain all earlier audit results'
    read = lambda name: json.loads((args.source / name).read_text())
    result = check(load_pack(args.parent), load_pack(args.child), read('before.json'),
                   read('after.json'), read('trainers.json')['trainers'][0])
    result.update(status='PASS', parent=str(args.parent), parent_sha256=digest(args.parent),
                  child=str(args.child), child_sha256=digest(args.child),
                  scope=__doc__, sources={name: digest(args.source/name) for name in
                                         ['before.json', 'after.json', 'contract.json', 'trainers.json']})
    args.output.write_text(json.dumps(result, indent=2) + '\n')
    print(json.dumps(result, indent=2))


if __name__ == '__main__':
    main()
