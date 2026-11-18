#!/usr/bin/env python3
"""Render native before/after terrain from actual packs; never writes pack content."""
import argparse
import json
import sys
from pathlib import Path

from PIL import Image, ImageDraw

sys.path.insert(0, str(Path(__file__).resolve().parents[2] / 'tools'))
from render_reference import SourceArt, load_pack
from render_archive import save_render


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('before', type=Path)
    parser.add_argument('after', type=Path)
    parser.add_argument('output', type=Path)
    args = parser.parse_args()
    contracts = json.loads(Path(__file__).with_name('caches.json').read_text())
    args.output.mkdir(parents=True, exist_ok=True)
    sheet = Image.new('RGB', (640, 320 * len(contracts)), '#f5f1e8')
    draw = ImageDraw.Draw(sheet)
    for column, (label, path) in enumerate([('Before', args.before), ('After', args.after)]):
        pack = load_pack(path)
        art = SourceArt(pack)
        for row, cache in enumerate(contracts):
            room = pack['data']['maps'][cache['map']]
            attr = room['attributes']
            terrain = art.render(attr['tileset_name'], attr['width'], attr['height'], room['blocks'])
            save_render(terrain, args.output / f"{cache['map']}-{label.lower()}-whole.png")
            x, y = cache['tile']
            # Ten by nine walking tiles: unchanged position, noon palette, native pixels.
            camera = terrain.crop((x*16-72, y*16-64, x*16+88, y*16+80))
            save_render(camera, args.output / f"r{cache['route']:02}-{label.lower()}-camera.png")
            sheet.paste(camera.resize((320, 288), Image.Resampling.NEAREST), (column*320, row*320+30))
            draw.text((column*320+4, row*320+4), f"R{cache['route']:02} {label} | {cache['item']}", fill='#222222')
        del art, pack
    save_render(sheet, args.output / 'before-after.png')


if __name__ == '__main__':
    main()
