#!/usr/bin/env python3
"""Render both actual connected cave levels at one native scale.

The lower source overview shows its Flash-lit, clue-read state. Letter A marks
an actual paired stair; the gap is annotation space, not invented walkable land.
Raw whole-level images are retained before adding any planning marks.
"""
import argparse
from copy import deepcopy
import json
from pathlib import Path
import sys

from PIL import Image,ImageDraw
ROOT=Path(__file__).resolve().parents[4]
sys.path.insert(0,str(ROOT/'planning/minnesota/tools'))
from render_reference import load_pack
from render_progress_atlas import native_room
from render_archive import save_render


def main():
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('--pack',type=Path,required=True);p.add_argument('--output',type=Path,required=True);args=p.parse_args();args.output.mkdir(parents=True,exist_ok=False)
    pack=load_pack(args.pack);root=pack['data']['maps']['MnR01Cave1F'];gallery=deepcopy(pack['data']['maps']['MnDriftlessLampGallery']);gallery['attributes'].update(time_of_day='nite',palette='PALETTE_NITE')
    a=native_room(root,pack,archive=args.output/'hollow',native_origin=True);b=native_room(gallery,pack,archive=args.output/'lit-gallery',native_origin=True)
    save_render(a,args.output/'hollow-native-whole.png');save_render(b,args.output/'gallery-lit-native-whole.png');offset=a.width+112
    canvas=Image.new('RGB',(offset+b.width,max(a.height,b.height)+54),'#18212a');canvas.paste(a,(0,36));canvas.paste(b,(offset,36));draw=ImageDraw.Draw(canvas)
    draw.text((8,8),'Lit hollow: original HM guide and Antidote',fill='white');draw.text((offset+8,8),'Lower gallery after FLASH and fossil reading',fill='white')
    for x,y in [(11*16,36+9*16),(offset+18*16,36+17*16)]:
        draw.rectangle((x,y,x+15,y+15),outline='#69f5d0',width=2);draw.text((x+3,y+2),'A',fill='#69f5d0')
    for y,label,color in [(80,'A <-> A','#69f5d0'),(102,'Paired stair','white'),(122,'Warp 2 / 3','white'),(160,'Same native','#a8b7c4'),(177,'pixel scale','#a8b7c4')]:draw.text((a.width+12,y),label,fill=color)
    draw.text((8,canvas.height-16),'Native source overview; actual dark/lit gameplay captures are retained separately.',fill='#a8b7c4')
    save_render(canvas,args.output/'connected-levels-original.png');save_render(canvas.resize((canvas.width*2,canvas.height*2),Image.Resampling.NEAREST),args.output/'connected-levels-2x.png')
    (args.output/'binding.json').write_text(json.dumps(dict(pack=str(args.pack),identity=pack['identity'],scope=__doc__),indent=2)+'\n');print(args.output/'connected-levels-2x.png')


if __name__=='__main__':main()
