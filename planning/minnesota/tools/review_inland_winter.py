#!/usr/bin/env python3
"""Render explicitly authored inland winter proposals without writing a pack.

The bound source candidate includes the reviewed Bemidji roof/cliff repair.
All map rows, actors, programs, doors and collision quadrants remain exact;
only the specified private art aliases change. This is source review, not
native publication or gameplay acceptance. Keep a fresh output per render.
"""
import argparse
from copy import deepcopy
import hashlib
import json
from pathlib import Path
import re
import time

from PIL import Image

from inland_winter_preview_art import apply_alias
from render_archive import save_render
from render_reference import SourceArt, load_pack, npc_palette_pack
from review_passage_native import actors


def sha(raw):
    return hashlib.sha256(raw).hexdigest()


def canonical(value):
    return json.dumps(value, sort_keys=True, separators=(',', ':')).encode()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--plan', type=Path, default=Path(
        'planning/minnesota/regions/inland-winter-belt-v01/winter-belt-source-v04.json'))
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    plan = json.loads(args.plan.read_text())
    assert plan['schema_version'] == 1
    world = json.loads(Path('planning/minnesota/world.json').read_text())
    selected = world['active_local_review_revision']
    assert selected == plan['selected_parent_revision'], 'active lineage changed; review and rebind the proposal'
    assert sha(Path(world[selected]['pack']).read_bytes()) == plan['selected_parent_sha256']
    source_path = Path(plan['source_parent_pack'])
    assert sha(source_path.read_bytes()) == plan['source_parent_sha256'], 'source candidate changed'
    pack = load_pack(source_path)
    original = deepcopy(pack)
    started = time.perf_counter()
    args.output.mkdir(parents=True, exist_ok=False)
    recipes = []
    for specification in plan['art']:
        definition = pack['data']['tilesets'][specification['source']]
        assert sha(canonical(definition)) == specification['source_definition_sha256']
        for path, expected in specification['source_runtime_sha256'].items():
            assert sha(bytes(pack['runtime_files'][path])) == expected, 'native source art changed'
        recipes.append(apply_alias(pack, specification))
    for record in plan['maps']:
        module = pack['data']['maps'][record['id']]
        assert sha(canonical(module)) == record['native_module_sha256'], 'complete module preimage changed'
        assert module['blocks'] == [int(word, 16) for row in record['rows'] for word in row.split()]
        module['attributes']['tileset_name'] = record['target_tileset']
        unchanged = deepcopy(module)
        unchanged['attributes']['tileset_name'] = record['source_tileset']
        assert unchanged == original['data']['maps'][record['id']]
    art = SourceArt(pack)
    npc_pack = npc_palette_pack(pack, 'day')
    images = {}
    for record in plan['maps']:
        name = record['id']
        module = pack['data']['maps'][name]
        attributes = module['attributes']
        image = actors(npc_pack, art.render(attributes['tileset_name'], attributes['width'],
                                          attributes['height'], module['blocks']), module)
        images[name] = image
        save_render(image, args.output / (name + '-original.png'))
        if name == 'MnBemidji':
            save_render(image.resize((1536, 1536), Image.Resampling.NEAREST),
                        args.output / (name + '-2x.png'))
    for prefix, width, height in [('MnR09', 64, 36), ('MnR17', 32, 56)]:
        whole = Image.new('RGB', (width * 32, height * 32))
        for name, image in images.items():
            match = re.fullmatch(prefix + r'C(\d{2})R(\d{2})', name)
            if match:
                column, row = map(int, match.groups())
                whole.paste(image, (column * 1024, row * 768))
        save_render(whole, args.output / (prefix + '-whole-original.png'))
        save_render(whole.resize((whole.width // 2, whole.height // 2), Image.Resampling.NEAREST),
                    args.output / (prefix + '-whole-overview.png'))
    changed = [name for name in pack['data']['maps']
               if pack['data']['maps'][name] != original['data']['maps'][name]]
    assert sorted(changed) == sorted(record['id'] for record in plan['maps'])
    assert all(pack['data']['maps'][name] == module for name, module in original['data']['maps'].items()
               if name not in changed)
    receipt = {'status': 'SOURCE_PROPOSAL_REQUIRES_VISUAL_REVIEW',
               'plan': str(args.plan), 'plan_sha256': sha(args.plan.read_bytes()),
               'source_candidate': str(source_path), 'source_candidate_sha256': plan['source_parent_sha256'],
               'selected_parent_revision': selected, 'map_count': len(changed),
               'compositions': recipes, 'all_original_map_rows_and_collision_exact': True,
               'all_other_maps_complete_exact': True, 'all_modules_except_tileset_exact': True,
               'no_pack_serialized': True, 'new_ice_quadrants': 0,
               'world_unchanged': json.loads(Path('planning/minnesota/world.json').read_text()) == world,
               'elapsed_after_load_seconds': time.perf_counter() - started,
               'scope': 'Full native source-art renders; not a runtime pack, earned progression, seasonal registration or GPU acceptance.'}
    (args.output / 'receipt.json').write_text(json.dumps(receipt, indent=2) + '\n')
    print(json.dumps(receipt))


if __name__ == '__main__':
    main()
