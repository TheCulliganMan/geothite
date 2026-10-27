#!/usr/bin/env python3
"""Source-pixel review for individually authored Central Lakes terrain/rooms."""
from render_archive import save_render
from render_stages import render_stage
import argparse
from functools import partial
from http.server import SimpleHTTPRequestHandler,ThreadingHTTPServer
import json
from pathlib import Path
import threading
import time

from render_reference import SourceArt,load_pack,layout_blocks
from review_gunflint import flood
import hashlib
from copy import deepcopy
from PIL import Image
from review_league_approach import HTML,render,verify_published

ROOT=Path(__file__).resolve().parents[3]
LAYOUT=ROOT/'planning/minnesota/routes/MN_R15.layout.json'
DOSSIER=ROOT/'planning/minnesota/routes/MN_R15.json'
OUTPUT=ROOT/'planning/minnesota/renders/central-lakes'
PAGE=HTML.replace('League Approach','Central Lakes West').replace('all eighteen chunks','single compact map').replace('Border Lake · Wind Tor · Granite switchback · Dry saddle · Cedar Shelter · Long Lake · Old quarry · Ely arrival.','Grass gate · Battle or meadow detour · Outlet bridge · Gated fishing dock · One lake cabin.').replace('Cedar Shelter','One enterable lake cabin').replace('width:1536px','width:1024px').replace('(native?3072:1536)','(native?1024:512)').replace('native=false','native=true')


def check_piers(design,layout,pack):
    from render_reference import layout_blocks
    blocks=layout_blocks(layout);w=layout['dimensions_metatiles'][0];c=pack['data']['tilesets']['kanto']['collision']
    def kind(x,y):return c[f'{blocks[y//2*w+x//2]:02x}'][y%2*2+x%2]
    for pier in layout['crossings']:
        for x,y in pier['deck_blocks']:assert all(kind(x*2+dx,y*2+dy)==pier['deck_collision'] for dx in [0,1] for dy in [0,1]),pier['id']+': native deck'
        for x,y in pier['south_rail_blocks']:assert [kind(x*2,y*2),kind(x*2,y*2+1)]==['FLOOR','WALL'],pier['id']+': rail'
        for x,y in pier['water_under_rail_blocks']:assert kind(x*2,y*2)=='WATER',pier['id']+': water below rail'
        for x,y in pier['core_walk_line']:assert kind(x,y) in {'FLOOR','LADDER'},pier['id']+': blocked walking line'
    for x in range(15,18):
        assert [kind(x*2,26),kind(x*2,27)]==['WALL','FLOOR'],'Bridge upper bollards'
    for x in range(18,21):
        assert [kind(x*2,12),kind(x*2,13)]==['WALL','FLOOR'],'Dock upper bollards'
    for y in list(range(6,14))+list(range(16,24)):
        assert kind(42,y)=='WALL','Fishing bank rail has an unintended opening'
    assert all(kind(42,y)=='FLOOR' for y in [14,15]),'Dock entrance is blocked'
    assert all(kind(34,y)=='WALL' for y in [14,15]),'Dock west end is open'
    for ledge in design['one_way_ledges']:
        x0,y0,x1,y1=ledge['bounds_core']
        assert all(kind(x,y0)=='HOP_DOWN' and kind(x,y0+1)=='WALL' for x in range(x0,x1)),ledge['id']+': source hop face'
        assert kind(*ledge['upper_approach'])=='FLOOR' and kind(*ledge['landing'])=='FLOOR',ledge['id']+': landing'
        assert all(kind(*at) in {'WALL','WATER'} for at in ledge['side_barriers']),ledge['id']+': sideways bypass'
    # Directional traversal: entering HOP_DOWN is allowed, departing south
    # jumps two cells. A wall never becomes bidirectional floor for this check.
    occupied={tuple(a['tile']) for k in ['trainers','residents','items'] for a in design[k]}
    hazards=set()
    from review_gunflint import VECTORS
    for a in design['trainers']:
        x,y=a['tile'];dx,dy=VECTORS[a['facing']]
        for n in range(1,a['sight_range']+1):
            at=(x+dx*n,y+dy*n)
            if kind(*at) not in {'FLOOR','TALL_GRASS'}:break
            hazards.add(at)
    def reach(start,forbid):
        seen={start};todo=[start]
        while todo:
            x,y=todo.pop()
            for dx,dy in [(1,0),(-1,0),(0,1),(0,-1)]:
                q=(x+dx,y+dy)
                if kind(x,y)=='HOP_DOWN' and dy==1:q=(x,y+2)
                if not(0<=q[0]<w*2 and 0<=q[1]<layout['dimensions_metatiles'][1]*2):continue
                if q in occupied or q in forbid or q in seen:continue
                if kind(*q) in {'FLOOR','TALL_GRASS','HOP_DOWN'}:seen.add(q);todo.append(q)
        return seen
    east,west=[tuple(v*2 for v in e['gate_metatile']) for e in design['endpoints']]
    assert west in reach(east,set()) and east in reach(west,set()),'Physical directed route disconnected'
    assert west not in reach(east,hazards),'Ron can be bypassed from the east'
    assert east in reach(west,hazards),'Eastbound approach lost its grass/ledge alternative'
    for door in design['doors']:assert kind(*door['arrival_tile'])=='FLOOR','Safe exterior return is not floor'
    used={blocks[i] for i in range(len(blocks))}
    return {'source_metatiles_retained':128,'source_metatile_varieties_used':len(used),'piers_checked':len(layout['crossings']),'water_barriers_checked':True,'one_way_ledges_checked':len(design['one_way_ledges'])}


def check_water_boundaries(design,layout,pack):
    """Audit native Kanto rock collision and complete reachable walking/Surf edge."""
    blocks=layout_blocks(layout);w,h=layout['dimensions_metatiles'];collision=pack['data']['tilesets']['kanto']['collision']
    def kind(x,y):return collision[f'{blocks[y//2*w+x//2]:02x}'][y%2*2+x%2]
    occupied={tuple(a['tile']) for key in ['trainers','residents','items'] for a in design[key]}
    land={(x,y) for y in range(h*2) for x in range(w*2) if kind(x,y) in {'FLOOR','TALL_GRASS'} and (x,y) not in occupied}
    water={(x,y) for y in range(h*2) for x in range(w*2) if kind(x,y)=='WATER'}
    start=tuple(v*2 for v in design['endpoints'][0]['gate_metatile']);surf=flood(start,land|water)
    outer=lambda q:q[0] in {0,w*2-1} or q[1] in {0,h*2-1}
    assert not {q for q in water if outer(q)},'Unclosed outer water collision'
    assert not {q for q in surf&water if outer(q)},'Reachable Surf escape'
    assert collision['6b']==['WATER','WATER','WALL','WALL'],'Wrong source rock collision'
    for boundary in layout['water_boundaries']:
        for x,y in boundary['blocks']:
            assert blocks[y*w+x]==0x6b,boundary['id']+': missing native rock'
            assert all(kind(x*2+dx,y*2+1)=='WALL' for dx in [0,1]),boundary['id']+': open rock row'
    assert all(blocks[x]==0x0f for x in range(12,20)),'North cap must close boundary'
    assert not {(x,2) for x in range(24,40)}&surf,'Outside water strip can be entered behind north rocks'
    return dict(water_boundary_revision='Source closure; current native publication pending',outer_water_tiles=0,reachable_outer_surf_tiles=0,native_water_rock_blocks=11,native_forest_cap_blocks=8,source_water_rock='Kanto6b: WATER/WATER/WALL/WALL',reachable_walk_surf_tiles=len(surf),outer_walk_openings=sorted(q for q in land if outer(q)))


def boundary_cameras(design,layout,pack,art):
    im=art.render('kanto',*layout['dimensions_metatiles'],layout_blocks(layout));images=[]
    for name,box in [('north-rocks',(416,0,576,144)),('north-west-join',(320,0,480,144)),('north-east-join',(560,0,720,144)),('south-rocks',(432,432,592,576))]:
        crop=render_stage(render_stage(im, 'crop', OUTPUT, 'review_central_lakes-boundary_cameras-line103-13-crop', box), 'resize', OUTPUT, 'review_central_lakes-boundary_cameras-line103-13-resize', (640,576),Image.Resampling.NEAREST);digest=hashlib.sha256(crop.tobytes()).hexdigest()[:8];save_render(crop, OUTPUT/f'boundary-{name}-{digest}.png');images.append(dict(id=name,source_pixel_bounds=box,image=f'boundary-{name}-{digest}.png'))
    (OUTPUT/'water-boundary-cameras.json').write_text(json.dumps(images,indent=2)+'\n')


def negative_water_checks(design,layout,pack):
    outcomes=[]
    for name,position in [('north cap hole',[15,0]),('north rock gap',[15,1]),('south rock gap',[16,17])]:
        candidate=deepcopy(layout);candidate['layers'].append(dict(kind='point',name=name,position=position,block='43'))
        try:check_water_boundaries(design,candidate,pack)
        except AssertionError as error:outcomes.append(dict(mutation=name,rejected=True,reason=str(error)))
        else:raise AssertionError('Uncaught water boundary defect: '+name)
    result=dict(source_only=True,cases=outcomes,layout_sha256=hashlib.sha256(LAYOUT.read_bytes()).hexdigest(),dossier_sha256=hashlib.sha256(DOSSIER.read_bytes()).hexdigest())
    (OUTPUT/'water-boundary-proof.json').write_text(json.dumps(result,indent=2)+'\n');return result


def compare_walking_baseline(design,layout,path):
    """Read historical native publication and reject any changed land/HOP/door collision."""
    baseline=load_pack(path);module=baseline['data']['maps']['MnR15C00R00'];old=module['blocks'];new=layout_blocks(layout);w,h=layout['dimensions_metatiles']
    assert len(old)==len(new),'Baseline extent changed'
    collision=baseline['data']['tilesets']['kanto']['collision'];land={'FLOOR','TALL_GRASS','HOP_DOWN','LADDER','DOOR'};changed=[]
    for y in range(h*2):
        for x in range(w*2):
            i=y//2*w+x//2;q=y%2*2+x%2;a,b=collision[f'{old[i]:02x}'][q],collision[f'{new[i]:02x}'][q]
            if a!=b:
                assert a not in land and b not in land,f'Changed playable collision at {x,y}: {a}→{b}'
                changed.append([x,y,a,b])
    result=dict(source_only=True,baseline=str(path),changed_metatiles=sum(a!=b for a,b in zip(old,new)),changed_collision_quadrants=len(changed),all_walking_grass_ledge_door_quadrants_preserved=True,changed_quadrants=changed)
    (OUTPUT/'water-boundary-baseline.json').write_text(json.dumps(result,indent=2)+'\n');return {key:value for key,value in result.items() if key not in {'changed_quadrants','baseline'}}


def main():
    parser=argparse.ArgumentParser();parser.add_argument('--pack',type=Path,default=ROOT/'content-packs/text-tui.crystalpack');parser.add_argument('--serve',action='store_true');parser.add_argument('--watch',action='store_true');parser.add_argument('--port',type=int,default=8770);parser.add_argument('--verify-published',action='store_true');parser.add_argument('--negative-checks',action='store_true');parser.add_argument('--compare-walking-baseline',type=Path);args=parser.parse_args()
    if args.negative_checks and (args.watch or args.serve):parser.error('--negative-checks requires a single run')
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
        server=ThreadingHTTPServer(('127.0.0.1',args.port),partial(Handler,directory=str(OUTPUT)));threading.Thread(target=server.serve_forever,daemon=True).start();print(f'Central Lakes review: http://127.0.0.1:{args.port}/',flush=True)
    last=None
    while True:
        revision=(LAYOUT.stat().st_mtime_ns,DOSSIER.stat().st_mtime_ns)
        if revision!=last:
            try:
                design,layout=json.loads(DOSSIER.read_text()),json.loads(LAYOUT.read_text());extra={**check_piers(design,layout,pack),**check_water_boundaries(design,layout,pack)};published=verify_published(design,layout,pack) if args.verify_published else {};report={**render(design,layout,pack,art,output=OUTPUT),**extra,**published};boundary_cameras(design,layout,pack,art);state['value']={'ok':not report['errors'],'revision':str(time.time_ns()),'error':'; '.join(report['errors']),**report};print(json.dumps(report),flush=True)
                if report['errors'] and not(args.watch or args.serve):raise SystemExit(1)
            except (AssertionError,KeyError,ValueError,IndexError,TypeError) as error:
                state['value']={'ok':False,'error':str(error)};print('DESIGN ERROR: '+str(error),flush=True)
                if not(args.watch or args.serve):raise SystemExit(1)
            last=revision
        if not(args.watch or args.serve):
            if args.negative_checks:print(json.dumps(negative_water_checks(design,layout,pack)),flush=True)
            if args.compare_walking_baseline:print(json.dumps(compare_walking_baseline(design,layout,args.compare_walking_baseline)),flush=True)
            break
        time.sleep(.2)


if __name__=='__main__':main()
