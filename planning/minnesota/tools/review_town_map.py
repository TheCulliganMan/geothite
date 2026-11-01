#!/usr/bin/env python3
"""Render native Town Map tiles from a verified external pack and Rust tilemap.

The runtime integration test renders the production UI separately. This source
review makes every native pixel visible at 4x and preserves versioned outputs.
"""
import argparse
import io
import re
from pathlib import Path

from PIL import Image
from render_reference import load_pack


def render(pack_path, tilemap_path, output):
    pack = load_pack(pack_path)
    files = pack['runtime_files']
    sheet = Image.open(io.BytesIO(bytes(files['gfx/pokegear/town_map.png']))).convert('RGB')
    rgb5 = [tuple(map(int, match)) for match in re.findall(
        r'RGB\s+(\d+),\s*(\d+),\s*(\d+)',
        bytes(files['gfx/pokegear/pokegear.pal']).decode())]
    colors = [tuple((value << 3) | (value >> 2) for value in color) for color in rgb5]
    palettes = [colors[i:i+4] for i in range(0, len(colors), 4)]
    assignments = pack['data']['pokegear_town_map_palette_map']['town_map']
    names = ['BORDER', 'EARTH', 'MOUNTAIN', 'CITY', 'POI', 'POI_MTN']
    tiles = tilemap_path.read_bytes()
    assert len(tiles) == 360 and all(tile < 48 for tile in tiles)
    image = Image.new('RGB', (160, 144))
    for index, tile in enumerate(tiles):
        palette = palettes[names.index(assignments[tile])]
        for y in range(8):
            for x in range(8):
                gray = sheet.getpixel((tile % 16 * 8 + x, tile // 16 * 8 + y))[0]
                image.putpixel((index % 20 * 8 + x, index // 20 * 8 + y),
                               palette[(255-gray)//64])
    assert not output.exists(), 'Keep earlier review renders; choose a new version.'
    output.parent.mkdir(parents=True, exist_ok=True)
    image.resize((640, 576), Image.Resampling.NEAREST).save(output)


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('pack', type=Path)
    parser.add_argument('tilemap', type=Path)
    parser.add_argument('output', type=Path)
    args = parser.parse_args()
    render(args.pack, args.tilemap, args.output)
