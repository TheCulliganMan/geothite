#!/usr/bin/env python3
"""Source-pixel review for individually authored River Bluffs terrain/rooms."""
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
LAYOUT=ROOT/'planning/minnesota/routes/MN_R05.layout.json'
DOSSIER=ROOT/'planning/minnesota/routes/MN_R05.json'
OUTPUT=ROOT/'planning/minnesota/renders/river-bluffs'
PAGE=HTML.replace('League Approach','River Bluffs North').replace('all eighteen chunks','three ridge maps').replace('Border Lake · Wind Tor · Granite switchback · Dry saddle · Cedar Shelter · Long Lake · Old quarry · Ely arrival.','City trailhead · Lower switchback · Dry rock cut · Upper ridge · Two return ledges · River pier.').replace('Cedar Shelter','Dry Rock Cut').replace('width:1536px','width:1024px').replace('(native?3072:1536)','(native?1024:512)').replace('native=false','native=true')


def check_piers(design,layout,pack):
    from render_reference import layout_blocks
    from review_headwaters import prepare
    prepare(pack,layout)
    blocks=layout_blocks(layout);w,h=layout['dimensions_metatiles'];c=pack['data']['tilesets']['mn_headwaters']['collision']
    def kind(x,y):return c[f'{blocks[y//2*w+x//2]:02x}'][y%2*2+x%2]
    occupied={tuple(a['tile']) for k in ['trainers','residents','items'] for a in design[k]}
    def reach(start,forbid):
        seen={start};todo=[start]
        while todo:
            x,y=todo.pop()
            for dx,dy in [(1,0),(-1,0),(0,1),(0,-1)]:
                q=(x+dx,y+dy)
                if kind(x,y)=='HOP_DOWN' and dy==1:q=(x,y+2)
                if not(0<=q[0]<w*2 and 0<=q[1]<h*2):continue
                if q in occupied or q in forbid or q in seen:continue
                if kind(*q) in {'FLOOR','TALL_GRASS','HOP_DOWN'}:seen.add(q);todo.append(q)
        return seen
    for pier in layout['crossings']:
        for x,y in pier['deck_blocks']:assert all(kind(x*2+dx,y*2+dy)=='FLOOR' for dx in [0,1] for dy in [0,1]),'Pier deck'
        for x,y in pier['north_rail_blocks']:assert [kind(x*2,y*2),kind(x*2,y*2+1)]==['WALL','FLOOR'],'North rail'
        for x,y in pier['south_rail_blocks']:assert [kind(x*2,y*2),kind(x*2,y*2+1)]==['FLOOR','WALL'],'South rail'
        for x,y in pier['water_under_rail_blocks']:assert kind(x*2,y*2)=='WATER','River below pier'
        for p in pier['core_walk_line']:assert kind(*p)=='FLOOR','Pier approach'
    assert kind(61,89)=='WALL','Pier end must close at the bollard'
    assert all(kind(*p)=='WALL' for p in [(51,16),(51,23),(55,21)]),'Lookout fences must protect all three river-facing edges'
    assert all(kind(*p)=='FLOOR' for p in [(51,17),(51,22),(54,21)]),'Lookout inner rail strips must remain walkable'
    for ledge in design['one_way_ledges']:
        x0,y0,x1,y1=ledge['bounds_core']
        assert all(kind(x,y0)=='HOP_DOWN' and kind(x,y0+1)=='WALL' for x in range(x0,x1)),'Native drop face'
        assert all(kind(*at)=='WALL' for at in ledge['side_barriers']),'Drop side opening'
    assert len(design['doors'])==2 and len(design['interiors'])==1,'Two-mouth rock cut'
    assert design['interiors'][0]['exit_doors']==[d['tile'] for d in design['doors']],'Mouth/ladder pairing'
    for door in design['doors']:assert kind(*door['tile'])=='CAVE' and kind(*door['return_tile'])=='FLOOR','Rock-cut footing'
    perimeter={(x,y) for y in range(h*2) for x in range(w*2) if x in {0,w*2-1} or y in {0,h*2-1}}
    assert not any(kind(*p)=='WATER' for p in perimeter),'River opens onto an unconnected outside edge'
    for barrier in layout['outer_water_barriers']:
        for x,y,block in barrier['blocks']:
            assert block in layout['additional_water_blocks'] and blocks[y*w+x]==int(block,16),'Boundary rock differs from authored native recipe'
            outer=[(x*2+1,y*2),(x*2+1,y*2+1)] if barrier['side']=='east' else [(x*2,y*2+int(barrier['side']=='south')),(x*2+1,y*2+int(barrier['side']=='south'))]
            assert all(p in perimeter and kind(*p)=='BUOY' for p in outer),'Water-rock outer quadrants must block Surf'
    from review_gunflint import flood
    allowed={(x,y) for y in range(h*2) for x in range(w*2) if kind(x,y) in {'FLOOR','TALL_GRASS','WATER'} and (x,y) not in occupied}
    seen=flood(tuple(v*2 for v in design['endpoints'][0]['gate_metatile']),allowed)
    water={p for p in seen if kind(*p)=='WATER'}
    assert water and not water&perimeter,'Reachable Surf water must be enclosed'
    seams=[];native=True
    for seam in layout['internal_water_connections']:
        y=seam['core_boundary_y'];xs=[x for x in range(w*2) if kind(x,y-1)==kind(x,y)=='WATER']
        assert xs,'Actual neighboring river maps must retain open water'
        available=all(name in pack['data']['maps'] for name in [seam['from_map'],seam['target_map']]);native&=available
        if available:
            for source,target,direction in [(seam['from_map'],seam['target_map'],'south'),(seam['target_map'],seam['from_map'],'north')]:
                assert dict(direction=direction,target_map=target,offset=0) in pack['data']['maps'][source]['attributes']['connections'],'Open river seam lacks a reciprocal native neighbor'
        seams.append(dict(core_boundary_y=y,open_water_core_x=xs))
    return {'source_metatile_varieties_used':len(set(blocks)),'native_drop_faces_checked':2,'pier_checked':True,'paired_cave_mouths_checked':2,'outer_perimeter_core_tiles_checked':len(perimeter),'reachable_surf_water_tiles':len(water),'reachable_outer_water_tiles':0,'river_rock_boundary_core_tiles':sum(kind(*p)=='BUOY' for p in perimeter),'internal_water_seams':seams,'published_water_neighbors_verified':native}


def main():
    parser=argparse.ArgumentParser();parser.add_argument('--pack',type=Path,default=ROOT/'content-packs/text-tui.crystalpack');parser.add_argument('--serve',action='store_true');parser.add_argument('--watch',action='store_true');parser.add_argument('--port',type=int,default=8775);parser.add_argument('--verify-published',action='store_true');args=parser.parse_args()
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
        server=ThreadingHTTPServer(('127.0.0.1',args.port),partial(Handler,directory=str(OUTPUT)));threading.Thread(target=server.serve_forever,daemon=True).start();print(f'River Bluffs review: http://127.0.0.1:{args.port}/',flush=True)
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
