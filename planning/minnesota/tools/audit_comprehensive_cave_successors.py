#!/usr/bin/env python3
"""Prove the bounded cave/directions delta without trusting the Rust publisher."""
import argparse
from copy import deepcopy
import json
from pathlib import Path

from render_reference import load_pack

CAVES = ['MnR10Cave1F', 'MnR20Cave1F']
TRAINERS = ['MN_EXTRA_R10_SPRING', 'MN_EXTRA_R10_SURVEY', 'MN_EXTRA_R20_WAYFINDER']
TEXTS = ['MN_R01_BLUFF_WALKERText', 'MN_R01_SIGN_05Text', 'MN_R01_T03Lost',
         'MN_R01_T03Won', 'MN_R03_T04Won', 'MN_R03_TERRACE_SIGNText', 'MN_R10_ESCAPE_ROPE']
EXPECTED = {f'data.maps.{name}' for name in ['MnR01C00R00', 'MnR03C00R00', *CAVES, 'MnVoyageursIce1F', 'MnVoyageursIceB1F']}
for name in CAVES:
    EXPECTED.update(f'data.{field}.{name}' for field in
                    ['map_attributes', 'map_dimensions', 'npcs', 'runtime_map_metadata'])
    EXPECTED.update([f'data.map_blocks.{name}_Blocks', f'data.map_scripts.{name}_MapEvents'])
for trainer in TRAINERS:
    EXPECTED.add(f'data.trainers.trainers.{trainer}')
    EXPECTED.update(f'data.map_scripts.{trainer}{suffix}' for suffix in
                    ['', 'After', 'Declined', 'Lost', 'Seen', 'Won'])
    EXPECTED.update(f'data.story_event_script_constants.global.{flag}' for flag in
                    [trainer, 'EVENT_' + trainer])
EXPECTED.update('data.map_scripts.' + text for text in TEXTS)
EXPECTED.add('data.story_event_script_constants.global.EVENT_MN_R10_ESCAPE_ROPE')
RESOLVED = {'data.map_scripts.MN_R01_FARMERText',
            'data.maps.MnR01Farmhouse.scripts.MN_R01_FARMERText',
            'data.maps.MnR01Farmhouse.script_text_bodies.MN_R01_FARMERText.commands'}
EXPECTED.update('data.map_blocks.' + name + '_Blocks' for name in ['MnVoyageursIce1F', 'MnVoyageursIceB1F'])
MISSING = object()


def merge(before, current, latest, path, adopted, resolved):
    if latest == before or latest == current:
        return current
    if current == before:
        adopted.add(path)
        return latest
    if path in RESOLVED:
        result = deepcopy(current)
        assert len(result) == 11 and result[4]['args'] == ['"The east ramp"']
        assert any(cmd['args'] == ['"IVY works at the"'] for cmd in result)
        assert any(cmd['args'] == ['"Use the east steps"'] for cmd in latest)
        result[4]['args'] = ['"The east steps"']
        resolved.add(path)
        return result
    assert all(isinstance(v, dict) for v in [before, current, latest]), ('conflict', path)
    result = {}
    for key in before.keys() | current.keys() | latest.keys():
        value = merge(before.get(key, MISSING), current.get(key, MISSING),
                      latest.get(key, MISSING), path + '.' + key, adopted, resolved)
        if value is not MISSING:
            result[key] = value
    return result


def check(current, common, latest, child):
    for pack, expected in [(current, '1442b9b9f2ff69b954c03bb3762605c9119c17e99f317cdaac6557186101a19c'),
                           (common, 'acbcb88bb951f3cc343ac550a4a83e423904a7b42fa4dfccfc88497d49092866'),
                           (latest, 'e964dc760a49b64510f93d1469169a5d88aa73104d12f026e8dcf4084f73bc8f')]:
        assert pack['identity']['content_hash'] == expected
    assert common['runtime_files'] == latest['runtime_files']
    assert common['audio_manifest'] == latest['audio_manifest']
    adopted, resolved = set(), set()
    expected = merge(common['data'], current['data'], latest['data'], 'data', adopted, resolved)
    assert adopted == EXPECTED and len(adopted) == 55, sorted(adopted ^ EXPECTED)
    assert resolved == RESOLVED, resolved
    assert child['data'] == expected, 'unapproved data change'
    report = deepcopy(current['report'])
    for marker in latest['report']['manifests']:
        if marker not in common['report']['manifests'] and marker not in report['manifests']:
            report['manifests'].append(marker)
    report['manifests'].append('comprehensive-cave-and-stair-successors-v1')
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
    for case in ['buoys', 'verdigrim', 'reed-class', 'audio', 'cave-exit', 'farmer-referral']:
        bad = deepcopy(child)
        if case == 'buoys':
            bad['data']['maps']['MnTallTaleTrail']['blocks'][63] = 7
        elif case == 'verdigrim':
            bad['data']['story_event_script_constants']['global'].pop('EVENT_MN_VERDIGRIM_CAUGHT')
        elif case == 'reed-class':
            bad['data']['trainers']['trainers']['MN_REED_RELAY']['trainer_class'] = 'RIVAL1'
        elif case == 'audio':
            bad['audio_manifest'] = None
        elif case == 'cave-exit':
            bad['data']['maps']['MnR10Cave1F']['events'] = []
        else:
            bad['data']['map_scripts']['MN_R01_FARMERText'][8]['args'] = ['"TAMARACK waits here"']
        try:
            check(current, common, latest, bad)
        except AssertionError:
            rejected.append(case)
        else:
            raise AssertionError('accepted deliberate corruption: ' + case)
    print(json.dumps(dict(adopted_paths=sorted(EXPECTED), resolved_paths=sorted(RESOLVED),
                         negative_cases_rejected=rejected,
                         scope='Exact bounded integration; native and earned campaign remain separate.'), indent=2))


if __name__ == '__main__':
    main()
