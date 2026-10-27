#!/usr/bin/env python3
"""Scoped source-only R01 review using native art and existing path/archive tools."""
import argparse
from copy import deepcopy
import hashlib
import json
from pathlib import Path
import sys
import time

from PIL import Image

ROOT = Path(__file__).resolve().parents[3]
sys.path.insert(0, str(ROOT / 'planning/minnesota/tools'))
from render_archive import save_render
from render_reference import SourceArt, load_pack, npc_palette_pack, room_sheet
from render_stages import render_stage
from review_cave_candidate import path
from review_north_shore import oriented_sprite

BASELINE = ROOT / 'planning/minnesota/renders/caves/r01-candidate/baseline-1630-1793070873062798000'
PACK_SHA = '1630ad063aa4a1cf252087bdf743ca48a2a8cb6c9c46ad95cb7aceb1c74c3b05'


def geometry(room, original, pack):
    assert {k:v for k,v in room.items() if k not in {'rows','review_contract'}} == {k:v for k,v in original.items() if k != 'rows'}, 'Original gameplay field changed'
    assert room['dimensions_metatiles'] == [8, 7]
    rows = [row.split() for row in room['rows']]
    assert len(rows) == 7 and all(len(row) == 8 for row in rows)
    blocks = [int(v, 16) for row in rows for v in row]
    collision = pack['data']['tilesets']['cave']['collision']
    def kind(at):
        x,y = at
        return collision[f'{blocks[y//2*8+x//2]:02x}'][y%2*2+x%2]
    actors = room['residents'] + room['items']
    bodies = {tuple(v['tile']) for v in actors}
    floor = {(x,y) for y in range(14) for x in range(16) if kind((x,y)) == 'FLOOR' and (x,y) not in bodies}
    entry = tuple(room['clear_arrival'])
    assert entry in floor
    assert all(path(entry, p, floor) for p in floor), 'Stranded ordinary Floor'
    assert all(kind((x,y)) == 'WALL' for y in range(14) for x in range(16) if x in [0,15] or y in [0,13]), 'Open perimeter'
    assert {(x,y) for y in range(14) for x in range(16) if kind((x,y)) == 'LADDER'} == {tuple(v) for v in room['exits']} == {(9,11)}
    assert all(kind((x,y)) in {'WALL','UP_WALL','FLOOR','LADDER'} for y in range(14) for x in range(16)), 'No new unsupported collision'
    interactions = []
    for actor in actors:
        x,y = actor['tile']
        assert kind((x,y)) == 'FLOOR'
        approaches = [q for q in [(x-1,y),(x+1,y),(x,y-1),(x,y+1)] if q in floor and path(entry,q,floor)]
        assert approaches, actor['id']
        interactions.append(dict(id=actor['id'],approaches=approaches,shortest_steps=min(len(path(entry,q,floor))-1 for q in approaches)))
    stairs = []
    for probe in room['review_contract']['controller_probes']['stairs']:
        x,y = probe['metatile']
        points = [probe['before']] + probe['lane'] + [probe['after']]
        assert blocks[y*8+x] == int(probe['block'],16)
        assert all(tuple(p) in floor for p in points), probe['id'] + ' blocked stair lane'
        assert kind(probe['blocked_target']) == 'WALL'
        closed = floor - {tuple(p) for p in probe['lane']}
        assert path(entry,tuple(probe['destination']),closed) is None, probe['id'] + ' bypassed climb'
        stairs.append(dict(id=probe['id'],points=points,closed_stair_disconnects_destination=True))
    assert all(tuple(view['tile']) in floor for view in room['review_contract']['viewpoints']), 'Camera on blocked or occupied tile'
    return blocks, floor, interactions, stairs


def negatives(room, original, pack):
    cases = ['moved guide','changed Antidote','changed flag','moved exit','blocked guide stair','blocked Antidote stair','guide stair bypass','Antidote stair bypass','fake ladder','removed ladder','open border','unintended water','blocked guide','added actor','camera on item']
    for label in cases:
        value = deepcopy(room)
        if label == 'moved guide': value['residents'][0]['tile'][0] += 1
        elif label == 'changed Antidote': value['items'][0]['item'] = 'POTION'
        elif label == 'changed flag': value['items'][0]['flag'] = 'EVENT_WRONG'
        elif label == 'moved exit': value['exits'][0][0] -= 1
        elif label == 'added actor': value['residents'].append(deepcopy(value['residents'][0]))
        elif label == 'camera on item': value['review_contract']['viewpoints'][0]['tile']=[11,5]
        else:
            x,y,b = {'blocked guide stair':(2,4,'01'),'blocked Antidote stair':(5,3,'01'),'guide stair bypass':(3,4,'02'),'Antidote stair bypass':(6,3,'0a'),'fake ladder':(3,4,'07'),'removed ladder':(4,5,'21'),'open border':(0,0,'02'),'unintended water':(4,4,'03'),'blocked guide':(2,2,'01')}[label]
            rows=[row.split() for row in value['rows']];rows[y][x]=b;value['rows']=[' '.join(row) for row in rows]
        try: geometry(value,original,pack)
        except AssertionError: pass
        else: raise AssertionError('Accepted corruption: ' + label)
    return cases


def main():
    parser=argparse.ArgumentParser()
    parser.add_argument('--source',type=Path,default=ROOT/'planning/minnesota/caves/MnR01Cave1F.candidate.json')
    parser.add_argument('--self-test',action='store_true')
    args=parser.parse_args();started=time.perf_counter()
    packpath=ROOT/'content-packs/minnesota-northstar-glacial-cleft-review.crystalpack'
    assert hashlib.sha256(packpath.read_bytes()).hexdigest()==PACK_SHA
    pack=load_pack(packpath);raw=args.source.read_bytes();room=json.loads(raw)
    previous=ROOT/'planning/minnesota/caves/MnR01Cave1F.previous.json';original=json.loads(previous.read_bytes())
    native=pack['data']['maps']['MnR01Cave1F']
    assert native['blocks']==[int(v,16) for row in original['rows'] for v in row.split()]
    source_actors=room['residents']+room['items']
    assert sorted((v['id'],*v['tile'],v.get('sprite','SPRITE_POKE_BALL')) for v in source_actors)==sorted((v['object_identifier'],v['x'],v['y'],v['sprite']) for v in native['objects'])
    assert native['script_field_pickups']==[dict(command='itemball',item_id='ANTIDOTE',quantity=1,event_flag='EVENT_MN_R01_ITEM_03',fruit_tree_id=None,source_script='MN_R01_ITEM_03',command_index=0)]
    expected_warp=dict(index=1,x=9,y=11,target_map_constant='MnR01C00R00',target_map='MnR01C00R00',target_warp_id=2)
    assert native['events']['warps']==[expected_warp]
    exterior=pack['data']['maps']['MnR01C00R00']
    exterior_pair=[v for v in exterior['events']['warps'] if v['target_map']=='MnR01Cave1F']
    assert exterior_pair==[dict(index=index,x=18,y=y,target_map_constant='MnR01Cave1F',target_map='MnR01Cave1F',target_warp_id=1) for index,y in [(1,17),(2,18)]]
    exterior_collision=pack['data']['tilesets'][exterior['attributes']['tileset_name']]['collision']
    for y,kind in [(17,'CAVE'),(18,'FLOOR')]:
        block=exterior['blocks'][y//2*exterior['attributes']['width']+9]
        assert exterior_collision[f'{block:02x}'][y%2*2]==kind
    blocks,floor,interactions,stairs=geometry(room,original,pack)
    rejected=negatives(room,original,pack) if args.self_test else []
    digest=hashlib.sha256(raw).hexdigest();out=ROOT/'planning/minnesota/renders/caves/r01-candidate'/digest[:16];out.mkdir(parents=True,exist_ok=True)
    (out/'MnR01Cave1F.candidate.json').write_bytes(raw);(out/'MnR01Cave1F.previous.json').write_bytes(previous.read_bytes())
    assert native['attributes']['time_of_day']==room['review_contract']['palette_group']=='nite'
    terrain=SourceArt(pack).render('cave',8,7,blocks,group='nite');save_render(terrain,out/'terrain-original.png')
    spritepack=npc_palette_pack(pack,'nite');whole=render_stage(terrain,'convert',out,'whole-rgba','RGBA')
    for actor in room['residents']+room['items']:
        sprite=oriented_sprite(spritepack,actor.get('sprite','SPRITE_POKE_BALL'),actor.get('facing','south'))
        save_render(sprite,out/(actor['id']+'-native-nite-rgba.png'));whole.alpha_composite(sprite,(actor['tile'][0]*16,actor['tile'][1]*16))
    save_render(whole,out/'whole-source-rgba.png');whole=render_stage(whole,'convert',out,'whole-rgb','RGB');save_render(whole,out/'whole-source.png');save_render(render_stage(whole,'resize',out,'whole2x',(512,448),Image.Resampling.NEAREST),out/'whole-2x.png')
    sizes={};worst=(0,0,0)
    for y in range(14):
        for x in range(16):
            n=1+min(sizes.get((x-1,y),0),sizes.get((x,y-1),0),sizes.get((x-1,y-1),0)) if (x,y) in floor else 0;sizes[x,y]=n
            if n>worst[0]:worst=(n,x-n+1,y-n+1)
    n,x,y=worst;weakest=[x+n//2,y+n//2]
    cameras=[];player=oriented_sprite(spritepack,'SPRITE_CHRIS','south')
    for view in room['review_contract']['viewpoints']+[dict(id='ACTUAL_WEAKEST_FLOOR',tile=weakest)]:
        name=view['id'].lower();x,y=view['tile'];assert (x,y) in floor
        left=max(0,min(x*16-80,whole.width-160));top=max(0,min(y*16-72,whole.height-144))
        crop=render_stage(whole,'crop',out,name,(left,top,left+160,top+144));save_render(crop,out/(name+'-terrain160.png'));rgba=render_stage(crop,'convert',out,name+'-player-rgba','RGBA');rgba.alpha_composite(player,(x*16-left,y*16-top));save_render(rgba,out/(name+'-player-rgba.png'));rgb=render_stage(rgba,'convert',out,name+'-rgb','RGB');save_render(rgb,out/(name+'-player160.png'));save_render(render_stage(rgb,'resize',out,name+'-4x',(640,576),Image.Resampling.NEAREST),out/(name+'-4x.png'));cameras.append((view['id']+' '+str(view['tile']),rgb))
    save_render(room_sheet(cameras,columns=3),out/'decision-cameras.png')
    save_render(room_sheet([('Original8x7',Image.open(BASELINE/'scene-rgb.png')),('Source candidate8x7',whole)],columns=2),out/'whole-before-after.png')
    report=dict(status='SOURCE ONLY; no native publication or controller proof',pack_sha256=PACK_SHA,candidate_sha256=digest,previous_sha256=hashlib.sha256(previous.read_bytes()).hexdigest(),dimensions_metatiles=[8,7],all_original_nonrow_fields_exact=True,native_original_all56_blocks_equal=True,forced_palette='nite',native_sprite_tile_top_left_no_minus4=True,all_floor_reachable_with_actors_present=True,interactions=interactions,stairs=stairs,widest_empty_floor_square_core=n,weakest_core=weakest,negative_cases_rejected=rejected,total_review_seconds=round(time.perf_counter()-started,3))
    report.update(native_original_actor_roster_exact=True,native_original_Antidote_program_exact=True,native_existing_inside_pair=expected_warp,native_existing_exterior_pair=exterior_pair,native_existing_exterior_CAVE_and_passive_FLOOR=True)
    (out/'source-review.json').write_text(json.dumps(report,indent=2)+'\n');(out/'review.py.snapshot').write_bytes(Path(__file__).read_bytes());print(json.dumps(dict(output=str(out),**report),indent=2))


if __name__=='__main__':main()
