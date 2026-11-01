#!/usr/bin/env python3
"""Hand-authored free basecamp: source pixels/collision, never native publication."""
import argparse
from collections import deque
from copy import deepcopy
from functools import partial
import hashlib
from http.server import SimpleHTTPRequestHandler, ThreadingHTTPServer
import json
from pathlib import Path
import threading
import time
from PIL import Image
from PIL.PngImagePlugin import PngInfo
from render_archive import save_render as archive_render
from render_reference import SourceArt, layout_blocks, load_pack, room_sheet
from review_north_shore import oriented_sprite

ROOT = Path(__file__).resolve().parents[3]
DESIGN = ROOT/'planning/minnesota/towns/MnBoundaryWaters.json'
LAYOUT = ROOT/'planning/minnesota/towns/MnBoundaryWaters.layout.json'
OUTPUT = ROOT/'planning/minnesota/renders/boundary-basecamp'
HTML = '''<!doctype html><meta charset="utf-8"><title>Boundary Waters basecamp</title><style>body{background:#14212a;color:#edf4e9;font:16px system-ui;margin:24px}img{image-rendering:pixelated;max-width:100%}p{max-width:80ch}pre{white-space:pre-wrap}h2{margin-top:40px}</style><h1>Free Boundary Waters basecamp — BARE source view</h1><p>This source preview draws the six ordinary town residents. Regional checkpoint officers are recorded separately and are not drawn here. The native town is published; connected checkpoint gameplay remains under review. Story access is free; Safari check-in is CLOSED without fees.</p><pre id="state"></pre><h2>Whole town</h2><img id="whole" src="whole-town.png"><h2>160×144 player views</h2><img id="views" src="viewpoints.png"><h2>Whole Survey Shelter</h2><img id="room" src="survey-shelter.png"><script>let old='';async function update(){let r=await(await fetch('revision.json?'+Date.now())).json();document.querySelector('#state').textContent=JSON.stringify(r,null,2);if(old!=r.hash){old=r.hash;for(let [id,path]of [['whole','whole-town.png'],['views','viewpoints.png'],['room','survey-shelter.png']])document.querySelector('#'+id).src=path+'?'+old}}setInterval(update,600);update()</script>'''

def adjacent(p):
    x,y=p
    return [(x+1,y),(x-1,y),(x,y+1),(x,y-1)]

def flood(start, cells, doors=frozenset()):
    seen={start}; todo=deque([start])
    while todo:
        at=todo.popleft()
        for q in adjacent(at):
            if q in cells and q not in seen:
                seen.add(q)
                if q not in doors: todo.append(q)
    return seen

def prepare(pack, layout):
    if layout['compiled_tileset'] in pack['data']['tilesets']: return
    definition=deepcopy(pack['data']['tilesets']['johto'])
    for recipe in layout['derived_blocks']:
        ts=recipe.get('collision_source_tileset',recipe['source_tileset'])
        block=recipe.get('collision_source_block',recipe['source_block'])
        definition['collision'][recipe['id']]=list(pack['data']['tilesets'][ts]['collision'][block])
    pack['data']['tilesets'][layout['compiled_tileset']]=definition

def validate(design, layout, pack, blocks):
    assert layout['schema_version']==2 and layout['reject_cliff_cuts'] is True
    assert layout['dimensions_metatiles']==design['dimensions_metatiles']==[32,24]
    for layer in layout['layers']:
        assert layer['name']
        if layer['kind'] in {'polygon','cliff_polygon','path'}:
            ps=layer['points']; pairs=list(zip(ps,ps[1:]))
            if layer['kind']!='path':pairs.append((ps[-1],ps[0]))
            assert all(a[0]==b[0] or a[1]==b[1] for a,b in pairs),layer['name']
    w,h=layout['dimensions_metatiles']; c=pack['data']['tilesets'][layout['compiled_tileset']]['collision']
    def kind(q):
        x,y=q
        return c[f'{blocks[y//2*w+x//2]:02x}'][y%2*2+x%2]
    cells={(x,y) for y in range(h*2) for x in range(w*2)}
    walk={p for p in cells if kind(p) in {'FLOOR','TALL_GRASS','DOOR'}}
    water={p for p in cells if kind(p).startswith('WATER') and kind(p)!='WATERFALL'}
    actors={tuple(a['tile'])for a in design['residents']}
    doors={tuple(d['tile'])for d in design['doors']}
    seen=flood((6,24),walk-actors,doors)
    for a in design['residents']:
        assert tuple(a['tile']) in walk,a['id']+' not on walking collision'
        assert set(adjacent(tuple(a['tile'])))&seen,a['id']+' has no free approach'
        assert all(0<len(line)<=18 for line in a['lines']),a['id']+' text width'
    for sign in design['signs']:
        at=tuple(sign['tile']);assert kind(at)=='WALL',sign['id']+' sign collision'
        assert set(adjacent(at))&seen,sign['id']+' unreachable'
        assert all(0<len(line)<=18 for line in sign['lines']),sign['id']+' text width'
    ports=[('MN_R12','E',[30,12],[28,12]),('MN_R13','S',[16,22],[16,20]),('MN_R20','W',[1,12],[3,12])]
    for actual,(route,side,gate,landing) in zip(design['ports'],ports):
        assert (actual['route'],actual['side'],actual['gate_metatile'],actual['landing_metatile'])==(route,side,gate,landing)
        for p in [gate,landing]:assert tuple(2*v for v in p) in seen,route+' free port disconnected'
    for d in design['doors']:
        assert kind(tuple(d['tile']))=='DOOR' and tuple(d['tile']) in seen
        assert kind(tuple(d['return_tile']))=='FLOOR' and tuple(d['return_tile']) in seen
        assert (d['warp_index'],d['return_warp_index'],d['target_warp'])==(1,2,3),'door must target room passive arrival3'
    assert not any(kind(p)=='CUT_TREE' for p in cells),'incidental native Cut canopy'
    surf=flood((6,24),(walk|water)-actors,doors)
    outer={p for p in cells if p[0] in {0,w*2-1} or p[1] in {0,h*2-1}}
    assert not (water&outer),'unconnected outer water'
    boundary=seen&outer
    assert boundary=={(0,24),(0,25),(63,24),(63,25),(32,47),(33,47)},('unexpected outer walking shoulders',boundary)
    assert (32,5) not in surf,'closed admission has a walking/Surf bypass'
    assert tuple(design['admission_contract']['early_return_safe_core']) in seen
    guards={tuple(p) for p in design['regional_gate_contract']['guard_tiles']}
    assert guards=={(32,43),(33,43)} and guards<=walk,'reserved League guards need real Floor'
    denied=flood((6,24),walk-actors-guards,doors)
    assert (32,47) not in denied and (33,47) not in denied,'League guards have a walking bypass'
    denied_surf=flood((6,24),(walk|water)-actors-guards,doors)
    assert (32,47) not in denied_surf and (33,47) not in denied_surf,'League guards have a Surf bypass'
    assert all(p in denied for p in [(0,24),(63,24),(32,40),tuple(design['doors'][0]['return_tile'])]),'League guard denies free town destination'
    dock=design['public_dock'];assert tuple(dock['launch_tile']) in seen and tuple(dock['water_target']) in water
    for x,y in dock['north_rail_blocks']:assert [kind((2*x,2*y)),kind((2*x,2*y+1))]==['WALL','FLOOR'],'public dock north rail must block walking into water'
    for x,y in dock['south_rail_blocks']:assert [kind((2*x,2*y)),kind((2*x,2*y+1))]==['FLOOR','WALL'],'public dock south rail must block walking into water'
    room=design['interiors'][0];rw,rh=room['dimensions_metatiles'];rb=[int(b,16)for row in room['rows']for b in row.split()];rc=pack['data']['tilesets']['lab']['collision']
    assert (rw,rh)==(8,6) and len(rb)==48
    assert room['passive_arrival']=={'index':3,'tile':[8,9]},'room arrival must be passive3 on Floor'
    def rk(q):x,y=q;return rc[f'{rb[y//2*rw+x//2]:02x}'][y%2*2+x%2]
    occupied={tuple(a['tile'])for a in room['residents']}
    room_walk={(x,y)for y in range(rh*2)for x in range(rw*2)if rk((x,y))in{'FLOOR','WARP_CARPET_DOWN'}}
    room_seen=flood((8,9),room_walk-occupied)
    for a in room['residents']:
        assert tuple(a['tile']) in room_walk,a['id']+' floor'
        assert set(adjacent(tuple(a['tile'])))&room_seen,a['id']+' approach'
        assert all(0<len(line)<=18 for line in a['lines']),a['id']+' text width'
    for s in room['signs']:
        assert rk(tuple(s['tile'])) in {'WALL','WINDOW'},s['id']+' physical BG'
        assert set(adjacent(tuple(s['tile'])))&room_seen,s['id']+' reading approach'
        assert all(0<len(line)<=18 for line in s['lines']),s['id']+' text width'
    assert all(tuple(e) in room_seen and rk(tuple(e))=='WARP_CARPET_DOWN'for e in room['exits'])
    assert rk((12,3))=='WALL' and rk((12,4))=='FLOOR' and (12,4) in room_seen
    assert all(p in room_seen for p in [(8,5),(10,5),(9,6),(3,1),(11,9)])
    return dict(source_only=True,reachable_walking_tiles=len(seen),reachable_surf_water_tiles=len(surf&water),reachable_outer_water=0,outer_walking_tiles=sorted(boundary),all_three_free_ports=True,story_room_free=True,closed_admission_bypass=False,room_reachable_tiles=len(room_seen),incidental_cut_tiles=0,league_reserved_guard_seals_south=True,league_guard_preserves_free_town=True)

def negative_checks(design, layout, pack):
    cases=[]
    opened=deepcopy(design)
    opened['residents']=[a for a in opened['residents'] if a['id']!='MN_BOUNDARY_PENDING_GATE_EAST']
    cases.append(('remove one closed-admission officer',opened,layout,'closed admission has a walking/Surf bypass'))
    rail=deepcopy(layout)
    rail['layers'].append(dict(kind='point',name='Negative: remove native dock north rail',position=[8,7],block='01'))
    cases.append(('replace north rail with floor',design,rail,'public dock north rail'))
    edge=deepcopy(layout)
    edge['layers'].append(dict(kind='path',name='Negative: expose lake to unconnected map edge',points=[[2,2],[0,2]],width_metatiles=1,material='water'))
    cases.append(('extend lake to outer edge',design,edge,'unconnected outer water'))
    bad_arrival=deepcopy(design);bad_arrival['doors'][0]['target_warp']=1
    cases.append(('send entry to active room carpet',bad_arrival,layout,'room passive arrival3'))
    passed=[]
    for name,d,l,expected in cases:
        try:validate(d,l,pack,layout_blocks(l))
        except AssertionError as error:
            assert expected in str(error),(name,'wrong rejection',str(error))
            passed.append(name)
        else:raise AssertionError(name+' unexpectedly accepted')
    return passed

def checkpoint_actors(design):
    """Read the separate overlay without modifying ordinary residents."""
    actors=[guard for checkpoint in design['regional_checkpoints']['checkpoints'] for guard in checkpoint['guards']]
    assert len(actors)==6 and len({a['id'] for a in actors})==6,'six distinct regional officers required'
    assert not ({a['id'] for a in actors}&{a['id'] for a in design['residents']}),'checkpoint officer duplicated as resident'
    return actors

def checkpoint_preview(design,layout,pack,blocks,state):
    """Check static body geometry. This never executes scripts or an escort."""
    width,height=layout['dimensions_metatiles']
    collision=pack['data']['tilesets'][layout['compiled_tileset']]['collision']
    def kind(p):
        x,y=p
        return collision[f'{blocks[y//2*width+x//2]:02x}'][y%2*2+x%2]
    cells={(x,y) for y in range(height*2) for x in range(width*2)}
    walk={p for p in cells if kind(p) in {'FLOOR','TALL_GRASS','DOOR'}}
    water={p for p in cells if kind(p).startswith('WATER') and kind(p)!='WATERFALL'}
    ordinary={tuple(a['tile']) for a in design['residents']}
    doors={tuple(d['tile']) for d in design['doors']}
    officers=checkpoint_actors(design)
    bodies={tuple(a['tile']) for a in officers}
    assert len(bodies)==6 and not bodies&ordinary,'checkpoint body overlaps an actor'
    for a in officers:
        assert kind(tuple(a['tile']))=='FLOOR',a['id']+' needs native Floor'
        assert a['sprite']=='SPRITE_OFFICER' and a['event_flag']==-1,a['id']+' native body contract'
    seams={'gunflint':{(63,24),(63,25)},'league':{(32,47),(33,47)},'voyageurs':{(0,24),(0,25)}}
    care={(56,24),(32,40),(6,24),tuple(design['public_dock']['launch_tile']),tuple(design['doors'][0]['return_tile'])}
    for checkpoint in design['regional_checkpoints']['checkpoints']:
        assert len(checkpoint['guards'])==len(checkpoint['inner_coord_tiles'])==len(checkpoint['outer_inward_coord_tiles'])==2,checkpoint['id']+' needs two exact lanes'
        for p in checkpoint['inner_coord_tiles']+checkpoint['outer_inward_coord_tiles']:
            assert kind(tuple(p))=='FLOOR',checkpoint['id']+' coordinate plane needs real Floor'
        blocked=ordinary|{tuple(a['tile']) for a in checkpoint['guards']}
        individual=flood((6,24),(walk|water)-blocked,doors)
        assert not individual&seams[checkpoint['id']],checkpoint['id']+' body pair has a walking/Surf bypass'
    closed_walk=flood((6,24),walk-ordinary-bodies,doors)
    closed_surf=flood((6,24),(walk|water)-ordinary-bodies,doors)
    assert care<=closed_walk,'closed bodies deny public landings, dock or shelter'
    assert not set.union(*seams.values())&closed_surf,'closed body composition exposes a departure'
    visible=design['residents']+(officers if state=='closed' else [])
    occupied={tuple(a['tile']) for a in visible}
    seen=flood((6,24),walk-occupied,doors)
    if state!='closed':
        assert set.union(*seams.values())<=seen,'open static body assumption disconnects a departure'
    views=[]
    for checkpoint in design['regional_checkpoints']['checkpoints']:
        tile=tuple(checkpoint['inner_coord_tiles'][0])
        assert tile in seen and tile not in occupied,checkpoint['id']+' camera needs clear walking approach'
        views.append(dict(id='CHECKPOINT_'+checkpoint['id'].upper(),tile=list(tile)))
    labels={
        'bare':('BARE_SOURCE_TOWN','Six ordinary residents; regional checkpoint overlay omitted.'),
        'closed':('STATIC_UNQUALIFIED','Six regional officers drawn at their actual native anchors. This static flood does not execute inward escort or refusal scripts.'),
        'qualified':('STATIC_QUALIFIED_ASSUMPTION','All regional requirements are assumed for framing; six regional bodies are omitted. No badges, signal, restoration or saved visibility were earned or executed.')}
    return visible,views,dict(preview_state=labels[state][0],state_explanation=labels[state][1],visible_town_actors=len(visible),regional_checkpoint_overlay_drawn=state=='closed',qualified_requirements_assumed=state=='qualified',earned_progression_proved=False,native_escort_executed=False,closed_bodies_seal_all_three_departures=True,closed_bodies_preserve_public_care=True,surf_is_conservative_reachability_superset=True,assumed_requirements=sorted({flag for checkpoint in design['regional_checkpoints']['checkpoints'] for flag in checkpoint['requires']})if state=='qualified'else[],state_reachable_walking_tiles=len(seen),state_departures_reachable=[name for name,tiles in seams.items() if tiles<=seen])

def save_render(im, name, alias=None, directory=OUTPUT, preview_state=None, camera=None):
    """Archive exact bytes for the version and optional latest alias."""
    info=PngInfo()
    if preview_state:
        info.add_text('preview_state',preview_state)
        info.add_text('render_kind','Static source preview; not production controller or earned progression proof')
        info.add_text('native_escort_executed','false')
    if camera:
        info.add_text('requested_core_tile',json.dumps(camera['tile']))
        info.add_text('source_crop_box_xyxy',json.dumps(camera['box']))
        info.add_text('camera_framing',camera['framing'])
    archive_render(im, directory/name,pnginfo=info)
    if alias is not None:
        archive_render(im, directory/alias,pnginfo=info)


def render(design, layout, pack, art, state='bare'):
    started=time.perf_counter();prepare(pack,layout);blocks=layout_blocks(layout)
    report=validate(design,layout,pack,blocks)
    report['source_negative_checks']=negative_checks(design,layout,pack)
    visible,checkpoint_views,state_report=checkpoint_preview(design,layout,pack,blocks,state)
    report=dict(source_only=True,underlying_bare_geometry=report)
    report.update(state_report)
    directory=OUTPUT/'states'/state
    directory.mkdir(parents=True,exist_ok=True)
    save_image=partial(save_render,directory=directory,preview_state=state_report['preview_state'])
    w,h=layout['dimensions_metatiles']
    terrain=art.render('johto',w,h,blocks,derived=layout['derived_blocks'])
    scene=terrain.convert('RGBA')
    for a in visible:
        scene.alpha_composite(oriented_sprite(pack,a['sprite'],a['facing']),(a['tile'][0]*16,a['tile'][1]*16-4))
    payload=DESIGN.read_bytes()+LAYOUT.read_bytes()+scene.tobytes()
    if state!='bare':payload+=state.encode()
    digest=hashlib.sha256(payload).hexdigest()[:8]
    save_image(terrain,f'terrain-original-{digest}.png')
    save_image(scene,f'whole-town-original-rgba-{digest}.png')
    save_image(scene.convert('RGB'),f'whole-town-{digest}.png','whole-town.png')
    views=[]
    checkpoint_boxes={}
    for v in design['viewpoints']+checkpoint_views:
        x,y=v['tile'];name=v['id'].lower()
        left,top=x*16-80,y*16-72
        framing='Centered source framing'
        if v['id'].startswith('CHECKPOINT_'):
            left=min(max(left,0),max(scene.width-160,0))
            top=min(max(top,0),max(scene.height-144,0))
            framing='Town extent clamped, matching native-pack comparison; neighboring route terrain is not composited'
            checkpoint_boxes[v['id']]=dict(requested_tile=v['tile'],source_crop_box=[left,top,left+160,top+144])
        camera=dict(tile=v['tile'],box=[left,top,left+160,top+144],framing=framing)
        crop=scene.crop(tuple(camera['box']))
        save_image(crop,f'camera-{name}-original-rgba-{digest}-160x144.png',camera=camera)
        im=crop.convert('RGB')
        save_image(im,f'camera-{name}-{digest}-160x144.png',f'camera-{name}-160x144.png',camera=camera)
        save_image(im.resize((640,576),Image.Resampling.NEAREST),f'viewport-{name}-{digest}-640x576.png',f'viewport-{name}.png',camera=camera)
        views.append((v['id'],im))
    report['checkpoint_camera_boxes']=checkpoint_boxes
    save_image(room_sheet(views,2),f'viewpoints-{digest}.png','viewpoints.png')
    r=design['interiors'][0];template=pack['data']['maps'][r['template']]['attributes']
    rb=[int(b,16)for row in r['rows']for b in row.split()]
    room_terrain=art.render('lab',8,6,rb,group=template['time_of_day']or'day')
    save_image(room_terrain,f'survey-shelter-terrain-original-{digest}.png')
    im=room_terrain.convert('RGBA')
    for a in r['residents']:
        im.alpha_composite(oriented_sprite(pack,a['sprite'],a['facing']),(a['tile'][0]*16,a['tile'][1]*16-4))
    save_image(im,f'survey-shelter-original-rgba-{digest}.png')
    save_image(im.convert('RGB'),f'survey-shelter-{digest}.png','survey-shelter.png')
    for name,(x,y) in [('chief-receiver',(9,5)),('medic-arrival',(6,8))]:
        camera=im.crop((x*16-80,y*16-72,x*16+80,y*16+72))
        save_image(camera,f'camera-room-{name}-original-rgba-{digest}-160x144.png')
        camera=camera.convert('RGB')
        save_image(camera,f'camera-room-{name}-{digest}-160x144.png')
        save_image(camera.resize((640,576),Image.Resampling.NEAREST),f'viewport-room-{name}-{digest}-640x576.png')
    report.update(design_sha256=hashlib.sha256(DESIGN.read_bytes()).hexdigest(),layout_sha256=hashlib.sha256(LAYOUT.read_bytes()).hexdigest(),whole_pixels_sha256=hashlib.sha256(scene.tobytes()).hexdigest(),hash=digest,whole_image=f'whole-town-{digest}.png',render_ms=round((time.perf_counter()-started)*1000,1),render_retention='All whole, original/preconversion, camera, converted viewport, room and sheet images retained by immutable version; prior aliases archived before replacement',output_directory=str(directory.relative_to(ROOT)),native_integration_status=design['status'],status=state_report['state_explanation']+' Safari remains CLOSED in all previews.')
    report_json=json.dumps(report,indent=2)+'\n'
    (directory/f'revision-{digest}.json').write_text(report_json)
    (directory/'revision.json').write_text(report_json);(directory/'index.html').write_text(HTML.replace('BARE source view',state_report['preview_state']).replace('This source preview draws the six ordinary town residents. Regional checkpoint officers are recorded separately and are not drawn here.',state_report['state_explanation']+' Checkpoint cameras clamp to the town extent, matching the native-pack comparison; neighboring route terrain is not composited.'))
    return report

def main():
    parser=argparse.ArgumentParser();parser.add_argument('--pack',type=Path,default=ROOT/'content-packs/text-tui.crystalpack');parser.add_argument('--serve',action='store_true');parser.add_argument('--watch',action='store_true');parser.add_argument('--port',type=int,default=8803);parser.add_argument('--checkpoint-state',choices=['bare','closed','qualified'],default='bare',help='Static source view only; qualified assumes requirements without earning or executing them.');args=parser.parse_args()
    pack=load_pack(args.pack);art=SourceArt(pack)
    directory=OUTPUT/'states'/args.checkpoint_state
    if args.serve:
        directory.mkdir(parents=True,exist_ok=True);server=ThreadingHTTPServer(('127.0.0.1',args.port),partial(SimpleHTTPRequestHandler,directory=str(directory)));threading.Thread(target=server.serve_forever,daemon=True).start();print(f'Basecamp review http://127.0.0.1:{args.port}/',flush=True)
    last=None
    while True:
        current=(DESIGN.stat().st_mtime_ns,LAYOUT.stat().st_mtime_ns)
        if current!=last:
            try: print(json.dumps(render(json.loads(DESIGN.read_text()),json.loads(LAYOUT.read_text()),pack,art,args.checkpoint_state)),flush=True)
            except (AssertionError,KeyError,ValueError,TypeError) as error:
                if not(args.serve or args.watch):raise
                (directory/'revision.json').write_text(json.dumps(dict(hash=str(time.time_ns()),status='INVALID SOURCE',error=str(error)))+'\n');print(str(error),flush=True)
            last=current
        if not(args.watch or args.serve):break
        time.sleep(.2)
if __name__=='__main__':main()
