#!/usr/bin/env python3
"""Source-art pixel crops at declared core positions; not a production camera."""
from pathlib import Path
from PIL import Image,ImageDraw
import json
ROOT=Path(__file__).resolve().parents[5];OUT=ROOT/'planning/minnesota/renders/north-shore-ely-geography-v01/cedar-neck-v03';source=Image.open(OUT/'neck-cedar-connected-native.png');cameras=[('neck-icy-grass-saddle','Neck',[7,14]),('neck-cedar-seam','Cedar',[0,8]),('cedar-west-port','Cedar',[4,8]),('keeper-clearing-widest-way','Cedar',[29,18]),('snowline-step','Cedar',[47,17]),('overlook-stair','Cedar',[55,21]),('overlook-trainer','Cedar',[55,18]),('town-side-warm-port','Cedar',[57,8])];sheet=Image.new('RGB',(4*660,2*626),'#14212c');d=ImageDraw.Draw(sheet)
receipt=[]
for i,(name,m,(x,y))in enumerate(cameras):
 px=(x+(16 if m=='Cedar'else 0))*16+8;py=y*16+8;crop=(px-80,py-72,px+80,py+72);im=source.crop(crop);im.save(OUT/f'{name}-native160.png');im.resize((640,576),Image.Resampling.NEAREST).save(OUT/f'{name}-4x.png');xx=i%4*660+10;yy=i//4*626+36;sheet.paste(im.resize((640,576),Image.Resampling.NEAREST),(xx,yy));d.text((xx,yy-27),f'{name} / {m} core{x},{y}',fill='white');d.text((xx,yy-13),'Source crop only / no GPU, collision or clamp claim',fill='#c7deed');receipt.append(dict(name=name,map=m,core=[x,y],crop_composite_pixels=list(crop)))
sheet.save(OUT/'player-scale-source-contact.png');(OUT/'source-camera-positions.json').write_text(json.dumps(dict(status='Eight160x144 source-art crops, integer4x; no production traversal, GPU or camera-clamp acceptance',cameras=receipt),indent=2)+'\n')
