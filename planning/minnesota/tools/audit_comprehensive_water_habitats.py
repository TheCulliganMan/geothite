#!/usr/bin/env python3
"""Read-only full-data preservation check for the campaign/habitat reconciliation."""
import argparse
import base64
from copy import deepcopy
import hashlib
import json
from pathlib import Path

from render_reference import load_pack

ABSENT = object()
HINT = 'MN_LEECH_SHORE_FISHERText'
NATURALIST = 'MN_EXTRA_LEECH_ISLAND_NATURALIST'
HASHES = ['1111a481d2261ccf9c327f24da1f431772d99b2004ffcd208659ff465bfc10a0',
          '4329e491655cd8d6c787a34266a5fa51e222b8cdbd596e56c3b046219c5df5db',
          '20fcdad6de43bb2e46a41faa49d7a8916cfe5bd2224680709f135166d3d2801b']
ROOT = Path(__file__).resolve().parents[3]
REPAIRS = json.loads((ROOT / 'planning/minnesota/pokemon/acquisition-v01/water-caves-v01/habitats.json').read_text())['fishing_repairs']


def by_source(rows):
    result = {}
    for row in rows:
        result.setdefault(row['source_script'], []).append(row)
    return result


def reconcile(old, live, donor, path):
    if old == donor or live == donor:
        return live
    if old == live:
        return donor
    assert ABSENT not in (old, live, donor), ('removed overlap', path)
    if HINT in path:
        return live
    if path.endswith('.fishing_group'):
        assert any(path in (f"data.maps.{r['map']}.attributes.fishing_group",
                            f"data.map_attributes.{r['map']}.fishing_group")
                   and donor == r['after'] == 'FISHGROUP_NONE' for r in REPAIRS), path
        assert live.startswith('FISHGROUP_MN_')
        return donor
    if path == 'data.map_blocks.MnLeechLake_Blocks':
        values = [list(base64.b64decode(v)) for v in (old, live, donor)]
        result = reconcile(*values, 'data.maps.MnLeechLake.blocks')
        return base64.b64encode(bytes(result)).decode('ascii')
    if path == 'data.maps.MnLeechLake.blocks':
        assert len(old) == len(live) == len(donor)
        return [reconcile(a, b, c, 'terrain cell') for a, b, c in zip(old, live, donor)]
    if path.endswith('.GlobalScriptRoots'):
        assert all(row in live and row in donor for row in old)
        return live + [row for row in donor if row not in live]
    if (path.startswith('data.global_scripts.script_') or path == 'data.maps.MnLeechLake.script_text_commands') and all(isinstance(value, list) for value in (old, live, donor)):
        a, b, c = map(by_source, (old, live, donor))
        changed = {key for key in a.keys() | c.keys()
                   if a.get(key) != c.get(key) and b.get(key) != c.get(key) and key != HINT}
        assert all(a.get(key) == b.get(key) for key in changed), ('program conflict', path)
        return ([row for row in live if row['source_script'] not in changed]
                + [row for row in donor if row['source_script'] in changed])
    if path in ('data.maps.MnLeechLake.map_event_section_commands',
                'data.map_scripts.MnLeechLake_MapEvents',
                'data.maps.MnLeechLake.script_field_pickups'):
        normalize = lambda row: {k: v for k, v in row.items() if k != 'command_index'}
        assert all(normalize(row) in [normalize(r) for r in donor] for row in live), path
        return donor
    assert all(isinstance(x, dict) for x in (old, live, donor)), ('unresolved overlap', path)
    result = {}
    for key in old.keys() | live.keys() | donor.keys():
        value = reconcile(old.get(key, ABSENT), live.get(key, ABSENT), donor.get(key, ABSENT), path + '.' + key)
        if value is not ABSENT:
            result[key] = value
    return result


def expected_data(current, common, donor):
    old, live, new = (deepcopy(p['data']) for p in (common, current, donor))
    globals = live['story_event_script_constants']['global']
    next_id = max(v for k, v in globals.items() if k.startswith('EVENT_'))
    for name in sorted(new['story_event_script_constants']['global']):
        if not name.startswith('EVENT_'):
            continue
        if name in globals:
            value = globals[name]
        else:
            next_id += 1
            value = next_id
        new['story_event_script_constants']['global'][name] = value
    trainer = new['trainers']['trainers'][NATURALIST]
    assert NATURALIST not in live['trainers']['trainers']
    next_trainer = 1 + max(globals[t['trainer_id']] for t in live['trainers']['trainers'].values()
                           if t['trainer_class'] == trainer['trainer_class'] and t['trainer_id'] in globals)
    new['story_event_script_constants']['global'][NATURALIST] = next_trainer
    order = [next(iter(row)) for row in live['story_events']]
    for data in (old, live, new):
        rows = data['story_events']
        data['story_events'] = {k: v for row in rows for k, v in row.items()}
        assert len(rows) == len(data['story_events'])
    result = reconcile(old, live, new, 'data')
    catalogs = result['story_events']
    result['story_events'] = [{k: catalogs[k]} for k in order]
    result['story_events'] += [{k: catalogs[k]} for k in sorted(catalogs.keys() - set(order))]
    lake = result['maps']['MnLeechLake']
    body = lake['scripts'][HINT]
    assert body.pop() == dict(command='done', args=[])
    body += [dict(command='para', args=['"The pine island\'s"']),
             dict(command='line', args=['"landing is west."']), dict(command='done', args=[])]
    lake['script_text_bodies'][HINT]['commands'] = [dict(row, command_index=i) for i, row in enumerate(body)]
    result['map_scripts'][HINT] = deepcopy(body)
    return result


def check(current, common, donor, child, expected):
    assert child['data'] == expected, 'complete regional data delta differs'
    report = deepcopy(current['report'])
    report['manifests'] += [m for m in donor['report']['manifests']
                            if m not in common['report']['manifests'] and m not in report['manifests']]
    report['manifests'].append('comprehensive-water-habitats-v1')
    assert child['report'] == report, 'report delta'
    files = dict(current['runtime_files'])
    for key, value in common['runtime_files'].items():
        assert donor['runtime_files'][key] == value
    for key in donor['runtime_files'].keys() - common['runtime_files'].keys():
        assert key == 'data/tilesets/mn_headwaters_animations.json'
        assert key not in files or files[key] == donor['runtime_files'][key]
        files[key] = donor['runtime_files'][key]
    assert child['runtime_files'] == files, 'native art/animation delta'
    for key in current.keys() | child.keys():
        if key not in ('identity', 'data', 'runtime_files', 'report'):
            assert current.get(key) == child.get(key), key
    assert child['data']['maps'].keys() == current['data']['maps'].keys()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ('current', 'common', 'donor', 'child'):
        parser.add_argument(name, type=Path)
    args = parser.parse_args()
    paths = [getattr(args, key) for key in ('current', 'common', 'donor', 'child')]
    hashes = [hashlib.sha256(p.read_bytes()).hexdigest() for p in paths]
    assert hashes[:3] == HASHES, 'exact reviewed inputs required'
    current, common, donor, child = map(load_pack, paths)
    expected = expected_data(current, common, donor)
    check(current, common, donor, child, expected)
    for fault in ('terrain', 'dialogue', 'reward', 'audio', 'art', 'report'):
        bad = deepcopy(child)
        if fault == 'terrain':
            bad['data']['maps']['MnLeechLake']['blocks'][0] ^= 1
        elif fault == 'dialogue':
            bad['data']['maps']['MnLeechLake']['scripts'][HINT].pop()
        elif fault == 'reward':
            bad['data']['story_event_script_constants']['global']['EVENT_MN_LEECH_EDDY_CACHE'] += 1
        elif fault == 'audio':
            bad['audio_manifest'] = None
        elif fault == 'art':
            bad['runtime_files'].clear()
        else:
            bad['report']['manifests'].pop()
        try:
            check(current, common, donor, bad, expected)
        except AssertionError:
            pass
        else:
            raise AssertionError('accepted deliberate ' + fault + ' corruption')
        del bad
    check(current, common, donor, child, expected)
    print(json.dumps(dict(pack=str(paths[-1]), sha256=hashes[-1], content_hash=child['identity']['content_hash'],
                          full_data_preservation=True, rejected_corruptions=6,
                          scope='Independent complete reconciliation audit; native gameplay and visual acceptance remain required.'), indent=2))


if __name__ == '__main__':
    main()
