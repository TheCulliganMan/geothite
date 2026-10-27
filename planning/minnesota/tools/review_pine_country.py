#!/usr/bin/env python3
"""Source-pixel review for individually authored Pine Country terrain/rooms."""
from render_archive import save_render
from render_stages import render_stage, retain_render_bytes
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
from review_league_approach import HTML,render as render_checked,verify_published
from review_grand_marais import sprite
from review_north_shore import oriented_sprite
from review_gunflint import flood,neighbors

ROOT=Path(__file__).resolve().parents[3]
LAYOUT=ROOT/'planning/minnesota/routes/MN_R06.layout.json'
DOSSIER=ROOT/'planning/minnesota/routes/MN_R06.json'
OUTPUT=ROOT/'planning/minnesota/renders/pine-country'
PAGE=HTML.replace('League Approach','Pine Country Road').replace('all eighteen chunks','two pine-road maps').replace('Border Lake · Wind Tor · Granite switchback · Dry saddle · Cedar Shelter · Long Lake · Old quarry · Ely arrival.','Needle Camp · Outlet bridge · Lake pier · Pine lookout · Brainerd approach.').replace('Cedar Shelter','Needle Camp cabin').replace('width:1536px','width:1024px').replace('(native?3072:1536)','(native?1024:512)').replace('native=false','native=true').replace('<button data-layer="terrain">Terrain</button>','<button data-layer="terrain">Terrain before Cut</button><button data-layer="after-cut/whole-route">After Cut</button>').replace('data-layer="ledges"','data-layer="after-cut/ledges"').replace('One-way paths</button>','One-way paths after Cut</button>').replace('Residents</button>','Residents before Cut</button>').replace("if(room)room.src='interiors.png?r='+rev","if(room)room.src='interiors.png?r='+rev;document.querySelector('#cut-gates').src='cut-gates.png?r='+rev;document.querySelector('#surf-boundaries').src='surf-boundaries.png?r='+rev").replace('<h2>Needle Camp cabin</h2>','<h2>Surf boundary rocks</h2><img id="surf-boundaries" src="surf-boundaries.png"><h2>Lookout Cut gate: before / after</h2><img id="cut-gates" src="cut-gates.png"><h2>Needle Camp cabin</h2>')


def check_piers(design,layout,pack):
    from render_reference import layout_blocks
    from review_headwaters import prepare
    from review_gunflint import flood
    prepare(pack,layout)
    assert design['dimensions_metatiles']==layout['dimensions_metatiles']==[32,44] and design['chunk_columns']==1 and design['chunk_rows']==2,'Retain the authored two-map extent'
    assert [len(design[k]) for k in ['trainers','residents','items','signs','doors','interiors']]==[5,3,3,6,1,1],'Retain the authored actor and recovery contracts'
    blocks=layout_blocks(layout);w,h=layout['dimensions_metatiles'];collision=pack['data']['tilesets']['mn_headwaters']['collision']
    def kind(x,y):return collision[f'{blocks[y//2*w+x//2]:02x}'][y%2*2+x%2]
    occupied={tuple(a['tile']) for k in ['trainers','residents','items'] for a in design[k]}
    walking={(x,y) for y in range(h*2) for x in range(w*2) if kind(x,y) in {'FLOOR','TALL_GRASS'} and (x,y) not in occupied}
    start,end=[tuple(v*2 for v in e['gate_metatile']) for e in design['endpoints']]
    assert end in flood(start,walking),'Two-way road disconnected'
    for crossing in layout['crossings']:
        for x,y in crossing['deck_blocks']:assert all(kind(x*2+dx,y*2+dy)=='FLOOR' for dx in [0,1] for dy in [0,1]),'Deck collision'
        for x,y in crossing['north_rail_blocks']:assert [kind(x*2,y*2),kind(x*2,y*2+1)]==['WALL','FLOOR'],'North rail'
        for x,y in crossing['south_rail_blocks']:assert [kind(x*2,y*2),kind(x*2,y*2+1)]==['FLOOR','WALL'],'South rail'
        for x,y in crossing['water_under_rail_blocks']:assert kind(x*2,y*2)=='WATER','Water below deck'
        for p in crossing['core_walk_line']:assert kind(*p)=='FLOOR','Deck or approach gap'
    bridge=layout['crossings'][0]
    bridge_cells={(x*2+dx,y*2+dy) for key in ['deck_blocks','north_rail_blocks','south_rail_blocks'] for x,y in bridge[key] for dx in [0,1] for dy in [0,1]}
    assert end not in flood(start,walking-bridge_cells),'Dry bridge bypass'
    assert kind(30,43)=='WALL' and kind(31,43)=='FLOOR','Closed pier end'
    for ledge in design['one_way_ledges']:
        x0,y0,x1,y1=ledge['bounds_core']
        assert all(kind(x,y0)=='HOP_DOWN' and kind(x,y0+1)=='WALL' for x in range(x0,x1)),'Native ledge face'
        assert all(kind(*at)=='WALL' for at in ledge['side_barriers']),'Ledge side closure'
    # A lake lookout must visibly contain water in the actual native camera frame.
    view=next(v for v in design['viewpoints'] if v['id']=='LOOKOUT_LAKE_VIEW')
    vx,vy=view['tile']
    visible_water=sum(kind(x,y)=='WATER' for y in range(vy-4,vy+5) for x in range(vx-5,vx+5))
    assert visible_water>=18,'Lake is outside the lookout camera'
    from collections import deque
    choice=design['battle_or_grass_choices'][0]
    trainer=next(t for t in design['trainers'] if t['id']==choice['trainer_id'])
    assert trainer['tile']==[45,55] and trainer['facing']=='east' and trainer['sight_range']==2,'Battle lane sight contract'
    x0,y0,x1,y1=choice['bounds_core'];band={(x,y) for x in range(x0,x1) for y in range(y0,y1)}
    assert all(kind(*p)=='TALL_GRASS' for p in band),'Grass alternative band changed'
    def cost(a,b,forbid):
        costs={a:0};todo=deque([a])
        while todo:
            x,y=todo.popleft()
            for q in [(x-1,y),(x+1,y),(x,y-1),(x,y+1)]:
                if q not in walking or q in forbid:continue
                n=costs[(x,y)]+int(q in band)
                if n>=costs.get(q,10**9):continue
                costs[q]=n;(todo.append if q in band else todo.appendleft)(q)
        return costs.get(b)
    approach,exit=tuple(choice['approach_core']),tuple(choice['exit_core'])
    assert cost(approach,exit,set())==0,'Short road is missing'
    for a,b in [(approach,exit),(exit,approach)]:
        n=cost(a,b,{(46,55),(47,55)})
        assert n is not None and n>=choice['minimum_grass_steps'],'Both battle and grass can be freely bypassed'
    return {'source_metatile_varieties_used':len(set(blocks)),'necessary_bridge_checked':True,'closed_fishing_pier_checked':True,'native_ledge_faces_checked':1,'battle_or_grass_choices_checked':1,'lookout_water_core_tiles_in_native_frame':visible_water}


def cut_states(design,layout,pack):
    """Prove actual-catalog Cut gates seal their complete optional targets."""
    blocks=layout_blocks(layout);after=list(blocks);w,h=layout['dimensions_metatiles']
    collision=pack['data']['tilesets'][layout['compiled_tileset']]['collision']
    def kind(source,point):
        x,y=point;return collision[f'{source[y//2*w+x//2]:02x}'][y%2*2+x%2]
    actors=design['trainers']+design['residents']+design['items']
    occupied={tuple(a['tile']) for a in actors}
    start,end=[tuple(v*2 for v in e['gate_metatile']) for e in design['endpoints']]
    def reachable(source):
        walking={(x,y) for y in range(h*2) for x in range(w*2) if kind(source,(x,y)) in {'FLOOR','TALL_GRASS'} and (x,y) not in occupied}
        return flood(start,walking)
    before_seen=reachable(blocks);gates=design['cut_gates'];assert len(gates)==1,'Retain the optional lookout gate'
    cut=pack['data']['field_moves']['cut'];assert cut['move_id']=='CUT' and cut['badge']=={'region':'johto','index':1},'Inherited Cut prerequisite changed'
    rules=cut['replacements'].get(layout['compiled_tileset'],cut['replacements'][layout['tileset']])
    if 'gfx/tilesets/mn_headwaters.png' in pack['runtime_files']:
        assert layout['compiled_tileset'] in cut['replacements'],'Native tileset lacks its inherited Cut behavior'
    declared={tuple(g['tile']) for g in gates}
    actual={(x,y) for y in range(h*2) for x in range(w*2) if kind(blocks,(x,y))=='CUT_TREE'}
    assert actual==declared,'Unintended decorative or missing CUT_TREE quadrant'
    for gate in gates:
        x,y=gate['tile'];source=blocks[y//2*w+x//2];rule=rules.get(source,rules.get(str(source)))
        assert rule and rule['variant']=='tree','Gate has no real native tree replacement'
        assert int(gate['replacement_block'],16)==rule['replacement_block_id'],'Replacement differs from the actual Cut catalog'
        assert tuple(gate['approach']) in before_seen and tuple(gate['approach']) in neighbors((x,y)),'Gate approach is not reachable before Cut'
        after[y//2*w+x//2]=rule['replacement_block_id']
    after_seen=reachable(after);assert end in before_seen and end in after_seen,'Cut blocks the main road'
    for point in [(9,62),(31,43)]:assert point in before_seen and point in after_seen,'Cut blocks recovery or the fishing pier'
    gated={actor_id for gate in gates for actor_id in gate['gated_actor_ids']}
    assert gated=={'MN_R06_T05','MN_R06_ITEM_04'},'Retain the optional Hiker and Ether contract'
    assert gated<={a['id'] for a in actors},'Unknown gated actor identity'
    for actor in actors:
        approaches=set(neighbors(tuple(actor['tile'])))
        if actor['id'] in gated:
            assert not approaches&before_seen,actor['id']+': freely reachable before Cut'
            assert approaches&after_seen,actor['id']+': unreachable after Cut'
        else:assert approaches&before_seen,actor['id']+': accidentally gated main-route actor'
    for actor in design['signs']+design['doors']:
        assert set(neighbors(tuple(actor['tile'])))&before_seen,actor['id']+': main interaction accidentally gated'
    reports=[]
    for gate in gates:
        for target in gate['targets']:
            target=tuple(target)
            assert target not in before_seen,'Gated target is reachable before Cut: '+str(target)
            assert target in after_seen,'Gated target remains unreachable after Cut: '+str(target)
        reports.append({'id':gate['id'],'tile':gate['tile'],'approach':gate['approach'],'replacement_block':gate['replacement_block'],'sealed_targets_checked':gate['targets'],'gated_actor_ids':gate['gated_actor_ids']})
    return after,dict(cut_gates_checked=reports,pre_cut_reachable_tiles=len(before_seen),post_cut_reachable_tiles=len(after_seen),cut_prerequisites={'party_known_move':'CUT','badge_region':'johto','badge_index':1},main_recovery_and_pier_available_before_cut=True)


def check_water_edges(design,layout,pack,after_blocks):
    """Audit every outside edge in both states using conservative Walk+Surf."""
    blocks=layout_blocks(layout);w,h=layout['dimensions_metatiles'];collision=pack['data']['tilesets'][layout['compiled_tileset']]['collision']
    def kind(source,point):
        x,y=point;return collision[f'{source[y//2*w+x//2]:02x}'][y%2*2+x%2]
    perimeter={(x,y) for y in range(h*2) for x in range(w*2) if x in {0,w*2-1} or y in {0,h*2-1}}
    assert len(perimeter)==300,'Complete 64 by 88 core perimeter must be audited'
    assert not any(kind(blocks,p)=='WATER' for p in perimeter),'Unclosed water on an outer map edge'
    for barrier in layout['outer_water_barriers']:
        assert barrier['side'] in {'west','south'},'Unexpected water barrier edge'
        for x,y,block in barrier['blocks']:
            assert block in {'30','34','38','39','3a'} and block in layout['additional_water_blocks'],'Use inspected original blue-water rock recipes'
            assert blocks[y*w+x]==int(block,16),'Declared water-rock recipe differs from terrain'
            outer=[(x*2,y*2),(x*2,y*2+1)] if barrier['side']=='west' else [(x*2,y*2+1),(x*2+1,y*2+1)]
            assert all(point in perimeter and kind(blocks,point)=='BUOY' for point in outer),'Water rocks do not close their outside quadrants'
    occupied={tuple(a['tile']) for k in ['trainers','residents','items'] for a in design[k]}
    start=tuple(v*2 for v in design['endpoints'][0]['gate_metatile']);reports=[]
    for name,source in [('before_cut',blocks),('after_cut',after_blocks)]:
        allowed={(x,y) for y in range(h*2) for x in range(w*2) if kind(source,(x,y)) in {'FLOOR','TALL_GRASS','WATER'} and (x,y) not in occupied}
        seen=flood(start,allowed);water={point for point in seen if kind(source,point)=='WATER'}
        assert water,'Audit must actually reach Surf water'
        assert not water&perimeter,'Surf reaches an unconnected outside edge'
        for view in design['surf_viewpoints']:assert tuple(view['tile']) in water,'Water boundary camera probe is not reachable by Surf'
        for gate in design['cut_gates']:
            for target in gate['targets']:
                if name=='before_cut':assert tuple(target) not in seen,'Surf bypasses the optional Cut gate'
                else:assert tuple(target) in seen,'Post-Cut target disconnected from Walk+Surf'
        reports.append(dict(state=name,reachable_water_tiles=len(water),reachable_outer_edge_water_tiles=0))
    seam=layout['internal_water_connections'][0];assert seam==dict(from_map='MnR06C00R00',direction='south',target_map='MnR06C00R01',reverse_direction='north',offset=0,core_boundary_y=48),'Retain reciprocal internal water seam'
    xs=[x for x in range(w*2) if kind(blocks,(x,47))==kind(blocks,(x,48))=='WATER']
    assert xs,'Internal neighboring maps must retain open matching water quadrants'
    native=all(name in pack['data']['maps'] for name in [seam['from_map'],seam['target_map']])
    if native:
        for source,target,direction in [(seam['from_map'],seam['target_map'],'south'),(seam['target_map'],seam['from_map'],'north')]:
            assert dict(direction=direction,target_map=target,offset=0) in pack['data']['maps'][source]['attributes']['connections'],'Open water seam has no real reciprocal native neighbor'
    return dict(outer_perimeter_core_tiles_checked=len(perimeter),outer_water_barrier_core_tiles=sum(kind(blocks,p)=='BUOY' for p in perimeter),surf_perimeter_states=reports,open_internal_water_seam_core_x=xs,internal_water_neighbors_verified_in_native_pack=native)


def render(design,layout,pack,art,output=OUTPUT):
    """Reuse complete post-Cut checks; render both actual authoring states."""
    started=time.perf_counter();after,gate_report=cut_states(design,layout,pack);water_report=check_water_edges(design,layout,pack,after)
    post=deepcopy(layout)
    for gate in design['cut_gates']:
        post['layers'].append(dict(kind='point',position=[v//2 for v in gate['tile']],block=gate['replacement_block']))
    post_design=deepcopy(design);post_design['cut_gates']=[]
    report=render_checked(post_design,post,pack,art,output=output/'after-cut')
    retain_render_bytes((output/'after-cut/ledges.png').read_bytes(), output/'ledges.png')
    blocks=layout_blocks(layout);width,height=layout['dimensions_metatiles']
    assert layout_blocks(post)==after,'Post-Cut preview differs from the actual catalog mutation'
    terrain=art.render(layout['tileset'],width,height,blocks,derived=layout['derived_blocks']);scene=render_stage(terrain, 'convert', output, 'review_pine_country-render-line183-101-convert', 'RGBA')
    for actor in design['trainers']+design['residents']:
        scene.alpha_composite(oriented_sprite(pack,actor['sprite'],actor['facing']),(actor['tile'][0]*16,actor['tile'][1]*16-4))
    for actor in design['items']:scene.alpha_composite(sprite(pack,'SPRITE_POKE_BALL'),(actor['tile'][0]*16,actor['tile'][1]*16-4))
    save_render(terrain, output/'terrain.png');save_render(render_stage(scene, 'convert', output, 'review_pine_country-render-line187-59-convert', 'RGB'), output/'whole-route.png')
    digest=hashlib.sha256(scene.tobytes()).hexdigest()[:8];save_render(render_stage(scene, 'convert', output, 'review_pine_country-render-line188-71-convert', 'RGB'), output/f'whole-route-{digest}.png')
    save_render(render_stage(render_stage(scene, 'resize', output, 'review_pine_country-render-line189-16-resize', (width*16,height*16),Image.Resampling.NEAREST), 'convert', output, 'review_pine_country-render-line189-16-convert', 'RGB'), output/'overview.png')
    overlay=scene.copy();draw=ImageDraw.Draw(overlay,'RGBA')
    for row in range(design['chunk_rows']):
        x,y=0,row*768;draw.rectangle((x,y,x+1023,min(y+767,scene.height-1)),outline=(255,255,255,160),width=1);draw.text((8,y+8),f'MnR06C00R{row:02}',fill='white',stroke_width=1,stroke_fill='black')
    for actor in design['trainers']+design['residents']+design['items']+design['signs']+design['doors']+design['viewpoints']+design['cut_gates']:
        x,y=actor['tile'];draw.rectangle((x*16,y*16,x*16+15,y*16+15),outline=(255,210,80,255),width=2);draw.text((x*16,y*16-12),actor['id'].removeprefix('MN_R06_'),fill='white',stroke_width=1,stroke_fill='black')
    save_render(render_stage(overlay, 'convert', output, 'review_pine_country-render-line195-16-convert', 'RGB'), output/'anchors.png')
    views=[]
    for view in design['viewpoints']:
        x,y=view['tile'];im=render_stage(render_stage(scene, 'crop', output, 'review_pine_country-render-line198-28-crop', (x*16-80,y*16-72,x*16+80,y*16+72)), 'convert', output, 'review_pine_country-render-line198-28-convert', 'RGB');views.append((view['id']+(' (after Cut access)' if view['id'] in {'LOOKOUT_RETURN','LOOKOUT_LAKE_VIEW'} else ''),im));save_render(render_stage(im, 'resize', output, 'review_pine_country-render-line198-219-resize', (640,576),Image.Resampling.NEAREST), output/('viewport-'+view['id'].lower()+'.png'))
    save_render(room_sheet(views,columns=2), output/'viewpoints.png')
    surf_views=[]
    for view in design['surf_viewpoints']:
        x,y=view['tile'];im=render_stage(render_stage(scene, 'crop', output, 'review_pine_country-render-line202-28-crop', (x*16-80,y*16-72,x*16+80,y*16+72)), 'convert', output, 'review_pine_country-render-line202-28-convert', 'RGB');surf_views.append((view['id']+' (Surf)',im));save_render(render_stage(im, 'resize', output, 'review_pine_country-render-line202-146-resize', (640,576),Image.Resampling.NEAREST), output/('viewport-'+view['id'].lower()+'.png'))
    save_render(room_sheet(surf_views,columns=2), output/'surf-boundaries.png')
    # Rooms are identical in both states; the existing renderer owns their drawing.
    retain_render_bytes((output/'after-cut/interiors.png').read_bytes(), output/'interiors.png')
    cut_views=[];after_scene=render_stage(Image.open(output/'after-cut/whole-route.png'), 'convert', output, 'review_pine_country-render-line206-29-convert', 'RGBA')
    for gate in design['cut_gates']:
        x,y=gate['approach']
        for label,im in [('BEFORE CUT: optional lookout sealed',scene),('AFTER CUT: native 5b to 3c',after_scene)]:
            cut_views.append((label,render_stage(render_stage(im, 'crop', output, 'review_pine_country-render-line210-36-crop', (x*16-80,y*16-72,x*16+80,y*16+72)), 'convert', output, 'review_pine_country-render-line210-36-convert', 'RGB')))
    save_render(room_sheet(cut_views,columns=2), output/'cut-gates.png')
    report.update(**gate_report,**water_report,status='INVALID — see errors' if report['errors'] else 'Before/after source geometry checked; native Cut denial, removal and persistence pending',render_ms=round((time.perf_counter()-started)*1000,1),image=f'whole-route-{digest}.png',after_cut_image='after-cut/'+report['image'])
    (output/'revision.json').write_text(json.dumps(report,indent=2)+'\n')
    return report


def main():
    parser=argparse.ArgumentParser();parser.add_argument('--pack',type=Path,default=ROOT/'content-packs/text-tui.crystalpack');parser.add_argument('--serve',action='store_true');parser.add_argument('--watch',action='store_true');parser.add_argument('--port',type=int,default=8776);parser.add_argument('--verify-published',action='store_true');args=parser.parse_args()
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
        server=ThreadingHTTPServer(('127.0.0.1',args.port),partial(Handler,directory=str(OUTPUT)));threading.Thread(target=server.serve_forever,daemon=True).start();print(f'Pine Country review: http://127.0.0.1:{args.port}/',flush=True)
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
