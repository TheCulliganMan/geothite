#!/usr/bin/env python3
"""Render an explicit cave-room candidate with native art; never export a pack."""
import argparse
from collections import deque
import hashlib
import json
from pathlib import Path
import time

from PIL import Image
from render_archive import save_render
from render_reference import SourceArt, load_pack, npc_palette_pack, room_sheet
from review_north_shore import oriented_sprite


def path(start, target, floor):
    previous = {start: None}
    todo = deque([start])
    while todo:
        at = todo.popleft()
        if at == target:
            result = [at]
            while previous[at] is not None:
                at = previous[at]
                result.append(at)
            return list(reversed(result))
        x, y = at
        for other in [(x-1,y),(x+1,y),(x,y-1),(x,y+1)]:
            if other in floor and other not in previous:
                previous[other] = at
                todo.append(other)
    return None


def review(pack, room, output):
    started = time.perf_counter()
    output.mkdir(parents=True, exist_ok=True)
    width, height = room['dimensions_metatiles']
    rows = [row.split() for row in room['rows']]
    assert len(rows) == height and all(len(row) == width for row in rows)
    blocks = [int(value, 16) for row in rows for value in row]
    collision = pack['data']['tilesets']['cave']['collision']
    kind = lambda p: collision[f'{blocks[p[1]//2*width+p[0]//2]:02x}'][p[1]%2*2+p[0]%2]
    actors = room.get('residents', []) + room.get('items', [])
    bodies = {tuple(actor['tile']) for actor in actors}
    floor = {(x,y) for y in range(height*2) for x in range(width*2)
             if kind((x,y)) == 'FLOOR' and (x,y) not in bodies}
    entry = tuple(room['clear_arrival'])
    assert entry in floor
    reports = []
    for actor in actors:
        at = tuple(actor['tile'])
        assert kind(at) == 'FLOOR'
        options = [path(entry, q, floor) for q in [(at[0]-1,at[1]),(at[0]+1,at[1]),
                                                   (at[0],at[1]-1),(at[0],at[1]+1)]]
        reachable = [p for p in options if p]
        assert reachable, actor['id'] + ': unreachable interaction'
        reports.append({'id':actor['id'],'approach_steps':min(len(p)-1 for p in reachable)})
    exits = {tuple(p) for p in room['exits']}
    actual_ladders = {(x,y) for y in range(height*2) for x in range(width*2) if kind((x,y)) == 'LADDER'}
    assert exits == actual_ladders, 'Unpaired or missing native mouth/ladder'
    for x,y in exits:
        assert any(p in floor and path(entry,p,floor) for p in [(x-1,y),(x+1,y),(x,y-1),(x,y+1)])
    waypoints = [tuple(p) for p in room['review_contract']['loop_waypoints_core']]
    for first, second in zip(waypoints, waypoints[1:]+waypoints[:1]):
        assert path(first, second, floor), 'Gallery/stair path disconnected'
    water = {(x,y) for y in range(height*2) for x in range(width*2) if kind((x,y)) == 'WATER'}
    assert not any(x in [0,width*2-1] or y in [0,height*2-1] for x,y in water)
    # A forced cave palette wins over the exterior clock. Match the installed
    # room before reviewing its candidate, rather than showing gold day stone
    # for an unchanged PALETTE_NITE room.
    native = pack['data']['maps'][room['id']]
    group = native['attributes']['time_of_day'] or 'day'
    declared = room['review_contract'].get('palette_group', group)
    assert declared == group, 'Candidate must preserve the installed palette contract'
    art = SourceArt(pack)
    terrain = art.render('cave', width, height, blocks, group=group)
    save_render(terrain, output/'terrain-original.png')
    sprite_pack = npc_palette_pack(pack, group)
    image = terrain.convert('RGBA')
    for actor in actors:
        frame = oriented_sprite(sprite_pack, actor.get('sprite', 'SPRITE_POKE_BALL'),
                                actor.get('facing', 'south'))
        save_render(frame, output/(actor['id']+'-native-frame.png'))
        image.alpha_composite(frame, (actor['tile'][0]*16, actor['tile'][1]*16-4))
    save_render(image, output/'whole-source-rgba.png')
    image = image.convert('RGB')
    save_render(image, output/'whole-source.png')
    cameras=[]
    for label, at in [('mouth',entry),('upper_gallery',waypoints[1]),('east_stair',waypoints[3]),
                      ('spring',tuple(room['items'][0]['tile'])),('lower_return',waypoints[-1])]:
        x,y=at
        crop=image.crop((x*16-80,y*16-72,x*16+80,y*16+72))
        save_render(crop,output/(label+'-160.png'))
        save_render(crop.resize((640,576),Image.Resampling.NEAREST),output/(label+'-4x.png'))
        cameras.append((label,crop))
    save_render(room_sheet(cameras,columns=3),output/'decision-cameras.png')
    report={'status':'SOURCE ONLY; no native publication/controller acceptance',
            'palette_group':group,
            'candidate_canonical_json_sha256':hashlib.sha256(json.dumps(room, sort_keys=True).encode()).hexdigest(),
            'dimensions_metatiles':[width,height], 'interaction_paths':reports,
            'native_stair_blocks':room['review_contract']['native_stair_blocks'],
            'unpaired_ladders':0,'unconnected_outer_water':0,
            'review_seconds':round(time.perf_counter()-started,3)}
    (output/'source-check.json').write_text(json.dumps(report,indent=2)+'\n')
    print(json.dumps(report))


if __name__ == '__main__':
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--pack',type=Path,required=True)
    parser.add_argument('--room',type=Path,required=True)
    parser.add_argument('--output',type=Path,required=True)
    args=parser.parse_args()
    review(load_pack(args.pack),json.loads(args.room.read_text()),args.output)
