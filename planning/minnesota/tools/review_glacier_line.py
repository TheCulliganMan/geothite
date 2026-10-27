#!/usr/bin/env python3
"""Review explicitly authored station rows; never export or modify a game pack."""
import argparse
from collections import deque
from copy import deepcopy
import hashlib
import json
from pathlib import Path
import time

from PIL import Image, ImageDraw

from render_archive import save_render
from render_reference import SourceArt, load_pack
from render_stages import render_stage
from review_north_shore import oriented_sprite

ROOT = Path(__file__).resolve().parents[3]
PLAN = ROOT / 'planning/minnesota'
STATION = PLAN / 'transit/MnStPaulUnionDepot.json'
NETWORK = PLAN / 'transit/glacier-line.json'
OUTPUT = PLAN / 'renders/glacier-line'


def flood(start, cells):
    seen = {start}
    todo = deque([start])
    while todo:
        x, y = todo.popleft()
        for q in [(x - 1, y), (x + 1, y), (x, y - 1), (x, y + 1)]:
            if q in cells and q not in seen:
                seen.add(q)
                todo.append(q)
    return seen


def validate(pack, station, network):
    w, h = station['dimensions_metatiles']
    rows = [[int(value, 16) for value in row.split()] for row in station['rows']]
    assert len(rows) == h and all(len(row) == w for row in rows), 'station row extent'
    blocks = sum(rows, [])
    collision = pack['data']['tilesets'][station['tileset']]['collision']
    def kind(q):
        x, y = q
        return collision[f'{blocks[y // 2 * w + x // 2]:02x}'][y % 2 * 2 + x % 2]
    cells = {(x, y) for y in range(h * 2) for x in range(w * 2)}
    walk = {q for q in cells if kind(q) in {'FLOOR', 'DOOR', 'UP_WALL', 'WARP_CARPET_DOWN'}}
    residents = station['residents']
    actors = {tuple(a['tile']) for a in residents}
    assert len(actors) == len(residents), 'overlapping station actors'
    start = tuple(station['public_arrival_core'])
    assert kind(start) == 'FLOOR' and start not in actors, 'unsafe public arrival'
    public = flood(start, walk - actors)
    boarding = station['boarding']
    platform = tuple(boarding['platform_core'])
    assert platform not in public, 'public bypass around conductor'
    for key in ['train_door_core', 'other_train_door_core']:
        door = tuple(boarding[key])
        assert kind(door) == 'DOOR' and door not in public, 'unsafe train door'
    conductor = tuple(boarding['conductor_core'])
    assert conductor in actors and tuple(boarding['approach_core']) in public, 'conductor approach'
    boarded = flood(start, walk - (actors - {conductor}))
    assert platform in boarded, 'opening the boarding throat must reach platform'
    for key in ['ride_arrival_core']:
        q = tuple(boarding[key])
        assert q in public and kind(q) == 'FLOOR', 'ride arrival belongs in public concourse'
    exits = {tuple(q) for q in station['exit_tiles_core']}
    assert exits <= public and all(kind(q) == 'WARP_CARPET_DOWN' for q in exits), 'public carpet exits'
    contract = station['warp_contract']
    assert contract['exit_warp_indices'] == [1,2,3,4] and len(exits)==4, 'source carpet warp order'
    assert contract['passive_arrival_warp_index']==5, 'source Floor arrival must follow all four carpets'
    assert contract['exterior_door_warp_index']==1 and contract['exterior_passive_return_warp_index']==2, 'source exterior warp order'
    outside = {q for q in public if q[0] in {0, w * 2 - 1} or q[1] in {0, h * 2 - 1}}
    assert outside == exits, 'unexpected public boundary opening'
    assert tuple(station['passive_return_core']) in public - exits, 'passive return retriggers exit'
    for actor in residents:
        q = tuple(actor['tile'])
        assert kind(q) == 'FLOOR', actor['id'] + ' must stand on Floor'
        assert any((q[0]+dx, q[1]+dy) in public for dx, dy in [(1,0),(-1,0),(0,1),(0,-1)]), actor['id'] + ' has no approach'
        assert all(0 < len(row) <= 18 for row in actor['lines']), 'native text width'
        name = actor['sprite'].removeprefix('SPRITE_').lower()
        assert f'gfx/sprites/{name}.png' in pack['runtime_files'], 'missing native sprite ' + name
    for view in station['viewpoints']:
        assert tuple(view['core']) in public, view['name'] + ' must be a playable camera position'
    for sign in station['signs']:
        x, y = sign['tile']
        assert kind((x,y)) == 'WALL', 'timetable must bind to solid native cabinet'
        assert any((x+dx,y+dy) in public for dx,dy in [(1,0),(-1,0),(0,1),(0,-1)]), 'inaccessible timetable'
        assert all(0 < len(line) <= 18 for line in sign['lines']), 'timetable native text width'
    stops = network['ordered_stops']
    assert [s['id'] for s in stops] == ['DETROIT_LAKES','STAPLES','ST_CLOUD','ST_PAUL','RED_WING','WINONA'], 'line order'
    assert len({s['visited_flag'] for s in stops}) == len(stops), 'shared visit flag'
    assert len({s['station'] for s in stops}) == len(stops), 'duplicate station binding'
    assert next(s for s in stops if s['id']=='ST_PAUL')['station'] == station['id'], 'St. Paul binding'
    return blocks, {'public_tiles':len(public), 'platform_locked':True, 'conductor_opening_reaches_platform':True,
                    'unbound_city_exterior':True, 'native_ride_verified':False}


def negative_checks(pack, station, network):
    cases = []
    changed = deepcopy(station)
    row = changed['rows'][4].split(); row[7]='07'; changed['rows'][4]=' '.join(row)
    cases.append(('second boarding throat', changed, network))
    changed = deepcopy(station); changed['boarding']['ride_arrival_core'] = [17,7]
    cases.append(('arrival behind conductor', changed, network))
    changed = deepcopy(station); changed['public_arrival_core'] = [17,9]
    cases.append(('arrival on conductor', changed, network))
    changed = deepcopy(network); changed['ordered_stops'][1]['visited_flag'] = changed['ordered_stops'][0]['visited_flag']
    cases.append(('shared visit flag', station, changed))
    for name, s, n in cases:
        try:
            validate(pack, s, n)
        except AssertionError:
            continue
        raise AssertionError('negative check accepted: ' + name)
    return [name for name, _, _ in cases]


def render(pack, station, network, output):
    started = time.perf_counter()
    blocks, checks = validate(pack, station, network)
    checks['rejected_mutations'] = negative_checks(pack, station, network)
    digest = hashlib.sha256(json.dumps([station, network], sort_keys=True).encode()).hexdigest()[:8]
    archive = output / digest
    archive.mkdir(parents=True, exist_ok=True)
    for name, source in [('station-source.json', station), ('network-source.json', network)]:
        payload = json.dumps(source, indent=2) + '\n'
        target = archive / name
        if target.exists() and target.read_text() != payload:
            raise ValueError('immutable source snapshot differs: ' + str(target))
        target.write_text(payload)
    w, h = station['dimensions_metatiles']
    terrain = SourceArt(pack).render(station['tileset'], w, h, blocks)
    save_render(terrain, archive / 'whole-terrain-original.png')
    scene = render_stage(terrain, 'convert', archive, 'scene-rgba', 'RGBA')
    for actor in station['residents']:
        sprite = oriented_sprite(pack, actor['sprite'], actor['facing'])
        scene.alpha_composite(sprite, (actor['tile'][0]*16, actor['tile'][1]*16-4))
    save_render(scene, archive / 'whole-station-rgba.png')
    scene = render_stage(scene, 'convert', archive, 'whole-rgb', 'RGB')
    save_render(scene, output / 'st-paul-union-depot.png')
    views = []
    for i, view in enumerate(station['viewpoints']):
        x, y = view['core']; cx, cy = x*16+8, y*16+8
        crop = render_stage(scene, 'crop', archive, f'camera-{i}', (cx-80,cy-72,cx+80,cy+72))
        player = oriented_sprite(pack, 'SPRITE_CHRIS', 'south')
        frame = render_stage(crop, 'convert', archive, f'camera-{i}-rgba', 'RGBA')
        frame.alpha_composite(player, (72,60))
        save_render(frame, archive / f'camera-{i}-160.png')
        preview = render_stage(frame, 'resize', archive, f'camera-{i}-2x', (320,288), Image.Resampling.NEAREST)
        views.append((view['name'], preview))
    sheet = Image.new('RGB', (660, len(views)//2*320+320), '#14212a')
    draw = ImageDraw.Draw(sheet)
    for i, (name, frame) in enumerate(views):
        x, y = i%2*330, i//2*320
        draw.text((x+5,y+5), name, fill='white'); sheet.paste(frame,(x+5,y+25))
    save_render(sheet, archive / 'player-views.png'); save_render(sheet, output / 'st-paul-player-views.png')
    report = dict(source_hash=digest, status='SOURCE ONLY; exterior and native ride unbound', checks=checks,
                  elapsed_seconds=round(time.perf_counter()-started,3))
    (archive/'review.json').write_text(json.dumps(report, indent=2)+'\n')
    print(json.dumps(report, indent=2))


if __name__ == '__main__':
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--pack', type=Path, default=ROOT/'content-packs/minnesota-quality-review.crystalpack')
    parser.add_argument('--output', type=Path, default=OUTPUT)
    args=parser.parse_args()
    render(load_pack(args.pack), json.loads(STATION.read_text()), json.loads(NETWORK.read_text()), args.output)
