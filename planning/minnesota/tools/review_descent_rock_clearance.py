#!/usr/bin/env python3
"""Review the complete explicit R10 cliff/boulder repair without writing a pack."""
import argparse
from copy import deepcopy
import json
from pathlib import Path
import time

from PIL import Image

from render_archive import save_render
from review_selection_lineage import require_selected_ancestor
from render_reference import SourceArt, load_pack, npc_palette_pack
from review_iron_rock_clearance import blocks, canonical, digest, flood, geometry
from review_passage_native import actors


def main():
    cli = argparse.ArgumentParser(description=__doc__)
    cli.add_argument('--source', type=Path, default=Path(
        'planning/minnesota/revisions/descent-rock-clearance-v01/source-v01.json'))
    cli.add_argument('--output', type=Path, required=True)
    args = cli.parse_args()
    plan = json.loads(args.source.read_text())
    world_bytes = Path('planning/minnesota/world.json').read_bytes()
    world = json.loads(world_bytes)
    require_selected_ancestor(world, plan['selected_parent_revision'], plan['selected_parent_sha256'])
    parent = Path(plan['parent_pack'])
    assert digest(parent.read_bytes()) == plan['parent_pack_sha256']
    pack = load_pack(parent)
    assert pack['identity'] == plan['parent_identity']
    before = {name: deepcopy(pack['data']['maps'][name])
              for name in ['MnR10C00R00', 'MnR10C01R00']}
    proposal = plan['maps'][0]
    assert proposal['id'] == 'MnR10C00R00'
    assert digest(canonical(before[proposal['id']])) == proposal['native_module_sha256']
    assert before[proposal['id']]['blocks'] == blocks(proposal['before_rows'])
    started = time.perf_counter()
    after = deepcopy(before)
    revised = after[proposal['id']]
    applied = set()
    for edit in plan['edits']:
        x, y = edit['metatile']
        assert 0 <= x < 32 and 0 <= y < 20 and (x, y) not in applied
        index = y * 32 + x
        assert revised['blocks'][index] == int(edit['before'], 16)
        revised['blocks'][index] = int(edit['after'], 16)
        applied.add((x, y))
    assert len(applied) == 6
    assert revised['blocks'] == blocks(proposal['after_rows'])
    old_kinds, old_floor = geometry(pack, before)
    new_kinds, new_floor = geometry(pack, after)
    protected = {'TALL_GRASS', 'WATER', 'CUT_TREE', 'ICE', 'CAVE', 'LADDER',
                 'UP_WALL', 'DOWN_WALL', 'WARP_CARPET_DOWN'}
    assert {p: k for p, k in old_kinds.items() if k in protected} == {
        p: k for p, k in new_kinds.items() if k in protected}
    old_main, new_main = flood((1, 10), old_floor), flood((1, 10), new_floor)
    assert old_main == new_main, 'ordinary public road or its reachable shoulders changed'
    old_upper, new_upper = flood((53, 5), old_floor), flood((53, 5), new_floor)
    assert not ((old_upper & new_floor) - new_upper), 'remaining upper floor became isolated'
    assert not new_upper & new_main, 'decorative upper ridge gained an unintended side entry'
    assert old_floor - new_floor == {
        (x * 2 + dx, y * 2 + dy)
        for x, y in [(26, 3), (27, 4), (26, 5)]
        for dx in range(2) for dy in range(2)}
    points = [(0, 10), (95, 24), (11, 19), (11, 22), (33, 11), (33, 14), (45, 20), (57, 20)]
    assert all(point in new_main for point in points)
    rock_count = lambda modules: sum(
        sum(block in {0x0a, 0x8b, 0xaf} for block in module['blocks'])
        for module in modules.values())
    assert rock_count(before) == rock_count(after) == 7
    args.output.mkdir(parents=True, exist_ok=False)
    art, npc_pack = SourceArt(pack), npc_palette_pack(pack, 'day')
    for stage, modules in [('before', before), ('after', after)]:
        whole = Image.new('RGB', (1536, 640))
        for name, module in modules.items():
            attr = module['attributes']
            image = actors(npc_pack, art.render(attr['tileset_name'], attr['width'], attr['height'], module['blocks']), module)
            save_render(image, args.output / f'{name}-{stage}-original.png')
            whole.paste(image, (int(name[-5:-3]) * 1024, 0))
        save_render(whole, args.output / f'whole-{stage}-original.png')
        save_render(whole.resize((1152, 480), Image.Resampling.NEAREST), args.output / f'whole-{stage}-overview.png')
        crop = whole.crop((736, 0, 1024, 384))
        save_render(crop, args.output / f'cliff-{stage}-original.png')
        save_render(crop.resize((576, 768), Image.Resampling.NEAREST), args.output / f'cliff-{stage}-2x.png')
    report = {'status': 'SOURCE_REVIEW_REQUIRES_NATIVE_VERIFICATION',
              'source': str(args.source), 'source_sha256': digest(args.source.read_bytes()),
              'parent_sha256': plan['parent_pack_sha256'], 'changed_metatiles': len(applied),
              'rock_count': 7, 'moved_rocks': 3,
              'main_ordinary_reach_exact': True, 'upper_remaining_ordinary_floor_reachable': True,
              'upper_ridge_not_open_to_public_road': True, 'stairs_and_endpoints_reachable': True,
              'grass_water_cut_ice_and_mouth_quadrants_exact': True,
              'elapsed_after_load_seconds': time.perf_counter() - started,
              'world_unchanged': world_bytes == Path('planning/minnesota/world.json').read_bytes(),
              'scope': 'Whole source and conservative Floor/Grass graphs. No native traversal, GPU or earned access claim.'}
    (args.output / 'receipt.json').write_text(json.dumps(report, indent=2) + '\n')
    print(json.dumps(report))


if __name__ == '__main__':
    main()
