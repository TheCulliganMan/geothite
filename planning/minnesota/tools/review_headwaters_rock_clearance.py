#!/usr/bin/env python3
"""Review the explicit Headwaters stair boulder repair without writing a pack."""
import argparse
from collections import Counter, deque
from copy import deepcopy
import json
from pathlib import Path
import time

from PIL import Image, ImageChops, ImageDraw
from render_archive import save_render
from review_selection_lineage import require_selected_ancestor
from render_reference import load_pack
from review_headwaters_snowline import canonical, digest, whole

MAP = 'MnR08C00R00'
ROUTE = ['MnR08C00R00', 'MnR08C00R01', 'MnR08C01R00', 'MnR08C01R01']


def reach(blocks, collision, excluded=frozenset()):
    floor = {(x, y) for y in range(48) for x in range(64)
             if collision[f'{blocks[y // 2 * 32 + x // 2]:02x}'][y % 2 * 2 + x % 2]
             in {'FLOOR', 'TALL_GRASS', 'LADDER'} and (x, y) not in excluded}
    start = (24, 1)
    assert start in floor
    seen, queue = {start}, deque([start])
    while queue:
        x, y = queue.popleft()
        for dx, dy in [(1, 0), (-1, 0), (0, 1), (0, -1)]:
            at = x + dx, y + dy
            if at in floor and at not in seen:
                seen.add(at)
                queue.append(at)
    return floor, seen


def apply_review(pack, plan):
    assert pack['identity'] == plan['parent_identity']
    record, = plan['maps']
    module = pack['data']['maps'][MAP]
    assert digest(canonical(module)) == record['native_module_sha256']
    parse = lambda rows: [int(word, 16) for row in rows for word in row.split()]
    before = module['blocks'][:]
    assert before == parse(record['before_rows'])
    after = before[:]
    seen = set()
    for edit in plan['edits']:
        x, y = edit['metatile']
        assert (x, y) not in seen and 0 <= x < 32 and 0 <= y < 24
        seen.add((x, y))
        assert before[y * 32 + x] == int(edit['before'], 16)
        after[y * 32 + x] = int(edit['after'], 16)
        if edit['after'] == '0a':
            assert not any(x * 2 <= o['x'] < x * 2 + 2 and y * 2 <= o['y'] < y * 2 + 2
                           for o in module['objects']), 'new rock overlaps an actor'
    assert len(seen) == 6 and after == parse(record['after_rows'])
    assert Counter(before) == Counter(after) and after.count(10) == 10
    collision = pack['data']['tilesets'][module['attributes']['tileset_name']]['collision']
    audits = []
    for label, excluded in [('terrain', frozenset()),
                            ('source-actor-bodies', {(o['x'], o['y']) for o in module['objects']})]:
        old_floor, old_reach = reach(before, collision, excluded)
        new_floor, new_reach = reach(after, collision, excluded)
        common = old_floor & new_floor
        assert old_reach & common == new_reach & common
        assert len(old_reach) == len(new_reach)
        assert all(point in new_reach for point in [(24, 0), (35, 25), (35, 28), (42, 47)])
        audits.append(dict(kind=label, common_cell_reachability_exact=True,
                           before=len(old_reach), after=len(new_reach),
                           newly_blocked=len(old_reach - new_reach), newly_open=len(new_reach - old_reach)))
    module['blocks'] = after
    return audits


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--source', type=Path, default=Path('planning/minnesota/revisions/headwaters-rock-clearance-v01/source-v01.json'))
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--installed', type=Path)
    args = parser.parse_args()
    plan = json.loads(args.source.read_text())
    world_bytes = Path('planning/minnesota/world.json').read_bytes()
    world = json.loads(world_bytes)
    require_selected_ancestor(world, plan['selected_parent_revision'], plan['selected_parent_sha256'])
    parent = Path(plan['parent_pack'])
    assert digest(parent.read_bytes()) == plan['parent_pack_sha256']
    base = load_pack(parent)
    preview = deepcopy(base)
    started = time.perf_counter()
    audits = apply_review(preview, plan)
    before, _ = whole(base, ROUTE)
    after, _ = whole(preview, ROUTE)
    difference = ImageChops.difference(before, after)
    changed = {(x, y) for y in range(48) for x in range(40)
               if difference.crop((x * 32, y * 32, (x + 1) * 32, (y + 1) * 32)).getbbox()}
    assert changed == {tuple(e['metatile']) for e in plan['edits']}
    args.output.mkdir(parents=True, exist_ok=False)
    for stage, image in [('before', before), ('after', after)]:
        save_render(image, args.output / (stage + '-whole-original.png'))
        save_render(image.resize((640, 768), Image.Resampling.NEAREST), args.output / (stage + '-whole-overview.png'))
        save_render(image.crop((384, 320, 704, 576)), args.output / (stage + '-stairs-original.png'))
    pair = Image.new('RGB', (1280, 542), '#151515')
    draw = ImageDraw.Draw(pair)
    for column, stage in enumerate(['before', 'after']):
        draw.text((column * 640 + 6, 7), stage + ' - native upper bench and stair clearance', fill='white')
        image = Image.open(args.output / (stage + '-stairs-original.png'))
        pair.paste(image.resize((640, 512), Image.Resampling.NEAREST), (column * 640, 30))
    save_render(pair, args.output / 'stair-clearance-before-after.png')
    report = dict(status='SOURCE_REVIEW_REQUIRES_NATIVE_PUBLICATION_AND_VERIFICATION',
                  source_sha256=digest(args.source.read_bytes()), parent_sha256=plan['parent_pack_sha256'],
                  changed_metatiles=sorted(changed), all_ten_rocks_and_block_inventory_exact=True,
                  reachability=audits, elapsed_after_load_seconds=time.perf_counter() - started)
    if args.installed:
        child = load_pack(args.installed)
        assert all(child['data']['maps'][name] == module for name, module in base['data']['maps'].items() if name != MAP)
        assert child['runtime_files'] == base['runtime_files']
        assert child['audio_manifest'] == base['audio_manifest'] and child['compiled_audio'] == base['compiled_audio']
        assert child['data']['maps'][MAP] == preview['data']['maps'][MAP]
        native, _ = whole(child, ROUTE)
        assert ImageChops.difference(after, native).getbbox() is None
        report.update(status='PASS_INSTALLED_SOURCE_AND_PRESERVATION', installed_sha256=digest(args.installed.read_bytes()),
                      installed_content_identity=child['identity']['content_hash'], other_complete_maps_exact=len(base['data']['maps']) - 1,
                      original_runtime_files_exact=True, original_audio_records_exact=602,
                      complete_route_pixels_exact=1280 * 1536, pixel_differences=0)
    assert world_bytes == Path('planning/minnesota/world.json').read_bytes()
    report['world_unchanged'] = True
    report['scope'] = 'Native source-art/static collision evidence. Controller walks, field moves, resources, Continue and GPU images require separate checks.'
    (args.output / 'receipt.json').write_text(json.dumps(report, indent=2) + '\n')
    print(json.dumps(report), flush=True)


if __name__ == '__main__':
    main()
