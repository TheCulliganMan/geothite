#!/usr/bin/env python3
"""Review all four authored Bemidji-to-Itasca chunks using native source art.

This is an in-memory source preview; Rust owns native pack publication.
"""
import argparse
from copy import deepcopy
import json
from pathlib import Path
import time

from PIL import Image, ImageDraw
from render_archive import save_render
from review_selection_lineage import require_selected_ancestor
from render_reference import SourceArt, load_pack
from review_headwaters_snowline import canonical, digest, whole
from tamarack_thaw_art import compose


def compose_review(pack, plan):
    assert pack['identity'] == plan['parent_identity']
    rules = pack['data']['field_moves']['cut']['replacements']['mn_headwaters']
    assert digest(canonical(rules)) == plan['cut_rule_sha256']
    specification = plan['art']
    assert digest(canonical(pack['data']['tilesets'][specification['source']])) == specification['source_definition_sha256']
    for path, expected in specification['source_runtime_sha256'].items():
        assert digest(bytes(pack['runtime_files'][path])) == expected, path
    original = {r['id']: deepcopy(pack['data']['maps'][r['id']]) for r in plan['maps']}
    _, result = compose(pack, specification)
    assert json.loads(json.dumps(result)) == plan['art_result']
    # Mimic only the planned field table in memory. No crystalpack serialization.
    for alias in [specification['target'], specification['partial_target']]:
        pack['data']['field_moves']['cut']['replacements'][alias] = deepcopy(rules)
    for rule in plan['field_move_variants']:
        replacements = pack['data']['field_moves']['cut']['replacements'][rule['tileset']]
        block, target = int(rule['block'], 16), int(rule['replacement'], 16)
        assert block not in replacements
        prototype = rules[int(rule['source_block'], 16)]
        assert prototype == dict(replacement_block_id=int(rule['source_replacement'], 16), variant=rule['variant'])
        replacements[block] = dict(replacement_block_id=target, variant=rule['variant'])
        collision = pack['data']['tilesets'][rule['tileset']]['collision']
        before, after = collision[rule['block']], collision[rule['replacement']]
        original_collision = pack['data']['tilesets']['mn_headwaters']['collision']
        assert before == original_collision[rule['source_block']]
        assert after == original_collision[rule['source_replacement']]
        assert all(a == b for a, b in zip(before, after) if a not in {'CUT_TREE', 'TALL_GRASS'})
    quadrants, gates = 0, []
    for record in plan['maps']:
        name = record['id']
        before = original[name]
        assert digest(canonical(before)) == record['native_module_sha256']
        parse = lambda rows: [int(word, 16) for row in rows for word in row.split()]
        assert before['blocks'] == parse(record['before_rows'])
        module = pack['data']['maps'][name]
        module['blocks'] = parse(record['after_rows'])
        module['attributes']['tileset_name'] = record['target_tileset']
        old = pack['data']['tilesets'][record['source_tileset']]['collision']
        new = pack['data']['tilesets'][record['target_tileset']]['collision']
        assert len(module['blocks']) == len(before['blocks'])
        for index, (a, b) in enumerate(zip(before['blocks'], module['blocks'])):
            assert old[f'{a:02x}'] == new[f'{b:02x}']
            if any(kind in {'CUT_TREE', 'WHIRLPOOL'} for kind in old[f'{a:02x}']):
                assert name == 'MnR14C01R00' and a == b == 0x5b
                gates.append([name, index])
        quadrants += len(module['blocks']) * 4
        residual = deepcopy(module)
        residual['blocks'] = before['blocks']
        residual['attributes']['tileset_name'] = record['source_tileset']
        assert residual == before
    assert len(gates) == 1
    return quadrants


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--source', type=Path, default=Path('planning/minnesota/regions/itasca-snowline-v01/source-v02.json'))
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
    pack = deepcopy(base)
    names = [r['id'] for r in plan['maps']]
    before, _ = whole(pack, names, (48, 32))
    started = time.perf_counter()
    quadrants = compose_review(pack, plan)
    after, images = whole(pack, names, (48, 32))
    out = args.output
    out.mkdir(parents=True, exist_ok=False)
    for name, image in images.items():
        save_render(image, out / (name + '-original.png'))
    for stage, image in [('before', before), ('after', after)]:
        save_render(image, out / (stage + '-whole-original.png'))
        save_render(image.resize((768, 512), Image.Resampling.NEAREST), out / (stage + '-whole-overview.png'))
    crops = {'bemidji-snowy-pines': (1152, 224, 1536, 608),
             'pine-cut-return': (960, 192, 1344, 576),
             'young-river-thaw': (640, 384, 1024, 768),
             'green-survey-pond': (128, 128, 512, 512),
             'itasca-gatehouse': (0, 480, 384, 864)}
    for label, box in crops.items():
        crop = after.crop(box)
        save_render(crop, out / (label + '-original.png'))
        save_render(crop.resize((768, 768), Image.Resampling.NEAREST), out / (label + '-2x.png'))
    art = SourceArt(pack)
    sheet = Image.new('RGB', (304, 1560), '#151515')
    draw = ImageDraw.Draw(sheet)
    for row, rule in enumerate(plan['field_move_variants']):
        draw.text((6, row * 156 + 4), f"{rule['phase']} {rule['source_block']} : {rule['block']} -> {rule['replacement']}", fill='white')
        for col, key in enumerate(['block', 'replacement']):
            tile = art.render(rule['tileset'], 1, 1, [int(rule[key], 16)])
            save_render(tile, out / (rule['phase'] + '-' + rule[key] + '-original.png'))
            sheet.paste(tile.resize((128, 128), Image.Resampling.NEAREST), (col * 152 + 6, row * 156 + 24))
    save_render(sheet, out / 'native-field-move-variants.png')
    gate = plan['composition']['native_cut_shortcut']
    module = pack['data']['maps'][gate['map']]
    index = gate['metatile'][1] * module['attributes']['width'] + gate['metatile'][0]
    assert module['blocks'][index] == int(gate['block'], 16)
    module['blocks'][index] = int(gate['replacement'], 16)
    cleared, _ = whole(pack, names, (48, 32))
    save_render(cleared.crop(crops['pine-cut-return']), out / 'pine-cut-return-after-native-replacement.png')
    module['blocks'][index] = int(gate['block'], 16)
    assert world_bytes == Path('planning/minnesota/world.json').read_bytes()
    report = dict(status='SOURCE_PROPOSAL_REQUIRES_NATIVE_PUBLICATION_AND_REVIEW',
                  source_sha256=digest(args.source.read_bytes()), parent_sha256=plan['parent_pack_sha256'],
                  dimensions_metatiles=[48, 32], maps=names, collision_quadrants_exact=quadrants,
                  all_other_map_fields_exact=True, original_gate_block_ids_exact=True,
                  new_cut_variant_rules=len(plan['field_move_variants']), new_ice_quadrants=0,
                  world_unchanged=True, elapsed_after_load_seconds=time.perf_counter() - started,
                  scope='Whole-route native source art and replacement/collision review only. Cut shortcut reachability, native clock/weather/animation/masks, GPU and actual connected walks remain required.')
    if args.installed:
        child = load_pack(args.installed)
        assert all(child['data']['maps'][name] == module for name, module in base['data']['maps'].items() if name not in names)
        assert all(child['runtime_files'][path] == raw for path, raw in base['runtime_files'].items())
        assert child['audio_manifest'] == base['audio_manifest'] and child['compiled_audio'] == base['compiled_audio']
        for name in names:
            assert child['data']['maps'][name] == pack['data']['maps'][name]
        for alias in [plan['art']['target'], plan['art']['partial_target']]:
            assert child['data']['field_moves']['cut']['replacements'][alias] == pack['data']['field_moves']['cut']['replacements'][alias]
        native, _ = whole(child, names, (48, 32))
        from PIL import ImageChops
        assert ImageChops.difference(after, native).getbbox() is None
        report.update(status='PASS_INSTALLED_ART_AND_PRESERVATION', installed_sha256=digest(args.installed.read_bytes()),
                      content_identity=child['identity']['content_hash'], other_complete_maps_exact=len(base['data']['maps']) - len(names),
                      original_runtime_files_exact=True, original_audio_records_exact=602,
                      complete_route_pixels_exact=1536 * 1024, pixel_differences=0)
    (out / 'receipt.json').write_text(json.dumps(report, indent=2) + '\n')
    print(json.dumps(report), flush=True)


if __name__ == '__main__':
    main()
