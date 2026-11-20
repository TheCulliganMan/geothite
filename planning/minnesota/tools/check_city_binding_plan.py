"""Check city room identities across authored revisions and a supplied native edition."""
import json

from check_city_expansion_plan import OPPOSITE, validate as original_expansion


def read(path):
    return json.loads(path.read_text())


def expansion_snapshot(root):
    return root / 'revisions/major-cities-tight-final-v02/after/planning/minnesota'


def tile(warp):
    return [warp['x'], warp['y']]


# Published clinic and shop fronts whose prospective reservations are one tile off.
REALIGNED_FRONTS = {'MnStCloudPokecenter1F': 1, 'MnStCloudMart': 1, 'MnBemidjiMart': -1}


def archived_facade(root, town, door, reservation):
    archive = expansion_snapshot(root)
    previous = next(r for r in read(archive / 'interiors.json')['contracts'] if r['interior'] == door['target'])
    for key, field in [('exterior_door_core', 'tile'), ('exterior_safe_return_core', 'return_tile')]:
        assert previous[key] == reservation[key], (door['target'], key)
        if town['id'] == 'MnDuluth':
            # current_city independently checks the current front and return against indexed anchors.
            assert door['target'] in {r['id'] for r in read(archive / town['authoring_file'])['interiors']}
        else:
            shift = REALIGNED_FRONTS[door['target']]
            assert door[field] == [previous[key][0] + shift, previous[key][1]], (door['target'], field)


def check_realigned_fronts(source, data):
    native = data['maps'][source['id']]
    warps = {w['index']: w for w in native['events']['warps']}
    for door in source['doors']:
        if door['target'] in REALIGNED_FRONTS:
            entry, safe = [warps[door[key]] for key in ('warp_index', 'return_warp_index')]
            assert entry['target_map_constant'] == data['maps'][door['target']]['attributes']['map_constant']
            assert tile(entry) == door['tile'] and tile(safe) == door['return_tile'], door['target']
            assert collision(data, native, entry) == 'DOOR' and collision(data, native, safe) == 'FLOOR'


def collision(data, module, warp):
    x, y = tile(warp)
    attr = module['attributes']
    assert 0 <= x < 2 * attr['width'] and 0 <= y < 2 * attr['height']
    block = module['blocks'][y // 2 * attr['width'] + x // 2]
    return data['tilesets'][attr['tileset_name']]['collision'][f'{block:02x}'][2 * (y % 2) + x % 2]


def current_city(root, town, source, registry, links, pack=None):
    city = town['id']
    archive = expansion_snapshot(root)
    old_world = read(archive / 'world.json')
    old_town = next(t for t in old_world['towns'] if t['id'] == city)
    old_registry = {r['interior']: r for r in read(archive / 'interiors.json')['contracts']}
    rooms, _ = original_expansion(archive, old_town, read(archive / old_town['authoring_file']),
                                  old_registry, old_world['connections'])
    creation = read(root / town['expansion_contract_file'])
    current = {r['id']: r for r in source['interiors']}
    contract = read(root / town['recomposition_contract_file'])
    assert contract['city'] == creation['city'] == city
    assert contract['dimensions_metatiles'] == source['dimensions_metatiles'] == town['dimensions_metatiles']
    assert creation['dimensions_metatiles'] == old_town['dimensions_metatiles']
    expected_before = town['dimensions_metatiles'] if city == 'MnDuluth' else creation['dimensions_metatiles']
    assert contract['old_dimensions_metatiles'] == expected_before
    assert set(r['id'] for r in creation['rooms']) == rooms
    coordinates = {r['index'] + 1: r['after'] for r in contract['anchor_overrides'] if r['kind'] == 'warp'}
    for door in source['doors']:
        for field, index in [('tile', 'warp_index'), ('return_tile', 'return_warp_index')]:
            x, y = coordinates[door[index]]
            if door['target'] == 'MnDuluthMart':
                x -= 1  # The repaired Mart moved the doorway and return to its left-hand quadrant.
            assert door[field] == [x, y], (city, door['target'], field)
    # The historical expansion owns the room and carpet indices. Later revisions
    # moved exterior facades and allowed the residents to change their lines.
    for room in creation['rooms']:
        name = room['id']
        current_room = current[name]
        for key in ('dimensions_metatiles', 'rows', 'tileset'):
            assert room[key] == current_room[key], (name, key)
        reservation = registry[name]
        old_pairs = old_registry[name]['paired_bindings']
        assert len(reservation['paired_bindings']) == len(old_pairs)
        for pair, original in zip(reservation['paired_bindings'], old_pairs):
            fixed = set(original)
            if pair['parent'] == city:
                fixed -= {'parent_entry_core', 'parent_safe_returns_core'}
            assert all(pair[k] == original[k] for k in fixed), (name, pair['parent'])
            if pair['parent'] == city:
                d = next(d for d in source['doors'] if d['target'] == name)
                assert pair['parent_entry_warp_index'] == d['warp_index']
                assert pair['arrival_warp_index'] == d['target_warp']
                dx = pair['parent_entry_core'][0] - d['tile'][0]
                dy = pair['parent_entry_core'][1] - d['tile'][1]
                assert (dx, dy) in ({(-1, 0), (0, 0)} if city == 'MnStPaul' else {(0, 0)}), name
                assert all(x == d['return_tile'][0]+dx and y == d['return_tile'][1]+dy
                           for x, y in pair['parent_safe_returns_core']), name
    seams = {}
    for link in links:
        if city not in (link['a'], link['b']):
            continue
        side = 'a' if link['a'] == city else 'b'
        other = 'b' if side == 'a' else 'a'
        # The world ledger names the origin of its coordinates. Duluth's road
        # ledger retains the expansion coordinates; its compact plan is newer.
        revision = contract if link.get('recomposition_contract_file') else creation
        road = next(r for r in revision['connections'] if r['target'] == link[other])
        assert link['direction_'+side] == road['direction']
        assert link['direction_'+other] == OPPOSITE[road['direction']]
        assert link['offset_'+side] == road['offset'] == -link['offset_'+other]
        assert link[side+'_edge_core'] == road['town_edge'] and link[other+'_edge_core'] == road['route_edge']
        seams[(link['a'], link['b'])] = road
    assert {r['target'] for r in contract['connections']} == {r['target'] for r in seams.values()}
    if pack is not None:
        data = pack['data']
        native = data['maps']
        city_map = native[city]
        assert [city_map['attributes'][k] for k in ('width', 'height')] == town['dimensions_metatiles']
        for seam in contract['connections']:
            assert {'direction': seam['direction'], 'target_map': seam['target'],
                    'offset': seam['offset']} in city_map['attributes']['connections'], (city, seam['target'])
        for door in source['doors']:
            room = native[door['target']]
            warps = {w['index']: w for w in city_map['events']['warps']}
            entry = warps[door['warp_index']]
            safe = warps[door['return_warp_index']]
            assert entry['target_map_constant'] == room['attributes']['map_constant'], (city, door['id'])
            assert safe['target_map_constant'] == city_map['attributes']['map_constant']
            assert safe['index'] == safe['target_warp_id'] and tile(safe) != tile(entry)
            assert collision(data, city_map, entry) == 'DOOR' and collision(data, city_map, safe) == 'FLOOR', door['id']
            arrivals = {w['index']: w for w in room['events']['warps']}
            assert entry['target_warp_id'] in arrivals
            returns = [w for w in arrivals.values() if w['target_map_constant'] == safe['target_map_constant']]
            assert returns and all(w['target_warp_id'] == safe['index'] for w in returns), (city, door['id'], 'street return')
            contract = registry[door['target']]
            pairs = [pair for pair in contract.get('paired_bindings', []) if pair['parent'] == city]
            if pairs:
                expected = {index for pair in pairs for index in pair['return_trigger_indices']}
            else:
                expected = set(range(1, len(contract['return_triggers_core']) + 1))
            active = {w['index'] for w in returns if collision(data, room, w) != 'FLOOR'}
            assert active == expected, (city, door['id'], 'street carpets')
            assert {w['index'] for w in returns} - active <= {entry['target_warp_id']}, door['id']
        for name in rooms:
            for pair in registry[name]['paired_bindings']:
                parent = native[pair['parent']]
                entry = next(w for w in parent['events']['warps'] if w['index'] == pair['parent_entry_warp_index'])
                module = native[name]
                assert entry['target_map_constant'] == module['attributes']['map_constant']
                assert entry['target_warp_id'] == pair['arrival_warp_index'], name
                warps = {w['index']: w for w in module['events']['warps']}
                assert collision(data, module, warps[entry['target_warp_id']]) == pair['arrival_collision'], name
                from_street = pair['parent'] == city
                assert collision(data, parent, entry) in ({'DOOR'} if from_street else {'LADDER', 'STAIRCASE'}), name
                for index in pair['return_trigger_indices']:
                    warp = warps[index]
                    assert warp['target_map_constant'] == parent['attributes']['map_constant'], (name, index)
                    target = next(w for w in parent['events']['warps'] if w['index'] == warp['target_warp_id'])
                    assert collision(data, module, warp) in ({'WARP_CARPET_DOWN'} if from_street else {'LADDER', 'STAIRCASE'}), name
                    assert collision(data, parent, target) in ({'FLOOR'} if from_street else {'FLOOR', 'LADDER', 'STAIRCASE'}), name
                    if from_street:
                        facade = next(d for d in source['doors'] if d['target'] == name and d['warp_index'] == entry['index'])
                        assert target['index'] == facade['return_warp_index'], (name, index, 'paired street return')
                    else:
                        assert tile(target) == pair['parent_safe_returns_core'][pair['return_trigger_indices'].index(index)], name
    return rooms, set(seams)


def check_secondary_return(root, pack):
    """Reject a valid neighboring landing substituted for only the fourth depot exit."""
    from copy import deepcopy

    world = read(root / 'world.json')
    registry = {r['interior']: r for r in read(root / 'interiors.json')['contracts']}
    town = next(t for t in world['towns'] if t['id'] == 'MnStPaul')
    source = read(root / town['authoring_file'])
    args = (root, town, source, registry, world['connections'])
    current_city(*args, pack)
    data = pack['data']
    maps = data['maps'].copy()
    maps['MnStPaulUnionDepot'] = deepcopy(maps['MnStPaulUnionDepot'])
    examples = {door['target']: door for door in source['doors']}
    depot = examples['MnStPaulUnionDepot']
    clinic = examples['MnStPaulPokecenter1F']
    bad = next(w for w in maps['MnStPaulUnionDepot']['events']['warps'] if w['index'] == 4)
    assert bad['target_warp_id'] == depot['return_warp_index']
    assert depot['return_warp_index'] != clinic['return_warp_index']
    bad['target_warp_id'] = clinic['return_warp_index']
    try:
        current_city(*args, {'data': {**data, 'maps': maps}})
    except AssertionError as error:
        assert error.args == ((town['id'], depot['id'], 'street return'),), error
        print("Rejected a fourth depot carpet pointed at the clinic's valid landing.")
    else:
        raise AssertionError('Misdirected fourth depot carpet was accepted')


if __name__ == '__main__':
    import argparse
    from pathlib import Path
    from render_reference import load_pack

    parser = argparse.ArgumentParser(description=check_secondary_return.__doc__)
    parser.add_argument('pack', type=Path)
    args = parser.parse_args()
    check_secondary_return(Path(__file__).resolve().parents[1], load_pack(args.pack))
