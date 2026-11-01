#!/usr/bin/env python3
"""Whole hand-authored St Cloud source review; never export or publish game data."""
import argparse
from collections import deque
from copy import deepcopy
import hashlib
import json
from pathlib import Path
import time

from PIL import Image, ImageDraw
from render_archive import save_render
from render_reference import SourceArt, layout_blocks, load_pack
from render_stages import render_stage
from review_north_shore import oriented_sprite

ROOT=Path(__file__).resolve().parents[3]
DESIGN=ROOT/'planning/minnesota/towns/MnStCloud.json'
LAYOUT=ROOT/'planning/minnesota/towns/MnStCloud.layout.json'
STATION=ROOT/'planning/minnesota/transit/MnStCloudStation.json'
OUTPUT=ROOT/'planning/minnesota/renders/st-cloud'

def adjacent(p):
    x,y=p
    return [(x-1,y),(x+1,y),(x,y-1),(x,y+1)]

def flood(start,cells,doors=frozenset()):
    seen={start};todo=deque([start])
    while todo:
        for q in adjacent(todo.popleft()):
            if q in cells and q not in seen:
                seen.add(q)
                if q not in doors:todo.append(q)
    return seen

def sampler(pack,ts,w,blocks):
    collision=pack['data']['tilesets'][ts]['collision']
    def kind(p):
        x,y=p
        return collision[f'{blocks[y//2*w+x//2]:02x}'][y%2*2+x%2]
    return kind

def text_sprite(pack,a):
    assert f'gfx/sprites/{a["sprite"].removeprefix("SPRITE_").lower()}.png' in pack['runtime_files'],'missing native sprite '+a['sprite']
    assert all(0<len(t)<=18 for t in a['lines']),a['id']+' text exceeds18 columns'

def validate_town(d,l,pack):
    assert d['id']==l['id']=='MnStCloud' and d['dimensions_metatiles']==l['dimensions_metatiles']==[24,18]
    assert l['schema_version']==2 and l['reject_cliff_cuts'] is True
    assert l['compiled_tileset']=='johto_modern' and not l.get('derived_blocks'),'Unchanged actual native art only'
    for layer in l['layers']:
        assert layer['name']
        if layer['kind'] in {'polygon','path','cliff_polygon'}:
            ps=layer['points'];ends=ps[1:] if layer['kind']=='path' else ps[1:]+ps[:1]
            assert all(a[0]==b[0] or a[1]==b[1]for a,b in zip(ps,ends)),'orthogonal authored strokes'
    b=layout_blocks(l);w,h=24,18;k=sampler(pack,l['compiled_tileset'],w,b)
    cells={(x,y)for y in range(h*2)for x in range(w*2)}
    actors={tuple(a['tile'])for a in d['residents']};doors={tuple(a['tile'])for a in d['doors']}
    assert len(actors)==len(d['residents']) and not actors&doors,'overlapping bodies/doors'
    walk={p for p in cells if k(p) in {'FLOOR','TALL_GRASS','DOOR'}}
    water={p for p in cells if k(p)=='WATER'}
    seen=flood((24,32),walk-actors,doors);surf=flood((24,32),(walk|water)-actors,doors)
    for a in d['residents']:
        text_sprite(pack,a);q=tuple(a['tile']);assert k(q)=='FLOOR',a['id']+' off FLOOR '+k(q)
        assert set(adjacent(q))&seen,a['id']+' inaccessible'
    for s in d['signs']:
        assert k(tuple(s['tile']))=='WALL',s['id']+' sign off actual WALL'
        assert set(adjacent(tuple(s['tile'])))&seen,s['id']+' sign inaccessible'
        assert all(0<len(t)<=18 for t in s['lines']),s['id']+' sign exceeds18 columns'
    assert doors=={p for p in cells if k(p)=='DOOR'},'false/unbound facade door'
    indices=[]
    for door in d['doors']:
        q=tuple(door['tile']);r=tuple(door['return_tile']);assert q in seen and r in seen,'door unreachable'
        assert k(q)=='DOOR' and k(r)=='FLOOR','unsafe native exterior door/return'
        assert r==(q[0],q[1]+1) and door['arrival_tile']==list(r),'door passive Floor geometry'
        assert door['target_warp']==3,'room entry must target passive Floor3'
        indices.extend([door['warp_index'],door['return_warp_index']])
    assert indices==list(range(1,9)),'exact exterior active/passive index order'
    expected=[('MN_R05','S',[12,16],[12,14]),('MN_R06','N',[12,1],[12,3])]
    assert len(d['ports'])==2
    for port,values in zip(d['ports'],expected):
        assert (port['route'],port['side'],port['gate_metatile'],port['landing_metatile'])==values,'reserved port changed'
        assert all(tuple(2*v for v in port[key])in seen for key in ['gate_metatile','landing_metatile']),'port disconnected'
    outer={p for p in cells if p[0]in{0,47} or p[1]in{0,35}}
    assert seen&outer=={(24,0),(25,0),(24,35),(25,35)},('unplanned walking edge',sorted(seen&outer))
    assert not water&outer,'outer river WATER escape requires native water rocks'
    assert not any(k(p)=='CUT_TREE'for p in cells),'incidental Cut quadrant'
    river=d['river_contract'];bridge={(x,y)for x in river['bridge_walk_core_x']for y in range(16,22)}
    assert all(k(q)=='FLOOR'for q in bridge),'bridge lane interrupted'
    assert all(k((22,y))==k((25,y))=='WALL'for y in range(16,22)),'bridge side rail gap'
    dry_without=flood((24,32),walk-actors-bridge,doors)
    assert (24,2)not in dry_without,'Mississippi bridge has a free dry bypass'
    assert tuple(river['launch_tile'])in seen and k(tuple(river['launch_tile']))=='FLOOR' and k(tuple(river['water_target']))=='WATER','Surf launch blocked'
    for x,y in river['outer_rock_blocks']:
        edge_x=x*2 if x==0 else x*2+1
        assert k((edge_x,y*2))==k((edge_x,y*2+1))=='BUOY','outer river closure is not native BUOY'
    ramp=river['overlook_ramp_metatile'];assert b[ramp[1]*24+ramp[0]]==0x74,'native74 ramp missing'
    assert tuple(river['overlook_core'])in seen,'overlook disconnected'
    rb=list(b);rb[ramp[1]*24+ramp[0]]=0x72;rk=sampler(pack,l['compiled_tileset'],24,rb)
    closed={p for p in cells if rk(p)in{'FLOOR','TALL_GRASS','DOOR'}}
    assert tuple(river['overlook_core'])not in flood((24,32),closed-actors,doors),'overlook ramp has an unintended free bypass'
    vx,vy=river['overlook_core'];visible=sum(k((x,y))=='WATER'for y in range(vy-4,vy+5)for x in range(vx-5,vx+5))
    assert visible>=15,'river overlook must show water at actual player scale'
    for v in d['viewpoints']:assert tuple(v['tile'])in seen,v['id']+' camera not on public route'
    gym=d['gym_contract'];assert gym['requires']==['BADGE_4'] and gym['win_only']==['BADGE_5','EVENT_MN_GYM_5_DEFEATED'] and gym['native_badge_index']==4,'gym story/badge authority'
    return dict(source_only=True,reachable_walk=len(seen),reachable_water=len(surf&water),closed_outer_water=True,required_bridge=True,bridge_walk_width_core=2,overlook_native_ramp=True,overlook_water_in160_frame=visible,doors=len(doors),native_metatile_varieties=len(set(b)),native_services_installed=False,native_train_installed=False),k,seen,b

CANONICAL=ROOT/'planning/minnesota/gyms/interiors.json'
GYM_BEFORE=OUTPUT/'canonical-gym-reconciliation/before-57cfb19f'

def canonical_gym(room,document=None):
    ref=room['source_reference']
    assert ref['path']=='planning/minnesota/gyms/interiors.json' and ref['gym_id']=='MnStCloudGym','canonical Gym5 source authority changed'
    assert not {'rows','residents','party','art','gates','stations','reward'}&room.keys(),'competing city-owned Gym5 source'
    data=json.loads(CANONICAL.read_text())if document is None else document
    gym=next(g for g in data['gyms']if g['id']==ref['gym_id'])
    digest=hashlib.sha256(json.dumps(gym,sort_keys=True,separators=(',',':')).encode()).hexdigest()
    assert digest==ref['room_sha256'],'canonical Gym5 object digest changed'
    needed=set(''.join(gym['rows']))|{g['open_symbol']for g in gym['gates']}|{'#'}
    # Exact gym_art::install BTreeMap character iteration, including separate
    # symbols with identical art recipes. Unused recipes get no native ID.
    symbols={s:f'{i:02x}'for i,s in enumerate(sorted(needed))}
    assert ref['symbol_blocks']==symbols,'canonical Gym5 native glyph allocation changed'
    assert room['tileset']=='mn_gym_5' and room['native_constant']==gym['constant'] and room['dimensions_metatiles']==gym['dimensions_metatiles'],'canonical Gym5 native identity changed'
    assert room['clear_arrival']==gym['arrival_core'] and room['passive_arrival']==dict(index=3,tile=gym['arrival_core'],collision='FLOOR'),'canonical Gym5 safe Floor3 changed'
    assert room['exits']==[dict(index=i+1,tile=at,target='MnStCloud',target_warp=6,collision='WARP_CARPET_DOWN')for i,at in enumerate(gym['exit_tiles_core'])],'canonical Gym5 city-return binding changed'
    return gym,symbols,digest

def canonical_actors(gym):
    return [dict(id='MnStCloudGymGuide',tile=gym['guide_at'],sprite='SPRITE_GYM_GUIDE',facing='south'),dict(id='MnStCloudGymLeader',tile=gym['leader_at'],sprite=gym['leader_sprite'],facing='south')]+[dict(id=t['id'],tile=t['at'],sprite=t['sprite'],facing='south')for t in gym['trainers']]

def canonical_phases(room,pack,document=None):
    gym,symbols,digest=canonical_gym(room,document);rows=[list(row)for row in gym['rows']];actors=canonical_actors(gym);occupied={tuple(a['tile'])for a in actors};events=set();states=[]
    assert len(occupied)==len(actors),'overlapping canonical Gym5 actors'
    assert gym['slot']==5 and gym['requires']==['BADGE_4'] and gym['badge_flag']=='ENGINE_STORMBADGE' and gym['trainer_class']=='BIRD_KEEPER' and gym['reward']=='TM_SWIFT','canonical Gym5 badge/leader/reward contract'
    assert gym['reward']in pack['data']['items'],'canonical reward unavailable in native pack'
    for actor in actors:
        assert f'gfx/sprites/{actor["sprite"].removeprefix("SPRITE_").lower()}.png'in pack['runtime_files'],'missing canonical actor sprite'
    for party in [gym['party']]+[t['party']for t in gym['trainers']]:
        for member in party:
            assert member['species']in pack['data']['pokemon'] and 1<=member['level']<=100,'canonical native party member'
            assert len(member.get('moves',[]))<=4 and all(move in pack['data']['moves']for move in member.get('moves',[])),'canonical native moves'
    assert len(gym['trainers'])==2 and not gym['stations'] and all(not t['optional']for t in gym['trainers']),'canonical two required trainer/wind puzzle'
    assert gym['leader_court_locked'] and len(gym['gates'])==2,'canonical staged crossing gates'
    width,height=gym['dimensions_metatiles']
    def kind(p):
        x,y=p;symbol=rows[y//2][x//2];source,block=gym['art'][symbol]
        return gym.get('winds',{}).get(symbol,pack['data']['tilesets'][source]['collision'][block][y%2*2+x%2])
    def reached():
        start=tuple(gym['arrival_core']);seen={start};queue=[start]
        for p in queue:
            for q in adjacent(p):
                if not(0<=q[0]<width*2 and 0<=q[1]<height*2) or q in occupied:continue
                if kind(q)not in {'FLOOR','WARP_CARPET_DOWN','WALK_RIGHT','WALK_LEFT'}:continue
                visited=set()
                while kind(q)in {'WALK_RIGHT','WALK_LEFT'}:
                    assert q not in visited,'canonical wind loop';visited.add(q)
                    q=(q[0]+(1 if kind(q)=='WALK_RIGHT'else -1),q[1])
                    assert 0<=q[0]<width*2 and q not in occupied,'canonical unsafe wind'
                if visited:assert kind(q)=='FLOOR','canonical unsafe wind landing'
                if q not in seen:seen.add(q);queue.append(q)
        return seen
    def record(stage):
        seen=reached();states.append(dict(stage=stage,events=sorted(events),rows=[''.join(r)for r in rows],blocks=[int(symbols[s],16)for row in rows for s in row],reachable=sorted(seen),source_fixture_only=True));return seen
    closed=record('ARRIVAL_CLOSED');leader_neighbors=set(adjacent(tuple(gym['leader_at'])))
    assert not leader_neighbors&closed,'closed canonical Gym5 court bypass'
    assert kind(tuple(gym['arrival_core']))=='FLOOR' and set(map(tuple,gym['exit_tiles_core']))<=closed and all(kind(tuple(p))=='WARP_CARPET_DOWN'for p in gym['exit_tiles_core']),'canonical arrival/carpets'
    for step in gym['walkthrough']:
        trainer=next((t for t in gym['trainers']if t['id']==step),None);actor=trainer or next(s for s in gym['stations']if s['id']==step)
        assert set(actor.get('requires',[]))<=events,'canonical walkthrough dependency '+step
        assert set(adjacent(tuple(actor['at'])))&reached(),'canonical walkthrough inaccessible '+step
        events.add('EVENT_'+step+'_DEFEATED'if trainer else actor['event'])
        for gate in gym['gates']:
            if set(gate['requires'])<=events:
                x,y=gate['at'];rows[y][x]=gate['open_symbol'];events.add(gate['event'])
        seen=record(step)
        if step!=gym['walkthrough'][-1]:assert not leader_neighbors&seen,'court opened before both required trainers'
    opened=reached();assert leader_neighbors&opened,'canonical open court unreachable'
    for actor in actors:assert kind(tuple(actor['tile']))=='FLOOR'and set(adjacent(tuple(actor['tile'])))&opened,'canonical actor footing/approach '+actor['id']
    assert 'EVENT_MN_GYM_5_COURT_OPEN'in events and {(x,y-1)for x,y in gym['exit_tiles_core']}<=opened,'canonical court event/ordinary return'
    for v in room['viewpoints']:
        state=next(s for s in states if s['stage']==v['phase']);assert tuple(v['tile'])in map(tuple,state['reachable']),'canonical camera must be actual playable Floor in declared puzzle phase'
    return gym,symbols,digest,states

def canonical_report(room,pack):
    gym,symbols,digest,states=canonical_phases(room,pack)
    return dict(id=room['id'],dimensions=gym['dimensions_metatiles'],canonical_room_sha256=digest,symbol_blocks=symbols,canonical_source=room['source_reference']['path'],canonical_whole_file_sha256_informational=hashlib.sha256(CANONICAL.read_bytes()).hexdigest(),recorded_whole_file_hash_matches=room['source_reference']['file_sha256_informational']==hashlib.sha256(CANONICAL.read_bytes()).hexdigest(),phase_reachability=[dict(stage=s['stage'],public_tiles=len(s['reachable']),events=s['events'])for s in states],party=gym['party'],reward=gym['reward'],native_city_rebinding_pending=True,native_gameplay_proved=False)

def unchanged_non_gym(d):
    before=json.loads((GYM_BEFORE/'MnStCloud.json').read_text());current=deepcopy(d)
    before['interiors']=[r for r in before['interiors']if r['id']!='MnStCloudGym'];current['interiors']=[r for r in current['interiors']if r['id']!='MnStCloudGym']
    assert before==current,'non-Gym5 city fields changed during reconciliation'
    assert LAYOUT.read_bytes()==(GYM_BEFORE/'MnStCloud.layout.json').read_bytes(),'exterior source changed during Gym5 reconciliation'
    assert STATION.read_bytes()==(GYM_BEFORE/'MnStCloudStation.json').read_bytes(),'station source changed during Gym5 reconciliation'


def validate_rooms(d,s,pack):
    rooms=[s if r['id']==s['id']else r for r in d['interiors']];reports=[]
    assert [(r['id'],r['dimensions_metatiles'])for r in rooms]==[(n,dim)for n,dim in [('MnStCloudPokecenter1F',[10,8]),('MnStCloudMart',[6,4]),('MnStCloudGym',canonical_gym(next(r for r in d['interiors']if r['id']=='MnStCloudGym'))[0]['dimensions_metatiles']),('MnStCloudStation',[10,10])]],'retained exact service dimensions'
    for r in rooms:
        if 'source_reference'in r:
            gym=canonical_gym(r)[0]
            assert d['gym_contract']['party']==[{k:m[k]for k in ['species','level']}for m in gym['party']],'legacy city gym summary differs from canonical party'
            reports.append(canonical_report(r,pack));continue
        w,h=r['dimensions_metatiles'];rows=[row.split()for row in r['rows']];assert len(rows)==h and all(len(row)==w for row in rows),r['id']+' rows'
        blocks=[int(b,16)for row in rows for b in row];k=sampler(pack,r['tileset'],w,blocks);cells={(x,y)for y in range(h*2)for x in range(w*2)}
        actors={tuple(a['tile'])for a in r['residents']};assert len(actors)==len(r['residents'])
        walk={p for p in cells if k(p)in{'FLOOR','DOOR','WARP_CARPET_DOWN'}}
        arrival=tuple(r['passive_arrival']['tile']);assert r['passive_arrival']['index']==3 and k(arrival)=='FLOOR' and arrival not in actors,r['id']+' unsafe passive Floor3'
        seen=flood(arrival,walk-actors);ex={tuple(e['tile'])for e in r['exits']}
        assert ex=={p for p in cells if k(p)=='WARP_CARPET_DOWN'} and len(ex)==2 and ex<=seen,r['id']+' carpet mismatch/unreachable'
        door=next(x for x in d['doors']if x['target']==r['id']);assert all(e['target']=='MnStCloud'and e['target_warp']==door['return_warp_index']for e in r['exits']),'wrong native room return'
        for a in r['residents']:
            text_sprite(pack,a);assert k(tuple(a['tile']))=='FLOOR',a['id']+' actor off FLOOR'
            if 'counter_tile'in a:assert k(tuple(a['counter_tile']))=='COUNTER'and tuple(a['approach'])in seen,a['id']+' counter service unreachable'
            else:assert set(adjacent(tuple(a['tile'])))&seen,a['id']+' inaccessible'
        for sign in r.get('signs',[]):
            assert k(tuple(sign['tile']))=='WALL' and tuple(sign['approach'])in seen,'room BG sign/cabinet approach'
            assert all(0<len(t)<=18 for t in sign['lines'])
        if 'pc'in r:assert k(tuple(r['pc']['tile']))=='PC' and tuple(r['pc']['approach'])in seen,'native PC inaccessible'
        assert not any(k(p)in{'PIT','LADDER','STAIRCASE'}for p in cells),'unbound room floor trigger'
        outer={p for p in cells if p[0]in{0,w*2-1}or p[1]in{0,h*2-1}}
        assert seen&outer==ex,r['id']+' unexpected room perimeter opening'
        for v in r.get('viewpoints',[]):
            q=tuple(v.get('tile',v.get('core')));assert q in seen and k(q)=='FLOOR','room camera must be actual playable Floor'
        if 'boarding'not in r:assert not any(k(p)=='DOOR'for p in cells),'unbound ordinary room DOOR'
        if 'boarding'in r:
            b=r['boarding'];con=tuple(b['conductor_core']);train={tuple(b[key])for key in ['train_door_core','other_train_door_core']}
            assert train=={p for p in cells if k(p)=='DOOR'} and not train&seen,'unsealed train door'
            assert con in actors and tuple(b['approach_core'])in seen and tuple(b['ride_arrival_core'])in seen and k(tuple(b['ride_arrival_core']))=='FLOOR','unsafe conductor/arrival'
            opened=flood(arrival,walk-(actors-{con}));assert train<=opened and tuple(b['platform_core'])in opened,'single opened throat must reach train'
            delta={'up':(0,-1),'down':(0,1),'left':(-1,0),'right':(1,0)}
            def movement(at,steps,blocked):
                for cmd in steps:
                    dx,dy=delta[cmd];at=(at[0]+dx,at[1]+dy)
                    assert at in walk and at not in blocked,'boarding/reset movement leaves native free floor'
                return at
            moved=movement(con,b['conductor_open_steps'],actors-{con});player=movement(tuple(b['approach_core']),b['player_board_steps'],(actors-{con})|{moved});assert player==tuple(b['train_door_core'])
            assert movement(moved,b['conductor_restore_steps'],(actors-{con})|{player})==con,'conductor reset does not close original throat'
            network=json.loads((ROOT/'planning/minnesota/transit/glacier-line.json').read_text());stop=next(x for x in network['ordered_stops']if x['id']=='ST_CLOUD')
            assert s['id']==stop['station'] and s['native_constant']==stop['constant'] and s['native_script_binding']['entry_callback']==stop['visited_flag'] and s['native_script_binding']['service_requires']=='EVENT_MN_COURIER_DEFEATED','station network contract'
        if r['kind']=='gym_5':
            leader=next(a for a in r['residents']if a.get('battle')=='MN_GYM_5');assert leader['party']==d['gym_contract']['party']and leader['requires']==['BADGE_4']and leader['native_badge_index']==4,'source leader progression'
        reports.append(dict(id=r['id'],public_tiles=len(seen),dimensions=[w,h],passive_arrival=list(arrival),carpets=sorted(ex)))
    return rooms,reports

def negative_checks(d,l,s,pack):
    cases=[]
    def reject(name,mutate):
        dd,ll,ss=deepcopy(d),deepcopy(l),deepcopy(s);mutate(dd,ll,ss)
        try:validate_town(dd,ll,pack);validate_rooms(dd,ss,pack)
        except(AssertionError,KeyError,IndexError):cases.append(name);return
        raise AssertionError('negative accepted: '+name)
    def block(ll,at,b):ll['layers'].append(dict(name='NEGATIVE only',kind='point',position=at,block=b))
    reject('wrong facade DOOR',lambda d,l,s:d['doors'][0].update(tile=[7,9]))
    reject('unsafe exterior passive',lambda d,l,s:d['doors'][1].update(return_tile=[14,9]))
    reject('reserved south port changed',lambda d,l,s:d['ports'][0].update(gate_metatile=[11,16]))
    reject('open outer river',lambda d,l,s:block(l,[23,9],'35'))
    reject('missing bridge lane',lambda d,l,s:block(l,[12,9],'35'))
    reject('bridge rail gap',lambda d,l,s:block(l,[11,9],'01'))
    reject('missing native overlook ramp',lambda d,l,s:block(l,[17,14],'72'))
    reject('decorative Cut tree',lambda d,l,s:block(l,[10,6],'5b'))
    reject('room arrival on active carpet',lambda d,l,s:d['interiors'][0]['passive_arrival'].update(tile=[10,15]))
    reject('room returns to active exterior door',lambda d,l,s:d['interiors'][1]['exits'][0].update(target_warp=3))
    reject('PC approach blocked',lambda d,l,s:d['interiors'][0]['pc'].update(approach=[0,0]))
    reject('legacy party summary drift',lambda d,l,s:d['gym_contract']['party'][0].update(level=99))
    reject('badge shortcut',lambda d,l,s:d['gym_contract'].update(requires=[]))
    reject('station arrival behind conductor',lambda d,l,s:s['boarding'].update(ride_arrival_core=[11,7]))
    reject('wrong station visited flag',lambda d,l,s:s['native_script_binding'].update(entry_callback='EVENT_MN_STATION_ST_PAUL_VISITED'))
    reject('broken conductor reset',lambda d,l,s:s['boarding'].update(conductor_restore_steps=['left','down']))
    canonical_room=next(r for r in d['interiors']if r['id']=='MnStCloudGym')
    document=json.loads(CANONICAL.read_text())
    def canonical_reject(name,change_room=None,change_gym=None,expected='canonical Gym5 object digest changed'):
        room=deepcopy(canonical_room);doc=deepcopy(document)
        if change_room:change_room(room)
        if change_gym:change_gym(next(g for g in doc['gyms']if g['id']=='MnStCloudGym'))
        try:canonical_phases(room,pack,doc)
        except AssertionError as error:
            assert expected in str(error),(name,str(error));cases.append(name);return
        raise AssertionError('canonical negative accepted: '+name)
    canonical_reject('stale canonical room SHA',change_room=lambda r:r['source_reference'].update(room_sha256='0'*64))
    canonical_reject('competing city gym rows',change_room=lambda r:r.update(rows=['#'*12]*12),expected='competing city-owned Gym5 source')
    canonical_reject('wrong canonical native glyph mapping',change_room=lambda r:r['source_reference']['symbol_blocks'].update(X='01'),expected='canonical Gym5 native glyph allocation changed')
    canonical_reject('changed canonical leader party',change_gym=lambda g:g['party'][0].update(level=29))
    canonical_reject('changed canonical leader move',change_gym=lambda g:g['party'][0]['moves'].__setitem__(0,'TACKLE'))
    canonical_reject('changed canonical leader sprite',change_gym=lambda g:g.update(leader_sprite='SPRITE_LASS'))
    canonical_reject('changed canonical wind direction',change_gym=lambda g:g['winds'].update({'>':'WALK_LEFT'}))
    canonical_reject('changed canonical court prerequisite',change_gym=lambda g:g['gates'][0].update(requires=[]))
    canonical_reject('changed canonical reward',change_gym=lambda g:g.update(reward='TM_RETURN'))
    # Whole-collection metadata and other gyms are explicitly not room authority.
    informational=deepcopy(canonical_room);informational['source_reference']['file_sha256_informational']='0'*64;canonical_phases(informational,pack)
    unrelated=deepcopy(document);unrelated['gyms'][0]['guide'][0]='Unrelated source edit';canonical_phases(canonical_room,pack,unrelated)
    return cases

def render(d,l,s,pack):
    started=time.perf_counter();report,k,seen,blocks=validate_town(d,l,pack);rooms,room_reports=validate_rooms(d,s,pack);report['rooms']=room_reports;report['negative_cases_rejected']=negative_checks(d,l,s,pack)
    digest=hashlib.sha256(json.dumps([d,l,s],sort_keys=True).encode()).hexdigest()[:12];out=OUTPUT/digest;out.mkdir(parents=True,exist_ok=True)
    for n,v in [('town-source.json',d),('layout-source.json',l),('station-source.json',s)]:
        payload=json.dumps(v,indent=2)+'\n';p=out/n;assert not p.exists()or p.read_text()==payload;p.write_text(payload)
    art=SourceArt(pack);terrain=art.render(l['compiled_tileset'],24,18,blocks);save_render(terrain,out/'town-terrain-original.png');scene=render_stage(terrain,'convert',out,'town-rgba','RGBA')
    for a in d['residents']:scene.alpha_composite(oriented_sprite(pack,a['sprite'],a['facing']),(a['tile'][0]*16,a['tile'][1]*16-4))
    save_render(scene,out/'whole-town-original-rgba.png');rgb=render_stage(scene,'convert',out,'town-rgb','RGB');save_render(rgb,out/'whole-town.png');save_render(rgb,OUTPUT/'whole-town.png')
    views=list(d['viewpoints']);best=(0,None)
    for q in seen:
        if k(q)!='FLOOR':continue
        if q[0]<5 or q[0]>42 or q[1]<4 or q[1]>31:continue
        score=sum(k((x,y))=='FLOOR'for y in range(q[1]-4,q[1]+5)for x in range(q[0]-5,q[0]+5))
        if score>best[0]:best=(score,q)
    views.append(dict(id='WIDEST_FLOOR_CAMERA',tile=list(best[1])))
    def cameras(im,entries,prefix):
        sheets=[]
        for v in entries:
            at=v.get('tile',v.get('core'));name=v.get('id',v.get('name'));x,y=at;cx=x*16+8;cy=y*16+8
            crop=render_stage(im,'crop',out,prefix+name,(cx-80,cy-72,cx+80,cy+72));frame=render_stage(crop,'convert',out,prefix+name+'-rgba','RGBA');frame.alpha_composite(oriented_sprite(pack,'SPRITE_CHRIS','south'),(72,60));save_render(frame,out/f'{prefix}{name}-native160.png');big=render_stage(frame,'resize',out,prefix+name+'-4x',(640,576),Image.Resampling.NEAREST);save_render(big,out/f'{prefix}{name}-4x.png');sheets.append((name,big))
        sheet=Image.new('RGB',(1304,618*((len(sheets)+1)//2)),(19,29,40));draw=ImageDraw.Draw(sheet)
        for i,(label,frame)in enumerate(sheets):x=i%2*652+6;y=i//2*618+24;draw.text((x,y-18),label,fill='white');sheet.paste(frame,(x,y))
        save_render(sheet,out/f'{prefix}views.png');return sheet
    cameras(scene,views,'town-');report['widest_floor_camera']=list(best[1])
    for r in rooms:
        if 'source_reference'in r:
            gym,symbols,room_digest,states=canonical_phases(r,pack)
            payload=json.dumps(gym,indent=2)+'\n';path=out/'MnStCloudGym-canonical-room-source.json';assert not path.exists()or path.read_text()==payload;path.write_text(payload)
            for state in [states[0],states[-1]]:
                phase='closed'if state['stage']=='ARRIVAL_CLOSED'else'open'
                w,h=gym['dimensions_metatiles'];terrain=Image.new('RGB',(w*32,h*32));inverse={int(value,16):symbol for symbol,value in symbols.items()}
                for index,block in enumerate(state['blocks']):
                    symbol=inverse[block];source,glyph=gym['art'][symbol]
                    terrain.paste(art.blocks(source,'indoor')[int(glyph,16)],(index%w*32,index//w*32))
                label=r['id']+'-'+phase
                save_render(terrain,out/(label+'-terrain-original.png'));im=render_stage(terrain,'convert',out,label+'-rgba','RGBA')
                for a in canonical_actors(gym):im.alpha_composite(oriented_sprite(pack,a['sprite'],a['facing']),(a['tile'][0]*16,a['tile'][1]*16-4))
                save_render(im,out/(label+'-whole-rgba.png'));save_render(render_stage(im,'convert',out,label+'-rgb','RGB'),out/(label+'-whole-rgb.png'))
                entries=[v for v in r['viewpoints']if v['phase']==state['stage']];cameras(im,entries,label+'-')
            continue
        w,h=r['dimensions_metatiles'];b=[int(v,16)for row in r['rows']for v in row.split()];im=art.render(r['tileset'],w,h,b);save_render(im,out/(r['id']+'-terrain-original.png'));im=render_stage(im,'convert',out,r['id']+'-rgba','RGBA')
        for a in r['residents']:im.alpha_composite(oriented_sprite(pack,a['sprite'],a['facing']),(a['tile'][0]*16,a['tile'][1]*16-4))
        save_render(im,out/(r['id']+'-whole-rgba.png'));save_render(render_stage(im,'convert',out,r['id']+'-rgb','RGB'),out/(r['id']+'-whole-rgb.png'))
        entries=r.get('viewpoints',[dict(id='arrival',tile=r['passive_arrival']['tile'])]);cameras(im,entries,r['id']+'-')
    report.update(hash=digest,status='SOURCE ONLY city; canonical Gym5 source is authoritative, native review-hall return rebinding/services/train/regional travel pending',render_ms=round((time.perf_counter()-started)*1000,1),directory=str(out),image=str(out/'whole-town.png'))
    (out/'review.json').write_text(json.dumps(report,indent=2)+'\n');(OUTPUT/'revision.json').write_text(json.dumps(report,indent=2)+'\n');print(json.dumps(report,indent=2));return report

def verify_published(pack,d,l,s):
    maps=pack['data']['maps'];city=maps[d['id']]
    assert city['blocks']==layout_blocks(l),'published town geometry differs'
    for door in d['doors']:
        room=maps[door['target']];enter=city['events']['warps'][door['warp_index']-1];back=city['events']['warps'][door['return_warp_index']-1]
        assert [enter['x'],enter['y']]==door['tile'] and enter['target_map_constant']==room['attributes']['map_constant'] and enter['target_warp_id']==3
        assert [back['x'],back['y']]==door['return_tile']
        assert len(room['events']['warps'])==3
        for warp in room['events']['warps']:
            assert warp['target_map_constant']=='MN_ST_CLOUD' and warp['target_warp_id']==door['return_warp_index']
    for source in [s if r['id']==s['id']else r for r in d['interiors']]:
        native=maps[source['id']]
        assert [native['attributes']['width'],native['attributes']['height']]==source['dimensions_metatiles']
        if 'source_reference'in source:
            gym,symbols,_=canonical_gym(source)
            assert native['blocks']==[int(symbols[c],16)for row in gym['rows']for c in row]
            collision=pack['data']['tilesets'][native['attributes']['tileset_name']]['collision']
            for symbol,index in symbols.items():
                ts,b=gym['art'][symbol];expected=([gym['winds'][symbol]]*4 if symbol in gym['winds']else pack['data']['tilesets'][ts]['collision'][b])
                assert collision[index]==expected,'canonical wind collision differs'
        else:
            assert native['blocks']==[int(b,16)for row in source['rows']for b in row.split()]
            assert {(a['script'],a['x'],a['y'])for a in native['objects']}=={(a['id'],*a['tile'])for a in source['residents']}
    assert city['attributes']['connections']==[dict(direction='south',target_map='MnR05C00R00',offset=0),dict(direction='north',target_map='MnR06C00R01',offset=4)]
    for name,direction,offset in [('MnR05C00R00','north',0),('MnR06C00R01','south',-4)]:
        assert dict(direction=direction,target_map='MnStCloud',offset=offset)in maps[name]['attributes']['connections']
    print('Published St Cloud: four native facades/returns, canonical wind collisions and both reciprocal route seams match source.')


def main():
    parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('--pack',type=Path,default=ROOT/'content-packs/minnesota-geography-review.crystalpack');parser.add_argument('--watch',action='store_true');parser.add_argument('--verify-published',action='store_true');parser.add_argument('--audit-reconciliation',action='store_true',help='Audit the retained historical non-gym snapshot separately from current authoring');args=parser.parse_args();pack=load_pack(args.pack);last=None
    if args.verify_published:
        verify_published(pack,json.loads(DESIGN.read_text()),json.loads(LAYOUT.read_text()),json.loads(STATION.read_text()))
    if args.audit_reconciliation:
        unchanged_non_gym(json.loads(DESIGN.read_text()));print('Historical non-gym reconciliation unchanged.');return
    while True:
        rev=tuple(p.stat().st_mtime_ns for p in [DESIGN,LAYOUT,STATION,CANONICAL])
        if rev!=last:
            # Retain exact authoring bytes even if an in-progress edit cannot
            # parse/render. A failed pass must not erase the last good images.
            sources=[p.read_bytes()for p in [DESIGN,LAYOUT,STATION]]
            key=hashlib.sha256(b'\0'.join(sources)).hexdigest()[:16]
            archive=OUTPUT/'source-passes'/key;archive.mkdir(parents=True,exist_ok=True)
            for path,payload in zip([DESIGN,LAYOUT,STATION],sources):
                target=archive/path.name;assert not target.exists()or target.read_bytes()==payload;target.write_bytes(payload)
            try:
                render(*(json.loads(value)for value in sources),pack)
            except(AssertionError,KeyError,IndexError,ValueError,TypeError)as error:
                failure=dict(status='INVALID SOURCE; previous renders retained',error=str(error),source_pass=key)
                (archive/'rejected.json').write_text(json.dumps(failure,indent=2)+'\n');print(json.dumps(failure),flush=True)
                if not args.watch:raise SystemExit(1)from error
            last=rev
        if not args.watch:break
        time.sleep(.15)
if __name__=='__main__':main()
