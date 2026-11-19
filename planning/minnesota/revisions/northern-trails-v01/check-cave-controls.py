#!/usr/bin/env python3
"""Compare the two compiled cave interiors, their native art and their reachable states."""
import argparse
import base64
import io
import json
import sys
from collections import deque
from pathlib import Path

from PIL import Image, ImageDraw, ImageFont

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE.parents[1] / 'tools'))
from render_reference import SourceArt, load_pack

BASIN = 'MnInnerBasin'
GROTTO = 'MnR12Cave1F'
CONTROLS = {
    (9, 10): ('MnBasinUpperDial', 'handwheel'),
    (18, 14): ('MnBasinCenterDial', 'handwheel'),
    (11, 18): ('MnBasinLowerDial', 'handwheel'),
    (14, 20): ('MnBasinLever', 'outlet_lever'),
}
PILLAR = (28, 16)
DIRECTIONS = ((0, 1), (-1, 0), (0, -1), (1, 0))


def require(value, message):
    if not value:
        raise ValueError(message)


def index(room, position):
    x, y = position
    return y // 2 * room['attributes']['width'] + x // 2


def compare_map(before, after, name, sites):
    old, new = [p['data']['maps'][name] for p in (before, after)]
    require({k: v for k, v in old.items() if k != 'blocks'} ==
            {k: v for k, v in new.items() if k != 'blocks'}, name + ' events or state changed')
    permitted = {index(old, p) for p in sites}
    changed = {i for i, (a, b) in enumerate(zip(old['blocks'], new['blocks'])) if a != b}
    require(len(old['blocks']) == len(new['blocks']) and changed == permitted, name + ' placements')
    label = new['attributes']['blocks_label']
    require(bytes(new['blocks']) == base64.b64decode(after['data']['map_blocks'][label]),
            name + ' native map mirror')
    return old, new


def verify_graphics(parent, child):
    original, revised = [p['data']['tilesets']['cave'] for p in (parent, child)]
    prev, cur = parent['runtime_files'], child['runtime_files']
    layout = 'data/tilesets/cave_metatiles.bin'
    old_blocks, blocks = bytes(prev[layout]), bytes(cur[layout])
    require(blocks.startswith(old_blocks) and len(blocks) == len(old_blocks) + 48, 'original cave recipes')
    require(all(revised['collision'][k] == v for k, v in original['collision'].items()), 'original cave collision')
    require(json.loads(bytes(cur['data/tilesets/cave.json'])) == revised['collision'], 'collision mirror')
    require(json.loads(bytes(cur['data/tilesets/cave_palette_map.json'])) == revised['palette_map'], 'palette mirror')
    first = len(old_blocks) // 16
    permitted = set(range(0x48, 0x50))
    require(not (permitted & set(old_blocks)), 'published cave recipe uses instrument glyph')
    old_gfx, gfx = [bytes(f['gfx/tilesets/cave.2bpp']) for f in (prev, cur)]
    require(len(gfx) == len(old_gfx) == 96 * 16, 'native cave bank size')
    require(old_gfx[:0x48 * 16] == gfx[:0x48 * 16] and old_gfx[0x50 * 16:] == gfx[0x50 * 16:] and
            not any(old_gfx[0x48 * 16:0x50 * 16]), 'an occupied cave glyph changed')
    require(all(v == (4 if i in permitted else original['palette_map'][i])
                for i, v in enumerate(revised['palette_map'])), 'native cave palettes')
    png = Image.open(io.BytesIO(bytes(cur['gfx/tilesets/cave.png']))).convert('L')
    before_png = Image.open(io.BytesIO(bytes(prev['gfx/tilesets/cave.png']))).convert('L')
    require(png.size == before_png.size == (128, 48), 'native image dimensions')
    for i in range(96):
        for y in range(8):
            lo, hi = gfx[i * 16 + 2 * y:i * 16 + 2 * y + 2]
            for x in range(8):
                c = ((lo >> (7 - x)) & 1) | (((hi >> (7 - x)) & 1) << 1)
                pos = (i % 16 * 8 + x, i // 16 * 8 + y)
                require(png.getpixel(pos) == 255 - 85 * c, 'PNG and 2bpp cave bank differ')
                if i not in permitted:
                    require(png.getpixel(pos) == before_png.getpixel(pos), 'other source glyph changed')
    designs = json.loads((HERE / 'outlet-controls.json').read_text())
    room = child['data']['maps'][BASIN]
    prior = parent['data']['maps'][BASIN]
    for (x, y), (script, design) in CONTROLS.items():
        require(any(e['script'] == script and (e['x'], e['y']) == (x, y)
                    for e in room['events']['bg_events']), 'missing control event ' + script)
        slot = index(room, (x, y))
        block, donor = room['blocks'][slot], prior['blocks'][slot]
        require(first <= block < first + 3 and revised['collision'][f'{block:02x}'] ==
                original['collision'][f'{donor:02x}'], 'control changed movement')
        cells = blocks[block * 16:block * 16 + 16]
        old_cells = old_blocks[donor * 16:donor * 16 + 16]
        top = x % 2 * 2 + y % 2 * 8
        covered = {top, top + 1, top + 4, top + 5}
        require(all(v == old_cells[i] for i, v in enumerate(cells) if i not in covered),
                'art spread outside the control')
        for dy, row in enumerate(designs[design]):
            for dx, shade in enumerate(row):
                glyph = cells[top + dx // 8 + dy // 8 * 4]
                require(glyph in permitted and png.getpixel((glyph % 16 * 8 + dx % 8,
                        glyph // 16 * 8 + dy % 8)) == 255 - 85 * int(shade), 'incorrect instrument pixel')
    for name, room in child['data']['maps'].items():
        if name != BASIN and room['attributes']['tileset_name'] == 'cave':
            require(all(block < first for block in room['blocks']), name + ' uses Inner Basin artwork')
    for key in prev:
        if key.startswith('gfx/tilesets/water/') or key.startswith('gfx/tilesets/waterfall/'):
            require(prev[key] == cur[key], 'native animation frames changed')


def terrain(pack, name, open_outlet=False):
    room = pack['data']['maps'][name]
    a = room['attributes']
    blocks = room['blocks'].copy()
    if open_outlet:
        deeds = room['scripts']['MnBasinRestoreTerrain']
        edits = [tuple(map(int, c['args'])) for c in deeds if c['command'] == 'changeblock']
        require(len(edits) == 9 and deeds[0]['args'] == ['EVENT_MN_BASIN_INNER_OUTLET_ACKNOWLEDGED'],
                'wrong outlet state callback')
        for x, y, block in edits:
            blocks[index(room, (x, y))] = block
    col = pack['data']['tilesets'][a['tileset_name']]['collision']
    grid = {(x, y): col[f'{blocks[index(room, (x, y))]:02x}'][y % 2 * 2 + x % 2]
            for y in range(a['height'] * 2) for x in range(a['width'] * 2)}
    require(set(grid.values()) <= {'WALL', 'FLOOR', 'UP_WALL', 'WATER', 'LADDER', 'WARP_CARPET_DOWN'},
            'this geometry check needs a new movement rule')
    return grid


def reached(room, grid, start, surf=False):
    actors = {(o['x'], o['y']) for o in room['objects']}
    seen = {start}
    queue = deque([start])
    while queue:
        x, y = queue.popleft()
        for dx, dy in DIRECTIONS:
            dest = (x + dx, y + dy)
            target = grid.get(dest, 'WALL')
            if dest in actors or dest in seen or target == 'WALL' or (target == 'WATER' and not surf):
                continue
            if (dy == -1 and grid[x, y] == 'UP_WALL') or (dy == 1 and target == 'UP_WALL'):
                continue
            queue.append(dest)
            seen.add(dest)
    return seen


def neighbors(point):
    return {(point[0] + dx, point[1] + dy) for dx, dy in DIRECTIONS}


def verify_routes(parent, child):
    prior, basin = compare_map(parent, child, BASIN, CONTROLS)
    for solved in (False, True):
        old_grid = terrain(parent, BASIN, solved)
        grid = terrain(child, BASIN, solved)
        require(grid == old_grid, 'outlet movement differs')
        dest = reached(basin, grid, (2, 25))
        require(dest == reached(prior, old_grid, (2, 25)) and len(dest) == (319 if solved else 228),
                'outlet rooms changed')
        approaches = neighbors((14, 2)) & dest
        require(bool(approaches) == solved, 'legendary access in wrong state')
        for target in CONTROLS:
            approaches_here = neighbors(target) & dest
            require(approaches_here and all((3, 25) in reached(basin, grid, s) for s in approaches_here),
                    'no ordinary route from a control back to the ladder')
    previous, grotto = compare_map(parent, child, GROTTO, {PILLAR})
    old_grid, grid = terrain(parent, GROTTO), terrain(child, GROTTO)
    solids = {(28, 16), (29, 16), (28, 17), (29, 17)}
    require({p for p in grid if grid[p] != old_grid[p]} == solids and
            all(old_grid[p] == 'FLOOR' and grid[p] == 'WALL' for p in solids), 'pillar footprint')
    dry = reached(grotto, grid, (17, 23))
    wet = reached(grotto, grid, (17, 23), True)
    require(dry == reached(previous, old_grid, (17, 23)) and len(dry) == 470 and
            wet == reached(previous, old_grid, (17, 23), True) - solids, 'spring route changed')
    contacts = neighbors(PILLAR) & wet
    require(contacts == {(27, 16), (28, 15)} and not (contacts & dry) and
            all(grid[p] == 'WATER' and (17, 23) in reached(grotto, grid, p, True) for p in contacts),
            'two paddling approaches and ordinary return required')
    require(sum(v == 'WATER' for v in grid.values()) == 112 and
            grotto['scripts']['MN_R12_GROTTO_REVIVE'] == [
                {'command': 'hiddenitem', 'args': ['REVIVE', 'EVENT_MN_R12_GROTTO_REVIVE']}], 'spring reward')
    print('Upper, center, lower and outlet controls retain their approaches and the return ladder.')
    print('Closed / opened outlet reaches 228 / 319 squares. Both water-only Revive approaches return.')
    print('All 470 dry Gunflint squares and all 112 original water squares are preserved.')


def gallery(parent, child, output):
    coords = [(BASIN, c, n) for c, n in zip(CONTROLS, ['Upper dial', 'Center dial', 'Lower dial', 'Outlet lever'])]
    coords.append((GROTTO, PILLAR, 'Gunflint spring pillar'))
    sheet = Image.new('RGB', (620, 673), '#1b2426')
    pen = ImageDraw.Draw(sheet)
    try:
        title = ImageFont.truetype('Arial Bold.ttf', 18)
        font = ImageFont.truetype('Arial.ttf', 14)
    except OSError:
        title = font = ImageFont.load_default()
    pen.text((12, 8), 'Inner Basin and Gunflint', fill='#f0eadb', font=title)
    for (ox, label) in [(12, 'Before'), (215, 'After'), (419, 'After · detail')]:
        pen.text((ox, 37), label, font=font, fill='#afc8be')
    arts = [SourceArt(parent), SourceArt(child)]
    for row, (map_name, (x, y), label) in enumerate(coords):
        yp = 60 + 120 * row
        pen.text((12, yp), label, fill='#f0eadb', font=font)
        cx, cy = x * 16 + 8, y * 16 + 8
        imgs = []
        for pack, art in zip((parent, child), arts):
            room = pack['data']['maps'][map_name]
            a = room['attributes']
            im = art.render('cave', a['width'], a['height'], room['blocks'], 'nite')
            imgs.append(im)
        for col, im in enumerate(imgs):
            crop = im.crop((cx - 96, cy - 48, cx + 96, cy + 48))
            sheet.paste(crop, (12 + col * 203, yp + 19))
        crop = imgs[1].crop((cx - 48, cy - 24, cx + 48, cy + 24))
        sheet.paste(crop.resize((192, 96), Image.Resampling.NEAREST), (418, yp + 19))
    output.parent.mkdir(parents=True, exist_ok=True)
    sheet.save(output)


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('parent', type=Path)
    parser.add_argument('child', type=Path)
    parser.add_argument('--gallery', type=Path)
    args = parser.parse_args()
    parent, child = load_pack(args.parent), load_pack(args.child)
    verify_graphics(parent, child)
    verify_routes(parent, child)
    if args.gallery:
        gallery(parent, child, args.gallery)
