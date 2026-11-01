#!/usr/bin/env python3
"""Encode an already hand-authored editor raster as native explicit row runs.

This does not choose terrain, alter source pixels or publish packs. Preserve the
input intent separately, then use the existing renderer to verify exact equality.
"""
import argparse
import hashlib
import json
from pathlib import Path

from render_reference import layout_blocks


def bake(layout):
    before = layout_blocks(layout)
    w, h = layout['dimensions_metatiles']
    layers = []
    for y in range(h):
        x = 0
        while x < w:
            block = before[y*w+x]
            end = x+1
            while end < w and before[y*w+end] == block:
                end += 1
            if block != 5:
                layers.append(dict(name=f'Explicit authored row {y:03} · {x:03}..{end-1:03}',
                    kind='block_rect', bounds=[x, y, end, y+1], block=f'{block:02x}'))
            x = end
    result = dict(schema_version=2, id=layout['id'], tileset=layout['tileset'],
        compiled_tileset=layout['compiled_tileset'], dimensions_metatiles=[w, h],
        shore_fitting='explicit_source_edges', reject_cliff_cuts=True,
        source_pixels_modified=False, layers=layers)
    assert layout_blocks(result) == before
    return result


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--input', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    source = json.loads(args.input.read_text())
    result = bake(source)
    args.output.write_text(json.dumps(result, indent=2)+'\n')
    print(json.dumps(dict(id=result['id'], blocks_sha256=hashlib.sha256(bytes(layout_blocks(result))).hexdigest(),
                          exact_raster_preserved=True, explicit_runs=len(result['layers']))))


if __name__ == '__main__':
    main()
