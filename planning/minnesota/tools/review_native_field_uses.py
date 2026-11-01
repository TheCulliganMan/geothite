#!/usr/bin/env python3
"""Render exact native field-use source and check its local terrain contracts."""
import argparse
from collections import deque
import json
from pathlib import Path

from PIL import Image
from render_reference import SourceArt, load_pack


def kind(collision, blocks, width, point):
    x,y = point
    return collision[f'{blocks[y//2*width+x//2]:02x}'][y%2*2+x%2]


def flood(collision, blocks, width, height, start, blocked, surf=False):
    seen = {start}
    todo = deque([start])
    while todo:
        x, y = todo.popleft()
        for point in [(x-1,y), (x+1,y), (x,y-1), (x,y+1)]:
            xx, yy = point
            if point in seen or point in blocked or not (0 <= xx < width*2 and 0 <= yy < height*2):
                continue
            terrain = kind(collision, blocks, width, point)
            if terrain in {'FLOOR', 'TALL_GRASS', 'LADDER'} or (surf and terrain == 'WATER'):
                seen.add(point)
                todo.append(point)
    return seen


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--pack', required=True, type=Path)
    parser.add_argument('--site', required=True, type=Path)
    parser.add_argument('--output', required=True, type=Path)
    args = parser.parse_args()
    pack = load_pack(args.pack)
    site = json.loads(args.site.read_text())
    data = pack['data']
    art = SourceArt(pack)
    args.output.mkdir(parents=True, exist_ok=False)
    gallery = site['gallery']
    blocks = [int(v,16) for row in gallery['rows'] for v in row.split()]
    width, height = gallery['dimensions_metatiles']
    collision = data['tilesets']['cave']['collision']
    arrival = tuple(gallery['passive_arrival'])
    exit_at = tuple(gallery['active_exit'])
    reached = flood(collision, blocks, width, height, arrival, {tuple(gallery['cache']), exit_at})
    assert {tuple(gallery['read_approach']), (8,5), (9,12)} <= reached
    assert kind(collision,blocks,width,exit_at) == 'LADDER'
    assert kind(collision,blocks,width,tuple(gallery['cache'])) == 'FLOOR'
    assert kind(collision,blocks,width,tuple(gallery['read'])) == 'WALL'
    assert not any(x in {0,width*2-1} or y in {0,height*2-1} for x,y in reached)
    art.render('cave',width,height,blocks,'nite').save(args.output/'gallery-lit-source.png')
    parent = data['maps'][gallery['parent']]
    parent_blocks = list(parent['blocks'])
    x, y = gallery['parent_stair_metatile']
    pw, ph = parent['attributes']['width'], parent['attributes']['height']
    assert parent_blocks[y*pw+x] == int(gallery['parent_before'],16)
    parent_blocks[y*pw+x] = int(gallery['parent_after'],16)
    occupied = {(o['x'],o['y']) for o in parent['objects']}
    occupied.add((9,11))  # Original active world exit cannot be an internal path.
    dry = flood(collision,parent_blocks,pw,ph,(9,10),occupied|{tuple(gallery['parent_active'])})
    assert {(5,6), tuple(gallery['parent_arrival'])} <= dry
    assert kind(collision,parent_blocks,pw,tuple(gallery['parent_active'])) == 'LADDER'
    assert sum(abs(a-b) for a,b in zip(gallery['parent_arrival'],gallery['parent_active'])) == 1
    art.render('cave',pw,ph,parent_blocks,'nite').save(args.output/'parent-stair-source.png')
    eddy = site['eddy']
    lake = data['maps'][eddy['map']]
    lw, lh = lake['attributes']['width'], lake['attributes']['height']
    blocks = list(lake['blocks'])
    cx, cy = eddy['cap_metatile']
    ex, ey = eddy['eddy_metatile']
    assert blocks[cy*lw+cx] == blocks[ey*lw+ex] == int(eddy['before'],16)
    blocks[cy*lw+cx] = int(eddy['cap_after'],16)
    blocks[ey*lw+ex] = int(eddy['closed_block'],16)
    collision = data['tilesets'][eddy['source_tileset']]['collision']
    target = tuple(eddy['cache'])
    x,y = target
    approaches = {(x-1,y),(x+1,y),(x,y-1),(x,y+1)}
    occupied = {(o['x'],o['y']) for o in lake['objects']}
    closed = flood(collision,blocks,lw,lh,tuple(eddy['approach']),occupied,True)
    assert not closed & approaches, 'hidden cache has a closed-state interaction bypass'
    image = art.render(eddy['source_tileset'],lw,lh,blocks,'day')
    image.save(args.output/'leech-closed-source.png')
    image.crop((864,248,1024,392)).resize((640,576),Image.Resampling.NEAREST).save(args.output/'leech-closed-player-source.png')
    blocks[ey*lw+ex] = int(eddy['open_block'],16)
    opened = flood(collision,blocks,lw,lh,tuple(eddy['approach']),occupied,True)
    assert opened & approaches == {tuple(eddy['cache_approach'])}
    art.render(eddy['source_tileset'],lw,lh,blocks,'day').save(args.output/'leech-open-source.png')
    assert collision[eddy['closed_block']][1:] == collision[eddy['open_block']][1:]
    print(json.dumps(dict(gallery_read_cache_approach_and_exit_connected=True,
        gallery_border_closed=True,parent_guide_stair_return_connected=True,
        closed_cache_has_no_interaction_approach=True,
        open_cache_has_one_native_water_approach=True,
        original_rock_and_water_quadrants_preserved=True,
        identity_string_bytes={key:len(value.encode()) for key,value in pack['identity'].items() if isinstance(value,str)},
        caution='Source terrain only; no native field action, Itemfinder, save safety, publication or visual acceptance is implied.'),indent=2))


if __name__ == '__main__':
    main()
