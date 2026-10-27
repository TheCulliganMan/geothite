#!/usr/bin/env python3
"""Read-only Duluth source/collision review; never builds or publishes a pack."""
import argparse
from collections import deque
from copy import deepcopy
from functools import partial
import hashlib
from http.server import SimpleHTTPRequestHandler, ThreadingHTTPServer
import json
import io
from pathlib import Path
import threading
import time

from PIL import Image
from ledge_paths import inspect_ledges
from render_archive import save_render
from render_reference import SourceArt, layout_blocks, load_pack, room_sheet
from render_stages import render_stage
from review_boundary_basecamp import adjacent, flood
from review_grand_marais import sprite

ROOT = Path(__file__).resolve().parents[3]
DESIGN = ROOT/'planning/minnesota/towns/MnDuluth.json'
LAYOUT = ROOT/'planning/minnesota/towns/MnDuluth.layout.json'
OUTPUT = ROOT/'planning/minnesota/renders/duluth'
CANONICAL = ROOT/'planning/minnesota/gyms/interiors.json'
BEFORE = OUTPUT/'canonical-gym-reconciliation/before-02bbba'


def canonical_gym(room, document=None):
    """Resolve one canonical room; the collection's whole-file hash is informative."""
    ref=room['source_reference']
    assert ref['path']=='planning/minnesota/gyms/interiors.json' and ref['gym_id']=='MnDuluthGym','canonical Gym8 authority changed'
    assert not {'rows','residents','party','art','gates','stations','reward'}&room.keys(),'competing city-owned Gym8 source'
    source=json.loads(CANONICAL.read_text()) if document is None else document
    gym=next(g for g in source['gyms'] if g['id']==ref['gym_id'])
    digest=hashlib.sha256(json.dumps(gym,sort_keys=True,separators=(',',':')).encode()).hexdigest()
    assert ref['room_sha256']==digest,'canonical Gym8 object digest changed'
    needed=set(''.join(gym['rows']))|{g['open_symbol'] for g in gym['gates']}|{'#'}
    # gym_art::install uses BTreeMap character order and assigns symbols.len().
    symbols={symbol:f'{index:02x}' for index,symbol in enumerate(sorted(needed))}
    assert ref['symbol_blocks']==symbols,'canonical native glyph allocation differs'
    assert room['tileset']=='mn_gym_8' and room['native_constant']==gym['constant'] and room['dimensions_metatiles']==gym['dimensions_metatiles'],'canonical Gym8 native identity differs'
    assert room['clear_arrival']==gym['arrival_core'] and room['passive_arrival']=={'index':3,'tile':gym['arrival_core'],'collision':'FLOOR'},'canonical passive arrival differs'
    assert room['exits']==[dict(index=i+1,tile=at,target='MnDuluth',target_warp=4,collision='WARP_CARPET_DOWN') for i,at in enumerate(gym['exit_tiles_core'])],'canonical carpets/exterior rebinding differs'
    return gym,symbols,digest


def gym_actors(gym):
    return [dict(id=gym['id']+'Guide',tile=gym['guide_at'],sprite='SPRITE_GYM_GUIDE',facing='south'),dict(id=gym['id']+'Leader',tile=gym['leader_at'],sprite=gym['leader_sprite'],facing='south')]+[dict(id=t['id'],tile=t['at'],sprite=t['sprite'],facing='south') for t in gym['trainers']]+[dict(id=gym['id']+s['id']+'Station',tile=s['at'],sprite=s['sprite'],facing='south') for s in gym['stations']]


def gym_kind(pack,gym,symbols,blocks,p):
    x,y=p;symbol=next(s for s,b in symbols.items() if int(b,16)==blocks[y//2*gym['dimensions_metatiles'][0]+x//2]);source,block=gym['art'][symbol]
    return pack['data']['tilesets'][source]['collision'][block][y%2*2+x%2]


def ice_resting_paths(start,kind,occupied,bounds):
    """Static button planner matching the native gym test's ICE advance rule."""
    paths={start:[]};todo=deque([start])
    while todo:
        p=todo.popleft()
        for name,dx,dy in [('left',-1,0),('up',0,-1),('right',1,0),('down',0,1)]:
            q=p
            for _ in range(64):
                n=(q[0]+dx,q[1]+dy)
                if not(0<=n[0]<bounds[0] and 0<=n[1]<bounds[1]) or n in occupied or kind(n) not in {'FLOOR','ICE'}:break
                q=n
                if kind(q)!='ICE':break
            if q!=p and q not in paths:paths[q]=paths[p]+[name];todo.append(q)
    return paths


def canonical_phases(room,pack):
    gym,symbols,digest=canonical_gym(room);blocks=[int(symbols[s],16) for row in gym['rows'] for s in row];occupied={tuple(a['tile']) for a in gym_actors(gym)};events=set();at=tuple(gym['arrival_core']);states=[];bounds=[v*2 for v in gym['dimensions_metatiles']]
    def kind(p):return gym_kind(pack,gym,symbols,blocks,p)
    def record(stage,buttons):
        reached=ice_resting_paths(at,kind,occupied,bounds)
        states.append(dict(stage=stage,events=sorted(events),blocks=list(blocks),resting_reachable=sorted(reached),actual_approach_core=list(at),planned_buttons=buttons,source_fixture_only=True))
        return reached
    assert kind(at)=='FLOOR' and all(kind(p)=='WARP_CARPET_DOWN' for p in gym['exit_tiles_core']),'canonical arrival or carpets blocked'
    reached=record('ARRIVAL_CLOSED',[])
    assert not set(adjacent(tuple(gym['leader_at'])))&reached.keys(),'closed canonical court bypass'
    for step in gym['walkthrough']:
        trainer=next((t for t in gym['trainers'] if t['id']==step),None);actor=trainer or next(s for s in gym['stations'] if s['id']==step)
        assert set(actor.get('requires',[]))<=events,'canonical walkthrough missing prerequisite '+step
        paths=ice_resting_paths(at,kind,occupied,bounds);approaches=[p for p in adjacent(tuple(actor['at'])) if p in paths]
        assert approaches,'canonical ICE walkthrough cannot approach '+step
        target=min(approaches,key=lambda p:(len(paths[p]),p));buttons=paths[target];at=target
        events.add('EVENT_'+step+'_DEFEATED' if trainer else actor['event'])
        for gate in gym['gates']:
            if set(gate['requires'])<=events:
                gx,gy=gate['at'];blocks[gy*gym['dimensions_metatiles'][0]+gx]=int(symbols[gate['open_symbol']],16);events.add(gate['event'])
        record(step,buttons)
    opened=ice_resting_paths(at,kind,occupied,bounds)
    assert set(adjacent(tuple(gym['leader_at'])))&opened.keys(),'canonical open court unreachable'
    assert {(x,y-1) for x,y in gym['exit_tiles_core']}<=opened.keys(),'canonical return floor inaccessible'
    for actor in gym_actors(gym):
        assert kind(tuple(actor['tile']))=='FLOOR' and set(adjacent(tuple(actor['tile'])))&opened.keys(),'canonical actor footing/approach '+actor['id']
    assert not any('EVENT_MN_GYM_8_COURT_OPEN' in s['events'] for s in states[:-1]),'canonical court opened before final trainer'
    assert 'EVENT_MN_GYM_8_COURT_OPEN' in states[-1]['events'],'canonical court never opened'
    for view in room['viewpoints']:
        state=next(s for s in states if s['stage']==view['phase'])
        assert tuple(view['tile']) in state['resting_reachable'],view['id']+' is not a legal ICE resting camera'
    return gym,symbols,digest,states


def canonical_report(room,pack):
    gym,symbols,digest,states=canonical_phases(room,pack)
    return dict(id=room['id'],dimensions=gym['dimensions_metatiles'],canonical_source=room['source_reference']['path'],canonical_room_sha256=digest,native_symbol_blocks=symbols,canonical_whole_file_sha256_informational=hashlib.sha256(CANONICAL.read_bytes()).hexdigest(),recorded_whole_file_hash_matches=room['source_reference']['file_sha256_informational']==hashlib.sha256(CANONICAL.read_bytes()).hexdigest(),phases=states,source_only=True,native_gameplay_proved=False)


def unchanged_non_gym(design):
    before=json.loads((BEFORE/'MnDuluth.json').read_text());current=deepcopy(design)
    before['interiors']=[r for r in before['interiors'] if r['id']!='MnDuluthGym'];current['interiors']=[r for r in current['interiors'] if r['id']!='MnDuluthGym']
    assert current==before,'non-Gym8 city source changed during gym reconciliation'
    assert LAYOUT.read_bytes()==(BEFORE/'MnDuluth.layout.json').read_bytes(),'exterior layout changed during gym reconciliation'


def oriented_sprite(pack, token, facing):
    """Keep original sprite, palette stages, selected frame and facing conversion."""
    target=OUTPUT/'sprites'/token.lower()
    art=sprite(pack,token,archive_output=target)
    source=Image.open(io.BytesIO(bytes(pack['runtime_files']['gfx/sprites/'+token.removeprefix('SPRITE_').lower()+'.png'])))
    save_render(source,target/'original-full-sheet.png')
    sheet=render_stage(source,'convert',target,'full-sheet-rgb','RGB')
    offset=16 if facing=='north' else 32 if facing in {'west','east'} else 0
    if offset:
        first=render_stage(sheet,'crop',target,'front-frame',(0,0,16,16))
        colors={gray[0]:pixel for gray,pixel in zip(first.getdata(),art.getdata())}
        frame=render_stage(sheet,'crop',target,facing+'-frame',(0,offset,16,offset+16))
        art=render_stage(frame,'convert',target,facing+'-frame-rgba','RGBA')
        art.putdata([colors[p[0]] for p in art.getdata()])
        save_render(art,target/(facing+'-frame-day-palette.png'))
    if facing=='east':
        save_render(art,target/'east-frame-before-flip.png')
        art=art.transpose(Image.Transpose.FLIP_LEFT_RIGHT)
        save_render(art,target/'east-frame-after-flip.png')
    save_render(art,target/(facing+'-final.png'))
    return art


def prepare(pack, layout):
    definition = deepcopy(pack['data']['tilesets'][layout['tileset']])
    for r in layout['derived_blocks']:
        definition['collision'][r['id']] = list(pack['data']['tilesets'][r.get('collision_source_tileset',r['source_tileset'])]['collision'][r.get('collision_source_block',r['source_block'])])
    pack['data']['tilesets'][layout['compiled_tileset']] = definition


def geometry(design, layout, pack):
    assert design['id']==layout['id']=='MnDuluth'
    assert design['dimensions_metatiles']==layout['dimensions_metatiles']==[32,24]
    assert layout['tileset']=='johto_modern' and not layout['source_pixels_modified']
    blocks=layout_blocks(layout);w,h=layout['dimensions_metatiles']
    cs=pack['data']['tilesets'][layout['compiled_tileset']]['collision']
    def kind(p):
        x,y=p
        return cs[f'{blocks[y//2*w+x//2]:02x}'][y%2*2+x%2]
    cells={(x,y) for y in range(h*2) for x in range(w*2)}
    actors={tuple(a['tile']) for a in design['residents']}
    assert len(actors)==len(design['residents']), 'duplicate actor anchor'
    walk={p for p in cells if kind(p) in {'FLOOR','DOOR','TALL_GRASS'}}
    water={p for p in cells if kind(p)=='WATER'}
    doors={tuple(d['tile']) for d in design['doors']}
    seen=flood((6,24),walk-actors,doors)
    surf=flood((6,24),(walk|water)-actors,doors)
    expected=[('MN_R10','W',[1,12],[3,12]),('MN_R11','N',[16,1],[16,3])]
    assert len(design['ports'])==len(expected), 'original two ports changed'
    for p,(route,side,gate,landing) in zip(design['ports'],expected):
        assert (p['route'],p['side'],p['gate_metatile'],p['landing_metatile'])==(route,side,gate,landing),route+' reservation changed'
        for pos in [gate,landing]:assert tuple(v*2 for v in pos) in seen,route+' landing disconnected'
    native_doors={p for p in cells if kind(p)=='DOOR'}
    assert native_doors==doors, ('unbound or false native facade door',native_doors,doors)
    indices=[]
    for d in design['doors']:
        assert tuple(d['tile']) in seen, d['id']+' door disconnected'
        assert kind(tuple(d['return_tile']))=='FLOOR' and tuple(d['return_tile']) in seen,d['id']+' unsafe exterior return'
        assert d['return_tile']==d['arrival_tile']==[d['tile'][0],d['tile'][1]+1],d['id']+' return must stand directly below facade'
        assert d['target_warp']==3, 'room entry requires passive FLOOR warp3'
        indices.extend([d['warp_index'],d['return_warp_index']])
    assert indices==list(range(1,len(doors)*2+1)), 'exterior active/passive warp order'
    assert not walk-seen-actors,('disconnected exterior walking island',sorted(walk-seen-actors))
    for a in design['residents']:
        assert kind(tuple(a['tile']))=='FLOOR',a['id']+' actor not on FLOOR: '+kind(tuple(a['tile']))
        assert set(adjacent(tuple(a['tile'])))&seen,a['id']+' inaccessible actor'
        assert all(0<len(s)<=18 for s in a['lines']),a['id']+' dialogue width'
    for s in design['signs']:
        assert kind(tuple(s['tile']))=='WALL',s['id']+' sign not on WALL'
        assert set(adjacent(tuple(s['tile'])))&seen,s['id']+' inaccessible sign'
        assert all(0<len(line)<=18 for line in s['lines']),s['id']+' text width'
    assert not any(kind(p)=='CUT_TREE' for p in cells), 'incidental Cut gate'
    outer={p for p in cells if p[0] in {0,63} or p[1] in {0,47}}
    assert not water&outer,'open Surf outer edge without neighboring water connection'
    assert seen&outer=={(0,24),(0,25),(32,0),(33,0)},('unexpected walking outer edge',seen&outer)
    harbor=design['harbor']
    assert tuple(harbor['launch_tile']) in seen and tuple(harbor['water_target']) in water,'low launch disconnected or dry'
    shore_edges={(a,b) for a in walk if kind(a)=='FLOOR' for b in adjacent(a) if b in water}
    approved={(tuple(e['bank']),tuple(e['water'])) for e in harbor['allowed_surf_edges']}
    assert shore_edges==approved,('unplanned or blocked harbor launch',sorted(shore_edges^approved))
    assert tuple(harbor['pier_approach']) in seen and tuple(harbor['pier_end']) in seen,'pier deck disconnected'
    for x,y in harbor['north_rail_blocks']:
        assert [kind((2*x,2*y)),kind((2*x,2*y+1))]==['WALL','FLOOR'], 'bridge north rail not solid'
    for x,y in harbor['south_rail_blocks']:
        assert [kind((2*x,2*y)),kind((2*x,2*y+1))]==['FLOOR','WALL'], 'bridge south rail not solid'
    for x,y in harbor['pier_left_rail']:
        assert [kind((2*x,2*y)),kind((2*x+1,2*y))]==['WALL','FLOOR'],'pier left rail not solid'
    for x,y in harbor['pier_right_rail']:
        assert [kind((2*x,2*y)),kind((2*x+1,2*y))]==['FLOOR','WALL'],'pier right rail not solid'
    ledges=inspect_ledges(design,lambda x,y:kind((x,y)),seen)
    declared_hops=set()
    for ledge in design['one_way_ledges']:
        assert kind(tuple(ledge['hop_core']))=='HOP_DOWN','native ledge missing'
        assert kind(tuple(ledge['reverse_blocked']))=='WALL','reverse ledge climb open'
        a=ledge['upper_approach'];b=ledge['landing']
        assert b==[a[0],a[1]+3], 'native ledge displacement differs'
        for x,y in ledge['source_blocks']:
            for dx in [0,1]:
                upper=(2*x+dx,2*y-1);hop=(2*x+dx,2*y);landing=(2*x+dx,2*y+2)
                declared_hops.add(hop)
                assert upper in seen and landing in seen and kind(upper)=='FLOOR' and kind(hop)=='HOP_DOWN' and kind(landing)=='FLOOR','unusable authored ledge quadrant'
                assert kind((2*x+dx,2*y+1))=='WALL','reverse side of authored ledge is open'
    assert {p for p in cells if kind(p).startswith('HOP_')}==declared_hops,'incidental or undeclared exterior hop'
    for v in design['viewpoints']:
        assert tuple(v['tile']) in seen,v['id']+' camera is not a reachable player tile'
    return dict(source_only=True,exterior_dimensions=[w,h],reachable_public_tiles=len(seen),reachable_surf_tiles=len(water&surf),approved_cove_surf_edges=len(shore_edges),doors=len(doors),outer_water_cells=0,incidental_cut=0,original_ports_retained=True,ledges=ledges),blocks,kind


def rooms(design,pack):
    assert {r['id'] for r in design['interiors']}=={d['target'] for d in design['doors']},'room/facade mismatch'
    result=[]
    reserved={'MnDuluthPokecenter1F':([10,8],[10,13],[10,15]),'MnDuluthMart':([6,4],[6,5],[6,7]),'MnDuluthGym':([7,10],[6,17],[6,19]),'MnDuluthHarborWorkshop':([8,6],[8,9],[8,11])}
    for r in design['interiors']:
        if r['id']=='MnDuluthGym':
            result.append(canonical_report(r,pack))
            continue
        w,h=r['dimensions_metatiles'];rows=[row.split() for row in r['rows']]
        assert len(rows)==h and all(len(row)==w for row in rows),r['id']+' room dimensions'
        bs=[int(b,16) for row in rows for b in row];cs=pack['data']['tilesets'][r['tileset']]['collision']
        def kind(p):
            x,y=p
            return cs[f'{bs[y//2*w+x//2]:02x}'][y%2*2+x%2]
        cells={(x,y) for y in range(h*2) for x in range(w*2)}
        actors={tuple(a['tile']) for a in r['residents']}
        walk={p for p in cells if kind(p) in {'FLOOR','ICE','WARP_CARPET_DOWN'}}
        arrival=tuple(r['passive_arrival']['tile']);assert r['passive_arrival']['index']==3 and kind(arrival)=='FLOOR',r['id']+' passive arrival unsafe'
        seen=flood(arrival,walk-actors)
        carpets={p for p in cells if kind(p)=='WARP_CARPET_DOWN'};exits={tuple(e['tile']) for e in r['exits']}
        assert carpets==exits and len(exits)==2 and exits<=seen,r['id']+' unbound or unreachable carpet'
        exterior=next(d for d in design['doors'] if d['target']==r['id'])
        assert all(e['target']=='MnDuluth' and e['target_warp']==exterior['return_warp_index'] for e in r['exits']),r['id']+' wrong exterior return'
        if r['id'] in reserved:
            dim,arr,exit=reserved[r['id']]
            assert [w,h]==dim and list(arrival)==arr and exit in [e['tile'] for e in r['exits']],r['id']+' shared reservation changed'
        for a in r['residents']:
            assert kind(tuple(a['tile']))=='FLOOR',a['id']+' room actor off FLOOR'
            assert all(0<len(line)<=18 for line in a['lines']),a['id']+' room text width'
            if 'counter_tile' in a:
                assert kind(tuple(a['counter_tile']))=='COUNTER' and tuple(a['approach']) in seen,a['id']+' counter approach'
            else:assert set(adjacent(tuple(a['tile'])))&seen,a['id']+' unreachable room actor'
            if 'party' in a:
                assert a['trainer_class'] in pack['data']['trainer_class_names'],a['id']+' unknown native trainer class'
                assert 1<=len(a['party'])<=6,a['id']+' illegal party size'
                assert all(mon['species'] in pack['data']['pokemon'] and 1<=mon['level']<=100 for mon in a['party']),a['id']+' illegal native party'
            if 'stock' in a:
                assert a['stock'] and all(item in pack['data']['items'] for item in a['stock']),a['id']+' unknown shop item'
        for s in r.get('signs',[]):
            assert kind(tuple(s['tile']))=='WALL' and tuple(s['approach']) in seen,s['id']+' inaccessible exhibit'
            assert all(0<len(line)<=18 for line in s['lines']),s['id']+' exhibit text width'
        if 'pc' in r:
            assert kind(tuple(r['pc']['tile']))=='PC' and tuple(r['pc']['approach']) in seen,'PC terminal unreachable'
        result.append(dict(id=r['id'],dimensions=[w,h],arrival=list(arrival),carpets=sorted(exits),public_tiles=len(seen),source_only=True))
    receiver=next(a for r in design['interiors'] for a in r.get('residents',[]) if a['id']=='MN_HARBOR_RECEIVER')
    assert receiver['tile']==[11,5] and receiver['sets']==['EVENT_MN_LIGHTHOUSE_SIGNAL'],'receiver Q07 anchor/flag changed'
    assert receiver['requires']==['BADGE_'+str(i) for i in range(1,9)]+['EVENT_MN_DEPOT_CLEARED'],'receiver Q07 prerequisites changed'
    return result


def negatives(design,layout,pack):
    results=[]
    def reject(name,change):
        d,l=deepcopy(design),deepcopy(layout);change(d,l)
        try:prepare(pack,l);geometry(d,l,pack);rooms(d,pack)
        except AssertionError as error:results.append(dict(case=name,rejected_by=str(error)));return
        raise AssertionError('negative mutation accepted: '+name)
    reject('false facade door',lambda d,l:d['doors'][0].update(tile=[8,11]))
    reject('unsafe exterior return',lambda d,l:d['doors'][0].update(return_tile=[9,10]))
    reject('changed original port',lambda d,l:d['ports'][0].update(gate_metatile=[1,11]))
    reject('outer Surf escape',lambda d,l:l['layers'].append(dict(name='Invalid outer water hole',kind='point',position=[31,18],block='35')))
    reject('open bridge north rail',lambda d,l:l['layers'].append(dict(name='Invalid railing gap',kind='point',position=[17,15],block='01')))
    reject('open pier rail',lambda d,l:l['layers'].append(dict(name='Invalid pier gap',kind='point',position=[7,21],block='01')))
    reject('ledge reverse hole',lambda d,l:l['layers'].append(dict(name='Invalid uphill shortcut',kind='point',position=[7,10],block='01')))
    reject('blocked shared room arrival',lambda d,l:d['interiors'][3]['passive_arrival'].update(tile=[0,0]))
    reject('wrong carpet return',lambda d,l:d['interiors'][1]['exits'][0].update(target_warp=1))
    reject('Q07 signal without all badges',lambda d,l:d['interiors'][3]['residents'][0].update(requires=['BADGE_8']))
    reject('disconnected floor island',lambda d,l:l['layers'].append(dict(name='Invalid isolated forest floor',kind='point',position=[4,1],block='01')))
    reject('unplanned island Surf launch',lambda d,l:l['layers'].append(dict(name='Invalid island shore gap',kind='point',position=[20,18],block='01')))
    reject('stale canonical room digest',lambda d,l:d['interiors'][2]['source_reference'].update(room_sha256='0'*64))
    reject('competing city-owned gym rows',lambda d,l:d['interiors'][2].update(rows=['#######']*10))
    reject('wrong native gym glyph allocation',lambda d,l:d['interiors'][2]['source_reference']['symbol_blocks'].update(X='01'))
    reject('canonical camera stands on actor',lambda d,l:d['interiors'][2]['viewpoints'][0].update(tile=[10,17]))
    # These changes must be accepted: the reference belongs to one Gym8 object,
    # not the bytes of every unrelated gym in the shared collection.
    reference=deepcopy(design['interiors'][2]);reference['source_reference']['file_sha256_informational']='0'*64
    canonical_gym(reference)
    collection=json.loads(CANONICAL.read_text());next(g for g in collection['gyms'] if g['id']!='MnDuluthGym')['leader']='UNRELATED TEST'
    canonical_gym(reference,collection)
    results.append(dict(case='unrelated gym and informational whole-file hash change',accepted=True))
    changed=json.loads(CANONICAL.read_text());next(g for g in changed['gyms'] if g['id']=='MnDuluthGym')['party'][0]['level']+=1
    try:canonical_gym(reference,changed)
    except AssertionError as error:results.append(dict(case='canonical Gym8 object itself changed',rejected_by=str(error)))
    else:raise AssertionError('changed canonical Gym8 object accepted under stale digest')
    return results


def render_canonical_gym(room,pack,art):
    gym,symbols,digest,states=canonical_phases(room,pack)
    out=OUTPUT/'canonical-gym-reconciliation'/digest[:16];out.mkdir(parents=True,exist_ok=True)
    payload=json.dumps(gym,sort_keys=True,separators=(',',':')).encode()
    (out/('canonical-source-object-'+digest+'.json')).write_bytes(payload)
    (out/'source-phase-fixtures.json').write_text(json.dumps(states,indent=2)+'\n')
    rendered=[];camera_views=[]
    phases=[('closed','ARRIVAL_CLOSED'),('west-open','MN_GYM_8_TRAINER_1'),('east-open','MN_GYM_8_TRAINER_2'),('court-open','MN_GYM_8_TRAINER_3')]
    inverse={int(block,16):symbol for symbol,block in symbols.items()}
    for label,stage in phases:
        state=next(s for s in states if s['stage']==stage)
        w,h=gym['dimensions_metatiles'];terrain=Image.new('RGB',(w*32,h*32))
        for index,block in enumerate(state['blocks']):
            source,source_block=gym['art'][inverse[block]]
            terrain.paste(art.blocks(source,'indoor')[int(source_block,16)],(index%w*32,index//w*32))
        save_render(terrain,out/(label+'-terrain-original.png'))
        scene=render_stage(terrain,'convert',out,label+'-rgba','RGBA')
        for actor in gym_actors(gym):scene.alpha_composite(oriented_sprite(pack,actor['sprite'],actor['facing']),(actor['tile'][0]*16,actor['tile'][1]*16-4))
        save_render(scene,out/(label+'-original-rgba.png'))
        rgb=render_stage(scene,'convert',out,label+'-rgb','RGB');save_render(rgb,out/(label+'.png'))
        if label in {'closed','court-open'}:rendered.append(('Canonical Gym8 '+label,rgb))
        if label=='closed':
            save_render(terrain,OUTPUT/'room-MnDuluthGym-terrain-original.png')
            save_render(scene,OUTPUT/'room-MnDuluthGym-original-rgba.png')
        border_source,border_block=gym['art']['#'];border=art.blocks(border_source,'indoor')[int(border_block,16)]
        padded=Image.new('RGBA',(w*32+192,h*32+192))
        for py in range(0,padded.height,32):
            for px in range(0,padded.width,32):padded.paste(border,(px,py))
        save_render(padded,out/(label+'-native-border-original.png'));padded.alpha_composite(scene,(96,96));save_render(padded,out/(label+'-native-border-scene.png'))
        for view in room['viewpoints']:
            if view['phase']!=stage:continue
            x,y=view['tile'];name=view['id']
            camera=render_stage(padded,'crop',out,name+'-native160',(x*16+96-80,y*16+96-72,x*16+96+80,y*16+96+72));save_render(camera,out/(name+'-native160.png'))
            camera_views.append((label+': '+name,camera))
            preview=render_stage(camera,'resize',out,name+'-4x',(640,576),Image.Resampling.NEAREST);save_render(preview,out/(name+'-4x.png'))
    save_render(room_sheet(rendered,2),out/'closed-open-comparison.png')
    save_render(room_sheet(camera_views,3),out/'legal-ice-resting-cameras.png')
    return rendered


def render(design,layout,pack):
    unchanged_non_gym(design)
    start=time.perf_counter();prepare(pack,layout)
    report,bs,_=geometry(design,layout,pack);report['rooms']=rooms(design,pack);report['negative_cases']=negatives(design,layout,pack)
    prepare(pack,layout);art=SourceArt(pack);w,h=layout['dimensions_metatiles']
    terrain=art.render(layout['tileset'],w,h,bs,derived=layout['derived_blocks']);save_render(terrain,OUTPUT/'terrain-original.png')
    scene=render_stage(terrain,'convert',OUTPUT,'whole-scene-rgba','RGBA')
    for a in design['residents']:scene.alpha_composite(oriented_sprite(pack,a['sprite'],a['facing']),(a['tile'][0]*16,a['tile'][1]*16-4))
    save_render(scene,OUTPUT/'whole-town-original-rgba.png');digest=hashlib.sha256(scene.tobytes()).hexdigest()[:10]
    rgb=render_stage(scene,'convert',OUTPUT,'whole-town-rgb','RGB');save_render(rgb,OUTPUT/'whole-town.png');save_render(rgb,OUTPUT/f'whole-town-{digest}.png')
    preview=render_stage(rgb,'resize',OUTPUT,'whole-town-half-preview',(512,384),Image.Resampling.NEAREST)
    save_render(preview,OUTPUT/'whole-town-half.png')
    views=[]
    for v in design['viewpoints']:
        x,y=v['tile'];x=max(0,min(x*16-80,w*32-160));y=max(0,min(y*16-72,h*32-144))
        crop=render_stage(scene,'crop',OUTPUT,v['id'].lower()+'-crop',(x,y,x+160,y+144));save_render(crop,OUTPUT/f'viewport-{v["id"].lower()}-native160.png')
        views.append((v['id'],crop));preview=render_stage(crop,'resize',OUTPUT,v['id'].lower()+'-4x',(640,576),Image.Resampling.NEAREST);save_render(preview,OUTPUT/f'viewport-{v["id"].lower()}-4x.png')
    save_render(room_sheet(views,3),OUTPUT/'viewpoints.png')
    room_images=[]
    for r in design['interiors']:
        if r['id']=='MnDuluthGym':
            room_images.extend(render_canonical_gym(r,pack,art))
            continue
        rw,rh=r['dimensions_metatiles'];rbs=[int(b,16) for row in r['rows'] for b in row.split()]
        raw=art.render(r['tileset'],rw,rh,rbs);save_render(raw,OUTPUT/f'room-{r["id"]}-terrain-original.png')
        im=render_stage(raw,'convert',OUTPUT,r['id']+'-rgba','RGBA')
        for a in r['residents']:im.alpha_composite(oriented_sprite(pack,a['sprite'],a['facing']),(a['tile'][0]*16,a['tile'][1]*16-4))
        save_render(im,OUTPUT/f'room-{r["id"]}-original-rgba.png');room_images.append((r['id'],im))
    save_render(room_sheet(room_images,3),OUTPUT/'rooms.png')
    # Compare original encoded exterior and six other room images, not counts.
    protected=['terrain-original.png','whole-town-original-rgba.png','whole-town.png']+[f'room-{r["id"]}-{suffix}.png' for r in design['interiors'] if r['id']!='MnDuluthGym' for suffix in ['terrain-original','original-rgba']]
    for filename in protected:assert (OUTPUT/filename).read_bytes()==(BEFORE/filename).read_bytes(),'non-Gym8 render changed: '+filename
    report['non_gym_preservation']=dict(source_fields_unchanged=True,exterior_layout_bytes_unchanged=True,encoded_image_files_unchanged=protected)
    report.update(source_pack_content_hash=pack['identity']['content_hash'],layout_sha256=hashlib.sha256(LAYOUT.read_bytes()).hexdigest(),dossier_sha256=hashlib.sha256(DESIGN.read_bytes()).hexdigest(),whole_town=f'whole-town-{digest}.png',render_seconds=round(time.perf_counter()-start,3),implementation_status='SOURCE ONLY; no new town/service/warps/train/cutscene/gameplay published')
    payload=(json.dumps(report,indent=2)+'\n').encode()
    OUTPUT.mkdir(parents=True,exist_ok=True);(OUTPUT/'source-review.json').write_bytes(payload)
    (OUTPUT/('source-review-'+hashlib.sha256(payload).hexdigest()+'.json')).write_bytes(payload)
    snapshots=OUTPUT/'source-snapshots';snapshots.mkdir(exist_ok=True)
    for source in [DESIGN,LAYOUT]:
        data=source.read_bytes();(snapshots/(source.stem+'-'+hashlib.sha256(data).hexdigest()+'.json')).write_bytes(data)
    return report


def main():
    parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('--pack',type=Path,default=ROOT/'content-packs/minnesota-quality-review.crystalpack');parser.add_argument('--serve',action='store_true');parser.add_argument('--port',type=int,default=8776);parser.add_argument('--gym-only',action='store_true',help='Check/render canonical Gym8 only; preserve the separate historical town baseline');args=parser.parse_args();pack=load_pack(args.pack)
    if args.gym_only:
        if args.serve:parser.error('--gym-only does not serve the whole-town review')
        room=next(r for r in json.loads(DESIGN.read_text())['interiors'] if r['id']=='MnDuluthGym')
        report=canonical_report(room,pack)
        render_canonical_gym(room,pack,SourceArt(pack))
        print(json.dumps(report),flush=True)
        return
    OUTPUT.mkdir(parents=True,exist_ok=True)
    (OUTPUT/'index.html').write_text('<!doctype html><meta charset=utf-8><title>Duluth source review</title><style>body{background:#14212a;color:#eef;font:16px system-ui;margin:24px}img{image-rendering:pixelated;max-width:100%}</style><h1>Duluth full source candidate</h1><p>Native city publication and production gameplay remain pending.</p><img id=town src=whole-town.png><h2>160px player cameras</h2><img id=views src=viewpoints.png><h2>Seven rooms: canonical Gym8 closed/open included</h2><img id=rooms src=rooms.png><script>setInterval(()=>{let t=Date.now();town.src="whole-town.png?"+t;views.src="viewpoints.png?"+t;rooms.src="rooms.png?"+t},1200)</script>')
    if args.serve:
        class Handler(SimpleHTTPRequestHandler):
            def log_message(self,*args):pass
        server=ThreadingHTTPServer(('127.0.0.1',args.port),partial(Handler,directory=str(OUTPUT)));threading.Thread(target=server.serve_forever,daemon=True).start();print(f'Duluth review http://127.0.0.1:{args.port}/',flush=True)
    old=None
    while True:
        own_gym=next(g for g in json.loads(CANONICAL.read_text())['gyms'] if g['id']=='MnDuluthGym')
        current=(DESIGN.stat().st_mtime_ns,LAYOUT.stat().st_mtime_ns,hashlib.sha256(json.dumps(own_gym,sort_keys=True,separators=(',',':')).encode()).hexdigest())
        if old!=current:
            try:print(json.dumps(render(json.loads(DESIGN.read_text()),json.loads(LAYOUT.read_text()),pack)),flush=True)
            except (AssertionError,KeyError,ValueError,IndexError) as error:
                print('DESIGN ERROR: '+str(error),flush=True)
                if not args.serve:raise SystemExit(1)
            old=current
        if not args.serve:break
        time.sleep(.3)


if __name__=='__main__':main()
