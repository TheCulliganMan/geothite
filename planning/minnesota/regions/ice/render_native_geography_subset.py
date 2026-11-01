#!/usr/bin/env python3
"""Render a connected northwest subset from an existing actual native atlas."""
import argparse
import hashlib
import json
from pathlib import Path
import sys
from PIL import Image, ImageDraw

ROOT = Path(__file__).resolve().parents[4]
sys.path.insert(0, str(ROOT / 'planning/minnesota/tools'))
from render_archive import save_render
from render_progress_atlas import font

NAMES = (['MnRainyRiverIcefields', 'MnBoundaryWaters', 'MnPreservePineEntry']
    + [f'MnR20C0{x}R0{y}' for y in range(2) for x in range(2)]
    + [f'MnBoundarySafariC0{x}R0{y}' for y in range(3) for x in range(2)]
    + [f'MnR13C00R0{y}' for y in range(3)])


def main():
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument('--atlas', type=Path, required=True)
    p.add_argument('--output', type=Path, required=True)
    args = p.parse_args()
    report_path = args.atlas / 'native-connection-audit.json'
    report = json.loads(report_path.read_text())
    entries = {r['map']: r for r in report['placements'] if r['map'] in NAMES}
    assert set(entries) == set(NAMES)
    assert len({r['component'] for r in entries.values()}) == 1
    x0 = min(r['origin_metatiles'][0] for r in entries.values())
    y0 = min(r['origin_metatiles'][1] for r in entries.values())
    x1 = max(r['origin_metatiles'][0] + r['dimensions_metatiles'][0] for r in entries.values())
    y1 = max(r['origin_metatiles'][1] + r['dimensions_metatiles'][1] for r in entries.values())
    scale = 32
    board = Image.new('RGB', ((x1-x0)*scale+96, (y1-y0)*scale+236), '#111e29')
    draw = ImageDraw.Draw(board)
    draw.text((32, 20), 'NORTHWEST · ACTUAL NATIVE MAP GEOMETRY', font=font(36, True), fill='white')
    draw.text((32, 70), f"{len(entries)} exact outdoor maps · native 32px/metatile · pack {report['pack_sha256'][:16]}", font=font(24), fill='#bad5d3')
    draw.text((32, 108), 'Dark gaps contain no authored terrain. Neighboring maps beyond this subset are omitted.', font=font(24), fill='#e8bf84')
    draw.text((32, 146), 'Rainy River joins existing R20; this panel does not invent an International Falls town or a direct R17 link.', font=font(24), fill='#e8bf84')
    images = []
    for name, r in entries.items():
        path = args.atlas / 'maps' / name / 'scene-rgb.png'
        image = Image.open(path).convert('RGB')
        assert image.size == tuple(v*scale for v in r['dimensions_metatiles'])
        x, y = r['origin_metatiles']
        px, py = 48+(x-x0)*scale, 204+(y-y0)*scale
        board.paste(image, (px, py))
        draw.rectangle((px, py, px+image.width-1, py+image.height-1), outline='#7da7a6', width=2)
        draw.text((px+6, py+5), name.removeprefix('Mn'), font=font(20, True), fill='white', stroke_width=2, stroke_fill='#111e29')
        images.append({'map': name, 'scene': str(path), 'scene_sha256': hashlib.sha256(path.read_bytes()).hexdigest(), **r})
    args.output.mkdir(parents=True, exist_ok=False)
    save_render(board, args.output / 'northwest-native-full-resolution.png')
    display = board.copy()
    display.thumbnail((2000, 2200), Image.Resampling.NEAREST)
    save_render(display, args.output / 'northwest-native-display.png')
    (args.output/'proof.json').write_text(json.dumps({'status':'ACTUAL_PACK_TERRAIN_NOT_CONTROLLER_CAPTURE',
        'pack_sha256':report['pack_sha256'], 'audit_sha256':hashlib.sha256(report_path.read_bytes()).hexdigest(),
        'world_metatile_bounds_exclusive':[x0,y0,x1,y1], 'pixels_per_metatile':scale,
        'maps':images, 'limits':['No synthetic terrain or invented connection.', 'Native runtime/controller and earned progression are separate evidence.']},indent=2)+'\n')


if __name__ == '__main__':
    main()
