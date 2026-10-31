#!/usr/bin/env python3
"""Read-only complete-parent, actor-aware floor and one-way lip audit."""
import argparse
from collections import deque
from copy import deepcopy
import hashlib
import json
from pathlib import Path
import sys

sys.path.insert(0, str(Path(__file__).resolve().parents[2] / 'tools'))
from render_reference import load_pack

cli = argparse.ArgumentParser(description=__doc__)
for key in ['parent', 'pack', 'source', 'output']:
    cli.add_argument('--' + key, type=Path, required=True)
a = cli.parse_args()
assert not a.output.exists(), 'retain every audit'
p, q = load_pack(a.parent), load_pack(a.pack)
before, after = p['data'], q['data']
room = 'MnIronDrillApproach'
source = json.loads(a.source.read_text())
assert before.keys() == after.keys()
for key in before:
    if key not in {'maps', 'map_blocks'}:
        assert before[key] == after[key], key
assert before['maps'].keys() == after['maps'].keys()
for name, module in before['maps'].items():
    expected = deepcopy(module)
    if name == room:
        expected['blocks'] = [int(v, 16) for row in source['rows'] for v in row.split()]
    assert after['maps'][name] == expected, name
assert before['map_blocks'].keys() == after['map_blocks'].keys()
for name, data in before['map_blocks'].items():
    if name != room + '_Blocks':
        assert after['map_blocks'][name] == data, name
assert p['runtime_files'] == q['runtime_files']
assert p['audio_manifest'] == q['audio_manifest']
assert p['compiled_audio'] == q['compiled_audio']
for name, value in p['report'].items():
    assert q['report'][name] == (value + ['minnesota-iron-crosscut-bends-v1'] if name == 'manifests' else value), name
m = after['maps'][room]
collision = after['tilesets']['cave']['collision']
def kind(at):
    x, y = at
    return collision[f"{m['blocks'][y//2*16+x//2]:02x}"][y % 2 * 2 + x % 2]
cells = {(x, y) for y in range(26) for x in range(32)}
assert all(kind((x,y)) == 'WALL' for x,y in cells if x in {0,31} or y in {0,25})
assert {q for q in cells if kind(q) == 'LADDER'} == {(7,21),(25,5)}
assert {q for q in cells if kind(q).startswith('HOP')} == {(13,18),(13,19)}
assert not any(kind(q) in {'WATER','PIT','CAVE','WARP_CARPET_DOWN'} for q in cells)
def neighbors(at):
    x, y = at
    return [(x-1,y),(x+1,y),(x,y-1),(x,y+1)]
def distances(start, floor):
    queue, seen = deque([start]), {start: 0}
    while queue:
        p = queue.popleft()
        for n in neighbors(p):
            if n in floor and n not in seen:
                seen[n] = seen[p] + 1
                queue.append(n)
    return seen
def return_commands(start, floor, allow_lip):
    queue, seen = deque([start]), {start:0}
    lips = {(13,18),(13,19)}
    while queue:
        at = queue.popleft()
        if at == (6,21):
            return seen[at]
        choices = neighbors(at)
        if allow_lip and at in lips:
            choices.append((at[0]-2,at[1]))
        for n in choices:
            if n not in seen and (n in floor or allow_lip and n in lips):
                seen[n] = seen[at]+1
                queue.append(n)
    raise AssertionError('activity return is disconnected')
states = []
for collected in [False, True]:
    bodies = {(o['x'],o['y']) for o in m['objects']
              if not (collected and o['script'] == 'MN_IRON_APPROACH_ETHER')}
    floor = {p for p in cells if kind(p) == 'FLOOR'} - bodies
    reached = distances((6,21), floor)
    assert set(reached) == floor, sorted(floor - set(reached))
    for p in [(4,5),(6,8),(6,9),(9,13),(10,13),(15,10),(19,13),(24,5),(25,18),(14,18),(14,19)]:
        assert p in reached, p
    returns = []
    for y in [18,19]:
        assert kind((13,y)) == 'HOP_LEFT' and kind((12,y)) == 'WALL'
        assert (11,y) in floor and (14,y) in floor
        reverse = distances((11,y), floor)[14,y]
        assert reverse >= 24
        returns.append(reverse)
    states.append(dict(cache_collected=collected, reachable_floor=len(floor),
                       entry_to_upper_passive_steps=reached[24,5],
                       ordinary_lip_return_steps=returns,
                       spoil_to_entry_ordinary_steps=return_commands((25,18),floor,False),
                       spoil_to_entry_with_lip_commands=return_commands((25,18),floor,True)))
assert m['blocks'][4*16+3] == 0x30
for x,y,block in [(6,9,0x3b),(6,8,0x2e),(6,10,0x2e),(5,9,2),(7,9,2)]:
    assert m['blocks'][y*16+x] == block
sha = lambda p: hashlib.sha256(p.read_bytes()).hexdigest()
receipt = dict(status='PASS', parent=str(a.parent), parent_sha256=sha(a.parent),
               pack=str(a.pack), pack_sha256=sha(a.pack), content_identity=q['identity']['content_hash'],
               source_sha256=sha(a.source), changed_map=room, unchanged_complete_maps=len(before['maps'])-1,
               original_programs_actors_warps_trainers_wildlife_and_all_assets_exact=True,
               cache_body_states=states, native_lip_is_standalone_and_has_blocked_reverse_and_sides=True,
               scope='Complete installed native map and source graphs; actual movement/battles/items and native visual review are separate.')
a.output.write_text(json.dumps(receipt, indent=2) + '\n')
print('PASS', receipt['unchanged_complete_maps'], 'other maps;', states)
