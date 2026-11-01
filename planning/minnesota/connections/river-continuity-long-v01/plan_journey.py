#!/usr/bin/env python3
"""Plan against frozen source, optionally bind exact written native output.

This never executes gameplay or writes a pack. Only the declared existing
Minneapolis gate-open changes are applied to a private planning map copy.
"""
import argparse
import base64
from collections import deque
from copy import deepcopy
import hashlib
import json
from pathlib import Path
import sys

ROOT = Path(__file__).resolve().parents[4]
HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(ROOT/'planning/minnesota/tools'))
sys.path.insert(0, str(HERE))
from audit_city_expansions import module_hash
from audit_outdoor_connections import collision, project
from physical_proof import water_step
from render_reference import load_pack
from review_mississippi_connections import bfs_water, input_plan

CONTRACT_SHA = '67c451f315baf3939f92ae619aae44c50963c069633dab177076ee18b99cd3cd'
COMPASS = {'up':(0,-1,'north'),'down':(0,1,'south'),
           'left':(-1,0,'west'),'right':(1,0,'east')}


def sha(raw):
    return hashlib.sha256(raw).hexdigest()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--pack', type=Path, required=True, help='Actual approved parent, before this river revision.')
    parser.add_argument('--expected-parent-sha256', required=True)
    parser.add_argument('--contract', type=Path, default=HERE/'source-v02/candidate.contract.json')
    parser.add_argument('--written-pack', type=Path)
    parser.add_argument('--expected-written-sha256')
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    assert bool(args.written_pack) == bool(args.expected_written_sha256)
    assert not args.output.exists(), 'Preserve every input-plan epoch.'
    args.output.mkdir(parents=True)
    assert sha(args.pack.read_bytes()) == args.expected_parent_sha256
    assert sha(args.contract.read_bytes()) == CONTRACT_SHA
    contract = json.loads(args.contract.read_text())
    parent = load_pack(args.pack)
    assert len(parent['data']['maps']) == 683
    candidate = deepcopy(parent)
    for row in contract['changes']:
        name = row['map']
        old = parent['data']['maps'][name]
        assert module_hash(old) == row['native_module_sha256'], ('Stale owned map',name)
        module = candidate['data']['maps'][name]
        width = module['attributes']['width']
        for cell in row['cells']:
            x,y = cell['metatile']
            assert module['blocks'][y*width+x] == int(cell['before'],16)
            module['blocks'][y*width+x] = int(cell['after'],16)
        if row['cells']:
            label = module['attributes']['blocks_label']
            candidate['data']['map_blocks'][label] = base64.b64encode(bytes(module['blocks'])).decode()
    bound_hash = None
    if args.written_pack:
        bound_hash = args.expected_written_sha256
        assert sha(args.written_pack.read_bytes()) == bound_hash
        written = load_pack(args.written_pack)
        assert written['data'] == candidate['data'], 'Written native data differs outside or inside frozen cells/mirrors'
        for key in ['audio_manifest','runtime_files','report','format_version']:
            assert written.get(key) == parent.get(key), ('Unrelated written payload changed',key)
        candidate = written
    maps = dict(candidate['data']['maps'])
    city = deepcopy(maps['MnMinneapolis'])
    maps['MnMinneapolis'] = city
    gates = [r for r in city['script_block_changes'] if r['source_script']=='MnMPUpperPoolBuoyOpen']
    assert len(gates) == 3
    for row in gates:
        city['blocks'][(row['y']//2)*city['attributes']['width']+row['x']//2] = row['block_id']
    native = {name:collision(candidate,m) for name,m in maps.items()if name.startswith('Mn')}
    planned = deepcopy(native)
    axes = {}
    for name,grid in native.items():
        for point,permission in grid.items():
            if permission.startswith('BRIDGE_'):
                axes[(name,*point)] = 'north_south' if permission.startswith('BRIDGE_NS') else 'east_west'
                planned[name][point] = 'WATER'
    manifest_path = ROOT/'planning/minnesota/connections/mississippi-v02/manifest.json'
    manifest = json.loads(manifest_path.read_text())
    itinerary = input_plan(maps,planned,axes,manifest,manifest_path,
                           waypoints=[('MnMississippiWinonaMouth',26,0),
                                      ('MnMississippiLowerWoodsC00R02',36,39),
                                      ('MnMississippiLowerWoodsC00R01',0,24),
                                      ('MnStPaul',18,52),
                                      ('MnR05C00R02',63,0),
                                      ('MnMississippiUpstreamWoodsC00R02',0,36),
                                      ('MnMississippiUpstreamWoodsC00R00',0,40)])
    inventory = json.loads((ROOT/manifest['bridge_inventory']).read_text())
    apertures=[]
    for i,row in enumerate(inventory['bridges']):
        name=row['map'];x,y=row['metatiles'][0]['metatile'];seed=(x*2,y*2)
        permission=native[name][seed]
        assert permission.startswith('BRIDGE_')
        connected={seed};todo=deque([seed])
        while todo:
            p=todo.popleft()
            for dx,dy,_ in COMPASS.values():
                q=p[0]+dx,p[1]+dy
                if q not in connected and native[name].get(q,'').startswith('BRIDGE_'):
                    connected.add(q);todo.append(q)
        apertures.append(dict(id=row.get('id',name+'-bridge-'+str(i+1)),map=name,
                              metatile_bounds_exclusive=[min(x for x,y in connected)//2,min(y for x,y in connected)//2,
                                  max(x for x,y in connected)//2+1,max(y for x,y in connected)//2+1],
                              actual_bridge_quadrants=len(connected)))
    assert len(apertures)==7
    start=itinerary['start'];launch=itinerary['launch'];node=(start['map'],*start['point'])
    assert native[node[0]][node[1:]]=='FLOOR'
    dx,dy,_=COMPASS[launch['direction']]
    assert (launch['map'],*launch['point'])==(node[0],node[1]+dx,node[2]+dy)
    node=(launch['map'],*launch['point']);assert native[node[0]][node[1:]]=='WATER'
    crossed=set();waterfalls=[];reach_counts={'LowerWoods':0,'UpstreamWoods':0}
    for i,step in enumerate(itinerary['steps']):
        target=(step['map'],*step['point']);dx,dy,direction=COMPASS[step['direction']]
        if step.get('action')=='waterfall_up':
            assert direction=='north'and target[0:2]==node[0:2]and target[2]<node[2]-1
            assert native[node[0]][node[1:]]==native[target[0]][target[1:]]=='WATER'
            assert all(native[node[0]][node[1],y]=='WATERFALL'for y in range(target[2]+1,node[2]))
            waterfalls.append(dict(input_index=i,before=list(node),after=list(target)))
        else:
            q=node[1]+dx,node[2]+dy;a=maps[node[0]]['attributes']
            if not(0<=q[0]<a['width']*2 and 0<=q[1]<a['height']*2):
                edge=next(e for e in a['connections']if e['direction']==direction)
                actual=(edge['target_map'],*project(a,maps[edge['target_map']]['attributes'],edge,node[1:]))
            else:
                actual=(node[0],*q)
            assert actual==target,(i,node,actual,target)
            assert water_step(native[node[0]][node[1:]],native[target[0]][target[1:]],direction),(i,node,target)
        assert target[1:] not in {(o['x'],o['y'])for o in maps[target[0]]['objects']},('Actor blocks water',i,target)
        for reach in reach_counts:
            if reach in node[0] and reach in target[0]:reach_counts[reach]+=1
        for aperture in apertures:
            x0,y0,x1,y1=aperture['metatile_bounds_exclusive']
            if target[0]==aperture['map']and x0*2<=target[1]<x1*2 and y0*2<=target[2]<y1*2:
                crossed.add(aperture['id'])
        node=target
    assert node==(itinerary['target']['map'],*itinerary['target']['point'])
    assert len(crossed)==7,('Bridge missing',crossed)
    origin=(launch['map'],*launch['point']);target=('MnStCloud',40,20)
    upper={n for n in maps if n.startswith('MnMississippiUpstreamWoods')}
    lower={n for n in maps if n.startswith('MnMississippiLowerWoods')}
    for excluded in [upper,lower,{'MnStPaul'}]:
        assert bfs_water(maps,planned,origin,target,axes,excluded)is None,('Whole reach bypassed',excluded)
    assert reach_counts=={'LowerWoods':361,'UpstreamWoods':374},reach_counts
    itinerary.update(status='WRITTEN_NATIVE_INPUT_PLAN_BOUND_CONTROLLER_PENDING'if bound_hash else 'SOURCE_INPUT_PLAN_ONLY_NOT_BOUND_OR_EXECUTED',
                     pack_sha256=bound_hash,parent_pack_sha256=args.expected_parent_sha256,
                     source_preimage_baseline_sha256=contract['parent_pack_sha256'],
                     source_contract_sha256=CONTRACT_SHA,bridge_apertures=apertures,
                     source_only_wooded_reach_inputs=reach_counts)
    (args.output/'all-bridges-input-plan.json').write_text(json.dumps(itinerary,indent=2)+'\n')
    proof=dict(status=itinerary['status'],actual_parent_sha256=args.expected_parent_sha256,
               written_pack_sha256=bound_hash,source_contract_sha256=CONTRACT_SHA,
               input_count=len(itinerary['steps']),source_only_wooded_reach_inputs=reach_counts,
               bridge_apertures=apertures,visited_bridges=sorted(crossed),waterfall_actions=waterfalls,
               missing_upper_lower_and_st_paul_bypass_negatives_rejected=True,
               gate_setup='Private planning copy opens the three existing declared blocks. Native input must earn/supply legal prerequisites and execute the existing badge callback; no pack is changed.',
               native_controller_executed=False,parent_pack_unchanged=sha(args.pack.read_bytes())==args.expected_parent_sha256,
               source_tool_sha256=sha(Path(__file__).read_bytes()))
    (args.output/'proof.json').write_text(json.dumps(proof,indent=2)+'\n')
    print(json.dumps(proof,indent=2))


if __name__=='__main__':
    main()
