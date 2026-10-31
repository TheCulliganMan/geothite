#!/usr/bin/env python3
"""Check the complete successor output and reject deliberate preservation faults."""
import argparse
from copy import deepcopy
import json
from pathlib import Path

from render_reference import load_pack

ABSENT = object()
GALLERY = 'MnDriftlessLampGallery'
GALLERY_PATHS = {
    'data.maps.' + GALLERY, 'data.map_attributes.' + GALLERY,
    'data.map_blocks.' + GALLERY + '_Blocks', 'data.npcs.' + GALLERY,
    'data.map_dimensions.MN_DRIFTLESS_LAMP_GALLERY',
    'data.map_scripts.MnDriftlessLampGallery_MapEvents',
    'data.map_scripts.MnLampFossilsText',
}
CACHE_PATHS = {'data.maps.MnStPaulUnionDepot.' + key for key in (
    'script_flag_commands', 'script_movements', 'script_text_commands',
    'script_runtime_commands', 'script_control_commands', 'script_object_commands',
    'script_variable_commands')}
CACHE_PATHS.update('data.global_scripts.' + key for key in (
    'script_text_commands', 'script_runtime_commands', 'script_map_commands',
    'script_control_commands'))


def groups(rows):
    result = {}
    for row in rows:
        result.setdefault(row['source_script'], []).append(row)
    return result


def cache_union(old, live, new):
    a, b, c = map(groups, (old, live, new))
    revised = {key for key in a.keys() | c.keys()
               if a.get(key) != c.get(key) and b.get(key) != c.get(key)}
    assert all(b.get(key) == a.get(key) for key in revised), 'divergent source program'
    return ([row for row in live if row['source_script'] not in revised]
            + [row for row in new if row['source_script'] in revised])


def merge(old, live, new, path):
    if old == new or live == new:
        return live
    if old == live:
        return new
    if path in GALLERY_PATHS:
        return new
    if path == 'data.runtime_map_metadata.MN_DRIFTLESS_LAMP_GALLERY':
        return dict(new, **{key: live[key] for key in ('groupId', 'mapId', 'groupName')})
    if path.endswith('.fishing_group') and any(name in path for name in (
            'MnDetroitLakesStation', 'MnWinonaStation', 'MnStaplesStation')):
        assert new == 'FISHGROUP_NONE' and live == 'FISHGROUP_MN_SHORE'
        return new
    if path in CACHE_PATHS:
        return cache_union(old, live, new)
    if path in ('data.maps.MnStPaulUnionDepot.map_script_section_commands',
                'data.map_scripts.MnStPaulUnionDepot_MapScripts'):
        result = deepcopy(live)
        for row in new:
            if any((x['command'], x['args']) == (row['command'], row['args']) for x in result):
                continue
            assert row['command'] == 'callback' and row['args'] == [
                'MAPCALLBACK_NEWMAP', 'MnGlacierST_PAULFirstVisit']
            result.append(deepcopy(row))
        for index, row in enumerate(result):
            if 'command_index' in row:
                row['command_index'] = index
        return result
    if path.endswith('.GlobalScriptRoots'):
        assert all(root in live and root in new for root in old)
        return live + [root for root in new if root not in live]
    assert all(isinstance(x, dict) for x in (old, live, new)), ('unresolved overlap', path)
    result = {}
    for key in old.keys() | live.keys() | new.keys():
        value = merge(old.get(key, ABSENT), live.get(key, ABSENT),
                      new.get(key, ABSENT), path + '.' + key)
        if value is not ABSENT:
            result[key] = value
    return result


def catalog_table(rows):
    assert all(len(row) == 1 for row in rows)
    result = {key: value for row in rows for key, value in row.items()}
    assert len(result) == len(rows)
    return result


def expected_data(current, common, donor):
    old, live, new = (deepcopy(pack['data']) for pack in (common, current, donor))
    constants = live['story_event_script_constants']['global']
    after = new['story_event_script_constants']['global']
    maximum = max(value for key, value in constants.items() if key.startswith('EVENT_'))
    for key in sorted(after):
        if not key.startswith('EVENT_'):
            continue
        if key in constants:
            after[key] = constants[key]
        else:
            maximum += 1
            assert maximum <= 65535
            after[key] = maximum
    skier = new['trainers']['trainers']['MN_EXTRA_FALLS_SKIER']
    assert 'MN_EXTRA_FALLS_SKIER' not in live['trainers']['trainers']
    maximum = max(constants[t['trainer_id']] for t in live['trainers']['trainers'].values()
                  if t['trainer_class'] == skier['trainer_class']
                  and t['trainer_id'] in constants)
    assert maximum < 255
    after['MN_EXTRA_FALLS_SKIER'] = maximum + 1
    order = [key for row in live['story_events'] for key in row]
    for data in (old, live, new):
        data['story_events'] = catalog_table(data['story_events'])
    expected = deepcopy(merge(old, live, new, 'data'))
    rows = expected['story_events']
    expected['story_events'] = [{key: rows[key]} for key in order]
    expected['story_events'] += [{key: rows[key]} for key in sorted(rows.keys() - set(order))]
    return expected


def check(current, common, donor, child, expected):
    assert current['identity']['content_hash'] == 'c6feb99daa624a3772c5e76cfc8878b0760d4ce9b498b47eacf145b5156f1d22'
    assert common['identity']['content_hash'] == 'b828c463da5d05885a90a49b4f822fee5e6d7c2d5f997443143d22bf376face9'
    assert donor['identity']['content_hash'] == '6b28ef1aab2c5e843d6321d4f2790f4f1b578a442ce9406d1a9a85dd657462fc'
    assert child['data'] == expected, 'unapproved data change or omitted successor'
    report = deepcopy(current['report'])
    report['maps'] = len(expected['maps'])
    for marker in donor['report']['manifests']:
        if marker not in common['report']['manifests'] and marker not in report['manifests']:
            report['manifests'].append(marker)
    report['manifests'].append('comprehensive-accepted-successors-v1')
    assert child['report'] == report, 'unapproved provenance change'
    for key in current.keys() | child.keys():
        if key not in {'data', 'report', 'identity'}:
            assert current.get(key) == child.get(key), ('unapproved art/audio/pack field', key)
    assert child['identity'] != current['identity']
    assert common['runtime_files'] == donor['runtime_files']
    assert common['compiled_audio'] == donor['compiled_audio']
    assert common['audio_manifest'] == donor['audio_manifest']


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for key in ('current', 'common', 'donor', 'child'):
        parser.add_argument(key, type=Path)
    args = parser.parse_args()
    current, common, donor, child = map(load_pack, (args.current, args.common, args.donor, args.child))
    expected = expected_data(current, common, donor)
    check(current, common, donor, child, expected)
    rejected = []
    for case in ('optional-terrain', 'audio', 'campaign-scene', 'event-alias', 'train-service', 'gallery-identity'):
        bad = deepcopy(child)
        if case == 'optional-terrain':
            bad['data']['maps']['MnTallTaleTrail']['blocks'][0] ^= 1
        elif case == 'audio':
            bad['audio_manifest'] = None
        elif case == 'campaign-scene':
            bad['data']['maps']['MnStPaulUnionDepot']['scenes']['scenes'] = []
        elif case == 'event-alias':
            g = bad['data']['story_event_script_constants']['global']
            g['EVENT_MN_EXTRA_FALLS_SKIER'] = g['EVENT_MN_LAMP_GALLERY_CACHE']
        elif case == 'train-service':
            scripts = bad['data']['maps']['MnStPaulUnionDepot']['scripts']
            del scripts['MnGlacierST_PAULRide0']
        else:
            bad['data']['runtime_map_metadata']['MN_DRIFTLESS_LAMP_GALLERY']['mapId'] += 1
        try:
            check(current, common, donor, bad, expected)
        except AssertionError:
            rejected.append(case)
        else:
            raise AssertionError('corruption accepted: ' + case)
    check(current, common, donor, child, expected)
    print(json.dumps(dict(preservation_passed=True, corruptions_rejected=rejected,
                          valid_child_rechecked=True, maps=len(child['data']['maps']),
                          scope='Exact complete data/provenance and byte-preservation audit; gameplay remains separate.'), indent=2))


if __name__ == '__main__':
    main()
