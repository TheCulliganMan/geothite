#!/usr/bin/env python3
"""Exact source-pixel review for the individually authored Gunflint Trail.

This checks physical plans, not runtime gameplay. The prepared pack adds art
only. Route compilation and production-controller evidence remain separate.
"""
from render_archive import save_render
from render_stages import render_stage
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

ROOT = Path(__file__).resolve().parents[3]
LAYOUT = ROOT / 'planning/minnesota/routes/MN_R12.layout.json'
DOSSIER = ROOT / 'planning/minnesota/routes/MN_R12.json'
OUTPUT = ROOT / 'planning/minnesota/renders/gunflint'
VECTORS = {'north': (0,-1), 'south': (0,1), 'west': (-1,0), 'east': (1,0)}
CLEAR = {'FLOOR', 'LADDER'}


def neighbors(tile):
    x,y = tile
    return [(x-1,y),(x+1,y),(x,y-1),(x,y+1)]


def flood(start, allowed):
    assert start in allowed, f'non-walkable starting tile {start}'
    seen = {start}; queue = deque([start])
    while queue:
        for tile in neighbors(queue.popleft()):
            if tile in allowed and tile not in seen:
                seen.add(tile); queue.append(tile)
    return seen


def check_rooms(design, pack):
    for room in design['interiors']:
        width,height = room['dimensions_metatiles']
        assert len(room['rows']) == height and all(len(row.split()) == width for row in room['rows']), room['id']+': rows'
        blocks = [int(b,16) for row in room['rows'] for b in row.split()]
        tileset = pack['data']['maps'][room['template']]['attributes']['tileset_name']
        collision = pack['data']['tilesets'][tileset]['collision']
        def kind(x,y): return collision[f'{blocks[y//2*width+x//2]:02x}'][y%2*2+x%2]
        actors = room.get('residents',[]) + room.get('items',[])
        occupied = {tuple(a['tile']) for a in actors}
        allowed = {(x,y) for y in range(height*2) for x in range(width*2) if kind(x,y) in CLEAR and (x,y) not in occupied}
        seen = flood(tuple(room['clear_arrival']), allowed)
        for tile in room['exits']:
            assert kind(*tile) == room['exit_collision'], room['id']+': exit collision'
            assert any(q in seen for q in neighbors(tile)), room['id']+': exit approach'
        for sign in room.get('signs',[]):
            assert kind(*sign['tile']) == 'WALL', sign['id']+': sign footing'
            assert any(q in seen for q in neighbors(sign['tile'])), sign['id']+': sign approach'
            assert all(0 < len(line) <= 18 for line in sign['lines']), sign['id']+': sign text'
        for a in actors:
            assert kind(*a['tile']) == 'FLOOR', a['id']+': room footing'
            assert any(q in seen for q in neighbors(a['tile'])), a['id']+': room approach'
            assert all(0 < len(line) <= 18 for line in a.get('lines',[])), a['id']+': room dialogue width'


def prepare(pack,layout):
    """Construct only an in-memory collision view; native publication is separate."""
    from copy import deepcopy
    definition=deepcopy(pack['data']['tilesets'].get(layout['compiled_tileset'],pack['data']['tilesets']['forest']))
    definition['collision']=deepcopy(pack['data']['tilesets']['forest']['collision'])
    for entry in layout['derived_blocks']:
        definition['collision'][entry['id']]=list(pack['data']['tilesets'][entry['source_tileset']]['collision'][entry['source_block']])
    pack['data']['tilesets'][layout['compiled_tileset']]=definition


def check(design,layout,pack,blocks):
    """Whole-map native-collision source proof before and after the sealed cache."""
    from copy import deepcopy
    from review_league_approach import check as physical_check
    from ledge_paths import shortest_walk
    prepare(pack,layout);w,h=layout['dimensions_metatiles']
    assert layout['reject_cliff_cuts'] is True and layout['schema_version']==2
    for layer in layout['layers']:
        if layer['kind'] in {'polygon','cliff_polygon','path'}:
            pts=layer['points'];ends=pts[1:] if layer['kind']=='path' else pts[1:]+pts[:1]
            assert all(a[0]==b[0] or a[1]==b[1] for a,b in zip(pts,ends)),layer['name']+': diagonal'
    after=list(blocks)
    for gate in design['cut_gates']:
        x,y=gate['tile'];after[y//2*w+x//2]=int(gate['replacement_block'],16)
    checked=deepcopy(design);checked['shortcut']=None
    report,post_kind,post_seen=physical_check(checked,layout,pack,after)
    collision=pack['data']['tilesets'][layout['compiled_tileset']]['collision']
    def kind(x,y):return collision[f'{blocks[y//2*w+x//2]:02x}'][y%2*2+x%2]
    occupied={tuple(a['tile']) for key in ['trainers','residents','items'] for a in design[key]}
    def allowed(opened=False,surf=False):
        k=post_kind if opened else kind
        return {(x,y) for y in range(h*2) for x in range(w*2) if k(x,y) in ({'FLOOR','TALL_GRASS','LADDER'}|({'WATER'} if surf else set())) and (x,y) not in occupied}
    start,end=[tuple(v*2 for v in e['gate_metatile']) for e in design['endpoints']]
    pre=flood(start,allowed());pre_surf=flood(start,allowed(surf=True));post_surf=flood(start,allowed(True,True))
    assert end in pre,'Main passage requires Cut or Surf'
    inventory={(x,y) for y in range(h*2) for x in range(w*2) if kind(x,y)=='CUT_TREE'}
    assert inventory=={tuple(g['tile']) for g in design['cut_gates']},'Unexpected Cut canopy'
    gates=[]
    for gate in design['cut_gates']:
        assert tuple(gate['approach']) in pre,'Unreachable Cut approach'
        x,y=gate['tile'];source=blocks[y//2*w+x//2]
        native=pack['data']['field_moves']['cut']['replacements']['forest'][source]
        assert native['replacement_block_id']==int(gate['replacement_block'],16),'Wrong native Cut replacement'
        for target in gate['targets']:
            assert tuple(target) not in pre_surf and tuple(target) in post_seen,f'Cut target {target}: preSurf={tuple(target) in pre_surf}, postWalk={tuple(target) in post_seen}; errors={report["errors"]}'
        gated=set(gate['gated_actor_ids'])
        for a in design['trainers']+design['residents']+design['items']:
            adjacent=any(q in pre for q in neighbors(tuple(a['tile'])))
            assert adjacent != (a['id'] in gated),a['id']+': wrong pre-Cut approach contract'
        gates.append(dict(id=gate['id'],source_override='forest0f→17',pre_cut_targets_reachable=False,post_cut_targets_reachable=True,new_walking_tiles=len(post_seen-pre)))
    water={(x,y) for y in range(h*2) for x in range(w*2) if kind(x,y)=='WATER'}
    assert not any(x in {0,w*2-1} or y in {0,h*2-1} for x,y in water),'Unconnected outer Surf water'
    outer={q for q in allowed() if q[0] in {0,w*2-1} or q[1] in {0,h*2-1}}
    assert outer=={(0,56),(0,57),(127,56),(127,57)},'Unplanned outer walking gap'
    for crossing in layout['crossings']:
        for x,y in crossing['blocks']:
            assert blocks[y*w+x]==0x37,'Bog portage must use native grass'
        assert tuple(crossing['safe_north']) in pre and tuple(crossing['safe_south']) in pre,'Bog landings disconnected'
    portages=[]
    from ledge_paths import shortest_walk
    for portage in design['portages']:
        a,b=tuple(portage['north_or_west']),tuple(portage['south_or_east']);x0,y0,x1,y1=portage['grass_bounds_core']
        band={(x,y) for y in range(y0,y1) for x in range(x0,x1)}
        assert all(kind(*q)=='TALL_GRASS' for q in band),portage['id']+': dry sedge bypass'
        direct=shortest_walk(a,b,allowed());alternative=shortest_walk(a,b,allowed()-band)
        assert len(direct)-1==portage['direct_walk_steps'] and len(alternative)-1==portage['dry_bank_alternative_steps'],portage['id']+': traversal contract changed'
        assert len(set(direct)&band)>=portage['minimum_grass_steps'],portage['id']+': portage misses grass'
        portages.append(dict(id=portage['id'],direct_steps=len(direct)-1,grass_steps=len(set(direct)&band),bank_alternative_steps=len(alternative)-1))
    for collision_name in ['CAVE','DOOR']:
        actual={(x,y) for y in range(h*2) for x in range(w*2) if kind(x,y)==collision_name}
        expected={tuple(door['tile']) for door in design['doors'] if door['collision']==collision_name}
        assert actual==expected,'Unclaimed native '+collision_name+' footprint'
    assert all(kind(x,y)!='LADDER' for y in range(h*2) for x in range(w*2)),'Obsolete pink masonry ladder crossing'
    cut_rule=pack['data']['field_moves']['cut']
    assert cut_rule['move_id']=='CUT' and cut_rule['badge']=={'region':'johto','index':1},'Wrong native Cut prerequisite'
    report.update(portages=portages,native_cut_prerequisite='party move CUT plus Johto HiveBadge (badge index1)')
    report.update(status='Source terrain proposal only; previous 96×96 native evidence is historical',pre_cut_walk_tiles=len(pre),post_cut_walk_tiles=len(post_seen),pre_cut_walk_surf_tiles=len(pre_surf),post_cut_walk_surf_tiles=len(post_surf),cut_gates=gates,outer_water_tiles=0,outer_walking_openings=sorted(outer),water_tiles=len(water),native_block_varieties=len(set(blocks)),pre_cut_endpoint_steps=len(shortest_walk(start,end,allowed()))-1,post_cut_endpoint_steps=len(shortest_walk(start,end,allowed(True)))-1)
    assert not report['errors'],'; '.join(report['errors'])
    return report,kind,pre


def verify_published(design,layout,pack):
    """Compare current native chunks/events; stale native packs must fail."""
    from review_league_approach import verify_published as verify
    return verify(design,layout,pack)


def render(design, layout, pack, art):
    started = time.perf_counter()
    blocks = layout_blocks(layout); width,height = layout['dimensions_metatiles']
    assert len(art.blocks('forest')) == 40, 'native forest art extent'
    report,kind,seen = check(design,layout,pack,blocks)
    terrain = art.render('forest',width,height,blocks,derived=layout['derived_blocks'])
    prepared_art_checked=False
    if 'gfx/tilesets/mn_gunflint.png' in pack['runtime_files']:
        actual=art.blocks('mn_gunflint')
        assert len(actual)==58
        assert all(actual[i].tobytes()==im.tobytes() for i,im in enumerate(art.blocks('forest'))),'Original forest pixels changed in prepared pack'
        for entry in layout['derived_blocks']:
            assert actual[int(entry['id'],16)].tobytes()==art.blocks(entry['source_tileset'])[int(entry['source_block'],16)].tobytes(),'Prepared native import differs'
        prepared_art_checked=True
    scene = render_stage(terrain, 'convert', OUTPUT, 'review_gunflint-render-line173-12-convert', 'RGBA')
    for a in design['trainers']+design['residents']:
        scene.alpha_composite(oriented_sprite(pack,a['sprite'],a['facing']),(a['tile'][0]*16,a['tile'][1]*16-4))
    for a in design['items']: scene.alpha_composite(sprite(pack,'SPRITE_POKE_BALL'),(a['tile'][0]*16,a['tile'][1]*16-4))
    OUTPUT.mkdir(parents=True,exist_ok=True)
    save_render(terrain, OUTPUT/'terrain.png'); save_render(render_stage(scene, 'convert', OUTPUT, 'review_gunflint-render-line178-60-convert', 'RGB'), OUTPUT/'whole-route.png')
    digest = hashlib.sha256(scene.tobytes()).hexdigest()[:8]
    save_render(render_stage(scene, 'convert', OUTPUT, 'review_gunflint-render-line180-16-convert', 'RGB'), OUTPUT/f'whole-route-{digest}.png')
    save_render(render_stage(render_stage(scene, 'resize', OUTPUT, 'review_gunflint-render-line181-16-resize', (width*16,height*16),Image.Resampling.NEAREST), 'convert', OUTPUT, 'review_gunflint-render-line181-16-convert', 'RGB'), OUTPUT/'overview.png')
    overlay = scene.copy(); draw = ImageDraw.Draw(overlay,'RGBA')
    for row in range(design['chunk_rows']):
        for col in range(design['chunk_columns']):
            x,y = col*1024,row*768
            draw.rectangle((x,y,x+1023,y+767),outline=(255,255,255,120),width=1)
            draw.text((x+8,y+8),f'MnR12C{col:02}R{row:02}',fill='white',stroke_width=1,stroke_fill='black')
    for a in design['trainers']+design['residents']+design['items']+design['signs']+design['doors']+design['viewpoints']:
        x,y = a['tile']; draw.rectangle((x*16,y*16,x*16+15,y*16+15),outline=(255,230,80,255),width=2)
        draw.text((x*16,y*16-12),a['id'].removeprefix('MN_R12_'),fill='white',stroke_width=1,stroke_fill='black')
    save_render(render_stage(overlay, 'convert', OUTPUT, 'review_gunflint-render-line191-16-convert', 'RGB'), OUTPUT/'anchors.png')
    views=[]
    for district in design['districts']:
        x0,y0,x1,y1 = district['bounds_metatiles']
        save_render(render_stage(render_stage(scene, 'crop', OUTPUT, 'review_gunflint-render-line195-20-crop', (x0*32,y0*32,x1*32,y1*32)), 'convert', OUTPUT, 'review_gunflint-render-line195-20-convert', 'RGB'), OUTPUT/('district-'+district['id'].lower()+'.png'))
    for view in design['viewpoints']:
        x,y = view['tile']
        im=render_stage(render_stage(scene, 'crop', OUTPUT, 'review_gunflint-render-line198-11-crop', (x*16-80,y*16-72,x*16+80,y*16+72)), 'convert', OUTPUT, 'review_gunflint-render-line198-11-convert', 'RGB'); views.append((view['id'],im))
        save_render(render_stage(im, 'resize', OUTPUT, 'review_gunflint-render-line199-20-resize', (640,576),Image.Resampling.NEAREST), OUTPUT/('viewport-'+view['id'].lower()+'.png'))
    save_render(room_sheet(views,3), OUTPUT/'viewpoints.png')
    save_render(room_sheet(views,3), OUTPUT/f'viewpoints-{digest}.png')
    after=list(blocks)
    for gate in design['cut_gates']:
        x,y=gate['tile'];after[y//2*width+x//2]=int(gate['replacement_block'],16)
    after_scene=render_stage(art.render('forest',width,height,after,derived=layout['derived_blocks']), 'convert', OUTPUT, 'review_gunflint-render-line205-16-convert', 'RGBA')
    for a in design['trainers']+design['residents']:after_scene.alpha_composite(oriented_sprite(pack,a['sprite'],a['facing']),(a['tile'][0]*16,a['tile'][1]*16-4))
    for a in design['items']:after_scene.alpha_composite(sprite(pack,'SPRITE_POKE_BALL'),(a['tile'][0]*16,a['tile'][1]*16-4))
    after_digest=hashlib.sha256(after_scene.tobytes()).hexdigest()[:8];save_render(render_stage(after_scene, 'convert', OUTPUT, 'review_gunflint-render-line208-83-convert', 'RGB'), OUTPUT/f'after-cut-{after_digest}.png')
    x,y=design['cut_gates'][0]['approach'];box=(x*16-80,y*16-72,x*16+80,y*16+72)
    for name,im in [('before',scene),('after',after_scene)]:save_render(render_stage(render_stage(render_stage(im, 'crop', OUTPUT, 'review_gunflint-render-line210-72-crop', box), 'convert', OUTPUT, 'review_gunflint-render-line210-72-convert', 'RGB'), 'resize', OUTPUT, 'review_gunflint-render-line210-72-resize', (640,576),Image.Resampling.NEAREST), OUTPUT/f'cut-{name}-{digest}.png')
    rooms = []
    for room in design['interiors']:
        tiles = [int(b,16) for row in room['rows'] for b in row.split()]
        tileset = pack['data']['maps'][room['template']]['attributes']['tileset_name']
        group = pack['data']['maps'][room['template']]['attributes']['time_of_day'] or 'day'
        image = render_stage(art.render(tileset,*room['dimensions_metatiles'],tiles,group=group), 'convert', OUTPUT, 'review_gunflint-render-line216-16-convert', 'RGBA')
        for a in room.get('residents',[]): image.alpha_composite(oriented_sprite(pack,a['sprite'],a['facing']),(a['tile'][0]*16,a['tile'][1]*16-4))
        for a in room.get('items',[]): image.alpha_composite(sprite(pack,'SPRITE_POKE_BALL'),(a['tile'][0]*16,a['tile'][1]*16-4))
        rooms.append((room['id'],render_stage(image, 'convert', OUTPUT, 'review_gunflint-render-line219-33-convert', 'RGB')))
    save_render(room_sheet(rooms,columns=2), OUTPUT/'interiors.png')
    report.update(status='Source geometry checked; consult the route runbook for current native publication, controller and earned progression evidence',source_native_blocks_used=58,prepared_art_pixels_equal=prepared_art_checked,
                  render_ms=round((time.perf_counter()-started)*1000,1),image=f'whole-route-{digest}.png',after_cut_image=f'after-cut-{after_digest}.png')
    (OUTPUT/'revision.json').write_text(json.dumps(report,indent=2)+'\n')
    return report


HTML = '''<!doctype html><meta charset="utf-8"><meta name="viewport" content="width=device-width">
<title>Gunflint Trail review</title><style>
body{margin:0;background:#14212b;color:#e7eff2;font:15px system-ui}header{position:sticky;top:0;background:#14212bf5;padding:16px;z-index:1}h1{font-size:22px;margin:0 0 8px}button{padding:8px 12px;background:#305165;color:white;border:0;margin:3px;cursor:pointer}main{padding:16px}.scroll{overflow:auto;max-height:78vh}img{image-rendering:pixelated}#map{width:1536px}.views{display:flex;flex-wrap:wrap;gap:12px}.views img{width:480px}#status{color:#b5e5cd}p{max-width:1000px}
</style><header><h1>Gunflint Trail — one complete authored route</h1><button data-layer="whole-route">Residents</button><button data-layer="terrain">Terrain</button><button data-layer="anchors">Anchors + chunks</button><button id="zoom">Half / native pixels</button><span id="status">Rendering…</span></header>
<main><p>Six clipped chunks. West Portage Lake · Windthrow saddle · Loons Lake · Stone Bog · Ranger clearing · Bear Hollow. Exact packed source pixels. This preview checks the physical plan; native publication and controller evidence are recorded separately.</p><div class="scroll"><img id="map"></div><h2>Native 160 × 144 framing, enlarged 4×</h2><div class="views"></div><h2>Authored rooms</h2><img id="rooms"></main><script>
let layer='whole-route',rev='',native=false;const map=document.querySelector('#map');function refresh(){map.src=layer+'.png?r='+rev;document.querySelectorAll('[data-view]').forEach(i=>i.src='viewport-'+i.dataset.view+'.png?r='+rev);document.querySelector('#rooms').src='interiors.png?r='+rev}for(const name of ['west_lake','loons_bank','cleft_approach','ranger_porch','bog_bank','bear_hollow','widest_ranger_yard','cut_approach','weak_west_bank','weak_north_loons']){let i=document.createElement('img');i.dataset.view=name;i.alt=name;document.querySelector('.views').appendChild(i)}document.querySelectorAll('[data-layer]').forEach(b=>b.onclick=()=>{layer=b.dataset.layer;refresh()});document.querySelector('#zoom').onclick=()=>{native=!native;map.style.width=(native?2048:1536)+'px'};setInterval(async()=>{let s=await(await fetch('/state',{cache:'no-store'})).json();let e=document.querySelector('#status');if(!s.ok){e.textContent='INVALID — previous image retained: '+s.error;e.style.color='#ff9b9b';return}if(s.revision!==rev){rev=s.revision;refresh();e.textContent=s.render_ms+' ms · '+s.clear_reachable_tiles+' clear tiles · static review';e.style.color='#b5e5cd'}},500)
</script>'''


def negative_checks(design,layout,pack):
    """Reject malformed source copies; never mutate packs or saved designs."""
    from copy import deepcopy
    cases={
        'Cut walking bypass':dict(kind='rect',material='lawn',bounds=[26,11,27,13]),
        'Cut Surf bypass':dict(kind='rect',material='water',bounds=[26,11,27,13]),
        'Undeclared Cut canopy':dict(kind='point',position=[5,15],block='0f'),
        'Unconnected outer Surf edge':dict(kind='point',position=[0,29],block='02'),
        'Unplanned walking edge':dict(kind='point',position=[0,31],block='01'),
        'Paved required grass':dict(kind='rect',material='lawn',bounds=[28,32,29,34]),
        'Paved Loons portage':dict(kind='rect',material='lawn',bounds=[49,18,54,19]),
        'Fake Raven entrance':dict(kind='point',position=[33,11],block='29'),
        'Fake hut door':dict(kind='point',position=[42,41],block='27'),
        'Paved bog portage':dict(kind='point',position=[16,42],block='01'),
        'Diagonal polygon closing edge':dict(kind='polygon',material='forest',points=[[4,4],[5,4],[5,5]]),
    }
    outcomes=[]
    for name,layer in cases.items():
        candidate=deepcopy(layout);candidate['layers'].append(dict(name='Negative '+name,**layer))
        try:check(design,candidate,pack,layout_blocks(candidate))
        except AssertionError as error:outcomes.append(dict(mutation=name,rejected=True,reason=str(error)))
        else:raise AssertionError('Uncaught malformed copy: '+name)
    candidate=deepcopy(design);candidate['interiors'][0]['exits']=[[16,23]]
    try:check(candidate,layout,pack,layout_blocks(layout))
    except AssertionError as error:outcomes.append(dict(mutation='Wrong cave exit quadrant',rejected=True,reason=str(error)))
    else:raise AssertionError('Uncaught wrong cave exit')
    result=dict(source_only=True,cases=outcomes,layout_sha256=hashlib.sha256(LAYOUT.read_bytes()).hexdigest(),dossier_sha256=hashlib.sha256(DOSSIER.read_bytes()).hexdigest())
    (OUTPUT/'negative-checks.json').write_text(json.dumps(result,indent=2)+'\n');return result


def main():
    parser = argparse.ArgumentParser(); parser.add_argument('--pack',type=Path,required=True,help='Explicit source-art pack for historical layout review; no current pack is selected implicitly')
    parser.add_argument('--watch',action='store_true'); parser.add_argument('--serve',action='store_true'); parser.add_argument('--port',type=int,default=8767)
    parser.add_argument('--verify-published',action='store_true'); parser.add_argument('--negative-checks',action='store_true')
    args = parser.parse_args()
    if args.negative_checks and (args.watch or args.serve): parser.error('--negative-checks requires one-shot source review')
    if args.verify_published and (args.watch or args.serve): parser.error('--verify-published requires a single run')
    pack = load_pack(args.pack); art = SourceArt(pack); last = None
    state = {'value':{'ok':False,'error':'initial render pending'}}
    if args.serve:
        OUTPUT.mkdir(parents=True,exist_ok=True); (OUTPUT/'index.html').write_text(HTML)
        class Handler(SimpleHTTPRequestHandler):
            def log_message(self,*args): pass
            def do_GET(self):
                if self.path.split('?')[0] == '/state':
                    body = json.dumps(state['value']).encode(); self.send_response(200); self.send_header('Content-Type','application/json')
                    self.send_header('Cache-Control','no-store'); self.send_header('Content-Length',str(len(body))); self.end_headers(); self.wfile.write(body)
                else: super().do_GET()
        server = ThreadingHTTPServer(('127.0.0.1',args.port),partial(Handler,directory=str(OUTPUT)))
        threading.Thread(target=server.serve_forever,daemon=True).start(); print(f'Gunflint review: http://127.0.0.1:{args.port}/',flush=True)
    while True:
        revision = LAYOUT.stat().st_mtime_ns,DOSSIER.stat().st_mtime_ns
        if revision != last:
            try:
                design,layout = json.loads(DOSSIER.read_text()),json.loads(LAYOUT.read_text())
                published = verify_published(design,layout,pack) if args.verify_published else {}
                report = {**render(design,layout,pack,art),**published}
                state['value'] = {'ok':True,'revision':str(time.time_ns()),**report}; print(json.dumps(report),flush=True)
            except (AssertionError,KeyError,ValueError,IndexError,TypeError) as error:
                state['value'] = {'ok':False,'error':str(error)}; print(f'DESIGN ERROR: {error}',flush=True)
                if not(args.watch or args.serve): raise SystemExit(1)
            last = revision
        if not(args.watch or args.serve):
            if args.negative_checks:print(json.dumps(negative_checks(design,layout,pack)),flush=True)
            break
        time.sleep(.2)


if __name__ == '__main__': main()
