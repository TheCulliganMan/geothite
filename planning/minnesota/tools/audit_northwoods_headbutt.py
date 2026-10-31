#!/usr/bin/env python3
"""Read-only v01/v02 check of the tree table and native subroutine-end fix."""
import argparse
from copy import deepcopy
import json
from pathlib import Path

from render_reference import load_pack

ROOT = Path(__file__).resolve().parents[3]
MAP = 'MnTallTaleTrail'
RESIDENTS = {'NOTCHBILL', 'PINIT', 'HATCHIT', 'DOZEBEAR', 'PITCHWICK', 'CARTICK'}


def active_species():
    roster = json.loads((ROOT / 'planning/minnesota/pokemon/roster.json').read_text())
    curation = json.loads((ROOT / 'planning/minnesota/pokemon/roster-curation.json').read_text())
    retired = {family['id'] for family in curation['retired_families']}
    return {mon['name'].upper() for family in roster['families']
            if family['id'] not in retired for mon in family['members']}


def check(before, after, active):
    assert MAP not in before['data']['field_encounters'], 'expected missing-table preimage'
    field = after['data']['field_encounters'][MAP]
    assert field['map_name'] == MAP
    assert set(field['tables']) == {'headbutt'}, 'only the authored tree activity'
    table = field['tables']['headbutt']
    residents = set()
    for bucket in ['common', 'rare']:
        slots = table[bucket]
        assert len(slots) == 3, 'three distinct residents per tree bucket'
        assert len({slot['species'] for slot in slots}) == 3
        assert sum(slot['weight'] for slot in slots) == 100
        for slot in slots:
            assert slot['species'] in active, 'foreign or retired tree resident'
            assert slot['species'] in RESIDENTS, 'never bypass rare discovery quests'
            assert 28 <= slot['level'] <= 32 and 0 < slot['weight'] <= 100
            residents.add(slot['species'])
    assert residents == RESIDENTS, 'complete authored common/rare tree cast'
    label = 'MnTaleComparisonTextPage'
    program = deepcopy(before['data']['maps'][MAP]['scripts'][label])
    assert len(program) == 5 and program[-1] == dict(command='return', args=[])
    program[-1] = dict(command='end', args=[])
    assert after['data']['maps'][MAP]['scripts'][label] == program, 'only the native return boundary may change'
    old_controls = before['data']['maps'][MAP]['script_control_commands']
    controls = deepcopy(after['data']['maps'][MAP]['script_control_commands'])
    end = dict(command='end', compare_value=None, target_label=None,
               resolved_target_script=None, source_script=label, command_index=4)
    assert controls.count(end) == 1
    controls.remove(end)
    assert controls == old_controls, 'only the newly parsed native end control'
    expected = deepcopy(before)
    expected['data']['field_encounters'][MAP] = deepcopy(field)
    expected['data']['maps'][MAP]['scripts'][label] = program
    expected['data']['maps'][MAP]['script_control_commands'] = deepcopy(after['data']['maps'][MAP]['script_control_commands'])
    expected['data']['map_scripts'][label] = program
    expected['identity'] = after['identity']
    assert expected == after, 'content outside the tree table and exact native end fix changed'
    assert before['identity']['content_hash'] != after['identity']['content_hash']


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('before', type=Path)
    parser.add_argument('after', type=Path)
    args = parser.parse_args()
    before, after = load_pack(args.before), load_pack(args.after)
    active = active_species()
    check(before, after, active)
    rejected = []
    for name in ['foreign-resident', 'zero-weight', 'unrelated-map', 'changed-comparison-text']:
        changed = deepcopy(after)
        if name == 'foreign-resident':
            changed['data']['field_encounters'][MAP]['tables']['headbutt']['common'][0]['species'] = 'MEW'
        elif name == 'zero-weight':
            changed['data']['field_encounters'][MAP]['tables']['headbutt']['common'][0]['weight'] = 0
        elif name == 'unrelated-map':
            changed['data']['maps']['NewBarkTown']['blocks'][0] ^= 1
        else:
            changed['data']['maps'][MAP]['scripts']['MnTaleComparisonTextPage'][1]['args'] = ['Unreviewed text']
        try:
            check(before, changed, active)
        except AssertionError:
            rejected.append(name)
        else:
            raise AssertionError('failed to reject ' + name)
    print(json.dumps(dict(
        before_content_hash=before['identity']['content_hash'],
        after_content_hash=after['identity']['content_hash'],
        maps=len(after['data']['maps']), native_tree_residents=sorted(RESIDENTS),
        other_maps_scripts_catalogs_audio_and_runtime_files_exact=True,
        rejected_negative_cases=rejected,
        caution='Preservation and roster checks do not establish native Headbutt gameplay or earned campaign completion.'
    ), indent=2))


if __name__ == '__main__':
    main()
