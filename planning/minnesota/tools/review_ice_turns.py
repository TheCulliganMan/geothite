#!/usr/bin/env python3
"""Preview whole authored ice rooms and native slide courses, retaining all passes."""
import argparse
import copy
import hashlib
import json
from pathlib import Path
import time

from PIL import Image

from render_reference import SourceArt, load_pack, npc_palette_pack, room_sheet
from review_passage_native import actors
from review_voyageurs_ice import course


def review(pack, art, source, output):
    started = time.perf_counter()
    raw = source.read_bytes()
    digest = hashlib.sha256(raw).hexdigest()
    version = output / (digest[:12] + '-' + str(time.time_ns()))
    version.mkdir(parents=True, exist_ok=False)
    (version / 'source.json').write_bytes(raw)
    spec = json.loads(raw)
    scenes, reports = [], []
    for room in spec['rooms']:
        native = copy.deepcopy(pack['data']['maps'][room['id']])
        width, height = room['dimensions_metatiles']
        rows = [[int(value, 16) for value in row.split()] for row in room['rows']]
        if len(rows) != height or any(len(row) != width for row in rows):
            raise ValueError('complete declared room rows required')
        native['blocks'] = [value for row in rows for value in row]
        image = actors(npc_palette_pack(pack, 'nite'), art.render(
            'ice_path', width, height, native['blocks'], group='nite'), native)
        image.save(version / (room['id'] + '-original.png'))
        image.resize((image.width * 2, image.height * 2), Image.Resampling.NEAREST).save(
            version / (room['id'] + '-2x.png'))
        scenes.append((room['id'], image))
        bodies = {(o['x'], o['y']) for o in native['objects']}
        collision = pack['data']['tilesets']['ice_path']['collision']
        kind, _, states, path = course(room, collision, tuple(room['arrival']), bodies)
        targets = []
        for obj in native['objects']:
            x, y = obj['x'], obj['y']
            approaches = [(x-1, y), (x+1, y), (x, y-1), (x, y+1)]
            reachable = [p for p in approaches if p in states]
            target = min(reachable, key=lambda p: len(path(p))) if reachable else None
            targets.append(dict(id=obj['script'], approach=target,
                                course=path(target) if target is not None else None))
        warp_courses = [dict(tile=w['tile'], course=path(tuple(w['tile']))
                             if tuple(w['tile']) in states else None) for w in room['warps']]
        reports.append(dict(id=room['id'], ice_quadrants=sum(
            kind((x, y)) in ['ICE', 'ICE_2B'] for y in range(height*2) for x in range(width*2)),
            stopping_states=len(states), targets=targets, warps=warp_courses,
            camera_stops={v['id']: tuple(v['tile']) in states for v in room['review_viewpoints']}))
    room_sheet(scenes, columns=2).save(version / 'whole-dungeon-original.png')
    board = Image.open(version / 'whole-dungeon-original.png')
    board.resize((board.width * 2, board.height * 2), Image.Resampling.NEAREST).save(
        version / 'whole-dungeon-2x.png')
    report = dict(status='SOURCE_PREVIEW_ONLY', source_sha256=digest, rooms=reports,
                  whole=str(version / 'whole-dungeon-2x.png'),
                  warm_render_seconds=round(time.perf_counter()-started, 3))
    (version / 'review.json').write_text(json.dumps(report, indent=2)+'\n')
    (output / 'latest.json').write_text(json.dumps(report, indent=2)+'\n')
    print(json.dumps(report), flush=True)
    return digest


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ['pack', 'source', 'output']:
        parser.add_argument('--'+name, type=Path, required=True)
    parser.add_argument('--watch', action='store_true')
    args = parser.parse_args()
    pack = load_pack(args.pack)
    art = SourceArt(pack)
    previous = None
    while True:
        digest = hashlib.sha256(args.source.read_bytes()).hexdigest()
        if digest != previous:
            previous = review(pack, art, args.source, args.output)
        if not args.watch:
            return
        time.sleep(0.2)


if __name__ == '__main__':
    main()
