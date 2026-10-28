#!/usr/bin/env python3
"""Fast town-only render loop. Reads source art once; never generates terrain or packs.

python review_grand_marais.py --watch --serve
Edit playable/grand-marais.json; preview refreshes without Cargo or a pack rebuild.
"""
from render_archive import save_render
import argparse
from collections import deque
import io
import json
from pathlib import Path
import re
import threading
import time
from http.server import SimpleHTTPRequestHandler, ThreadingHTTPServer
from functools import partial

import cbor2
from PIL import Image, ImageDraw
from render_reference import SourceArt, room_sheet

ROOT = Path(__file__).resolve().parents[3]
DESIGN = ROOT / 'planning/minnesota/playable/grand-marais.json'
OUTPUT = ROOT / 'planning/minnesota/renders/grand-marais'


def sprite(pack, token, archive_output=None, frame_offset=0):
    """Color the native frame; optionally retain every input/conversion locally."""
    # Other route renderers reuse this helper without writing into this town.
    def retain(image, suffix):
        if archive_output is not None:
            save_render(image, archive_output/(token.lower()+suffix))
    name = token.removeprefix('SPRITE_').lower().replace('-', '_')
    files = pack['runtime_files']
    file = 'gfx/sprites/' + name + '.png'
    if file not in files:
        # Mirror the production species-object resolver. These are original
        # menu-icon assets, not invented species art or a battle-image crop.
        assert f'gfx/pokemon/{name}/overworld.2bpp' not in files, 'custom overworld bank needs its own palette decoder'
        candidates = ['gfx/sprites/poke_ball.png'] if name == 'pokeball' else []
        candidates.append(f'gfx/icons/{name}.png')
        icon = pack['data']['menu_icons'].get(name.upper())
        if icon:
            candidates.append('gfx/icons/' + icon.lower().removeprefix('icon_') + '.png')
        file = next((path for path in candidates if path in files), file)
    original = Image.open(io.BytesIO(bytes(pack['runtime_files'][file])))
    # Native one/two-frame icons do not have a third directional frame.
    count = original.height // original.width
    assert original.width == 16 and count in {1, 2, 3, 6}, 'native 16px sprite sheet'
    if count == 1:
        frame_offset = 0
    elif count == 2 and frame_offset >= 32:
        frame_offset = 0
    retain(original, '-source-palette.png')
    original_crop = original.crop((0, frame_offset, 16, frame_offset+16))
    retain(original_crop, '-source-crop.png')
    sheet = original.convert('RGB')
    retain(sheet, '-source-rgb.png')
    source_crop = sheet.crop((0, frame_offset, 16, frame_offset+16))
    retain(source_crop, '-source-crop-rgb.png')
    art = source_crop.convert('RGBA')
    retain(art, '-before-day-palette-rgba.png')
    raw = bytes(pack['runtime_files']['gfx/overworld/npc_sprites.pal']).decode()
    day = raw.split('; day')[1].split('; nite')[0]
    colors = []
    for line in day.splitlines():
        if 'RGB' not in line: continue
        values = list(map(int, re.findall(r'\d+', line.split(';')[0])))
        colors.extend(tuple((v << 3) | (v >> 2) for v in values[i:i+3]) for i in range(0,len(values),3))
    palette_id = pack['data']['sprite_palette_defaults'].get(token, 0) & 7
    palette = colors[palette_id*4:palette_id*4+4]
    levels = [0 if p[0]>=213 else 1 if p[0]>=128 else 2 if p[0]>=43 else 3
              for p in art.getdata()]
    # Match native copy_source_sprite_rgba: enclosed paper pixels are opaque
    # eye/coat highlights; only the background connected to the border is clear.
    background, clear, pending = levels[0], set(), deque()
    def enqueue(x, y):
        index = y*16+x
        if index not in clear and levels[index] == background:
            clear.add(index); pending.append((x,y))
    for x in range(16):
        enqueue(x,0); enqueue(x,15)
    for y in range(1,15):
        enqueue(0,y); enqueue(15,y)
    while pending:
        x,y=pending.popleft()
        for nx,ny in ((x-1,y),(x+1,y),(x,y-1),(x,y+1)):
            if 0<=nx<16 and 0<=ny<16: enqueue(nx,ny)
    art.putdata([(0,0,0,0) if i in clear else (*palette[level],255)
                 for i,level in enumerate(levels)])
    retain(art, '-day-palette-rgba.png')
    return art


def validate(design, pack, blocks):
    w, h = design['width']*2, design['height']*2
    collision = pack['data']['tilesets'][design['tileset']]['collision']
    def kind(x,y):
        return collision[f'{blocks[(y//2)*design["width"]+x//2]:02x}'][(y%2)*2+x%2]
    walk = {(x,y) for y in range(h) for x in range(w) if kind(x,y) in {'FLOOR','TALL_GRASS','DOOR'}}
    actors = {tuple(a['tile']) for a in design['actors']}
    for a in design['actors']:
        assert tuple(a['tile']) in walk, f'{a["id"]} stands on {kind(*a["tile"])}'
        for lines in [a['lines'],a.get('signal_lines',[])]:
            assert all(len(line)<=18 for line in lines),f'{a["id"]}: fit authored dialogue lines to 18 characters'
    start = (design['doors'][0]['tile'][0], design['doors'][0]['tile'][1]+1)
    seen = {start}; queue = deque([start])
    doors = {tuple(d['tile']) for d in design['doors']}
    while queue:
        x,y = queue.popleft()
        for q in [(x+1,y),(x-1,y),(x,y+1),(x,y-1)]:
            if q in walk and q not in actors and q not in seen:
                seen.add(q)
                # Entering a door changes maps. Never route through a building facade.
                if q not in doors: queue.append(q)
    for d in design['doors']:
        assert kind(*d['tile']) == 'DOOR', f'{d["label"]} warp is not on a door'
        assert tuple(d['tile']) in seen, f'{d["label"]} is unreachable'
        assert (d['tile'][0],d['tile'][1]+1) in seen, f'{d["label"]} has no reachable arrival tile'
    for a in design['actors']:
        x,y=a['tile']
        assert any(q in seen for q in [(x+1,y),(x-1,y),(x,y+1),(x,y-1)]), f'{a["id"]} cannot be spoken to'
    for sign in design['signs']:
        x,y=sign['tile']
        assert all(len(line)<=18 for line in sign['lines']),f'{sign["label"]}: fit sign text to 18 characters'
        assert kind(x,y) == 'WALL', f'{sign["label"]} event is not on a physical sign'
        assert any(q in seen for q in [(x+1,y),(x-1,y),(x,y+1),(x,y-1)]), f'{sign["label"]} cannot be read'
    missing=walk-seen-actors
    assert not missing,f'disconnected walkable tiles: {sorted(missing)}'
    return {'reachable_tiles':len(seen),'doors':len(doors),'actors':len(actors),'status':'geometry passed; controller verification is separate'}, kind, seen


def render(design, pack, art):
    started = time.perf_counter()
    for entries in design['dialogue'].values():
        for label,lines in entries.items():
            assert lines and all(line.strip() and len(line)<=18 for line in lines),f'{label}: fit dialogue lines to 18 characters'
    blocks = [int(v,16) for row in design['rows'] for v in row.split()]
    assert len(blocks)==design['width']*design['height']
    report, kind, reachable = validate(design,pack,blocks)
    clean = art.render(design['tileset'],design['width'],design['height'],blocks)
    rooms=[]
    for room in design['interiors']:
        assert len(room['rows'])==room['height'],f'{room["map"]}: room height'
        assert all(len(row.split())==room['width'] for row in room['rows']),f'{room["map"]}: room width'
        template=pack['data']['maps'][room['template']]
        tileset=template['attributes']['tileset_name']
        tiles=[int(v,16) for row in room['rows'] for v in row.split()]
        for x,y in room['exits']:
            block=tiles[(y//2)*room['width']+x//2]
            assert pack['data']['tilesets'][tileset]['collision'][f'{block:02x}'][(y%2)*2+x%2]=='WARP_CARPET_DOWN',f'{room["map"]}: exit is off its carpet'
        pixels=art.render(tileset,room['width'],room['height'],tiles)
        save_render(pixels, OUTPUT/(room['map']+'-terrain-original.png'))
        rooms.append((room['map'].removeprefix('MnGrandMarais'),pixels))
    save_render(room_sheet(rooms,columns=2), OUTPUT/'interiors-terrain.png')
    save_render(clean, OUTPUT/'terrain.png')
    scene = clean.convert('RGBA')
    for a in design['actors']:
        s = sprite(pack,a['sprite'],OUTPUT/'sprite-inputs'); scene.alpha_composite(s,(a['tile'][0]*16,a['tile'][1]*16-4))
    save_render(scene, OUTPUT/'town-original-rgba.png')
    save_render(scene.convert('RGB'), OUTPUT/'town.png')
    collision = scene.copy(); draw = ImageDraw.Draw(collision,'RGBA')
    for y in range(design['height']*2):
        for x in range(design['width']*2):
            color = (25,180,110,50) if (x,y) in reachable else (255,60,60,70) if kind(x,y) not in {'WATER','FLOOR','TALL_GRASS'} else (0,0,0,0)
            draw.rectangle((x*16,y*16,x*16+15,y*16+15),fill=color)
    for d in design['doors']:
        x,y=d['tile'];draw.rectangle((x*16,y*16,x*16+15,y*16+15),outline=(255,210,50,255),width=2)
        draw.text((x*16,y*16-11),str(d['id']),fill=(255,255,255,255),stroke_width=1,stroke_fill=(0,0,0,255))
    save_render(collision, OUTPUT/'collision-original-rgba.png')
    save_render(collision.convert('RGB'), OUTPUT/'collision.png')
    # Native Game Boy viewport: 160x144 pixels, shown at integer 4x scale.
    doors = {d['target']:d['tile'] for d in design['doors']}
    actors = {a['id']:a['tile'] for a in design['actors']}
    station = doors['MN_GRAND_MARAIS_FIELD_STATION']
    lighthouse = doors['MN_GRAND_MARAIS_LIGHTHOUSE']
    harbor = actors['HarborChild']
    painter = actors['Painter']
    views = [('waterfront',(harbor[0],harbor[1]-1)),
             ('artist-point',(painter[0],painter[1]-1)),
             ('field-station',(station[0]+3,station[1]+3)),
             ('lighthouse',(lighthouse[0]+2,lighthouse[1]-1))]
    for name,at in views:
        x,y=at
        crop=scene.crop((x*16-80,y*16-72,x*16+80,y*16+72))
        save_render(crop, OUTPUT/(name+'-native-original-rgba-160x144.png'))
        save_render(crop.convert('RGB'), OUTPUT/(name+'-native-rgb-160x144.png'))
        preview=crop.resize((640,576),Image.Resampling.NEAREST)
        save_render(preview, OUTPUT/(name+'.png'))
        save_render(preview.convert('RGB'), OUTPUT/(name+'-preview-rgb-640x576.png'))
    report['render_ms']=round((time.perf_counter()-started)*1000,1)
    report['revision']=time.time_ns()
    (OUTPUT/'revision.json').write_text(json.dumps(report))
    return report


HTML='''<!doctype html><meta charset="utf-8"><title>Grand Marais • town review</title>
<style>body{margin:0;background:#101b25;color:#e8eef2;font:15px system-ui}header{position:sticky;top:0;padding:14px 24px;background:#142432;z-index:2;border-bottom:1px solid #355}h1{font-size:22px;margin:0 0 8px}button{padding:8px 14px;margin-right:8px;background:#294457;color:white;border:0;border-radius:5px;cursor:pointer}main{padding:24px}img{image-rendering:pixelated;max-width:none}#map{width:1792px}p{max-width:900px;color:#b9ccd8}.crops{display:flex;gap:18px;flex-wrap:wrap}.crops img{width:480px}#status{font:13px monospace;color:#99e1bb}.mapwrap{overflow:auto;border:1px solid #355;max-height:78vh}.labels{line-height:1.7}h2{font-size:18px}</style>
<header><h1>Grand Marais — one town, source-art review</h1><button onclick="mode='town';refresh()">Town</button><button onclick="mode='terrain';refresh()">Terrain only</button><button onclick="mode='collision';refresh()">Collision + doors</button><button onclick="zoom=zoom===2?1:2;document.querySelector('#map').style.width=(document.querySelector('#map').naturalWidth*zoom)+'px'">1× / 2×</button><span id="status">Loading…</span></header>
<main><p>Original hand-authored town. Exact source tiles and sprites. Six doors, working interiors, a local lighthouse delivery, and optional grass encounters. Planned trail connections are paused. This overview is a design preview; production playability is tested separately.</p><div class="mapwrap"><img id="map"></div><p class="labels">1 Field Station · 2 Pokémon Center · 3 Mart · 4 Folk School · 5 Fisher’s Cottage · 6 Lighthouse</p><h2>Game Boy framing • 160 × 144 native pixels</h2><div class="crops"><img data-crop="waterfront"><img data-crop="artist-point"><img data-crop="field-station"><img data-crop="lighthouse"></div><h2>Authored room geometry</h2><p>Furniture and exit carpets update from the same JSON. Residents and scripts are verified in the published pack.</p><img data-crop="interiors-terrain"></main>
<script>let mode='town',rev='',zoom=2;function refresh(){document.querySelector('#map').src=mode+'.png?r='+rev;document.querySelector('#map').onload=()=>document.querySelector('#map').style.width=(document.querySelector('#map').naturalWidth*zoom)+'px';document.querySelectorAll('[data-crop]').forEach(i=>i.src=i.dataset.crop+'.png?r='+rev)}setInterval(async()=>{let s=await(await fetch('revision.json',{cache:'no-store'})).json();if(s.revision!==rev){rev=s.revision;let label=document.querySelector('#status');if(s.error){label.textContent='INVALID DESIGN — showing previous render: '+s.error;label.style.color='#ff8888'}else{refresh();label.style.color='#99e1bb';label.textContent=s.render_ms+' ms render · '+s.doors+' doors · '+s.reachable_tiles+' reachable tiles'}}},300);</script>'''


def main():
    parser=argparse.ArgumentParser();parser.add_argument('--pack',type=Path,default=ROOT/'content-packs/text-tui.crystalpack');parser.add_argument('--watch',action='store_true');parser.add_argument('--serve',action='store_true');parser.add_argument('--port',type=int,default=8765);args=parser.parse_args()
    OUTPUT.mkdir(parents=True,exist_ok=True)
    pack=cbor2.loads(args.pack.read_bytes()[22:]);art=SourceArt(pack)
    (OUTPUT/'index.html').write_text(HTML)
    if args.serve:
        class QuietHandler(SimpleHTTPRequestHandler):
            def log_message(self,*args): pass
        server=ThreadingHTTPServer(('127.0.0.1',args.port),partial(QuietHandler,directory=str(OUTPUT)))
        threading.Thread(target=server.serve_forever,daemon=True).start()
        print(f'Review http://127.0.0.1:{args.port}/',flush=True)
    previous=None
    while True:
        changed=DESIGN.stat().st_mtime_ns
        if changed!=previous:
            try:print(json.dumps(render(json.loads(DESIGN.read_text()),pack,art)),flush=True)
            except (AssertionError,ValueError,KeyError,IndexError,TypeError) as error:
                print(f'Design error: {error}',flush=True)
                (OUTPUT/'revision.json').write_text(json.dumps({'revision':time.time_ns(),'error':str(error)}))
            previous=changed
        if not args.watch:break
        time.sleep(.15)


if __name__=='__main__':main()
