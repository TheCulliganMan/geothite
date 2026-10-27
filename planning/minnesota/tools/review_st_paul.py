#!/usr/bin/env python3
"""Read-only native-source-art review of the explicitly authored St Paul town."""
import argparse
from copy import deepcopy
import hashlib
import json
from pathlib import Path
import time

from PIL import Image

from render_archive import save_render
from render_reference import SourceArt,layout_blocks,load_pack,room_sheet
from render_stages import render_stage
from review_north_shore import oriented_sprite
from review_gunflint import flood,neighbors
from ledge_paths import shortest_walk

ROOT=Path(__file__).resolve().parents[3]
DESIGN=ROOT/'planning/minnesota/towns/MnStPaul.json'
LAYOUT=ROOT/'planning/minnesota/towns/MnStPaul.layout.json'
OUTPUT=ROOT/'planning/minnesota/renders/st-paul'


def canonical_gym(room):
    reference=room['source_reference'];source=ROOT/reference['path'];data=json.loads(source.read_text());gym=next(g for g in data['gyms'] if g['id']==reference['gym_id'])
    needed=set(''.join(gym['rows']))|{g['open_symbol'] for g in gym['gates']}|{'#'}
    symbols={symbol:f'{index:02x}' for index,symbol in enumerate(sorted(needed))}
    return gym,symbols,hashlib.sha256(source.read_bytes()).hexdigest(),hashlib.sha256(json.dumps(gym,sort_keys=True,separators=(',',':')).encode()).hexdigest()


def gym_actors(gym):
    return [dict(id='Leader',tile=gym['leader_at'],sprite=gym['leader_sprite'],facing='south'),dict(id='Guide',tile=gym['guide_at'],sprite='SPRITE_GYM_GUIDE',facing='south')]+[dict(id=a['id'],tile=a['at'],sprite=a['sprite'],facing='south') for a in gym['trainers']]


def check_canonical_gym(room,pack,errors):
    gym,symbols,file_hash,room_hash=canonical_gym(room);ref=room['source_reference'];name=room['id']
    if ref['file_sha256']!=file_hash or ref['room_sha256']!=room_hash:errors.append(name+': canonical gym source hash differs')
    if ref['symbol_blocks']!=symbols:errors.append(name+': native glyph allocation differs')
    if room['dimensions_metatiles']!=gym['dimensions_metatiles'] or room['tileset']!='mn_gym_4' or room['native_constant']!=gym['constant'] or 'rows' in room or 'residents' in room:errors.append(name+': competing canonical gym geometry')
    exits=[dict(index=index,tile=at,target='MnStPaul',target_warp=10) for index,at in enumerate(gym['exit_tiles_core'],1)]
    if room['exits']!=gym['exit_tiles_core'] or room['passive_arrival']!={'index':3,'tile':gym['arrival_core']} or room['exit_warps']!=exits:errors.append(name+': canonical carpet/passive anchors differ')
    rows=[list(row) for row in gym['rows']];actors=gym_actors(gym);occupied={tuple(a['tile']) for a in actors}
    def kind(at):
        x,y=at;source,block=gym['art'][rows[y//2][x//2]];return pack['data']['tilesets'][source]['collision'][block][y%2*2+x%2]
    width,height=gym['dimensions_metatiles']
    def reached():
        walk={(x,y) for y in range(height*2) for x in range(width*2) if kind((x,y)) in {'FLOOR','ICE','WARP_CARPET_DOWN','WARP_PANEL'} and (x,y) not in occupied}
        seen=flood(tuple(gym['arrival_core']),walk)
        while True:
            old=len(seen)
            for lift in gym.get('lifts',[]):
                a,b=map(tuple,lift['ends_core'])
                for source,target in [(a,b),(b,a)]:
                    if source in seen:seen |= flood(target,walk)
            if len(seen)==old:return seen
    def adjacent(seen,at):return any(p in seen for p in neighbors(tuple(at)))
    closed=reached()
    if kind(tuple(gym['arrival_core']))!='FLOOR' or any(kind(tuple(at))!='WARP_CARPET_DOWN' or tuple(at) not in closed for at in room['exits']):errors.append(name+': canonical native arrival/carpet blocked')
    if adjacent(closed,gym['leader_at']):errors.append(name+': canonical closed court bypass')
    completed=set()
    for step in gym['walkthrough']:
        trainer=next((a for a in gym['trainers'] if a['id']==step),None);actor=trainer or next(a for a in gym['stations'] if a['id']==step)
        if not set(actor.get('requires',[]))<=completed or not adjacent(reached(),actor['at']):errors.append(name+': canonical walkthrough blocked '+step)
        completed.add('EVENT_'+step+'_DEFEATED' if trainer else actor['event'])
        for gate in gym['gates']:
            if set(gate['requires'])<=completed:rows[gate['at'][1]][gate['at'][0]]=gate['open_symbol'];completed.add(gate['event'])
    opened=reached()
    for actor in actors:
        if kind(tuple(actor['tile']))!='FLOOR' or not adjacent(opened,actor['tile']):errors.append(name+': canonical actor footing/approach '+actor['id'])
    return dict(id=name,dimensions=gym['dimensions_metatiles'],canonical_source=ref['path'],room_sha256=room_hash,symbol_blocks=symbols,closed_reachable=len(closed),open_reachable=len(opened),walkthrough=gym['walkthrough'],passive_arrival=room['passive_arrival'],native_gameplay_proved=False)


def check(design,layout,pack,blocks):
    width,height=layout['dimensions_metatiles'];c=pack['data']['tilesets']['kanto']['collision']
    def kind(at):
        x,y=at;return c[f'{blocks[y//2*width+x//2]:02x}'][y%2*2+x%2]
    actors={tuple(a['tile']) for a in design['residents']}
    errors=[]
    def text_width(value,path='town'):
        if isinstance(value,dict):
            for key,child in value.items():
                if key.endswith('lines') and any(len(line)>18 for line in child):errors.append(path+': native text line exceeds18 columns')
                else:text_width(child,path+'/'+key)
        elif isinstance(value,list):
            for index,child in enumerate(value):text_width(child,path+'/'+str(index))
    text_width(design)
    if len(actors)!=len(design['residents']):errors.append('Exterior actor overlap')
    if any('COURIER' in a['id'] for a in design['residents']):errors.append('Q03 courier must remain in Root-owned depot')
    for a in layout['layers']:
        if 'native facade' in a['name'] and blocks[a['position'][1]*width+a['position'][0]]!=int(a['block'],16):errors.append(a['name']+': native facade overwritten')
        if a['kind'] not in {'polygon','cliff_polygon','path'}:continue
        points=a['points'];edges=zip(points,points[1:]+points[:1]) if a['kind']!='path' else zip(points,points[1:])
        if not all(a[0]==b[0] or a[1]==b[1] for a,b in edges):errors.append(a['name']+': nonorthogonal stroke')
    walk={(x,y) for y in range(height*2) for x in range(width*2) if kind((x,y)) in {'FLOOR','LADDER','TALL_GRASS'} and (x,y) not in actors}
    seen=flood((6,24),walk)
    for door in design['doors']:
        if kind(tuple(door['tile']))!='DOOR':errors.append(door['id']+': missing native DOOR')
        if tuple(door['return_tile']) not in seen:errors.append(door['id']+': unsafe/unreachable return')
    for actor in design['residents']:
        at=tuple(actor['tile'])
        if kind(at)!='FLOOR' or not any(q in seen for q in neighbors(at)):errors.append(actor['id']+': actor footing/approach')
    for sign in design['signs']:
        at=tuple(sign['tile'])
        if kind(at)!='WALL' or not any(q in seen for q in neighbors(at)):errors.append(sign['id']+': sign footing/approach')
    for view in design['viewpoints']:
        if tuple(view['tile']) not in seen:errors.append(view['id']+': not ordinary reachable Floor')
    for at in [(0,24),(0,25),(6,24),(6,25),(16,6),(17,6)]:
        if at not in seen:errors.append('Public street anchor blocked '+str(at))
    for at in [(x,y) for x in range(42,48) for y in [16,17]]+[(46,18),(47,18)]:
        if at not in seen or kind(at)!='FLOOR':errors.append('Upper civic street shoulder encroaches '+str(at))
    guard={tuple(p) for p in design['fairgrounds_port']['closed_gate_core']}
    for at in [(16,0),(17,0)]:
        if at in seen:errors.append('Unbound fair gate has a free walk-around')
        if at not in flood((6,24),walk|guard):errors.append('Reserved fair street is not a real future walking seam')
    waters={(x,y) for y in range(height*2) for x in range(width*2) if kind((x,y)).startswith('WATER')}
    boundary={(x,y) for x,y in waters if x in {0,width*2-1} or y in {0,height*2-1}}
    if boundary:errors.append('Unconnected outer Surf water '+str(sorted(boundary)))
    cut=[(x,y) for y in range(height*2) for x in range(width*2) if kind((x,y))=='CUT_TREE']
    if cut:errors.append('Incidental Cut '+str(cut))
    drop=design['one_way_ledges'][0];approach,landing=tuple(drop['approach']),tuple(drop['landing'])
    if kind((39,20))!='HOP_DOWN' or kind((39,21))!='WALL' or approach not in seen or landing not in seen:errors.append('Bluff drop/landing is not an actual native one-way bank')
    route=shortest_walk(landing,approach,walk)
    if not route or len(route)<10:errors.append('Bluff ordinary return is disconnected or trivial')
    rooms=[]
    for room in design['interiors']:
        rooms.append(check_room(room,pack,errors))
    slots={v for door in design['doors'] for v in [door['warp_index'],door['return_warp_index']]}
    if slots!=set(range(1,11)):errors.append('Exterior active/passive warp slots must be distinct1..10')
    for door in design['doors']:
        room=next((r for r in design['interiors'] if r['id']==door['target']),None)
        if room and (room['passive_arrival']['index']!=door['target_warp'] or any(w['target_warp']!=door['return_warp_index'] for w in room['exit_warps'])):errors.append(door['id']+': reciprocal room warp indices differ')
    depot=json.loads((ROOT/design['depot_binding']['source']).read_text());binding=design['depot_binding']
    if depot['public_arrival_core']!=binding['interior_public_arrival_core'] or depot['warp_contract']['passive_arrival_warp_index']!=5 or depot['warp_contract']['exterior_passive_return_warp_index']!=2:errors.append('Root-owned depot reciprocal source ordering differs')
    if next(d for d in design['doors'] if d['target']=='MnStPaulUnionDepot')['target_warp']!=5:errors.append('Depot door must target public passive5, never carpet or train door')
    station=pack['data']['tilesets']['train_station']['collision'];rows=[[int(v,16) for v in row.split()] for row in depot['rows']]
    def station_kind(at):
        x,y=at;return station[f'{rows[y//2][x//2]:02x}'][y%2*2+x%2]
    if station_kind((15,21))!='FLOOR' or any(station_kind(tuple(p))!='WARP_CARPET_DOWN' for p in depot['exit_tiles_core']):errors.append('Root depot public Floor/carpet source collision differs')
    return dict(source_only=True,errors=errors,reachable_walking_tiles=len(seen),doors_checked=len(design['doors']),outer_water=len(boundary),ordinary_actors=len(actors),fairgrounds_status='Reserved source street only; native connection unbound',rooms_with_rows=sum('rows'in r for r in design['interiors']),canonical_room_references=sum('source_reference'in r for r in design['interiors']),room_geometry=rooms,bluff_ordinary_return_steps=len(route)-1 if route else None,native_gameplay_proved=False)


def check_room(room,pack,errors):
    if 'source_reference' in room:return check_canonical_gym(room,pack,errors)
    w,h=room['dimensions_metatiles'];rows=[[int(v,16) for v in row.split()] for row in room['rows']];name=room['id'];c=pack['data']['tilesets'][room['tileset']]['collision']
    if len(rows)!=h or any(len(row)!=w for row in rows):errors.append(name+': room rows/dimensions');return {}
    def kind(at):
        x,y=at;return c[f'{rows[y//2][x//2]:02x}'][y%2*2+x%2]
    actors={tuple(a['tile']) for a in room['residents']};allowed={(x,y) for y in range(h*2) for x in range(w*2) if kind((x,y)) in {'FLOOR','LADDER','WARP_CARPET_DOWN'} and (x,y) not in actors}
    arrival=tuple(room['passive_arrival']['tile']);seen=flood(arrival,allowed)
    if kind(arrival)!='FLOOR' or arrival in actors:errors.append(name+': unsafe passive Floor arrival')
    for at in room['exits']:
        if kind(tuple(at))!='WARP_CARPET_DOWN' or tuple(at) not in seen:errors.append(name+': native carpet unreachable')
    for a in room['residents']:
        at=tuple(a['tile'])
        if kind(at)!='FLOOR':errors.append(name+'/'+a['id']+': actor needs Floor')
        approach=a.get('interaction_approach')
        if approach:
            if tuple(approach) not in seen or kind(tuple(a['native_counter']))!='COUNTER':errors.append(name+'/'+a['id']+': native counter interaction blocked')
        elif not any(p in seen for p in neighbors(at)):errors.append(name+'/'+a['id']+': no actor approach')
    for sign in room.get('signs',[]):
        if kind(tuple(sign['tile']))!='WALL' or tuple(sign['approach']) not in seen:errors.append(name+'/'+sign['id']+': native board/approach missing')
    if 'pc' in room:
        pc=room['pc']
        if kind(tuple(pc['tile']))!='PC' or tuple(pc['approach']) not in seen:errors.append(name+': real PC tile/approach missing')
        if any(kind((x,y))=='LADDER' for y in range(h*2) for x in range(w*2)):errors.append(name+': unbound clinic stair remains')
    return dict(id=name,dimensions=[w,h],reachable_tiles=len(seen),active_carpets=len(room['exits']),passive_arrival=room['passive_arrival'])


def self_test(design,layout,pack):
    """Reject valid-schema semantic defects without mutating authoritative data."""
    cases=[]
    def tile(name,at,block,diagnostic):
        def mutate(d,l):l['layers'].append(dict(kind='point',name='NEGATIVE '+name,position=at,block=block))
        cases.append((name,mutate,diagnostic))
    tile('depot door erased',[22,18],'01','missing native DOOR')
    tile('western public street closed',[0,12],'13','Public street anchor blocked')
    tile('future fair street erased',[8,0],'13','not a real future walking seam')
    tile('bluff ledge faked with Floor',[19,10],'01','actual native one-way bank')
    tile('bluff landing buried',[19,11],'0f','actual native one-way bank')
    tile('Gym roof overwritten by grove',[25,6],'0f','native facade overwritten')
    tile('upper civic walking lane overwritten by grove',[23,8],'0f','Upper civic street shoulder encroaches')
    cases.extend([
        ('passive return placed on active depot Door',lambda d,l:d['doors'][-1].update(return_tile=[44,37]),'unsafe/unreachable return'),
        ('depot targets active carpet',lambda d,l:d['doors'][-1].update(target_warp=1),'must target public passive5'),
        ('fair guard removed',lambda d,l:d['residents'].pop(-1),'free walk-around'),
        ('actor body overlap',lambda d,l:d['residents'][1].update(tile=d['residents'][0]['tile']),'Exterior actor overlap'),
        ('duplicate exterior Q03 courier',lambda d,l:d['residents'][0].update(id='MN_CITY_COURIER'),'must remain in Root-owned depot'),
        ('clinic restores unbound upstairs stair',lambda d,l:d['interiors'][0]['rows'].__setitem__(3,'12 11 27 2e 2f'),'unbound clinic stair'),
        ('PC label targets ordinary Floor',lambda d,l:d['interiors'][0]['pc'].update(tile=[8,1]),'real PC tile/approach'),
        ('room arrival targets active carpet',lambda d,l:d['interiors'][1]['passive_arrival'].update(tile=[3,7]),'unsafe passive Floor'),
        ('canonical gym digest altered',lambda d,l:d['interiors'][3]['source_reference'].update(room_sha256='0'*64),'canonical gym source hash differs'),
        ('reciprocal room return points to another door',lambda d,l:d['interiors'][1]['exit_warps'][0].update(target_warp=2),'reciprocal room warp indices'),
        ('competing gym dimensions',lambda d,l:d['interiors'][3].update(dimensions_metatiles=[8,8]),'competing canonical gym geometry'),
        ('canonical native glyph mapping altered',lambda d,l:d['interiors'][3]['source_reference']['symbol_blocks'].update(X='01'),'native glyph allocation differs'),
        ('canonical arrival targets old alternative room',lambda d,l:d['interiors'][3]['passive_arrival'].update(tile=[7,13]),'canonical carpet/passive anchors differ'),
        ('dialogue exceeds handheld text width',lambda d,l:d['residents'][0]['lines'].append('A deliberately oversized text line'),'native text line exceeds18'),
    ])
    def open_water(d,l):l['layers'].append(dict(name='NEGATIVE outer Surf water',kind='rect',material='water',bounds=[30,22,32,24]))
    cases.append(('unconnected outer Surf water',open_water,'Unconnected outer Surf water'))
    good=check(design,layout,pack,layout_blocks(layout));assert not good['errors'],good['errors']
    rejected=[]
    for name,mutate,diagnostic in cases:
        d,l=deepcopy(design),deepcopy(layout);mutate(d,l);errors=check(d,l,pack,layout_blocks(l))['errors']
        assert any(diagnostic in error for error in errors),(name,errors)
        rejected.append(dict(name=name,diagnostic=diagnostic))
    return dict(source_only=True,negative_cases=rejected)


def render(design,layout,pack,art):
    start=time.perf_counter();blocks=layout_blocks(layout);report=check(design,layout,pack,blocks)
    terrain=art.render('kanto',32,24,blocks);save_render(terrain,OUTPUT/'terrain-original.png')
    scene=render_stage(terrain,'convert',OUTPUT,'town-terrain-to-scene','RGBA')
    for a in design['residents']:scene.alpha_composite(oriented_sprite(pack,a['sprite'],a['facing']),(a['tile'][0]*16,a['tile'][1]*16-4))
    key=hashlib.sha256(scene.tobytes()).hexdigest()[:8];save_render(scene,OUTPUT/f'whole-town-original-rgba-{key}.png')
    rgb=render_stage(scene,'convert',OUTPUT,'whole-scene-'+key,'RGB');save_render(rgb,OUTPUT/f'whole-town-{key}.png');save_render(rgb,OUTPUT/'whole-town.png')
    views=[]
    for view in design['viewpoints']:
        x,y=view['tile'];name=view['id'].lower();crop=render_stage(scene,'crop',OUTPUT,name+'-'+key,(x*16-80,y*16-72,x*16+80,y*16+72));save_render(crop,OUTPUT/f'camera-{name}-original-rgba-{key}-160.png');rgb=render_stage(crop,'convert',OUTPUT,name+'-'+key,'RGB');save_render(rgb,OUTPUT/f'camera-{name}-{key}-160.png');preview=render_stage(rgb,'resize',OUTPUT,name+'-'+key+'-4x',(640,576),Image.Resampling.NEAREST);save_render(preview,OUTPUT/f'viewport-{name}-{key}.png');views.append((view['id'],rgb))
    save_render(room_sheet(views,columns=2),OUTPUT/f'viewpoints-{key}.png')
    room_views=[];room_key=key+'-'+hashlib.sha256(DESIGN.read_bytes()).hexdigest()[:8]
    for room in design['interiors']:
        w,h=room['dimensions_metatiles'];gym=canonical_gym(room)[0] if 'source_reference' in room else None
        for phase in ['closed','open'] if gym else ['ordinary']:
            name=room['id']+('-'+phase if gym else '')
            if gym:
                rows=[list(row) for row in gym['rows']]
                if phase=='open':
                    for gate in gym['gates']:rows[gate['at'][1]][gate['at'][0]]=gate['open_symbol']
                raw=Image.new('RGB',(w*32,h*32))
                for y,row in enumerate(rows):
                    for x,symbol in enumerate(row):
                        source,block=gym['art'][symbol];raw.paste(art.blocks(source,'indoor')[int(block,16)],(x*32,y*32))
                actors=gym_actors(gym)
            else:
                blocks=[int(v,16) for row in room['rows'] for v in row.split()];raw=art.render(room['tileset'],w,h,blocks);actors=room['residents']
            save_render(raw,OUTPUT/f'{name}-terrain-original-{room_key}.png');im=render_stage(raw,'convert',OUTPUT,name+'-scene-'+room_key,'RGBA')
            for a in actors:im.alpha_composite(oriented_sprite(pack,a['sprite'],a['facing']),(a['tile'][0]*16,a['tile'][1]*16-4))
            save_render(im,OUTPUT/f'{name}-original-rgba-{room_key}.png');rgb=render_stage(im,'convert',OUTPUT,name+'-'+room_key,'RGB');save_render(rgb,OUTPUT/f'{name}-{room_key}.png');room_views.append((name,rgb))
            camera_scene=im;offset=0
            if gym:
                source,block=gym['art']['#'];border=art.blocks(source,'indoor')[int(block,16)];camera_scene=Image.new('RGBA',(w*32+192,h*32+192))
                for py in range(0,camera_scene.height,32):
                    for px in range(0,camera_scene.width,32):camera_scene.paste(border,(px,py))
                save_render(camera_scene,OUTPUT/f'{name}-camera-native-border-original-{room_key}.png');camera_scene.alpha_composite(im,(96,96));save_render(camera_scene,OUTPUT/f'{name}-camera-padded-scene-original-{room_key}.png');offset=96
            cameras=[('arrival',room['passive_arrival']['tile'])]+([('leader',gym['leader_at']),('west-pad',[gym['lifts'][0]['ends_core'][0][0],gym['lifts'][0]['ends_core'][0][1]-1]),('main-pad',[gym['lifts'][3]['ends_core'][0][0],gym['lifts'][3]['ends_core'][0][1]-1]),('court-grille',[11,6])] if gym else [])
            for label,(x,y) in cameras:
                camera=render_stage(camera_scene,'crop',OUTPUT,name+'-'+label+'-'+room_key,(x*16-80+offset,y*16-72+offset,x*16+80+offset,y*16+72+offset));save_render(camera,OUTPUT/f'{name}-{label}-original-rgba-{room_key}-160.png');camera=render_stage(camera,'convert',OUTPUT,name+'-'+label+'-'+room_key,'RGB');save_render(camera,OUTPUT/f'{name}-{label}-{room_key}-160.png')
    save_render(room_sheet(room_views,columns=2),OUTPUT/f'rooms-{room_key}.png')
    report.update(image=f'whole-town-{key}.png',room_sheet=f'rooms-{room_key}.png',hash=key,layout_sha256=hashlib.sha256(LAYOUT.read_bytes()).hexdigest(),dossier_sha256=hashlib.sha256(DESIGN.read_bytes()).hexdigest(),render_ms=round((time.perf_counter()-start)*1000,1),status='Source whole-town candidate; native town/services/story/rail integration remain pending')
    OUTPUT.mkdir(parents=True,exist_ok=True);(OUTPUT/'revision.json').write_text(json.dumps(report,indent=2)+'\n');print(json.dumps(report),flush=True)
    return report


def verify_published(pack, design, layout):
    """Compare published geometry, both facade bindings and every internal pad."""
    maps=pack['data']['maps'];city=maps[design['id']]
    assert city['blocks']==layout_blocks(layout), 'published city terrain differs'
    for door in design['doors']:
        room=maps[door['target']]
        enter=city['events']['warps'][door['warp_index']-1]
        back=city['events']['warps'][door['return_warp_index']-1]
        assert [enter['x'],enter['y']]==door['tile'] and enter['target_map_constant']==door['target_constant'] and enter['target_warp_id']==door['target_warp']
        assert [back['x'],back['y']]==door['return_tile']
        count=4 if door['target']=='MnStPaulUnionDepot' else 2
        for warp in room['events']['warps'][:count+1]:
            assert warp['target_map_constant']=='MN_ST_PAUL' and warp['target_warp_id']==door['return_warp_index'], 'carpet/passive city return differs'
    rooms=list(design['interiors'])+[json.loads((ROOT/design['depot_binding']['source']).read_text())]
    for room in rooms:
        native=maps[room['id']]
        assert [native['attributes']['width'],native['attributes']['height']]==room['dimensions_metatiles']
        if 'source_reference' in room:
            gym,symbols,_,_=canonical_gym(room)
            assert native['blocks']==[int(symbols[c],16) for row in gym['rows'] for c in row]
            assert len(native['events']['warps'])==3+2*len(gym['lifts'])
            for i,lift in enumerate(gym['lifts']):
                for side,at in enumerate(lift['ends_core']):
                    warp=native['events']['warps'][3+i*2+side]
                    assert [warp['x'],warp['y']]==at and warp['target_map_constant']==gym['constant'] and warp['target_warp_id']==4+i*2+(1-side), 'internal pad was rebound'
        else:
            assert native['blocks']==[int(b,16) for row in room['rows'] for b in row.split()]
            assert native['attributes']['tileset_name']==room['tileset']
            assert {(a['script'],a['x'],a['y']) for a in native['objects']}=={(a['id'],*a['tile']) for a in room['residents']}
    links=city['attributes']['connections'];route=maps['MnR04C00R00']
    assert links==[dict(direction='west',target_map='MnR04C00R00',offset=3)]
    assert dict(direction='east',target_map='MnStPaul',offset=-3) in route['attributes']['connections']
    depot=maps['MnStPaulUnionDepot'];collision=pack['data']['tilesets']['train_station']['collision']
    bodies={(a['x'],a['y']) for a in depot['objects']}
    allowed={(x,y) for y in range(24) for x in range(32) if (x,y) not in bodies and collision[f"{depot['blocks'][y//2*16+x//2]:02x}"][y%2*2+x%2] in {'FLOOR','DOOR','LADDER','WARP_CARPET_DOWN'}}
    public=flood((15,21),allowed)
    assert not ({(17,7),(12,5),(17,5)} & public), 'unfinished platform/train doors must stay sealed'
    print('Published St Paul: five real facades, public depot arrival5, canonical Gym4 and all four internal pad pairs, reciprocal R04 seam match source.')


def main():
    parser=argparse.ArgumentParser();parser.add_argument('--pack',type=Path,default=ROOT/'content-packs/minnesota-geography-review.crystalpack');parser.add_argument('--watch',action='store_true');parser.add_argument('--self-test',action='store_true');parser.add_argument('--verify-published',action='store_true');args=parser.parse_args();pack=load_pack(args.pack);art=SourceArt(pack);last=None
    if args.verify_published:
        verify_published(pack,json.loads(DESIGN.read_text()),json.loads(LAYOUT.read_text()))
    if args.self_test:
        print(json.dumps(self_test(json.loads(DESIGN.read_text()),json.loads(LAYOUT.read_text()),pack)));return
    while True:
        stamp=(DESIGN.stat().st_mtime_ns,LAYOUT.stat().st_mtime_ns)
        if stamp!=last:
            report=render(json.loads(DESIGN.read_text()),json.loads(LAYOUT.read_text()),pack,art);last=stamp
            if report['errors'] and not args.watch:raise SystemExit(1)
        if not args.watch:break
        time.sleep(.2)


if __name__=='__main__':main()
