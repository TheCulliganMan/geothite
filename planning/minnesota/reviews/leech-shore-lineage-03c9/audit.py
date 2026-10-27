#!/usr/bin/env python3
"""Read-only 541 shore lineage and bounded 538 geometry-merge preflight."""
from collections import Counter
from copy import deepcopy
import base64
import hashlib
import json
from pathlib import Path
import sys

ROOT = Path(__file__).resolve().parents[4]
HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(ROOT / 'planning/minnesota/tools'))
from render_reference import load_pack
from progress_atlas_story import module_digest

INPUTS = {
    'story_parent538': ('.local/minnesota-city-revisions/lake-country-da27b381f69f64d6530383216077de0802c5b9dc42b12c316319cbc5daeb6d50.crystalpack', 'da27b381f69f64d6530383216077de0802c5b9dc42b12c316319cbc5daeb6d50'),
    'shore541': ('content-packs/minnesota-northstar-leech-shore.crystalpack', '03c9d073629c60debc66368f07a4170d0b305026c84fd3d5fdcfd96f95c60a97'),
    'geometry538': ('content-packs/minnesota-northstar-map-quality.crystalpack', '9aa950a35453f06cfee2e4a9b674c6d902af7c311f9ca64a083665ac6c60ddb2'),
}
ADDED = {'MnLeechLake', 'MnLeechLakeCareCabin', 'MnLeechLakeFisherHouse'}
CAPS = [
    ('MnR07C00R00', [11, 0], 2, 5), ('MnR07C00R01', [11, 23], 2, 5),
    ('MnR08C00R01', [11, 23], 2, 5), ('MnR08C00R01', [12, 23], 2, 5),
    ('MnR08C00R01', [14, 23], 2, 5), ('MnR08C00R01', [15, 23], 2, 5),
    ('MnR08C00R00', [0, 17], 2, 5), ('MnR08C00R00', [0, 18], 2, 5),
    ('MnBemidji', [12, 0], 1, 5), ('MnBemidji', [0, 12], 1, 5),
    ('MnBemidji', [23, 12], 1, 5), ('MnBrainerd', [16, 0], 5, 1),
]
LINKS = [
    ('MnBrainerd', 'MnR07C00R01', 'north', 6, [[32, 0], [33, 0]], [[20, 47], [21, 47]]),
    ('MnLeechLake', 'MnR07C00R00', 'south', 6, [[32, 47], [33, 47]], [[20, 0], [21, 0]]),
    ('MnLeechLake', 'MnR08C00R01', 'north', 3, [[32, 0], [33, 0]], [[26, 47], [27, 47]]),
    ('MnBemidji', 'MnR08C00R00', 'south', -8, [[24, 47], [25, 47]], [[40, 0], [41, 0]]),
]


def canonical(value):
    return json.dumps(value, sort_keys=True, separators=(',', ':'))


def main():
    packs, inputs = {}, {}
    for name, (relative, expected) in INPUTS.items():
        path = ROOT / relative
        actual = hashlib.sha256(path.read_bytes()).hexdigest()
        assert actual == expected, name + ': input epoch changed'
        packs[name] = load_pack(path)
        inputs[name] = {'path': relative, 'file_sha256': actual, 'maps': len(packs[name]['data']['maps'])}
    old, new, geometry = [packs[k] for k in ['story_parent538', 'shore541', 'geometry538']]
    a, b, g = [p['data'] for p in [old, new, geometry]]
    checks = []
    def check(name, result):
        checks.append({'check': name, 'passed': bool(result)})
    check('exact538→541 addition with no removed maps', len(a['maps']) == 538 and len(b['maps']) == 541 and set(b['maps']) - set(a['maps']) == ADDED and set(a['maps']) <= set(b['maps']))
    changed = {n for n in a['maps'] if a['maps'][n] != b['maps'][n]}
    expected_changed = {entry[0] for entry in CAPS}
    check('exact6 existing changed modules', changed == expected_changed)
    check('all532 unrelated whole modules exact', all(a['maps'][n] == b['maps'][n] for n in set(a['maps']) - expected_changed))
    for field in ['format_version', 'compiled_audio', 'audio_manifest', 'audio_compression', 'runtime_files']:
        check('whole ' + field + ' exact', old[field] == new[field])
    def kind(name, tile):
        x, y = tile
        m = b['maps'][name]
        block = m['blocks'][y // 2 * m['attributes']['width'] + x // 2]
        return b['tilesets'][m['attributes']['tileset_name']]['collision'][f'{block:02x}'][y % 2 * 2 + x % 2]
    terrain_changes = []
    for name in sorted(changed):
        before, after = a['maps'][name], b['maps'][name]
        expected = list(before['blocks'])
        for map_name, at, source, target in CAPS:
            if map_name != name:
                continue
            index = at[1] * before['attributes']['width'] + at[0]
            check(name + ' declared terrain preimage ' + str(at), expected[index] == source)
            expected[index] = target
            check(name + ' declared cap/mouth actual collision ' + str(at), all(kind(name, [at[0] * 2 + dx, at[1] * 2 + dy]) == ('WALL' if target == 5 else 'FLOOR') for dy in [0, 1] for dx in [0, 1]))
            terrain_changes.append({'map': name, 'metatile': at, 'before': source, 'after': target})
        check(name + ' exact full grid after declared changes', after['blocks'] == expected)
        check(name + ' all attributes except connection records/flags exact', {k: v for k, v in before['attributes'].items() if k not in {'connections', 'connection_flags'}} == {k: v for k, v in after['attributes'].items() if k not in {'connections', 'connection_flags'}})
        added_connections = []
        for town, route, direction, offset, _, _ in LINKS:
            if name == town:
                added_connections.append({'direction': direction, 'target_map': route, 'offset': offset})
            if name == route:
                added_connections.append({'direction': 'south' if direction == 'north' else 'north', 'target_map': town, 'offset': -offset})
        check(name + ' only explicit reciprocal connection appended', after['attributes']['connections'] == before['attributes']['connections'] + added_connections)
        check(name + ' connection flags match actual direction enum', set(after['attributes']['connection_flags'].split(' | ')) == {c['direction'].upper() for c in after['attributes']['connections']})
        if name != 'MnBrainerd':
            check(name + ' all remaining module fields exact', {k: v for k, v in before.items() if k not in {'attributes', 'blocks'}} == {k: v for k, v in after.items() if k not in {'attributes', 'blocks'}})
    before, after = a['maps']['MnBrainerd'], b['maps']['MnBrainerd']
    appended_fields = ['script_flag_commands', 'script_object_commands', 'script_movements', 'script_text_commands', 'script_variable_commands', 'script_control_commands']
    for field in appended_fields:
        previous = Counter(canonical(v) for v in before[field])
        actual = Counter(canonical(v) for v in after[field])
        check('Brainerd all old typed ' + field + ' records retained exact', not (previous - actual))
        check('Brainerd new typed ' + field + ' records owned only by Lake checkpoint', all(json.loads(v)['source_script'].startswith('MnBrainerdLakeCheck') for v in actual - previous))
    for field in ['scripts', 'script_text_bodies']:
        check('Brainerd all old ' + field + ' records exact', all(after[field].get(k) == v for k, v in before[field].items()))
        check('Brainerd only Lake checkpoint adds ' + field, all(k.startswith('MnBrainerdLakeCheck') for k in set(after[field]) - set(before[field])))
    check('Brainerd original bodies remain an exact prefix', after['objects'][:len(before['objects'])] == before['objects'])
    check('Brainerd exactly2 actual north wardens appended', [(o['object_identifier'], o['x'], o['y'], o['sprite'], o['script']) for o in after['objects'][len(before['objects']):]] == [('MN_BRAINERD_NORTH_WARDEN_WEST', 32, 2, 'SPRITE_OFFICER', 'MnBrainerdLakeCheck'), ('MN_BRAINERD_NORTH_WARDEN_EAST', 33, 2, 'SPRITE_OFFICER', 'MnBrainerdLakeCheck')])
    check('Brainerd original warp/bg events exact', all(before['events'][k] == after['events'][k] for k in ['warps', 'bg_events']))
    expected_coords = [{'x': x, 'y': y, 'scene_id': '-1', 'script_name': root} for x in [32, 33] for y, root in [(3, 'MnBrainerdLakeCheck'), (1, 'MnBrainerdLakeCheckInward')]]
    check('Brainerd only4 exact Lake coord triggers appended', after['events']['coord_events'] == before['events']['coord_events'] + expected_coords)
    new_flags = [v for v in after['script_flag_commands'] if v not in before['script_flag_commands']]
    check('new checkpoint flags are Lake-only', {v['flag_id'] for v in new_flags} == {'ENGINE_MINERALBADGE'})
    stripped = []
    for command in after['map_event_section_commands']:
        if command['command'] == 'coord_event' and command['args'][-1].startswith('MnBrainerdLakeCheck'):
            continue
        if command['command'] == 'object_event' and 'MnBrainerdLakeCheck' in command['args']:
            continue
        entry = deepcopy(command)
        entry['command_index'] = len(stripped)
        stripped.append(entry)
    check('Brainerd old raw event records retained after removing only new coord/guard rows', stripped == before['map_event_section_commands'])
    allowed = {'attributes', 'blocks', 'objects', 'events', 'map_event_section_commands', 'scripts', 'script_text_bodies'} | set(appended_fields)
    check('all other Brainerd module fields exact', all(before[k] == after[k] for k in before if k not in allowed))
    links = []
    for town, route, direction, offset, town_edge, route_edge in LINKS:
        reverse = 'south' if direction == 'north' else 'north'
        check(town + '→' + route + ' reciprocal exact offsets', {'direction': direction, 'target_map': route, 'offset': offset} in b['maps'][town]['attributes']['connections'] and {'direction': reverse, 'target_map': town, 'offset': -offset} in b['maps'][route]['attributes']['connections'])
        check(town + '→' + route + ' actual aligned twoFloor lanes', all(kind(town, t) == kind(route, r) == 'FLOOR' and t[0] - 2 * offset == r[0] for t, r in zip(town_edge, route_edge)))
        links.append({'town': town, 'route': route, 'town_direction': direction, 'offset': offset, 'reciprocal_offset': -offset, 'town_edge': town_edge, 'route_edge': route_edge})
    for name in sorted(ADDED):
        metadata = b['runtime_map_metadata'][b['maps'][name]['attributes']['map_constant']]
        check(name + ' group78 stable identity', (metadata['groupId'], metadata['mapId']) == (78, {'MnLeechLake': 1, 'MnLeechLakeCareCabin': 2, 'MnLeechLakeFisherHouse': 3}[name]))
    check('original46 recovery/HOME rows exact; only2 appended', all(b['runtime_spawn_points'].get(k) == v for k, v in a['runtime_spawn_points'].items()) and set(b['runtime_spawn_points']) - set(a['runtime_spawn_points']) == {'275', '276'})
    check('new recovery destinations exact shore/care', [(b['runtime_spawn_points'][str(i)]['mapName'], b['runtime_spawn_points'][str(i)]['tileX'], b['runtime_spawn_points'][str(i)]['tileY']) for i in [275, 276]] == [('MnLeechLake', 32, 44), ('MnLeechLakeCareCabin', 10, 13)])
    changed_tables = {'maps', 'map_scripts', 'map_attributes', 'map_dimensions', 'map_blocks', 'runtime_spawn_points', 'runtime_map_metadata', 'story_event_script_constants', 'npcs', 'pokegear_landmarks'}
    check('only10 declared data catalogs differ', {k for k in a if a[k] != b.get(k)} == changed_tables)
    check('all other full typed data catalogs exact', all(a[k] == b[k] for k in a if k not in changed_tables))
    global_scripts = a['map_scripts']
    check('all existing global labels exact except owned Brainerd MapEvents', all(b['map_scripts'].get(k) == v for k, v in global_scripts.items() if k != 'MnBrainerd_MapEvents'))
    owned_labels = set()
    for name in ADDED | {'MnBrainerd'}:
        module = b['maps'][name]
        owned_labels.update(module['scripts'])
        owned_labels.update([module['attributes']['map_scripts_label'], module['attributes']['map_events_label']])
        coherent = all(b['map_scripts'].get(k) == v for k, v in module['scripts'].items())
        for label, field in [('map_events_label', 'map_event_section_commands'), ('map_scripts_label', 'map_script_section_commands')]:
            raw = b['map_scripts'][module['attributes'][label]]
            indexed = module[field]
            coherent = coherent and all(set(row) == {'command', 'args'} for row in raw)
            coherent = coherent and all(set(row) == {'command', 'args', 'command_index'} and row['command_index'] == i for i, row in enumerate(indexed))
            coherent = coherent and raw == [{k: v for k, v in row.items() if k != 'command_index'} for row in indexed]
        check(name + ' all raw script mirrors coherent with exact derived sequential indices', coherent)
    check('only41 owned global labels added and none removed', len(set(b['map_scripts']) - set(global_scripts)) == 41 and set(b['map_scripts']) - set(global_scripts) <= owned_labels and set(global_scripts) <= set(b['map_scripts']))
    check('all541 global map attribute mirrors equal complete installed attributes', set(b['map_attributes']) == set(b['maps']) and all(b['map_attributes'][k] == m['attributes'] for k, m in b['maps'].items()))
    check('all541 global NPC mirrors equal complete installed objects', set(b['npcs']) == set(b['maps']) and all(b['npcs'][k] == m['objects'] for k, m in b['maps'].items()))
    check('all541 global block payloads equal complete installed grids', len(b['map_blocks']) == len(b['maps']) and all(list(base64.b64decode(b['map_blocks'][m['attributes']['blocks_label']], validate=True)) == m['blocks'] for m in b['maps'].values()))
    for field in ['runtime_map_metadata', 'map_dimensions']:
        check('all old ' + field + ' rows exact and only3 new constants appended', all(b[field].get(k) == v for k, v in a[field].items()) and len(b[field]) == len(a[field]) + 3)
    check('only exact mapgroup78 global constant appended', b['story_event_script_constants']['global'] == a['story_event_script_constants']['global'] | {'MAPGROUP_MINNESOTA_LEECH_SHORE': 78} and {k: v for k, v in b['story_event_script_constants'].items() if k != 'global'} == {k: v for k, v in a['story_event_script_constants'].items() if k != 'global'})
    for field in a['pokegear_landmarks']:
        if field == 'map_to_landmark':
            check('all original landmark bindings exact; only3 new maps bound', all(b['pokegear_landmarks'][field].get(k) == v for k, v in a['pokegear_landmarks'][field].items()) and len(b['pokegear_landmarks'][field]) == len(a['pokegear_landmarks'][field]) + 3)
        else:
            check('whole landmark ' + field + ' exact', a['pokegear_landmarks'][field] == b['pokegear_landmarks'][field])
    gyms = [n for n in a['maps'] if n.startswith('Mn') and n.endswith('Gym')]
    check('all8 complete gyms/Pine course exact acrossall3 epochs', len(gyms) == 8 and all(a['maps'][n] == b['maps'][n] == g['maps'][n] for n in gyms))
    geometry_changed = {n for n in a['maps'] if a['maps'][n] != g['maps'][n]}
    check('nine geometry module changes do not overlap6 shore changes', len(geometry_changed) == 9 and not (geometry_changed & changed))
    for name in geometry_changed:
        check(name + ' geometry branch keeps all programs/events/actors', all(a['maps'][name][k] == g['maps'][name][k] for k in a['maps'][name] if k not in {'attributes', 'blocks'}))
    check('geometry branch changes only five known catalog fields', {k for k in a if a[k] != g[k]} == {'maps', 'map_attributes', 'map_blocks', 'items', 'tilesets'})
    items = deepcopy(g['items'])
    check('canonical independent Repel repair is200→100', a['items']['REPEL']['repel_steps'] == b['items']['REPEL']['repel_steps'] == 200 and g['items']['REPEL']['repel_steps'] == 100)
    items['REPEL']['repel_steps'] = a['items']['REPEL']['repel_steps']
    check('all geometry item fields except Repel duration exact', items == a['items'])
    check('only mn_r10_rockstairs tileset appended', set(g['tilesets']) - set(a['tilesets']) == {'mn_r10_rockstairs'} and all(g['tilesets'].get(k) == v for k, v in a['tilesets'].items()))
    art_files = sorted(set(geometry['runtime_files']) - set(old['runtime_files']))
    check('exact5 R10 art files appended, originals exact', len(art_files) == 5 and all('mn_r10_rockstairs' in k for k in art_files) and all(geometry['runtime_files'].get(k) == v for k, v in old['runtime_files'].items()))
    report = {'status': 'PASS' if all(c['passed'] for c in checks) else 'FAIL', 'scope': 'READ ONLY lineage comparison and merge preflight; no pack output/native gameplay execution', 'inputs': inputs, 'checks': checks, 'errors': [c['check'] for c in checks if not c['passed']], 'new_map_ids': sorted(ADDED), 'changed_existing_maps': sorted(changed), 'terrain_patches': terrain_changes, 'four_new_links': links, 'new_recovery_ids': [275, 276], 'preserved_gyms': gyms, 'preserved_original_module_count': 532, 'geometry_maps_to_merge': sorted(geometry_changed), 'geometry_independent_catalog_patch': {'item': 'REPEL', 'field': 'repel_steps', 'from': 200, 'to': 100}, 'geometry_new_tileset': {'id': 'mn_r10_rockstairs', 'canonical_sha256': module_digest(g['tilesets']['mn_r10_rockstairs']), 'runtime_files': [{'path': k, 'sha256': hashlib.sha256(bytes(geometry['runtime_files'][k])).hexdigest()} for k in art_files]}, 'protected_pine_modules': [{'map': n, 'whole_module_sha256': module_digest(b['maps'][n])} for n in sorted(b['maps']) if n.startswith('MnBemidji')], 'existing_module_hashes': [{'map': n, 'before': module_digest(a['maps'][n]), 'after': module_digest(b['maps'][n]), 'whole_exact': a['maps'][n] == b['maps'][n]} for n in sorted(a['maps'])]}
    raw = (json.dumps(report, indent=2) + '\n').encode()
    digest = hashlib.sha256(raw).hexdigest()
    path = HERE / ('lineage-proof-' + digest + '.json')
    if path.exists():
        assert path.read_bytes() == raw
    else:
        path.write_bytes(raw)
    print(json.dumps({'status': report['status'], 'checks': len(checks), 'errors': report['errors'], 'proof': str(path), 'sha256': digest}))


if __name__ == '__main__':
    main()
