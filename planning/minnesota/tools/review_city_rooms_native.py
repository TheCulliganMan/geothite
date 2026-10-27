#!/usr/bin/env python3
"""Retain headless interior frames and compare installed native source pixels.

Views are an array of {map, core:[x,y], id?:str}. Each point is an explicit
empty-party, south-facing, hour12 placement, not earned regional progression.
The untouched GPU image is authoritative. A source mismatch or unmeasurable
player leaves REVIEW_DIFFERENCES and retained diagnostics, never pixel approval.
Use --strict when a nonmatching comparison must also return a failing exit code.
"""
import argparse
import json
import os
from pathlib import Path
import shutil
import subprocess

from PIL import Image, ImageChops

from render_archive import retain_render_file, save_render
from render_reference import SourceArt, load_pack, npc_palette_pack, room_sheet
from render_stages import render_stage
from review_city_native import priority_scene
from review_north_shore import oriented_sprite
from review_passage_native import actors, digest, write
from review_shelter_depth_native import measured_player

ROOT = Path(__file__).resolve().parents[3]
PAD = 512  # Whole native metatiles, keeping negative border sub-tile phase.


def effective_period(attributes):
    """Mirror visible_effective_map_time_of_day for this explicit hour12 fixture."""
    declared = attributes.get('time_of_day')
    if declared and declared.lower() in {'dark', 'darkness'}:
        return 'dark'
    if attributes['environment'].upper() in {'INDOOR', 'GATE'}:
        return 'indoor'
    period = declared or 'day'
    return {'morning': 'morn', 'night': 'nite'}.get(period.lower(), period.lower())


def bordered_scene(art, module, scene, period):
    """Repeat only the installed active map border; no invented room scenery."""
    attr = module['attributes']
    border = art.blocks(attr['tileset_name'], period)[attr['border_block']]
    padded = Image.new('RGBA', (scene.width + PAD * 2, scene.height + PAD * 2))
    for y in range(0, padded.height, 32):
        for x in range(0, padded.width, 32):
            padded.paste(border, (x, y))
    padded.paste(scene, (PAD, PAD))
    return padded, border


def compare(pack, art, module, actual, at, folder):
    """Build expected pixels independently, measuring only exact CHRIS placement."""
    attr = module['attributes']; x, y = at
    period = effective_period(attr)
    sprite_period = 'day' if period == 'indoor' else period
    sprite_pack = npc_palette_pack(pack, sprite_period)
    ground = art.render(attr['tileset_name'], attr['width'], attr['height'],
                        module['blocks'], group=period)
    whole = actors(sprite_pack, ground, module)
    save_render(ground, folder / 'whole-native-terrain.png')
    save_render(whole, folder / 'whole-native-actors-source.png')
    player = oriented_sprite(sprite_pack, 'SPRITE_CHRIS', 'south')
    save_render(player, folder / 'standing-chris-source-rgba.png')
    row = dict(environment=attr['environment'], tileset=attr['tileset_name'],
               declared_period=attr.get('time_of_day'), effective_period=period,
               npc_palette_bank=sprite_period, border_block=attr['border_block'],
               compared_pixels=640 * 576, pixel_mismatches=None,
               centered_camera_assumption_used=False)
    try:
        origin, opaque = measured_player(actual, player)
    except AssertionError as error:
        row['comparison_error'] = str(error)
        row['pixel_verified'] = False
        return row
    row.update(measured_player_origin=list(origin), exact_opaque_player_pixels=opaque)
    scene = whole.copy(); scene.alpha_composite(player, (x * 16, y * 16))
    save_render(scene, folder / 'whole-player-before-priority.png')
    box = (x * 16 - 256, y * 16 - 192, x * 16 + 256, y * 16 + 192)
    row['source_foreground_pixels_over_actors'] = priority_scene(pack, module, scene, ground, box)
    save_render(scene, folder / 'whole-player-with-native-priority.png')
    padded, border = bordered_scene(art, module, scene, period)
    save_render(border, folder / 'native-border-block.png')
    save_render(padded, folder / 'whole-bordered-native-source.png')
    padded_box = tuple(value + PAD for value in box)
    crop = render_stage(padded, 'crop', folder, 'source512', padded_box)
    scaled = render_stage(crop, 'resize', folder, 'source3x', (1536, 1152), Image.Resampling.NEAREST)
    left, top = 768 - origin[0], 576 - origin[1]
    ref_box = (left, top, left + 640, top + 576)
    reference = render_stage(scaled, 'crop', folder, 'reference640', ref_box).convert('RGB')
    save_render(reference, folder / 'source-reference640.png')
    delta = ImageChops.difference(actual, reference)
    save_render(delta, folder / 'difference.png')
    errors = sum(pixel != (0, 0, 0) for pixel in delta.getdata())
    row.update(pixel_mismatches=errors, pixel_verified=errors == 0,
               source_world_box=list(box), source_padding_pixels=PAD,
               reference_crop_box=list(ref_box), mismatch_bounds=delta.getbbox())
    return row


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ['pack', 'binary', 'views', 'output']:
        parser.add_argument('--' + name, type=Path, required=True)
    parser.add_argument('--resume', action='store_true')
    parser.add_argument('--strict', action='store_true')
    args = parser.parse_args()
    pack_path, binary, out = args.pack.resolve(), args.binary.resolve(), args.output.resolve()
    views = json.loads(args.views.read_text()); assert isinstance(views, list) and views
    labels = [v.get('id', f'{i:02d}-{v["map"]}-{v["core"][0]}-{v["core"][1]}')
              for i, v in enumerate(views)]
    assert len(labels) == len(set(labels)) and all(Path(v).name == v for v in labels)
    out.mkdir(parents=True, exist_ok=args.resume)
    report = dict(pack=str(pack_path), pack_sha256=digest(pack_path), binary=str(binary),
                  binary_sha256=digest(binary), views=str(args.views.resolve()),
                  views_sha256=digest(args.views), tool_sha256=digest(Path(__file__)),
                  fixture='Explicit empty-party position, south-facing, hour12. No progression claim.',
                  captures=[], status='INCOMPLETE', personally_visually_reviewed=False)
    retained = json.loads((out / 'capture-progress.json').read_text()) if args.resume else None
    if retained:
        for key in ['pack_sha256', 'binary_sha256', 'views_sha256']:
            assert retained[key] == report[key], key
    pack = load_pack(pack_path); art = SourceArt(pack)
    private = ROOT / '.local' / f'city-rooms-native-{os.getpid()}'
    assert not private.exists(); shutil.copy2(binary, private)
    assert digest(private) == report['binary_sha256']
    env = dict(os.environ, CRYSTAL_RENDER_TEST_HEADLESS='1',
               CRYSTAL_RENDER_TEST_CAPTURE_TARGET='image', CRYSTAL_RENDER_TEST_WINDOW_SIZE='640x576',
               CRYSTAL_PREVIEW_MUTE='1')
    env.pop('CRYSTAL_RENDER_TEST_SECOND_VIEW', None)
    images = []
    try:
        for label, view in zip(labels, views):
            name = view['map']; x, y = view['core']; module = pack['data']['maps'][name]
            attr = module['attributes']
            assert attr['environment'].upper() in {'INDOOR', 'GATE', 'CAVE', 'DUNGEON'}
            assert 0 <= x < attr['width'] * 2 and 0 <= y < attr['height'] * 2
            block = module['blocks'][y // 2 * attr['width'] + x // 2]
            kind = pack['data']['tilesets'][attr['tileset_name']]['collision'][f'{block:02x}'][y % 2 * 2 + x % 2]
            assert kind == 'FLOOR', (label, kind)
            assert not any((o['x'], o['y']) == (x, y) for o in module['objects']), label
            folder = out / label; folder.mkdir(exist_ok=args.resume); png = folder / 'production.png'
            command = [str(private), '--pack', str(pack_path), '--map', name, '--x', str(x), '--y', str(y),
                       '--view', '2d', '--hour', '12', '--screenshot', str(png)]
            previous = next((v for v in retained['captures'] if v['id'] == label), None) if retained else None
            if previous:
                assert digest(png) == previous['production_sha256']
            else:
                retain_render_file(png)
                with (folder / 'capture.log').open('wb') as log:
                    result = subprocess.run(command, env=env, stdout=log, stderr=subprocess.STDOUT, timeout=120)
                assert result.returncode == 0, (label, result.returncode)
            retain_render_file(png)
            actual = render_stage(Image.open(png), 'convert', folder, 'production', 'RGB')
            assert actual.size == (640, 576)
            raw = sorted((folder / '.render-archive' / png.name).glob('gpu-readback-*.json'))
            assert raw, 'retain actual raw GPU descriptor'
            row = dict(id=label, map=name, core=[x, y], collision=kind, command=command,
                       production_sha256=digest(png), raw_gpu_descriptors=[str(p.relative_to(out)) for p in raw])
            try:
                row.update(compare(pack, art, module, actual, (x, y), folder))
            except (AssertionError, KeyError, ValueError) as error:
                row.update(pixel_verified=False, pixel_mismatches=None, comparison_error=str(error))
            assert digest(png) == row['production_sha256'], 'actual capture must remain untouched'
            report['captures'].append(row); write(out / 'capture-progress.json', report)
            images.append((label, actual))
            print(label, 'pixel_mismatches', row.get('pixel_mismatches'), row.get('comparison_error', ''), flush=True)
        save_render(room_sheet(images, columns=3), out / 'production-decisions.png')
        report['status'] = 'PIXEL_PASS' if all(v['pixel_verified'] for v in report['captures']) else 'REVIEW_DIFFERENCES'
    finally:
        private.unlink()
        report['private_binary_removed'] = not private.exists()
        report['pack_unchanged'] = digest(pack_path) == report['pack_sha256']
        write(out / 'capture-progress.json', report)
        snapshot = out / ('review_city_rooms_native-' + report['tool_sha256'] + '.py.snapshot')
        if not snapshot.exists():
            snapshot.write_bytes(Path(__file__).read_bytes())
    if args.strict:
        assert report['status'] == 'PIXEL_PASS', 'inspect original captures and retained source differences'


if __name__ == '__main__':
    main()
