#!/usr/bin/env python3
"""Read-only source review for one hand-authored 96x72 Duluth candidate."""
import argparse
from collections import deque
from copy import deepcopy
import hashlib
import json
from pathlib import Path
import time
from PIL import Image
from render_archive import save_render
from render_progress_atlas import scene
from render_reference import SourceArt, layout_blocks, load_pack, room_sheet
from review_north_shore import oriented_sprite


def flood(start, floor):
    seen={start};todo=deque([start])
    while todo:
        x,y=todo.popleft()
        for q in [(x-1,y),(x+1,y),(x,y-1),(x,y+1)]:
            if q in floor and q not in seen:seen.add(q);todo.append(q)
    return seen


def review(pack, directory, output):
    started=time.perf_counter();output.mkdir(parents=True,exist_ok=True)
    design=json.loads((directory/'MnDuluth.candidate.json').read_text());layout=json.loads((directory/'MnDuluth.layout.candidate.json').read_text())
    assert design['dimensions_metatiles']==layout['dimensions_metatiles'] and layout['dimensions_metatiles'] in [[96,72],[64,48]]
    w,h=layout['dimensions_metatiles'];cw,ch=w*2,h*2;offset=design['expansion_contract']['old_core_offset_metatiles'];entry=(101,60) if w==96 else (69,36)
    assert layout['tileset']=='johto_modern' and layout['compiled_tileset']=='mn_duluth_expanded'
    assert [(r['id'],r['source_tileset'],r['source_block']) for r in layout['derived_blocks']]==[('80','cave','12'),('81','cave','30')]+[(f'{i:02x}','kanto',v) for i,v in enumerate(['20','54','21','37','7d','3a','7e','68','7f','69'],0x82)]
    assert all(r['collision']==pack['data']['tilesets'][r['source_tileset']]['collision'][r['source_block']] for r in layout['derived_blocks'])
    for layer in layout['layers']:
        if layer['kind'] in ['cliff_polygon','polygon']:
            assert all(a[0]==b[0] or a[1]==b[1] for a,b in zip(layer['points'],layer['points'][1:]+layer['points'][:1])),layer['name']+' diagonal cliff edge'
    blocks=layout_blocks(layout);c=deepcopy(pack['data']['tilesets']['johto_modern']['collision'])
    for recipe in layout['derived_blocks']:c[recipe['id']]=pack['data']['tilesets'][recipe['source_tileset']]['collision'][recipe['source_block']]
    kind=lambda p:c[f'{blocks[p[1]//2*w+p[0]//2]:02x}'][p[1]%2*2+p[0]%2]
    actors=design['residents']+design['expansion_contract']['checkpoint_officers']+design['expansion_contract']['new_items'];bodies={tuple(a['tile']) for a in actors}
    ground={(x,y) for y in range(ch) for x in range(cw) if kind((x,y))=='FLOOR' and (x,y) not in bodies};seen=flood(entry,ground)
    errors=[];doorchecks=[]
    def adjacent(p):x,y=p;return [(x-1,y),(x+1,y),(x,y-1),(x,y+1)]
    for door in design['doors']:
        at,ret=tuple(door['tile']),tuple(door['return_tile']);ok=kind(at)=='DOOR' and ret in seen
        if not ok:errors.append(dict(type='door',id=door['id'],at=at,kind=kind(at),return_kind=kind(ret),return_reachable=ret in seen))
        doorchecks.append(dict(id=door['id'],tile=at,return_core=ret,okay=ok))
    for actor in actors:
        at=tuple(actor['tile'])
        if kind(at)!='FLOOR' or not any(q in seen for q in adjacent(at)):errors.append(dict(type='actor',id=actor['id'],at=at,kind=kind(at),approach_reachable=any(q in seen for q in adjacent(at))))
    border={(x,y) for y in range(ch) for x in range(cw) if x in [0,cw-1] or y in [0,ch-1]};legal={tuple(p) for p in design['expansion_contract'].get('connection_lanes',[[0,72],[0,73],[96,0],[97,0]])}
    actual={p for p in border if kind(p) in {'FLOOR','WATER','TALL_GRASS','UP_WALL'}}
    if actual!=legal:errors.append(dict(type='border',extra=sorted(actual-legal),missing=sorted(legal-actual)))
    cut=[(x,y) for y in range(ch) for x in range(cw) if kind((x,y))=='CUT_TREE']
    if cut:errors.append(dict(type='incidental_cut_tree',at=cut))
    stairchecks=[]
    for stair in design['expansion_contract']['stairs_metatiles']:
        x,y=stair['at'];lane_x=2*x+(stair['block']=='80');lane=[(lane_x,2*y-1),(lane_x,2*y),(lane_x,2*y+1),(lane_x,2*y+2)];badside=(2*x+(stair['block']=='81'),2*y+1);okay=all(q in seen for q in lane) and kind(badside)=='WALL'
        if not okay:errors.append(dict(type='stairs',id=stair['id'],lane=[dict(at=q,kind=kind(q),reachable=q in seen) for q in lane],side=badside,side_kind=kind(badside)))
        stairchecks.append(dict(id=stair['id'],lane=lane,blocked_side=badside,okay=okay))
    # All original native blocks are compared; intentional internalized borders are enumerated.
    old=pack['data']['maps']['MnDuluth'];oldchanges=[]
    for y in range(24):
        for x in range(32):
            a=old['blocks'][y*32+x];b=blocks[(y+offset[1])*w+x+offset[0]]
            if a!=b:oldchanges.append(dict(old_meta=[x,y],new_meta=[x+offset[0],y+offset[1]],before=f'{a:02x}',after=f'{b:02x}'))
    art=SourceArt(pack);whole=scene(art,pack,'johto_modern',w,h,blocks,actors,derived=layout['derived_blocks'],archive=output/'whole-stages',native_origin=True);save_render(whole,output/'whole-city.png');save_render(whole.resize((w*16,h*16),Image.Resampling.NEAREST),output/'whole-city-half.png')
    views=design['expansion_contract'].get('viewpoints',[('ORIGINAL_GYM',[80,60]),('OBSERVATION_STAIRS',[17,24]),('UPPER_HILL_STREET',[37,18]),('HILLSIDE_FORK',[96,20]),('CIVIC_STAIRS',[75,34]),('WAREHOUSE_STREET',[39,60]),('WEST_GATE',[6,72]),('LAKEWALK_LANDMARK',[135,73]),('RICE_POINT',[34,111]),('PARK_POINT',[139,122]),('NORTH_ARRIVAL',[96,6]),('WEAKEST_NORTHWEST',[61,46])]);cameras=[]
    for label,at in views:
        x,y=at;left=min(max(x*16-80,0),whole.width-160);top=min(max(y*16-72,0),whole.height-144);crop=whole.crop((left,top,left+160,top+144));save_render(crop,output/(label+'-terrain-160.png'));frame=oriented_sprite(pack,'SPRITE_CHRIS','south');save_render(frame,output/'player-native-standing-frame.png');player=crop.convert('RGBA');player.alpha_composite(frame,(x*16-left,y*16-top));save_render(player,output/(label+'-160.png'));save_render(player.resize((640,576),Image.Resampling.NEAREST),output/(label+'-4x.png'));cameras.append((label,player.convert('RGB')))
    save_render(room_sheet(cameras,columns=4),output/'decision-cameras.png')
    roomerrors=[];roomimages=[];newrooms=design['interiors'][7:]
    for room in newrooms:
        rw,rh=room['dimensions_metatiles'];bs=[int(v,16) for row in room['rows'] for v in row.split()];ts=room['tileset'];cs=pack['data']['tilesets'][ts]['collision'];rk=lambda p:cs[f'{bs[p[1]//2*rw+p[0]//2]:02x}'][p[1]%2*2+p[0]%2];ra=room['residents'];occ={tuple(a['tile']) for a in ra};rf={(x,y) for y in range(rh*2) for x in range(rw*2) if rk((x,y))=='FLOOR' and (x,y) not in occ};rseen=flood(tuple(room['clear_arrival']),rf)
        for actor in ra:
            at=tuple(actor['tile'])
            if rk(at)!='FLOOR' or (tuple(actor['approach']) not in rseen if actor.get('approach') else not any(q in rseen for q in adjacent(at))):roomerrors.append(dict(room=room['id'],actor=actor['id'],at=at,kind=rk(at)))
        for warp in room['warps']+room['exits']:
            at=tuple(warp['tile']);wanted=warp['collision']
            if rk(at)!=wanted or (wanted=='FLOOR' and at not in rseen) or (wanted!='FLOOR' and not any(q in rseen for q in adjacent(at))):roomerrors.append(dict(room=room['id'],warp=warp['index'],at=at,kind=rk(at),wanted=wanted))
        im=scene(art,pack,ts,rw,rh,bs,ra,archive=output/'rooms'/room['id'],group='indoor',native_origin=True);save_render(im,output/'rooms'/(room['id']+'.png'));roomimages.append((room['id'],im))
    save_render(room_sheet(roomimages,columns=5),output/'new-interiors-sheet.png')
    result=dict(status='SOURCE ONLY; native integration/controller pending',errors=errors,room_errors=roomerrors,dimensions_metatiles=layout['dimensions_metatiles'],native_floor_tiles=len(ground),ordinary_component_floor=len(seen),all_doors=doorchecks,stairs=stairchecks,original_core_exceptions=oldchanges,viewpoints=views,new_facades=len(design['expansion_contract']['facades']),new_rooms=len(newrooms),new_trainers=len(design['expansion_contract']['new_trainers']),source_hashes={name:hashlib.sha256((directory/name).read_bytes()).hexdigest() for name in ['MnDuluth.candidate.json','MnDuluth.layout.candidate.json']},seconds=round(time.perf_counter()-started,3));(output/'source-proof.json').write_text(json.dumps(result,indent=2)+'\n');print(json.dumps(dict(errors=errors,room_errors=roomerrors,ordinary_component_floor=len(seen),new_rooms=len(newrooms),seconds=result['seconds']),indent=2))


if __name__=='__main__':
    parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('--pack',type=Path,required=True);parser.add_argument('--candidate',type=Path,required=True);parser.add_argument('--output',type=Path,required=True);args=parser.parse_args();review(load_pack(args.pack),args.candidate,args.output)
