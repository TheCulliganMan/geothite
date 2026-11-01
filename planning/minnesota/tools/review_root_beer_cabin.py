#!/usr/bin/env python3
"""Inspect exact native cabin pixels and collision; never publish content packs."""
from render_archive import save_render
import argparse
from collections import deque
from functools import partial
import hashlib
import io
from http.server import SimpleHTTPRequestHandler,ThreadingHTTPServer
import json
from pathlib import Path
import threading
import time

from PIL import Image,ImageDraw

from render_reference import SourceArt,load_pack,room_sheet
from review_north_shore import oriented_sprite

ROOT=Path(__file__).resolve().parents[3]
DESIGN=ROOT/'planning/minnesota/playable/root-beer-cabin.json'
OUTPUT=ROOT/'planning/minnesota/renders/root-beer-cabin'


def check(design,pack):
    assert design['id']=='MnBoundaryRootBeerCabin' and design['dimensions_metatiles']==[5,4],'Retain the actual cabin extent/identity'
    assert pack['data']['maps'][design['template']]['attributes']['tileset_name']==design['tileset']=='traditional_house','Template/tileset mismatch'
    width,height=design['dimensions_metatiles'];rows=design['rows']
    assert len(rows)==height and all(len(row.split())==width for row in rows),'Room rows'
    blocks=[int(b,16) for row in rows for b in row.split()];collision=pack['data']['tilesets'][design['tileset']]['collision']
    def kind(point):
        x,y=point;assert 0<=x<width*2 and 0<=y<height*2,'Core coordinate outside cabin'
        return collision[f'{blocks[y//2*width+x//2]:02x}'][y%2*2+x%2]
    assert collision[f'{design["border_block"]:02x}']==['WALL']*4,'Exterior room border must block movement'
    exits={tuple(p) for p in design['exits']};assert exits=={(5,7),(6,7)},'Both native carpet exits'
    actual_exits={(x,y) for y in range(height*2) for x in range(width*2) if kind((x,y))=='WARP_CARPET_DOWN'}
    assert actual_exits==exits,'Missing or unintended carpet exit'
    assert all(kind(p)=='WARP_CARPET_DOWN' for p in exits),'Exit collision'
    arrival=tuple(design['clear_arrival']);assert arrival==(5,6) and kind(arrival)=='FLOOR','Safe indoor FLOOR arrival'
    assert design['warp_contract']['interior_passive_arrival_warp']==dict(index=3,tile=[5,6],collision='FLOOR',active_trigger=False),'Passive arrival record'
    assert design['warp_contract']['interior_exit_warp_indices']==[1,2] and design['warp_contract']['exterior_door_target_warp']==3,'Exit/arrival indices differ'
    floor={(x,y) for y in range(height*2) for x in range(width*2) if kind((x,y))=='FLOOR'}
    def flood(occupied):
        seen={arrival};todo=deque([arrival])
        while todo:
            x,y=todo.popleft()
            for p in [(x-1,y),(x+1,y),(x,y-1),(x,y+1)]:
                if p not in floor or p in occupied or p in seen:continue
                seen.add(p);todo.append(p)
        return seen
    person=design['dorothy_presence']['serving_preview'];anchor=tuple(person['tile']);assert kind(anchor)=='FLOOR','Dorothy stands on furniture'
    base=flood(set());serving=flood({anchor})
    assert base==floor and serving==floor-{anchor},'Furniture or serving Dorothy strands walking floor'
    counter=design['service_counter'];approach=tuple(counter['customer_approach']);between=tuple(counter['interaction_counter_tile'])
    assert counter['customer_facing']=='left' and counter['dorothy_anchor']==person['tile'],'Counter facing/anchor'
    assert anchor==(approach[0]-2,approach[1]) and between==(approach[0]-1,approach[1]),'Actual two-tile counter interaction'
    assert kind(between)=='COUNTER' and approach in serving,'Serving counter/approach unreachable'
    assert all(kind(tuple(p))=='COUNTER' for p in counter['native_counter_tiles']),'Counter recipe collision'
    for x,y in exits:assert any(p in serving for p in [(x-1,y),(x+1,y),(x,y-1),(x,y+1)]),'Carpet exit has no clear approach'
    for label,lines in counter['dialogue'].items():assert lines and all(0<len(line)<=18 for line in lines),label+': native 18-column dialogue'
    assert not design['residents'],'Do not publish a second default Dorothy inside'
    assert design['dorothy_presence']['default_review_state']=='outside_present_inside_empty','Keep exterior Dorothy visible by default'
    service=design['hospitality_contract'];assert service['first_gift']['event_flag']=='EVENT_MN_ROOT_BEER_FIRST_GIFT' and service['purchase']['unit_price']==300 and service['consumption']['hp_restore_points']==60,'Keep the specified gift/drink contract'
    return blocks,kind,base,dict(room='MnBoundaryRootBeerCabin',reachable_floor_tiles=len(base),serving_floor_tiles=len(serving),carpet_exit_tiles=design['exits'],passive_arrival=design['clear_arrival'],counter_approach=list(approach),counter_tile=list(between),dorothy_serving_anchor=list(anchor),default_indoor_dorothys=0,conditional_serving_dorothys=1,source_pixels_modified=False,source_metatile_varieties=len(set(blocks)),status='Source pixels and static collision checked; service/warp publication and native input proof pending')


def render(design,pack,art):
    started=time.perf_counter();blocks,kind,walkable,report=check(design,pack);w,h=design['dimensions_metatiles']
    OUTPUT.mkdir(parents=True,exist_ok=True)
    terrain=art.render(design['tileset'],w,h,blocks)
    save_render(terrain, OUTPUT/'cabin-terrain-original.png')
    empty=terrain.convert('RGBA');serving=empty.copy();actor=design['dorothy_presence']['serving_preview']
    sprite_file='gfx/sprites/'+actor['sprite'].removeprefix('SPRITE_').lower()+'.png'
    sprite_source=Image.open(io.BytesIO(bytes(pack['runtime_files'][sprite_file])))
    save_render(sprite_source, OUTPUT/'dorothy-source-palette.png')
    sprite=oriented_sprite(pack,actor['sprite'],actor['facing'])
    save_render(sprite, OUTPUT/'dorothy-oriented-original-rgba.png')
    serving.alpha_composite(sprite,(actor['tile'][0]*16,actor['tile'][1]*16-4))
    for label,scene in [('empty',empty),('serving',serving)]:
        digest=hashlib.sha256(scene.tobytes()).hexdigest()[:8];save_render(scene, OUTPUT/f'cabin-{label}-original-rgba.png');save_render(scene.convert('RGB'), OUTPUT/f'cabin-{label}.png');save_render(scene.convert('RGB'), OUTPUT/f'cabin-{label}-{digest}.png')
        for factor in [2,4]:
            scaled=scene.resize((w*32*factor,h*32*factor),Image.Resampling.NEAREST)
            save_render(scaled, OUTPUT/f'cabin-{label}-{factor}x-original-rgba.png')
            save_render(scaled.convert('RGB'), OUTPUT/f'cabin-{label}-{factor}x.png')
        view=Image.new('RGBA',(160,144));tile=art.blocks(design['tileset'])[design['border_block']]
        for y in range(0,144,32):
            for x in range(0,160,32):view.paste(tile,(x,y))
        save_render(tile, OUTPUT/'viewport-border-tile-original.png')
        save_render(view, OUTPUT/f'viewport-{label}-border-original-rgba-160x144.png')
        save_render(view.convert('RGB'), OUTPUT/f'viewport-{label}-border-rgb-160x144.png')
        view.paste(scene,(0,8))
        save_render(view, OUTPUT/f'viewport-{label}-native-original-rgba-160x144.png')
        save_render(view.convert('RGB'), OUTPUT/f'viewport-{label}-native-rgb-160x144.png')
        preview=view.resize((640,576),Image.Resampling.NEAREST)
        save_render(preview, OUTPUT/f'viewport-{label}-preview-original-rgba-640x576.png')
        save_render(preview.convert('RGB'), OUTPUT/f'viewport-{label}.png')
        report[label+'_image']=f'cabin-{label}-{digest}.png'
    save_render(room_sheet([('DEFAULT: DOROTHY OUTSIDE, CABIN EMPTY',empty),('FUTURE ENTRY SCENE: SERVING INSIDE',serving)],columns=2), OUTPUT/'two-states.png')
    overlay=serving.copy();draw=ImageDraw.Draw(overlay,'RGBA')
    for y in range(h*2):
        for x in range(w*2):
            color=(30,210,120,55) if (x,y) in walkable else (255,90,60,80)
            draw.rectangle((x*16,y*16,x*16+15,y*16+15),fill=color)
    for label,point in [('ARRIVAL',[5,6]),('COUNTER',[4,3]),('A HERE',[5,3]),('DOROTHY',[3,3]),('EXIT 1',[5,7]),('EXIT 2',[6,7])]:
        x,y=point;draw.rectangle((x*16,y*16,x*16+15,y*16+15),outline='yellow',width=1)
    save_render(overlay, OUTPUT/'collision-original-rgba.png')
    enlarged_overlay=overlay.resize((640,512),Image.Resampling.NEAREST)
    save_render(enlarged_overlay, OUTPUT/'collision-4x-original-rgba.png')
    save_render(enlarged_overlay.convert('RGB'), OUTPUT/'collision.png')
    report.update(render_ms=round((time.perf_counter()-started)*1000,1),revision=str(time.time_ns()),ok=True,error='');(OUTPUT/'revision.json').write_text(json.dumps(report,indent=2)+'\n');return report


HTML='''<!doctype html><meta charset="utf-8"><title>Root Beer Cabin source review</title><style>body{background:#14212b;color:#edf4ee;font:16px system-ui;margin:24px}img{image-rendering:pixelated;max-width:none}button{margin:5px;padding:10px}p{max-width:950px}</style><h1>Isle of Pines — complete root beer cabin</h1><p>Native source pixels, 5×4 metatiles. Default keeps Dorothy outside; serving portrait is a separate future entry-scene preview. Item, shop, presence and warp contracts remain unimplemented.</p><button onclick="mode='empty';refresh()">Default empty cabin</button><button onclick="mode='serving';refresh()">Conditional serving preview</button><p id="status">Rendering…</p><img id="cabin"><h2>Both distinct scene states</h2><img id="states"><h2>Collision and exact approaches</h2><img id="collision"><script>let mode='empty',rev='';function refresh(){document.querySelector('#cabin').src='cabin-'+mode+'-4x.png?r='+rev;document.querySelector('#states').src='two-states.png?r='+rev;document.querySelector('#collision').src='collision.png?r='+rev}setInterval(async()=>{const s=await(await fetch('/state',{cache:'no-store'})).json();document.querySelector('#status').textContent=s.ok?s.render_ms+' ms · '+s.reachable_floor_tiles+' FLOOR cells · '+s.status:'INVALID: '+s.error;if(s.revision!==rev){rev=s.revision;refresh()}},300)</script>'''


def main():
    parser=argparse.ArgumentParser();parser.add_argument('--pack',type=Path,default=ROOT/'content-packs/text-tui.crystalpack');parser.add_argument('--serve',action='store_true');parser.add_argument('--watch',action='store_true');parser.add_argument('--port',type=int,default=8781);args=parser.parse_args()
    pack=load_pack(args.pack);art=SourceArt(pack);state={'value':dict(ok=False,error='Initial render pending')}
    if args.serve:
        OUTPUT.mkdir(parents=True,exist_ok=True);(OUTPUT/'index.html').write_text(HTML)
        class Handler(SimpleHTTPRequestHandler):
            def log_message(self,*args):pass
            def do_GET(self):
                if self.path.split('?')[0]=='/state':
                    body=json.dumps(state['value']).encode();self.send_response(200);self.send_header('Content-Type','application/json');self.send_header('Cache-Control','no-store');self.send_header('Content-Length',str(len(body)));self.end_headers();self.wfile.write(body)
                else:super().do_GET()
        server=ThreadingHTTPServer(('127.0.0.1',args.port),partial(Handler,directory=str(OUTPUT)));threading.Thread(target=server.serve_forever,daemon=True).start();print(f'Root Beer Cabin review: http://127.0.0.1:{args.port}/',flush=True)
    last=None
    while True:
        stamp=DESIGN.stat().st_mtime_ns
        if stamp!=last:
            try:state['value']=render(json.loads(DESIGN.read_text()),pack,art);print(json.dumps(state['value']),flush=True)
            except (AssertionError,KeyError,ValueError,IndexError,TypeError) as error:
                state['value']=dict(ok=False,error=str(error),revision=str(time.time_ns()));print('DESIGN ERROR: '+str(error),flush=True)
                if not(args.serve or args.watch):raise SystemExit(1)
            last=stamp
        if not(args.serve or args.watch):break
        time.sleep(.2)


if __name__=='__main__':main()
