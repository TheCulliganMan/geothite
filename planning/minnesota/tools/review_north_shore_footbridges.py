#!/usr/bin/env python3
"""Read-only whole North Shore native review with explicit dry walking course.

Use --terrain for the eight-chunk hand-authored source preview. Output must be
fresh: originals, crops and overlays are retained for every iteration.
"""
import argparse
from collections import deque
import hashlib
import json
from pathlib import Path
import time

from PIL import Image, ImageDraw

from render_archive import save_render
from render_reference import SourceArt, load_pack, npc_palette_pack, room_sheet
from review_passage_native import actors


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--pack', type=Path, required=True)
    parser.add_argument('--terrain', type=Path)
    parser.add_argument('--wide-rails', action='store_true',
                        help='Preview native rail caps 85/86 with four clear walking tiles')
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    args.output.mkdir(parents=True, exist_ok=False)
    pack = load_pack(args.pack)
    started = time.perf_counter()
    art = SourceArt(pack)
    terrain = json.loads(args.terrain.read_text()) if args.terrain else None
    if args.wide_rails:
        assert terrain, '--wide-rails requires an authored terrain source'
        blocks = list(art.blocks('mn_north_shore'))
        assert len(blocks) in [0x85, 0x87], 'installed narrow or four-lane coastal art required'
        if len(blocks) == 0x85:
            for rail_column in [0, 3]:
                cap = blocks[1].copy()
                for row in range(4):
                    cap.paste(blocks[0x44].crop((0, row*8, 8, row*8+8)), (rail_column*8, row*8))
                blocks.append(cap)
        art.cache['mn_north_shore', 'day', None] = blocks
    elif terrain:
        blocks = list(art.blocks('mn_north_shore'))
        assert len(blocks) == 0x84, 'source preview requires original art extent'
        bridge = Image.new('RGB', (32, 32))
        for row in range(4):
            for column, (block, source_column) in enumerate([(0x44, 0), (1, 1), (1, 2), (0x44, 0)]):
                bridge.paste(blocks[block].crop((source_column*8, row*8, source_column*8+8, row*8+8)), (column*8, row*8))
        art.cache['mn_north_shore', 'day', None] = [*blocks, bridge]
    scene = Image.new('RGBA', (1792, 3072), '#181d22')
    kinds, bodies = {}, set()
    for name, module in pack['data']['maps'].items():
        if not name.startswith('MnR11C'):
            continue
        column, row = map(int, name[6:].split('R'))
        a = module['attributes']
        blocks = ([int(v, 16) for line in terrain[name]['rows'] for v in line.split()]
                  if terrain else module['blocks'])
        assert len(blocks) == a['width']*a['height']
        image = actors(npc_palette_pack(pack, 'day'), art.render(a['tileset_name'], a['width'], a['height'], blocks), module)
        scene.alpha_composite(image, (column*1024, row*768))
        collision = pack['data']['tilesets'][a['tileset_name']]['collision']
        for y in range(a['height']*2):
            for x in range(a['width']*2):
                block = blocks[y//2*a['width']+x//2]
                if args.wide_rails and block in [0x85, 0x86]:
                    values = (['WALL', 'FLOOR', 'WALL', 'FLOOR'] if block == 0x85
                              else ['FLOOR', 'WALL', 'FLOOR', 'WALL'])
                else:
                    values = ['LADDER']*4 if terrain and block == 0x84 else collision[f'{block:02x}']
                kinds[x+column*64, y+row*48] = values[y%2*2+x%2]
        bodies.update((o['x']+column*64, o['y']+row*48) for o in module['objects'])
    save_render(scene, args.output/'whole-original.png')
    save_render(scene.resize((896, 1536), Image.Resampling.NEAREST), args.output/'whole-overview.png')
    allowed = {at for at, kind in kinds.items() if kind in {'FLOOR', 'TALL_GRASS', 'LADDER', 'DOOR'}}-bodies
    start, end = (46, 191), (71, 8)
    parents, queue = {start: None}, deque([start])
    while queue and end not in parents:
        x, y = queue.popleft()
        for at in [(x-1, y), (x+1, y), (x, y-1), (x, y+1)]:
            if at in allowed and at not in parents:
                parents[at] = (x, y)
                queue.append(at)
    assert end in parents, 'Grand Marais must be reachable without Surf or hops'
    course, at = [], end
    while at is not None:
        course.append(at)
        at = parents[at]
    course.reverse()
    overlay = scene.convert('RGB')
    draw = ImageDraw.Draw(overlay)
    points = [(x*16+8, y*16+8) for x, y in course]
    draw.line(points, fill='#173a33', width=11)
    draw.line(points, fill='#79f9d4', width=5)
    save_render(overlay, args.output/'dry-course-original.png')
    save_render(overlay.resize((896, 1536), Image.Resampling.NEAREST), args.output/'dry-course-overview.png')
    views, sheets = [], []
    for i, (x, y, length) in enumerate([(40,40,2), (40,76,2), (46,96,4), (40,110,2), (34,122,2), (40,162,2), (46,186,2)]):
        crop = scene.crop((x*16-112, y*16-112, (x+2)*16+112, (y+length)*16+112))
        save_render(crop, args.output/f'crossing-{i}-original.png')
        save_render(crop.resize((crop.width*2, crop.height*2), Image.Resampling.NEAREST), args.output/f'crossing-{i}-2x.png')
        sheets.append((f'Crossing {i+1}', crop))
        views.append(dict(id=f'crossing-{i+1}', map=f'MnR11C00R{y//48:02}', core=[x,y%48]))
    save_render(room_sheet(sheets, 3), args.output/'all-crossings.png')
    for label, name, core in [('duluth-landing','MnR11C00R03',[46,47]), ('grand-marais-landing','MnR11C01R00',[7,8])]:
        views.append(dict(id=label, map=name, core=core))
    (args.output/'native-views.json').write_text(json.dumps(views, indent=2)+'\n')
    receipt = dict(status='PASS', scope='Static native source and dry course; controller replay is separate.',
                   wide_rail_preview=args.wide_rails,
                   pack=str(args.pack), pack_sha256=hashlib.sha256(args.pack.read_bytes()).hexdigest(),
                   source=str(args.terrain) if args.terrain else 'installed pack',
                   warm_render_and_graph_seconds=time.perf_counter()-started, walking_steps=len(course)-1,
                   dry_course=course, bridge_count=7)
    (args.output/'receipt.json').write_text(json.dumps(receipt, indent=2)+'\n')
    print('PASS', receipt['walking_steps'], 'dry steps;', round(receipt['warm_render_and_graph_seconds'],3), 'seconds after pack load')


if __name__ == '__main__':
    main()
