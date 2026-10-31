#!/usr/bin/env python3
"""Read-only field-branch preservation and explicit mutation-rejection audit."""
import argparse
import base64
from copy import deepcopy
import json
from pathlib import Path
from render_reference import load_pack

PARENT = 'MnR16C01R00'
FIELD = 'MnFieldMargin'
CONSTANT = 'MN_FIELD_MARGIN'
FLAGS = {'EVENT_MN_KENSIGARD_' + key for key in [
    'CARETAKER_READ', 'SOIL_READ', 'BURIED_EDGE_READ', 'GROOVES_FIRST_READ',
    'GROOVES_REVISIT_READY', 'CHANGING_GROOVES_READ', 'DIRECTIONS_ACKNOWLEDGED',
    'BOUNDARY_LOCATED', 'BOUNDARY_REPORTED', 'NOTICE_CORRECTED', 'REVEALED', 'CAUGHT']}
ROOT = Path(__file__).resolve().parents[3]


def check(parent, child, site):
    old, new = parent['data'], child['data']
    assert new['maps'].keys() - old['maps'].keys() == {FIELD}
    outside, original, field = new['maps'][PARENT], old['maps'][PARENT], new['maps'][FIELD]
    attributes = deepcopy(original['attributes'])
    assert not any(c['direction'].lower() == 'north' for c in attributes['connections'])
    attributes['connections'].append(dict(direction='north', target_map=FIELD, offset=3))
    attributes['connection_flags'] += ' | NORTH'
    assert outside['attributes'] == attributes
    blocks = list(original['blocks'])
    assert blocks[11] == 5
    blocks[11] = 2
    assert outside['blocks'] == blocks
    assert outside['objects'][:-1] == original['objects']
    caretaker = outside['objects'][-1]
    assert (caretaker['script'], caretaker['sprite'], caretaker['x'], caretaker['y']) == (
        'MnKensigardCaretaker', 'SPRITE_POKEFAN_M', 23, 5)
    assert outside['events'] == original['events']
    assert all(outside['scripts'].get(k) == v for k, v in original['scripts'].items())
    assert all(outside['script_text_bodies'].get(k) == v for k, v in original['script_text_bodies'].items())
    assert all(k.startswith('MnKensigard') for k in outside['scripts'].keys() - original['scripts'].keys())
    assert (field['attributes']['width'], field['attributes']['height'], field['attributes']['tileset_name']) == (
        14, 16, 'mn_prairie_headwaters')
    assert field['attributes']['connections'] == [dict(direction='south', target_map=PARENT, offset=-3)]
    assert field['attributes']['connection_flags'] == 'SOUTH'
    assert field['blocks'] == [int(v, 16) for row in site['field']['rows'] for v in row.split()]
    collision = new['tilesets'][field['attributes']['tileset_name']]['collision']
    def kind(x, y):
        block = field['blocks'][y // 2 * 14 + x // 2]
        return collision[f'{block:02x}'][y % 2 * 2 + x % 2]
    seen, pending = {(16, 31)}, [(16, 31)]
    while pending:
        x, y = pending.pop()
        for point in [(x-1, y), (x+1, y), (x, y-1), (x, y+1)]:
            if (0 <= point[0] < 28 and 0 <= point[1] < 32 and point not in seen
                    and point != (13, 5) and kind(*point) in ['FLOOR', 'TALL_GRASS']):
                seen.add(point)
                pending.append(point)
    assert len(seen) == 399
    for read in site['field']['reads'].values():
        assert kind(*read['at']) == 'WALL' and kind(*read['approach']) == 'FLOOR'
        assert tuple(read['approach']) in seen
    assert (13, 6) in seen and (16, 30) in seen
    for y in range(32):
        for x in range(28):
            assert kind(x, y) not in ['CUT_TREE', 'WATER', 'BUOY']
            if x in [0, 27] or y in [0, 31]:
                assert kind(x, y) == 'WALL' or (y == 31 and x in [16, 17])
    assert len(field['objects']) == 1
    actor = field['objects'][0]
    assert (actor['script'], actor['sprite'], actor['x'], actor['y']) == (
        'MN_MARGIN_KENSIGARD', 'SPRITE_KENSIGARD', 13, 5)
    assert not field['events']['warps']
    assert field['events']['coord_events'] == [dict(x=13, y=6, scene_id='-1', script_name='MnKensigardReveal')]
    assert field['events']['bg_events'] == [dict(x=r['at'][0], y=r['at'][1], event_type='BGEVENT_READ', script=r['script'])
                                          for r in site['field']['reads'].values()]
    wild = [(command['args'][0], command['args'][1]) for program in field['scripts'].values()
            if isinstance(program, list) for command in program if command['command'] == 'loadwildmon']
    assert wild == [('KENSIGARD', '50')]
    assert not any(command['command'] in ['givepoke', 'giveegg'] for program in field['scripts'].values()
                   if isinstance(program, list) for command in program)
    expected_encounters = deepcopy(old['wild_encounters'][PARENT])
    expected_encounters['map_name'] = FIELD
    assert new['wild_encounters'][FIELD] == expected_encounters
    for module in [outside, field]:
        name = module['id']
        assert new['npcs'][name] == module['objects']
        assert new['map_attributes'][name] == module['attributes']
        assert new['map_blocks'][module['attributes']['blocks_label']] == base64.b64encode(bytes(module['blocks'])).decode()
        programs = dict(module['scripts'])
        for attr, section in [('map_scripts_label', 'map_script_section_commands'), ('map_events_label', 'map_event_section_commands')]:
            programs[module['attributes'][attr]] = [dict(command=row['command'], args=row['args']) for row in module[section]]
        assert all(new['map_scripts'][label] == commands for label, commands in programs.items())
    meta = deepcopy(old['runtime_map_metadata'][original['attributes']['map_constant']])
    meta.update(name=FIELD, constant=CONSTANT, width=14, height=16, environment=field['attributes']['environment'])
    meta['mapId'] = max(m['mapId'] for m in old['runtime_map_metadata'].values() if m['groupId'] == meta['groupId']) + 1
    assert new['runtime_map_metadata'][CONSTANT] == meta
    before, after = old['story_event_script_constants']['global'], new['story_event_script_constants']['global']
    assert after.keys() - before.keys() == FLAGS
    assert all(after.get(k) == v for k, v in before.items())
    ids = {after[k] for k in FLAGS}
    assert len(ids) == 12 and not ids & {v for k, v in before.items() if k.startswith('EVENT_')}
    report = deepcopy(parent['report'])
    report['maps'] += 1
    report['reachable_maps'].append(FIELD)
    report['solvable_maps'].append(FIELD)
    report['manifests'].extend(['northstar-lair-mnfieldmargin', 'minnesota-kensigard-field-margin-v1'])
    assert child['report'] == report
    normalized = deepcopy(child)
    data = normalized['data']
    for table, key in [('maps', FIELD), ('map_attributes', FIELD), ('map_dimensions', CONSTANT),
                       ('map_blocks', field['attributes']['blocks_label']), ('runtime_map_metadata', CONSTANT),
                       ('npcs', FIELD), ('wild_encounters', FIELD)]:
        del data[table][key]
    assert data['pokegear_landmarks']['map_to_landmark'].pop(FIELD) == old['pokegear_landmarks']['map_to_landmark'][PARENT]
    for table, key in [('maps', PARENT), ('map_attributes', PARENT), ('map_blocks', original['attributes']['blocks_label']), ('npcs', PARENT)]:
        data[table][key] = deepcopy(old[table][key])
    labels = set(outside['scripts']) | set(field['scripts'])
    for module in [outside, field]:
        for attr in ['map_scripts_label', 'map_events_label']:
            labels.add(module['attributes'][attr])
    assert new['map_scripts'].keys() - old['map_scripts'].keys() == labels - old['map_scripts'].keys()
    for label in labels:
        if label in old['map_scripts']:
            data['map_scripts'][label] = deepcopy(old['map_scripts'][label])
        else:
            del data['map_scripts'][label]
    for flag in FLAGS:
        del data['story_event_script_constants']['global'][flag]
    normalized['report'], normalized['identity'] = parent['report'], parent['identity']
    assert normalized == parent, 'unrelated world, scripts, encounters, art, species or audio changed'


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('parent')
    parser.add_argument('child')
    args = parser.parse_args()
    parent, child = load_pack(args.parent), load_pack(args.child)
    site = json.loads((ROOT / 'planning/minnesota/pokemon/acquisition-v01/kensigard/site.json').read_text())
    check(parent, child, site)
    mutations = [
        ('old route', lambda p: p['data']['maps'][PARENT]['blocks'].__setitem__(16, 2)),
        ('old script', lambda p: p['data']['map_scripts'].__setitem__('AUDIT_FOREIGN_PROGRAM', [])),
        ('foreign field', lambda p: p['data']['wild_encounters'][FIELD].__setitem__('map_name', 'Johto')),
        ('seam', lambda p: p['data']['maps'][FIELD]['attributes']['connections'][0].__setitem__('offset', 0)),
        ('species', lambda p: p['data']['pokemon'].__setitem__('AUDIT_FOREIGN', {})),
        ('audio', lambda p: p.__setitem__('audio_manifest', None)),
    ]
    for label, mutate in mutations:
        changed = deepcopy(child)
        mutate(changed)
        try:
            check(parent, changed, site)
        except (AssertionError, KeyError, TypeError):
            pass
        else:
            raise AssertionError('accepted mutation: ' + label)
        del changed
    print(f'PASS: {len(parent["data"]["maps"])-1} unrelated complete maps, route actors/travel, original art/species/audio exact; six mutations rejected')


if __name__ == '__main__':
    main()
