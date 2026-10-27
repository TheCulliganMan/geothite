#!/usr/bin/env python3
"""Whole preserve terrain proposal; no Safari runtime or game pack is created."""
from render_archive import save_render
from render_stages import render_stage
import argparse
from collections import deque
from copy import deepcopy
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
from review_gunflint import flood, neighbors
from ledge_paths import inspect_ledges, shortest_walk, draw_ledge_paths

ROOT = Path(__file__).resolve().parents[3]
LAYOUT = ROOT / 'planning/minnesota/zones/boundary-waters-safari.layout.json'
CONCEPT = ROOT / 'planning/minnesota/zones/boundary-waters-safari.json'
OUTPUT = ROOT / 'planning/minnesota/renders/boundary-safari'


def prepare(pack, layout):
    """Local collision view of root-owned Rust art imports; no pack publication."""
    prepare_headwaters(pack, layout)
    definition = deepcopy(pack['data']['tilesets']['mn_headwaters'])
    for entry in layout['derived_blocks']:
        source = entry.get('collision_source_tileset', entry['source_tileset'])
        block = entry.get('collision_source_block', entry['source_block'])
        definition['collision'][entry['id']] = list(pack['data']['tilesets'][source]['collision'][block])
    pack['data']['tilesets']['mn_boundary_safari'] = definition


def inspect(layout, pack):
    assert layout['schema_version'] == 2 and layout['reject_cliff_cuts'] is True
    width, height = layout['dimensions_metatiles']
    for layer in layout['layers']:
        if layer['kind'] not in {'polygon', 'cliff_polygon', 'path'}:
            continue
        points = layer['points']
        edges = zip(points, points[1:]) if layer['kind'] == 'path' else zip(points, points[1:] + points[:1])
        assert all(a[0] == b[0] or a[1] == b[1] for a, b in edges), layer['name'] + ': diagonal edge'
    prepare(pack, layout)
    blocks = layout_blocks(layout)
    collisions = pack['data']['tilesets'][layout['compiled_tileset']]['collision']
    def kind(x, y, opened=False):
        block = blocks[y // 2 * width + x // 2]
        for gate in layout['cut_gates']:
            if opened and (x // 2, y // 2) == (gate['tile'][0] // 2, gate['tile'][1] // 2):
                block = int(gate['replacement_block'], 16)
        return collisions[f'{block:02x}'][y % 2 * 2 + x % 2]
    actors = layout['retire_anchors'] + layout.get('landmark_actors', [])
    decorations = layout.get('decorations', [])
    occupied = {tuple(a['tile']) for a in actors + decorations}
    def allowed(opened=False, surf=False):
        kinds = {'FLOOR', 'TALL_GRASS'} | ({'WATER'} if surf else set())
        return {(x, y) for y in range(height * 2) for x in range(width * 2)
                if kind(x, y, opened) in kinds and (x, y) not in occupied}
    start = tuple(layout['entry_contract']['arrival_tile'])
    assert kind(*start) == 'FLOOR'
    walking = flood(start, allowed())
    opened = flood(start, allowed(True))
    surfing = flood(start, allowed(False, True))
    all_open = flood(start, allowed(True, True))
    errors = []
    for actor in actors:
        tile = tuple(actor['tile'])
        if kind(*tile) != 'FLOOR': errors.append(actor['id'] + ': ranger not on FLOOR')
        if tuple(actor['approach']) not in all_open: errors.append(actor['id'] + ': unsafe return approach')
        if not all(0 < len(line) <= 18 for line in actor['lines']): errors.append(actor['id'] + ': text width')
        if tile[0] % 64 in {0, 63} or tile[1] % 48 in {0, 47}: errors.append(actor['id'] + ': actor on seam')
    for decoration in decorations:
        assert kind(*decoration['tile']) == 'FLOOR', decoration['id'] + ': stone must stand on clearing'
    camp_checks = []
    for camp in layout.get('campsites', []):
        assert tuple(camp['fire_ring']['center_tile']) in all_open, camp['id'] + ': fire ring center unreachable'
        assert kind(*camp['landing']) == 'FLOOR' and kind(*camp['water']) == 'WATER', camp['id'] + ': invalid landing'
        assert abs(camp['landing'][0]-camp['water'][0])+abs(camp['landing'][1]-camp['water'][1]) == 1
        assert tuple(camp['landing']) in surfing, camp['id'] + ': camp should not require Cut'
        x,y=camp['bench'];assert blocks[y*width+x]==0x83, camp['id']+': native wooden seat missing'
        marker=camp['interaction_contract']['tile']
        assert kind(*marker)=='WALL' and tuple(camp['interaction_contract']['approach']) in surfing, camp['id']+': inaccessible camp marker'
        assert tuple(camp['interaction_contract']['approach']) in neighbors(tuple(marker)), camp['id']+': nonadjacent marker approach'
        assert len(camp['fire_ring']['stone_tiles'])==4 and all(tuple(p) in occupied for p in camp['fire_ring']['stone_tiles'])
        camp_checks.append({'id':camp['id'],'landing':camp['landing'],'water':camp['water'],
                            'marker':marker,'unlit_stones':camp['fire_ring']['stone_tiles'],
                            'native_bench_block':'83 = Kanto08','center_reachable':True})
    if 'root_beer_lady' in layout:
        cabin = layout['root_beer_lady']['door']
        assert kind(*cabin['tile']) == 'DOOR', 'Cabin requires a real source DOOR'
        assert kind(*cabin['return_tile']) == 'FLOOR' and tuple(cabin['return_tile']) in all_open, 'Unsafe cabin return'
    for sign in layout['signs']:
        tile = tuple(sign['tile'])
        if kind(*tile) != 'WALL' or not any(p in all_open for p in neighbors(tile)): errors.append(sign['id'] + ': sign quadrant/approach')
    entry_gate = layout['entry_gate']
    gate_tiles = {tuple(tile) for tile in entry_gate['gate_tiles']}
    assert all(kind(*tile) == 'FLOOR' for tile in gate_tiles), 'Entry gate throat must be walkable'
    for fence in entry_gate['fence_blocks']:
        x,y = fence['position']
        assert blocks[y*width+x] == int(fence['block'],16), 'Entry gate fence/post missing'
    court = tuple(entry_gate['forecourt_tile'])
    inside = tuple(entry_gate['inside_tile'])
    assert court in walking and inside in walking, 'Entry gate must connect the arrival court'
    assert inside not in flood(court, allowed()-gate_tiles), 'Entry gate can be walked around'
    assert tuple(entry_gate['launch_target']) not in flood(court, allowed()-gate_tiles), 'Entry gate fails to control launch approach'
    assert tuple(entry_gate['launch_target']) not in flood(court, allowed(True, True)-gate_tiles), 'Entry gate has a Surf or opened-Cut bypass'
    for gate in layout['cut_gates']:
        assert kind(*gate['tile']) == 'CUT_TREE' and tuple(gate['approach']) in surfing
        rule = pack['data']['field_moves']['cut']['replacements']['johto'][int(gate['source_block'], 16)]
        assert rule['replacement_block_id'] == int(gate['replacement_block'], 16)
        assert all(tuple(t) not in surfing and tuple(t) in all_open for t in gate['targets']), 'Cut does not seal rare thicket against walking/Surf'
    cut_inventory = {(x, y) for y in range(height * 2) for x in range(width * 2) if kind(x, y) == 'CUT_TREE'}
    assert cut_inventory == {tuple(g['tile']) for g in layout['cut_gates']}, 'Undeclared decorative Cut'
    for landing in layout['surf_landings']:
        assert kind(*landing['land']) == 'FLOOR' and kind(*landing['water']) == 'WATER', landing['id'] + ': native Surf landing geometry ' + str([(p,kind(*landing[p])) for p in ['land','water']])
        assert tuple(landing['habitat_target']) not in opened and tuple(landing['habitat_target']) in all_open, 'Quiet Island must require Surf'
    for dock in layout['docks']:
        for x, y in dock['north_rail_blocks']: assert [kind(x * 2, y * 2), kind(x * 2, y * 2 + 1)] == ['WALL', 'FLOOR'], dock['id'] + ': north rail'
        for x, y in dock['south_rail_blocks']: assert [kind(x * 2, y * 2), kind(x * 2, y * 2 + 1)] == ['FLOOR', 'WALL'], dock['id'] + ': south rail'
        for x, y in dock['water_under_rail_blocks']: assert kind(x * 2, y * 2) == 'WATER', dock['id'] + ': missing water below'
        for end in dock.get('closed_end_blocks', []):
            x,y=end['position'];assert blocks[y*width+x]==int(end['block'],16), dock['id']+': walking end closure missing'
        for land, water in zip(dock.get('launch_tiles', []), dock.get('water_targets', [])):
            assert kind(*land) == 'FLOOR' and kind(*water) == 'WATER', dock['id'] + ': Surf mouth blocked'
    for ledge in layout['one_way_ledges']:
        x0, y0, x1, y1 = ledge['bounds_core']
        assert all(kind(x, y0) == 'HOP_DOWN' and kind(x, y0 + 1) == 'WALL' for x in range(x0, x1)), 'Native ledge face'
        assert all(kind(*p) == 'WALL' for p in ledge['side_barriers']), 'Ledge side closure'
    returns = inspect_ledges(layout, kind, all_open)
    water = {(x, y) for y in range(height * 2) for x in range(width * 2) if kind(x, y) == 'WATER'}
    assert not any(x in {0, width * 2 - 1} or y in {0, height * 2 - 1} for x, y in water), 'Unconnected outer Surf exit'
    assert all(kind(width*2-1,y) in {'BUOY','WALL'} and kind(0,y) in {'BUOY','WALL'} for y in range(height*2)), 'Visible east/west BUOY boundary must block'
    assert all(kind(x,height*2-1) in {'BUOY','WALL'} and kind(x,0) in {'BUOY','WALL'} for x in range(width*2)), 'Visible north/south BUOY boundary must block'
    for view in layout['viewpoints']:
        if tuple(view['tile']) not in all_open: errors.append(view['id'] + ': unreachable camera')
    remaining = set(water)
    water_ids, components = {}, []
    while remaining:
        component = flood(min(remaining), water)
        number = len(components)
        for point in component: water_ids[point] = number
        remaining -= component
        components.append({'id': number, 'water_tiles': len(component),
                           'bounds_core': [min(p[0] for p in component), min(p[1] for p in component),
                                           max(p[0] for p in component)+1, max(p[1] for p in component)+1]})
    portages = []
    for portage in layout['portages']:
        west, east = tuple(portage['west_land']), tuple(portage['east_land'])
        ww, ew = tuple(portage['west_water']), tuple(portage['east_water'])
        assert kind(*west) == kind(*east) == 'FLOOR', portage['id'] + ': unsafe landing'
        assert ww in water_ids and ew in water_ids, portage['id'] + ': missing lake approach'
        water_route = shortest_walk(ww, ew, water)
        if portage.get('water_relation') == 'connected_arms_shortcut':
            assert water_route and len(water_route)-1 >= portage['minimum_water_detour_steps'], portage['id'] + ': insignificant paddle detour'
        else:
            assert water_ids[ww] != water_ids[ew], portage['id'] + ': not a real water-separated portage'
        assert abs(west[0]-ww[0])+abs(west[1]-ww[1]) == 1 and abs(east[0]-ew[0])+abs(east[1]-ew[1]) == 1, portage['id'] + ': dismount/relaunch not adjacent'
        route = shortest_walk(west, east, allowed())
        assert route, portage['id'] + ': land portage disconnected'
        x0,y0,x1,y1 = portage['grass_bounds_core']
        band = {(x,y) for y in range(y0,y1) for x in range(x0,x1)}
        assert all(kind(*p) == 'TALL_GRASS' for p in band), portage['id'] + ': habitat neck is not grass'
        assert shortest_walk(west,east,allowed()-band) is None, portage['id'] + ': free grass bypass'
        land_graph = allowed()
        costs, todo = {west:0}, deque([west])
        while todo:
            at = todo.popleft()
            for point in neighbors(at):
                if point not in land_graph: continue
                value = costs[at] + int(point in band)
                if value < costs.get(point,10**9):
                    costs[point] = value
                    (todo.append if point in band else todo.appendleft)(point)
        assert costs[east] >= portage['minimum_grass_steps'], portage['id'] + ': short grass neck'
        portages.append({'id':portage['id'],'ordinary_land_steps':len(route)-1,
                         'minimum_grass_steps':costs[east], 'water_component_pair':[water_ids[ww],water_ids[ew]],
                         'alternative_paddle_steps':len(water_route)-1 if water_route else None})
    itineraries=[]
    for itinerary in layout.get('sample_expeditions',[]):
        steps=0
        for a,b in zip(itinerary['waypoints'],itinerary['waypoints'][1:]):
            route=shortest_walk(tuple(a),tuple(b),all_open)
            assert route,itinerary['id']+': unreachable stop'
            steps += len(route)-1
        itineraries.append({'id':itinerary['id'],'conservative_movement_steps':steps,
                            'projected_steps_left_from_500':500-steps,
                            'basis':'Source geometry only; no native expedition counter exists'})
    circuits = []
    for circuit in layout['circuits']:
        visits = []
        for a, b in zip(circuit['path_metatiles'], circuit['path_metatiles'][1:]):
            dx, dy = (b[0] > a[0]) - (b[0] < a[0]), (b[1] > a[1]) - (b[1] < a[1])
            for step in range(abs(b[0] - a[0]) + abs(b[1] - a[1]) + 1):
                x, y = a[0] + dx * step, a[1] + dy * step
                options = {(x * 2 + qx, y * 2 + qy) for qx in [0,1] for qy in [0,1]} & opened
                if not options: errors.append(circuit['id'] + ': disconnected at ' + str((x,y))); break
                visits.append(min(options))
        steps = 0
        for a, b in zip(visits, visits[1:]):
            route = shortest_walk(a, b, opened)
            if route is None: errors.append(circuit['id'] + ': broken ordinary return'); break
            steps += len(route) - 1
        circuits.append({'id': circuit['id'], 'authored_loop_walk_steps': steps, 'remaining_from_500': 500 - steps})
    distances = []
    for habitat in layout['habitat_anchors']:
        at = tuple(habitat['tile'])
        if kind(*at) not in {'TALL_GRASS', 'WATER'}: errors.append(habitat['id'] + ': capture anchor is not grass/water')
        if at not in surfing: errors.append(habitat['id'] + ': ordinary habitat incorrectly requires Cut')
        route = shortest_walk(start, at, all_open)
        if route is None: errors.append(habitat['id'] + ': unreachable habitat')
        distances.append({'id': habitat['id'], 'shortest_walk_surf_steps': len(route)-1 if route else None})
    seams = []
    columns, rows = (width+31)//32, (height+23)//24
    for row in range(rows):
        for column in range(columns):
            name=f'C{column:02}R{row:02}'
            if column+1<columns:
                x=(column+1)*64
                seams.append(dict(a=name,b=f'C{column+1:02}R{row:02}',direction='east',water_pairs=sum((x-1,y) in water and (x,y) in water for y in range(row*48,min((row+1)*48,height*2)))))
            if row+1<rows:
                y=(row+1)*48
                seams.append(dict(a=name,b=f'C{column:02}R{row+1:02}',direction='south',water_pairs=sum((x,y-1) in water and (x,y) in water for x in range(column*64,min((column+1)*64,width*2)))))
    return {'status': 'Terrain proposal / source-only collision checks; no native Safari runtime', 'errors': errors,
            'walk_reachable_tiles': len(walking), 'walk_after_cut_tiles': len(opened),
            'walk_surf_reachable_tiles': len(surfing), 'walk_surf_after_cut_tiles': len(all_open),
            'cut_targets_sealed_without_cut': True, 'quiet_island_requires_surf': True, 'campsites':camp_checks,
            'entry_gate':{'gate_tiles':entry_gate['gate_tiles'], 'arrival_to_launch_must_cross_gate':True,
                          'surf_and_opened_cut_bypass':False,
                          'native_status':'Source geometry only; paid entry/session integration pending'},
            'root_beer_lady':{'door':layout['root_beer_lady']['door'], 'npc':layout['landmark_actors'][0]['tile'], 'service_status':'Proposed root binding; no native service in source preview'},
            'water_tiles': len(water), 'water_percent':round(len(water)/(width*height*4)*100,2),
            'water_components':components, 'largest_connected_water_share_percent':round(max(c['water_tiles'] for c in components)/len(water)*100,2),
            'portages':portages, 'sample_expeditions':itineraries, 'outer_water_tiles': 0, 'native_block_varieties': len(set(blocks)),
            'ordinary_trainers': 0, 'source_water_seams_pending_native_connections': seams, 'circuits': circuits, 'habitat_distances': distances,
            'ledge_returns': returns}, blocks, kind


def render(layout, concept, pack, art):
    started = time.perf_counter()
    report, blocks, kind = inspect(layout, pack)
    width, height = layout['dimensions_metatiles']
    terrain = art.render('johto', width, height, blocks, derived=layout['derived_blocks'])
    scene = render_stage(terrain, 'convert', OUTPUT, 'review_boundary_safari-render-line241-12-convert', 'RGBA')
    for actor in layout['retire_anchors'] + layout.get('landmark_actors', []) + layout.get('decorations', []):
        scene.alpha_composite(oriented_sprite(pack, actor['sprite'], actor['facing']), (actor['tile'][0]*16, actor['tile'][1]*16-4))
    OUTPUT.mkdir(parents=True, exist_ok=True)
    save_render(terrain, OUTPUT/'terrain.png'); save_render(render_stage(scene, 'convert', OUTPUT, 'review_boundary_safari-render-line245-60-convert', 'RGB'), OUTPUT/'whole-preserve.png')
    digest = hashlib.sha256(scene.tobytes()).hexdigest()[:8]
    save_render(render_stage(scene, 'convert', OUTPUT, 'review_boundary_safari-render-line247-16-convert', 'RGB'), OUTPUT/f'whole-preserve-{digest}.png')
    after_blocks = list(blocks)
    for gate in layout['cut_gates']:
        x,y = gate['tile']; after_blocks[(y//2)*width+x//2] = int(gate['replacement_block'],16)
    after_scene = render_stage(art.render('johto',width,height,after_blocks,derived=layout['derived_blocks']), 'convert', OUTPUT, 'review_boundary_safari-render-line251-18-convert', 'RGBA')
    for actor in layout['retire_anchors'] + layout.get('landmark_actors', []) + layout.get('decorations', []):
        after_scene.alpha_composite(oriented_sprite(pack,actor['sprite'],actor['facing']),(actor['tile'][0]*16,actor['tile'][1]*16-4))
    after_digest = hashlib.sha256(after_scene.tobytes()).hexdigest()[:8]
    save_render(render_stage(after_scene, 'convert', OUTPUT, 'review_boundary_safari-render-line255-16-convert', 'RGB'), OUTPUT/f'after-cut-{after_digest}.png')
    for gate in layout['cut_gates']:
        x,y=gate['approach'];box=(x*16-80,y*16-72,x*16+80,y*16+72)
        for name,im in [('before',scene),('after',after_scene)]:
            save_render(render_stage(render_stage(render_stage(im, 'crop', OUTPUT, 'review_boundary_safari-render-line259-24-crop', box), 'convert', OUTPUT, 'review_boundary_safari-render-line259-24-convert', 'RGB'), 'resize', OUTPUT, 'review_boundary_safari-render-line259-24-resize', (640,576),Image.Resampling.NEAREST), OUTPUT/f'cut-{name}-{digest}.png')
    report['cut_render_evidence']={'before':f'whole-preserve-{digest}.png','after':f'after-cut-{after_digest}.png',
                                  'before_crop':f'cut-before-{digest}.png','after_crop':f'cut-after-{digest}.png'}
    save_render(draw_ledge_paths(scene, layout, report['ledge_returns']), OUTPUT/'ledges.png')
    overlay = scene.copy(); draw = ImageDraw.Draw(overlay)
    for chunk in concept['candidate_chunks']:
        x, y = chunk['column']*1024, chunk['row']*768
        w, h = chunk['dimensions_metatiles']; draw.rectangle((x,y,x+w*32-1,y+h*32-1),outline='#ffffff',width=2)
        draw.text((x+5,y+5),chunk['id'],fill='white',stroke_width=1,stroke_fill='black')
    for actor in layout['retire_anchors']+layout.get('landmark_actors', [])+layout['habitat_anchors']+layout['signs']+layout['viewpoints']:
        x,y=actor['tile']; draw.rectangle((x*16,y*16,x*16+15,y*16+15),outline='#ffc748',width=2)
        draw.text((x*16,y*16-12),actor['id'],fill='white',stroke_width=1,stroke_fill='black')
    save_render(render_stage(overlay, 'convert', OUTPUT, 'review_boundary_safari-render-line271-16-convert', 'RGB'), OUTPUT/'anchors.png')
    views=[]
    for view in layout['viewpoints']:
        x,y=view['tile']; im=render_stage(render_stage(scene, 'crop', OUTPUT, 'review_boundary_safari-render-line274-29-crop', (x*16-80,y*16-72,x*16+80,y*16+72)), 'convert', OUTPUT, 'review_boundary_safari-render-line274-29-convert', 'RGB')
        views.append((view['id'],im));save_render(im, OUTPUT/f'viewport-{view["id"].lower()}-original160.png');save_render(render_stage(im, 'resize', OUTPUT, 'review_boundary_safari-render-line275-50-resize', (640,576),Image.Resampling.NEAREST), OUTPUT/f'viewport-{view["id"].lower()}.png')
    view_sheet = room_sheet(views,columns=3)
    save_render(view_sheet, OUTPUT/'viewpoints.png')
    save_render(view_sheet, OUTPUT/f'viewpoints-{digest}.png')
    report.update(image=f'whole-preserve-{digest}.png',render_ms=round((time.perf_counter()-started)*1000,1),revision=str(time.time_ns()))
    (OUTPUT/'revision.json').write_text(json.dumps(report,indent=2)+'\n')
    return report


PAGE='''<!doctype html><meta charset="utf-8"><title>Boundary Waters Safari terrain proposal</title><style>body{background:#14212b;color:#eef;font:16px system-ui;margin:20px}img{image-rendering:pixelated;max-width:none}.scroll{overflow:auto;max-height:77vh}header{position:sticky;top:0;background:#14212bf5;padding:12px}button{padding:10px;margin:4px}</style><header><h1>Boundary Waters Safari — whole terrain proposal</h1><p>Water-first source proposal. Native Surf required; free story basecamp is outside. Safari session/capture-only battles/500-step accounting/paid entry are not implemented.</p><button data-layer="whole-preserve">Whole preserve</button><button data-layer="terrain">Terrain</button><button data-layer="anchors">Anchors/chunks</button><button data-layer="ledges">One-way return</button><p id="status"></p></header><div class="scroll"><img id="map"></div><h2>Player-scale source views</h2><img id="views"><script>let layer='whole-preserve',revision='';const map=document.querySelector('#map');function refresh(){map.src=layer+'.png?r='+revision;document.querySelector('#views').src='viewpoints.png?r='+revision}document.querySelectorAll('[data-layer]').forEach(b=>b.onclick=()=>{layer=b.dataset.layer;refresh()});setInterval(async()=>{let s=await(await fetch('/state',{cache:'no-store'})).json();document.querySelector('#status').textContent=s.ok?s.render_ms+' ms · '+s.status:'INVALID: '+s.error;if(revision!==s.revision){revision=s.revision;refresh()}},500)</script>'''


def main():
    global LAYOUT, CONCEPT, OUTPUT
    parser=argparse.ArgumentParser();parser.add_argument('--pack',type=Path,default=ROOT/'content-packs/text-tui.crystalpack');parser.add_argument('--layout',type=Path,default=LAYOUT);parser.add_argument('--concept',type=Path,default=CONCEPT);parser.add_argument('--output',type=Path,default=OUTPUT);parser.add_argument('--serve',action='store_true');parser.add_argument('--port',type=int,default=8780);args=parser.parse_args()
    LAYOUT, CONCEPT, OUTPUT = args.layout, args.concept, args.output
    pack=load_pack(args.pack);art=SourceArt(pack);state={'value':{'ok':False,'error':'Initial render pending'}}
    if args.serve:
        OUTPUT.mkdir(parents=True,exist_ok=True);(OUTPUT/'index.html').write_text(PAGE)
        class Handler(SimpleHTTPRequestHandler):
            def log_message(self,*args):pass
            def do_GET(self):
                if self.path.split('?')[0]=='/state':
                    body=json.dumps(state['value']).encode();self.send_response(200);self.send_header('Content-Type','application/json');self.send_header('Content-Length',str(len(body)));self.end_headers();self.wfile.write(body)
                else:super().do_GET()
        server=ThreadingHTTPServer(('127.0.0.1',args.port),partial(Handler,directory=str(OUTPUT)));threading.Thread(target=server.serve_forever,daemon=True).start()
        print(f'Boundary Safari terrain proposal: http://127.0.0.1:{args.port}/',flush=True)
    last=None
    while True:
        revision=(LAYOUT.stat().st_mtime_ns,CONCEPT.stat().st_mtime_ns)
        if revision!=last:
            try:
                report=render(json.loads(LAYOUT.read_text()),json.loads(CONCEPT.read_text()),pack,art)
                state['value']={'ok':not report['errors'],'error':'; '.join(report['errors']),**report};print(json.dumps(report),flush=True)
                if report['errors'] and not args.serve:raise SystemExit(1)
            except (AssertionError,KeyError,ValueError,IndexError,TypeError) as error:
                state['value']={'ok':False,'error':str(error)};print('DESIGN ERROR: '+str(error),flush=True)
                if not args.serve:raise SystemExit(1)
            last=revision
        if not args.serve:break
        time.sleep(.2)


if __name__=='__main__':main()
