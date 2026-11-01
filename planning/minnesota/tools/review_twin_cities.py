#!/usr/bin/env python3
"""Source-pixel review for individually authored Twin Cities terrain/rooms."""
import argparse
from functools import partial
from http.server import SimpleHTTPRequestHandler,ThreadingHTTPServer
import json
from pathlib import Path
import threading
import time

from render_reference import SourceArt,load_pack
from review_league_approach import HTML,render,verify_published

ROOT=Path(__file__).resolve().parents[3]
LAYOUT=ROOT/'planning/minnesota/routes/MN_R04.layout.json'
DOSSIER=ROOT/'planning/minnesota/routes/MN_R04.json'
OUTPUT=ROOT/'planning/minnesota/renders/twin-cities'
PAGE=HTML.replace('League Approach','Twin Cities River Crossing').replace('all eighteen chunks','one compact civic park').replace('Border Lake · Wind Tor · Granite switchback · Dry saddle · Cedar Shelter · Long Lake · Old quarry · Ely arrival.','Rose walk · Battle or grass · Gold-railed bridge · Eastern flower islands · Picnic habitat.').replace('<h2>Cedar Shelter</h2><img id="rooms" src="interiors.png">','').replace('width:1536px','width:896px').replace('(native?3072:1536)','(native?896:448)').replace('native=false','native=true')


def check_piers(design,layout,pack):
    from copy import deepcopy
    from collections import deque
    from render_reference import layout_blocks
    if 'mn_river_park' not in pack['data']['tilesets']:
        definition=deepcopy(pack['data']['tilesets']['park'])
        definition['collision']['40']=[definition['collision']['19'][i] for i in [2,3,0,1]]
        definition['collision']['41']=definition['collision']['08'][:]
        definition['collision']['42']=definition['collision']['09'][:]
        definition['collision']['43']=definition['collision']['19'][:]
        pack['data']['tilesets']['mn_river_park']=definition
    blocks=layout_blocks(layout);w,h=layout['dimensions_metatiles'];c=pack['data']['tilesets']['mn_river_park']['collision']
    def kind(x,y):return c[f'{blocks[y//2*w+x//2]:02x}'][y%2*2+x%2]
    occupied={tuple(a['tile']) for k in ['trainers','residents','items'] for a in design[k]}
    allowed={(x,y) for y in range(h*2) for x in range(w*2) if kind(x,y) in {'FLOOR','TALL_GRASS'} and (x,y) not in occupied}
    def cost(start,end,blocked,band):
        costs={start:0};todo=deque([start])
        while todo:
            x,y=todo.popleft()
            for q in [(x-1,y),(x+1,y),(x,y-1),(x,y+1)]:
                if q not in allowed or q in blocked:continue
                n=costs[(x,y)]+int(q in band)
                if n>=costs.get(q,10**9):continue
                costs[q]=n;(todo.append if q in band else todo.appendleft)(q)
        return costs.get(end)
    west,east=[tuple(v*2 for v in e['gate_metatile']) for e in design['endpoints']]
    choice=design['battle_or_grass_choices'][0];x0,y0,x1,y1=choice['bounds_core'];band={(x,y) for x in range(x0,x1) for y in range(y0,y1)}
    assert all(kind(*p)=='TALL_GRASS' for p in band),'Rose walk grass changed'
    trainer=next(t for t in design['trainers'] if t['id']==choice['trainer_id']);assert trainer['facing']=='west' and trainer['sight_range']==1,'Cole must remain adjacent to the narrow lane'
    sight={(18,18)}
    assert cost(west,east,set(),band)==0,'Paved lane missing'
    for a,b in [(west,east),(east,west)]:
        n=cost(a,b,sight,band);assert n is not None and n>=4,'Battle and habitat can both be freely bypassed'
    bridge=layout['crossings'][0]
    for x,y in bridge['deck_blocks']:assert all(kind(x*2+dx,y*2+dy)=='FLOOR' for dx in [0,1] for dy in [0,1]),'Bridge deck'
    for x,y in bridge['north_rail_blocks']:assert [kind(x*2,y*2),kind(x*2,y*2+1)]==['WALL','FLOOR'],'North rail'
    for x,y in bridge['south_rail_blocks']:assert [kind(x*2,y*2),kind(x*2,y*2+1)]==['FLOOR','WALL'],'South rail'
    for x,y in bridge['water_under_rail_blocks']:assert kind(x*2,y*2)=='WATER_21','Native river water'
    for p in bridge['core_walk_line']:assert kind(*p)=='FLOOR','Bridge approach'
    cells={(x*2+dx,y*2+dy) for k in ['deck_blocks','north_rail_blocks','south_rail_blocks'] for x,y in bridge[k] for dx in [0,1] for dy in [0,1]}
    assert cost(west,east,cells,set()) is None,'River bridge can be bypassed'
    if 'gfx/tilesets/mn_river_park.png' in pack['runtime_files']:
        art=SourceArt(pack);native=art.blocks('mn_river_park');original=art.blocks('park')
        assert len(native)==68 and all(a.tobytes()==b.tobytes() for a,b in zip(native,original)),'Original Park art changed'
        for block in range(0x40,0x44):
            assert art.render('park',1,1,[block],derived=layout['derived_blocks']).tobytes()==native[block].tobytes(),'Derived Park source mismatch'
    return {'source_metatile_varieties_used':len(set(blocks)),'bridge_checked':True,'battle_or_grass_choices_checked':1,'post_battle_paved_lane_checked':True}


def main():
    parser=argparse.ArgumentParser();parser.add_argument('--pack',type=Path,default=ROOT/'content-packs/text-tui.crystalpack');parser.add_argument('--serve',action='store_true');parser.add_argument('--watch',action='store_true');parser.add_argument('--port',type=int,default=8774);parser.add_argument('--verify-published',action='store_true');args=parser.parse_args()
    if args.verify_published and (args.watch or args.serve):parser.error('--verify-published requires a single run')
    pack=load_pack(args.pack);art=SourceArt(pack);state={'value':{'ok':False,'error':'Initial render pending'}}
    if args.serve:
        OUTPUT.mkdir(parents=True,exist_ok=True);(OUTPUT/'index.html').write_text(PAGE)
        class Handler(SimpleHTTPRequestHandler):
            def log_message(self,*args):pass
            def do_GET(self):
                if self.path.split('?')[0]=='/state':
                    body=json.dumps(state['value']).encode();self.send_response(200);self.send_header('Content-Type','application/json');self.send_header('Cache-Control','no-store');self.send_header('Content-Length',str(len(body)));self.end_headers();self.wfile.write(body)
                else:super().do_GET()
        server=ThreadingHTTPServer(('127.0.0.1',args.port),partial(Handler,directory=str(OUTPUT)));threading.Thread(target=server.serve_forever,daemon=True).start();print(f'Twin Cities review: http://127.0.0.1:{args.port}/',flush=True)
    last=None
    while True:
        revision=(LAYOUT.stat().st_mtime_ns,DOSSIER.stat().st_mtime_ns)
        if revision!=last:
            try:
                design,layout=json.loads(DOSSIER.read_text()),json.loads(LAYOUT.read_text());extra=check_piers(design,layout,pack);published=verify_published(design,layout,pack) if args.verify_published else {};report={**render(design,layout,pack,art,output=OUTPUT),**extra,**published};state['value']={'ok':not report['errors'],'revision':str(time.time_ns()),'error':'; '.join(report['errors']),**report};print(json.dumps(report),flush=True)
                if report['errors'] and not(args.watch or args.serve):raise SystemExit(1)
            except (AssertionError,KeyError,ValueError,IndexError,TypeError) as error:
                state['value']={'ok':False,'error':str(error)};print('DESIGN ERROR: '+str(error),flush=True)
                if not(args.watch or args.serve):raise SystemExit(1)
            last=revision
        if not(args.watch or args.serve):break
        time.sleep(.2)


if __name__=='__main__':main()
