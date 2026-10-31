#!/usr/bin/env python3
"""Compare a closed lake-current native view with coherent source animation frames.

Uses the original Surf actor after terrain animation, retaining every pixel and
all tested frames. No masks, fitted translations or modified GPU captures.
"""
import argparse
import json
import re
from pathlib import Path
import sys
from PIL import Image, ImageChops
sys.path.insert(0,str(Path(__file__).resolve().parents[2]/'tools'))
from render_archive import save_render
from render_reference import SourceArt,load_pack
from review_city_native import native_scene_crop,phases
from review_connected_route_native import camera_origin,sources
from review_north_shore import oriented_sprite

cli=argparse.ArgumentParser(description=__doc__)
for name in ['pack','image','output']:cli.add_argument('--'+name,type=Path,required=True)
a=cli.parse_args();a.output.mkdir(parents=True,exist_ok=False)
p=load_pack(a.pack);art=SourceArt(p);m=p['data']['maps']['MnLeechLake'];attrs=m['attributes'];name=attrs['tileset_name'];x,y=59,20
box=(x*16-256,y*16-192,x*16+256,y*16+192)
base=native_scene_crop(art,m,art.render(name,32,24,m['blocks']).convert('RGBA'),box)
layout=bytes(p['runtime_files'][f'data/tilesets/{name}_metatiles.bin'])
clips=json.loads(bytes(p['runtime_files'][f'data/tilesets/{name}_animations.json']))
colors=[];group=''
for line in bytes(p['runtime_files']['gfx/tilesets/bg_tiles.pal']).decode().splitlines():
 if line.strip().startswith(';'):group=line.strip()[1:].strip().lower()
 if line.strip().startswith('RGB') and group=='day':
  values=list(map(int,re.findall(r'\d+',line.split(';')[0])))
  colors.extend(tuple((v<<3)|(v>>2)for v in values[i:i+3])for i in range(0,len(values),3))
assert len(colors)==32
placements=[]
for i,b in enumerate(m['blocks']):
 for j,g in enumerate(layout[b*16:(b+1)*16]):
  xx,yy=i%32*32+j%4*8,i//32*32+j//4*8
  if str(g)in clips and xx<box[2] and xx+8>box[0] and yy<box[3] and yy+8>box[1]:placements.append((g,xx-box[0],yy-box[1]))
assert {g for g,_,_ in placements}=={50,51,66,67}
actual=Image.open(a.image).convert('RGB');assert actual.size==(640,576)
origin,lcd=camera_origin(sources(p,'MnLeechLake'),[x,y]);best=640*576;chosen=None
actor=oriented_sprite(p,'SPRITE_SURF','south')
save_render(actor,a.output/'native-surf-actor.png')
for water,scene in phases(p,m,base,box,a.output):
 for frame in range(4):
  rendered=scene.copy()
  for glyph,xx,yy in placements:
   clip=clips[str(glyph)];assert clip['frame_ticks']==11 and len(clip['frames'])==4
   raw=clip['frames'][frame];palette_index=p['data']['tilesets'][name]['palette_map'][glyph]&7
   palette=colors[palette_index*4:palette_index*4+4];tile=Image.new('RGBA',(8,8))
   for v in range(8):
    lo,hi=raw[v*2:v*2+2]
    for u in range(8):
     bit=1<<(7-u);tile.putpixel((u,v),palette[(2 if hi&bit else 0)|(1 if lo&bit else 0)]+(255,))
   rendered.paste(tile,(xx,yy))
  rendered.alpha_composite(actor,(256,192))
  save_render(rendered,a.output/f'water{water}-whirlpool{frame}-source512.png')
  scaled=rendered.resize((1536,1152),Image.Resampling.NEAREST)
  left,top=768-origin[0],576-origin[1]
  reference=scaled.crop((left,top,left+640,top+576)).convert('RGB')
  save_render(reference,a.output/f'water{water}-whirlpool{frame}-source640.png')
  delta=ImageChops.difference(actual,reference);save_render(delta,a.output/f'water{water}-whirlpool{frame}-difference.png')
  errors=sum(v!=(0,0,0)for v in delta.getdata())
  if errors<best:best=errors;chosen=[water,frame];save_render(reference,a.output/'closest-source.png')
report=dict(status='PASS'if best==0 else'REVIEW_DIFFERENCES',pixel_mismatches=best,compared_pixels=640*576,consistent_water_and_whirlpool_frames=chosen,native_whirlpool_frame_ticks=11,derived_player_origin=origin,lcd_origin_8px_cells=lcd,excluded_pixels=0,scope='Explicit native Surf camera at59,20, south-facing, hour12; all terrain and actor pixels compared.')
(a.output/'receipt.json').write_text(json.dumps(report,indent=2)+'\n');print(report);assert best==0
