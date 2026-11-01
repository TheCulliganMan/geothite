#!/usr/bin/env python3
"""Source-pixel review for individually authored Prairie Farm terrain/rooms."""
from render_archive import save_render
from render_stages import render_stage
import argparse
from functools import partial
from http.server import SimpleHTTPRequestHandler,ThreadingHTTPServer
import json
from pathlib import Path
import threading
import time

from render_reference import SourceArt,load_pack,layout_blocks,room_sheet
from review_gunflint import flood,neighbors
from review_grand_marais import sprite
from review_north_shore import oriented_sprite
from PIL import Image,ImageDraw
import hashlib
from copy import deepcopy
from review_league_approach import HTML,check,verify_published
from ledge_paths import draw_ledge_paths

ROOT=Path(__file__).resolve().parents[3]
LAYOUT=ROOT/'planning/minnesota/routes/MN_R02.layout.json'
DOSSIER=ROOT/'planning/minnesota/routes/MN_R02.json'
OUTPUT=ROOT/'planning/minnesota/renders/prairie-farm'
PAGE=HTML.replace('League Approach','Prairie Farm Road').replace('all eighteen chunks','single compact map').replace('Border Lake · Wind Tor · Granite switchback · Dry saddle · Cedar Shelter · Long Lake · Old quarry · Ely arrival.','Nursery bank · Farm drainage bridge · Western pasture · Lind Farm · Tool shed.').replace('Cedar Shelter','Lind Farm and tool shed').replace('width:1536px','width:960px').replace('(native?3072:1536)','(native?960:480)').replace('native=false','native=true')


def check_piers(design,layout,pack):
    from render_reference import layout_blocks
    blocks=layout_blocks(layout);w,h=layout['dimensions_metatiles'];c=pack['data']['tilesets'][layout['compiled_tileset']]['collision']
    def kind(x,y):return c[f'{blocks[y//2*w+x//2]:02x}'][y%2*2+x%2]
    bridge=layout['crossings'][0]
    for x,y in bridge['deck_blocks']:assert all(kind(x*2+dx,y*2+dy)=='FLOOR' for dx in [0,1] for dy in [0,1]),'Bridge deck'
    for x,y in bridge['north_rail_blocks']:assert [kind(x*2,y*2),kind(x*2,y*2+1)]==['WALL','FLOOR'],'Bridge north bollards'
    for x,y in bridge['south_rail_blocks']:assert [kind(x*2,y*2),kind(x*2,y*2+1)]==['FLOOR','WALL'],'Bridge south rail'
    for x,y in bridge['core_walk_line']:assert kind(x,y)=='FLOOR','Bridge walking line'
    ledge=design['one_way_ledges'][0]
    x0,y0,x1,y1=ledge['bounds_core']
    assert all(kind(x0,y)=='WALL' and kind(x0+1,y)=='HOP_LEFT' for y in range(y0,y1)),'Native west drop'
    assert kind(*ledge['upper_approach'])==kind(*ledge['landing'])=='FLOOR','Bank landings'
    assert all(kind(*p)=='WALL' for p in ledge['side_barriers']),'Bank side opening'
    occupied={tuple(a['tile']) for k in ['trainers','residents','items'] for a in design[k]}
    def reach(start,forbid):
        seen={start};todo=[start]
        while todo:
            x,y=todo.pop()
            for dx,dy in [(1,0),(-1,0),(0,1),(0,-1)]:
                q=(x+dx,y+dy)
                if kind(x,y)=='HOP_LEFT' and dx==-1:q=(x-2,y)
                if not(0<=q[0]<w*2 and 0<=q[1]<h*2):continue
                if q in occupied or q in forbid or q in seen:continue
                if kind(*q) in {'FLOOR','TALL_GRASS','HOP_LEFT'}:seen.add(q);todo.append(q)
        return seen
    east,west=[tuple(v*2 for v in e['gate_metatile']) for e in design['endpoints']]
    assert west in reach(east,set()) and east in reach(west,set()),'Both route directions'
    lane={(x,y) for x in range(48,55) for y in range(23,28)}
    assert (40,24) in reach(east,lane),'Nursery alternative missing'
    assert east not in reach((40,24),lane),'Bank can be climbed in reverse'
    deck={(x,y) for x in range(28,36) for y in range(22,28)}
    assert west not in reach(east,deck),'Drain bridge can be bypassed'
    return {'source_metatile_varieties_used':len(set(blocks)),'bridge_checked':True,'one_way_ledges_checked':1,'directional_nursery_choice_checked':True}


def check_water_boundary(design,layout,pack):
    """Prove this deliberately unmountable channel has no reachable Surf edge."""
    blocks=layout_blocks(layout);w,h=layout['dimensions_metatiles'];c=pack['data']['tilesets'][layout['compiled_tileset']]['collision']
    def kind(x,y):return c[f'{blocks[y//2*w+x//2]:02x}'][y%2*2+x%2]
    occupied={tuple(a['tile']) for key in ['trainers','residents','items'] for a in design[key]}
    land={(x,y) for y in range(h*2) for x in range(w*2) if kind(x,y) in {'FLOOR','TALL_GRASS'} and (x,y) not in occupied}
    water={(x,y) for y in range(h*2) for x in range(w*2) if kind(x,y)=='WATER'}
    start=tuple(v*2 for v in design['endpoints'][0]['gate_metatile']);surf=flood(start,land|water)
    mount_pairs={(a,b) for a in land for b in neighbors(a) if b in water}
    assert not mount_pairs,'Channel has a land→water mounting opening'
    assert not surf&water,'Previously fenced channel is Surf reachable'
    assert all(kind(x,22)=='WALL' and kind(x,27)=='WALL' for x in range(28,36)),'Bridge edge permits mounting'
    outer={q for q in water if q[0] in {0,w*2-1} or q[1] in {0,h*2-1}}
    assert outer=={(x,y) for x in range(30,34) for y in [0,43]},'Unexpected drainage boundary footprint'
    assert not outer&surf,'Reachable outer Surf escape'
    return dict(boundary_rule='Reachability: entire channel sealed by native fences and both bridge rails; no water seam claimed',water_tiles=len(water),reachable_water_tiles=len(water&surf),land_water_mount_pairs=len(mount_pairs),outer_water_tiles=len(outer),reachable_outer_surf_tiles=len(outer&surf),outer_water_coordinates=sorted(outer),north_bridge_wall_y=22,south_bridge_wall_y=27,terrain_revision='Unchanged30×22; this is an audit, not a redesign')


def render(design,layout,pack,art,output=OUTPUT):
    """Render the complete source plan using the appropriate reachable-water proof."""
    started=time.perf_counter();blocks=layout_blocks(layout);w,h=layout['dimensions_metatiles'];report,kind,seen=check(design,layout,pack,blocks)
    report.update(check_water_boundary(design,layout,pack));assert not report['errors'],'; '.join(report['errors'])
    terrain=art.render(layout['compiled_tileset'],w,h,blocks);scene=render_stage(terrain, 'convert', output, 'review_prairie_farm-render-line87-49-convert', 'RGBA')
    for a in design['trainers']+design['residents']:scene.alpha_composite(oriented_sprite(pack,a['sprite'],a['facing']),(a['tile'][0]*16,a['tile'][1]*16-4))
    for a in design['items']:scene.alpha_composite(sprite(pack,'SPRITE_POKE_BALL'),(a['tile'][0]*16,a['tile'][1]*16-4))
    output.mkdir(parents=True,exist_ok=True);digest=hashlib.sha256(scene.tobytes()).hexdigest()[:8]
    save_render(terrain, output/'terrain.png');save_render(render_stage(scene, 'convert', output, 'review_prairie_farm-render-line91-59-convert', 'RGB'), output/'whole-route.png');save_render(render_stage(scene, 'convert', output, 'review_prairie_farm-render-line91-119-convert', 'RGB'), output/f'whole-route-{digest}.png');save_render(render_stage(render_stage(scene, 'resize', output, 'review_prairie_farm-render-line91-189-resize', (w*16,h*16),Image.Resampling.NEAREST), 'convert', output, 'review_prairie_farm-render-line91-189-convert', 'RGB'), output/'overview.png');save_render(draw_ledge_paths(scene,design,report['ledge_returns']), output/'ledges.png')
    overlay=scene.copy();draw=ImageDraw.Draw(overlay)
    for a in design['trainers']+design['residents']+design['items']+design['signs']+design['doors']:
        x,y=a['tile'];draw.rectangle((x*16,y*16,x*16+15,y*16+15),outline='#ffd250',width=2);draw.text((x*16,y*16-12),a['id'],fill='white',stroke_fill='black',stroke_width=1)
    save_render(render_stage(overlay, 'convert', output, 'review_prairie_farm-render-line95-16-convert', 'RGB'), output/'anchors.png');views=[]
    for v in design['viewpoints']:
        x,y=v['tile'];im=render_stage(render_stage(scene, 'crop', output, 'review_prairie_farm-render-line97-25-crop', (x*16-80,y*16-72,x*16+80,y*16+72)), 'convert', output, 'review_prairie_farm-render-line97-25-convert', 'RGB');views.append((v['id'],im));save_render(render_stage(im, 'resize', output, 'review_prairie_farm-render-line97-125-resize', (640,576),Image.Resampling.NEAREST), output/f'viewport-{v["id"].lower()}.png')
    save_render(room_sheet(views,2), output/'viewpoints.png');save_render(room_sheet(views,2), output/f'viewpoints-{digest}.png')
    # Audit the broadest yard separately from the best landmark camera.
    x,y=52,36;save_render(render_stage(render_stage(render_stage(scene, 'crop', output, 'review_prairie_farm-render-line100-26-crop', (x*16-80,y*16-72,x*16+80,y*16+72)), 'convert', output, 'review_prairie_farm-render-line100-26-convert', 'RGB'), 'resize', output, 'review_prairie_farm-render-line100-26-resize', (640,576),Image.Resampling.NEAREST), output/f'widest-tool-yard-{digest}.png')
    rooms=[]
    for room in design['interiors']:
        template=pack['data']['maps'][room['template']]['attributes'];im=render_stage(art.render(template['tileset_name'],*room['dimensions_metatiles'],[int(b,16) for row in room['rows'] for b in row.split()],group=template['time_of_day'] or 'day'), 'convert', output, 'review_prairie_farm-render-line103-73-convert', 'RGBA')
        for a in room.get('residents',[]):im.alpha_composite(oriented_sprite(pack,a['sprite'],a['facing']),(a['tile'][0]*16,a['tile'][1]*16-4))
        for a in room.get('items',[]):im.alpha_composite(sprite(pack,'SPRITE_POKE_BALL'),(a['tile'][0]*16,a['tile'][1]*16-4))
        save_render(im, output/f'room-{room["id"]}-original-rgba.png')
        rooms.append((room['id'],im))
    save_render(room_sheet(rooms,1), output/'interiors.png');report.update(status='Golden prairie art; native collision and geography unchanged; earned regional progression remains separate',image=f'whole-route-{digest}.png',render_ms=round((time.perf_counter()-started)*1000,1));(output/'revision.json').write_text(json.dumps(report,indent=2)+'\n');return report


def negative_boundary_checks(design,layout,pack):
    outcomes=[]
    for name,position,block in [('North bridge mounting gap',[16,11],'43'),('South bridge mounting gap',[16,13],'43'),('West fence mounting gap',[14,5],'01')]:
        candidate=deepcopy(layout);candidate['layers'].append(dict(kind='point',name=name,position=position,block=block))
        try:check_water_boundary(design,candidate,pack)
        except AssertionError as error:outcomes.append(dict(mutation=name,rejected=True,reason=str(error)))
        else:raise AssertionError('Uncaught Surf mounting gap: '+name)
    result=dict(source_only=True,cases=outcomes,layout_sha256=hashlib.sha256(LAYOUT.read_bytes()).hexdigest(),dossier_sha256=hashlib.sha256(DOSSIER.read_bytes()).hexdigest());(OUTPUT/'water-boundary-proof.json').write_text(json.dumps(result,indent=2)+'\n');return result


def main():
    parser=argparse.ArgumentParser();parser.add_argument('--pack',type=Path,default=ROOT/'content-packs/minnesota-prairie-review.crystalpack');parser.add_argument('--serve',action='store_true');parser.add_argument('--watch',action='store_true');parser.add_argument('--port',type=int,default=8772);parser.add_argument('--verify-published',action='store_true');parser.add_argument('--negative-checks',action='store_true');args=parser.parse_args()
    if args.negative_checks and (args.watch or args.serve):parser.error('--negative-checks requires one-shot audit')
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
        server=ThreadingHTTPServer(('127.0.0.1',args.port),partial(Handler,directory=str(OUTPUT)));threading.Thread(target=server.serve_forever,daemon=True).start();print(f'Prairie Farm review: http://127.0.0.1:{args.port}/',flush=True)
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
        if not(args.watch or args.serve):
            if args.negative_checks:print(json.dumps(negative_boundary_checks(design,layout,pack)),flush=True)
            break
        time.sleep(.2)


if __name__=='__main__':main()
