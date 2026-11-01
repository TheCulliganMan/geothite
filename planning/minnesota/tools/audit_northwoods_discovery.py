#!/usr/bin/env python3
"""Read-only current-parent preservation audit for the northwoods review."""
import argparse
import base64
from copy import deepcopy
import json
from pathlib import Path

from audit_northwoods_headbutt import active_species
from render_reference import load_pack

ROOT = Path(__file__).resolve().parents[3]
MAP, OUTSIDE, PROP = 'MnTallTaleTrail', 'MnBemidjiNorthwoods', 'mn_northwoods_props'
CONSTANT = 'MN_TALL_TALE_TRAIL'
GATE, GATE_CONSTANT = 'MnTallTaleForestGate', 'MN_TALL_TALE_FOREST_GATE'
FLAGS = {'EVENT_MN_TALL_TALE_' + name for name in [
    'CULVERT_READ', 'PRINTS_READ', 'STACK_READ', 'CAMP_DIRECTIONS',
    'CAMP_PATH_ACKNOWLEDGED', 'LOG_ACCOUNT_READ', 'OX_REVEALED',
    'CAMP_KEEPER_REVEALED', 'BABEHEMOTH_CAUGHT', 'BUNJANT_CAUGHT']}
FILES = set()


def check(parent, child, site, active, headbutt):
    before, after = parent['data'], child['data']
    assert MAP not in before['maps'] and PROP not in before['tilesets']
    assert GATE not in before['maps']
    outside = deepcopy(before['maps'][OUTSIDE])
    assert not outside['events']['warps']
    for x, y, value in [(6,4,8),(7,4,9),(6,5,26),(7,5,17),(6,6,1)]:
        assert outside['blocks'][y*8+x] == 5
        outside['blocks'][y*8+x] = value
    def warp(index, x, y, constant, target, destination):
        return dict(index=index,x=x,y=y,target_map_constant=constant,
                    target_map=constant,target_warp_id=destination)
    outside_constant = outside['attributes']['map_constant']
    outside['events']['warps'] = [warp(1,13,11,GATE_CONSTANT,GATE,5),
                                 warp(2,13,12,outside_constant,OUTSIDE,2)]
    commands = outside['map_event_section_commands']
    offset = next(i for i,row in enumerate(commands) if row['command'] == 'def_warp_events') + 1
    commands[offset:offset] = [dict(command='warp_event',args=['13','11',GATE_CONSTANT,'5']),
                               dict(command='warp_event',args=['13','12',outside_constant,'2'])]
    for index,row in enumerate(commands):
        row['command_index'] = index
    assert outside == after['maps'][OUTSIDE], 'unreviewed corridor change'
    room = after['maps'][MAP]
    attrs = room['attributes']
    assert [attrs['width'], attrs['height']] == [20, 20] and attrs['tileset_name'] == 'johto'
    assert attrs['connections'] == [] and attrs['connection_flags'] is None
    blocks = [int(site['legend'][token], 16) for row in site['rows'] for token in row.split()]
    for x,y,value in [(1,15,8),(2,15,9),(1,16,26),(2,16,17),(1,17,1)]:
        assert blocks[y*20+x] in [2,5]
        blocks[y*20+x] = value
    assert room['blocks'] == blocks, 'published terrain differs from authored rows'
    collision = after['tilesets']['johto']['collision']
    assert all('BUOY' not in collision[f'{block:02x}'] for block in blocks), 'buoys remain in the pool'

    assert room['events']['warps'] == [warp(1,3,33,GATE_CONSTANT,GATE,6),warp(2,3,34,CONSTANT,MAP,2)]
    assert not room['events']['coord_events']
    gate = after['maps'][GATE]
    original_gate = before['maps']['IlexForestAzaleaGate']
    assert gate['blocks'] == original_gate['blocks'], 'original Ilex room geometry changed'
    gate_attrs = deepcopy(original_gate['attributes'])
    gate_attrs.update(map_constant=GATE_CONSTANT,map_group_constant=outside['attributes']['map_group_constant'],
        blocks_label=GATE+'_Blocks',map_scripts_label=GATE+'_MapScripts',map_events_label=GATE+'_MapEvents',
        connections=[],connection_flags=None,location=outside['attributes']['location'])
    assert gate['attributes'] == gate_attrs
    assert gate['events'] == dict(warps=[
        warp(1,0,4,outside_constant,OUTSIDE,2),warp(2,9,4,CONSTANT,MAP,2),
        warp(3,0,5,outside_constant,OUTSIDE,2),warp(4,9,5,CONSTANT,MAP,2),
        warp(5,1,4,GATE_CONSTANT,GATE,5),warp(6,8,4,GATE_CONSTANT,GATE,6)],coord_events=[],bg_events=[])
    assert gate['scenes'] == dict(scenes=[])
    actor = deepcopy(before['maps']['NewBarkTown']['objects'][0])
    actor.update(sprite='SPRITE_GRANNY',x=5,y=2,spritemovedata='SPRITEMOVEDATA_STANDING_DOWN',
        move_range_x=0,move_range_y=0,pal=0,script='MnTaleGateAttendant',
        object_identifier='MNTALLTALEFORESTGATE_NPC1',event_flag='-1')
    assert gate['objects'] == [actor]
    command = lambda name,args:dict(command=name,args=args)
    assert gate['scripts'] == dict(MnTaleGateAttendant=[command('jumptext',['MnTaleGateHint'])],
        MnTaleGateHint=[command('text',['"TALL TALE TRAIL"']),command('line',['"is through here."']),
                       command('para',['"Read the ground."']),command('line',['"Then ask the crew."']),command('done',[])])
    assert {(o['script'], o['sprite'], o['x'], o['y']) for o in room['objects']} == {
        ('MnTaleWorker', 'SPRITE_FISHER', 10, 34),
        ('MN_TALE_BABEHEMOTH', 'SPRITE_BABEHEMOTH', 18, 13),
        ('MN_TALE_BUNJANT', 'SPRITE_BUNJANT', 30, 8)}
    assert len(room['objects']) == 3
    wild = [(c['args'][0], c['args'][1]) for p in room['scripts'].values()
            if isinstance(p, list) for c in p if c['command'] == 'loadwildmon']
    assert set(wild) == {('BABEHEMOTH', '60'), ('BUNJANT', '60')} and len(wild) == 2
    for program in room['scripts'].values():
        if isinstance(program, list):
            assert not any(c['command'] in ['givepoke', 'giveegg'] for c in program)
    grass = after['wild_encounters'][MAP]['grass']
    for slots in grass.values():
        assert len(slots) == 7
        assert all(slot['species'] in active and 28 <= slot['level'] <= 32 for slot in slots)
    assert (MAP in after['field_encounters']) == headbutt
    if headbutt:
        field = after['field_encounters'][MAP]
        assert set(field['tables']) == {'headbutt'}
        assert all(slot['species'] in active for bucket in field['tables']['headbutt'].values() for slot in bucket)
    global_before = before['story_event_script_constants']['global']
    global_after = after['story_event_script_constants']['global']
    assert global_after.keys() - global_before.keys() == FLAGS
    ids = [global_after[name] for name in FLAGS]
    assert len(set(ids)) == 10 and all(0 <= number <= 65535 for number in ids)
    assert not set(ids) & {value for key, value in global_before.items() if key.startswith('EVENT_')}
    assert child['runtime_files'].keys() - parent['runtime_files'].keys() == FILES
    expected_report = deepcopy(parent['report'])
    expected_report['maps'] += 2
    expected_report['reachable_maps'].extend([GATE, MAP])
    expected_report['solvable_maps'].extend([GATE, MAP])
    expected_report['manifests'].extend(['northstar-lair-mntalltaleforestgate', 'northstar-lair-mntalltaletrail', 'minnesota-northwoods-tall-tales-v1'])
    assert child['report'] == expected_report, 'unreviewed publication bookkeeping'
    normalized = deepcopy(child)
    data = normalized['data']
    labels = set()
    for name in [GATE, MAP]:
        attributes = after['maps'][name]['attributes']
        for kind,key in [('maps',name),('map_attributes',name),('map_dimensions',attributes['map_constant']),
                         ('map_blocks',attributes['blocks_label']),('runtime_map_metadata',attributes['map_constant']),
                         ('npcs',name)]:
            del data[kind][key]
        assert data['pokegear_landmarks']['map_to_landmark'].pop(name) == before['pokegear_landmarks']['map_to_landmark'][OUTSIDE]
        labels |= set(after['maps'][name]['scripts']) | {attributes['map_scripts_label'],attributes['map_events_label']}
    del data['wild_encounters'][MAP]
    for kind, key in [('maps', OUTSIDE), ('map_attributes', OUTSIDE),
                      ('map_blocks', before['maps'][OUTSIDE]['attributes']['blocks_label'])]:
        data[kind][key] = deepcopy(before[kind][key])
    assert after['map_attributes'][OUTSIDE] == outside['attributes']
    assert after['map_blocks'][outside['attributes']['blocks_label']] == base64.b64encode(bytes(outside['blocks'])).decode()
    if headbutt:
        del data['field_encounters'][MAP]
    assert after['map_scripts'].keys() - before['map_scripts'].keys() == labels
    for label in labels:
        del data['map_scripts'][label]
    event_label = outside['attributes']['map_events_label']
    assert after['map_scripts'][event_label] == [dict(command=row['command'],args=row['args']) for row in outside['map_event_section_commands']]
    data['map_scripts'][event_label] = deepcopy(before['map_scripts'][event_label])
    for flag in FLAGS:
        del data['story_event_script_constants']['global'][flag]
    normalized['report'] = parent['report']
    normalized['identity'] = parent['identity']
    assert normalized == parent, 'unrelated map, program, catalog, audio or runtime file changed'


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('parent', type=Path)
    parser.add_argument('child', type=Path)
    parser.add_argument('--headbutt', action='store_true')
    args = parser.parse_args()
    parent, child = load_pack(args.parent), load_pack(args.child)
    site = json.loads((ROOT / 'planning/minnesota/pokemon/acquisition-v01/northwoods/site.json').read_text())
    active = active_species()
    check(parent, child, site, active, args.headbutt)
    assert child['data']['tilesets'] == parent['data']['tilesets'], 'all original tilesets must remain exact'
    assert all(block < 128 for block in child['data']['maps'][MAP]['blocks']), 'custom prop remains in trail'
    rejected = []
    for name in ['unrelated-map', 'changed-original-seam', 'foreign-grass', 'old-art']:
        changed = deepcopy(child)
        if name == 'unrelated-map':
            changed['data']['maps']['NewBarkTown']['blocks'][0] ^= 1
        elif name == 'changed-original-seam':
            changed['data']['maps'][OUTSIDE]['attributes']['connections'][0]['offset'] += 1
        elif name == 'foreign-grass':
            changed['data']['wild_encounters'][MAP]['grass']['day'][0]['species'] = 'MEW'
        else:
            path = 'gfx/tilesets/johto.2bpp'
            values = list(changed['runtime_files'][path]); values[0] ^= 1
            changed['runtime_files'][path] = values
        try:
            check(parent, changed, site, active, args.headbutt)
        except AssertionError:
            rejected.append(name)
        else:
            raise AssertionError('failed to reject ' + name)
    print(json.dumps(dict(content_hash=child['identity']['content_hash'],
        maps=len(child['data']['maps']), preserved_other_complete_maps=len(parent['data']['maps'])-1,
        all_other_programs_catalogs_audio_and_runtime_files_exact=True,
        original_128_art_blocks_exact_at_three_times=True,
        rejected_negative_cases=rejected,
        caution='This audit does not establish controller traversal, captures or earned campaign completion.'), indent=2))


if __name__ == '__main__':
    main()
