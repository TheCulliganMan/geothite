#!/usr/bin/env python3
"""Independently prove the exact conflict-free newer-world integration."""
import argparse
from copy import deepcopy
import json
from pathlib import Path

from render_reference import load_pack

EXPECTED = {
    'data.npcs.MnIronDrillApproach',
    'data.map_scripts.MnIronDrillApproach_MapEvents',
    'data.map_scripts.MN_IRON_APPROACH_ETHER',
    'data.map_blocks.MnIronDrillApproach_Blocks',
    'data.map_blocks.MnR14C00R00_Blocks',
    'data.story_event_script_constants.global.EVENT_MN_IRON_APPROACH_ETHER',
    'data.maps.MnR14C00R00',
    'data.maps.MnIronDrillApproach.blocks',
    'data.maps.MnIronDrillApproach.script_field_pickups',
    'data.maps.MnIronDrillApproach.objects',
    'data.maps.MnIronDrillApproach.scripts',
    'data.maps.MnIronDrillApproach.attributes.border_block',
    'data.maps.MnIronDrillApproach.map_event_section_commands',
    'data.map_attributes.MnIronDrillApproach.border_block',
}
MISSING = object()


def merge(before, current, latest, path, adopted):
    if latest == before or latest == current:
        return current
    if current == before:
        adopted.add(path)
        return latest
    assert all(isinstance(value, dict) for value in [before, current, latest]), ('conflict', path)
    result = {}
    for key in before.keys() | current.keys() | latest.keys():
        value = merge(before.get(key, MISSING), current.get(key, MISSING),
                      latest.get(key, MISSING), path + '.' + key, adopted)
        if value is not MISSING:
            result[key] = value
    return result


def check(current, common, latest, child):
    assert current['identity']['content_hash'] == '6192149ec0b740acea79b9d503d8a4491aa949ac779ab2d0fb466f7af0b62721'
    assert common['identity']['content_hash'] == 'a7f6270ac5c7cdc83fe0f7484fd8485990ceb9749a0fde803dc43cbd22b90bad'
    assert latest['identity']['content_hash'] == '762ea924b566ae78f839d72944bf5b19c9397a4730b4c6413f29bbd555a5e406'
    adopted = set()
    expected_data = merge(common['data'], current['data'], latest['data'], 'data', adopted)
    assert adopted == EXPECTED, sorted(adopted ^ EXPECTED)
    assert child['data'] == expected_data, 'unapproved data delta'
    report = deepcopy(current['report'])
    for marker in latest['report']['manifests']:
        if marker not in common['report']['manifests'] and marker not in report['manifests']:
            report['manifests'].append(marker)
    report['manifests'].append('comprehensive-current-world-bank-cache-v1')
    assert child['report'] == report
    for key in current.keys() | child.keys():
        if key not in {'data', 'report', 'identity'}:
            assert child.get(key) == current.get(key), key
    assert child['identity'] != current['identity']


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ['current', 'common', 'latest', 'child']:
        parser.add_argument(name, type=Path)
    args = parser.parse_args()
    current, common, latest, child = map(load_pack, [args.current, args.common, args.latest, args.child])
    check(current, common, latest, child)
    rejected = []
    for case in ['optional-area', 'regional-fishing', 'reed-class', 'audio', 'cache-flag']:
        bad = deepcopy(child)
        if case == 'optional-area':
            bad['data']['maps']['MnTallTaleTrail']['blocks'][0] ^= 1
        elif case == 'regional-fishing':
            bad['data']['maps']['MnIronDrillApproach']['attributes']['fishing_group'] = 'FISHGROUP_LAKE'
        elif case == 'reed-class':
            bad['data']['trainers']['trainers']['MN_REED_RELAY']['trainer_class'] = 'RIVAL1'
        elif case == 'audio':
            bad['audio_manifest'] = None
        else:
            bad['data']['story_event_script_constants']['global'].pop('EVENT_MN_IRON_APPROACH_ETHER')
        try:
            check(current, common, latest, bad)
        except AssertionError:
            rejected.append(case)
        else:
            raise AssertionError('failed to reject ' + case)
    print(json.dumps(dict(accepted_delta_paths=sorted(EXPECTED),
                         negative_cases_rejected=rejected,
                         scope='Exact newer-world and campaign preservation; native and earned progression remain separate.'), indent=2))


if __name__ == '__main__':
    main()
