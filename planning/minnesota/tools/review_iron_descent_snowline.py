#!/usr/bin/env python3
"""Render the complete explicit Iron Range descent snowline without writing a pack."""
import argparse
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
from tamarack_thaw_art import compose


def digest(raw):
    return hashlib.sha256(raw).hexdigest()


def canonical(value):
    return json.dumps(value, sort_keys=True, separators=(',', ':')).encode()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--source', type=Path, default=Path(
        'planning/minnesota/regions/iron-descent-snowline-v01/source-v01.json'))
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
    originals = {record['id']: deepcopy(pack['data']['maps'][record['id']]) for record in plan['maps']}
    specification = plan['art']
    assert digest(canonical(pack['data']['tilesets'][specification['source']])) == specification['source_definition_sha256']
    for path, expected in specification['source_runtime_sha256'].items():
        assert digest(bytes(pack['runtime_files'][path])) == expected
    started = time.perf_counter()
    variants, result = compose(pack, specification)
    assert json.loads(json.dumps(result)) == plan['art_result'], 'art source recipe changed'
    for record in plan['maps']:
        module = pack['data']['maps'][record['id']]
        assert digest(canonical(module)) == record['native_module_sha256']
        assert module['blocks'] == [int(word, 16) for row in record['before_rows'] for word in row.split()]
        module['blocks'] = [int(word, 16) for row in record['after_rows'] for word in row.split()]
        module['attributes']['tileset_name'] = record['target_tileset']
        before = originals[record['id']]
        old_collision = pack['data']['tilesets'][record['source_tileset']]['collision']
        new_collision = pack['data']['tilesets'][record['target_tileset']]['collision']
        assert [old_collision[f'{block:02x}'] for block in before['blocks']] == [
            new_collision[f'{block:02x}'] for block in module['blocks']], 'snowline changed traversal'
        unchanged = deepcopy(module)
        unchanged['blocks'] = before['blocks']
        unchanged['attributes']['tileset_name'] = record['source_tileset']
        assert unchanged == before, 'unrelated module edit'
    art, npc_pack = SourceArt(pack), npc_palette_pack(pack, 'day')
    args.output.mkdir(parents=True, exist_ok=False)
    whole = Image.new('RGB', (1536, 640))
    for record in plan['maps']:
        name = record['id']
        module = pack['data']['maps'][name]
        attr = module['attributes']
        image = actors(npc_pack, art.render(attr['tileset_name'], attr['width'], attr['height'], module['blocks']), module)
        save_render(image, args.output / (name + '-original.png'))
        whole.paste(image, (int(name[-5:-3]) * 1024, int(name[-2:]) * 768))
    save_render(whole, args.output / 'whole-iron-descent-original.png')
    save_render(whole.resize((1152, 480), Image.Resampling.NEAREST), args.output / 'whole-iron-descent-overview.png')
    for name, box in [('western-stairs', (96, 96, 480, 480)),
                      ('grass-saddle-and-thaw', (512, 128, 896, 512)),
                      ('green-shore', (1088, 160, 1472, 544))]:
        crop = whole.crop(box)
        save_render(crop, args.output / (name + '-original.png'))
        save_render(crop.resize((768, 768), Image.Resampling.NEAREST), args.output / (name + '-2x.png'))
    receipt = {'status': 'SOURCE_PROPOSAL_REQUIRES_NATIVE_PUBLICATION_AND_VISUAL_REVIEW',
               'source': str(args.source), 'source_sha256': digest(args.source.read_bytes()),
               'parent_pack': str(parent), 'parent_sha256': plan['parent_pack_sha256'],
               'maps': [record['id'] for record in plan['maps']],
               'collision_quadrants_exact': True, 'all_other_map_fields_exact': True,
               'new_ice_quadrants': 0, 'cold_variants': len(variants),
               'original_native_glyph_addresses_preserved': True,
               'elapsed_after_load_seconds': time.perf_counter() - started,
               'world_unchanged': Path('planning/minnesota/world.json').read_bytes() == world_bytes,
               'scope': 'Complete daytime source-art review only. Weather, clock, masks, native animation and real controller traversal still require the native child and renderer checks.'}
    (args.output / 'receipt.json').write_text(json.dumps(receipt, indent=2) + '\n')
    print(json.dumps(receipt))


if __name__ == '__main__':
    main()
