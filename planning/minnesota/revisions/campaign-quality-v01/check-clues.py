#!/usr/bin/env python3
"""Verify five native route clues against the authored 32px masks and their own parent pack."""
import argparse
import base64
import io
import json
import sys
from pathlib import Path

from PIL import Image

MINNESOTA = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(MINNESOTA / 'tools'))
from render_reference import SourceArt, load_pack

SOURCE = MINNESOTA / 'discoveries/route-07-11-clues-v01'


def require(condition, message):
    if not condition:
        raise ValueError(message)


def pickups(pack):
    result = {}
    for name, room in pack['data']['maps'].items():
        if not name.startswith('Mn'):
            continue
        for event in room['events']['bg_events']:
            if event['event_type'] != 'BGEVENT_ITEM':
                continue
            script = event['script']
            [pickup] = [p for p in room['script_field_pickups'] if p['source_script'] == script and p['command'] == 'hiddenitem']
            result[name, (event['x'], event['y']), script] = (pickup['item_id'], pickup['quantity'], pickup['event_flag'])
    return result


def address(logical, palettes, size):
    return size // 2 + (logical & 127) if palettes[logical] & 8 else logical


def levels(bits):
    return [(bits[2 * y] >> (7-x) & 1) + 2 * (bits[2*y+1] >> (7-x) & 1)
            for y in range(8) for x in range(8)]


def check_bank(parent, child, cache, before_art, after_art):
    name = cache['map']
    old, new = (p['data']['maps'][name] for p in (parent, child))
    attrs = old['attributes']
    bank = attrs['tileset_name']
    require(new['attributes']['tileset_name'] == bank, name + ' changed tileset')
    x, y = cache['tile']
    at = (y//2) * attrs['width'] + x//2
    original, placed = old['blocks'][at], new['blocks'][at]
    expected_blocks = list(old['blocks'])
    expected_blocks[at] = placed
    require(list(new['blocks']) == expected_blocks, name + ' changed outside cache')
    mirror = child['data']['map_blocks'][attrs['blocks_label']]
    require(base64.b64decode(mirror, validate=True) == bytes(expected_blocks), name + ' stale block mirror')
    [design] = json.loads((SOURCE / f"r{cache['route']:02d}-art.json").read_text())
    require(design['tileset'] == bank and design['foreground'] is None, 'unexpected source art')
    bs = [p['data']['tilesets'][bank] for p in (parent, child)]
    files = [p['runtime_files'] for p in (parent, child)]
    layout = [bytes(f[f'data/tilesets/{bank}_metatiles.bin']) for f in files]
    require(layout[1].startswith(layout[0]), bank + ' inherited metatiles changed')
    missing = original == cache['from_block']
    require((placed == len(layout[0])//16 and len(layout[1]) == len(layout[0])+16) if missing else
            (placed == original and layout[1] == layout[0]), bank + ' unexpected block allocation')
    collision = dict(bs[0]['collision'])
    collision[f'{placed:02x}'] = collision[f"{cache['from_block']:02x}"]
    require(bs[1]['collision'] == collision and
            json.loads(bytes(files[1][f'data/tilesets/{bank}.json'])) == collision, bank + ' collision')
    before, after = (bytes(f[f'gfx/tilesets/{bank}.2bpp']) for f in files)
    count = len(before)//16
    palettes = list(bs[0]['palette_map'])
    current = bs[1]['palette_map']
    require(len(before) == len(after), bank + ' changed graphics bank size')
    expected_backgrounds, expected_foregrounds = [json.loads(bytes(files[0].get(
        f'data/tilesets/{bank}_{suffix}.json', b'{}'))) for suffix in ('native_backgrounds','foregrounds')]
    original_backgrounds = dict(expected_backgrounds)
    original_foregrounds = dict(expected_foregrounds)
    applied = set()
    target_glyphs = layout[1][16*placed:16*placed+16]
    donor = design['background']
    original_glyphs = layout[0][16*donor:16*donor+16]
    sequence = cache['first_glyph']
    sheet = Image.open(io.BytesIO(bytes(files[1][f'gfx/tilesets/{bank}.png']))).convert('L')
    for i, (src, dst) in enumerate(zip(original_glyphs, target_glyphs, strict=True)):
        dx, dy = i%4*8, i//4*8
        mask = ''.join(row[dx:dx+8] for row in design['rows'][dy:dy+8])
        if mask == '.' * 64:
            require(src == dst, bank + ' background glyph was replaced')
            continue
        require(dst == sequence, bank + ' changed logical glyph address')
        sequence += 1
        physical_source = address(src, bs[0]['palette_map'], count)
        physical_destination = address(dst, current, count)
        prior = levels(before[physical_source*16:physical_source*16+16])
        expected = [base if ink == '.' else int(ink) for base, ink in zip(prior, mask, strict=True)]
        actual = levels(after[physical_destination*16:physical_destination*16+16])
        require(expected == actual, f'{bank} glyph {dst:02x} differs from the original ink')
        sx, sy = 8*(physical_destination % (sheet.width//8)), 8*(physical_destination//(sheet.width//8))
        png = list(sheet.crop((sx, sy, sx+8, sy+8)).get_flattened_data())
        require([255 - 85*px for px in expected] == png, bank + ' PNG and 2bpp differ')
        applied.add(physical_destination)
        palettes[dst] = 8 | (palettes[src] & 7)
        for expected_table, source_table in [(expected_backgrounds,original_backgrounds),(expected_foregrounds,original_foregrounds)]:
            if str(src) in source_table:
                expected_table[str(dst)] = source_table[str(src)]
            else:
                expected_table.pop(str(dst), None)
    for i in range(count):
        if not missing or i not in applied:
            require(before[i*16:i*16+16] == after[i*16:i*16+16], f'{bank} changed existing tile {i:02x}')
    require(palettes == current and json.loads(bytes(files[1][f'data/tilesets/{bank}_palette_map.json'])) == current,
            bank + ' incompatible color or VRAM mappings')
    for suffix, table in [('native_backgrounds',expected_backgrounds),('foregrounds',expected_foregrounds)]:
        require(json.loads(bytes(files[1].get(f'data/tilesets/{bank}_{suffix}.json', b'{}'))) == table,
                bank + ' incompatible native palette overrides')
    for target in (f'data/tilesets/{bank}_animations.json', f'gfx/tilesets/{bank}.pal'):
        require(files[0].get(target) == files[1].get(target), bank + ' changed animation or native palette')
    # Outside every ink stroke, the restored clue must render as its donor, even at night.
    for time in ('morn', 'day', 'nite'):
        ground = before_art.blocks(bank,time)[donor]
        final = after_art.blocks(bank,time)[placed]
        require(all(ground.getpixel((gx,gy)) == final.getpixel((gx,gy))
                    for gy, row in enumerate(design['rows']) for gx, ink in enumerate(row) if ink == '.'),
                bank + ' introduced an edge around the clue during ' + time)
    return missing, len(applied)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('parent', type=Path)
    parser.add_argument('child', type=Path)
    args = parser.parse_args()
    parent, child = load_pack(args.parent), load_pack(args.child)
    a, b = SourceArt(parent), SourceArt(child)
    source = json.loads((SOURCE/'caches.json').read_text())
    inventory = pickups(child)
    require(inventory == pickups(parent), 'a Minnesota hidden item, receipt, script, coordinate or flag changed')
    stamps = []
    for cache in source:
        require(inventory.get((cache['map'],tuple(cache['tile']),cache['script'])) ==
                (cache['item'],1,cache['flag']), 'authored reward or receipt differs')
        stamps.append(check_bank(parent,child,cache,a,b))
    print(f'{len(inventory)} Minnesota item receipts preserved. All {len(stamps)} route clues match their native art. '
          f'{sum(s[0] for s in stamps)} appended, {sum(s[1] for s in stamps)} painted glyphs verified in three light conditions.')


if __name__ == '__main__':
    main()
