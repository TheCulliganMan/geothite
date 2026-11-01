#!/usr/bin/env python3
"""Bounded R19 oak-edge candidate; existing route renderer and native art only."""
import argparse
from collections import deque
from copy import deepcopy
import hashlib
import json
from pathlib import Path
import time

from PIL import Image

from render_archive import save_render
from render_reference import SourceArt, layout_blocks, load_pack, room_sheet
from review_league_approach import check, render
from review_prairie_forest_link import inspect_contracts, negative_checks

ROOT = Path(__file__).resolve().parents[3]
PACK_SHA = '9aa950a35453f06cfee2e4a9b674c6d902af7c311f9ca64a083665ac6c60ddb2'
CAMERAS = [('OAK_HOOK_APPROACH', [25,41]), ('HABITAT_APPROACH', [25,46]),
           ('OBSERVATION_NOOK', [23,45]), ('WEST_LOOP_JOIN', [21,44]),
           ('EAST_REWARD_JOIN', [29,41]), ('WALKER_APPROACH', [27,40])]
MASKS = {'UP_WALL': {(0,1)}, 'DOWN_WALL': {(0,-1)},
         'LEFT_WALL': {(1,0)}, 'RIGHT_WALL': {(-1,0)},
         'UP_LEFT_WALL': {(0,1),(1,0)}, 'UP_RIGHT_WALL': {(0,1),(-1,0)},
         'DOWN_LEFT_WALL': {(0,-1),(1,0)}, 'DOWN_RIGHT_WALL': {(0,-1),(-1,0)}}
LAND = {'FLOOR', 'TALL_GRASS', *MASKS}


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def graph(kind, occupied):
    """Ordinary native directional land; entering and leaving side walls matter."""
    result = {}
    for y in range(60):
        for x in range(64):
            at = x,y
            if at in occupied or kind(*at) not in LAND:
                continue
            result[at] = []
            for dx,dy in [(1,0),(-1,0),(0,1),(0,-1)]:
                q = x+dx,y+dy
                if not (0 <= q[0] < 64 and 0 <= q[1] < 60) or q in occupied:
                    continue
                if kind(*q) not in LAND:
                    continue
                if (dx,dy) in MASKS.get(kind(*q), set()) or (-dx,-dy) in MASKS.get(kind(*at), set()):
                    continue
                result[at].append(q)
    return result


def distances(edges, start, band=frozenset(), forbidden=frozenset()):
    costs = {start: 0}; todo = deque([start])
    while todo:
        at = todo.popleft()
        for q in edges.get(at, []):
            if q in forbidden:
                continue
            cost = costs[at] + int(q in band)
            if cost >= costs.get(q, 10**9):
                continue
            costs[q] = cost
            (todo.append if q in band else todo.appendleft)(q)
    return costs


def hook_geometry(kind, occupied):
    edges = graph(kind, occupied)
    assert kind(25,42)=='HEADBUTT_TREE', 'Oak hook does not turn the road'
    lanes = [[(x,y) for y in range(40,46)] for x in [26,27]]
    assert all(kind(*q)=='FLOOR' and q not in occupied for lane in lanes for q in lane), 'Both ordinary lanes beside hook must remain safe'
    assert kind(24,46)==kind(25,46)=='FLOOR', 'Lower two-core return missing'
    assert all(kind(x,y)=='HEADBUTT_TREE' for y in [42,43,44,45,46,47] for x in [28,29]), 'Eastern field edge detached'
    assert all(kind(x,48)=='WALL' for x in [28,29]), 'Oak edge must attach to original forest boundary'
    steps = [distances(edges,a,set(edges))[b] for a,b in [((25,41),(25,44)),((25,44),(25,41))]]
    assert steps == [5,5], 'Short ordinary hook turn has a bypass or is overextended'
    for q in [(21,44),(29,41),(23,45)]:
        assert q in distances(edges,(25,41)), 'Branch or board nook sealed'
    return dict(dry_hook_steps_both_directions=steps,actual_two_lane_hook_core=lanes,lower_return_core=[[24,46],[25,46]])


def validate(design, before, after, pack, contract):
    assert after['layers'][:len(before['layers'])] == before['layers'], 'Original layer prefix changed'
    retained = deepcopy(after); retained['layers'] = before['layers']
    assert retained == before, 'Retained art/extent/layout field changed'
    strokes = after['layers'][len(before['layers']):]
    assert [dict(position=s['position'],block=s['block']) for s in strokes] == contract['strokes'], 'Hand strokes changed'
    assert all(s['kind']=='point' for s in strokes), 'Unexpected terrain stroke kind'
    old, new = layout_blocks(before), layout_blocks(after)
    changes = [(i%32,i//32,a,b) for i,(a,b) in enumerate(zip(old,new)) if a != b]
    assert changes, 'Candidate has no change'
    assert all(10<=x<15 and 19<=y<24 for x,y,_,_ in changes), 'Changed cell outside approved envelope'
    assert len(changes)==len(strokes), 'Redundant or hidden stroke'
    native = [pack['data']['maps'][f'MnR19C00R{y//24:02}']['blocks'][y%24*32+x] for y in range(30) for x in range(32)]
    assert native == old, 'Actual R19 native preimage differs from source'
    report,kind,seen = check(design,after,pack,new)
    assert not report['errors'], report['errors']
    report.update(inspect_contracts(design,after,pack))
    occupied = {tuple(a['tile']) for key in ['trainers','residents','items'] for a in design[key]}
    edges = graph(kind, occupied)
    report.update(hook_geometry(kind,occupied))
    start,end = [tuple(2*v for v in e['gate_metatile']) for e in design['endpoints']]
    forward, backward = distances(edges,start),distances(edges,end)
    assert end in forward and start in backward, 'Native directional ordinary route disconnected'
    assert all(q in forward for q in seen), 'Floor graph claims a directional-unreachable square'
    band = {(x,y) for y in range(48,52) for x in range(22,28)}
    assert all(kind(*q)=='TALL_GRASS' for q in band), 'Original required habitat changed'
    assert distances(edges,start,band)[end] == distances(edges,end,band)[start] == 4, 'Required four grass steps changed'
    assert end not in distances(edges,start,forbidden=band), 'Dry habitat bypass'
    # Preserve both branches, the existing board nook, and all requested cameras.
    probes = [(25,37),(25,48),(21,44),(29,41),(23,45)] + [tuple(at) for _,at in CAMERAS]
    assert all(q in forward and q in backward for q in probes), 'Branch, nook or camera inaccessible'
    assert all(kind(*q) in {'FLOOR','TALL_GRASS'} for q in probes), 'Unsafe approach probe'
    assert all(kind(*at)=='FLOOR' for _,at in CAMERAS), 'Unsafe decision camera'
    for actor in design['trainers']+design['residents']+design['items']+design['signs']:
        at = tuple(actor['tile'])
        assert any(q in forward for q in [(at[0]-1,at[1]),(at[0]+1,at[1]),(at[0],at[1]-1),(at[0],at[1]+1)]), actor['id']+': no native directional interaction approach'
    report.update(changed_metatiles=[dict(at=[x,y],before=f'{a:02x}',after=f'{b:02x}') for x,y,a,b in changes],
                  directional_reachable_tiles=len(forward),required_grass_steps_both_directions=[4,4],
                  source_semantics='collision.rs enter/leave side-wall masks; ordinary land, no hop edges or Surf',
                  native_directional_probes=[list(q) for q in probes],
                  boundary_source_unchanged=True,all_actors_dossier_unchanged=True,
                  explicit_remaining_weakness='Widest upper oak bend35,33 remains outside this bounded candidate.',
                  proof_limit='Source/static collision and native-art composite; no native publication/controller/GPU/Continue acceptance.')
    return report,kind,edges


def main():
    parser=argparse.ArgumentParser();parser.add_argument('candidate',type=Path);args=parser.parse_args()
    out=args.candidate; contract=json.loads((out/'contract.json').read_text());pack_path=ROOT/contract['pack']
    assert digest(pack_path)==PACK_SHA==contract['pack_sha256'], 'Frozen pack changed'
    for name in ['MN_R19.json','MN_R19.layout.json']:
        assert (out/('baseline-'+name)).read_bytes()==(ROOT/'planning/minnesota/routes'/name).read_bytes(), 'Live source no longer matches frozen preimage: '+name
    before=json.loads((out/'baseline-MN_R19.layout.json').read_text());after=json.loads((out/'MN_R19.layout.candidate.json').read_text());design=json.loads((out/'baseline-MN_R19.json').read_text());pack=load_pack(pack_path)
    started=time.perf_counter(); report,kind,edges=validate(design,before,after,pack,contract)
    # Use the established whole-route renderer, with only additional camera entries.
    visual=deepcopy(design); visual['viewpoints'] += [dict(id=name,tile=at) for name,at in CAMERAS]
    for name,layout in [('before',before),('after',after)]:
        result=render(visual,layout,pack,SourceArt(pack),output=out/name)
        assert not result['errors'],result['errors']
    views=[]
    for camera,_ in CAMERAS:
        for phase in ['before','after']:
            im=Image.open(out/phase/f'viewport-{camera.lower()}-native-rgb.png')
            views.append((phase+' '+camera,im))
    save_render(room_sheet(views,columns=2),out/'before-after-decisions.png')
    # Inspect the widest remaining local walkable shoulder with exactly the same
    # source-scene framing; whole-route weakest remains recorded by shared renderer.
    local=[]
    for q in edges:
        x,y=q
        if 20<=x<30 and 38<=y<48 and kind(x,y)=='FLOOR':
            score=sum((xx,yy) in edges for yy in range(y-3,y+4) for xx in range(x-3,x+4))
            local.append((score,x,y))
    _,x,y=max(local);report['widest_remaining_local_camera_core']=[x,y]
    for phase in ['before','after']:
        image=Image.open(out/phase/'whole-route-original-rgba.png')
        crop=image.crop((x*16-80,y*16-72,x*16+80,y*16+72));save_render(crop,out/phase/'viewport-local_worst-native-rgba.png');save_render(crop.convert('RGB'),out/phase/'viewport-local_worst-native-rgb.png')
    report['existing_negative_cases_rejected']=negative_checks(design,after,pack)
    # These mutations bypass the editorial stroke guard and must fail actual
    # collision/route geometry, so preservation checks cannot mask a weak proof.
    occupied={tuple(a['tile']) for key in ['trainers','residents','items'] for a in design[key]}
    geometry_negatives=[]
    for label,mutate in [
        ('straight dry lane restored through hook',lambda l:l['layers'].append(dict(kind='point',position=[12,21],block='01'))),
        ('eastern shoulder opens',lambda l:l['layers'].append(dict(kind='point',position=[14,22],block='02'))),
        ('ordinary two-lane passage blocked',lambda l:l['layers'].append(dict(kind='point',position=[13,21],block='61'))),
        ('western branch arrival blocked',lambda l:l['layers'].append(dict(kind='point',position=[10,22],block='61'))),
    ]:
        bad=deepcopy(after);mutate(bad);blocks=layout_blocks(bad);collision=pack['data']['tilesets']['johto']['collision']
        bad_kind=lambda x,y:collision[f'{blocks[y//2*32+x//2]:02x}'][y%2*2+x%2]
        try:hook_geometry(bad_kind,occupied)
        except (AssertionError,KeyError):geometry_negatives.append(label)
        else:raise AssertionError('Accepted bad collision geometry: '+label)
    report['geometry_negative_cases_rejected']=geometry_negatives
    rejected=[]
    for label,mutate in [
        ('out-of-envelope stroke',lambda l:l['layers'].append(dict(kind='point',position=[15,23],block='61'))),
        ('original-layer edit',lambda l:l['layers'][0].update(width_metatiles=3)),
        ('art identity changed',lambda l:l.update(compiled_tileset='kanto')),
        ('dry southern grass opening',lambda l:l['layers'].append(dict(kind='point',position=[12,24],block='01'))),
        ('western branch sealed',lambda l:l['layers'].append(dict(kind='rect',bounds=[10,21,15,23],material='oak'))),
    ]:
        bad=deepcopy(after);mutate(bad)
        try:validate(design,before,bad,pack,contract)
        except (AssertionError,KeyError,IndexError):rejected.append(label)
        else:raise AssertionError('Accepted bad source: '+label)
    for name in ['MN_R19.json','MN_R19.layout.json']:
        assert (out/('baseline-'+name)).read_bytes()==(ROOT/'planning/minnesota/routes'/name).read_bytes(), 'Live source changed during read-only review: '+name
    assert digest(pack_path)==PACK_SHA, 'Pack changed during review'
    report.update(rejected_additional_cases=rejected,review_ms=round((time.perf_counter()-started)*1000,1),
                  source_hashes={name:digest(out/name) for name in ['baseline-MN_R19.json','baseline-MN_R19.layout.json','MN_R19.layout.candidate.json','contract.json']},pack_sha256=PACK_SHA,
                  active_source_preimages_unchanged=True,reviewer_sha256=digest(Path(__file__)),
                  collision_source_sha256=digest(ROOT/'crates/crystal-core/src/world/collision.rs'))
    (out/'source-proof.json').write_text(json.dumps(report,indent=2)+'\n')
    print(json.dumps(report,indent=2))


if __name__=='__main__':
    main()
