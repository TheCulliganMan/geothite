#!/usr/bin/env python3
"""Read-only whole-grotto source geometry; native gameplay remains separate."""
import argparse
from collections import deque
import hashlib
import json
from pathlib import Path
import sys
sys.path.insert(0,str(Path(__file__).resolve().parents[2]/'tools'))
from render_reference import load_pack
cli=argparse.ArgumentParser(description=__doc__)
for name in ['pack','source','output']:cli.add_argument('--'+name,type=Path,required=True)
a=cli.parse_args();assert not a.output.exists();p=load_pack(a.pack);s=json.loads(a.source.read_text());assert s['dimensions_metatiles']==[20,14];b=[int(v,16)for row in s['rows']for v in row.split()];assert len(s['rows'])==14 and all(len(row.split())==20 for row in s['rows']);co=p['data']['tilesets']['cave']['collision'];kind=lambda q:co[f'{b[q[1]//2*20+q[0]//2]:02x}'][q[1]%2*2+q[0]%2]
bodies={(5,5),(35,3)};cells={(x,y)for y in range(28)for x in range(40)};floor={q for q in cells if kind(q)=='FLOOR'and q not in bodies};water={q for q in cells if kind(q)=='WATER'}
def neighbors(q):
 x,y=q;return[(x-1,y),(x+1,y),(x,y-1),(x,y+1)]
def route(start,target,allowed):
 todo=deque([start]);previous={start:None}
 while todo and target not in previous:
  q=todo.popleft()
  for v in neighbors(q):
   if v in allowed and v not in previous:previous[v]=q;todo.append(v)
 if target not in previous:return None
 path=[target]
 while previous[path[-1]]is not None:path.append(previous[path[-1]])
 return list(reversed(path))
def reached(start,allowed):
 todo=deque([start]);seen={start}
 while todo:
  for q in neighbors(todo.popleft()):
   if q in allowed and q not in seen:seen.add(q);todo.append(q)
 return seen
seen=reached((17,22),floor);assert seen==floor,sorted(floor-seen)
for q in [(6,5),(6,8),(6,9),(6,10),(34,3),(34,8),(34,9),(34,10),(21,19),(36,18),(28,24)]:assert q in seen,q
assert kind((35,9))=='WALL'
assert {q for q in cells if kind(q)in['LADDER','WARP_CARPET_DOWN','CAVE']}=={(17,23)}
assert not any('HOP'in kind(q)or kind(q)=='PIT'for q in cells)
assert all(0<x<39 and 0<y<27 for x,y in water)
assert reached((22,19),water)==water
assert kind((21,19))=='FLOOR' and kind((22,19))==kind((23,19))=='WATER'
assert all(kind(q)=='WALL'for q in [(28,16),(29,16),(28,17),(29,17)])
approaches={q for q in neighbors((28,16))if q in floor|water};assert approaches=={(27,16),(28,15)} and approaches<=water
routes={}
for label,start,end,allowed in [('west-fin',(13,18),(13,10),{q for q in floor if q[0]<=13}),('east-fin',(13,18),(13,10),{q for q in floor if q[0]>=13}),('north-spring',(21,19),(36,18),{q for q in floor if q[1]<=19}),('south-spring',(21,19),(36,18),{q for q in floor if q[1]>=18})]:
 path=route(start,end,allowed);assert path,label;routes[label]=dict(steps=len(path)-1,path=path)
result=dict(status='SOURCE_COLLISION_PASS',pack=str(a.pack),pack_sha256=hashlib.sha256(a.pack.read_bytes()).hexdigest(),source=str(a.source),source_sha256=hashlib.sha256(a.source.read_bytes()).hexdigest(),floor_tiles=len(floor),water_tiles=len(water),all_floor_reachable=True,all_water_one_contained_component=True,pillar_hidden_item_has_only_water_approaches=sorted(approaches),routes=routes,scope='Source graph with actor bodies blocked, native30/36 stair quadrants and the retained mouth. Does not prove native movement, item use, rest or earned progression.')
a.output.write_text(json.dumps(result,indent=2)+'\n');print('PASS',len(floor),'floor tiles,',len(water),'water tiles;', {k:v['steps']for k,v in routes.items()})
