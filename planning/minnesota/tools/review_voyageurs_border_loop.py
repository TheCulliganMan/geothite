#!/usr/bin/env python3
"""Native-pixel loop for explicit Voyageurs Border Loop authoring; never chooses terrain."""
from render_archive import save_render
from render_stages import render_stage, retain_render_bytes
import argparse
from functools import partial
from http.server import SimpleHTTPRequestHandler,ThreadingHTTPServer
import json
import hashlib
from pathlib import Path
import threading
import time

from PIL import Image

from render_reference import SourceArt,layout_blocks,load_pack
from review_headwaters import prepare
from review_league_approach import render as render_route,verify_published
from review_gunflint import flood

ROOT=Path(__file__).resolve().parents[3]
LAYOUT=ROOT/'planning/minnesota/routes/MN_R20.layout.json'
DOSSIER=ROOT/'planning/minnesota/routes/MN_R20.json'
OUTPUT=ROOT/'planning/minnesota/renders/voyageurs-border-loop'
HTML='''<!doctype html><meta charset="utf-8"><title>Voyageurs Border Loop review</title><style>body{background:#14212b;color:#e7eff2;font:16px system-ui;margin:20px}img{image-rendering:pixelated}header{position:sticky;top:0;background:#14212bf5;padding:12px}button{margin:4px;padding:10px}.scroll{overflow:auto;max-height:70vh}#map{width:2048px}</style><header><h1>Voyageurs Border Loop — complete hand-authored route</h1><button data-layer="whole-route">Residents</button><button data-layer="terrain">Terrain</button><button data-layer="anchors">Anchors</button><button data-layer="ledges">Directed bank</button><p id="status">Rendering…</p></header><p>Falls lake pier · Bridged outlets · Granite narrows · Two-chamber cave · Sheltered meadow · Boundary shore neck. Static source review; regional connections and controller evidence remain separate.</p><div class="scroll"><img id="map"></div><h2>Native 160×144 camera views, enlarged 2×</h2><img id="views"><script>let layer='whole-route',rev='';function refresh(){document.querySelector('#map').src=layer+'.png?r='+rev;document.querySelector('#views').src='viewpoints.png?r='+rev}document.querySelectorAll('[data-layer]').forEach(b=>b.onclick=()=>{layer=b.dataset.layer;refresh()});setInterval(async()=>{const s=await(await fetch('/state',{cache:'no-store'})).json();document.querySelector('#status').textContent=s.ok?s.render_ms+' ms · '+s.status:'INVALID: '+s.error;if(s.revision!==rev){rev=s.revision;refresh()}},400)</script>'''


def border_contract(design,layout,pack):
    for layer in layout['layers']:
        if layer['kind']in{'polygon','cliff_polygon','path'}:
            points=layer['points'];ends=points[1:]if layer['kind']=='path'else points[1:]+points[:1]
            assert all(a[0]==b[0]or a[1]==b[1]for a,b in zip(points,ends)),layer['name']+': diagonal native polygon'
    blocks=layout_blocks(layout);w,h=layout['dimensions_metatiles'];collision=pack['data']['tilesets'][layout['compiled_tileset']]['collision']
    def kind(x,y):return collision[f'{blocks[y//2*w+x//2]:02x}'][y%2*2+x%2]
    for block in layout['additional_water_blocks']:
        assert set(collision[block])=={'BUOY','WATER'},'Water closure must use actual native blue-water rock collision'
    occupied={tuple(a['tile'])for key in ['trainers','residents','items']for a in design[key]}
    walking={(x,y)for y in range(h*2)for x in range(w*2)if kind(x,y)in{'FLOOR','TALL_GRASS','LADDER'}and(x,y)not in occupied}
    start,end=[tuple(v*2 for v in e['gate_metatile'])for e in design['endpoints']]
    # The road works on clear native floor except its explicitly priced neck.
    # Optional habitat shoulders must not silently become required detours.
    gx0,gy0,gx1,gy1=design['required_grass_crossings'][0]['bounds_core']
    neck={(x,y)for y in range(gy0,gy1)for x in range(gx0,gx1)}
    clear={p for p in walking if kind(*p)in{'FLOOR','LADDER'}or p in neck}
    assert end in flood(start,clear),'Painted main road requires unintended grass detour'
    for c in layout['crossings']:
        assert all(kind(*p)=='FLOOR'for p in c['core_walk_line']),c['id']+': interrupted deck'
        assert all([kind(x*2,y*2),kind(x*2,y*2+1)]==['WALL','FLOOR']for x,y in c['north_rail_blocks']),c['id']+': north rail'
        assert all([kind(x*2,y*2),kind(x*2,y*2+1)]==['FLOOR','WALL']for x,y in c['south_rail_blocks']),c['id']+': south rail'
        cells={(x*2+dx,y*2+dy)for key in ['north_rail_blocks','south_rail_blocks']for x,y in c[key]for dx in[0,1]for dy in[0,1]}
        assert end not in flood(start,walking-cells),c['id']+': free walking bypass'
    assert all('CUT_TREE'not in collision[f'{block:02x}']for block in blocks),'Incidental decorative Cut tree'
    perimeter={(x,0)for x in range(w*2)}|{(x,h*2-1)for x in range(w*2)}|{(0,y)for y in range(h*2)}|{(w*2-1,y)for y in range(h*2)}
    assert all(kind(*p)not in{'WATER','WATERFALL','WHIRLPOOL'}for p in perimeter),'Unconnected exterior water edge'
    assert{p for p in perimeter if kind(*p)in{'FLOOR','TALL_GRASS','LADDER'}}=={(0,44),(0,45),(w*2-1,36),(w*2-1,37)},'Unplanned exterior walking port'
    ledge=design['one_way_ledges'][0]
    assert all(kind(x,50)=='HOP_DOWN'and kind(x,51)=='WALL'for x in range(18,22)),'Native wind shelter bank face'
    assert all(kind(*p)=='WALL'for p in ledge['side_barriers']),'Wind shelter reverse side opening'
    assert all(kind(19,y)=='WALL'for y in range(24,28))and kind(18,24)=='WALL'and kind(18,27)=='WALL','Closed lake pier end'
    from ledge_paths import shortest_walk
    room=design['interiors'][0];rw,rh=room['dimensions_metatiles']
    rb=[int(v,16)for row in room['rows']for v in row.split()]
    rc=pack['data']['tilesets'][pack['data']['maps'][room['template']]['attributes']['tileset_name']]['collision']
    def room_kind(x,y):return rc[f'{rb[y//2*rw+x//2]:02x}'][y%2*2+x%2]
    item=room['items'][0];rtaken={tuple(item['tile'])}
    inside={(x,y)for y in range(rh*2)for x in range(rw*2)if room_kind(x,y)in{'FLOOR','LADDER'}and(x,y)not in rtaken}
    inside_steps=len(shortest_walk(tuple(room['exits'][0]),tuple(room['exits'][1]),inside))-1
    assert inside_steps>0,'Cave mouths disconnected'
    assert room['exit_doors']==[a['tile']for a in design['doors']],'Cave mouth identity mismatch'
    for i,door in enumerate(design['doors']):
        assert door['target_warp']==i+1 and door['return_warp_index']==2,'Cave warp ownership'
        assert kind(*door['tile'])=='CAVE'and kind(*door['return_tile'])=='FLOOR','Unsafe mouth return'
    outside_steps=len(shortest_walk(tuple(design['doors'][0]['return_tile']),tuple(design['doors'][1]['return_tile']),walking))-1
    vertical=[y for y in range(h*2)if kind(63,y)=='WATER'and kind(64,y)=='WATER']
    horizontal=[x for x in range(w*2)if kind(x,47)=='WATER'and kind(x,48)=='WATER']
    maps=pack['data']['maps'];names={f'MnR20C{c:02}R{r:02}'for r in range(2)for c in range(2)}
    published=bool(names & maps.keys())
    if published:
        assert names <= maps.keys(),'Partially published Voyageurs exterior'
        for row in range(2):
            west,east=f'MnR20C00R{row:02}',f'MnR20C01R{row:02}'
            for source,direction,target in [(west,'east',east),(east,'west',west)]:
                assert {'direction':direction,'target_map':target,'offset':0}in maps[source]['attributes']['connections'],'Missing reciprocal east/west water neighbor'
        for col in range(2):
            north,south=f'MnR20C{col:02}R00',f'MnR20C{col:02}R01'
            for source,direction,target in [(north,'south',south),(south,'north',north)]:
                assert {'direction':direction,'target_map':target,'offset':0}in maps[source]['attributes']['connections'],'Missing reciprocal north/south water neighbor'
    return dict(published_reciprocal_water_neighbors_verified=published,cave_mouth_inside_walk_steps=inside_steps,cave_mouth_outside_walk_steps=outside_steps,main_road_only_required_neck_grass=True,necessary_protected_outlet_bridges=2,unconnected_outer_water_cells=0,incidental_cut_quadrants=0,internal_water_vertical_pairs=len(vertical),internal_water_horizontal_pairs=len(horizontal),source_metatile_varieties=len(set(blocks)),travel='Free ordinary mainland walking; no Safari admission/balls/step lifecycle authored.')


def render(design,layout,pack,art):
    started=time.perf_counter()
    prepare(pack,layout)
    extra=border_contract(design,layout,pack)
    report=render_route(design,layout,pack,art,OUTPUT)
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
    save_render(render_stage(render_stage(scene, 'crop', OUTPUT, 'review_voyageurs_border_loop-render-line112-16-crop', (cx*16-80,cy*16-72,cx*16+80,cy*16+72)), 'resize', OUTPUT, 'review_voyageurs_border_loop-render-line112-16-resize', (640,576),Image.Resampling.NEAREST), OUTPUT/'viewport-emptiest_ground.png')
    report.update(largest_blank_ground_square_metatiles=size,emptiest_ground_camera_core=[cx,cy])
    for name in ['viewpoints','viewport-emptiest_ground','interiors']:
        data=(OUTPUT/(name+'.png')).read_bytes();digest=hashlib.sha256(data).hexdigest()[:8]
        immutable=name+'-'+digest+'.png';retain_render_bytes(data, OUTPUT/immutable);report[name+'_image']=immutable
    report['render_ms']=round((time.perf_counter()-started)*1000,1)
    (OUTPUT/'revision.json').write_text(json.dumps(report,indent=2)+'\n')
    return report


def main():
    p=argparse.ArgumentParser();p.add_argument('--pack',type=Path,default=ROOT/'content-packs/text-tui.crystalpack');p.add_argument('--serve',action='store_true');p.add_argument('--watch',action='store_true');p.add_argument('--port',type=int,default=8787);p.add_argument('--verify-published',action='store_true');args=p.parse_args()
    pack=load_pack(args.pack);art=SourceArt(pack);state={'ok':False,'error':'Initial render pending'}
    if args.serve:
        OUTPUT.mkdir(parents=True,exist_ok=True);(OUTPUT/'index.html').write_text(HTML)
        class Handler(SimpleHTTPRequestHandler):
            def log_message(self,*args):pass
            def do_GET(self):
                if self.path.split('?')[0]=='/state':
                    body=json.dumps(state).encode();self.send_response(200);self.send_header('Content-Type','application/json');self.send_header('Cache-Control','no-store');self.send_header('Content-Length',str(len(body)));self.end_headers();self.wfile.write(body)
                else:super().do_GET()
        server=ThreadingHTTPServer(('127.0.0.1',args.port),partial(Handler,directory=str(OUTPUT)));threading.Thread(target=server.serve_forever,daemon=True).start();print(f'Voyageurs Border Loop: http://127.0.0.1:{args.port}/',flush=True)
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
