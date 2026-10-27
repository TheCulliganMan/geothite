#!/usr/bin/env python3
"""Render and inspect the explicit Bemidji SOURCE candidate; never publish packs."""
import argparse
from copy import deepcopy
from functools import partial
from http.server import SimpleHTTPRequestHandler, ThreadingHTTPServer
import threading
import hashlib
import io
import json
from pathlib import Path
import time
from PIL import Image
from ledge_paths import inspect_ledges, shortest_walk
from render_archive import save_render
from render_reference import SourceArt, layout_blocks, load_pack, room_sheet
from render_stages import render_stage
from review_boundary_basecamp import adjacent, flood
from review_grand_marais import sprite

ROOT=Path(__file__).resolve().parents[3]
DESIGN=ROOT/'planning/minnesota/towns/MnBemidji.json'
LAYOUT=ROOT/'planning/minnesota/towns/MnBemidji.layout.json'
OUTPUT=ROOT/'planning/minnesota/renders/bemidji'
GYMS=ROOT/'planning/minnesota/gyms/interiors.json'

def native_actor(pack,a):
    token=a['sprite'];facing=a.get('facing','south');target=OUTPUT/'sprites'/token.lower()
    image=sprite(pack,token,archive_output=target)
    source=Image.open(io.BytesIO(bytes(pack['runtime_files']['gfx/sprites/'+token.removeprefix('SPRITE_').lower()+'.png'])))
    save_render(source,target/'original-full-sheet.png')
    sheet=render_stage(source,'convert',target,'full-sheet-rgb','RGB')
    offset=16 if facing=='north' else 32 if facing in {'west','east'} else 0
    if offset:
        first=render_stage(sheet,'crop',target,'front-frame',(0,0,16,16))
        colors={gray[0]:pixel for gray,pixel in zip(first.getdata(),image.getdata())}
        frame=render_stage(sheet,'crop',target,facing+'-frame',(0,offset,16,offset+16))
        image=render_stage(frame,'convert',target,facing+'-rgba','RGBA')
        image.putdata([colors[p[0]] for p in image.getdata()]);save_render(image,target/(facing+'-palette.png'))
    if facing=='east':
        save_render(image,target/'east-before-flip.png');image=image.transpose(Image.Transpose.FLIP_LEFT_RIGHT);save_render(image,target/'east-after-flip.png')
    save_render(image,target/(facing+'-final.png'));return image

def collision(pack,tileset,w,bs):
    cs=pack['data']['tilesets'][tileset]['collision']
    return lambda p:cs[f'{bs[p[1]//2*w+p[0]//2]:02x}'][p[1]%2*2+p[0]%2]

def prepare(p,l):
    definition=deepcopy(p['data']['tilesets'][l['tileset']])
    for r in l['derived_blocks']:
        definition['collision'][r['id']]=list(p['data']['tilesets'][r['collision_source_tileset']]['collision'][r['collision_source_block']])
    p['data']['tilesets'][l['compiled_tileset']]=definition
    gym=next(g for g in json.loads(GYMS.read_text())['gyms'] if g['id']=='MnBemidjiGym')
    needed=set(''.join(gym['rows']))|{gate['open_symbol'] for gate in gym['gates']}|{gate['replacement_symbol'] for gate in gym.get('cut_gates',[])}
    definition=deepcopy(p['data']['tilesets']['elite_four_room']);definition['collision']={}
    for i,char in enumerate(sorted(needed)):
        source,block=gym['art'][char];definition['collision'][f'{i:02x}']=list(p['data']['tilesets'][source]['collision'][block])
    p['data']['tilesets']['mn_gym_7']=definition

def checks(d,l,p):
    prepare(p,l)
    assert d['id']==l['id']=='MnBemidji'
    assert d['dimensions_metatiles']==l['dimensions_metatiles']==[24,24]
    assert l['tileset']=='johto_modern' and not l['source_pixels_modified']
    assert l['derived_blocks']==[dict(id=f'{0x80+i:02x}',source_tileset='johto',source_block=b,collision_source_tileset='johto',collision_source_block=b) for i,b in enumerate(['22','23','28','29'])], 'timber facade source art drift'
    bs=layout_blocks(l);k=collision(p,l['compiled_tileset'],24,bs);cells={(x,y) for y in range(48) for x in range(48)}
    actors={tuple(a['tile']) for a in d['residents']};doors={tuple(a['tile']) for a in d['doors']}
    walk={v for v in cells if k(v) in {'FLOOR','DOOR','TALL_GRASS'}};water={v for v in cells if k(v)=='WATER'}
    seen=flood((24,40),walk-actors,doors);surf=flood((24,40),(walk|water)-actors,doors)
    expected=[('MN_R08','S',[12,22],[12,20]),('MN_R09','E',[22,12],[20,12]),('MN_R14','W',[1,12],[3,12]),('MN_R17','N',[12,1],[12,3])]
    assert len(d['ports'])==4
    for port,(route,side,gate,landing) in zip(d['ports'],expected):
        assert (port['route'],port['side'],port['gate_metatile'],port['landing_metatile'])==(route,side,gate,landing),'reserved port changed'
        assert all(tuple(2*x for x in tile) in seen for tile in [gate,landing]),route+' disconnected'
    assert doors=={v for v in cells if k(v)=='DOOR'},'unbound facade DOOR'
    assert len(doors)==len(d['doors'])==6
    assert [(a['warp_index'],a['return_warp_index']) for a in d['doors']]==[(2*i+1,2*i+2) for i in range(6)], 'active/passive exterior warp order'
    for a in d['doors']:
        pos=tuple(a['tile']);ret=tuple(a['return_tile']);assert pos in seen,a['id']+' disconnected'
        assert ret in seen and k(ret)=='FLOOR',a['id']+' return unsafe'
        assert a['return_tile']==a['arrival_tile']==[pos[0],pos[1]+1] and a['target_warp']==3
    assert not walk-seen-actors,('isolated public walking island',sorted(walk-seen-actors))
    for a in d['residents']:
        assert k(tuple(a['tile']))=='FLOOR',a['id']+' off FLOOR'
        assert set(adjacent(tuple(a['tile'])))&seen,a['id']+' inaccessible'
    for a in d['signs']:
        assert k(tuple(a['tile']))=='WALL' and set(adjacent(tuple(a['tile'])))&seen,a['id']+' sign inaccessible'
    assert all(0<len(line)<=18 for a in d['residents']+d['signs'] for line in a['lines']),'text width'
    assert not any(k(v)=='CUT_TREE' for v in cells),'incidental Cut'
    edge={v for v in cells if v[0] in {0,47} or v[1] in {0,47}}
    assert not surf&water&edge,'unconnected Surf boundary escape'
    assert seen&edge=={(0,24),(0,25),(47,24),(47,25),(24,0),(25,0),(24,47),(25,47)},('unexpected walking exit',seen&edge)
    shore={(a,b) for a in walk if k(a)=='FLOOR' for b in adjacent(a) if b in water}
    approved={(tuple(e['bank']),tuple(e['water'])) for e in d['lakefront']['allowed_surf_edges']}
    assert shore==approved,('unplanned shore launch',sorted(shore^approved))
    for key in ['pier_approach','pier_end','launch_tile']:assert tuple(d['lakefront'][key]) in seen,key+' disconnected'
    assert tuple(d['lakefront']['water_target']) in surf&water
    for x,y in d['lakefront']['north_rail_blocks']:assert [k((2*x,2*y)),k((2*x,2*y+1))]==['WALL','FLOOR'],'pier north rail hole'
    for x,y in d['lakefront']['south_rail_blocks']:assert [k((2*x,2*y)),k((2*x,2*y+1))]==['FLOOR','WALL'],'pier south rail hole'
    ledges=inspect_ledges(d,lambda x,y:k((x,y)),seen)
    hops=set()
    for a in d['one_way_ledges']:
        assert k(tuple(a['hop_core']))=='HOP_DOWN' and k(tuple(a['reverse_blocked']))=='WALL','ledge reverse opened'
        for x,y in a['source_blocks']:
            for dx in [0,1]:
                upper=(2*x+dx,2*y-1);hop=(2*x+dx,2*y);land=(2*x+dx,2*y+2);hops.add(hop)
                assert upper in seen and land in seen and k(upper)=='FLOOR' and k(land)=='FLOOR' and k(hop)=='HOP_DOWN' and k((hop[0],hop[1]+1))=='WALL','unusable ledge quadrant'
    assert {v for v in cells if k(v).startswith('HOP_')}==hops,'undeclared hop'
    assert all(tuple(v['tile']) in seen for v in d['viewpoints']),'camera not legal walking tile'
    officer=next(a for a in d['residents'] if a['id']=='MN_BEMIDJI_OFFICER')
    assert officer['tile']==[21,19] and officer['requires']==['BADGE_7','EVENT_MN_HEADWATERS_READING'] and officer['sets']==['EVENT_MN_HEADWATERS_REPORT'],'Q05 contract changed'
    room_reports=[]
    assert {r['id'] for r in d['interiors']}=={a['target'] for a in d['doors']}
    for r in d['interiors']:
        w,h=r['dimensions_metatiles'];rows=[row.split() for row in r['rows']];assert len(rows)==h and all(len(row)==w for row in rows),r['id']+' rows'
        rb=[int(v,16) for row in rows for v in row];rk=collision(p,r['tileset'],w,rb);rc={(x,y) for y in range(h*2) for x in range(w*2)}
        ra={tuple(a['tile']) for a in r['residents']};rw={v for v in rc if rk(v) in {'FLOOR','WARP_CARPET_DOWN'}}
        arrival=tuple(r['passive_arrival']['tile']);assert r['passive_arrival']['index']==3 and rk(arrival)=='FLOOR',r['id']+' unsafe passive arrival'
        reach=flood(arrival,rw-ra);open_reach=reach
        if r['id']=='MnBemidjiGym':
            canonical=next(g for g in json.loads(GYMS.read_text())['gyms'] if g['id']==r['id'])
            signature=hashlib.sha256(json.dumps(canonical,sort_keys=True,separators=(',',':')).encode()).hexdigest()
            assert signature==r['canonical_gym_sha256'], 'canonical gym changed; refresh city preview'
            needed=set(''.join(canonical['rows']))|{gate['open_symbol'] for gate in canonical['gates']}|{gate['replacement_symbol'] for gate in canonical.get('cut_gates',[])};symbols={c:i for i,c in enumerate(sorted(needed))}
            assert r['tileset']=='mn_gym_7' and r['rows']==[' '.join(f'{symbols[c]:02x}' for c in row) for row in canonical['rows']], 'canonical gym terrain drift'
            assert r['dimensions_metatiles']==canonical['dimensions_metatiles'] and list(arrival)==canonical['arrival_core'] and [e['tile'] for e in r['exits']]==canonical['exit_tiles_core'], 'canonical gym extent/arrival drift'
            expected=[('MnBemidjiGymLeader',canonical['leader_sprite'],canonical['leader_at']),('MnBemidjiGymGuide','SPRITE_GYM_GUIDE',canonical['guide_at'])]+[(a['id'],a['sprite'],a['at']) for a in canonical['trainers']]+[(r['id']+a['id']+'Station',a['sprite'],a['at']) for a in canonical['stations']]
            assert sorted((a['id'],a['sprite'],a['tile']) for a in r['residents'])==sorted(expected), 'canonical gym actor drift'
            leader=next(a for a in r['residents'] if a['id']=='MnBemidjiGymLeader')
            assert leader['party']==canonical['party'] and leader['trainer_class']==canonical['trainer_class'] and leader['requires']==canonical['requires'] and leader['reward']==canonical['reward'], 'canonical leader contract drift'
            for trainer in canonical['trainers']:
                a=next(a for a in r['residents'] if a['id']==trainer['id'])
                assert a['party']==trainer['party'] and a['trainer_class']==trainer['trainer_class'] and a['optional']==trainer['optional'] and a['defeat_flag']=='EVENT_'+trainer['id']+'_DEFEATED', 'canonical trainer contract drift'
            for station in canonical['stations']:
                a=next(a for a in r['residents'] if a['id']==r['id']+station['id']+'Station')
                assert (a['requires'],a['options'],a['answer'],a['sets'],a['native_canonical_prompt'])==(station['requires'],station['options'],station['answer'],[station['event']],station['prompt']), 'canonical station contract drift'
            guide=next(a for a in r['residents'] if a['id']=='MnBemidjiGymGuide')
            assert guide['native_canonical_lines']==canonical['guide'], 'canonical guide contract drift'
            assert r['cut_contracts']==canonical['cut_gates'] and r['court_requires']==canonical['court_requires'], 'canonical Cut or battle qualification drift'
            assert not canonical['gates'] and len(canonical['cut_gates'])==2 and canonical['leader_court_locked'], 'two native Cut hedges'
            rows=[list(row) for row in canonical['rows']]
            def phase_walk():
                floor=set()
                for y in range(h*2):
                    for x in range(w*2):
                        source,block=canonical['art'][rows[y//2][x//2]]
                        if p['data']['tilesets'][source]['collision'][block][y%2*2+x%2] in {'FLOOR','WARP_CARPET_DOWN'}:floor.add((x,y))
                return flood(arrival,floor-ra)
            assert phase_walk()==reach and not set(adjacent(tuple(leader['tile'])))&reach, 'canonical initial court bypass'
            for gate in canonical['cut_gates']:
                assert tuple(gate['approach_core']) in phase_walk(),'unreachable native Cut approach'
                x,y=gate['tree_core'];source,block=canonical['art'][rows[y//2][x//2]];replacement=canonical['art'][gate['replacement_symbol']]
                native=p['data']['field_moves']['cut']['replacements'][source][int(block,16)]
                assert native['variant']=='tree' and replacement==[source,f'{native["replacement_block_id"]:02x}'],'native Cut replacement drift'
                assert p['data']['tilesets'][source]['collision'][block][y%2*2+x%2]=='CUT_TREE','missing Cut quadrant'
                rows[y//2][x//2]=gate['replacement_symbol']
                assert set(map(tuple,gate['targets_core']))<=phase_walk(),'Cut target remains sealed'
            open_reach=phase_walk()
            assert set(adjacent(tuple(leader['tile'])))&open_reach, 'canonical completion state blocks leader'
        exits={tuple(a['tile']) for a in r['exits']};assert exits=={v for v in rc if rk(v)=='WARP_CARPET_DOWN'} and len(exits)==2 and exits<=reach,r['id']+' carpets'
        ext=next(a for a in d['doors'] if a['target']==r['id']);assert all(a['target']=='MnBemidji' and a['target_warp']==ext['return_warp_index'] for a in r['exits']),r['id']+' wrong room return'
        outer={v for v in rc if v[0] in {0,w*2-1} or v[1] in {0,h*2-1}}
        if r['id']=='MnBemidjiGym':
            assert all(rk(v) in {'FLOOR','WARP_CARPET_DOWN'} for v in reach&outer),'native greenhouse floor edge'
        else:assert reach&outer==exits,r['id']+' room outer floor leak'
        for a in r['residents']:
            assert rk(tuple(a['tile']))=='FLOOR',a['id']+' actor off FLOOR'
            assert all(0<len(line)<=18 for line in a['lines']),a['id']+' text width'
            if 'counter_tile' in a:assert rk(tuple(a['counter_tile']))=='COUNTER' and tuple(a['approach']) in reach,a['id']+' counter approach'
            else:assert set(adjacent(tuple(a['tile'])))&(open_reach if a.get('phase') in {'court_open','after_south_cut','after_both_cuts'} else reach),a['id']+' inaccessible'
            if 'party' in a:
                assert a['trainer_class'] in p['data']['trainer_class_names'],a['id']+' invalid trainer class'
                assert all(m['species'] in p['data']['pokemon'] and 1<=m['level']<=100 for m in a['party']),a['id']+' invalid party'
            if 'stock' in a:assert all(v in p['data']['items'] for v in a['stock'])
        for e in r.get('exhibits',[]):
            assert rk(tuple(e['tile']))==e['collision'] and tuple(e['approach']) in reach,e['id']+' exhibit inaccessible'
            assert all(0<len(line)<=18 for line in e['lines']),e['id']+' exhibit text width'
        if 'pc' in r:assert rk(tuple(r['pc']['tile']))=='PC' and tuple(r['pc']['approach']) in reach,'PC unreachable'
        room_reports.append(dict(id=r['id'],public_tiles=len(reach),passive_arrival=list(arrival),exits=sorted(exits),open_phase_public_tiles=len(open_reach),canonical_authority=r.get('canonical_source')))
    return dict(source_only=True,reachable_walking_tiles=len(seen),reachable_lake_tiles=len(surf&water),doors=6,original_ports_retained=True,ledges=ledges,rooms=room_reports)

def negatives(d,l,p):
    result=[]
    def reject(name,change):
        a,b=deepcopy(d),deepcopy(l);change(a,b)
        try:checks(a,b,p)
        except AssertionError as e:result.append(dict(case=name,rejected_by=str(e)));return
        raise AssertionError('negative accepted: '+name)
    reject('wrong original route landing',lambda a,b:a['ports'][0].update(landing_metatile=[10,20]))
    reject('false facade door',lambda a,b:a['doors'][0].update(tile=[11,9]))
    reject('unsafe return standing inside facade',lambda a,b:a['doors'][0].update(return_tile=[10,9]))
    reject('outer lake rock gap',lambda a,b:b['layers'].append(dict(name='Invalid water edge',kind='point',position=[23,20],block='35')))
    reject('unplanned bank Surf launch',lambda a,b:b['layers'].append(dict(name='Invalid shore rail',kind='point',position=[14,15],block='01')))
    reject('pier north rail hole',lambda a,b:b['layers'].append(dict(name='Invalid pier',kind='point',position=[16,17],block='01')))
    reject('ledge reverse opening',lambda a,b:b['layers'].append(dict(name='Invalid ledge',kind='point',position=[5,10],block='01')))
    reject('wrong room return',lambda a,b:a['interiors'][0]['exits'][0].update(target_warp=1))
    reject('unsafe room passive arrival',lambda a,b:a['interiors'][0]['passive_arrival'].update(tile=[0,0]))
    reject('report without independent reading',lambda a,b:a['residents'][0].update(requires=['BADGE_7']))
    reject('unreviewed timber palette change',lambda a,b:b['derived_blocks'][0].update(palette_override=2))
    reject('active return warp reused',lambda a,b:a['doors'][1].update(return_warp_index=3))
    reject('canonical gym actor drift',lambda a,b:a['interiors'][2]['residents'][0].update(tile=[11,3]))
    reject('canonical Cut gate drift',lambda a,b:a['interiors'][2]['cut_contracts'][0].update(targets_core=[]))
    reject('canonical leader party drift',lambda a,b:a['interiors'][2]['residents'][0]['party'][0].update(level=39))
    if next(g for g in json.loads(GYMS.read_text())['gyms'] if g['id']=='MnBemidjiGym')['stations']:
        reject('canonical station answer drift',lambda a,b:next(v for v in a['interiors'][2]['residents'] if v['id'].endswith('Station')).update(answer=3))
    else:
        reject('invented canonical station body',lambda a,b:a['interiors'][2]['residents'].append(dict(id='MnBemidjiGymInventedStation',sprite='SPRITE_PAPER',tile=[5,14],lines=['Unapproved marker'])))
    return result

def render(d,l,p,skip_checks=False):
    start=time.perf_counter();prepare(p,l)
    snapshots=OUTPUT/'source-snapshots';snapshots.mkdir(parents=True,exist_ok=True)
    for source in [DESIGN,LAYOUT]:
        data=source.read_bytes();(snapshots/(source.stem+'-'+hashlib.sha256(data).hexdigest()+'.json')).write_bytes(data)
    art=SourceArt(p);bs=layout_blocks(l);w,h=l['dimensions_metatiles']
    raw=art.render(l['tileset'],w,h,bs,derived=l['derived_blocks']);save_render(raw,OUTPUT/'terrain-original.png');scene=render_stage(raw,'convert',OUTPUT,'whole-scene-rgba','RGBA')
    for a in d['residents']:scene.alpha_composite(native_actor(p,a),(a['tile'][0]*16,a['tile'][1]*16-4))
    save_render(scene,OUTPUT/'whole-town-original-rgba.png');rgb=render_stage(scene,'convert',OUTPUT,'whole-town-rgb','RGB');digest=hashlib.sha256(rgb.tobytes()).hexdigest()[:10]
    save_render(rgb,OUTPUT/'whole-town.png');save_render(rgb,OUTPUT/f'whole-town-{digest}.png')
    preview=render_stage(rgb,'resize',OUTPUT,'whole-town-preview',(384,384),Image.Resampling.NEAREST);save_render(preview,OUTPUT/'whole-town-half.png')
    views=[]
    for v in d['viewpoints']:
        x,y=v['tile'];x=max(0,min(x*16-80,w*32-160));y=max(0,min(y*16-72,h*32-144))
        im=render_stage(scene,'crop',OUTPUT,v['id'].lower()+'-crop',(x,y,x+160,y+144));save_render(im,OUTPUT/f'viewport-{v["id"].lower()}-native160.png');views.append((v['id'],im))
        zoom=render_stage(im,'resize',OUTPUT,v['id'].lower()+'-4x',(640,576),Image.Resampling.NEAREST);save_render(zoom,OUTPUT/f'viewport-{v["id"].lower()}-4x.png')
    save_render(room_sheet(views,3),OUTPUT/'viewpoints.png');rooms=[]
    for r in d['interiors']:
        rw,rh=r['dimensions_metatiles'];rb=[int(v,16) for row in r['rows'] for v in row.split()]
        if r['id']=='MnBemidjiGym':
            canonical=next(g for g in json.loads(GYMS.read_text())['gyms'] if g['id']==r['id']);im=Image.new('RGB',(rw*32,rh*32))
            for y,row in enumerate(canonical['rows']):
                for x,char in enumerate(row):
                    source,block=canonical['art'][char];im.paste(art.blocks(source,'indoor')[int(block,16)],(x*32,y*32))
        else:im=art.render(r['tileset'],rw,rh,rb)
        save_render(im,OUTPUT/f'room-{r["id"]}-terrain-original.png');im=render_stage(im,'convert',OUTPUT,r['id']+'-rgba','RGBA')
        for a in r['residents']:im.alpha_composite(native_actor(p,a),(a['tile'][0]*16,a['tile'][1]*16-4))
        save_render(im,OUTPUT/f'room-{r["id"]}-original.png');rooms.append((r['id'],im))
        if 'cut_contracts' in r:
            opened=im.copy();save_render(opened,OUTPUT/'gym-open-before-state-art.png')
            for gate in r['cut_contracts']:
                x,y=gate['tree_core'];source,block=canonical['art'][gate['replacement_symbol']]
                opened.paste(art.blocks(source,'indoor')[int(block,16)],(x//2*32,y//2*32))
            save_render(opened,OUTPUT/'gym-open-source-state.png')
            views=[]
            for name,image in [('initial',im),('requirements-complete',opened)]:
                lx,ly=canonical['leader_at'];cx=max(0,min(lx*16-80,rw*32-160));cy=max(0,min(ly*16-72,rh*32-144))
                camera=render_stage(image,'crop',OUTPUT,'gym-'+name+'-gate-camera',(cx,cy,cx+160,cy+144))
                save_render(camera,OUTPUT/('gym-'+name+'-gate-native160.png'));views.append((name+' source state',camera))
            save_render(room_sheet(views,2),OUTPUT/'gym-gate-state-review.png')
    save_render(room_sheet(rooms,3),OUTPUT/'rooms.png')
    report={} if skip_checks else checks(d,l,p)
    if not skip_checks:report['negative_cases']=negatives(d,l,p)
    report.update(canonical_gyms_input=str(GYMS),canonical_gyms_file_sha256=hashlib.sha256(GYMS.read_bytes()).hexdigest(),canonical_gym_station_count=len(next(g for g in json.loads(GYMS.read_text())['gyms'] if g['id']=='MnBemidjiGym')['stations']),implementation_status='Source analysis only. Native publication and controller evidence are recorded separately in gyms/README.md; earned regional progression remains unproved.',whole_town=f'whole-town-{digest}.png',source_pack_content_hash=p['identity']['content_hash'],layout_sha256=hashlib.sha256(LAYOUT.read_bytes()).hexdigest(),dossier_sha256=hashlib.sha256(DESIGN.read_bytes()).hexdigest(),render_seconds=round(time.perf_counter()-start,3))
    payload=(json.dumps(report,indent=2)+'\n').encode();OUTPUT.mkdir(parents=True,exist_ok=True);(OUTPUT/'source-review.json').write_bytes(payload);(OUTPUT/('source-review-'+hashlib.sha256(payload).hexdigest()+'.json')).write_bytes(payload)
    target=OUTPUT/'source-snapshots';target.mkdir(exist_ok=True)
    for source in [DESIGN,LAYOUT]:
        data=source.read_bytes();(target/(source.stem+'-'+hashlib.sha256(data).hexdigest()+'.json')).write_bytes(data)
    return report

def main():
    global GYMS,DESIGN,OUTPUT
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--pack',type=Path,default=ROOT/'content-packs/minnesota-quality-review.crystalpack')
    parser.add_argument('--render-only',action='store_true')
    parser.add_argument('--gyms',type=Path,default=GYMS)
    parser.add_argument('--design',type=Path,default=DESIGN)
    parser.add_argument('--output',type=Path,default=OUTPUT)
    parser.add_argument('--serve',action='store_true')
    parser.add_argument('--port',type=int,default=8797)
    args=parser.parse_args();GYMS=args.gyms;DESIGN=args.design;OUTPUT=args.output;OUTPUT.mkdir(parents=True,exist_ok=True)
    (OUTPUT/'index.html').write_text('<!doctype html><meta charset=utf-8><title>Bemidji source review</title><style>body{background:#14212a;color:#eef;font:16px system-ui;margin:24px}img{image-rendering:pixelated;max-width:100%}</style><h1>Bemidji SOURCE candidate</h1><p>Native publication, services, battles, regional seams and earned progression remain pending. Giant Paul and Babe art is not installed.</p><pre id=reviewstatus></pre><h2>Whole town</h2><img id=town src=whole-town.png><h2>160px cameras</h2><img id=views src=viewpoints.png><h2>All six rooms</h2><img id=rooms src=rooms.png><script>setInterval(async()=>{let t=Date.now();reviewstatus.textContent=await(await fetch("status.txt?"+t)).text();town.src="whole-town.png?"+t;views.src="viewpoints.png?"+t;rooms.src="rooms.png?"+t},1200)</script>')
    if args.serve:
        class Handler(SimpleHTTPRequestHandler):
            def log_message(self,*args):pass
        server=ThreadingHTTPServer(('127.0.0.1',args.port),partial(Handler,directory=str(OUTPUT)))
        threading.Thread(target=server.serve_forever,daemon=True).start()
        print(f'Bemidji review http://127.0.0.1:{server.server_address[1]}/',flush=True)
    pack=load_pack(args.pack)
    previous=None
    while True:
        current=(DESIGN.stat().st_mtime_ns,LAYOUT.stat().st_mtime_ns,GYMS.stat().st_mtime_ns)
        if current!=previous:
            try:
                report=render(json.loads(DESIGN.read_text()),json.loads(LAYOUT.read_text()),pack,args.render_only)
                payload=json.dumps(report,indent=2);(OUTPUT/'status.txt').write_text(payload);print(payload,flush=True)
            except (AssertionError,KeyError,ValueError,IndexError) as error:
                message='DESIGN ERROR: '+str(error);(OUTPUT/'status.txt').write_text(message);print(message,flush=True)
                if not args.serve:raise
            previous=current
        if not args.serve:break
        time.sleep(.3)
if __name__=='__main__':main()
