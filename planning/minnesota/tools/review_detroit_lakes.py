#!/usr/bin/env python3
"""Detroit Lakes source review: exact native art, no publication or train service."""
import argparse
from copy import deepcopy
from functools import partial
import hashlib
from http.server import SimpleHTTPRequestHandler, ThreadingHTTPServer
import json
from pathlib import Path
import threading
import time

from PIL import Image

from render_archive import save_render
from render_reference import SourceArt, layout_blocks, load_pack, room_sheet
from render_stages import render_stage
from review_boundary_basecamp import adjacent, flood
from review_north_shore import oriented_sprite

ROOT = Path(__file__).resolve().parents[3]
DESIGN = ROOT / 'planning/minnesota/towns/MnDetroitLakes.json'
LAYOUT = ROOT / 'planning/minnesota/towns/MnDetroitLakes.layout.json'
OUTPUT = ROOT / 'planning/minnesota/renders/detroit-lakes'
STATION = ROOT / 'planning/minnesota/transit/MnDetroitLakesStation.json'


def prepare(pack, layout):
    definition = deepcopy(pack['data']['tilesets']['johto'])
    for recipe in layout['derived_blocks']:
        source = recipe.get('collision_source_tileset', recipe['source_tileset'])
        block = recipe.get('collision_source_block', recipe['source_block'])
        definition['collision'][recipe['id']] = list(pack['data']['tilesets'][source]['collision'][block])
    pack['data']['tilesets'][layout['compiled_tileset']] = definition


def validate(design, layout, pack, blocks):
    assert layout['schema_version'] == 2 and layout['reject_cliff_cuts'] is True
    assert design['dimensions_metatiles'] == layout['dimensions_metatiles'] == [24, 18]
    assert design['id'] == layout['id'] == 'MnDetroitLakes'
    for layer in layout['layers']:
        if layer['kind'] in {'polygon', 'cliff_polygon', 'path'}:
            ps = layer['points']
            ends = ps[1:] if layer['kind'] == 'path' else ps[1:] + ps[:1]
            assert all(a[0] == b[0] or a[1] == b[1] for a, b in zip(ps, ends)), layer['name']
    w, h = layout['dimensions_metatiles']
    collision = pack['data']['tilesets'][layout['compiled_tileset']]['collision']
    def kind(p):
        x, y = p
        return collision[f'{blocks[y//2*w+x//2]:02x}'][y%2*2+x%2]
    cells = {(x, y) for y in range(h*2) for x in range(w*2)}
    actors = {tuple(a['tile']) for a in design['residents']}
    assert len(actors) == len(design['residents']), 'overlapping town actors'
    walk = {p for p in cells if kind(p) in {'FLOOR', 'TALL_GRASS', 'DOOR'}}
    water = {p for p in cells if kind(p).startswith('WATER') and kind(p) != 'WATERFALL'}
    doors = {tuple(d['tile']) for d in design['doors']}
    seen = flood((6, 18), walk-actors, doors)
    surf = flood((6, 18), (walk|water)-actors, doors)
    for a in design['residents']:
        assert kind(tuple(a['tile'])) == 'FLOOR', a['id']+' actor off FLOOR'
        assert set(adjacent(tuple(a['tile']))) & seen, a['id']+' has no free approach'
        assert all(0 < len(line) <= 18 for line in a['lines']), a['id']+' text width'
    for s in design['signs']:
        assert kind(tuple(s['tile'])) == 'WALL', s['id']+' sign off WALL'
        assert set(adjacent(tuple(s['tile']))) & seen, s['id']+' cannot be read'
        assert all(0 < len(line) <= 18 for line in s['lines']), s['id']+' text width'
    assert len(design['ports']) == 2, 'exact two geographically valid town ports'
    expected = [('MN_R15', 'E', [22,9], [20,9]), ('MN_R16','W',[1,9],[3,9])]
    for p, (route, side, gate, landing) in zip(design['ports'], expected):
        assert (p['route'],p['side'],p['gate_metatile'],p['landing_metatile']) == (route,side,gate,landing), route+' changed reserved port'
        assert all(tuple(2*v for v in a) in seen for a in (gate,landing)), route+' disconnected'
    native_doors = {p for p in cells if kind(p) == 'DOOR'}
    assert native_doors == doors, ('unbound or false native facade DOOR',native_doors,doors)
    indices = []
    for d in design['doors']:
        assert tuple(d['tile']) in seen and kind(tuple(d['tile'])) == 'DOOR', d['id']+' inactive door'
        assert tuple(d['return_tile']) in seen and kind(tuple(d['return_tile'])) == 'FLOOR', d['id']+' unsafe return'
        assert d['arrival_tile'] == d['return_tile'] == [d['tile'][0],d['tile'][1]+1]
        assert d['target_warp'] == 3, 'entry must target passive room FLOOR3'
        indices.extend([d['warp_index'],d['return_warp_index']])
    assert indices == list(range(1,11)), 'one native town map needs exact active/passive warp order'
    assert not any(kind(p) == 'CUT_TREE' for p in cells), 'incidental decorative Cut canopy'
    outer = {p for p in cells if p[0] in {0,47} or p[1] in {0,35}}
    assert not water & outer, 'unconnected outer lake WATER needs real rocks'
    assert seen & outer == {(0,18),(0,19),(47,18),(47,19)}, ('unplanned walking edge',seen&outer)
    dock = design['public_dock']
    assert tuple(dock['launch_tile']) in seen and tuple(dock['water_target']) in water, 'beach Surf launch'
    assert tuple(dock['dock_approach']) in seen, 'dock is not connected to cabin street'
    for x,y in dock['north_rail_blocks']:
        assert [kind((2*x,2*y)),kind((2*x,2*y+1))] == ['WALL','FLOOR'], 'dock north rail'
    for x,y in dock['south_rail_blocks']:
        assert [kind((2*x,2*y)),kind((2*x,2*y+1))] == ['FLOOR','WALL'], 'dock south rail'
    for view in design['viewpoints']:
        assert tuple(view['tile']) in seen or tuple(view['tile']) in actors, view['id']+' viewpoint is unreachable'
    return dict(source_only=True,reachable_walking_tiles=len(seen),reachable_water_tiles=len(water&surf),native_facade_doors=len(doors),outer_water_cells=0,two_real_ports_retained=True,incidental_cut=0,train_service_installed=False,room_publication_pending=False),kind



def rooms_for(design):
    station = json.loads(STATION.read_text())
    reserved=next(r for r in design['interiors'] if r['id']==station['id'])
    assert station['dimensions_metatiles']==reserved['dimensions_metatiles'], 'station reservation drift'
    assert station['native_constant']==reserved['native_constant'], 'station identity drift'
    return [station if r['id'] == station['id'] else r for r in design['interiors']]


def validate_rooms(design, pack):
    results=[]
    assert len(design['interiors'])==5 and {r['id'] for r in design['interiors']}=={d['target'] for d in design['doors']}, 'unbound room reservation'
    for room in rooms_for(design):
        w,h=room['dimensions_metatiles']
        rows=[row.split() for row in room['rows']]
        assert len(rows)==h and all(len(row)==w for row in rows), room['id']+' room dimensions'
        blocks=[int(b,16) for row in rows for b in row]
        cs=pack['data']['tilesets'][room['tileset']]['collision']
        def kind(p):
            x,y=p
            return cs[f'{blocks[y//2*w+x//2]:02x}'][y%2*2+x%2]
        cells={(x,y) for y in range(h*2) for x in range(w*2)}
        actors={tuple(a['tile']) for a in room['residents']}
        assert len(actors)==len(room['residents'])
        walk={p for p in cells if kind(p) in {'FLOOR','WARP_CARPET_DOWN','DOOR'}}
        arrival=tuple(room['passive_arrival']['tile'])
        assert room['passive_arrival']['index']==3 and kind(arrival)=='FLOOR',room['id']+' unsafe passive arrival'
        seen=flood(arrival,walk-actors)
        carpets={p for p in cells if kind(p)=='WARP_CARPET_DOWN'}
        exits={tuple(e['tile']) for e in room['exits']}
        assert carpets==exits and len(exits)==2 and exits<=seen,room['id']+' unbound/blocked carpets'
        exterior=next(d for d in design['doors'] if d['target']==room['id'])
        assert all(e['target']=='MnDetroitLakes' and e['target_warp']==exterior['return_warp_index'] for e in room['exits']),room['id']+' wrong carpet return'
        for a in room['residents']:
            assert kind(tuple(a['tile']))=='FLOOR',a['id']+' off FLOOR'
            assert all(0<len(line)<=18 for line in a['lines']),a['id']+' dialogue width'
            if 'counter_tile' in a:
                assert kind(tuple(a['counter_tile']))=='COUNTER' and tuple(a['approach']) in seen,a['id']+' counter service approach'
            else:
                assert set(adjacent(tuple(a['tile'])))&seen,a['id']+' inaccessible actor'
        for sign in room.get('signs',[]):
            assert kind(tuple(sign['tile']))=='WALL' and tuple(sign['approach']) in seen,sign['id']+' sign placement'
            assert all(0<len(line)<=18 for line in sign['lines'])
        if 'pc' in room:
            assert kind(tuple(room['pc']['tile']))=='PC' and tuple(room['pc']['approach']) in seen,room['id']+' PC approach'
        if 'boarding' in room:
            b=room['boarding'];doors={p for p in cells if kind(p)=='DOOR'}
            assert doors=={tuple(b['train_door_core']),tuple(b['other_train_door_core'])}
            assert not doors&seen,'conductor can be bypassed to either native train door'
            opened=flood(arrival,walk-(actors-{tuple(b['conductor_core'])}))
            assert doors<=opened,'native platform throat does not reach both train doors'
            vectors={'up':(0,-1),'down':(0,1),'left':(-1,0),'right':(1,0)}
            def steps(start,commands):
                at=tuple(start)
                for cmd in commands:
                    dx,dy=vectors[cmd];at=(at[0]+dx,at[1]+dy)
                    assert at in walk,'station authored movement leaves actual walkable floor'
                return at
            conductor=steps(b['conductor_core'],b['conductor_open_steps'])
            assert steps(conductor,b['conductor_restore_steps'])==tuple(b['conductor_core'])
            assert steps(b['approach_core'],b['player_board_steps'])==tuple(b['train_door_core'])
        results.append(dict(id=room['id'],dimensions=[w,h],passive_arrival=list(arrival),carpets=sorted(exits),reachable_public_tiles=len(seen),source_only=True))
    return results


def negative_checks(design,layout,pack):
    cases=[]
    def reject(name,mutate):
        dd,ll=deepcopy(design),deepcopy(layout);mutate(dd,ll)
        try:
            prepare(pack,ll);validate(dd,ll,pack,layout_blocks(ll));validate_rooms(dd,pack)
        except AssertionError as error:
            cases.append(dict(case=name,rejected_by=str(error)));return
        raise AssertionError('negative accepted: '+name)
    reject('false facade door',lambda d,l:d['doors'][0].update(tile=[10,9]))
    reject('unsafe exterior return',lambda d,l:d['doors'][0].update(return_tile=[11,9]))
    reject('broken reserved port',lambda d,l:d['ports'][0].update(gate_metatile=[21,9]))
    reject('outer Surf escape',lambda d,l:l['layers'].append(dict(name='invalid lake hole',kind='point',position=[0,14],block='35')))
    reject('broken protective dock rail',lambda d,l:l['layers'].append(dict(name='invalid rail gap',kind='point',position=[10,13],block='71')))
    reject('unsafe room arrival',lambda d,l:d['interiors'][0]['passive_arrival'].update(tile=[0,0]))
    reject('wrong carpet return',lambda d,l:d['interiors'][1]['exits'][0].update(target_warp=1))
    reject('blocked PC approach',lambda d,l:d['interiors'][0]['pc'].update(approach=[0,0]))
    return cases


def render(design, layout, pack):
    started = time.perf_counter()
    prepare(pack, layout)
    art = SourceArt(pack)
    blocks = layout_blocks(layout)
    report, _ = validate(design, layout, pack, blocks)
    report['rooms']=validate_rooms(design,pack)
    report['rejected_adversarial_cases']=negative_checks(design,layout,pack)
    w,h = layout['dimensions_metatiles']
    terrain = art.render('johto',w,h,blocks,derived=layout['derived_blocks'])
    save_render(terrain,OUTPUT/'terrain-original.png')
    scene = render_stage(terrain,'convert',OUTPUT,'town-scene-rgba','RGBA')
    for a in design['residents']:
        scene.alpha_composite(oriented_sprite(pack,a['sprite'],a['facing']),(a['tile'][0]*16,a['tile'][1]*16-4))
    save_render(scene,OUTPUT/'whole-town-original-rgba.png')
    digest = hashlib.sha256(scene.tobytes()).hexdigest()[:10]
    rgb = render_stage(scene,'convert',OUTPUT,'whole-town-rgb','RGB')
    save_render(rgb,OUTPUT/'whole-town.png')
    save_render(rgb,OUTPUT/f'whole-town-{digest}.png')
    views=[]
    for v in design['viewpoints']:
        x,y=v['tile']; box=(max(0,min(x*16-80,w*32-160)),max(0,min(y*16-72,h*32-144)))
        crop=render_stage(scene,'crop',OUTPUT,v['id'].lower()+'-crop',(box[0],box[1],box[0]+160,box[1]+144))
        save_render(crop,OUTPUT/f'viewport-{v["id"].lower()}-native160.png')
        views.append((v['id'],crop))
        preview=render_stage(crop,'resize',OUTPUT,v['id'].lower()+'-4x',(640,576),Image.Resampling.NEAREST)
        save_render(preview,OUTPUT/f'viewport-{v["id"].lower()}-4x.png')
    save_render(room_sheet(views,3),OUTPUT/'viewpoints.png')
    room_images=[]
    for room in rooms_for(design):
        rw,rh=room['dimensions_metatiles'];bs=[int(b,16) for row in room['rows'] for b in row.split()]
        raw=art.render(room['tileset'],rw,rh,bs)
        save_render(raw,OUTPUT/f'room-{room["id"]}-terrain-original.png')
        im=render_stage(raw,'convert',OUTPUT,room['id']+'-rgba','RGBA')
        for a in room['residents']:
            im.alpha_composite(oriented_sprite(pack,a['sprite'],a['facing']),(a['tile'][0]*16,a['tile'][1]*16-4))
        save_render(im,OUTPUT/f'room-{room["id"]}-original-rgba.png')
        rhash=hashlib.sha256(im.tobytes()).hexdigest()[:10]
        save_render(im,OUTPUT/f'room-{room["id"]}-{rhash}.png')
        room_images.append((room['id'],im))
        for v in room.get('viewpoints',[]):
            x,y=v['tile'];bx=max(0,min(x*16-80,rw*32-160));by=max(0,min(y*16-72,rh*32-144))
            crop=render_stage(im,'crop',OUTPUT,room['id']+'-'+v['id']+'-native160',(bx,by,bx+160,by+144))
            save_render(crop,OUTPUT/f'room-{room["id"]}-{v["id"]}-native160.png')
    save_render(room_sheet(room_images,2),OUTPUT/'rooms.png')
    report.update(image=f'whole-town-{digest}.png' ,render_ms=round((time.perf_counter()-started)*1000,1))
    (OUTPUT/'revision.json').write_text(json.dumps(report,indent=2)+'\n')
    return report


def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--pack',type=Path,default=ROOT/'content-packs/text-tui.crystalpack')
    parser.add_argument('--serve',action='store_true')
    parser.add_argument('--port',type=int,default=8798)
    args=parser.parse_args();pack=load_pack(args.pack)
    OUTPUT.mkdir(parents=True,exist_ok=True)
    html='''<!doctype html><meta charset=utf-8><title>Detroit Lakes source review</title><style>body{background:#14212a;color:#eef;font:16px system-ui;margin:24px}img{image-rendering:pixelated;max-width:100%}</style><h1>Detroit Lakes — full source candidate</h1><p>Native town, rooms, station and service remain uninstalled.</p><img id=town src=whole-town.png><h2>Native player views</h2><img id=views src=viewpoints.png><h2>All five source interiors</h2><img id=rooms src=rooms.png><script>setInterval(()=>{let t=Date.now();town.src='whole-town.png?'+t;views.src='viewpoints.png?'+t;rooms.src='rooms.png?'+t},1200)</script>'''
    (OUTPUT/'index.html').write_text(html)
    if args.serve:
        class Handler(SimpleHTTPRequestHandler):
            def log_message(self,*args):pass
        server=ThreadingHTTPServer(('127.0.0.1',args.port),partial(Handler,directory=str(OUTPUT)))
        threading.Thread(target=server.serve_forever,daemon=True).start()
        print(f'Detroit source review http://127.0.0.1:{args.port}/',flush=True)
    previous=None
    while True:
        current=(DESIGN.stat().st_mtime_ns,LAYOUT.stat().st_mtime_ns,STATION.stat().st_mtime_ns)
        if previous!=current:
            try:print(json.dumps(render(json.loads(DESIGN.read_text()),json.loads(LAYOUT.read_text()),pack)),flush=True)
            except (AssertionError,KeyError,ValueError,IndexError) as e:
                print('DESIGN ERROR: '+str(e),flush=True)
                if not args.serve:raise SystemExit(1)
            previous=current
        if not args.serve:break
        time.sleep(.3)


if __name__=='__main__':main()
