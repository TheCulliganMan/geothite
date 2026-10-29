#!/usr/bin/env python3
"""Render the nine installed public caves at one native scale, without cropping."""
import argparse
import hashlib
import json
from pathlib import Path

from PIL import Image, ImageDraw

from render_archive import save_render
from render_reference import SourceArt, load_pack, npc_palette_pack
from review_north_shore import oriented_sprite

ROOMS = [
    ('MnR01Cave1F', 'Limestone hollows'),
    ('MnR03Cave1F', 'Bank seep'),
    ('MnR05RockCut', 'River rock cut'),
    ('MnR09Cave1F', 'Public mine'),
    ('MnR10Cave1F', 'Iron gallery'),
    ('MnR12Cave1F', 'Gunflint shelter'),
    ('MnR13Cave1F', 'Spring shelter'),
    ('MnR17Cave1F', 'Talus den'),
    ('MnR20Cave1F', 'Two-mouth seep cleft'),
]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--pack', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    args.output.mkdir(parents=True, exist_ok=False)
    pack = load_pack(args.pack)
    art = SourceArt(pack)
    sprites = npc_palette_pack(pack, 'nite')
    scenes = []
    for name, label in ROOMS:
        module = pack['data']['maps'][name]
        a = module['attributes']
        assert a['tileset_name'] == 'cave' and a['time_of_day'] == 'nite'
        scene = art.render('cave', a['width'], a['height'], module['blocks'], 'nite').convert('RGBA')
        for actor in module['objects']:
            facing = {'UP': 'north', 'RIGHT': 'east', 'LEFT': 'west'}.get(
                actor['spritemovedata'].rsplit('_', 1)[-1], 'south')
            scene.alpha_composite(oriented_sprite(sprites, actor['sprite'], facing),
                                  (actor['x'] * 16, actor['y'] * 16))
        if name == 'MnR09Cave1F' and 'MnNDrillPublicTiles' in module['scripts']:
            label += ' - story shaft closed'
        save_render(scene, args.output / (name + '-whole.png'))
        scenes.append((name, label, a, scene))
    width = max(scene.width for _, _, _, scene in scenes)
    height = max(scene.height for _, _, _, scene in scenes)
    card_width, card_height = width + 32, height + 60
    sheet = Image.new('RGB', (card_width * 3, card_height * 3 + 64), '#17232c')
    draw = ImageDraw.Draw(sheet)
    draw.text((16, 14), 'CURRENT PUBLIC CAVES - SAME NATIVE SCALE - 32px / METATILE', fill='white')
    draw.text((16, 32), 'Actual installed rooms. Separate interiors; this sheet does not imply connections between them.', fill='#c9d5df')
    for i, (name, label, a, scene) in enumerate(scenes):
        left, top = i % 3 * card_width + 16, i // 3 * card_height + 64
        draw.text((left, top), name + ' - ' + label, fill='white')
        draw.text((left, top + 18), f"{a['width']}x{a['height']} metatiles", fill='#c9d5df')
        sheet.paste(scene.convert('RGB'), (left + (width - scene.width) // 2,
                                          top + 42 + (height - scene.height) // 2))
    save_render(sheet, args.output / 'all-nine-public-caves.png')
    receipt = dict(pack=str(args.pack), pack_sha256=hashlib.sha256(args.pack.read_bytes()).hexdigest(),
                   native_scale_pixels_per_metatile=32, maximum_room_pixels=[width, height],
                   all_rooms_uncropped=True, rooms=[name for name, _ in ROOMS],
                   scope='Read-only installed native-art collection, with original actor palettes. '
                         'Traversal and GPU acceptance are separately recorded.')
    (args.output / 'receipt.json').write_text(json.dumps(receipt, indent=2) + '\n')
    (args.output / 'review_cave_collection.py.snapshot').write_bytes(Path(__file__).read_bytes())
    print(json.dumps(receipt, indent=2))


if __name__ == '__main__':
    main()
