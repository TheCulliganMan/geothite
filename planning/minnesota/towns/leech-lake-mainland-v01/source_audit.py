#!/usr/bin/env python3
"""Check the complete proposed town without building a pack; no gameplay claim."""
import argparse
from collections import deque
import hashlib
import json
from pathlib import Path
import sys
sys.path.insert(0, str(Path(__file__).resolve().parents[2] / 'tools'))
from render_reference import load_pack

cli = argparse.ArgumentParser(description=__doc__)
for key in ['pack','source','output']:
    cli.add_argument('--'+key,type=Path,required=True)
a = cli.parse_args()
assert not a.output.exists()
s = json.loads(a.source.read_text())
p = load_pack(a.pack)
old = p['data']['maps']['MnLeechLake']
b = [int(v,16) for row in s['rows'] for v in row.split()]
assert len(s['rows']) == 24 and all(len(row.split()) == 32 for row in s['rows'])
co = p['data']['tilesets']['mn_headwaters']['collision']
k = lambda q: co[f'{b[q[1]//2*32+q[0]//2]:02x}'][q[1]%2*2+q[0]%2]
bodies = {(o['x'],o['y']) for o in old['objects'] if o['script'] != 'MnLeechLakeCheck'} | {(19,24),(19,26)}

def flood(start, grass=True, surf=False, west=False):
    todo, previous = deque([start]), {start:None}
    while todo:
        x,y = todo.popleft()
        for q in [(x-1,y),(x+1,y),(x,y-1),(x,y+1)]:
            if q in previous or q in bodies or not (0 <= q[0] < 64 and 0 <= q[1] < 48) or (west and q[0] >= 22):
                continue
            allowed = {'FLOOR'} | ({'TALL_GRASS'} if grass else set()) | ({'WATER'} if surf else set())
            if k(q) not in allowed:
                continue
            previous[q] = (x,y)
            todo.append(q)
    return previous

def route(previous, end):
    if end not in previous:
        return None
    path = [end]
    while previous[path[-1]] is not None:
        path.append(previous[path[-1]])
    return list(reversed(path))

for i,(before,after) in enumerate(zip(old['blocks'],b)):
    if before != after:
        x,y = i%32,i//32
        assert 3 <= x <= 18 and 4 <= y <= 22 and before in [2,4,5]
        assert after in [2,3,4,5,0x3c,0x3d,0x3e,0x3f]
surf = flood((32,47),surf=True)
dry = flood((32,47),grass=False)
stranded = [(x,y) for y in range(48) for x in range(64)
            if k((x,y)) in ['FLOOR','TALL_GRASS'] and (x,y) not in surf and (x,y) not in bodies]
assert not stranded, stranded
assert all(k((x,y)) != 'CUT_TREE' for y in range(48) for x in range(64))
assert all(0 < x < 63 and 0 < y < 47 for x,y in surf if k((x,y)) == 'WATER')
assert all(q in dry for q in [(11,14),(9,36),(31,29),(35,22),(32,0),(33,0)])
start,end = (11,14),(9,36)
ordinary = route(flood(start,grass=False),end)
short = route(flood(start,west=True),end)
assert ordinary and short and len(ordinary)-len(short) >= 16
assert end not in flood(start,grass=False,west=True)
assert (18,24) not in flood(start,grass=False) and (18,26) not in flood(start,grass=False)
proof = dict(status='SOURCE_COLLISION_PASS',source_sha256=hashlib.sha256(a.source.read_bytes()).hexdigest(),
    parent=str(a.pack),parent_sha256=hashlib.sha256(a.pack.read_bytes()).hexdigest(),
    ordinary_steps=len(ordinary)-1,trail_steps=len(short)-1,
    trail_grass_steps=sum(k(q)=='TALL_GRASS' for q in short),ordinary_path=ordinary,trail_path=short,
    all_floor_and_grass_reachable_with_intended_native_access=True,no_cut_quadrants=True,
    unconnected_surf_edges_visibly_closed=True,original_public_approaches_retained=True,
    scope='Source collision only. Wardens removed only in this read-only reachability graph to inspect the existing gated ports; native wardens/scripts remain intact. No gameplay completion claim.')
a.output.write_text(json.dumps(proof,indent=2)+'\n')
print('PASS source:',proof['ordinary_steps'],'safe steps;',proof['trail_steps'],'trail steps;',proof['trail_grass_steps'],'grass steps')
