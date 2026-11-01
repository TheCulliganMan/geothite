#!/usr/bin/env python3
"""Independently verify the accepted mine/route merge and all preserved content."""
import argparse
from copy import deepcopy
import json
from pathlib import Path

from render_reference import load_pack

MISSING = object()
FISHING = {'data.maps.MnIronVentilation.attributes.fishing_group',
           'data.map_attributes.MnIronVentilation.fishing_group'}
ART = {'data/tilesets/mn_iron_vent_rock' + suffix
       for suffix in ['.json', '_metatiles.bin', '_palette_map.json']}
ART.update('gfx/tilesets/mn_iron_vent_rock' + suffix for suffix in ['.2bpp', '.png'])
ART.update({'data/tilesets/mn_north_shore.json', 'data/tilesets/mn_north_shore_metatiles.bin'})
EXPECTED = {'data.maps.' + name for name in ['MnR11C00R00', 'MnR11C00R02', 'MnR11C00R03']}
EXPECTED.update('data.map_blocks.' + name + '_Blocks' for name in ['MnIronVentilation', 'MnR11C00R00', 'MnR11C00R01', 'MnR11C00R02', 'MnR11C00R03'])
EXPECTED.update(prefix + field for prefix in ['data.maps.MnIronVentilation.attributes.', 'data.map_attributes.MnIronVentilation.'] for field in ['border_block', 'palette', 'tileset_name', 'time_of_day'])
EXPECTED.update({'data.maps.MnIronVentilation.blocks', 'data.tilesets.mn_iron_vent_rock', 'data.tilesets.mn_north_shore', 'data.maps.MnR11C00R01.blocks'})


def merge(before, current, latest, path, adopted, resolved):
    if latest == before or latest == current:
        return current
    if current == before:
        adopted.add(path)
        return latest
    if path in FISHING:
        assert (before, current, latest) == ('FISHGROUP_SHORE', 'FISHGROUP_MN_SHORE', 'FISHGROUP_MN_LAKE')
        resolved.add(path)
        return latest
    assert all(isinstance(value, dict) for value in [before, current, latest]), ('conflict', path)
    result = {}
    for key in before.keys() | current.keys() | latest.keys():
        value = merge(before.get(key, MISSING), current.get(key, MISSING),
                      latest.get(key, MISSING), path + '.' + key, adopted, resolved)
        if value is not MISSING:
            result[key] = value
    return result


def check(current, common, latest, child):
    assert current['identity']['content_hash'] == '74a0fbdaee301653502d0d4e93eac4e24600dcc34617b88e3e860a76565bbe6e'
    assert common['identity']['content_hash'] == 'd1977ad46aeb9734e8c268d857e01c169a031a8abdc3a935e88a60420ce67b9c'
    assert latest['identity']['content_hash'] == 'c021bf20b9ff9f6354a6e92a9ed8035eb91533a91a4c590819902431a7bf3734'
    adopted, resolved = set(), set()
    data = merge(common['data'], current['data'], latest['data'], 'data', adopted, resolved)
    assert adopted == EXPECTED, sorted(adopted ^ EXPECTED)
    assert resolved == FISHING
    assert child['data'] == data, 'unapproved content change'
    assert common['audio_manifest'] == latest['audio_manifest']
    delta = {key for key in common['runtime_files'].keys() | latest['runtime_files'].keys() if common['runtime_files'].get(key) != latest['runtime_files'].get(key)}
    assert delta == ART
    assert all(current['runtime_files'].get(key) == common['runtime_files'].get(key) for key in ART)
    files = dict(current['runtime_files'])
    files.update((key, latest['runtime_files'][key]) for key in ART)
    assert child['runtime_files'] == files, 'unapproved native art change'
    report = deepcopy(current['report'])
    for marker in latest['report']['manifests']:
        if marker not in common['report']['manifests'] and marker not in report['manifests']:
            report['manifests'].append(marker)
    report['manifests'].append('comprehensive-vent-bridges-v1')
    assert child['report'] == report
    for key in current.keys() | child.keys():
        if key not in {'data', 'report', 'identity', 'runtime_files'}:
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
    for case in ['optional-map', 'fishing-regression', 'audio', 'existing-art', 'mine-boundary']:
        bad = deepcopy(child)
        if case == 'optional-map':
            bad['data']['maps']['MnTallTaleTrail']['blocks'][0] ^= 1
        elif case == 'fishing-regression':
            bad['data']['maps']['MnIronVentilation']['attributes']['fishing_group'] = 'FISHGROUP_SHORE'
        elif case == 'audio':
            bad['audio_manifest'] = None
        elif case == 'existing-art':
            bad['runtime_files']['data/tilesets/cave_metatiles.bin'] = b'corrupted'
        else:
            bad['data']['maps']['MnIronVentilation']['blocks'][0] ^= 1
        try:
            check(current, common, latest, bad)
        except AssertionError:
            rejected.append(case)
        else:
            raise AssertionError('failed to reject ' + case)
    print(json.dumps(dict(adopted_paths=sorted(EXPECTED), resolved_paths=sorted(FISHING),
                         added_native_art=sorted(ART), negative_cases_rejected=rejected,
                         scope='Exact preservation only; native and earned progression remain separate.'), indent=2))


if __name__ == '__main__':
    main()
