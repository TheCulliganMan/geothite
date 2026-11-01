#!/usr/bin/env python3
"""Render the hand-authored ice dungeon and plan native slide inputs; no pack writes."""
import argparse
from collections import deque
from copy import deepcopy
import json
from pathlib import Path

from render_archive import save_render
from render_reference import SourceArt, load_pack, npc_palette_pack, room_sheet
from review_north_shore import oriented_sprite

DIRECTIONS = [('left', -1, 0), ('right', 1, 0), ('up', 0, -1), ('down', 0, 1)]


def course(room, collision, start, bodies):
    width, height = room['dimensions_metatiles']
    blocks = [int(v, 16) for row in room['rows'] for v in row.split()]

    def kind(at):
        x, y = at
        if not (0 <= x < width*2 and 0 <= y < height*2):
            return 'WALL'
        return collision[f'{blocks[y//2*width+x//2]:02x}'][y%2*2+x%2]

    def advance(at, dx, dy):
        trace = []
        while True:
            other = (at[0]+dx, at[1]+dy)
            # Native UP_WALL blocks leaving north and entering south;
            # east/west entry is a legal dry stop, not an ice bumper.
            if (kind(at) == 'UP_WALL' and dy == -1) or (kind(other) == 'UP_WALL' and dy == 1):
                return at, trace
            if other in bodies or kind(other) not in {'FLOOR', 'ICE', 'ICE_2B', 'LADDER', 'UP_WALL'}:
                return at, trace
            at = other
            trace.append(at)
            if kind(at) not in {'ICE', 'ICE_2B'}:
                return at, trace

    parents = {start: None}
    todo = deque([start])
    while todo:
        at = todo.popleft()
        for button, dx, dy in DIRECTIONS:
            other, trace = advance(at, dx, dy)
            if other not in parents:
                parents[other] = (at, button, trace)
                todo.append(other)

    def path(target):
        assert target in parents, (room['id'], 'unreachable stopping point', target)
        route = []
        at = target
        while parents[at] is not None:
            before, button, trace = parents[at]
            route.append({'button': button, 'stop': list(at), 'steps': len(trace)})
            at = before
        return list(reversed(route))

    return kind, advance, parents, path


def review(pack, source, output, published):
    output.mkdir(parents=True, exist_ok=True)
    art = SourceArt(pack)
    collision = pack['data']['tilesets']['ice_path']['collision']
    result = []
    scenes = []
    for room in source['rooms']:
        width, height = room['dimensions_metatiles']
        assert len(room['rows']) == height and all(len(r.split()) == width for r in room['rows'])
        blocks = [int(v, 16) for row in room['rows'] for v in row.split()]
        if published:
            native = pack['data']['maps'][room['id']]
            assert native['blocks'] == blocks
            assert native['attributes']['tileset_name'] == 'ice_path'
        actors = room['residents'] + room['items']
        bodies = {tuple(a['tile']) for a in actors}
        kind, advance, states, path = course(room, collision, tuple(room['arrival']), bodies)
        targets = []
        for item in room['items']:
            x, y = item['tile']
            approaches = [(x+dx,y+dy) for _,dx,dy in DIRECTIONS if (x+dx,y+dy) in states]
            assert approaches, (item['id'], 'no interaction approach')
            to = min(approaches, key=lambda at: len(path(at)))
            targets.append({'id':item['id'], 'approach': list(to), 'course':path(to)})
        for warp in room['warps']:
            targets.append({'id': 'warp-'+str(warp['tile']), 'course': path(tuple(warp['tile']))})
        # Collection removes a real stopping body. Every state must still have
        # a way back to the dry arrival with all pickups absent.
        _, back_advance, clear_states, _ = course(room, collision, tuple(room['arrival']),
                                                 {tuple(a['tile']) for a in room['residents']})
        reverse = {at: [] for at in clear_states}
        for at in clear_states:
            for _, dx, dy in DIRECTIONS:
                other, _ = back_advance(at, dx, dy)
                reverse.setdefault(other, []).append(at)
        returning = {tuple(room['arrival'])}
        todo = deque(returning)
        while todo:
            for other in reverse[todo.popleft()]:
                if other not in returning:
                    returning.add(other)
                    todo.append(other)
        assert returning == set(clear_states), (room['id'], 'slide trap', set(clear_states)-returning)
        ice = {(x,y) for y in range(height*2) for x in range(width*2) if kind((x,y)) == 'ICE'}
        assert ice
        assert all(kind((x,y)) == 'WALL' for y in (0,height*2-1) for x in range(width*2))
        assert all(kind((x,y)) == 'WALL' for x in (0,width*2-1) for y in range(height*2))
        terrain = art.render('ice_path', width, height, blocks, 'nite').convert('RGBA')
        sprite_pack = npc_palette_pack(pack, 'nite')
        for actor in actors:
            frame = oriented_sprite(sprite_pack, actor.get('sprite','SPRITE_POKE_BALL'), 'south')
            terrain.alpha_composite(frame, (actor['tile'][0]*16, actor['tile'][1]*16))
        scene = terrain.convert('RGB')
        save_render(scene,output/(room['id']+'-whole.png'))
        scenes.append((room['id'], scene))
        views = []
        for view in room['review_viewpoints']:
            x,y = view['tile']
            crop=scene.crop((x*16-72,y*16-64,x*16+88,y*16+80))
            save_render(crop, output/(room['id']+'-'+view['id']+'.png'))
            views.append((view['id'],crop))
        save_render(room_sheet(views,columns=3),output/(room['id']+'-cameras.png'))
        result.append({'id':room['id'], 'ice_tiles':len(ice), 'stopping_states':len(states),
                       'all_clear_states_return':True, 'targets':targets})
    save_render(room_sheet(scenes,columns=2),output/'whole-dungeon.png')
    (output/'planned-courses.json').write_text(json.dumps(result,indent=2)+'\n')
    print(json.dumps(result,indent=2))


def preserve(base, final, source):
    """Check all retained maps, native source files, audio and recovery data."""
    old, new = base['data'], final['data']
    exterior = source['exterior']['map']
    additions = {room['id'] for room in source['rooms']}
    assert set(new['maps'])-set(old['maps']) == additions
    assert set(old['maps']) <= set(new['maps'])
    for name, module in old['maps'].items():
        if name != exterior:
            assert new['maps'][name] == module, name
    before, after = old['maps'][exterior], new['maps'][exterior]
    expected = deepcopy(before)
    width = before['attributes']['width']
    for change in source['exterior']['changes']:
        x,y = change['metatile']
        index = y*width+x
        assert expected['blocks'][index] == change['previous']
        expected['blocks'][index] = change['replacement']
    assert after['events']['warps'][:len(before['events']['warps'])] == before['events']['warps']
    expected['events']['warps'] = after['events']['warps']
    expected['map_event_section_commands'] = after['map_event_section_commands']
    assert expected == after, 'unexpected exterior story/actor/attribute change'
    assert base['runtime_files'] == final['runtime_files']
    for key in ['audio_manifest','compiled_audio']:
        assert base.get(key) == final.get(key), key
    changed = {'maps','map_attributes','map_blocks','map_scripts','npcs',
               'map_dimensions','runtime_map_metadata','pokegear_landmarks',
               'story_event_script_constants'}
    assert set(old) == set(new)
    for key in old:
        if key not in changed:
            assert old[key] == new[key], key
    for key,value in old['story_event_script_constants']['global'].items():
        assert new['story_event_script_constants']['global'][key] == value, key
    return {'unrelated_maps_unchanged':len(old['maps'])-1, 'new_maps':len(additions),
            'exterior_metatiles_changed':len(source['exterior']['changes']),
            'native_runtime_files_and_audio_unchanged':True,
            'unowned_data_tables_unchanged':True}


if __name__ == '__main__':
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--pack',type=Path,required=True)
    parser.add_argument('--source',type=Path,default=Path('planning/minnesota/ice-caves/voyageurs-ice.json'))
    parser.add_argument('--output',type=Path,required=True)
    parser.add_argument('--published',action='store_true')
    parser.add_argument('--base',type=Path,help='Also check preservation against this verified parent')
    args=parser.parse_args()
    pack=load_pack(args.pack);source=json.loads(args.source.read_text())
    review(pack,source,args.output,args.published)
    if args.base:
        report=preserve(load_pack(args.base),pack,source)
        (args.output/'preservation.json').write_text(json.dumps(report,indent=2)+'\n')
        print(json.dumps(report))
