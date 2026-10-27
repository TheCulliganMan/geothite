#!/usr/bin/env python3
"""Source-pixel review for individually authored Northern Lake Circuit terrain/rooms."""
import argparse
from functools import partial
from http.server import SimpleHTTPRequestHandler,ThreadingHTTPServer
import json
from pathlib import Path
import threading
import time

from render_reference import SourceArt,load_pack
from review_league_approach import HTML,render as render_source,verify_published

ROOT=Path(__file__).resolve().parents[3]
LAYOUT=ROOT/'planning/minnesota/routes/MN_R08.layout.json'
DOSSIER=ROOT/'planning/minnesota/routes/MN_R08.json'
OUTPUT=ROOT/'planning/minnesota/renders/northern-lakes'
PAGE=HTML.replace('League Approach','Northern Lake Circuit').replace('all eighteen chunks','four interlake maps').replace('Border Lake · Wind Tor · Granite switchback · Dry saddle · Cedar Shelter · Long Lake · Old quarry · Ely arrival.','Cedar inlet bridge · Long shore circuit · Fishing pier · Cut-gated pond island · Birch return drop · Long Lake.').replace('Cedar Shelter','No enterable facades').replace('width:1536px','width:1280px').replace('(native?3072:1536)','(native?1280:640)').replace('native=false','native=true').replace('>Residents<','>Before Cut<').replace('<button data-layer="terrain">','<button data-layer="after-cut/whole-route">After Cut (source preview)</button><button data-layer="terrain">')


def check_layout_contract(layout):
    """Mirror the native schema's orthogonal strokes before source rendering."""
    assert layout['schema_version'] == 2 and layout['reject_cliff_cuts'] is True, 'Native route editor schema/guard required'
    count = 0
    for layer in layout['layers']:
        assert layer.get('name'), 'Every native stroke needs an author label'
        if layer['kind'] not in {'polygon', 'cliff_polygon', 'path'}:
            continue
        points = layer['points']
        edges = zip(points, points[1:] + points[:1]) if layer['kind'] != 'path' else zip(points, points[1:])
        assert all(a[0] == b[0] or a[1] == b[1] for a, b in edges), layer['name'] + ': native stroke/closing edge must be orthogonal'
        if layer['kind'] != 'path':
            count += 1
    return {'native_orthogonal_polygons_checked': count}


def check_piers(design,layout,pack):
    from render_reference import layout_blocks
    from review_headwaters import prepare
    prepare(pack,layout)
    blocks=layout_blocks(layout);w,h=layout['dimensions_metatiles'];c=pack['data']['tilesets']['mn_headwaters']['collision']
    def kind(x,y):return c[f'{blocks[y//2*w+x//2]:02x}'][y%2*2+x%2]
    from review_gunflint import flood
    from ledge_paths import shortest_walk
    for crossing in layout['crossings']:
        for x,y in crossing['north_rail_blocks']:
            assert [kind(x*2,y*2),kind(x*2,y*2+1)]==['WALL','FLOOR'],crossing['id']+': north rail'
        for x,y in crossing['south_rail_blocks']:
            assert [kind(x*2,y*2),kind(x*2,y*2+1)]==['FLOOR','WALL'],crossing['id']+': south rail'
        for x,y in crossing['water_under_rail_blocks']:
            assert kind(x*2,y*2)=='WATER',crossing['id']+': water below boards'
    for ledge in design['one_way_ledges']:
        x0,y0,x1,y1=ledge['bounds_core']
        assert all(kind(x,y0)=='HOP_DOWN' and kind(x,y0+1)=='WALL' for x in range(x0,x1)),'Native drop face'
        assert all(kind(*at)=='WALL' for at in ledge['side_barriers']),'Drop sides must close'
    assert kind(58,73)=='WALL','Fishing pier needs its closed west end'
    return {'source_metatile_varieties_used':len(set(blocks)),'native_drop_faces_checked':len(design['one_way_ledges']),'railed_crossings_checked':len(layout['crossings'])}


def check_cut_and_choices(design, layout, pack):
    """Prove the actual gate before/after the source-authored 63 -> 3f rule.

    No controller or pack is changed. The separate geometry render uses the
    post-Cut collision row only to check access to the explicitly gated actors.
    """
    from collections import deque
    from render_reference import layout_blocks
    from review_gunflint import flood, neighbors
    blocks = layout_blocks(layout)
    width, height = layout['dimensions_metatiles']
    collisions = pack['data']['tilesets'][layout['compiled_tileset']]['collision']
    assert len(design['cut_gates']) == 1, 'This route has one explicit island gate'
    gate = design['cut_gates'][0]
    cut = pack['data']['field_moves']['cut']
    assert cut['move_id'] == 'CUT' and cut['badge'] == {'region': 'johto', 'index': 1}, 'Source Cut move/Hive Badge rule changed'
    source = int(gate['source_block'], 16)
    replacement = cut['replacements']['johto'][source]['replacement_block_id']
    assert replacement == int(gate['replacement_block'], 16) == 0x3f, 'Use the actual Johto 63 -> 3f tree replacement'
    tx, ty = gate['tile']
    inventory = {(x, y) for y in range(height * 2) for x in range(width * 2)
                 if collisions[f'{blocks[y // 2 * width + x // 2]:02x}'][y % 2 * 2 + x % 2] == 'CUT_TREE'}
    assert inventory == {tuple(g['tile']) for g in design['cut_gates']}, 'Undeclared decorative Cut quadrant or missing real gate'
    assert blocks[ty // 2 * width + tx // 2] == source
    def kind(x, y, opened=False):
        block = blocks[y // 2 * width + x // 2]
        if opened and (x // 2, y // 2) == (tx // 2, ty // 2):
            block = replacement
        return collisions[f'{block:02x}'][y % 2 * 2 + x % 2]
    actors = design['trainers'] + design['residents'] + design['items']
    occupied = {tuple(a['tile']) for a in actors}
    def floors(opened):
        return {(x, y) for y in range(height * 2) for x in range(width * 2)
                if kind(x, y, opened) in {'FLOOR', 'TALL_GRASS'} and (x, y) not in occupied}
    start, end = [tuple(v * 2 for v in e['gate_metatile']) for e in design['endpoints']]
    before, after = [flood(start, floors(opened)) for opened in [False, True]]
    assert end in before and end in after, 'Cut must never block required route travel'
    assert kind(tx, ty) == 'CUT_TREE' and tuple(gate['approach']) in before
    assert all(kind(*p) in {'WALL', 'WATER'} for p in gate['side_barriers']), 'Cut gate has an open side'
    for actor in actors:
        access = lambda seen: any(p in seen for p in neighbors(tuple(actor['tile'])))
        assert access(after), actor['id'] + ': unreachable even after Cut'
        assert access(before) == (actor['id'] not in gate['gated_actor_ids']), actor['id'] + ': incorrect Cut gating'
    for view in design['viewpoints']:
        assert (tuple(view['tile']) in before) == (view['id'] not in gate['gated_viewpoints']), view['id'] + ': incorrect pre-Cut camera access'
    for sign in design['signs']:
        assert any(p in before for p in neighbors(tuple(sign['tile']))), sign['id'] + ': unexpectedly gated sign'
    assert all(tuple(p) not in before and tuple(p) in after for p in gate['targets']), 'Island reward/trainer bypass'
    surf_floor = {(x, y) for y in range(height * 2) for x in range(width * 2)
                  if kind(x, y) in {'FLOOR', 'TALL_GRASS', 'WATER'} and (x, y) not in occupied}
    surf_before = flood(start, surf_floor)
    surf_open = {(x, y) for y in range(height * 2) for x in range(width * 2)
                 if kind(x, y, True) in {'FLOOR', 'TALL_GRASS', 'WATER'} and (x, y) not in occupied}
    surf_after = flood(start, surf_open)
    assert end in surf_before and end in surf_after, 'Cut/Surf gate blocks required regional travel'
    assert tuple(gate['approach']) in surf_before, 'Surf-safe Cut approach is inaccessible'
    assert all(tuple(p) not in surf_before and tuple(p) in surf_after for p in gate['targets']), 'Surf lands behind the Cut gate'
    for actor in actors:
        reachable = any(p in surf_before for p in neighbors(tuple(actor['tile'])))
        assert reachable == (actor['id'] not in gate['gated_actor_ids']), actor['id'] + ': Surf bypasses Cut or strands ordinary actor'
    for record in gate['shore_rail_blocks']:
        x, y = record['position']
        assert blocks[y * width + x] == int(record['block'], 16), 'Missing source island rail'
    for x, y in gate['shore_rock_blocks']:
        assert all(kind(x * 2 + dx, y * 2 + dy) == 'WALL' for dx in [0, 1] for dy in [0, 1]), 'Island water rock is not fully blocking'
    # The island bridge itself must be necessary even with the Cut gate open.
    bridge = next(p for p in layout['crossings'] if p['id'] == 'BIRCH_ISLAND_BRIDGE')
    bridge_cells = {(x * 2 + dx, y * 2 + dy) for key in ['north_rail_blocks', 'south_rail_blocks']
                    for x, y in bridge[key] for dx in [0, 1] for dy in [0, 1]}
    island_without_bridge = flood(start, floors(True) - bridge_cells)
    assert all(tuple(p) not in island_without_bridge for p in gate['targets']), 'Island has a dry bridge bypass'
    choices = []
    for choice in design['route_choices']:
        x0, y0, x1, y1 = choice['short_lane_region_core']
        region = {(x, y) for y in range(y0, y1) for x in range(x0, x1)} & floors(False)
        x0, y0, x1, y1 = choice['grass_bounds_core']
        band = {(x, y) for y in range(y0, y1) for x in range(x0, x1)}
        assert all(kind(*p) == 'TALL_GRASS' for p in band), choice['id'] + ': wrong habitat collision'
        origin, destination = tuple(choice['short_lane_from']), tuple(choice['short_lane_to'])
        assert destination in flood(origin, region), choice['id'] + ': disconnected lane'
        assert destination not in flood(origin, region - band), choice['id'] + ': free dry bypass'
        costs, todo = {origin: 0}, deque([origin])
        while todo:
            at = todo.popleft()
            for point in neighbors(at):
                if point not in region:
                    continue
                value = costs[at] + int(point in band)
                if value < costs.get(point, 10**9):
                    costs[point] = value
                    (todo.append if point in band else todo.appendleft)(point)
        assert costs[destination] >= choice['minimum_grass_steps'], choice['id'] + ': shallow habitat'
        choices.append({'id': choice['id'], 'minimum_grass_steps': costs[destination]})
    return {'cut_gate_checked': True, 'cut_source_block': '63', 'cut_replacement_block': '3f',
            'cut_move': 'CUT', 'cut_badge': cut['badge'], 'all_cut_tree_quadrants': [list(p) for p in sorted(inventory)], 'pre_cut_reachable_tiles': len(before),
            'post_cut_reachable_tiles': len(after), 'cut_unlocked_tiles': len(after - before),
            'cut_gated_actors': gate['gated_actor_ids'], 'route_habitat_choices': choices,
            'cut_gate_blocks_surf_landing': True, 'walk_surf_pre_cut_reachable_tiles': len(surf_before),
            'walk_surf_post_cut_reachable_tiles': len(surf_after), 'walk_surf_cut_unlocked_tiles': len(surf_after - surf_before)}


def check_water_boundaries(design, layout, pack):
    """Reject outer WATER and inspect every clipped-map walking/Surf join.

    The union of walkable land and all WATER is deliberately more permissive
    than native Surf. Enclosure under that graph cannot conceal a Surf exit.
    Source-only neighbor pairs are labelled; native connections are inspected
    only when the supplied pack actually contains all four authored maps.
    """
    from render_reference import layout_blocks
    from review_gunflint import flood
    blocks = layout_blocks(layout)
    width, height = layout['dimensions_metatiles']
    core_width, core_height = width * 2, height * 2
    collisions = pack['data']['tilesets'][layout['compiled_tileset']]['collision']
    def kind(x, y):
        return collisions[f'{blocks[y // 2 * width + x // 2]:02x}'][y % 2 * 2 + x % 2]
    water = {(x, y) for y in range(core_height) for x in range(core_width)
             if kind(x, y) == 'WATER'}
    perimeter = {(x, y) for y in range(core_height) for x in range(core_width)
                 if x in {0, core_width - 1} or y in {0, core_height - 1}}
    assert not water & perimeter, 'Surfable WATER reaches an unconnected outer route edge; add visible native rock/buoy collision'
    remaining, components = set(water), []
    while remaining:
        component = flood(min(remaining), water)
        remaining -= component
        xs, ys = [p[0] for p in component], [p[1] for p in component]
        components.append({'water_tiles': len(component),
                           'bounds_core': [min(xs), min(ys), max(xs) + 1, max(ys) + 1]})
    gate = design['cut_gates'][0]
    actors = {tuple(a['tile']) for key in ['trainers', 'residents', 'items'] for a in design[key]}
    combined = {(x, y) for y in range(core_height) for x in range(core_width)
                if kind(x, y) in {'FLOOR', 'TALL_GRASS', 'WATER'} and (x, y) not in actors}
    start = tuple(v * 2 for v in design['endpoints'][0]['gate_metatile'])
    reach = flood(start, combined)
    assert not reach & water & perimeter, 'Walking plus Surf can escape through outer WATER'
    # Check both tree states; the source replacement opens one land quadrant.
    tx, ty = gate['tile']
    replacement = collisions[gate['replacement_block']]
    for dy in [0, 1]:
        for dx in [0, 1]:
            point = (tx // 2 * 2 + dx, ty // 2 * 2 + dy)
            if replacement[dy * 2 + dx] in {'FLOOR', 'TALL_GRASS', 'WATER'} and point not in actors:
                combined.add(point)
    opened_reach = flood(start, combined)
    assert not opened_reach & water & perimeter, 'Cut opens an unintended outer Surf exit'
    prefix = design['map_prefix']
    chunks = {}
    for row in range(design['chunk_rows']):
        for col in range(design['chunk_columns']):
            name = f'{prefix}C{col:02}R{row:02}'
            chunks[(col, row)] = {'map': name, 'dimensions_metatiles': [min(32, width - col * 32), min(24, height - row * 24)]}
    pairs = []
    maps = pack['data']['maps']
    native_available = all(chunk['map'] in maps for chunk in chunks.values())
    for (col, row), chunk in chunks.items():
        for direction, reverse, neighbor in [('east', 'west', (col + 1, row)), ('south', 'north', (col, row + 1))]:
            if neighbor not in chunks:
                continue
            other = chunks[neighbor]
            if direction == 'east':
                edge = (col + 1) * 64
                open_water = sum((edge - 1, y) in water and (edge, y) in water
                                 for y in range(row * 48, min(core_height, (row + 1) * 48)))
            else:
                edge = (row + 1) * 48
                open_water = sum((x, edge - 1) in water and (x, edge) in water
                                 for x in range(col * 64, min(core_width, (col + 1) * 64)))
            if native_available:
                for source, target, facing in [(chunk, other, direction), (other, chunk, reverse)]:
                    attributes = maps[source['map']]['attributes']
                    assert [attributes['width'], attributes['height']] == source['dimensions_metatiles'], source['map'] + ': incorrect clipped dimensions'
                    assert any(c['direction'] == facing and c['target_map'] == target['map'] and c['offset'] == 0
                               for c in attributes['connections']), source['map'] + ': missing reciprocal native water/walking neighbor'
            pairs.append({'a': chunk['map'], 'direction': direction, 'b': other['map'], 'open_water_tile_pairs': open_water})
    return {'outer_water_tiles': 0, 'water_tiles_checked': len(water),
            'water_components': components, 'reachable_water_before_cut': len(reach & water),
            'reachable_water_after_cut': len(opened_reach & water),
            'water_clearance_core': {'west': min(x for x, _ in water),
                                     'east': core_width - 1 - max(x for x, _ in water),
                                     'north': min(y for _, y in water),
                                     'south': core_height - 1 - max(y for _, y in water)},
            'clipped_chunk_dimensions': list(chunks.values()), 'internal_neighbor_pairs': pairs,
            'native_reciprocal_neighbor_pairs_checked': len(pairs) if native_available else 0,
            'native_boundary_status': 'Native map dimensions and reciprocal connections inspected' if native_available else 'Source enclosure checked; native publication/reciprocal connection verification pending'}


def render_revision(design, layout, pack, art):
    """Render the real pre-Cut pixels, validating gated anchors after Cut.

    The temporary collision view is labelled in the report and never changes
    pack files, source pixels, native runtime state or the original dictionary.
    """
    started = time.perf_counter()
    extra = check_layout_contract(layout)
    extra.update(check_piers(design, layout, pack))
    extra.update(check_cut_and_choices(design, layout, pack))
    extra.update(check_water_boundaries(design, layout, pack))
    preview_pack = dict(pack)
    preview_pack['data'] = dict(pack['data'])
    preview_pack['data']['tilesets'] = dict(pack['data']['tilesets'])
    tileset = layout['compiled_tileset']
    definition = dict(pack['data']['tilesets'][tileset])
    definition['collision'] = dict(definition['collision'])
    definition['collision']['63'] = list(definition['collision']['3f'])
    preview_pack['data']['tilesets'][tileset] = definition
    after_design = dict(design)
    after_design.pop('shortcut', None)
    after_design.pop('cut_gates', None)
    report = render_source(after_design, layout, preview_pack, art, output=OUTPUT)
    gate = design['cut_gates'][0]
    opened_layout = dict(layout)
    opened_layout['layers'] = list(layout['layers']) + [
        {'kind': 'point', 'position': [gate['tile'][0] // 2, gate['tile'][1] // 2],
         'block': gate['replacement_block'], 'name': 'Native Cut override source preview only'}]
    after_report = render_source(after_design, opened_layout, pack, art, output=OUTPUT / 'after-cut')
    report['errors'].extend('After Cut: ' + error for error in after_report['errors'])
    report['after_cut_source_image'] = 'after-cut/' + after_report['image']
    report.update(extra, review_ms=round((time.perf_counter() - started) * 1000, 1), anchor_check_state='After native Cut replacement; actual image pixels show the uncut tree',
                  status='Source pixels and explicit pre/post-Cut static collision checked; native gameplay remains separate')
    (OUTPUT / 'revision.json').write_text(json.dumps(report, indent=2) + '\n')
    return report


def main():
    parser=argparse.ArgumentParser();parser.add_argument('--pack',type=Path,default=ROOT/'content-packs/text-tui.crystalpack');parser.add_argument('--serve',action='store_true');parser.add_argument('--watch',action='store_true');parser.add_argument('--port',type=int,default=8778);parser.add_argument('--verify-published',action='store_true');args=parser.parse_args()
    if args.verify_published and (args.watch or args.serve):parser.error('--verify-published requires a single run')
    pack=load_pack(args.pack);art=SourceArt(pack);state={'value':{'ok':False,'error':'Initial render pending'}}
    if args.serve:
        OUTPUT.mkdir(parents=True,exist_ok=True);(OUTPUT/'index.html').write_text(PAGE)
        class Handler(SimpleHTTPRequestHandler):
            def log_message(self,*args):pass
            def do_GET(self):
                if self.path.split('?')[0]=='/state':
                    body=json.dumps(state['value']).encode();self.send_response(200);self.send_header('Content-Type','application/json');self.send_header('Cache-Control','no-store');self.send_header('Content-Length',str(len(body)));self.end_headers();self.wfile.write(body)
                else:super().do_GET()
        server=ThreadingHTTPServer(('127.0.0.1',args.port),partial(Handler,directory=str(OUTPUT)));threading.Thread(target=server.serve_forever,daemon=True).start();print(f'Northern Lake Circuit review: http://127.0.0.1:{args.port}/',flush=True)
    last=None
    while True:
        revision=(LAYOUT.stat().st_mtime_ns,DOSSIER.stat().st_mtime_ns)
        if revision!=last:
            try:
                design,layout=json.loads(DOSSIER.read_text()),json.loads(LAYOUT.read_text());published=verify_published(design,layout,pack) if args.verify_published else {};report={**render_revision(design,layout,pack,art),**published};state['value']={'ok':not report['errors'],'revision':str(time.time_ns()),'error':'; '.join(report['errors']),**report};print(json.dumps(report),flush=True)
                if report['errors'] and not(args.watch or args.serve):raise SystemExit(1)
            except (AssertionError,KeyError,ValueError,IndexError,TypeError) as error:
                state['value']={'ok':False,'error':str(error)};print('DESIGN ERROR: '+str(error),flush=True)
                if not(args.watch or args.serve):raise SystemExit(1)
            last=revision
        if not(args.watch or args.serve):break
        time.sleep(.2)


if __name__=='__main__':main()
