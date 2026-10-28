#!/usr/bin/env python3
"""Review InternationalFalls's explicit native town and both room contracts, without export."""
import argparse
from copy import deepcopy
import json
from pathlib import Path
import time

from render_archive import save_render
from render_reference import SourceArt, layout_blocks, load_pack, room_sheet
from render_stages import render_stage
from review_boundary_basecamp import adjacent, flood
from review_north_shore import oriented_sprite

ROOT = Path(__file__).resolve().parents[3]
TOWN = ROOT / 'planning/minnesota/towns/MnInternationalFalls.json'
LAYOUT = ROOT / 'planning/minnesota/towns/MnInternationalFalls.layout.json'
OUTPUT = ROOT / 'planning/minnesota/renders/international-falls'


def validate(town, layout, pack):
    assert town['id'] == layout['id'] == 'MnInternationalFalls'
    assert town['dimensions_metatiles'] == layout['dimensions_metatiles'] == [16, 14]
    assert town['compiled_tileset'] == layout['compiled_tileset'] == 'mn_international_falls'
    assert not layout['source_pixels_modified']
    assert [(b['id'],b['source_tileset'],b['source_block']) for b in layout['derived_blocks']] == [('80','mn_winter_outdoor','3c'),('81','mn_winter_outdoor','3f')]
    doors = {tuple(d['tile']) for d in town['doors']}
    assert len(doors) == 2 and len(town['interiors']) == 2
    assert {d['target'] for d in town['doors']} == {r['id'] for r in town['interiors']}
    assert [v for d in town['doors'] for v in (d['warp_index'], d['return_warp_index'])] == [1, 2, 3, 4]
    q = town['quest_contract']
    assert q['id'] == 'SQ03' and q['ranger'] == 'MN_FALLS_RANGER'
    assert q['reward_item'] == 'PP_UP' and q['reward_quantity'] == 1
    assert q['outlet_requires'] == ['BADGE_'+str(i) for i in range(1,9)] + ['EVENT_MN_BORDER_REPORT','EVENT_MN_LIGHTHOUSE_SIGNAL']
    assert q['arrival_mode'] == 'MAPCALLBACK_NEWMAP'
    assert q['arrival_map'] == 'MnBoundaryWaters' and q['arrival_tiles'] == [[0,24],[0,25]] and q['arrival_facing'] == 'RIGHT'
    assert all(0 < len(line) <= 18 for k,rows in q.items() if k.endswith('_lines') for line in rows)
    maps = [(town, layout_blocks(layout), (20, 25))]
    maps += [(r, [int(b, 16) for row in r['rows'] for b in row.split()], tuple(r['passive_arrival']['tile'])) for r in town['interiors']]
    stats = []
    for plan, blocks, start in maps:
        w, h = plan['dimensions_metatiles']
        assert len(blocks) == w*h, plan['id']+' wrong row width/extent'
        tileset = 'johto_modern' if plan is town else plan['tileset']
        cs = deepcopy(pack['data']['tilesets'][tileset]['collision'])
        if plan is town:
            cs.update({b['id']:b['collision'] for b in layout['derived_blocks']})
        def kind(p):
            x, y = p
            return cs[f'{blocks[y//2*w+x//2]:02x}'][y % 2*2+x % 2]
        cells = {(x, y) for y in range(h*2) for x in range(w*2)}
        bodies = {tuple(a['tile']) for a in plan['residents']}
        assert len(bodies) == len(plan['residents']) and start not in bodies
        floor = {p for p in cells if kind(p) in {'FLOOR', 'DOOR', 'WARP_CARPET_DOWN'}}
        water = {p for p in cells if kind(p) == 'WATER'}
        assert kind(start) == 'FLOOR', plan['id']+' unsafe arrival'
        seen = flood(start, floor-bodies, doors if plan is town else set())
        surf = flood(start, (floor | water)-bodies, doors if plan is town else set())
        for actor in plan['residents']:
            at, approach = tuple(actor['tile']), tuple(actor['approach'])
            assert kind(at) == 'FLOOR' and approach in seen, actor['id']+' blocked actor/approach'
            if 'counter_tile' in actor:
                counter = tuple(actor['counter_tile'])
                assert kind(counter) == 'COUNTER', actor['id']+' missing native counter'
                assert tuple(at[i]+approach[i] for i in range(2)) == tuple(2*v for v in counter)
            assert all(0 < len(line) <= 18 for line in actor['lines']), actor['id']+' text width'
        for sign in plan.get('signs', []):
            at = tuple(sign['tile'])
            assert kind(at) == 'WALL' and set(adjacent(at)) & seen, sign['id']+' blocked placard'
        if plan is town:
            assert {p for p in cells if kind(p) == 'DOOR'} == doors, 'false or missing facade DOOR'
            for d in town['doors']:
                assert tuple(d['tile']) in seen and tuple(d['return_tile']) in seen
                assert kind(tuple(d['return_tile'])) == 'FLOOR' and d['target_warp'] == 3
                assert d['return_tile'] == [d['tile'][0], d['tile'][1]+1]
            edge = {p for p in floor if p[0] in {0, 31} or p[1] in {0, 27}}
            assert edge == {(20,27),(21,27),(31,14),(31,15)}, ('unplanned walking edge', edge)
            assert not surf & water, 'town must not expose unbound Surf water'
            assert all(kind(p) != 'CUT_TREE' for p in cells), 'incidental Cut canopy'
            assert [(p['route'],p['side'])for p in town['ports']] == [('MN_R17','S'),('MN_R20','E')]
            for view in town['viewpoints']:
                assert tuple(view['tile']) in seen and kind(tuple(view['tile'])) == 'FLOOR', view['id']+' occupied or blocked view'
        else:
            assert len(blocks) == w*h and len(plan['rows']) == h
            assert all(len(row.split()) == w for row in plan['rows'])
            carpets = {p for p in cells if kind(p) == 'WARP_CARPET_DOWN'}
            assert carpets == {tuple(e['tile']) for e in plan['exits']} and len(carpets) == 2
            assert carpets <= seen and plan['passive_arrival']['index'] == 3
            d = next(d for d in town['doors'] if d['target'] == plan['id'])
            assert all(e['target'] == town['id'] and e['target_warp'] == d['return_warp_index'] for e in plan['exits'])
            if 'pc' in plan:
                assert kind(tuple(plan['pc']['tile'])) == 'PC' and tuple(plan['pc']['approach']) in seen
        stats.append((plan['id'], len(seen), len(water & surf)))
    return stats


def negative_checks(town, layout, pack):
    cases = []
    def reject(name, mutate):
        t, l = deepcopy(town), deepcopy(layout)
        mutate(t, l)
        try:
            validate(t, l, pack)
        except AssertionError:
            cases.append(name)
            return
        raise AssertionError('unsafe source accepted: '+name)
    reject('false facade', lambda t, l: t['doors'][0].update(tile=[16, 7]))
    reject('actor-blocked arrival', lambda t, l: t['interiors'][0]['passive_arrival'].update(tile=[3, 1]))
    reject('wrong carpet return', lambda t, l: t['interiors'][1]['exits'][0].update(target_warp=1))
    reject('non-counter care', lambda t, l: t['interiors'][0]['residents'][0].update(counter_tile=[3, 3]))
    reject('blocked PC', lambda t, l: t['interiors'][0]['pc'].update(approach=[9, 1]))
    reject('extra reward', lambda t, l: t['quest_contract'].update(reward_quantity=2))
    reject('arrival shortcut', lambda t, l: t['quest_contract'].update(arrival_mode='coordinate_event'))
    reject('wrong-way arrival', lambda t, l: t['quest_contract'].update(arrival_facing='LEFT'))
    reject('missing eighth badge', lambda t, l: t['quest_contract']['outlet_requires'].pop(7))
    reject('unbound walking edge', lambda t, l: l['layers'].append(dict(name='invalid north exit', kind='point', position=[9, 0], block='80')))
    reject('unbound river Surf', lambda t, l: l['layers'].extend([
        dict(name='invalid bank gap', kind='point', position=[4, 6], block='35'),
        dict(name='invalid rail gap', kind='point', position=[5, 6], block='80')]))
    return cases


def render(town, layout, pack):
    started = time.perf_counter()
    stats = validate(town, layout, pack)
    art = SourceArt(pack)
    room_images = []
    for plan in [town, *town['interiors']]:
        w, h = plan['dimensions_metatiles']
        blocks = layout_blocks(layout) if plan is town else [int(b, 16) for row in plan['rows'] for b in row.split()]
        tileset = 'johto_modern' if plan is town else plan['tileset']
        terrain = art.render(tileset, w, h, blocks, group='day' if plan is town else 'indoor', derived=layout['derived_blocks'] if plan is town else None)
        installed = pack['data']['maps'].get(plan['id'])
        if installed:
            assert installed['blocks'] == blocks, plan['id']+' installed geometry differs'
            actual = art.render(installed['attributes']['tileset_name'],w,h,blocks,
                                group='day' if plan is town else 'indoor')
            assert actual.tobytes() == terrain.tobytes(), plan['id']+' installed native colors differ'
        save_render(terrain, OUTPUT / (plan['id']+'-terrain-original.png'))
        scene = render_stage(terrain, 'convert', OUTPUT, plan['id']+'-scene', 'RGBA')
        for actor in plan['residents']:
            scene.alpha_composite(oriented_sprite(pack, actor['sprite'], actor['facing']),
                                  (actor['tile'][0]*16, actor['tile'][1]*16))
        save_render(scene, OUTPUT / (plan['id']+'-whole-rgba.png'))
        rgb = render_stage(scene, 'convert', OUTPUT, plan['id']+'-rgb', 'RGB')
        save_render(rgb, OUTPUT / (plan['id']+'-whole.png'))
        if plan is not town:
            room_images.append((plan['id'], rgb))
        views = []
        for view in plan.get('viewpoints', []):
            x, y = view['tile']
            bx, by = max(0, min(x*16-80, w*32-160)), max(0, min(y*16-72, h*32-144))
            crop = render_stage(scene, 'crop', OUTPUT, plan['id']+'-'+view['id'], (bx, by, bx+160, by+144))
            save_render(crop, OUTPUT / (plan['id']+'-'+view['id']+'-native160.png'))
            views.append((view['id'], crop))
        save_render(room_sheet(views, 3), OUTPUT / (plan['id']+'-views.png'))
    save_render(room_sheet(room_images, 2), OUTPUT / 'rooms.png')
    return dict(source_only=True, maps=stats, rejected=negative_checks(town, layout, pack),
                render_ms=round((time.perf_counter()-started)*1000, 1))


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--pack', type=Path, default=ROOT/'content-packs/minnesota-moorhead-v01.crystalpack')
    args = parser.parse_args()
    pack = load_pack(args.pack)
    print(json.dumps(render(json.loads(TOWN.read_text()), json.loads(LAYOUT.read_text()), pack)))


if __name__ == '__main__':
    main()
