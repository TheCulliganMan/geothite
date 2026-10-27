#!/usr/bin/env python3
"""Native-pixel loop for explicit Red River Prairie authoring; never chooses terrain."""
from render_archive import save_render
from render_stages import render_stage
import argparse
from collections import deque
from functools import partial
from http.server import SimpleHTTPRequestHandler,ThreadingHTTPServer
import json
from pathlib import Path
import threading
import time

from PIL import Image

from render_reference import SourceArt,layout_blocks,load_pack
from review_headwaters import prepare
from review_league_approach import render as render_route,verify_published
from review_gunflint import flood,neighbors,VECTORS

ROOT=Path(__file__).resolve().parents[3]
LAYOUT=ROOT/'planning/minnesota/routes/MN_R16.layout.json'
DOSSIER=ROOT/'planning/minnesota/routes/MN_R16.json'
OUTPUT=ROOT/'planning/minnesota/renders/red-river-prairie'
HTML='''<!doctype html><meta charset="utf-8"><title>Red River Prairie review</title><style>body{background:#14212b;color:#e7eff2;font:16px system-ui;margin:20px}img{image-rendering:pixelated}header{position:sticky;top:0;background:#14212bf5;padding:12px}button{margin:4px;padding:10px}.scroll{overflow:auto;max-height:70vh}#map{width:2048px}</style><header><h1>Red River Prairie — complete hand-authored route</h1><button data-layer="whole-route">Residents</button><button data-layer="terrain">Terrain</button><button data-layer="anchors">Anchors</button><button data-layer="ledges">Directed bank</button><p id="status">Rendering…</p></header><p>Red River bridge · Hayfield bank · Flood meadow · Drainage footbridge · Seed and pollinator plots · Cottonwood neck. Static source review; regional connections and controller evidence remain separate.</p><div class="scroll"><img id="map"></div><h2>Native 160×144 camera views, enlarged 2×</h2><img id="views"><script>let layer='whole-route',rev='';function refresh(){document.querySelector('#map').src=layer+'.png?r='+rev;document.querySelector('#views').src='viewpoints.png?r='+rev}document.querySelectorAll('[data-layer]').forEach(b=>b.onclick=()=>{layer=b.dataset.layer;refresh()});setInterval(async()=>{const s=await(await fetch('/state',{cache:'no-store'})).json();document.querySelector('#status').textContent=s.ok?s.render_ms+' ms · '+s.status:'INVALID: '+s.error;if(s.revision!==rev){rev=s.revision;refresh()}},400)</script>'''


def prairie_contract(design,layout,pack):
    blocks=layout_blocks(layout);w,h=layout['dimensions_metatiles'];collision=pack['data']['tilesets'][layout['compiled_tileset']]['collision']
    def kind(x,y):return collision[f'{blocks[y//2*w+x//2]:02x}'][y%2*2+x%2]
    occupied={tuple(a['tile']) for k in ['trainers','residents','items'] for a in design[k]}
    walking={(x,y) for y in range(h*2) for x in range(w*2) if kind(x,y) in {'FLOOR','TALL_GRASS','LADDER'} and (x,y) not in occupied}
    start,end=[tuple(v*2 for v in e['gate_metatile'])for e in design['endpoints']]
    for crossing in layout['crossings']:
        assert all(kind(*p)=='FLOOR' for p in crossing['core_walk_line']),crossing['id']+': interrupted walking line'
        assert all([kind(x*2,y*2),kind(x*2+1,y*2),kind(x*2,y*2+1),kind(x*2+1,y*2+1)]==['WALL','WALL','FLOOR','FLOOR']for x,y in crossing['north_rail_blocks']),crossing['id']+': unprotected north rail'
        assert all([kind(x*2,y*2),kind(x*2+1,y*2),kind(x*2,y*2+1),kind(x*2+1,y*2+1)]==['FLOOR','FLOOR','WALL','WALL']for x,y in crossing['south_rail_blocks']),crossing['id']+': unprotected south rail'
        cells={(x*2+dx,y*2+dy)for key in ['north_rail_blocks','south_rail_blocks']for x,y in crossing[key]for dx in [0,1]for dy in [0,1]}
        assert end not in flood(start,walking-cells),crossing['id']+': free walking bypass'
    cuts=[(x*2+q%2,y*2+q//2)for y in range(h)for x in range(w)for q,k in enumerate(collision[f'{blocks[y*w+x]:02x}'])if k=='CUT_TREE']
    assert not cuts,'Incidental Cut tree quadrants: '+str(cuts)
    outer={(x,0)for x in range(w*2)}|{(x,h*2-1)for x in range(w*2)}|{(0,y)for y in range(h*2)}|{(w*2-1,y)for y in range(h*2)}
    assert not[p for p in outer if kind(*p)in{'WATER','WHIRLPOOL','WATERFALL'}],'Unconnected exterior water edge'
    ports={p for p in outer if kind(*p)in{'FLOOR','TALL_GRASS','LADDER'}}
    assert ports=={(0,24),(0,25),(w*2-1,24),(w*2-1,25)},'Unexpected exterior walking opening'
    choice=design['battle_or_grass_choices'][0];trainer=next(a for a in design['trainers']if a['id']==choice['trainer_id']);dx,dy=VECTORS[trainer['facing']]
    hazards={(trainer['tile'][0]+dx*n,trainer['tile'][1]+dy*n)for n in range(1,trainer['sight_range']+1)}
    x0,y0,x1,y1=choice['bounds_core'];band={(x,y)for y in range(y0,y1)for x in range(x0,x1)}
    assert all(kind(*p)=='TALL_GRASS'for p in band),'Battle alternative is not native grass'
    assert end in flood(start,walking-hazards),'Unbeaten trainer leaves no grass alternative'
    assert end not in flood(start,walking-hazards-band),'Unbeaten trainer has a free dry walk-around'
    assert end in flood(start,walking-band),'Defeated trainer does not open the clear apron'
    safe=walking-hazards;costs={start:0};queue=deque([start])
    while queue:
        at=queue.popleft()
        for q in neighbors(at):
            if q not in safe:continue
            cost=costs[at]+int(q in band)
            if cost<costs.get(q,10**9):
                costs[q]=cost;(queue.append if q in band else queue.appendleft)(q)
    assert costs[end]==choice['minimum_grass_steps'],'Battle alternative grass price changed'
    assert all(kind(x,2)=='BUOY'for x in range(12,24)),'River north closure'
    assert all(kind(x,45)=='BUOY'for x in range(14,28)),'River south closure'
    assert all(kind(19,y)=='WALL'for y in range(14,18))and kind(18,14)=='WALL'and kind(18,17)=='WALL','Fishing landing closed tip'
    ledge=design['one_way_ledges'][0]
    x0,y0,x1,y1=ledge['bounds_core']
    assert ledge['direction']=='south'and y1==y0+2,'Native hayfield bank extent'
    assert all(kind(x,y0)=='HOP_DOWN'and kind(x,y0+1)=='WALL'for x in range(x0,x1)),'Native hayfield bank face'
    assert [x for x in range(w*2)if kind(x,y0)=='HOP_DOWN']==list(range(x0,x1)),'Undeclared wider hayfield jump face'
    assert all(kind(*p)=='WALL'for p in ledge['side_barriers']),'Bank side bypass'
    hx,hy=ledge['hop_from']
    assert ledge['upper_approach']==[hx,hy-1]and ledge['reverse_blocked']==[hx,hy+1]and ledge['landing']==[hx,hy+2],'Native hop coordinate ownership'
    assert kind(*ledge['upper_approach'])=='FLOOR'and kind(*ledge['landing'])=='FLOOR','Native hop requires actual clear approach and landing'
    seam=[y for y in range(h*2)if kind(63,y)=='WATER'and kind(64,y)=='WATER']
    maps=pack['data']['maps'];published='MnR16C00R00'in maps
    if published:
        assert'MnR16C01R00'in maps,'Missing eastern neighbor'
        for source,target,direction in [('MnR16C00R00','MnR16C01R00','east'),('MnR16C01R00','MnR16C00R00','west')]:
            assert{'direction':direction,'target_map':target,'offset':0}in maps[source]['attributes']['connections'],'Missing reciprocal internal water neighbor'
    return dict(necessary_native_bridges=2,incidental_cut_quadrants=len(cuts),unconnected_outer_water_cells=0,internal_water_seam_pairs=len(seam),published_reciprocal_water_neighbor_verified=published,source_metatile_varieties=len(set(blocks)),battle_alternative_grass_steps=costs[end],outer_walking_ports=sorted(ports))


def render(design,layout,pack,art):
    prepare(pack,layout)
    extra=prairie_contract(design,layout,pack)
    report=render_route(design,layout,pack,art,OUTPUT)
    report.update(extra)
    # Locate the largest genuinely blank ground square so review does not only
    # pick flattering landmarks. This is analysis of explicit authored blocks.
    blocks=layout_blocks(layout);w,h=layout['dimensions_metatiles']
    occupied={tuple(v//2 for v in a['tile'])for key in ['trainers','residents','items']for a in design[key]}
    sizes={};best=(0,0,0)
    for y in range(h):
        for x in range(w):
            size=1+min(sizes.get((x-1,y),0),sizes.get((x,y-1),0),sizes.get((x-1,y-1),0))if blocks[y*w+x]in{1,2}and(x,y)not in occupied else 0
            sizes[x,y]=size
            if size>best[0]:best=size,x-size+1,y-size+1
    size,x,y=best;cx,cy=x*2+size,y*2+size
    scene=Image.open(OUTPUT/'whole-route.png')
    save_render(render_stage(render_stage(scene, 'crop', OUTPUT, 'review_red_river_prairie-render-line96-16-crop', (cx*16-80,cy*16-72,cx*16+80,cy*16+72)), 'resize', OUTPUT, 'review_red_river_prairie-render-line96-16-resize', (640,576),Image.Resampling.NEAREST), OUTPUT/'viewport-emptiest_ground.png')
    report.update(largest_blank_ground_square_metatiles=size,emptiest_ground_camera_core=[cx,cy])
    (OUTPUT/'revision.json').write_text(json.dumps(report,indent=2)+'\n')
    return report


def main():
    p=argparse.ArgumentParser();p.add_argument('--pack',type=Path,default=ROOT/'content-packs/minnesota-prairie-review.crystalpack');p.add_argument('--serve',action='store_true');p.add_argument('--watch',action='store_true');p.add_argument('--port',type=int,default=8786);p.add_argument('--verify-published',action='store_true');args=p.parse_args()
    pack=load_pack(args.pack);art=SourceArt(pack);state={'ok':False,'error':'Initial render pending'}
    if args.serve:
        OUTPUT.mkdir(parents=True,exist_ok=True);(OUTPUT/'index.html').write_text(HTML)
        class Handler(SimpleHTTPRequestHandler):
            def log_message(self,*args):pass
            def do_GET(self):
                if self.path.split('?')[0]=='/state':
                    body=json.dumps(state).encode();self.send_response(200);self.send_header('Content-Type','application/json');self.send_header('Cache-Control','no-store');self.send_header('Content-Length',str(len(body)));self.end_headers();self.wfile.write(body)
                else:super().do_GET()
        server=ThreadingHTTPServer(('127.0.0.1',args.port),partial(Handler,directory=str(OUTPUT)));threading.Thread(target=server.serve_forever,daemon=True).start();print(f'Red River Prairie: http://127.0.0.1:{args.port}/',flush=True)
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
