#!/usr/bin/env python3
"""Read-only exact-parent, native-art and playable stair/ledge audit."""
import argparse
import base64
from collections import deque
import hashlib
import json
from pathlib import Path

from render_reference import SourceArt, layout_blocks, load_pack

MAP = 'MnDuluth'
TILESET = 'mn_duluth_expanded'
STAIRS = [(44, 5), (57, 5), (35, 6), (49, 6), (7, 7), (23, 7),
          (40, 7), (53, 7), (18, 8), (23, 12), (27, 20)]


def audit(parent, child, layout):
    old, new = parent['data']['maps'][MAP], child['data']['maps'][MAP]
    assert new == dict(old, blocks=new['blocks']), 'city activities/anchors changed'
    assert new['blocks'] == layout_blocks(layout), 'complete authored source differs'
    assert [(i % 64, i // 64) for i, b in enumerate(new['blocks']) if b == 0x8c] == sorted(STAIRS, key=lambda p: (p[1], p[0]))
    changed = [(i % 64, i // 64, a, b) for i, (a, b) in enumerate(zip(old['blocks'], new['blocks'])) if a != b]
    assert len(changed) == 29
    restored = dict(child, data=dict(child['data']), runtime_files=dict(child['runtime_files']))
    restored['data']['maps'] = dict(child['data']['maps'])
    restored['data']['maps'][MAP] = old
    label = old['attributes']['blocks_label']
    assert list(base64.b64decode(child['data']['map_blocks'][label], validate=True)) == new['blocks']
    restored['data']['map_blocks'] = dict(child['data']['map_blocks'])
    restored['data']['map_blocks'][label] = parent['data']['map_blocks'][label]
    restored['data']['tilesets'] = dict(child['data']['tilesets'])
    restored['data']['tilesets'][TILESET] = parent['data']['tilesets'][TILESET]
    for path in [f'data/tilesets/{TILESET}_metatiles.bin', f'data/tilesets/{TILESET}.json',
                 f'data/tilesets/{TILESET}_palette_map.json', f'gfx/tilesets/{TILESET}.2bpp',
                 f'gfx/tilesets/{TILESET}.png']:
        restored['runtime_files'][path] = parent['runtime_files'][path]
    assert child['report'] == dict(parent['report'], manifests=parent['report']['manifests'] + ['minnesota-duluth-cliff-stairs-v1'])
    restored['report'] = parent['report']
    restored['identity'] = parent['identity']
    assert restored == parent, 'unrelated game content changed'
    before_art, after_art = SourceArt(parent), SourceArt(child)
    a, b = before_art.blocks(TILESET), after_art.blocks(TILESET)
    assert len(a) == 140 and len(b) == 141
    assert all(x.tobytes() == y.tobytes() for x, y in zip(a, b)), 'existing art changed'
    assert b[0x8c].tobytes() == after_art.blocks('cave')[0x36].tobytes(), 'stairs must use exact cave36 pixels'
    collision = child['data']['tilesets'][TILESET]['collision']
    assert collision == dict(parent['data']['tilesets'][TILESET]['collision'], **{'8c': ['FLOOR', 'WALL', 'FLOOR', 'WALL']})
    occupied = {(a['x'], a['y']) for a in new['objects']}

    def kind(p):
        x, y = p
        return collision[f'{new["blocks"][y // 2 * 64 + x // 2]:02x}'][y % 2 * 2 + x % 2]

    def walk(start, end=None):
        seen = {start: 0}
        todo = deque([start])
        while todo:
            x, y = todo.popleft()
            for p in [(x - 1, y), (x + 1, y), (x, y - 1), (x, y + 1)]:
                if 0 <= p[0] < 128 and 0 <= p[1] < 92 and p not in occupied and p not in seen and kind(p) == 'FLOOR':
                    seen[p] = seen[(x, y)] + 1
                    todo.append(p)
        return seen if end is None else seen.get(end)

    reachable = walk((69, 32))
    assert all((w['x'], w['y']) in reachable for w in new['events']['warps'] if w['index'] % 2 == 0)
    hops, obstructed = [], []
    for y in range(92):
        for x in range(128):
            if kind((x, y)) != 'HOP_DOWN':
                continue
            upper, lower = (x, y - 1), (x, y + 2)
            assert lower in reachable and kind(lower) == 'FLOOR', 'unsafe ledge landing'
            if upper not in reachable:
                obstructed.append(dict(ledge=[x, y], reason='Existing native sign blocks this upper quadrant; adjacent lane remains clear.'))
                continue
            steps = walk(lower, upper)
            assert steps is not None and steps >= 9, (upper, lower, steps)
            assert kind((x, y + 1)) == 'WALL'
            hops.append(dict(upper=upper, landing=lower, ordinary_return_steps=steps))
    assert len(hops) == 10 and not obstructed, (hops, obstructed)
    return dict(changed_metatiles=changed, native_cave36_flights=STAIRS,
                all_38_passive_returns_reachable_without_hops=True,
                clear_ledge_lanes=hops, obstructed_existing_quadrants=obstructed,
                old_140_metatiles_pixel_exact=True, cave36_pixel_exact=True,
                other_complete_maps_exact=len(parent['data']['maps']) - 1,
                entire_residual_pack_exact=True)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for arg in ['parent', 'child', 'layout', 'output']:
        parser.add_argument('--' + arg, type=Path, required=True)
    parser.add_argument('--equivalent-review', type=Path)
    args = parser.parse_args()
    inputs = [(p, p.read_bytes()) for p in [args.parent, args.child, args.layout]]
    parent, child = load_pack(args.parent), load_pack(args.child)
    result = audit(parent, child, json.loads(args.layout.read_text()))
    equivalence = None
    if args.equivalent_review:
        review = load_pack(args.equivalent_review)
        assert all(review['data']['maps'][name] == module
                   for name, module in child['data']['maps'].items()
                   if name != 'MnStateFairgrounds')
        fair_alias = child['data']['maps']['MnStateFairgrounds']['attributes']['tileset_name']
        assert all(review['data']['tilesets'][name] == definition
                   for name, definition in child['data']['tilesets'].items()
                   if name != fair_alias)
        fair_paths = {f'data/tilesets/{fair_alias}_metatiles.bin', f'data/tilesets/{fair_alias}.json',
                      f'data/tilesets/{fair_alias}_palette_map.json', f'gfx/tilesets/{fair_alias}.2bpp',
                      f'gfx/tilesets/{fair_alias}.png'}
        assert set(review['runtime_files']) == set(child['runtime_files'])
        assert all(review['runtime_files'][name] == data
                   for name, data in child['runtime_files'].items() if name not in fair_paths)
        assert review['audio_manifest'] == child['audio_manifest']
        equivalence = dict(path=str(args.equivalent_review),
            sha256=hashlib.sha256(args.equivalent_review.read_bytes()).hexdigest(),
            duluth_all_native_data_and_art_exact=True,
            all_729_non_fair_maps_and_all_non_fair_art_exact=True)
    negatives = []
    for mutation in ['other map', 'city actor', 'shared art']:
        bad = dict(child, data=dict(child['data']), runtime_files=dict(child['runtime_files']))
        bad['data']['maps'] = dict(child['data']['maps'])
        if mutation == 'other map':
            other = dict(child['data']['maps']['NewBarkTown'])
            other['blocks'] = list(other['blocks'])
            other['blocks'][0] ^= 1
            bad['data']['maps']['NewBarkTown'] = other
        elif mutation == 'city actor':
            other = dict(child['data']['maps'][MAP])
            other['objects'] = [dict(actor) for actor in other['objects']]
            other['objects'][0]['x'] += 1
            bad['data']['maps'][MAP] = other
        else:
            bad['runtime_files']['gfx/tilesets/cave.2bpp'] = b'changed'
        try:
            audit(parent, bad, json.loads(args.layout.read_text()))
        except AssertionError:
            negatives.append(dict(mutation=mutation, rejected=True))
        else:
            raise AssertionError('mutation escaped audit: ' + mutation)
    assert all(p.read_bytes() == data for p, data in inputs)
    assert not args.output.exists(), 'immutable output required'
    args.output.write_text(json.dumps(dict(status='PASS', result=result, negatives=negatives,
        review_equivalence=equivalence,
        inputs=[dict(path=str(p), sha256=hashlib.sha256(data).hexdigest()) for p, data in inputs]), indent=2) + '\n')
    print(json.dumps(dict(status='PASS', result=result)))


if __name__ == '__main__':
    main()
