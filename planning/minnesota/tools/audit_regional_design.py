#!/usr/bin/env python3
"""Audit a released Minnesota parent and its regional design successor without playing."""
import argparse
import base64
from collections import Counter, defaultdict, deque
from heapq import heappop, heappush
import hashlib
import json
from pathlib import Path

import cbor2

from render_reference import SourceArt

SOURCE = Path(__file__).resolve().parents[1] / 'revisions/regional-design-v01'
PARENTS = {
    '02e18ba71203d8aa2e10658c6babc53512daeef592ee53fb2900630f91aa57cb':
        '79c1c85ebc25b200c9fe30689299af09bc88182828931576e512736395ebd521',
    'f4dd4819f039631c81372078d8640f7929114d46bdef90f5ae90d42612a789a0':
        '9fadbe2b9ecf745d0cc2852e745b12d6fd2634dc694e38ac0f0986e4e38624e3',
}
MANIFEST = 'northstar-regional-design-v1'
POLICY = 'data/northstar-save-compatibility.json'
FAIR = 'MnStateFairgrounds'
WATER = 'MN_FAIR_SHOW_RING_FRESH_WATER'
FLAG = 'EVENT_' + WATER
FLOOR = {'FLOOR', 'TALL_GRASS'}
DIRS = {'UP': (0, -1), 'RIGHT': (1, 0), 'DOWN': (0, 1), 'LEFT': (-1, 0)}
TREE_ART = {
    '5daba319bbabc346cc6c7eec31f70713061070585466d4266bfb72412a0c0aae',
    '158f1d9525e075a7147096ca8be735829fec287b0cc15269838e49662142d22f',
}


def check(condition, message):
    if not condition:
        raise ValueError(message)


def load(path):
    raw = path.read_bytes()
    check(raw[:12] == b'CRYSTALPACK\0' and int.from_bytes(raw[14:18], 'big') == len(raw) - 22,
          f'{path}: invalid pack framing')
    # The header checksum covers the entire CBOR body, independently of the identity it declares.
    body = memoryview(raw)[22:]
    checksum = 2166136261
    for byte in body:
        checksum = ((checksum ^ byte) * 16777619) & 0xffffffff
    check(checksum == int.from_bytes(raw[18:22], 'big'), f'{path}: invalid frame checksum')
    return hashlib.sha256(raw).hexdigest(), cbor2.loads(body)


def same_except(before, after, allowed, label):
    changed = {k for k in before.keys() | after.keys()
               if k not in before or k not in after or before[k] != after[k]}
    check(changed <= set(allowed), f'{label}: unexpected changes {changed - set(allowed)}')
    return changed


def coords(record):
    return record['x'], record['y']


class Terrain:
    def __init__(self, pack, name):
        self.map = pack['data']['maps'][name]
        attrs = self.map['attributes']
        self.width = attrs['width']
        self.tileset = attrs['tileset_name']
        self.blocks = self.map['blocks']
        collisions = pack['data']['tilesets'][self.tileset]['collision']
        self.tiles = {(x, y): collisions[f'{self.block(x, y):02x}'][2 * (y % 2) + x % 2]
                      for y in range(2 * attrs['height']) for x in range(2 * self.width)}
        self.occupied = {coords(o) for o in self.map['objects']}
        self.warps = {coords(w) for w in self.map['events']['warps']}

    def block(self, x, y):
        return self.blocks[y//2 * self.width + x//2]

    def reach(self, entries, cut=None):
        """Conservative on-foot routes, with forced ice sliding and occupied squares."""
        allowed = {p for p, kind in self.tiles.items() if kind in FLOOR or kind in {'LADDER', 'ICE'}}
        if cut:
            allowed.add(cut)
        allowed -= self.occupied | self.warps
        starts = set(map(tuple, entries))
        check(starts <= allowed, f'{self.map["id"]}: non-walkable seed {starts - allowed}')
        seen = starts.copy()
        pending = deque(starts)
        while pending:
            x, y = pending.popleft()
            for dx, dy in DIRS.values():
                at = (x + dx, y + dy)
                if at not in allowed:
                    continue
                while self.tiles[at] == 'ICE':
                    nxt = (at[0] + dx, at[1] + dy)
                    if nxt not in allowed:
                        break
                    at = nxt
                if at not in seen:
                    seen.add(at)
                    pending.append(at)
        return seen


def check_metadata(parent_sha, old, new):
    same_except(old, new, {'data', 'identity', 'report', 'runtime_files'}, 'pack envelope')
    same_except(old['report'], new['report'], {'manifests'}, 'diagnostics and counts')
    manifests = new['report']['manifests']
    check(manifests.count(MANIFEST) == 1 and [m for m in manifests if m != MANIFEST]
          == old['report']['manifests'], 'manifest lineage')
    a, b = old['identity'], new['identity']
    same_except(a, b, {'runtime_modpack_id', 'content_hash'}, 'identity')
    check(b['runtime_modpack_id'] == '+'.join(manifests) and a['content_hash'] != b['content_hash'],
          'compiled identity lineage')
    prior = json.loads(bytes(old['runtime_files'][POLICY]))
    policy = json.loads(bytes(new['runtime_files'][POLICY]))
    check(policy['schema_version'] == 5 and policy['target_manifest'] == MANIFEST
          and policy['inherited'] == prior, 'inherited save migration or version')
    raw_id = '+'.join(m for m in manifests if m not in {'realtime-clock', 'player-customization'})
    check(policy['target_modpack_ids'] == [raw_id, raw_id + '+realtime-clock+player-customization'],
          'save compatibility targets')
    sources = policy['sources']
    check(len(sources) == 2 and {s['modpack_hash']:s['content_hash'] for s in sources} == PARENTS
          and {'modpack_hash': parent_sha, 'modpack_id': a['runtime_modpack_id'],
               'content_hash': a['content_hash']} in sources, 'released save origin')
    print('Save policy preserves previous migration data and names both released parents.')


def check_art(old, new):
    """Bound native changes to the allowed atlases, including reused original blocks."""
    a, b = old['data']['tilesets'], new['data']['tilesets']
    snow = 'mn_winter_gunflint'
    iron = 'mn_iron_descent_thaw'
    appended = {'cave': {'40'}, 'kanto': {'83', '84'}, iron: {'d7'}}
    same_except(a, b, {*appended, snow}, 'tilesets')
    files = {'data/tilesets/kanto.json', 'data/tilesets/kanto_metatiles.bin',
             'data/tilesets/cave.json', 'data/tilesets/cave_metatiles.bin', POLICY,
             f'data/tilesets/{snow}_metatiles.bin', f'data/tilesets/{snow}_foregrounds.json',
             f'data/tilesets/{snow}_palette_map.json', f'gfx/tilesets/{snow}.png',
             f'gfx/tilesets/{snow}.2bpp', f'data/tilesets/{iron}.json',
             f'data/tilesets/{iron}_metatiles.bin'}
    same_except(old['runtime_files'], new['runtime_files'], files, 'runtime assets')
    for ts, extra in appended.items():
        same_except(a[ts], b[ts], {'collision'}, ts)
        check(same_except(a[ts]['collision'], b[ts]['collision'], extra, ts) == extra
              and not extra & a[ts]['collision'].keys(), f'{ts}: append-only collision')
        path = f'data/tilesets/{ts}_metatiles.bin'
        prev, curr = bytes(old['runtime_files'][path]), bytes(new['runtime_files'][path])
        check(curr.startswith(prev) and len(curr) == len(prev) + 16 * len(extra)
              and b[ts]['collision'] == json.loads(bytes(new['runtime_files'][f'data/tilesets/{ts}.json'])),
              f'{ts}: native tables')
    same_except(a[snow], b[snow], {'palette_map'}, snow)
    original, current = SourceArt(old), SourceArt(new)
    iron_art = current.blocks(iron, 'day')
    lip = iron_art[0xd7]
    for box, source in [((0, 0, 32, 16), 0xa0), ((0, 24, 32, 32), 0x56)]:
        check(lip.crop(box).tobytes() == iron_art[source].crop(box).tobytes(),
              'Route 10 must retain its tan shelf and use the native visible ledge lip')
    for group in ('day', 'nite'):
        prev, curr = original.blocks(snow, group), current.blocks(snow, group)
        check(len(prev) == len(curr) and {i for i in range(len(prev)) if prev[i].tobytes() != curr[i].tobytes()}
              == {0x0f, 0x17}, 'Gunflint artwork leaked or Cut art did not change')
        left, right = (0, 16, 16, 32), (16, 0, 32, 32)
        check(curr[15].crop(right).tobytes() == curr[23].crop(right).tobytes()
              and curr[15].crop(left).tobytes() != curr[23].crop(left).tobytes()
              and curr[23].crop(left).tobytes() == curr[0x3c].crop(left).tobytes(),
              'Cut must clear only the small tree on snow')
    users = {name for name, m in new['data']['maps'].items()
             if m['attributes']['tileset_name'] == snow and set(m['blocks']) & {0x0f, 0x17}}
    check(users == {'MnR12C00R00'}, 'snowy tree changed other maps')
    cave = current.blocks('cave', 'nite')
    stairs = (16, 16, 32, 32)
    check(cave[0x40].crop(stairs).tobytes() == cave[0x12].crop(stairs).tobytes()
          and cave[0x40].crop(left).tobytes() == cave[2].crop(left).tobytes(), 'stair glyph on warp')
    return current


def compare_map(name, a, b, terrain, block_edits, caches, trainers, hours, fair):
    """Project permitted edits onto the parent; compare all other packed records exactly."""
    allowed = {'blocks', 'objects', 'events', 'map_event_section_commands'}
    if name == FAIR:
        allowed |= {'scripts', 'script_field_pickups'}
    same_except(a, b, allowed, name)
    expected = list(a['blocks'])
    for xy, (before, after) in block_edits.items():
        x, y = xy
        idx = y * terrain.width + x
        check(expected[idx] in before, f'{name} {xy}: wrong terrain parent')
        expected[idx] = after
    check(expected == b['blocks'], f'{name}: terrain differs from proposal')

    check(len(a['objects']) == len(b['objects']), f'{name}: actor count')
    for old_actor, actor in zip(a['objects'], b['objects']):
        label = actor['script']
        expect = dict(old_actor)
        if label in trainers:
            facing, radius = trainers[label]['facing'], trainers[label]['range']
            expect.update(spritemovedata='SPRITEMOVEDATA_STANDING_' + facing, radius=radius,
                          sightline_direction_override=facing)
        elif name == FAIR and label in hours:
            expect['hram_x'], expect['hram_y'] = hours[label]
        check(actor == expect, f'{name}: actor {label}')

    same_except(a['events'], b['events'], {'bg_events'}, name + ' warps and coordinate events')
    bg = []
    for old_event in a['events']['bg_events']:
        event = dict(old_event)
        if event['script'] in caches:
            change = caches[event['script']]
            check(coords(event) == tuple(change['from']) and event['event_type'] == 'BGEVENT_ITEM',
                  f'{name}: source cache')
            event['x'], event['y'] = change['to']
        bg.append(event)
    if name == FAIR:
        fountain = fair['fountain']
        check(fountain == {'script':WATER, 'item':'FRESH_WATER', 'tile':[42, 48], 'approach':[42, 49]},
              'State Fair single optional new item')
        bg.append(dict(x=42, y=48, event_type='BGEVENT_ITEM', script=WATER))
        script = [dict(command='hiddenitem', args=['FRESH_WATER', FLAG])]
        check(b['scripts'] == {**a['scripts'], WATER:script}, 'State Fair programs')
        pickup = dict(command='hiddenitem', item_id='FRESH_WATER', quantity=1, event_flag=FLAG,
                      fruit_tree_id=None, source_script=WATER, command_index=0)
        check(b['script_field_pickups'] == a['script_field_pickups'] + [pickup], 'State Fair receipts')
    check(b['events']['bg_events'] == bg, name + ' background events')

    rows = []
    for row in a['map_event_section_commands']:
        op, args = row['command'], list(row['args'])
        if op == 'object_event':
            label = args[11]
            if label in trainers:
                args[3] = 'SPRITEMOVEDATA_STANDING_' + trainers[label]['facing']
                args[10] = str(trainers[label]['range'])
            elif name == FAIR and label in hours:
                args[6:8] = map(str, hours[label])
        elif op == 'bg_event' and args[3] in caches:
            args[:2] = map(str, caches[args[3]]['to'])
        elif op == 'def_object_events' and name == FAIR:
            rows.append(dict(command='bg_event', args=['42', '48', 'BGEVENT_ITEM', WATER]))
        rows.append(dict(command=op, args=args))
    check(b['map_event_section_commands'] == [dict(r, command_index=i) for i, r in enumerate(rows)],
          name + ' native event records')
    return rows


def lanes(grid, plan, data):
    starts = plan['entries']
    for x, y in starts:
        edges = grid.map['attributes']['connections']
        candidates = []
        for edge in edges:
            neighbor = data['maps'][edge['target_map']]
            target = neighbor['attributes']
            delta = 2 * edge['offset']
            last_y = 2 * grid.map['attributes']['height'] - 1
            pairs = {
                'north': (y == 0, (x - delta, 2 * target['height'] - 1)),
                'south': (y == last_y, (x - delta, 0)),
                'west': (x == 0, (2 * target['width'] - 1, y - delta)),
                'east': (x == 2 * grid.width - 1, (0, y - delta)),
            }
            on_edge, (u, v) = pairs[edge['direction']]
            if on_edge and 0 <= u < 2 * target['width'] and 0 <= v < 2 * target['height']:
                block = neighbor['blocks'][v//2 * target['width'] + u//2]
                quadrants = data['tilesets'][target['tileset_name']]['collision'][f'{block:02x}']
                candidates.append(quadrants[2 * (v%2) + u%2] in FLOOR
                                  and (u, v) not in {coords(o) for o in neighbor['objects']})
        check(any(candidates), f'{grid.map["id"]}: no connected walkable approach at {(x,y)}')
    opened = grid.reach(starts)
    gate = plan.get('cut')
    if gate:
        x, y = tile = tuple(gate['tile'])
        rule = data['field_moves']['cut']['replacements'][grid.tileset][grid.block(x, y)]
        replacing = data['tilesets'][grid.tileset]['collision'][f'{rule["replacement_block_id"]:02x}']
        check(grid.tiles[tile] == 'CUT_TREE' and rule['variant'] == 'tree'
              and replacing[2 * (y%2) + x%2] in FLOOR
              and any((x + dx, y + dy) in opened for dx, dy in DIRS.values()), 'native Cut access')
        after_cut = grid.reach(starts, tile)
    actors = {o['script']:o for o in grid.map['objects'] if o['object_type'] == 'OBJECTTYPE_TRAINER'}
    check(actors.keys() == plan['trainers'].keys(), grid.map['id'] + ' trainer coverage')
    conflict = set(map(tuple, starts))
    for rows in grid.map['events'].values():
        conflict.update(map(coords, rows))
    for label, actor in actors.items():
        dx, dy = DIRS[actor['sightline_direction_override']]
        ray = {(actor['x'] + n * dx, actor['y'] + n * dy) for n in range(1, actor['radius'] + 1)}
        available = opened
        if gate and gate['trainer'] == label:
            check(not ray & opened, label + ' bypasses Cut')
            available = after_cut
        check(ray and ray <= available and not ray & conflict and all(grid.tiles[p] in FLOOR for p in ray),
              label + ' unreachable or overlapping sightline')
        conflict.update(ray)
        trainer = grid.map['trainer_scripts'][label]
        party = data['trainers']['trainers'][trainer['trainer_id']]
        check(party['trainer_class'] == trainer['trainer_class'] and party['party'] and trainer['event_flag'],
              label + ' battle or receipt')
    return Counter(o['radius'] for o in actors.values())


def caches_on_map(grid, changes, data, art, trees):
    checked = 0
    for label, change in changes.items():
        at, stand = tuple(change['to']), tuple(change['approach'])
        check(abs(at[0] - stand[0]) + abs(at[1] - stand[1]) == 1
              and at not in grid.occupied | grid.warps and stand in grid.reach([change['entry']])
              and grid.tiles[stand] in FLOOR, label + ' item-facing square')
        events = [e for e in grid.map['events']['bg_events'] if e['script'] == label]
        receipt = [r for r in grid.map['script_field_pickups'] if r['source_script'] == label]
        check(len(events) == 1 and coords(events[0]) == at and len(receipt) == 1
              and receipt[0]['event_flag'] in data['story_event_script_constants']['global']
              and receipt[0]['command'] == 'hiddenitem' and grid.map['scripts'][label] ==
              [dict(command='hiddenitem', args=[receipt[0]['item_id'], receipt[0]['event_flag']])],
              label + ' native receipt')
        if label in trees:
            x, y = at
            rect = (16 * (x%2), 16 * (y%2), 16 * (x%2 + 1), 16 * (y%2 + 1))
            pixels = art.blocks(grid.tileset, 'day')[grid.block(x, y)].crop(rect).tobytes()
            check(hashlib.sha256(pixels).hexdigest() in TREE_ART
                  and grid.tiles[at] == 'HEADBUTT_TREE' and 0x15 in data['field_moves']['headbutt']['target_collisions'],
                  label + ' native tree artwork or Headbutt target')
            encounter = data['field_encounters'].get(grid.map['id'], {}).get('tables', {}).get('headbutt')
            check(bool(encounter) == (label != 'MN_HIDDEN_R06'), label + ' Headbutt encounter coverage')
            checked += 1
    return checked


def fair_access(grid, fair, data, hours):
    hours_count = Counter()
    copies = defaultdict(list)
    for o in grid.map['objects']:
        if o['script'] in hours:
            copies[o['script']].append(o)
            first, last = o['hram_x'], o['hram_y']
            for h in range(first, last + 1):
                hours_count[h] += 1
            check(o['object_type'] == 'OBJECTTYPE_SCRIPT' and grid.map['scripts'][o['script']]
                  == [dict(command='jumptextfaceplayer', args=[o['script'] + 'Text'])],
                  o['script'] + ' scheduled a gameplay actor')
        if o['object_type'] == 'OBJECTTYPE_TRAINER':
            battle = grid.map['trainer_scripts'][o['script']]
            check(data['trainers']['trainers'][battle['trainer_id']]['party']
                  and battle['event_flag'] in data['story_event_script_constants']['global'],
                  o['script'] + ' missing linked Fair battle')
    check(len(copies) == len(hours) and all(len(v) == 1 for v in copies.values()), 'Fair hours coverage')
    check((42, 49) in grid.reach([(95, 64)]) and grid.tiles[42, 48] == 'WALL', 'crowded Fair fountain')
    print(f'Fair: {len(hours)} scheduled conversations, {len(grid.map["trainer_scripts"])} linked battles; '
          f'{max(hours_count.values())} scheduled guests at peak; one reachable Fresh Water.')


def plan():
    def read(name):
        return json.loads((SOURCE/f'{name}.json').read_text())
    blocks, caches = defaultdict(dict), defaultdict(dict)
    for name in ('caches', 'roads', 'caves'):
        for row in read(name):
            key = row['map']
            for e in row.get('blocks', []):
                blocks[key][tuple(e['at'])] = ({e['from']}, e['to'])
            for e in row.get('caches', []):
                caches[key][e['script']] = e
    trees = {e['script'] for row in read('caches') for e in row['caches']}
    # The original carrier return ledge is authored directly in the native patch.
    blocks['MnR10C00R00'].update({(12, 10):({0xa0}, 0xd7), (13, 10):({0xa0}, 0xd7)})
    fair = read('fair')
    for (x, y), (u, v) in zip(fair['avenue'], fair['avenue'][1:]):
        for i in range(min(x, u), max(x, u) + 1):
            for j in range(min(y, v), max(y, v) + 1):
                blocks[FAIR][i, j] = ({1, 2}, 0x71)
    blocks[FAIR][21, 24] = ({0x86}, 0x87)
    hours = {f'MnFairMazeGuest{guest:03}':shift['hours'] for shift in fair['shifts'] for guest in shift['guests']}
    return blocks, caches, trees, read('trainers'), hours, fair


def pine_crossing(pack):
    """Compare the same on-foot northbound passage with and without crossing a trainer ray."""
    north = Terrain(pack, 'MnR06C00R00')
    south = Terrain(pack, 'MnR06C00R01')
    mid = north.map['attributes']['height'] * 2
    bottom = mid + south.map['attributes']['height'] * 2 - 1
    floor, grass, watched = set(), set(), set()
    for grid, offset in [(north, 0), (south, mid)]:
        for (x, y), kind in grid.tiles.items():
            if kind in FLOOR and (x, y) not in grid.warps | grid.occupied:
                floor.add((x, y + offset))
                if kind == 'TALL_GRASS':
                    grass.add((x, y + offset))
        for actor in grid.map['objects']:
            if actor['object_type'] == 'OBJECTTYPE_TRAINER':
                dx, dy = DIRS[actor['sightline_direction_override']]
                for n in range(1, actor['radius'] + 1):
                    watched.add((actor['x'] + dx * n, actor['y'] + offset + dy * n))
    def crossing(avoid):
        queue = [(0, 0, 0, p) for p in sorted(floor) if p[1] == bottom]
        seen = set()
        while queue:
            grass_count, exposed, steps, pos = heappop(queue)
            if pos in seen:
                continue
            seen.add(pos)
            if pos[1] == 0:
                return grass_count, exposed, steps
            for dx, dy in DIRS.values():
                nxt = (pos[0] + dx, pos[1] + dy)
                if nxt in floor and nxt not in seen and (not avoid or nxt not in watched):
                    heappush(queue, (grass_count + (nxt in grass), exposed + (nxt in watched), steps + 1, nxt))
        return None
    for avoid in (False, True):
        result = crossing(avoid)
        print(f'Route 6 northbound (allow watched tiles = {not avoid}): '
              f'{result} (grass steps, watched steps, total steps)' if result else
              f'Route 6 northbound (allow watched tiles = {not avoid}): no ordinary ground crossing.')


def audit(parent, child):
    parent_sha, old = load(parent)
    check(parent_sha in PARENTS, f'{parent}: expected released raw or hosted parent, got {parent_sha}')
    child_sha, new = load(child)
    print(f'Parent SHA-256 {parent_sha}\nChild SHA-256  {child_sha}\nChild content  {new["identity"]["content_hash"]}')
    check_metadata(parent_sha, old, new)
    art = check_art(old, new)
    a, b = old['data'], new['data']
    blocks, caches, trees, roads, hours, fair = plan()
    changed = set(blocks) | set(caches) | set(roads)
    same_except(a, b, {'maps', 'map_scripts', 'map_blocks', 'npcs', 'tilesets', 'story_event_script_constants'}, 'content')
    check(a['maps'].keys() == b['maps'].keys(), 'map set')
    same_except(a['maps'], b['maps'], changed, 'map modules')
    same_except(a['npcs'], b['npcs'], changed, 'NPC index')
    constants = 'story_event_script_constants'
    same_except(a[constants], b[constants], {'global'}, 'map constants')
    prev, now = a[constants]['global'], b[constants]['global']
    check(same_except(prev, now, {FLAG}, 'global constants') == {FLAG} and FLAG not in prev
          and max(v for k, v in prev.items() if k.startswith('EVENT_')) < now[FLAG] <= 65535,
          'fresh, distinct native receipt')
    event_labels, block_labels = {WATER}, set()
    counts = Counter()
    old_ranges, new_ranges = Counter(), Counter()
    for name in sorted(changed):
        before, after = a['maps'][name], b['maps'][name]
        grid = Terrain(new, name)
        design = roads.get(name, {}).get('trainers', {})
        rows = compare_map(name, before, after, grid, blocks[name], caches[name], design, hours, fair)
        attrs = after['attributes']
        event_name, block_name = attrs['map_events_label'], attrs['blocks_label']
        event_labels.add(event_name)
        block_labels.add(block_name)
        check(b['map_scripts'][event_name] == rows and b['npcs'][name] == after['objects']
              and base64.b64decode(b['map_blocks'][block_name], validate=True) == bytes(grid.blocks),
              name + ' split publication')
        if name in roads:
            old_grid = Terrain(old, name)
            for o in before['objects']:
                if o['object_type'] != 'OBJECTTYPE_TRAINER':
                    continue
                dx, dy = DIRS[o['spritemovedata'].split('_')[-1]]
                old_ranges[o['radius']] += 1
                if all(old_grid.tiles.get((o['x'] + dx * i, o['y'] + dy * i), 'WALL') == 'WALL'
                       for i in range(1, o['radius'] + 1)):
                    counts['formerly_wall_facing'] += 1
            new_ranges.update(lanes(grid, roads[name], b))
        if blocks[name]:
            old_grid = Terrain(old, name)
            for at, kind in old_grid.tiles.items():
                current = grid.tiles[at]
                new_lip = (name == 'MnR10C00R00' and at[1] == 20 and 24 <= at[0] <= 27
                           and kind == 'FLOOR' and current == 'HOP_DOWN')
                check(kind == current or kind == 'WALL' or {kind, current} <= FLOOR or new_lip,
                      f'{name} {at}: previous traversal {kind} replaced with {current}')
        counts['trees'] += caches_on_map(grid, caches[name], b, art, trees)
        counts['moved'] += len(caches[name])
        if name == FAIR:
            fair_access(grid, fair, b, hours)
        if name == 'MnR12Cave1F':
            check(grid.tiles[17, 23] == 'WARP_CARPET_DOWN' and (17, 23) in grid.warps
                  and grid.block(17, 23) == 0x40, 'Gunflint native stair')
        if name == 'MnR10C00R00':
            for x in range(24, 28):
                check(grid.block(x, 20) == 0xd7 and [grid.tiles[x, y] for y in range(19, 23)]
                      == ['FLOOR', 'HOP_DOWN', 'WALL', 'FLOOR'] and not
                      ({(x, 19), (x, 20), (x, 22)} & (grid.occupied | grid.warps)),
                      f'Route 10: unusable takeoff, north egress, or landing at x={x}')
    same_except(a['map_scripts'], b['map_scripts'], event_labels, 'split scripts')
    same_except(a['map_blocks'], b['map_blocks'], block_labels, 'split map blocks')
    check(b['map_scripts'][WATER] == b['maps'][FAIR]['scripts'][WATER], 'published Fair cache script')
    check(sum(new_ranges.values()) == 125 and counts['formerly_wall_facing'] == 9
          and counts['trees'] == 5 and counts['moved'] == 7, 'release coverage totals')
    print(f'{len(a["maps"])} maps compared, {len(changed)} authored map modules. All map warps, old '
          'item and defeat receipts, badge scripts, parties, wild and field encounters preserved.')
    print('Terrain metatile changes by map: ' + ', '.join(
        f'{name} {len(changes)}' for name, changes in sorted(blocks.items()) if changes))
    print('Route 10: exactly two carrier metatiles at (12,10) and (13,10) use the native ledge lip; '
          'four takeoffs each have an open landing and northward egress. '
          'Gunflint: native stair quadrant at (17,23); Cut leaves the neighboring pine standing.')
    print(f'125 reachable fixed sightlines, including {sum("cut" in r for r in roads.values())} accessible '
          f'after Cut; {counts["formerly_wall_facing"]} parent rays into walls or outside their own maps corrected. Ranges '
          f'{dict(sorted(old_ranges.items()))} -> {dict(sorted(new_ranges.items()))}.')
    print('Seven relocated hidden items retain their original receipts; five reachable native tree faces, '
          'four with Headbutt encounter tables. Route 6 south has the tree and no encounter table.')
    pine_crossing(new)


if __name__ == '__main__':
    cli = argparse.ArgumentParser(description=__doc__)
    cli.add_argument('parent', type=Path)
    cli.add_argument('child', type=Path)
    args = cli.parse_args()
    try:
        audit(args.parent, args.child)
    except (ValueError, KeyError, IndexError) as exc:
        cli.exit(1, f'Audit failed: {exc}\n')
