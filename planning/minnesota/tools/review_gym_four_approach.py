#!/usr/bin/env python3
"""Read-only whole-map and native-camera review of the fourth gym's approach."""
import argparse
import json
from pathlib import Path

from PIL import Image, ImageChops

from render_archive import archive_existing, save_render
from render_reference import SourceArt, layout_blocks, load_pack
from review_north_shore import oriented_sprite

ROOT = Path(__file__).resolve().parents[3]
PLAN = ROOT / 'planning/minnesota'
ORIGINS = {'MnMinneapolis': (0, 0), 'MnR04C00R00': (1024, 96), 'MnStPaul': (1920, 0)}


def review(path, output):
    pack = load_pack(path)
    art = SourceArt(pack)
    terrain_world = Image.new('RGBA', (2944, 768), '#17232c')
    whole = terrain_world.copy()
    scenes = {}
    for name, origin in ORIGINS.items():
        module = pack['data']['maps'][name]
        attrs = module['attributes']
        source_file = ('routes/MN_R04.layout.json' if name.startswith('MnR04')
                       else f'towns/{name}.layout.json')
        source = layout_blocks(json.loads((PLAN / source_file).read_text()))
        if name == 'MnMinneapolis':
            for x, y in [(16, 23), (31, 12)]:
                assert source[y * 32 + x] == 5
                source[y * 32 + x] = 1
        assert source == module['blocks'], f'{name}: published terrain differs from source contract'
        terrain = art.render(attrs['tileset_name'], attrs['width'], attrs['height'], source)
        save_render(terrain, output / f'{name}-terrain.png')
        scene = terrain.convert('RGBA')
        save_render(scene, output / f'{name}-terrain-rgba.png')
        terrain_world.alpha_composite(scene, origin)
        for actor in module['objects']:
            facing = {'UP': 'north', 'DOWN': 'south', 'LEFT': 'west', 'RIGHT': 'east'}.get(
                actor['spritemovedata'].rsplit('_', 1)[-1], 'south')
            scene.alpha_composite(oriented_sprite(pack, actor['sprite'], facing),
                                  (actor['x'] * 16, actor['y'] * 16))
        save_render(scene, output / f'{name}-whole.png')
        whole.alpha_composite(scene, origin)
        scenes[name] = scene
    save_render(terrain_world, output / 'whole-connected-terrain.png')
    save_render(whole, output / 'whole-connected-cities-crossing.png')
    for name, label, (x, y) in [('MnMinneapolis', 'east-checkpoint', (56, 24)),
                               ('MnR04C00R00', 'st-paul-arrival', (52, 18))]:
        # Classic 2D draws the current map's actors over connected terrain.
        scoped = terrain_world.copy()
        ox, oy = ORIGINS[name]
        scoped.alpha_composite(scenes[name], (ox, oy))
        px, py = ox + x * 16, oy + y * 16
        scoped.alpha_composite(oriented_sprite(pack, 'SPRITE_CHRIS', 'south'), (px, py))
        save_render(scoped, output / f'{label}-source-player-fixture.png')
        crop = scoped.crop((px-256, py-192, px+256, py+192))
        save_render(crop, output / f'{label}-camera-original.png')
        scaled = crop.resize((1536, 1152), Image.Resampling.NEAREST)
        save_render(scaled, output / f'{label}-camera-scaled.png')
        framed = scaled.crop((128, 176, 1408, 976))
        save_render(framed, output / f'{label}-camera-rgba.png')
        reference = framed.convert('RGB')
        save_render(reference, output / f'{label}-camera-reference.png')
        native = output / f'native-{label}.png'
        if native.is_file():
            actual = Image.open(native).convert('RGB')
            save_render(actual, output / f'{label}-native-rgb.png')
            difference = ImageChops.difference(reference, actual)
            save_render(difference, output / f'{label}-native-difference.png')
            count = sum(pixel != (0, 0, 0) for pixel in difference.getdata())
            print(f'{label}: {count}/1024000 differing source/native pixels')
    print(f"{pack['identity']['maps']} maps; content {pack['identity']['content_hash']}")


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--pack', type=Path, default=ROOT / 'content-packs/minnesota-gym-four-approach.crystalpack')
    parser.add_argument('--output', type=Path, default=PLAN / 'renders/gym-four-approach')
    args = parser.parse_args()
    archive_existing(args.output)
    review(args.pack, args.output)
