#!/usr/bin/env python3
"""Independently verify R10's two exact native joins and whole-pack preservation."""
import argparse
import copy
import hashlib
import json
from pathlib import Path

from render_reference import SourceArt, load_pack


def audit(parent, child):
    tileset = 'mn_r10_rockstairs'
    original = parent['data']['tilesets'][tileset]
    collision = dict(original['collision'], **{
        '81': ['TALL_GRASS', 'WALL', 'TALL_GRASS', 'WALL'],
        '82': original['collision']['80']})
    assert child['data']['tilesets'][tileset] == dict(original, collision=collision)
    raw_path = f'data/tilesets/{tileset}_metatiles.bin'
    json_path = f'data/tilesets/{tileset}.json'
    raw = bytes(parent['runtime_files'][raw_path])
    assert len(raw) == 129 * 16
    edge = b''.join(raw[3*16+r*4:3*16+r*4+2] +
                    raw[0x69*16+r*4+2:0x69*16+r*4+4] for r in range(4))
    stair = raw[0x72*16:0x72*16+8] + raw[0x80*16+8:0x81*16]
    assert bytes(child['runtime_files'][raw_path]) == raw + edge + stair
    assert json.loads(bytes(child['runtime_files'][json_path])) == collision
    before_art, after_art = SourceArt(parent), SourceArt(child)
    for bank in ['morn', 'day', 'nite', 'dark']:
        before, after = before_art.blocks(tileset, bank), after_art.blocks(tileset, bank)
        assert len(before) == 129 and len(after) == 131
        assert all(a.tobytes() == b.tobytes() for a, b in zip(before, after))
        expected_edge = before[0x69].copy()
        expected_edge.paste(before[3].crop((0, 0, 16, 32)), (0, 0))
        expected_stair = before[0x80].copy()
        expected_stair.paste(before[0x72].crop((0, 0, 32, 16)), (0, 0))
        assert after[0x81].tobytes() == expected_edge.tobytes(), bank
        assert after[0x82].tobytes() == expected_stair.tobytes(), bank
    assert set(parent) == set(child)
    assert set(parent['runtime_files']) == set(child['runtime_files'])
    assert child['report'] == dict(parent['report'], manifests=parent['report']['manifests'] +
                                   ['minnesota-r10-cache-terrain-art-v1'])
    normalized = dict(child, data=dict(child['data'], tilesets=dict(child['data']['tilesets'])),
                      runtime_files=dict(child['runtime_files']), report=parent['report'],
                      identity=parent['identity'])
    normalized['data']['tilesets'][tileset] = original
    for path in [raw_path, json_path]:
        normalized['runtime_files'][path] = parent['runtime_files'][path]
    assert normalized == parent, 'unrelated maps/data/art/foregrounds/audio/root fields changed'
    return dict(status='PASS', original_metatiles_pixel_exact_in_all_four_banks=129,
                new_native_joins=['81', '82'], no_new_glyphs_palettes_foregrounds=True,
                all_maps_actors_events_scripts_encounters_audio_roots_exact=True)


def audit_west_terrace(parent, child):
    tileset = 'mn_r10_rockstairs'
    original = parent['data']['tilesets'][tileset]
    collision = dict(original['collision'], **{
        '83': ['WALL', 'TALL_GRASS', 'WALL', 'TALL_GRASS'],
        '84': ['TALL_GRASS', 'TALL_GRASS', 'WALL', 'WALL']})
    assert child['data']['tilesets'][tileset] == dict(original, collision=collision)
    raw_path = f'data/tilesets/{tileset}_metatiles.bin'
    json_path = f'data/tilesets/{tileset}.json'
    raw = bytes(parent['runtime_files'][raw_path])
    assert len(raw) == 131 * 16
    west = b''.join(raw[0x68*16+r*4:0x68*16+r*4+2] +
                    raw[3*16+r*4+2:3*16+r*4+4] for r in range(4))
    front = raw[3*16:3*16+8] + raw[0x72*16+8:0x73*16]
    assert bytes(child['runtime_files'][raw_path]) == raw + west + front
    assert json.loads(bytes(child['runtime_files'][json_path])) == collision
    before_art, after_art = SourceArt(parent), SourceArt(child)
    for bank in ['morn', 'day', 'nite', 'dark']:
        before, after = before_art.blocks(tileset, bank), after_art.blocks(tileset, bank)
        assert len(before) == 131 and len(after) == 133
        assert all(a.tobytes() == b.tobytes() for a, b in zip(before, after))
        expected_west = before[0x68].copy()
        expected_west.paste(before[3].crop((16, 0, 32, 32)), (16, 0))
        expected_front = before[0x72].copy()
        expected_front.paste(before[3].crop((0, 0, 32, 16)), (0, 0))
        assert after[0x83].tobytes() == expected_west.tobytes(), bank
        assert after[0x84].tobytes() == expected_front.tobytes(), bank
    assert set(parent) == set(child)
    assert set(parent['runtime_files']) == set(child['runtime_files'])
    assert child['report'] == dict(parent['report'], manifests=parent['report']['manifests'] +
                                   ['minnesota-r10-west-terrace-art-v1'])
    normalized = dict(child, data=dict(child['data'], tilesets=dict(child['data']['tilesets'])),
                      runtime_files=dict(child['runtime_files']), report=parent['report'],
                      identity=parent['identity'])
    normalized['data']['tilesets'][tileset] = original
    for path in [raw_path, json_path]:
        normalized['runtime_files'][path] = parent['runtime_files'][path]
    assert normalized == parent, 'unrelated maps/data/art/foregrounds/audio/root fields changed'
    return dict(status='PASS', original_metatiles_pixel_exact_in_all_four_banks=131,
                new_native_joins=['83', '84'], no_new_glyphs_palettes_foregrounds=True,
                all_maps_actors_events_scripts_encounters_audio_roots_exact=True)


def audit_carrier_corner(parent, child, corrected=False):
    tileset = 'mn_r10_rockstairs'
    original = parent['data']['tilesets'][tileset]
    collision = dict(original['collision'], **{
        '85': ['FLOOR', 'WALL', 'FLOOR', 'FLOOR' if corrected else 'UP_WALL']})
    assert child['data']['tilesets'][tileset] == dict(original, collision=collision)
    raw_path = f'data/tilesets/{tileset}_metatiles.bin'
    json_path = f'data/tilesets/{tileset}.json'
    raw = bytes(parent['runtime_files'][raw_path])
    assert len(raw) == 133 * 16
    expected = bytearray(16)
    quadrants = [(0x71, 0), (0x69, 1), (0x71, 2), (0x71, 3) if corrected else (0x70, 1)]
    for quadrant, (block, source) in enumerate(quadrants):
        for row in range(2):
            for column in range(2):
                destination = (quadrant//2*2+row)*4+quadrant%2*2+column
                index = block*16+(source//2*2+row)*4+source%2*2+column
                expected[destination] = raw[index]
    assert bytes(child['runtime_files'][raw_path]) == raw + bytes(expected)
    assert json.loads(bytes(child['runtime_files'][json_path])) == collision
    before_art, after_art = SourceArt(parent), SourceArt(child)
    for bank in ['morn', 'day', 'nite', 'dark']:
        before, after = before_art.blocks(tileset, bank), after_art.blocks(tileset, bank)
        assert len(before) == 133 and len(after) == 134
        assert all(a.tobytes() == b.tobytes() for a, b in zip(before, after))
        corner = before[0x71].copy()
        corner.paste(before[0x69].crop((16, 0, 32, 16)), (16, 0))
        if not corrected:
            corner.paste(before[0x70].crop((16, 0, 32, 16)), (16, 16))
        assert after[0x85].tobytes() == corner.tobytes(), bank
    assert set(parent) == set(child)
    assert set(parent['runtime_files']) == set(child['runtime_files'])
    assert child['report'] == dict(parent['report'], manifests=parent['report']['manifests'] +
                                   ['minnesota-r10-carrier-corner-art-v2' if corrected else
                                    'minnesota-r10-carrier-corner-art-v1'])
    normalized = dict(child, data=dict(child['data'], tilesets=dict(child['data']['tilesets'])),
                      runtime_files=dict(child['runtime_files']), report=parent['report'],
                      identity=parent['identity'])
    normalized['data']['tilesets'][tileset] = original
    for path in [raw_path, json_path]:
        normalized['runtime_files'][path] = parent['runtime_files'][path]
    assert normalized == parent, 'unrelated maps/data/art/foregrounds/audio/root fields changed'
    return dict(status='PASS', original_metatiles_pixel_exact_in_all_four_banks=133,
                new_native_joins=['85'], no_new_glyphs_palettes_foregrounds=True,
                plain_inside_floor_without_extra_rim=corrected,
                all_maps_actors_events_scripts_encounters_audio_roots_exact=True)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for field in ['parent', 'pack', 'output']:
        parser.add_argument('--' + field, type=Path, required=True)
    mode = parser.add_mutually_exclusive_group()
    mode.add_argument('--west-terrace', action='store_true')
    mode.add_argument('--carrier-corner', action='store_true')
    mode.add_argument('--carrier-corner-v2', action='store_true')
    args = parser.parse_args()
    inputs = {str(path): hashlib.sha256(path.read_bytes()).hexdigest()
              for path in [args.parent, args.pack]}
    parent, child = load_pack(args.parent), load_pack(args.pack)
    if args.carrier_corner_v2:
        auditor = lambda parent, child: audit_carrier_corner(parent, child, corrected=True)
    elif args.carrier_corner:
        auditor = audit_carrier_corner
    elif args.west_terrace:
        auditor = audit_west_terrace
    else:
        auditor = audit
    report = auditor(parent, child)
    # A valid audit must reject an unrelated actor or foreground alteration.
    bad = dict(child, data=dict(child['data'], maps=dict(child['data']['maps'])))
    original = child['data']['maps']['MnR10C00R00']
    changed = dict(original, objects=copy.deepcopy(original['objects']))
    changed['objects'][0]['x'] += 1
    bad['data']['maps']['MnR10C00R00'] = changed
    try:
        auditor(parent, bad)
    except AssertionError:
        report['unrelated_actor_mutation_rejected'] = True
    else:
        raise AssertionError('Accepted unrelated actor mutation')
    bad = dict(child, runtime_files=dict(child['runtime_files']))
    path = next(p for p in child['runtime_files'] if p.endswith('_foregrounds.json'))
    bad['runtime_files'][path] = bytes(child['runtime_files'][path]) + b'\n'
    try:
        auditor(parent, bad)
    except AssertionError:
        report['unrelated_foreground_mutation_rejected'] = True
    else:
        raise AssertionError('Accepted unrelated foreground mutation')
    assert all(hashlib.sha256(Path(path).read_bytes()).hexdigest() == value
               for path, value in inputs.items())
    report['inputs'] = inputs
    assert not args.output.exists()
    args.output.write_text(json.dumps(report, indent=2) + '\n')
    print(json.dumps(report), flush=True)


if __name__ == '__main__':
    main()
