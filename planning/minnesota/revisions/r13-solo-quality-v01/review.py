#!/usr/bin/env python3
"""Read-only native-art review of hand-authored Ely approach changes."""
import argparse
import hashlib
import importlib.util
import json
from pathlib import Path
import sys

from PIL import Image, ImageDraw

ROOT = Path(__file__).resolve().parents[4]
sys.path.insert(0, str(ROOT / 'planning/minnesota/tools'))
from audit_outdoor_connections import collision
from render_archive import save_render
from render_reference import load_pack, room_sheet
from review_major_city import ReviewArt, player_camera, render_map

HELPER = ROOT / 'planning/minnesota/revisions/north-shore-ely-geography-v01/inland-winter-r12-r13/review_inland.py'
spec = importlib.util.spec_from_file_location('inland_review', HELPER)
inland = importlib.util.module_from_spec(spec)
spec.loader.exec_module(inland)

PACK = ROOT / 'content-packs/minnesota-superior-compact-v03.crystalpack'
PACK_SHA = '032304d0fd1d51863a2c3bf2562b20728e22c7535d1c194555621bc8e85b4899'
VIEWS = [
    ('north-ridge', [31, 7]), ('wind-tor', [46, 7]),
    ('shelter-turn', [25, 69]), ('central-clearing', [29, 80]),
    ('east-grass-choice', [43, 86]), ('south-clearing', [34, 97]),
    ('cut-refuge', [18, 102]), ('ely-arrival', [34, 123]),
]


def sha(value):
    return hashlib.sha256(value).hexdigest()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--epoch', required=True)
    args = parser.parse_args()
    assert '/' not in args.epoch and args.epoch not in {'.', '..'}
    output = ROOT / 'planning/minnesota/renders/r13-solo-quality-v01' / args.epoch
    assert not output.exists(), 'Every review epoch must be retained'
    assert sha(PACK.read_bytes()) == PACK_SHA, 'Explicit reviewed native parent required'
    pack = load_pack(PACK)
    module, objects, placements = inland.whole(pack, 'MnR13')
    kinds = collision(pack, module)
    bodies = {tuple(o['tile']) for o in objects}
    output.mkdir(parents=True)
    review = ReviewArt(pack)
    ground, scene, _ = render_map(review, module, objects, 'day', output / 'actual')
    save_render(scene.resize((512, 1024), Image.Resampling.NEAREST), output / 'whole-review.png')
    grid = scene.copy()
    draw = ImageDraw.Draw(grid)
    for y in range(64):
        draw.line((0, y * 32, 1024, y * 32), fill='#a58d75', width=1)
        draw.text((0, y * 32 + 2), str(y), fill='white', stroke_width=1, stroke_fill='black')
    for x in range(32):
        draw.line((x * 32, 0, x * 32, 2048), fill='#a58d75', width=1)
        draw.text((x * 32 + 3, 1), str(x), fill='white', stroke_width=1, stroke_fill='black')
    save_render(grid, output / 'whole-metatile-grid.png')
    frames = []
    cameras = []
    for name, requested in VIEWS:
        candidates = [(abs(x-requested[0]) + abs(y-requested[1]), x, y)
                      for (x, y), token in kinds.items()
                      if token == 'FLOOR' and (x, y) not in bodies]
        _, x, y = min(candidates)
        frame, _ = player_camera(review, module, objects, ground, scene, 'day',
                                  {'core': [x, y]}, output / 'views' / name)
        frames.append((f'{name} core{x},{y}', frame.resize((480, 432), Image.Resampling.NEAREST)))
        cameras.append({'id': name, 'requested_core': requested, 'actual_core': [x, y]})
    save_render(room_sheet(frames, 4), output / 'player-cameras.png')
    receipt = {
        'status': 'Actual native source-art review; not GPU or input proof',
        'parent_file_sha256': PACK_SHA, 'identity': pack['identity']['content_hash'],
        'module_preimages': {p['map']: sha(json.dumps(pack['data']['maps'][p['map']],
            sort_keys=True, separators=(',', ':'), ensure_ascii=False).encode()) for p in placements},
        'actors': [{'id': o['id'], 'core': o['tile']} for o in objects], 'cameras': cameras,
        'canonical_preimages': {str(p.relative_to(ROOT)): sha(p.read_bytes()) for p in [
            ROOT / 'planning/minnesota/routes/MN_R13.layout.json',
            ROOT / 'planning/minnesota/routes/MN_R13.json']},
    }
    (output / 'receipt.json').write_text(json.dumps(receipt, indent=2) + '\n')
    (output / 'review.snapshot.py').write_bytes(Path(__file__).read_bytes())
    print(output)


if __name__ == '__main__':
    main()
