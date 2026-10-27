#!/usr/bin/env python3
"""Composite the current authored maps; never generate or rearrange game terrain.

Review placements below are authored separately from gameplay coordinates.
Dashed leaders represent world.json hub links, not implemented walking seams.
"""
from render_archive import save_render
import argparse
import gc
import hashlib
import heapq
import json
from pathlib import Path

from PIL import Image, ImageDraw, ImageFont

from render_reference import SourceArt, layout_blocks, load_pack, npc_palette_pack
from review_boundary_native import FACING, verify as verify_boundary
from review_north_shore import oriented_sprite
from progress_atlas_story import StoryAtlas, save_review_crops, snapshot_source_epoch

ROOT = Path(__file__).resolve().parents[3]
PLAN = ROOT / 'planning/minnesota'
SCALE = 12
SIZE = (530, 886)
BACKGROUND = '#121f2b'
COLORS = {'native': '#68d7ba', 'draft': '#edbf67', 'blockout': '#8294a7'}
PUBLISHED = {
    '01': 'driftless', '02': 'prairie-farm', '03': 'river-valley', '04': 'twin-cities',
    '05': 'river-bluffs', '06': 'pine-country', '07': 'leech-lake', '08': 'northern-lakes',
    '09': 'iron-range', '10': 'iron-range-descent', '16': 'red-river-prairie',
    '11': 'north-shore', '12': 'gunflint', '13': 'league-approach',
    '17': 'border-pines', '18': 'mille-lacs-shore', '19': 'prairie-forest-link',
    '20': 'voyageurs-border-loop',
    '14': 'headwaters', '15': 'central-lakes',
}
SOURCE_REVISIONS = {'11', '12', '14', '17', '18', '19', '20'}
# Topological review arrangement. Every image uses the same exterior scale.
PLACEMENTS = {
    '01': (205, 846), '02': (117, 845), '03': (72, 797), '04': (203, 775),
    '05': (161, 676), '06': (160, 535), '07': (160, 384), '08': (156, 243),
    '09': (257, 238), '10': (267, 338), '11': (432, 354), '12': (372, 218),
    '13': (416, 33), '14': (12, 219), '15': (78, 484), '16': (12, 526),
    '17': (179, 33), '18': (257, 484), '19': (12, 346), '20': (266, 33),
}
HUBS = {
    'WINONA': (265, 856), 'ROCHESTER': (174, 856), 'MANKATO': (84, 845),
    'MINNEAPOLIS': (174, 781), 'ST PAUL': (265, 781), 'ST CLOUD': (176, 626),
    'BRAINERD': (176, 478), 'LEECH LAKE': (176, 328), 'BEMIDJI': (176, 196),
    'GRAND RAPIDS': (305, 325), 'DULUTH': (390, 444),
    'BOUNDARY WATERS': (455, 18), 'ELY': (463, 196),
    'LAKE ITASCA': (52, 320), 'DETROIT LAKES': (54, 488),
    'MOORHEAD': (55, 617), 'INTERNATIONAL FALLS': (210, 18),
    'MILLE LACS': (390, 540),
}
TOWN_AT = (470, 326)
CAMP_AT = (470, 24)
GALLERY = (302, 548, 518, 882)
SAFARI_AT = (350, 155)
CABIN_AT = (435, 161)
CITY_MAPS = {'BRAINERD': 'MnBrainerd', 'BEMIDJI': 'MnBemidji', 'DULUTH': 'MnDuluth',
             'GRAND RAPIDS': 'MnGrandRapids'}


def read(path):
    return json.loads(path.read_text())


def font(size, bold=False):
    filename = 'Arial Bold.ttf' if bold else 'Arial.ttf'
    path = Path('/System/Library/Fonts/Supplemental') / filename
    return ImageFont.truetype(str(path), size) if path.exists() else ImageFont.load_default(size=size)


def scene(art, pack, tileset, width, height, blocks, actors, derived=None, archive=None, group='day', native_origin=False):
    terrain = art.render(tileset, width, height, blocks, group=group, derived=derived)
    if archive:
        save_render(terrain, archive / 'terrain-original.png')
    image = terrain.convert('RGBA')
    # Production INDOOR/GATE backgrounds use their indoor bank; object palettes
    # still select the native day bank (there is no fifth NPC palette bank).
    sprite_pack = npc_palette_pack(pack, 'day' if group == 'indoor' else group)
    for actor in actors:
        if 'tile' not in actor:
            continue
        image.alpha_composite(oriented_sprite(sprite_pack, actor.get('sprite', 'SPRITE_POKE_BALL'),
                                              actor.get('facing', 'south')),
                              (actor['tile'][0] * 16, actor['tile'][1] * 16 - (0 if native_origin else 4)))
    if archive:
        save_render(image, archive / 'scene-original-rgba.png')
    converted = image.convert('RGB')
    if archive:
        save_render(converted, archive / 'scene-rgb.png')
    return converted


def native_room(module, pack, archive=None, native_origin=False):
    a = module['attributes']
    group = a['time_of_day'] or 'day'
    if native_origin:
        if group in {'dark', 'darkness'}:
            group = 'dark'
        elif (a['environment'] or '').lower() in {'indoor', 'gate'}:
            group = 'indoor'
    actors = [dict(tile=[o['x'], o['y']], sprite=o['sprite'],
                   facing=FACING.get(o['spritemovedata'].removeprefix('SPRITEMOVEDATA_STANDING_'), 'south'))
              for o in module['objects']]
    return scene(SourceArt(pack), pack, a['tileset_name'], a['width'], a['height'],
                 module['blocks'], actors, archive=archive, group=group, native_origin=native_origin)


def room_tag(index):
    """Letter tags stay readable when the room inventory exceeds Z."""
    tag = ''
    while index >= 0:
        tag = chr(ord('A') + index % 26) + tag
        index = index // 26 - 1
    return tag


def verify_native_route(design, blocks, pack):
    """Check current terrain/events without depending on older dossier schemas."""
    width, height = design['dimensions_metatiles']
    maps, prefix = pack['data']['maps'], design['map_prefix']
    actual = []
    for y in range(height):
        for x in range(width):
            module = maps[f'{prefix}C{x // 32:02}R{y // 24:02}']
            actual.append(module['blocks'][(y % 24) * module['attributes']['width'] + x % 32])
    assert blocks == actual, design['id'] + ': native terrain differs from current authoring'
    actors = []
    for row in range(design['chunk_rows']):
        for col in range(design['chunk_columns']):
            module = maps[f'{prefix}C{col:02}R{row:02}']
            actors.extend((o['x'] + col * 64, o['y'] + row * 48, o['sprite']) for o in module['objects'])
    expected = [(a['tile'][0], a['tile'][1], a.get('sprite', 'SPRITE_POKE_BALL'))
                for key in ['trainers', 'residents', 'items'] for a in design[key]]
    assert sorted(actors) == sorted(expected), design['id'] + ': native actor placement differs'
    for room in design['interiors']:
        assert maps[room['id']]['blocks'] == [int(b, 16) for line in room['rows'] for b in line.split()], room['id']


def guarded_city_connections(pack, design, module, blocks, source):
    """Certify declared installed mouth patches/guards before drawing solid links."""
    maps = pack['data']['maps']
    expected = list(blocks)
    guards, links = [], {}
    passages = source['passages']
    assert len(module['attributes']['connections']) == len(passages) == len(design['ports'])
    for patch in source['patches']:
        target = maps[patch['map']]
        x, y = patch['metatile']
        index = y * target['attributes']['width'] + x
        assert target['blocks'][index] == int(patch['block'], 16), 'Installed mouth patch differs: ' + patch['map']
        if patch['map'] == design['id']:
            assert expected[index] == int(patch['expected'], 16), 'Stale city mouth preimage: ' + design['id']
            expected[index] = int(patch['block'], 16)
    def kind(target, tile):
        x, y = tile
        a = target['attributes']
        block = target['blocks'][(y // 2) * a['width'] + x // 2]
        return pack['data']['tilesets'][a['tileset_name']]['collision'][f'{block:02x}'][(y % 2) * 2 + x % 2]
    for passage in passages:
        assert all(len(passage[field]) == 2 for field in
                   ['identifiers', 'bodies', 'inside', 'outside', 'town_edge', 'route_edge'])
        neighbor = maps[passage['route']]
        assert dict(direction=passage['outward'], target_map=passage['route'],
                    offset=passage['offset']) in module['attributes']['connections']
        assert dict(direction=passage['reverse'], target_map=design['id'],
                    offset=-passage['offset']) in neighbor['attributes']['connections']
        port = next(p for p in design['ports'] if passage['route'].startswith('MnR' + p['route'][-2:]))
        assert port['side'] == {'north': 'N', 'south': 'S', 'west': 'W', 'east': 'E'}[passage['outward']]
        for actor_id, tile in zip(passage['identifiers'], passage['bodies']):
            actor = next(a for a in module['objects'] if a['object_identifier'] == actor_id)
            assert [actor['x'], actor['y']] == tile and actor['sprite'] == 'SPRITE_OFFICER'
            assert actor['script'] == passage['root'] and actor['spritemovedata'] == 'SPRITEMOVEDATA_STANDING_' + passage['incoming_facing']
            guards.append(dict(id=actor_id, tile=tile, sprite='SPRITE_OFFICER'))
        for tiles, script in [(passage['inside'], passage['root']),
                              (passage['outside'], passage['root'] + 'Inward')]:
            for tile in tiles:
                assert kind(module, tile) == 'FLOOR'
                assert any([e['x'], e['y']] == tile and e['script_name'] == script
                           for e in module['events']['coord_events'])
        for town_tile, route_tile in zip(passage['town_edge'], passage['route_edge']):
            assert kind(module, town_tile) == kind(neighbor, route_tile) == 'FLOOR'
            axis = 0 if port['side'] in {'N', 'S'} else 1
            assert town_tile[axis] // 2 == port['gate_metatile'][axis]
            assert route_tile[axis] == town_tile[axis] - 2 * passage['offset'], 'Native seam lane offset differs'
        col, row = passage['route'].rsplit('C', 1)[1].split('R')
        route_gate = [int(col) * 32 + (sum(p[0] for p in passage['route_edge']) / 2 + .5) / 2,
                      int(row) * 24 + (sum(p[1] for p in passage['route_edge']) / 2 + .5) / 2]
        links[(design['name'], port['route'][-2:])] = dict(
            port, native_route_gate_metatile=route_gate,
            native_route_side={'north': 'N', 'south': 'S', 'west': 'W', 'east': 'E'}[passage['reverse']])
    assert module['blocks'] == expected, design['name'] + ': undeclared native exterior drift'
    return expected, guards, links


def installed_route_mouths(design, blocks, pack):
    """Overlay only source-declared mouths belonging to installed city links."""
    if design['id'] != 'MN_R10':
        return blocks
    result = list(blocks)
    width = design['dimensions_metatiles'][0]
    for town, path in [('MnGrandRapids', 'towns/MnGrandRapids.json'),
                       ('MnDuluth', 'towns/MnDuluth.connections.json')]:
        module = pack['data']['maps'].get(town)
        if module is None or not any(c['target_map'].startswith(design['map_prefix'])
                                     for c in module['attributes']['connections']):
            continue
        source = read(PLAN / path)['regional_connections']
        for patch in source['patches']:
            if not patch['map'].startswith(design['map_prefix']):
                continue
            neighbor = pack['data']['maps'][patch['map']]
            x, y = patch['metatile']
            local_index = y * neighbor['attributes']['width'] + x
            assert neighbor['blocks'][local_index] == int(patch['block'], 16), 'Native route mouth differs: ' + patch['map']
            col, row = patch['map'].rsplit('C', 1)[1].split('R')
            index = (int(row) * 24 + y) * width + int(col) * 32 + x
            assert result[index] == int(patch['expected'], 16), 'Stale route mouth preimage: ' + patch['map']
            result[index] = int(patch['block'], 16)
    return result


def cells(rect):
    x0, y0, x1, y1 = rect
    return {(x, y) for x in range(x0, x1 + 1) for y in range(y0, y1 + 1)}


def route_leader(start, end, blocked, used):
    """A* only routes diagram leaders through margins; it never touches maps."""
    start, end = tuple(round(v) for v in start), tuple(round(v) for v in end)
    blocked = blocked - {start, end}
    parents, cost = {}, {start: 0}
    queue = [(0, start)]
    while queue:
        _, at = heapq.heappop(queue)
        if at == end:
            path = [end]
            while path[-1] != start:
                path.append(parents[path[-1]])
            return path[::-1]
        x, y = at
        for point in [(x - 1, y), (x + 1, y), (x, y - 1), (x, y + 1)]:
            nx, ny = point
            if not (2 <= nx < SIZE[0] - 2 and 10 <= ny < SIZE[1] - 4) or point in blocked:
                continue
            distance = cost[at] + 1 + min(used.get(point, 0), 5) * 2
            if distance >= cost.get(point, float('inf')):
                continue
            cost[point], parents[point] = distance, at
            heapq.heappush(queue, (distance + abs(nx - end[0]) + abs(ny - end[1]), point))
    raise ValueError(f'No diagram connection {start} -> {end}')


def edge(bounds, toward):
    x0, y0, x1, y1 = bounds
    x, y = toward
    if abs(x - (x0 + x1) / 2) > abs(y - (y0 + y1) / 2):
        return (x0 - 2 if x < x0 else x1 + 2, min(max(y, y0), y1))
    return (min(max(x, x0), x1), y0 - 2 if y < y0 else y1 + 2)


def render(base_path, output, region_path=None, story_baseline=None):
    components = output.parent / 'components'
    world = read(PLAN / 'world.json')
    pack = load_pack(base_path)
    art = SourceArt(pack)
    region_pack = load_pack(region_path) if region_path else None
    assert story_baseline is None or region_pack is not None, 'Story adapter requires an actual composed region pack'
    story = StoryAtlas(region_pack, story_baseline, world, PLAN) if story_baseline else None
    if story:
        art = SourceArt(region_pack)
    city_maps = story.cities if story else CITY_MAPS
    source_epoch = snapshot_source_epoch(PLAN, output, world, city_maps) if story else {}
    # The existing gray Mankato hub is too close to R03 for a real 24x18 town.
    # This changes only atlas placement, never source or gameplay coordinates.
    hub_centers = dict(HUBS)
    if story:
        hub_centers['MANKATO'] = (91, 856)
        hub_centers['GRAND RAPIDS'] = (305, 313)
    def installed_room(module, pack, archive=None):
        return native_room(module, pack, archive, native_origin=bool(story))
    designs = {entry['id'][-2:]: read(PLAN / entry['file']) for entry in world['routes']}
    assert set(designs) == set(PLACEMENTS) == {f'{n:02}' for n in range(1, 21)}
    routes, rooms = {}, []
    published_routes = {}
    source_revisions = set(SOURCE_REVISIONS)
    for rid, design in designs.items():
        layout = read(PLAN / 'routes' / f'MN_R{rid}.layout.json')
        width, height = layout['dimensions_metatiles']
        actors = sum((design.get(k, []) for k in ['trainers', 'residents', 'items']), [])
        blocks = layout_blocks(layout)
        if region_pack:
            blocks = installed_route_mouths(design, blocks, region_pack)
        if story:
            blocks = story.route_overlays(design, blocks)
            actors = []
            native_art_variants = set()
            for row in range(design['chunk_rows']):
                for col in range(design['chunk_columns']):
                    module = region_pack['data']['maps'][f'{design["map_prefix"]}C{col:02}R{row:02}']
                    native_art_variants.add((module['attributes']['tileset_name'], module['attributes']['time_of_day'] or 'day'))
                    actors.extend(dict(tile=[o['x'] + col * 64, o['y'] + row * 48], sprite=o['sprite'], facing=FACING.get(o['spritemovedata'].removeprefix('SPRITEMOVEDATA_STANDING_'), 'south')) for o in module['objects'])
            assert len(native_art_variants) == 1, design['id'] + ': route chunks disagree on native art/time'
            native_tileset, native_time = next(iter(native_art_variants))
            assert native_tileset == layout['compiled_tileset'], design['id'] + ': compiled source/native tileset differs'
            routes[rid] = scene(art, region_pack, native_tileset, width, height,
                                blocks, actors, archive=components / f'R{rid}', group=native_time, native_origin=True)
        else:
            routes[rid] = scene(art, pack, layout['tileset'], width, height,
                                blocks, actors, layout.get('derived_blocks'), components / f'R{rid}')
        if rid in PUBLISHED:
            published = region_pack if region_pack else load_pack(ROOT / f'content-packs/minnesota-{PUBLISHED[rid]}.crystalpack')
            try:
                verify_native_route(design, blocks, published)
            except (AssertionError, KeyError) as error:
                if story:
                    raise AssertionError(design['id'] + ': story atlas requires an exact whole native route after declared source overlays') from error
                source_revisions.add(rid)
                print(f'R{rid}: historical pack; source revision pending: {error}', flush=True)
            else:
                published_routes[rid] = PUBLISHED[rid]
                if story:
                    story.report.setdefault('route_grid_certificate', []).append(dict(route=design['id'], whole_grid_matches_native=True, dimensions_metatiles=[width, height], blocks_sha256=hashlib.sha256(bytes(blocks)).hexdigest()))
                for room in design['interiors']:
                    rooms.append((f'R{rid}', room['id'], room['id'].removeprefix(design['map_prefix']),
                                  installed_room(published['data']['maps'][room['id']], published, components / room['id'])))
            del published
            if region_pack is None:
                gc.collect()
        print(f'R{rid}: {width}x{height}; ' + ('native matches authoring' if rid in published_routes else 'source layout'), flush=True)
    source_revisions.difference_update(published_routes)
    town_design = read(PLAN / 'playable/grand-marais.json')
    town_pack = region_pack if region_pack else load_pack(ROOT / 'content-packs/minnesota-grand-marais.crystalpack')
    town = town_pack['data']['maps']['MnGrandMarais']
    assert town['blocks'] == [int(b, 16) for row in town_design['rows'] for b in row.split()]
    town_image = installed_room(town, town_pack, components / 'MnGrandMarais')
    for suffix, label in [('FieldStation', 'Field Station'), ('Pokecenter', 'Pokémon Center'),
                          ('Mart', 'Mart'), ('ArtistHouse', 'Folk School'),
                          ('FisherHouse', 'Fisher Cottage'), ('Lighthouse', 'Lighthouse')]:
        name = 'MnGrandMarais' + suffix
        rooms.append(('GM', name, label, installed_room(town_pack['data']['maps'][name], town_pack, components / name)))
    del town_pack
    gc.collect()
    assert len(rooms) == 6 + sum(len(designs[rid]['interiors']) for rid in published_routes)
    safari_layout = read(PLAN / 'zones/boundary-waters-safari.layout.json')
    safari_width, safari_height = safari_layout['dimensions_metatiles']
    safari_bounds = (*SAFARI_AT, SAFARI_AT[0] + safari_width, SAFARI_AT[1] + safari_height)
    cabin = read(PLAN / 'playable/root-beer-cabin.json')
    safari_pack = region_pack if region_pack else load_pack(ROOT / 'content-packs/minnesota-boundary-waters.crystalpack')
    safari_terrain, safari_source, safari_actors, cabin_map, _ = verify_boundary(
        safari_pack, safari_layout, cabin, SourceArt(safari_pack))
    save_render(safari_terrain, components / 'BoundaryWaters' / 'terrain-original.png')
    save_render(safari_source, components / 'BoundaryWaters' / 'terrain-source-original.png')
    safari_image = safari_terrain.convert('RGBA')
    for entry in safari_actors.values():
        actor = entry['actor']
        ax, ay = entry['tile']
        facing = FACING[actor['spritemovedata'].rsplit('_', 1)[-1]]
        safari_image.alpha_composite(oriented_sprite(safari_pack, actor['sprite'], facing),
                                     (ax * 16, ay * 16 - (0 if story else 4)))
    save_render(safari_image, components / 'BoundaryWaters' / 'scene-original-rgba.png')
    cabin_image = installed_room(cabin_map, safari_pack, components / cabin['id'])
    rooms.append(('BW', cabin['id'], 'Root Beer Cabin', cabin_image))
    del safari_pack
    gc.collect()

    camp_design, camp_image, walking_links = None, None, {}
    if region_pack and 'MnBoundaryWaters' in region_pack['data']['maps']:
        camp_design = read(PLAN / 'towns/MnBoundaryWaters.json')
        camp_layout = read(PLAN / 'towns/MnBoundaryWaters.layout.json')
        camp = region_pack['data']['maps']['MnBoundaryWaters']
        assert camp['blocks'] == layout_blocks(camp_layout), 'Boundary basecamp: current terrain differs'
        expected_actors = list(camp_design['residents'])
        if camp['attributes']['connections']:
            checkpoints = camp_design['regional_checkpoints']['checkpoints']
            expected_actors += [guard for checkpoint in checkpoints for guard in checkpoint['guards']]
            assert len(camp['attributes']['connections']) == 3, 'Boundary basecamp: partial walking integration'
            opposite = {'east': 'west', 'west': 'east', 'north': 'south', 'south': 'north'}
            for port in camp_design['ports']:
                link = port['native_connection']
                assert any(c['direction'] == link['direction'] and c['target_map'] == link['target_map']
                           and c['offset'] == link['offset_metatiles'] for c in camp['attributes']['connections'])
                neighbor = region_pack['data']['maps'][link['target_map']]
                assert any(c['direction'] == opposite[link['direction']] and c['target_map'] == camp_design['id']
                           and c['offset'] == link['reciprocal_offset_metatiles'] for c in neighbor['attributes']['connections'])
                walking_links[('BOUNDARY WATERS', port['route'][-2:])] = port
        expected = sorted((a['tile'][0], a['tile'][1], a['sprite']) for a in expected_actors)
        actual = sorted((a['x'], a['y'], a['sprite']) for a in camp['objects'])
        assert actual == expected, 'Boundary basecamp: native actors differ'
        camp_image = installed_room(camp, region_pack, components / camp_design['id'])
        for room in camp_design['interiors']:
            module = region_pack['data']['maps'][room['id']]
            assert module['blocks'] == [int(b, 16) for line in room['rows'] for b in line.split()], room['id']
            rooms.append(('BC', room['id'], 'Survey Shelter',
                          installed_room(module, region_pack, components / room['id'])))

    cities = {}
    if region_pack:
        for place, map_name in city_maps.items():
            if map_name not in region_pack['data']['maps']:
                continue
            design = read(PLAN / 'towns' / f'{map_name}.json')
            layout = read(PLAN / 'towns' / f'{map_name}.layout.json')
            module = region_pack['data']['maps'][map_name]
            if story:
                actual_design = dict(design, dimensions_metatiles=[module['attributes']['width'], module['attributes']['height']], doors=story.city_doors[place])
                cities[place] = dict(design=actual_design, module=module,
                                     image=installed_room(module, region_pack, components / map_name))
                owned = [pair for pair in story.report['interior_pairs'] if pair['town'] == map_name]
                for pair in owned:
                    target = pair['map']
                    rooms.append((place, target, target.removeprefix(map_name), installed_room(region_pack['data']['maps'][target], region_pack, components / target)))
                print(f'{place}: {story.epoch}-exact whole native exterior and {len(owned)} paired interiors', flush=True)
                continue
            expected_blocks, guards = layout_blocks(layout), []
            if place in {'DULUTH', 'GRAND RAPIDS'} and module['attributes']['connections']:
                source = (read(PLAN / 'towns/MnDuluth.connections.json')['regional_connections']
                          if place == 'DULUTH' else design['regional_connections'])
                expected_blocks, guards, links = guarded_city_connections(
                    region_pack, design, module, expected_blocks, source)
                walking_links.update(links)
            assert module['blocks'] == expected_blocks, place + ': current exterior terrain differs'
            assert [module['attributes']['width'], module['attributes']['height']] == design['dimensions_metatiles']
            expected = sorted((a['tile'][0], a['tile'][1], a['sprite']) for a in design['residents'] + guards)
            actual = sorted((a['x'], a['y'], a['sprite']) for a in module['objects'])
            assert actual == expected, place + ': exterior actors differ'
            cities[place] = dict(design=design, module=module,
                                 image=installed_room(module, region_pack, components / map_name))
            for room in design['interiors']:
                target = room['id']
                interior = region_pack['data']['maps'][target]
                doors = [d for d in design['doors'] if d['target'] == target]
                assert doors, target + ': missing exterior facade'
                for door in doors:
                    warp = module['events']['warps'][door['warp_index'] - 1]
                    assert [warp['x'], warp['y']] == door['tile'], target + ': native exterior door differs'
                    assert warp['target_map_constant'] == interior['attributes']['map_constant'], target + ': native door target differs'
                label = target.removeprefix(map_name)
                rooms.append((place, target, label, installed_room(interior, region_pack, components / target)))
            print(f'{place}: actual native exterior and {len(design["interiors"])} paired interiors', flush=True)

    paired_room_count = len(rooms)
    review_room_count = 0
    if region_pack:
        included = {room[1] for room in rooms}
        review_names = sorted(name for name in region_pack['data']['maps']
                              if name.startswith('Mn') and name.endswith('Gym') and name not in included)
        if 'MnGymReviewHall' in region_pack['data']['maps']:
            review_names.append('MnGymReviewHall')
        for name in review_names:
            rooms.append(('REVIEW', name, 'Unconnected review room',
                          installed_room(region_pack['data']['maps'][name], region_pack, components / name)))
        review_room_count = len(review_names)
    if story:
        walking_links = story.walking_links

    columns = 4 if len(rooms) > 24 else 3
    row_heights = [max(38, max(room[3].height + 68 for room in rooms[i:i+columns])
                       // SCALE + 1) for i in range(0, len(rooms), columns)]
    row_offsets = [sum(height + 1 for height in row_heights[:i]) for i in range(len(row_heights))]
    gallery_bottom = max(GALLERY[3], GALLERY[1] + 18 + sum(height + 1 for height in row_heights) + 3)
    canvas_height = max(SIZE[1], gallery_bottom + 10)
    canvas = Image.new('RGB', (SIZE[0] * SCALE, canvas_height * SCALE), BACKGROUND)
    draw = ImageDraw.Draw(canvas)
    blocked = cells((10, 30, 157, 176)) | cells((*GALLERY[:3], gallery_bottom))
    blocked |= cells((safari_bounds[0] - 1, safari_bounds[1] - 8,
                      safari_bounds[2] + 1, safari_bounds[3] + 1))
    blocked |= cells((CABIN_AT[0] - 1, CABIN_AT[1] - 1, CABIN_AT[0] + 21, CABIN_AT[1] + 23))
    bounds = {}
    for rid, design in designs.items():
        x, y = PLACEMENTS[rid]
        w, h = design['dimensions_metatiles']
        bounds[rid] = (x, y, x + w, y + h)
        blocked |= cells((x - 1, y - 1, x + w + 1, y + h + 1))
    hubs = {name: (x - 14, y - 3, x + 14, y + 3) for name, (x, y) in hub_centers.items()}
    hubs['GRAND MARAIS'] = (*TOWN_AT, TOWN_AT[0] + town_design['width'], TOWN_AT[1] + town_design['height'])
    if camp_design:
        width, height = camp_design['dimensions_metatiles']
        hubs['BOUNDARY WATERS'] = (*CAMP_AT, CAMP_AT[0] + width, CAMP_AT[1] + height)
    for place, city in cities.items():
        width, height = city['design']['dimensions_metatiles']
        x, y = hub_centers[place]
        hubs[place] = (x - width // 2, y - height // 2, x - width // 2 + width, y - height // 2 + height)
    for name, rect in hubs.items():
        if name in cities or name == 'GRAND MARAIS' or (name == 'BOUNDARY WATERS' and camp_design):
            x0, y0, x1, y1 = rect
            blocked |= cells((x0 - 1, y0 - 8, x1 + 1, y1 + 1))
        else:
            blocked |= cells(rect)
    used, leader_count = {}, 0
    for rid, design in designs.items():
        x0, y0, x1, y1 = bounds[rid]
        color = COLORS['native' if rid in published_routes else 'draft' if rid in source_revisions else 'blockout']
        for endpoint in design['endpoints']:
            gx, gy = endpoint['gate_metatile']
            gate = (x0 + gx + .5, y0 + gy + .5)
            side = endpoint['side']
            native_link = (endpoint['place'], rid) in walking_links
            if native_link:
                resolved_port = walking_links[(endpoint['place'], rid)]
                if 'native_route_gate_metatile' in resolved_port:
                    assert side == resolved_port['native_route_side'], rid + ': planned endpoint differs from native seam side'
                    gx, gy = resolved_port['native_route_gate_metatile']
                    gate = (x0 + gx, y0 + gy)
                gate = {'N': (gate[0], y0), 'S': (gate[0], y1),
                        'W': (x0, gate[1]), 'E': (x1, gate[1])}[side]
            stem = {'N': (round(gate[0]), y0 - 3), 'S': (round(gate[0]), y1 + 3),
                    'W': (x0 - 3, round(gate[1])), 'E': (x1 + 3, round(gate[1]))}[side]
            hub = hubs[endpoint['place']]
            if native_link:
                port = walking_links[(endpoint['place'], rid)]
                px, py = port['gate_metatile']
                hx0, hy0, hx1, hy1 = hub
                camp_gate = {'N': (hx0 + px + .5, hy0), 'S': (hx0 + px + .5, hy1),
                             'W': (hx0, hy0 + py + .5), 'E': (hx1, hy0 + py + .5)}[port['side']]
                destination = tuple(round(value) for value in {
                    'N': (camp_gate[0], hy0 - 9), 'S': (camp_gate[0], hy1 + 3),
                    'W': (hx0 - 3, camp_gate[1]), 'E': (hx1 + 3, camp_gate[1])}[port['side']])
            else:
                destination = edge(hub, stem)
                if endpoint['place'] == 'GRAND MARAIS' or endpoint['place'] in cities:
                    # Planned leaders meet a side of the built town so their
                    # endpoint cannot become trapped inside its title margin.
                    hx0, hy0, hx1, hy1 = hub
                    destination = (hx0 - 3 if stem[0] < (hx0 + hx1) / 2 else hx1 + 3,
                                   min(max(stem[1], hy0), hy1))
            path = route_leader(stem, destination, blocked, used)
            for point in path:
                used[point] = used.get(point, 0) + 1
            # Dashed annotation is intentionally distinguishable from map roads.
            draw.line([tuple(round(v * SCALE) for v in gate), tuple(v * SCALE for v in stem)], fill=color, width=4)
            for i, (a, b) in enumerate(zip(path, path[1:])):
                if native_link or i % 5 < 3:
                    draw.line([tuple(v * SCALE for v in a), tuple(v * SCALE for v in b)], fill=color, width=4)
            hx0, hy0, hx1, hy1 = hub
            boundary = camp_gate if native_link else (min(max(destination[0], hx0), hx1), min(max(destination[1], hy0), hy1))
            draw.line([tuple(v * SCALE for v in destination),
                       tuple(v * SCALE for v in boundary)], fill=color, width=4)
            leader_count += 1
    assert leader_count == 40
    # Optional admission is a proposed warp link, not a walking connection.
    arrival = safari_layout['entry_contract']['arrival_tile']
    gate = (SAFARI_AT[0] + arrival[0] / 2, SAFARI_AT[1] + arrival[1] / 2)
    stem = (safari_bounds[0] - 3, round(gate[1]))
    destination = edge(hubs['BOUNDARY WATERS'], stem)
    path = route_leader(stem, destination, blocked, used)
    draw.line([tuple(round(v * SCALE) for v in gate), tuple(v * SCALE for v in stem)], fill=COLORS['draft'], width=4)
    for index, (a, b) in enumerate(zip(path, path[1:])):
        if index % 5 < 3:
            draw.line([tuple(v * SCALE for v in a), tuple(v * SCALE for v in b)], fill=COLORS['draft'], width=4)

    x, y = SAFARI_AT
    canvas.paste(safari_image.resize((safari_width * SCALE, safari_height * SCALE), Image.Resampling.NEAREST), (x * SCALE, y * SCALE))
    draw.rectangle((x * SCALE - 3, y * SCALE - 3, (x + safari_width) * SCALE + 2, (y + safari_height) * SCALE + 2), outline=COLORS['native'], width=3)
    draw.text((x * SCALE, (y - 7) * SCALE), 'BOUNDARY WATERS SAFARI', font=font(23, True), fill=COLORS['native'])
    draw.text((x * SCALE, (y - 4) * SCALE), 'NATIVE TERRAIN + CABIN · EXPEDITIONS CLOSED', font=font(17), fill=COLORS['native'])
    # The actual empty room has two working carpets; Dorothy remains outside.
    cx, cy = (value * SCALE for value in CABIN_AT)
    draw.rectangle((cx, cy, cx + 20 * SCALE, cy + 22 * SCALE), outline=COLORS['native'], width=2)
    draw.text((cx + 9, cy + 10), 'ROOT BEER CABIN', font=font(19, True), fill=COLORS['native'])
    draw.text((cx + 9, cy + 40), 'NATIVE ROOM · BOTH RETURNS', font=font(13), fill=COLORS['native'])
    canvas.paste(cabin_image, (cx + 30, cy + 76))

    for rid, image in routes.items():
        x, y = PLACEMENTS[rid]
        w, h = designs[rid]['dimensions_metatiles']
        category = 'native' if rid in published_routes else 'draft' if rid in source_revisions else 'blockout'
        color = COLORS[category]
        canvas.paste(image.resize((w * SCALE, h * SCALE), Image.Resampling.NEAREST), (x * SCALE, y * SCALE))
        draw.rectangle((x * SCALE - 3, y * SCALE - 3, (x + w) * SCALE + 2, (y + h) * SCALE + 2), outline=color, width=3)
        draw.rectangle((x * SCALE - 4, (y - 7) * SCALE, (x + w) * SCALE + 4, y * SCALE - 5), fill=BACKGROUND)
        draw.text((x * SCALE, (y - 7) * SCALE), f'R{rid}  {designs[rid]["name"]}', font=font(23, True), fill='#f4f7fb')
        label = {'native': 'NATIVE SLICE', 'draft': 'SOURCE REVISION · PUBLICATION PENDING', 'blockout': 'EARLIER BLOCKOUT'}[category]
        draw.text((x * SCALE, (y - 4) * SCALE), f'{label}  ·  {w} × {h}', font=font(17), fill=color)
    for name, (x0, y0, x1, y1) in hubs.items():
        if name in cities or name == 'GRAND MARAIS' or (name == 'BOUNDARY WATERS' and camp_image is not None):
            image = cities[name]['image'] if name in cities else town_image if name == 'GRAND MARAIS' else camp_image
            canvas.paste(image.resize(((x1 - x0) * SCALE, (y1 - y0) * SCALE), Image.Resampling.NEAREST),
                         (x0 * SCALE, y0 * SCALE))
            draw.rectangle(tuple(v * SCALE for v in (x0 - 1, y0 - 1, x1 + 1, y1 + 1)), outline=COLORS['native'], width=3)
            draw.text((x0 * SCALE, (y0 - 6) * SCALE), name, font=font(23, True), fill=COLORS['native'])
        else:
            draw.rectangle(tuple(v * SCALE for v in (x0, y0, x1, y1)), fill='#203443', outline='#7691a2', width=2)
            draw.text(((x0 + x1) / 2 * SCALE, (y0 + y1) / 2 * SCALE - 3), name, font=font(21, True), fill='#e5edf3', anchor='mm')
            caption = 'SAFARI BASECAMP · PLANNED' if name == 'BOUNDARY WATERS' else 'HUB TO BUILD'
            draw.text(((x0 + x1) / 2 * SCALE, y1 * SCALE + 5), caption, font=font(13), fill='#96aabd', anchor='mt')

    # Every built interior has a matching exterior door tag, rather than a
    # line that would obscure the map artwork over a long atlas distance.
    column_width = (GALLERY[2] - GALLERY[0]) // columns
    frame_width = column_width - 3
    room_indices = {entry[1]: index for index, entry in enumerate(rooms)}
    for index, (parent, name, label, image) in enumerate(rooms):
        tag = room_tag(index)
        if parent == 'GM':
            door = next(d for d in town['events']['warps'] if d['target_map_constant'] == town_pack_constant(name))
            entrances = [(TOWN_AT[0] + door['x'] / 2, TOWN_AT[1] + door['y'] / 2)]
        elif parent == 'BW':
            door = safari_layout['root_beer_lady']['door']['tile']
            entrances = [(SAFARI_AT[0] + door[0] / 2, SAFARI_AT[1] + door[1] / 2)]
        elif parent == 'BC':
            entrances = [(CAMP_AT[0] + door['tile'][0] / 2, CAMP_AT[1] + door['tile'][1] / 2)
                         for door in camp_design['doors'] if door['target'] == name]
            assert entrances, name + ': basecamp interior has no actual door'
        elif parent in cities:
            x, y, _, _ = hubs[parent]
            entrances = [(x + door['tile'][0] / 2, y + door['tile'][1] / 2)
                         for door in cities[parent]['design']['doors'] if door['target'] == name]
            internal_pair = story.room_sources.get(name) if story else None
            assert entrances or (internal_pair and internal_pair['pair_kind'] == 'internal_stairs'), name + ': city interior has no actual exterior/internal door'
        elif parent == 'REVIEW':
            # These actual rooms have no published town facade. Showing them
            # separately must not fabricate an exterior or regional connection.
            entrances = []
        else:
            design = designs[parent[1:]]
            x, y = PLACEMENTS[parent[1:]]
            entrances = [(x + door['tile'][0] / 2, y + door['tile'][1] / 2)
                         for door in design['doors'] if door['target'] == name]
            assert entrances, name + ': interior has no exterior entrance'
        for at in entrances:
            px, py = (round(v * SCALE) for v in at)
            half_width = max(12, round(draw.textlength(tag, font=font(17, True)) / 2) + 4)
            draw.rectangle((px - half_width, py - 10, px + half_width, py + 14), fill='#162637', outline='#efda9b', width=2)
            draw.text((px + 1, py + 1), tag, font=font(17, True), fill='#fff0c4', anchor='mm')
        col, row = index % columns, index // columns
        x, y = (GALLERY[0] + col * column_width) * SCALE, (GALLERY[1] + 18 + row_offsets[row]) * SCALE
        frame_height = row_heights[row] * SCALE
        assert y + frame_height <= gallery_bottom * SCALE, name + ': room gallery would clip'
        draw.rectangle((x, y, x + frame_width * SCALE, y + frame_height), outline='#3d5669', width=2)
        pair = story.room_sources.get(name) if story else None
        if pair and pair['pair_kind'] == 'internal_stairs':
            label += ' · via ' + room_tag(room_indices[pair['parent_map']]) + ' stairs'
        draw.text((x + 14, y + 12), f'{tag}  {parent} / {label}', font=font(23, True), fill='#f2e2b7')
        draw.text((x + 14, y + 44), name, font=font(16), fill='#adc4d3')
        assert image.height + 68 <= frame_height and image.width <= frame_width * SCALE, name + ': room would overflow its review frame'
        canvas.paste(image, (x + (frame_width * SCALE - image.width) // 2, y + 68))
    if story:
        for name, pair in story.room_sources.items():
            if pair['pair_kind'] != 'internal_stairs':
                continue
            parent_index = room_indices[pair['parent_map']]
            parent_image = rooms[parent_index][3]
            col, row = parent_index % columns, parent_index // columns
            gx = (GALLERY[0] + col * column_width) * SCALE + (frame_width * SCALE - parent_image.width) // 2
            gy = (GALLERY[1] + 18 + row_offsets[row]) * SCALE + 68
            px, py = gx + pair['source_tile'][0] * 16 + 8, gy + pair['source_tile'][1] * 16 + 8
            tag = room_tag(room_indices[name])
            draw.rectangle((px - 15, py - 10, px + 15, py + 14), fill='#162637', outline='#efda9b', width=2)
            draw.text((px, py + 1), tag, font=font(17, True), fill='#fff0c4', anchor='mm')
            pair['parent_gallery_tag'] = room_tag(parent_index)
            pair['room_tag'] = tag
    gallery_title = f'{paired_room_count} PAIRED INTERIORS'
    if review_room_count:
        gallery_title += f' + {review_room_count} UNCONNECTED REVIEW ROOMS'
    draw.text((GALLERY[0] * SCALE, GALLERY[1] * SCALE), gallery_title, font=font(38, True), fill='#eff5f8')
    draw.text((GALLERY[0] * SCALE, (GALLERY[1] + 5) * SCALE), 'Letter tags join actual doors/stairs; REVIEW rooms have no town facade.' if story else 'Letter tags join paired rooms to actual doors; REVIEW rooms have no town facade.', font=font(22), fill='#b8cbd7')
    draw.text((GALLERY[0] * SCALE, (GALLERY[1] + 9) * SCALE), 'Rooms use native pixels; exterior overview uses one shared scale.', font=font(19), fill='#94adbe')

    draw.text((12 * SCALE, 2 * SCALE), 'MINNESOTA · CURRENT MAPS, CONNECTED', font=font(49, True), fill='#f2f5fa')
    draw.text((12 * SCALE, 9 * SCALE), 'Actual maps · solid native walking links / dashed planned links · visual approval and full regional integration remain open', font=font(25), fill='#b2c8d8')
    x, y = 14 * SCALE, 35 * SCALE
    lines = [
        ('20 ROUTE LAYOUTS', 38, '#f2f5fa'),
        (f'{len(published_routes)} native routes + {1 + bool(camp_design) + len(cities)} towns', 28, '#d8e7ee'),
        (f'{paired_room_count} paired interiors with matching door tags', 27, '#d8e7ee'),
        (f'{review_room_count} unconnected gym review rooms', 27, '#d8e7ee'),
        ('', 24, '#d8e7ee'),
        ('TEAL · native local slices', 28, COLORS['native']),
        ('Publication does not imply visual acceptance.', 25, '#d8e7ee'),
        (f'GRAY · {len(routes) - len(published_routes) - len(source_revisions)} earlier blockouts to redesign', 28, COLORS['blockout']),
        (f'AMBER · {len(source_revisions)} route revisions; regional links pending', 25, COLORS['draft']),
        ('Preserve terrain/cabin built; expeditions remain closed.', 25, '#d8e7ee'),
        ('', 24, '#d8e7ee'),
        (f'{len(walking_links)} installed walking links (geometry certified).' if story else f'{len(walking_links)} solid guarded walking links installed.', 25 if story else 27, COLORS['native']),
        ('Dashed links remain planned hub travel.', 27, '#d8e7ee'),
        ('Hub boxes reserve towns that are not built yet.', 27, '#d8e7ee'),
        ('Each route is the current complete terrain raster.', 26, '#d8e7ee'),
        ('Route sizes are preserved at a common scale.', 27, '#d8e7ee'),
        ('', 24, '#d8e7ee'),
        ('Size can vary. Worthwhile features and pacing matter.', 26, '#f2e2b7'),
        ('This is a progress view, not a finished region.', 27, '#d8e7ee'),
    ]
    for text, size, color in lines:
        draw.text((x, y), text, font=font(size, size >= 38), fill=color)
        y += 52
    draw.text((12 * SCALE, (canvas_height - 8) * SCALE), 'Source pixels: user-supplied local pack · no generated connecting terrain · 40 regional endpoint links shown', font=font(23), fill='#adc4d3')
    output.parent.mkdir(parents=True, exist_ok=True)
    assert all(path.read_bytes() == raw for path, raw in source_epoch.items()), 'A planned source changed while the atlas rendered; retain this attempt and start a new epoch'
    save_render(canvas, output, optimize=True)
    # Smaller identical composition is for visual QA only; deliver the full PNG.
    save_render(canvas.resize((canvas.width // 3, canvas.height // 3), Image.Resampling.BOX), output.with_name('progress-inspection.png'))
    summary = dict(
        output=str(output), dimensions_pixels=list(canvas.size),
        image_sha256=hashlib.sha256(output.read_bytes()).hexdigest(),
        source_pack_sha256=hashlib.sha256(base_path.read_bytes()).hexdigest(),
        region_pack_sha256=hashlib.sha256(region_path.read_bytes()).hexdigest() if region_path else None,
        native_routes=sorted(published_routes), pending_source_routes=sorted(source_revisions),
        towns=['GRAND MARAIS'] + (['BOUNDARY WATERS'] if camp_design else []) + list(cities),
        paired_room_count=paired_room_count, unconnected_review_room_count=review_room_count,
        room_map_ids=[room[1] for room in rooms],
        route_endpoint_leaders=leader_count, **({'installed_walking_links': len(walking_links)} if story else {'solid_guarded_walking_links': len(walking_links)}),
        planned_endpoint_leaders=leader_count - len(walking_links),
        native_ports=[dict(place=place, route=rid, side=port['side'], gate_metatile=port['gate_metatile'])
                      for (place, rid), port in sorted(walking_links.items())])
    output.with_name('atlas-summary.json').write_text(json.dumps(summary, indent=2) + '\n')
    if story:
        story.report['source_provenance'] = [{'path': str(path.relative_to(PLAN)), 'sha256': hashlib.sha256(raw).hexdigest()} for path, raw in source_epoch.items()]
        summary.update(render_mode=f'{story.map_count}-map {story.epoch}-certified native module composition', rendered_native_art_pack_sha256=summary['region_pack_sha256'], render_scope='Native source-art reconstruction; not a production GPU capture or gameplay proof')
        output.with_name('atlas-summary.json').write_text(json.dumps(summary, indent=2) + '\n')
        story.report.update(region_pack_sha256=summary['region_pack_sha256'], paired_room_count=paired_room_count, unconnected_review_room_count=review_room_count, town_count=len(summary['towns']))
        output.with_name('story-certificate.json').write_text(json.dumps(story.report, indent=2) + '\n')
        save_review_crops(canvas, output, hubs, GALLERY, row_offsets, row_heights, SCALE, story.cities)
    print(f'Atlas: {output} · {canvas.width}×{canvas.height} · 20 routes, {1 + bool(camp_design) + len(cities)} towns, {paired_room_count} paired rooms + {review_room_count} review rooms, 40 endpoint links', flush=True)


def town_pack_constant(name):
    suffix = name.removeprefix('MnGrandMarais')
    labels = {'FieldStation': 'MN_GRAND_MARAIS_FIELD_STATION', 'Pokecenter': 'MN_GRAND_MARAIS_POKECENTER',
              'Mart': 'MN_GRAND_MARAIS_MART', 'ArtistHouse': 'MN_GRAND_MARAIS_ARTIST_HOUSE',
              'FisherHouse': 'MN_GRAND_MARAIS_FISHER_HOUSE', 'Lighthouse': 'MN_GRAND_MARAIS_LIGHTHOUSE'}
    return labels[suffix]


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--pack', type=Path, default=ROOT / 'content-packs/text-tui.crystalpack')
    parser.add_argument('--region-pack', type=Path,
                        help='Compare every native map against one composed review pack')
    parser.add_argument('--output', type=Path, default=PLAN / 'renders/atlas/minnesota-current-connected.png')
    parser.add_argument('--story-baseline', type=Path, help='Enable exact native-city mode using the certified DA27/538 or03C9/541 baseline pack')
    args = parser.parse_args()
    render(args.pack, args.output, args.region_pack, args.story_baseline)


if __name__ == '__main__':
    main()
