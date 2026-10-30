#!/usr/bin/env python3
"""Strict League entrance preservation; native gameplay remains a separate gate."""
import argparse
import base64
import copy
import json

from render_reference import load_pack


def audit(before, after):
    camp, road = 'MnBoundaryWaters', 'MnR13C00R00'
    for key in before.keys() | after.keys():
        if key not in {'data', 'report', 'identity'}:
            assert before.get(key) == after.get(key), key
    assert before['report'].keys() == after['report'].keys()
    for key, value in before['report'].items():
        if key != 'manifests':
            assert after['report'][key] == value, key
    assert sorted(after['report']['manifests']) == sorted(
        before['report']['manifests'] + ['northstar-canonical-league-entrance-v1'])
    old, new = before['data'], after['data']
    assert old.keys() == new.keys()
    for key, value in old.items():
        if key not in {'maps', 'map_attributes', 'map_blocks'}:
            assert new[key] == value, key
    assert old['maps'].keys() == new['maps'].keys()
    assert old['map_attributes'].keys() == new['map_attributes'].keys()
    expected_blocks = dict(old['map_blocks'])
    for name, value in old['maps'].items():
        expected = copy.deepcopy(value) if name in {camp, road} else value
        if name in {camp, road}:
            direction, target = ('south', road) if name == camp else ('north', camp)
            edges = [e for e in expected['attributes']['connections']
                     if e['direction'] == direction and e['target_map'] == target]
            assert len(edges) == 1
            assert edges[0]['offset'] in ({0, 1} if name == camp else {0, -1})
            edges[0]['offset'] = 0
            if name == road:
                expected['blocks'][16] = expected['blocks'][15]
                expected_blocks[expected['attributes']['blocks_label']] = base64.b64encode(
                    bytes(expected['blocks'])).decode()
        assert new['maps'][name] == expected, name
        assert new['map_attributes'][name] == expected['attributes'], name
    assert new['map_blocks'] == expected_blocks
    return dict(other_maps_exact=len(old['maps']) - 2, native_turnout_metatiles=1,
                camp_terrain_actors_events_exact=True, all_scripts_exact=True,
                original_art_species_audio_exact=True, canonical_offsets=[0, 0],
                gameplay_acceptance=False)


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('parent')
    parser.add_argument('child')
    args = parser.parse_args()
    print(json.dumps(audit(load_pack(args.parent), load_pack(args.child))))
