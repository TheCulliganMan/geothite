#!/usr/bin/env python3
"""Check warm controller course geometry against an actual native pack.

Read-only static evidence; this does not execute controller inputs or a GPU.
"""
import argparse
from collections import Counter
import hashlib
import json
from pathlib import Path
import sys

ROOT=Path(__file__).resolve().parents[6]
sys.path.insert(0,str(ROOT/'planning/minnesota/tools'))
from render_reference import load_pack
from audit_outdoor_connections import collision

COURSES=[
    ('northern-lane',[(40,8),(40,1),(41,1),(41,8)]),
    ('upper-bank',[(34,24),(34,17),(35,17),(35,24),(36,24),(36,17),(37,17),(37,24)]),
    ('shore-cove',[(43,30),(48,30),(48,31),(43,31),(44,31),(44,32),(48,32),(48,33),(44,33),(44,34)]),
    ('green-grass',[(44,30),(44,29),(46,29),(47,29),(44,29),(44,30)]),
    ('palisade-stair',[(26,48),(25,48),(24,48),(24,49),(23,49)]),
    ('birch-stair',[(37,89),(37,90),(37,91),(37,92)]),
    ('lester-stair',[(40,170),(39,170),(38,170),(37,170)]),
    ('palisade-upper',[(23,49),(23,51),(18,51)]),
    ('palisade-return',[(18,54),(28,54),(28,49),(26,49),(26,48),(24,48),(24,51),(18,51)]),
    ('superior-water',[(49,187),(58,187),(58,189)]),
    ('baptism-water',[(21,27),(18,27),(18,26)]),
]
FLOOR_CAMERAS=[('north-coast-exit',(40,8)),('house-and-falls',(23,22)),
    ('ordinary-upper-bank',(36,24)),('superior-cove',(48,33)),
    ('rock-terrace',(25,64)),('second-stream',(33,78)),
    ('birch-cave-stair',(37,91)),('stoney-falls',(25,152)),
    ('duluth-approach',(47,189))]


def sha(raw):return hashlib.sha256(raw).hexdigest()


def points(turns):
    result=[turns[0]]
    for target in turns[1:]:
        p=result[-1]
        assert p[0]==target[0]or p[1]==target[1]
        while p!=target:
            p=(p[0]+(target[0]>p[0])-(target[0]<p[0]),p[1]+(target[1]>p[1])-(target[1]<p[1]))
            result.append(p)
    return result


def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--pack',type=Path,required=True)
    parser.add_argument('--expected-sha256',required=True)
    parser.add_argument('--output',type=Path,required=True)
    args=parser.parse_args()
    assert not args.output.exists(),'Keep every native review epoch'
    assert sha(args.pack.read_bytes())==args.expected_sha256
    pack=load_pack(args.pack);grid={};bodies=set();sights=set();aliases={}
    for name,m in pack['data']['maps'].items():
        if not name.startswith('MnR11C'):continue
        column,row=int(name[6:8]),int(name[9:11]);aliases[name]=m['attributes']['tileset_name']
        assert aliases[name]=='mn_north_shore'
        grid.update({(x+column*64,y+row*48):t for (x,y),t in collision(pack,m).items()})
        for actor in m['objects']:
            p=actor['x']+column*64,actor['y']+row*48;bodies.add(p)
            if actor['object_type']=='OBJECTTYPE_TRAINER':
                assert actor['move_range_x']==actor['move_range_y']==0
                assert actor['sightline_direction_override']is None
                direction=actor['spritemovedata'].rsplit('_',1)[-1]
                dx,dy={'LEFT':(-1,0),'RIGHT':(1,0),'UP':(0,-1),'DOWN':(0,1)}[direction]
                sights.update((p[0]+dx*n,p[1]+dy*n)for n in range(1,actor['radius']+1))
    assert len(aliases)==8 and 'ICE'not in grid.values()
    contract_path=Path(__file__).resolve().parents[1]/'source-v01/native.contract.json'
    assert sha(contract_path.read_bytes())=='ad3d97771d1bd0c2e4bcba253a3dba7e060ee7996f8e723527796644fdff0976'
    contract=json.loads(contract_path.read_bytes());former=set()
    for row in contract['modules']:
        for cell in row['cells']:
            if cell['before']!=0x85:continue
            x,y=cell['source_metatile'];former.update((x*2+dx,y*2+dy)for dx in range(2)for dy in range(2))
    assert len(former)==52
    visited=set();reports=[]
    for name,turns in COURSES:
        path=points(turns);allowed={'WATER'}if name.endswith('-water')else{'FLOOR','TALL_GRASS'}
        assert all(grid[p]in allowed and p not in bodies|sights for p in path),name
        if name in {'northern-lane','upper-bank','shore-cove'}:visited.update(path)
        reports.append(dict(id=name,turns=turns,steps=len(path)-1,
            entered_collision_counts=dict(Counter(grid[p]for p in path[1:])),
            all_actors_and_conservative_trainer_sight_tiles_avoided=True))
    assert former<=visited
    assert grid[18,52]=='HOP_DOWN'
    for p in [(25,49),(36,91),(38,171)]:assert grid[p]=='WALL'
    assert grid[58,190]=='BUOY' and grid[18,25]=='WATERFALL'and grid[18,19]=='WATER'
    for p in [(49,188),(22,27)]:assert grid[p]=='FLOOR'and p not in bodies|sights
    cameras=[]
    for name,p in FLOOR_CAMERAS:
        assert grid[p]=='FLOOR'and p not in bodies|sights
        cameras.append(dict(id=name,global_core=p,map=f'MnR11C{p[0]//64:02}R{p[1]//48:02}',
            local_core=[p[0]%64,p[1]%48],collision='FLOOR',unoccupied=True))
    leaf=ROOT/'crates/crystal-bevy/src/bevy_shell/tests/north_shore_warm.rs'
    report=dict(status='STATIC_NATIVE_COURSE_PASS_NOT_EXECUTED_INPUT_OR_GPU',
        pack_sha256=args.expected_sha256,identity=pack['identity']['content_hash'],
        source_contract_sha256=sha(contract_path.read_bytes()),controller_leaf_sha256=sha(leaf.read_bytes()),
        tool_sha256=sha(Path(__file__).read_bytes()),aliases=aliases,former_ice_quadrants_covered=52,
        courses=reports,legal_initial_floor_cameras=cameras,
        no_deactivated_trainers_or_postfixture_state_writes_required=True,
        limits=['Static paths do not prove input, ledge hopping, Waterfall gates or Save/Continue.',
            'Actual visible coastal closure uses the retained southern Superior buoy at58,190; no old Cedar/Grand Marais seam is asserted.',
            'Native captures must use the actual saved state or explicitly declared legal initial Floor fixture; grass46,29 is a reached Save only.'])
    args.output.mkdir(parents=True)
    (args.output/'proof.json').write_text(json.dumps(report,indent=2)+'\n')
    (args.output/'check_static.snapshot.py').write_bytes(Path(__file__).read_bytes())
    print(json.dumps({k:report[k]for k in ['status','pack_sha256','former_ice_quadrants_covered']}))


if __name__=='__main__':main()
