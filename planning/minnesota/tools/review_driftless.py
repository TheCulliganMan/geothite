#!/usr/bin/env python3
"""Source-pixel review for individually authored Driftless terrain/rooms."""
import argparse
from copy import deepcopy
from functools import partial
from http.server import SimpleHTTPRequestHandler,ThreadingHTTPServer
import json
from pathlib import Path
import threading
import time

from render_reference import SourceArt,load_pack
from review_league_approach import HTML,render,verify_published

ROOT=Path(__file__).resolve().parents[3]
LAYOUT=ROOT/'planning/minnesota/routes/MN_R01.layout.json'
DOSSIER=ROOT/'planning/minnesota/routes/MN_R01.json'
OUTPUT=ROOT/'planning/minnesota/renders/driftless'
PAGE=HTML.replace('League Approach','Driftless Trail').replace('all eighteen chunks','single compact map').replace('Border Lake · Wind Tor · Granite switchback · Dry saddle · Cedar Shelter · Long Lake · Old quarry · Ely arrival.','Winona grass gate · Nelson farm · Low limestone bench · One-way descent · Two small hollows · Creek dock.').replace('Cedar Shelter','Limestone hollow and Nelson farm').replace('width:1536px','width:1024px').replace('(native?3072:1536)','(native?1024:512)').replace('native=false','native=true').replace('Driftless Trail — three clipped maps','Driftless Trail — complete starter route').replace('Border Lake · Wind Tor ramp and drop · Grass Saddle · Cedar Shelter · Long Lake high bank · Dry Pass · Cut refuge · Ely arrival.','Winona grass gate · Nelson farm · Low limestone bench · One-way descent · Two small hollows · Closed creek dock.')

PAGE=PAGE.replace('#map{width:512px}', '#map{width:1024px}').replace('Border Lake · Wind Tor ramp and drop · Grass Saddle · Limestone hollow and Nelson farm · Long Lake high bank · Dry Pass · Cut refuge · Ely arrival.', 'Winona grass gate · Nelson farm · Limestone bench · One-way descent · Two hollows · Closed creek dock.')


def check_piers(design,layout,pack):
    from render_reference import layout_blocks
    blocks=layout_blocks(layout);w=layout['dimensions_metatiles'][0];c=pack['data']['tilesets']['johto']['collision']
    def kind(x,y):return c[f'{blocks[y//2*w+x//2]:02x}'][y%2*2+x%2]
    dock=layout['crossings'][0]
    for x,y in dock['deck_blocks']:assert all(kind(x*2+dx,y*2+dy)=='FLOOR' for dx in [0,1] for dy in [0,1]),'Dock deck'
    for x,y in dock['north_rail_blocks']:assert [kind(x*2,y*2),kind(x*2,y*2+1)]==['WALL','FLOOR'],'Dock north rail'
    for x,y in dock['south_rail_blocks']:assert [kind(x*2,y*2),kind(x*2,y*2+1)]==['FLOOR','WALL'],'Dock south rail'
    for x,y in dock['core_walk_line']:assert kind(x,y)=='FLOOR','Dock entry'
    for y in range(32,38):
        for x in range(10,20):
            if kind(x,y)=='FLOOR':
                for dx,dy in [(0,1),(0,-1),(1,0),(-1,0)]:
                    assert kind(x+dx,y+dy)!='WATER','Dock corner or rail leaks directly to water'
    for ledge in design['one_way_ledges']:
        x0,y0,x1,y1=ledge['bounds_core']
        assert all(kind(x,y0)=='HOP_DOWN' and kind(x,y0+1)=='WALL' for x in range(x0,x1)),'Native descent ledge'
        assert kind(*ledge['upper_approach'])==kind(*ledge['landing'])=='FLOOR','Ledge landings'
    assert [kind(40,18),kind(41,18),kind(40,19),kind(41,19)]==['WALL','FLOOR','FLOOR','FLOOR'],'Native uphill ramp'
    occupied={tuple(a['tile']) for k in ['trainers','residents','items'] for a in design[k]}
    height=layout['dimensions_metatiles'][1]*2
    def reach(start,forbid):
        seen={start};todo=[start]
        while todo:
            x,y=todo.pop()
            for dx,dy in [(1,0),(-1,0),(0,1),(0,-1)]:
                q=(x+dx,y+dy)
                if kind(x,y)=='HOP_DOWN' and dy==1:q=(x,y+2)
                if not(0<=q[0]<w*2 and 0<=q[1]<height):continue
                if q in occupied or q in forbid or q in seen:continue
                if kind(*q) in {'FLOOR','TALL_GRASS','HOP_DOWN'}:seen.add(q);todo.append(q)
        return seen
    # This is a directional alternative to the central grass, rather than a
    # decorative ledge in an otherwise bidirectional loop. The shared entrance
    # grass remains mandatory in either direction and is checked separately.
    meadow={(x,y) for x in range(30,36) for y in range(22,28)}
    assert (4,24) in reach((52,24),meadow),'Westbound bluff alternative disconnected'
    assert (52,24) not in reach((4,24),meadow),'Eastbound route can climb around the ledge'
    ramp={(41,18),(40,19),(41,19)}
    assert (29,15) not in reach((29,18),ramp),'Bluff has an unintended uphill opening'
    assert (29,15) in reach((29,18),set()),'Bluff uphill ramp disconnected'
    from review_gunflint import flood
    perimeter={(x,y) for y in range(height) for x in range(w*2) if x in {0,w*2-1} or y in {0,height-1}}
    assert len(perimeter)==204 and not any(kind(*at)=='WATER' for at in perimeter),'Unconnected outer creek edge'
    assert all(kind(x,39)=='BUOY' for x in range(2,20)),'South creek must close with native blue-water rocks'
    assert not any(kind(x,y)=='CUT_TREE' for y in range(height) for x in range(w*2)),'Decorative Cut canopy remains'
    water={(x,y) for y in range(height) for x in range(w*2) if kind(x,y)=='WATER'}
    assert water,'Actual creek surface required'
    # Seed the actual water as well as the land: a fenced dock need not provide
    # a Surf launch, but a late-game surfacing fixture must have a closed lake.
    conservative={(x,y) for y in range(height) for x in range(w*2) if kind(x,y) in {'FLOOR','LADDER','TALL_GRASS','WATER'} and (x,y) not in occupied}
    water_component=flood(next(iter(water)),conservative)
    assert not water_component & perimeter & water,'Surfing creek component reaches an outer water edge'
    return {'complete_outer_perimeter_core_tiles':len(perimeter),'outer_water_tiles':0,'south_creek_buoy_tiles':18,'creek_water_tiles':len(water),'native_cut_quadrants':0,'source_metatile_varieties_used':len(set(blocks)),'native_ramp_checked':True,'one_way_ledges_checked':1,'directional_grass_choice_checked':True,'dock_checked':True}


def self_test(design,layout,pack):
    """Reject boundary repairs that break the original directional route."""
    cases=[]
    def change(name,position,block):
        def mutate(d,l):l['layers'].append(dict(kind='point',name=name,position=position,block=block))
        cases.append((name,mutate))
    change('open south creek edge',[3,19],'35')
    change('brown land boulder replaces water rock',[3,19],'0a')
    change('decorative Cut toe',[14,10],'63')
    change('native ramp removed',[20,9],'72')
    change('native ledge flattened',[14,8],'01')
    change('dock south rail removed',[7,18],'01')
    change('dock north rail removed',[7,16],'01')
    change('dock walking entry blocked',[10,17],'05')
    change('north-west dock cap removed',[5,16],'35')
    change('south-west dock cap removed',[5,18],'35')
    check_piers(design,layout,pack)
    rejected=[]
    for name,mutate in cases:
        d,l=deepcopy(design),deepcopy(layout);mutate(d,l)
        try:check_piers(d,l,pack)
        except AssertionError:rejected.append(name)
        else:raise AssertionError('Negative fixture accepted: '+name)
    return dict(negative_cases_rejected=len(rejected),cases=rejected)


def main():
    parser=argparse.ArgumentParser();parser.add_argument('--pack',type=Path,default=ROOT/'content-packs/text-tui.crystalpack');parser.add_argument('--serve',action='store_true');parser.add_argument('--watch',action='store_true');parser.add_argument('--port',type=int,default=8793);parser.add_argument('--verify-published',action='store_true');parser.add_argument('--self-test',action='store_true');args=parser.parse_args()
    if args.verify_published and (args.watch or args.serve):parser.error('--verify-published requires a single run')
    pack=load_pack(args.pack);art=SourceArt(pack);state={'value':{'ok':False,'error':'Initial render pending'}}
    if args.self_test:
        print(json.dumps(self_test(json.loads(DOSSIER.read_text()),json.loads(LAYOUT.read_text()),pack)));return
    if args.serve:
        OUTPUT.mkdir(parents=True,exist_ok=True);(OUTPUT/'index.html').write_text(PAGE)
        class Handler(SimpleHTTPRequestHandler):
            def log_message(self,*args):pass
            def do_GET(self):
                if self.path.split('?')[0]=='/state':
                    body=json.dumps(state['value']).encode();self.send_response(200);self.send_header('Content-Type','application/json');self.send_header('Cache-Control','no-store');self.send_header('Content-Length',str(len(body)));self.end_headers();self.wfile.write(body)
                else:super().do_GET()
        server=ThreadingHTTPServer(('127.0.0.1',args.port),partial(Handler,directory=str(OUTPUT)));threading.Thread(target=server.serve_forever,daemon=True).start();print(f'Driftless review: http://127.0.0.1:{args.port}/',flush=True)
    last=None
    while True:
        revision=(LAYOUT.stat().st_mtime_ns,DOSSIER.stat().st_mtime_ns)
        if revision!=last:
            try:
                design,layout=json.loads(DOSSIER.read_text()),json.loads(LAYOUT.read_text());extra=check_piers(design,layout,pack);published=verify_published(design,layout,pack) if args.verify_published else {};report={**render(design,layout,pack,art,output=OUTPUT),**extra,**published};(OUTPUT/'revision.json').write_text(json.dumps(report,indent=2)+'\n');state['value']={'ok':not report['errors'],'revision':str(time.time_ns()),'error':'; '.join(report['errors']),**report};print(json.dumps(report),flush=True)
                if report['errors'] and not(args.watch or args.serve):raise SystemExit(1)
            except (AssertionError,KeyError,ValueError,IndexError,TypeError) as error:
                state['value']={'ok':False,'error':str(error)};print('DESIGN ERROR: '+str(error),flush=True)
                if not(args.watch or args.serve):raise SystemExit(1)
            last=revision
        if not(args.watch or args.serve):break
        time.sleep(.2)


if __name__=='__main__':main()
