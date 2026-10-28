#!/usr/bin/env python3
"""Capture the complete ice review course with the headless production renderer."""
import argparse
import json
import os
from pathlib import Path
import shutil
import subprocess

from PIL import Image, ImageChops

from render_archive import retain_render_file, save_render
from render_reference import SourceArt, load_pack, npc_palette_pack, room_sheet
from review_city_native import priority_scene
from review_connected_route_native import camera_origin
from review_north_shore import oriented_sprite
from review_passage_native import actors, digest


def compare(pack, art, module, at, actual, output):
    a = module['attributes']
    group = a['time_of_day'] or 'day'
    terrain = art.render(a['tileset_name'],a['width'],a['height'],module['blocks'],group)
    sprites = npc_palette_pack(pack,group)
    scene = actors(sprites,terrain,module)
    x,y = at
    scene.alpha_composite(oriented_sprite(sprites,'SPRITE_CHRIS','south'),(x*16,y*16))
    box=(x*16-256,y*16-192,x*16+256,y*16+192)
    priority_scene(pack,module,scene,terrain,box)
    left,top=box[0]//32*32,box[1]//32*32
    right,bottom=(box[2]+31)//32*32,(box[3]+31)//32*32
    padded=Image.new('RGBA',(right-left,bottom-top))
    border=art.blocks(a['tileset_name'],group)[a['border_block']]
    for yy in range(0,padded.height,32):
        for xx in range(0,padded.width,32):padded.paste(border,(xx,yy))
    padded.alpha_composite(scene,(-left,-top))
    crop=padded.crop((box[0]-left,box[1]-top,box[2]-left,box[3]-top))
    save_render(crop,output/'source-original512.png')
    origin,_=camera_origin([(module,0,0)],at)
    # The original Ice Path atlas scrolls visible glyph $b5 left and $b1
    # down at the native 19-frame schedule. Match a single coherent clock
    # across all occurrences; the production image is never modified.
    layout=bytes(pack['runtime_files']['data/tilesets/ice_path_metatiles.bin'])
    animated=[]
    for index,block in enumerate(module['blocks']):
        for j,glyph in enumerate(layout[block*16:(block+1)*16]):
            gx,gy=index%a['width']*32+j%4*8,index//a['width']*32+j//4*8
            if glyph in (0xb5,0xb1) and gx >= box[0] and gx+8 <= box[2] and gy >= box[1] and gy+8 <= box[3]:
                animated.append((glyph,gx-box[0],gy-box[1],terrain.crop((gx,gy,gx+8,gy+8))))
    best=actual.width*actual.height
    best_clock=None
    seen=set()
    for clock in (range(168) if animated else [0]):
        phase=lambda offset: 0 if clock < offset else ((clock-offset)//19+1)%8
        pair=(phase(4),phase(16))
        if pair in seen:continue
        seen.add(pair)
        variant=crop.copy()
        for glyph,gx,gy,original in animated:
            tile=ImageChops.offset(original,-pair[0],0) if glyph==0xb5 else ImageChops.offset(original,0,pair[1]*3)
            variant.paste(tile,(gx,gy))
        scaled=variant.resize((1536,1152),Image.Resampling.NEAREST)
        lx,ty=768-origin[0],576-origin[1]
        reference=scaled.crop((lx,ty,lx+640,ty+576)).convert('RGB')
        delta=ImageChops.difference(actual,reference)
        errors=sum(v != (0,0,0) for v in delta.getdata())
        if errors < best:
            best,best_clock=errors,clock
            save_render(reference,output/'source-reference.png')
            save_render(delta,output/'difference.png')
            if best == 0:break
    return best,best_clock



def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--pack',type=Path,required=True)
    parser.add_argument('--binary',type=Path,required=True)
    parser.add_argument('--source',type=Path,default=Path('planning/minnesota/ice-caves/voyageurs-ice.json'))
    parser.add_argument('--output',type=Path,required=True)
    args=parser.parse_args()
    output=args.output.resolve();output.mkdir(parents=True,exist_ok=False)
    pack_path=args.pack.resolve();pack=load_pack(pack_path);art=SourceArt(pack)
    private=Path('.local')/f'ice-review-capture-{os.getpid()}'
    shutil.copy2(args.binary,private)
    report={'pack_sha256':digest(pack_path),'binary_sha256':digest(private),'captures':[],
            'fixture':'Disposable location-tester party fixture, declared position, south facing at noon; native special ice palette.'}
    env=dict(os.environ,CRYSTAL_RENDER_TEST_HEADLESS='1',CRYSTAL_RENDER_TEST_CAPTURE_TARGET='image',CRYSTAL_RENDER_TEST_WINDOW_SIZE='640x576',CRYSTAL_PREVIEW_MUTE='1')
    env.pop('CRYSTAL_RENDER_TEST_SECOND_VIEW',None)
    try:
        for room in json.loads(args.source.read_text())['rooms']:
            views=[]
            for view in room['review_viewpoints']:
                folder=output/(room['id']+'-'+view['id']);folder.mkdir()
                png=folder/'production.png';x,y=view['tile']
                command=[str(private.resolve()),'--pack',str(pack_path),'--map',room['id'],'--x',str(x),'--y',str(y),'--hour','12','--view','2d','--screenshot',str(png)]
                with (folder/'capture.log').open('wb') as log:
                    result=subprocess.run(command,env=env,stdout=log,stderr=subprocess.STDOUT,timeout=120)
                assert result.returncode == 0,folder
                retain_render_file(png);actual=Image.open(png).convert('RGB')
                errors,clock=compare(pack,art,pack['data']['maps'][room['id']],view['tile'],actual,folder)
                report['captures'].append({'id':folder.name,'pixel_mismatches':errors,'coherent_ice_clock':clock,'production_sha256':digest(png)})
                print(folder.name,errors,flush=True)
                (output/'capture-progress.json').write_text(json.dumps(report,indent=2)+'\n')
                views.append((view['id'],actual))
            save_render(room_sheet(views,columns=3),output/(room['id']+'-production-cameras.png'))
    finally:
        private.unlink()
        assert digest(pack_path) == report['pack_sha256']
        report['status']='PASS' if len(report['captures']) == 9 and all(r['pixel_mismatches'] == 0 for r in report['captures']) else 'REVIEW'
        (output/'capture-progress.json').write_text(json.dumps(report,indent=2)+'\n')
    assert report['status'] == 'PASS', 'inspect retained full-frame differences'


if __name__ == '__main__':main()
