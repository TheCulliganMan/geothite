#!/usr/bin/env python3
"""Exact preservation of the explicit cliff/bank and private stair-alias stages."""
import argparse
import base64
import copy
import hashlib
import json
from pathlib import Path

from render_reference import SourceArt, load_pack


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def expected_terrain(parent, plan, ramps=False):
    expected = copy.deepcopy(parent['data'])
    maps = expected['maps']
    if ramps:
        aliases = {a['source']: a for a in plan['aliases']}
        names = {p['map'] for p in plan['placements']}
        for p in plan['placements']:
            m = maps[p['map']]; x, y = p['metatile']
            at = y*m['attributes']['width']+x
            assert m['blocks'][at] == 0x74
            m['blocks'][at] = aliases[p['tileset']]['native_metatile_count']
            m['attributes']['tileset_name'] = aliases[p['tileset']]['alias']
    else:
        names = ({p['map'] for p in plan['changes']}
                 | {p['map'] for p in plan.get('boulder_moves', [])}
                 | {p['map'] for p in plan.get('corner_boulders', [])}
                 | {p['map'] for p in plan.get('trainer_turns', [])})
        for p in plan['changes']:
            m = maps[p['map']]; x, y = p['metatile']; at = y*m['attributes']['width']+x
            assert m['blocks'][at] == int(p['before'], 16)
            m['blocks'][at] = int(p['after'], 16)
        for p in plan.get('boulder_moves', []):
            m = maps[p['map']]; x, y = p['from']; w = m['attributes']['width']
            assert m['blocks'][y*w+x] == 0x0a
            assert m['blocks'][(y-1)*w+x] == int(p['destination_before'], 16)
            m['blocks'][y*w+x] = 0x72
            m['blocks'][(y-1)*w+x] = 0x0a
        for p in plan.get('corner_boulders', []):
            m = maps[p['map']]; x, y = p['metatile']; w = m['attributes']['width']
            assert m['blocks'][y*w+x] == 0x0a
            m['blocks'][y*w+x] = int(p['corner'], 16)
            if p['destination'] is not None:
                dx, dy = p['destination']
                assert m['blocks'][dy*w+dx] == int(p['destination_before'], 16)
                m['blocks'][dy*w+dx] = 0x0a
        for p in plan.get('trainer_turns', []):
            objects = maps[p['map']]['objects']
            matches = [o for o in objects if o['script'] == p['script']]
            assert len(matches) == 1
            o = matches[0]
            assert [o['x'],o['y']] == p['tile'] and o['spritemovedata'] == p['before']
            assert o['sightline_direction_override'] is None
            o['spritemovedata'] = p['after']
    for name in names:
        m = maps[name]
        expected['map_blocks'][m['attributes']['blocks_label']] = base64.b64encode(bytes(m['blocks'])).decode()
    return expected, names


def audit_cliffs(parent, child, plan):
    expected, names = expected_terrain(parent, plan)
    assert child['data'] == expected, 'cliff stage changed unrelated game data'
    assert child['runtime_files'] == parent['runtime_files'], 'cliff stage changed art/programs/audio'
    assert child['audio_manifest'] == parent['audio_manifest']
    manifest = ('minnesota-outdoor-trainer-sightlines-v1' if plan.get('trainer_turns')
                else 'minnesota-outdoor-cliff-corners-v1' if plan.get('corner_boulders')
                else 'minnesota-outdoor-boulder-setbacks-v1'
                if not plan['changes'] and plan.get('boulder_moves')
                else 'minnesota-outdoor-cliff-joins-v1')
    assert manifest not in parent['report']['manifests']
    expected_report = dict(parent['report'], manifests=parent['report']['manifests']+[manifest])
    assert child['report'] == expected_report
    corner_cells = sum(1 + (p['destination'] is not None) for p in plan.get('corner_boulders', []))
    return dict(status='PASS', maps=len(names), terrain_cells=len(plan['changes'])+2*len(plan.get('boulder_moves', []))+corner_cells, all_other_game_data_runtime_audio_exact=True)


def audit_stairs(parent, child, plan):
    expected, names = expected_terrain(parent, plan, ramps=True)
    old_art, new_art = SourceArt(parent), SourceArt(child)
    additions = set()
    manifests = []
    for a in plan['aliases']:
        source, alias, count = a['source'], a['alias'], a['native_metatile_count']
        assert alias not in parent['data']['tilesets']
        old = parent['data']['tilesets'][source]; new = child['data']['tilesets'][alias]
        expected_collision = dict(old['collision'], **{f'{count:02x}':['FLOOR','FLOOR','WALL','FLOOR']})
        assert new['collision'] == expected_collision
        assert dict(new, collision=old['collision'], palette_map=old['palette_map']) == old
        assert len(new_art.blocks(alias)) == count+1
        for i in range(count):
            assert new_art.blocks(alias)[i].tobytes() == old_art.blocks(source)[i].tobytes(), (alias, i)
        stair = old_art.blocks('cave')[0x12].copy()
        stair.paste(old_art.blocks(source)[0x72].crop((0,0,32,16)), (0,0))
        assert new_art.blocks(alias)[count].tobytes() == stair.tobytes(), alias
        for move in ['cut', 'whirlpool']:
            rules = expected['field_moves'][move]['replacements']
            if source in rules: rules[alias] = copy.deepcopy(rules[source])
        expected['tilesets'][alias] = new
        suffixes = [f'data/tilesets/{alias}_metatiles.bin', f'data/tilesets/{alias}.json', f'data/tilesets/{alias}_palette_map.json', f'gfx/tilesets/{alias}.2bpp', f'gfx/tilesets/{alias}.png']
        fg_source = f'data/tilesets/{source}_foregrounds.json'; fg_alias = f'data/tilesets/{alias}_foregrounds.json'
        if fg_alias in child['runtime_files']:
            inherited = json.loads(bytes(parent['runtime_files'].get(fg_source, b'{}')))
            actual = json.loads(bytes(child['runtime_files'][fg_alias]))
            assert all(actual[k] == v for k,v in inherited.items())
            suffixes.append(fg_alias)
        additions.update(suffixes)
        old_raw = bytes(parent['runtime_files'][f'data/tilesets/{source}_metatiles.bin'])
        raw = bytes(child['runtime_files'][f'data/tilesets/{alias}_metatiles.bin'])
        assert raw[:len(old_raw)] == old_raw and len(raw) == len(old_raw)+16
        manifests.append(alias.replace('_','-')+'-expanded-city-art')
    assert child['data'] == expected, 'stairs changed unrelated data or field-move behavior'
    assert set(child['runtime_files']) == set(parent['runtime_files']) | additions
    for path, value in parent['runtime_files'].items(): assert child['runtime_files'][path] == value, path
    assert child['audio_manifest'] == parent['audio_manifest']
    assert child['report'] == dict(parent['report'], manifests=parent['report']['manifests']+manifests+['minnesota-outdoor-ramp-stairs-v1'])
    return dict(status='PASS', maps=len(names), stairs=len(plan['placements']), aliases=len(plan['aliases']), all_original_pixels_events_gates_audio_exact=True, inherited_winter_foregrounds_preserved=True)


def audit_north(parent, child, plan):
    alias = 'mn_outdoor_headwaters_stairs'
    expected = copy.deepcopy(parent['data'])
    old = parent['data']['tilesets'][alias]['collision']
    assert '87' not in old and '88' not in old
    expected_collision = dict(old, **{'87':old['74'],'88':old['74']})
    expected['tilesets'][alias]['collision'] = expected_collision
    names = set()
    assert len(plan['placements']) == 4
    for p in plan['placements']:
        m = expected['maps'][p['map']];x,y = p['metatile'];w = m['attributes']['width']
        assert m['attributes']['tileset_name'] == p['tileset'] and m['blocks'][y*w+x] == 0x74
        block,inside = (0x87,0x01) if p['inside_landing']=='paved' else (0x88,0x71)
        assert p['inside_landing'] in ['paved','rock'] and m['blocks'][(y+1)*w+x] == inside
        m['blocks'][y*w+x] = block;m['attributes']['tileset_name'] = alias;names.add(p['map'])
    for name in names:
        m = expected['maps'][name]
        expected['map_blocks'][m['attributes']['blocks_label']] = base64.b64encode(bytes(m['blocks'])).decode()
    assert child['data'] == expected, 'north stairs changed unrelated data or field rules'
    raw_path = f'data/tilesets/{alias}_metatiles.bin';collision_path = f'data/tilesets/{alias}.json'
    raw = bytes(parent['runtime_files'][raw_path]);assert len(raw) == 0x87*16
    joined = b''.join(raw[0x86*16+8:0x87*16]+raw[landing*16+8:(landing+1)*16] for landing in [0x74,0x70])
    assert bytes(child['runtime_files'][raw_path]) == raw+joined
    assert json.loads(bytes(child['runtime_files'][collision_path])) == expected_collision
    assert set(parent['runtime_files']) == set(child['runtime_files'])
    for path,value in parent['runtime_files'].items():
        if path not in [raw_path,collision_path]:assert child['runtime_files'][path] == value,path
    for movement in ['cut','whirlpool']:
        rules = child['data']['field_moves'][movement]['replacements']
        if 'mn_headwaters' in rules:assert rules[alias] == rules['mn_headwaters']
    before, after = SourceArt(parent).blocks(alias), SourceArt(child).blocks(alias)
    assert len(before) == 0x87 and len(after) == 0x89
    assert all(a.tobytes() == b.tobytes() for a,b in zip(before,after))
    for destination,landing in [(0x87,0x74),(0x88,0x70)]:
        image = before[landing].copy();image.paste(before[0x86].crop((0,16,32,32)),(0,0))
        assert image.tobytes() == after[destination].tobytes()
    assert child['audio_manifest'] == parent['audio_manifest']
    assert child['report'] == dict(parent['report'],manifests=parent['report']['manifests']+['minnesota-outdoor-north-stairs-v1'])
    return dict(status='PASS',maps=len(names),north_stairs=4,native74_collision_exact=True,
                gray_treads_and_context_matched_inside_landings_exact=True,all_original_pixels_glyphs_palettes_foregrounds_audio_exact=True,
                r07_cut_whirlpool_rules_preserved=True)


def main():
    cli = argparse.ArgumentParser(description=__doc__)
    cli.add_argument('--parent', type=Path, required=True)
    cli.add_argument('--pack', type=Path, required=True)
    cli.add_argument('--plan', type=Path, required=True)
    cli.add_argument('--stage', choices=['cliffs', 'stairs', 'north'], required=True)
    cli.add_argument('--output', type=Path, required=True)
    args = cli.parse_args(); args.output.mkdir(exist_ok=False, parents=True)
    inputs = {str(p): sha(p) for p in [args.parent, args.pack, args.plan]}
    parent, child = load_pack(args.parent), load_pack(args.pack)
    plan = json.loads(args.plan.read_text())
    result = {'cliffs':audit_cliffs,'stairs':audit_stairs,'north':audit_north}[args.stage](parent, child, plan)
    assert set(parent) == set(child), 'unrelated pack root fields changed'
    for key,value in parent.items():
        if key not in ['data','runtime_files','report','identity']:
            assert child[key] == value, f'unrelated pack root field changed: {key}'
    result['all_other_pack_roots_including_compiled_audio_exact'] = True
    assert all(sha(Path(p)) == h for p,h in inputs.items())
    result['inputs'] = inputs
    (args.output/'receipt.json').write_text(json.dumps(result, indent=2)+'\n')
    print(json.dumps(result), flush=True)


if __name__ == '__main__': main()
