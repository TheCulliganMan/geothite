#!/usr/bin/env python3
"""Review the complete native basin and its unchanged nine-block drain.

Only in-memory terrain is changed for source review. Rust owns publication.
Every invocation uses a fresh output directory and preserves its originals.
"""
import argparse
import base64
from collections import deque
from copy import deepcopy
import hashlib
import json
from pathlib import Path
import time

from PIL import Image, ImageDraw

from render_archive import save_render
from review_selection_lineage import require_selected_ancestor
from render_progress_atlas import native_room
from render_reference import load_pack


def digest(raw):
    return hashlib.sha256(raw).hexdigest()


def canonical(value):
    return json.dumps(value, sort_keys=True, separators=(',', ':'), ensure_ascii=False).encode()


def grid(pack, module):
    collision = pack['data']['tilesets']['cave']['collision']
    return {(x, y): collision[f'{module["blocks"][y // 2 * 14 + x // 2]:02x}'][y % 2 * 2 + x % 2]
            for y in range(28) for x in range(28)}


def flood(kinds, surf=False):
    allowed = {at for at, token in kinds.items()
               if (token == 'FLOOR' or surf and token == 'WATER') and at != (14, 2)}
    reached, queue = {(2, 25)}, deque([(2, 25)])
    while queue:
        x, y = queue.popleft()
        for at in [(x - 1, y), (x + 1, y), (x, y - 1), (x, y + 1)]:
            if at in allowed and at not in reached:
                reached.add(at); queue.append(at)
    return reached


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--source', type=Path,
                        default=Path('planning/minnesota/caves/inner-basin-bedrock-v01/source-v03.json'))
    parser.add_argument('--installed', type=Path)
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    plan = json.loads(args.source.read_text())
    parent = Path(plan['parent_pack'])
    assert digest(parent.read_bytes()) == plan['parent_pack_sha256']
    world = json.loads(Path('planning/minnesota/world.json').read_text())
    require_selected_ancestor(world, plan['selected_ancestor_revision'], plan['selected_ancestor_sha256'])
    base = load_pack(parent)
    original = base['data']['maps'][plan['map']]
    assert base['identity'] == plan['parent_identity']
    assert digest(canonical(original)) == plan['native_module_sha256']
    assert digest(canonical(base['data']['tilesets']['cave'])) == plan['tileset_sha256']
    parse = lambda rows: [int(v, 16) for row in rows for v in row.split()]
    assert original['blocks'] == parse(plan['before_rows'])
    room = deepcopy(original)
    room['blocks'] = parse(plan['rows'])
    room['attributes']['border_block'] = plan['border_block']
    installed = None
    if args.installed:
        installed = load_pack(args.installed)
        assert installed['data']['maps'][plan['map']] == room
        residual = deepcopy(installed)
        residual['data']['maps'][plan['map']] = original
        residual['data']['map_attributes'][plan['map']] = base['data']['map_attributes'][plan['map']]
        label = original['attributes']['blocks_label']
        assert list(base64.b64decode(installed['data']['map_blocks'][label])) == room['blocks']
        residual['data']['map_blocks'][label] = base['data']['map_blocks'][label]
        assert installed['report']['manifests'] == base['report']['manifests'] + ['minnesota-inner-basin-bedrock-v1']
        assert base['identity'] == dict(installed['identity'], content_hash=base['identity']['content_hash'],
                                        runtime_modpack_id=base['identity']['runtime_modpack_id'])
        residual['identity'], residual['report'] = base['identity'], base['report']
        assert residual == base, 'Entire unrelated parent must remain exact'
    started = time.perf_counter()
    args.output.mkdir(parents=True, exist_ok=False)
    states, scenes = [], []
    for label, module, opened in [('before', deepcopy(original), False),
                                  ('closed', deepcopy(room), False), ('drained', deepcopy(room), True)]:
        if opened:
            for change in module['script_block_changes']:
                index = change['y'] // 2 * 14 + change['x'] // 2
                assert module['blocks'][index] == original['blocks'][index]
                module['blocks'][index] = change['block_id']
        else:
            # The native callback hides Strandrak until the outlet is acknowledged.
            module['objects'] = []
        kinds = grid(base, module)
        reached, surf = flood(kinds), flood(kinds, True)
        assert all(token == 'WALL' for (x, y), token in kinds.items() if x in [0, 27] or y in [0, 27])
        assert tuple(plan['creature_approach']) in reached if opened else tuple(plan['creature_approach']) not in surf
        for control in plan['preserved_controls']:
            assert kinds[tuple(control['at'])] == 'WALL'
            assert tuple(control['approach']) in reached
        if label != 'before':
            assert {(x, y) for (x, y), token in kinds.items() if token == 'FLOOR' and y >= 8} <= reached
        scene = native_room(module, installed or base, archive=args.output / label, native_origin=True)
        scenes.append((label, scene))
        states.append(dict(state=label, reachable_dry=len(reached), reachable_with_surf=len(surf),
                           floor=sum(t == 'FLOOR' for t in kinds.values()),
                           creature_approach_reached=tuple(plan['creature_approach']) in reached))
    sheet = Image.new('RGB', (448 * 3, 476), '#0c1721')
    draw = ImageDraw.Draw(sheet)
    for i, (label, scene) in enumerate(scenes):
        draw.text((i * 448 + 8, 8), label, fill='white')
        sheet.paste(scene, (i * 448, 28))
    save_render(sheet, args.output / 'whole-before-closed-drained.png')
    for name, at in plan['review_points'].items():
        scene = scenes[2 if name == 'drained_chamber' else 1][1]
        x, y = at
        crop = scene.crop((x * 16 - 80, y * 16 - 72, x * 16 + 80, y * 16 + 72))
        save_render(crop, args.output / (name + '-original.png'))
        save_render(crop.resize((640, 576), Image.Resampling.NEAREST), args.output / (name + '-4x.png'))
    # These named first-route maps and all their fishing/rod contracts are part
    # of the complete residual check above; no perimeter repair may alter them.
    report = dict(status='PASS_SOURCE_AND_INSTALLED_PRESERVATION' if installed else 'PASS_SOURCE_ONLY',
                  parent_sha256=plan['parent_pack_sha256'], source_sha256=digest(args.source.read_bytes()),
                  installed_sha256=digest(args.installed.read_bytes()) if installed else None,
                  states=states, source_palette='nite from original PALETTE_NITE',
                  unchanged_complete_maps=len(base['data']['maps']) - 1 if installed else None,
                  route_pixels=448 * 448, review_seconds=time.perf_counter() - started,
                  source_block_variety=len(set(original['blocks'])), new_block_variety=len(set(room['blocks'])),
                  scope='Source/native art and entire-parent preservation only. Controller input, actual drain acknowledgment, battle, field-move and Save/Continue acceptance remain separate.')
    (args.output / 'receipt.json').write_text(json.dumps(report, indent=2) + '\n')
    print(json.dumps(report, indent=2), flush=True)


if __name__ == '__main__':
    main()
