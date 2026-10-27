#!/usr/bin/env python3
"""Fast, read-only R17 native-art loop with actual directional wall masks."""
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
from render_reference import load_pack,SourceArt,npc_palette_pack,room_sheet
from review_north_shore import oriented_sprite
from render_archive import save_render
from render_stages import render_stage


def inspect(pack,room,output):
    started=time.perf_counter();output.mkdir(parents=True,exist_ok=True)
    encoded=(json.dumps(room,indent=2)+'\n').encode()
    snapshot=output/('source-'+hashlib.sha256(encoded).hexdigest()+'.json')
    assert not snapshot.exists()or snapshot.read_bytes()==encoded,'source snapshot collision'
    snapshot.write_bytes(encoded)
    original=json.loads((ROOT/'planning/minnesota/caves/MnR17Cave1F.previous.json').read_text())
    allowed={'rows','purpose','candidate_status','review_contract'}
    assert all(original.get(k)==room.get(k)for k in set(original)|set(room)if k not in allowed),'original whole actor/item/mouth/source fields changed'
    native=pack['data']['maps'][room['id']]
    assert native['blocks']==[int(v,16)for r in original['rows']for v in r.split()],'native preimage differs from frozen previous source'
    assert native['script_field_pickups']==[dict(command='itemball',item_id='REVIVE',quantity=1,event_flag='EVENT_MN_R17_ITEM_04',fruit_tree_id=None,source_script='MN_R17_ITEM_04',command_index=0)]
    w,h=room['dimensions_metatiles'];assert(w,h)==(7,8)
    rows=[r.split()for r in room['rows']];assert len(rows)==h and all(len(r)==w for r in rows)
    blocks=[int(v,16)for r in rows for v in r];col=pack['data']['tilesets']['cave']['collision']
    kind=lambda p:col[f'{blocks[p[1]//2*w+p[0]//2]:02x}'][p[1]%2*2+p[0]%2]
    actors=room.get('residents',[])+room.get('items',[]);bodies={tuple(a['tile'])for a in actors}
    assert len(bodies)==len(actors)
    entry=tuple(room['clear_arrival']);assert kind(entry)=='FLOOR'and entry not in bodies
    exits={tuple(p)for p in room['exits']};assert exits=={(x,y)for y in range(h*2)for x in range(w*2)if kind((x,y))=='LADDER'}
    assert all(kind((x,y))=='WALL'for y in range(h*2)for x in range(w*2)if x in(0,w*2-1)or y in(0,h*2-1)), 'closed original perimeter required'
    standable={'FLOOR','UP_WALL'}
    def reachable(origin):
        todo=deque([origin]);seen={origin};parents={origin:None}
        while todo:
            x,y=todo.popleft()
            for target,direction in [((x-1,y),'left'),((x+1,y),'right'),((x,y-1),'up'),((x,y+1),'down')]:
                nx,ny=target
                if not(0<=nx<w*2 and 0<=ny<h*2)or target in bodies or target in seen:continue
                src,dest=kind((x,y)),kind(target)
                if dest not in standable:continue
                # Crystal entry masks: UP_WALL blocks entering Down. Departure
                # masks independently block leaving Up. It is not a solid tile.
                if(src=='UP_WALL'and direction=='up')or(dest=='UP_WALL'and direction=='down'):continue
                seen.add(target);parents[target]=(x,y);todo.append(target)
        return seen,parents
    seen,parents=reachable(entry)
    floors={(x,y)for y in range(h*2)for x in range(w*2)if kind((x,y))=='FLOOR'and(x,y)not in bodies}
    assert floors<=seen,('unreachable actual Floor',sorted(floors-seen))
    assert all(entry in reachable(p)[0]for p in seen),'directional wall creates a one-way trap'
    interactions=[]
    for actor in actors:
        x,y=actor['tile'];assert kind((x,y))=='FLOOR'
        options=[p for p in [(x-1,y),(x+1,y),(x,y-1),(x,y+1)]if p in seen]
        assert options,actor['id']+': no reachable interaction'
        distances=[]
        for p in options:
            route=[];at=p
            while at is not None:route.append(at);at=parents[at]
            distances.append((len(route)-1,p))
        distance,approach=min(distances);interactions.append(dict(id=actor['id'],approach=list(approach),steps=distance))
    for x,y in exits:assert any(p in seen for p in [(x-1,y),(x+1,y),(x,y-1),(x,y+1)])
    for stair in room.get('review_contract',{}).get('stairs',[]):
        core=[tuple(p)for p in stair['walk_core']];assert all(kind(p)=='FLOOR'and p in seen for p in core)
        assert all(abs(a[0]-b[0])+abs(a[1]-b[1])==1 for a,b in zip(core,core[1:]))
        for a,b in zip(core,core[1:]):assert b in reachable(a)[0]and a in reachable(b)[0]
        for p in stair.get('wall_side_core',[]):assert kind(tuple(p))=='WALL'
    group=native['attributes']['time_of_day'];assert group=='nite'
    assert room.get('review_contract',{}).get('palette_group',group)==group,'declared palette drift'
    terrain=SourceArt(pack).render('cave',w,h,blocks,group=group);save_render(terrain,output/'terrain-original.png')
    spritepack=npc_palette_pack(pack,group);whole=terrain.convert('RGBA')
    for actor in actors:
        frame=oriented_sprite(spritepack,actor.get('sprite','SPRITE_POKE_BALL'),actor.get('facing','south'))
        save_render(frame,output/(actor['id']+'-native-nite-rgba.png'))
        whole.alpha_composite(frame,(actor['tile'][0]*16,actor['tile'][1]*16))
    save_render(whole,output/'whole-source-rgba.png');rgb=render_stage(whole,'convert',output,'whole-rgb','RGB');save_render(rgb,output/'whole-original224x256.png')
    large=render_stage(rgb,'resize',output,'whole-4x',(w*128,h*128),Image.Resampling.NEAREST);save_render(large,output/'whole-4x.png')
    views=room.get('review_contract',{}).get('viewpoints',[{'id':'mouth','tile':room['clear_arrival']},{'id':'original-reward','tile':[5,6]},{'id':'original-wide','tile':[8,8]}]);cameras=[]
    for view in views:
        x,y=view['tile'];assert(x,y)in seen
        # Source review frame only: centered160×144, retained without implying
        # the production camera's map-edge clamping has already been verified.
        box=(x*16-72,y*16-64,x*16+88,y*16+80)
        player=oriented_sprite(spritepack,'SPRITE_CHRIS','south')
        frame=whole.copy();frame.alpha_composite(player,(x*16,y*16))
        save_render(frame,output/(view['id']+'-source-with-player-rgba.png'))
        source_frame=render_stage(frame,'convert',output,view['id']+'-rgb','RGB')
        crop=render_stage(source_frame,'crop',output,view['id']+'-camera',box);save_render(crop,output/(view['id']+'-original160.png'))
        save_render(render_stage(crop,'resize',output,view['id']+'-4x',(640,576),Image.Resampling.NEAREST),output/(view['id']+'-4x.png'));cameras.append((view['id'],crop))
    save_render(room_sheet(cameras,columns=3),output/'decision-cameras.png')
    widest=(0,None)
    for y in range(h*2):
        start=None
        for x in range(w*2+1):
            if(x,y)in floors:
                if start is None:start=x
            elif start is not None:
                if x-start>widest[0]:widest=(x-start,[start,y,x-1,y])
                start=None
    report=dict(widest_horizontal_free_floor_run=widest[0],widest_run_core=widest[1],status='SOURCE ONLY; actor-present directed collision/static native-art review, not native input acceptance',dimensions_metatiles=[w,h],forced_palette=group,reachable_floor=len(floors),unreachable_floor=0,reachable_up_wall=sorted([list(p)for p in seen if kind(p)=='UP_WALL']),directional_traps=0,interactions=interactions,unpaired_ladders=0,open_boundary=0,review_seconds=round(time.perf_counter()-started,3),native_pack_sha256=hashlib.sha256(Path(args.pack).read_bytes()).hexdigest())
    (output/'source-review.json').write_text(json.dumps(report,indent=2)+'\n');return report


if __name__=='__main__':
    parser=argparse.ArgumentParser();parser.add_argument('--pack',type=Path,required=True);parser.add_argument('--room',type=Path,required=True);parser.add_argument('--output',type=Path,required=True);parser.add_argument('--self-test',action='store_true');args=parser.parse_args()
    cli_started=time.perf_counter();pack=load_pack(args.pack);room=json.loads(args.room.read_text());report=inspect(pack,room,args.output)
    if args.self_test:
        mutations=[('reward-identity',lambda v:v['items'][0].update(item='MASTER_BALL')),
            ('reward-flag',lambda v:v['items'][0].update(flag='EVENT_MN_DEPOT_CLEARED')),
            ('reward-quantity',lambda v:v['items'][0].update(quantity=99)),
            ('reward-anchor',lambda v:v['items'][0].update(tile=[6,5])),
            ('native-mouth',lambda v:v.update(exits=[[8,13]])),
            ('palette',lambda v:v['review_contract'].update(palette_group='day'))]
        for name,at,block in [('neck-severed',[2,4],'01'),('bend-severed',[3,5],'01'),
            ('actor-floor',[2,2],'1d'),('open-edge',[0,0],'02'),
            ('unpaired-ladder',[2,5],'1b'),('roof-mask-not-ordinary-floor',[2,0],'18')]:
            def change(v,at=at,block=block):
                row=v['rows'][at[1]].split();row[at[0]]=block;v['rows'][at[1]]=' '.join(row)
            mutations.append((name,change))
        negatives=[]
        for name,mutate in mutations:
            invalid=deepcopy(room);mutate(invalid);folder=args.output/'negative-proof'/name;folder.mkdir(parents=True,exist_ok=True)
            (folder/'MnR17Cave1F.candidate.json').write_text(json.dumps(invalid,indent=2)+'\n')
            try:inspect(pack,invalid,folder)
            except AssertionError as error:
                negatives.append(dict(case=name,rejected=True,reason=str(error)))
            else:raise AssertionError('accepted negative '+name)
        report['negative_checks']=negatives
        (args.output/'source-proof.json').write_text(json.dumps(report,indent=2)+'\n')
    report['cli_seconds_including_pack_read_and_negatives']=round(time.perf_counter()-cli_started,3)
    (args.output/'source-proof.json').write_text(json.dumps(report,indent=2)+'\n')
    print(json.dumps(report))
