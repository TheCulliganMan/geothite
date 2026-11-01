#!/usr/bin/env python3
"""Fast source-pixel R10 review; does not publish packs or prove runtime input."""
from render_archive import save_render
from render_stages import render_stage
import argparse
from collections import deque
from copy import deepcopy
from functools import partial
from http.server import SimpleHTTPRequestHandler, ThreadingHTTPServer
import json
from pathlib import Path
import threading
import time
from PIL import Image
from render_reference import SourceArt, layout_blocks, load_pack, room_sheet
from review_league_approach import HTML, check, render as checked_render, verify_published
from review_gunflint import flood, neighbors

ROOT=Path(__file__).resolve().parents[3]
DOSSIER=ROOT/'planning/minnesota/routes/MN_R10.json'
LAYOUT=ROOT/'planning/minnesota/routes/MN_R10.layout.json'
OUTPUT=ROOT/'planning/minnesota/renders/iron-range-descent'
PAGE=HTML.replace('League Approach','Iron Range Descent').replace('all eighteen chunks','carrier bench to Superior').replace('Border Lake · Wind Tor · Granite switchback · Dry saddle · Cedar Shelter · Long Lake · Old quarry · Ely arrival.','Carrier bank · Dry mine cleft · Pine neck · Superior shore · Duluth approach.').replace('Cedar Shelter','Dry mine gallery').replace('width:1536px','width:1536px').replace('(native?3072:1536)','(native?1536:768)')

def inspect_contracts(design,layout,pack):
    blocks=layout_blocks(layout);w,h=layout['dimensions_metatiles']
    collision=pack['data']['tilesets'][layout['compiled_tileset']]['collision']
    def kind(x,y):return collision[f'{blocks[y//2*w+x//2]:02x}'][y%2*2+x%2]
    assert (w,h)==(48,20),'Preserve authored two-chunk extent'
    assert design['required_flags']==['BADGE_7','EVENT_MN_DEPOT_CLEARED'],'Preserve actual Q06 release flags'
    occupied={tuple(a['tile']) for k in ['trainers','residents','items'] for a in design[k]}
    start,end=[tuple(2*v for v in e['gate_metatile']) for e in design['endpoints']]
    walk={(x,y) for y in range(h*2) for x in range(w*2) if kind(x,y) in {'FLOOR','TALL_GRASS'} and (x,y) not in occupied}
    seen=flood(start,walk)
    assert end in seen,'Main road disconnected'
    widths=[]
    for probe in design['bench_width_probes']:
        x,lo,hi=probe['x'],probe['y_start'],probe['y_end']
        assert hi-lo==probe['walking_tiles']==4,'Routine carrier lane width must use actual core tiles'
        assert all(kind(x,y)=='FLOOR' for y in range(lo,hi)),'Routine bench lane interrupted'
        assert kind(x,lo-1) in {'WALL','UP_WALL'} and kind(x,hi) in {'WALL','UP_WALL'},'Routine bench lane widened beyond four walking tiles'
        widths.append(dict(id=probe['id'],actual_walking_width=hi-lo))
    reward_costs=[]
    for reward in design['optional_grass_rewards']:
        actor=next(a for a in design['items'] if a['id']==reward['actor_id'])
        x0,y0,x1,y1=reward['bounds_core'];band={(x,y) for y in range(y0,y1) for x in range(x0,x1)}
        assert all(kind(*p)=='TALL_GRASS' for p in band),'Reward threshold includes non-grass'
        costs={start:0};todo=deque([start])
        while todo:
            at=todo.popleft()
            for q in neighbors(at):
                if q not in walk:continue
                n=costs[at]+int(q in band)
                if n>=costs.get(q,10**9):continue
                costs[q]=n;(todo.append if q in band else todo.appendleft)(q)
        cost=min(costs.get(q,10**9) for q in neighbors(tuple(actor['tile'])))
        assert reward['minimum_grass_steps']<=cost<10**9,'Upper reward grass is freely bypassed or sealed'
        reward_costs.append(dict(actor_id=actor['id'],minimum_grass_steps=cost))
    stairs=layout['compiled_tileset']=='mn_r10_rockstairs'
    if stairs:
        assert design['id']=='MN_R10' and layout.get('tileset')=='johto','Exact R10 native stair alias'
        assert layout.get('derived_blocks')==[dict(id='80',source_tileset='cave',source_block='12',collision='FLOOR/FLOOR/WALL/FLOOR',required_typed_extension='r10_stairs_revision::revise_r10_rock_stairs')],'Exact cave12 right-lane recipe only'
        assert blocks[10*w+5]==0x80 and blocks[11*w+4]==0x0a,'Retained native stair and attached shoulder'
        assert [kind(10,20),kind(11,20),kind(10,21),kind(11,21)]==['FLOOR','FLOOR','WALL','FLOOR'],'Actual cave12 stair quadrants'
        assert all(kind(11,y)=='FLOOR' and (11,y)not in occupied for y in range(19,23)),'Native right lane must preserve both directions'
        source=pack['data']['tilesets']['johto'];native_stair=pack['data']['tilesets']['cave']['collision']['12']
        assert set(collision)==set(source['collision'])|{'80'} and len(source['collision'])==128,'Only one appended stair address'
        assert all(collision[key]==value for key,value in source['collision'].items()),'Original128 collision addresses changed'
        assert collision['80']==native_stair==['FLOOR','FLOOR','WALL','FLOOR'],'Native stair collision drift'
        files=pack['runtime_files'];original=bytes(files['data/tilesets/johto_metatiles.bin']);alias=bytes(files['data/tilesets/mn_r10_rockstairs_metatiles.bin'])
        assert len(original)==128*16 and len(alias)==129*16 and alias[:128*16]==original,'Original128 raw metatile addresses changed'
        alias_definition=pack['data']['tilesets']['mn_r10_rockstairs'];original_gfx=bytes(files['gfx/tilesets/johto.2bpp']);alias_gfx=bytes(files['gfx/tilesets/mn_r10_rockstairs.2bpp'])
        for glyph in set(original):
            palette=source['palette_map'][glyph]
            assert alias_definition['palette_map'][glyph]==palette,'Original used glyph palette changed'
            physical=glyph if palette&8==0 else (glyph&127)+96
            assert original_gfx[physical*16:(physical+1)*16]==alias_gfx[physical*16:(physical+1)*16],'Original physical glyph pixels changed'
        art=SourceArt(pack);legacy=art.blocks('johto');current=art.blocks('mn_r10_rockstairs')
        assert all(a.tobytes()==b.tobytes() for a,b in zip(legacy,current[:128])),'Original128 native block pixels changed'
        assert current[128].tobytes()==art.blocks('cave')[0x12].tobytes(),'Actual native stair pixels differ from exact source recipe'
    else:
        assert layout['compiled_tileset']=='johto' and not layout.get('derived_blocks'),'Legacy R10 must retain unextended Johto'
        assert [kind(10,20),kind(11,20),kind(10,21),kind(11,21)]==['WALL','FLOOR','FLOOR','FLOOR'],'Actual native 74 ramp quadrants'
    for ledge in design['one_way_ledges']:
        x0,y0,x1,y1=ledge['bounds_core']
        assert all(kind(x,y0)=='HOP_DOWN' and kind(x,y0+1)=='WALL' for x in range(x0,x1)),'True native ledge face'
        assert all(kind(*p)=='WALL' for p in ledge['side_barriers']),'Sideways ledge leak'
    assert not design['cut_gates'] and design['shortcut'] is None,'R10 has no useful required Cut target'
    assert not any(kind(x,y)=='CUT_TREE' for y in range(h*2) for x in range(w*2)),'Incidental decorative Cut quadrant'
    view=next(v for v in design['viewpoints'] if v['id']=='SUPERIOR_GLIMPSE');vx,vy=view['tile']
    water_visible=sum(kind(x,y)=='WATER' for y in range(vy-4,vy+5) for x in range(vx-5,vx+5))
    assert water_visible>=20,'Superior must appear in actual player camera'
    for left,right in [(27,34),(40,46)]:
        assert blocks[13*w+left]==0x48 and blocks[13*w+right]==0x4a,'Native rail caps'
        assert all(blocks[13*w+x]==0x49 for x in range(left+1,right)),'Continuous rail must use 49, not repeated corner 48'
        assert all(kind(x,27)=='WALL' for x in range(left*2,right*2+2)),'Public bank rail has a water gap'
    assert kind(73,29)=='FLOOR' and kind(73,30)=='WATER','Intentional clear Surf landing'
    perimeter={(x,y) for y in range(h*2) for x in range(w*2) if x in {0,w*2-1} or y in {0,h*2-1}}
    assert len(perimeter)==268,'Audit entire exterior perimeter'
    assert not any(kind(*p)=='WATER' for p in perimeter),'Unclosed water at outer map edge'
    surf_allowed=walk|{(x,y) for y in range(h*2) for x in range(w*2) if kind(x,y)=='WATER'}
    surf_seen=flood(start,surf_allowed);water={p for p in surf_seen if kind(*p)=='WATER'}
    assert water and not water&perimeter,'Reachable Surf water escapes outer boundary'
    for view in design['surf_viewpoints']:assert tuple(view['tile']) in water,'Unreachable Surf camera'
    for x in range(21,48):assert kind(x*2,39)==kind(x*2+1,39)=='BUOY','Southern native water-rock closure'
    for y in range(14,20):assert kind(95,y*2)==kind(95,y*2+1)=='BUOY','Eastern native water-rock closure'
    seam=layout['internal_water_neighbor'];assert seam['core_seam_x']==64 and seam['a']=='MnR10C00R00' and seam['b']=='MnR10C01R00','Retain actual two-chunk water seam'
    ys=[y for y in range(h*2) if kind(63,y)==kind(64,y)=='WATER'];assert ys,'Internal lake seam must stay open'
    native=all(k in pack['data']['maps'] for k in [seam['a'],seam['b']])
    if native:
        # Only final authored dimensions qualify as publication; old reservations do not.
        native=all(pack['data']['maps'][k]['attributes']['height']==20 for k in [seam['a'],seam['b']])
    if native:
        for a,b,d in [(seam['a'],seam['b'],'east'),(seam['b'],seam['a'],'west')]:
            assert dict(direction=d,target_map=b,offset=0) in pack['data']['maps'][a]['attributes']['connections'],'Water seam lacks reciprocal native neighbor'
    rocky=design.get('rocky_revision_contract')
    if rocky:
        assert stairs or (layout['compiled_tileset']=='johto' and not layout.get('derived_blocks')),'Rock revision retains original Johto plus only the bounded stair recipe'
        assert collision['0a']==['WALL']*4,'Joined ore erratics require native blocking collision'
        assert collision['71']==['FLOOR']*4 and collision['01']==['FLOOR']*4,'Rock apron and pale gravel use their native Floor'
        for x,y in [(7,5),(7,6),(8,6),(14,12),(25,7),(25,8),(26,8)]:
            assert blocks[y*w+x]==0x0a,'Authored attached bedrock-toe erratic seam missing'
        for x,y in [(8,4),(12,14),(21,12),(26,6),(33,9)]:
            assert blocks[y*w+x] in range(0x68,0x76),'Authored native contoured bedrock formation missing'
        assert blocks[5*w+9]==0x6f,'North carrier rib must retain native sculpted face'
        for x,y in [(5,11),(8,8),(12,8),(18,6)]:
            assert blocks[y*w+x]==0x01,'Pale gravel haul-road contrast missing'
    return dict(rocky_native_collision_contract_checked=bool(rocky),source_metatile_varieties_used=len(set(blocks)),routine_bench_lane_widths=widths,optional_grass_reward_costs=reward_costs,native_ramp_quadrants_checked=not stairs,native_right_stair_quadrants_and_original128_art_checked=stairs,decorative_cut_tiles=0,lookout_water_tiles_in_native_frame=water_visible,complete_outer_perimeter_tiles=len(perimeter),outer_buoy_tiles=sum(kind(*p)=='BUOY' for p in perimeter),reachable_surf_water_tiles=len(water),reachable_outer_water_tiles=0,open_internal_water_seam_core_y=ys,native_internal_water_neighbors_checked=native)

def verify_connected_published(design,layout,pack):
    """Certify the declared installed city mouths before ordinary event proof."""
    from render_progress_atlas import installed_route_mouths
    expected=layout_blocks(layout);connected=installed_route_mouths(design,expected,pack)
    differences=[(i%48,i//48,a,b)for i,(a,b)in enumerate(zip(expected,connected))if a!=b]
    if differences:
        assert differences==[(0,5,0x68,1),(47,10,2,5),(47,11,2,5),(47,12,2,1),(47,13,2,5)],'Exact five installed R10 seam patches required'
    adapted=dict(pack);adapted['data']=dict(pack['data']);adapted['data']['maps']=dict(pack['data']['maps'])
    for col,width in [(0,32),(1,16)]:
        name=f'MnR10C{col:02}R00';module=pack['data']['maps'][name]
        assert module['blocks']==[connected[y*48+col*32+x]for y in range(20)for x in range(width)],'Undeclared native terrain drift: '+name
        module=dict(module);module['blocks']=[expected[y*48+col*32+x]for y in range(20)for x in range(width)];adapted['data']['maps'][name]=module
    report=verify_published(design,layout,adapted);report['installed_city_seam_metatiles_checked']=len(differences)
    return report

def render(design,layout,pack,art):
    report=checked_render(design,layout,pack,art,output=OUTPUT)
    try:report.update(inspect_contracts(design,layout,pack))
    except AssertionError as error:report['errors'].append(str(error))
    scene=Image.open(OUTPUT/'whole-route.png');views=[]
    for view in design['surf_viewpoints']:
        x,y=view['tile'];im=render_stage(scene, 'crop', OUTPUT, 'review_iron_range_descent-render-line99-28-crop', (x*16-80,y*16-72,x*16+80,y*16+72));views.append((view['id']+' (Surf)',im));save_render(render_stage(im, 'resize', OUTPUT, 'review_iron_range_descent-render-line99-126-resize', (640,576),Image.Resampling.NEAREST), OUTPUT/('viewport-'+view['id'].lower()+'.png'))
    save_render(room_sheet(views,columns=2), OUTPUT/'surf-boundaries.png')
    report['status']='INVALID — see errors' if report['errors'] else 'Source geometry checked; visual review and native controller travel remain separate'
    (OUTPUT/'revision.json').write_text(json.dumps(report,indent=2)+'\n')
    return report

def negative_checks(design,layout,pack):
    """Reject meaningful regressions without changing either authoring file."""
    def point(l,at,block):l['layers'].append(dict(kind='point',position=at,block=block))
    cases=[
        ('unclosed east water',lambda d,l:point(l,[47,17],'35')),
        ('unclosed south water',lambda d,l:point(l,[34,19],'35')),
        ('rail corner repeated as straight',lambda d,l:point(l,[30,13],'48')),
        ('missing actual ramp',lambda d,l:point(l,[5,10],'71')),
        ('walkable ledge face',lambda d,l:point(l,[12,10],'71')),
        ('incidental Cut tree',lambda d,l:point(l,[6,8],'5b')),
        ('missing grass neck',lambda d,l:l['layers'].append(dict(kind='rect',material='forest',bounds=[25,9,28,11]))),
        ('trainer in canopy',lambda d,l:d['trainers'][3].update(tile=[85,23])),
        ('exit off ladder',lambda d,l:d['interiors'][0].update(exits=[[3,12]])),
        ('unregistered internal water neighbor',lambda d,l:l['internal_water_neighbor'].update(b='MissingNeighbor')),
        ('dry bypass into ore reward',lambda d,l:point(l,[11,5],'71')),
        ('routine bench widened beyond four tiles',lambda d,l:point(l,[8,9],'71')),
    ]
    if design.get('rocky_revision_contract'):
        cases += [
            ('missing attached ore toe',lambda d,l:point(l,[14,12],'05')),
            ('flattened bedrock rib',lambda d,l:point(l,[9,5],'71')),
            ('erased gravel haul-road contrast',lambda d,l:point(l,[8,8],'71')),
        ]
    if layout['compiled_tileset']=='mn_r10_rockstairs':
        cases += [
            ('wrong native stair source',lambda d,l:l['derived_blocks'][0].update(source_block='30')),
            ('transformed native stair recipe',lambda d,l:l['derived_blocks'][0].update(tile_row_order=[3,2,1,0])),
            ('palette override on native stair',lambda d,l:l['derived_blocks'][0].update(palette_override=7)),
            ('missing attached stair shoulder',lambda d,l:point(l,[4,11],'71')),
        ]
    for label,mutate in cases:
        d,l=deepcopy(design),deepcopy(layout);mutate(d,l);rejected=False
        try:
            report,_,_=check(d,l,pack,layout_blocks(l))
            rejected=bool(report['errors'])
            inspect_contracts(d,l,pack)
        except AssertionError:rejected=True
        assert rejected,'Negative fixture accepted: '+label
    return [label for label,_ in cases]

def main():
    parser=argparse.ArgumentParser();parser.add_argument('--pack',type=Path,default=ROOT/'content-packs/text-tui.crystalpack');parser.add_argument('--watch',action='store_true');parser.add_argument('--serve',action='store_true');parser.add_argument('--port',type=int,default=8782);parser.add_argument('--verify-published',action='store_true');parser.add_argument('--self-test',action='store_true');args=parser.parse_args()
    pack=load_pack(args.pack);art=SourceArt(pack);state={'ok':False,'error':'Initial render pending'}
    if args.serve:
        OUTPUT.mkdir(parents=True,exist_ok=True);(OUTPUT/'index.html').write_text(PAGE)
        class Handler(SimpleHTTPRequestHandler):
            def log_message(self,*args):pass
            def do_GET(self):
                if self.path.split('?')[0]=='/state':
                    body=json.dumps(state).encode();self.send_response(200);self.send_header('Content-Type','application/json');self.send_header('Cache-Control','no-store');self.send_header('Content-Length',str(len(body)));self.end_headers();self.wfile.write(body)
                else:super().do_GET()
        server=ThreadingHTTPServer(('127.0.0.1',args.port),partial(Handler,directory=str(OUTPUT)));threading.Thread(target=server.serve_forever,daemon=True).start();print(f'Iron Range review: http://127.0.0.1:{args.port}/',flush=True)
    last=None
    while True:
        revision=tuple(p.stat().st_mtime_ns for p in [DOSSIER,LAYOUT,Path(__file__)])
        if revision!=last:
            try:
                d,l=json.loads(DOSSIER.read_text()),json.loads(LAYOUT.read_text());report=render(d,l,pack,art)
                if args.self_test:report['negative_cases_rejected']=negative_checks(d,l,pack)
                if args.verify_published:report.update(verify_connected_published(d,l,pack))
                state.update(ok=not report['errors'],error='; '.join(report['errors']),revision=str(time.time_ns()),**report);print(json.dumps(report),flush=True)
            except (AssertionError,KeyError,ValueError,IndexError,TypeError) as error:
                state.update(ok=False,error=str(error));print('DESIGN ERROR: '+str(error),flush=True)
            if not state['ok'] and not(args.watch or args.serve):raise SystemExit(1)
            last=revision
        if not(args.watch or args.serve):break
        time.sleep(.15)

if __name__=='__main__':main()
