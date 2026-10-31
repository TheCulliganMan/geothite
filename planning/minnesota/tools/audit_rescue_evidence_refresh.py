#!/usr/bin/env python3
"""Read-only bounded rescue/fence preservation check; no pack mutation."""
import argparse
import base64
from copy import deepcopy
import json
from render_reference import load_pack

OFFICE = 'MnMinneapolisDispatchOffice'
HOME = 'MnMinneapolisDispatcherHouse'
TILESET = 'mn_northstar_holding'
NOTE = 'EVENT_MN_DISPATCH_NOTE'
CELLS = [(6, 1, 64), (6, 2, 64), (6, 4, 66), (7, 4, 65), (8, 4, 65)]
TARGETS = {'MN_MP_RESCUED_DISPATCHER', 'MnMPDispatchCopy'}


def command(op, *args):
    return dict(command=op, args=list(args))


def check(parent, child):
    old, new = parent['data'], child['data']
    added_art = TILESET not in old['tilesets']
    expected_report = deepcopy(parent['report'])
    if added_art:
        expected_report['manifests'].append('northstar-gray-holding-fence-v1')
    expected_report['manifests'].append('northstar-rescue-evidence-refresh-v1')
    assert child['report'] == expected_report
    for key in parent.keys() | child.keys():
        if key not in {'identity', 'data', 'report', 'runtime_files'}:
            assert parent.get(key) == child.get(key), key
    assert all(child['runtime_files'].get(k) == v for k, v in parent['runtime_files'].items())
    files = child['runtime_files'].keys() - parent['runtime_files'].keys()
    expected_files = {f'data/tilesets/{TILESET}_metatiles.bin',
                      f'data/tilesets/{TILESET}.json',
                      f'data/tilesets/{TILESET}_palette_map.json',
                      f'gfx/tilesets/{TILESET}.2bpp', f'gfx/tilesets/{TILESET}.png'}
    assert files == (expected_files if added_art else set())
    permitted = {'maps', 'map_attributes', 'map_blocks', 'map_scripts', 'tilesets', 'field_moves'}
    for key in old.keys() | new.keys():
        if key not in permitted:
            assert old.get(key) == new.get(key), key
    assert new['maps'].keys() == old['maps'].keys()
    for name, original in old['maps'].items():
        revised = new['maps'][name]
        if name not in {OFFICE, HOME}:
            assert revised == original, name
            continue
        assert revised['objects'] == original['objects']
        assert revised['events'] == original['events']
        assert revised['scenes'] == original['scenes']
        assert revised['script_text_bodies'] == original['script_text_bodies']
        assert revised['map_script_section_commands'] == original['map_script_section_commands']
        assert revised['map_event_section_commands'] == original['map_event_section_commands']
        assert revised['scripts'].keys() == original['scripts'].keys()
        assert all(revised['scripts'][k] == v for k, v in original['scripts'].items() if k not in TARGETS)
        expected_attributes = dict(original['attributes'])
        expected_blocks = list(original['blocks'])
        if name == OFFICE:
            expected_attributes['tileset_name'] = TILESET
            for x, y, block in CELLS:
                assert original['blocks'][y * 10 + x] == (56 if added_art else block)
                expected_blocks[y * 10 + x] = block
        assert revised['attributes'] == expected_attributes
        assert revised['blocks'] == expected_blocks
    before = old['maps'][HOME]['scripts']['MN_MP_RESCUED_DISPATCHER']
    index = before.index(command('writetext', 'MnNICEEllisQuestText'))
    expected = before[:index + 1] + [command('promptbutton'), command('closetext'), command('end')]
    assert new['maps'][HOME]['scripts']['MN_MP_RESCUED_DISPATCHER'] == expected
    expected = [command('checkevent', NOTE), command('iftrue', 'MnMPDispatchDone')]
    for flag in ['EVENT_MN_ICE_CAPTIVE_RESCUED', 'EVENT_MN_ICE_HOLDING_POWER_OFF',
                 'EVENT_MN_ICE_ENFORCER_DEFEATED']:
        expected += [command('checkevent', flag), command('iffalse', 'MnMPDispatchLocked')]
    expected += [command('opentext'), command('writetext', 'MnMPDispatchReadText'),
                 command('promptbutton'), command('setevent', NOTE), command('closetext'), command('end')]
    assert new['maps'][OFFICE]['scripts']['MnMPDispatchCopy'] == expected
    for label in old['map_scripts'].keys() | new['map_scripts'].keys():
        if label in TARGETS:
            room = HOME if label == 'MN_MP_RESCUED_DISPATCHER' else OFFICE
            assert new['map_scripts'][label] == new['maps'][room]['scripts'][label]
        else:
            assert old['map_scripts'].get(label) == new['map_scripts'].get(label), label
    for table, permitted_key in [('map_attributes', OFFICE),
                                 ('map_blocks', old['maps'][OFFICE]['attributes']['blocks_label'])]:
        assert old[table].keys() == new[table].keys()
        assert all(new[table][k] == v for k, v in old[table].items() if k != permitted_key)
    assert new['map_attributes'][OFFICE] == new['maps'][OFFICE]['attributes']
    block_label = new['maps'][OFFICE]['attributes']['blocks_label']
    assert new['map_blocks'][block_label] == base64.b64encode(bytes(new['maps'][OFFICE]['blocks'])).decode()
    assert new['tilesets'].keys() - old['tilesets'].keys() == ({TILESET} if added_art else set())
    assert all(new['tilesets'].get(k) == v for k, v in old['tilesets'].items())
    moves = deepcopy(old['field_moves'])
    if added_art:
        for kind in ['cut', 'whirlpool']:
            replacements = moves[kind]['replacements']
            if 'underground' in replacements:
                replacements[TILESET] = deepcopy(replacements['underground'])
    assert new['field_moves'] == moves


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('parent')
    parser.add_argument('child')
    args = parser.parse_args()
    parent, child = load_pack(args.parent), load_pack(args.child)
    check(parent, child)
    rejected = []
    for case in ['outdoor', 'home-evidence', 'unguarded-office', 'audio']:
        changed = deepcopy(child)
        if case == 'outdoor':
            changed['data']['maps']['MnR13C00R00']['blocks'][0] ^= 1
        elif case == 'home-evidence':
            changed['data']['maps'][HOME]['scripts']['MN_MP_RESCUED_DISPATCHER'].append(command('setevent', NOTE))
        elif case == 'unguarded-office':
            changed['data']['maps'][OFFICE]['scripts']['MnMPDispatchCopy'] = [command('setevent', NOTE), command('end')]
        else:
            changed['audio_manifest'] = None
        try:
            check(parent, changed)
        except AssertionError:
            rejected.append(case)
        else:
            raise AssertionError('failed to reject ' + case)
    print(json.dumps(dict(preserved_other_complete_maps=len(parent['data']['maps'])-2,
        preserved_event_identities_species_audio_and_original_assets=True,
        rejected_negative_cases=rejected,
        caution='Derived script tables require production runtime proof; this is not visual or campaign acceptance.'), indent=2))


if __name__ == '__main__':
    main()
