#!/usr/bin/env python3
"""Compare a habitat publication with its exact parent, without changing either."""
import argparse
from copy import deepcopy
import json
from pathlib import Path

from render_reference import load_pack


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('parent', type=Path)
    parser.add_argument('child', type=Path)
    args = parser.parse_args()
    parent, child = load_pack(args.parent), load_pack(args.child)
    before, after = parent['data'], child['data']
    for key in parent.keys() | child.keys():
        if key not in ['identity', 'report', 'data']:
            assert parent.get(key) == child.get(key), key
    for key in before.keys() | after.keys():
        if key not in ['maps', 'map_attributes', 'wild_encounters', 'fishing']:
            assert before.get(key) == after.get(key), key
    assert before['maps'].keys() == after['maps'].keys()
    changed_headers = []
    for name, original in before['maps'].items():
        expected = deepcopy(original)
        group = after['maps'][name]['attributes'].get('fishing_group')
        if group != original['attributes'].get('fishing_group'):
            changed_headers.append(name)
            assert group.startswith('FISHGROUP_MN_HABITAT_'), name
            expected['attributes']['fishing_group'] = group
        assert expected == after['maps'][name], name
    assert len(changed_headers) == 7, changed_headers
    assert before['map_attributes'].keys() == after['map_attributes'].keys()
    for name, original in before['map_attributes'].items():
        expected = dict(original)
        if name in changed_headers:
            expected['fishing_group'] = after['maps'][name]['attributes']['fishing_group']
        assert expected == after['map_attributes'][name], ('map_attributes', name)
    assert before['wild_encounters'].keys() == after['wild_encounters'].keys()
    added_zones = 0
    for name, original in before['wild_encounters'].items():
        revised = after['wild_encounters'][name]
        old_zones, new_zones = original.get('zones', []), revised.get('zones', [])
        assert new_zones[:len(old_zones)] == old_zones, name
        added_zones += len(new_zones) - len(old_zones)
        for key in original.keys() | revised.keys():
            if key not in ['zones', 'water']:
                assert original.get(key) == revised.get(key), (name, key)
        if original.get('water') != revised.get('water'):
            assert name in changed_headers, name
    assert added_zones == 14, added_zones
    old_fishing, new_fishing = before['fishing'], after['fishing']
    assert old_fishing['time_groups'] == new_fishing['time_groups']
    for section in ['groups', 'swarm_rules']:
        for key, original in old_fishing[section].items():
            assert new_fishing[section][key] == original, (section, key)
    for name in changed_headers:
        original_id = before['maps'][name]['attributes']['fishing_group']
        local_id = after['maps'][name]['attributes']['fishing_group']
        pairs = [(original_id, local_id)]
        rules = [r for r in old_fishing['swarm_rules'].values()
                 if r['base_group'] == original_id]
        assert len(rules) == 1
        rule = rules[0]
        pairs.append((rule['swarm_group'], local_id + '_SWARM'))
        matching = [r for r in new_fishing['swarm_rules'].values()
                    if r['base_group'] == local_id]
        assert len(matching) == 1
        expected_rule = dict(rule, base_group=local_id, swarm_group=local_id + '_SWARM')
        assert matching[0] == expected_rule, name
        for old_id, new_id in pairs:
            expected = deepcopy(old_fishing['groups'][old_id])
            local = new_fishing['groups'][new_id]
            expected['source_index'] = local['source_index']
            for rod in ['GOOD_ROD', 'SUPER_ROD']:
                slot = expected['rod_tables'][rod]['slots'][0]
                replacement = local['rod_tables'][rod]['slots'][0]
                slot['species'], slot['level'] = replacement['species'], replacement['level']
            assert expected == local, (name, new_id)
    assert len(new_fishing['groups']) == len(old_fishing['groups']) + 14
    assert len(new_fishing['swarm_rules']) == len(old_fishing['swarm_rules']) + 7
    print(json.dumps(dict(maps=len(after['maps']), added_grass_zones=added_zones,
        local_rod_maps=len(changed_headers), old_rod_and_time_tables_exact=True,
        original_swarm_rules_exact=True, terrain_actors_events_scripts_exact=True,
        unrelated_catalogs_art_audio_exact=True,
        content_identity=child['identity']['content_hash'])))


if __name__ == '__main__':
    main()
