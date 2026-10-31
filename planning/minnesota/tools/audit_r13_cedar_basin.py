#!/usr/bin/env python3
"""Read-only Cedar Shelter terrain, original-content and access audit.

Static graphs are separate from actual controller and native render acceptance.
"""
import argparse
import base64
from collections import deque
from copy import deepcopy
import hashlib
import json
from pathlib import Path

from audit_regional_species_policy import audit
from render_reference import load_pack

ROOT = Path(__file__).resolve().parents[3]
ROOM = 'MnR13Cave1F'


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def flood(allowed, start):
    seen, queue = {start}, deque([start])
    while queue:
        x, y = queue.popleft()
        for at in [(x-1, y), (x+1, y), (x, y-1), (x, y+1)]:
            if at in allowed and at not in seen:
                seen.add(at)
                queue.append(at)
    return seen


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ['parent', 'child', 'source', 'output']:
        parser.add_argument('--'+name, type=Path, required=True)
    args = parser.parse_args()
    assert not args.output.exists(), 'retain prior audit results'
    parent, child = load_pack(args.parent), load_pack(args.child)
    read = lambda name: json.loads((args.source/name).read_text())
    blocks = lambda source: [int(v, 16) for row in source['rows'] for v in row.split()]
    old, room = parent['data']['maps'][ROOM], child['data']['maps'][ROOM]
    assert old['blocks'] == blocks(read('before.json'))
    after = read('after.json')
    assert room['blocks'] == blocks(after)
    expected = dict(old, blocks=room['blocks'])
    assert expected == room, 'original room programs, events, actors and attributes must remain exact'
    expected_data = dict(parent['data'])
    expected_data['maps'] = dict(expected_data['maps'], **{ROOM: room})
    expected_data['map_blocks'] = dict(expected_data['map_blocks'], **{
        old['attributes']['blocks_label']: base64.b64encode(bytes(room['blocks'])).decode()})
    assert expected_data == child['data'], 'unrelated data changed'
    report = deepcopy(parent['report'])
    report['manifests'].append('minnesota-r13-cedar-basin-v1')
    assert report == child['report']
    for key in parent.keys() | child.keys():
        if key not in {'identity', 'data', 'report'}:
            assert parent[key] == child[key], key
    collision = child['data']['tilesets']['cave']['collision']
    kinds = {(x, y): collision[f'{room["blocks"][y//2*10+x//2]:02x}'][y%2*2+x%2]
             for y in range(16) for x in range(20)}
    bodies = {(o['x'], o['y']) for o in room['objects']}
    ordinary = {p for p, k in kinds.items() if k in {'FLOOR', 'LADDER'}} - bodies
    seen = flood(ordinary, (7, 14))
    assert ordinary == seen, sorted(ordinary-seen)
    water = {p for p, k in kinds.items() if k == 'WATER'}
    assert len(water) == 24 and flood(water, (4, 7)) == water
    assert all(0 < x < 19 and 0 < y < 15 for x, y in water)
    assert all(tuple(view['tile']) in seen for view in after['review_contract']['viewpoints'])
    assert kinds[3, 7] == 'FLOOR' and kinds[4, 7] == 'WATER'
    assert room['attributes']['fishing_group'] == 'FISHGROUP_MN_LAKE'
    for stair in after['review_contract']['controller_probes']['stairs']:
        for at in [stair['before'], *stair['lane'], stair['after']]:
            assert tuple(at) in seen
        assert kinds[tuple(stair['blocked_target'])] == 'WALL'
    curation = json.loads((ROOT/'planning/minnesota/pokemon/roster-curation.json').read_text())
    roster = json.loads((ROOT/'planning/minnesota/pokemon/roster.json').read_text())
    retired = {f['id'] for f in curation['retired_families']}
    active = {m['name'].upper() for f in roster['families'] if f['id'] not in retired for m in f['members']}
    for rod in child['data']['fishing']['groups']['FISHGROUP_MN_LAKE']['rod_tables'].values():
        for slot in rod['slots']:
            if slot['species'] is None:
                assert slot['time_group'] in child['data']['fishing']['time_groups']
            else:
                assert slot['species'] in active
    regional = audit(dict(child, data=dict(child['data'], maps={ROOM: room})), active)
    result = dict(status='PASS', parent=str(args.parent), parent_sha256=digest(args.parent),
                  pack=str(args.child), pack_sha256=digest(args.child),
                  content_identity=child['identity']['content_hash'],
                  unchanged_other_maps=len(child['data']['maps'])-1,
                  original_room_fields_preserved_except_blocks=True,
                  all_art_runtime_audio_exact=True, regional_policy=regional,
                  ordinary_reachable_tiles=len(seen), connected_water_tiles=len(water),
                  changed_blocks=sum(a != b for a, b in zip(old['blocks'], room['blocks'])),
                  sources={p.name: digest(p) for p in args.source.glob('*.json')})
    args.output.write_text(json.dumps(result, indent=2)+'\n')
    print(json.dumps(result, indent=2))


if __name__ == '__main__':
    main()
