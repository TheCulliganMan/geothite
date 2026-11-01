#!/usr/bin/env python3
"""Prove the34 regional fishing rebindings are the only content changes."""
import argparse
from copy import deepcopy
import json
from pathlib import Path

from render_reference import load_pack
from audit_regional_species_policy import audit, ROOT


def check(parent, child):
    expected = deepcopy(parent)
    count = 0
    targets = {'FISHGROUP_SHORE': 'FISHGROUP_MN_SHORE',
               'FISHGROUP_LAKE': 'FISHGROUP_MN_LAKE'}
    for name, module in expected['data']['maps'].items():
        if not name.startswith('Mn'):
            continue
        before = module['attributes'].get('fishing_group')
        if before not in targets:
            continue
        assert expected['data']['map_attributes'][name]['fishing_group'] == before
        module['attributes']['fishing_group'] = targets[before]
        expected['data']['map_attributes'][name]['fishing_group'] = targets[before]
        count += 1
    assert count == 34, count
    expected['report']['manifests'].append('minnesota-current-fishing-references-v1')
    for key in expected.keys() | child.keys():
        if key != 'identity':
            assert expected.get(key) == child.get(key), key
    assert expected['identity'] != child['identity']
    return count


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('parent', type=Path)
    parser.add_argument('child', type=Path)
    args = parser.parse_args()
    parent, child = map(load_pack, [args.parent, args.child])
    count = check(parent, child)
    roster = json.loads((ROOT / 'planning/minnesota/pokemon/roster.json').read_text())
    retired = {f['id'] for f in json.loads((ROOT / 'planning/minnesota/pokemon/roster-curation.json').read_text())['retired_families']}
    active = {m['name'].upper() for f in roster['families'] if f['id'] not in retired for m in f['members']}
    policy = audit(child, active)
    rejected = []
    for case in ['terrain', 'original-fish', 'raw-attribute', 'script', 'team']:
        bad = deepcopy(child)
        if case == 'terrain':
            bad['data']['maps']['MnIronDrillApproach']['blocks'][0] ^= 1
        elif case == 'original-fish':
            bad['data']['fishing']['groups']['FISHGROUP_SHORE']['rod_tables'] = {}
        elif case == 'raw-attribute':
            bad['data']['map_attributes']['MnBoundaryIntake']['fishing_group'] = 'FISHGROUP_SHORE'
        elif case == 'script':
            bad['data']['maps']['MnBoundaryIntake']['scripts'] = {}
        else:
            bad['data']['trainers']['trainers']['MN_REED_RELAY']['party'] = []
        try:
            check(parent, bad)
        except AssertionError:
            rejected.append(case)
        else:
            raise AssertionError('failed to reject ' + case)
    print(json.dumps(dict(changed_bindings=count, species_policy=policy,
                         negative_cases_rejected=rejected,
                         scope='Exact content preservation and species references; native fishing acceptance remains separate.'), indent=2))


if __name__ == '__main__':
    main()
