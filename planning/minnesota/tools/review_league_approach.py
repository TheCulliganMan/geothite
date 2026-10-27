#!/usr/bin/env python3
"""Review explicit League Approach strokes against the supplied source pixels.

This tool never chooses geography. Static checks do not prove native gameplay.
"""
from render_archive import retain_render_file, save_render
import argparse
from collections import deque
from functools import partial
import hashlib
from http.server import SimpleHTTPRequestHandler, ThreadingHTTPServer
import json
from pathlib import Path
import threading
import time

from PIL import Image, ImageDraw

from render_reference import SourceArt, layout_blocks, load_pack, room_sheet
from review_grand_marais import sprite
from review_north_shore import oriented_sprite
from review_gunflint import check_rooms, flood, neighbors, VECTORS
from ledge_paths import inspect_ledges, draw_ledge_paths

ROOT = Path(__file__).resolve().parents[3]
LAYOUT = ROOT / 'planning/minnesota/routes/MN_R13.layout.json'
DOSSIER = ROOT / 'planning/minnesota/routes/MN_R13.json'
OUTPUT = ROOT / 'planning/minnesota/renders/league-approach'
CLEAR = {'FLOOR', 'LADDER'}


def check(design, layout, pack, blocks):
    width,height = layout['dimensions_metatiles']
    collision = pack['data']['tilesets'][layout['compiled_tileset']]['collision']
    def kind(x,y): return collision[f'{blocks[y//2*width+x//2]:02x}'][y%2*2+x%2]
    actors = design['trainers']+design['residents']+design['items']
    occupied = {tuple(a['tile']) for a in actors}
    assert len(occupied) == len(actors), 'overlapping actors'
    floor = {(x,y) for y in range(height*2) for x in range(width*2) if kind(x,y) in CLEAR and (x,y) not in occupied}
    grass = {(x,y) for y in range(height*2) for x in range(width*2) if kind(x,y) == 'TALL_GRASS'}
    start,end = [tuple(v*2 for v in e['gate_metatile']) for e in design['endpoints']]
    crossings = design.get('required_grass_crossings',[])
    traversable = floor | (grass-occupied) if crossings or design.get('battle_or_grass_choices') else floor
    seen = flood(start,traversable)
    errors = []
    for crossing in crossings:
        x0,y0,x1,y1 = crossing['bounds_core']
        band = {(x,y) for y in range(y0,y1) for x in range(x0,x1)}
        if band-grass: errors.append(crossing['id']+': declared grass contains another terrain')
        if end in flood(start,traversable-band): errors.append(crossing['id']+': grass can be avoided')
        # 0/1 shortest path counts actual grass steps, independent of road length.
        from collections import deque
        costs={start:0}; queue=deque([start])
        while queue:
            at=queue.popleft()
            for q in neighbors(at):
                if q not in traversable: continue
                cost=costs[at]+int(q in band)
                if cost < costs.get(q,10**9):
                    costs[q]=cost
                    (queue.append if q in band else queue.appendleft)(q)
        if costs.get(end,0) < crossing['minimum_grass_steps']: errors.append(crossing['id']+': grass crossing is shorter than declared')
    if end not in seen: errors.append('Route endpoints disconnected')
    if grass-occupied-flood(start,floor | (grass-occupied)):
        missing = grass-occupied-flood(start,floor | (grass-occupied))
        errors.append(f'Stranded grass: {len(missing)} core tiles, {sorted(missing)[:12]}')
    for key in ['trainers','residents','items','signs','doors']:
        for a in design[key]:
            x,y = a['tile']
            expected = 'WALL' if key == 'signs' else a.get('collision','CAVE') if key == 'doors' else 'FLOOR'
            if kind(x,y) != expected: errors.append(f'{a["id"]}: {kind(x,y)} at {x,y}, expected {expected}')
            if not any(q in seen and q[0]//64 == x//64 and q[1]//48 == y//48 for q in neighbors((x,y))):
                errors.append(f'{a["id"]}: no clear same-chunk approach at {x,y}')
            if key != 'doors' and (x%64 in {0,63} or y%48 in {0,47}): errors.append(a['id']+': actor on seam')
            for field in ['lines','before_lines','after_lines','loss_lines']:
                if not all(0 < len(line) <= 18 for line in a.get(field,[])): errors.append(a['id']+': text width')
    for a in design['trainers']:
        if a['trainer_class'] not in {t['trainer_class'] for t in pack['data']['trainers']['trainers'].values()}: errors.append(a['id']+': unknown trainer class')
        for p in a['party']:
            if p['species'] not in pack['data']['pokemon']: errors.append(a['id']+': unknown species')
    for a in design['items']:
        if a['item'] not in pack['data']['items']: errors.append(a['id']+': unknown item')
    for door in design['doors']:
        if (door['tile'] != door['arrival_tile'] and not door.get('return_warp_index')) or tuple(door['return_tile']) not in seen: errors.append(door['id']+': unsafe native return')
    for view in design['viewpoints']:
        if tuple(view['tile']) not in seen: errors.append(view['id']+': blocked viewpoint')
    paths=design['loops']+[{'id':'MAIN_ROAD','path_metatiles':design['main_path_metatiles']}]
    for loop in paths:
        points = loop['path_metatiles']
        if loop['id']!='MAIN_ROAD' and points[0] != points[-1]: errors.append(loop['id']+': open loop')
        for a,b in zip(points,points[1:]):
            assert a[0] == b[0] or a[1] == b[1],loop['id']+': diagonal'
            dx,dy = (b[0]>a[0])-(b[0]<a[0]),(b[1]>a[1])-(b[1]<a[1])
            for step in range(max(abs(b[0]-a[0]),abs(b[1]-a[1]))+1):
                mx,my = a[0]+step*dx,a[1]+step*dy
                if not any((mx*2+qx,my*2+qy) in seen for qx in [0,1] for qy in [0,1]):
                    # Declared hops are directed transitions, never ordinary floor.
                    hop_kinds={'south':((0,1),'HOP_DOWN'),'west':((-1,0),'HOP_LEFT'),'east':((1,0),'HOP_RIGHT')}
                    hop=any((dx,dy)==hop_kinds.get(ledge['direction'],(None,None))[0]
                            and ledge['bounds_core'][0]<=mx*2<ledge['bounds_core'][2]
                            and ledge['bounds_core'][1]<=my*2<ledge['bounds_core'][3]
                            and any(kind(mx*2+qx,my*2+qy)==hop_kinds[ledge['direction']][1] for qx in [0,1] for qy in [0,1])
                            and tuple(ledge['landing']) in seen
                            for ledge in design.get('one_way_ledges',[]))
                    if not hop: errors.append(f'{loop["id"]}: blocked join {mx,my}'); break
    cut = design.get('shortcut')
    if cut:
        x,y = cut['tile']
        if kind(x,y) != 'CUT_TREE' or tuple(cut['approach']) not in seen: errors.append('Cut: source quadrant or approach')
        if any(kind(x+dx,y) in CLEAR for dx in [-1,1]): errors.append('Cut can be sidestepped')
    hazards = set()
    for a in design['trainers']:
        x,y = a['tile']; dx,dy = VECTORS[a['facing']]
        for step in range(1,a['sight_range']+1):
            at=x+dx*step,y+dy*step
            if not(0<=at[0]<width*2 and 0<=at[1]<height*2) or kind(*at) not in CLEAR | {'TALL_GRASS'}: break
            hazards.add(at)
    if not design.get('mandatory_trainer_lanes') and end not in flood(start,traversable-hazards): errors.append('Initial trainer sight blocks the intended main route')
    chunks = {(x//64,y//48) for x,y in seen}
    expected_chunks=design['chunk_columns']*design['chunk_rows']
    if len(chunks) != expected_chunks: errors.append(f'Unused chunks: {expected_chunks-len(chunks)}')
    try: check_rooms(design,pack)
    except AssertionError as error: errors.append('Room: '+str(error))
    ledge_returns=[]
    try: ledge_returns=inspect_ledges(design,kind,seen)
    except AssertionError as error: errors.append('Ledge: '+str(error))
    return dict(clear_reachable_tiles=len(seen & floor),walkable_reachable_tiles=len(seen),required_grass_crossings=len(crossings),accessible_encounter_tiles=len(grass),map_chunks_retained=len(chunks),
                trainers=len(design['trainers']),residents=len(design['residents']),pickups=len(design['items']),
                signs=len(design['signs']),interiors=len(design['interiors']),ledge_returns=ledge_returns,errors=errors),kind,seen


def verify_published(design,layout,pack):
    """Compare every native chunk and physical event with the current authoring."""
    maps=pack['data']['maps']; expected=layout_blocks(layout); actual=[]
    width,height=layout['dimensions_metatiles']; prefix=design['map_prefix']
    for y in range(height):
        for x in range(width):
            module=maps[f'{prefix}C{x//32:02}R{y//24:02}']
            actual.append(module['blocks'][y%24*module['attributes']['width']+x%32])
    differences=[(i%width,i//width,a,b) for i,(a,b) in enumerate(zip(expected,actual)) if a!=b]
    assert not differences,f'Native/editor metatiles differ: {differences[:12]}'
    objects={}; signs={}; warps={}
    for row in range(design['chunk_rows']):
        for col in range(design['chunk_columns']):
            module=maps[f'{prefix}C{col:02}R{row:02}']; a=module['attributes']
            assert (a['width'],a['height'],a['tileset_name'])==(min(32,width-col*32),min(24,height-row*24),layout['compiled_tileset'])
            for actor in module['objects']: objects[actor['script']]=(actor,[actor['x']+col*64,actor['y']+row*48],module)
            for sign in module['events']['bg_events']: signs[sign['script']]=[sign['x']+col*64,sign['y']+row*48]
            for warp in module['events']['warps']: warps[(warp['x']+col*64,warp['y']+row*48)]=warp
    assert len(objects)==sum(len(design[k]) for k in ['trainers','residents','items']) and len(signs)==len(design['signs']) and len(warps)==sum(1+bool(d.get('return_warp_index')) for d in design['doors']),'Native event inventory'
    facing={'north':'UP','south':'DOWN','west':'LEFT','east':'RIGHT'}
    def text(module,label): return [c['args'][0].strip('"') for c in module['scripts'][label] if c['command'] in {'text','line','para'}]
    for actor in design['trainers']+design['residents']+design['items']:
        native,tile,module=objects[actor['id']]
        assert tile==actor['tile'] and native['sprite']==actor.get('sprite','SPRITE_POKE_BALL'),actor['id']+': placement'
        assert native['spritemovedata']=='SPRITEMOVEDATA_STANDING_'+facing[actor.get('facing','south')],actor['id']+': facing'
        for key,suffix in [('lines','Text'),('before_lines','Seen'),('after_lines','Won'),('loss_lines','Lost')]:
            if key in actor: assert text(module,actor['id']+suffix)==actor[key],actor['id']+': dialogue'
        if actor in design['items']: assert native['event_flag']==actor['flag']
        if actor in design['trainers']:
            trainer=pack['data']['trainers']['trainers'][actor['id']]
            assert trainer['name']==actor['display_name'].upper()
            assert [(p['species'],p['level']) for p in trainer['party']]==[(p['species'],p['level']) for p in actor['party']]
            assert module['trainer_scripts'][actor['id']]['event_flag']==actor['defeated_flag']
    for sign in design['signs']:
        assert signs[sign['id']]==sign['tile']
        x,y=sign['tile']; assert text(maps[f'{prefix}C{x//64:02}R{y//48:02}'],sign['id']+'Text')==sign['lines']
    for room in design['interiors']:
        module=maps[room['id']]; assert module['blocks']==[int(b,16) for row in room['rows'] for b in row.split()]
        assert module['attributes']['border_block']==room['border_block']
        assert [[w['x'],w['y']] for w in module['events']['warps']]==room['exits']
        assert [(a['script'],[a['x'],a['y']]) for a in module['objects']]==[(a['id'],a['tile']) for a in room['residents']+room['items']]
        for actor in room['residents']:
            assert text(module,actor['id']+'Text')==actor['lines']
            if actor.get('service')=='heal_party':
                assert any(c['source_script']==actor['id'] and c['command']=='special' and c['args']==['HealParty'] for c in module['script_runtime_commands']),actor['id']+': healing routine not compiled'
        for index,warp in enumerate(module['events']['warps']):
            door=next(d for d in design['doors'] if d['target']==room['id'] and (not room.get('exit_doors') or d['tile']==room['exit_doors'][index]))
            x,y=door['tile'];exterior=maps[f'{prefix}C{x//64:02}R{y//48:02}']['attributes']['map_constant']
            assert warp['target_map_constant']==exterior and warp['target_warp_id']==door.get('return_warp_index',door['warp_index']),room['id']+': wrong mouth return'
        for door in [d for d in design['doors'] if d['target']==room['id']]:
            warp=warps[tuple(door['tile'])]
            assert warp['target_map_constant']==module['attributes']['map_constant'] and warp['target_warp_id']==door['target_warp'],room['id']+': wrong mouth entry'
    return {'published_maps_checked':design['chunk_columns']*design['chunk_rows']+len(design['interiors']),'published_metatiles_checked':len(actual),'published_outdoor_objects_checked':len(objects)}


def inspect_league(design, layout, pack, blocks):
    """Check the authored gate against walking and Surf before any publication."""
    width,height = layout['dimensions_metatiles']
    collisions = pack['data']['tilesets'][layout['compiled_tileset']]['collision']
    after = list(blocks)
    for gate in design.get('cut_gates', []):
        x,y = gate['tile']; after[y//2*width+x//2] = int(gate['replacement_block'],16)
    report,after_kind,seen = check(design,layout,pack,after)
    def kind(x,y): return collisions[f'{blocks[y//2*width+x//2]:02x}'][y%2*2+x%2]
    occupied = {tuple(a['tile']) for key in ['trainers','residents','items'] for a in design[key]}
    start,end = [tuple(v*2 for v in e['gate_metatile']) for e in design['endpoints']]
    def allowed(surface):
        return {(x,y) for y in range(height*2) for x in range(width*2)
                if kind(x,y) in surface and (x,y) not in occupied}
    land = allowed(CLEAR | {'TALL_GRASS'})
    walking = flood(start,land)
    surfing = flood(start,allowed(CLEAR | {'TALL_GRASS','WATER'}))
    assert end in walking, 'Required route must remain open without Cut or Surf'
    choices=[]
    for choice in design.get('battle_or_grass_choices',[]):
        trainer=next(a for a in design['trainers'] if a['id']==choice['trainer_id'])
        x,y=trainer['tile']; dx,dy=VECTORS[trainer['facing']]
        sight=set()
        for step in range(1,trainer['sight_range']+1):
            at=x+dx*step,y+dy*step
            if at not in land: break
            sight.add(at)
        x0,y0,x1,y1=choice['bounds_core']
        band={(x,y) for y in range(y0,y1) for x in range(x0,x1)}
        assert all(kind(*at)=='TALL_GRASS' for at in band), choice['id']+': false grass band'
        available=land-sight
        assert end not in flood(start,available-band), choice['id']+': free dry bypass'
        costs={start:0}; todo=deque([start])
        while todo:
            at=todo.popleft()
            for q in neighbors(at):
                if q not in available: continue
                cost=costs[at]+int(q in band)
                if cost<costs.get(q,10**9):
                    costs[q]=cost
                    (todo.append if q in band else todo.appendleft)(q)
        assert end in costs and costs[end]>=choice['minimum_grass_steps'], choice['id']+': unpaid or disconnected battle alternative'
        choices.append(dict(id=choice['id'],minimum_grass_steps=costs[end]))
    for ledge in design.get('one_way_ledges',[]):
        if 'hop_from' in ledge:
            expected={'south':'HOP_DOWN','east':'HOP_RIGHT','west':'HOP_LEFT'}[ledge['direction']]
            assert kind(*ledge['hop_from'])==expected, ledge['id']+': hop_from is not the native ledge face'
    cut_tiles = {(x,y) for y in range(height*2) for x in range(width*2) if kind(x,y)=='CUT_TREE'}
    assert cut_tiles == {tuple(g['tile']) for g in design.get('cut_gates',[])}, 'Undeclared native Cut canopy'
    for gate in design.get('cut_gates',[]):
        x,y = gate['tile']; block = blocks[y//2*width+x//2]
        native = pack['data']['field_moves']['cut']['replacements'][layout['compiled_tileset']][block]
        assert native['replacement_block_id'] == int(gate['replacement_block'],16), 'Fake Cut replacement'
        assert tuple(gate['approach']) in walking, 'Unreachable Cut approach'
        assert all(tuple(tile) not in surfing and tuple(tile) in seen for tile in gate['targets']), 'Cut target has a walking/Surf bypass'
        for actor_id in gate['gated_actor_ids']:
            actor = next(a for key in ['trainers','residents','items'] for a in design[key] if a['id']==actor_id)
            approaches = set(neighbors(tuple(actor['tile'])))
            assert not approaches & surfing and approaches & seen, actor_id+': gate does not seal interaction'
    water = {(x,y) for y in range(height*2) for x in range(width*2) if kind(x,y)=='WATER'}
    assert not any(x in {0,width*2-1} or y in {0,height*2-1} for x,y in water), 'Unconnected outer Surf edge'
    report.update(walk_before_cut=len(walking),walk_surf_before_cut=len(surfing),cut_opened_walk_tiles=len(seen-walking),outer_water_tiles=0,battle_or_grass_proofs=choices)
    return report,kind,seen


def render(design,layout,pack,art,output=OUTPUT):
    started=time.perf_counter(); blocks=layout_blocks(layout); width,height=layout['dimensions_metatiles']
    report,kind,seen=inspect_league(design,layout,pack,blocks)
    terrain=(art.render(layout['tileset'],width,height,blocks,derived=layout['derived_blocks'])
             if layout.get('derived_blocks') else art.render(layout['compiled_tileset'],width,height,blocks))
    scene=terrain.convert('RGBA')
    for actor in design['trainers']+design['residents']:
        scene.alpha_composite(oriented_sprite(pack,actor['sprite'],actor['facing']),(actor['tile'][0]*16,actor['tile'][1]*16-4))
    for actor in design['items']: scene.alpha_composite(sprite(pack,'SPRITE_POKE_BALL'),(actor['tile'][0]*16,actor['tile'][1]*16-4))
    output.mkdir(parents=True,exist_ok=True)
    save_render(terrain, output/'terrain.png')
    save_render(scene, output/'whole-route-original-rgba.png')
    save_render(scene.convert('RGB'), output/'whole-route.png')
    save_render(draw_ledge_paths(scene,design,report['ledge_returns']), output/'ledges.png')
    digest=hashlib.sha256(scene.tobytes()).hexdigest()[:8]; save_render(scene.convert('RGB'), output/f'whole-route-{digest}.png')
    occupied={tuple(v//2 for v in actor['tile']) for key in ['trainers','residents','items'] for actor in design[key]}
    sizes={}; largest=(0,0,0)
    for y in range(height):
        for x in range(width):
            size=(1+min(sizes.get((x-1,y),0),sizes.get((x,y-1),0),sizes.get((x-1,y-1),0))
                  if blocks[y*width+x] in {1,2,0x71} and (x,y) not in occupied else 0)
            sizes[x,y]=size
            if size>largest[0]:largest=(size,x-size+1,y-size+1)
    size,x,y=largest; cx,cy=x*2+size,y*2+size
    empty=scene.crop((cx*16-80,cy*16-72,cx*16+80,cy*16+72))
    save_render(empty, output/'viewport-emptiest_ground-native-rgba.png')
    save_render(empty.convert('RGB'), output/'viewport-emptiest_ground-native-rgb.png')
    save_render(empty.convert('RGB').resize((640,576),Image.Resampling.NEAREST), output/'viewport-emptiest_ground.png')
    report.update(largest_blank_ground_square_metatiles=size,emptiest_ground_camera_core=[cx,cy])
    overview = scene.resize((width*16,height*16),Image.Resampling.NEAREST)
    save_render(overview, output/'overview-original-rgba.png')
    save_render(overview.convert('RGB'), output/'overview.png')
    overlay=scene.copy(); draw=ImageDraw.Draw(overlay,'RGBA')
    for row in range(design['chunk_rows']):
        for col in range(design['chunk_columns']):
            x,y=col*1024,row*768; draw.rectangle((x,y,x+1023,y+767),outline=(255,255,255,160),width=1)
            draw.text((x+8,y+8),f'{design["map_prefix"]}C{col:02}R{row:02}',fill='white',stroke_width=1,stroke_fill='black')
    for actor in design['trainers']+design['residents']+design['items']+design['signs']+design['doors']+design['viewpoints']:
        x,y=actor['tile']; draw.rectangle((x*16,y*16,x*16+15,y*16+15),outline=(255,210,80,255),width=2)
        draw.text((x*16,y*16-12),actor['id'].removeprefix(design['id']+'_'),fill='white',stroke_width=1,stroke_fill='black')
    save_render(overlay, output/'anchors-original-rgba.png')
    save_render(overlay.convert('RGB'), output/'anchors.png')
    views=[]
    for view in design['viewpoints']:
        x,y=view['tile']; raw=scene.crop((x*16-80,y*16-72,x*16+80,y*16+72))
        save_render(raw, output/('viewport-'+view['id'].lower()+'-native-rgba.png'))
        im=raw.convert('RGB'); views.append((view['id'],im))
        save_render(im, output/('viewport-'+view['id'].lower()+'-native-rgb.png'))
        save_render(im.resize((640,576),Image.Resampling.NEAREST), output/('viewport-'+view['id'].lower()+'.png'))
    save_render(room_sheet(views,columns=2), output/'viewpoints.png')
    rooms=[]
    for room in design['interiors']:
        template=pack['data']['maps'][room['template']]['attributes']; tiles=[int(b,16) for row in room['rows'] for b in row.split()]
        terrain_room=art.render(template['tileset_name'],*room['dimensions_metatiles'],tiles,group=template['time_of_day'] or 'day')
        save_render(terrain_room, output/('room-'+room['id']+'-terrain-original.png'))
        im=terrain_room.convert('RGBA')
        for actor in room.get('residents',[]): im.alpha_composite(oriented_sprite(pack,actor['sprite'],actor['facing']),(actor['tile'][0]*16,actor['tile'][1]*16-4))
        for actor in room.get('items',[]): im.alpha_composite(sprite(pack,'SPRITE_POKE_BALL'),(actor['tile'][0]*16,actor['tile'][1]*16-4))
        save_render(im, output/('room-'+room['id']+'-original-rgba.png'))
        rooms.append((room['id'],im))
    if rooms:save_render(room_sheet(rooms,columns=1), output/'interiors.png')
    else:
        retain_render_file(output/'interiors.png')
        (output/'interiors.png').unlink(missing_ok=True)
    report.update(status='INVALID — see errors' if report['errors'] else 'Source pixels and static collision checked; native gameplay is separate',
                  render_ms=round((time.perf_counter()-started)*1000,1),image=f'whole-route-{digest}.png')
    (output/'revision.json').write_text(json.dumps(report,indent=2)+'\n'); return report


HTML='''<!doctype html><meta charset="utf-8"><title>League Approach review</title><style>body{background:#14212b;color:#e7eff2;font:16px system-ui;margin:20px}img{image-rendering:pixelated}header{position:sticky;top:0;background:#14212bf5;padding:12px}button{margin:4px;padding:10px}.scroll{overflow:auto;max-height:78vh}#map{width:512px}p{max-width:1000px}</style><header><h1>League Approach — three clipped maps</h1><button data-layer="whole-route">Residents</button><button data-layer="terrain">Terrain</button><button data-layer="anchors">Anchors and chunks</button><button id="ledges" data-layer="ledges" hidden>One-way paths</button><button id="zoom">Half / native pixels</button><p id="status">Rendering…</p></header><p>Border Lake · Wind Tor ramp and drop · Grass Saddle · Cedar Shelter · Long Lake high bank · Dry Pass · Cut refuge · Ely arrival. Explicit hand-selected strokes, unchanged packed source art. Static review; production controller evidence is separate.</p><div class="scroll"><img id="map"></div><h2>Native 160 × 144 framing, enlarged 2×</h2><img id="views" src="viewpoints.png"><h2>Cedar Shelter</h2><img id="rooms" src="interiors.png"><script>let layer='whole-route',rev='',native=false;const map=document.querySelector('#map');function refresh(){map.src=layer+'.png?r='+rev;document.querySelector('#views').src='viewpoints.png?r='+rev;const room=document.querySelector('#rooms');if(room)room.src='interiors.png?r='+rev}document.querySelectorAll('[data-layer]').forEach(b=>b.onclick=()=>{layer=b.dataset.layer;refresh()});document.querySelector('#zoom').onclick=()=>{native=!native;map.style.width=(native?1024:512)+'px'};setInterval(async()=>{const s=await(await fetch('/state',{cache:'no-store'})).json();document.querySelector('#ledges').hidden=!(s.ledge_returns||[]).length;document.querySelector('#status').textContent=s.ok?s.render_ms+' ms · '+s.clear_reachable_tiles+' clear tiles · '+s.status:'INVALID: '+s.error;if(s.revision!==rev){rev=s.revision;refresh()}},400)</script>'''


def main():
    parser=argparse.ArgumentParser(); parser.add_argument('--pack',type=Path,default=ROOT/'content-packs/text-tui.crystalpack')
    parser.add_argument('--serve',action='store_true'); parser.add_argument('--watch',action='store_true'); parser.add_argument('--port',type=int,default=8768)
    parser.add_argument('--verify-published',action='store_true')
    args=parser.parse_args()
    if args.verify_published and (args.watch or args.serve): parser.error('--verify-published requires a single run')
    pack=load_pack(args.pack); art=SourceArt(pack); state={'value':{'ok':False,'error':'Initial render pending'}}
    if args.serve:
        OUTPUT.mkdir(parents=True,exist_ok=True); (OUTPUT/'index.html').write_text(HTML)
        class Handler(SimpleHTTPRequestHandler):
            def log_message(self,*args): pass
            def do_GET(self):
                if self.path.split('?')[0]=='/state':
                    body=json.dumps(state['value']).encode(); self.send_response(200); self.send_header('Content-Type','application/json'); self.send_header('Cache-Control','no-store'); self.send_header('Content-Length',str(len(body))); self.end_headers(); self.wfile.write(body)
                else: super().do_GET()
        server=ThreadingHTTPServer(('127.0.0.1',args.port),partial(Handler,directory=str(OUTPUT))); threading.Thread(target=server.serve_forever,daemon=True).start()
        print(f'League Approach review: http://127.0.0.1:{args.port}/',flush=True)
    last=None
    while True:
        revision=(LAYOUT.stat().st_mtime_ns,DOSSIER.stat().st_mtime_ns)
        if revision!=last:
            try:
                design,layout=json.loads(DOSSIER.read_text()),json.loads(LAYOUT.read_text())
                published=verify_published(design,layout,pack) if args.verify_published else {}
                report={**render(design,layout,pack,art),**published}
                state['value']={'ok':not report['errors'],'revision':str(time.time_ns()),'error':'; '.join(report['errors']),**report}; print(json.dumps(report),flush=True)
                if report['errors'] and not(args.serve or args.watch): raise SystemExit(1)
            except (AssertionError,KeyError,ValueError,IndexError,TypeError) as error:
                state['value']={'ok':False,'error':str(error)}; print('DESIGN ERROR: '+str(error),flush=True)
                if not(args.serve or args.watch): raise SystemExit(1)
            last=revision
        if not(args.serve or args.watch): break
        time.sleep(.2)


if __name__=='__main__': main()
