#!/usr/bin/env python3
"""Check an installed-city recomposition plan; this never simulates gameplay."""
from copy import deepcopy
import hashlib
import json


def read(path):
    return json.loads(path.read_bytes())


def digest(value):
    return hashlib.sha256(json.dumps(value, sort_keys=True, separators=(',', ':'), ensure_ascii=False).encode()).hexdigest()


def _st_paul_owned_cache_digest(data, names):
    """Fingerprint only the accepted city's and 42 rooms' native cache entries."""
    tables = {field:{} for field in ['map_attributes','npcs','map_dimensions','runtime_map_metadata','map_blocks','map_scripts']}
    labels = set()
    for name in names:
        module = data['maps'][name];attributes = module['attributes']
        for field in ['map_attributes','npcs']: tables[field][name] = data[field].get(name)
        for field in ['map_dimensions','runtime_map_metadata','map_blocks']:
            key = attributes['blocks_label' if field == 'map_blocks' else 'map_constant']
            tables[field][key] = data[field].get(key)
        labels.update(module['scripts'])
        labels.update(attributes[field] for field in ['map_scripts_label','map_events_label'])
    tables['map_scripts'] = {label:[label in data['map_scripts'],data['map_scripts'].get(label)] for label in labels}
    return digest(tables)


def _validate_st_paul(root, town, authored, registry, connections, pack):
    """Check the exact published compact St. Paul source and its native mirrors.

    The hashes identify the accepted 2fe6fc2a parent, not future river hooks.
    Rooms and expansion recipes remain unchanged creation history. This is
    source/data verification; controller and GPU evidence remains separate.
    """
    city = 'MnStPaul'
    plan = read(root / town['recomposition_contract_file'])
    assert plan['schema_version'] == 1 and plan['city'] == city == authored['id']
    assert digest(plan) == 'ce369875f541ef125dbb025ee263eb6ac640bd0081fa168d5098987bbfe6af17', 'changed frozen St. Paul anchors/ports'
    assert plan['old_dimensions_metatiles'] == [64, 48]
    assert plan['dimensions_metatiles'] == town['dimensions_metatiles'] == authored['dimensions_metatiles'] == [40, 34]
    assert plan['native_module_sha256'] == 'a844020f56664c5047ad9c03e2d3e9d3dd4e1b07fec34fee38fe30193e4f5d88'
    assert authored['recomposition_contract_file'] == town['recomposition_contract_file']
    assert authored['expansion_contract_file'] == town['expansion_contract_file']
    creation = read(root / town['expansion_contract_file'])
    assert creation['city'] == city and creation['dimensions_metatiles'] == [64, 48]
    assert digest(creation) == '042220881b8cba2209d3635abff0929c1b9d7d0e51d72b40a9f2eaa45edcb1dc', 'changed St. Paul creation programs/rooms'
    provenance = authored['recomposition_provenance']
    assert provenance['native_pack_sha256'] == '2fe6fc2a2c59833886b9ffa2fa04584f1f95841efaf87e865f11230cf4cdf54e'
    assert provenance['base_pack_sha256'] == '125ac44497f9fde7a4ebaae3a8326d2d75f626a86d6e838be12ce28d82283fc7'
    previous_path = root / provenance['previous_dossier_file']
    assert hashlib.sha256(previous_path.read_bytes()).hexdigest() == provenance['previous_dossier_sha256']
    before = read(previous_path)
    assert before['dimensions_metatiles'] == [64, 48]
    assert before['interiors'] == authored['interiors'] and len(authored['interiors']) == 41
    assert before['new_activities'] == authored['new_activities'] and before['expansion_contract'] == authored['expansion_contract']
    assert before['fairgrounds_port'] == authored['fairgrounds_port'], 'unverified fair opening'
    assert before['ports'] == authored['ports'] == town['ports']
    for field in ['residents', 'signs', 'doors', 'new_buildings']:
        left, right = deepcopy(before[field]), deepcopy(authored[field])
        assert len(left) == len(right)
        coordinate_keys = {'residents':['tile'], 'signs':['tile'], 'doors':['tile','return_tile','arrival_tile'], 'new_buildings':['origin_metatile','door_core','passive_core']}[field]
        for old, new in zip(left, right):
            for key in coordinate_keys: new[key] = old[key]
        assert left == right, field + ': non-coordinate source changed'
    anchors = {(a['kind'], a['index']):a for a in plan['anchor_overrides']}
    expected = {(kind, i) for kind,count in [('object',23),('background',11),('coordinate',0),('warp',80)] for i in range(count)}
    assert len(anchors) == len(plan['anchor_overrides']) == 114 and set(anchors) == expected
    for anchor in anchors.values():
        for key,bounds in [('before',[128,96]),('after',[80,68])]:
            assert len(anchor[key]) == 2 and all(type(v) is int and 0 <= v < limit for v,limit in zip(anchor[key],bounds))
    old_objects = before['residents'] + creation['residents'] + creation['trainers'] + creation['items']
    old_signs = before['signs'] + creation['signs']
    assert len(old_objects) == 23 and len(old_signs) == 11
    for kind, rows in [('object',old_objects),('background',old_signs)]:
        for i,row in enumerate(rows): assert anchors[kind,i]['before'] == row['tile']
    for field,kind in [('residents','object'),('signs','background')]:
        for i,row in enumerate(authored[field]): assert anchors[kind,i]['after'] == row['tile']
    old_doors = {d['target']:d for d in before['doors']}
    doors = {d['target']:d for d in authored['doors']}
    assert len(doors) == len(authored['doors']) == 40 and set(doors) == set(old_doors)
    for name,door in doors.items():
        for key,index_key in [('tile','warp_index'),('return_tile','return_warp_index')]:
            anchor = anchors['warp',door[index_key]-1]
            assert anchor['before'] == old_doors[name][key] and anchor['after'] == door[key]
        assert door['arrival_tile'] == door['return_tile']
        reservation = registry[name]
        if 'paired_bindings' in reservation:
            pair = next(p for p in reservation['paired_bindings'] if p['parent'] == city)
            assert pair['parent_entry_core'] == door['tile'] and pair['parent_entry_warp_index'] == door['warp_index']
            assert all(v == door['return_tile'] for v in pair['parent_safe_returns_core'])
        else:
            for field,key in [('exterior_door_core','tile'),('exterior_safe_return_core','return_tile'),('exterior_warp_index','warp_index'),('exterior_return_warp_index','return_warp_index')]: assert reservation[field] == door[key]
    for building in authored['new_buildings']:
        door = doors[building['room']]
        assert building['door_core'] == door['tile'] and building['passive_core'] == door['return_tile']
    depot = authored['depot_binding']
    assert depot['door_core'] == doors['MnStPaulUnionDepot']['tile'] and depot['safe_return_core'] == doors['MnStPaulUnionDepot']['return_tile']
    depot_source = read(root / 'transit/MnStPaulUnionDepot.json')
    assert depot_source['exterior_binding']['door_core'] == depot['door_core'] and depot_source['exterior_binding']['safe_return_core'] == depot['safe_return_core']
    depot_programs = deepcopy(depot_source)
    for key in ['door_core','safe_return_core']: depot_programs['exterior_binding'].pop(key)
    assert digest(depot_programs) == '4840d0d603e9805794d555deaaf224cbb677bf64b2826b20e3fa9213ea1ad807', 'original depot program source changed'
    rooms = {r['id']:r for r in creation['rooms']}
    rich = {r['id']:r for r in authored['interiors']}
    assert len(rooms) == len(creation['rooms']) == 37 and set(rooms) < set(rich)
    native_rooms = set(rich) | {'MnStPaulUnionDepot'}
    assert len(native_rooms) == 42 and native_rooms == {r['id'] for r in town['interiors']}
    assert set(provenance['preserved_room_module_sha256']) == native_rooms
    assert digest(provenance['preserved_room_module_sha256']) == '0dc156b316a3287f3bf40c9c53475943ba5782ff8c5e1255c1e90e5a1eac5888', 'changed frozen native room hashes'
    constants = {r['native_constant']:name for name,r in rooms.items()}
    constants[authored['native_constant']] = city
    names = {name:constant for constant,name in constants.items()}
    source_warps = {city:{}}
    for original in creation['doors']:
        door = doors[original['target']]
        assert all(door[k] == v for k,v in original.items() if k not in ['tile','return_tile','arrival_tile'])
        for index,tile,target,target_warp,passive in [(door['warp_index'],door['tile'],names[door['target']],door['target_warp'],False),(door['return_warp_index'],door['return_tile'],names[city],door['return_warp_index'],True)]:
            assert index not in source_warps[city]
            source_warps[city][index] = dict(index=index,tile=tile,target=target,target_warp=target_warp,passive=passive)
    for name,room in rooms.items():
        assert rich[name] == room, name + ': unchanged expansion source room differs'
        source_warps[name] = {w['index']:w for w in room['warps']}
        assert len(source_warps[name]) == len(room['warps'])
    for name,room in rooms.items():
        reservation = registry[name]
        assert reservation['town'] == city and reservation['expansion_contract_file'] == town['expansion_contract_file']
        assert reservation['source_room_id'] == name and reservation['arrival_core'] == room['arrival_core']
        incoming = {(parent,w['index']):w for parent,warps in source_warps.items() for w in warps.values() if not w['passive'] and constants[w['target']] == name}
        assert len(reservation['paired_bindings']) == len(incoming)
        seen = set()
        for pair in reservation['paired_bindings']:
            key = pair['parent'],pair['parent_entry_warp_index'];assert key in incoming and key not in seen;seen.add(key)
            source = incoming[key];arrival = source_warps[name][source['target_warp']]
            assert pair['parent_entry_core'] == source['tile'] and pair['arrival_core'] == arrival['tile'] and pair['arrival_warp_index'] == arrival['index']
            returns = [w for w in room['warps'] if not w['passive'] and constants[w['target']] == pair['parent']]
            assert pair['return_trigger_indices'] == [w['index'] for w in returns]
            assert pair['return_triggers_core'] == [w['tile'] for w in returns]
            assert pair['parent_safe_returns_core'] == [source_warps[pair['parent']][w['target_warp']]['tile'] for w in returns]
    assert plan['connections'] == [dict(direction='west',target='MnR04C00R00',old_offset=15,offset=15,town_edge=[[0,48],[0,49]],route_edge=[[55,18],[55,19]])]
    links = [v for v in connections if city in {v['a'],v['b']}]
    assert len(links) == 1 and {links[0]['a'],links[0]['b']} == {city,'MnR04C00R00'}, 'unverified extra city seam'
    link = links[0];side = 'a' if link['a'] == city else 'b';other = 'b' if side == 'a' else 'a';seam = plan['connections'][0]
    assert link['recomposition_contract_file'] == town['recomposition_contract_file']
    assert link['direction_'+side] == 'west' and link['direction_'+other] == 'east'
    assert link['offset_'+side] == 15 and link['offset_'+other] == -15
    assert link[side+'_edge_core'] == seam['town_edge'] and link[other+'_edge_core'] == seam['route_edge']
    assert plan['spawn_overrides'] == [{'identifier':271,'before':[6,48],'after':[6,48]}]
    layout = read(root / town['layout_file'])
    assert digest(layout) == 'dcf5b29aa2f41360daab0d083ed255c6127d199628197512290124ee79fc87aa', 'changed native compact layout or unverified river hooks'
    for building in authored['new_buildings']:
        cells = {tuple(layer['position']) for layer in layout['layers'] if layer.get('name','').startswith(building['id']+' exact native facade ')}
        x,y = building['origin_metatile'];w,h = building['dimensions_metatiles']
        assert cells == {(u,v) for u in range(x,x+w) for v in range(y,y+h)}, building['id'] + ': facade placement differs from native layout'
    if pack is not None:
        import base64
        from audit_city_expansions import section_rows
        from render_reference import layout_blocks
        data = pack['data'];module = data['maps'][city]
        assert _st_paul_owned_cache_digest(data,native_rooms | {city}) == '36ca6427f5aae52c41d791b58651c2c52a706e24424ddd4b8fb280385d2f3f7f', 'changed native owned cache/program entry'
        assert digest(module) == provenance['native_city_module_sha256'] == '10c8ddb5dd3005d0f586917c6b3b64160d08e08283b4ce83adcf1de067a6dd2b'
        assert module['blocks'] == layout_blocks(layout)
        assert not module['events']['coord_events'] and len(module['objects']) == 23 and len(module['events']['bg_events']) == 11 and len(module['events']['warps']) == 80
        for name,expected_hash in provenance['preserved_room_module_sha256'].items(): assert digest(data['maps'][name]) == expected_hash, name + ': native room changed'
        for kind,path in [('object',('objects',)),('background',('events','bg_events')),('warp',('events','warps'))]:
            rows = module
            for key in path: rows = rows[key]
            for i,row in enumerate(rows): assert [row['x'],row['y']] == anchors[kind,i]['after']
        for name in native_rooms | {city}:
            m = data['maps'][name];a = m['attributes']
            assert data['map_attributes'][name] == a and data['npcs'][name] == m['objects']
            assert list(base64.b64decode(data['map_blocks'][a['blocks_label']],validate=True)) == m['blocks']
            assert data['map_dimensions'][a['map_constant']] == {'width':a['width'],'height':a['height']}
            assert all(data['runtime_map_metadata'][a['map_constant']][k] == a[k] for k in ['width','height'])
            for attribute,field in [('map_scripts_label','map_script_section_commands'),('map_events_label','map_event_section_commands')]:
                expected_rows = [{k:v for k,v in row.items() if k != 'command_index'} for row in m[field]]
                assert data['map_scripts'][a[attribute]] == expected_rows
                assert [r['command_index'] for r in m[field]] == list(range(len(expected_rows)))
            assert section_rows(m) == [{k:v for k,v in row.items() if k != 'command_index'} for row in m['map_event_section_commands']]
        assert [data['runtime_spawn_points']['271'][k] for k in ['tileX','tileY']] == [6,48]
        assert data['runtime_spawn_points']['272']['mapName'] == 'MnStPaulPokecenter1F'
        assert [data['runtime_spawn_points']['272'][k] for k in ['tileX','tileY']] == [3,5]
    return set(rooms), {(link['a'],link['b'])}


def validate(root, town, authored, registry, connections, pack=None):
    """Validate current exterior coordinates while retaining creation history.

    Return the original expansion room IDs and the three actual road identities,
    matching check_city_expansion_plan's caller contract. Original five rooms are
    still checked by check_plan's ordinary facade path. This is a planning check,
    not proof badge scripts, input, battles, menus or Continue execute.
    """
    if town['id'] == 'MnStPaul':
        return _validate_st_paul(root, town, authored, registry, connections, pack)
    city = town['id']; plan = read(root / town['recomposition_contract_file'])
    assert plan['schema_version'] == 1 and plan['city'] == city == 'MnMinneapolis'
    assert plan['old_dimensions_metatiles'] == [64, 48]
    assert plan['dimensions_metatiles'] == town['dimensions_metatiles'] == authored['dimensions_metatiles'] == [40, 32]
    assert plan['waterfall_block'] == 153 and plan['north_water_checkpoint'] is True
    assert authored['recomposition_contract_file'] == town['recomposition_contract_file']
    assert authored['expansion_contract_file'] == town['expansion_contract_file']
    creation = read(root / town['expansion_contract_file'])
    assert creation['city'] == city and creation['dimensions_metatiles'] == [64, 48]
    provenance = authored['recomposition_provenance']
    previous_path = root / provenance['previous_dossier_file']
    assert hashlib.sha256(previous_path.read_bytes()).hexdigest() == provenance['previous_dossier_sha256']
    before = read(previous_path)
    assert before['interiors'] == authored['interiors'], 'recomposition changed one of44 original source rooms'
    assert len(authored['interiors']) == len(town['interiors']) == 44
    assert before['expansion_contract'] == authored['expansion_contract'], 'creation history changed'
    for field in ['residents', 'signs', 'doors']:
        left, right = deepcopy(before[field]), deepcopy(authored[field])
        assert len(left) == len(right)
        for a, b in zip(left, right):
            for key in (['tile', 'return_tile', 'arrival_tile'] if field == 'doors' else ['tile']):
                b[key] = a[key]
        assert left == right, field + ': non-coordinate content changed'
    left, right = deepcopy(before['expansion_activities']), deepcopy(authored['expansion_activities'])
    right['status'] = left['status']
    assert len(left['pickups']) == len(right['pickups']) == 8
    for a, b in zip(left['pickups'], right['pickups']): b['tile'] = a['tile']
    assert left == right, 'activity programs/prices/flags changed'
    anchors = {(a['kind'], a['index']): a for a in plan['anchor_overrides']}
    expected = {(kind, i) for kind, count in [('object', 28), ('background', 34), ('coordinate', 8), ('warp', 80)] for i in range(count)}
    assert len(anchors) == len(plan['anchor_overrides']) == 150 and set(anchors) == expected
    for anchor in anchors.values():
        for key, bounds in [('before', [128, 96]), ('after', [80, 64])]:
            assert len(anchor[key]) == 2 and all(type(v) is int and 0 <= v < limit for v, limit in zip(anchor[key], bounds))
    old_doors = {d['target']: d for d in before['doors']}; doors = {d['target']: d for d in authored['doors']}
    assert len(doors) == len(authored['doors']) == 40 and set(doors) == set(old_doors)
    for name, door in doors.items():
        old = old_doors[name]
        for key, index_key in [('tile', 'warp_index'), ('return_tile', 'return_warp_index')]:
            anchor = anchors['warp', door[index_key] - 1]
            assert anchor['before'] == old[key] and anchor['after'] == door[key]
        assert door['arrival_tile'] == door['return_tile']
        contract = registry[name]
        if 'paired_bindings' in contract:
            pair = next(p for p in contract['paired_bindings'] if p['parent'] == city)
            assert pair['parent_entry_core'] == door['tile']
            assert pair['parent_entry_warp_index'] == door['warp_index']
            assert all(v == door['return_tile'] for v in pair['parent_safe_returns_core'])
        else:
            for field, key in [('exterior_door_core', 'tile'), ('exterior_safe_return_core', 'return_tile'), ('exterior_warp_index', 'warp_index'), ('exterior_return_warp_index', 'return_warp_index')]: assert contract[field] == door[key]
    rooms = {r['id']: r for r in creation['rooms']}; rich = {r['id']: r for r in authored['interiors']}
    assert len(rooms) == 39 and set(rooms) < set(rich)
    constants = {r['native_constant']: name for name, r in rooms.items()}; constants[authored['native_constant']] = city
    names = {name: constant for constant, name in constants.items()}
    source_warps = {city: {}}
    for original in creation['doors']:
        door = doors[original['target']]
        assert all(door[k] == v for k, v in original.items() if k not in ['tile', 'return_tile', 'arrival_tile'])
        for index, tile, target, target_warp, passive in [(door['warp_index'], door['tile'], names[door['target']], door['target_warp'], False), (door['return_warp_index'], door['return_tile'], names[city], door['return_warp_index'], True)]:
            source_warps[city][index] = dict(index=index, tile=tile, target=target, target_warp=target_warp, passive=passive)
    for name, room in rooms.items():
        assert rich[name]['rows'] == room['rows'] and rich[name]['tileset'] == room['tileset']
        assert rich[name]['dimensions_metatiles'] == room['dimensions_metatiles']
        source_warps[name] = {w['index']: w for w in room['warps']}
        assert len(source_warps[name]) == len(room['warps'])
    for name, room in rooms.items():
        contract = registry[name]
        assert contract['town'] == city and contract['expansion_contract_file'] == town['expansion_contract_file']
        assert contract['source_room_id'] == name and contract['arrival_core'] == room['arrival_core']
        incoming = {(parent, w['index']): w for parent, warps in source_warps.items() for w in warps.values() if not w['passive'] and constants[w['target']] == name}
        assert len(contract['paired_bindings']) == len(incoming)
        seen = set()
        for pair in contract['paired_bindings']:
            key = pair['parent'], pair['parent_entry_warp_index']; assert key in incoming and key not in seen; seen.add(key)
            source = incoming[key]; arrival = source_warps[name][source['target_warp']]
            assert pair['parent_entry_core'] == source['tile'] and pair['arrival_core'] == arrival['tile'] and pair['arrival_warp_index'] == arrival['index']
            returns = [w for w in room['warps'] if not w['passive'] and constants[w['target']] == pair['parent']]
            assert pair['return_trigger_indices'] == [w['index'] for w in returns]
            assert pair['return_triggers_core'] == [w['tile'] for w in returns]
            assert pair['parent_safe_returns_core'] == [source_warps[pair['parent']][w['target_warp']]['tile'] for w in returns]
    opposite = {'north': 'south', 'south': 'north', 'east': 'west', 'west': 'east'}; identities = set()
    for seam in plan['connections']:
        found = [v for v in connections if {v['a'], v['b']} == {city, seam['target']}]; assert len(found) == 1
        link = found[0]; side = 'a' if link['a'] == city else 'b'; other = 'b' if side == 'a' else 'a'
        assert link['recomposition_contract_file'] == town['recomposition_contract_file']
        assert link['direction_' + side] == seam['direction'] and link['direction_' + other] == opposite[seam['direction']]
        assert link['offset_' + side] == seam['offset'] and link['offset_' + other] == -seam['offset']
        assert link[side + '_edge_core'] == seam['town_edge'] and link[other + '_edge_core'] == seam['route_edge']
        identities.add((link['a'], link['b']))
    assert len(plan['water_seams']) == 2 and sum(len(v['changes']) for v in plan['neighbor_terrain']) == 54
    for seam in plan['water_seams']:
        link = next(v for v in connections if {v['a'], v['b']} == {city, seam['target']}); side = 'a' if link['a'] == city else 'b'; other = 'b' if side == 'a' else 'a'
        assert len(seam['town_edge']) == len(seam['route_edge']) == 6
        assert link[side + '_water_edge_core'] == seam['town_edge'] and link[other + '_water_edge_core'] == seam['route_edge']
    assert plan['spawn_overrides'] == [{'identifier': 260, 'before': [64, 64], 'after': [40, 48]}]
    if pack is not None:
        from render_reference import layout_blocks
        module = pack['data']['maps'][city]; attr = module['attributes']; layout = read(root / town['layout_file'])
        assert [attr['width'], attr['height']] == [40, 32] and attr['tileset_name'] == layout['compiled_tileset']
        assert module['blocks'] == layout_blocks(layout)
        assert len(module['events']['coord_events']) == 20 and len(module['objects']) == 28 and len(module['events']['warps']) == 80
        assert {(v['x'], v['y']) for v in module['events']['coord_events'][8:]} == {(x, y) for x in range(32, 38) for y in [1, 4]}
        assert set(provenance['preserved_room_module_sha256']) == set(rich)
        for name, sha in provenance['preserved_room_module_sha256'].items(): assert digest(pack['data']['maps'][name]) == sha, name + ': native room changed'
        spawn = pack['data']['runtime_spawn_points']['260']; assert [spawn['tileX'], spawn['tileY']] == [40, 48]
    return set(rooms), identities
