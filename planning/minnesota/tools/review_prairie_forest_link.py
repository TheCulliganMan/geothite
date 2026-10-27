#!/usr/bin/env python3
"""Review native ecotone geography; never exports content or simulates gameplay."""
from render_archive import save_render
from render_stages import render_stage
import argparse
from collections import deque
from copy import deepcopy
from functools import partial
from http.server import SimpleHTTPRequestHandler,ThreadingHTTPServer
import json
from pathlib import Path
import threading
import time
from PIL import Image
from render_reference import SourceArt,load_pack,layout_blocks,room_sheet
from review_league_approach import HTML,check,render as render_checked,verify_published
from review_gunflint import flood,neighbors,VECTORS
from ledge_paths import shortest_walk
ROOT=Path(__file__).resolve().parents[3];DOSSIER=ROOT/'planning/minnesota/routes/MN_R19.json';LAYOUT=ROOT/'planning/minnesota/routes/MN_R19.layout.json';OUTPUT=ROOT/'planning/minnesota/renders/prairie-forest-link'
PAGE=HTML.replace('League Approach','Prairie Forest Link').replace('all eighteen chunks','Itasca pines to Detroit prairie').replace('Border Lake · Wind Tor · Granite switchback · Dry saddle · Cedar Shelter · Long Lake · Old quarry · Ely arrival.','Native oak openings · Real battle/grass choice · Two-way ramp loop · Spring wetland · Prairie observations.').replace('<h2>Cedar Shelter</h2><img id="rooms" src="interiors.png">','<h2>Wetland Surf probe</h2><img id="surf" src="surf-boundaries.png">').replace("if(room)room.src='interiors.png?r='+rev","if(room)room.src='interiors.png?r='+rev;document.querySelector('#surf').src='surf-boundaries.png?r='+rev").replace('width:1536px','width:1024px').replace('(native?3072:1536)','(native?1024:512)')

def inspect_contracts(d,l,p):
    b=layout_blocks(l);w,h=l['dimensions_metatiles'];c=p['data']['tilesets'][l['compiled_tileset']]['collision']
    def kind(x,y):return c[f'{b[y//2*w+x//2]:02x}'][y%2*2+x%2]
    assert (w,h)==(32,30) and d['required_flags']==['BADGE_6'],'Retain regional gate and bent extent'
    actors=d['trainers']+d['residents']+d['items'];occupied={tuple(a['tile']) for a in actors};start,end=[tuple(2*v for v in e['gate_metatile']) for e in d['endpoints']]
    walk={(x,y) for y in range(h*2) for x in range(w*2) if kind(x,y) in {'FLOOR','TALL_GRASS'} and (x,y) not in occupied};seen=flood(start,walk);assert end in seen,'Main bent road disconnected'
    assert d['cut_gates']==[] and d['one_way_ledges']==[] and d['shortcut'] is None,'Two-way inspection loop does not use a decorative gate or cliff jump'
    assert not any(kind(x,y)=='CUT_TREE' for y in range(h*2) for x in range(w*2)),'Native canopy has an incidental Cut quadrant'
    sight=set()
    for trainer in d['trainers']:
        dx,dy=VECTORS[trainer['facing']];x,y=trainer['tile']
        for step in range(1,trainer['sight_range']+1):
            at=(x+dx*step,y+dy*step)
            if not(0<=at[0]<w*2 and 0<=at[1]<h*2) or kind(*at) not in {'FLOOR','TALL_GRASS'}:break
            sight.add(at)
    peaceful=flood(start,walk-sight)
    for obs in d['quest_contract']['observations']:
        sign=next(a for a in d['signs'] if a['id']==obs['sign_id']);assert set(neighbors(tuple(sign['tile'])))&peaceful,'Observation requires Surf/Cut or an unavoidable battle'
    assert len(d['quest_contract']['observations'])==3 and d['quest_contract']['reward_map']=='MnDetroitLakes' and d['quest_contract']['reward_item']=='MAX_REPEL','Preserve three signs and actual Detroit ranger reward'
    quest=d['quest_contract']
    assert all(0<len(line)<=18 for lines in quest['text_labels'].values() for line in lines),'Observation quest text exceeds native line width'
    for obs in quest['observations']:
        assert any(v['command']=='setevent' and v['args']==[obs['flag']] for v in quest['script_labels'][obs['sign_id']]),'Read event must use its own explicit observation flag'
    tree=d['headbutt_contract'];assert tree['source_table_map']=='Route29' and tree['level_override']==28 and 'headbutt' in p['data']['field_encounters']['Route29']['tables'],'Reference an actual native Headbutt table'
    def cost(a,z,band,forbid=frozenset()):
        costs={a:0};todo=deque([a])
        while todo:
            at=todo.popleft()
            for q in neighbors(at):
                if q not in walk or q in forbid:continue
                n=costs[at]+int(q in band)
                if n>=costs.get(q,10**9):continue
                costs[q]=n;(todo.append if q in band else todo.appendleft)(q)
        return min((costs.get(q,10**9) for q in z),default=10**9)
    choices=[]
    for choice in d['battle_or_grass_choices']:
        t=next(a for a in d['trainers'] if a['id']==choice['trainer_id']);assert t['tile']==[43,21] and t['facing']=='south' and t['sight_range']==3,'Trainer must cover all four actual lane rows'
        x0,y0,x1,y1=choice['bounds_core'];band={(x,y) for y in range(y0,y1) for x in range(x0,x1)};assert all(kind(*q)=='TALL_GRASS' for q in band),'Alternative habitat changed'
        a,z=tuple(choice['approach_core']),tuple(choice['exit_core']);assert cost(a,{z},band)==0,'Short road missing'
        dx,dy=VECTORS[t['facing']];hazards={(t['tile'][0]+dx*i,t['tile'][1]+dy*i) for i in range(1,t['sight_range']+1)}
        for point in hazards:assert kind(*point)=='FLOOR','Actual trainer sight stopped before lane closure'
        steps=cost(a,{z},band,hazards);reverse=cost(z,{a},band,hazards);assert choice['minimum_grass_steps']<=min(steps,reverse)<10**9,'Battle and habitat freely bypassed or sealed'
        choices.append(dict(trainer_id=t['id'],actual_grass_steps_either_direction=[steps,reverse],native_sight_cells=[list(q) for q in sorted(hazards)]))
    rewards=[]
    for reward in d['optional_grass_rewards']:
        a=next(a for a in d['items'] if a['id']==reward['actor_id']);x0,y0,x1,y1=reward['bounds_core'];band={(x,y) for y in range(y0,y1) for x in range(x0,x1) if kind(x,y)=='TALL_GRASS'};steps=cost(start,set(neighbors(tuple(a['tile']))),band);assert reward['minimum_grass_steps']<=steps<10**9,'Meadow reward is freely reachable or sealed';rewards.append(dict(actor_id=a['id'],actual_grass_steps=steps))
    loop=d['ramp_loop']
    for x,y in loop['ramps_metatiles']:assert [kind(2*x,2*y),kind(2*x+1,2*y),kind(2*x,2*y+1),kind(2*x+1,2*y+1)]==['WALL','FLOOR','FLOOR','FLOOR'],'Actual native74ramp required'
    assert all(tuple(at) in seen for at in loop['walking_lower_core']+loop['clear_upper_core']),'Ramp approach disconnected'
    upper={(x,y) for x,y in seen if 6<=x<23 and 24<=y<36};upper_path=shortest_walk(*map(tuple,loop['clear_upper_core']),upper);assert upper_path and len(upper_path)>12,'Two ramp heads need an actual upper inspection path'
    assert not any(kind(*q)=='HOP_DOWN' for q in upper),'Original brief requires two-way steps, not cliff jumps'
    perimeter={(x,y) for y in range(h*2) for x in range(w*2) if x in {0,w*2-1} or y in {0,h*2-1}};assert len(perimeter)==244 and not any(kind(*q)=='WATER' for q in perimeter),'Unclosed water at unconnected edge'
    surf=flood(start,walk|{(x,y) for y in range(h*2) for x in range(w*2) if kind(x,y)=='WATER'});water={at for at in surf if kind(*at)=='WATER'};assert water and not water&perimeter,'Actual accessible pond must stay within the route'
    assert kind(43,33)=='FLOOR' and kind(43,34)=='WATER','Wetland launch needs real land/water quadrants'
    for v in d['surf_viewpoints']:assert tuple(v['tile']) in water,'Unreachable wetland Surf probe'
    for x in [32,33,34,35]:assert (x,33) in seen,'Four-core-wide routine oak passage'
    assert kind(31,33)=='HEADBUTT_TREE' and kind(36,33)=='HEADBUTT_TREE','Oak edges should bound the passage rather than leave an empty lawn'
    return dict(source_metatile_varieties_used=len(set(b)),battle_or_grass_choice_proofs=choices,optional_meadow_rewards=rewards,observation_signs_walking_checked=3,actual_native_ramps_checked=2,upper_two_ramp_path_steps=len(upper_path)-1,native_cut_quadrants=0,oak_through_lane_core_width=4,outer_perimeter_core_tiles=len(perimeter),reachable_surf_water_tiles=len(water),reachable_outer_water_tiles=0)

def render(d,l,p,art):
    report=render_checked(d,l,p,art,output=OUTPUT)
    try:report.update(inspect_contracts(d,l,p))
    except AssertionError as error:report['errors'].append(str(error))
    scene=Image.open(OUTPUT/'whole-route.png');views=[]
    for v in d['surf_viewpoints']:
        x,y=v['tile'];im=render_stage(scene, 'crop', OUTPUT, 'review_prairie_forest_link-render-line86-25-crop', (x*16-80,y*16-72,x*16+80,y*16+72));views.append((v['id'],im));save_render(render_stage(im, 'resize', OUTPUT, 'review_prairie_forest_link-render-line86-110-resize', (640,576),Image.Resampling.NEAREST), OUTPUT/('viewport-'+v['id'].lower()+'.png'))
    save_render(room_sheet(views,columns=1), OUTPUT/'surf-boundaries.png');report['status']='INVALID — see errors' if report['errors'] else 'Source geometry checked; consult the route runbook for current native publication, controller and earned progression evidence';(OUTPUT/'revision.json').write_text(json.dumps(report,indent=2)+'\n');return report

def negative_checks(d,l,p):
    def point(layout,at,block):layout['layers'].append(dict(kind='point',position=at,block=block))
    cases=[
        ('trainer faces away from narrow lane',lambda d,l:d['trainers'][0].update(facing='north')),
        ('trainer sees only part of lane',lambda d,l:d['trainers'][0].update(sight_range=2)),
        ('free dry habitat detour',lambda d,l:l['layers'].append(dict(kind='rect',material='floor',bounds=[20,13,24,15]))),
        ('southern habitat removed',lambda d,l:l['layers'].append(dict(kind='rect',material='floor',bounds=[11,24,14,26]))),
        ('missing west native ramp',lambda d,l:point(l,[5,18],'72')),
        ('missing east native ramp',lambda d,l:point(l,[9,18],'72')),
        ('hop replaces a two-way step',lambda d,l:point(l,[5,18],'57')),
        ('incidental native Cut canopy',lambda d,l:point(l,[12,15],'63')),
        ('wetland observation requires Surf',lambda d,l:d['signs'][2].update(tile=[49,35])),
        ('pond escapes unconnected map edge',lambda d,l:point(l,[31,20],'35')),
        ('oak passage becomes empty lawn',lambda d,l:point(l,[15,16],'02')),
        ('reward moved outside its habitat',lambda d,l:d['items'][3].update(tile=[35,45])),
    ]
    for label,mutate in cases:
        design,layout=deepcopy(d),deepcopy(l);mutate(design,layout);rejected=False
        try:
            report,_,_=check(design,layout,p,layout_blocks(layout));rejected=bool(report['errors']);inspect_contracts(design,layout,p)
        except AssertionError:rejected=True
        assert rejected,'Bad fixture accepted: '+label
    return [label for label,_ in cases]

def main():
    parser=argparse.ArgumentParser();parser.add_argument('--pack',type=Path,default=ROOT/'content-packs/text-tui.crystalpack');parser.add_argument('--watch',action='store_true');parser.add_argument('--serve',action='store_true');parser.add_argument('--port',type=int,default=8785);parser.add_argument('--verify-published',action='store_true');parser.add_argument('--self-test',action='store_true');args=parser.parse_args();p=load_pack(args.pack);art=SourceArt(p);state={'ok':False,'error':'Initial source render pending'}
    if args.serve:
        OUTPUT.mkdir(parents=True,exist_ok=True);(OUTPUT/'index.html').write_text(PAGE)
        class Handler(SimpleHTTPRequestHandler):
            def log_message(self,*args):pass
            def do_GET(self):
                if self.path.split('?')[0]=='/state':
                    body=json.dumps(state).encode();self.send_response(200);self.send_header('Content-Type','application/json');self.send_header('Cache-Control','no-store');self.send_header('Content-Length',str(len(body)));self.end_headers();self.wfile.write(body)
                else:super().do_GET()
        server=ThreadingHTTPServer(('127.0.0.1',args.port),partial(Handler,directory=str(OUTPUT)));threading.Thread(target=server.serve_forever,daemon=True).start();print(f'Prairie forest review: http://127.0.0.1:{args.port}/',flush=True)
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
