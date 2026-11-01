#!/usr/bin/env python3
"""Read-only full-town review of the source-bound Bemidji roof/cliff repair.

--source previews the explicit proposal over its certified parent. Without it,
render the installed candidate and verify its complete authored rows. Always
choose a fresh output directory; original renders remain reusable.
"""
import argparse
import hashlib
import json
from pathlib import Path
import time

from PIL import Image

from render_archive import save_render
from render_reference import SourceArt, load_pack, npc_palette_pack
from review_passage_native import actors


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--pack', type=Path, required=True)
    parser.add_argument('--plan', type=Path, default=Path(
        'planning/minnesota/towns/bemidji-roof-plateau-v01'))
    parser.add_argument('--source', action='store_true')
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    args.output.mkdir(parents=True, exist_ok=False)
    pack_hash = hashlib.sha256(args.pack.read_bytes()).hexdigest()
    pack = load_pack(args.pack)
    started = time.perf_counter()
    contract = json.loads((args.plan / 'contract.json').read_text())
    before = json.loads((args.plan / 'before.json').read_text())
    after = json.loads((args.plan / 'after.json').read_text())
    parse = lambda source: [int(word, 16) for row in source['rows'] for word in row.split()]
    module = pack['data']['maps']['MnBemidji']
    assert module['attributes']['tileset_name'] == contract['tileset']
    assert [module['attributes']['width'], module['attributes']['height']] == [24, 24]
    art = SourceArt(pack)
    blocks = parse(after)
    if args.source:
        assert pack_hash == contract['parent_pack_sha256'], 'certified source parent required'
        assert module['blocks'] == parse(before), 'complete source preimage changed'
        native = list(art.blocks(contract['tileset']))
        for recipe in contract['native_roof_recipes']:
            assert int(recipe['id'], 16) == len(native)
            native.append(art.blocks(recipe['source_tileset'])[int(recipe['source_block'], 16)])
        art.cache[contract['tileset'], 'day', None] = native
    else:
        assert module['blocks'] == blocks, 'installed terrain differs from authored repair'
    image = actors(npc_palette_pack(pack, 'day'),
                   art.render(contract['tileset'], 24, 24, blocks), module)
    save_render(image, args.output / 'whole-original.png')
    save_render(image.resize((1536, 1536), Image.Resampling.NEAREST),
                args.output / 'whole-2x.png')
    for name, box in [('tower', (96, 480, 256, 640)),
                      ('plateau', (32, 64, 384, 384))]:
        crop = image.crop(box)
        save_render(crop, args.output / (name + '-original.png'))
        save_render(crop.resize((crop.width * 2, crop.height * 2), Image.Resampling.NEAREST),
                    args.output / (name + '-2x.png'))
    receipt = {'status': 'SOURCE_PREVIEW' if args.source else 'INSTALLED_SOURCE_MATCH',
               'pack': str(args.pack), 'pack_sha256': pack_hash,
               'plan': str(args.plan), 'elapsed_after_load_seconds': time.perf_counter() - started,
               'scope': 'Native day art and exact terrain rows; gameplay and GPU review are separate.'}
    (args.output / 'receipt.json').write_text(json.dumps(receipt, indent=2) + '\n')
    print(json.dumps(receipt))


if __name__ == '__main__':
    main()
