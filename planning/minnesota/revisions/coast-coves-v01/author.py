#!/usr/bin/env python3
"""Explicit native coastal headlands; author source only, never write packs."""
import copy,json,sys
from pathlib import Path
ROOT=Path(__file__).resolve().parents[4];sys.path.insert(0,str(ROOT/'planning/minnesota/tools'))
from render_reference import SourceArt,load_pack,layout_blocks
from PIL import Image,ImageDraw
D=Path(__file__).resolve().parent;O=ROOT/'planning/minnesota/renders/coast-coves-v01'/str(__import__('time').time_ns());O.mkdir(parents=True,exist_ok=False)
p=load_pack(ROOT/'content-packs/minnesota-r10-rock-gallery-v01.crystalpack');layout=json.loads((D/'MN_R11.before.layout.json' if (D/'MN_R11.before.layout.json').exists() else ROOT/'planning/minnesota/routes/MN_R11.layout.json').read_text());W,H=56,96;b=[5]*(W*H)
for n,m in p['data']['maps'].items():
 if n.startswith('MnR11C'):
  ox=int(n[6:8])*32;oy=int(n[9:11])*24;w=m['attributes']['width'];h=m['attributes']['height']
  for y in range(h):b[(oy+y)*W+ox:(oy+y)*W+ox+w]=m['blocks'][y*w:(y+1)*w]
old=layout_blocks(layout);print('source/native drift',sum(x!=y for x,y in zip(old,b)))
# Exact installed warm coast preimage, with missing ragged terminal padding closed.
for i,(v,z) in enumerate(zip(old,b)):
 if v!=z:layout['layers'].append(dict(name=f'installed warm compact cell {i%W},{i//W}',kind='point',position=[i%W,i//W],block=f'{z:02x}'))
assert layout_blocks(layout)==b
(D/'MN_R11.before.layout.json').write_text(json.dumps(layout,indent=2)+'\n')
a=b.copy()
def rect(x0,y0,x1,y1,v):
 for y in range(y0,y1+1):
  for x in range(x0,x1+1):a[y*W+x]=v
# Split Rock's short basalt finger: stepped pine shoulders, sedge saddle, lawn tip.
for y,end in [(30,24),(31,26),(32,28),(33,31),(34,31),(35,29),(36,27),(37,24)]:rect(23,y,end,y,5)
rect(21,33,27,33,1);rect(26,33,27,33,3);rect(28,33,29,34,2);rect(28,34,28,34,1)
# Deep wooded cove north of the middle cape; retain all original dry path cells.
for y,start in [(39,23),(40,22),(41,22),(42,23)]:
 for x in range(start,25):
  if b[y*W+x]==5:a[y*W+x]=0x35
# Long quiet reach gains a sheltered hooked headland, open to actual Superior.
for y,end in [(65,24),(66,27),(67,31),(68,32),(69,31),(70,27)]:rect(23,y,end,y,5)
rect(18,68,27,68,1);rect(26,68,27,68,3);rect(28,67,29,68,2);rect(29,68,30,68,1)
# South pine cape gives the southern cascade an off-road lake view.
for y,start,end in [(85,29,29),(86,27,31),(87,27,33),(88,27,35),(89,27,34),(90,27,32),(91,29,30)]:rect(start,y,end,y,5)
rect(25,89,30,89,1);rect(27,89,28,89,3);rect(31,88,32,89,2);rect(32,89,32,89,1)
# Fit original native warm-water glyphs around each stepped contour.
water={k for k,v in p['data']['tilesets']['mn_north_shore']['collision'].items() if v==['WATER']*4};water={int(k,16) for k in water};snapshot=a.copy()
def land(x,y):return 0<=x<W and 0<=y<H and snapshot[y*W+x]not in water
for y0,y1 in [(29,43),(64,71),(84,92)]:
 for y in range(y0,y1+1):
  for x in range(21,38):
   if snapshot[y*W+x]not in water:continue
   n,we,e=land(x,y-1),land(x-1,y),land(x+1,y)
   a[y*W+x]=0x54 if n and we else 0x55 if n and e else 0x58 if we else 0x59 if e else 0x76 if n else 0x79 if land(x-1,y-1)else 0x7a if land(x+1,y-1)else 0x35
a[33*W+22]=b[33*W+22] # Retain the exact native pickup-anchor block.
changes=[]
for i,(v,z) in enumerate(zip(b,a)):
 if v!=z:
  x,y=i%W,i//W;changes.append(dict(metatile=[x,y],before=f'{v:02x}',after=f'{z:02x}'));layout['layers'].append(dict(name=f'coastal cove native cell {x},{y}',kind='point',position=[x,y],block=f'{z:02x}'))
assert layout_blocks(layout)==a
(D/'MN_R11.candidate.layout.json').write_text(json.dumps(layout,indent=2)+'\n');(D/'delta.json').write_text(json.dumps(changes,indent=2)+'\n');print('changed',len(changes))
art=SourceArt(p)
for label,blocks in [('before',b),('candidate',a)]:
 im=art.render('mn_north_shore',W,H,blocks)
 # Ragged north-east terminal is absent, not fictitious terrain.
 ImageDraw.Draw(im).rectangle((36*32,0,56*32,24*32-1),fill='#202020')
 im.save(O/f'{label}-native.png');preview=im.copy();preview.thumbnail((700,1200));preview.save(O/f'{label}-preview.png')
 for name,x,y in [('split-rock-finger',58,68),('quiet-middle-cape',60,136),('southern-pine-cape',64,178),('weak-approach',35,135)]:
  im.crop((x*16-80,y*16-72,x*16+80,y*16+72)).resize((640,576),Image.Resampling.NEAREST).save(O/f'{label}-{name}.png')
