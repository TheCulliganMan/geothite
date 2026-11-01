#!/usr/bin/env python3
"""Read-only preservation and native stair-pixel proof for the R09 cliff repair."""
import argparse
import copy
import hashlib
import json
import re
from pathlib import Path

from render_reference import SourceArt, layout_blocks, load_pack
from review_directional_grass import assess


def audit(parent, child, layout, dossier):
    restored = copy.deepcopy(child)
    blocks = layout_blocks(layout)
    changed = []
    for name, module in child['data']['maps'].items():
        if not re.fullmatch(r'MnR09C\d{2}R\d{2}', name):
            assert module == parent['data']['maps'][name], name
            continue
        col, row = int(name[-5:-3]), int(name[-2:])
        a = module['attributes']
        expected = [blocks[(row*24+y)*64+col*32+x]
                    for y in range(a['height']) for x in range(a['width'])]
        assert module['blocks'] == expected, name
        old = parent['data']['maps'][name]
        changed.extend(dict(map=name, local_meta=[i % a['width'], i // a['width']],
                            before=x, after=y)
                       for i, (x, y) in enumerate(zip(old['blocks'], expected)) if x != y)
        restored['data']['maps'][name]['blocks'] = old['blocks']
        label = a['blocks_label']
        restored['data']['map_blocks'][label] = parent['data']['map_blocks'][label]
    tileset = 'mn_iron_range'
    art, before = SourceArt(child), SourceArt(parent)
    native, original, cave = art.blocks(tileset), before.blocks(tileset), art.blocks('cave')
    assert len(native) == len(original) + 3 == 136
    old_definition = parent['data']['tilesets'][tileset]
    new_definition = child['data']['tilesets'][tileset]
    assert set(new_definition['collision']) == set(old_definition['collision']) | {'85', '86', '87'}
    assert all(new_definition['collision'][k] == v for k, v in old_definition['collision'].items())
    raw_path = f'data/tilesets/{tileset}_metatiles.bin'
    old_raw, new_raw = bytes(parent['runtime_files'][raw_path]), bytes(child['runtime_files'][raw_path])
    assert new_raw[:len(old_raw)] == old_raw
    assert all(old_definition['palette_map'][glyph] == new_definition['palette_map'][glyph]
               for glyph in set(old_raw))
    assert all(x.tobytes() == y.tobytes() for x, y in zip(original, native))
    for destination, source in [(0x85, 0x12), (0x86, 0x30), (0x87, 0x36)]:
        assert native[destination].tobytes() == cave[source].tobytes()
        assert child['data']['tilesets'][tileset]['collision'][f'{destination:02x}'] == child['data']['tilesets']['cave']['collision'][f'{source:02x}']
    restored['data']['tilesets'][tileset] = parent['data']['tilesets'][tileset]
    paths = [f'data/tilesets/{tileset}_metatiles.bin',
             f'data/tilesets/{tileset}.json', f'data/tilesets/{tileset}_palette_map.json',
             f'gfx/tilesets/{tileset}.2bpp', f'gfx/tilesets/{tileset}.png']
    assert set(parent['runtime_files']) == set(child['runtime_files'])
    for path in paths:
        restored['runtime_files'][path] = parent['runtime_files'][path]
    assert child['report']['manifests'] == parent['report']['manifests'] + ['minnesota-iron-cliff-native-stairs-v1']
    restored['report'] = parent['report']
    restored['identity'] = parent['identity']
    assert restored == parent, 'unrelated game content changed'
    collision = child['data']['tilesets'][tileset]['collision']
    states = assess(dossier, layout, blocks, collision)
    lanes = []
    for x in [38, 39]:
        d = copy.deepcopy(dossier)
        loop = copy.deepcopy(d['directional_grass_loops'][-1])
        loop.update(outbound_start_core=[x, 36], outbound_end_core=[x, 33], northbound_minimum_grass_steps=2)
        d['directional_grass_loops'] = [loop]
        lanes.extend(assess(d, layout, blocks, collision))
    coast = []
    for name, m in child['data']['maps'].items():
        if name.startswith('MnR11C') or name in ['MnDuluth', 'MnGrandMarais', 'MnMaraisCedarApproach']:
            c = child['data']['tilesets'][m['attributes']['tileset_name']]['collision']
            assert all('ICE' not in c[f'{v:02x}'] for v in m['blocks'] + [v['block_id'] for v in m['script_block_changes']]), name
            coast.append(name)
    return dict(changed_cells=changed, other_maps_exact=len(child['data']['maps'])-4,
                original_133_blocks_pixel_exact=True, exact_cave_stairs=['12', '30', '36'],
                all_other_content_exact=True, directional_loops=states,
                new_ledge_lanes=lanes, ice_free_coast=coast)


def main():
    p = argparse.ArgumentParser(description=__doc__)
    for flag in ['parent', 'child', 'layout', 'dossier', 'output']:
        p.add_argument('--'+flag, type=Path, required=True)
    args = p.parse_args()
    parent, child = load_pack(args.parent), load_pack(args.child)
    layout, dossier = json.loads(args.layout.read_text()), json.loads(args.dossier.read_text())
    report = audit(parent, child, layout, dossier)
    negatives = []
    for name in ['foreign-map', 'story-script', 'original-art', 'stair-collision', 'audio']:
        bad = copy.deepcopy(child)
        if name == 'foreign-map': bad['data']['maps']['MnGrandMarais']['blocks'][0] ^= 1
        if name == 'story-script': bad['data']['maps']['MnR09C00R00']['objects'][0]['script'] += 'BAD'
        if name == 'original-art': bad['runtime_files']['gfx/tilesets/johto.2bpp'] = b'bad'
        if name == 'stair-collision': bad['data']['tilesets']['mn_iron_range']['collision']['85'][3] = 'LADDER'
        if name == 'audio': bad['audio_manifest'] = {}
        try: audit(parent, bad, layout, dossier)
        except (AssertionError, KeyError, ValueError): negatives.append(name)
        else: raise AssertionError('accepted mutation '+name)
    report.update(negative_mutations_rejected=negatives,
                  parent_sha256=hashlib.sha256(args.parent.read_bytes()).hexdigest(),
                  child_sha256=hashlib.sha256(args.child.read_bytes()).hexdigest(),
                  identity=child['identity']['content_hash'])
    assert not args.output.exists()
    args.output.write_text(json.dumps(report, indent=2)+'\n')
    print(json.dumps({k:v for k,v in report.items() if k != 'changed_cells'}))


if __name__ == '__main__': main()
