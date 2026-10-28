#!/usr/bin/env python3
"""Whole North Shore source review; preserves the shared native sprite helper."""
from render_archive import save_render
from render_stages import render_stage
import argparse
from copy import deepcopy
from functools import partial
import hashlib
from http.server import SimpleHTTPRequestHandler,ThreadingHTTPServer
import io
import json
from pathlib import Path
import threading
import time
from PIL import Image,ImageDraw
from render_reference import SourceArt,layout_blocks,load_pack,room_sheet
from review_grand_marais import sprite

def oriented_sprite(pack,token,facing):
    offset=16 if facing=='north' else 32 if facing in {'west','east'} else 0
    art=sprite(pack,token,frame_offset=offset)
    return art.transpose(Image.Transpose.FLIP_LEFT_RIGHT) if facing=='east' else art

ROOT=Path(__file__).resolve().parents[3];LAYOUT=ROOT/'planning/minnesota/routes/MN_R11.layout.json';DOSSIER=ROOT/'planning/minnesota/routes/MN_R11.json';OUTPUT=ROOT/'planning/minnesota/renders/north-shore'

def check_river_catchments(design,layout,kind,ordinary):
    """Check authored source drainage; this does not dispatch native HM input.

    The hydrological graph includes native06 decks as visually spanned water.
    The separate native-water graph never pretends Surf passes under LADDER.
    """
    from collections import deque
    from review_gunflint import flood,neighbors
    w,h=layout['dimensions_metatiles']
    water={(x,y) for y in range(h*2) for x in range(w*2) if kind(x,y) in {'WATER','WATERFALL'}}
    drainage=water|{(x,y) for y in range(h*2) for x in range(w*2) if kind(x,y)=='LADDER'}
    ground={(x,y) for y in range(h*2) for x in range(w*2) if kind(x,y) in {'FLOOR','TALL_GRASS','LADDER'}}
    expected={(10,y) for y in range(33,36)}
    proofs=[]
    for catchment in design['river_catchments']:
        name=catchment['id'];fall=catchment['cascade_blocks'];expected.update(map(tuple,fall))
        head,base,view,sea=[tuple(catchment[key]) for key in ['head_water_core','base_water_core','viewpoint_core','sea_join_core']]
        assert kind(*head)==kind(*base)==kind(*sea)=='WATER',name+': source pool, plunge pool and sea mouth must be ordinary WATER'
        assert view in ordinary,name+': fall view needs an actual walking approach'
        x,y=view;box=(x*16-80,y*16-72,x*16+80,y*16+72)
        for fx,fy in fall:
            assert all(kind(fx*2+dx,fy*2+dy)=='WATERFALL' for dx in [0,1] for dy in [0,1]),name+': native cascade face missing'
            assert box[0]<=fx*32 and box[1]<=fy*32 and box[2]>=(fx+1)*32 and box[3]>=(fy+1)*32,name+': complete cascade outside walking camera'
        connected=flood(head,water)
        assert base in connected,name+': cascade and plunge water are disconnected'
        assert sea in flood(head,drainage),name+': creek mouth disconnected from Superior'
        for place in ['base','upper']:
            bank_at,launch_at=[tuple(catchment[place+key]) for key in ['_surf_bank_core','_surf_launch_core']]
            assert kind(*bank_at)=='FLOOR' and kind(*launch_at)=='WATER' and launch_at in connected,name+': ordinary-WATER bank/launch anchor changed'
            assert sum(abs(a-b) for a,b in zip(bank_at,launch_at))==1,name+': Surf launch is not beside its bank'
            if place=='base':assert bank_at in ordinary,name+': base Surf bank needs ordinary walking access'
        for stage in catchment['climb_stages']:
            assert all(kind(*stage[key])=='WATER' and tuple(stage[key]) in connected for key in ['base_water','upper_water']),name+': cascade stage needs real ordinary-water landings'
        for key in ['north_bank_core','south_bank_core']:
            assert all(tuple(p) in ordinary and kind(*p)=='FLOOR' for p in catchment[key]),name+': both bridge-bank lanes must remain clear Floor'
        if 'intermediate_water_core' in catchment:
            middle=tuple(catchment['intermediate_water_core'])
            assert kind(*middle)=='WATER' and middle in connected,name+': two-stage drop needs real ordinary-water plunge pool'
        grass_cost=None
        if 'bank_encounter_choice' in catchment:
            choice=catchment['bank_encounter_choice'];origin,target=[tuple(choice[key]) for key in ['from_core','view_core']]
            costs={origin:0};todo=deque([origin])
            while todo:
                at=todo.popleft()
                for q in neighbors(at):
                    if q not in ordinary:continue
                    step=int(kind(*q)=='TALL_GRASS');value=costs[at]+step
                    if value<costs.get(q,10**9):
                        costs[q]=value
                        (todo.append if step else todo.appendleft)(q)
            grass_cost=costs.get(target)
            assert grass_cost is not None and grass_cost>=choice['minimum_grass_steps'],name+': optional sedge-bank view has a free dry bypass'
        bank={p for q in connected for p in neighbors(q) if p in ground}
        assert bank,name+': headwater has no native dismount bank'
        remaining=set(bank);components=0
        while remaining:
            land=flood(min(remaining),ground);remaining-=land;components+=1
            assert any(kind(*q)=='WATER' for p in land for q in neighbors(p) if 0<=q[0]<w*2 and 0<=q[1]<h*2),name+': bank component has no ordinary-WATER Surf relaunch'
        proofs.append(dict(id=name,native_cascade_blocks=fall,head_to_base_water_connected=True,ordinary_water_bank_components=components,complete_falls_in_walking_frame=True,optional_bank_minimum_grass_steps=grass_cost,sea_drainage_includes_visual_native06_deck=True,native_surf_under_deck_proved=False,extra_HM_controller_proved=False))
    actual={(x,y) for y in range(h) for x in range(w) if kind(x*2,y*2)=='WATERFALL'}
    assert actual==expected,'Every placed native waterfall must belong to a named source drainage'
    return dict(river_catchments=proofs,native_cascade_metatiles=len(actual))

def check(design,layout,pack,blocks):
    from review_league_approach import inspect_league as common_check
    from review_gunflint import flood,neighbors
    report,kind,seen=common_check(design,layout,pack,blocks)
    w,h=layout['dimensions_metatiles'];actors=design['trainers']+design['residents']+design['items'];occupied={tuple(a['tile']) for a in actors}
    start,end=[tuple(2*v for v in e['gate_metatile']) for e in design['endpoints']]
    walking={(x,y) for y in range(h*2) for x in range(w*2) if kind(x,y) in {'FLOOR','LADDER','TALL_GRASS'} and (x,y) not in occupied}
    ordinary=flood(start,walking)
    assert end in ordinary,'Both ends must connect with actual native LADDER bridges, without field moves'
    assert (w,h)==(32,72) and design['required_flags']==[f'BADGE_{i}' for i in range(1,9)]+['EVENT_MN_LIGHTHOUSE_SIGNAL'],'Retain full Q07 late-route requirements'
    assert len(design['trainers'])==10 and len(design['residents'])==5 and len(design['items'])==4,'Preserve native actor/party/resource identities'
    assert design['cut_gates']==[] and design['shortcut'] is None,'No decorative Cut gate'
    assert not any(kind(x,y)=='CUT_TREE' for y in range(h*2) for x in range(w*2)),'Undeclared native Cut quadrant'
    crossings=[]
    for crossing in layout['crossings']:
        cells={(x*2+dx,y*2+dy) for x,y in crossing['blocks'] for dx in [0,1] for dy in [0,1]}
        assert all(kind(*at)=='LADDER' and at in ordinary for at in cells),'Native06crossing must retain all four authentic LADDER quadrants'
        if crossing['necessary']:assert end not in flood(start,walking-cells),'Gorge has an unintended dry crossing'
        crossings.append(dict(id=crossing['id'],source_block='06',walking_collision='LADDER',necessary=crossing['necessary'],core_cells=len(cells)))
    assert [blocks[y*w+10] for y in range(33,36)]==[0x80]*3,'Preserve authentic three-block waterfall face'
    summit=design['waterfall_summit'];arrival,bank,launch,return_at,base=[tuple(summit[key]) for key in ['water_arrival','safe_bank','surf_launch','cascade_return','base_water']]
    assert (arrival,bank,launch,return_at,base)==((20,65),(20,61),(20,62),(20,66),(20,72)),'Keep bounded summit return coordinates'
    assert kind(*arrival)==kind(*launch)=='WATER' and kind(*bank)=='FLOOR','Summit must have ordinary WATER arrival/relaunch beside a real bank'
    assert 0x29 not in pack['data']['field_moves']['surf']['blocked_collisions'],'Actual native Surf rule must permit ordinary WATER mounting'
    summit_land=flood(bank,walking);safe_launches={q for at in summit_land for q in neighbors(at) if 0<=q[0]<w*2 and 0<=q[1]<h*2 and kind(*q)=='WATER'}
    assert launch in safe_launches,'Dismounting anywhere on the connected summit bank must retain a native ordinary WATER relaunch'
    summit_water=flood(launch,{(x,y) for y in range(h*2) for x in range(w*2) if kind(x,y) in {'WATER','WATERFALL'}})
    assert arrival in summit_water and return_at in summit_water and base in summit_water,'Summit spring must connect back down the actual waterfall to the gorge water'
    assert all(kind(20,y)=='WATERFALL' for y in range(66,72)),'Summit return must retain the native downward cascade'

    view=next(v for v in design['viewpoints'] if v['id']=='WATERFALL_BANK');vx,vy=view['tile']
    assert vx*16-80<=10*32 and vx*16+80>=11*32 and vy*16-72<=33*32 and vy*16+72>=36*32,'Full source waterfall must fit the actual160x144frame'
    lake_views=[]
    for name in ['BASALT_LOOKOUT','SPLIT_ROCK_LOOKOUT']:
        vx,vy=next(v['tile'] for v in design['viewpoints'] if v['id']==name);water=sum(kind(x,y)=='WATER' for y in range(vy-4,vy+5) for x in range(vx-5,vx+5));assert water>=15,name+': lake outside advertised camera';lake_views.append(dict(id=name,visible_water_core_tiles=water))
    door=design['doors'][0];assert door['tile']==[51,99] and kind(*door['tile'])=='DOOR','Actual source lighthouse door quadrant'
    assert door['arrival_tile']==door['return_tile']==[51,100] and kind(51,100)=='FLOOR' and door['return_warp_index']==2,'Safe separate exterior passive arrival'
    room=design['interiors'][0];assert room['id']=='MnR11SplitRockLighthouse' and room['dimensions_metatiles']==[6,5] and room['exits']==[[4,9],[5,9]],'Preserve actual native room identity/carpet exits'
    assert room['residents'][0]['id']=='MnR11Keeper' and room['residents'][0]['tile']==[7,3],'Preserve native keeper identity/footing'
    allowed=walking|{(x,y) for y in range(h*2) for x in range(w*2) if kind(x,y) in {'WATER','WATERFALL'}};surf=flood(start,allowed);water={at for at in surf if kind(*at) in {'WATER','WATERFALL'}}
    perimeter={(x,y) for y in range(h*2) for x in range(w*2) if x in {0,w*2-1} or y in {0,h*2-1}}
    assert len(perimeter)==412 and not any(kind(*at) in {'WATER','WATERFALL'} for at in perimeter),'Open water escapes a complete unconnected map perimeter'
    assert water and not water&perimeter,'Audit must actually reach sea/gorge/Waterfall surfaces'
    for x in range(48,64):assert kind(x,0)==kind(x,143)=='BUOY','North/south native water rocks'
    for y in range(144):assert kind(63,y)=='BUOY','East native water rocks'
    seams=[]
    for row in [1,2]:
        y=row*48;xs=[x for x in range(w*2) if kind(x,y-1)==kind(x,y)=='WATER'];assert xs,'Preserve actual open internal sea seam';seams.append(dict(north=f'MnR11C00R{row-1:02}',south=f'MnR11C00R{row:02}',core_boundary_y=y,water_core_x=xs))
    native=all(name in pack['data']['maps'] for name in ['MnR11C00R00','MnR11C00R01','MnR11C00R02'])
    if native:
        native=all(pack['data']['maps'][name]['attributes']['width']==32 for name in ['MnR11C00R00','MnR11C00R01','MnR11C00R02'])
        for seam in seams:
            for a,b,d in [(seam['north'],seam['south'],'south'),(seam['south'],seam['north'],'north')]:assert dict(direction=d,target_map=b,offset=0) in pack['data']['maps'][a]['attributes']['connections'],'Open internal sea requires real reciprocal native neighbor'
    for view in design['surf_viewpoints']:assert tuple(view['tile']) in water,'Surf boundary camera not reachable'
    report.update(check_river_catchments(design,layout,kind,ordinary))
    report.update(waterfall_summit_safe_bank=list(bank),waterfall_summit_surf_launch=list(launch),waterfall_summit_bank_cells=len(summit_land),waterfall_summit_native_relaunches=len(safe_launches),crossings=crossings,source_metatile_varieties_used=len(set(blocks)),lake_view_proofs=lake_views,full_waterfall_in_frame=True,native_cut_quadrants=0,complete_outer_perimeter_core_tiles=len(perimeter),outer_buoy_core_tiles=sum(kind(*at)=='BUOY' for at in perimeter),reachable_surf_waterfall_tiles=len(water),reachable_outer_water_tiles=0,internal_water_seams=seams,internal_neighbors_present_in_supplied_pack=native)
    return report,kind,seen


def verify_published(design,layout,pack):
    from review_league_approach import verify_published as common_verify
    return common_verify(design,layout,pack)


def render(design,layout,pack,art):
    from review_gunflint import flood
    started=time.perf_counter();blocks=layout_blocks(layout);w,h=layout['dimensions_metatiles']
    try:report,kind,seen=check(design,layout,pack,blocks)
    except AssertionError as error:
        from review_league_approach import check as common_check
        report,kind,seen=common_check(design,layout,pack,blocks);report['errors'].append(str(error))
    original,published=art.blocks('johto'),art.blocks('mn_north_shore');assert len(published)==len(original)+1
    assert all(a.tobytes()==b.tobytes() for a,b in zip(original,published)),'Original Johto source pixels changed'
    assert published[128].tobytes()==art.blocks('cave')[0x2c].tobytes(),'Actual waterfall source differs'
    terrain=art.render('mn_north_shore',w,h,blocks);scene=render_stage(terrain, 'convert', OUTPUT, 'review_north_shore-render-line109-58-convert', 'RGBA')
    for a in design['trainers']+design['residents']:scene.alpha_composite(oriented_sprite(pack,a['sprite'],a['facing']),(a['tile'][0]*16,a['tile'][1]*16-4))
    for a in design['items']:scene.alpha_composite(sprite(pack,'SPRITE_POKE_BALL'),(a['tile'][0]*16,a['tile'][1]*16-4))
    OUTPUT.mkdir(parents=True,exist_ok=True);save_render(terrain, OUTPUT/'terrain.png');save_render(render_stage(scene, 'convert', OUTPUT, 'review_north_shore-render-line112-100-convert', 'RGB'), OUTPUT/'whole-route.png');digest=hashlib.sha256(scene.tobytes()).hexdigest()[:8];save_render(render_stage(scene, 'convert', OUTPUT, 'review_north_shore-render-line112-215-convert', 'RGB'), OUTPUT/f'whole-route-{digest}.png');save_render(render_stage(scene, 'resize', OUTPUT, 'review_north_shore-render-line112-285-resize', (512,1152),Image.Resampling.NEAREST), OUTPUT/'overview.png')
    overlay=scene.copy();draw=ImageDraw.Draw(overlay)
    for row in range(design['chunk_rows']):draw.rectangle((0,row*768,1023,(row+1)*768-1),outline='white',width=1);draw.text((8,row*768+8),f'MnR11C00R{row:02}',fill='white',stroke_width=1,stroke_fill='black')
    for a in design['trainers']+design['residents']+design['items']+design['signs']+design['doors']+design['viewpoints']:
        x,y=a['tile'];draw.rectangle((x*16,y*16,x*16+15,y*16+15),outline='#ffd250',width=2);draw.text((x*16,y*16-12),a['id'].removeprefix('MN_R11_'),fill='white',stroke_width=1,stroke_fill='black')
    save_render(overlay, OUTPUT/'anchors.png')
    views=[]
    for v in design['viewpoints']+design['surf_viewpoints']:
        x,y=v['tile'];im=render_stage(render_stage(scene, 'crop', OUTPUT, 'review_north_shore-render-line120-25-crop', (x*16-80,y*16-72,x*16+80,y*16+72)), 'convert', OUTPUT, 'review_north_shore-render-line120-25-convert', 'RGB');views.append((v['id'],im));save_render(render_stage(im, 'resize', OUTPUT, 'review_north_shore-render-line120-125-resize', (640,576),Image.Resampling.NEAREST), OUTPUT/('viewport-'+v['id'].lower()+'.png'))
    save_render(room_sheet(views[:len(design['viewpoints'])],columns=2), OUTPUT/'viewpoints.png');save_render(room_sheet(views[len(design['viewpoints']):],columns=2), OUTPUT/'surf-boundaries.png')
    rooms=[]
    for room in design['interiors']:
        template=pack['data']['maps'][room['template']]['attributes'];im=render_stage(art.render(template['tileset_name'],*room['dimensions_metatiles'],[int(b,16) for row in room['rows'] for b in row.split()]), 'convert', OUTPUT, 'review_north_shore-render-line124-73-convert', 'RGBA')
        for a in room['residents']:im.alpha_composite(oriented_sprite(pack,a['sprite'],a['facing']),(a['tile'][0]*16,a['tile'][1]*16-4))
        rooms.append((room['id'],im));save_render(render_stage(im, 'resize', OUTPUT, 'review_north_shore-render-line126-50-resize', (im.width*4,im.height*4),Image.Resampling.NEAREST), OUTPUT/'lighthouse-room.png')
    save_render(room_sheet(rooms,columns=1), OUTPUT/'interiors.png')
    occupied={tuple(v//2 for v in a['tile']) for key in ['trainers','residents','items'] for a in design[key]};sizes={};largest=(0,0,0)
    for y in range(h):
        for x in range(w):
            size=1+min(sizes.get((x-1,y),0),sizes.get((x,y-1),0),sizes.get((x-1,y-1),0)) if blocks[y*w+x] in {1,2,0x71} and (x,y) not in occupied else 0;sizes[x,y]=size
            if size>largest[0]:largest=(size,x-size+1,y-size+1)
    size,x,y=largest;cx,cy=x*2+size,y*2+size;save_render(render_stage(render_stage(scene, 'crop', OUTPUT, 'review_north_shore-render-line133-57-crop', (cx*16-80,cy*16-72,cx*16+80,cy*16+72)), 'resize', OUTPUT, 'review_north_shore-render-line133-57-resize', (640,576),Image.Resampling.NEAREST), OUTPUT/'viewport-emptiest_ground.png')
    report.update(status='INVALID — see errors' if report['errors'] else 'Source geometry checked; consult the route runbook for current native publication, controller and earned progression evidence',largest_blank_ground_square_metatiles=size,emptiest_ground_camera_core=[cx,cy],original_johto_blocks_preserved=128,native_waterfall_source_pixels_checked=True,render_ms=round((time.perf_counter()-started)*1000,1),image=f'whole-route-{digest}.png');(OUTPUT/'revision.json').write_text(json.dumps(report,indent=2)+'\n');return report


def self_test(design,layout,pack):
    """Reject meaningful route regressions without rewriting source or packs."""
    cases=[]
    def blocks_case(name,x,y,value):
        def mutate(d,l,b):b[y*l['dimensions_metatiles'][0]+x]=value
        cases.append((name,mutate))
    blocks_case('missing necessary bridge',20,37,0x35)
    blocks_case('fake FLOOR bridge loses native LADDER',20,37,0x01)
    def bypass(d,l,b):
        for y in range(34,41):b[y*32+19]=1
    cases.append(('dry bypass around necessary gorge bridge',bypass))
    blocks_case('waterfall face removed',10,34,0x35)
    blocks_case('Baptism native cascade erased',9,11,0x35)
    blocks_case('Temperance intermediate plunge replaced by land',9,51,0x05)
    blocks_case('Baptism bridge ladder quadrant erased',20,19,0x35)
    blocks_case('Temperance bridge faked with ordinary Floor',20,54,0x01)
    def erase_bank_sedge(d,l,b):
        for y in range(52,54):
            for x in range(14,16):b[y*32+x]=0x02
    cases.append(('Temperance optional bank loses encounter choice',erase_bank_sedge))
    def block_new_creek_mouth(d,l,b):
        for y in range(18,22):b[y*32+22]=0x05
    cases.append(('Baptism outlet disconnected from Superior',block_new_creek_mouth))
    def erase_summit_spring(d,l,b):
        for y in range(31,33):
            for x in range(9,12):b[y*32+x]=0x71
    cases.append(('old isolated dry waterfall summit trap',erase_summit_spring))
    blocks_case('summit bank buried under canopy',10,30,0x05)

    blocks_case('open east sea boundary',31,40,0x35)
    blocks_case('open north sea boundary',27,0,0x35)
    blocks_case('open south sea boundary',27,71,0x35)
    blocks_case('brown land boulder substituted for native sea rock',31,40,0x0a)
    blocks_case('unintentional decorative Cut canopy',15,5,0x5b)
    blocks_case('required grass replaced by free dry land',17,64,0x01)
    blocks_case('source lighthouse entrance removed',25,49,0x01)
    cases.append(('waterfall advertised outside player camera',lambda d,l,b:d['viewpoints'][3].update(tile=[41,67])))
    cases.append(('unsafe exterior room arrival',lambda d,l,b:d['doors'][0].update(arrival_tile=[51,99])))
    cases.append(('keeper identity changed',lambda d,l,b:d['interiors'][0]['residents'][0].update(id='DuplicateKeeper')))
    cases.append(('actor overlap',lambda d,l,b:d['trainers'][0].update(tile=d['residents'][0]['tile'])))
    cases.append(('Q07 receiver gate removed',lambda d,l,b:d['required_flags'].remove('EVENT_MN_LIGHTHOUSE_SIGNAL')))
    report,_,_=check(design,layout,pack,layout_blocks(layout));assert not report['errors'],report['errors']
    rejected=[]
    for name,mutate in cases:
        d,l=deepcopy(design),deepcopy(layout);b=layout_blocks(l);mutate(d,l,b)
        try:
            report,_,_=check(d,l,pack,b)
            assert not report['errors'],'; '.join(report['errors'])
        except AssertionError:rejected.append(name)
        else:raise AssertionError('Negative fixture unexpectedly accepted: '+name)
    function_text=Path(__file__).read_text().split('def oriented_sprite',1)[1].split('\nROOT=',1)[0]
    signature=hashlib.sha256(('def oriented_sprite'+function_text).strip().encode()).hexdigest()
    # Exact function body is a public dependency of the other route reviewers.
    assert signature=='a89d4250cf58a3ef6aec822b30587bf6305d4bc1a0d0b1904e4326a61b3401da',signature
    return dict(negative_cases_rejected=len(rejected),cases=rejected,shared_oriented_sprite_sha256=signature)


HTML='''<!doctype html><meta charset="utf-8"><title>North Shore review</title><style>body{background:#14212b;color:#e7eff2;font:16px system-ui;margin:20px}img{image-rendering:pixelated}header{position:sticky;top:0;background:#14212bf5;padding:12px}button{padding:10px;margin:4px}.scroll{overflow:auto;max-height:78vh}#map{width:1024px}</style><header><h1>North Shore Trail — complete three-map coast</h1><button data-layer="whole-route">Residents</button><button data-layer="terrain">Terrain</button><button data-layer="anchors">Anchors</button><button id="zoom">Half/native pixels</button><p id="status">Checking…</p></header><p>Native source-art review. Q07 gate, native publication and production controller acceptance remain separate.</p><div class="scroll"><img id="map"></div><h2>160×144 framing</h2><img id="views"><h2>Actual emptiest-ground camera</h2><img id="empty"><h2>Complete keeper room</h2><img id="rooms"><h2>Surf boundary probes</h2><img id="surf"><script>let layer='whole-route',rev='',native=true;function refresh(){document.querySelector('#map').src=layer+'.png?r='+rev;for(const[id,file]of[['views','viewpoints'],['rooms','interiors'],['surf','surf-boundaries'],['empty','viewport-emptiest_ground']])document.querySelector('#'+id).src=file+'.png?r='+rev}document.querySelectorAll('[data-layer]').forEach(b=>b.onclick=()=>{layer=b.dataset.layer;refresh()});document.querySelector('#zoom').onclick=()=>{native=!native;document.querySelector('#map').style.width=(native?1024:512)+'px'};setInterval(async()=>{const s=await(await fetch('/state',{cache:'no-store'})).json();document.querySelector('#status').textContent=s.ok?s.render_ms+'ms · '+s.status:'INVALID: '+s.error;if(rev!==s.revision){rev=s.revision;refresh()}},400)</script>'''


def main():
    parser=argparse.ArgumentParser();parser.add_argument('--pack',type=Path,default=ROOT/'content-packs/minnesota-north-shore-art.crystalpack');parser.add_argument('--watch',action='store_true');parser.add_argument('--serve',action='store_true');parser.add_argument('--port',type=int,default=8790);parser.add_argument('--verify-published',action='store_true');parser.add_argument('--self-test',action='store_true');args=parser.parse_args();pack=load_pack(args.pack);art=SourceArt(pack);state={'ok':False,'error':'Initial source render pending'}
    if args.self_test:
        print(json.dumps(self_test(json.loads(DOSSIER.read_text()),json.loads(LAYOUT.read_text()),pack)));return
    if args.serve:
        OUTPUT.mkdir(parents=True,exist_ok=True);(OUTPUT/'index.html').write_text(HTML)
        class Handler(SimpleHTTPRequestHandler):
            def log_message(self,*args):pass
            def do_GET(self):
                if self.path.split('?')[0]=='/state':
                    body=json.dumps(state).encode();self.send_response(200);self.send_header('Content-Type','application/json');self.send_header('Cache-Control','no-store');self.send_header('Content-Length',str(len(body)));self.end_headers();self.wfile.write(body)
                else:super().do_GET()
        server=ThreadingHTTPServer(('127.0.0.1',args.port),partial(Handler,directory=str(OUTPUT)));threading.Thread(target=server.serve_forever,daemon=True).start();print(f'North Shore source review: http://127.0.0.1:{args.port}/',flush=True)
    last=None
    while True:
        revision=tuple(p.stat().st_mtime_ns for p in [DOSSIER,LAYOUT])
        if revision!=last:
            try:
                d,l=json.loads(DOSSIER.read_text()),json.loads(LAYOUT.read_text());r=render(d,l,pack,art)
                if args.verify_published:r.update(verify_published(d,l,pack))
                state.update(ok=not r['errors'],error='; '.join(r['errors']),revision=str(time.time_ns()),**r);print(json.dumps(r),flush=True)
            except (AssertionError,ValueError,KeyError,IndexError,TypeError) as error:state.update(ok=False,error=str(error));print('DESIGN ERROR: '+str(error),flush=True)
            if not state['ok'] and not(args.watch or args.serve):raise SystemExit(1)
            last=revision
        if not(args.watch or args.serve):break
        time.sleep(.15)
if __name__=='__main__':main()
