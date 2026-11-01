#!/usr/bin/env python3
"""Read-only source review for R18 marina, shore-watch walks and Surf islet."""
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
from review_league_approach import HTML,render as render_checked,check,verify_published
from review_headwaters import prepare
from review_gunflint import flood,neighbors
from review_grand_marais import sprite
ROOT=Path(__file__).resolve().parents[3]
DOSSIER=ROOT/'planning/minnesota/routes/MN_R18.json';LAYOUT=ROOT/'planning/minnesota/routes/MN_R18.layout.json';OUTPUT=ROOT/'planning/minnesota/renders/mille-lacs-shore'
PAGE=HTML.replace('League Approach','Mille Lacs Shore').replace('all eighteen chunks','marina, meadow and lake').replace('Border Lake · Wind Tor · Granite switchback · Dry saddle · Cedar Shelter · Long Lake · Old quarry · Ely arrival.','Two marina piers · Three walking shore-watch stops · Meadow habitat · Public launch · Optional Surf islet.').replace('<h2>Cedar Shelter</h2><img id="rooms" src="interiors.png">','<h2>Surf lake/islet probes</h2><img id="surf" src="surf-boundaries.png">').replace("if(room)room.src='interiors.png?r='+rev","if(room)room.src='interiors.png?r='+rev;document.querySelector('#surf').src='surf-boundaries.png?r='+rev").replace('width:1536px','width:1280px').replace('(native?3072:1536)','(native?1280:640)')

def walking_design(d):
    ordinary=deepcopy(d);surf_ids=set(d['surf_only_actor_ids']);ordinary['items']=[a for a in d['items'] if a['id'] not in surf_ids];return ordinary

def check_lake(d,l,p):
    prepare(p,l);blocks=layout_blocks(l);w,h=l['dimensions_metatiles'];c=p['data']['tilesets'][l['compiled_tileset']]['collision']
    def kind(x,y):return c[f'{blocks[y//2*w+x//2]:02x}'][y%2*2+x%2]
    assert (w,h)==(40,24) and d['required_flags']==['BADGE_6'],'Retain exact authored shore and story gate'
    actors=d['trainers']+d['residents']+d['items'];occupied={tuple(a['tile']) for a in actors}
    start,end=[tuple(2*v for v in e['gate_metatile']) for e in d['endpoints']]
    walk={(x,y) for y in range(h*2) for x in range(w*2) if kind(x,y) in {'FLOOR','TALL_GRASS'} and (x,y) not in occupied}
    seen=flood(start,walk);surf_allowed=walk|{(x,y) for y in range(h*2) for x in range(w*2) if kind(x,y)=='WATER'};surf=flood(start,surf_allowed)
    assert end in seen,'Walking-only main route disconnected'
    widths=[]
    for probe in d['routine_width_probes']:
        x,lo,hi=probe['x'],probe['y_start'],probe['y_end'];assert hi-lo==probe['walking_tiles']==4,'Four actual core tiles, not metatiles'
        assert all(kind(x,y)=='FLOOR' for y in range(lo,hi)) and kind(x,lo-1)==kind(x,hi)=='WALL','Routine shore has become a wide empty shoulder'
        widths.append(dict(id=probe['id'],walking_tiles=hi-lo))
    assert d['cut_gates']==[] and d['one_way_ledges']==[] and d['shortcut'] is None,'Flat shore is not a copied ridge/Cut sequence'
    assert not any(kind(x,y)=='CUT_TREE' for y in range(h*2) for x in range(w*2)),'Incidental native Cut quadrant'
    q=d['quest_contract'];assert q['id']=='SQ04' and q['reward_item']=='GOOD_ROD' and q['reward_item'] in p['data']['items'],'Use actual existing rod reward'
    assert p['data']['story_event_script_constants']['global']['ENGINE_MINERALBADGE']==q['badge_check']['source_constant_value']==31,'Use actual native sixth-badge constant'
    assert all(0<len(line)<=18 for lines in q['text_labels'].values() for line in lines),'Quest text exceeds native eighteen-column width'
    assert all(v['actor_id'] in q['script_labels'] and any(c['command']=='setevent' and c['args']==[v['flag']] for c in q['script_labels'][v['actor_id']]) for v in q['volunteers']),'Quest source labels must set each explicit report event'
    quest_ids={a['actor_id'] for a in q['volunteers']}|{q['ranger_id']};assert len(quest_ids)==4,'Exactly three volunteers and one dock ranger'
    for a in d['residents']:
        assert a['id'] in quest_ids and set(neighbors(tuple(a['tile'])))&seen,'Quest actor must be reachable without Surf'
    assert q['reward_flag']=='EVENT_MN_SHORE_WATCH_REWARDED','Preserve SQ04 completion identity'
    for a in d['items']:
        approaches=set(neighbors(tuple(a['tile'])));assert kind(*a['tile'])=='FLOOR' and a['item'] in p['data']['items'],'Native item anchor/catalog'
        if a['id'] in d['surf_only_actor_ids']:
            assert not approaches&seen,'Optional offshore reward has dry access'
            assert approaches&surf,'Optional offshore reward is unreachable with Surf'
        else:assert approaches&seen,'Ordinary reward accidentally requires Surf'
    for pier in l['piers']:
        for x,y in pier['deck_blocks']:assert all(kind(2*x+dx,2*y+dy)=='FLOOR' for dy in [0,1] for dx in [0,1]),'Pier deck is not ordinary Floor'
        for x,y in pier['north_rail_blocks']:assert [kind(2*x,y*2),kind(2*x,y*2+1)]==['WALL','FLOOR'],'Actual north timber rail'
        for x,y in pier['south_rail_blocks']:assert [kind(2*x,y*2),kind(2*x,y*2+1)]==['FLOOR','WALL'],'Actual south timber rail'
        for x,y in pier['water_under_rail_blocks']:assert kind(x*2,y*2)=='WATER','Actual water beneath timber rail'
        for x,y,b in pier['end_cap_blocks']:assert blocks[y*w+x]==int(b,16),'Closed native pier corner/end changed'
        for x,y,b in pier.get('bank_cap_blocks',[]):assert blocks[y*w+x]==int(b,16),'Protected pier/bank corner changed'
        assert all(tuple(at) in seen or tuple(at) in occupied for at in pier['core_walk_line']),'Pier approach/deck disconnected'
        start_x=2*pier['deck_blocks'][0][0];end_x=2*pier['end_cap_blocks'][0][0];deck_y=2*pier['deck_blocks'][0][1]
        deck_floor={(x,y) for x in range(start_x,end_x+1) for y in range(deck_y-1,deck_y+3) if kind(x,y)=='FLOOR'}
        assert not any(kind(*q)=='WATER' for at in deck_floor for q in neighbors(at)),'Walking pier deck has an unintended Surf launch through its rails/corners'
    assert kind(49,19)=='FLOOR' and kind(49,20)=='WATER','Intentional native Surf landing'
    perimeter={(x,y) for y in range(h*2) for x in range(w*2) if x in {0,w*2-1} or y in {0,h*2-1}}
    assert len(perimeter)==252 and not any(kind(*at)=='WATER' for at in perimeter),'Complete unconnected outside water boundary must close'
    assert not {a for a in surf&perimeter if kind(*a)=='WATER'},'Surf escapes outside edge'
    for x in range(w*2):assert kind(x,47)=='BUOY','Southern visible water-rock barrier'
    for y in range(20,48):assert kind(79,y)=='BUOY','Eastern visible water-rock barrier'
    for y in range(40,48):assert kind(0,y)=='BUOY','Western visible water-rock barrier'
    seam=[y for y in range(h*2) if kind(63,y)==kind(64,y)=='WATER'];assert seam,'Lake must genuinely span both chunk neighbors'
    assert l['internal_water_neighbor']==dict(a='MnR18C00R00',direction='east',b='MnR18C01R00',reverse='west',offset=0,core_seam_x=64),'Retain real reciprocal water-neighbor contract'
    native=all(name in p['data']['maps'] for name in ['MnR18C00R00','MnR18C01R00'])
    if native:
        for a,b,direction in [('MnR18C00R00','MnR18C01R00','east'),('MnR18C01R00','MnR18C00R00','west')]:assert dict(direction=direction,target_map=b,offset=0) in p['data']['maps'][a]['attributes']['connections'],'Unconnected open internal lake seam'
    for v in d['surf_viewpoints']:assert tuple(v['tile']) in surf,'Surf probe not reachable'
    vx,vy=next(v['tile'] for v in d['viewpoints'] if v['id']=='EASTERN_BEACH');water_visible=sum(kind(x,y)=='WATER' for x in range(vx-5,vx+5) for y in range(vy-4,vy+5));assert water_visible>=15,'Beach lookout does not show the lake'
    return dict(source_metatile_varieties_used=len(set(blocks)),routine_shore_widths=widths,walking_shore_watch_actors_checked=4,surf_only_items_checked=d['surf_only_actor_ids'],marina_piers_checked=len(l['piers']),unintended_pier_surf_launches=0,outside_perimeter_tiles=len(perimeter),reachable_surf_water_tiles=sum(kind(*a)=='WATER' for a in surf),reachable_outer_water_tiles=0,visible_beach_water_tiles=water_visible,internal_lake_seam_core_y=seam,native_internal_neighbors_checked=native)

def render(d,l,p,art):
    prepare(p,l);r=render_checked(walking_design(d),l,p,art,output=OUTPUT)
    try:r.update(check_lake(d,l,p))
    except AssertionError as error:r['errors'].append(str(error))
    if 'gfx/tilesets/mn_headwaters.png' in p['runtime_files']:
        published,original=art.blocks('mn_headwaters'),art.blocks('johto')
        assert all(a.tobytes()==b.tobytes() for a,b in zip(published,original)),'Native original Johto pixels changed'
        for entry in l['derived_blocks']:
            expected=art.render('johto',1,1,[int(entry['id'],16)],derived=l['derived_blocks'])
            assert expected.tobytes()==published[int(entry['id'],16)].tobytes(),'Source/native timber recipe differs'
        r['original_blocks_preserved']=len(original);r['native_pier_recipes_compared']=3
    scene=render_stage(Image.open(OUTPUT/'whole-route.png'), 'convert', OUTPUT, 'review_mille_lacs_shore-render-line94-10-convert', 'RGBA');anchors=render_stage(Image.open(OUTPUT/'anchors.png'), 'convert', OUTPUT, 'review_mille_lacs_shore-render-line94-71-convert', 'RGBA');draw=ImageDraw.Draw(anchors)
    for a in d['items']:
        if a['id'] not in d['surf_only_actor_ids']:continue
        x,y=a['tile'];at=(x*16,y*16-4);actor=sprite(p,'SPRITE_POKE_BALL');scene.alpha_composite(actor,at);anchors.alpha_composite(actor,at);draw.rectangle((x*16,y*16,x*16+15,y*16+15),outline='#ffd250',width=2);draw.text((x*16,y*16-12),a['id'],fill='white',stroke_width=1,stroke_fill='black')
    digest=hashlib.sha256(scene.tobytes()).hexdigest()[:8];save_render(render_stage(scene, 'convert', OUTPUT, 'review_mille_lacs_shore-render-line98-71-convert', 'RGB'), OUTPUT/'whole-route.png');save_render(render_stage(scene, 'convert', OUTPUT, 'review_mille_lacs_shore-render-line98-131-convert', 'RGB'), OUTPUT/f'whole-route-{digest}.png');save_render(render_stage(scene, 'resize', OUTPUT, 'review_mille_lacs_shore-render-line98-201-resize', (640,384),Image.Resampling.NEAREST), OUTPUT/'overview.png');save_render(anchors, OUTPUT/'anchors.png')
    views=[]
    for v in d['surf_viewpoints']:
        x,y=v['tile'];im=render_stage(render_stage(scene, 'crop', OUTPUT, 'review_mille_lacs_shore-render-line101-25-crop', (x*16-80,y*16-72,x*16+80,y*16+72)), 'convert', OUTPUT, 'review_mille_lacs_shore-render-line101-25-convert', 'RGB');views.append((v['id'],im));save_render(render_stage(im, 'resize', OUTPUT, 'review_mille_lacs_shore-render-line101-125-resize', (640,576),Image.Resampling.NEAREST), OUTPUT/('viewport-'+v['id'].lower()+'.png'))
    save_render(room_sheet(views,columns=2), OUTPUT/'surf-boundaries.png');r.update(pickups=len(d['items']),image=f'whole-route-{digest}.png',status='INVALID — see errors' if r['errors'] else 'Source geometry checked; consult the route runbook for current native publication, controller and earned progression evidence');(OUTPUT/'revision.json').write_text(json.dumps(r,indent=2)+'\n');return r

def negative_checks(d,l,p):
    def point(layout,at,block):layout['layers'].append(dict(kind='point',position=at,block=block))
    cases=[
        ('open east water edge',lambda d,l:point(l,[39,15],'35')),
        ('open south water edge',lambda d,l:point(l,[20,23],'35')),
        ('open west water edge',lambda d,l:point(l,[0,21],'35')),
        ('missing north timber rail',lambda d,l:point(l,[11,10],'71')),
        ('open pier end',lambda d,l:point(l,[15,11],'71')),
        ('missing corner pier cap',lambda d,l:point(l,[15,10],'82')),
        ('volunteer requires Surf',lambda d,l:d['residents'][0].update(tile=[59,35])),
        ('offshore reward freely on road',lambda d,l:d['items'][3].update(tile=[29,13])),
        ('grass neck removed',lambda d,l:l['layers'].append(dict(kind='rect',material='lawn',bounds=[18,5,21,9]))),
        ('decorative Cut tree',lambda d,l:point(l,[16,5],'5b')),
        ('nonexistent reward rod',lambda d,l:d['quest_contract'].update(reward_item='CANOE_ROD')),
        ('unconnected water seam',lambda d,l:l['internal_water_neighbor'].update(b='MissingMap')),
        ('wide routine shoulder',lambda d,l:point(l,[28,4],'02')),
    ]
    for label,mutate in cases:
        design,layout=deepcopy(d),deepcopy(l);mutate(design,layout);rejected=False
        try:
            report,_,_=check(walking_design(design),layout,p,layout_blocks(layout));rejected=bool(report['errors']);check_lake(design,layout,p)
        except AssertionError:rejected=True
        assert rejected,'Bad fixture accepted: '+label
    return [label for label,_ in cases]

def main():
    parser=argparse.ArgumentParser();parser.add_argument('--pack',type=Path,default=ROOT/'content-packs/text-tui.crystalpack');parser.add_argument('--watch',action='store_true');parser.add_argument('--serve',action='store_true');parser.add_argument('--port',type=int,default=8784);parser.add_argument('--verify-published',action='store_true');parser.add_argument('--self-test',action='store_true');args=parser.parse_args();p=load_pack(args.pack);art=SourceArt(p);state={'ok':False,'error':'Initial source render pending'}
    if args.serve:
        OUTPUT.mkdir(parents=True,exist_ok=True);(OUTPUT/'index.html').write_text(PAGE)
        class Handler(SimpleHTTPRequestHandler):
            def log_message(self,*args):pass
            def do_GET(self):
                if self.path.split('?')[0]=='/state':
                    body=json.dumps(state).encode();self.send_response(200);self.send_header('Content-Type','application/json');self.send_header('Cache-Control','no-store');self.send_header('Content-Length',str(len(body)));self.end_headers();self.wfile.write(body)
                else:super().do_GET()
        server=ThreadingHTTPServer(('127.0.0.1',args.port),partial(Handler,directory=str(OUTPUT)));threading.Thread(target=server.serve_forever,daemon=True).start();print(f'Mille Lacs shore review: http://127.0.0.1:{args.port}/',flush=True)
    last=None
    while True:
        revision=tuple(v.stat().st_mtime_ns for v in [DOSSIER,LAYOUT])
        if revision!=last:
            try:
                d,l=json.loads(DOSSIER.read_text()),json.loads(LAYOUT.read_text());r=render(d,l,p,art)
                if args.self_test:r['negative_cases_rejected']=negative_checks(d,l,p)
                if args.verify_published:r.update(verify_published(d,l,p))
                state.update(ok=not r['errors'],error='; '.join(r['errors']),revision=str(time.time_ns()),**r);print(json.dumps(r),flush=True)
            except (AssertionError,KeyError,ValueError,IndexError,TypeError) as error:state.update(ok=False,error=str(error));print('DESIGN ERROR: '+str(error),flush=True)
            if not state['ok'] and not(args.watch or args.serve):raise SystemExit(1)
            last=revision
        if not(args.watch or args.serve):break
        time.sleep(.15)
if __name__=='__main__':main()
