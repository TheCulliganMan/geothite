#!/usr/bin/env python3
"""Read-only exact-parent and complete survey-cave geometry/art review.

Only the cave block array, its encoded payload and identity may differ.
Static geometry and native pixels do not substitute for controller/MCP play.
"""
import argparse
import base64
from collections import deque
import hashlib
import json
from pathlib import Path
import sys

ROOT = Path(__file__).resolve().parents[4]
sys.path.insert(0, str(ROOT / 'planning/minnesota/tools'))
from audit_regional_species_policy import audit as roster_audit
from render_archive import save_render
from render_reference import SourceArt, load_pack, npc_palette_pack, room_sheet
from review_passage_native import actors

ROOM = 'MnR10Cave1F'


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def path(floor, start, target):
    todo, distances = deque([start]), {start: 0}
    while todo:
        x, y = todo.popleft()
        if (x, y) == target:
            return distances[x, y]
        for at in [(x-1, y), (x+1, y), (x, y-1), (x, y+1)]:
            if at in floor and at not in distances:
                distances[at] = distances[x, y] + 1
                todo.append(at)
    return None


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for key in ['parent', 'pack', 'output']:
        parser.add_argument('--' + key, type=Path, required=True)
    args = parser.parse_args()
    args.output.mkdir(parents=True, exist_ok=False)
    directory = Path(__file__).parent
    before = json.loads((directory / 'before.json').read_text())
    after = json.loads((directory / 'after.json').read_text())
    contract = json.loads((directory / 'contract.json').read_text())
    assert sha(args.parent) == contract['parent_pack_sha256']
    parent, child = load_pack(args.parent), load_pack(args.pack)
    old, current = parent['data']['maps'][ROOM], child['data']['maps'][ROOM]
    decode = lambda s: [int(word, 16) for row in s['rows'] for word in row.split()]
    assert decode(before) == old['blocks'] and decode(after) == current['blocks']
    for field in ['rows', 'purpose', 'candidate_status', 'review_contract']:
        before.pop(field, None)
        after.pop(field, None)
    assert before == after, 'retained authoring field changed'
    revised = dict(old, blocks=current['blocks'])
    assert revised == current, 'event, actor, program, reward, palette or other module field changed'
    expected = dict(parent['data'])
    expected['maps'] = dict(expected['maps'], **{ROOM: revised})
    expected['map_blocks'] = dict(expected['map_blocks'], **{
        old['attributes']['blocks_label']: base64.b64encode(bytes(current['blocks'])).decode()})
    assert child['data'] == expected, 'unapproved global or other-map delta'
    for key in parent.keys() | child.keys():
        if key not in ['identity', 'data']:
            assert child.get(key) == parent.get(key), key
    assert parent['identity'] != child['identity']

    collision = child['data']['tilesets']['cave']['collision']
    for index, block in enumerate(old['blocks']):
        protected = block in [0x30, 0x36] or any(k in ['WATER', 'LADDER', 'HOP_DOWN']
                                               for k in collision[f'{block:02x}'])
        assert not protected or current['blocks'][index] == block
    kinds = {(x, y): collision[f'{current["blocks"][y//2*24+x//2]:02x}'][y % 2 * 2 + x % 2]
             for y in range(36) for x in range(48)}
    bodies = {(o['x'], o['y']) for o in current['objects']}
    floor = {p for p, kind in kinds.items() if kind == 'FLOOR'} - bodies
    assert len(bodies) == 5
    assert all(path(floor, (3, 12), at) is not None for at in floor)
    for at in bodies:
        assert kinds[at] == 'FLOOR'
        assert any(abs(at[0]-x)+abs(at[1]-y) == 1 for x, y in floor)
    assert {(x, y) for (x, y), kind in kinds.items() if kind == 'LADDER'} == {(3, 13)}
    assert all(kind == 'WALL' for (x, y), kind in kinds.items()
               if x in [0, 47] or y in [0, 35])
    source = json.loads((directory / 'after.json').read_text())
    assert all(tuple(v['tile']) in floor for v in source['review_contract']['viewpoints'])
    drops = []
    for x in [14, 15]:
        assert kinds[x, 16] == 'HOP_DOWN' and kinds[x, 17] == 'WALL'
        assert kinds[x-1, 17] == kinds[x+1, 17] == 'WALL'
        steps = path(floor, (x, 18), (x, 15))
        assert steps is not None
        drops.append(dict(x=x, ordinary_return_steps=steps))
    assert kinds[21, 13] == 'WALL' and kinds[35, 7] == 'WALL'
    assert kinds[36, 28] == 'FLOOR' and kinds[36, 27] == 'WATER'

    roster = json.loads((ROOT / 'planning/minnesota/pokemon/roster.json').read_text())
    curation = json.loads((ROOT / 'planning/minnesota/pokemon/roster-curation.json').read_text())
    retired = {f['id'] for f in curation['retired_families']}
    active = {m['name'].upper() for f in roster['families'] if f['id'] not in retired for m in f['members']}
    scoped = dict(child, data=dict(child['data'], maps={ROOM: current}))
    regional = roster_audit(scoped, active)
    scenes = []
    for title, pack, module in [('Before', parent, old), ('Survey spurs', child, current)]:
        art = SourceArt(pack)
        terrain = art.render('cave', 24, 18, module['blocks'], 'nite')
        scene = actors(npc_palette_pack(pack, 'nite'), terrain, module)
        save_render(scene, args.output / (title.split()[0].lower() + '-whole-original.png'))
        scenes.append((title, scene))
    save_render(room_sheet(scenes, 2), args.output / 'whole-before-after.png')
    result = dict(status='PASS', parent=str(args.parent), parent_sha256=sha(args.parent),
                  pack=str(args.pack), pack_sha256=sha(args.pack),
                  content_identity=child['identity']['content_hash'], unchanged_other_maps=len(child['data']['maps'])-1,
                  only_changed_gameplay_fields=['data.maps.MnR10Cave1F.blocks',
                                                'data.map_blocks.MnR10Cave1F_Blocks'],
                  original_programs_events_actors_rewards_and_all_other_roots_exact=True,
                  reachable_floor=len(floor), ordinary_drop_returns=drops,
                  native_water_stairs_and_mouth_blocks_unchanged=True, regional_species=regional,
                  regional_audit_scope='Touched cave wild/fishing and visible actors, all regional trainer teams and all active evolutions.',
                  all_renders_retained=True, scope=__doc__)
    (args.output / 'receipt.json').write_text(json.dumps(result, indent=2) + '\n')
    print(json.dumps(result, indent=2))


if __name__ == '__main__':
    main()
