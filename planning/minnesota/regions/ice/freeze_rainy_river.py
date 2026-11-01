#!/usr/bin/env python3
"""Freeze explicit source/preimages and meaningful Rainy River physical negatives."""
from pathlib import Path
from collections import deque
from copy import deepcopy
import argparse
import hashlib
import importlib.util
import json
import sys
import time

ROOT=Path(__file__).resolve().parents[4]
AUTHORING_DIR=Path(__file__).resolve().parent
sys.path.insert(0,str(ROOT/'planning/minnesota/tools'))
sys.path.insert(0,str(AUTHORING_DIR))
from render_reference import load_pack
from audit_city_expansions import module_hash
import review_rainy_river as review

SHA=lambda data:hashlib.sha256(data).hexdigest()
BASE_SHA='3d1cbdd18b05ce10ed66db7ab56d41f0032f6e045a0e72ad753064c5dc6ec8df'
R20_SHA='9002397e9453ccfddbf992afa99d1912ca50e660530ce0c56bb5cc63c8de8ed4'


def certify(preview,layout,dossier):
    checks=[]
    def require(label,value):
        assert value,label
        checks.append(label)
    values,kind,edge,course,bodies=review.geometry(preview,layout,dossier)
    require('bounded one-map32x24 geography',layout['dimensions_metatiles']==[32,24])
    allowed={(63,44),(63,45)}
    edge_points={(x,y)for y in range(48)for x in range(64)if x in [0,63]or y in [0,47]}
    require('exact reciprocal2coreport, all other outer ice/snow closed',{p for p in edge_points if kind(p)in{'FLOOR','TALL_GRASS','ICE','WATER'}}==allowed)
    require('all boundaryIce absent',not any(kind(p)=='ICE'for p in edge_points))
    for block,token in [(60,'FLOOR'),(61,'TALL_GRASS'),(62,'ICE'),(63,'WALL')]:
        require(f'{block:02x} exact native collision',preview['data']['tilesets']['mn_winter_outdoor']['collision'][f'{block:02x}']==[token]*4)
    require('no incidental Cut',not any('CUT_TREE'in preview['data']['tilesets']['mn_winter_outdoor']['collision'][f'{v:02x}']for v in values))
    start=tuple(dossier['arrival_core'])
    require('safe exterior arrival',start==(60,44)and kind(start)=='FLOOR'and start not in bodies)
    for actor in dossier['trainers']+dossier['residents']+dossier['items']:
        at=tuple(actor['tile']);require(actor['id']+' Floor body',kind(at)=='FLOOR')
        require(actor['id']+' actor-present interaction',any(course(start,(at[0]+dx,at[1]+dy))is not None for _,(dx,dy)in review.DIRECTIONS))
    require('lower actual west glide17steps',edge((54,36),(-1,0))==((37,36),[(x,36)for x in range(53,36,-1)]))
    require('lower actual east glide17steps',edge((37,36),(1,0))==((54,36),[(x,36)for x in range(38,55)]))
    require('upper cache ice gap down3steps',edge((19,7),(0,1))==((19,10),[(19,y)for y in range(8,11)]))
    require('upper cache ice gap up3steps',edge((19,10),(0,-1))==((19,7),[(19,y)for y in range(9,6,-1)]))
    require('cache island cannot be reached on ordinary snow',course(start,(20,11),ordinary=True)is None)
    snow=course((37,36),(54,36),ordinary=True)
    require('ordinary snow return reaches easternbank',snow is not None)
    # Independent ordinary-step Dijkstra measures the defined western bank neck.
    import heapq
    q=[(0,0,(37,36))];best={(37,36):(0,0)};target=(9,28)
    while q:
        cost,steps,at=heapq.heappop(q)
        if best[at]!=(cost,steps):continue
        if at==target:break
        for _,(dx,dy)in review.DIRECTIONS:
            nxt=(at[0]+dx,at[1]+dy)
            if not(6<=nxt[0]<=37 and 28<=nxt[1]<=38)or nxt in bodies or kind(nxt)not in{'FLOOR','TALL_GRASS'}:continue
            score=(cost+int(kind(nxt)=='TALL_GRASS'),steps+1)
            if score<best.get(nxt,(99999,99999)):best[nxt]=score;heapq.heappush(q,(*score,nxt))
    require('short western hut-bank neck costs6realgrasssteps',best.get(target,(None,))[0]==6)
    require('true complete native facade+return',kind((9,27))=='DOOR'and kind((9,28))=='FLOOR'and course(start,(9,28))is not None)
    door=dossier['doors'][0];room=dossier['interiors'][0]
    require('explicit exterior warp1/interiorpassive2/exteriorpassive2 pairing',door['warp_index']==1 and door['return_warp_index']==2 and door['target_warp']==2 and door['return_tile']==[9,28]and room['exits']==[[5,7]]and room['exit_target_warps']==[2]and room['passive_arrival']=={'index':2,'tile':[5,6]})
    rw,rh=room['dimensions_metatiles'];rows=[int(v,16)for row in room['rows']for v in row.split()];rc=preview['data']['tilesets'][room['tileset']]['collision']
    def room_kind(p):
        x,y=p
        if not(0<=x<rw*2 and 0<=y<rh*2):return 'WALL'
        return rc[f'{rows[y//2*rw+x//2]:02x}'][y%2*2+x%2]
    occupied={tuple(a['tile'])for a in room['residents']};clear=tuple(room['clear_arrival'])
    require('room arrival not occupied and ordinary Floor',room_kind(clear)=='FLOOR'and clear not in occupied)
    seen={clear};q=deque([clear])
    while q:
        at=q.popleft()
        for _,(dx,dy)in review.DIRECTIONS:
            p=(at[0]+dx,at[1]+dy)
            if p not in seen and p not in occupied and room_kind(p)=='FLOOR':seen.add(p);q.append(p)
    for a in room['residents']:
        x,y=a['tile'];require(a['id']+' actual reachable Floor interaction',room_kind((x,y))=='FLOOR'and any((x+dx,y+dy)in seen for _,(dx,dy)in review.DIRECTIONS))
    require('native carpet exit reachable',room_kind((5,7))=='WARP_CARPET_DOWN'and (5,6)in seen)
    return checks,dict(snow_return_steps=sum(len(x['traversed'])for x in snow),western_neck_minimum_grass_steps=best[target][0])


def main():
    parser=argparse.ArgumentParser();parser.add_argument('--pack',type=Path,required=True);parser.add_argument('--output',type=Path,required=True);args=parser.parse_args();out=args.output;out.mkdir(parents=True,exist_ok=False)
    assert SHA(args.pack.read_bytes())==BASE_SHA
    native=load_pack(args.pack);assert module_hash(native['data']['maps']['MnR20C00R00'])==R20_SHA
    files={n:AUTHORING_DIR/n for n in ['MnRainyRiverIcefields.layout.json','MnRainyRiverIcefields.json','winter-art-v01.json','review_rainy_river.py','freeze_rainy_river.py']}
    for name,p in files.items():(out/name).write_bytes(p.read_bytes())
    layout=json.loads(files['MnRainyRiverIcefields.layout.json'].read_text());dossier=json.loads(files['MnRainyRiverIcefields.json'].read_text());spec=json.loads(files['winter-art-v01.json'].read_text())
    assert SHA(files['winter-art-v01.json'].read_bytes())=='4efd8ef08fc06bfc870144014602aca5ca639b34015bade3dddb5ac769aedd74'
    preview,receipts=review.winter.derive(native,spec)
    checks,metrics=certify(preview,layout,dossier)
    print('positive',len(checks),metrics,flush=True)
    negatives=[]
    def append_at(target,point,block):target['layers'].append(dict(kind='point',position=point,block=block,name='negative physical counterexample'))
    mutations={
      'unconnected_outer_ice':lambda l,d:append_at(l,[4,0],'3e'),
      'unconnected_outer_snow':lambda l,d:append_at(l,[30,23],'3c'),
      'lower_west_snowstop_erased':lambda l,d:append_at(l,[18,18],'3e'),
      'lower_east_snowstop_erased':lambda l,d:append_at(l,[27,18],'3e'),
      'cache_ice_gap_dry_bypass':lambda l,d:append_at(l,[9,4],'3c'),
      'snow_return_severed':lambda l,d:append_at(l,[13,18],'3f'),
      'grass_neck_free_bypass':lambda l,d:append_at(l,[9,15],'3c'),
      'incidental_cut_tree':lambda l,d:append_at(l,[12,18],'0f'),
      'occupied_exterior_arrival':lambda l,d:d['residents'][0].__setitem__('tile',[60,44]),
      'unsafe_carpet_return':lambda l,d:append_at(l,[4,14],'3f'),
      'facade_door_removed':lambda l,d:append_at(l,[4,13],'3f'),
      'room_arrival_body_overlap':lambda l,d:d['interiors'][0]['residents'][0].__setitem__('tile',[5,6]),
      'room_return_wrong_passive_index':lambda l,d:d['interiors'][0]['exit_target_warps'].__setitem__(0,1),
    }
    for label,mutate in mutations.items():
        l,d=deepcopy(layout),deepcopy(dossier);mutate(l,d)
        try:certify(preview,l,d)
        except AssertionError as e:negatives.append(dict(id=label,rejection=str(e)));print('rejected',label,str(e),flush=True)
        else:raise AssertionError('accepted corruption '+label)
    r20=native['data']['maps']['MnR20C00R00'];require_port=[]
    for y in [44,45]:
        b=r20['blocks'][y//2*32];token=native['data']['tilesets'][r20['attributes']['tileset_name']]['collision'][f'{b:02x}'][y%2*2]
        assert b==1 and token=='FLOOR';require_port.append(dict(core=[0,y],block=1,collision=token))
    assert not any(c['direction']=='west'for c in r20['attributes']['connections'])
    constant_names=native['data']['trainer_class_names']
    classes={str(x)for x in constant_names}if isinstance(constant_names,dict)else set()
    print('trainer class verification',[(t['trainer_class'],t['trainer_class']in classes)for t in dossier['trainers']],flush=True)
    for t in dossier['trainers']:
        assert t['trainer_class']in classes,('uninstalled trainerclass',t['trainer_class'])
        for p in t['party']:assert p['species']in native['data']['pokemon']and 1<=p['level']<=100
    values=review.geometry(preview,layout,dossier)[0]
    contract=dict(schema_version=1,status='FROZEN_SOURCE_NATIVE_PUBLICATION_PENDING',parent_pack_sha256=BASE_SHA,parent_content_identity=native['identity']['content_hash'],new_maps=[dict(id='MnRainyRiverIcefields',map_constant='MN_RAINY_RIVER_ICEFIELDS',dimensions_metatiles=[32,24],tileset='mn_winter_outdoor',source_layout='planning/minnesota/regions/ice/MnRainyRiverIcefields.layout.json',layout_sha256=SHA(files['MnRainyRiverIcefields.layout.json'].read_bytes()),source_blocks_sha256=SHA(bytes(values)),dossier='planning/minnesota/regions/ice/MnRainyRiverIcefields.json',dossier_sha256=SHA(files['MnRainyRiverIcefields.json'].read_bytes()),native_group_proposal=84,native_id_proposal=1),dict(id='MnRainyRiverWarmingHut',map_constant='MN_RAINY_RIVER_WARMING_HUT',dimensions_metatiles=[5,4],template='VioletNicknameSpeechHouse',native_group_proposal=84,native_id_proposal=2)],art_contract=dict(path='planning/minnesota/regions/ice/winter-art-v01.json',sha256=SHA(files['winter-art-v01.json'].read_bytes()),only_new_alias=True),existing_map_changes=[dict(map='MnR20C00R00',native_module_sha256=R20_SHA,blocks_change=[],all_native_programs_events_actors_warps_attributes_exact_except=['connections','connection_flags'],connections_before=r20['attributes']['connections'],add_connection=dict(direction='west',target_map='MnRainyRiverIcefields',offset=0),edge_core_preimages=require_port)],new_map_connection=dict(direction='east',target_map='MnR20C00R00',offset=0,reverse_offset=0,region_core=[[63,44],[63,45]],target_core=[[0,44],[0,45]]),preserve_all_681_existing_modules_except=['MnR20C00R00 connection vector/flag only'],preserve_existing_global_fields='All old programs/parties/flags/rooms/terrain/art/audio/HOME/recovery/catalogs/encounters unchanged. Bounded new map metadata/landmark binding, new flag namespace, legal native trainer parties/room program rows and newly derived winter alias are additions only.',landmark_parent='MnR20C00R00',no_new_pokegear_landmark_quota=True,unimplemented_links=dossier['unimplemented_links'],regional_species=dossier['future_habitat_reservation'],native_status='NOT_PUBLISHED_OR_CONTROLLER_PROVEN')
    (out/'native-preimage.contract.json').write_text(json.dumps(contract,indent=2)+'\n')
    report=dict(status='SOURCE_PHYSICAL_PASS',checks=checks,metrics=metrics,rejected_negative_variants=negatives,base_sha256=BASE_SHA,native_R20_preimage=R20_SHA,source_hashes={n:SHA(p.read_bytes())for n,p in files.items()},original_maps_exact=preview['data']['maps']==native['data']['maps'],native_playability_claim=False)
    raw=(json.dumps(report,indent=2)+'\n').encode();file=out/('proof-'+SHA(raw)+'.json');file.write_bytes(raw);print(file,flush=True)
    review.PREVIEW_CACHE[(id(native),SHA(json.dumps(spec,sort_keys=True).encode()))]=(preview,receipts,review.SourceArt(preview))
    review.review(native,layout,dossier,spec,out)
    # Same exact source/art/native data, separate retained output: measured warm loop.
    warm=out/'warm-rerender';warm.mkdir();result=review.review(native,layout,dossier,spec,warm)
    assert not result['errors']
    print('warm_source_seconds',result['source_seconds'],flush=True)

if __name__=='__main__':main()
