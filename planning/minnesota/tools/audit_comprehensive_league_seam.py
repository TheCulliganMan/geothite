#!/usr/bin/env python3
"""Prove the one-metatile League seam repair and unchanged gate/content data."""
import argparse
import base64
from copy import deepcopy
import json
from pathlib import Path

from render_reference import load_pack


def check(parent, child):
    assert parent['identity']['content_hash'] == 'f657896b50632d9308e28185e9f6801a1b08e1afa14b61fb16f5319ed5c814bb'
    expected = deepcopy(parent)
    module = expected['data']['maps']['MnR13C00R00']
    assert module['blocks'][16] == 0x8b
    module['blocks'][16] = 0x83
    label = module['attributes']['blocks_label']
    before = base64.b64decode(expected['data']['map_blocks'][label])
    assert list(before) == parent['data']['maps']['MnR13C00R00']['blocks']
    expected['data']['map_blocks'][label] = base64.b64encode(bytes(module['blocks'])).decode()
    expected['report']['manifests'].append('comprehensive-league-seam-repair-v1')
    for key in expected.keys() | child.keys():
        if key != 'identity':
            assert expected.get(key) == child.get(key), key
    assert child['identity'] != parent['identity']
    collision = child['data']['tilesets'][module['attributes']['tileset_name']]['collision']
    assert collision['83'] == ['FLOOR'] * 4
    assert collision['8b'] == ['WALL'] * 4
    # The apron joins the existing30/31 corridor without adding a bypass in
    # Boundary Waters; its entire map, events, guards and checks are identical.
    width = module['attributes']['width']
    for x, y in [(32, 0), (33, 0), (32, 1), (33, 1), (31, 1), (30, 2)]:
        block = module['blocks'][y // 2 * width + x // 2]
        assert collision[f'{block:02x}'][y % 2 * 2 + x % 2] == 'FLOOR'
    assert not any([actor['x'], actor['y']] in [[32, 0], [33, 0], [32, 1], [33, 1]]
                   for actor in module['objects']), 'new apron must not contain an actor body'


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('parent', type=Path)
    parser.add_argument('child', type=Path)
    args = parser.parse_args()
    parent, child = map(load_pack, [args.parent, args.child])
    check(parent, child)
    rejected = []
    for case in ['gate-events', 'other-route-tile', 'roster', 'audio', 'block-mirror']:
        bad = deepcopy(child)
        if case == 'gate-events':
            bad['data']['maps']['MnBoundaryWaters']['events']['coord_events'] = []
        elif case == 'other-route-tile':
            bad['data']['maps']['MnR13C00R00']['blocks'][17] ^= 1
        elif case == 'roster':
            bad['data']['pokemon'].pop('VERDIGRIM')
        elif case == 'audio':
            bad['audio_manifest'] = None
        else:
            bad['data']['map_blocks'][bad['data']['maps']['MnR13C00R00']['attributes']['blocks_label']] = ''
        try:
            check(parent, bad)
        except AssertionError:
            rejected.append(case)
        else:
            raise AssertionError('failed to reject ' + case)
    print(json.dumps(dict(changed_metatiles=1, opened_walking_quadrants=4,
                         gate_map_and_requirements_unchanged=True,
                         negative_cases_rejected=rejected,
                         scope='Exact preservation and static seam; native two-way traversal and earned access remain separate.'), indent=2))


if __name__ == '__main__':
    main()
