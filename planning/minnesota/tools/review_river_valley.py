#!/usr/bin/env python3
"""Source-pixel review for individually authored River Valley terrain/rooms."""
from render_archive import save_render
from render_stages import render_stage
import argparse
from copy import deepcopy
from functools import partial
from http.server import SimpleHTTPRequestHandler,ThreadingHTTPServer
import json
from pathlib import Path
import threading
import time

from PIL import Image
from render_reference import SourceArt,load_pack,room_sheet
from review_league_approach import HTML,render,verify_published

ROOT=Path(__file__).resolve().parents[3]
LAYOUT=ROOT/'planning/minnesota/routes/MN_R03.layout.json'
DOSSIER=ROOT/'planning/minnesota/routes/MN_R03.json'
OUTPUT=ROOT/'planning/minnesota/renders/river-valley'
PAGE=HTML.replace('League Approach','Minnesota River Valley').replace('all eighteen chunks','two compact maps').replace('Border Lake · Wind Tor · Granite switchback · Dry saddle · Cedar Shelter · Long Lake · Old quarry · Ely arrival.','Mankato bend · Lower timber bridge · Habitat shelf · Low terrace · Upper crossing · River Pocket Cave · Bird meadow.').replace('Cedar Shelter','River Pocket Cave').replace('width:1536px','width:768px').replace('(native?3072:1536)','(native?768:384)').replace('native=false','native=true').replace('Minnesota River Valley — three clipped maps','Minnesota River Valley — two connected maps').replace('Border Lake · Wind Tor ramp and drop · Grass Saddle · River Pocket Cave · Long Lake high bank · Dry Pass · Cut refuge · Ely arrival.','Mankato bend · Lower timber bridge · Habitat shelf · Low terrace · Upper crossing · River Pocket Cave · Bird meadow.')

PAGE=PAGE.replace('#map{width:512px}', '#map{width:768px}').replace('(native?1024:512)', '(native?768:384)')


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
    start,end=[tuple(v*2 for v in e['gate_metatile']) for e in design['endpoints']]
    assert end in reach(start,set()) and start in reach(end,set()),'River road disconnected'
    for bridge in layout['crossings']:
        for x,y in bridge['deck_blocks']:assert all(kind(x*2+dx,y*2+dy)=='FLOOR' for dx in [0,1] for dy in [0,1]),'Bridge deck'
        for x,y in bridge['north_rail_blocks']:assert [kind(x*2,y*2),kind(x*2,y*2+1)]==['WALL','FLOOR'],'Timber north rail'
        for x,y in bridge['south_rail_blocks']:assert [kind(x*2,y*2),kind(x*2,y*2+1)]==['FLOOR','WALL'],'Timber south rail'
        for x,y in bridge['water_under_rail_blocks']:assert kind(x*2,y*2)=='WATER','Water beneath rail'
        for x,y in bridge['core_walk_line']:assert kind(x,y)=='FLOOR','Crossing approaches'
        cells={(x*2+dx,y*2+dy) for name in ['deck_blocks','north_rail_blocks','south_rail_blocks'] for x,y in bridge[name] for dx in [0,1] for dy in [0,1]}
        assert end not in reach(start,cells),bridge['id']+': dry bypass'
    ledge=design['one_way_ledges'][0]
    x0,y0,x1,y1=ledge['bounds_core']
    assert all(kind(x,y0)=='HOP_DOWN' and kind(x,y0+1)=='WALL' for x in range(x0,x1)),'Native terrace ledge'
    assert kind(*ledge['upper_approach'])==kind(*ledge['landing'])=='FLOOR','Terrace landings'
    assert all(kind(*p)=='WALL' for p in ledge['side_barriers']),'Terrace side closure'
    assert kind(41,40)=='FLOOR' and kind(41,41)=='FLOOR','Native terrace ramp'
    assert tuple(ledge['upper_approach']) not in reach(tuple(ledge['landing']),{(41,40),(40,41),(41,41)}),'Unintended terrace ascent'
    band={(x,y) for x in range(26,34) for y in range(34,40)}
    assert end not in reach(start,band),'One-way route bypasses required habitat'
    if 'gfx/tilesets/mn_headwaters.png' in pack['runtime_files']:
        art=SourceArt(pack);native=art.blocks('mn_headwaters');original=art.blocks('johto')
        assert len(native)==131 and all(a.tobytes()==b.tobytes() for a,b in zip(native,original)),'Original Johto pixels changed'
        for entry in layout['derived_blocks']:
            assert art.render('johto',1,1,[int(entry['id'],16)],derived=layout['derived_blocks']).tobytes()==native[int(entry['id'],16)].tobytes(),'Native bridge pixels differ'
    from review_gunflint import flood
    perimeter={(x,y) for y in range(h*2) for x in range(w*2) if x in {0,w*2-1} or y in {0,h*2-1}}
    assert len(perimeter)==220 and not any(kind(*at)=='WATER' for at in perimeter),'Open unconnected river edge'
    assert all(kind(x,0)=='BUOY' for x in range(30,38)) and all(kind(x,63)=='BUOY' for x in range(28,36)),'Native blue-water river boundary required'
    assert not any(kind(x,y)=='CUT_TREE' for y in range(h*2) for x in range(w*2)),'Decorative Cut canopy remains'
    for bridge in layout['crossings']:
        xs=[x*2 for x,y in bridge['deck_blocks']];ys=[y*2 for x,y in bridge['deck_blocks']]
        for y in range(min(ys)-2,max(ys)+4):
            for x in range(min(xs)-2,max(xs)+4):
                if kind(x,y)=='FLOOR':
                    assert all(kind(x+dx,y+dy)!='WATER' for dx,dy in [(0,1),(0,-1),(1,0),(-1,0)]),'Bridge corner/rail opens to water'
    allowed={(x,y) for y in range(h*2) for x in range(w*2) if kind(x,y) in {'FLOOR','LADDER','TALL_GRASS','WATER'} and (x,y) not in occupied}
    surfing=flood(start,allowed);water={at for at in surfing if kind(*at)=='WATER'}
    assert water and not water & perimeter,'Reachable Surf must stay within the river boundary'
    for v in design['surf_viewpoints']:
        at=tuple(v['tile']);assert kind(*at)=='WATER','Surf boundary camera must be actual water'
        component=flood(at,allowed);assert not {p for p in component & perimeter if kind(*p)=='WATER'},'Fixture-seeded water escapes boundary'
    # The existing river seam is open only to actual reciprocal source chunks.
    seam_x=[x for x in range(w*2) if kind(x,47)==kind(x,48)=='WATER']
    assert seam_x,'River seam was closed without cause'
    native=all(name in pack['data']['maps'] for name in ['MnR03C00R00','MnR03C00R01'])
    if native:
        for a,b,direction in [('MnR03C00R00','MnR03C00R01','south'),('MnR03C00R01','MnR03C00R00','north')]:
            assert dict(direction=direction,target_map=b,offset=0) in pack['data']['maps'][a]['attributes']['connections'],'River seam lacks real reciprocal neighbor'
    return {'complete_outer_perimeter_core_tiles':len(perimeter),'outer_water_tiles':0,'boundary_buoy_tiles':16,'reachable_surf_water_tiles':len(water),'native_cut_quadrants':0,'open_internal_water_seam_core_x':seam_x,'supplied_pack_has_reciprocal_route_neighbors':native,'source_metatile_varieties_used':len(set(blocks)),'bridges_checked':2,'one_way_ledges_checked':1,'bidirectional_main_road_checked':True}


def self_test(design,layout,pack):
    """Reject bounded river repairs that break the proven route topology."""
    cases=[]
    def change(name,position,block):
        def mutate(d,l):l['layers'].append(dict(kind='point',name=name,position=position,block=block))
        cases.append((name,mutate))
    change('north river escape',[16,0],'35')
    change('south river escape',[15,31],'35')
    change('brown land rock in water boundary',[16,0],'0a')
    change('decorative Cut toe',[11,6],'63')
    change('terrace ramp removed',[20,20],'72')
    change('native ledge flattened',[18,20],'01')
    change('upper deck blocked',[10,14],'05')
    change('lower deck blocked',[15,27],'05')
    change('upper north rail removed',[10,13],'01')
    change('lower south rail removed',[15,28],'01')
    check_piers(design,layout,pack);rejected=[]
    for name,mutate in cases:
        d,l=deepcopy(design),deepcopy(layout);mutate(d,l)
        try:check_piers(d,l,pack)
        except AssertionError:rejected.append(name)
        else:raise AssertionError('Negative fixture accepted: '+name)
    return dict(negative_cases_rejected=len(rejected),cases=rejected)


def main():
    parser=argparse.ArgumentParser();parser.add_argument('--pack',type=Path,default=ROOT/'content-packs/text-tui.crystalpack');parser.add_argument('--serve',action='store_true');parser.add_argument('--watch',action='store_true');parser.add_argument('--port',type=int,default=8794);parser.add_argument('--verify-published',action='store_true');parser.add_argument('--self-test',action='store_true');args=parser.parse_args()
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
        server=ThreadingHTTPServer(('127.0.0.1',args.port),partial(Handler,directory=str(OUTPUT)));threading.Thread(target=server.serve_forever,daemon=True).start();print(f'River Valley review: http://127.0.0.1:{args.port}/',flush=True)
    last=None
    while True:
        revision=(LAYOUT.stat().st_mtime_ns,DOSSIER.stat().st_mtime_ns)
        if revision!=last:
            try:
                design,layout=json.loads(DOSSIER.read_text()),json.loads(LAYOUT.read_text());extra=check_piers(design,layout,pack);published=verify_published(design,layout,pack) if args.verify_published else {};report={**render(design,layout,pack,art,output=OUTPUT),**extra,**published};scene=Image.open(OUTPUT/'whole-route.png');views=[]
                for v in design['surf_viewpoints']:
                    x,y=v['tile'];im=render_stage(scene, 'crop', OUTPUT, 'review_river_valley-main-line142-37-crop', (x*16-80,y*16-72,x*16+80,y*16+72));views.append((v['id'],im));save_render(render_stage(im, 'resize', OUTPUT, 'review_river_valley-main-line142-122-resize', (640,576),Image.Resampling.NEAREST), OUTPUT/('viewport-'+v['id'].lower()+'.png'))
                save_render(room_sheet(views,columns=2), OUTPUT/'surf-boundaries.png')
                (OUTPUT/'revision.json').write_text(json.dumps(report,indent=2)+'\n');state['value']={'ok':not report['errors'],'revision':str(time.time_ns()),'error':'; '.join(report['errors']),**report};print(json.dumps(report),flush=True)
                if report['errors'] and not(args.watch or args.serve):raise SystemExit(1)
            except (AssertionError,KeyError,ValueError,IndexError,TypeError) as error:
                state['value']={'ok':False,'error':str(error)};print('DESIGN ERROR: '+str(error),flush=True)
                if not(args.watch or args.serve):raise SystemExit(1)
            last=revision
        if not(args.watch or args.serve):break
        time.sleep(.2)


if __name__=='__main__':main()
