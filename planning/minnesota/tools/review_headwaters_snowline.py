#!/usr/bin/env python3
"""Review complete authored Bemidji-to-Leech thaw rows using native source art.

Python mutates only a decoded review view. Rust owns game-pack publication.
"""
import argparse
from copy import deepcopy
import hashlib
import json
from pathlib import Path
import time

from PIL import Image
from inland_winter_preview_art import apply_alias
from render_archive import save_render
from review_selection_lineage import require_selected_ancestor
from render_reference import SourceArt, load_pack, npc_palette_pack
from review_passage_native import actors
from tamarack_thaw_art import compose


def digest(raw):
    return hashlib.sha256(raw).hexdigest()


def canonical(value):
    return json.dumps(json.loads(json.dumps(value)), sort_keys=True, separators=(',', ':')).encode()


def compose_review(pack, plan):
    assert pack['identity'] == plan['parent_identity']
    original = deepcopy(pack['data']['maps'])
    for specification in [plan['art'], plan['stair_art']]:
        assert digest(canonical(pack['data']['tilesets'][specification['source']])) == specification['source_definition_sha256']
        for path, expected in specification['source_runtime_sha256'].items():
            assert digest(bytes(pack['runtime_files'][path])) == expected, path
    variants, result = compose(pack, plan['art'])
    assert json.loads(json.dumps(result)) == plan['art_result']
    assert apply_alias(pack, plan['stair_art']) == plan['stair_art_result']
    inherited = plan['inherited_animations']
    assert digest(bytes(pack['runtime_files'][inherited['source']])) == inherited['sha256']
    alias = plan['stair_art']['target']
    pack['runtime_files'][f'data/tilesets/{alias}_native_backgrounds.json'] = json.dumps(inherited['full_stair_backgrounds']).encode()
    quadrants = 0
    for record in plan['maps']:
        name = record['id']
        before = original[name]
        assert digest(canonical(before)) == record['native_module_sha256'], name
        parse = lambda rows: [int(word, 16) for row in rows for word in row.split()]
        assert before['blocks'] == parse(record['before_rows'])
        module = pack['data']['maps'][name]
        module['blocks'] = parse(record['after_rows'])
        module['attributes']['tileset_name'] = record['target_tileset']
        old = pack['data']['tilesets'][record['source_tileset']]['collision']
        new = pack['data']['tilesets'][record['target_tileset']]['collision']
        assert len(module['blocks']) == len(before['blocks'])
        assert all(old[f'{a:02x}'] == new[f'{b:02x}'] for a, b in zip(before['blocks'], module['blocks']))
        quadrants += len(module['blocks']) * 4
        residual = deepcopy(module)
        residual['blocks'] = before['blocks']
        residual['attributes']['tileset_name'] = record['source_tileset']
        assert residual == before
    return original, quadrants


def whole(pack, maps, dimensions=(40, 48)):
    art = SourceArt(pack)
    npc = npc_palette_pack(pack, 'day')
    result = Image.new('RGB', (dimensions[0] * 32, dimensions[1] * 32))
    images = {}
    for name in maps:
        module = pack['data']['maps'][name]
        attr = module['attributes']
        image = actors(npc, art.render(attr['tileset_name'], attr['width'], attr['height'], module['blocks']), module)
        images[name] = image
        result.paste(image, (int(name[-5:-3]) * 1024, int(name[-2:]) * 768))
    return result, images


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--source', type=Path, default=Path('planning/minnesota/regions/headwaters-snowline-v01/source-v01.json'))
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    plan = json.loads(args.source.read_text())
    world_bytes = Path('planning/minnesota/world.json').read_bytes()
    world = json.loads(world_bytes)
    require_selected_ancestor(world, plan['selected_parent_revision'], plan['selected_parent_sha256'])
    parent = Path(plan['parent_pack'])
    assert digest(parent.read_bytes()) == plan['parent_pack_sha256']
    pack = load_pack(parent)
    names = [r['id'] for r in plan['maps']]
    before, _ = whole(pack, names)
    started = time.perf_counter()
    original, quadrants = compose_review(pack, plan)
    after, images = whole(pack, names)
    out = args.output
    out.mkdir(parents=True, exist_ok=False)
    for name, image in images.items():
        save_render(image, out / (name + '-original.png'))
    for stage, image in [('before', before), ('after', after)]:
        save_render(image, out / (stage + '-whole-original.png'))
        save_render(image.resize((640, 768), Image.Resampling.NEAREST), out / (stage + '-whole-overview.png'))
    crops = {'bemidji-stair-arrival': (320, 0, 704, 384),
             'north-lake-gate': (0, 192, 384, 576),
             'snow-portage': (448, 672, 832, 1056),
             'thaw-west-bank': (32, 928, 416, 1312),
             'thaw-east-bank': (896, 928, 1280, 1312),
             'green-leech-road': (448, 1152, 832, 1536)}
    for label, box in crops.items():
        image = after.crop(box)
        save_render(image, out / (label + '-original.png'))
        save_render(image.resize((768, 768), Image.Resampling.NEAREST), out / (label + '-2x.png'))
    gate = plan['composition']['native_cut_gate']
    module = pack['data']['maps'][gate['map']]
    attr = module['attributes']
    pos = gate['metatile'][1] * attr['width'] + gate['metatile'][0]
    assert module['blocks'][pos] == int(gate['block'], 16)
    module['blocks'][pos] = int(gate['replacement'], 16)
    cleared = actors(npc_palette_pack(pack, 'day'), SourceArt(pack).render(attr['tileset_name'], attr['width'], attr['height'], module['blocks']), module)
    save_render(cleared.crop(crops['north-lake-gate']), out / 'north-lake-gate-after-cut-original.png')
    module['blocks'][pos] = int(gate['block'], 16)
    receipt = dict(status='SOURCE_PROPOSAL_REQUIRES_NATIVE_PUBLICATION_AND_REVIEW',
                   source=str(args.source), source_sha256=digest(args.source.read_bytes()),
                   parent_sha256=plan['parent_pack_sha256'], dimensions_metatiles=[40, 48],
                   maps=names, collision_quadrants_exact=quadrants, all_other_map_fields_exact=True,
                   new_ice_quadrants=0, elapsed_after_load_seconds=time.perf_counter() - started,
                   world_unchanged=world_bytes == Path('planning/minnesota/world.json').read_bytes(),
                   scope='Whole-route native source-art composition only. Existing lake gate is a source replacement preview, not gameplay evidence. Publication, clock/weather/grass masks, native GPU, original Cut gate and actual connected walks remain required.')
    (out / 'receipt.json').write_text(json.dumps(receipt, indent=2) + '\n')
    print(json.dumps(receipt), flush=True)


if __name__ == '__main__':
    main()
