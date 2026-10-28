#!/usr/bin/env python3
"""Render the actual coast-to-Ely pack at its connection-derived positions.

Read-only inspection: no proposed terrain, invented connections or pack writes.
Retain a fresh output directory per invocation, including source-pixel originals.
This is source-art review; controller and GPU evidence remain separate.
"""
import argparse
from collections import deque
import hashlib
import json
from pathlib import Path

from PIL import Image, ImageDraw

from audit_outdoor_connections import neighbor_origin, OPPOSITE
from render_archive import save_render
from render_progress_atlas import native_room
from render_reference import load_pack


def positions_from_connections(maps):
    """Require one consistent reciprocal exterior graph and no overlaps."""
    positions = {'MnMinneapolis': (0, 0)}
    queue = deque(positions)
    while queue:
        name = queue.popleft()
        a = maps[name]['attributes']
        assert len({e['direction'] for e in a['connections']}) == len(a['connections'])
        for edge in a['connections']:
            target = edge['target_map']
            assert target in maps, (name, edge)
            b = maps[target]['attributes']
            inverse = dict(direction=OPPOSITE[edge['direction']], target_map=name,
                           offset=-edge['offset'])
            assert inverse in b['connections'], (name, edge, 'missing reciprocal')
            point = neighbor_origin(positions[name], a, b, edge)
            if target in positions:
                assert positions[target] == point, (name, edge, 'inconsistent cycle')
            else:
                positions[target] = point
                queue.append(target)
    # Other exterior groups can exist; require the full requested region below.
    names = sorted(positions)
    for i, name in enumerate(names):
        x, y = positions[name]
        a = maps[name]['attributes']
        for target in names[i + 1:]:
            u, v = positions[target]
            b = maps[target]['attributes']
            assert not (max(x, u) < min(x + a['width'], u + b['width']) and
                        max(y, v) < min(y + a['height'], v + b['height'])), (name, target, 'overlap')
    return positions


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--pack', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--compact', action='store_true',
                        help='Require the compact coast/Gunflint revision geometry.')
    args = parser.parse_args()
    assert not args.output.exists(), 'choose a fresh immutable render epoch'
    pack = load_pack(args.pack)
    maps = {n: m for n, m in pack['data']['maps'].items() if n.startswith('Mn') and
            (m['attributes']['environment'] or '').lower() in {'town', 'route', 'outdoor'}}
    positions = positions_from_connections(maps)
    selected = sorted(n for n in maps if n.startswith(('MnR11C', 'MnR12C', 'MnR13C')) or
                      n in {'MnGrandMarais', 'MnMaraisCedarApproach', 'MnMaraisInlandNeck',
                            'MnBoundaryWaters', 'MnPreservePineEntry', 'MnEly'})
    expected_origins = ({'MnGrandMarais': (226, -400),
                         'MnMaraisCedarApproach': (222, -412),
                         'MnR12C00R00': (190, -432),
                         'MnR12C00R01': (190, -408),
                         'MnBoundaryWaters': (158, -430)} if args.compact else
                        {'MnGrandMarais': (246, -400), 'MnMaraisCedarApproach': (227, -419),
                         'MnMaraisInlandNeck': (162, -419)})
    for name, point in expected_origins.items():
        assert positions.get(name) == point, (name, positions.get(name), point)
    if args.compact:
        assert 'MnMaraisInlandNeck' not in maps, 'long inland connector must be retired'
        assert [maps['MnMaraisCedarApproach']['attributes'][key]
                for key in ['width', 'height']] == [20, 12]
        assert [maps['MnR11C01R00']['attributes'][key]
                for key in ['width', 'height']] == [4, 24]
    assert all(n in positions for n in selected)
    args.output.mkdir(parents=True)
    x0 = min(positions[n][0] for n in selected)
    y0 = min(positions[n][1] for n in selected)
    x1 = max(positions[n][0] + maps[n]['attributes']['width'] for n in selected)
    y1 = max(positions[n][1] + maps[n]['attributes']['height'] for n in selected)
    original = Image.new('RGB', ((x1 - x0) * 32, (y1 - y0) * 32), '#14202b')
    placements = []
    for name in selected:
        module = maps[name]
        image = native_room(module, pack, archive=args.output / 'maps' / name, native_origin=True)
        x, y = positions[name]
        original.paste(image, ((x - x0) * 32, (y - y0) * 32))
        placements.append(dict(map=name, origin_metatiles=[x, y],
                               dimensions_metatiles=[module['attributes']['width'], module['attributes']['height']],
                               connections=module['attributes']['connections']))
    save_render(original, args.output / 'connected-original-32px-per-metatile.png')
    preview = original.resize((original.width // 4, original.height // 4), Image.Resampling.NEAREST)
    labeled = Image.new('RGB', (preview.width, preview.height + 72), '#14202b')
    labeled.paste(preview, (0, 72))
    draw = ImageDraw.Draw(labeled)
    title = ('ACTUAL PACK: SHORT COASTAL BEND / COMPACT GUNFLINT / ELY' if args.compact
             else 'ACTUAL PACK: SUPERIOR COAST / INLAND WINTER / ELY')
    draw.text((12, 10), title, fill='white')
    draw.text((12, 30), '8px/metatile. Dark gaps contain no terrain. Source art; gameplay/GPU proof is separate.', fill='white')
    digest = hashlib.sha256(args.pack.read_bytes()).hexdigest()
    draw.text((12, 50), 'Pack SHA256: ' + digest, fill='white')
    for row in placements:
        x, y = row['origin_metatiles']
        draw.text(((x - x0) * 8 + 2, (y - y0) * 8 + 74), row['map'].removeprefix('Mn'),
                  fill='white', stroke_width=1, stroke_fill='black')
    save_render(labeled, args.output / 'connected-review.png')
    (args.output / 'receipt.json').write_text(json.dumps(dict(
        status='ACTUAL_PACK_SOURCE_ART_NOT_CONTROLLER_OR_GPU_ACCEPTANCE', pack_sha256=digest,
        identity=pack['identity']['content_hash'], reached_exteriors=len(positions),
        placements=placements, tool_sha256=hashlib.sha256(Path(__file__).read_bytes()).hexdigest()), indent=2) + '\n')
    (args.output / Path(__file__).name).write_bytes(Path(__file__).read_bytes())
    print(args.output / 'connected-review.png')


if __name__ == '__main__':
    main()
