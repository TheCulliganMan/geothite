#!/usr/bin/env python3
"""Read-only city/R03 source composition and matched native-camera review."""
import argparse
import json
from pathlib import Path

from PIL import Image, ImageChops

from render_archive import archive_existing, save_render
from render_reference import SourceArt, layout_blocks, load_pack
from review_north_shore import oriented_sprite

ROOT = Path(__file__).resolve().parents[3]
PLAN = ROOT / 'planning/minnesota'


def review(path, output):
    pack = load_pack(path)
    art = SourceArt(pack)
    links = json.loads((PLAN / 'playable/river-city-links.json').read_text())
    prairie = json.loads((PLAN / 'playable/prairie-story.json').read_text())
    river_layout = json.loads((PLAN / 'routes/MN_R03.layout.json').read_text())
    river_blocks = layout_blocks(river_layout)
    for closure in links['route_edge_closures']:
        x, y = closure['metatile']
        assert river_blocks[y * 24 + x] == closure['from']
        river_blocks[y * 24 + x] = closure['to']
    scenes = {}
    origins = {'MnMinneapolis': (0, 0), 'MnR03C00R00': (192, 768),
               'MnR03C00R01': (192, 1536), 'MnMankato': (96, 1792)}
    world = Image.new('RGBA', (1024, 2368), '#17232c')
    for name, origin in origins.items():
        module = pack['data']['maps'][name]
        attrs = module['attributes']
        if name in {'MnMankato', 'MnMinneapolis'}:
            source = layout_blocks(json.loads((PLAN / f'towns/{name}.layout.json').read_text()))
            for link in links['links'] + prairie['links']:
                if link['town'] == name:
                    x, y = link['mouth_metatile']
                    assert source[y * attrs['width'] + x] == 5
                    source[y * attrs['width'] + x] = 1
            assert module['blocks'] == source, f'{name}: source and published forest mouths differ'
        else:
            row = 24 if name.endswith('R01') else 0
            expected = river_blocks[row * 24:(row + attrs['height']) * 24]
            assert module['blocks'] == expected, f'{name}: source plus declared bank caps differ'
        terrain = art.render(attrs['tileset_name'], attrs['width'], attrs['height'], module['blocks'])
        save_render(terrain, output / f'{name}-terrain.png')
        scene = terrain.convert('RGBA')
        save_render(scene, output / f'{name}-terrain-rgba.png')
        for actor in module['objects']:
            facing = {'UP': 'north', 'DOWN': 'south', 'LEFT': 'west', 'RIGHT': 'east'}.get(
                actor['spritemovedata'].rsplit('_', 1)[-1], 'south')
            scene.alpha_composite(oriented_sprite(pack, actor['sprite'], facing),
                                  (actor['x'] * 16, actor['y'] * 16))
        save_render(scene, output / f'{name}-whole.png')
        scenes[name] = scene
        world.alpha_composite(scene, origin)
    save_render(world, output / 'whole-connected-city-valley.png')
    for name, label, (x, y) in [('MnMankato', 'mankato-north', (24, 7)),
                               ('MnMinneapolis', 'minneapolis-south', (32, 42))]:
        scene = world.copy()
        ox, oy = origins[name]
        px, py = ox + x * 16, oy + y * 16
        scene.alpha_composite(oriented_sprite(pack, 'SPRITE_CHRIS', 'south'), (px, py))
        save_render(scene, output / f'{label}-source-player-fixture.png')
        crop = scene.crop((px - 256, py - 192, px + 256, py + 192))
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
            print(f'{label}: {count}/{1280*800} differing source/native pixels; inspect retained frames')
    print(f"{pack['identity']['maps']} maps; content {pack['identity']['content_hash']}")


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--pack', type=Path, default=ROOT / 'content-packs/minnesota-northstar-river-story.crystalpack')
    parser.add_argument('--output', type=Path, default=PLAN / 'renders/river-story')
    args = parser.parse_args()
    archive_existing(args.output)
    review(args.pack, args.output)
