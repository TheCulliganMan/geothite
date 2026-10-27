#!/usr/bin/env python3
"""Bounded R10 candidate review; reuses native source art and route rendering."""
import argparse
from collections import deque
from copy import deepcopy
import hashlib
import json
from pathlib import Path

from PIL import Image

from render_archive import save_render
from render_reference import SourceArt, layout_blocks, load_pack, room_sheet
from review_iron_range_descent import inspect_contracts
from review_league_approach import inspect_league, render

ROOT = Path(__file__).resolve().parents[3]
PACK_SHA = '31258a1788a986b158d0679e4f3a26c4dd476964311f707a6e09c9640cad48c6'
SEAMS = [(0,5,0x68,1),(47,10,2,5),(47,11,2,5),(47,12,2,1),(47,13,2,5)]


def path(kind, occupied, start, target, forbidden=frozenset()):
    """Native ordinary directional land, with no jump edge or Surf."""
    parents={start:None};todo=deque([start])
    directions=[(-1,0),(1,0),(0,-1),(0,1)]
    masks={'UP_WALL':{(0,1)},'DOWN_WALL':{(0,-1)},
           'LEFT_WALL':{(1,0)},'RIGHT_WALL':{(-1,0)}}
    land={'FLOOR','TALL_GRASS','HOP_DOWN','HOP_LEFT','HOP_RIGHT',*masks}
    while todo:
        at=todo.popleft()
        if at==target:
            result=[at]
            while parents[at] is not None:at=parents[at];result.append(at)
            return result[::-1]
        for dx,dy in directions:
            q=at[0]+dx,at[1]+dy
            if q in parents or q in occupied or q in forbidden or not(0<=q[0]<96 and 0<=q[1]<40):continue
            current,destination=kind(*at),kind(*q)
            if destination not in land or (dx,dy) in masks.get(destination,set()) or (-dx,-dy) in masks.get(current,set()):continue
            parents[q]=at;todo.append(q)
    return None


def validate(design, old, new, pack, source):
    assert new['compiled_tileset']=='mn_r10_rockstairs'
    retained=deepcopy(new);retained['compiled_tileset']=old['compiled_tileset'];retained.pop('derived_blocks');retained['layers']=retained['layers'][:-2]
    assert retained==old,'Retained layout contract changed'
    expected=[dict(id='80',source_tileset='cave',source_block=source,collision=('FLOOR/FLOOR/WALL/FLOOR' if source=='12' else 'FLOOR/FLOOR/FLOOR/WALL'),required_typed_extension='r10_stairs_revision::revise_r10_rock_stairs')]
    assert new['derived_blocks']==expected,'Exact stair art recipe drift'
    before,after=layout_blocks(old),layout_blocks(new)
    changes=[(i%48,i//48,a,b) for i,(a,b) in enumerate(zip(before,after)) if a!=b]
    assert changes==[(5,10,0x74,0x80),(4,11,0x71,0x0a)],('Unexpected terrain delta',changes)
    assert all(4<=x<8 and 9<=y<13 for x,y,_,_ in changes),'Out-of-bounds terrain change'
    adapted=dict(pack);adapted['data']=dict(pack['data']);adapted['data']['tilesets']=dict(pack['data']['tilesets'])
    ts=deepcopy(pack['data']['tilesets']['johto']);assert len(ts['collision'])==128
    ts['collision']['80']=pack['data']['tilesets']['cave']['collision'][source]
    adapted['data']['tilesets']['mn_r10_rockstairs']=ts
    assert ts['collision']['80']==(['FLOOR','FLOOR','WALL','FLOOR'] if source=='12' else ['FLOOR','FLOOR','FLOOR','WALL'])
    # Certify old installed geography including the separately installed city seams.
    native=[]
    for y in range(20):
        for x in range(48):
            m=pack['data']['maps'][f'MnR10C{x//32:02}R00'];native.append(m['blocks'][y*m['attributes']['width']+x%32])
    assert [(i%48,i//48,a,b) for i,(a,b) in enumerate(zip(before,native)) if a!=b]==SEAMS,'Installed seam preimage drift'
    old_report=inspect_contracts(design,old,pack)
    report,kind,seen=inspect_league(design,new,adapted,after)
    assert not report['errors'],report['errors']
    occupied={tuple(a['tile']) for k in ['trainers','residents','items'] for a in design[k]}
    lane=[(11,y) for y in range(19,23)] if source=='12' else [(11,19),(10,19),(10,20),(10,21),(10,22),(11,22)]
    assert all(kind(*q)=='FLOOR' and q not in occupied for q in lane),'Blocked staircase lane'
    wall=(10,21) if source=='12' else (11,21)
    assert kind(*wall)=='WALL','Native stair wrong-side wall missing'
    assert path(kind,occupied,lane[0],lane[-1]) and path(kind,occupied,lane[-1],lane[0]),'Climb not bidirectional'
    assert path(kind,occupied,(10,22),(11,19),{(11,20),(11,21)}) is None if source=='12' else True,'Straight climb bypass through stair side'
    start,end=[tuple(2*v for v in e['gate_metatile']) for e in design['endpoints']]
    assert path(kind,occupied,start,end) and path(kind,occupied,end,start),'Native directional ordinary road disconnected'
    report.update(changed_metatiles=changes,old_native_contract=old_report,stair_lane_core=lane,wrong_side_wall_core=wall,installed_seam_overrides=SEAMS,native_directional_two_way_route=True,native_directional_two_way_climb=True,proof_limit='Source collision/geometry only; native art append, GPU capture and production controller pending')
    return adapted,report


def main():
    parser=argparse.ArgumentParser();parser.add_argument('candidate',type=Path);parser.add_argument('--pack',type=Path,default=ROOT/'content-packs/minnesota-northstar-repel-review.crystalpack');args=parser.parse_args()
    assert hashlib.sha256(args.pack.read_bytes()).hexdigest()==PACK_SHA,'Frozen native epoch changed'
    out=args.candidate;contract=json.loads((out/'contract.json').read_text());old=json.loads((out/'baseline-MN_R10.layout.json').read_text());new=json.loads((out/'MN_R10.layout.candidate.json').read_text());design=json.loads((out/'baseline-MN_R10.json').read_text());pack=load_pack(args.pack)
    adapted,report=validate(design,old,new,pack,contract['stair_source_block'])
    render_design=deepcopy(design);render_design['viewpoints'] += [dict(id='STAIR_UPPER_APPROACH',tile=[11,19]),dict(id='STAIR_LOWER_APPROACH',tile=[11,22])]
    for label,layout,sourcepack in [('before',old,pack),('after',new,adapted)]:
        render(render_design,layout,sourcepack,SourceArt(pack),output=out/label)
    views=[]
    for name in ['western_ramp','west_routine_lane','stair_upper_approach','stair_lower_approach','emptiest_ground']:
        for label in ['before','after']:
            im=Image.open(out/label/f'viewport-{name}-native-rgb.png');views.append((label+' '+name,im))
    save_render(room_sheet(views,columns=2),out/'before-after-decisions.png')
    rejected=[]
    for name,mutate in [
        ('changed-actor-independent-field',lambda l:l.update(dimensions_metatiles=[49,20])),
        ('wrong-art-orientation',lambda l:l['derived_blocks'][0].update(source_block='36')),
        ('transformed-native-art',lambda l:l['derived_blocks'][0].update(tile_row_order=[3,2,1,0])),
        ('out-of-envelope-stroke',lambda l:l['layers'].append(dict(kind='point',name='bad',position=[9,11],block='0a'))),
        ('missing-stair',lambda l:l['layers'][-2].update(block='71')),
        ('removed-attached-shoulder',lambda l:l['layers'][-1].update(block='71')),
    ]:
        bad=deepcopy(new);mutate(bad)
        try:validate(design,old,bad,pack,contract['stair_source_block'])
        except (AssertionError,KeyError,IndexError):rejected.append(name)
        else:raise AssertionError('Accepted bad source: '+name)
    report.update(rejected_negative_cases=rejected,source_hashes={p.name:hashlib.sha256(p.read_bytes()).hexdigest() for p in out.glob('*.json')},pack_sha256=PACK_SHA)
    (out/'source-proof.json').write_text(json.dumps(report,indent=2)+'\n')
    print(json.dumps({k:v for k,v in report.items() if k in ['changed_metatiles','stair_lane_core','wrong_side_wall_core','errors','rejected_negative_cases']},indent=2))


if __name__=='__main__':main()
