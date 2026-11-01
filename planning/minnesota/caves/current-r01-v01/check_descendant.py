#!/usr/bin/env python3
"""Read-only R01 preservation proof across a later region composition."""
import argparse
import hashlib
import json
from pathlib import Path
import sys

ROOT = Path(__file__).resolve().parents[4]
sys.path.insert(0, str(ROOT / 'planning/minnesota/tools'))
from render_archive import save_render
from render_reference import SourceArt, load_pack, npc_palette_pack
from review_north_shore import oriented_sprite

ROOM = 'MnR01Cave1F'


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def scene(pack):
    module = pack['data']['maps'][ROOM]
    a = module['attributes']
    image = SourceArt(pack).render('cave', a['width'], a['height'], module['blocks'], 'nite').convert('RGBA')
    sprites = npc_palette_pack(pack, 'nite')
    for actor in module['objects']:
        face = {'UP': 'north', 'RIGHT': 'east', 'LEFT': 'west'}.get(actor['spritemovedata'].rsplit('_', 1)[-1], 'south')
        image.alpha_composite(oriented_sprite(sprites, actor['sprite'], face), (actor['x'] * 16, actor['y'] * 16))
    return image


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--pack', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    baseline = ROOT / 'content-packs/minnesota-r01-limestone-hollows-v01.crystalpack'
    assert digest(baseline) == 'c7e3e0297533789a3c9e32f29db20cf69d73972b2a6f744192bc183fa23c74e1'
    assert not args.output.exists(), 'Preserve every review epoch.'
    before, actual = load_pack(baseline), load_pack(args.pack)
    old, new = [pack['data']['maps'][ROOM] for pack in [before, actual]]
    assert old == new, 'R01 native geometry, programs, actors, dialogue or events changed.'
    label = old['attributes']['blocks_label']
    assert before['data']['map_blocks'][label] == actual['data']['map_blocks'][label]
    assert before['data']['map_attributes'][ROOM] == actual['data']['map_attributes'][ROOM]
    assert before['runtime_files'] == actual['runtime_files'], 'Native art/runtime files changed.'
    assert before['data']['tilesets'] == actual['data']['tilesets']
    assert before['compiled_audio'] == actual['compiled_audio']
    assert before['audio_manifest'] == actual['audio_manifest']
    for field in ['species', 'moves', 'items']:
        if field in before['data']:
            assert before['data'][field] == actual['data'][field], field
    old_image, new_image = scene(before), scene(actual)
    assert old_image.tobytes() == new_image.tobytes()
    args.output.mkdir(parents=True)
    save_render(old_image, args.output / 'accepted-R01-whole.png')
    save_render(new_image, args.output / 'MnR01Cave1F-whole.png')
    receipt = dict(status='PASS', accepted_R01_pack_sha256=digest(baseline),
                   descendant_pack=str(args.pack), descendant_pack_sha256=digest(args.pack),
                   descendant_identity=actual['identity']['content_hash'],
                   complete_R01_native_module_exact=True, R01_payload_and_attribute_mirrors_exact=True,
                   native_runtime_art_and_tilesets_exact=True, audio_programs_and_manifest_exact=True,
                   whole_room_source_pixels=old_image.width * old_image.height,
                   differing_source_pixels=0,
                   scope='Read-only descendant preservation/source evidence. Original R01 controller and GPU logs keep their c7e3e029 pack binding; later R05/League changes are independently validated by their publication owner.')
    (args.output / 'receipt.json').write_text(json.dumps(receipt, indent=2) + '\n')
    (args.output / 'check_descendant.py.snapshot').write_bytes(Path(__file__).read_bytes())
    print(json.dumps(receipt, indent=2))


if __name__ == '__main__':
    main()
