#!/usr/bin/env python3
"""Check only the three free-basecamp guarded walking-link source contracts.

This is read-only metadata validation, not collision, script execution, native
pack verification or an earned-progression proof. No movement graph is built.
"""

import argparse
from copy import deepcopy
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
CAMP = 'MnBoundaryWaters'
BADGES_AND_SIGNAL = [f'BADGE_{i}' for i in range(1, 9)] + [
    'EVENT_MN_LIGHTHOUSE_SIGNAL'
]
# Frozen native resolver contracts; offsets count 32px metatiles, coordinates
# count 16px walking tiles. Existing internal zero-offset links are out of scope.
LINKS = (
    dict(route='MN_R12', a='MnR12C00R01', checkpoint='gunflint',
         direction_a='west', direction_b='east', offset_a=-8, offset_b=8,
         a_edge_core=[[0, 8], [0, 9]], b_edge_core=[[63, 24], [63, 25]],
         script='MnBoundaryGunflintCheck', chunk=[0, 1], side='W',
         inner=[[60, 24], [60, 25]], inward='LEFT', outward='RIGHT',
         guard_ids=['MN_BOUNDARY_GUNFLINT_GUARD_NORTH',
                    'MN_BOUNDARY_GUNFLINT_GUARD_SOUTH']),
    dict(route='MN_R13', a='MnR13C00R00', checkpoint='league',
         direction_a='north', direction_b='south', offset_a=-1, offset_b=1,
         a_edge_core=[[30, 0], [31, 0]], b_edge_core=[[32, 47], [33, 47]],
         script='MnBoundaryLeagueCheck', chunk=[0, 0], side='N',
         inner=[[32, 42], [33, 42]], inward='UP', outward='DOWN',
         guard_ids=['MN_BOUNDARY_LEAGUE_GUARD_WEST',
                    'MN_BOUNDARY_LEAGUE_GUARD_EAST']),
    dict(route='MN_R20', a='MnR20C01R00', checkpoint='voyageurs',
         direction_a='east', direction_b='west', offset_a=6, offset_b=-6,
         a_edge_core=[[63, 36], [63, 37]], b_edge_core=[[0, 24], [0, 25]],
         script='MnBoundaryVoyageursCheck', chunk=[1, 0], side='E',
         inner=[[3, 24], [3, 25]], inward='RIGHT', outward='LEFT',
         guard_ids=['MN_BOUNDARY_VOYAGEURS_GUARD_NORTH',
                    'MN_BOUNDARY_VOYAGEURS_GUARD_SOUTH']),
)
WARP_ONLY_FIELDS = (
    'warp_index_status', 'a_gate_local_metatile', 'a_arrival_local_metatile',
    'b_gate_metatile', 'b_arrival_metatile',
)


def _read_sources(root):
    town = json.loads((root / 'towns' / f'{CAMP}.json').read_text())
    routes = {s['route']: json.loads(
        (root / 'routes' / f"{s['route']}.json").read_text()) for s in LINKS}
    return town, routes


def validate(world, root=ROOT):
    """Return source-contract errors; ``root`` is the Minnesota planning folder.

    Unrelated connections are left to check_plan. Missing/malformed source JSON
    is returned as an error. This function never writes files or loads a pack.
    """
    try:
        town, routes = _read_sources(Path(root))
    except (OSError, ValueError, TypeError) as exc:
        return [f'boundary walking links: cannot read source contracts: {exc}']
    return _validate(world, town, routes)


def _validate(world, town, routes):
    errors = []

    def same_value(actual, expected):
        # Enforce JSON number/boolean types even inside coordinate arrays.
        if type(actual) is not type(expected):
            return False
        if isinstance(expected, dict):
            return actual.keys() == expected.keys() and all(
                same_value(actual[key], value) for key, value in expected.items())
        if isinstance(expected, list):
            return len(actual) == len(expected) and all(
                same_value(a, b) for a, b in zip(actual, expected))
        return actual == expected

    def expect(actual, expected, where):
        if not same_value(actual, expected):
            errors.append(f'{where}: expected {expected!r}, found {actual!r}')

    def records(value, where):
        if not isinstance(value, list) or any(not isinstance(v, dict) for v in value):
            errors.append(f'{where}: expected an array of records')
            return []
        return value

    def one(values, field, identifier, where):
        found = [v for v in values if v.get(field) == identifier]
        if len(found) != 1:
            errors.append(f'{where}: expected one {field}={identifier!r}, found {len(found)}')
            return {}
        return found[0]

    def flags(actual, expected, where):
        if (not isinstance(actual, list) or
                any(not isinstance(v, str) for v in actual) or
                len(actual) != len(set(actual)) or set(actual) != set(expected)):
            errors.append(f'{where}: expected unique flags {expected!r}, found {actual!r}')

    if not isinstance(world, dict) or not isinstance(town, dict):
        return ['boundary walking links: world and source town must be records']
    connections = records(world.get('connections'), 'world.connections')
    chunks = records(world.get('chunks'), 'world.chunks')
    towns = records(world.get('towns'), 'world.towns')
    world_town = one(towns, 'id', CAMP, 'world.towns')
    expect(town.get('id'), CAMP, 'source town.id')
    for label, value in (('source town', town), ('world town', world_town)):
        expect(value.get('dimensions_metatiles'), [32, 24], f'{label}.dimensions_metatiles')
    source_ports = records(town.get('ports'), 'source town.ports')
    world_ports = records(world_town.get('ports'), 'world town.ports')
    regional = town.get('regional_checkpoints', {})
    if not isinstance(regional, dict):
        errors.append('source town.regional_checkpoints: expected a record')
        regional = {}
    checkpoints = records(regional.get('checkpoints'), 'source checkpoints')
    expect(regional.get('free_inward_care'), True, 'source checkpoints.free_inward_care')
    expect(regional.get('capture_expeditions'), 'CLOSED_NO_FEE', 'source checkpoints.capture_expeditions')
    expected_identities = tuple((s['route'], s['a'], CAMP) for s in LINKS)
    for c in connections:
        identity = (c.get('route'), c.get('a'), c.get('b'))
        if (CAMP in (c.get('a'), c.get('b')) or
                c.get('gate_map') == CAMP) and identity not in expected_identities:
            errors.append(f'world.connections: unexpected guarded/basecamp identity {identity!r}')
    expect(len(checkpoints), 3, 'source checkpoints count')

    for spec in LINKS:
        label = f"boundary link {spec['route']}"
        matches = [c for c in connections if
                   (c.get('route'), c.get('a'), c.get('b')) ==
                   (spec['route'], spec['a'], CAMP)]
        if len(matches) != 1:
            errors.append(f'{label}: expected exactly one route/chunk/camp record, found {len(matches)}')
            continue
        link = matches[0]
        required = BADGES_AND_SIGNAL + (
            ['EVENT_MN_SURVEY_RESTORED'] if spec['checkpoint'] == 'league' else [])
        for field, expected in dict(
                kind='guarded_connection', gate_map=CAMP,
                gate_script=spec['script'], checkpoint=spec['checkpoint'],
                direction_a=spec['direction_a'], direction_b=spec['direction_b'],
                offset_a=spec['offset_a'], offset_b=spec['offset_b'],
                offset_units='32px_metatiles', a_edge_core=spec['a_edge_core'],
                b_edge_core=spec['b_edge_core'], requires_apply_to='camp_departure',
                fee=0, inbound_requires_flags=[], inbound_access='free_care',
                inbound_facing_policy='escort_unless_outward').items():
            expect(link.get(field), expected, f'{label}.{field}')
        flags(link.get('requires_flags'), required, f'{label}.requires_flags')
        for field in WARP_ONLY_FIELDS:
            if field in link:
                errors.append(f'{label}.{field}: obsolete warp-only field on a walking link')

        # The exact lane pairs above also obey the core resolver's transverse
        # translation: target = source - 2 * source_metatile_offset. This is
        # arithmetic alignment, not a replacement for the native resolver test.
        axis = 1 if spec['direction_a'] in ('east', 'west') else 0
        a_lanes, b_lanes = link.get('a_edge_core'), link.get('b_edge_core')
        if (a_lanes == spec['a_edge_core'] and b_lanes == spec['b_edge_core'] and
                type(link.get('offset_a')) is int and type(link.get('offset_b')) is int):
            for lane, (a, b) in enumerate(zip(a_lanes, b_lanes)):
                expect(a[axis] - 2 * link['offset_a'], b[axis], f'{label}.lane{lane} alignment')
                expect(b[axis] - 2 * link['offset_b'], a[axis], f'{label}.lane{lane} reverse alignment')
        chunk = one(chunks, 'id', spec['a'], f'{label} chunk')
        for field, expected in dict(route=spec['route'], column=spec['chunk'][0],
                                    row=spec['chunk'][1], dimensions_metatiles=[32, 24]).items():
            expect(chunk.get(field), expected, f'{label} chunk.{field}')
        route = routes.get(spec['route'], {})
        if not isinstance(route, dict):
            errors.append(f'{label}: malformed source route')
            route = {}
        expect(route.get('id'), spec['route'], f'{label} source route.id')
        endpoint = one(records(route.get('endpoints'), f'{label} source endpoints'),
                       'place', 'BOUNDARY WATERS', f'{label} source endpoints')
        expect(endpoint.get('side'), spec['side'], f'{label} source endpoint.side')
        # Existing endpoint gate/landing markers remain wayfinding points, not
        # native seam triggers; both must lie in the selected clipped chunk.
        for field in ('gate_metatile', 'landing_metatile'):
            p = endpoint.get(field)
            if not (isinstance(p, list) and len(p) == 2 and
                    all(type(v) is int for v in p) and
                    32 * spec['chunk'][0] <= p[0] < 32 * (spec['chunk'][0] + 1) and
                    24 * spec['chunk'][1] <= p[1] < 24 * (spec['chunk'][1] + 1)):
                errors.append(f'{label} source endpoint.{field}: not in connected chunk')

        native_port = dict(target_map=spec['a'], direction=spec['direction_b'],
                           offset_metatiles=spec['offset_b'],
                           reciprocal_offset_metatiles=spec['offset_a'],
                           checkpoint=spec['checkpoint'])
        source_port = one(source_ports, 'route', spec['route'], 'source town.ports')
        world_port = one(world_ports, 'route', spec['route'], 'world town.ports')
        expect(source_port.get('native_connection'), native_port, f'{label} source port.native_connection')
        expect(world_port.get('native_connection'), native_port, f'{label} world port.native_connection')
        for field in ('side', 'gate_metatile', 'landing_metatile'):
            expect(world_port.get(field), source_port.get(field), f'{label} world port.{field}')
        expect(source_port.get('side'), spec['direction_b'][0].upper(), f'{label} source port.side')

        cp = one(checkpoints, 'id', spec['checkpoint'], 'source checkpoints')
        outward_delta = {'RIGHT': [1, 0], 'DOWN': [0, 1], 'LEFT': [-1, 0]}[spec['outward']]
        outer = [[x + 2 * outward_delta[0], y + 2 * outward_delta[1]]
                 for x, y in spec['inner']]
        for field, expected in dict(
                route=spec['route'], native_target_map=spec['a'],
                direction=spec['direction_b'], offset_metatiles=spec['offset_b'],
                reciprocal_offset_metatiles=spec['offset_a'], script_root=spec['script'],
                fee=0, inner_coord_tiles=spec['inner'], outer_inward_coord_tiles=outer,
                native_inward_movement=dict(command='step', args=[spec['inward']]),
                outward_refusal_steps=1, inbound_free_care_steps=2,
                scene='always_active', native_outward_facing=spec['outward'],
                outer_passage_policy='Native ifequal OUTWARD skips the escort; inward and transverse arrivals receive two inward steps to free care').items():
            expect(cp.get(field), expected, f'{label} source checkpoint.{field}')
        flags(cp.get('requires'), required, f'{label} source checkpoint.requires')
        guards = records(cp.get('guards'), f'{label} source checkpoint.guards')
        expect([g.get('id') for g in guards], spec['guard_ids'], f'{label} source guard IDs')
        for lane, guard in enumerate(guards[:2]):
            inner = spec['inner'][lane]
            body = [inner[i] + outward_delta[i] for i in (0, 1)]
            for field, expected in dict(id=spec['guard_ids'][lane],
                    object_identifier=spec['guard_ids'][lane], sprite='SPRITE_OFFICER',
                    tile=body, facing={'LEFT': 'west', 'UP': 'north', 'RIGHT': 'east'}[spec['inward']],
                    event_flag=-1).items():
                expect(guard.get(field), expected, f'{label} source guard{lane}.{field}')
    return errors


def self_test(world, root=ROOT):
    """Reject adversarial in-memory copies without changing source metadata."""
    town, routes = _read_sources(Path(root))
    baseline = _validate(world, town, routes)
    if baseline:
        raise AssertionError('self-test requires valid current metadata: ' + '; '.join(baseline))
    cases = [
        ('bad kind', 'kind', 'connection'),
        ('huge offset', 'offset_a', 999999),
        ('nonreciprocal offset', 'offset_b', 5),
        ('wrong lane coordinate', 'a_edge_core', [[0, 10], [0, 11]]),
        ('noninteger lane coordinate', 'a_edge_core', [[0.0, 8], [0, 9]]),
        ('wrong script', 'gate_script', 'NOT_A_NATIVE_SCRIPT'),
        ('missing badge', 'requires_flags', BADGES_AND_SIGNAL[1:]),
        ('duplicate requirement', 'requires_flags', BADGES_AND_SIGNAL + ['BADGE_1']),
        ('paid care', 'fee', 500),
        ('boolean fee', 'fee', False),
        ('inbound prerequisite', 'inbound_requires_flags', ['BADGE_1']),
        ('transverse stranding policy', 'inbound_facing_policy', 'escort_only_inward'),
    ]
    for label, field, bad in cases:
        candidate = deepcopy(world)
        link = next(c for c in candidate['connections'] if
                    c.get('a') == LINKS[0]['a'] and c.get('b') == CAMP)
        link[field] = bad
        found = _validate(candidate, town, routes)
        if not any(f'boundary link MN_R12.{field}:' in e for e in found):
            raise AssertionError(f'{label}: failed to reject the mutated field: {found!r}')
    candidate = deepcopy(world)
    candidate['connections'].append(deepcopy(next(c for c in candidate['connections']
                                                 if c.get('a') == LINKS[0]['a'] and c.get('b') == CAMP)))
    if not any('exactly one route/chunk/camp record, found 2' in e for e in _validate(candidate, town, routes)):
        raise AssertionError('duplicate identity was not rejected')
    bad_town = deepcopy(town)
    bad_town['regional_checkpoints']['checkpoints'][0]['inner_coord_tiles'][0][0] -= 1
    if not any('source checkpoint.inner_coord_tiles:' in e for e in _validate(world, bad_town, routes)):
        raise AssertionError('source checkpoint plane drift was not rejected')
    bad_town = deepcopy(town)
    bad_town['regional_checkpoints']['checkpoints'][1]['requires'].remove('EVENT_MN_LIGHTHOUSE_SIGNAL')
    if not any('MN_R13 source checkpoint.requires:' in e for e in _validate(world, bad_town, routes)):
        raise AssertionError('source League signal omission was not rejected')
    candidate = deepcopy(world)
    next(t for t in candidate['towns'] if t['id'] == CAMP)['ports'][0]['native_connection']['offset_metatiles'] = 999999
    if not any('world port.native_connection:' in e for e in _validate(candidate, town, routes)):
        raise AssertionError('world/source port mismatch was not rejected')
    # Ordinary internal links are deliberately not interpreted by this leaf.
    candidate = deepcopy(world)
    candidate['connections'].append(dict(kind='connection', a='unrelated_a', b='unrelated_b',
                                         offset_a=0, offset_b=0))
    if _validate(candidate, town, routes):
        raise AssertionError('unrelated zero-offset connection was changed by this leaf')
    candidate['connections'].append(dict(kind='guarded_connection',
                                         a='another_route', b='another_town',
                                         gate_map='another_town'))
    if _validate(candidate, town, routes):
        raise AssertionError('unrelated guarded connection was changed by this leaf')
    return len(cases) + 5


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--root', type=Path, default=ROOT)
    parser.add_argument('--self-test', action='store_true')
    args = parser.parse_args()
    world = json.loads((args.root / 'world.json').read_text())
    errors = validate(world, args.root)
    if errors:
        parser.exit(1, '\n'.join(errors) + '\n')
    count = self_test(world, args.root) if args.self_test else 0
    print(f'Boundary walking-link source contracts valid: 3 guarded links, 6 paired lanes; {count} adversarial checks.')
    print('Static metadata only; native collision, controller inputs and earned progression remain separate proofs.')


if __name__ == '__main__':
    main()
