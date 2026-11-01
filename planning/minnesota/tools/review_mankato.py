#!/usr/bin/env python3
"""Review Mankato's explicit native-source town and its physical door contracts."""
import argparse
import json
from collections import deque
from pathlib import Path

from PIL import Image
from render_archive import save_render
from render_reference import SourceArt, layout_blocks, load_pack
from review_north_shore import oriented_sprite

ROOT = Path(__file__).resolve().parents[3]
PLAN = ROOT / 'planning/minnesota'


def review(pack, town, layout, output, published=False):
    width, height = layout['dimensions_metatiles']
    blocks = layout_blocks(layout)
    art = SourceArt(pack)
    terrain = art.render('johto', width, height, blocks)
    save_render(terrain, output / 'Mankato-terrain.png')
    collision = pack['data']['tilesets']['johto']['collision']

    def kind(tile):
        x, y = tile
        if not (0 <= x < width * 2 and 0 <= y < height * 2):
            return 'WALL'
        block = blocks[(y // 2) * width + x // 2]
        return collision[f'{block:02x}'][(y % 2) * 2 + x % 2]

    occupied = {tuple(a['tile']) for a in town['residents']}
    floor = {'FLOOR', 'DOOR', 'LADDER', 'WARP_CARPET_DOWN'}

    def distances(start):
        seen = {tuple(start): 0}
        queue = deque(seen)
        while queue:
            x, y = queue.popleft()
            for tile in [(x-1,y), (x+1,y), (x,y-1), (x,y+1)]:
                if tile not in seen and tile not in occupied and kind(tile) in floor:
                    seen[tile] = seen[(x,y)] + 1
                    queue.append(tile)
        return seen

    reached = distances(town['ports'][0]['landing_metatile'][i] * 2 for i in range(2))
    for door in town['doors']:
        assert kind(door['tile']) == 'DOOR', ('facade door', door)
        assert kind(door['return_tile']) == 'FLOOR', ('safe return', door)
        assert tuple(door['return_tile']) in reached, ('unreachable facade', door)
    assert {t for t in reached if kind(t) == 'DOOR'} == {tuple(d['tile']) for d in town['doors']}, 'unbound visible door'
    for port in town['ports']:
        assert tuple(c * 2 for c in port['landing_metatile']) in reached, 'unreachable port'
    for sign in town['signs']:
        assert kind(sign['tile']) == 'WALL', ('sign quadrant', sign)
        x,y=sign['tile']
        assert any(t in reached for t in [(x-1,y),(x+1,y),(x,y-1),(x,y+1)]), 'unreadable sign'
    for a in town['residents']:
        assert kind(a['tile']) == 'FLOOR', ('actor floor', a)
    assert not any(x in (0,width*2-1) or y in (0,height*2-1) for x,y in reached), 'unconnected outer edge'
    assert not any('CUT_TREE' in collision[f'{b:02x}'] for b in set(blocks)), 'incidental Cut tree'
    ledge = town['ledge_contract']
    assert kind(ledge['approach_core']) == kind(ledge['landing_core']) == 'FLOOR'
    assert kind(ledge['hop_core']) == 'HOP_DOWN'
    assert kind(ledge['blocked_reverse_core']) == 'WALL'
    return_length = distances(ledge['landing_core'])[tuple(ledge['approach_core'])]
    assert return_length >= ledge['minimum_ordinary_return_steps'], return_length
    if published:
        module = pack['data']['maps'][town['id']]
        assert module['blocks'] == blocks, 'published exterior differs'
        assert [module['attributes']['width'],module['attributes']['height']] == [width,height]
        for door in town['doors']:
            room = pack['data']['maps'][door['target']]
            warps = module['events']['warps']
            enter, back = warps[door['warp_index']-1], warps[door['return_warp_index']-1]
            assert [enter['x'],enter['y']] == door['tile'] and enter['target_warp_id'] == 3
            assert [back['x'],back['y']] == door['return_tile']
            for warp in room['events']['warps']:
                assert warp['target_map_constant'] == 'MN_MANKATO' and warp['target_warp_id'] == door['return_warp_index']
        save_render(art.render('johto',width,height,module['blocks']),output/'Mankato-published-terrain.png')
    scene = terrain.convert('RGBA')
    for actor in town['residents']:
        x,y = actor['tile']
        scene.alpha_composite(oriented_sprite(pack,actor['sprite'],actor['facing']),(x*16,y*16-4))
    save_render(scene,output/'Mankato-whole.png')
    for view in town['viewpoints']:
        x,y=view['tile']; left=max(0,min(x*16+8-80,width*32-160)); top=max(0,min(y*16+8-72,height*32-144))
        save_render(scene.crop((left,top,left+160,top+144)),output/f'Mankato-{view["id"]}-160.png')
    print(f'Mankato: 3 bound facades, 2 reachable port landings, closed perimeter, ordinary terrace return {return_length} steps; published={published}')


def main():
    parser=argparse.ArgumentParser()
    parser.add_argument('--pack',type=Path,default=ROOT/'content-packs/text-tui.crystalpack')
    parser.add_argument('--output',type=Path,default=PLAN/'renders/mankato-city')
    parser.add_argument('--verify-published',action='store_true')
    args=parser.parse_args()
    review(load_pack(args.pack),json.loads((PLAN/'towns/MnMankato.json').read_text()),json.loads((PLAN/'towns/MnMankato.layout.json').read_text()),args.output,args.verify_published)


if __name__ == '__main__':
    main()
