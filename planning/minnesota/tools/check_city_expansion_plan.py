#!/usr/bin/env python3
"""Validate explicit major-city room/road reservations, without simulating play."""
import json


OPPOSITE = {'north': 'south', 'south': 'north', 'east': 'west', 'west': 'east'}


def read(path):
    return json.loads(path.read_text())


def validate(root, town, authored, registry, connections, pack=None):
    """Certify every new room and indexed pair against its executable contract.

    Rich dossiers can use the older presentation schema; the strict expansion
    contract owns native warps/programs. Arrival_core is the clear room fixture
    position. Indexed actual arrivals, including auto-step stairs, live in
    paired_bindings. An optional pack adds native block/collision comparisons.
    """
    path = town['expansion_contract_file']
    plan = read(root / path)
    city = town['id']
    assert plan['schema_version'] == 1 and plan['city'] == city
    assert plan['dimensions_metatiles'] == town['dimensions_metatiles'] == [64, 48]
    assert authored['expansion_contract_file'] == path
    rooms = {room['id']: room for room in plan['rooms']}
    assert len(rooms) == len(plan['rooms']), city + ': duplicate expansion room'
    constants = {room['native_constant']: name for name, room in rooms.items()}
    constants[authored['native_constant']] = city
    assert len(constants) == len(rooms) + 1
    names = {name: constant for constant, name in constants.items()}
    roster = {room['id']: room for room in town['interiors']}
    rich = {room['id']: room for room in authored['interiors']}
    assert set(rooms) <= set(roster) & set(rich), city + ': missing room reservation'
    doors = {door['target']: door for door in authored['doors']}
    assert len(doors) == len(authored['doors'])
    source_warps = {city: {}}
    incoming = {name: [] for name in rooms}
    for door in plan['doors']:
        assert door['target'] in rooms and door['target'] in doors
        assert all(doors[door['target']].get(key) == value for key, value in door.items()), door['target'] + ': changed native door field'
        active = dict(index=door['warp_index'], tile=door['tile'], target=names[door['target']], target_warp=door['target_warp'], passive=False)
        passive = dict(index=door['return_warp_index'], tile=door['return_tile'], target=names[city], target_warp=door['return_warp_index'], passive=True)
        for warp in [active, passive]:
            assert warp['index'] not in source_warps[city], city + ': duplicate exterior warp'
            source_warps[city][warp['index']] = warp
    for name, room in rooms.items():
        assert room['dimensions_metatiles'] == roster[name]['dimensions_metatiles'] == rich[name]['dimensions_metatiles']
        assert room['rows'] == rich[name]['rows'] and room['tileset'] == rich[name]['tileset'], name + ': stale rich room terrain'
        width, height = room['dimensions_metatiles']
        assert len(room['rows']) == height and all(len(row.split()) == width for row in room['rows'])
        for row in room['rows']:
            assert all(0 <= int(value, 16) <= 255 for value in row.split())
        source_warps[name] = {warp['index']: warp for warp in room['warps']}
        assert len(source_warps[name]) == len(room['warps'])
        assert set(source_warps[name]) == set(range(1, len(room['warps']) + 1)), name + ': noncontiguous warp indices'
        for warp in room['warps']:
            assert warp['target'] in constants
            assert len(warp['tile']) == 2 and all(type(value) is int for value in warp['tile'])
            assert 0 <= warp['tile'][0] < width * 2 and 0 <= warp['tile'][1] < height * 2
    for parent, warps in source_warps.items():
        for warp in warps.values():
            if warp['passive']:
                continue
            target = constants[warp['target']]
            assert warp['target_warp'] in source_warps[target], parent + ': missing indexed target'
            if target in incoming:
                incoming[target].append((parent, warp))
    for name, room in rooms.items():
        reservation = registry[name]
        assert reservation['town'] == city and reservation['expansion_contract_file'] == path
        assert reservation['source_file'] == roster[name]['authoring_file'] == town['authoring_file']
        assert reservation['source_room_id'] == name
        assert reservation['dimensions_metatiles'] == room['dimensions_metatiles']
        assert reservation['arrival_core'] == room['arrival_core'], name + ': stale clear room arrival'
        pairs = reservation['paired_bindings']
        expected = {(parent, warp['index']): warp for parent, warp in incoming[name]}
        assert expected and len(pairs) == len(expected), name + ': incomplete indexed entry roster'
        seen = set()
        for pair in pairs:
            key = pair['parent'], pair['parent_entry_warp_index']
            assert key not in seen and key in expected, name + ': duplicate/unknown indexed entry'
            seen.add(key); source = expected[key]
            arrival = source_warps[name][source['target_warp']]
            assert pair['parent_entry_core'] == source['tile']
            assert pair['arrival_core'] == arrival['tile'] and pair['arrival_warp_index'] == arrival['index']
            assert (pair['arrival_collision'] == 'FLOOR' if arrival['passive']
                    else pair['arrival_collision'] in {'STAIRCASE', 'LADDER'})
            returns = [warp for warp in room['warps'] if not warp['passive'] and warp['target'] == names[pair['parent']]]
            assert returns, name + ': no reciprocal active return'
            assert pair['return_trigger_indices'] == [warp['index'] for warp in returns]
            assert pair['return_triggers_core'] == [warp['tile'] for warp in returns]
            assert pair['parent_safe_returns_core'] == [source_warps[pair['parent']][warp['target_warp']]['tile'] for warp in returns]
            assert pair['pair_kind'] == ('exterior_facade' if pair['parent'] == city else 'internal_stairs')
        primary = pairs[0]
        assert reservation['parent'] == primary['parent']
        assert reservation['arrival_warp_index'] == primary['arrival_warp_index']
        assert reservation['return_triggers_core'] == primary['return_triggers_core']
        assert reservation['return_trigger_core'] == primary['return_triggers_core'][0]
    identities = set()
    for seam in plan['connections']:
        found = [link for link in connections if {link['a'], link['b']} == {city, seam['target']}]
        assert len(found) == 1, city + ': missing unique city road'
        link = found[0]; side = 'a' if link['a'] == city else 'b'; other = 'b' if side == 'a' else 'a'
        assert link['kind'] == 'connection' and link['expansion_contract_file'] == path
        assert link['direction_' + side] == seam['direction']
        assert link['direction_' + other] == OPPOSITE[seam['direction']]
        assert link['offset_' + side] == seam['offset'] and link['offset_' + other] == -seam['offset']
        assert link[side + '_edge_core'] == seam['town_edge'] and link[other + '_edge_core'] == seam['route_edge']
        identities.add((link['a'], link['b']))
    if pack is not None:
        for name, room in rooms.items():
            module = pack['data']['maps'][name]
            attr = module['attributes']
            assert [attr['width'], attr['height']] == room['dimensions_metatiles']
            assert attr['tileset_name'] == room['tileset'] and attr['map_constant'] == room['native_constant']
            assert module['blocks'] == [int(value, 16) for row in room['rows'] for value in row.split()], name + ': native room terrain differs'
            assert len(module['events']['warps']) == len(room['warps']), name + ': native warp roster differs'
            collision = pack['data']['tilesets'][room['tileset']]['collision']
            def kind(at):
                x, y = at
                block = module['blocks'][y // 2 * attr['width'] + x // 2]
                return collision[f'{block:02x}'][y % 2 * 2 + x % 2]
            assert kind(room['arrival_core']) == 'FLOOR', name + ': clear room arrival is blocked'
            for warp in room['warps']:
                native = next(w for w in module['events']['warps'] if w['index'] == warp['index'])
                assert [native['x'], native['y']] == warp['tile'] and native['target_map_constant'] == warp['target'] and native['target_warp_id'] == warp['target_warp']
                token = kind(warp['tile'])
                assert (token == 'FLOOR' if warp['passive'] else token in {
                    'STAIRCASE', 'LADDER', 'WARP_CARPET_DOWN', 'WARP_CARPET_UP',
                    'WARP_CARPET_LEFT', 'WARP_CARPET_RIGHT'}), name + ': native warp collision differs'
            for pair in registry[name]['paired_bindings']:
                assert kind(pair['arrival_core']) == pair['arrival_collision'], name + ': actual indexed arrival collision differs'
        native_city = pack['data']['maps'][city]
        assert [native_city['attributes']['width'], native_city['attributes']['height']] == plan['dimensions_metatiles']
        from render_reference import layout_blocks
        layout = read(root / town['layout_file'])
        assert native_city['blocks'] == layout_blocks(layout), city + ': native exterior differs'
        assert native_city['attributes']['tileset_name'] == layout['compiled_tileset']
        for seam in plan['connections']:
            assert {'direction': seam['direction'], 'target_map': seam['target'], 'offset': seam['offset']} in native_city['attributes']['connections']
    return set(rooms), identities
