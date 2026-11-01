#!/usr/bin/env python3
"""Explicit Ely ridge edit and native-source proof; never writes a pack."""
import argparse
from collections import deque
from copy import deepcopy
import heapq
import json
from pathlib import Path
import sys

from PIL import Image

sys.path.insert(0, str(Path(__file__).resolve().parent))
import review
from audit_outdoor_connections import collision
from render_archive import save_render
from render_reference import layout_blocks, load_pack, room_sheet
from review_major_city import ReviewArt, player_camera, render_map

DIR = {'Up': (0, -1), 'Down': (0, 1), 'Left': (-1, 0), 'Right': (1, 0)}
WALK = {'FLOOR', 'TALL_GRASS', 'ICE', 'HOP_DOWN'}


def edges(kinds, point, bodies):
    for button, (dx, dy) in DIR.items():
        target = (point[0] + dx, point[1] + dy)
        if kinds.get(point) == 'HOP_DOWN' and button == 'Down':
            target = (point[0], point[1] + 2)
            if kinds.get(target) not in {'FLOOR', 'TALL_GRASS'} or target in bodies:
                continue
            yield button, target, 1
            continue
        if kinds.get(target) not in WALK or target in bodies:
            continue
        # Native ice retains its incoming direction until dry ground or a wall.
        while kinds.get(target) == 'ICE':
            nxt = (target[0] + dx, target[1] + dy)
            if kinds.get(nxt) not in WALK or nxt in bodies:
                break
            target = nxt
        yield button, target, 1


def path(kinds, start, target, bodies, grass_first=False):
    start, target = tuple(start), tuple(target)
    pending = [(0, 0, start)]
    costs = {start: (0, 0)}
    previous = {}
    while pending:
        primary, secondary, at = heapq.heappop(pending)
        if costs.get(at) != (primary, secondary):
            continue
        if at == target:
            course = []
            while at != start:
                old, button = previous[at]
                course.append({'from': old, 'button': button, 'to': at})
                at = old
            return list(reversed(course))
        for button, nxt, count in edges(kinds, at, bodies):
            grass = int(kinds[nxt] == 'TALL_GRASS')
            cost = (primary + (grass if grass_first else count), secondary + (count if grass_first else grass))
            if cost < costs.get(nxt, (10**9, 10**9)):
                costs[nxt] = cost
                previous[nxt] = at, button
                heapq.heappush(pending, (*cost, nxt))
    return None


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--epoch', required=True)
    args = parser.parse_args()
    assert '/' not in args.epoch and args.epoch not in {'.', '..'}
    source = Path(__file__).resolve().parent / args.epoch
    output = review.ROOT / 'planning/minnesota/renders/r13-solo-quality-v01' / args.epoch
    assert not source.exists() and not output.exists(), 'Retain every epoch'
    assert review.sha(review.PACK.read_bytes()) == review.PACK_SHA
    base = load_pack(review.PACK)
    original, objects, placements = review.inland.whole(base, 'MnR13')
    candidate = deepcopy(original)
    pack = deepcopy(base)
    alias = original['attributes']['tileset_name']
    recipe_path = f'data/tilesets/{alias}_metatiles.bin'
    recipes = bytes(pack['runtime_files'][recipe_path])
    new_block = len(recipes) // 16
    assert new_block == 0x8c, 'Reviewed unused snow ledge slot changed'
    # Keep the authentic native brown lip, place native snow in its three
    # upper glyph rows. No pixel editing, new glyph or palette is required.
    snow_ledge = recipes[0x83*16:0x83*16+12] + recipes[0x56*16+12:0x57*16]
    pack['runtime_files'][recipe_path] = recipes + snow_ledge
    pack['data']['tilesets'][alias]['collision']['8c'] = list(pack['data']['tilesets'][alias]['collision']['56'])
    assert pack['data']['tilesets'][alias]['collision']['8c'] == ['HOP_DOWN', 'HOP_DOWN', 'WALL', 'WALL']
    edits = {}

    def place(x, y, after, reason):
        index = y*32+x
        before = original['blocks'][index]
        assert (x, y) not in edits
        assert before in {0x83, 0x8b}, (x, y, hex(before), reason)
        if before != after:
            edits[x, y] = {'metatile': [x, y], 'before': before, 'after': after, 'reason': reason}
            candidate['blocks'][index] = after

    # One fir spur ties into the existing granite ridge, framing the drop.
    for x, y in [(12,40),(12,41),(13,41),(15,41)]:
        place(x, y, 0x8b, 'Granite-foot pine spur closes the immediate reverse bypass')
    for x, y in [(11,39),(11,40),(11,41),(11,42)]:
        place(x, y, 0x84, 'Short snowy grass ascent around the pine spur')
    place(14, 41, new_block, 'Two-lane downward lip with a dry landing')
    for x, y in [(14,44),(14,45),(14,46),(15,46)]:
        place(x, y, 0x83, 'Narrow lower passage joins the drop to the southbound trail')
    # An asymmetric fir elbow removes the second empty snow plaza, leaving
    # the refuge branch and the ordinary southward road intact.
    for x, y in [(17,48),(18,48),(19,48),(20,48),(18,49)]:
        place(x, y, 0x8b, 'South clearing becomes a winding fir-lined approach')

    old_kinds, kinds = collision(base, original), collision(pack, candidate)
    bodies = {tuple(o['tile']) for o in objects}
    for actor in objects:
        assert kinds[tuple(actor['tile'])] == old_kinds[tuple(actor['tile'])], actor
    # No perimeter, water, ice course, Cut, Headbutt, cliff face or original
    # ledge changes. Changed empty snow/tree cells are explicitly enumerated.
    for at, token in old_kinds.items():
        if at[0] in {0,63} or at[1] in {0,127} or token not in {'FLOOR','WALL'}:
            assert kinds[at] == token, (at, token, kinds[at])
    start, finish = (30,0), (34,127)
    courses = {}
    for name, a, b in [('north-to-ely',start,finish),('ely-to-north',finish,start),
                        ('new-hop-approach',(28,81),(28,82)),
                        ('grassy-uphill-return',(28,84),(28,81))]:
        course = path(kinds, a, b, bodies)
        assert course is not None, ('Disconnected authored course', name)
        courses[name] = {'from_core': a, 'to_core': b, 'inputs': course,
            'input_count': len(course), 'grass_steps': sum(kinds[tuple(s['to'])]=='TALL_GRASS' for s in course)}
    assert kinds[28,82] == kinds[29,82] == 'HOP_DOWN'
    assert kinds[28,83] == kinds[29,83] == kinds[27,82] == kinds[30,82] == 'WALL'
    assert kinds[28,84] == kinds[29,84] == 'FLOOR'
    assert courses['grassy-uphill-return']['input_count'] >= 12
    # The short return must use grass. Preserve the old outer route as a real
    # alternative, but measure its cost instead of calling all grass mandatory.
    cheapest = path(kinds, (28,84), (28,81), bodies, grass_first=True)
    assert courses['grassy-uphill-return']['grass_steps'] >= 4
    assert cheapest and len(cheapest) >= courses['grassy-uphill-return']['input_count'] + 24
    approaches = []
    for actor in objects:
        x, y = actor['tile']
        choices = [(x+dx,y+dy) for dx,dy in DIR.values() if kinds.get((x+dx,y+dy)) in WALK and (x+dx,y+dy) not in bodies]
        accessible = [p for p in choices if path(kinds, start, p, bodies) is not None]
        # The Cut refuge's original trainer/item remain intentionally gated.
        was_accessible = any(path(old_kinds, start, p, bodies) is not None for p in choices)
        assert not was_accessible or accessible, ('Lost actor approach',actor['id'])
        approaches.append({'actor': actor['id'], 'before_reachable': was_accessible, 'after_approaches': accessible})
    source.mkdir(parents=True)
    output.mkdir(parents=True)
    layout_path = review.ROOT/'planning/minnesota/routes/MN_R13.layout.json'
    dossier_path = review.ROOT/'planning/minnesota/routes/MN_R13.json'
    layout = json.loads(layout_path.read_text())
    assert layout_blocks(layout) == original['blocks'], 'Canonical/native R13 has drifted'
    for cell in edits.values():
        layout['layers'].append({'kind':'point','name':cell['reason'],
                                 'position':cell['metatile'],'block':f"{cell['after']:02x}"})
    assert layout_blocks(layout) == candidate['blocks']
    (source/'MN_R13.layout.candidate.json').write_text(json.dumps(layout,indent=2)+'\n')
    for original_path in [layout_path,dossier_path]:
        (source/(original_path.name+'.before')).write_bytes(original_path.read_bytes())
    contract = {'schema_version':1,'status':'SOURCE_REVIEW_NOT_NATIVE_PUBLICATION',
        'parent_pack_sha256':review.PACK_SHA,'parent_identity':base['identity']['content_hash'],
        'map_prefix':'MnR13','alias':alias,'edits':list(edits.values()),
        'appended_recipe':{'block':new_block,'source_block':0x56,'snow_block':0x83,
                           'snow_glyph_rows':3,'glyphs':list(snow_ledge),'collision':['HOP_DOWN','HOP_DOWN','WALL','WALL']},
        'modules':[{ 'map':p['map'],'sha256':review.sha(json.dumps(base['data']['maps'][p['map']],sort_keys=True,separators=(',',':'),ensure_ascii=False).encode())}for p in placements],
        'source_preimages':{str(p.relative_to(review.ROOT)):review.sha(p.read_bytes())for p in [layout_path,dossier_path]}}
    (source/'native.contract.json').write_text(json.dumps(contract,indent=2)+'\n')
    (source/'input-plan.json').write_text(json.dumps(courses,indent=2)+'\n')
    art = ReviewArt(pack)
    paired = []
    for label, m in [('before',original),('proposal',candidate)]:
        ground, scene, _ = render_map(art,m,objects,'day',output/label)
        paired.append((label,scene.resize((512,1024),Image.Resampling.NEAREST)))
        frames=[]
        for name, core in [('upper-ridge',[28,80]),('drop-lip',[28,82]),
                           ('lower-landing',[28,84]),('grassy-return',[22,82]),
                           ('southern-elbow',[33,97]),('refuge',[18,102])]:
            assert kinds[tuple(core)] in WALK and tuple(core) not in bodies
            frame,_=player_camera(art,m,objects,ground,scene,'day',{'core':core},output/label/'views'/name)
            frames.append((name,frame.resize((480,432),Image.Resampling.NEAREST)))
        save_render(room_sheet(frames,3),output/f'{label}-players.png')
    save_render(room_sheet(paired,2),output/'whole-before-proposal.png')
    proof={'status':'STATIC_SOURCE_INPUTS_NOT_EXECUTED','changed_metatiles':len(edits),
           'new_ledge':{'approach':[28,81],'lip':[28,82],'landing':[28,84],'reverse_and_sides_blocked':True},
           'short_return_grass':courses['grassy-uphill-return']['grass_steps'],
           'dry_detour_inputs':len(cheapest),
           'dry_detour_extra_inputs':len(cheapest)-courses['grassy-uphill-return']['input_count'],
           'actor_approaches':approaches,'courses':{k:{n:v for n,v in row.items()if n!='inputs'}for k,row in courses.items()}}
    (output/'proof.json').write_text(json.dumps(proof,indent=2)+'\n')
    (output/'propose.snapshot.py').write_bytes(Path(__file__).read_bytes())
    print(json.dumps(proof,indent=2))


if __name__=='__main__':
    main()
