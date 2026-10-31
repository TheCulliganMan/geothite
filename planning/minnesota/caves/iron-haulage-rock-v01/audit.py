#!/usr/bin/env python3
"""Read-only full-pack preservation and native pixel audit of the haulage child."""
import argparse
import base64
from copy import deepcopy
import hashlib
import json
from pathlib import Path
import re
import sys

import cbor2
from PIL import Image, ImageChops, ImageDraw

ROOT = Path(__file__).resolve().parents[4]
sys.path.insert(0, str(ROOT / 'planning/minnesota/tools'))
from render_reference import SourceArt, npc_palette_pack
from review_passage_native import actors


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--pack', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    args.output.mkdir(parents=True, exist_ok=False)
    source = Path(__file__).parent
    contract = json.loads((source / 'contract.json').read_text())
    parent = ROOT / contract['parent_pack']
    assert sha(parent) == contract['parent_pack_sha256']
    # Both files have already passed the strict Rust writer/readback. Bind the
    # exact bytes here, then independently compare every decoded field.
    old, new = [cbor2.loads(p.read_bytes()[22:]) for p in [parent, args.pack]]
    expected = deepcopy(old)
    expected['identity'] = new['identity']
    room, alias = 'MnIronHaulage', 'mn_iron_haulage_rock'
    before, after = [p['data']['maps'][room] for p in [old, new]]
    assert hashlib.sha256(json.dumps(before, sort_keys=True, separators=(',', ':'), ensure_ascii=False).encode()).hexdigest() == contract['native_module_sha256']
    for module, name in [(before, 'before.json'), (after, 'after.json')]:
        assert module['blocks'] == [int(v, 16) for row in json.loads((source / name).read_text())['rows'] for v in row.split()]
    expected['data']['maps'][room]['blocks'] = after['blocks']
    for key, value in dict(tileset_name=alias, border_block=46, time_of_day='nite',
                           palette='PALETTE_NITE', fishing_group='FISHGROUP_MN_LAKE').items():
        expected['data']['maps'][room]['attributes'][key] = value
        expected['data']['map_attributes'][room][key] = value
    label = before['attributes']['blocks_label']
    expected['data']['map_blocks'][label] = new['data']['map_blocks'][label]
    assert base64.b64decode(new['data']['map_blocks'][label]) == bytes(after['blocks'])
    expected['data']['tilesets'][alias] = new['data']['tilesets'][alias]
    files = {f'data/tilesets/{alias}.json', f'data/tilesets/{alias}_metatiles.bin',
             f'data/tilesets/{alias}_palette_map.json', f'gfx/tilesets/{alias}.2bpp',
             f'gfx/tilesets/{alias}.png'}
    assert set(new['runtime_files']) - set(old['runtime_files']) == files
    for path in files:
        expected['runtime_files'][path] = new['runtime_files'][path]
    expected['report']['manifests'].append('minnesota-iron-haulage-native-rock-v1-expanded-city-art')
    assert expected == new, 'undeclared native content changed'
    assert json.loads(bytes(new['runtime_files'][f'data/tilesets/{alias}.json'])) == new['data']['tilesets'][alias]['collision']
    art = SourceArt(new)
    native, private = art.blocks('cave', 'nite'), art.blocks(alias, 'nite')
    for i in range(64):
        assert ImageChops.difference(private[i], native[2 if i == 13 else i]).getbbox() is None
    assert ImageChops.difference(private[64], art.blocks('underground', 'nite')[0x2b]).getbbox() is None
    grate = art.blocks('facility', 'nite')[0x16].crop((16, 0, 32, 32))
    wanted = Image.new('RGB', (32, 32))
    wanted.paste(grate, (0, 0)); wanted.paste(grate, (16, 0))
    assert ImageChops.difference(private[65], wanted).getbbox() is None
    for destination, original in enumerate([0, 3, 2, 1]):
        a = (destination % 2 * 16, destination // 2 * 16)
        b = (original % 2 * 16, original // 2 * 16)
        assert ImageChops.difference(private[66].crop((*a, a[0] + 16, a[1] + 16)), native[11].crop((*b, b[0] + 16, b[1] + 16))).getbbox() is None
    definition = new['data']['tilesets'][alias]
    assert definition['collision']['42'] == ['FLOOR', 'LADDER', 'FLOOR', 'FLOOR']
    assert definition['collision']['0d'] == ['FLOOR'] * 4
    roster = json.loads((ROOT / 'planning/minnesota/pokemon/roster.json').read_text())
    curation = json.loads((ROOT / 'planning/minnesota/pokemon/roster-curation.json').read_text())
    retired = {f['id'] for f in curation['retired_families']}
    normalize = lambda s: re.sub('[^A-Z0-9]', '', s.upper())
    allowed = {normalize(m['name']) for f in roster['families'] if f['id'] not in retired for m in f['members']}
    parties = {}
    for trainer in ['MN_LODE_ARC_WEIGH', 'MN_LODE_ARC_CONVEYOR']:
        party = new['data']['trainers']['trainers'][trainer]['party']
        assert all(normalize(p['species']) in allowed for p in party)
        parties[trainer] = [{'species': p['species'], 'level': p['level']} for p in party]
    for species, evolutions in new['data']['evolutions'].items():
        if normalize(species) in allowed:
            assert all(normalize(e['species']) in allowed for e in evolutions)
    for pack, module, group, name in [(old, before, 'day', 'before'), (new, after, 'nite', 'after')]:
        a = module['attributes']
        scene = actors(npc_palette_pack(pack, group), SourceArt(pack).render(a['tileset_name'], a['width'], a['height'], module['blocks'], group), module)
        scene.save(args.output / (name + '-original.png'))
        scene.resize((scene.width * 2, scene.height * 2), Image.Resampling.NEAREST).save(args.output / (name + '-2x.png'))
    left = Image.open(args.output / 'before-original.png').convert('RGB')
    right = Image.open(args.output / 'after-original.png').convert('RGB')
    sheet = Image.new('RGB', (left.width + right.width + 24, left.height + 30), '#14171c')
    draw = ImageDraw.Draw(sheet)
    draw.text((8, 7), 'Previous warehouse treatment', fill='white')
    draw.text((left.width + 16, 7), 'Native rock galleries', fill='white')
    sheet.paste(left, (8, 24)); sheet.paste(right, (left.width + 16, 24))
    sheet.save(args.output / 'whole-before-after.png')
    receipt = dict(status='PASS', parent_pack=str(parent.relative_to(ROOT)), parent_sha256=sha(parent),
                   pack=str(args.pack), pack_sha256=sha(args.pack), content_identity=new['identity']['content_hash'],
                   unchanged_other_maps=len(new['data']['maps']) - 1,
                   original_programs_warps_actors_and_controls_exact=True,
                   all_existing_runtime_files_and_audio_exact=True, new_runtime_files=sorted(files),
                   all_native_alias_pixels_and_collision_recipes_exact=True, active_regional_species=len(allowed),
                   guard_parties=parties, active_evolution_closure=True)
    (args.output / 'receipt.json').write_text(json.dumps(receipt, indent=2) + '\n')
    print('PASS full preservation, native art and regional guard parties')


if __name__ == '__main__':
    main()
