#!/usr/bin/env python3
"""Review the hand-authored R12 depth candidate with existing cave rendering."""
import argparse
from collections import deque
from copy import deepcopy
import hashlib
import json
from pathlib import Path
import time

from PIL import Image

from render_archive import save_render
from render_reference import load_pack, npc_palette_pack, room_sheet
from render_stages import render_stage
from review_cave_candidate import review
from review_north_shore import oriented_sprite

ROOT = Path(__file__).resolve().parents[3]


def sha(raw):
    return hashlib.sha256(raw).hexdigest()


def walk_graph(kind, bodies, surf=False):
    """Native ordinary cave land, including directional UP_WALL enter/leave."""
    allowed={'FLOOR','UP_WALL'} | ({'WATER'} if surf else set())
    edges={}
    for y in range(24):
        for x in range(32):
            at=x,y
            if at in bodies or kind(at) not in allowed:
                continue
            edges[at]=[]
            for dx,dy in [(1,0),(-1,0),(0,1),(0,-1)]:
                q=x+dx,y+dy
                if not(0<=q[0]<32 and 0<=q[1]<24) or q in bodies or kind(q) not in allowed:
                    continue
                if (kind(at)=='UP_WALL' and dy==-1) or (kind(q)=='UP_WALL' and dy==1):
                    continue
                edges[at].append(q)
    return edges


def walk(edges,start,target=None,forbidden=frozenset()):
    parents={start:None};todo=deque([start])
    while todo:
        at=todo.popleft()
        if at==target:
            result=[]
            while at is not None:
                result.append(at);at=parents[at]
            return result[::-1]
        for q in edges.get(at,[]):
            if q not in parents and q not in forbidden:
                parents[q]=at;todo.append(q)
    return set(parents) if target is None else None


def validate(room,previous,pack):
    retained=deepcopy(room)
    for key in ['rows','purpose','candidate_status','review_contract']:
        retained[key]=previous[key]
    assert retained==previous, 'Original nonterrain field/actor/program/item/mouth changed'
    assert room['dimensions_metatiles']==[16,12], 'Original extent changed'
    rows=[row.split() for row in room['rows']]
    assert len(rows)==12 and all(len(row)==16 for row in rows), 'Invalid rows'
    oldrows=[row.split() for row in previous['rows']]
    changes=[dict(at=[x,y],before=a,after=b) for y in range(12) for x,(a,b) in enumerate(zip(oldrows[y],rows[y])) if a!=b]
    assert all(1<=x<15 and 1<=y<11 for x,y in [c['at'] for c in changes]), 'Original perimeter or mouth changed'
    module=pack['data']['maps'][room['id']]
    assert module['blocks']==[int(v,16) for row in oldrows for v in row], 'Native preimage differs from current source'
    assert module['attributes']['time_of_day']=='nite' and room['review_contract']['palette_group']=='nite'
    blocks=[int(v,16) for row in rows for v in row];collision=pack['data']['tilesets']['cave']['collision']
    kind=lambda p:collision[f'{blocks[p[1]//2*16+p[0]//2]:02x}'][p[1]%2*2+p[0]%2]
    assert all(kind((x,y)) in {'FLOOR','WALL','UP_WALL','LADDER','HOP_DOWN','WATER'} for y in range(24) for x in range(32)), 'Unsupported native collision'
    actors=room['residents']+room['items'];bodies={tuple(a['tile']) for a in actors}
    graph=walk_graph(kind,bodies);entry=tuple(room['clear_arrival']);seen=walk(graph,entry)
    floor={q for q in graph if kind(q)=='FLOOR'}
    assert floor<=seen, 'Ordinary Floor not reachable with actor bodies present'
    assert all(kind(tuple(a['tile']))=='FLOOR' and any(q in seen for q in [(a['tile'][0]-1,a['tile'][1]),(a['tile'][0]+1,a['tile'][1]),(a['tile'][0],a['tile'][1]-1),(a['tile'][0],a['tile'][1]+1)]) for a in actors), 'Actor inaccessible'
    ladders={(x,y) for y in range(24) for x in range(32) if kind((x,y))=='LADDER'}
    assert ladders=={tuple(p) for p in previous['exits']}=={(17,23)}, 'Unpaired ladder/mouth'
    assert kind((17,22))=='FLOOR', 'Unsafe arrival'
    probes=room['review_contract']['controller_probes'];stairs=[]
    assert len(probes['stairs'])==3, 'Three distinct stair roles expected'
    for stair in probes['stairs']:
        x,y=stair['metatile'];assert blocks[y*16+x]==int(stair['block'],16), stair['id']+': wrong native art'
        lane=[tuple(p) for p in [stair['before']]+stair['lane']+[stair['after']]]
        assert all(kind(p)=='FLOOR' and p not in bodies for p in lane), stair['id']+': blocked lane'
        assert all(b in graph[a] and a in graph[b] for a,b in zip(lane,lane[1:])), stair['id']+': not bidirectional'
        a,b=tuple(stair['blocked_from']),tuple(stair['blocked_target'])
        assert kind(b)=='WALL' and b not in graph[a], stair['id']+': side bypass'
        stairs.append(dict(id=stair['id'],metatile=stair['metatile'],block=stair['block'],both_direction_lane=lane,blocked_side=b))
    assert sum(b in {0x12,0x30,0x36} for b in blocks)==3, 'Unreviewed or missing stairs'
    # Distinct branch geometry: each upper destination has its own useful stair;
    # blocking one does not block the other. The throat is the sole dry ascent.
    branches=[]
    targets={'OTTA_ROOF':(6,5),'CACHE_SPUR':(26,5)}
    for stair in probes['stairs']:
        forbidden={tuple(p) for p in stair['lane']}
        reachable=walk(graph,entry,forbidden=forbidden)
        if stair['id']=='RISING_THROAT':
            assert not set(targets.values()) & reachable, 'Main climb bypassed'
        else:
            own=targets[stair['id']];other=next(at for name,at in targets.items() if name!=stair['id'])
            assert own not in reachable and other in reachable, 'Destination branch not distinct: '+stair['id']
        branches.append(dict(closed_stair=stair['id'],otta_reachable=(6,5) in reachable,cache_reachable=(26,5) in reachable))
    ledges=[]
    expected_hops={(14,14),(15,14)}
    assert {(x,y) for y in range(24) for x in range(32) if kind((x,y))=='HOP_DOWN'}==expected_hops
    assert probes['ledge_shoulders']==[[13,15],[16,15]], 'Declared ledge shoulders must use Wall course, not Floor lip'
    assert all(kind(tuple(p))=='WALL' for p in probes['ledge_shoulders']), 'Declared course shoulders open'
    for lane in probes['ledge_lanes']:
        approach,hop,landing,reverse=[tuple(lane[k]) for k in ['upper_approach','hop_from','landing','reverse_target']]
        assert approach in floor and landing in floor and kind(hop)=='HOP_DOWN' and kind(reverse)=='WALL'
        assert all(kind((reverse[0]+dx,reverse[1]))=='WALL' for dx in [-1,1]), 'Ledge course side bypass'
        path=walk(graph,landing,approach)
        assert path and 9<=len(path)-1<=30, 'Ordinary ledge return disconnected/trivial/empty detour'
        assert reverse not in graph[landing], 'Reverse ledge climb possible'
        ledges.append(dict(upper_approach=approach,hop_from=hop,landing=landing,reverse_wall=reverse,ordinary_return_steps=len(path)-1,ordinary_return_core=path))
    surf=walk_graph(kind,bodies,True);surfseen=walk(surf,entry)
    water={(x,y) for y in range(24) for x in range(32) if kind((x,y))=='WATER'}
    assert water and water<=surfseen, 'Disconnected/unreachable seep water'
    assert not any(x in {0,31} or y in {0,23} for x,y in water), 'Reachable Surf reaches unconnected perimeter'
    assert kind((21,17))=='FLOOR' and kind((22,17))=='WATER' and (22,17) in surf[(21,17)], 'Native bank/launch missing'
    throat=next(s for s in probes['stairs'] if s['id']=='RISING_THROAT')
    assert not set(targets.values()) & walk(surf,entry,forbidden={tuple(p) for p in throat['lane']}), 'Surf bypasses rising throat'
    for v in room['review_contract']['viewpoints']:
        assert tuple(v['tile']) in floor, v['id']+': unsafe camera'
    return dict(changed_metatiles=changes,all_retained_nonterrain_fields_exact=True,original_outer_perimeter_and_mouth_exact=True,
                reachable_floor_tiles=len(floor),reachable_directional_land_tiles=len(seen),native_stairs=stairs,
                destination_branch_cut_checks=branches,actual_two_lane_ledge_returns=ledges,
                actual_reachable_pool_water_tiles=len(water),reachable_outer_water_tiles=0,
                dry_actor_routes=True,surf_cannot_bypass_ascent=True,native_module_sha256=sha(json.dumps(module,sort_keys=True,separators=(',',':')).encode()),
                source_collision_semantics='Actual collision.rs UP_WALL enter-down/leave-up masks; no assumed solid directional roof walls.'),kind,floor


def change(room,x,y,value):
    rows=[row.split() for row in room['rows']];rows[y][x]=value;room['rows']=[' '.join(row) for row in rows]


def main():
    parser=argparse.ArgumentParser();parser.add_argument('candidate',type=Path);parser.add_argument('--pack',type=Path,default=ROOT/'content-packs/minnesota-northstar-map-quality.crystalpack');args=parser.parse_args()
    out=args.candidate;source=out/'MnR12Cave1F.depth.candidate.json';previous=out/'MnR12Cave1F.previous.json'
    raw=source.read_bytes();room=json.loads(raw);old=json.loads(previous.read_bytes());packhash=sha(args.pack.read_bytes());pack=load_pack(args.pack);started=time.perf_counter()
    report,kind,floor=validate(room,old,pack)
    # All whole-room art and NPC composition comes from the established renderer.
    for phase,data in [('before',old),('after',room)]:
        review(pack,data,out/phase)
    scene=Image.open(out/'after/whole-source.png');save_render(room_sheet([('INSTALLED PREIMAGE',Image.open(out/'before/whole-source.png')),('R12 DEPTH CANDIDATE',scene)],columns=2),out/'whole-before-after.png')
    sizes={};largest=(0,0,0)
    for y in range(24):
        for x in range(32):
            n=1+min(sizes.get((x-1,y),0),sizes.get((x,y-1),0),sizes.get((x-1,y-1),0)) if (x,y) in floor else 0
            sizes[x,y]=n
            if n>largest[0]:largest=n,x-n+1,y-n+1
    n,x,y=largest;weakest=[x+n//2,y+n//2]
    views=room['review_contract']['viewpoints']+[dict(id='ACTUAL_WEAKEST_FLOOR',tile=weakest)]
    player=oriented_sprite(npc_palette_pack(pack,'nite'),'SPRITE_CHRIS','south');cameras=[];pairs=[]
    for view in views:
        x,y=view['tile'];name=view['id'];assert (x,y) in floor
        for phase in ['before','after']:
            im=Image.open(out/phase/'whole-source.png');crop=render_stage(im,'crop',out/phase,name,(x*16-80,y*16-72,x*16+80,y*16+72));save_render(crop,out/phase/(name+'-terrain160.png'));pairs.append((phase+' '+name,crop))
            if phase=='after':
                rgba=render_stage(crop,'convert',out/phase,name+'-player-rgba','RGBA');rgba.alpha_composite(player,(80,72));save_render(rgba,out/phase/(name+'-player-rgba.png'));rgb=render_stage(rgba,'convert',out/phase,name+'-rgb','RGB');save_render(rgb,out/phase/(name+'-player160.png'));cameras.append((name+' '+str(view['tile']),rgb))
    save_render(room_sheet(cameras,columns=3),out/'decision-cameras.png');save_render(room_sheet(pairs,columns=2),out/'before-after-cameras.png')
    rejected=[]
    for label,mutation in [
        ('moved original Otta',lambda r:r['residents'][0].update(tile=[6,5])),
        ('changed original Max Ether',lambda r:r['items'][0].update(quantity=2)),
        ('outer Surf breach',lambda r:change(r,15,8,'0f')),
        ('missing native mouth',lambda r:change(r,8,11,'01')),
        ('main climb sealed',lambda r:change(r,9,7,'21')),
        ('Otta spur sealed',lambda r:change(r,5,3,'21')),
        ('Ether spur sealed',lambda r:change(r,11,3,'21')),
        ('stair wrong-side opens',lambda r:change(r,9,7,'02')),
        ('reverse ledge course opens',lambda r:change(r,7,7,'02')),
        ('ledge course side opens',lambda r:change(r,8,7,'02')),
        ('left ledge course side opens',lambda r:change(r,6,7,'02')),
        ('wrong Floor lip declared as Wall shoulder',lambda r:r['review_contract']['controller_probes'].update(ledge_shoulders=[[13,14],[16,14]])),
        ('new fake ladder',lambda r:change(r,6,5,'07')),
    ]:
        bad=deepcopy(room);mutation(bad)
        try:validate(bad,old,pack)
        except (AssertionError,KeyError):rejected.append(label)
        else:raise AssertionError('Accepted bad fixture: '+label)
    assert source.read_bytes()==raw and sha(args.pack.read_bytes())==packhash, 'Inputs changed during review'
    active=json.loads((ROOT/'planning/minnesota/routes/MN_R12.json').read_bytes());active_room=next(r for r in active['interiors'] if r['id']==old['id'])
    assert active_room==old, 'Active owned room changed during review'
    report.update(status='SOURCE ONLY: no native publication/controller/GPU/Continue acceptance',pack=str(args.pack),pack_sha256=packhash,
                  source_sha256=sha(raw),previous_sha256=sha(previous.read_bytes()),native_palette='nite',
                  widest_plain_floor_square_core=n,actual_weakest_floor_camera_core=weakest,
                  rejected_negative_cases=rejected,active_owned_room_unchanged=True,
                  review_seconds=round(time.perf_counter()-started,3),reviewer_sha256=sha(Path(__file__).read_bytes()))
    (out/'source-proof.json').write_text(json.dumps(report,indent=2)+'\n');(out/'review.py.frozen.snapshot').write_bytes(Path(__file__).read_bytes());print(json.dumps(report,indent=2))


if __name__=='__main__':
    main()
