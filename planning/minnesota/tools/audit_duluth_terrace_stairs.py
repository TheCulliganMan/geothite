#!/usr/bin/env python3
"""Read-only native palette, preservation and corner-stair proof for Duluth."""
import argparse
import base64
from collections import deque
import hashlib
import json
from pathlib import Path

from render_reference import SourceArt, layout_blocks, load_pack

TILESET = 'mn_duluth_expanded'
SOURCES = [0x12, 0x30, 0x36]
CLIFF_EDGES = [(24, 14), (25, 14), (84, 14), (85, 14)] + [(x, 40) for x in range(40, 50)]


def audit(parent, child, layout):
    a, b = parent['data'], child['data']
    old, city = a['maps']['MnDuluth'], b['maps']['MnDuluth']
    blocks = layout_blocks(layout)
    assert city == dict(old, blocks=blocks)
    assert blocks[20 * 64 + 28] == 0x8f, 'native36 continues the right-corner wall'
    assert blocks[20 * 64 + 27] == 0x72
    assert [blocks[20*64+x] for x in range(18,22)] == [0x6c,0x72,0x72,0x72]
    assert blocks[18*64+18] == blocks[19*64+18] == 0x68
    assert not {0x80, 0x81, 0x8c}.intersection(blocks), 'no placed gray stair variants'
    assert [blocks.count(b) for b in [0x8d, 0x8e, 0x8f, 0x90]] == [3, 6, 1, 1]
    label = city['attributes']['blocks_label']
    assert list(base64.b64decode(b['map_blocks'][label], validate=True)) == blocks
    previous, current = SourceArt(parent), SourceArt(child)
    old_art, new_art = previous.blocks(TILESET), current.blocks(TILESET)
    assert len(old_art) == 141 and len(new_art) == 145
    assert all(x.tobytes() == y.tobytes() for x, y in zip(old_art, new_art))
    collision = b['tilesets'][TILESET]['collision']
    for destination, source in enumerate(SOURCES, 0x8d):
        landing = 0x69 if source == 0x36 else 0x72
        expected = current.blocks('cave')[source].copy()
        expected.paste(old_art[landing].crop((0, 0, 32, 16)), (0, 0))
        assert new_art[destination].tobytes() == expected.tobytes()
        assert collision[f'{destination:02x}'] == b['tilesets']['cave']['collision'][f'{source:02x}']
    north = current.blocks('cave')[0x12].copy()
    north.paste(current.blocks('cave')[0x12].crop((0,16,32,32)), (0,0))
    north.paste(old_art[0x70].crop((0,16,32,32)), (0,16))
    assert new_art[0x90].tobytes() == north.tobytes()
    assert collision['90'] == ['WALL','FLOOR','FLOOR','FLOOR']
    assert [blocks[13*64+x] for x in range(19,28)] == [0x70]*4+[0x90]+[0x70]*4
    assert all(blocks[y*64+28]==0x69 for y in range(14,20))
    assert blocks[64]==0x6a and all(blocks[64+x]==0x70 for x in range(1,29)) and blocks[64+29]==0x6b
    assert all(blocks[y*64]==0x68 for y in range(2,6)) and blocks[6*64]==0x6c
    assert all(blocks[y*64+29]==0x69 for y in range(2,6))
    plateau_contract = json.loads((Path(__file__).resolve().parents[1] /
        'revisions/duluth-garden-terrace-v02/rear-left-plateau.contract.json').read_text())
    assert plateau_contract['map'] == 'MnDuluth'
    assert plateau_contract['dimensions'] == [64, 46]
    plateau_cells = set()
    for segment in plateau_contract['segments']:
        x0, y0 = segment['from']
        x1, y1 = segment['to']
        assert 0 <= x0 <= x1 < 64 and 0 <= y0 <= y1 < 46
        for y in range(y0, y1 + 1):
            for x in range(x0, x1 + 1):
                assert (x, y) not in plateau_cells, 'plateau contract overlaps'
                plateau_cells.add((x, y))
                assert blocks[y * 64 + x] == int(segment['block'], 16), segment['name']
    assert set(collision) == set(a['tilesets'][TILESET]['collision']) | {'8d', '8e', '8f', '90'}
    assert all(collision[k] == v for k, v in a['tilesets'][TILESET]['collision'].items())
    raw_path = f'data/tilesets/{TILESET}_metatiles.bin'
    raw = bytes(parent['runtime_files'][raw_path])
    assert bytes(child['runtime_files'][raw_path])[:len(raw)] == raw
    assert all(a['tilesets'][TILESET]['palette_map'][g] == b['tilesets'][TILESET]['palette_map'][g] for g in set(raw))
    native_raw = bytes(child['runtime_files']['data/tilesets/cave_metatiles.bin'])
    for i, source in enumerate(SOURCES):
        imported = bytes(child['runtime_files'][raw_path])[(0x8d+i)*16:(0x8e+i)*16]
        original = native_raw[source*16+8:(source+1)*16]
        landing = 0x69 if source == 0x36 else 0x72
        assert imported[:8] == raw[landing*16:landing*16+8], 'outdoor landing must remain exact'
        imported = imported[8:]
        assert all(b['tilesets'][TILESET]['palette_map'][g] & 7 == b['tilesets']['cave']['palette_map'][h] & 7
                   for g, h in zip(imported, original))
    normalized = dict(child, data=dict(b, maps=dict(b['maps']), map_blocks=dict(b['map_blocks']), tilesets=dict(b['tilesets'])), runtime_files=dict(child['runtime_files']))
    normalized['data']['maps']['MnDuluth'] = old
    normalized['data']['map_blocks'][label] = a['map_blocks'][label]
    normalized['data']['tilesets'][TILESET] = a['tilesets'][TILESET]
    assert set(child['runtime_files']) == set(parent['runtime_files'])
    paths = [raw_path, f'data/tilesets/{TILESET}.json', f'data/tilesets/{TILESET}_palette_map.json', f'gfx/tilesets/{TILESET}.png', f'gfx/tilesets/{TILESET}.2bpp']
    for path in paths: normalized['runtime_files'][path] = parent['runtime_files'][path]
    assert child['report'] == dict(parent['report'], manifests=parent['report']['manifests'] + ['minnesota-duluth-terrace-stairs-v2', 'minnesota-duluth-north-stair-v1'])
    normalized['report'], normalized['identity'] = parent['report'], parent['identity']
    assert normalized == parent, 'unrelated content changed'
    from review_duluth_stair_heights import assess
    heights = assess(blocks, collision)
    assert heights['status'] == 'PASS', heights
    assert all(f['step_side_collision'] == (['WALL','UP_WALL'] if f.get('orientation')=='north' else ['WALL','WALL']) for f in heights['stairs'])
    return dict(rear_left_plateau_contract_cells=len(plateau_cells), stair_height_consistency=heights, changed_metatiles=sum(x != y for x,y in zip(old['blocks'],blocks)), corner_native_block='36', corner_metatile=[28,20], native_per_glyph_palettes_preserved=True, outdoor_landing_rows_exact=True, all_141_existing_metatiles_pixel_exact=True, all_729_other_maps_art_audio_story_exact=True, **assess_geometry(city, blocks, collision))


def assess_geometry(city, blocks, collision):
    """Check source or published collision without claiming production input."""
    occupied = {(o['x'], o['y']) for o in city['objects']}

    def kind(at):
        x, y = at
        return collision[f'{blocks[y // 2 * 64 + x // 2]:02x}'][y % 2 * 2 + x % 2]

    def flood(start, forbidden=frozenset()):
        found = {start: 0}
        todo = deque([start])
        while todo:
            x, y = todo.popleft()
            for p in [(x-1,y),(x+1,y),(x,y-1),(x,y+1)]:
                if 0 <= p[0] < 128 and 0 <= p[1] < 92 and p not in found and p not in occupied and p not in forbidden and kind(p) == 'FLOOR':
                    found[p] = found[(x,y)] + 1
                    todo.append(p)
        return found

    accessible = flood((55,33))
    assert all((w['x'],w['y']) in accessible for w in city['events']['warps'] if w['index'] % 2 == 0)
    assert all(kind((56,y)) == 'FLOOR' for y in range(39,43))
    assert kind((55,41)) == kind((57,41)) == 'WALL'
    assert all(kind((x,y)) == 'WALL' for x in range(58,64) for y in [36,37])
    assert all(kind((x,y)) == 'WALL' for x in range(54,64) for y in [44,45])
    sealed = {(56,y) for y in range(39,43)}
    assert (46,39) not in flood((46,42), sealed), 'corner stair must be the ordinary uphill return'
    assert all(kind((x,y)) == 'FLOOR' and kind((x,y+1)) == 'WALL' for x,y in CLIFF_EDGES), 'high cliff faces must have no hop tiles'
    assert not any(collision[f'{block:02x}'][q].startswith('HOP_') for block in blocks for q in range(4)), 'no remaining cliff hops in Duluth'
    return dict(all_38_passive_returns_reachable=True, garden_main_street_steps=accessible[(64,37)], closed_high_cliff_lanes=len(CLIFF_EDGES))


def main():
    parser=argparse.ArgumentParser(description=__doc__)
    for key in ['parent','pack','layout','output']:parser.add_argument('--'+key,type=Path,required=True)
    args=parser.parse_args();inputs=[(p,p.read_bytes()) for p in [args.parent,args.pack,args.layout]]
    result=audit(load_pack(args.parent),load_pack(args.pack),json.loads(args.layout.read_text()))
    assert all(p.read_bytes()==raw for p,raw in inputs)
    assert not args.output.exists()
    args.output.write_text(json.dumps(dict(status='PASS',result=result,inputs=[dict(path=str(p),sha256=hashlib.sha256(raw).hexdigest()) for p,raw in inputs]),indent=2)+'\n');print(json.dumps(result))


if __name__ == '__main__':main()
