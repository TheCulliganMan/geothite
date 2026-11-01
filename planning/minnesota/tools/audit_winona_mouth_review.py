#!/usr/bin/env python3
"""Read-only landing/source, native river seams and active habitat review."""
import argparse
import hashlib
import json
from collections import Counter, deque
from pathlib import Path

from render_reference import layout_blocks, load_pack


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--pack', type=Path, required=True)
    parser.add_argument('--layout', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    raw = args.pack.read_bytes()
    source = args.layout.read_bytes()
    pack = load_pack(args.pack)
    data = pack['data']
    name = 'MnMississippiWinonaMouth'
    module = data['maps'][name]
    attr = module['attributes']
    assert [attr['width'], attr['height']] == [16, 4]
    assert attr['tileset_name'] == 'johto' and attr['border_block'] == 5
    assert layout_blocks(json.loads(source)) == module['blocks']
    assert not module['objects'] and all(not v for v in module['events'].values())

    def kind(map_name, point):
        m = data['maps'][map_name]
        x, y = point
        block = m['blocks'][y // 2 * m['attributes']['width'] + x // 2]
        return data['tilesets'][m['attributes']['tileset_name']]['collision'][f'{block:02x}'][y % 2 * 2 + x % 2]

    terrain = {(x, y): kind(name, (x, y)) for y in range(8) for x in range(32)}
    counts = Counter(terrain.values())
    assert counts == {'FLOOR': 18, 'WATER': 72, 'WALL': 166}
    floor = {p for p, k in terrain.items() if k == 'FLOOR'}
    water = {p for p, k in terrain.items() if k == 'WATER'}
    assert not any(x in [0, 31] or y in [0, 7] for x, y in floor)

    def flood(start, allowed):
        found = {start}
        queue = deque([start])
        while queue:
            x, y = queue.popleft()
            for other in [(x - 1, y), (x + 1, y), (x, y - 1), (x, y + 1)]:
                if other in allowed and other not in found:
                    found.add(other)
                    queue.append(other)
        return found

    assert flood((17, 3), floor) == floor
    assert flood((26, 0), water) == water
    landings = sorted(p for p in floor if any(q in water for q in [(p[0] - 1, p[1]), (p[0] + 1, p[1]), (p[0], p[1] - 1), (p[0], p[1] + 1)]))
    assert (17, 3) in landings and (21, 3) in landings
    perimeter = {p for p in water if p[0] in [0, 31] or p[1] in [0, 7]}
    assert perimeter == {(x, y) for x in range(26, 30) for y in [0, 7]}
    seams = []
    assert len(attr['connections']) == 2
    for direction, target, offset in [('north', 'MnMississippiLowerWoodsC00R02', -5), ('south', 'MnWinona', 0)]:
        edge = next(c for c in attr['connections'] if c['direction'] == direction)
        assert edge['target_map'] == target and edge['offset'] == offset
        other = data['maps'][target]['attributes']
        opposite = 'south' if direction == 'north' else 'north'
        back = next(c for c in other['connections'] if c['direction'] == opposite and c['target_map'] == name)
        assert back['offset'] == -offset
        for x in range(26, 30):
            start = (x, 0 if direction == 'north' else 7)
            arrival = (x - offset * 2, other['height'] * 2 - 1 if direction == 'north' else 0)
            assert kind(name, start) == kind(target, arrival) == 'WATER'
            assert arrival[0] - back['offset'] * 2 == x
            seams.append(dict(side=direction, source=list(start), target_map=target, arrival=list(arrival), reciprocal=True))

    roster = json.loads(Path('planning/minnesota/pokemon/roster.json').read_text())
    curation = json.loads(Path('planning/minnesota/pokemon/roster-curation.json').read_text())
    retired = {r['id'] for r in curation['retired_families']}
    removed = {r['id'] for r in curation['removed_evolutions']}
    active = {s['name'].upper() for family in roster['families'] if family['id'] not in retired for s in family['members'] if s['id'] not in removed}
    wild = data['wild_encounters'][name]
    species = {slot['species'] for bank in ['grass', 'water'] for rows in wild[bank].values() for slot in rows}
    fishing = data['fishing']
    group = fishing['groups'][attr['fishing_group']]
    fish = {slot['species'] for rod in group['rod_tables'].values() for slot in rod['slots'] if slot['species']}
    time_groups = {slot['time_group'] for rod in group['rod_tables'].values() for slot in rod['slots'] if slot['time_group']}
    fish |= {fishing['time_groups'][g][k] for g in time_groups for k in ['day_species', 'night_species']}
    assert species | fish <= active, (species | fish) - active
    assert args.pack.read_bytes() == raw and args.layout.read_bytes() == source
    report = dict(status='PASS', pack=str(args.pack), pack_sha256=hashlib.sha256(raw).hexdigest(), map=name, native_source_exact=True, layout=str(args.layout), layout_sha256=hashlib.sha256(source).hexdigest(), terrain_counts=dict(counts), all_18_dry_bank_tiles_connected=True, no_overland_through_route=True, all_72_water_tiles_connected=True, ordinary_landings=[list(p) for p in landings], water_perimeter_only_eight_reciprocal_lanes=True, seams=seams, native_wild_species=sorted(species), native_fishing_species=sorted(fish), all_species_active=True, actors=0, no_events=True, decision='Keep the tiny dry boat landing and bench; it serves fishing, Surf dismount/relaunch and the river seam rather than an overland encounter route.', pack_unchanged=True, reviewer_sha256=hashlib.sha256(Path(__file__).read_bytes()).hexdigest())
    assert not args.output.exists()
    args.output.write_text(json.dumps(report, indent=2) + '\n')
    print(json.dumps(dict(status='PASS', dry_bank_tiles=len(floor), water_tiles=len(water), reciprocal_lanes=len(seams), species=sorted(species | fish))))


if __name__ == '__main__':
    main()
