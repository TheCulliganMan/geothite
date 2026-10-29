#!/usr/bin/env python3
"""Review the explicit Winona depot overlay with supplied native source art."""
import argparse
from copy import deepcopy
import json
from pathlib import Path
import time

from render_archive import save_render
from render_reference import SourceArt, load_pack, room_sheet
from review_boundary_basecamp import adjacent, flood
from review_north_shore import oriented_sprite

ROOT = Path(__file__).resolve().parents[3]
SOURCE = ROOT / 'planning/minnesota/transit/MnWinonaStation.json'


def validate(pack, room):
    assert room['id'] == 'MnWinonaStation'
    assert room['native_constant'] == 'MN_WINONA_STATION'
    assert room['dimensions_metatiles'] == [10, 9]
    assert room['passive_arrival'] == dict(index=3, tile=[11, 15])
    binding = room['exterior_binding']; door = binding['door']
    assert binding['hub'] == 'MnWinona'
    assert door == dict(warp_index=9, return_warp_index=10, tile=[19, 13],
                        return_tile=[19, 14], arrival_tile=[11, 15],
                        target='MnWinonaStation', target_warp=3, collision='DOOR')
    assert [(p['position'], p['before'], p['after']) for p in binding['patches']] == [
        ([9, 5], '02', '18'), ([10, 5], '02', '19'), ([9, 6], '02', '16'),
        ([10, 6], '02', '1e'), ([11, 6], '02', '45')]
    town = pack['data']['maps']['MnWinona']; a = town['attributes']
    assert [a['width'], a['height']] == [16, 14] and a['tileset_name'] == 'johto'
    blocks = list(town['blocks'])
    for patch in binding['patches']:
        x, y = patch['position']; index = y * 16 + x
        assert blocks[index] == int(patch['before'], 16), 'changed town preimage'
        blocks[index] = int(patch['after'], 16)
    rows = [int(block, 16) for row in room['rows'] for block in row.split()]
    assert len(rows) == 90
    seen = None
    for tileset, terrain, width, height, start, actors in [
        ('johto', blocks, 16, 14, (19, 14), town['objects']),
        ('train_station', rows, 10, 9, (11, 15), room['residents']),
    ]:
        collision = pack['data']['tilesets'][tileset]['collision']
        def kind(point):
            x, y = point
            assert 0 <= x < width * 2 and 0 <= y < height * 2
            return collision[f'{terrain[y//2*width+x//2]:02x}'][y % 2 * 2 + x % 2]
        bodies = {tuple(actor['tile']) if 'tile' in actor else (actor['x'], actor['y']) for actor in actors}
        floor = {(x, y) for y in range(height*2) for x in range(width*2)
                 if kind((x, y)) in {'FLOOR', 'DOOR', 'WARP_CARPET_DOWN'}}
        assert kind(start) == 'FLOOR' and start not in bodies
        seen = flood(start, floor-bodies, set())
        if tileset == 'johto':
            assert kind((19, 13)) == 'DOOR' and (19, 13) in seen
            assert all(kind(at) == 'FLOOR' for at in [(7, 8), (17, 8), (7, 24), (18, 20)])
            assert all(kind(at) == 'DOOR' for at in [(7, 7), (17, 7), (7, 23), (18, 19)])
            continue
        assert len(bodies) == 4
        for actor in actors:
            assert kind(tuple(actor['tile'])) == 'FLOOR'
            assert tuple(actor['approach']) in seen
            assert tuple(actor['approach']) in adjacent(tuple(actor['tile']))
            assert all(0 < len(line) <= 18 for line in actor['lines'])
        assert {(6, 5), (11, 5), (11, 7)}.isdisjoint(seen), 'unprotected platform'
        assert (10, 17) in seen and (11, 17) in seen
        assert all(tuple(sign['approach']) in seen for sign in room['signs'])
    story = room['story_contract']
    assert story['reward_item'] == 'ICE_HEAL' and story['reward_quantity'] == 1
    assert [story[key] for key in ['visit_flag', 'read_flag', 'heard_flag', 'gift_flag']] == [
        'EVENT_MN_STATION_WINONA_VISITED', 'EVENT_MN_WINONA_DEPOT_LOG_READ',
        'EVENT_MN_WINONA_DEPOT_MASON_HEARD', 'EVENT_MN_WINONA_DEPOT_DRY_KIT_GIVEN']
    return blocks, rows


def review(pack, room, output):
    start = time.monotonic(); blocks, rows = validate(pack, room)
    art = SourceArt(pack); images = []
    for name, tileset, width, height, terrain, people in [
        ('town', 'johto', 16, 14, blocks, []),
        ('hall', 'train_station', 10, 9, rows, room['residents']),
    ]:
        image = art.render(tileset, width, height, terrain).convert('RGBA')
        save_render(image, output / (name+'-whole-source.png'))
        for actor in people:
            image.alpha_composite(oriented_sprite(pack, actor['sprite'], actor['facing']),
                                  tuple(value*16 for value in actor['tile']))
        save_render(image, output / (name+'-whole-with-residents.png'))
        images.append((name, image))
    save_render(room_sheet(images, 2), output / 'whole-place-review.png')
    rejected = 0
    for mutate in [
        lambda v: v['passive_arrival'].update(tile=[11, 9]),
        lambda v: v['exterior_binding']['patches'][0].update(after='19'),
        lambda v: v['residents'][0].update(tile=[10, 9]),
        lambda v: v['residents'][1].update(approach=[6, 5]),
        lambda v: v['story_contract'].update(reward_quantity=2),
    ]:
        bad = deepcopy(room); mutate(bad)
        try:
            validate(pack, bad)
        except AssertionError:
            rejected += 1
    assert rejected == 5
    print(f'Source geometry and five counterexamples pass in {time.monotonic()-start:.3f}s.')


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--pack', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    review(load_pack(args.pack), json.loads(SOURCE.read_text()), args.output)
