#!/usr/bin/env python3
"""Read-only native collision proof for the three hand-authored coast capes."""
import json,sys
from collections import deque
from pathlib import Path
ROOT=Path(__file__).resolve().parents[4];sys.path.insert(0,str(ROOT/'planning/minnesota/tools'))
from render_reference import load_pack,layout_blocks
p=load_pack(ROOT/'content-packs/minnesota-r10-rock-gallery-v01.crystalpack');d=Path(__file__).resolve().parent;b=layout_blocks(json.loads((d/'MN_R11.before.layout.json').read_text()));a=layout_blocks(json.loads((d/'MN_R11.candidate.layout.json').read_text()));c=p['data']['tilesets']['mn_north_shore']['collision'];W,H=112,192
occupied=set()
for n,m in p['data']['maps'].items():
 if n.startswith('MnR11C'):
  ox=int(n[6:8])*64;oy=int(n[9:11])*48
  for o in m['objects']:occupied.add((ox+o['x'],oy+o['y']))
def kind(blocks,q):
 x,y=q
 return c[f'{blocks[y//2*56+x//2]:02x}'][y%2*2+x%2]
def flood(blocks,start,mode):
 seen={start};q=deque([start])
 while q:
  x,y=q.popleft()
  for v in [(x-1,y),(x+1,y),(x,y-1),(x,y+1)]:
   xx,yy=v
   if 0<=xx<W and 0<=yy<H and v not in seen and v not in occupied and kind(blocks,v) in mode:seen.add(v);q.append(v)
 return seen
for name,start,end,grass in [('split-rock',(40,66),(58,68),(52,66)),('middle',(34,136),(60,136),(52,136)),('south',(48,178),(64,178),(54,178))]:
 assert end in flood(a,start,{'FLOOR','TALL_GRASS','LADDER'}),(name,'native walking access')
 assert end not in flood(a,start,{'FLOOR','LADDER'}),(name,'grass saddle is avoidable')
 assert kind(a,grass)=='TALL_GRASS'
 x,y=end;assert any(kind(a,(xx,yy))=='WATER' for xx in range(x-5,x+6)for yy in range(y-4,y+5)),name+' real water in player camera'
 assert all(kind(a,(x,y))!='ICE' for y in range(H)for x in range(W)),name+' noICE'
 print(name,'reachable grass-priced view')
# Every boundary quadrant remains byte-for-byte identical. Paired water seams retained.
assert all(a[y*56+x]==b[y*56+x]for y in range(96)for x in range(56)if x in [0,55]or y in [0,95])
sea=flood(a,(90,80),{'WATER','WATERFALL'});assert all(q in sea for q in [(90,50),(90,94),(90,96),(90,142),(90,144),(90,188)]),'continuous actual Superior'
print('native sea continuous; route perimeter unchanged; all three capes require their grass saddles')
