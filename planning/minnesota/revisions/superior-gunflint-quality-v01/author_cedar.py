#!/usr/bin/env python3
"""Detached short snowline approach; source composition only, never pack writing."""
import copy
import hashlib
import json
import sys
from collections import deque
from pathlib import Path

ROOT = Path(__file__).resolve().parents[4]
sys.path.insert(0, str(ROOT / 'planning/minnesota/tools'))
from render_reference import load_pack, room_sheet
from review_major_city import ReviewArt, render_map, player_camera


def main():
    epoch = sys.argv[1]
    source = Path(__file__).resolve().parent / epoch
    output = ROOT / 'planning/minnesota/renders/superior-gunflint-quality-v01' / epoch
    assert not source.exists() and not output.exists(), 'immutable epoch required'
    source.mkdir(parents=True)
    output.mkdir(parents=True)
    parent = ROOT / 'content-packs/minnesota-superior-inland-winter-v01.crystalpack'
    digest = hashlib.sha256(parent.read_bytes()).hexdigest()
    assert digest == '65cf6f98e65ada839ec5afaa57728bf8289d3a50ba53ba6b9b50f08d3620bb17'
    pack = load_pack(parent)
    base = pack['data']['maps']['MnMaraisCedarApproach']
    width, height = 20, 12
    cells = [0x8b] * (width * height)

    def put(x, y, block):
        cells[y * width + x] = block

    def path(points):
        for (x, y), (xx, yy) in zip(points, points[1:]):
            assert x == xx or y == yy
            for u in range(min(x, xx), max(x, xx) + 1):
                for v in range(min(y, yy), max(y, yy) + 1):
                    put(u, v, 0x83 if v < 10 else 0x01)

    # A short forest walk, with one little occupied shoulder at each bend.
    # These explicit paths are terrain authoring, not a route generator.
    path([(0, 7), (3, 7), (3, 5), (7, 5), (7, 7), (13, 7), (13, 11)])
    path([(13, 7), (16, 7), (16, 10), (13, 10)])
    for x, y in [(2, 6), (6, 4), (7, 4), (8, 5), (14, 6), (15, 6), (15, 7)]:
        put(x, y, 0x83)
    # Required grass across the short westward trip. The ledge return also
    # crosses grass; the drop is useful, and its reverse has an ordinary path.
    put(5, 5, 0x84)
    put(6, 5, 0x84)
    put(16, 8, 0x84)
    put(16, 9, 0x84)
    put(13, 9, 0x56)
    put(2, 6, 0x88)
    # The southern edge opens into warm town. Only the real connection opens.
    for y in [10, 11]:
        for x in range(width):
            if cells[y * width + x] == 0x8b:
                put(x, y, 0x05)
    module = copy.deepcopy(base)
    module['blocks'] = cells
    module['attributes'].update(width=width, height=height, border_block=0x8b,
        connections=[dict(direction='west', target_map='MnR12C00R01', offset=4),
                     dict(direction='south', target_map='MnGrandMarais', offset=4)])
    anchors = {'MN_COHESION_MARAIS_KEEPER': [15, 11],
               'MN_COHESION_MARAIS_WALKER': [31, 15]}
    objects = []
    for obj in module['objects']:
        at = anchors[obj['object_identifier']]
        obj['x'], obj['y'] = at
        objects.append(dict(id=obj['object_identifier'], tile=at,
                            sprite=obj['sprite'], facing='south'))
    collision = pack['data']['tilesets']['mn_winter_north_shore']['collision']
    kind = {(x, y): collision[f'{cells[y//2*width+x//2]:02x}'][y%2*2+x%2]
            for y in range(height*2) for x in range(width*2)}
    occupied = {tuple(o['tile']) for o in objects}
    dry = {p for p, k in kind.items() if k in {'FLOOR', 'TALL_GRASS'} and p not in occupied}
    def flood(start):
        reached, queue = {start}, deque([start])
        while queue:
            x, y = queue.popleft()
            for at in [(x-1,y), (x+1,y), (x,y-1), (x,y+1)]:
                if at in dry and at not in reached:
                    reached.add(at)
                    queue.append(at)
        return reached
    reached = flood((0, 14))
    assert (26, 23) in reached, 'ordinary return must connect without using hop'
    outer = {p for p in dry if p[0] in {0,39} or p[1] in {0,23}}
    assert outer == {(0,14),(0,15),(26,23),(27,23)}, outer
    assert all(any((o['tile'][0]+dx,o['tile'][1]+dy) in reached
                   for dx,dy in [(1,0),(-1,0),(0,1),(0,-1)]) for o in objects)
    layers = [dict(kind='point', name='Explicit short snowline cell',
                   position=[x,y], block=f'{cells[y*width+x]:02x}')
              for y in range(height) for x in range(width)]
    layout = dict(schema_version=2, id='MnMaraisCedarApproach', tileset='johto',
                  compiled_tileset='mn_winter_north_shore', dimensions_metatiles=[20,12],
                  shore_fitting='explicit_source_edges', layers=layers)
    dossier = json.loads((ROOT/'planning/minnesota/towns/MnMaraisCedarApproach.json').read_text())
    for actor in dossier['actors']:
        actor['tile'] = anchors[actor['id']]
    dossier.update(status='DETACHED_SOURCE_REVIEW', dimensions_metatiles=[20,12],
        ports=[dict(side='west', core=[[0,14],[0,15]]), dict(side='south', core=[[26,23],[27,23]])],
        native_connections=module['attributes']['connections'],
        signs=[dict(dossier['signs'][0], tile=[4,12])],
        views=[dict(id=n,core=at) for n,at in [('west-arrival',[5,14]),('grass-turn',[14,10]),('ledge-choice',[26,16]),('south-arrival',[26,21])]],
        ledge=dict(block=[13,9], upper_core=[26,17], jump_core=[26,18], landing_core=[26,20],
                   reverse_blocked=True, ordinary_return_metatiles=[[13,10],[16,10],[16,7],[13,7]]))
    for stale in ['native_geography_publication','geography_contract','sign_direction_corrections']:
        dossier.pop(stale, None)
    receipt = dict(status='DETACHED_NOT_NATIVE_PUBLISHED', parent_sha256=digest,
        dimensions=[20,12], origin=[222,-412], actor_anchors=anchors,
        sign_anchor=[4,12], native_connections=module['attributes']['connections'],
        rows=[' '.join(f'{v:02x}' for v in cells[y*width:(y+1)*width]) for y in range(height)],
        static_checks=dict(ordinary_return=True, actor_approaches=True, exact_closed_perimeter=True),
        limitations=['Hop direction, encounters and dialogue require production controller verification.'])
    for name, value in [('MnMaraisCedarApproach.candidate.layout.json',layout),
                        ('MnMaraisCedarApproach.candidate.json',dossier),('contract.json',receipt)]:
        (source/name).write_text(json.dumps(value,indent=2)+'\n')
    for path_name in ['MnMaraisCedarApproach.layout.json','MnMaraisCedarApproach.json']:
        (source/(path_name+'.before')).write_bytes((ROOT/'planning/minnesota/towns'/path_name).read_bytes())
    art = ReviewArt(pack)
    ground, scene, _ = render_map(art,module,objects,'day',output/'whole')
    scene.save(output/'whole-original.png')
    cameras = []
    for view in dossier['views']:
        frame,_ = player_camera(art,module,objects,ground,scene,'day',view,output/'cameras'/view['id'])
        cameras.append((view['id'],frame))
    room_sheet(cameras,2).save(output/'player-contact.png')
    print(output)


if __name__ == '__main__':
    main()
