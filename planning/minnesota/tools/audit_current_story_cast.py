#!/usr/bin/env python3
"""Independently check the exact bounded cast integration; never mutate packs."""
import argparse
from copy import deepcopy
import json
from pathlib import Path

from render_reference import load_pack

DONOR = '74bfb7af0c8147c13812005770425d82a4dbf5a38655ac3e563d49708c7c11eb'
ROOT = Path(__file__).resolve().parents[3]
BINDINGS = [
    'planning/minnesota/npcs/tamarack-ice/01-tamarack/binding-v1.json',
    'planning/minnesota/npcs/regional-trainers-v01/story-bindings/01-reed.json',
]


def event_rows(rows, actor, sprite):
    found = [row for row in rows if row['command'] == 'object_event'
             and row['args'][:2] == list(map(str, actor['tile']))
             and row['args'][11] == actor['script']]
    assert len(found) == 1
    assert found[0]['args'][2] == actor['previous_sprite']
    found[0]['args'][2] = sprite


def check(parent, donor, child):
    assert donor['identity']['content_hash'] == DONOR
    expected = deepcopy(parent)
    for path in BINDINGS:
        binding = json.loads((ROOT / path).read_text())
        cls = binding['trainer_class']
        sprite = 'SPRITE_' + cls
        for folder in ['sprites', 'trainers']:
            for ext in ['2bpp', 'gbcpal', 'png']:
                asset = f'gfx/{folder}/{cls.lower()}.{ext}'
                assert asset not in expected['runtime_files']
                expected['runtime_files'][asset] = donor['runtime_files'][asset]
        data = expected['data']
        assert sprite not in data['sprite_palette_defaults']
        data['sprite_palette_defaults'][sprite] = donor['data']['sprite_palette_defaults'][sprite]
        if binding['trainer_ids']:
            assert cls not in data['trainer_class_names']
            data['trainer_class_names'][cls] = donor['data']['trainer_class_names'][cls]
        for trainer_id in binding['trainer_ids']:
            trainer = data['trainers']['trainers'][trainer_id]
            assert trainer['trainer_class'] == 'RIVAL1'
            trainer['trainer_class'] = cls
            assert trainer == donor['data']['trainers']['trainers'][trainer_id]
        for actor in binding['actors']:
            module = data['maps'][actor['map']]
            for rows in [module['objects'], data['npcs'][actor['map']]]:
                found = [row for row in rows if row['script'] == actor['script']
                         and [row['x'], row['y']] == actor['tile']]
                assert len(found) == 1
                assert found[0]['sprite'] == actor['previous_sprite']
                found[0]['sprite'] = sprite
                found[0]['sprite_has_facings'] = True
            event_rows(module['map_event_section_commands'], actor, sprite)
            label = module['attributes']['map_events_label']
            if label in module['scripts']:
                event_rows(module['scripts'][label], actor, sprite)
            if label in data['map_scripts']:
                event_rows(data['map_scripts'][label], actor, sprite)
    expected['report']['manifests'].append('northstar-current-cast-integration-v1')
    for key in expected.keys() | child.keys():
        if key != 'identity':
            assert expected.get(key) == child.get(key), f'unapproved delta: {key}'
    assert child['identity'] != parent['identity']


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('parent', type=Path)
    parser.add_argument('donor', type=Path)
    parser.add_argument('child', type=Path)
    args = parser.parse_args()
    parent, donor, child = map(load_pack, [args.parent, args.donor, args.child])
    check(parent, donor, child)
    rejected = []
    for case in ['team', 'terrain', 'audio', 'actor-position', 'story', 'original-art']:
        changed = deepcopy(child)
        if case == 'team':
            changed['data']['trainers']['trainers']['MN_REED_RELAY']['party'] = []
        elif case == 'terrain':
            changed['data']['maps']['MnIceRelayFloor']['blocks'][0] ^= 1
        elif case == 'audio':
            changed['audio_manifest'] = None
        elif case == 'actor-position':
            changed['data']['maps']['MnWinonaFieldStation']['objects'][0]['x'] += 1
        elif case == 'story':
            changed['data']['maps']['MnIceRelayFloor']['scripts'] = {}
        else:
            key = next(iter(parent['runtime_files']))
            changed['runtime_files'].pop(key)
        try:
            check(parent, donor, changed)
        except AssertionError:
            rejected.append(case)
        else:
            raise AssertionError('failed to reject ' + case)
    print(json.dumps(dict(accepted=True, trainers=6, actors=10, assets=12,
                         negative_cases_rejected=rejected,
                         scope='Exact static preservation; gameplay acceptance remains separate.'), indent=2))


if __name__ == '__main__':
    main()
