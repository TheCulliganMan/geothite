#!/usr/bin/env python3
"""Exact source-art review of the hand-authored Headwaters terrain and events."""
from render_archive import save_render
from render_stages import render_stage
import argparse
from copy import deepcopy
from functools import partial
import hashlib
from http.server import SimpleHTTPRequestHandler, ThreadingHTTPServer
import json
from pathlib import Path
import threading
import time

from PIL import Image,ImageDraw

from render_reference import SourceArt,layout_blocks,load_pack,room_sheet
from review_grand_marais import sprite
from review_north_shore import oriented_sprite
from review_league_approach import check,verify_published
from review_gunflint import flood

ROOT=Path(__file__).resolve().parents[3]
LAYOUT=ROOT/'planning/minnesota/routes/MN_R14.layout.json'
DOSSIER=ROOT/'planning/minnesota/routes/MN_R14.json'
OUTPUT=ROOT/'planning/minnesota/renders/headwaters'


def prepare(pack,layout):
    """Read-only preview definitions; this never exports a pack or source catalog."""
    if 'mn_headwaters' in pack['data']['tilesets']:return
    definition=deepcopy(pack['data']['tilesets']['johto'])
    for entry in layout['derived_blocks']:
        source=entry.get('collision_source_tileset',entry['source_tileset'])
        block=entry.get('collision_source_block',entry['source_block'])
        definition['collision'][entry['id']]=list(pack['data']['tilesets'][source]['collision'][block])
    pack['data']['tilesets']['mn_headwaters']=definition


def render(design,layout,pack,art):
    started=time.perf_counter();prepare(pack,layout);blocks=layout_blocks(layout);w,h=layout['dimensions_metatiles']
    report,kind,seen=check(design,layout,pack,blocks)
    # The crossing must be an actual bridge, not a decorative strip beside an
    # accidental dry dam. Removing its walking cells separates the two gates.
    bridge=layout['crossings'][0]; bridge_cells={(x*2+dx,y*2+dy) for name in ['deck_blocks','north_rail_blocks','south_rail_blocks'] for x,y in bridge[name] for dx in [0,1] for dy in [0,1]}
    start,end=[tuple(v*2 for v in e['gate_metatile']) for e in design['endpoints']]
    if end in flood(start,seen-bridge_cells):report['errors'].append('The boardwalk can be bypassed across a dry river dam')
    for x,y in bridge['deck_blocks']:
        if not all((x*2+dx,y*2+dy) in seen for dx in [0,1] for dy in [0,1]):report['errors'].append('Blocked boardwalk deck')
    if not all(kind(x*2,y*2)=='WALL' and kind(x*2,y*2+1)=='FLOOR' for x,y in bridge['north_rail_blocks']):report['errors'].append('North rail recipe')
    if not all(kind(x*2,y*2)=='FLOOR' and kind(x*2,y*2+1)=='WALL' for x,y in bridge['south_rail_blocks']):report['errors'].append('South rail recipe')
    if not all(kind(x*2,y*2)=='WATER' for x,y in bridge['water_under_rail_blocks']):report['errors'].append('Water beneath pier rail')
    terrain=art.render('johto',w,h,blocks,derived=layout['derived_blocks'])
    if 'gfx/tilesets/mn_headwaters.png' in pack['runtime_files']:
        published=art.blocks('mn_headwaters');original=art.blocks('johto');assert len(published)==131
        assert all(a.tobytes()==b.tobytes() for a,b in zip(published,original)),'original Johto art changed'
        for entry in layout['derived_blocks']:
            sample=art.render('johto',1,1,[int(entry['id'],16)],derived=layout['derived_blocks'])
            assert sample.tobytes()==published[int(entry['id'],16)].tobytes(),'native pier source pixels differ'
    scene=render_stage(terrain, 'convert', OUTPUT, 'review_headwaters-render-line59-10-convert', 'RGBA')
    for a in design['trainers']+design['residents']:scene.alpha_composite(oriented_sprite(pack,a['sprite'],a['facing']),(a['tile'][0]*16,a['tile'][1]*16-4))
    for a in design['items']:scene.alpha_composite(sprite(pack,'SPRITE_POKE_BALL'),(a['tile'][0]*16,a['tile'][1]*16-4))
    OUTPUT.mkdir(exist_ok=True,parents=True);save_render(terrain, OUTPUT/'terrain.png');save_render(render_stage(scene, 'convert', OUTPUT, 'review_headwaters-render-line62-100-convert', 'RGB'), OUTPUT/'whole-route.png')
    digest=hashlib.sha256(scene.tobytes()).hexdigest()[:8];save_render(render_stage(scene, 'convert', OUTPUT, 'review_headwaters-render-line63-71-convert', 'RGB'), OUTPUT/f'whole-route-{digest}.png')
    save_render(render_stage(render_stage(scene, 'resize', OUTPUT, 'review_headwaters-render-line64-16-resize', (1536,1152),Image.Resampling.NEAREST), 'convert', OUTPUT, 'review_headwaters-render-line64-16-convert', 'RGB'), OUTPUT/'overview.png')
    overlay=scene.copy();draw=ImageDraw.Draw(overlay,'RGBA')
    for row in range(3):
        for col in range(3):
            x,y=col*1024,row*768;draw.rectangle((x,y,x+1023,y+767),outline=(255,255,255,160),width=1);draw.text((x+8,y+8),f'MnR14C{col:02}R{row:02}',fill='white',stroke_width=1,stroke_fill='black')
    for a in design['trainers']+design['residents']+design['items']+design['signs']+design['viewpoints']:
        x,y=a['tile'];draw.rectangle((x*16,y*16,x*16+15,y*16+15),outline=(255,210,80,255),width=2);draw.text((x*16,y*16-12),a['id'].removeprefix('MN_R14_'),fill='white',stroke_width=1,stroke_fill='black')
    save_render(render_stage(overlay, 'convert', OUTPUT, 'review_headwaters-render-line71-16-convert', 'RGB'), OUTPUT/'anchors.png')
    views=[]
    for v in design['viewpoints']:
        x,y=v['tile'];im=render_stage(render_stage(scene, 'crop', OUTPUT, 'review_headwaters-render-line74-25-crop', (x*16-80,y*16-72,x*16+80,y*16+72)), 'convert', OUTPUT, 'review_headwaters-render-line74-25-convert', 'RGB');views.append((v['id'],im));save_render(render_stage(im, 'resize', OUTPUT, 'review_headwaters-render-line74-125-resize', (640,576),Image.Resampling.NEAREST), OUTPUT/('viewport-'+v['id'].lower()+'.png'))
    save_render(room_sheet(views,columns=3), OUTPUT/'viewpoints.png')
    report.update(status='INVALID — see errors' if report['errors'] else 'Source pixels and static collision checked; native controller evidence is separate',render_ms=round((time.perf_counter()-started)*1000,1),image=f'whole-route-{digest}.png',source_metatiles_checked=131)
    (OUTPUT/'revision.json').write_text(json.dumps(report,indent=2)+'\n');return report


HTML='''<!doctype html><meta charset="utf-8"><title>Headwaters review</title><style>body{background:#14212b;color:#e7eff2;font:16px system-ui;margin:20px}img{image-rendering:pixelated}header{position:sticky;top:0;background:#14212bf5;padding:12px}button{padding:10px;margin:4px}.scroll{overflow:auto;max-height:75vh}#map{width:1536px}p{max-width:1000px}</style><header><h1>Headwaters Trail — all nine chunks</h1><button data-layer="whole-route">Residents</button><button data-layer="terrain">Terrain</button><button data-layer="anchors">Anchors/chunks</button><button id="zoom">Half/native pixels</button><p id="status">Rendering…</p></header><p>Survey Pond · Young Mississippi boardwalk · Mature pines · Fern hollow · Stream bend · Marsh outlook. Explicit hand-authored terrain and native source pixels. Static review; connected regional progression remains separate.</p><div class="scroll"><img id="map"></div><h2>Native 160×144 framing, enlarged 2×</h2><img id="views"><script>let layer='whole-route',rev='',native=false;const map=document.querySelector('#map');function refresh(){map.src=layer+'.png?r='+rev;document.querySelector('#views').src='viewpoints.png?r='+rev}document.querySelectorAll('[data-layer]').forEach(b=>b.onclick=()=>{layer=b.dataset.layer;refresh()});document.querySelector('#zoom').onclick=()=>{native=!native;map.style.width=(native?3072:1536)+'px'};setInterval(async()=>{const s=await(await fetch('/state',{cache:'no-store'})).json();document.querySelector('#status').textContent=s.ok?s.render_ms+' ms · '+s.clear_reachable_tiles+' clear tiles · '+s.status:'INVALID: '+s.error;if(s.revision!==rev){rev=s.revision;refresh()}},400)</script>'''


def main():
    parser=argparse.ArgumentParser();parser.add_argument('--pack',type=Path,default=ROOT/'content-packs/text-tui.crystalpack');parser.add_argument('--serve',action='store_true');parser.add_argument('--watch',action='store_true');parser.add_argument('--port',type=int,default=8769);parser.add_argument('--verify-published',action='store_true');args=parser.parse_args()
    if args.verify_published and (args.watch or args.serve):parser.error('--verify-published requires a single run')
    pack=load_pack(args.pack);art=SourceArt(pack);state={'value':{'ok':False,'error':'Initial render pending'}}
    if args.serve:
        OUTPUT.mkdir(parents=True,exist_ok=True);(OUTPUT/'index.html').write_text(HTML)
        class Handler(SimpleHTTPRequestHandler):
            def log_message(self,*args):pass
            def do_GET(self):
                if self.path.split('?')[0]=='/state':
                    body=json.dumps(state['value']).encode();self.send_response(200);self.send_header('Content-Type','application/json');self.send_header('Cache-Control','no-store');self.send_header('Content-Length',str(len(body)));self.end_headers();self.wfile.write(body)
                else:super().do_GET()
        server=ThreadingHTTPServer(('127.0.0.1',args.port),partial(Handler,directory=str(OUTPUT)));threading.Thread(target=server.serve_forever,daemon=True).start();print(f'Headwaters review: http://127.0.0.1:{args.port}/',flush=True)
    last=None
    while True:
        revision=(LAYOUT.stat().st_mtime_ns,DOSSIER.stat().st_mtime_ns)
        if revision!=last:
            try:
                design,layout=json.loads(DOSSIER.read_text()),json.loads(LAYOUT.read_text());published=verify_published(design,layout,pack) if args.verify_published else {};report={**render(design,layout,pack,art),**published};state['value']={'ok':not report['errors'],'revision':str(time.time_ns()),'error':'; '.join(report['errors']),**report};print(json.dumps(report),flush=True)
                if report['errors'] and not(args.watch or args.serve):raise SystemExit(1)
            except (AssertionError,KeyError,ValueError,IndexError,TypeError) as error:
                state['value']={'ok':False,'error':str(error)};print('DESIGN ERROR: '+str(error),flush=True)
                if not(args.watch or args.serve):raise SystemExit(1)
            last=revision
        if not(args.watch or args.serve):break
        time.sleep(.2)


if __name__=='__main__':main()
