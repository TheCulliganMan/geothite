#!/usr/bin/env python3
"""Prepare guarded compact Superior sources; canonical files stay read-only by default.

Native pack writing belongs to Rust. This preparer reads the explicit final pack,
compares terrain/attributes/anchors/paired warps, then saves immutable before and
candidate source files. --apply runs the same mandatory final-pack cell/anchor
parity checks before writing. An optional Rust parity log can also be SHA-bound.
Use a fresh --output directory for every invocation.
"""
import argparse
import copy
from collections import deque
import hashlib
import json
from pathlib import Path
import sys

ROOT = Path(__file__).resolve().parents[4]
sys.path.insert(0, str(ROOT / 'planning/minnesota/tools'))
from render_reference import layout_blocks, load_pack

REV = 'planning/minnesota/revisions/superior-gunflint-quality-v01'
COAST = 'planning/minnesota/revisions/north-shore-ely-geography-v01/coast-quality-v01/source-v04'
CEDAR = REV + '/cedar-source-v02'
RETIRED = {'MnR12C01R00', 'MnR12C01R01', 'MnR12C00R02', 'MnR12C01R02', 'MnMaraisInlandNeck'}
PARITY_TEST = 'superior_inland_canonical_rust_source_matches_every_published_terrain_cell'


def digest(raw):
    return hashlib.sha256(raw).hexdigest()


def read(path):
    return json.loads(Path(path).read_bytes())


def encoded(value):
    return (json.dumps(value, indent=2, ensure_ascii=False) + '\n').encode()


def connection(maps, name, side, target, offset):
    found = [e for e in maps[name]['attributes']['connections'] if e['direction'] == side]
    assert found == [dict(direction=side, target_map=target, offset=offset)], (name, side, found)


def tile_kind(data, name, point):
    m = data['maps'][name]
    x, y = point
    a = m['attributes']
    assert 0 <= x < a['width'] * 2 and 0 <= y < a['height'] * 2
    b = m['blocks'][y // 2 * a['width'] + x // 2]
    return data['tilesets'][a['tileset_name']]['collision'][f'{b:02x}'][y % 2 * 2 + x % 2]


def map_name(data, token):
    """Warp payloads retain native constants while module keys are names."""
    if token in data['maps']:
        return token
    resolved = [name for name, module in data['maps'].items()
                if module['attributes'].get('map_constant') == token]
    assert len(resolved) == 1, ('unresolved native map token', token)
    return resolved[0]


def exterior_positions(data):
    """Prove actual native origins, reciprocal offsets, cycles and nonoverlap."""
    maps = {n: m for n, m in data['maps'].items() if n.startswith('Mn')
            and (m['attributes'].get('environment') or '').lower() in ['town', 'route', 'outdoor']}
    positions = {'MnMinneapolis': (0, 0)}
    queue = deque(positions)
    opposite = dict(north='south', south='north', east='west', west='east')
    while queue:
        name = queue.popleft()
        a = maps[name]['attributes']
        x, y = positions[name]
        for edge in a['connections']:
            target = map_name(data, edge['target_map'])
            b = maps[target]['attributes']
            d, offset = edge['direction'], edge['offset']
            origin = {'north': (x+offset, y-b['height']), 'south': (x+offset, y+a['height']),
                      'west': (x-b['width'], y+offset), 'east': (x+a['width'], y+offset)}[d]
            connection(data['maps'], target, opposite[d], name, -offset)
            if target in positions:
                assert positions[target] == origin, ('inconsistent native connection cycle', name, target)
            else:
                positions[target] = origin
                queue.append(target)
    assert set(positions) == set(maps), 'disconnected actual Minnesota exterior'
    names = sorted(maps)
    for i, name in enumerate(names):
        x, y = positions[name]
        a = maps[name]['attributes']
        for other in names[i+1:]:
            xx, yy = positions[other]
            b = maps[other]['attributes']
            assert not (x < xx+b['width'] and xx < x+a['width'] and y < yy+b['height'] and yy < y+a['height']), ('native exterior overlap', name, other)
    expected = {'MnGrandMarais': (226,-400), 'MnR11C01R00': (222,-400),
                'MnMaraisCedarApproach': (222,-412), 'MnR12C00R00': (190,-432),
                'MnR12C00R01': (190,-408), 'MnBoundaryWaters': (158,-430)}
    for name, origin in expected.items():
        assert positions[name] == origin, ('actual native origin differs', name, positions[name], origin)
    return {name: list(positions[name]) for name in expected}


def aggregate_anchor(data, prefix, point):
    x, y = point
    if prefix == 'MnR12':
        return ('MnR12C00R00', (x, y)) if y < 48 else ('MnR12C00R01', (x, y - 48))
    return prefix, (x, y)


def check_dossier(data, dossier, prefix):
    """Source IDs and positions must name the actual published native actor/event."""
    for category in ['trainers', 'residents', 'items', 'actors']:
        for actor in dossier.get(category, []):
            point = actor.get('tile')
            if point is None:
                continue
            name, local = aggregate_anchor(data, prefix, point)
            matches = [o for o in data['maps'][name]['objects']
                       if [o['x'], o['y']] == list(local)
                       and actor['id'] in [o.get('script'), o.get('object_identifier'), o.get('label')]]
            assert len(matches) == 1, ('native actor differs', category, actor['id'], name, local)
            if category == 'items':
                pickups = [v for v in data['maps'][name]['script_field_pickups'] if v['source_script'] == actor['id']]
                assert len(pickups) == 1 and pickups[0]['item_id'] == actor['item'], ('native pickup differs', actor)
                assert pickups[0]['quantity'] == actor.get('quantity', 1)
                assert pickups[0]['event_flag'] == actor.get('flag', actor.get('event_flag'))
    for sign in dossier.get('signs', []):
        name, local = aggregate_anchor(data, prefix, sign['tile'])
        assert any([e['x'], e['y']] == list(local) and e['script'] == sign['id']
                   for e in data['maps'][name]['events']['bg_events']), ('native sign differs', sign)
    for door in dossier.get('doors', []):
        name, local = aggregate_anchor(data, prefix, door['tile'])
        entry = [w for w in data['maps'][name]['events']['warps']
                 if w['index'] == door['warp_index'] and [w['x'], w['y']] == list(local)]
        assert len(entry) == 1 and map_name(data, entry[0]['target_map']) == door['target'], ('native door differs', door)
        assert entry[0]['target_warp_id'] == door['target_warp']
        room = data['maps'][door['target']]
        room_exits = [w for w in room['events']['warps'] if map_name(data, w['target_map']).startswith('MnR12C')]
        assert room_exits, ('missing room return', door['target'])
        for warp in room_exits:
            assert map_name(data, warp['target_map']) == name, ('room returns to wrong chunk', warp)
            assert warp['target_warp_id'] == door['warp_index'], ('room returns to wrong exterior entry', warp)
        return_name, clear = aggregate_anchor(data, prefix, door['return_tile'])
        assert return_name in data['maps'] and tile_kind(data, return_name, clear) == 'FLOOR', ('unsafe return', door)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--pack', type=Path, required=True)
    parser.add_argument('--expected-pack-sha256', required=True)
    parser.add_argument('--gunflint-source', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--replay-contract', type=Path, required=True)
    parser.add_argument('--apply', action='store_true')
    parser.add_argument('--parity-evidence', type=Path)
    parser.add_argument('--expected-parity-evidence-sha256')
    args = parser.parse_args()
    assert not args.output.exists(), 'use a fresh preparation epoch'
    assert digest(args.pack.read_bytes()) == args.expected_pack_sha256, 'explicit final pack SHA differs'
    if args.parity_evidence or args.expected_parity_evidence_sha256:
        assert args.parity_evidence and args.expected_parity_evidence_sha256, 'provide both Rust parity evidence and its SHA'
        evidence = args.parity_evidence.read_bytes()
        assert digest(evidence) == args.expected_parity_evidence_sha256, 'parity log changed'
        text = evidence.decode()
        assert PARITY_TEST in text and 'test result: ok.' in text and 'FAILED' not in text, 'Rust parity is not proven'
    pack = load_pack(args.pack)
    data = pack['data']
    maps = data['maps']
    assert not RETIRED.intersection(maps), 'retired native modules remain'
    for name, dims in [('MnR11C01R00', [4, 24]), ('MnR12C00R00', [32, 24]),
                       ('MnR12C00R01', [32, 8]), ('MnMaraisCedarApproach', [20, 12]),
                       ('MnGrandMarais', [20, 19])]:
        assert [maps[name]['attributes']['width'], maps[name]['attributes']['height']] == dims, name
    for row in data['runtime_map_metadata'].values():
        assert row['name'] not in RETIRED, 'retired runtime map registry remains'
    replay = read(args.replay_contract)
    source_files = {
        'coast_layout': ROOT / COAST / 'MN_R11.layout.candidate.json',
        'coast_dossier': ROOT / COAST / 'MN_R11.dossier.candidate.json',
        'cedar_layout': ROOT / CEDAR / 'MnMaraisCedarApproach.candidate.layout.json',
        'cedar_dossier': ROOT / CEDAR / 'MnMaraisCedarApproach.candidate.json',
        'r12_layout': args.gunflint_source / 'MN_R12.candidate.layout.json',
        'r12_dossier': args.gunflint_source / 'MN_R12.candidate.json',
    }
    candidates = {k: read(v) for k, v in source_files.items()}
    original_cedar_cells = layout_blocks(candidates['cedar_layout'])
    assert 'reject_cliff_cuts' not in candidates['cedar_layout'], 'unexpected Cedar schema epoch'
    candidates['cedar_layout']['reject_cliff_cuts'] = True
    assert layout_blocks(candidates['cedar_layout']) == original_cedar_cells, 'Cedar schema normalization changed terrain'
    bound_inputs = dict(coast=ROOT / COAST / 'compact-bend.contract.json',
                        r12_layout=source_files['r12_layout'], r12_dossier=source_files['r12_dossier'],
                        cedar=ROOT / CEDAR / 'contract.json')
    assert replay['id'] == 'MN_COMPACT_SUPERIOR_V1'
    assert set(replay['source_sha256']) == set(bound_inputs)
    for key, source in bound_inputs.items():
        assert digest(source.read_bytes()) == replay['source_sha256'][key], ('replay source differs', key)
    coast_guards = read(ROOT / COAST / 'ragged-footprint.contract.json')['source_sha256']
    for name, expected in coast_guards.items():
        assert digest((ROOT / COAST / name).read_bytes()) == expected, ('coast source changed', name)
    cedar_contract = read(bound_inputs['cedar'])
    assert layout_blocks(candidates['cedar_layout']) == [int(v,16) for row in cedar_contract['rows'] for v in row.split()]

    # Bind each actual canonical preimage to the detached author's before bytes.
    before_pairs = [
        ('planning/minnesota/routes/MN_R11.layout.json', ROOT / COAST / 'MN_R11.layout.before.json'),
        ('planning/minnesota/routes/MN_R11.json', ROOT / COAST / 'MN_R11.dossier.before.json'),
        ('planning/minnesota/routes/MN_R12.layout.json', args.gunflint_source / 'MN_R12.layout.json.before'),
        ('planning/minnesota/routes/MN_R12.json', args.gunflint_source / 'MN_R12.json.before'),
        ('planning/minnesota/towns/MnMaraisCedarApproach.layout.json', ROOT / CEDAR / 'MnMaraisCedarApproach.layout.json.before'),
        ('planning/minnesota/towns/MnMaraisCedarApproach.json', ROOT / CEDAR / 'MnMaraisCedarApproach.json.before'),
    ]
    for canonical, before in before_pairs:
        assert (ROOT / canonical).read_bytes() == before.read_bytes(), ('stale authoring preimage', canonical)
    # Terrain parity follows actual native footprints, not the retired source canvas.
    for key, names in [('coast_layout', [f'MnR11C{c:02}R{r:02}' for r in range(4) for c in range(2)]),
                       ('r12_layout', ['MnR12C00R00', 'MnR12C00R01']),
                       ('cedar_layout', ['MnMaraisCedarApproach'])]:
        layout = candidates[key]
        cells = layout_blocks(layout)
        width = layout['dimensions_metatiles'][0]
        for name in names:
            m = maps[name]
            a = m['attributes']
            if key == 'coast_layout':
                c, r = int(name[6:8]), int(name[9:11])
                ox, oy = c * 32, r * 24
            elif key == 'r12_layout':
                ox, oy = 0, 24 if name.endswith('R01') else 0
            else:
                ox, oy = 0, 0
            expected = [cells[(oy+y)*width+ox+x] for y in range(a['height']) for x in range(a['width'])]
            assert expected == m['blocks'], ('native/source terrain differs', name)
            assert a['tileset_name'] == layout['compiled_tileset'], ('native/source tileset differs', name)
    check_dossier(data, candidates['r12_dossier'], 'MnR12')
    check_dossier(data, candidates['cedar_dossier'], 'MnMaraisCedarApproach')
    edges = [('MnR11C01R00', 'east', 'MnGrandMarais', 0),
             ('MnGrandMarais', 'north', 'MnMaraisCedarApproach', -4),
             ('MnMaraisCedarApproach', 'west', 'MnR12C00R01', 4),
             ('MnR12C00R00', 'west', 'MnBoundaryWaters', 2),
             ('MnR12C00R00', 'south', 'MnR12C00R01', 0)]
    opposite = dict(east='west', west='east', north='south', south='north')
    for a, side, b, offset in edges:
        connection(maps, a, side, b, offset)
        connection(maps, b, opposite[side], a, -offset)
    assert not any(e['direction'] == 'east' for e in maps['MnMaraisCedarApproach']['attributes']['connections'])
    assert not any(e['direction'] == 'south' for e in maps['MnR12C00R01']['attributes']['connections'])
    native_positions = exterior_positions(data)
    old = {}
    outputs = {}

    def edit(path):
        if path not in old:
            old[path] = (ROOT / path).read_bytes()
            outputs[path] = json.loads(old[path])
        return outputs[path]

    def replace(path, value):
        edit(path)
        outputs[path] = copy.deepcopy(value)

    publication = dict(status='MATCHES_COMPOSED_NATIVE_GEOGRAPHY', final_pack_sha256=args.expected_pack_sha256,
                       content_identity=pack['identity']['content_hash'],
                       verification='revisions/superior-gunflint-quality-v01/VERIFICATION.md',
                       progression_scope='Explicit local fixtures do not establish earned campaign progression.')
    for key, destination in [('coast_layout', 'routes/MN_R11.layout.json'), ('coast_dossier', 'routes/MN_R11.json'),
                             ('r12_layout', 'routes/MN_R12.layout.json'), ('r12_dossier', 'routes/MN_R12.json'),
                             ('cedar_layout', 'towns/MnMaraisCedarApproach.layout.json'),
                             ('cedar_dossier', 'towns/MnMaraisCedarApproach.json')]:
        value = copy.deepcopy(candidates[key])
        if key.endswith('dossier'):
            value['status'] = publication['status']
            value['native_geography_publication'] = publication
        if key == 'r12_dossier':
            value['native_integration']['native_status'] = (
                'PUBLISHED: two native chunks32x24 and32x8 match final pack '
                + args.expected_pack_sha256 + '. Terrain, relocated anchors and paired room returns '
                'are checked against the actual pack; controller/visual evidence is recorded separately in the verification report.')
            value['implementation'] = (
                'Published compact32x32 Gunflint native route. Exact source terrain, endpoints, '
                'actor/item/sign anchors, room entries and clear returns match the bound final pack. '
                'Original room content, parties, flags, rewards and story identities are retained. '
                'Local production controller and visual acceptance are recorded in '
                'revisions/superior-gunflint-quality-v01/VERIFICATION.md; earned campaign progression is separate.')
            value['anchor_status'] = (
                'Exact source/native anchor and paired-warp parity checked during this promotion; '
                + str(len(replay['module_preimages'])) + ' original native module preimages are bound by the Rust replay contract. '
                'The final pack SHA and source bindings are recorded in native_geography_publication and active world replay metadata.')
        replace('planning/minnesota/' + destination, value)
    town = edit('planning/minnesota/towns/MnGrandMarais.json')
    town['native_geography_publication'] = publication
    town['world_origin_proposed'] = [226, -400]
    for port in town['ports']:
        if port['side'] == 'N':
            port['native_connection'] = 'Town north offset-4 to Cedar south+4; town core18/19 to Cedar26/27. Winter begins inland.'
    town['implementation']['review'] = '../revisions/superior-gunflint-quality-v01/VERIFICATION.md'
    # The town's local terrain, doors, rooms and services are untouched.
    town_layout = read(ROOT / 'planning/minnesota/towns/MnGrandMarais.layout.json')
    assert layout_blocks(town_layout) == maps['MnGrandMarais']['blocks'], 'town local terrain changed'
    # Existing six local doors and every installed room stay exact.
    design = read(ROOT / 'planning/minnesota/playable/grand-marais.json')
    for door in design['doors']:
        assert any(w['index'] == door['id'] and [w['x'],w['y']] == door['tile']
                   and w['target_map_constant'] == door['target']
                   for w in maps['MnGrandMarais']['events']['warps']), ('town door drift', door)
    room_guards = read(ROOT / 'planning/minnesota/revisions/north-shore-ely-geography-v01/grand-marais/source-v02/source-preimages.json')['native_rooms_sha256']
    for name, expected in room_guards.items():
        actual = json.dumps(maps[name],sort_keys=True,separators=(',',':'),ensure_ascii=False).encode()
        assert digest(actual) == expected, ('original town room changed', name)
    town_connections = edit('planning/minnesota/towns/MnGrandMarais.connections.json')
    town_connections.update(world_origin_proposed=[226, -400], attributes_after=maps['MnGrandMarais']['attributes'],
                            pack_sha256=args.expected_pack_sha256, content_identity=pack['identity']['content_hash'])
    for seam in town_connections['foot_seams']:
        if seam['side'] == 'west':
            seam.update(target_core=[[7, 8], [7, 9]], offset=0)
        elif seam['side'] == 'north':
            seam.update(target_core=[[26, 23], [27, 23]], offset=-4)
    if 'surf_seam' in town_connections:
        town_connections['surf_seam']['target_core_x'] = 7
    def compact_boundary_port(record):
        ports = [v for v in record['ports'] if v.get('route') == 'MN_R12' and v.get('side') == 'E']
        assert len(ports) == 1, 'exact Boundary Waters Gunflint port required'
        native = ports[0]['native_connection']
        assert native['target_map'] == 'MnR12C00R01' and native['direction'] == 'east'
        assert native['offset_metatiles'] == 8 and native['reciprocal_offset_metatiles'] == -8
        native.update(target_map='MnR12C00R00', offset_metatiles=-2, reciprocal_offset_metatiles=2)

    boundary = edit('planning/minnesota/towns/MnBoundaryWaters.json')
    compact_boundary_port(boundary)
    checkpoints = [v for v in boundary['regional_checkpoints']['checkpoints'] if v['id'] == 'gunflint']
    assert len(checkpoints) == 1
    checkpoint = checkpoints[0]
    assert checkpoint['native_target_map'] == 'MnR12C00R01' and checkpoint['direction'] == 'east'
    assert checkpoint['offset_metatiles'] == 8 and checkpoint['reciprocal_offset_metatiles'] == -8
    checkpoint.update(native_target_map='MnR12C00R00', offset_metatiles=-2, reciprocal_offset_metatiles=2)
    # R13/league and R20/Voyageurs ports, flags, officers and programs stay exact.
    neck = edit('planning/minnesota/towns/MnMaraisInlandNeck.json')
    neck.update(status='RETIRED_HISTORY_NOT_ACTIVE_NATIVE_GEOGRAPHY',
                retirement_reason='Long snowy connector removed; camp pickup/flag retained in compact Gunflint.',
                retired_by_pack_sha256=args.expected_pack_sha256)
    interior = edit('planning/minnesota/interiors.json')
    doors = {d['target']: d for d in candidates['r12_dossier']['doors']}
    for record in interior['contracts']:
        if record.get('interior') not in doors:
            continue
        door = doors[record['interior']]
        record.update(exterior_anchor_metatile=[door['tile'][0]//2, door['tile'][1]//2],
                      exterior_door_core=door['tile'], exterior_arrival_core=door['arrival_tile'],
                      exterior_clear_return_core=door['return_tile'], runtime_settled_exterior_core=door['return_tile'],
                      native_status='Compact exterior paired return matches final native pack; room local content preserved.',
                      validation='Exact final native paired warp and clear-return collision checked by source promotion.',
                      native_review_pack_sha256=args.expected_pack_sha256)
        record['mouths'] = [dict(exterior_door_core=door['tile'], interior_target_warp=door['target_warp'],
                                 exterior_entry_warp=door['warp_index'], exterior_return_warp=door['warp_index'],
                                 safe_exterior_return_core=door['return_tile'])]
        record['return_semantics'] = 'Original room exits return to compact exterior paired entry; ordinary Down reaches the recorded clear Floor.'
    world = edit('planning/minnesota/world.json')
    for route in world['routes']:
        if route['id'] == 'MN_R12':
            route['dimensions_metatiles'] = [32, 32]
    world['chunks'] = [c for c in world['chunks'] if c['id'] not in RETIRED]
    for chunk in world['chunks']:
        if chunk['id'] == 'MnR11C01R00':
            chunk.update(dimensions_metatiles=[4, 24], source_origin_metatiles=[32, 0])
        elif chunk['id'] == 'MnR12C00R01':
            chunk['dimensions_metatiles'] = [32, 8]
    for entry in world['towns']:
        if entry['id'] == 'MnGrandMarais':
            # Update only the owned fields; unrelated town notes remain exact.
            for field in ['native_geography_publication', 'world_origin_proposed', 'ports']:
                entry[field] = copy.deepcopy(town[field])
            entry.setdefault('implementation', {})['review'] = town['implementation']['review']
    boundary_entries = [v for v in world['towns'] if v['id'] == 'MnBoundaryWaters']
    assert len(boundary_entries) == 1
    compact_boundary_port(boundary_entries[0])
    updated = []
    for edge in world['connections']:
        if edge.get('a') in RETIRED or edge.get('b') in RETIRED:
            continue
        if edge.get('route') == 'MN_R12' and edge.get('b') == 'MnBoundaryWaters':
            edge.update(a='MnR12C00R00', offset_a=2, offset_b=-2,
                        a_edge_core=[[0, 28], [0, 29]], b_edge_core=[[63, 24], [63, 25]])
        if edge.get('a') == 'MnGrandMarais' and edge.get('b') == 'MnMaraisCedarApproach':
            edge.update(offset_a=-4, offset_b=4)
        updated.append(edge)
    world['connections'] = updated
    # Existing guarded BW and north-town descriptors survive; add only the new
    # Cedar/lower-R12 seam after removing the obsolete Neck edges.
    world['connections'].append(dict(kind='connection', a='MnMaraisCedarApproach', b='MnR12C00R01',
                                     direction_a='west', direction_b='east', offset_a=4, offset_b=-4,
                                     native_source='revisions/superior-gunflint-quality-v01/BRIEF.md'))
    camps = [v for v in candidates['r12_dossier']['items'] if v.get('flag') == 'EVENT_MN_GUNFLINT_CAMP_SUPPLY']
    assert len(camps) == 1 and camps[0]['item'] == 'BERRY' and camps[0]['quantity'] == 3
    camp_map, _ = aggregate_anchor(data, 'MnR12', camps[0]['tile'])
    assert data['story_event_script_constants']['global']['EVENT_MN_GUNFLINT_CAMP_SUPPLY'] == 2387
    def repo_path(path):
        resolved = path.resolve()
        return str(resolved.relative_to(ROOT)) if resolved.is_relative_to(ROOT) else str(resolved)

    replay_output = 'content-packs/' + args.pack.stem + '-replay.crystalpack'
    replay_command = ['target/runtime-fps/wasm-dev/build_compact_superior',
                      'content-packs/minnesota-superior-inland-winter-v01.crystalpack',
                      repo_path(args.replay_contract), digest(args.replay_contract.read_bytes()),
                      repo_path(bound_inputs['coast']), repo_path(bound_inputs['r12_layout']),
                      repo_path(bound_inputs['r12_dossier']), repo_path(bound_inputs['cedar']), replay_output]
    world['superior_inland_source'] = dict(
        parent='content-packs/minnesota-superior-inland-winter-v01.crystalpack',
        final_pack_sha256=args.expected_pack_sha256, content_identity=pack['identity']['content_hash'],
        builder='build_compact_superior_pack', cli='build_compact_superior',
        replay_contract=str(args.replay_contract.relative_to(ROOT)) if args.replay_contract.is_relative_to(ROOT) else str(args.replay_contract),
        replay_contract_sha256=digest(args.replay_contract.read_bytes()),
        replay_source_files={key: repo_path(path) for key, path in bound_inputs.items()},
        replay_source_sha256={key: digest(path.read_bytes()) for key, path in bound_inputs.items()},
        replay_command=replay_command, replay_output_must_be_fresh=True,
        extra_layouts=['towns/MnMaraisCedarApproach.layout.json'],
        camp_event=dict(name='EVENT_MN_GUNFLINT_CAMP_SUPPLY', value=2387, active_map=camp_map),
        actual_owned_origins_metatiles=native_positions,
        source_canvas=dict(route='MN_R11', dimensions=[56, 96], unpublished_bounds=[36, 0, 56, 24]))
    assert 'MnMaraisInlandNeck' not in json.dumps(world), 'retired Neck remains active in world metadata'
    # Every canonical write is gated by the exact pack/source/native parity
    # checks above; Rust rebuild parity is independent follow-up verification.
    # Preserve whole-file snapshots and exact output bytes for review/replay.
    args.output.mkdir(parents=True)
    manifest = dict(schema_version=1, status='PREPARED_NOT_APPLIED', pack_sha256=args.expected_pack_sha256,
                    content_identity=pack['identity']['content_hash'],
                    source_sha256={str(p): digest(p.read_bytes()) for p in source_files.values()},
                    replay_contract_sha256=digest(args.replay_contract.read_bytes()), files={})
    for index, (name, value) in enumerate(outputs.items()):
        before_name = f'{index:02}-{Path(name).name}.before'
        after_name = f'{index:02}-{Path(name).name}.candidate'
        after = encoded(value)
        (args.output / before_name).write_bytes(old[name])
        (args.output / after_name).write_bytes(after)
        manifest['files'][name] = dict(before=before_name, before_sha256=digest(old[name]),
                                     candidate=after_name, candidate_sha256=digest(after))
    if args.apply:
        # Validate every preimage before the first canonical write.
        for name, item in manifest['files'].items():
            assert digest((ROOT / name).read_bytes()) == item['before_sha256'], ('concurrent source mutation', name)
        for name, item in manifest['files'].items():
            (ROOT / name).write_bytes((args.output / item['candidate']).read_bytes())
        manifest['status'] = 'APPLIED_WITH_EXACT_PREIMAGES_AND_NATIVE_PARITY'
    (args.output / 'manifest.json').write_bytes(encoded(manifest))
    print(json.dumps(dict(status=manifest['status'], files=len(outputs), output=str(args.output),
                          pack_sha256=args.expected_pack_sha256), indent=2))


if __name__ == '__main__':
    main()
