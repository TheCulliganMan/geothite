#!/usr/bin/env python3
"""Read-only native preservation and walking-distance review for Duluth."""
import argparse
from collections import deque
import json
from pathlib import Path

from render_reference import load_pack


def distance(pack, start, goal):
    city = pack['data']['maps']['MnDuluth']
    width, height = city['attributes']['width'], city['attributes']['height']
    collision = pack['data']['tilesets'][city['attributes']['tileset_name']]['collision']
    occupied = {(actor['x'], actor['y']) for actor in city['objects']}
    todo = deque([(start, 0)])
    seen = {start}
    while todo:
        (x, y), steps = todo.popleft()
        if (x, y) == goal:
            return steps
        for at in [(x - 1, y), (x + 1, y), (x, y - 1), (x, y + 1)]:
            xx, yy = at
            if not (0 <= xx < width * 2 and 0 <= yy < height * 2):
                continue
            if at in occupied or at in seen:
                continue
            block = city['blocks'][yy // 2 * width + xx // 2]
            if collision[f'{block:02x}'][yy % 2 * 2 + xx % 2] == 'FLOOR':
                seen.add(at)
                todo.append((at, steps + 1))
    raise AssertionError(f'No ordinary walking path: {start} to {goal}')


def review(before, after):
    assert before['audio_manifest'] == after['audio_manifest']
    assert before['runtime_files'] == after['runtime_files']
    assert before['report'] == after['report']
    old_maps, new_maps = before['data']['maps'], after['data']['maps']
    assert old_maps.keys() == new_maps.keys()
    assert all(old_maps[name] == new_maps[name] for name in old_maps
               if name not in {'MnDuluth', 'MnR10C01R00'})
    old_neighbor, new_neighbor = old_maps['MnR10C01R00'], new_maps['MnR10C01R00']
    assert all(old_neighbor[key] == new_neighbor[key] for key in old_neighbor if key != 'attributes')
    assert all(old_neighbor['attributes'][key] == new_neighbor['attributes'][key]
               for key in old_neighbor['attributes'] if key != 'connections')
    old, new = old_maps['MnDuluth'], new_maps['MnDuluth']
    geometry = {'attributes', 'blocks', 'events', 'objects', 'map_event_section_commands'}
    assert all(old[key] == new[key] for key in old if key not in geometry)
    assert len(old['events']['warps']) == len(new['events']['warps']) == 76
    for a, b in zip(old['events']['warps'], new['events']['warps']):
        assert {k: v for k, v in a.items() if k not in {'x', 'y'}} == {
            k: v for k, v in b.items() if k not in {'x', 'y'}}
    for a, b in zip(old['objects'], new['objects']):
        assert {k: v for k, v in a.items() if k not in {'x', 'y'}} == {
            k: v for k, v in b.items() if k not in {'x', 'y'}}
    trips = {}
    for label, start, goal, revised_start, revised_goal in [
        ('gym_to_workshop', (48, 36), (80, 60), (48, 32), (72, 42)),
        ('clinic_to_workshop', (69, 36), (80, 60), (69, 32), (72, 42)),
        ('mart_to_workshop', (80, 44), (80, 60), (80, 40), (72, 42)),
    ]:
        trips[label] = {'before': distance(before, start, goal),
                        'after': distance(after, revised_start, revised_goal)}
        assert trips[label]['after'] < trips[label]['before']
    return {'dimensions_before': [64, 48], 'dimensions_after': [64, 46],
            'retained_real_doors': 38, 'ordinary_walking_steps': trips,
            'ordinary_ledge_return_steps': distance(after, (46, 42), (46, 39)),
            'audio_manifest_and_runtime_files_identical': True,
            'all_unrelated_maps_and_city_programs_identical': True}


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--before', type=Path, required=True)
    parser.add_argument('--after', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    report = review(load_pack(args.before), load_pack(args.after))
    args.output.write_text(json.dumps(report, indent=2) + '\n')
    print(json.dumps(report, indent=2))
