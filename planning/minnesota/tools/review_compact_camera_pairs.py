#!/usr/bin/env python3
"""Retain matched classic-camera crops beside their authored source references."""
import argparse
import json
from pathlib import Path

from PIL import Image, ImageDraw


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('captures', type=Path)
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    assert not args.output.exists(), 'preserve each comparison epoch'
    receipt = json.loads((args.captures / 'receipt.json').read_text())
    assert all(row['returncode'] == 0 for row in receipt['captures'])
    args.output.mkdir(parents=True)
    pairs = []
    for row in receipt['captures']:
        folder = args.captures / row['name']
        native = Image.open(folder / 'native-2d-original.png').convert('RGB')
        source = Image.open(folder / 'source160-original.png').convert('RGB')
        assert native.size == (640, 576) and source.size == (160, 144)
        # The production capture is three physical pixels per source pixel.
        # Preserve its full overscan separately; the center is classic160×144.
        crop = native.crop((80, 72, 560, 504))
        crop.save(args.output / (row['name'] + '-native-classic-original.png'))
        pair = Image.new('RGB', (648, 324), '#17222b')
        draw = ImageDraw.Draw(pair)
        draw.text((4, 3), row['name'], fill='white')
        draw.text((4, 18), 'Authored source', fill='white')
        draw.text((328, 18), 'Actual native classic center', fill='white')
        pair.paste(source.resize((320, 288), Image.Resampling.NEAREST), (0, 36))
        pair.paste(crop.resize((320, 288), Image.Resampling.NEAREST), (328, 36))
        pair.save(args.output / (row['name'] + '-comparison.png'))
        pairs.append(pair)
    for start in range(0, len(pairs), 6):
        page = Image.new('RGB', (1944, 648), '#17222b')
        for i, pair in enumerate(pairs[start:start + 6]):
            page.paste(pair, ((i % 3) * 648, (i // 3) * 324))
        page.save(args.output / f'comparison-contact-{start // 6 + 1:02}.png')
    (args.output / 'receipt.json').write_text(json.dumps(dict(
        pack_sha256=receipt['pack_sha256'], captures=str(args.captures),
        source_scope=receipt['scope'], compared_count=len(pairs),
        crop_policy='Original640x576 retained; centered80,72..560,504 gives480x432 native3x classic view.'), indent=2) + '\n')
    print(args.output)


if __name__ == '__main__':
    main()
