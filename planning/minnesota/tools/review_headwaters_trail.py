#!/usr/bin/env python3
"""Native-pixel loop for explicit Headwaters Trail authoring; never chooses terrain."""
from render_archive import save_render
from render_stages import render_stage, retain_render_bytes
import argparse
from functools import partial
from http.server import SimpleHTTPRequestHandler,ThreadingHTTPServer
import json
import hashlib
from collections import deque
from pathlib import Path
import threading
import time

from PIL import Image

from render_reference import SourceArt,layout_blocks,load_pack
from review_headwaters import prepare
from review_league_approach import render as render_route,verify_published
from review_gunflint import flood,neighbors

ROOT=Path(__file__).resolve().parents[3]
LAYOUT=ROOT/'planning/minnesota/routes/MN_R14.layout.json'
DOSSIER=ROOT/'planning/minnesota/routes/MN_R14.json'
OUTPUT=ROOT/'planning/minnesota/renders/headwaters-trail'
HTML='''<!doctype html><meta charset="utf-8"><title>Headwaters Trail review</title><style>body{background:#14212b;color:#e7eff2;font:16px system-ui;margin:20px}img{image-rendering:pixelated}header{position:sticky;top:0;background:#14212bf5;padding:12px}button{margin:4px;padding:10px}.scroll{overflow:auto;max-height:70vh}#map{width:1536px}</style><header><h1>Headwaters Trail — complete hand-authored route</h1><button data-layer="whole-route">Residents</button><button data-layer="terrain">Terrain</button><button data-layer="anchors">Anchors</button><button data-layer="ledges">Directed bank</button><p id="status">Rendering…</p></header><p>Survey Pond · Young river boardwalk · Cedar bank · Mature pine Cut return · Fern hollow · Fen outlook. Static source review; regional connections and controller evidence remain separate.</p><div class="scroll"><img id="map"></div><h2>Native 160×144 camera views, enlarged 2×</h2><img id="views"><script>let layer='whole-route',rev='';function refresh(){document.querySelector('#map').src=layer+'.png?r='+rev;document.querySelector('#views').src='viewpoints.png?r='+rev}document.querySelectorAll('[data-layer]').forEach(b=>b.onclick=()=>{layer=b.dataset.layer;refresh()});setInterval(async()=>{const s=await(await fetch('/state',{cache:'no-store'})).json();document.querySelector('#status').textContent=s.ok?s.render_ms+' ms · '+s.status:'INVALID: '+s.error;if(s.revision!==rev){rev=s.revision;refresh()}},400)</script>'''


def border_contract(design,layout,pack):
    from ledge_paths import shortest_walk
    blocks=layout_blocks(layout);w,h=layout['dimensions_metatiles'];collision=pack['data']['tilesets'][layout['compiled_tileset']]['collision']
    def kind(x,y):return collision[f'{blocks[y//2*w+x//2]:02x}'][y%2*2+x%2]
    for layer in layout['layers']:
        if layer['kind']in{'polygon','cliff_polygon','path'}:
            points=layer['points'];ends=points[1:]if layer['kind']=='path'else points[1:]+points[:1]
            assert all(a[0]==b[0]or a[1]==b[1]for a,b in zip(points,ends)),layer['name']+': diagonal native polygon'
    actors={tuple(a['tile'])for k in ['trainers','residents','items']for a in design[k]}
    walking={(x,y)for y in range(h*2)for x in range(w*2)if kind(x,y)in{'FLOOR','TALL_GRASS','LADDER'}and(x,y)not in actors}
    start,end=[tuple(v*2 for v in e['gate_metatile'])for e in design['endpoints']]
    assert end in flood(start,walking),'Main route is field-move dependent'
    band=design['required_grass_crossings'][0]['bounds_core'];x0,y0,x1,y1=band
    clear={p for p in walking if kind(*p)in{'FLOOR','LADDER'}or x0<=p[0]<x1 and y0<=p[1]<y1}
    assert end in flood(start,clear),'Main road needs unintended optional grass'
    bridge=layout['crossings'][0]
    for p in bridge['core_walk_line']:assert kind(*p)=='FLOOR','Blocked native crossing'
    for key,expected in [('north_rail_blocks',['WALL','FLOOR']),('south_rail_blocks',['FLOOR','WALL'])]:
        assert all([kind(x*2,y*2),kind(x*2,y*2+1)]==expected for x,y in bridge[key]),'Bridge rail collision'
    deck={(x*2+dx,y*2+dy)for key in ['north_rail_blocks','south_rail_blocks']for x,y in bridge[key]for dx in[0,1]for dy in[0,1]}
    assert end not in flood(start,walking-deck),'Free walking river bypass'
    perimeter={(x,0)for x in range(w*2)}|{(x,h*2-1)for x in range(w*2)}|{(0,y)for y in range(h*2)}|{(w*2-1,y)for y in range(h*2)}
    assert not any(kind(*p)in{'WATER','WATERFALL','WHIRLPOOL'}for p in perimeter),'Open outer Surf edge'
    assert{p for p in perimeter if kind(*p)in{'FLOOR','TALL_GRASS','LADDER'}}=={(0,42),(0,43),(95,26),(95,27)},'Unplanned walking port'
    for block in layout['additional_water_blocks']:assert set(collision[block])=={'BUOY','WATER'},'Non-native water rock'
    gate=design['cut_gates'][0];tree=tuple(gate['tile']);inventory={(x,y)for y in range(h*2)for x in range(w*2)if kind(x,y)=='CUT_TREE'}
    assert inventory=={tree},'Incidental Cut canopy'
    source=blocks[tree[1]//2*w+tree[0]//2]
    replacement=int(gate['replacement_block'],16);native=pack['data']['field_moves']['cut']
    assert native['replacements']['johto'][source]['replacement_block_id']==replacement,'Wrong Cut replacement'
    assert native['badge']=={'region':'johto','index':1},'Wrong native Cut permission'
    post=walking|{tree}
    near,far=(71,21),(71,19)
    before_path=shortest_walk(near,far,walking);after_path=shortest_walk(near,far,post)
    assert before_path and after_path and len(before_path)-len(after_path)>=12,'Cut is not a useful shortened circuit'
    assert kind(70,20)not in{'FLOOR','TALL_GRASS'}and kind(72,20)not in{'FLOOR','TALL_GRASS'},'Tree has a local walk-around'
    ledge=design['one_way_ledges'][0]
    assert all(kind(x,24)=='HOP_DOWN'and kind(x,25)=='WALL'for x in range(20,36)),'Low bank face'
    assert all(kind(*p)=='WALL'for p in ledge['side_barriers']),'Low bank side bypass'
    assert all(kind(10,y)=='WALL'for y in range(16,20))and kind(11,16)=='WALL'and kind(11,19)=='WALL','Survey landing has an open tip/corner'
    # Resource entry price is checked against all approaches, not its image.
    seed=next(a for a in design['items']if a['id']=='MN_R14_ITEM_02')
    costs={start:0};todo=deque([start])
    while todo:
        at=todo.popleft()
        for q in neighbors(at):
            if q not in walking:continue
            cost=costs[at]+int(kind(*q)=='TALL_GRASS')
            if cost<costs.get(q,10**9):costs[q]=cost;(todo.append if kind(*q)=='TALL_GRASS'else todo.appendleft)(q)
    seed_price=min(costs.get(q,10**9)for q in neighbors(tuple(seed['tile'])))
    assert 2<=seed_price<10**9,'Seed meadow resource has a free dry approach'
    maps=pack['data']['maps'];names={f'MnR14C{c:02}R{r:02}'for r in range(2)for c in range(2)};published=bool(names&maps.keys())
    if published:
        assert names<=maps.keys(),'Partial Headwaters native publication'
        for row in range(2):
            west,east=f'MnR14C00R{row:02}',f'MnR14C01R{row:02}'
            for source,direction,target in [(west,'east',east),(east,'west',west)]:assert {'direction':direction,'target_map':target,'offset':0}in maps[source]['attributes']['connections'],'Missing reciprocal horizontal neighbor'
        for col in range(2):
            north,south=f'MnR14C{col:02}R00',f'MnR14C{col:02}R01'
            for source,direction,target in [(north,'south',south),(south,'north',north)]:assert {'direction':direction,'target_map':target,'offset':0}in maps[source]['attributes']['connections'],'Missing reciprocal vertical water neighbor'
    vertical=[y for y in range(h*2)if kind(63,y)=='WATER'and kind(64,y)=='WATER']
    horizontal=[x for x in range(w*2)if kind(x,47)=='WATER'and kind(x,48)=='WATER']
    return dict(seed_meadow_grass_steps=seed_price,published_reciprocal_neighbors_verified=published,necessary_native_bridges=1,outer_water_cells=0,incidental_cut_quadrants=0,cut_before_walk_steps=len(before_path)-1,cut_after_walk_steps=len(after_path)-1,cut_saved_walk_steps=len(before_path)-len(after_path),cut_prerequisite='Native party CUT and Johto badge index1; source proof does not execute field move or persistence',internal_vertical_water_pairs=len(vertical),internal_horizontal_water_pairs=len(horizontal),source_metatile_varieties=len(set(blocks)),main_road_only_required_neck_grass=True)


def render(design,layout,pack,art):
    started=time.perf_counter()
    prepare(pack,layout)
    from copy import deepcopy
    rules=pack['data']['field_moves']['cut']['replacements']
    rules.setdefault('mn_headwaters',deepcopy(rules['johto']))
    extra=border_contract(design,layout,pack)
    from copy import deepcopy
    plain=deepcopy(design);plain['shortcut']=None
    report=render_route(plain,layout,pack,art,OUTPUT)
    opened=deepcopy(layout);gate=design['cut_gates'][0];x,y=gate['tile'];opened['layers'].append(dict(kind='point',name='Native Cut replacement',position=[x//2,y//2],block=gate['replacement_block']))
    after_design=deepcopy(plain);after_design['cut_gates']=[]
    after_report=render_route(after_design,opened,pack,art,OUTPUT/'after-cut-views')
    assert not after_report['errors'],after_report['errors']
    data=(OUTPUT/'after-cut-views'/after_report['image']).read_bytes();after_name='whole-route-after-cut-'+hashlib.sha256(data).hexdigest()[:8]+'.png';retain_render_bytes(data, OUTPUT/after_name);report['after_cut_image']=after_name
    report.update(extra)
    # Locate the largest genuinely blank ground square so review does not only
    # pick flattering landmarks. This is analysis of explicit authored blocks.
    blocks=layout_blocks(layout);w,h=layout['dimensions_metatiles']
    occupied={tuple(v//2 for v in a['tile'])for key in ['trainers','residents','items']for a in design[key]}
    collision=pack['data']['tilesets'][layout['compiled_tileset']]['collision']
    actor_tiles={tuple(a['tile'])for key in ['trainers','residents','items']for a in design[key]}
    walk={(x,y)for y in range(h*2)for x in range(w*2)if collision[f'{blocks[y//2*w+x//2]:02x}'][y%2*2+x%2]in{'FLOOR','TALL_GRASS','LADDER'}and(x,y)not in actor_tiles}
    reached=flood(tuple(v*2 for v in design['endpoints'][0]['gate_metatile']),walk)
    sizes={};best=(0,0,0)
    for y in range(h):
        for x in range(w):
            size=1+min(sizes.get((x-1,y),0),sizes.get((x,y-1),0),sizes.get((x-1,y-1),0))if blocks[y*w+x]in{1,2}and(x,y)not in occupied and(x*2,y*2)in reached else 0
            sizes[x,y]=size
            if size>best[0]:best=size,x-size+1,y-size+1
    size,x,y=best;cx,cy=x*2+size,y*2+size
    scene=Image.open(OUTPUT/'whole-route.png')
    save_render(render_stage(render_stage(scene, 'crop', OUTPUT, 'review_headwaters_trail-render-line125-16-crop', (cx*16-80,cy*16-72,cx*16+80,cy*16+72)), 'resize', OUTPUT, 'review_headwaters_trail-render-line125-16-resize', (640,576),Image.Resampling.NEAREST), OUTPUT/'viewport-emptiest_ground.png')
    report.update(largest_blank_ground_square_metatiles=size,emptiest_ground_camera_core=[cx,cy])
    for state,folder in [('before',OUTPUT),('after',OUTPUT/'after-cut-views')]:
        data=(folder/'viewport-cut_return.png').read_bytes();digest=hashlib.sha256(data).hexdigest()[:8];name='cut-gate-'+state+'-'+digest+'.png';retain_render_bytes(data, OUTPUT/name);report['cut_gate_'+state+'_image']=name
    for name in ['viewpoints','viewport-emptiest_ground']:
        data=(OUTPUT/(name+'.png')).read_bytes();digest=hashlib.sha256(data).hexdigest()[:8]
        immutable=name+'-'+digest+'.png';retain_render_bytes(data, OUTPUT/immutable);report[name+'_image']=immutable
    report['render_ms']=round((time.perf_counter()-started)*1000,1)
    (OUTPUT/'revision.json').write_text(json.dumps(report,indent=2)+'\n')
    return report


def main():
    p=argparse.ArgumentParser();p.add_argument('--pack',type=Path,default=ROOT/'content-packs/text-tui.crystalpack');p.add_argument('--serve',action='store_true');p.add_argument('--watch',action='store_true');p.add_argument('--port',type=int,default=8788);p.add_argument('--verify-published',action='store_true');args=p.parse_args()
    pack=load_pack(args.pack);art=SourceArt(pack);state={'ok':False,'error':'Initial render pending'}
    if args.serve:
        OUTPUT.mkdir(parents=True,exist_ok=True);(OUTPUT/'index.html').write_text(HTML)
        class Handler(SimpleHTTPRequestHandler):
            def log_message(self,*args):pass
            def do_GET(self):
                if self.path.split('?')[0]=='/state':
                    body=json.dumps(state).encode();self.send_response(200);self.send_header('Content-Type','application/json');self.send_header('Cache-Control','no-store');self.send_header('Content-Length',str(len(body)));self.end_headers();self.wfile.write(body)
                else:super().do_GET()
        server=ThreadingHTTPServer(('127.0.0.1',args.port),partial(Handler,directory=str(OUTPUT)));threading.Thread(target=server.serve_forever,daemon=True).start();print(f'Headwaters Trail: http://127.0.0.1:{args.port}/',flush=True)
    last=None
    while True:
        current=LAYOUT.stat().st_mtime_ns,DOSSIER.stat().st_mtime_ns
        if current!=last:
            try:
                design,layout=json.loads(DOSSIER.read_text()),json.loads(LAYOUT.read_text());report=render(design,layout,pack,art)
                if args.verify_published:report.update(verify_published(design,layout,pack))
                state.clear();state.update(ok=not report['errors'],revision=str(time.time_ns()),error='; '.join(report['errors']),**report);print(json.dumps(report),flush=True)
                if report['errors']and not(args.serve or args.watch):raise SystemExit(1)
            except(AssertionError,KeyError,ValueError,TypeError,IndexError)as error:
                state.clear();state.update(ok=False,error=str(error));print('DESIGN ERROR: '+str(error),flush=True)
                if not(args.serve or args.watch):raise SystemExit(1)
            last=current
        if not(args.serve or args.watch):break
        time.sleep(.2)


if __name__=='__main__':main()
