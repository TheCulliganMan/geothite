"""Strict installed-city adapters for two explicitly certified story epochs."""
import hashlib
import json
import re

from render_reference import load_pack

DA27_SHA = 'da27b381f69f64d6530383216077de0802c5b9dc42b12c316319cbc5daeb6d50'
SHORE_SHA = '03c9d073629c60debc66368f07a4170d0b305026c84fd3d5fdcfd96f95c60a97'
STORY_CITIES = {
    'WINONA': 'MnWinona', 'ROCHESTER': 'MnRochester', 'MANKATO': 'MnMankato',
    'MINNEAPOLIS': 'MnMinneapolis', 'ST PAUL': 'MnStPaul', 'ST CLOUD': 'MnStCloud',
    'BRAINERD': 'MnBrainerd', 'BEMIDJI': 'MnBemidji', 'DULUTH': 'MnDuluth',
    'GRAND RAPIDS': 'MnGrandRapids',
}
EXISTING_TOWNS = {'GRAND MARAIS': 'MnGrandMarais', 'BOUNDARY WATERS': 'MnBoundaryWaters'}
OPPOSITE = {'north': 'south', 'south': 'north', 'east': 'west', 'west': 'east'}
SIDES = {'north': 'N', 'south': 'S', 'east': 'E', 'west': 'W'}


def module_digest(module):
    return hashlib.sha256(json.dumps(module, sort_keys=True, separators=(',', ':')).encode()).hexdigest()


class StoryAtlas:
    """Certify exact baseline cities and resolve installed native geometry/room pairs.

    A room may have an interior parent rather than an exterior facade. Link
    certification proves reciprocal terrain geometry, not checkpoint scripts.
    No pack or source object is mutated.
    """

    def __init__(self, pack, baseline_path, world, plan):
        baseline_sha = hashlib.sha256(baseline_path.read_bytes()).hexdigest()
        assert baseline_sha in {DA27_SHA, SHORE_SHA}, 'Story baseline must be exact DA27/538 or03C9/541'
        self.epoch = 'DA27' if baseline_sha == DA27_SHA else '03C9'
        self.map_count = 538 if baseline_sha == DA27_SHA else 541
        self.cities = dict(STORY_CITIES)
        if baseline_sha == SHORE_SHA:
            self.cities['LEECH LAKE'] = 'MnLeechLake'
        baseline = load_pack(baseline_path)
        self.maps = pack['data']['maps']
        self.pack = pack
        self.plan = plan
        self.constants = {m['attributes']['map_constant']: name for name, m in self.maps.items()}
        self.report = {'baseline_pack_sha256': baseline_sha, 'baseline_epoch': self.epoch, 'city_module_certificate': [], 'route_contract_overlays': [], 'installed_walking_links': [], 'interior_pairs': [], 'scope': 'Actual installed map geometry and exact certified-baseline city modules; checkpoint behavior is not inferred from geometry'}
        assert len(self.maps) == len(baseline['data']['maps']) == self.map_count, 'Pack count differs from exact certified baseline lineage'
        names = set()
        for city in (self.cities | EXISTING_TOWNS).values():
            names.update(n for n in baseline['data']['maps'] if n.startswith(city))
        # Include all eight actual gym programs even if a city registry changes.
        gyms = {n for n in baseline['data']['maps'] if n.startswith('Mn') and n.endswith('Gym')}
        assert len(gyms) == 8
        names.update(gyms)
        assert {n for n in self.maps if any(n.startswith(c) for c in (self.cities | EXISTING_TOWNS).values())} <= names, 'Unexpected city-family module'
        for name in sorted(names):
            expected = baseline['data']['maps'][name]
            assert self.maps.get(name) == expected, name + ': whole city/room module differs from certified ' + self.epoch + ' baseline'
            self.report['city_module_certificate'].append({'map': name, 'canonical_module_sha256': module_digest(expected), 'whole_module_exact': True})
        self.room_sources = {}
        self.city_doors = {}
        self.walking_links = self._links(world)
        for place, city in self.cities.items():
            self._rooms(place, city)

    def target(self, warp):
        target = warp.get('target_map')
        return target if target in self.maps else self.constants.get(warp['target_map_constant'])

    def kind(self, name, tile):
        x, y = tile
        module = self.maps[name]
        a = module['attributes']
        assert 0 <= x < a['width'] * 2 and 0 <= y < a['height'] * 2
        block = module['blocks'][(y // 2) * a['width'] + x // 2]
        return self.pack['data']['tilesets'][a['tileset_name']]['collision'][f'{block:02x}'][(y % 2) * 2 + x % 2]

    def _links(self, world):
        result = {}
        for place, city in (self.cities | EXISTING_TOWNS).items():
            module = self.maps[city]
            for connection in module['attributes']['connections']:
                route = connection['target_map']
                match = re.fullmatch(r'MnR(\d\d)C(\d\d)R(\d\d)', route)
                assert match, city + ': native connection is not a route chunk'
                rid, column, row = match.groups()
                design = next(r for r in world['routes'] if r['id'] == 'MN_R' + rid)
                dossier = json.loads((self.plan / design['file']).read_text())
                endpoint = next(e for e in dossier['endpoints'] if e['place'] == place)
                direction, offset = connection['direction'], connection['offset']
                reverse = OPPOSITE[direction]
                assert endpoint['side'] == SIDES[reverse], city + ': world route endpoint side differs'
                assert {'direction': reverse, 'target_map': city, 'offset': -offset} in self.maps[route]['attributes']['connections'], city + ': missing reciprocal native connection/offset'
                a, b = module['attributes'], self.maps[route]['attributes']
                axis = 0 if direction in {'north', 'south'} else 1
                extent = a['width' if axis == 0 else 'height'] * 2
                neighbor_extent = b['width' if axis == 0 else 'height'] * 2
                lanes = []
                for value in range(extent):
                    other = value - offset * 2
                    if not 0 <= other < neighbor_extent:
                        continue
                    if direction == 'north':
                        town_tile, route_tile = (value, 0), (other, b['height'] * 2 - 1)
                    elif direction == 'south':
                        town_tile, route_tile = (value, a['height'] * 2 - 1), (other, 0)
                    elif direction == 'west':
                        town_tile, route_tile = (0, value), (b['width'] * 2 - 1, other)
                    else:
                        town_tile, route_tile = (a['width'] * 2 - 1, value), (0, other)
                    if self.kind(city, town_tile) == self.kind(route, route_tile) == 'FLOOR':
                        lanes.append((town_tile, route_tile))
                assert len(lanes) == 2 and lanes[1][0][axis] == lanes[0][0][axis] + 1, city + ' / ' + route + ': expected exactly two adjacent legal aligned Floor lanes'
                route_mid = [int(column) * 32 + (lanes[0][1][0] + lanes[1][1][0] + 1) / 4, int(row) * 24 + (lanes[0][1][1] + lanes[1][1][1] + 1) / 4]
                result[(place, rid)] = {'side': SIDES[direction], 'gate_metatile': [lanes[0][0][i] // 2 for i in [0, 1]], 'native_route_gate_metatile': route_mid, 'native_route_side': SIDES[reverse]}
                self.report['installed_walking_links'].append({'town': city, 'route': route, 'world_place': place, 'world_route': dossier['id'], 'direction': direction, 'offset': offset, 'reciprocal_offset': -offset, 'town_edge': [list(p[0]) for p in lanes], 'route_edge': [list(p[1]) for p in lanes], 'both_floor_lanes_certified': True, 'checkpoint_semantics': 'Not inferred from geometry'})
        assert len(result) == len(self.report['installed_walking_links']), 'Duplicate world town/route link'
        return result

    def _rooms(self, place, city):
        queue = [city]
        visited = {city}
        doors = []
        while queue:
            parent = queue.pop(0)
            for warp in self.maps[parent]['events']['warps']:
                target = self.target(warp)
                if target is None or target == city or target == parent:
                    continue
                # Only genuine owned rooms; train platform destinations and
                # unrelated transit links are not new city facades.
                if not target.startswith(city):
                    continue
                if parent == city:
                    doors.append({'target': target, 'tile': [warp['x'], warp['y']], 'warp_index': warp['index']})
                if target in visited:
                    continue
                arrival = next(w for w in self.maps[target]['events']['warps'] if w['index'] == warp['target_warp_id'])
                arrival_kind = self.kind(target, (arrival['x'], arrival['y']))
                if parent == city:
                    assert arrival_kind == 'FLOOR', target + ': exterior entry arrival is not safe Floor'
                else:
                    assert self.kind(parent, (warp['x'], warp['y'])) == arrival_kind == 'STAIRCASE', target + ': internal pair must be the actual two native staircases'
                returns = [w for w in self.maps[target]['events']['warps'] if self.target(w) == parent]
                assert returns, target + ': no actual paired return to ' + parent
                pair = {'town': city, 'place': place, 'map': target, 'parent_map': parent, 'source_tile': [warp['x'], warp['y']], 'source_warp_index': warp['index'], 'arrival_tile': [arrival['x'], arrival['y']], 'arrival_collision': arrival_kind, 'return_warp_indices': [w['index'] for w in returns], 'pair_kind': 'exterior_facade' if parent == city else 'internal_stairs'}
                self.room_sources[target] = pair
                self.report['interior_pairs'].append(pair)
                visited.add(target)
                queue.append(target)
        expected = {n for n in self.maps if n.startswith(city) and n != city}
        assert visited - {city} == expected, city + ': an owned room has no actual exterior/internal warp pair'
        self.city_doors[place] = doors

    def route_overlays(self, design, blocks):
        """Apply only the two published source contracts, then caller checks all cells."""
        result = list(blocks)
        rid, width = design['id'], design['dimensions_metatiles'][0]
        patches = []
        if rid == 'MN_R03':
            contract = json.loads((self.plan / 'playable/river-city-links.json').read_text())
            assert contract['manifest'] == 'minnesota-river-city-links-v2'
            for entry in contract['route_edge_closures']:
                patches.append((entry['metatile'], entry['from'], entry['to'], 'playable/river-city-links.json'))
            assert len(patches) == 6
        elif rid == 'MN_R06':
            contract = json.loads((self.plan / 'playable/lake-country-story.json').read_text())
            assert contract['manifest'] == 'minnesota-lake-country-story-v1'
            for entry in contract['caps']:
                if not entry['map'].startswith(design['map_prefix']):
                    continue
                match = re.fullmatch(r'MnR06C(\d\d)R(\d\d)', entry['map'])
                assert match
                column, row = map(int, match.groups())
                patches.append(([column * 32 + entry['metatile'][0], row * 24 + entry['metatile'][1]], entry['previous'], entry['replacement'], 'playable/lake-country-story.json'))
            assert len(patches) == 6
        elif self.epoch == '03C9' and rid in {'MN_R07', 'MN_R08'}:
            # Exact source contract published by the pinned03C9 builder. These
            # are the ONLY additional overlays admitted by the541 lineage.
            points = [[11, 0], [11, 47]] if rid == 'MN_R07' else [[0, 17], [0, 18], [11, 47], [12, 47], [14, 47], [15, 47]]
            patches.extend((at, 2, 5, 'crates/crystal-assets/src/leech_shore_connections.rs') for at in points)
        for at, before, after, contract in patches:
            index = at[1] * width + at[0]
            assert result[index] in {before, after}, rid + ': source cap preimage differs from declared contract'
            state = 'source_already_current' if result[index] == after else 'explicit_source_overlay'
            result[index] = after
            contract_path = (self.plan.parents[1] if contract.startswith('crates/') else self.plan) / contract
            self.report['route_contract_overlays'].append({'route': rid, 'metatile': at, 'before': before, 'after': after, 'contract': contract, 'contract_sha256': hashlib.sha256(contract_path.read_bytes()).hexdigest(), 'published_baseline_sha256': self.report['baseline_pack_sha256'], 'state': state})
        return result


def save_review_crops(canvas, output, hubs, gallery, row_offsets, row_heights, scale, city_maps):
    """Keep full-resolution atlas crops for every city and all gallery rows."""
    from PIL import Image, ImageDraw
    from render_archive import save_render
    cities = []
    for name in list(EXISTING_TOWNS) + list(city_maps):
        x0, y0, x1, y1 = hubs[name]
        crop = canvas.crop(((x0 - 5) * scale, (y0 - 10) * scale, (x1 + 5) * scale, (y1 + 5) * scale))
        save_render(crop, output.parent / 'review-crops' / (name.lower().replace(' ', '-') + '-native-city-links.png'))
        cities.append((name, crop))
    width = max(image.width for _, image in cities) + 20
    height = max(image.height for _, image in cities) + 35
    for start in range(0, len(cities), 6):
        entries = cities[start:start + 6]
        sheet = Image.new('RGB', (width * 3, height * ((len(entries) + 2) // 3)), '#121f2b')
        draw = ImageDraw.Draw(sheet)
        for index, (name, image) in enumerate(entries):
            x, y = index % 3 * width + 10, index // 3 * height + 26
            draw.text((x, y - 20), name, fill='white')
            sheet.paste(image, (x, y))
        save_render(sheet, output.parent / 'review-crops' / f'city-link-sheet-{start // 6 + 1}.png')
    for start in range(0, len(row_heights), 4):
        end = min(start + 4, len(row_heights)) - 1
        y0 = (gallery[1] + 18 + row_offsets[start]) * scale
        y1 = (gallery[1] + 18 + row_offsets[end] + row_heights[end]) * scale
        crop = canvas.crop((gallery[0] * scale, y0, gallery[2] * scale, y1))
        save_render(crop, output.parent / 'review-crops' / f'gallery-rows-{start + 1:02}-{end + 1:02}-native.png')


def snapshot_source_epoch(plan, output, world, city_maps):
    """Retain the planned-source inputs and reject concurrent source writes."""
    paths = {plan / 'world.json', plan / 'playable/grand-marais.json',
             plan / 'playable/root-beer-cabin.json', plan / 'playable/river-city-links.json',
             plan / 'playable/lake-country-story.json', plan / 'towns/MnDuluth.connections.json',
             plan / 'zones/boundary-waters-safari.layout.json'}
    for route in world['routes']:
        paths.add(plan / route['file'])
        paths.add(plan / 'routes' / (route['id'] + '.layout.json'))
    for city in (city_maps | EXISTING_TOWNS).values():
        for suffix in ['.json', '.layout.json']:
            path = plan / 'towns' / (city + suffix)
            if path.exists():
                paths.add(path)
    result = {}
    for path in sorted(paths):
        raw = path.read_bytes()
        if path.name == 'world.json':
            assert json.loads(raw) == world, 'World source changed during atlas setup'
        destination = output.parent / 'source-snapshots' / path.relative_to(plan)
        destination.parent.mkdir(parents=True, exist_ok=True)
        if destination.exists():
            assert destination.read_bytes() == raw, 'Use a new output epoch for changed sources'
        else:
            destination.write_bytes(raw)
        result[path] = raw
    return result
