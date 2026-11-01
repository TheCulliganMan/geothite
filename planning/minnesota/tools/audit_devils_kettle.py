#!/usr/bin/env python3
"""Read-only residual preservation check for the optional falls branch."""
import argparse
import base64
from copy import deepcopy
import json

from render_reference import load_pack

MAP = 'MnDevilsKettle'
OUTSIDE = 'MnMaraisCedarApproach'
CONSTANT = 'MN_DEVILS_KETTLE'
CAUGHT = 'EVENT_MN_DEVILS_KETTLE_CAUGHT'


def audit(before, after, site):
    assert before['runtime_files'] == after['runtime_files'], 'all native art/audio files'
    for key in before.keys() | after.keys():
        if key not in {'data', 'report', 'identity'}:
            assert before.get(key) == after.get(key), key
    expected_report = deepcopy(before['report'])
    expected_report['manifests'] += ['northstar-lair-mndevilskettle', 'minnesota-devils-kettle-v1']
    expected_report['maps'] += 1
    for field in ['reachable_maps', 'solvable_maps']:
        expected_report[field].append(MAP)
    assert after['report'] == expected_report, 'unexpected report drift'
    old, new = before['data'], after['data']
    original, outside, room = old['maps'][OUTSIDE], new['maps'][OUTSIDE], new['maps'][MAP]
    assert new['maps'].keys() - old['maps'].keys() == {MAP}
    for name, module in old['maps'].items():
        if name != OUTSIDE:
            assert new['maps'][name] == module, name
    blocks = list(original['blocks']); blocks[4*20+6] = 0x82
    assert outside['blocks'] == blocks
    for key in original.keys() | outside.keys():
        if key not in {'blocks', 'events', 'map_event_section_commands', 'scripts', 'script_map_commands', 'script_control_commands'}:
            assert original.get(key) == outside.get(key), ('outside', key)
    assert outside['events']['coord_events'] == original['events']['coord_events'] + [
        dict(x=x,y=8,scene_id='-1',script_name='MnKettleTrailEntry') for x in [12,13]]
    assert outside['scripts'] == dict(original['scripts'], MnKettleTrailEntry=[
        dict(command='warp',args=[CONSTANT,'30','32']),dict(command='end',args=[])])
    for field in ['script_map_commands','script_control_commands']:
        assert [row for row in outside[field] if row['source_script'] != 'MnKettleTrailEntry'] == original[field]
    assert room['events']['coord_events'] == [
        dict(x=28,y=y,scene_id='-1',script_name='MnKettleTrailExit') for y in [32,33]]
    assert original['events']['bg_events'] == outside['events']['bg_events']
    warps = original['events']['warps']; n = len(warps)
    def warp(index, x, y, target, target_id):
        return dict(index=index, x=x, y=y, target_map_constant=target,
                    target_map=target, target_warp_id=target_id)
    outside_constant = original['attributes']['map_constant']
    assert outside['events']['warps'] == warps + [
        warp(n+1,12,8,CONSTANT,3), warp(n+2,13,8,CONSTANT,3),
        warp(n+3,14,9,outside_constant,n+3)]
    assert room['events']['warps'] == [warp(1,28,32,outside_constant,n+3),
                                      warp(2,28,33,outside_constant,n+3),
                                      warp(3,30,32,CONSTANT,3)]
    expected_blocks = [int(site['legend'][token],16)
                       for row in site['rows'] for token in row.split()]
    assert room['blocks'] == expected_blocks
    assert (room['attributes']['width'],room['attributes']['height']) == (20,18)
    assert room['attributes']['tileset_name'] == 'mn_north_shore'
    assert [(o['object_identifier'],o['sprite'],o['x'],o['y']) for o in room['objects']] == [
        ('MN_DEVILS_KETTLURK','SPRITE_KETTLURK',20,12),
        ('MNDEVILSKETTLE_NPC2','SPRITE_FISHER',30,25)]
    expected_habitat = deepcopy(old['wild_encounters'][OUTSIDE])
    expected_habitat['map_name'] = MAP
    # Both fields use skip_serializing_if when cleared by the Rust publisher.
    for key in ['zones', 'swarm_overrides']:
        expected_habitat.pop(key, None)
    assert new['wild_encounters'][MAP] == expected_habitat, 'native cedar habitat copy'
    # Remove only the declared new records and restore the one existing block.
    residual = deepcopy(new)
    for key, record in [('maps',MAP),('map_attributes',MAP),('map_dimensions',CONSTANT),
                        ('npcs',MAP),('runtime_map_metadata',CONSTANT),
                        ('wild_encounters',MAP),('map_blocks',room['attributes']['blocks_label'])]:
        assert record not in old[key]
        del residual[key][record]
    residual['maps'][OUTSIDE] = original
    block_label = outside['attributes']['blocks_label']
    assert new['map_blocks'][block_label] == base64.b64encode(bytes(blocks)).decode()
    residual['map_blocks'][block_label] = old['map_blocks'][block_label]
    assert new['map_blocks'][room['attributes']['blocks_label']] == base64.b64encode(bytes(expected_blocks)).decode()
    # Added map script mirrors must be exact lowerings of the authored module.
    programs = dict(room['scripts'])
    for field, section in [('map_scripts_label','map_script_section_commands'),
                           ('map_events_label','map_event_section_commands')]:
        programs[room['attributes'][field]] = [dict(command=r['command'],args=r['args']) for r in room[section]]
    programs['MnKettleTrailEntry'] = outside['scripts']['MnKettleTrailEntry']
    assert new['map_scripts'].keys() - old['map_scripts'].keys() == programs.keys()
    for label, program in programs.items():
        assert new['map_scripts'][label] == program, label
        del residual['map_scripts'][label]
    label = outside['attributes']['map_events_label']
    assert new['map_scripts'][label] == [dict(command=r['command'],args=r['args']) for r in outside['map_event_section_commands']]
    residual['map_scripts'][label] = old['map_scripts'][label]
    globals_ = residual['story_event_script_constants']['global']
    assert globals_.keys() - old['story_event_script_constants']['global'].keys() == {CAUGHT}
    assert globals_[CAUGHT] not in {v for k,v in old['story_event_script_constants']['global'].items() if k.startswith('EVENT_')}
    del globals_[CAUGHT]
    landmarks = residual['pokegear_landmarks']['map_to_landmark']
    assert landmarks[MAP] == landmarks[OUTSIDE]
    del landmarks[MAP]
    assert residual == old, 'unrelated compiled content drift'
    return dict(other_maps_exact=len(old['maps'])-1, native_art_and_audio_exact=True,
                species_catalog_exact=True, existing_warps_preserved=n,
                changed_existing_metatiles=1, new_maps=1, gameplay_acceptance=False)


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('parent'); parser.add_argument('child'); parser.add_argument('site')
    args = parser.parse_args()
    with open(args.site) as handle:
        site = json.load(handle)
    before, after = load_pack(args.parent), load_pack(args.child)
    report = audit(before, after, site)
    negatives = []
    for label in ['unrelated map', 'native runtime art', 'native habitat', 'reused event identity']:
        bad = dict(after, data=dict(after['data']))
        if label == 'native runtime art':
            bad['runtime_files'] = dict(after['runtime_files'])
            key = next(iter(bad['runtime_files']))
            bad['runtime_files'][key] = b'unreviewed native art'
        elif label == 'unrelated map':
            bad['data']['maps'] = dict(after['data']['maps'])
            name = next(name for name in before['data']['maps'] if name != OUTSIDE)
            module = deepcopy(after['data']['maps'][name])
            module['blocks'][0] ^= 1
            bad['data']['maps'][name] = module
        elif label == 'native habitat':
            bad['data']['wild_encounters'] = dict(after['data']['wild_encounters'])
            habitat = deepcopy(after['data']['wild_encounters'][MAP])
            habitat['grass']['day'][0]['species'] = 'PIDGEY'
            bad['data']['wild_encounters'][MAP] = habitat
        else:
            flags = deepcopy(after['data']['story_event_script_constants'])
            flags['global'][CAUGHT] = next(v for k,v in before['data']['story_event_script_constants']['global'].items() if k.startswith('EVENT_'))
            bad['data']['story_event_script_constants'] = flags
        try:
            audit(before, bad, site)
        except AssertionError:
            negatives.append(dict(mutation=label, rejected=True))
        else:
            raise AssertionError('accepted corruption: '+label)
    print(json.dumps(dict(report, negative_cases=negatives)))
