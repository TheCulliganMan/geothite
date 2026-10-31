#!/usr/bin/env python3
"""Independently verify the accepted mine/route merge and all preserved content."""
import argparse
from copy import deepcopy
import json
from pathlib import Path

from render_reference import load_pack

MISSING = object()
FISHING = {'data.maps.MnIronHaulage.attributes.fishing_group',
           'data.map_attributes.MnIronHaulage.fishing_group'}
ART = {'data/tilesets/mn_iron_haulage_rock' + suffix
       for suffix in ['.json', '_metatiles.bin', '_palette_map.json']}
ART.update('gfx/tilesets/mn_iron_haulage_rock' + suffix for suffix in ['.2bpp', '.png'])
EXPECTED = {'data.maps.' + name for name in
            ['MnR07C00R00', 'MnR07C00R01', 'MnR08C00R00', 'MnR08C00R01']}
EXPECTED.update('data.map_blocks.' + name + '_Blocks' for name in
                ['MnIronHaulage', 'MnR07C00R00', 'MnR08C00R01'])
EXPECTED.update('data.map_scripts.' + name for name in
                ['MN_R07_DROP_SIGNText', 'MN_R07_T02Won', 'MN_R08_BIRCH_WALKERText',
                 'MN_R08_SIGN_03Text', 'MN_R08_T07Won'])
EXPECTED.update(prefix + field for prefix in
                ['data.maps.MnIronHaulage.attributes.', 'data.map_attributes.MnIronHaulage.']
                for field in ['border_block', 'palette', 'tileset_name', 'time_of_day'])
EXPECTED.update({'data.maps.MnIronHaulage.blocks', 'data.tilesets.mn_iron_haulage_rock'})


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
    assert current['identity']['content_hash'] == '8e66a4740371ad2502ee83edc5b2e3011b0d1186423ceb31d0af047abe8a82f8'
    assert common['identity']['content_hash'] == '41aad022dd980dc1bf1f5784fb8a8a465648d500c1119ee2ac2cda2946121ffa'
    assert latest['identity']['content_hash'] == '66bf8c7f11c17ce5c8eee4a69f7dc07c28b79dcfc67d336dd2c9b2baf9b79d4c'
    adopted, resolved = set(), set()
    data = merge(common['data'], current['data'], latest['data'], 'data', adopted, resolved)
    assert adopted == EXPECTED, sorted(adopted ^ EXPECTED)
    assert resolved == FISHING
    assert child['data'] == data, 'unapproved content change'
    assert common['audio_manifest'] == latest['audio_manifest']
    assert all(latest['runtime_files'][key] == value for key, value in common['runtime_files'].items())
    assert latest['runtime_files'].keys() - common['runtime_files'].keys() == ART
    assert not ART & current['runtime_files'].keys()
    files = dict(current['runtime_files'])
    files.update((key, latest['runtime_files'][key]) for key in ART)
    assert child['runtime_files'] == files, 'unapproved native art change'
    report = deepcopy(current['report'])
    for marker in latest['report']['manifests']:
        if marker not in common['report']['manifests'] and marker not in report['manifests']:
            report['manifests'].append(marker)
    report['manifests'].append('comprehensive-haulage-route-successors-v1')
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
            bad['data']['maps']['MnIronHaulage']['attributes']['fishing_group'] = 'FISHGROUP_SHORE'
        elif case == 'audio':
            bad['audio_manifest'] = None
        elif case == 'existing-art':
            bad['runtime_files']['data/tilesets/cave_metatiles.bin'] = b'corrupted'
        else:
            bad['data']['maps']['MnIronHaulage']['blocks'][0] ^= 1
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
