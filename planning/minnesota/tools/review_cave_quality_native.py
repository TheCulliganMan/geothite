#!/usr/bin/env python3
"""Capture every declared cave camera headlessly and compare full native frames.

Uses the production camera clamp, authored border, forced night palettes and one
coherent water animation phase. Captured GPU images are never changed.
"""
import argparse
import json
import os
from pathlib import Path
import re
import shutil
import subprocess

from PIL import Image, ImageChops

from render_archive import retain_render_file, save_render
from render_reference import SourceArt, load_pack, npc_palette_pack, room_sheet
from review_city_native import phases, priority_scene
from review_connected_route_native import camera_origin
from review_north_shore import oriented_sprite
from review_passage_native import actors, digest, write


def compare(pack, art, module, at, actual, output, hidden_actors=()):
    assert set(hidden_actors) <= {o["script"] for o in module["objects"]}, "default-hidden source actor missing"
    module = dict(module)
    module["objects"] = [o for o in module["objects"] if o["script"] not in hidden_actors]
    a = module['attributes']
    assert a['time_of_day'] == 'nite'
    sprites = npc_palette_pack(pack, 'nite')
    tileset = a['tileset_name']
    terrain = art.render(tileset, a['width'], a['height'], module['blocks'], 'nite')
    scene = actors(sprites, terrain, module)
    x, y = at
    scene.alpha_composite(oriented_sprite(sprites, 'SPRITE_CHRIS', 'south'), (x*16, y*16))
    box = (x*16-256, y*16-192, x*16+256, y*16+192)
    priority_scene(pack, module, scene, terrain, box)
    left, top = box[0]//32*32, box[1]//32*32
    right, bottom = (box[2]+31)//32*32, (box[3]+31)//32*32
    padded = Image.new('RGBA', (right-left, bottom-top))
    border = art.blocks(tileset, 'nite')[a['border_block']]
    for yy in range(0, padded.height, 32):
        for xx in range(0, padded.width, 32):
            padded.paste(border, (xx, yy))
    padded.alpha_composite(scene, (-left, -top))
    crop = padded.crop((box[0]-left, box[1]-top, box[2]-left, box[3]-top))
    save_render(crop, output/'source-original512.png')
    # Reuse the native water decoder with the installed night bank selected.
    phase_pack = dict(pack)
    phase_pack['runtime_files'] = dict(pack['runtime_files'])
    raw = bytes(pack['runtime_files']['gfx/tilesets/bg_tiles.pal']).decode()
    parts = re.split(r'^;\s*(morn|day|nite|dark|indoor)\s*$', raw, flags=re.M)
    banks = {parts[i]: parts[i+1] for i in range(1, len(parts), 2)}
    phase_pack['runtime_files']['gfx/tilesets/bg_tiles.pal'] = ('; day\n'+banks['nite']).encode()
    origin, lcd = camera_origin([(module, 0, 0)], at)
    best = actual.width*actual.height
    matched = None
    layout = bytes(pack['runtime_files'][f'data/tilesets/{tileset}_metatiles.bin'])
    water = []
    for i, block in enumerate(module['blocks']):
        for j, glyph in enumerate(layout[block*16:(block+1)*16]):
            gx, gy = i % a['width']*32+j%4*8, i // a['width']*32+j//4*8
            if glyph == 0x14 and gx < box[2] and gx+8 > box[0] and gy < box[3] and gy+8 > box[1]:
                water.append((gx-box[0], gy-box[1]))
    variants = [(None, crop)]
    for frame, variant in phases(phase_pack, module, crop, box, output):
        tile = Image.open(output/f'native-water14-phase{frame}-8.png').convert('RGBA')
        # Native cave water combines four source frames with eight left-scroll
        # positions. The same pair must match every water glyph in the frame.
        for scroll in range(8):
            shifted = variant.copy()
            glyph = ImageChops.offset(tile, -scroll, 0)
            for position in water:
                shifted.paste(glyph, position)
            variants.append(((frame, scroll), shifted))
    for phase, variant in variants:
        scaled = variant.resize((1536, 1152), Image.Resampling.NEAREST)
        lx, ty = 768-origin[0], 576-origin[1]
        reference = scaled.crop((lx, ty, lx+640, ty+576)).convert('RGB')
        delta = ImageChops.difference(actual, reference)
        errors = sum(pixel != (0, 0, 0) for pixel in delta.getdata())
        save_render(reference, output/f'phase-{phase}-source640.png')
        save_render(delta, output/f'phase-{phase}-difference.png')
        if errors < best:
            best, matched = errors, phase
            save_render(reference, output/'closest-source-reference.png')
    return {'pixel_mismatches': best, 'compared_pixels': 640*576,
            'coherent_water_phase': matched, 'derived_player_origin': list(origin),
            'lcd_origin_8px_cells': lcd}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ['pack', 'binary', 'output']:
        parser.add_argument('--'+name, type=Path, required=True)
    parser.add_argument('--directory', type=Path,
                        default=Path('planning/minnesota/caves/johto-quality-v01'))
    parser.add_argument('--resume', action='store_true')
    parser.add_argument('--room', help='Capture one complete room; default remains all nine.')
    args = parser.parse_args()
    output = args.output.resolve()
    output.mkdir(parents=True, exist_ok=args.resume)
    pack_path, binary = args.pack.resolve(), args.binary.resolve()
    pack = load_pack(pack_path)
    art = SourceArt(pack)
    private = Path('.local')/f'cave-quality-capture-{os.getpid()}'
    shutil.copy2(binary, private)
    report = {'pack_sha256': digest(pack_path), 'binary_sha256': digest(private),
              'fixture': 'Local position, south-facing, noon, one original Minnesota Cindore30; forced native night palette.',
              'captures': []}
    old = json.loads((output/'capture-progress.json').read_text()) if args.resume else None
    if old:
        assert all(old[k] == report[k] for k in ['pack_sha256', 'binary_sha256'])
    env = dict(os.environ, CRYSTAL_RENDER_TEST_HEADLESS='1',
               CRYSTAL_RENDER_TEST_CAPTURE_TARGET='image',
               CRYSTAL_RENDER_TEST_WINDOW_SIZE='640x576', CRYSTAL_PREVIEW_MUTE='1',
               CRYSTAL_RENDER_TEST_PARTY_SPECIES='CINDORE',
               CRYSTAL_RENDER_TEST_PARTY_SIZE='1')
    env.pop('CRYSTAL_RENDER_TEST_SECOND_VIEW', None)
    try:
        for source in sorted(args.directory.glob('*.json')):
            if source.name.endswith('.previous.json'):
                continue
            room = json.loads(source.read_text())
            if args.room is not None and room['id'] != args.room:
                continue
            module = pack['data']['maps'][room['id']]
            assert module['blocks'] == [int(v, 16) for row in room['rows'] for v in row.split()]
            sheet = []
            for view in room['review_contract']['viewpoints']:
                label = room['id']+'-'+view['id']
                folder = output/label
                folder.mkdir(exist_ok=args.resume)
                png = folder/'production.png'
                prior = next((v for v in old['captures'] if v['id'] == label), None) if old else None
                if prior:
                    assert digest(png) == prior['production_sha256']
                else:
                    if png.exists():
                        retain_render_file(png)
                    x, y = view['tile']
                    command = [str(private.resolve()), '--pack', str(pack_path), '--map', room['id'],
                               '--x', str(x), '--y', str(y), '--view', '2d', '--hour', '12',
                               '--screenshot', str(png)]
                    with (folder/'capture.log').open('wb') as log:
                        result = subprocess.run(command, env=env, stdout=log,
                                                stderr=subprocess.STDOUT, timeout=120)
                    assert result.returncode == 0, label
                retain_render_file(png)
                actual = Image.open(png).convert('RGB')
                assert actual.size == (640, 576)
                row = {'id': label, 'source_sha256': digest(source),
                       'production_sha256': digest(png),
                       **compare(pack, art, module, view['tile'], actual, folder, room.get('default_hidden_actors', []))}
                report['captures'].append(row)
                write(output/'capture-progress.json', report)
                print(label, 'mismatches', row['pixel_mismatches'], flush=True)
                sheet.append((view['id'], actual))
            save_render(room_sheet(sheet, 3), output/(room['id']+'-production-cameras.png'))
        assert report['captures'], 'Selected room must have native review cameras.'
        report['status'] = 'PASS' if all(v['pixel_mismatches'] == 0 for v in report['captures']) else 'REVIEW_DIFFERENCES'
    finally:
        private.unlink()
        assert digest(pack_path) == report['pack_sha256']
        write(output/'capture-progress.json', report)
    assert report['status'] == 'PASS', 'inspect retained full-frame source/native differences'


if __name__ == '__main__':
    main()
