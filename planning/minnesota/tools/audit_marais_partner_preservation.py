#!/usr/bin/env python3
"""Read-only exact-parent comparison for the Grand Marais gift retry fix."""
import argparse
import copy
import json

from render_reference import load_pack


def command(name, *args):
    return dict(command=name, args=list(args))


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('parent')
    parser.add_argument('child')
    args = parser.parse_args()
    before, after = load_pack(args.parent), load_pack(args.child)
    for key in before.keys() | after.keys():
        if key not in {'data', 'identity', 'report'}:
            assert before.get(key) == after.get(key), key
    a, b = before['data'], after['data']
    room = 'MnGrandMaraisFieldStation'
    original = 'MnRewardMachinePartOriginal'
    for key in a.keys() | b.keys():
        if key not in {'maps', 'map_scripts'}:
            assert a.get(key) == b.get(key), key
    assert a['maps'].keys() == b['maps'].keys()
    for name, old in a['maps'].items():
        new = b['maps'][name]
        if name != room:
            assert old == new, name
            continue
        for key in old.keys() | new.keys():
            if key not in {'scripts', 'script_text_commands', 'script_scene_commands',
                           'script_control_commands', 'script_flag_commands',
                           'script_item_grants', 'script_text_bodies'}:
                assert old.get(key) == new.get(key), (name, key)
        gift_index = next(i for i, instruction in enumerate(old['scripts'][original])
                          if instruction['command'] == 'givepoke')
        for key in ['script_text_commands', 'script_scene_commands',
                    'script_control_commands', 'script_flag_commands', 'script_item_grants']:
            expected_records = copy.deepcopy(old[key])
            for record in expected_records:
                if record['source_script'] == original and record['command_index'] > gift_index:
                    record['command_index'] += 1
            actual_records = list(new[key])
            if key == 'script_text_commands':
                extra = dict(command='jumptextfaceplayer', text_label='MnMaraisPartnerFullText',
                             source_script='MnMaraisPartnerFull', command_index=0)
                actual_records.remove(extra)
            if key == 'script_control_commands':
                extra = dict(command='ifequal', compare_value='2', target_label='MnMaraisPartnerFull',
                             resolved_target_script='MnMaraisPartnerFull', source_script=original,
                             command_index=gift_index + 1)
                actual_records.remove(extra)
            assert actual_records == expected_records, key
        expected_bodies = copy.deepcopy(old['script_text_bodies'])
        expected_bodies['MnMaraisPartnerFullText'] = dict(
            label='MnMaraisPartnerFullText', commands=[dict(**instruction, command_index=i)
                for i, instruction in enumerate(new['scripts']['MnMaraisPartnerFullText'])])
        assert new['script_text_bodies'] == expected_bodies
        for label, program in old['scripts'].items():
            if label != original:
                assert program == new['scripts'][label], label
        expected = old['scripts'][original][:2]
        for instruction in old['scripts'][original][2:]:
            expected.append(instruction)
            if instruction['command'] == 'givepoke':
                expected.append(command('ifequal', '2', 'MnMaraisPartnerFull'))
        assert new['scripts'][original] == expected
        assert new['scripts'].keys() - old['scripts'].keys() == {
            'MnMaraisPartnerFull', 'MnMaraisPartnerFullText'}
        for instruction in new['scripts']['MnMaraisPartnerFullText']:
            if instruction['command'] in {'text', 'line', 'cont', 'para'}:
                assert all(len(row.strip('"')) <= 18 for row in instruction['args'])
    for label, program in a['map_scripts'].items():
        if label != original:
            assert program == b['map_scripts'][label], label
    print(json.dumps(dict(
        other_maps_exact=len(a['maps']) - 1,
        only_original_program_change_is_delivery_guard=True,
        actors_geometry_encounters_items_teams_constants_art_audio_exact=True,
        content_identity=after['identity']['content_hash'])))


if __name__ == '__main__':
    main()
