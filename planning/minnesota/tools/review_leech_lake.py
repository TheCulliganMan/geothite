#!/usr/bin/env python3
"""Source-pixel review for the hand-authored Leech Lake outlet route."""
from render_archive import save_render
from render_stages import render_stage
import argparse
from copy import deepcopy
from functools import partial
import hashlib
from http.server import SimpleHTTPRequestHandler,ThreadingHTTPServer
import json
from pathlib import Path
import threading
import time

from PIL import Image,ImageDraw

from render_reference import SourceArt,load_pack,layout_blocks,room_sheet
from review_league_approach import HTML,check,render as render_open_route,verify_published
from review_grand_marais import sprite
from review_north_shore import oriented_sprite
from ledge_paths import draw_ledge_paths

ROOT=Path(__file__).resolve().parents[3]
LAYOUT=ROOT/'planning/minnesota/routes/MN_R07.layout.json'
DOSSIER=ROOT/'planning/minnesota/routes/MN_R07.json'
OUTPUT=ROOT/'planning/minnesota/renders/leech-lake'
PAGE=HTML.replace('League Approach','Leech Lake Approach').replace('all eighteen chunks','two wetland maps').replace('Border Lake · Wind Tor · Granite switchback · Dry saddle · Cedar Shelter · Long Lake · Old quarry · Ely arrival.','Outlet rest · Southern causeway · Pine peninsula · Wet meadow · Upper bridge · Fishing landing · Sandbank return.').replace('Cedar Shelter','No enterable facade').replace('width:1536px','width:1024px').replace('(native?3072:1536)','(native?1024:512)').replace('native=false','native=true').replace('<h2>No enterable facade</h2><img id="rooms" src="interiors.png">','<p>This route has no enterable facades.</p>')
PAGE=PAGE.replace('<button data-layer="terrain">Terrain</button>','<button data-layer="terrain">Terrain</button><button data-layer="after-cut">After Cut</button>')


def check_piers(design,layout,pack):
    """Check source-authenticated decks, closed rails, crossings and the native drop."""
    from render_reference import layout_blocks
    from review_headwaters import prepare
    from review_gunflint import flood
    prepare(pack,layout)
    blocks=layout_blocks(layout);w,h=layout['dimensions_metatiles'];c=pack['data']['tilesets']['mn_headwaters']['collision']
    def kind(x,y):return c[f'{blocks[y//2*w+x//2]:02x}'][y%2*2+x%2]
    occupied={tuple(a['tile']) for k in ['trainers','residents','items'] for a in design[k]}
    walk={(x,y) for y in range(h*2) for x in range(w*2) if kind(x,y) in {'FLOOR','TALL_GRASS'} and (x,y) not in occupied}
    start,end=[tuple(v*2 for v in e['gate_metatile']) for e in design['endpoints']]
    for crossing in layout['crossings']:
        for x,y in crossing['deck_blocks']:assert all(kind(x*2+dx,y*2+dy)=='FLOOR' for dx in [0,1] for dy in [0,1]),crossing['id']+': deck'
        for x,y in crossing['north_rail_blocks']:assert [kind(x*2,y*2),kind(x*2,y*2+1)]==['WALL','FLOOR'],crossing['id']+': north rail'
        for x,y in crossing['south_rail_blocks']:assert [kind(x*2,y*2),kind(x*2,y*2+1)]==['FLOOR','WALL'],crossing['id']+': south rail'
        for x,y in crossing['water_under_rail_blocks']:assert kind(x*2,y*2)=='WATER',crossing['id']+': water under rail'
        assert all(kind(*p)=='FLOOR' for p in crossing['core_walk_line']),crossing['id']+': approach'
        if crossing['id']!='FISHING_LANDING':
            removed={(x*2+dx,y*2+dy) for key in ['deck_blocks','north_rail_blocks','south_rail_blocks'] for x,y in crossing[key] for dx in [0,1] for dy in [0,1]}
            assert end not in flood(start,walk-removed),crossing['id']+': dry bypass'
    assert kind(47,15)=='WALL','Fishing landing end is open'
    for ledge in design['one_way_ledges']:
        x0,y0,x1,y1=ledge['bounds_core']
        assert all(kind(x,y0)=='HOP_DOWN' and kind(x,y0+1)=='WALL' for x in range(x0,x1)),'Native sandbank face'
        assert all(kind(*p)=='WALL' for p in ledge['side_barriers']),'Native sandbank side bypass'
    return {'source_metatile_varieties_used':len(set(blocks)),'necessary_bridges_checked':2,'closed_end_landing_checked':True,'native_drop_faces_checked':1}


def check_surf_boundary(design,layout,pack):
    """Check every exterior edge and actual native water-neighbor records."""
    blocks=layout_blocks(layout);w,h=layout['dimensions_metatiles']
    collision=pack['data']['tilesets'][layout['compiled_tileset']]['collision']
    def kind(x,y):return collision[f'{blocks[y//2*w+x//2]:02x}'][y%2*2+x%2]
    edges={'north':[(x,0) for x in range(w*2)],'south':[(x,h*2-1) for x in range(w*2)],'west':[(0,y) for y in range(h*2)],'east':[(w*2-1,y) for y in range(h*2)]}
    assert all(kind(*p) not in {'WATER','WHIRLPOOL','WATERFALL'} for edge in edges.values() for p in edge),'Open water reaches an unconnected outer edge'
    assert all(kind(63,y)=='BUOY' for y in range(24,80)),'East water-rock closure has a hole'
    assert all(kind(62,y)=='WATER' for y in range(25,79)),'Rock boundary damaged the inner water lane'
    assert blocks[12*w+31]==0x32 and blocks[39*w+31]==0x3a,'Native rock end caps'
    assert all(blocks[y*w+31]==0x36 for y in range(13,39)),'Native east-facing water rocks'
    seam=[x for x in range(w*2) if kind(x,47)=='WATER' and kind(x,48)=='WATER']
    assert 31 in seam and 61 in seam,'Intended internal water seam was closed'
    maps=pack['data']['maps'];upper='MnR07C00R00';lower='MnR07C00R01'
    published=upper in maps or lower in maps
    if published:
        assert upper in maps and lower in maps,'Only half the water neighbor exists'
        for source,target,direction in [(upper,lower,'south'),(lower,upper,'north')]:
            assert {'direction':direction,'target_map':target,'offset':0} in maps[source]['attributes']['connections'],'Internal water edge lacks actual reciprocal neighbor'
    return {'outer_water_edges_checked':list(edges),'unconnected_outer_water_cells':0,'native_buoy_closure_cells':56,'internal_reciprocal_water_pairs':len(seam),'published_water_neighbor_verified':published}


def inspect_cut_gate(design,layout,pack):
    """Prove the sealed cache with the actual native block replacement.

    The ordinary route remains connected before Cut. A permissive water graph
    also fails to enter the hollow, guarding against an unintended Surf landing.
    This is source geometry evidence, never native field-move execution.
    """
    from review_gunflint import flood,neighbors
    blocks=layout_blocks(layout);w,h=layout['dimensions_metatiles']
    collision=pack['data']['tilesets'][layout['compiled_tileset']]['collision']
    actors=design['trainers']+design['residents']+design['items']
    occupied={tuple(a['tile']) for a in actors}
    start,end=[tuple(v*2 for v in e['gate_metatile']) for e in design['endpoints']]
    def reach(current,water=False):
        def kind(x,y):return collision[f'{current[y//2*w+x//2]:02x}'][y%2*2+x%2]
        kinds={'FLOOR','TALL_GRASS'}|({'WATER'} if water else set())
        allowed={(x,y) for y in range(h*2) for x in range(w*2) if kind(x,y) in kinds and (x,y) not in occupied}
        return flood(start,allowed)
    gates=design['cut_gates'];declared={tuple(g['tile']) for g in gates}
    actual={(x,y) for y in range(h*2) for x in range(w*2) if collision[f'{blocks[y//2*w+x//2]:02x}'][y%2*2+x%2]=='CUT_TREE'}
    assert actual==declared,'Incidental Cut tree or missing gate'
    before=reach(blocks);before_surf=reach(blocks,True)
    assert end in before,'Cut accidentally gates the main route'
    gated_ids={name for gate in gates for name in gate['gated_actor_ids']}
    for actor in actors:
        if actor['id'] not in gated_ids:
            assert any(q in before for q in neighbors(tuple(actor['tile']))),actor['id']+': unintentionally gated actor'
    assert all(tuple(view['tile']) in before for view in design['viewpoints']),'Initial player viewpoint is gated'
    opened=deepcopy(layout);post=list(blocks);reports=[]
    for gate in gates:
        x,y=gate['tile'];index=y//2*w+x//2;previous=blocks[index];replacement=int(gate['replacement_block'],16)
        native=pack['data']['field_moves']['cut']
        row=native['replacements']['johto'][previous]
        assert row['variant']=='tree' and row['replacement_block_id']==replacement,'Non-native Cut replacement'
        assert native['badge']=={'region':'johto','index':1},'Unexpected native Cut permission'
        assert tuple(gate['approach']) in before,'Cut entrance cannot be approached'
        targets={tuple(t) for t in gate['targets']}
        assert targets and not targets&before and not targets&before_surf,'Cache has a walking/Surf bypass'
        for name in gate['gated_actor_ids']:
            actor=next(a for a in actors if a['id']==name)
            approaches={q for q in neighbors(tuple(actor['tile'])) if collision[f'{blocks[q[1]//2*w+q[0]//2]:02x}'][q[1]%2*2+q[0]%2]=='FLOOR'}
            assert approaches<=targets and not approaches&before_surf,'Unsealed cache interaction approach'
        post[index]=replacement
        opened['layers'].append({'name':gate['id']+' native post-Cut state','kind':'point','block':gate['replacement_block'],'position':[x//2,y//2]})
        reports.append({'id':gate['id'],'source_block':f'{previous:02x}','replacement_block':gate['replacement_block'],'targets_checked':len(targets),'pre_cut_walk_and_surf_bypasses':False})
    after=reach(post)
    for gate in gates:assert {tuple(t) for t in gate['targets']}<=after,'Native replacement does not open the cache'
    plain=deepcopy(design);plain['cut_gates']=[]
    report,_,_=check(plain,opened,pack,post)
    assert not report['errors'],'Post-Cut physical checks: '+ '; '.join(report['errors'])
    check_piers(design,opened,pack)
    check_surf_boundary(design,opened,pack)
    return opened,plain,report,{'cut_gates_checked':reports,'initial_walkable_reachable_tiles':len(before),'cut_unlocked_walking_tiles':len(after-before),'cut_native_execution_verified':False}


def render(design,layout,pack,art,output=OUTPUT):
    """Render both source states, checking all ordinary anchors after Cut."""
    started=time.perf_counter()
    opened,plain,_,gates=inspect_cut_gate(design,layout,pack)
    report=render_open_route(plain,opened,pack,art,output=output/'after-cut-views')
    after=render_stage(Image.open(output/'after-cut-views/whole-route.png'), 'convert', output, 'review_leech_lake-render-line140-10-convert', 'RGB')
    after_digest=hashlib.sha256(after.tobytes()).hexdigest()[:8]
    save_render(after, output/f'whole-route-after-{after_digest}.png');save_render(after, output/'after-cut.png')
    w,h=layout['dimensions_metatiles'];blocks=layout_blocks(layout)
    terrain=art.render(layout['tileset'],w,h,blocks,derived=layout['derived_blocks'])
    scene=render_stage(terrain, 'convert', output, 'review_leech_lake-render-line145-10-convert', 'RGBA')
    for actor in design['trainers']+design['residents']:
        scene.alpha_composite(oriented_sprite(pack,actor['sprite'],actor['facing']),(actor['tile'][0]*16,actor['tile'][1]*16-4))
    for actor in design['items']:scene.alpha_composite(sprite(pack,'SPRITE_POKE_BALL'),(actor['tile'][0]*16,actor['tile'][1]*16-4))
    digest=hashlib.sha256(render_stage(scene, 'convert', output, 'review_leech_lake-render-line149-26-convert', 'RGB').tobytes()).hexdigest()[:8]
    save_render(terrain, output/'terrain.png');save_render(render_stage(scene, 'convert', output, 'review_leech_lake-render-line150-59-convert', 'RGB'), output/'whole-route.png')
    save_render(render_stage(scene, 'convert', output, 'review_leech_lake-render-line151-16-convert', 'RGB'), output/f'whole-route-{digest}.png');save_render(render_stage(scene, 'convert', output, 'review_leech_lake-render-line151-86-convert', 'RGB'), output/f'whole-route-before-{digest}.png')
    save_render(render_stage(render_stage(scene, 'resize', output, 'review_leech_lake-render-line152-16-resize', (w*16,h*16),Image.Resampling.NEAREST), 'convert', output, 'review_leech_lake-render-line152-16-convert', 'RGB'), output/'overview.png')
    save_render(draw_ledge_paths(scene,design,report['ledge_returns']), output/'ledges.png')
    overlay=scene.copy();draw=ImageDraw.Draw(overlay,'RGBA')
    for row in range(design['chunk_rows']):
        y=row*768;draw.rectangle((0,y,1023,y+767),outline=(255,255,255,160),width=1);draw.text((8,y+8),f'MnR07C00R{row:02}',fill='white',stroke_width=1,stroke_fill='black')
    anchors=sum((design[k] for k in ['trainers','residents','items','signs','viewpoints']),[])
    for anchor in anchors:
        x,y=anchor['tile'];draw.rectangle((x*16,y*16,x*16+15,y*16+15),outline=(255,210,80,255),width=2);draw.text((x*16,y*16-12),anchor['id'].removeprefix('MN_R07_'),fill='white',stroke_width=1,stroke_fill='black')
    save_render(render_stage(overlay, 'convert', output, 'review_leech_lake-render-line160-16-convert', 'RGB'), output/'anchors.png')
    views=[]
    for view in design['viewpoints']:
        x,y=view['tile'];im=render_stage(render_stage(scene, 'crop', output, 'review_leech_lake-render-line163-28-crop', (x*16-80,y*16-72,x*16+80,y*16+72)), 'convert', output, 'review_leech_lake-render-line163-28-convert', 'RGB');views.append((view['id'],im));save_render(render_stage(im, 'resize', output, 'review_leech_lake-render-line163-131-resize', (640,576),Image.Resampling.NEAREST), output/('viewport-'+view['id'].lower()+'.png'))
        if view['id']=='CUT_GATE':
            save_render(render_stage(im, 'resize', output, 'review_leech_lake-render-line165-24-resize', (640,576),Image.Resampling.NEAREST), output/f'cut-gate-before-{digest}.png')
            save_render(render_stage(render_stage(after, 'crop', output, 'review_leech_lake-render-line166-24-crop', (x*16-80,y*16-72,x*16+80,y*16+72)), 'resize', output, 'review_leech_lake-render-line166-24-resize', (640,576),Image.Resampling.NEAREST), output/f'cut-gate-after-{after_digest}.png')
    save_render(room_sheet(views,columns=2), output/'viewpoints.png')
    report.update(**gates,**check_surf_boundary(design,layout,pack),image=f'whole-route-before-{digest}.png',after_cut_image=f'whole-route-after-{after_digest}.png',render_ms=round((time.perf_counter()-started)*1000,1),status='Source pre/post-Cut geometry and complete Surf boundary checked; native execution and persistence remain unverified')
    (output/'revision.json').write_text(json.dumps(report,indent=2)+'\n')
    return report


def main():
    parser=argparse.ArgumentParser();parser.add_argument('--pack',type=Path,default=ROOT/'content-packs/text-tui.crystalpack');parser.add_argument('--serve',action='store_true');parser.add_argument('--watch',action='store_true');parser.add_argument('--port',type=int,default=8777);parser.add_argument('--verify-published',action='store_true');args=parser.parse_args()
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
        server=ThreadingHTTPServer(('127.0.0.1',args.port),partial(Handler,directory=str(OUTPUT)));threading.Thread(target=server.serve_forever,daemon=True).start();print(f'Leech Lake review: http://127.0.0.1:{args.port}/',flush=True)
    last=None
    while True:
        revision=(LAYOUT.stat().st_mtime_ns,DOSSIER.stat().st_mtime_ns)
        if revision!=last:
            try:
                design,layout=json.loads(DOSSIER.read_text()),json.loads(LAYOUT.read_text());extra=check_piers(design,layout,pack);published=verify_published(design,layout,pack) if args.verify_published else {};report={**render(design,layout,pack,art,output=OUTPUT),**extra,**published};state['value']={'ok':not report['errors'],'revision':str(time.time_ns()),'error':'; '.join(report['errors']),**report};print(json.dumps({**report,'ledge_returns':[{k:v for k,v in row.items() if k!='return_path_core'} for row in report['ledge_returns']]}),flush=True)
                if report['errors'] and not(args.watch or args.serve):raise SystemExit(1)
            except (AssertionError,KeyError,ValueError,IndexError,TypeError) as error:
                state['value']={'ok':False,'error':str(error)};print('DESIGN ERROR: '+str(error),flush=True)
                if not(args.watch or args.serve):raise SystemExit(1)
            last=revision
        if not(args.watch or args.serve):break
        time.sleep(.2)


if __name__=='__main__':main()
