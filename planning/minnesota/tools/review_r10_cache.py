#!/usr/bin/env python3
"""Read-only whole-route R10 previews with native glyph recipes and warm reload.

This renders source art and checks the existing ordinary source contracts.
It does not certify optional cache grass, native publication or gameplay; those
must pass the Rust publisher and production input on the resulting native pack.
"""
import argparse
import copy
import hashlib
import json
from pathlib import Path
import time

from PIL import Image

from render_reference import SourceArt, layout_blocks, load_pack
from review_league_approach import inspect_league
from review_passage_native import actors


def render(pack, art, layout_path, dossier_path, output):
    started = time.perf_counter()
    raw_layout, raw_dossier = layout_path.read_bytes(), dossier_path.read_bytes()
    layout, dossier = json.loads(raw_layout), json.loads(raw_dossier)
    assert layout['id'] == dossier['id'] == 'MN_R10'
    assert layout['dimensions_metatiles'] == [48, 20]
    assert layout['compiled_tileset'] == 'mn_r10_rockstairs'
    blocks = layout_blocks(layout)
    preview = dict(pack, data=dict(pack['data'], tilesets=dict(pack['data']['tilesets'])))
    definition = copy.deepcopy(pack['data']['tilesets'][layout['compiled_tileset']])
    for recipe in layout.get('derived_blocks', []):
        kinds = recipe['collision']
        kinds = kinds.split('/') if isinstance(kinds, str) else kinds
        assert len(kinds) == 4
        if recipe['id'] in definition['collision']:
            assert definition['collision'][recipe['id']] == kinds
        else:
            definition['collision'][recipe['id']] = kinds
    preview['data']['tilesets'][layout['compiled_tileset']] = definition
    report, _, _ = inspect_league(dossier, layout, preview, blocks)
    assert not report['errors'], report['errors']
    terrain = art.render(layout['tileset'], 48, 20, blocks,
                         derived=layout.get('derived_blocks'))
    image = terrain.copy()
    for column in range(2):
        module = pack['data']['maps'][f'MnR10C{column:02}R00']
        x = column * 1024
        chunk = image.crop((x, 0, min(1536, x + 1024), 640))
        image.paste(actors(pack, chunk, module), (x, 0))
    digest = hashlib.sha256(raw_layout + raw_dossier).hexdigest()
    destination = output / (digest[:12] + '-' + str(time.time_ns()))
    destination.mkdir(parents=True, exist_ok=False)
    terrain.save(destination / 'terrain-original.png')
    image.save(destination / 'whole-route-original.png')
    for name, box in [('cache-mine', (10, 2, 23, 11)),
                      ('western-stair', (3, 7, 9, 13)), ('whole-west', (0, 0, 24, 14))]:
        crop = image.crop(tuple(value * 32 for value in box))
        crop.save(destination / (name + '-original.png'))
        crop.resize((crop.width * 3, crop.height * 3), Image.Resampling.NEAREST).save(
            destination / (name + '-3x.png'))
    record = dict(layout=str(layout_path), dossier=str(dossier_path),
                  layout_sha256=hashlib.sha256(raw_layout).hexdigest(),
                  dossier_sha256=hashlib.sha256(raw_dossier).hexdigest(),
                  native_source_identity=pack['identity'],
                  output=str(destination), render_seconds=round(time.perf_counter()-started, 3),
                  source_contracts=report,
                  scope='Source preview only. Optional cache grass, native pixels and gameplay '
                        'must be checked by the Rust publisher and production controller.')
    (destination / 'source.json').write_text(json.dumps(record, indent=2) + '\n')
    (output / 'latest.json').write_text(json.dumps(record, indent=2) + '\n')
    return record


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for field in ['pack', 'layout', 'dossier', 'output']:
        parser.add_argument('--' + field, type=Path, required=True)
    parser.add_argument('--watch', action='store_true')
    args = parser.parse_args()
    pack = load_pack(args.pack)
    art = SourceArt(pack)
    previous = None
    while True:
        digest = hashlib.sha256(args.layout.read_bytes() + args.dossier.read_bytes()).hexdigest()
        if digest != previous:
            try:
                print(json.dumps(render(pack, art, args.layout, args.dossier, args.output)), flush=True)
            except (AssertionError, KeyError, ValueError) as error:
                if not args.watch:
                    raise
                args.output.mkdir(parents=True, exist_ok=True)
                failure = dict(status='SOURCE_ERROR', source_digest=digest, error=str(error))
                (args.output / f'source-error-{time.time_ns()}.json').write_text(
                    json.dumps(failure, indent=2) + '\n')
                print(json.dumps(failure), flush=True)
            previous = digest
        if not args.watch:
            break
        time.sleep(0.25)


if __name__ == '__main__':
    main()
