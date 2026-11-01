#!/usr/bin/env python3
"""Read-only published basecamp/source comparison; preserves rejected render passes.

This consumes completed external packs. It never compiles/publishes maps, changes
story state, exports a catalog, or substitutes for production controller proof.
"""
import argparse
from collections import deque
import hashlib
import json
from pathlib import Path
import time

from PIL import Image

from render_archive import retain_render_file, save_render
from render_reference import SourceArt, layout_blocks, load_pack, room_sheet
from review_north_shore import oriented_sprite

ROOT = Path(__file__).resolve().parents[3]
TOWN = ROOT / 'planning/minnesota/towns/MnBoundaryWaters.json'
LAYOUT = ROOT / 'planning/minnesota/towns/MnBoundaryWaters.layout.json'
OUTPUT = ROOT / 'planning/minnesota/renders/boundary-basecamp/native-compare'
MAP = 'MnBoundaryWaters'
CONSTANT = 'MN_BOUNDARY_WATERS'
ROOM = 'MnBoundaryWatersSurveyShelter'
ROOM_CONSTANT = 'MN_BOUNDARY_WATERS_SURVEY_SHELTER'
CHIEF = 'MN_SURVEY_CHIEF'
RECEIVER = 'MN_BOUNDARY_SURVEY_RECEIVER'
MEDIC = 'MN_BOUNDARY_SURVEY_MEDIC'
SIGNAL = 'EVENT_MN_LIGHTHOUSE_SIGNAL'
DEFEATED = 'EVENT_MN_SURVEY_CHIEF_DEFEATED'
RESTORED = 'EVENT_MN_SURVEY_RESTORED'
BADGES = ['ENGINE_ZEPHYRBADGE', 'ENGINE_HIVEBADGE', 'ENGINE_PLAINBADGE',
          'ENGINE_FOGBADGE', 'ENGINE_STORMBADGE', 'ENGINE_MINERALBADGE',
          'ENGINE_GLACIERBADGE', 'ENGINE_RISINGBADGE']
FACING = dict(UP='north', DOWN='south', LEFT='west', RIGHT='east')
CONNECTION_MANIFEST = 'minnesota-boundary-basecamp-connections-v1'


def connected(pack):
    return CONNECTION_MANIFEST in pack['report']['manifests']


def checkpoint_actors(town):
    return [dict(actor, script=checkpoint['script_root'])
            for checkpoint in town['regional_checkpoints']['checkpoints']
            for actor in checkpoint['guards']]


def expected_town_actors(pack, town):
    # This is the initial unqualified body composition. Saved/qualified native
    # visibility is deliberately left to the production-controller proof.
    return town['residents']+(checkpoint_actors(town) if connected(pack) else [])


def adjacent(tile):
    x, y = tile
    return [(x-1, y), (x+1, y), (x, y-1), (x, y+1)]


def flood(start, cells, doors=frozenset()):
    if start not in cells:
        return set()
    seen, pending = {start}, deque([start])
    while pending:
        for tile in adjacent(pending.popleft()):
            if tile in cells and tile not in seen:
                seen.add(tile)
                if tile not in doors:
                    pending.append(tile)
    return seen


def collision(pack, module, tile):
    x, y = tile
    width, height = module['attributes']['width'], module['attributes']['height']
    assert 0 <= x < width*2 and 0 <= y < height*2, 'Collision coordinate outside native map'
    block = module['blocks'][y//2*width+x//2]
    return pack['data']['tilesets'][module['attributes']['tileset_name']]['collision'][f'{block:02x}'][y%2*2+x%2]


def text_lines(module, label):
    return [entry['args'][0].strip('"') for entry in module['script_text_bodies'][label]['commands']
            if entry['command'] in {'text', 'line', 'para'}]


class Artifacts:
    """No conversion is implicit; retain each original and derived image."""
    def __init__(self, directory):
        self.directory = directory
        self.files = {}
        directory.mkdir(parents=True, exist_ok=True)

    def save(self, name, image):
        identity = f'{image.mode}:{image.width}x{image.height}:'.encode()+image.tobytes()
        digest = hashlib.sha256(identity).hexdigest()[:12]
        path = self.directory / f'{name}-{digest}.png'
        if path.exists():
            with Image.open(path) as previous:
                assert (previous.mode, previous.size, previous.tobytes()) == (image.mode, image.size, image.tobytes()), 'Immutable image collision'
            retain_render_file(path)
        else:
            save_render(image, path)
        self.files[name] = dict(file=path.name, mode=image.mode, size=list(image.size))
        return path.name

    def cameras(self, prefix, scene, viewpoints):
        views = []
        for view in viewpoints:
            x, y = view['tile']
            name = prefix+'-'+view['id'].lower()
            left = min(max(x*16-80, 0), max(scene.width-160, 0))
            top = min(max(y*16-72, 0), max(scene.height-144, 0))
            box = (left, top, left+160, top+144)
            original = scene.crop(box)
            self.save(name+'-original-160x144', original)
            self.files[name+'-original-160x144'].update(requested_tile=view['tile'], source_crop_box=list(box))
            rgb = original.convert('RGB')
            self.save(name+'-rgb-160x144', rgb)
            self.save(name+'-2x', rgb.resize((320, 288), Image.Resampling.NEAREST))
            self.save(name+'-4x', rgb.resize((640, 576), Image.Resampling.NEAREST))
            views.append((view['id'], rgb))
        if views:
            self.save(prefix+'-camera-sheet', room_sheet(views, columns=3))


def native_terrain(pack, art, name):
    module = pack['data']['maps'][name]
    attrs = module['attributes']
    return art.render(attrs['tileset_name'], attrs['width'], attrs['height'], module['blocks'])


def actor_scene(pack, terrain, actors):
    scene = terrain.convert('RGBA')
    for actor in actors:
        if 'spritemovedata' in actor:
            facing = FACING[actor['spritemovedata'].rsplit('_', 1)[-1]]
            tile = (actor['x'], actor['y'])
            assert actor['pal'] == 0, 'This static review requires the actual default native actor palette'
        else:
            facing, tile = actor['facing'], actor['tile']
        scene.alpha_composite(oriented_sprite(pack, actor['sprite'], facing), (tile[0]*16, tile[1]*16-4))
    return scene


def render_set(pack, art, artifacts, prefix, terrain, actors, viewpoints):
    # Keep source-pixel terrain and the RGBA canvas before compositing actors.
    artifacts.save(prefix+'-terrain-original', terrain)
    artifacts.save(prefix+'-terrain-rgba', terrain.convert('RGBA'))
    scene = actor_scene(pack, terrain, actors)
    artifacts.save(prefix+'-whole-original-rgba', scene)
    artifacts.save(prefix+'-whole-rgb', scene.convert('RGB'))
    artifacts.cameras(prefix, scene, viewpoints)
    return scene


def verify_maps(pack, source, town, layout, expected_blocks):
    assert town['id'] == layout['id'] == MAP and town['native_constant'] == CONSTANT
    assert town['dimensions_metatiles'] == layout['dimensions_metatiles'] == [32, 24]
    assert town['compiled_tileset'] == layout['compiled_tileset'] == 'mn_boundary_safari'
    assert town['interiors'][0]['id'] == ROOM and town['interiors'][0]['native_constant'] == ROOM_CONSTANT
    for name, constant, size, tileset, map_id in [
        (MAP, CONSTANT, [32, 24], 'mn_boundary_safari', 1),
        (ROOM, ROOM_CONSTANT, [8, 6], 'lab', 2),
    ]:
        module = pack['data']['maps'][name]
        attr = module['attributes']
        assert [attr['width'], attr['height']] == size, name+': native dimensions'
        assert attr['map_constant'] == constant and attr['tileset_name'] == tileset
        metadata = pack['data']['runtime_map_metadata'][constant]
        assert metadata['name'] == name and metadata['constant'] == constant
        assert (metadata['groupId'], metadata['mapId']) == (62, map_id)
        assert [metadata['width'], metadata['height']] == size
        assert len(module['blocks']) == size[0]*size[1]
        assert name not in pack['data']['wild_encounters'], 'A free town is not a Safari encounter mode'
    assert pack['data']['maps'][MAP]['blocks'] == expected_blocks, 'All 768 actual town blocks must match current authored strokes'
    room_blocks = [int(block, 16) for row in town['interiors'][0]['rows'] for block in row.split()]
    assert pack['data']['maps'][ROOM]['blocks'] == room_blocks, 'All 48 actual shelter blocks must match current source rows'
    assert pack['data']['runtime_spawn_points'] == source['data']['runtime_spawn_points'], 'Native HOME/recovery table changed'
    return dict(exterior_blocks=768, shelter_blocks=48, native_group=62,
                town_size=[32, 24], shelter_size=[8, 6], unchanged_home_recovery=True,
                native_connections=pack['data']['maps'][MAP]['attributes']['connections'])


def verify_art(pack, source, actual_art, source_art, town, layout, expected_blocks):
    used_sprites = {actor['sprite'] for actor in expected_town_actors(pack, town)+town['interiors'][0]['residents']}
    for tileset in ['mn_boundary_safari', 'lab']:
        assert pack['data']['tilesets'][tileset] == source['data']['tilesets'][tileset], 'Original native art/collision definition changed: '+tileset
    keys = [key for key in source['runtime_files']
            if key == 'gfx/tilesets/bg_tiles.pal'
            or any(key.startswith('gfx/tilesets/'+name+'.') or key.startswith('data/tilesets/'+name+'_')
                   for name in ['mn_boundary_safari', 'lab'])
            or key in {'gfx/sprites/'+token.removeprefix('SPRITE_').lower()+'.png' for token in used_sprites}]
    for key in keys:
        assert pack['runtime_files'][key] == source['runtime_files'][key], 'Native source art bytes changed: '+key
    expected_town = source_art.render('mn_boundary_safari', 32, 24, expected_blocks)
    expected_room = source_art.render('lab', 8, 6, [int(b, 16) for row in town['interiors'][0]['rows'] for b in row.split()])
    assert native_terrain(pack, actual_art, MAP).tobytes() == expected_town.tobytes(), 'Whole native/source town pixel mismatch'
    assert native_terrain(pack, actual_art, ROOM).tobytes() == expected_room.tobytes(), 'Whole native/source shelter pixel mismatch'
    return dict(unchanged_native_art_files=len(keys), native_source_town_pixels_equal=True,
                native_source_shelter_pixels_equal=True)


def verify_actors(pack, town):
    counts = {}
    for name, design in [(MAP, town), (ROOM, town['interiors'][0])]:
        module = pack['data']['maps'][name]
        actual = {actor['object_identifier']: actor for actor in module['objects']}
        residents = expected_town_actors(pack, town) if name == MAP else design['residents']
        expected = {actor['id']: actor for actor in residents}
        assert len(actual) == len(module['objects']), name+': duplicate native actor identity'
        assert actual.keys() == expected.keys(), name+': actual actors must match authoritative source'
        for key, source in expected.items():
            assert source.get('object_identifier', key) == key
            assert source.get('object_type', 'OBJECTTYPE_SCRIPT') == 'OBJECTTYPE_SCRIPT'
            actor = actual[key]
            assert [actor['x'], actor['y']] == source['tile'] and actor['sprite'] == source['sprite'], key+': native body mismatch'
            assert actor['script'] == source.get('script', key) and actor['object_type'] == 'OBJECTTYPE_SCRIPT'
            assert actor['event_flag'] == str(source.get('event_flag', -1)) and actor['pal'] == 0
            assert FACING[actor['spritemovedata'].rsplit('_', 1)[-1]] == source['facing'], key+': facing changed'
        signs = module['events']['bg_events']
        assert len(signs) == len(design['signs'])
        assert {sign['script']: [sign['x'], sign['y']] for sign in signs} == {sign['id']: sign['tile'] for sign in design['signs']}
        assert all(sign['event_type'] == 'BGEVENT_READ' for sign in signs)
        for source in design['signs']:
            if source['id'] != RECEIVER:
                assert text_lines(module, source['id']+'Text') == source['lines']
        for source in design['residents']:
            if source['id'] != CHIEF:
                assert text_lines(module, source['id']+'Text') == source['lines']
        counts[name] = dict(actors=len(actual), signs=len(signs))
    return dict(native_source_actors_signs=counts)


def verify_warps(pack, town):
    outside, room = pack['data']['maps'][MAP], pack['data']['maps'][ROOM]
    door, interior = town['doors'][0], town['interiors'][0]
    assert door['tile'] == [43, 19] and door['return_tile'] == door['arrival_tile'] == [43, 20]
    assert (door['warp_index'], door['return_warp_index'], door['target'], door['target_warp']) == (1, 2, ROOM, 3)
    assert interior['passive_arrival'] == dict(index=3, tile=[8, 9])
    assert interior['clear_arrival'] == [8, 9] and interior['exits'] == [[8, 11], [9, 11]]
    expected = [(outside, [(1, [43, 19], 'DOOR', ROOM_CONSTANT, 3), (2, [43, 20], 'FLOOR', ROOM_CONSTANT, 3)]),
                (room, [(1, [8, 11], 'WARP_CARPET_DOWN', CONSTANT, 2),
                        (2, [9, 11], 'WARP_CARPET_DOWN', CONSTANT, 2),
                        (3, [8, 9], 'FLOOR', CONSTANT, 2)])]
    for module, bindings in expected:
        assert len(module['events']['warps']) == len(bindings)
        for actual, (index, tile, kind, target, target_index) in zip(module['events']['warps'], bindings):
            assert actual['index'] == index and [actual['x'], actual['y']] == tile
            assert actual['target_map'] == actual['target_map_constant'] == target, 'Native warp target fields require the constant, not a map name'
            assert actual['target_warp_id'] == target_index
            assert collision(pack, module, tile) == kind
        if module is room or not connected(pack):
            assert not module['events']['coord_events'], 'Unexpected native coordinate event in bare town or shelter'
    for module, token, cells in [(outside, 'DOOR', {(43, 19)}), (room, 'WARP_CARPET_DOWN', {(8, 11), (9, 11)})]:
        attr = module['attributes']
        found = {(x, y) for y in range(attr['height']*2) for x in range(attr['width']*2) if collision(pack, module, (x, y)) == token}
        assert found == cells, 'Unbound native door/carpet quadrant'
    for module, tile in [(outside, (43, 20)), (room, (8, 9))]:
        assert all((actor['x'], actor['y']) != tile for actor in module['objects']), 'Passive arrival occupied'
    return dict(outside_door=[43, 19], outside_passive_return=[43, 20], shelter_passive_arrival=[8, 9],
                shelter_passive_index=3, actual_exit_carpets=[[8, 11], [9, 11]],
                paired_constant_fields=True)


def verify_geometry(pack, town):
    outside, room = pack['data']['maps'][MAP], pack['data']['maps'][ROOM]
    checkpoint_bodies = {tuple(actor['tile']) for actor in checkpoint_actors(town)} if connected(pack) else set()
    def graph(module, clear_checkpoints=False):
        attr = module['attributes']
        cells = {(x, y) for y in range(attr['height']*2) for x in range(attr['width']*2)}
        bodies = {(actor['x'], actor['y']) for actor in module['objects']}
        if clear_checkpoints and module is outside:
            bodies -= checkpoint_bodies
        walk = {p for p in cells if collision(pack, module, p) in {'FLOOR', 'TALL_GRASS', 'DOOR', 'WARP_CARPET_DOWN'}}-bodies
        water = {p for p in cells if collision(pack, module, p) == 'WATER'}
        return cells, walk, water
    cells, walk, water = graph(outside)
    doors = {(43, 19)}
    seen = flood((6, 24), walk, doors)
    surf = flood((6, 24), walk | water, doors)
    closed_seen, closed_surf = seen, surf
    if connected(pack):
        assert not {p for p in seen if p[0] in {0, 63} or p[1] in {0, 47}}, 'Unqualified checkpoint walking bypass'
        for checkpoint in town['regional_checkpoints']['checkpoints']:
            assert all(tuple(p) in seen for p in checkpoint['inner_coord_tiles'])
            assert not {tuple(p) for p in checkpoint['outer_inward_coord_tiles']} & surf, 'Unqualified checkpoint walking/Surf bypass'
        # Geometry-only conditional path: native temporary visibility and actual
        # permission flow must be established by the controller, not this graph.
        _, open_walk, _ = graph(outside, clear_checkpoints=True)
        seen = flood((6, 24), open_walk, doors)
        surf = flood((6, 24), open_walk | water, doors)
    assert not any(collision(pack, outside, p) == 'CUT_TREE' for p in cells), 'Incidental Cut gate'
    assert not {p for p in water if p[0] in {0, 63} or p[1] in {0, 47}}, 'Unconnected outer water requires visible native closure'
    outer = {p for p in seen if p[0] in {0, 63} or p[1] in {0, 47}}
    assert outer == {(0, 24), (0, 25), (63, 24), (63, 25), (32, 47), (33, 47)}, 'Unexpected outer walking shoulder or disconnected free port'
    for port in town['ports']:
        for key in ['gate_metatile', 'landing_metatile']:
            assert tuple(2*v for v in port[key]) in seen, 'Free native route port disconnected'
    assert {(43, 19), (43, 20), (32, 8), (32, 10)} <= seen
    assert (32, 5) not in surf, 'Closed expedition gate has a native walking/Surf bypass'
    dock = town['public_dock']
    assert tuple(dock['launch_tile']) in seen and tuple(dock['water_target']) in water
    for key, expected in [('north_rail_blocks', ['WALL', 'WALL', 'FLOOR', 'FLOOR']),
                          ('south_rail_blocks', ['FLOOR', 'FLOOR', 'WALL', 'WALL'])]:
        for x, y in dock[key]:
            assert [collision(pack, outside, (x*2+dx, y*2+dy)) for dy in range(2) for dx in range(2)] == expected
    for design, module, reached in [(town, outside, seen), (town['interiors'][0], room, flood((8, 9), graph(room)[1]))]:
        for actor in design['residents']:
            assert collision(pack, module, actor['tile']) == 'FLOOR'
            assert set(adjacent(actor['tile'])) & reached, actor['id']+': no reachable native approach'
        for sign in design['signs']:
            assert collision(pack, module, sign['tile']) == 'WALL'
            assert set(adjacent(sign['tile'])) & reached, sign['id']+': no native reading approach'
    room_seen = flood((8, 9), graph(room)[1])
    assert {(8, 5), (10, 5), (9, 6), (12, 4), (4, 7), (8, 11), (9, 11)} <= room_seen
    assert collision(pack, room, (12, 3)) == 'WALL' and room['blocks'][14] == 0x08, 'Actual native labPC08 receiver required'
    return dict(reachable_walking_tiles=len(seen), reachable_public_water_tiles=len(surf & water),
                shelter_reachable_tiles=len(room_seen), free_three_ports=True,
                closed_admission_bypass=False, unconnected_outer_water_cells=0,
                unqualified_reachable_walking_tiles=len(closed_seen),
                unqualified_reachable_water_tiles=len(closed_surf & water),
                port_geometry_visibility='checkpoint bodies removed only for static conditional topology' if connected(pack) else 'bare town')


def verify_story_and_services(pack, source, town):
    outside, room = pack['data']['maps'][MAP], pack['data']['maps'][ROOM]
    global_flags = pack['data']['story_event_script_constants']['global']
    assert all(name in global_flags for name in [SIGNAL, DEFEATED, RESTORED])
    assert len({global_flags[name] for name in [SIGNAL, DEFEATED, RESTORED]}) == 3
    for key, value in source['data']['story_event_script_constants']['global'].items():
        assert global_flags[key] == value, 'Existing global story constant changed: '+key
    chief = next(actor for actor in town['interiors'][0]['residents'] if actor['id'] == CHIEF)
    trainer = pack['data']['trainers']['trainers'][chief['battle']]
    assert trainer['trainer_class'] == 'EXECUTIVEM'
    assert [(p['species'], p['level']) for p in trainer['party']] == [(p['species'], p['level']) for p in chief['party']]
    assert chief['win_flag'] == DEFEATED and town['story_contract']['Q07']['MS12_sets'] == RESTORED
    assert sorted(chief['requires']) == sorted(['BADGE_'+str(n) for n in range(1, 9)]+[SIGNAL])
    assert town['story_contract']['Q07']['MS11_sets'] == DEFEATED
    assert town['story_contract']['Q07']['shelter'] == ROOM and town['story_contract']['Q07']['chief_actor'] == CHIEF
    assert town['story_contract']['Q07']['battle'] == chief['battle']
    checks = [c for c in room['script_flag_commands'] if c['source_script'] == CHIEF and c['command'] == 'checkflag']
    assert [c['flag_id'] for c in checks] == BADGES, 'Actual chief must check every Johto badge'
    flags = room['script_flag_commands']
    assert any(c['source_script'] == CHIEF and c['command'] == 'checkevent' and c['flag_id'] == SIGNAL for c in flags)
    actual_sets = sorted((c['source_script'], c['flag_id']) for c in flags if c['command'] == 'setevent')
    assert actual_sets == sorted([(RECEIVER, RESTORED), (CHIEF, DEFEATED)])
    assert not any('SAFARI' in c['flag_id'] for c in flags), 'Q07 cannot require or grant a Safari state'
    body = room['scripts'][CHIEF]
    commands = [entry['command'] for entry in body]
    assert commands.index('loadtrainer') < commands.index('startbattle') < commands.index('reloadmapafterbattle') < commands.index('setevent')
    battle = next(b for b in room['scripted_trainer_battles'] if b['source_script'] == CHIEF)
    assert battle['request']['trainer_id'] == chief['battle']
    assert battle['startbattle_command_index'] == commands.index('startbattle')
    assert any(c['source_script'] == CHIEF and c['command'] == 'reloadmapafterbattle' for c in room['script_map_commands'])
    receiver = room['scripts'][RECEIVER]
    assert [c['args'][0] for c in receiver if c['command'] == 'checkevent'] == [RESTORED, DEFEATED]
    assert any(c['command'] == 'yesorno' for c in receiver), 'Separate real receiver confirmation required'
    assert town['story_contract']['Q07']['free_access'] is True and town['story_contract']['Q07']['paid_state_required'] is False
    for module in [outside, room]:
        assert not module['script_economy_commands'] and not module['script_shop_commands'] and not module['script_item_grants'], 'Town/Q07 must not depend on fees or invented rewards'
    assert not outside['script_runtime_commands']
    if not connected(pack):
        assert not outside['script_flag_commands']
    service = [c for c in room['script_runtime_commands'] if c['command'] == 'special']
    assert len(service) == 1 and service[0]['source_script'] == MEDIC and service[0]['args'] == ['HealParty']
    assert any(c['source_script'] == MEDIC and c['command'] == 'faceplayer' for c in room['script_object_commands'])
    assert next(a for a in town['interiors'][0]['residents'] if a['id'] == MEDIC)['service'] == 'heal_party'
    for actor_id in town['admission_contract']['physical_gate_actors']+[town['admission_contract']['admission_actor']]:
        assert text_lines(outside, actor_id+'Text') == ['EXPEDITIONS CLOSED', 'Safari permits are', 'not available yet.', 'No fee is charged.']
    return dict(q07_badge_checks=8, q07_upstream_signal_required=True,
                chief_party=[dict(species=p['species'], level=p['level']) for p in trainer['party']],
                chief_win_and_receiver_events_separate=True, free_medic_native_special='HealParty',
                native_admission='CLOSED_NO_FEE', safari_session_or_fee_installed=False)


def verify_connections_and_checkpoints(pack, town):
    module = pack['data']['maps'][MAP]
    contract = town['regional_checkpoints']
    checkpoints = contract['checkpoints']
    assert contract['bare_town_actor_count'] == len(town['residents']) == 6
    assert contract['connected_town_actor_count'] == len(town['residents'])+len(checkpoint_actors(town)) == 12
    assert len(checkpoints) == 3 and len({c['id'] for c in checkpoints}) == 3
    assert contract['free_inward_care'] is True and contract['capture_expeditions'] == 'CLOSED_NO_FEE'
    if not connected(pack):
        assert not module['attributes']['connections'] and not module['events']['coord_events']
        assert not any(c['command'] == 'coord_event' for c in module['map_event_section_commands'])
        assert not module['script_flag_commands']
        return dict(integration_state='BARE', regional_guard_bodies=0, regional_coord_planes=0,
                    native_reciprocal_connections=0, qualified_visibility_proven_here=False)
    expected_connections, expected_coords, expected_flags = [], [], []
    reverse = dict(east='west', west='east', north='south', south='north')
    global_flags = pack['data']['story_event_script_constants']['global']
    actual_flags = module['script_flag_commands']
    for checkpoint in checkpoints:
        root, direction = checkpoint['script_root'], checkpoint['direction']
        target, offset = checkpoint['native_target_map'], checkpoint['offset_metatiles']
        assert checkpoint['reciprocal_offset_metatiles'] == -offset and checkpoint['fee'] == 0
        assert checkpoint['scene'] == 'always_active'
        port = next(p for p in town['ports'] if p['route'] == checkpoint['route'])
        assert port['side'].lower() == direction[0]
        assert port['native_connection'] == dict(target_map=target, direction=direction,
            offset_metatiles=offset, reciprocal_offset_metatiles=-offset, checkpoint=checkpoint['id'])
        expected_connections.append(dict(direction=direction, target_map=target, offset=offset))
        neighbor = pack['data']['maps'][target]
        back = [c for c in neighbor['attributes']['connections'] if c['direction'] == reverse[direction]]
        assert back == [dict(direction=reverse[direction], target_map=MAP, offset=-offset)], target+': reciprocal offset/target changed'
        assert len(checkpoint['guards']) == len(checkpoint['inner_coord_tiles']) == len(checkpoint['outer_inward_coord_tiles']) == 2
        for key, script in [('inner_coord_tiles', root), ('outer_inward_coord_tiles', root+'Inward')]:
            for x, y in checkpoint[key]:
                expected_coords.append(dict(x=x, y=y, scene_id='', script_name=script))
                assert collision(pack, module, (x, y)) == 'FLOOR', 'Coordinate plane must be real native Floor'
        # Match actual native along-edge subtraction in both directions, and
        # require the mapped seam lane to be real walkable Floor on both maps.
        for actor in checkpoint['guards']:
            x, y = actor['tile']
            if direction in {'east', 'west'}:
                camp_edge = (module['attributes']['width']*2-1 if direction == 'east' else 0, y)
                route_edge = (0 if direction == 'east' else neighbor['attributes']['width']*2-1, y-offset*2)
                assert route_edge[1]-(-offset)*2 == y
            else:
                camp_edge = (x, module['attributes']['height']*2-1 if direction == 'south' else 0)
                route_edge = (x-offset*2, 0 if direction == 'south' else neighbor['attributes']['height']*2-1)
                assert route_edge[0]-(-offset)*2 == x
            assert collision(pack, module, camp_edge) == collision(pack, neighbor, route_edge) == 'FLOOR', 'Native reciprocal seam lane blocked'
        native_requirements = [BADGES[int(flag.removeprefix('BADGE_'))-1] if flag.startswith('BADGE_') else flag
                               for flag in checkpoint['requires']]
        assert all(flag in global_flags for flag in native_requirements)
        assert native_requirements[:8] == BADGES and native_requirements[8:]
        assert all(flag in {SIGNAL, RESTORED} for flag in native_requirements[8:])
        for label in [root, root+'Reconcile']:
            raw = module['scripts'][label]
            checks = [(entry['command'], entry['args'][0], index) for index, entry in enumerate(raw)
                      if entry['command'] in {'checkflag', 'checkevent'}]
            wanted = [('checkflag' if flag in BADGES else 'checkevent', flag, index*2)
                      for index, flag in enumerate(native_requirements)]
            assert checks == wanted, label+': native prerequisite commands differ from source'
            expected_flags.extend(dict(command=command, flag_id=flag, source_script=label, command_index=index)
                                  for command, flag, index in wanted)
            for _, flag, index in wanted:
                denied = root+'Show' if label.endswith('Reconcile') else root+('Badges' if flag in BADGES else 'Signal' if flag == SIGNAL else 'Restored')
                assert raw[index+1] == dict(command='iffalse', args=[denied]), 'Native refusal branch changed'
            assert not any(entry['command'] in {'setevent', 'setflag', 'clearevent', 'clearflag'} for entry in raw)
        guards = {actor['id'] for actor in checkpoint['guards']}
        object_commands = module['script_object_commands']
        for label, command in [(root, 'disappear'), (root+'Inward', 'disappear'), (root+'Show', 'appear')]:
            rows = [c for c in object_commands if c['source_script'] == label and c['command'] == command]
            assert len(rows) == 2 and {c['object_id'] for c in rows} == guards
        movement = checkpoint['native_inward_movement']
        assert movement['command'] == 'step' and movement['args'] == [{'east':'LEFT','west':'RIGHT','south':'UP','north':'DOWN'}[direction]]
        for suffix, count in [('OneStep', checkpoint['outward_refusal_steps']), ('TwoSteps', checkpoint['inbound_free_care_steps'])]:
            label = root+suffix
            assert module['scripts'][label] == [movement]*count+[dict(command='step_end', args=[])]
            parsed = [entry for entry in module['script_movements'] if entry['label'] == label]
            assert len(parsed) == (3 if suffix == 'OneStep' else 1)
            assert {entry['source_script'] for entry in parsed} == ({root+'Badges', root+'Signal', root+'Restored'} if suffix == 'OneStep' else {root+'Inward'})
            expected_steps = [dict(command='step', direction=movement['args'][0], duration=None, index=i) for i in range(count)]
            expected_steps.append(dict(command='step_end', direction=None, duration=None, index=count))
            assert all(entry['steps'] == expected_steps for entry in parsed), 'Native movement parser differs from actual source step macro'
        incoming = module['scripts'][root+'Inward']
        assert incoming[0] == dict(command='readvar', args=['VAR_FACING'])
        outward_facing = {'east': 'RIGHT', 'west': 'LEFT', 'south': 'DOWN', 'north': 'UP'}[direction]
        assert checkpoint['native_outward_facing'] == outward_facing, 'Source checkpoint skip direction must face outward'
        assert checkpoint['outer_passage_policy'], 'Source must state the outward-only skip policy'
        assert incoming[1] == dict(command='ifequal', args=[outward_facing, root+'End']), 'Only actual outward facing may skip the free inward/transverse escort'
        branch = [c for c in module['script_control_commands'] if c['source_script'] == root+'Inward' and c['command_index'] == 1]
        assert branch == [dict(command='ifequal', compare_value=outward_facing,
                               target_label=root+'End', resolved_target_script=root+'End',
                               source_script=root+'Inward', command_index=1)], 'Compiled native outward-only branch differs from source policy'
        assert any(c['source_script'] == root+'Inward' and c['command'] == 'readvar' and c['target'] == 'VAR_FACING'
                   for c in module['script_variable_commands'])
    order = lambda rows: sorted(rows, key=lambda row: json.dumps(row, sort_keys=True))
    assert order(module['attributes']['connections']) == order(expected_connections), 'Unexpected camp connection target/direction/offset'
    assert order(module['events']['coord_events']) == order(expected_coords) and len(expected_coords) == 12, 'Exact twelve native checkpoint planes required'
    raw_coords = [c for c in module['map_event_section_commands'] if c['command'] == 'coord_event']
    assert len(raw_coords) == 12 and all(len(c['args']) == 4 and c['args'][2] == '-1' for c in raw_coords), 'Raw coord_event scene token must be native -1; empty is valid only in the typed always-active scene'
    assert sorted(c['args'] for c in raw_coords) == sorted([str(c['x']), str(c['y']), '-1', c['script_name']] for c in expected_coords), 'Raw native coordinate section differs from typed source planes'
    assert order(actual_flags) == order(expected_flags), 'Compiled native checkpoint flag table differs from source requirements'
    return dict(integration_state='CONNECTED_UNQUALIFIED_INITIAL_BODIES', regional_guard_bodies=6,
                regional_coord_planes=12, native_reciprocal_connections=6,
                checkpoint_prerequisites_match_source=True, native_movement_macros_match_source=True,
                native_wire_coordinate_scene_encoding='-1',
                outer_passage_policy='outward-only skip; inward and transverse arrivals escorted two native steps inward',
                qualified_visibility_proven_here=False)


def review(pack, source, town, layout, directory):
    started = time.perf_counter()
    artifacts = Artifacts(directory)
    actual_art, source_art = SourceArt(pack), SourceArt(source)
    expected_blocks = layout_blocks(layout)
    report = dict(pack_content_hash=pack['identity']['content_hash'], pack_maps=len(pack['data']['maps']),
                  source_pack_content_hash=source['identity']['content_hash'], source_pack_maps=len(source['data']['maps']),
                  source_json_sha256=hashlib.sha256(TOWN.read_bytes()).hexdigest(),
                  source_layout_sha256=hashlib.sha256(LAYOUT.read_bytes()).hexdigest(),
                  integration_state='CONNECTED_UNQUALIFIED_INITIAL_BODIES' if connected(pack) else 'BARE',
                  integration_manifest=CONNECTION_MANIFEST if connected(pack) else None,
                  pack_reader='Python read-only pack decoder; this does not establish successful Rust runtime-file verification',
                  status='Static native-pack/source-art review; production controller, engine captures and earned regional progression are separate',
                  errors=[], checks_passed=[])
    def attempt(name, operation):
        try:
            value = operation()
            if isinstance(value, dict):
                report.update(value)
            report['checks_passed'].append(name)
        except (AssertionError, KeyError, ValueError, TypeError, IndexError) as error:
            report['errors'].append(dict(check=name, error=str(error) or type(error).__name__))
    room_design = town['interiors'][0]
    room_blocks = [int(block, 16) for row in room_design['rows'] for block in row.split()]
    room_views = [dict(id='CHIEF_AND_RECEIVER', tile=[10, 5]), dict(id='FREE_MEDIC_AND_EXIT', tile=[5, 8]), dict(id='PASSIVE_ROOM_ARRIVAL', tile=[8, 9])]
    town_views = town['viewpoints']
    if connected(pack):
        town_views = town_views+[dict(id='UNQUALIFIED_'+c['id'].upper()+'_CHECKPOINT', tile=c['inner_coord_tiles'][0])
                                for c in town['regional_checkpoints']['checkpoints']]
    # Render before validation: mismatched/rejected actual geometry still exists
    # at full native resolution, with its source comparison and all conversions.
    town_suffix = '-connected-unqualified' if connected(pack) else '-bare'
    attempt('render-source-town', lambda: render_set(source, source_art, artifacts, 'source-town'+town_suffix, source_art.render('mn_boundary_safari', 32, 24, expected_blocks), expected_town_actors(pack, town), town_views))
    attempt('render-source-shelter', lambda: render_set(source, source_art, artifacts, 'source-shelter', source_art.render('lab', 8, 6, room_blocks), room_design['residents'], room_views))
    for name, prefix, views in [(MAP, 'native-town'+town_suffix, town_views), (ROOM, 'native-shelter', room_views)]:
        def render_actual(name=name, prefix=prefix, views=views):
            terrain = native_terrain(pack, actual_art, name)
            scene = render_set(pack, actual_art, artifacts, prefix, terrain, pack['data']['maps'][name]['objects'], views)
            if name == ROOM:
                artifacts.save(prefix+'-whole-rgba-4x', scene.resize((scene.width*4, scene.height*4), Image.Resampling.NEAREST))
            return None
        attempt('render-'+prefix, render_actual)
    for name, operation in [
        ('native-identities-and-source-blocks', lambda: verify_maps(pack, source, town, layout, expected_blocks)),
        ('original-source-art-and-whole-pixels', lambda: verify_art(pack, source, actual_art, source_art, town, layout, expected_blocks)),
        ('exact-native-actors-and-signs', lambda: verify_actors(pack, town)),
        ('explicit-integration-and-native-checkpoint-contracts', lambda: verify_connections_and_checkpoints(pack, town)),
        ('native-door-carpets-and-passive-arrivals', lambda: verify_warps(pack, town)),
        ('native-walking-surf-and-free-approaches', lambda: verify_geometry(pack, town)),
        ('actual-q07-and-free-native-medic', lambda: verify_story_and_services(pack, source, town)),
    ]:
        attempt(name, operation)
    report.update(artifacts=artifacts.files, ok=not report['errors'], review_ms=round((time.perf_counter()-started)*1000, 1))
    encoded = (json.dumps(report, indent=2)+'\n').encode()
    digest = hashlib.sha256(encoded).hexdigest()
    name = f'comparison-{report["pack_content_hash"][:12]}-{digest}.json'
    path = directory/name
    if path.exists():
        assert path.read_bytes() == encoded
    else:
        path.write_bytes(encoded)
    report['immutable_report'] = name
    return report


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--pack', type=Path, required=True, help='Completed actual native town pack; never generated by this tool')
    parser.add_argument('--source-pack', type=Path, default=ROOT/'content-packs/minnesota-region-review.crystalpack', help='Completed 469-map source/art baseline')
    parser.add_argument('--output', type=Path, default=OUTPUT)
    args = parser.parse_args()
    report = review(load_pack(args.pack), load_pack(args.source_pack), json.loads(TOWN.read_text()), json.loads(LAYOUT.read_text()), args.output)
    print(json.dumps(report, indent=2))
    if not report['ok']:
        raise SystemExit(1)


if __name__ == '__main__':
    main()
