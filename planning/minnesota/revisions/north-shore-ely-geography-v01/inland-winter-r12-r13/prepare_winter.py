#!/usr/bin/env python3
"""Detached, explicit inland winter terrain; never writes a pack or canonical file.

Requires actual installed cold art. Fresh epochs preserve every source/render.
Geography is inherited exactly; final publication preimages must be rebound only
after Root's west-entry Grand Marais geography has been published.
"""
import argparse
import base64
from collections import Counter
from copy import deepcopy
import hashlib
import heapq
import json
from pathlib import Path
import sys

from PIL import Image, ImageDraw

ROOT = Path(__file__).resolve().parents[5]
sys.path.insert(0, str(ROOT / 'planning/minnesota/tools'))
sys.path.insert(0, str(Path(__file__).resolve().parent))
from audit_outdoor_connections import collision
from render_archive import save_render
from render_reference import layout_blocks, load_pack, room_sheet
from review_inland import whole
from review_major_city import ReviewArt, player_camera, render_map

REGIONS = {
    'MnR12': dict(id='MN_R12', old='mn_gunflint', cold='mn_winter_gunflint',
        remap={0x00:0x40, 0x01:0x3c, 0x04:0x3c, 0x05:0x3f, 0x0e:0x42, 0x10:0x43,
            0x13:0x41, 0x14:0x46, 0x16:0x48, 0x36:0x3c, 0x37:0x3d},
        ice=0x3e, courses=[dict(id='loons-north-rime', metas=[[x,6] for x in range(47,50)],
            traversals=[[[93,13],'right',[100,13]], [[100,12],'left',[93,12]]]),
            dict(id='bog-portage-rime', metas=[[16,y] for y in range(36,39)],
            traversals=[[[33,71],'down',[33,78]], [[32,78],'up',[32,71]]])],
        views=[('warm-west-snowline',[6,56]), ('loons-ice-entry',[93,13]),
            ('loons-ice-stop',[100,12]), ('cut-clearing-approach',[54,24]),
            ('raven-rock-granite',[66,24]), ('bog-ice-entry',[33,71]),
            ('bog-grass-portage',[33,79]), ('southern-ranger-bay',[85,84]),
            ('inland-east-port',[126,56])]),
    'MnR13': dict(id='MN_R13', old='johto', cold='mn_winter_north_shore',
        remap={0x01:0x83, 0x02:0x83, 0x03:0x84, 0x04:0x83, 0x05:0x8b, 0x45:0x88},
        ice=0x85, courses=[dict(id='wind-tor-frost-run',metas=[[23,y] for y in range(4,8)],
            traversals=[[[46,7],'down',[46,16]], [[47,16],'up',[47,7]]])],
        views=[('wind-tor-ice-entry',[46,7]), ('wind-tor-ice-stop',[47,16]),
            ('wind-tor-ledge',[39,23]), ('cedar-shelter-return',[14,54]),
            ('required-ridge-grass',[19,41]), ('long-lake-high-bank',[47,84]),
            ('cut-refuge-approach',[18,102]), ('ely-snow-approach',[34,126])]),
}
DIR = dict(up=(0,-1), down=(0,1), left=(-1,0), right=(1,0))
FOOT = {'FLOOR','TALL_GRASS','ICE','DOOR','CAVE','LADDER'}


def sha(raw): return hashlib.sha256(raw).hexdigest()


def canonical(value):
    return json.dumps(value, sort_keys=True, separators=(',',':'), ensure_ascii=False).encode()


def write(path, value):
    assert not path.exists(), ('preserve every epoch', path)
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(value, indent=2, ensure_ascii=False)+'\n')


def season(region, module):
    w,h=module['attributes']['width'],module['attributes']['height']
    warm=Image.new('1',(w,h))
    if region['id']=='MN_R12':
        # The western BW valley stays warm; snow advances irregularly through
        # a pine-clad saddle, not a straight coast-wide altitude stripe.
        ImageDraw.Draw(warm).polygon([(0,22),(6,22),(8,24),(10,26),(8,30),(6,32),(0,34)], fill=1)
    result=[region['remap'].get(b,b) if not warm.getpixel((i%w,i//w)) else b
        for i,b in enumerate(module['blocks'])]
    for course in region['courses']:
        for x,y in course['metas']:
            assert module['blocks'][y*w+x] in {0x01,0x02,0x71}
            result[y*w+x]=region['ice']
    return result


def sight_cells(pack,prefix):
    bodies=set();sight=set()
    _,_,rows=whole(pack,prefix)
    for row in rows:
        ox,oy=row['origin']
        for actor in pack['data']['maps'][row['map']]['objects']:
            x,y=ox*2+actor['x'],oy*2+actor['y'];bodies.add((x,y))
            if 'TRAINER' not in str(actor.get('object_type','')):continue
            facing=actor['spritemovedata'].rsplit('_',1)[-1].lower()
            if facing not in DIR:continue
            dx,dy=DIR[facing]
            for n in range(1,int(actor.get('radius',0))+1):sight.add((x+dx*n,y+dy*n))
    return bodies,sight


def shortest(kinds,start,target,allowed=FOOT):
    """Static grass-weighted native adjacency, not controller input execution."""
    todo=[(0,0,tuple(start))];distance={tuple(start):(0,0)}
    while todo:
        grass,steps,at=heapq.heappop(todo)
        if distance.get(at)!=(grass,steps):continue
        if at==tuple(target):return [grass,steps]
        for dx,dy in DIR.values():
            nxt=(at[0]+dx,at[1]+dy)
            if kinds.get(nxt)not in allowed:continue
            cost=(grass+(kinds[nxt]=='TALL_GRASS'),steps+1)
            if cost<distance.get(nxt,(10**9,10**9)):
                distance[nxt]=cost;heapq.heappush(todo,(*cost,nxt))
    return None


def physical_proof(base,candidate,prefix,region):
    old,objects,_=whole(base,prefix);new,_,_=whole(candidate,prefix)
    before,after=collision(base,old),collision(candidate,new)
    expected={(x*2+dx,y*2+dy) for c in region['courses'] for x,y in c['metas']
        for dx in range(2) for dy in range(2)}
    changes={p for p in before if before[p]!=after[p]}
    assert changes==expected
    assert all(before[p]=='FLOOR' and after[p]=='ICE' for p in expected)
    bodies,sights=sight_cells(base,prefix)
    assert not expected.intersection(bodies|sights), ('ice crosses actor or sight',prefix)
    course_proof=[];all_entries=[]
    for course in region['courses']:
        for start,button,stop in course['traversals']:
            start,stop=tuple(start),tuple(stop);dx,dy=DIR[button]
            assert before[start]==after[start]=='FLOOR' and before[stop]==after[stop]=='FLOOR'
            at=(start[0]+dx,start[1]+dy);passed=[]
            while after.get(at)=='ICE':
                assert at not in bodies|sights
                passed.append(at);at=(at[0]+dx,at[1]+dy)
            assert at==stop and passed and stop not in bodies|sights
            course_proof.append(dict(id=course['id'],from_core=start,button=button,
                to_core=stop,ice_core=passed,released_frames_after_arrival=96))
        cells={(x*2+dx,y*2+dy)for x,y in course['metas']for dx in range(2)for dy in range(2)}
        for x,y in sorted(cells):
            for button,(dx,dy)in DIR.items():
                start=(x-dx,y-dy)
                if start in cells or after.get(start)not in FOOT:continue
                at=(x,y);passed=[]
                while at in cells:
                    passed.append(at);at=(at[0]+dx,at[1]+dy)
                wall=after.get(at)not in FOOT
                stop=passed[-1]if wall else at
                assert start not in bodies|sights and stop not in bodies|sights
                assert not wall or after[at]=='WALL', 'no forced ledge, gate or water landing'
                # At a native intentional wall stop, a perpendicular/reverse
                # input must still have at least one ordinary Floor exit.
                exits=[]
                if wall:
                    for turn,(tx,ty)in DIR.items():
                        pos=(stop[0]+tx,stop[1]+ty);path=[]
                        while pos in cells:
                            path.append(pos);pos=(pos[0]+tx,pos[1]+ty)
                        if after.get(pos)=='FLOOR'and pos not in bodies|sights:
                            exits.append(dict(button=turn,ice_core=path,dry_stop_core=pos))
                    assert exits
                all_entries.append(dict(id=course['id'],from_core=start,button=button,
                    to_core=stop,ice_core=passed,stop_kind='intentional_wall_stop_on_ICE'if wall else'ordinary_FLOOR',
                    ordinary_turn_exits=exits))
    # Compare ordinary Foot adjacency treating ICE as traversable, not as a
    # claim that arbitrary mid-slide stops are legal controller states.
    assert {p for p,k in before.items() if k in FOOT}=={p for p,k in after.items() if k in FOOT}
    assert {p:k for p,k in before.items() if p not in expected}=={p:k for p,k in after.items() if p not in expected}
    ports=[];w,h=new['attributes']['width']*2,new['attributes']['height']*2
    for p in after:
        if p[0] in {0,w-1}or p[1] in {0,h-1}:
            assert before[p]==after[p]
            if after[p]in FOOT:ports.append(p)
    # Every native water, grass, ledge, Cut/Headbutt tree, doorway and cave
    # quadrant is retained. No snow donor introduces incidental Cut collision.
    preserved=Counter(k for p,k in before.items()if k not in {'FLOOR','WALL'})
    assert all(before[p]==after[p]for p in before if before[p]not in {'FLOOR','WALL'})
    endpoints=([(0,56),(127,56)]if prefix=='MnR12'else[(30,0),(34,127)])
    minimum=4 if prefix=='MnR12'else 6
    old_grass=shortest(before,*endpoints);new_grass=shortest(after,*endpoints)
    assert old_grass==new_grass and new_grass[0]==minimum
    gate=([54,23]if prefix=='MnR12'else[18,103])
    approach=([54,24]if prefix=='MnR12'else[18,102])
    targets=([[54,19]]if prefix=='MnR12'else[[17,106],[20,109]])
    replacement=(0x17 if prefix=='MnR12'else 0x3f)
    opened=dict(after);gx,gy=gate[0]//2,gate[1]//2
    repl=candidate['data']['tilesets'][region['cold']]['collision'][f'{replacement:02x}']
    for dy in range(2):
        for dx in range(2):opened[(gx*2+dx,gy*2+dy)]=repl[dy*2+dx]
    cut=[]
    for target in targets:
        closed=shortest(after,approach,target,FOOT|{'WATER','WATERFALL'})
        open_path=shortest(opened,approach,target)
        assert closed is None and open_path is not None
        cut.append(dict(target=target,closed_even_with_water_adjacency_unreachable=True,open_native_replacement_path=open_path))
    ledge=None
    if prefix=='MnR13':
        assert after[(39,24)]=='HOP_DOWN'and after[(39,25)]=='WALL'
        ledge=shortest(after,[39,26],[39,23]);assert ledge[1]>=19
    return dict(status='STATIC_PROPOSED_DATA_NOT_NATIVE_INPUT',collision_changes={'FLOOR->ICE':len(changes)},
        exact_outer_ports=ports,preserved_special_quadrants=dict(preserved),
        native_actor_count=len(objects),all_actor_bodies_and_sights_off_ice=True,
        all_other_quadrants_exact=True,ordinary_foot_adjacency_exact_if_ice_treated_as_traversable=True,
        bounded_slide_courses=course_proof,all_boundary_entry_courses=all_entries,
        main_foot_minimum_grass_steps=new_grass[0],main_foot_minimum_grass_path_length=new_grass[1],
        unchanged_cut_isolation=cut,retained_wind_tor_ordinary_return=ledge,
        limit='Courses use intended native straight-slide semantics, not an executed controller. Adjacency minimum treats ICE as traversable and does not permit arbitrary native mid-slide stops. No earned progression, badge denial, field move interaction or campaign acceptance claimed.')


def main():
    ap=argparse.ArgumentParser(description=__doc__)
    ap.add_argument('--pack',type=Path,required=True);ap.add_argument('--expected-sha256',required=True)
    ap.add_argument('--source-output',type=Path,required=True);ap.add_argument('--render-output',type=Path,required=True)
    args=ap.parse_args();assert sha(args.pack.read_bytes())==args.expected_sha256
    assert not args.source_output.exists() and not args.render_output.exists()
    base=load_pack(args.pack);candidate=dict(base,data=dict(base['data'],maps=dict(base['data']['maps']),
        map_attributes=dict(base['data']['map_attributes']),map_blocks=dict(base['data']['map_blocks'])))
    for region in REGIONS.values():
        assert region['cold']in base['data']['tilesets'], 'actual art alias publication required'
    assert base['data']['tilesets']['mn_winter_north_shore']['collision']['8b']==['WALL']*4
    args.source_output.mkdir(parents=True);args.render_output.mkdir(parents=True)
    rows=[];sources={};candidate_files={};proof={}
    for prefix,region in REGIONS.items():
        original,objects,placements=whole(base,prefix);blocks=season(region,original)
        assert original['attributes']['tileset_name']==region['old']
        w,h=original['attributes']['width'],original['attributes']['height']
        lp=ROOT/f"planning/minnesota/routes/{region['id']}.layout.json"
        dp=lp.with_name(region['id']+'.json')
        layout=json.loads(lp.read_bytes());dossier=json.loads(dp.read_bytes())
        for path in [lp,dp]:
            sources[str(path.relative_to(ROOT))]=sha(path.read_bytes())
            (args.source_output/(path.name+'.before')).write_bytes(path.read_bytes())
        # Compare the unfitted original strokes to the actual native grid.
        # Explicit source edges disable any second fitting pass after overrides.
        authored=layout_blocks(dict(layout,shore_fitting='explicit_source_edges'))
        assert len(authored)==w*h
        fitted=[];seasonal=[]
        for i,(raw,old,new) in enumerate(zip(authored,original['blocks'],blocks)):
            if raw!=old:fitted.append(dict(kind='point',name='Preserve exact installed native fitted terrain before inland winter',position=[i%w,i//w],block=f'{old:02x}'))
            if old!=new:seasonal.append(dict(kind='point',name='Deliberate inland winter snowline, icy grass and bounded frost courses',position=[i%w,i//w],block=f'{new:02x}'))
        layout['layers']+=fitted+seasonal
        layout['compiled_tileset']=region['cold'];layout['shore_fitting']='explicit_source_edges'
        review=dict(status='DETACHED_PRE_GEOGRAPHY_PROPOSAL_NOT_APPLIED',
            instruction='Warm Superior coast and Grand Marais; substantial winter inland near Ely.',
            baseline_pack_sha256=args.expected_sha256,full_region_metatiles=[w,h],
            unchanged='Native water, cliffs, gates, actors, scripts, story, rooms, ledges and ports. Ice alters only listed original Floor quadrants.',
            authored_strokes_retained=len(layout['layers'])-len(fitted)-len(seasonal),
            native_reconciliation_points=len(fitted),seasonal_points=len(seasonal),
            pending='Root final west-entry geography binding, native winter publisher, controller, GPU and joint guarded source promotion.')
        layout['inland_winter_review']=review
        dossier['compiled_tileset']=region['cold'];dossier['inland_winter_review']=review
        assert layout_blocks(layout)==blocks, 'explicit fitted source must not be fitted a second time'
        for name,value in [(region['id']+'.winter.candidate.layout.json',layout),(region['id']+'.winter.candidate.json',dossier)]:
            write(args.source_output/name,value);candidate_files[name]=sha((args.source_output/name).read_bytes())
        for place in placements:
            name=place['map'];m=base['data']['maps'][name];new=deepcopy(m)
            ox,oy=place['origin'];cw,ch=place['dimensions'];cells=[]
            for y in range(ch):
                for x in range(cw):
                    i=y*cw+x;b=blocks[(oy+y)*w+ox+x]
                    if new['blocks'][i]!=b:
                        old=new['blocks'][i];cells.append(dict(metatile=[x,y],source_metatile=[ox+x,oy+y],before=old,after=b,
                            before_quadrants=base['data']['tilesets'][region['old']]['collision'][f'{old:02x}'],
                            after_quadrants=base['data']['tilesets'][region['cold']]['collision'][f'{b:02x}']))
                    new['blocks'][i]=b
            new['attributes']['tileset_name']=region['cold'];label=m['attributes']['blocks_label']
            assert base['data']['map_attributes'][name]==m['attributes']
            assert base64.b64decode(base['data']['map_blocks'][label])==bytes(m['blocks'])
            candidate['data']['maps'][name]=new;candidate['data']['map_attributes'][name]=new['attributes']
            candidate['data']['map_blocks'][label]=base64.b64encode(bytes(new['blocks'])).decode()
            rows.append(dict(map=name,dimensions_metatiles=[cw,ch],module_sha256=sha(canonical(m)),
                candidate_module_sha256=sha(canonical(new)),old_tileset=region['old'],new_tileset=region['cold'],
                blocks_label=label,old_blocks_sha256=sha(bytes(m['blocks'])),new_blocks_sha256=sha(bytes(new['blocks'])),cells=cells))
        proof[prefix]=physical_proof(base,candidate,prefix,region)
    # Exact whole-pack inverse of only nine modules, their block/attr mirrors.
    inverse=dict(candidate,data=dict(candidate['data'],maps=dict(candidate['data']['maps']),
        map_attributes=dict(candidate['data']['map_attributes']),map_blocks=dict(candidate['data']['map_blocks'])))
    for row in rows:
        name=row['map'];inverse['data']['maps'][name]=base['data']['maps'][name]
        inverse['data']['map_attributes'][name]=base['data']['map_attributes'][name]
        inverse['data']['map_blocks'][row['blocks_label']]=base['data']['map_blocks'][row['blocks_label']]
    assert inverse==base
    contact=[];cameras=[]
    for prefix,region in REGIONS.items():
        images=[];paired={}
        for state,pack in [('before',base),('candidate',candidate)]:
            m,objects,_=whole(pack,prefix);review=ReviewArt(pack)
            ground,scene,_=render_map(review,m,objects,'day',args.render_output/prefix/state)
            factor=min(1,780/scene.width,1250/scene.height)
            images.append((prefix+' '+state,scene.resize((round(scene.width*factor),round(scene.height*factor)),Image.Resampling.NEAREST)))
            for label,at in region['views']:
                assert collision(pack,m)[tuple(at)]in FOOT, ('camera not Foot',prefix,label,at)
                frame,_=player_camera(review,m,objects,ground,scene,'day',dict(core=at),args.render_output/prefix/state/'cameras'/label)
                paired[(label,state)]=frame.resize((480,432),Image.Resampling.NEAREST)
        contact.extend(images)
        region_cameras=[]
        for label,_ in region['views']:
            for state in ['before','candidate']:
                region_cameras.append((prefix+' '+label+' '+state,paired[(label,state)]))
        cameras.extend(region_cameras)
        for first in range(0,len(region_cameras),8):
            save_render(room_sheet(region_cameras[first:first+8],2),args.render_output/prefix/f'player-comparison-{first//8+1:02}.png')
    save_render(room_sheet(contact,2),args.render_output/'whole-inland-before-candidate.png')
    save_render(room_sheet(cameras,2),args.render_output/'all-player-cameras-before-candidate.png')
    contract=dict(schema_version=1,id='MN_INLAND_R12_R13_WINTER',status='PRE_GEOGRAPHY_REVIEW_ONLY_NOT_FROZEN_FOR_PUBLICATION',
        source_preimage_pack_sha256=args.expected_sha256,source_preimage_identity=base['identity']['content_hash'],
        source_files_sha256=sources,candidates_sha256=candidate_files,modules=rows,
        changed_metatiles=sum(len(r['cells'])for r in rows),changed_modules=len(rows),
        flag_changes=[],program_changes=[],art_changes=[],connection_changes=[],
        publication_binding='Rebind exact full modules after Root publishes final west-entry coast geography; block preimages must remain exact.',
        tools_sha256={Path(__file__).name:sha(Path(__file__).read_bytes())})
    write(args.source_output/'native.review.contract.json',contract)
    write(args.render_output/'physical-proof.json',dict(regions=proof,whole_parent_inverse_exact=True,
        contract_sha256=sha((args.source_output/'native.review.contract.json').read_bytes())))
    (args.render_output/'prepare_winter.snapshot.py').write_bytes(Path(__file__).read_bytes())
    print(json.dumps(dict(changed=contract['changed_metatiles'],modules=len(rows),
        contract=args.source_output/'native.review.contract.json'),default=str))


if __name__=='__main__':main()
