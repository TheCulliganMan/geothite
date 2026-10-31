#!/usr/bin/env python3
"""Read-only exact-parent, paired-mouth and shelf-drop audit for Rock Cut."""
import argparse
import base64
from collections import deque
from copy import deepcopy
import hashlib
import json
from pathlib import Path
import sys

ROOT = Path(__file__).resolve().parents[4]
sys.path.insert(0, str(ROOT/'planning/minnesota/tools'))
from audit_regional_species_policy import audit
from render_reference import load_pack

ROOM = 'MnR05RockCut'


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def paths(allowed, start):
    previous, queue = {start: None}, deque([start])
    while queue:
        x, y = queue.popleft()
        for at in [(x-1, y), (x+1, y), (x, y-1), (x, y+1)]:
            if at in allowed and at not in previous:
                previous[at] = (x, y)
                queue.append(at)
    return previous


def distance(previous, target):
    count = 0
    while previous[target] is not None:
        target = previous[target]
        count += 1
    return count


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ['parent', 'pack', 'output']:
        parser.add_argument('--'+name, type=Path, required=True)
    args = parser.parse_args()
    assert not args.output.exists(), 'retain prior verification results'
    source = Path(__file__).parent
    before, after, contract = [json.loads((source/(n+'.json')).read_text()) for n in ['before', 'after', 'contract']]
    parent, pack = load_pack(args.parent), load_pack(args.pack)
    old, room = parent['data']['maps'][ROOM], pack['data']['maps'][ROOM]
    blocks = lambda v: [int(t, 16) for row in v['rows'] for t in row.split()]
    assert old['blocks'] == blocks(before) and room['blocks'] == blocks(after)
    assert sha(args.parent) == contract['parent_pack_sha256']
    expected = deepcopy(parent)
    expected['identity'] = pack['identity']
    expected['data']['maps'][ROOM]['blocks'] = room['blocks']
    expected['data']['map_blocks'][old['attributes']['blocks_label']] = base64.b64encode(bytes(room['blocks'])).decode()
    assert expected == pack, 'only room terrain, mirrored block payload and derived identity may change'
    maps = pack['data']['maps']
    constants = {m['attributes']['map_constant']: n for n, m in maps.items()}
    def kind(module, x, y):
        a = module['attributes']
        return pack['data']['tilesets'][a['tileset_name']]['collision'][f'{module["blocks"][y//2*a["width"]+x//2]:02x}'][y%2*2+x%2]
    pairs = []
    for inside in room['events']['warps']:
        outside_name = constants[inside['target_map_constant']]
        outside = maps[outside_name]
        passive = next(w for w in outside['events']['warps'] if w['index'] == inside['target_warp_id'])
        active = [w for w in outside['events']['warps'] if w['target_map_constant'] == room['attributes']['map_constant'] and w['target_warp_id'] == inside['index'] and kind(outside, w['x'], w['y']) == 'CAVE']
        assert len(active) == 1
        active = active[0]
        assert [passive['x'], passive['y']] == [active['x'], active['y']+1]
        assert kind(outside, passive['x'], passive['y']) == 'FLOOR'
        assert kind(room, inside['x'], inside['y']) == 'LADDER'
        pairs.append(dict(inside=[inside['x'], inside['y']], outside=outside_name,
                          active=[active['x'], active['y']], passive=[passive['x'], passive['y']]))
    assert len(pairs) == 2
    kinds = {(x, y): kind(room, x, y) for y in range(20) for x in range(16)}
    bodies = {(o['x'], o['y']) for o in room['objects']}
    floor = {p for p, k in kinds.items() if k == 'FLOOR'}-bodies
    seen = paths(floor, (5, 16))
    assert set(seen) == floor
    assert all(k == 'WALL' for (x, y), k in kinds.items() if x in [0, 15] or y in [0, 19])
    assert all(tuple(v['tile']) in seen for v in after['review_contract']['viewpoints'])
    hops = {p for p, k in kinds.items() if k == 'HOP_DOWN'}
    assert hops == {(6, 6), (7, 6)}
    drops = []
    for x, y in sorted(hops):
        approach, landing = (x, y-1), (x, y+2)
        assert approach in floor and landing in floor
        assert all(kinds[at] == 'WALL' for at in [(x-1,y+1),(x,y+1),(x+1,y+1)])
        ordinary = distance(paths(floor, landing), approach)
        assert ordinary >= 7
        drops.append(dict(lip=[x,y], approach=approach, landing=landing, ordinary_return_steps=ordinary, descending_inputs=2))
    curation = json.loads((ROOT/'planning/minnesota/pokemon/roster-curation.json').read_text())
    roster = json.loads((ROOT/'planning/minnesota/pokemon/roster.json').read_text())
    retired = {f['id'] for f in curation['retired_families']}
    active = {m['name'].upper() for f in roster['families'] if f['id'] not in retired for m in f['members']}
    regional = audit(dict(pack, data=dict(pack['data'], maps={ROOM:room})), active)
    result = dict(status='PASS', pack=str(args.pack), pack_sha256=sha(args.pack), content_identity=pack['identity']['content_hash'],
                  parent=str(args.parent), parent_sha256=sha(args.parent), unchanged_other_maps=len(maps)-1,
                  original_events_programs_actors_services_items_art_audio_exact=True,
                  changed_metatiles=sum(a!=b for a,b in zip(old['blocks'],room['blocks'])),
                  paired_mouths=pairs, reachable_floor_tiles=len(floor), drops=drops, regional_policy=regional,
                  sources={p.name:sha(p) for p in source.glob('*.json')}, scope=__doc__)
    args.output.write_text(json.dumps(result,indent=2)+'\n')
    print(json.dumps(result,indent=2))


if __name__ == '__main__':
    main()
