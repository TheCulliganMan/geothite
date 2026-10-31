#!/usr/bin/env python3
"""Review all nine explicit cave layouts using the supplied pack's native art.

This tool never publishes packs. Rust owns publication and native geometry
validation; production controller tests own gameplay evidence.
"""
import argparse
from collections import deque
import hashlib
import json
from pathlib import Path
import time

from PIL import Image

from render_archive import save_render
from render_reference import SourceArt, load_pack, npc_palette_pack, room_sheet
from review_north_shore import oriented_sprite


def inspect(pack, directory, output, published, room_id=None):
    output.mkdir(parents=True, exist_ok=True)
    art = SourceArt(pack)
    rooms = []
    reports = []
    for source in sorted(directory.glob('*.json')):
        if source.name.endswith('.previous.json'):
            continue
        room = json.loads(source.read_text())
        if room_id is not None and room['id'] != room_id:
            continue
        native = pack['data']['maps'][room['id']]
        width, height = room['dimensions_metatiles']
        blocks = [int(word, 16) for row in room['rows'] for word in row.split()]
        assert len(blocks) == width * height
        assert [native['attributes'][k] for k in ['width', 'height']] == [width, height]
        assert native['attributes']['tileset_name'] == 'cave'
        if published:
            assert blocks == native['blocks'], room['id'] + ': published rows differ'
            assert room['border_block'] == native['attributes']['border_block']
        collision = pack['data']['tilesets']['cave']['collision']

        def kind(at):
            x, y = at
            return collision[f'{blocks[y//2*width+x//2]:02x}'][y%2*2+x%2]

        bodies = {(ob['x'], ob['y']) for ob in native['objects']}
        floor = {(x, y) for y in range(height*2) for x in range(width*2)
                 if kind((x, y)) == 'FLOOR' and (x, y) not in bodies}
        entry = tuple(room['clear_arrival'])
        assert entry in floor, room['id'] + ': blocked arrival'
        seen = {entry}
        todo = deque([entry])
        while todo:
            x, y = todo.popleft()
            for at in [(x-1, y), (x+1, y), (x, y-1), (x, y+1)]:
                if at in floor and at not in seen:
                    seen.add(at)
                    todo.append(at)
        assert seen == floor, (room['id'], 'unreachable ordinary floor', sorted(floor-seen))
        for at in bodies:
            assert kind(at) == 'FLOOR', (room['id'], 'actor terrain', at)
            assert any(abs(at[0]-x)+abs(at[1]-y) == 1 for x, y in seen)
        exits = {(warp['x'], warp['y']) for warp in native['events']['warps']}
        assert exits == {tuple(at) for at in room['exits']}
        ladders = {(x, y) for y in range(height*2) for x in range(width*2)
                   if kind((x, y)) == 'LADDER'}
        assert exits == ladders, room['id'] + ': unpaired ladder'
        water = {(x, y) for y in range(height*2) for x in range(width*2)
                 if kind((x, y)) == 'WATER'}
        assert not any(x in (0, width*2-1) or y in (0, height*2-1) for x, y in water)
        group = native['attributes']['time_of_day'] or 'day'
        image = art.render('cave', width, height, blocks, group).convert('RGBA')
        sprite_pack = npc_palette_pack(pack, group)
        for ob in native['objects']:
            facing = {'SPRITEMOVEDATA_STANDING_RIGHT': 'east',
                      'SPRITEMOVEDATA_STANDING_LEFT': 'west',
                      'SPRITEMOVEDATA_STANDING_UP': 'north'}.get(ob['spritemovedata'], 'south')
            image.alpha_composite(oriented_sprite(sprite_pack, ob['sprite'], facing),
                                  (ob['x']*16, ob['y']*16))
        scene = image.convert('RGB')
        save_render(scene, output/(room['id']+'-whole.png'))
        rooms.append((room['id'], scene))
        # Repeat the actual authored border when the player camera reaches an edge.
        border = art.blocks('cave', group)[room['border_block']]
        padded = Image.new('RGB', (scene.width+192, scene.height+192))
        for y in range(0, padded.height, 32):
            for x in range(0, padded.width, 32):
                padded.paste(border, (x, y))
        padded.paste(scene, (96, 96))
        cameras = []
        for view in room['review_contract']['viewpoints']:
            x, y = view['tile']
            assert (x, y) in floor, (room['id'], 'camera needs clear native floor', view)
            crop = padded.crop((96+x*16-72, 96+y*16-64,
                                96+x*16+88, 96+y*16+80))
            save_render(crop, output/(room['id']+'-'+view['id']+'-160.png'))
            cameras.append((view['id'], crop))
        if cameras:
            save_render(room_sheet(cameras, columns=3), output/(room['id']+'-cameras.png'))
        reports.append({'id': room['id'], 'ordinary_floor': len(floor),
                        'water_tiles': len(water), 'actors': len(bodies),
                        'mouths': len(exits), 'published_rows_match': published})
    assert len(rooms) == (1 if room_id else 9), 'review every selected Minnesota cave'
    save_render(room_sheet(rooms, columns=3), output/('selected-room-whole.png' if room_id else 'all-nine-whole.png'))
    (output/'source-proof.json').write_text(json.dumps(reports, indent=2)+'\n')
    print(json.dumps(reports, indent=2))


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--pack', type=Path, required=True)
    parser.add_argument('--directory', type=Path,
                        default=Path('planning/minnesota/caves/johto-quality-v01'))
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--published', action='store_true')
    parser.add_argument('--room', help='Review one complete room; default remains all nine.')
    parser.add_argument('--watch', action='store_true',
                        help='Keep native art loaded; retain a fresh render epoch per source edit.')
    args = parser.parse_args()
    pack = load_pack(args.pack)
    if not args.watch:
        inspect(pack, args.directory, args.output, args.published, args.room)
    else:
        args.output.mkdir(parents=True, exist_ok=True)
        pack_digest = hashlib.sha256(args.pack.read_bytes()).hexdigest()
        previous = None
        while True:
            files = sorted(p for p in args.directory.glob('*.json')
                           if not p.name.endswith('.previous.json'))
            try:
                source_bytes = {p.name: p.read_bytes() for p in files}
            except FileNotFoundError:
                # Editors can replace a file between the directory read and copy.
                time.sleep(0.25)
                continue
            digest = hashlib.sha256(b''.join(
                name.encode() + b'\0' + raw for name, raw in source_bytes.items())).hexdigest()
            if digest != previous:
                started = time.perf_counter()
                destination = args.output / ('source-' + str(time.time_ns()))
                destination.mkdir()
                snapshot = destination / 'source-inputs'
                snapshot.mkdir()
                for name, raw in source_bytes.items():
                    (snapshot / name).write_bytes(raw)
                result = dict(source_sha256=digest, output=str(destination),
                              source_snapshot=str(snapshot), pack=str(args.pack),
                              loaded_pack_sha256=pack_digest)
                try:
                    inspect(pack, snapshot, destination, args.published, args.room)
                    result['status'] = 'PASS_SOURCE_REVIEW'
                except (AssertionError, KeyError, ValueError, OSError) as error:
                    result.update(status='SOURCE_ERROR', error=str(error))
                result['render_seconds'] = round(time.perf_counter() - started, 3)
                text = json.dumps(result, indent=2) + '\n'
                (destination / 'watch-result.json').write_text(text)
                (args.output / 'latest.json').write_text(text)
                print(json.dumps(result), flush=True)
                previous = digest
            time.sleep(0.25)
