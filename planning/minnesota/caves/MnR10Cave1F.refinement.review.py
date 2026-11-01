#!/usr/bin/env python3
"""R10 source-only refinement, preserving the current installed gallery epoch."""
import argparse
from collections import deque
from copy import deepcopy
import hashlib
import json
from pathlib import Path
import sys
import time
from PIL import Image
ROOT=Path(__file__).resolve().parents[3]
sys.path.insert(0,str(ROOT/'planning/minnesota/tools'))
from render_archive import save_render
from render_reference import SourceArt,load_pack,npc_palette_pack,room_sheet
from render_stages import render_stage
from review_cave_candidate import path
from review_north_shore import oriented_sprite
BASELINE=ROOT/'planning/minnesota/renders/caves/r10-refinement/baseline-e549-1793071614573053000'
PACK_SHA='e549c178f37a3137e8ca7f119aeee79460164f1dbd3ea6d205be8928415fd04c'
STAIRS=[([3,4],'12',[(7,7),(7,8),(7,9),(7,10)],(6,9)),([10,3],'36',[(20,5),(20,6),(20,7),(20,8)],(21,7)),([10,4],'36',[(20,7),(20,8),(20,9),(20,10)],(21,9)),([3,7],'12',[(7,13),(7,14),(7,15),(7,16)],(6,15))]


def native_walk_path(kind, bodies, start, target, forbidden=frozenset()):
    """Bounded native ordinary walk: UP_WALL is directional land, not solid.

    Mirrors collision.rs is_direction_blocked/blocked_leaving and land entry
    for this room's exact token roster. No hop edge is used for uphill returns.
    """
    parents={start:None};todo=deque([start])
    while todo:
        at=todo.popleft()
        if at==target:
            result=[at]
            while parents[at]is not None:at=parents[at];result.append(at)
            return list(reversed(result))
        x,y=at
        for dx,dy in [(-1,0),(1,0),(0,-1),(0,1)]:
            to=(x+dx,y+dy)
            if to in parents or to in bodies or to in forbidden or not(0<=to[0]<36 and 0<=to[1]<28):continue
            current,destination=kind(at),kind(to)
            if destination not in {'FLOOR','UP_WALL','LADDER','HOP_DOWN'}:continue
            if current=='UP_WALL'and dy==-1:continue
            if destination=='UP_WALL'and dy==1:continue
            parents[to]=at;todo.append(to)
    return None


def validate(room,original,pack):
    retained=deepcopy(room);retained['review_contract'].pop('refinement',None)
    assert {k:v for k,v in retained.items()if k!='rows'}=={k:v for k,v in original.items()if k!='rows'},'Original nonterrain contract changed'
    assert room['dimensions_metatiles']==[18,14]
    rows=[row.split()for row in room['rows']];assert len(rows)==14 and all(len(row)==18 for row in rows)
    blocks=[int(v,16)for row in rows for v in row];oldblocks=[int(v,16)for row in original['rows']for v in row.split()];collision=pack['data']['tilesets']['cave']['collision']
    def kind(at,source=blocks):
        x,y=at;return collision[f'{source[y//2*18+x//2]:02x}'][y%2*2+x%2]
    assert all(kind((x,y))in {'FLOOR','WALL','UP_WALL','LADDER','HOP_DOWN','WATER'}for y in range(28)for x in range(36)),'Unsupported new cave collision'
    bodies={tuple(a['tile'])for a in room['residents']+room['items']};floor={(x,y)for y in range(28)for x in range(36)if kind((x,y))=='FLOOR'and(x,y)not in bodies};entry=tuple(room['clear_arrival']);assert entry in floor
    unreachable=[p for p in floor if not path(entry,p,floor)];assert not unreachable,('Stranded Floor',unreachable)
    assert all(native_walk_path(kind,bodies,entry,p)for p in floor),'Native directional land disconnected'
    assert all(kind((x,y))=='WALL'for y in range(28)for x in range(36)if x in [0,35]or y in [0,27]),'Open boundary'
    assert {(x,y)for y in range(28)for x in range(36)if kind((x,y))=='LADDER'}=={tuple(v)for v in room['exits']}=={(3,13)}
    assert blocks[6*18+1]==oldblocks[6*18+1]==7,'Original full mouth block changes'
    for index,(old,new)in enumerate(zip(oldblocks,blocks)):
        if any(v=='WATER'for v in collision[f'{old:02x}']):assert old==new,'Existing spring changes'
        assert not any(v=='WATER'for v in collision[f'{new:02x}'])or any(v=='WATER'for v in collision[f'{old:02x}']),'Added water'
    approaches=[('MN_R10_GALLERY_READER',(10,5)),('MN_R10_ITEM_04',(28,23))]
    for actor,at in approaches:assert at in floor and path(entry,at,floor),actor+' inaccessible'
    for at,block,lane,side in STAIRS:
        assert blocks[at[1]*18+at[0]]==int(block,16)
        assert all(p in floor for p in lane),(at,'blocked native stair')
        assert kind(side)=='WALL'
    returns=[]
    for x,expected in [(12,13),(13,15)]:
        assert kind((x,5))=='FLOOR'and kind((x,6))=='HOP_DOWN'and kind((x,7))=='WALL'and kind((x,8))=='FLOOR'
        ordinary=path((x,8),(x,5),floor);assert ordinary and len(ordinary)-1==expected,('Original ordinary return changed',x,ordinary)
        native_return=native_walk_path(kind,bodies,(x,8),(x,5));assert native_return and len(native_return)-1==expected,('Native directional return differs',x,native_return)
        returns.append(dict(lane=x,steps=len(ordinary)-1,path_core=ordinary,native_directional_steps=len(native_return)-1,native_directional_path=native_return))
    assert kind((11,7))==kind((14,7))=='WALL'
    assert {(x,y)for y in range(28)for x in range(36)if kind((x,y)).startswith('HOP')}=={(12,6),(13,6)}
    for view in room['review_contract']['refinement']['viewpoints']:assert tuple(view['tile'])in floor,view['id']+' camera blocked'
    return blocks,floor,returns


def negative_checks(room,original,pack):
    rejected=[]
    for label in ['changed retained review contract','added top-level editorial field','moved reader','changed Revive','moved Revive','moved exit','upper west stair','upper east stair','lower east stair','lower west stair','removed lip','opened lip side','changed spring','extra water','open border']:
        bad=deepcopy(room)
        if label=='changed retained review contract':bad['review_contract']['one_way_ledges'][0]['minimum_return_steps']+=1
        elif label=='added top-level editorial field':bad['refinement_contract']={}
        elif label=='moved reader':bad['residents'][0]['tile'][0]+=1
        elif label=='changed Revive':bad['items'][0]['quantity']+=1
        elif label=='moved Revive':bad['items'][0]['tile'][0]-=1
        elif label=='moved exit':bad['exits'][0][0]+=1
        else:
            x,y,b={'upper west stair':(3,4,'01'),'upper east stair':(10,3,'01'),'lower east stair':(10,4,'01'),'lower west stair':(3,7,'01'),'removed lip':(6,3,'02'),'opened lip side':(5,3,'02'),'changed spring':(13,9,'02'),'extra water':(4,5,'03'),'open border':(0,0,'02')}[label]
            rows=[row.split()for row in bad['rows']];rows[y][x]=b;bad['rows']=[' '.join(row)for row in rows]
        try:validate(bad,original,pack)
        except AssertionError:rejected.append(label)
        else:raise AssertionError('Accepted corruption: '+label)
    return rejected


def directional_audit(room,original,pack):
    """Record real alternate walks; stairs around a loop are not claimed gates."""
    sample={(1,1):'UP_WALL',(1,0):'FLOOR',(1,2):'FLOOR',(0,1):'FLOOR',(2,1):'FLOOR'}
    sample_kind=lambda p:sample.get(p,'WALL')
    assert native_walk_path(sample_kind,set(),(1,1),(1,0))is None
    assert native_walk_path(sample_kind,set(),(1,0),(1,1))is None
    assert native_walk_path(sample_kind,set(),(1,2),(1,1))==[(1,2),(1,1)]
    assert native_walk_path(sample_kind,set(),(1,1),(1,2))==[(1,1),(1,2)]
    assert native_walk_path(sample_kind,set(),(0,1),(2,1))==[(0,1),(1,1),(2,1)]
    reports=[]
    collision=pack['data']['tilesets']['cave']['collision']
    for label,source in [('installed',original),('refinement',room)]:
        blocks=[int(v,16)for row in source['rows']for v in row.split()]
        kind=lambda at:collision[f'{blocks[at[1]//2*18+at[0]//2]:02x}'][at[1]%2*2+at[0]%2]
        bodies={tuple(a['tile'])for a in source['residents']+source['items']}
        for at,block,lane,side in STAIRS:
            alternative=native_walk_path(kind,bodies,lane[0],lane[-1],frozenset(lane[1:-1]))
            reports.append(dict(epoch=label,stair_metatile=at,closed_lane_alternative_steps=len(alternative)-1 if alternative else None,alternative_core=alternative,claim='ordinary alternate route; no sole-stair gating claim'))
    return dict(UP_WALL_directional_land_probes=5,source_semantics='collision.rs is_direction_blocked/is_direction_blocked_leaving and movement.rs occupied-tile land entry; ordinary paths exclude hop edges',rust_collision_source_sha256=hashlib.sha256((ROOT/'crates/crystal-core/src/world/collision.rs').read_bytes()).hexdigest(),stair_alternatives=reports)


def main():
    parser=argparse.ArgumentParser();parser.add_argument('--source',type=Path,default=ROOT/'planning/minnesota/caves/MnR10Cave1F.refinement.candidate.json');args=parser.parse_args();started=time.perf_counter();packpath=ROOT/'content-packs/minnesota-northstar-limestone-review.crystalpack';assert hashlib.sha256(packpath.read_bytes()).hexdigest()==PACK_SHA;pack=load_pack(packpath);previous=ROOT/'planning/minnesota/caves/MnR10Cave1F.refinement.previous.json';original=json.loads(previous.read_bytes());raw=args.source.read_bytes();room=json.loads(raw);digest=hashlib.sha256(raw).hexdigest();out=ROOT/'planning/minnesota/renders/caves/r10-refinement'/digest[:16];out.mkdir(parents=True,exist_ok=True);(out/'MnR10Cave1F.candidate.json').write_bytes(raw);(out/'MnR10Cave1F.previous.json').write_bytes(previous.read_bytes());native=pack['data']['maps'][room['id']];assert native['blocks']==[int(v,16)for row in original['rows']for v in row.split()]
    blocks,floor,returns=validate(room,original,pack);rejected=negative_checks(room,original,pack);directional=directional_audit(room,original,pack);assert native['attributes']['time_of_day']=='nite';terrain=SourceArt(pack).render('cave',18,14,blocks,group='nite');save_render(terrain,out/'terrain-original.png');whole=render_stage(terrain,'convert',out,'whole-rgba','RGBA');sp=npc_palette_pack(pack,'nite')
    for actor in room['residents']+room['items']:
        sprite=oriented_sprite(sp,actor.get('sprite','SPRITE_POKE_BALL'),actor.get('facing','south'));save_render(sprite,out/(actor['id']+'-native-nite-rgba.png'));whole.alpha_composite(sprite,(actor['tile'][0]*16,actor['tile'][1]*16))
    save_render(whole,out/'whole-source-rgba.png');whole=render_stage(whole,'convert',out,'whole-rgb','RGB');save_render(whole,out/'whole-source.png');save_render(render_stage(whole,'resize',out,'whole2x',(1152,896),Image.Resampling.NEAREST),out/'whole-2x.png');save_render(room_sheet([('CURRENT INSTALLED18x14',Image.open(BASELINE/'scene-rgb.png')),('SOURCE REFINEMENT18x14',whole)],columns=2),out/'whole-before-after.png')
    sizes={};worst=(0,0,0)
    for y in range(28):
        for x in range(36):
            n=1+min(sizes.get((x-1,y),0),sizes.get((x,y-1),0),sizes.get((x-1,y-1),0))if(x,y)in floor else 0;sizes[x,y]=n
            if n>worst[0]:worst=(n,x-n+1,y-n+1)
    n,x,y=worst;weakest=[x+n//2,y+n//2];views=room['review_contract']['refinement']['viewpoints']+[dict(id='ACTUAL_WEAKEST_FLOOR',tile=weakest)];player=oriented_sprite(sp,'SPRITE_CHRIS','south');cameras=[]
    for view in views:
        name=view['id'].lower();x,y=view['tile'];assert(x,y)in floor;left=max(0,min(x*16-80,whole.width-160));top=max(0,min(y*16-72,whole.height-144));crop=render_stage(whole,'crop',out,name,(left,top,left+160,top+144));save_render(crop,out/(name+'-terrain160.png'));rgba=render_stage(crop,'convert',out,name+'-player-rgba','RGBA');rgba.alpha_composite(player,(x*16-left,y*16-top));save_render(rgba,out/(name+'-player-rgba.png'));rgb=render_stage(rgba,'convert',out,name+'-rgb','RGB');save_render(rgb,out/(name+'-player160.png'));save_render(render_stage(rgb,'resize',out,name+'-4x',(640,576),Image.Resampling.NEAREST),out/(name+'-4x.png'));cameras.append((view['id']+' '+str(view['tile']),rgb))
    save_render(room_sheet(cameras,columns=3),out/'decision-cameras.png');report=dict(status='SOURCE ONLY; current18x14 baseline, not historical12x8',pack_sha256=PACK_SHA,candidate_sha256=digest,previous_sha256=hashlib.sha256(previous.read_bytes()).hexdigest(),all_original_nonterrain_values_except_review_metadata_exact=True,all_original_review_contract_subfields_exact=True,native_original_all252_blocks_match=True,all_floor_connected_actors_present=True,all_spring_blocks_unchanged=True,native_four_stairs_and_both13_15_step_returns_preserved=returns,widest_empty_floor_square_core=n,weakest_core=weakest,negative_cases_rejected=rejected,native_directional_audit=directional,total_review_seconds=round(time.perf_counter()-started,3));(out/'source-review.json').write_text(json.dumps(report,indent=2)+'\n');(out/'review.py.snapshot').write_bytes(Path(__file__).read_bytes());print(json.dumps(dict(output=str(out),**report),indent=2))


if __name__=='__main__':main()
