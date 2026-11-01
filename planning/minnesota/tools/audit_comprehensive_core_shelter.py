#!/usr/bin/env python3
"""Independently verify the accepted mine/route merge and all preserved content."""
import argparse
from copy import deepcopy
import json
from pathlib import Path

from render_reference import load_pack

MISSING = object()
FISHING = {'data.map_attributes.' + name + '.fishing_group'
           for name in ['MnIronDrillCore', 'MnIronPumpGallery']}
FISHING.update('data.maps.' + name + '.attributes.fishing_group'
               for name in ['MnIronDrillCore', 'MnIronPumpGallery'])
CACHE = 'data.maps.MnMinneapolis.script_text_commands'
ART = {'data/tilesets/mn_iron_core_rock' + suffix
       for suffix in ['.json', '_metatiles.bin', '_palette_map.json']}
ART.update('gfx/tilesets/mn_iron_core_rock' + suffix for suffix in ['.2bpp', '.png'])


def grouped(rows):
    result = {}
    for row in rows:
        result.setdefault(row['source_script'], []).append(row)
    return result


def merge(before, current, latest, path, adopted, resolved):
    if latest == before or latest == current:
        return current
    if current == before:
        adopted.add(path)
        return latest
    if path == CACHE:
        before, revised, donor = map(grouped, [before, current, latest])
        assert {k for k in before.keys() | donor.keys() if before.get(k) != donor.get(k)} == {'MnCornerSign'}
        assert {k for k in before.keys() | revised.keys() if before.get(k) != revised.get(k)} == {'MnMPEastCheckBadges', 'MnMPNorthCheckBadges'}
        assert revised.get('MnCornerSign') == before.get('MnCornerSign')
        resolved.add(path)
        return ([row for row in current if row['source_script'] != 'MnCornerSign']
                + [row for row in latest if row['source_script'] == 'MnCornerSign'])
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
    assert current['identity']['content_hash'] == 'e0814d35357b488a07959c080746420e30cdaf9d3e1c8d117d97ea96b72611c8'
    assert common['identity']['content_hash'] == 'c021bf20b9ff9f6354a6e92a9ed8035eb91533a91a4c590819902431a7bf3734'
    assert latest['identity']['content_hash'] == '01e986be28fb7e6fa88ffad3182b70da75541af92a8820c4f5183ee85438527a'
    adopted, resolved = set(), set()
    data = merge(common['data'], current['data'], latest['data'], 'data', adopted, resolved)
    assert len(adopted) == 117, sorted(adopted)
    assert resolved == FISHING | {CACHE}
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
    report['manifests'].append('comprehensive-core-shelter-v1')
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
    for case in ['optional-map', 'fishing-regression', 'audio', 'existing-art', 'mine-boundary', 'city-gate-cache']:
        bad = deepcopy(child)
        if case == 'optional-map':
            bad['data']['maps']['MnTallTaleTrail']['blocks'][0] ^= 1
        elif case == 'fishing-regression':
            bad['data']['maps']['MnIronDrillCore']['attributes']['fishing_group'] = 'FISHGROUP_SHORE'
        elif case == 'audio':
            bad['audio_manifest'] = None
        elif case == 'existing-art':
            bad['runtime_files']['data/tilesets/cave_metatiles.bin'] = b'corrupted'
        elif case == 'mine-boundary':
            bad['data']['maps']['MnIronDrillCore']['blocks'][0] ^= 1
        else:
            bad['data']['maps']['MnMinneapolis']['script_text_commands'] = []
        try:
            check(current, common, latest, bad)
        except AssertionError:
            rejected.append(case)
        else:
            raise AssertionError('failed to reject ' + case)
    print(json.dumps(dict(adopted_path_count=117, resolved_paths=sorted(FISHING | {CACHE}),
                         added_native_art=sorted(ART), negative_cases_rejected=rejected,
                         scope='Exact preservation only; native and earned progression remain separate.'), indent=2))


if __name__ == '__main__':
    main()
