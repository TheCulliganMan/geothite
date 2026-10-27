#!/usr/bin/env python3
"""Review hand-authored Iron Range terrain; no packs or story runtime are built."""
from render_archive import save_render
from render_stages import render_stage
import argparse
from copy import deepcopy
from collections import deque
from functools import partial
import hashlib
from http.server import SimpleHTTPRequestHandler, ThreadingHTTPServer
import json
from pathlib import Path
import threading
import time

from PIL import Image, ImageDraw
from render_reference import SourceArt, layout_blocks, load_pack, room_sheet
from review_headwaters import prepare as prepare_headwaters
from review_north_shore import oriented_sprite
from review_grand_marais import sprite
from review_league_approach import check
from review_gunflint import flood, neighbors, VECTORS
from ledge_paths import shortest_walk, draw_ledge_paths

ROOT = Path(__file__).resolve().parents[3]
LAYOUT = ROOT/'planning/minnesota/routes/MN_R09.layout.json'
DOSSIER = ROOT/'planning/minnesota/routes/MN_R09.json'
OUTPUT = ROOT/'planning/minnesota/renders/iron-range'


def prepare(pack, layout):
    """Preview root-owned bounded native art imports without publishing a pack."""
    prepare_headwaters(pack, layout)
    definition = deepcopy(pack['data']['tilesets']['mn_headwaters'])
    for entry in layout['derived_blocks']:
        source = entry.get('collision_source_tileset', entry['source_tileset'])
        block = entry.get('collision_source_block', entry['source_block'])
        definition['collision'][entry['id']] = list(pack['data']['tilesets'][source]['collision'][block])
    pack['data']['tilesets']['mn_iron_range'] = definition


def inspect(design, layout, pack):
    assert layout['schema_version']==2 and layout['reject_cliff_cuts'] is True
    w,h = layout['dimensions_metatiles']
    for layer in layout['layers']:
        if layer['kind'] in {'polygon','cliff_polygon','path'}:
            points=layer['points'];ends=points[1:] if layer['kind']=='path' else points[1:]+points[:1]
            assert all(a[0]==b[0] or a[1]==b[1] for a,b in zip(points,ends)), layer['name']+': diagonal edge'
    prepare(pack,layout)
    blocks=layout_blocks(layout);after=list(blocks)
    for gate in design['cut_gates']:
        x,y=gate['tile'];after[y//2*w+x//2]=int(gate['replacement_block'],16)
    report,kind,post_walk=check(design,layout,pack,after)
    collisions=pack['data']['tilesets'][layout['compiled_tileset']]['collision']
    def before_kind(x,y):return collisions[f'{blocks[y//2*w+x//2]:02x}'][y%2*2+x%2]
    occupied={tuple(a['tile']) for a in design['trainers']+design['residents']+design['items']}
    def allowed(opened=False,surf=False):
        k=kind if opened else before_kind
        return {(x,y) for y in range(h*2) for x in range(w*2)
                if k(x,y) in ({'FLOOR','TALL_GRASS'}|({'WATER'} if surf else set())) and (x,y) not in occupied}
    start,end=[tuple(v*2 for v in e['gate_metatile']) for e in design['endpoints']]
    pre_walk=flood(start,allowed());pre_surf=flood(start,allowed(False,True));post_surf=flood(start,allowed(True,True))
    assert end in pre_walk,'Cut must not gate required east-west passage'
    gates=[]
    for gate in design['cut_gates']:
        assert before_kind(*gate['tile'])=='CUT_TREE' and tuple(gate['approach']) in pre_walk,'Cut approach/real source quadrant'
        rule=pack['data']['field_moves']['cut']['replacements']['johto'][blocks[gate['tile'][1]//2*w+gate['tile'][0]//2]]
        assert rule['replacement_block_id']==int(gate['replacement_block'],16)
        assert all(tuple(p) not in pre_surf and tuple(p) in post_surf for p in gate['targets']),'Cut has a walking/Surf bypass'
        gates.append({'id':gate['id'],'native_block_override':'63→3f','sealed_targets':gate['targets'],'new_walk_tiles':len(post_walk-pre_walk)})
    assert {(x,y) for y in range(h*2) for x in range(w*2) if before_kind(x,y)=='CUT_TREE'}=={tuple(g['tile']) for g in design['cut_gates']},'Accidental decorative Cut quadrant'
    water={(x,y) for y in range(h*2) for x in range(w*2) if before_kind(x,y)=='WATER'}
    assert not any(x in {0,w*2-1} or y in {0,h*2-1} for x,y in water),'Unconnected outer Surf exit'
    outer_walk={p for p in allowed() if p[0] in {0,w*2-1} or p[1] in {0,h*2-1}}
    assert outer_walk=={(0,50),(0,51),(127,24),(127,25)},'Unplanned outer walking opening'
    # Actual native rail collision, rather than painted fences, seals the alignment.
    rail=layout['rail_crossing'];cx,cy=rail['crossing_metatile'];cross={(cx*2+dx,cy*2+dy) for dx in [0,1] for dy in [0,1]}
    assert all(before_kind(*p)=='FLOOR' for p in cross),'Crossing must be genuine walking rail block84'
    assert all(blocks[cy*w+x]==(0x84 if x==cx else 0x83) for x in range(*rail['track_span_metatiles'])),'Native track alignment'
    assert end not in flood(start,allowed()-cross),'Rail can be walked around instead of using its crossing'
    for crossing in layout['crossings']:
        for x,y in crossing['north_rail_blocks']:assert [before_kind(x*2,y*2),before_kind(x*2,y*2+1)]==['WALL','FLOOR']
        for x,y in crossing['south_rail_blocks']:assert [before_kind(x*2,y*2),before_kind(x*2,y*2+1)]==['FLOOR','WALL']
        for x,y in crossing['water_under_rail_blocks']:assert before_kind(x*2,y*2)=='WATER'
        cells={(x*2+dx,y*2+dy) for p in ['north_rail_blocks','south_rail_blocks'] for x,y in crossing[p] for dx in [0,1] for dy in [0,1]}
        assert end not in flood(start,allowed()-cells),'Creek crossing has a free dry bypass'
    for ledge in design['one_way_ledges']:
        x0,y0,x1,y1=ledge['bounds_core'];assert all(before_kind(x,y0)=='HOP_DOWN' and before_kind(x,y0+1)=='WALL' for x in range(x0,x1))
        assert all(before_kind(*p)=='WALL' for p in ledge['side_barriers'])
    rail_choice=design['battle_or_grass_choices'][0]
    trainer=next(a for a in design['trainers'] if a['id']==rail_choice['trainer_id'])
    tx,ty=trainer['tile'];dx,dy=VECTORS[trainer['facing']];sight=set()
    for distance in range(1,trainer['sight_range']+1):
        at=tx+dx*distance,ty+dy*distance
        if before_kind(*at) not in {'FLOOR','TALL_GRASS'}:break
        sight.add(at)
    x0,y0,x1,y1=rail_choice['bounds_core'];band={(x,y) for y in range(y0,y1) for x in range(x0,x1)}
    assert all(before_kind(*p)=='TALL_GRASS' for p in band),'Rail alternate is not real habitat'
    choice_start,choice_end=(100,40),(100,34)
    no_battle=allowed()-sight
    assert shortest_walk(choice_start,choice_end,no_battle),'Rail grass alternative is sealed'
    assert shortest_walk(choice_start,choice_end,no_battle-band) is None,'Rail trainer has a free dry bypass'
    costs={choice_start:0};todo=deque([choice_start])
    while todo:
        at=todo.popleft()
        for q in neighbors(at):
            if q not in no_battle:continue
            value=costs[at]+int(q in band)
            if value<costs.get(q,10**9):
                costs[q]=value
                (todo.append if q in band else todo.appendleft)(q)
    assert costs[choice_end]>=rail_choice['minimum_grass_steps'],'Rail encounter detour is shorter than declared'

    for camp_id in design['cut_gates'][0]['gated_actor_ids']:
        actor=next(a for a in design['residents']+design['items'] if a['id']==camp_id)
        assert not any(p in pre_surf for p in neighbors(tuple(actor['tile']))),'Cut actor can be reached before the tree opens'
    rooms=[]
    for room in design['interiors']:
        rw,rh=room['dimensions_metatiles'];rb=[int(b,16) for row in room['rows'] for b in row.split()];rc=pack['data']['tilesets']['cave']['collision']
        rk=lambda x,y:rc[f'{rb[y//2*rw+x//2]:02x}'][y%2*2+x%2]
        actors=room['residents']+room.get('story_trainers',[])+room.get('items',[])+room.get('story_objects',[]);occ={tuple(a['tile']) for a in actors}
        land={(x,y) for y in range(rh*2) for x in range(rw*2) if rk(x,y) in {'FLOOR','LADDER'} and (x,y) not in occ}
        seen=flood(tuple(room['clear_arrival']),land)
        for a in actors:
            assert rk(*a['tile'])=='FLOOR' and any(p in seen for p in neighbors(tuple(a['tile']))),a['id']+': mine actor footing/approach'
        rooms.append({'id':room['id'],'reachable_floor':len(seen),'return_ladder':room['exits'][0],
                      'foreman_approach':room['story_trainers'][0]['approach'],'story_status':'Native binding available in iron_range_story.rs; source preview checks geometry only'})
    report.update(status='Rockier source candidate; revised native terrain and controller review pending; Q06 lifecycle remains separate',
                  pre_cut_walk_tiles=len(pre_walk),post_cut_walk_tiles=len(post_walk),pre_cut_walk_surf_tiles=len(pre_surf),post_cut_walk_surf_tiles=len(post_surf),
                  cut_gates=gates,outer_water_tiles=0,outer_walking_openings=sorted(outer_walk),water_tiles=len(water),native_block_varieties=len(set(blocks)),
                  rail_crossing_core=sorted(cross),rail_crossing_only_walk_link=True,rail_trainer_sight_tiles=sorted(sight),rail_grass_choice_has_no_free_dry_bypass=True,rail_alternate_minimum_grass_steps=costs[choice_end],
                  pre_cut_endpoint_walk_steps=len(shortest_walk(start,end,allowed()))-1,rooms=rooms)
    return report,blocks,after,before_kind


def render(design,layout,pack,art):
    started=time.perf_counter();report,blocks,after,kind=inspect(design,layout,pack);w,h=layout['dimensions_metatiles']
    terrain=art.render('johto',w,h,blocks,derived=layout['derived_blocks']);scene=render_stage(terrain, 'convert', OUTPUT, 'review_iron_range-render-line137-82-convert', 'RGBA')
    for a in design['trainers']+design['residents']:scene.alpha_composite(oriented_sprite(pack,a['sprite'],a['facing']),(a['tile'][0]*16,a['tile'][1]*16-4))
    for a in design['items']:scene.alpha_composite(sprite(pack,'SPRITE_POKE_BALL'),(a['tile'][0]*16,a['tile'][1]*16-4))
    OUTPUT.mkdir(parents=True,exist_ok=True);digest=hashlib.sha256(scene.tobytes()).hexdigest()[:8]
    save_render(terrain, OUTPUT/'terrain.png');save_render(render_stage(scene, 'convert', OUTPUT, 'review_iron_range-render-line141-59-convert', 'RGB'), OUTPUT/'whole-route.png');save_render(render_stage(scene, 'convert', OUTPUT, 'review_iron_range-render-line141-119-convert', 'RGB'), OUTPUT/f'whole-route-{digest}.png')
    save_render(draw_ledge_paths(scene,design,report['ledge_returns']), OUTPUT/'ledges.png')
    overlay=scene.copy();draw=ImageDraw.Draw(overlay)
    for row in range(design['chunk_rows']):
        for col in range(design['chunk_columns']):
            x,y=col*1024,row*768;draw.rectangle((x,y,min(x+1023,w*32-1),min(y+767,h*32-1)),outline='white',width=2);draw.text((x+5,y+5),f'MnR09C{col:02}R{row:02}',fill='white',stroke_fill='black',stroke_width=1)
    for a in design['trainers']+design['residents']+design['items']+design['signs']+design['doors']:
        x,y=a['tile'];draw.rectangle((x*16,y*16,x*16+15,y*16+15),outline='#ffc748',width=2);draw.text((x*16,y*16-12),a['id'],fill='white',stroke_width=1,stroke_fill='black')
    save_render(render_stage(overlay, 'convert', OUTPUT, 'review_iron_range-render-line149-16-convert', 'RGB'), OUTPUT/'anchors.png')
    views=[]
    for v in design['viewpoints']:
        x,y=v['tile'];im=render_stage(render_stage(scene, 'crop', OUTPUT, 'review_iron_range-render-line152-25-crop', (x*16-80,y*16-72,x*16+80,y*16+72)), 'convert', OUTPUT, 'review_iron_range-render-line152-25-convert', 'RGB');views.append((v['id'],im));save_render(render_stage(im, 'resize', OUTPUT, 'review_iron_range-render-line152-125-resize', (640,576),Image.Resampling.NEAREST), OUTPUT/f'viewport-{v["id"].lower()}.png')
    save_render(room_sheet(views,3), OUTPUT/'viewpoints.png');save_render(room_sheet(views,3), OUTPUT/f'viewpoints-{digest}.png')
    rooms=[]
    for room in design['interiors']:
        rw,rh=room['dimensions_metatiles'];im=render_stage(art.render('cave',rw,rh,[int(b,16) for row in room['rows'] for b in row.split()]), 'convert', OUTPUT, 'review_iron_range-render-line156-46-convert', 'RGBA')
        for a in room['residents']+room.get('story_trainers',[])+room.get('story_objects',[]):im.alpha_composite(oriented_sprite(pack,a['sprite'],a['facing']),(a['tile'][0]*16,a['tile'][1]*16-4))
        save_render(render_stage(im, 'convert', OUTPUT, 'review_iron_range-render-line158-20-convert', 'RGB'), OUTPUT/f'room-{room["id"]}.png');rooms.append((room['id'],render_stage(im, 'convert', OUTPUT, 'review_iron_range-render-line158-97-convert', 'RGB')))
    save_render(room_sheet(rooms,1), OUTPUT/'rooms.png')
    after_scene=render_stage(art.render('johto',w,h,after,derived=layout['derived_blocks']), 'convert', OUTPUT, 'review_iron_range-render-line160-16-convert', 'RGBA')
    for a in design['trainers']+design['residents']:after_scene.alpha_composite(oriented_sprite(pack,a['sprite'],a['facing']),(a['tile'][0]*16,a['tile'][1]*16-4))
    for a in design['items']:after_scene.alpha_composite(sprite(pack,'SPRITE_POKE_BALL'),(a['tile'][0]*16,a['tile'][1]*16-4))
    after_digest=hashlib.sha256(after_scene.tobytes()).hexdigest()[:8];save_render(render_stage(after_scene, 'convert', OUTPUT, 'review_iron_range-render-line163-83-convert', 'RGB'), OUTPUT/f'after-cut-{after_digest}.png')
    for gate in design['cut_gates']:
        x,y=gate['approach'];box=(x*16-80,y*16-72,x*16+80,y*16+72)
        for label,im in [('before',scene),('after',after_scene)]:save_render(render_stage(render_stage(render_stage(im, 'crop', OUTPUT, 'review_iron_range-render-line166-77-crop', box), 'convert', OUTPUT, 'review_iron_range-render-line166-77-convert', 'RGB'), 'resize', OUTPUT, 'review_iron_range-render-line166-77-resize', (640,576),Image.Resampling.NEAREST), OUTPUT/f'cut-{label}-{digest}.png')
    report.update(image=f'whole-route-{digest}.png',after_cut_image=f'after-cut-{after_digest}.png',render_ms=round((time.perf_counter()-started)*1000,1),revision=str(time.time_ns()))
    (OUTPUT/'revision.json').write_text(json.dumps(report,indent=2)+'\n');return report


def negative_checks(design,layout,pack):
    """Optional read-only malformed-design checks; never changes saved authoring."""
    cases={
        'Cut side entry':([5,28],'3f'), 'outer Surf hole':([63,20],'35'),
        'spruce grass paved':([7,24],'02'), 'extra rail gap':([48,15],'84'),
        'rail crossing sealed':([50,15],'83'), 'painted ledge':([40,20],'02'),
        'free dry trainer bypass':([51,18],'01'),
    }
    outcomes=[]
    for name,(position,block) in cases.items():
        candidate=deepcopy(layout)
        candidate['layers'].append(dict(name='Negative '+name,kind='point',position=position,block=block))
        try:
            report,*_=inspect(design,candidate,pack)
            assert not report['errors'],'; '.join(report['errors'])
        except AssertionError as error:
            outcomes.append(dict(mutation=name,rejected=True,reason=str(error) or 'Native ledge mismatch'))
        else:raise AssertionError('Missed defect: '+name)
    for name in ['mine exit mismatch','diagonal closing edge']:
        d,l=deepcopy(design),deepcopy(layout)
        if name=='mine exit mismatch':d['interiors'][0]['exits']=[[12,19]]
        else:l['layers'].append(dict(name=name,kind='polygon',material='forest',points=[[1,1],[3,1],[3,3],[2,3]]))
        try:
            report,*_=inspect(d,l,pack)
            assert not report['errors'],'; '.join(report['errors'])
        except AssertionError as error:outcomes.append(dict(mutation=name,rejected=True,reason=str(error)))
        else:raise AssertionError('Missed defect: '+name)
    art=SourceArt(pack)
    for ident,source in [(0x83,0x7e),(0x84,0x7f)]:
        assert art.render('johto',1,1,[ident],derived=layout['derived_blocks']).tobytes()==art.blocks('johto_modern')[source].tobytes()
    result=dict(negative_cases=outcomes,native_rail_source_pixels_equal=True,source_only=True,
                layout_sha256=hashlib.sha256(LAYOUT.read_bytes()).hexdigest(),dossier_sha256=hashlib.sha256(DOSSIER.read_bytes()).hexdigest())
    (OUTPUT/'negative-checks.json').write_text(json.dumps(result,indent=2)+'\n')
    return result


PAGE='''<!doctype html><meta charset="utf-8"><title>Iron Range West source review</title><style>body{background:#14212b;color:#eef;font:16px system-ui;margin:20px}img{image-rendering:pixelated;max-width:none}.scroll{overflow:auto;max-height:78vh}header{position:sticky;top:0;background:#14212bf5;padding:12px}button{padding:10px;margin:4px}</style><header><h1>Iron Range West — whole route</h1><p>Native source pixels and static collision only. Q06 story battle/instrument delivery, native pack publication and gameplay remain separate.</p><button data-layer="whole-route">Whole route</button><button data-layer="terrain">Terrain</button><button data-layer="anchors">Anchors</button><button data-layer="ledges">Ledge return</button><p id="status"></p></header><div class="scroll"><img id="map" width="2048"></div><h2>Native player framing</h2><img id="views"><h2>Mine proposal</h2><img id="rooms"><script>let layer='whole-route',revision='';function refresh(){document.querySelector('#map').src=layer+'.png?r='+revision;document.querySelector('#views').src='viewpoints.png?r='+revision;document.querySelector('#rooms').src='rooms.png?r='+revision}document.querySelectorAll('[data-layer]').forEach(b=>b.onclick=()=>{layer=b.dataset.layer;refresh()});setInterval(async()=>{let s=await(await fetch('/state',{cache:'no-store'})).json();document.querySelector('#status').textContent=s.ok?s.render_ms+' ms · '+s.status:'INVALID: '+s.error;if(revision!==s.revision){revision=s.revision;refresh()}},500)</script>'''


def main():
    parser=argparse.ArgumentParser();parser.add_argument('--pack',type=Path,default=ROOT/'content-packs/text-tui.crystalpack');parser.add_argument('--serve',action='store_true');parser.add_argument('--negative-checks',action='store_true');parser.add_argument('--port',type=int,default=8791);args=parser.parse_args()
    pack=load_pack(args.pack);art=SourceArt(pack);state={'value':{'ok':False,'error':'Initial source render pending'}}
    if args.serve:
        OUTPUT.mkdir(parents=True,exist_ok=True);(OUTPUT/'index.html').write_text(PAGE)
        class Handler(SimpleHTTPRequestHandler):
            def log_message(self,*args):pass
            def do_GET(self):
                if self.path.split('?')[0]=='/state':
                    body=json.dumps(state['value']).encode();self.send_response(200);self.send_header('Content-Type','application/json');self.send_header('Content-Length',str(len(body)));self.end_headers();self.wfile.write(body)
                else:super().do_GET()
        server=ThreadingHTTPServer(('127.0.0.1',args.port),partial(Handler,directory=str(OUTPUT)));threading.Thread(target=server.serve_forever,daemon=True).start();print(f'Iron Range source review http://127.0.0.1:{args.port}/',flush=True)
    if args.negative_checks and args.serve:parser.error('--negative-checks requires a one-shot run')
    last=None
    while True:
        revision=(LAYOUT.stat().st_mtime_ns,DOSSIER.stat().st_mtime_ns)
        if revision!=last:
            try:
                report=render(json.loads(DOSSIER.read_text()),json.loads(LAYOUT.read_text()),pack,art);state['value']={'ok':not report['errors'],'error':'; '.join(report['errors']),**report};print(json.dumps(report),flush=True)
                if report['errors'] and not args.serve:raise SystemExit(1)
            except (AssertionError,KeyError,ValueError,IndexError,TypeError) as e:
                state['value']={'ok':False,'error':str(e)};print('DESIGN ERROR: '+str(e),flush=True)
                if not args.serve:raise SystemExit(1)
            last=revision
        if not args.serve:
            if args.negative_checks:print(json.dumps(negative_checks(json.loads(DOSSIER.read_text()),json.loads(LAYOUT.read_text()),pack)),flush=True)
            break
        time.sleep(.2)


if __name__=='__main__':main()
