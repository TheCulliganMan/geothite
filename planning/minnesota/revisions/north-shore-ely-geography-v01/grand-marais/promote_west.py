#!/usr/bin/env python3
"""Promote matching town/source connections with exact canonical preimages.

No pack writing. Run without --apply for a read-only final-pack comparison.
Frozen proposal epochs remain unchanged. Requires the composed 694-map pack.
"""
import argparse
import copy
import hashlib
import json
from pathlib import Path
import sys

ROOT = Path(__file__).resolve().parents[5]
sys.path.insert(0, str(ROOT / 'planning/minnesota/tools'))
from render_reference import layout_blocks, load_pack

SOURCE = Path(__file__).resolve().parent / 'source-v02'


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def read(path):
    return json.loads(path.read_bytes())


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--pack', type=Path, required=True)
    parser.add_argument('--expected-sha256', required=True)
    parser.add_argument('--apply', action='store_true')
    args = parser.parse_args()
    assert digest(args.pack) == args.expected_sha256
    guards = read(SOURCE / 'source-preimages.json')['canonical_before_sha256']
    promotion = read(SOURCE.parent / 'promotion-guards-v01.json')
    guards['planning/minnesota/world.json'] = promotion['canonical_world_sha256']
    guards['planning/minnesota/interiors.json'] = '2ce5e685a29434c8ae0c70f525daead37b0997c3ca41c0ecf8bb330be2e26cc5'
    for name, expected in guards.items():
        assert digest(ROOT / name) == expected, ('stale canonical source', name)
    for name, expected in promotion['candidate_sha256'].items():
        assert digest(SOURCE / name) == expected, ('stale candidate', name)
    assert digest(SOURCE / 'native-edit.contract.json') == 'c47d93919126bc32f3aa8d6b6f9034aff5427ceacea866ba93044a9576852278'
    pack = load_pack(args.pack)
    maps = pack['data']['maps']
    assert len(maps) == 694
    town = maps['MnGrandMarais']
    layout = read(SOURCE / 'MnGrandMarais.layout.candidate.json')
    assert layout_blocks(layout) == town['blocks']
    for index, layer in enumerate(layout['layers']):
        layer.setdefault('name', f'Accepted Superior town source layer {index}')
    connections = read(SOURCE / 'MnGrandMarais.connections.candidate.json')
    assert connections['attributes_after'] == town['attributes']
    for name in connections['preserved_room_ids']:
        raw = json.dumps(maps[name], sort_keys=True, separators=(',', ':'), ensure_ascii=False).encode()
        assert hashlib.sha256(raw).hexdigest() == read(SOURCE / 'source-preimages.json')['native_rooms_sha256'][name]
    dossier = read(SOURCE / 'MnGrandMarais.dossier.candidate.json')
    dossier['implementation'].update(interiors=6, scope='Warm west-entry Superior town; north inland branch.',
                                     review='../revisions/north-shore-ely-geography-v01/VERIFICATION.md')
    dossier['native_connections_source'] = 'MnGrandMarais.connections.json'
    dossier['layout'] = 'MnGrandMarais.layout.json'
    connections.update(status='MATCHES_COMPOSED_NATIVE_PACK', canonical_world_edited=True,
                       pack_sha256=args.expected_sha256, content_identity=pack['identity']['content_hash'])
    design = read(ROOT / 'planning/minnesota/playable/grand-marais.json')
    design['rows'] = [' '.join(f'{v:02x}' for v in town['blocks'][y * 20:(y + 1) * 20]) for y in range(19)]
    design['signs'][1]['lines'] = ['SUPERIOR HARBOR', 'NORTH SHORE: WEST', 'GUNFLINT: NORTH', 'ELY: INLAND SNOW']
    world = read(ROOT / 'planning/minnesota/world.json')
    town_index = next(i for i, t in enumerate(world['towns']) if t['id'] == 'MnGrandMarais')
    world['towns'][town_index] = copy.deepcopy(dossier)
    # Replace only superseded coastal transport descriptions. Existing internal
    # R11/R12 chunk links and unrelated town/route data retain their values.
    old_edges = [e for e in world['connections'] if e.get('b') == 'MnGrandMaraisShoreGate' or
                 (e.get('a') == 'MnR12C01R01' and e.get('b') == 'MnGrandMarais')]
    assert len(old_edges) == 3, 'exact obsolete coast/gatehouse descriptors'
    world['connections'] = [e for e in world['connections'] if e not in old_edges]
    pairs = [('MnR11C01R00', 'east', 'MnGrandMarais'),
             ('MnGrandMarais', 'north', 'MnMaraisCedarApproach'),
             ('MnMaraisCedarApproach', 'west', 'MnMaraisInlandNeck'),
             ('MnMaraisInlandNeck', 'west', 'MnR12C01R01')]
    opposite = dict(north='south', south='north', east='west', west='east')
    for a, direction, b in pairs:
        edge = next(e for e in maps[a]['attributes']['connections'] if e['direction'] == direction)
        assert edge['target_map'] == b
        inverse = dict(direction=opposite[direction], target_map=a, offset=-edge['offset'])
        assert inverse in maps[b]['attributes']['connections']
        world['connections'].append(dict(kind='connection', a=a, b=b, direction_a=direction,
            direction_b=opposite[direction], offset_a=edge['offset'], offset_b=-edge['offset'],
            native_source='revisions/north-shore-ely-geography-v01/geography/native-west-v02/contract.json'))
    # The transport room was never part of this accepted native lineage.
    # Preserve historical files; canonical descriptors now point at real seams.
    world['superior_inland_source'] = dict(
        replay_order=['build_grand_marais_superior', 'build_superior_inland_geography', 'build_inland_winter'],
        parent='content-packs/minnesota-superior-inland-art-v01.crystalpack',
        final_pack_sha256=args.expected_sha256, content_identity=pack['identity']['content_hash'],
        extra_layouts=['towns/MnMaraisCedarApproach.layout.json', 'towns/MnMaraisInlandNeck.layout.json'])
    registration = pack['data']['runtime_map_metadata']['MN_MARAIS_INLAND_NECK']
    assert (registration['groupId'], registration['mapId']) == (40, 8)
    assert pack['data']['story_event_script_constants']['global']['EVENT_MN_GUNFLINT_CAMP_SUPPLY'] == 2387
    assert '280' not in pack['data']['runtime_spawn_points']
    world['superior_inland_source']['new_map_registration'] = registration
    world['superior_inland_source']['camp_event'] = dict(name='EVENT_MN_GUNFLINT_CAMP_SUPPLY', value=2387)
    if 'EVENT_MN_GUNFLINT_CAMP_SUPPLY' not in world['planned_flags']:
        world['planned_flags'].append('EVENT_MN_GUNFLINT_CAMP_SUPPLY')
    interiors = read(ROOT / 'planning/minnesota/interiors.json')
    obsolete = [c for c in interiors['contracts'] if c['interior'] == 'MnGrandMaraisShoreGate']
    assert len(obsolete) == 1 and 'MnGrandMaraisShoreGate' not in maps
    interiors['contracts'] = [c for c in interiors['contracts'] if c not in obsolete]
    outputs = {
        'planning/minnesota/towns/MnGrandMarais.layout.json': layout,
        'planning/minnesota/towns/MnGrandMarais.json': dossier,
        'planning/minnesota/towns/MnGrandMarais.connections.json': connections,
        'planning/minnesota/playable/grand-marais.json': design,
        'planning/minnesota/world.json': world,
        'planning/minnesota/interiors.json': interiors,
    }
    new_layout = ROOT / 'planning/minnesota/towns/MnGrandMarais.layout.json'
    assert not new_layout.exists(), 'new canonical town layout target already exists'
    if args.apply:
        # All file, pack, raster, room and reciprocal guards precede any writes.
        for name, value in outputs.items():
            target = ROOT / name
            old_text = target.read_text() if target.exists() else ''
            ascii_style = not any(ord(character) > 127 for character in old_text)
            target.write_text(json.dumps(value, indent=2, ensure_ascii=ascii_style) + '\n')
    print(json.dumps(dict(applied=args.apply, outputs=list(outputs), pack_sha256=args.expected_sha256,
                          town_cells=len(town['blocks']), preserved_rooms=6), indent=2))


if __name__ == '__main__':
    main()
