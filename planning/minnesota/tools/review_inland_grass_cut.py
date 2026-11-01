#!/usr/bin/env python3
"""Review explicit native grass-Cut composites without writing a game pack."""
import argparse
from copy import deepcopy
import hashlib
import json
from pathlib import Path
import time

from PIL import Image, ImageChops, ImageDraw

from render_archive import save_render
from review_selection_lineage import require_selected_ancestor
from render_reference import SourceArt, load_pack


def digest(raw):
    return hashlib.sha256(raw).hexdigest()


def canonical(value):
    # CBOR integer keys must first become JSON string keys, as in Rust Value.
    return json.dumps(json.loads(json.dumps(value)), sort_keys=True, separators=(',', ':')).encode()


def compose(pack, plan):
    assert digest(canonical(pack['data']['field_moves']['cut'])) == plan['cut_rule_sha256']
    for alias, binding in plan['tilesets'].items():
        assert digest(canonical(pack['data']['tilesets'][alias])) == binding['definition_sha256']
        for path, expected in binding['runtime_sha256'].items():
            assert digest(bytes(pack['runtime_files'][path])) == expected
    for record in plan['composites']:
        alias = record['tileset']
        grass, ground, target = [int(record[key], 16) for key in ['grass_block', 'ground_block', 'replacement_block']]
        path = f'data/tilesets/{alias}_metatiles.bin'
        layout = bytes(pack['runtime_files'][path])
        assert target == len(layout) // 16 < 256
        definition = pack['data']['tilesets'][alias]
        collision = definition['collision'][record['grass_block']]
        assert record['collision'] == ['FLOOR' if kind == 'TALL_GRASS' else kind for kind in collision]
        parts = bytes(int(word, 16) for row in record['native_glyph_rows'] for word in row.split())
        assert len(parts) == 16
        for index, glyph in enumerate(parts):
            source = ground if collision[(index // 8) * 2 + (index % 4) // 2] == 'TALL_GRASS' else grass
            assert glyph == layout[source * 16 + index]
        pack['runtime_files'][path] = layout + parts
        definition['collision'][record['replacement_block']] = list(record['collision'])
    prototype = pack['data']['field_moves']['cut']['replacements']['johto'][3]
    assert prototype == {'replacement_block_id': 2, 'variant': 'grass'}
    for rule in plan['rules']:
        alias = rule['tileset']
        grass, target = int(rule['grass_block'], 16), int(rule['replacement_block'], 16)
        replacements = pack['data']['field_moves']['cut']['replacements'].setdefault(alias, {})
        assert grass not in replacements
        replacements[grass] = {**prototype, 'replacement_block_id': target}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--source', type=Path, default=Path(
        'planning/minnesota/revisions/inland-grass-cut-v01/source-v02.json'))
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
    original = deepcopy(pack)
    started = time.perf_counter()
    compose(pack, plan)
    assert pack['data']['maps'] == original['data']['maps']
    art, original_art = SourceArt(pack), SourceArt(original)
    args.output.mkdir(parents=True, exist_ok=False)
    sheet = Image.new('RGB', (600, len(plan['composites']) * 146), (22, 22, 22))
    draw = ImageDraw.Draw(sheet)
    unchanged_pixels = 0
    non_grass_pixels = 0
    for row, record in enumerate(plan['composites']):
        alias = record['tileset']
        grass, floor = int(record['grass_block'], 16), int(record['replacement_block'], 16)
        before, after = art.blocks(alias)[grass], art.blocks(alias)[floor]
        collision = pack['data']['tilesets'][alias]['collision'][record['grass_block']]
        for q in range(4):
            x, y = q % 2 * 16, q // 2 * 16
            box = (x, y, x + 16, y + 16)
            if collision[q] != 'TALL_GRASS':
                assert ImageChops.difference(before.crop(box), after.crop(box)).getbbox() is None
                non_grass_pixels += 256
        save_render(before, args.output / f'{alias}-{grass:02x}-before-original.png')
        save_render(after, args.output / f'{alias}-{floor:02x}-cleared-original.png')
        draw.text((6, row * 146 + 5), f'{alias}: {grass:02x} -> {floor:02x}', fill='white')
        sheet.paste(before.resize((128, 128), Image.Resampling.NEAREST), (6, row * 146 + 18))
        sheet.paste(after.resize((128, 128), Image.Resampling.NEAREST), (154, row * 146 + 18))
    for alias in plan['tilesets']:
        a, b = art.blocks(alias), original_art.blocks(alias)
        for left, right in zip(a, b):
            assert ImageChops.difference(left.convert('RGB'), right.convert('RGB')).getbbox() is None
            unchanged_pixels += 1024
    save_render(sheet, args.output / 'native-grass-cut-composites.png')
    report = {'status': 'SOURCE_REVIEW_REQUIRES_NATIVE_VERIFICATION', 'source_sha256': digest(args.source.read_bytes()),
              'native_cut_rules': len(plan['rules']), 'new_floor_composites': len(plan['composites']),
              'unchanged_original_block_pixels': unchanged_pixels, 'unchanged_non_grass_quadrant_pixels': non_grass_pixels,
              'every_map_exact': True, 'only_tall_grass_quadrants_cleared': True,
              'elapsed_after_load_seconds': time.perf_counter() - started,
              'world_unchanged': world_bytes == Path('planning/minnesota/world.json').read_bytes(),
              'scope': 'Read-only source composition and unchanged placed terrain. Native menu Cut, badge denials, movement, resets/Continue and rendered captures require the actual child.'}
    (args.output / 'receipt.json').write_text(json.dumps(report, indent=2) + '\n')
    print(json.dumps(report))


if __name__ == '__main__':
    main()
