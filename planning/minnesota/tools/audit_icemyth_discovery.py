#!/usr/bin/env python3
"""Independent bounded exhibit/service-room audit; never writes a pack."""
import argparse
import base64
from copy import deepcopy
import json
from pathlib import Path
from render_reference import load_pack

HALL = 'MnStPaulFairExhibitionHall'
ROOM = 'MnIcemythServicePassage'
CONSTANT = 'MN_ICEMYTH_SERVICE_PASSAGE'
FLAGS = {'EVENT_MN_ICEMYTH_' + suffix for suffix in [
    'ATTENDANT_ACCOUNT', 'DELIVERY_ACCOUNT', 'VISITOR_ACCOUNT', 'LATCH_READ',
    'TRAY_READ', 'LABEL_CORRECTED', 'MOVED_ICE_FOUND', 'CAUGHT']}
ROOT = Path(__file__).resolve().parents[3]


def warp(index, x, y, target, destination):
    return dict(index=index, x=x, y=y, target_map_constant=target,
                target_map=target, target_warp_id=destination)


def check(parent, child, site):
    old, new = parent['data'], child['data']
    assert ROOM not in old['maps'] and new['maps'].keys() - old['maps'].keys() == {ROOM}
    hall, original, room = new['maps'][HALL], old['maps'][HALL], new['maps'][ROOM]
    assert hall['attributes'] == original['attributes']
    expected_blocks = list(original['blocks'])
    assert expected_blocks[38] == 15
    expected_blocks[38] = 37
    assert hall['blocks'] == expected_blocks
    assert hall['objects'][:len(original['objects'])] == original['objects']
    added = hall['objects'][len(original['objects']):]
    assert len(added) == 4
    assert {(o['script'], o['sprite'], o['x'], o['y']) for o in added} == {
        ('MnIcemythAttendant', 'SPRITE_POKEFAN_M', 10, 8),
        ('MnIcemythDelivery', 'SPRITE_SAILOR', 7, 8),
        ('MnIcemythVisitor', 'SPRITE_LASS', 4, 8),
        ('MN_EXHIBIT_FROZEN_CASE', 'SPRITE_ICEMYTH', 10, 7)}
    hall_constant = original['attributes']['map_constant']
    assert hall['events']['warps'] == original['events']['warps'] + [
        warp(4, 12, 8, CONSTANT, 2), warp(5, 13, 8, hall_constant, 5)]
    assert hall['events']['coord_events'] == original['events']['coord_events']
    assert hall['events']['bg_events'] == original['events']['bg_events'] + [
        dict(x=9, y=8, event_type='BGEVENT_READ', script='MnIcemythLatch'),
        dict(x=13, y=7, event_type='BGEVENT_READ', script='MnIcemythTray')]
    assert all(hall['scripts'].get(k) == v for k, v in original['scripts'].items())
    assert all(hall['script_text_bodies'].get(k) == v for k, v in original['script_text_bodies'].items())
    assert all(label.startswith(('MnIcemyth', 'MN_EXHIBIT_'))
               for label in hall['scripts'].keys() - original['scripts'].keys())
    assert [room['attributes']['width'], room['attributes']['height']] == [7, 7]
    assert room['attributes']['tileset_name'] == 'house'
    assert room['blocks'] == [int(token, 16) for row in site['service']['rows'] for token in row.split()]
    assert len(room['objects']) == 1
    actor = room['objects'][0]
    assert (actor['script'], actor['sprite'], actor['x'], actor['y']) == (
        'MN_EXHIBIT_ICEMYTH', 'SPRITE_ICEMYTH', 9, 5)
    assert room['events']['warps'] == [warp(1, 2, 10, hall_constant, 5), warp(2, 3, 11, CONSTANT, 2)]
    assert not room['events']['coord_events']
    assert room['events']['bg_events'] == [dict(x=5, y=4, event_type='BGEVENT_READ', script='MnIcemythCase')]
    wild = [(c['args'][0], c['args'][1]) for program in room['scripts'].values()
            if isinstance(program, list) for c in program if c['command'] == 'loadwildmon']
    assert wild == [('ICEMYTH', '50')]
    assert not any(c['command'] in ['loadwildmon', 'givepoke', 'giveegg']
                   for label, program in hall['scripts'].items()
                   if label not in original['scripts'] and isinstance(program, list) for c in program)
    assert [(c['args'][0], c['command']) for c in hall['scripts']['MnIcemythAttendant']
            if c['command'] in ['setevent', 'disappear']] == [
                ('EVENT_MN_ICEMYTH_LABEL_CORRECTED', 'setevent'),
                ('MN_EXHIBIT_FROZEN_CASE', 'disappear')]
    assert not any(c['command'] in ['givepoke', 'giveegg']
                   for program in room['scripts'].values() if isinstance(program, list) for c in program)
    for module in [hall, room]:
        name = module['id']
        assert new['npcs'][name] == module['objects']
        assert new['map_attributes'][name] == module['attributes']
        assert new['map_blocks'][module['attributes']['blocks_label']] == base64.b64encode(bytes(module['blocks'])).decode()
        programs = dict(module['scripts'])
        for attr, section in [('map_scripts_label', 'map_script_section_commands'),
                              ('map_events_label', 'map_event_section_commands')]:
            programs[module['attributes'][attr]] = [dict(command=row['command'], args=row['args']) for row in module[section]]
        assert all(new['map_scripts'][label] == commands for label, commands in programs.items())
    meta = deepcopy(old['runtime_map_metadata'][hall_constant])
    meta.update(name=ROOM, constant=CONSTANT, width=7, height=7,
                environment=room['attributes']['environment'])
    meta['mapId'] = max(row['mapId'] for row in old['runtime_map_metadata'].values()
                         if row['groupId'] == meta['groupId']) + 1
    assert new['runtime_map_metadata'][CONSTANT] == meta
    before, after = old['story_event_script_constants']['global'], new['story_event_script_constants']['global']
    assert after.keys() - before.keys() == FLAGS
    assert all(after.get(k) == v for k, v in before.items())
    ids = {after[k] for k in FLAGS}
    assert len(ids) == 8 and not ids & {v for k, v in before.items() if k.startswith('EVENT_')}
    expected_report = deepcopy(parent['report'])
    expected_report['maps'] += 1
    expected_report['reachable_maps'].append(ROOM)
    expected_report['solvable_maps'].append(ROOM)
    expected_report['manifests'].extend(['northstar-lair-mnicemythservicepassage', 'minnesota-icemyth-exhibition-v1'])
    assert child['report'] == expected_report
    normalized = deepcopy(child)
    data = normalized['data']
    for table, key in [('maps', ROOM), ('map_attributes', ROOM), ('map_dimensions', CONSTANT),
                       ('map_blocks', room['attributes']['blocks_label']), ('runtime_map_metadata', CONSTANT), ('npcs', ROOM)]:
        del data[table][key]
    assert data['pokegear_landmarks']['map_to_landmark'].pop(ROOM) == old['pokegear_landmarks']['map_to_landmark'][HALL]
    for table, key in [('maps', HALL), ('map_attributes', HALL),
                       ('map_blocks', original['attributes']['blocks_label']), ('npcs', HALL)]:
        data[table][key] = deepcopy(old[table][key])
    labels = set(hall['scripts']) | set(room['scripts'])
    for module in [hall, room]:
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
    normalized['report'] = parent['report']
    normalized['identity'] = parent['identity']
    assert normalized == parent, 'unrelated world, crowd, scripts, species, assets, audio or HOME changed'


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('parent')
    parser.add_argument('child')
    args = parser.parse_args()
    parent, child = load_pack(args.parent), load_pack(args.child)
    site = json.loads((ROOT / 'planning/minnesota/pokemon/acquisition-v01/icemyth/site.json').read_text())
    check(parent, child, site)
    rejected = []
    for case in ['old-map', 'fair-crowd', 'old-program', 'stair', 'foreign-encounter', 'audio']:
        changed = deepcopy(child)
        if case == 'old-map':
            changed['data']['maps']['NewBarkTown']['blocks'][0] ^= 1
        elif case == 'fair-crowd':
            changed['data']['maps']['MnStateFairgrounds']['objects'].pop()
        elif case == 'old-program':
            label = next(k for k, v in parent['data']['maps'][HALL]['scripts'].items() if v)
            changed['data']['maps'][HALL]['scripts'][label] = []
        elif case == 'stair':
            changed['data']['maps'][HALL]['blocks'][38] = 15
        elif case == 'foreign-encounter':
            program = changed['data']['maps'][ROOM]['scripts']['MN_EXHIBIT_ICEMYTH']
            next(c for c in program if c['command'] == 'loadwildmon')['args'][0] = 'MEW'
        else:
            changed['audio_manifest'] = None
        try:
            check(parent, changed, site)
        except AssertionError:
            rejected.append(case)
        else:
            raise AssertionError('failed to reject ' + case)
    print(json.dumps(dict(maps=len(child['data']['maps']),
        preserved_other_complete_maps=len(parent['data']['maps'])-1,
        original_crowd_species_runtime_assets_audio_and_spawns_exact=True,
        rejected_negative_cases=rejected,
        caution='No runtime, earned access, capture or visual acceptance is implied.'), indent=2))


if __name__ == '__main__':
    main()
