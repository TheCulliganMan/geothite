#!/usr/bin/env python3
"""Hand-authored Border Pines source preview; no game packs or runtime emulation."""
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
from render_reference import SourceArt,layout_blocks,load_pack,room_sheet
from review_north_shore import oriented_sprite
from review_grand_marais import sprite
from review_league_approach import check
from review_gunflint import flood,neighbors
from ledge_paths import shortest_walk

ROOT=Path(__file__).resolve().parents[3]
LAYOUT=ROOT/'planning/minnesota/routes/MN_R17.layout.json'
DOSSIER=ROOT/'planning/minnesota/routes/MN_R17.json'
OUTPUT=ROOT/'planning/minnesota/renders/border-pines'


def prepare(pack,layout):
    """Preview exact source collisions only; Rust owns compiled tileset publication."""
    definition=deepcopy(pack['data']['tilesets']['forest'])
    for a in layout['derived_blocks']:
        source=a.get('collision_source_tileset',a['source_tileset']);block=a.get('collision_source_block',a['source_block'])
        definition['collision'][a['id']]=list(pack['data']['tilesets'][source]['collision'][block])
    pack['data']['tilesets']['mn_border_pines']=definition


def inspect(design,layout,pack):
    assert layout['schema_version']==2 and layout['reject_cliff_cuts'] is True
    width,height=layout['dimensions_metatiles']
    for layer in layout['layers']:
        if layer['kind'] in {'polygon','cliff_polygon','path'}:
            points=layer['points'];ends=points[1:] if layer['kind']=='path' else points[1:]+points[:1]
            assert all(a[0]==b[0] or a[1]==b[1] for a,b in zip(points,ends)),layer['name']+': diagonal'
    prepare(pack,layout);blocks=layout_blocks(layout);after=list(blocks);cut=design['shortcut'];x,y=cut['tile'];after[y//2*width+x//2]=int(cut['replacement_block'],16)
    post_design=deepcopy(design);post_design['shortcut']=None
    report,post_kind,post_walk=check(post_design,layout,pack,after)
    collisions=pack['data']['tilesets'][layout['compiled_tileset']]['collision']
    def kind(x,y):return collisions[f'{blocks[y//2*width+x//2]:02x}'][y%2*2+x%2]
    actors=design['trainers']+design['residents']+design['items'];occupied={tuple(a['tile']) for a in actors}
    def allowed(opened=False,surf=False):
        k=post_kind if opened else kind
        return {(x,y) for y in range(height*2) for x in range(width*2) if k(x,y) in ({'FLOOR','TALL_GRASS'}|({'WATER'} if surf else set())) and (x,y) not in occupied}
    start,end=[tuple(v*2 for v in e['gate_metatile']) for e in design['endpoints']]
    pre_walk=flood(start,allowed());pre_surf=flood(start,allowed(False,True));post_surf=flood(start,allowed(True,True))
    assert end in pre_walk,'Main route requires Cut/Surf'
    assert kind(*cut['tile'])=='CUT_TREE' and tuple(cut['approach']) in pre_walk,'Cut tree or approach'
    rule=pack['data']['field_moves']['cut']['replacements']['forest'][int(cut['source_block'],16)]
    assert rule['replacement_block_id']==int(cut['replacement_block'],16)
    inventory={(x,y) for y in range(height*2) for x in range(width*2) if kind(x,y)=='CUT_TREE'}
    assert inventory=={tuple(cut['tile'])},'Decorative Cut canopy'
    a,b=tuple(cut['compare_from']),tuple(cut['compare_to'])
    before=shortest_walk(a,b,allowed());opened=shortest_walk(a,b,allowed(True));before_surf=shortest_walk(a,b,allowed(False,True))
    assert before and opened and len(before)-len(opened)>=cut['minimum_saved_steps'],'Cut is not a useful shorter walking link'
    assert len(before_surf)-len(opened)>=cut['minimum_saved_steps'],'Surf freely walks/lands around the Cut tree'
    assert all(kind(x+dx,y) not in {'FLOOR','TALL_GRASS','WATER'} for dx in [-1,1]),'Cut side opening'
    water={(x,y) for y in range(height*2) for x in range(width*2) if kind(x,y)=='WATER'}
    assert not any(x in {0,width*2-1} or y in {0,height*2-1} for x,y in water),'Unconnected outer water edge'
    outer_walk={p for p in allowed() if p[0] in {0,width*2-1} or p[1] in {0,height*2-1}}
    assert outer_walk=={(32,111),(33,111),(36,0),(37,0)},'Unplanned walking boundary opening'
    for crossing in layout['crossings']+layout['docks']:
        for x,y in crossing['north_rail_blocks']:assert [kind(x*2,y*2),kind(x*2,y*2+1)]==['WALL','FLOOR'],crossing['id']+': north rail'
        for x,y in crossing['south_rail_blocks']:assert [kind(x*2,y*2),kind(x*2,y*2+1)]==['FLOOR','WALL'],crossing['id']+': south rail'
        for x,y in crossing['water_under_rail_blocks']:assert kind(x*2,y*2)=='WATER',crossing['id']+': water below'
        for end_block in crossing.get('closed_end_blocks',[]):
            x,y=end_block['position'];assert blocks[y*width+x]==int(end_block['block'],16)
    bridge=layout['crossings'][0];cells={(x*2+dx,y*2+dy) for field in ['north_rail_blocks','south_rail_blocks'] for x,y in bridge[field] for dx in [0,1] for dy in [0,1]}
    assert end not in flood(start,allowed()-cells),'Timber crossing has a free walking bypass before Cut'
    report.update(status='Source terrain proposal; native maps, Cut persistence, room returns, services and SQ03 journey remain separate',pre_cut_walk_tiles=len(pre_walk),post_cut_walk_tiles=len(post_walk),pre_cut_walk_surf_tiles=len(pre_surf),post_cut_walk_surf_tiles=len(post_surf),cut_shortcut=dict(before_walk_steps=len(before)-1,after_walk_steps=len(opened)-1,saved_walk_steps=len(before)-len(opened),before_surf_steps=len(before_surf)-1,source_override='forest0f→17',native_status='Source comparison only'),outer_water_tiles=0,water_tiles=len(water),outer_walking_openings=sorted(outer_walk),native_block_varieties=len(set(blocks)),pre_cut_endpoint_steps=len(shortest_walk(start,end,allowed()))-1,post_cut_endpoint_steps=len(shortest_walk(start,end,allowed(True)))-1,native_ordinary_trainers=len(design['trainers']),rooms=[dict(id=r['id'],dimensions=r['dimensions_metatiles'],exits=r['exits'],safe_arrival=r['clear_arrival']) for r in design['interiors']])
    return report,blocks,after


def render(design,layout,pack,art):
    started=time.perf_counter();report,blocks,after=inspect(design,layout,pack);w,h=layout['dimensions_metatiles']
    def scene_for(native_blocks):
        im=render_stage(art.render('forest',w,h,native_blocks,derived=layout['derived_blocks']), 'convert', OUTPUT, 'review_border_pines-render-line85-11-convert', 'RGBA')
        for a in design['trainers']+design['residents']:im.alpha_composite(oriented_sprite(pack,a['sprite'],a['facing']),(a['tile'][0]*16,a['tile'][1]*16-4))
        for a in design['items']:im.alpha_composite(sprite(pack,'SPRITE_POKE_BALL'),(a['tile'][0]*16,a['tile'][1]*16-4))
        return im
    scene=scene_for(blocks);after_scene=scene_for(after);digest=hashlib.sha256(scene.tobytes()).hexdigest()[:8];adigest=hashlib.sha256(after_scene.tobytes()).hexdigest()[:8]
    OUTPUT.mkdir(parents=True,exist_ok=True);save_render(render_stage(scene, 'convert', OUTPUT, 'review_border_pines-render-line90-57-convert', 'RGB'), OUTPUT/'whole-route.png');save_render(render_stage(scene, 'convert', OUTPUT, 'review_border_pines-render-line90-117-convert', 'RGB'), OUTPUT/f'whole-route-{digest}.png');save_render(render_stage(after_scene, 'convert', OUTPUT, 'review_border_pines-render-line90-187-convert', 'RGB'), OUTPUT/f'after-cut-{adigest}.png')
    save_render(art.render('forest',w,h,blocks,derived=layout['derived_blocks']), OUTPUT/'terrain.png')
    overlay=scene.copy();draw=ImageDraw.Draw(overlay)
    for row in range(design['chunk_rows']):
        y=row*768;draw.rectangle((0,y,w*32-1,min(y+767,h*32-1)),outline='white',width=2);draw.text((6,y+6),f'MnR17C00R{row:02}',fill='white',stroke_fill='black',stroke_width=1)
    for a in design['trainers']+design['residents']+design['items']+design['signs']+design['doors']:
        x,y=a['tile'];draw.rectangle((x*16,y*16,x*16+15,y*16+15),outline='#ffc748',width=2);draw.text((x*16,y*16-12),a['id'],fill='white',stroke_fill='black',stroke_width=1)
    save_render(render_stage(overlay, 'convert', OUTPUT, 'review_border_pines-render-line97-16-convert', 'RGB'), OUTPUT/'anchors.png');views=[]
    for v in design['viewpoints']:
        x,y=v['tile'];im=render_stage(render_stage(scene, 'crop', OUTPUT, 'review_border_pines-render-line99-25-crop', (x*16-80,y*16-72,x*16+80,y*16+72)), 'convert', OUTPUT, 'review_border_pines-render-line99-25-convert', 'RGB');views.append((v['id'],im));save_render(render_stage(im, 'resize', OUTPUT, 'review_border_pines-render-line99-125-resize', (640,576),Image.Resampling.NEAREST), OUTPUT/f'viewport-{v["id"].lower()}.png')
    sheet=room_sheet(views,3);save_render(sheet, OUTPUT/'viewpoints.png');save_render(sheet, OUTPUT/f'viewpoints-{digest}.png')
    rooms=[]
    for room in design['interiors']:
        rw,rh=room['dimensions_metatiles'];tileset=pack['data']['maps'][room['template']]['attributes']['tileset_name'];im=render_stage(art.render(tileset,rw,rh,[int(b,16) for row in room['rows'] for b in row.split()]), 'convert', OUTPUT, 'review_border_pines-render-line103-123-convert', 'RGBA')
        for a in room.get('residents',[]):im.alpha_composite(oriented_sprite(pack,a['sprite'],a['facing']),(a['tile'][0]*16,a['tile'][1]*16-4))
        for a in room.get('items',[]):im.alpha_composite(sprite(pack,'SPRITE_POKE_BALL'),(a['tile'][0]*16,a['tile'][1]*16-4))
        save_render(render_stage(im, 'convert', OUTPUT, 'review_border_pines-render-line106-20-convert', 'RGB'), OUTPUT/f'room-{room["id"]}.png');rooms.append((room['id'],render_stage(im, 'convert', OUTPUT, 'review_border_pines-render-line106-97-convert', 'RGB')))
    save_render(room_sheet(rooms,2), OUTPUT/'rooms.png')
    x,y=design['shortcut']['approach'];box=(x*16-80,y*16-72,x*16+80,y*16+72)
    for name,im in [('before',scene),('after',after_scene)]:save_render(render_stage(render_stage(render_stage(im, 'crop', OUTPUT, 'review_border_pines-render-line109-72-crop', box), 'convert', OUTPUT, 'review_border_pines-render-line109-72-convert', 'RGB'), 'resize', OUTPUT, 'review_border_pines-render-line109-72-resize', (640,576),Image.Resampling.NEAREST), OUTPUT/f'cut-{name}-{digest}.png')
    report.update(image=f'whole-route-{digest}.png',after_cut_image=f'after-cut-{adigest}.png',render_ms=round((time.perf_counter()-started)*1000,1),revision=str(time.time_ns()))
    (OUTPUT/'revision.json').write_text(json.dumps(report,indent=2)+'\n');return report


def negative_checks(design,layout,pack):
    """Reject malformed copies without touching terrain, dossiers or packs."""
    cases={
        'Cut walking sidestep':dict(kind='rect',material='lawn',bounds=[17,34,18,36]),
        'Cut Surf sidestep':dict(kind='rect',material='water',bounds=[17,34,18,36]),
        'unexpected Cut canopy':dict(kind='point',position=[15,40],block='0f'),
        'outer water hole':dict(kind='point',position=[0,34],block='02'),
        'outer walking hole':dict(kind='point',position=[0,24],block='01'),
        'wrong timber north rail':dict(kind='point',position=[11,32],block='3a'),
        'free marsh crossing':dict(kind='path',material='lawn',width_metatiles=1,points=[[9,35],[17,35],[17,33],[15,33]]),
        'free dry sedge bypass':dict(kind='path',material='lawn',width_metatiles=1,points=[[12,47],[10,47],[10,43],[12,43]]),
        'fake hut door':dict(kind='point',position=[22,24],block='27'),
    }
    outcomes=[]
    for name,layer in cases.items():
        l=deepcopy(layout);l['layers'].append(dict(name='Negative '+name,**layer))
        try:
            report,*_=inspect(design,l,pack)
            assert not report['errors'],'; '.join(report['errors'])
        except AssertionError as error:outcomes.append(dict(mutation=name,rejected=True,reason=str(error) or 'Native timber rail mismatch'))
        else:raise AssertionError('Uncaught defect: '+name)
    d=deepcopy(design);d['interiors'][0]['exits']=[[8,13]]
    try:
        report,*_=inspect(d,layout,pack)
        assert not report['errors'],'; '.join(report['errors'])
    except AssertionError as error:outcomes.append(dict(mutation='Wrong cave return ladder',rejected=True,reason=str(error)))
    else:raise AssertionError('Uncaught cave return mismatch')
    art=SourceArt(pack);source=art.blocks('kanto')[0x77]
    assert art.render('forest',1,1,[0x3a],derived=layout['derived_blocks']).tobytes()==source.tobytes()
    arranged=Image.new('RGB',(32,32))
    for row,old in enumerate([2,3,0,1]):arranged.paste(source.crop((0,old*8,32,old*8+8)),(0,row*8))
    assert art.render('forest',1,1,[0x3b],derived=layout['derived_blocks']).tobytes()==arranged.tobytes()
    result=dict(source_only=True,negative_cases=outcomes,native_rail_source_rows_equal=True,
                layout_sha256=hashlib.sha256(LAYOUT.read_bytes()).hexdigest(),dossier_sha256=hashlib.sha256(DOSSIER.read_bytes()).hexdigest())
    (OUTPUT/'negative-checks.json').write_text(json.dumps(result,indent=2)+'\n');return result


PAGE='''<!doctype html><meta charset="utf-8"><title>Border Pines source review</title><style>body{background:#14212b;color:#eef;font:16px system-ui;margin:20px}img{image-rendering:pixelated;max-width:none}.scroll{overflow:auto;max-height:78vh}header{position:sticky;top:0;background:#14212bf5;padding:12px}button{padding:10px;margin:4px}</style><header><h1>Border Pines — full woodland route</h1><p>Terrain and static event planning. Native services, story, map seams and Cut persistence remain separate.</p><button data-layer="whole-route">Whole route</button><button data-layer="terrain">Terrain</button><button data-layer="anchors">Anchors</button><p id="status"></p></header><div class="scroll"><img id="map" width="1024"></div><h2>Player views</h2><img id="views"><h2>Native-source room proposals</h2><img id="rooms"><script>let layer='whole-route',revision='';function refresh(){document.querySelector('#map').src=layer+'.png?r='+revision;document.querySelector('#views').src='viewpoints.png?r='+revision;document.querySelector('#rooms').src='rooms.png?r='+revision}document.querySelectorAll('[data-layer]').forEach(b=>b.onclick=()=>{layer=b.dataset.layer;refresh()});setInterval(async()=>{let s=await(await fetch('/state',{cache:'no-store'})).json();document.querySelector('#status').textContent=s.ok?s.render_ms+' ms · '+s.status:'INVALID: '+s.error;if(revision!==s.revision){revision=s.revision;refresh()}},500)</script>'''


def main():
    parser=argparse.ArgumentParser();parser.add_argument('--pack',type=Path,default=ROOT/'content-packs/text-tui.crystalpack');parser.add_argument('--serve',action='store_true');parser.add_argument('--negative-checks',action='store_true');parser.add_argument('--port',type=int,default=8792);args=parser.parse_args()
    pack=load_pack(args.pack);art=SourceArt(pack);state={'value':{'ok':False,'error':'Initial render pending'}}
    if args.serve:
        OUTPUT.mkdir(parents=True,exist_ok=True);(OUTPUT/'index.html').write_text(PAGE)
        class Handler(SimpleHTTPRequestHandler):
            def log_message(self,*args):pass
            def do_GET(self):
                if self.path.split('?')[0]=='/state':
                    body=json.dumps(state['value']).encode();self.send_response(200);self.send_header('Content-Type','application/json');self.send_header('Content-Length',str(len(body)));self.end_headers();self.wfile.write(body)
                else:super().do_GET()
        server=ThreadingHTTPServer(('127.0.0.1',args.port),partial(Handler,directory=str(OUTPUT)));threading.Thread(target=server.serve_forever,daemon=True).start();print(f'Border Pines source review http://127.0.0.1:{args.port}/',flush=True)
    if args.negative_checks and args.serve:parser.error('--negative-checks requires one-shot review')
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
