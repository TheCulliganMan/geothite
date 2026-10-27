#!/usr/bin/env python3
"""Render and check explicit Grand Rapids source; never exports a game pack."""
import argparse
from collections import deque
import json
from pathlib import Path

from PIL import Image, ImageChops

from render_archive import save_render
from render_reference import SourceArt, layout_blocks, load_pack, room_sheet
from render_stages import render_stage
import review_duluth

ROOT = Path(__file__).resolve().parents[3]
OUT = ROOT/'planning/minnesota/renders/grand-rapids'



def connection_context(pack, town, town_scene):
    """Use actual adjacent chunks; preserve source/native viewport comparisons."""
    art=SourceArt(pack);world=Image.new('RGBA',(2816,864))
    for name,origin in [('MnR09C01R00',(0,0)),('MnR10C00R00',(1792,224))]:
        m=pack['data']['maps'][name];a=m['attributes'];blocks=list(m['blocks'])
        for patch in town['regional_connections']['patches']:
            if patch['map']!=name:continue
            x,y=patch['metatile'];i=y*a['width']+x
            assert blocks[i] in [int(patch['expected'],16),int(patch['block'],16)],'stale neighboring mouth'
            blocks[i]=int(patch['block'],16)
        terrain=art.render(a['tileset_name'],a['width'],a['height'],blocks,'day')
        save_render(terrain,OUT/(name+'-context-terrain.png'))
        scene=render_stage(terrain,'convert',OUT,name+'-context-scene','RGBA')
        for actor in m['objects']:
            move=actor['spritemovedata']
            facing='north' if move.endswith('_UP') else 'west' if move.endswith('_LEFT') else 'east' if move.endswith('_RIGHT') else 'south'
            scene.alpha_composite(review_duluth.oriented_sprite(pack,actor['sprite'],facing),(actor['x']*16,actor['y']*16))
        save_render(scene,OUT/(name+'-context-scene.png'));world.alpha_composite(scene,origin)
    world.alpha_composite(town_scene,(1024,0));save_render(world,OUT/'connection-world-source.png')
    scaled=render_stage(world,'resize',OUT,'connection-world-native-scale',(8448,2592),Image.Resampling.NEAREST)
    player=review_duluth.oriented_sprite(pack,'SPRITE_CHRIS','south')
    player=render_stage(player,'resize',OUT,'native-reference-player',(48,48),Image.Resampling.NEAREST)
    for label,(x,y) in [('east-checkpoint',(43,24)),('west-checkpoint',(4,24))]:
        view=Image.new('RGBA',(1280,800));view.paste(scaled,(640-(1024+x*16)*3,400-y*48));view.alpha_composite(player,(640,400))
        save_render(view,OUT/(label+'-reference-rgba.png'))
        reference=render_stage(view,'convert',OUT,label+'-reference','RGB');save_render(reference,OUT/(label+'-reference.png'))
        native_path=OUT/('native-'+label+'-2d.png')
        if native_path.exists():
            native=Image.open(native_path);native=render_stage(native,'convert',OUT,label+'-native','RGB')
            diff=ImageChops.difference(native,reference);save_render(diff,OUT/(label+'-native-source-difference.png'))
            print(label,'different source/native pixels:',sum(p!=(0,0,0) for p in diff.getdata()),'of',1280*800)

def review(pack_path, published=False, connected=False):
    global OUT
    if connected: OUT=OUT/"connected"
    review_duluth.OUTPUT=OUT
    pack=load_pack(pack_path);art=SourceArt(pack)
    town=json.loads((ROOT/'planning/minnesota/towns/MnGrandRapids.json').read_text())
    layout=json.loads((ROOT/'planning/minnesota/towns/MnGrandRapids.layout.json').read_text())
    specs=[dict(id=town['id'],tileset='johto_modern',dimensions_metatiles=[24,24],residents=town['residents'],blocks=layout_blocks(layout),start=[6,24])]
    specs += [dict(r,blocks=[int(b,16) for row in r['rows'] for b in row.split()],start=r['passive_arrival']['tile']) for r in town['interiors']]
    if connected:
        exterior=specs[0]
        exterior['residents']=list(exterior['residents'])
        for patch in town['regional_connections']['patches']:
            if patch['map']!=town['id']:continue
            x,y=patch['metatile'];index=y*24+x
            assert exterior['blocks'][index]==int(patch['expected'],16),'stale source mouth'
            exterior['blocks'][index]=int(patch['block'],16)
        for passage in town['regional_connections']['passages']:
            for name,at in zip(passage['identifiers'],passage['bodies']):
                exterior['residents'].append(dict(id=name,tile=at,sprite='SPRITE_OFFICER',facing='east' if passage['incoming_facing']=='RIGHT' else 'west'))

    rooms=[]
    for s in specs:
        w,h=s['dimensions_metatiles'];collision=pack['data']['tilesets'][s['tileset']]['collision'];blocks=s['blocks']
        def kind(p):
            x,y=p
            return collision[f'{blocks[y//2*w+x//2]:02x}'][y%2*2+x%2]
        occupied={tuple(a['tile']) for a in s['residents']}
        start=tuple(s['start']);seen={start};todo=deque([start])
        while todo:
            x,y=todo.popleft()
            for q in [(x-1,y),(x+1,y),(x,y-1),(x,y+1)]:
                if q in seen or q in occupied or not(0<=q[0]<w*2 and 0<=q[1]<h*2):continue
                if kind(q) not in {'FLOOR','DOOR','WARP_CARPET_DOWN'}:continue
                seen.add(q)
                if kind(q) not in {'DOOR','WARP_CARPET_DOWN'}:todo.append(q)
        assert kind(start)=='FLOOR',(s['id'],'arrival',start,kind(start))
        for a in s['residents']:
            x,y=a['tile'];approaches=[(x-1,y),(x+1,y),(x,y-1),(x,y+1)]
            if a.get('counter_tile'):approaches.append(tuple(a['approach']))
            assert kind((x,y))=='FLOOR' and set(approaches)&seen,(s['id'],a['id'],'unreachable',kind((x,y)))
        if s['id']==town['id']:
            for d in town['doors']:
                assert tuple(d['tile']) in seen and kind(d['tile'])=='DOOR',(d['target'],'door',kind(d['tile']))
                assert tuple(d['return_tile']) in seen and kind(d['return_tile'])=='FLOOR',(d['target'],'return')
            assert {(6,24),(40,24)}<=seen,'ports disconnected'
            doors={tuple(d['tile']) for d in town['doors']}
            assert {p for p in seen if kind(p)=='DOOR'}==doors,'unpaired reachable facade'
        else:
            for e in s['exits']:assert tuple(e['tile']) in seen and kind(e['tile'])=='WARP_CARPET_DOWN',(s['id'],'carpet')
        if published:
            native=pack['data']['maps'][s['id']]
            assert native['blocks']==blocks and native['attributes']['tileset_name']==s['tileset'],s['id']+' source/native terrain differs'
            assert {a['object_identifier']: [a['x'],a['y']] for a in native['objects']}=={a['id']:a['tile'] for a in s['residents']},s['id']+' source/native actors differ'
        im=art.render(s['tileset'],w,h,blocks,'day' if s['id']==town['id'] else 'indoor');save_render(im,OUT/(s['id']+'-terrain.png'))
        scene=render_stage(im,'convert',OUT,s['id']+'-scene','RGBA')
        for a in s['residents']:scene.alpha_composite(review_duluth.oriented_sprite(pack,a['sprite'],a['facing']),(a['tile'][0]*16,a['tile'][1]*16))
        save_render(scene,OUT/(s['id']+'-scene.png'))
        if s['id']==town['id']:
            if connected:connection_context(pack,town,scene)
            for label,p in [('depot',[26,14]),('riverwalk',[31,32]),('widest-street',[22,24]),('east-checkpoint',[44,24]),('west-checkpoint',[3,24])]:
                x,y=p;crop=render_stage(scene,'crop',OUT,label,(x*16-144,y*16-112,x*16+160,y*16+128));save_render(render_stage(crop,'resize',OUT,label,(608,480)),OUT/(label+'-camera.png'))
        else:rooms.append((s['id'],scene))
        print(s['id'],len(seen),'reachable tiles; actors/doors/returns checked')
    save_render(room_sheet(rooms),OUT/'rooms.png')


if __name__=='__main__':
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--pack',default=str(ROOT/'content-packs/minnesota-northstar-canopy-review.crystalpack'))
    parser.add_argument('--verify-published',action='store_true')
    parser.add_argument('--connected',action='store_true')
    args=parser.parse_args();review(args.pack,args.verify_published,args.connected)
