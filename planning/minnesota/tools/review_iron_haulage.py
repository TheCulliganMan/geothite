#!/usr/bin/env python3
"""Fast whole-floor native-art review of the authored haulage rock galleries.

This read-only preview never writes pack assets. Rust publication and production
input/camera review remain separate requirements.
"""
import argparse
from collections import deque
import hashlib
import json
from pathlib import Path
import time

from PIL import Image

from render_reference import SourceArt, load_pack, npc_palette_pack
from review_passage_native import actors


def render(pack, art, source, output):
    started = time.perf_counter()
    raw = source.read_bytes()
    digest = hashlib.sha256(raw).hexdigest()
    version = output / (digest[:12] + '-' + str(time.time_ns()))
    version.mkdir(parents=True, exist_ok=False)
    (version / 'source.json').write_bytes(raw)
    spec = json.loads(raw)
    width, height = spec['dimensions_metatiles']
    rows = [[int(word, 16) for word in row.split()] for row in spec['rows']]
    assert len(rows) == height and all(len(row) == width for row in rows)
    blocks = [b for row in rows for b in row]
    assert spec['block_overrides'] == [dict(id='0d', source_tileset='cave', source_block='02',
               collision=['FLOOR'] * 4,
               reason='Retained pressure scripts open block13; preserve that numeric floor meaning in the private alias.')]
    drawn = [2 if b == 13 else b for b in blocks]
    image = art.render('cave', width, height, drawn, group='nite', derived=spec['derived_blocks'])
    module = pack['data']['maps'][spec['id']]
    image = actors(npc_palette_pack(pack, 'nite'), image, module)
    image.save(version / 'whole-original.png')
    image.resize((image.width * 2, image.height * 2), Image.Resampling.NEAREST).save(version / 'whole-2x.png')
    collision = dict(pack['data']['tilesets']['cave']['collision'])
    collision['0d'] = ['FLOOR'] * 4
    for recipe in spec['derived_blocks']:
        collision[recipe['id']] = recipe['collision']

    def kind(x, y, opened=False):
        b = blocks[y // 2 * width + x // 2]
        if opened and [x // 2, y // 2] == [9, 8]:
            b = 13
        return collision[f'{b:02x}'][y % 2 * 2 + x % 2]

    bodies = {(o['x'], o['y']) for o in module['objects']}

    def flood(start, opened):
        floor = {(x, y) for y in range(height * 2) for x in range(width * 2)
                 if kind(x, y, opened) == 'FLOOR' and (x, y) not in bodies}
        assert start in floor
        seen, queue = {start}, deque([start])
        while queue:
            x, y = queue.popleft()
            for n in [(x - 1, y), (x + 1, y), (x, y - 1), (x, y + 1)]:
                if n in floor and n not in seen:
                    seen.add(n)
                    queue.append(n)
        return seen, floor

    for opened in [False, True]:
        west, floor = flood((4, 23), opened)
        east, _ = flood((28, 23), opened)
        assert (west == east == floor) if opened else (not west & east and west | east == floor), dict(
            opened=opened, disconnected=sorted(floor - west - east),
            overlapping=sorted(west & east) if not opened else [])
    for warp in module['events']['warps']:
        expected = 'LADDER' if warp['index'] % 2 else 'FLOOR'
        assert kind(warp['x'], warp['y']) == expected, warp
    for obj in module['objects']:
        assert kind(obj['x'], obj['y']) == 'FLOOR', obj['script']
    assert all(kind(x, y) == 'WALL' for y in range(height * 2) for x in range(width * 2)
               if x in [0, width * 2 - 1] or y in [0, height * 2 - 1])
    receipt = dict(status='SOURCE_ONLY', source_sha256=digest,
                   whole=str(version / 'whole-2x.png'),
                   warm_render_seconds=round(time.perf_counter() - started, 3),
                   closed_west_floor=len(flood((4, 23), False)[0]),
                   closed_east_floor=len(flood((28, 23), False)[0]),
                   open_floor=len(flood((4, 23), True)[0]))
    (version / 'review.json').write_text(json.dumps(receipt, indent=2) + '\n')
    (output / 'latest.json').write_text(json.dumps(receipt, indent=2) + '\n')
    print(json.dumps(receipt), flush=True)
    return digest


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for key in ['pack', 'source', 'output']:
        parser.add_argument('--' + key, type=Path, required=True)
    parser.add_argument('--watch', action='store_true')
    args = parser.parse_args()
    pack = load_pack(args.pack)
    art = SourceArt(pack)
    previous = None
    while True:
        digest = hashlib.sha256(args.source.read_bytes()).hexdigest()
        if digest != previous:
            try:
                previous = render(pack, art, args.source, args.output)
            except (AssertionError, KeyError, ValueError) as error:
                previous = digest
                receipt = dict(status='SOURCE_ERROR', source_sha256=digest, error=str(error))
                version = max(args.output.glob(digest[:12] + '-*'), key=lambda p: p.stat().st_mtime_ns)
                (version / 'review.json').write_text(json.dumps(receipt, indent=2) + '\n')
                (args.output / 'latest.json').write_text(json.dumps(receipt, indent=2) + '\n')
                print(json.dumps(receipt), flush=True)
                if not args.watch:
                    raise
        if not args.watch:
            return
        time.sleep(0.2)


if __name__ == '__main__':
    main()
