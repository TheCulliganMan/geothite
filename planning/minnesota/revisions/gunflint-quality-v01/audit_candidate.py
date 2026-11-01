#!/usr/bin/env python3
"""Read-only source traversal audit, including actual straight ICE transitions."""
import json,sys
from pathlib import Path
from collections import deque,Counter
ROOT=Path(__file__).resolve().parents[4];sys.path.insert(0,str(ROOT/'planning/minnesota/tools'))
from render_reference import load_pack,layout_blocks
src=Path(sys.argv[1]);p=load_pack(ROOT/'content-packs/minnesota-superior-inland-winter-v01.crystalpack');l=json.loads((src/'MN_R12.candidate.layout.json').read_text());d=json.loads((src/'MN_R12.candidate.json').read_text());b=layout_blocks(l);w,h=l['dimensions_metatiles'];coll=dict(p['data']['tilesets'][l['compiled_tileset']]['collision'])
for r in l['derived_blocks']:
 if r['id'] not in coll:coll[r['id']]=p['data']['tilesets'][r['source_tileset']]['collision'][r['source_block']]
FOOT={'FLOOR','TALL_GRASS','DOOR','CAVE','LADDER','WARP_CARPET_DOWN','WARP_CARPET_RIGHT'}
def kind(at,opened=False):
 x,y=at
 if not(0<=x<w*2 and 0<=y<h*2):return 'OUTSIDE'
 block=b[y//2*w+x//2]
 if opened and [x//2,y//2]==[v//2 for v in d['cut_gates'][0]['tile']]:block=int(d['cut_gates'][0]['replacement_block'],16)
 return coll[f'{block:02x}'][(y%2)*2+x%2]
occupied={tuple(o['tile'])for field in ['residents','trainers','items']for o in d[field]}
doors={tuple(o['tile']) for o in d['doors']}
def nexts(at,opened=False):
 x,y=at
 for dx,dy in [(1,0),(-1,0),(0,1),(0,-1)]:
  cur=(x+dx,y+dy)
  if cur in occupied or cur in doors:continue
  k=kind(cur,opened)
  if k not in FOOT|{'ICE'}:continue
  if k=='ICE':
   while True:
    n=(cur[0]+dx,cur[1]+dy);nk=kind(n,opened)
    if n in occupied or n in doors or nk not in FOOT|{'ICE'}:break
    cur=n
    if nk!='ICE':break
  yield cur

def flood(start,opened=False):
 dist={start:0};q=deque([start])
 while q:
  a=q.popleft()
  for c in nexts(a,opened):
   if c not in dist:dist[c]=dist[a]+1;q.append(c)
 return dist
start=(62,54);before=flood(start);after=flood(start,True)
report=dict(status='STATIC_SOURCE_GRAPH_NOT_CONTROLLER',dimensions=[w,h],reachable_before=len(before),reachable_after=len(after),native_counts=Counter(f'{x:02x}'for x in b),ports=[],actors=[],doors=[],cut_gate={})
for q in [(63,54),(63,55),(0,28),(0,29)]:report['ports'].append(dict(core=q,kind=kind(q),reachable=q in before,inputs=before.get(q)))
for field in ['residents','trainers','items','signs']:
 for o in d[field]:
  x,y=o['tile'];ns=[(x+1,y),(x-1,y),(x,y+1),(x,y-1)];r=[q for q in ns if q in before];ra=[q for q in ns if q in after]
  report['actors'].append(dict(id=o['id'],tile=o['tile'],kind=kind((x,y)),approaches_before=r,approaches_after=ra,expected_cut_gated=o['id']=='MN_R12_ITEM_01'))
for door in d['doors']:report['doors'].append(dict(id=door['id'],tile=door['tile'],kind=kind(tuple(door['tile'])),return_tile=door['return_tile'],return_reachable=tuple(door['return_tile'])in before))
report['cut_gate']=dict(tree=d['cut_gates'][0]['tile'],kind=kind(tuple(d['cut_gates'][0]['tile'])),approach_reachable=tuple(d['cut_gates'][0]['approach'])in before,targets=[dict(core=q,before=tuple(q)in before,after=tuple(q)in after)for q in d['cut_gates'][0]['targets']])
report['open_outer_boundaries']=[dict(core=[x,y],kind=kind((x,y)))for y in range(h*2)for x in range(w*2)if(x in [0,w*2-1]or y in [0,h*2-1])and kind((x,y))not in ['WALL','HEADBUTT_TREE']]
print(json.dumps(report,indent=2))
