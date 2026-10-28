#!/usr/bin/env python3
"""Read-only explicit Rainy River Icefields source render and physical review."""
from collections import deque
from copy import deepcopy
from pathlib import Path
import argparse
import hashlib
import importlib.util
import json
import sys
import time
from PIL import Image, ImageDraw

ROOT = Path(__file__).resolve().parents[4]
sys.path.insert(0, str(ROOT / 'planning/minnesota/tools'))
from render_reference import SourceArt, load_pack, layout_blocks, room_sheet
from render_archive import save_render
from render_stages import render_stage
from review_north_shore import oriented_sprite

AUTHORING_DIR = Path(__file__).resolve().parent
spec = importlib.util.spec_from_file_location('winter_preview', AUTHORING_DIR / 'preview_winter_art_v02.py')
winter = importlib.util.module_from_spec(spec)
spec.loader.exec_module(winter)
SHA = lambda raw: hashlib.sha256(raw).hexdigest()
PREVIEW_CACHE = {}
DIRECTIONS = [('up',(0,-1)),('right',(1,0)),('down',(0,1)),('left',(-1,0))]


def geometry(pack, layout, dossier):
    blocks = layout_blocks(layout)
    w,h = layout['dimensions_metatiles']
    collision = pack['data']['tilesets'][layout['compiled_tileset']]['collision']
    bodies = {tuple(a['tile']) for a in dossier['trainers']+dossier['residents']+dossier['items']}
    def kind(point):
        x,y=point
        if not (0<=x<w*2 and 0<=y<h*2): return 'OUTSIDE'
        block=blocks[y//2*w+x//2]
        return collision[f'{block:02x}'][(y%2)*2+x%2]
    def edge(start,direction):
        dx,dy=direction
        target=(start[0]+dx,start[1]+dy)
        if target in bodies or kind(target) not in {'FLOOR','TALL_GRASS','ICE'}: return None
        traversed=[target]
        while kind(target)=='ICE':
            following=(target[0]+dx,target[1]+dy)
            if following in bodies or kind(following) not in {'FLOOR','TALL_GRASS','ICE'}: break
            target=following;traversed.append(target)
            if len(traversed)>w*2+h*2: raise AssertionError('Unbounded ice run')
        return target,traversed
    def course(start,finish,ordinary=False):
        start,finish=tuple(start),tuple(finish);q=deque([start]);seen={start:None}
        while q:
            at=q.popleft()
            if at==finish: break
            for label,direction in DIRECTIONS:
                result=edge(at,direction)
                if result is None:continue
                target,steps=result
                if ordinary and any(kind(point)=='ICE' for point in steps):continue
                if target not in seen:seen[target]=(at,label,steps);q.append(target)
        if finish not in seen:return None
        path=[];at=finish
        while seen[at] is not None:
            previous,label,steps=seen[at];path.append(dict(button=label,from_core=list(previous),to_core=list(at),traversed=[list(x)for x in steps]));at=previous
        return list(reversed(path))
    return blocks,kind,edge,course,bodies


def review(native, layout, dossier, art_spec, output):
    started=time.perf_counter()
    key=(id(native),SHA(json.dumps(art_spec,sort_keys=True).encode()))
    if key not in PREVIEW_CACHE:
        preview,receipts=winter.derive(native,art_spec)
        PREVIEW_CACHE[key]=(preview,receipts,SourceArt(preview))
    pack,receipts,art=PREVIEW_CACHE[key]
    blocks,kind,edge,course,bodies=geometry(pack,layout,dossier)
    w,h=layout['dimensions_metatiles'];checks=[];errors=[]
    def check(label,predicate):
        (checks if predicate else errors).append(label)
    for layer in layout['layers']:
        if layer['kind']=='polygon':
            points=layer['points']
            check(layer['name']+' exact orthogonal edges',all(a[0]==b[0]or a[1]==b[1]for a,b in zip(points,points[1:]+points[:1])))
    entry=tuple(dossier['arrival_core'])
    check('entry safe unoccupied snow',kind(entry)=='FLOOR' and entry not in bodies)
    allowed={(63,44),(63,45)}
    perimeter={(x,y)for y in range(h*2)for x in range(w*2)if x in [0,w*2-1]or y in [0,h*2-1]}
    exposed={p for p in perimeter if kind(p) in {'FLOOR','TALL_GRASS','ICE','WATER'}}
    check('only actual reciprocal eastern port open',exposed==allowed)
    check('no incidental Cut gate',all('CUT_TREE' not in pack['data']['tilesets'][layout['compiled_tileset']]['collision'][f'{b:02x}']for b in set(blocks)))
    check('all three distinct field materials present',all(b in blocks for b in [60,61,62]))
    for block,token in [(60,'FLOOR'),(61,'TALL_GRASS'),(62,'ICE')]:
        check(f'{block:02x} exact native field semantics',pack['data']['tilesets'][layout['compiled_tileset']]['collision'][f'{block:02x}']==[token]*4)
    for actor in dossier['trainers']+dossier['residents']:
        lines=actor.get('lines',[])+actor.get('before_lines',[])+actor.get('after_lines',[])+actor.get('loss_lines',[])
        check(actor['id']+' readable native18column text',all(len(line)<=18 for line in lines))
    reports=[]
    for actor in dossier['trainers']+dossier['residents']+dossier['items']:
        at=tuple(actor['tile']);check(actor['id']+' ordinary actor terrain',kind(at)=='FLOOR')
        paths=[course(entry,(at[0]+dx,at[1]+dy))for _,(dx,dy)in DIRECTIONS]
        paths=[p for p in paths if p is not None]
        check(actor['id']+' reachable interaction',bool(paths))
        reports.append(dict(id=actor['id'],reachable=bool(paths),minimum_core_steps=min([sum(len(e['traversed'])for e in p)for p in paths],default=None)))
    check('native complete door and clear return',kind((9,27))=='DOOR'and kind((9,28))=='FLOOR' and course(entry,(9,28))is not None)
    check('real lower crossing west slide',edge((54,36),(-1,0))==((37,36),[(x,36)for x in range(53,36,-1)]))
    check('real lower crossing east slide',edge((37,36),(1,0))==((54,36),[(x,36)for x in range(38,55)]))
    snow_return=course((37,36),(54,36),ordinary=True)
    check('ordinary snow return exists',snow_return is not None)
    check('upper cache island down slide',edge((19,7),(0,1))==((19,10),[(19,y)for y in range(8,11)]))
    check('upper cache island north return',edge((19,10),(0,-1))==((19,7),[(19,y)for y in range(9,6,-1)]))
    check('cache island actually requires ice',course(entry,(20,11),ordinary=True)is None)
    for trainer in dossier['trainers']:
        for member in trainer['party']:
            check(member['species']+' installed native species',member['species'] in native['data']['pokemon']and 1<=member['level']<=100)
    for item in dossier['items']:check(item['item']+' installed native item',item['item']in native['data']['items'])
    terrain=art.render(layout['compiled_tileset'],w,h,blocks)
    save_render(terrain,output/'whole-native-source-terrain.png')
    scene=render_stage(terrain,'convert',output,'terrain-rgba','RGBA')
    for actor in dossier['trainers']+dossier['residents']+dossier['items']:
        frame=oriented_sprite(pack,actor.get('sprite','SPRITE_POKE_BALL'),actor.get('facing','south'))
        save_render(frame,output/(actor['id']+'-native-frame.png'))
        scene.alpha_composite(frame,(actor['tile'][0]*16,actor['tile'][1]*16))
    save_render(scene,output/'whole-native-source-rgba.png')
    scene=render_stage(scene,'convert',output,'whole-rgb','RGB')
    save_render(scene,output/'whole-native-source.png')
    save_render(scene.resize((w*16,h*16),Image.Resampling.NEAREST),output/'whole-preview-half.png')
    cameras=[]
    for view in dossier['viewpoints']:
        label=view['id'];x,y=view['tile']
        check(label+' legal unoccupied Floor camera',kind((x,y))=='FLOOR'and(x,y)not in bodies)
        crop=render_stage(scene,'crop',output,label,(x*16-72,y*16-64,x*16+88,y*16+80)).convert('RGBA')
        player=oriented_sprite(pack,'SPRITE_CHRIS','south');crop.alpha_composite(player,(72,64))
        save_render(crop,output/(label+'-source160-rgba.png'))
        crop=render_stage(crop,'convert',output,label+'-rgb','RGB');save_render(crop,output/(label+'-source160.png'))
        zoom=render_stage(crop,'resize',output,label+'-4x',(640,576),Image.Resampling.NEAREST);save_render(zoom,output/(label+'-4x.png'))
        cameras.append((label,crop))
    save_render(room_sheet(cameras,columns=3),output/'decision-cameras.png')
    for room in dossier['interiors']:
        values=[int(b,16)for row in room['rows']for b in row.split()]
        rw,rh=room['dimensions_metatiles'];rc=native['data']['tilesets'][room['tileset']]['collision']
        def room_kind(p):
            x,y=p
            if not(0<=x<rw*2 and 0<=y<rh*2):return 'OUTSIDE'
            return rc[f'{values[y//2*rw+x//2]:02x}'][y%2*2+x%2]
        occupied={tuple(a['tile'])for a in room['residents']}
        clear=tuple(room['clear_arrival'])
        check(room['id']+' safe passive arrival',room_kind(clear)=='FLOOR'and clear not in occupied)
        check(room['id']+' exact carpet exit',all(room_kind(tuple(exit))==room['exit_collision']for exit in room['exits']))
        check(room['id']+' exact explicit paired exterior return',room['exit_doors']==[[9,27]]and dossier['doors'][0]['target']==room['id']and dossier['doors'][0]['return_warp_index']==2 and room['exit_target_warps']==[2])
        for actor in room['residents']:
            check(actor['id']+' legal Floor body',room_kind(tuple(actor['tile']))=='FLOOR')
            check(actor['id']+' native18column lines',all(len(line)<=18 for line in actor['lines']))
        im=art.render(room['tileset'],*room['dimensions_metatiles'],values,'indoor').convert('RGBA')
        for actor in room['residents']:
            im.alpha_composite(oriented_sprite(pack,actor['sprite'],actor['facing']),(actor['tile'][0]*16,actor['tile'][1]*16))
        save_render(im,output/(room['id']+'-whole-original.png'))
        save_render(im.resize((im.width*3,im.height*3),Image.Resampling.NEAREST),output/(room['id']+'-whole-3x.png'))
    report=dict(status='SOURCE_ONLY_NOT_NATIVE_GAMEPLAY',checks=checks,errors=errors,source_seconds=round(time.perf_counter()-started,3),art_receipts=receipts,actor_interactions=reports,snow_return_core_steps=sum(len(e['traversed'])for e in snow_return)if snow_return else None,source_art_spec_sha256=SHA(json.dumps(art_spec,indent=2).encode()+b'\n'),native_maps_unchanged=pack['data']['maps']==native['data']['maps'])
    (output/'review.json').write_text(json.dumps(report,indent=2)+'\n')
    (output/'source-controller-plan.json').write_text(json.dumps(dict(ordinary_snow_return=snow_return,port_to_shelter=course(entry,(9,28)),lower_glide_west=edge((54,36),(-1,0)),lower_glide_east=edge((37,36),(1,0))),indent=2)+'\n')
    print(json.dumps(report,indent=2))
    return report


if __name__=='__main__':
    parser=argparse.ArgumentParser();parser.add_argument('--pack',type=Path,required=True);parser.add_argument('--output',type=Path,required=True);args=parser.parse_args()
    output=args.output;output.mkdir(parents=True,exist_ok=True)
    files={'MnRainyRiverIcefields.layout.json':AUTHORING_DIR/'MnRainyRiverIcefields.layout.json','MnRainyRiverIcefields.json':AUTHORING_DIR/'MnRainyRiverIcefields.json','winter-art-v01.json':AUTHORING_DIR/'winter-art-v01.json'}
    for name,path in files.items():(output/name).write_bytes(path.read_bytes())
    (output/'review_rainy_river.py').write_bytes(Path(__file__).read_bytes())
    pack=load_pack(args.pack);report=review(pack,json.loads(files['MnRainyRiverIcefields.layout.json'].read_text()),json.loads(files['MnRainyRiverIcefields.json'].read_text()),json.loads(files['winter-art-v01.json'].read_text()),output)
    raise SystemExit(bool(report['errors']))
