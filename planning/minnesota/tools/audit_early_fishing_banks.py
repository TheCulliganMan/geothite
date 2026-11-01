#!/usr/bin/env python3
"""Keep the starting area's ordinary fishing banks open during map revisions."""
import argparse
from collections import deque
import hashlib
import json
from pathlib import Path

from render_reference import load_pack

# Native walking coordinates, approached without Surf, Cut or sliding ice.
BANKS = [
    ('MnWinona', (7, 8), (25, 11), (26, 11),
     [(25, y) for y in [8, 9, 10, 11, 18, 19, 20, 21, 22, 23]]),
    ('MnR01C00R00', (25, 34), (19, 29), (19, 30),
     [(x, 29) for x in range(14, 20)]),
]


def audit(pack):
    results = []
    for name, start, bank, water, required in BANKS:
        module = pack['data']['maps'][name]
        attrs = module['attributes']
        collision = pack['data']['tilesets'][attrs['tileset_name']]['collision']
        width, height = attrs['width'], attrs['height']
        terrain = {(x, y): collision[f"{module['blocks'][y//2*width+x//2]:02x}"][y%2*2+x%2]
                   for y in range(height*2) for x in range(width*2)}
        bodies = {(actor['x'], actor['y']) for actor in module['objects']}
        floor = {point for point, kind in terrain.items()
                 if kind in {'FLOOR', 'TALL_GRASS'} and point not in bodies}
        assert start in floor, (name, 'original dry return blocked')
        reached, pending = {start}, deque([start])
        while pending:
            x, y = pending.popleft()
            for dx, dy in [(1, 0), (-1, 0), (0, 1), (0, -1)]:
                point = x+dx, y+dy
                if point in floor and point not in reached:
                    reached.add(point)
                    pending.append(point)
        banks = sorted(point for point in reached if any(
            terrain.get((point[0]+dx, point[1]+dy)) == 'WATER'
            for dx, dy in [(1, 0), (-1, 0), (0, 1), (0, -1)]))
        assert set(required) <= set(banks), (name, 'early casting bank closed')
        assert bank in banks and terrain[water] == 'WATER'
        results.append(dict(map=name, dry_start=start, reachable_bank_count=len(banks),
                            reachable_bank_tiles=banks, required_bank_tiles=required,
                            bank=bank, water_faced=water, actor_bodies_excluded=True))
    return results


def main():
    cli = argparse.ArgumentParser(description=__doc__)
    cli.add_argument('--pack', type=Path, required=True)
    cli.add_argument('--output', type=Path, required=True)
    args = cli.parse_args()
    raw = args.pack.read_bytes()
    banks = audit(load_pack(args.pack))
    assert args.pack.read_bytes() == raw, 'input pack changed during audit'
    args.output.mkdir(parents=True, exist_ok=False)
    receipt = dict(status='PASS', pack=str(args.pack), pack_sha256=hashlib.sha256(raw).hexdigest(),
                   banks=banks, scope='Static dry-bank reachability from actual house/guru returns, '
                   'excluding water, ice, Cut and occupied actor tiles. Native rod gift, casting, '
                   'encounter, Run and Continue are separate controller requirements.')
    (args.output/'receipt.json').write_text(json.dumps(receipt, indent=2)+'\n')
    print(json.dumps({row['map']: row['reachable_bank_count'] for row in banks}))


if __name__ == '__main__':
    main()
