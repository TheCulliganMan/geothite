#!/usr/bin/env python3
"""Review the explicit R09 rock relocation without serializing a game pack."""
import argparse
from collections import deque
from copy import deepcopy
import hashlib
import json
from pathlib import Path
import time

from PIL import Image

from render_archive import save_render
from review_selection_lineage import require_selected_ancestor
from render_reference import SourceArt, load_pack, npc_palette_pack
from review_passage_native import actors


def digest(raw):
    return hashlib.sha256(raw).hexdigest()


def canonical(value):
    return json.dumps(value, sort_keys=True, separators=(',', ':')).encode()


def blocks(rows):
    return [int(word, 16) for row in rows for word in row.split()]


def geometry(pack, modules):
    kinds, occupied = {}, set()
    for name, module in modules.items():
        cx, cy = int(name[-5:-3]), int(name[-2:])
        attr = module['attributes']
        collision = pack['data']['tilesets'][attr['tileset_name']]['collision']
        for y in range(attr['height'] * 2):
            for x in range(attr['width'] * 2):
                block = module['blocks'][y // 2 * attr['width'] + x // 2]
                kinds[x + cx * 64, y + cy * 48] = collision[f'{block:02x}'][y % 2 * 2 + x % 2]
        occupied.update((obj['x'] + cx * 64, obj['y'] + cy * 48) for obj in module['objects'])
    floor = {p for p, kind in kinds.items() if kind in {'FLOOR', 'TALL_GRASS'}} - occupied
    return kinds, floor


def flood(start, allowed):
    assert start in allowed
    found, queue = {start}, deque([start])
    while queue:
        x, y = queue.popleft()
        for target in [(x - 1, y), (x + 1, y), (x, y - 1), (x, y + 1)]:
            if target in allowed and target not in found:
                found.add(target)
                queue.append(target)
    return found


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--source', type=Path, default=Path(
        'planning/minnesota/revisions/iron-rock-clearance-v01/source-v02.json'))
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    plan = json.loads(args.source.read_text())
    world_bytes = Path('planning/minnesota/world.json').read_bytes()
    world = json.loads(world_bytes)
    require_selected_ancestor(world, plan['selected_parent_revision'], plan['selected_parent_sha256'])
    parent = Path(plan['parent_pack'])
    assert digest(parent.read_bytes()) == plan['parent_pack_sha256']
    pack = load_pack(parent)
    assert pack['identity'] == plan['parent_identity']
    modules = {}
    for record in plan['maps']:
        module = pack['data']['maps'][record['id']]
        assert digest(canonical(module)) == record['native_module_sha256']
        assert module['blocks'] == blocks(record['before_rows'])
        modules[record['id']] = deepcopy(module)
    started = time.perf_counter()
    before = deepcopy(modules)
    applied = set()
    for edit in plan['edits']:
        x, y = edit['metatile']
        assert (x, y) not in applied
        name = f'MnR09C{x // 32:02}R{y // 24:02}'
        module = modules[name]
        index = (y % 24) * module['attributes']['width'] + x % 32
        assert module['blocks'][index] == int(edit['before'], 16)
        module['blocks'][index] = int(edit['after'], 16)
        applied.add((x, y))
    assert len(applied) == plan['contracts']['changed_metatiles']
    for record in plan['maps']:
        assert modules[record['id']]['blocks'] == blocks(record['after_rows'])
    old_kinds, old_floor = geometry(pack, before)
    new_kinds, new_floor = geometry(pack, modules)
    protected = {'TALL_GRASS', 'CUT_TREE', 'WATER', 'CAVE', 'WARP_CARPET_DOWN'}
    assert {p: k for p, k in old_kinds.items() if k in protected} == {
        p: k for p, k in new_kinds.items() if k in protected}
    old_reach = flood((0, 50), old_floor)
    new_reach = flood((0, 50), new_floor)
    assert not ((old_reach & new_floor) - new_reach), 'an existing ordinary floor area became isolated'
    assert all(point in new_reach for point in [(127, 24), (36, 33), (36, 38), (49, 39), (49, 42)])
    assert (11, 63) not in new_reach, 'Cut pocket became freely reachable'
    rock_count = lambda ms: sum(module['blocks'].count(10) for module in ms.values())
    assert rock_count(before) == plan['contracts']['original_rock_count']
    assert rock_count(modules) == plan['contracts']['resulting_rock_count']
    art = SourceArt(pack)
    npc_pack = npc_palette_pack(pack, 'day')
    args.output.mkdir(parents=True, exist_ok=False)
    for stage, stage_modules in [('before', before), ('after', modules)]:
        whole = Image.new('RGB', (2048, 1152))
        for name, module in stage_modules.items():
            attr = module['attributes']
            image = actors(npc_pack, art.render(attr['tileset_name'], attr['width'], attr['height'], module['blocks']), module)
            save_render(image, args.output / f'{name}-{stage}-original.png')
            whole.paste(image, (int(name[-5:-3]) * 1024, int(name[-2:]) * 768))
        save_render(whole, args.output / f'whole-{stage}-original.png')
        save_render(whole.resize((1024, 576), Image.Resampling.NEAREST), args.output / f'whole-{stage}-overview.png')
        for name, box in [('north-crowns', (640, 128, 1376, 448)),
                          ('mine-apron', (512, 416, 864, 704)),
                          ('haul-toe', (864, 704, 1120, 960))]:
            crop = whole.crop(box)
            save_render(crop, args.output / f'{name}-{stage}-original.png')
            save_render(crop.resize((crop.width * 2, crop.height * 2), Image.Resampling.NEAREST), args.output / f'{name}-{stage}-2x.png')
    receipt = {'status': 'SOURCE_REVIEW_REQUIRES_NATIVE_VERIFICATION',
               'source_sha256': digest(args.source.read_bytes()), 'source': str(args.source),
               'parent_sha256': plan['parent_pack_sha256'], 'changed_cells': len(applied),
               'rock_count': rock_count(modules), 'all_previous_remaining_ordinary_floor_stays_reachable': True,
               'grass_cut_water_and_mouth_quadrants_exact': True,
               'ordinary_endpoints_and_both_stairs_reachable': True,
               'pre_cut_pocket_sealed': True, 'no_pack_written': True,
               'elapsed_after_load_seconds': time.perf_counter() - started,
               'world_unchanged': Path('planning/minnesota/world.json').read_bytes() == world_bytes,
               'scope': 'Conservative Floor/Grass static graph excludes directional walls and ledges; native movement, field moves and return gates need separate production verification.'}
    (args.output / 'receipt.json').write_text(json.dumps(receipt, indent=2) + '\n')
    print(json.dumps(receipt))


if __name__ == '__main__':
    main()
