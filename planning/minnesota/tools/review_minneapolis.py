#!/usr/bin/env python3
"""Read-only whole-city and ICE holding-room review using native source art."""
import argparse
from collections import deque
import json
from pathlib import Path
from PIL import Image
from render_reference import SourceArt, layout_blocks, load_pack, room_sheet
from render_archive import save_render
from render_stages import render_stage
from review_north_shore import oriented_sprite
ROOT=Path(__file__).resolve().parents[3]
OUT=ROOT/'planning/minnesota/renders/minneapolis'

def review(path):
    pack=load_pack(path);art=SourceArt(pack)
    town=json.loads((ROOT/'planning/minnesota/towns/MnMinneapolis.json').read_text())
    layout=json.loads((ROOT/'planning/minnesota/towns/MnMinneapolis.layout.json').read_text())
    specs=[dict(id=town['id'],tileset='johto_modern',dimensions_metatiles=[32,24],blocks=layout_blocks(layout),residents=town['residents'],start=[32,40])]
    for room in town['interiors']:
        blocks=pack['data']['maps'][room['id']]['blocks'] if room['template']=='installed_native' else [int(v,16) for row in room['rows'] for v in row.split()]
        specs.append(dict(room,blocks=blocks,start=room['passive_arrival']['tile']))
    rooms=[]
    for spec in specs:
        w,h=spec['dimensions_metatiles'];blocks=spec['blocks'];collision=pack['data']['tilesets'][spec['tileset']]['collision']
        def kind(at):x,y=at;return collision[f'{blocks[y//2*w+x//2]:02x}'][y%2*2+x%2]
        bodies={tuple(a['tile']) for a in spec['residents']}
        def flood(start):
            todo=deque([start]);seen={start}
            while todo:
                x,y=todo.popleft()
                for q in [(x-1,y),(x+1,y),(x,y-1),(x,y+1)]:
                    if q in seen or q in bodies or not(0<=q[0]<w*2 and 0<=q[1]<h*2):continue
                    if kind(q)not in {'FLOOR','LADDER','DOOR','WARP_CARPET_DOWN'}:continue
                    seen.add(q)
                    if kind(q) not in {'DOOR','WARP_CARPET_DOWN'}:todo.append(q)
            return seen
        seen=flood(tuple(spec['start']));assert kind(spec['start'])=='FLOOR',(spec['id'],'arrival')
        if spec['id']==town['id']:
            for door in town['doors']:
                assert kind(door['tile'])=='DOOR' and tuple(door['tile'])in seen,(door['target'],'door')
                assert kind(door['return_tile'])=='FLOOR' and tuple(door['return_tile'])in seen,(door['target'],'return')
            for port in town['ports']:assert tuple(v*2 for v in port['landing_metatile'])in seen,'port disconnected'
            assert {p for p in seen if kind(p)=='DOOR'}=={tuple(d['tile'])for d in town['doors']},'unpaired facade'
        elif spec['template']!='installed_native':
            for exit in spec['exits']:assert tuple(exit['tile'])in seen and kind(exit['tile'])=='WARP_CARPET_DOWN',(spec['id'],'carpet')
        for actor in spec['residents']:
            x,y=actor['tile'];approaches={(x-1,y),(x+1,y),(x,y-1),(x,y+1)}
            if actor.get('counter_tile'):approaches.add(tuple(actor['approach']))
            assert kind((x,y))=='FLOOR' and approaches&seen,(spec['id'],actor['id'],'approach')
        actors=[a for a in spec['residents'] if a['id']!='MN_MP_RESCUED_DISPATCHER']
        if spec['id']==town['ice_scene']['room']:
            scene=town['ice_scene'];assert tuple(scene['enforcer']['approach'])in seen and kind(scene['enforcer']['tile'])=='FLOOR','enforcer approach';captive=tuple(scene['captive']['tile']);assert captive not in seen,'captive may not walk out of a closed cell'
            assert tuple(scene['release']['approach'])in seen and tuple(scene['note']['approach'])in seen,'latch/note approach'
            gx,gy=scene['release']['gate_metatile'];blocks[gy*w+gx]=int(scene['release']['open_block'],16)
            assert captive in flood(tuple(spec['start'])),'released cell must physically open'
            p=captive
            for step in scene['captive_departure_steps']:
                if step=='step_end':break
                p=(p[0]-1,p[1])if step=='step_left'else(p[0],p[1]+1)
                assert kind(p)in {'FLOOR','WARP_CARPET_DOWN'} and p!=tuple(scene['release']['approach']) and p!=tuple(scene['enforcer']['tile']),'captive departure must avoid walls, player and guard'
            assert p==(11,15),'captive must actually reach the carpet'

            blocks[gy*w+gx]=int(scene['release']['closed_block'],16)
            actors += [dict(scene['enforcer'],facing='south'),dict(scene['captive'],facing='west')]
        im=art.render(spec['tileset'],w,h,blocks,'day' if spec['id']==town['id']else'indoor')
        save_render(im,OUT/(spec['id']+'-terrain.png'))
        im=render_stage(im,'convert',OUT,spec['id']+'-scene','RGBA')
        if spec.get('template')=='installed_native':
            for a in pack['data']['maps'][spec['id']]['objects']:
                move=a['spritemovedata'];facing='north'if move.endswith('_UP')else'west'if move.endswith('_LEFT')else'east'if move.endswith('_RIGHT')else'south'
                im.alpha_composite(oriented_sprite(pack,a['sprite'],facing),(a['x']*16,a['y']*16))
        for a in actors:im.alpha_composite(oriented_sprite(pack,a['sprite'],a['facing']),(a['tile'][0]*16,a['tile'][1]*16))
        save_render(im,OUT/(spec['id']+'-scene.png'));rooms.append((spec['id'],im))
    save_render(room_sheet(rooms[1:],columns=2),OUT/'all-interiors.png')
    print('Whole city, five paired entrances, all ordinary interactions, and closed/open cell reachability pass. Native publication remains separate.')
if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('--pack',default=str(ROOT/'content-packs/minnesota-minneapolis-gym-remake.crystalpack'));a=p.parse_args();review(a.pack)
