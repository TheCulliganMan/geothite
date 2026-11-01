#!/usr/bin/env python3
"""Resolve the reviewed continuation's source rows and compare its actual pack.

This checks authoring provenance and installed terrain, not gameplay or visual
quality. Rust remains responsible for building and serializing the pack.
"""
import argparse
import hashlib
import json
from pathlib import Path
import re

from render_reference import load_pack


ROOT = Path(__file__).resolve().parents[3]
DEFAULT_INDEX = ROOT / 'planning/minnesota/revisions/reviewed-maps-v01/sources-v11.json'


def require(condition, message):
    if not condition:
        raise ValueError(message)


def bound_source(binding):
    path = Path(binding['path'])
    require(not path.is_absolute() and '..' not in path.parts, 'relative source required')
    raw = (ROOT / path).read_bytes()
    require(hashlib.sha256(raw).hexdigest() == binding['sha256'], f'source drift: {path}')
    return json.loads(raw)


def pointer(value, path):
    require(path.startswith('/'), 'absolute JSON pointer required')
    for part in path[1:].split('/'):
        part = part.replace('~1', '/').replace('~0', '~')
        value = value[int(part)] if isinstance(value, list) else value[part]
    return value


def audit(pack_path, index_path):
    index = json.loads(index_path.read_text())
    require(index['schema_version'] == 1
            and index['id'] in {'minnesota-reviewed-maps-sources-v1',
                                'minnesota-reviewed-maps-sources-v2',
                                'minnesota-reviewed-maps-sources-v3',
                                'minnesota-reviewed-maps-sources-v4',
                                'minnesota-reviewed-maps-sources-v5',
                                'minnesota-reviewed-maps-sources-v6',
                                'minnesota-reviewed-maps-sources-v7',
                                'minnesota-reviewed-maps-sources-v8',
                                'minnesota-reviewed-maps-sources-v9',
                                'minnesota-reviewed-maps-sources-v10',
                                'minnesota-reviewed-maps-sources-v11'}, 'reviewed index required')
    require(hashlib.sha256(pack_path.read_bytes()).hexdigest() == index['pack_sha256'],
            'installed reviewed pack differs')
    recipe = bound_source(index['build_recipe'])
    require(recipe['expected_pack_sha256'] == index['pack_sha256'], 'recipe output differs')
    require(index['parent_source_index'] == recipe['parent_source_index']['path']
            and index['parent_source_index_sha256'] == recipe['parent_source_index']['sha256'],
            'parent source index differs')
    parent = bound_source(recipe['parent_source_index'])
    if index['id'] in {'minnesota-reviewed-maps-sources-v3',
                       'minnesota-reviewed-maps-sources-v4',
                       'minnesota-reviewed-maps-sources-v5',
                       'minnesota-reviewed-maps-sources-v6',
                       'minnesota-reviewed-maps-sources-v7',
                       'minnesota-reviewed-maps-sources-v8',
                       'minnesota-reviewed-maps-sources-v9',
                       'minnesota-reviewed-maps-sources-v10',
                       'minnesota-reviewed-maps-sources-v11'}:
        replaced_fields = {'schema_version', 'parent_revision', 'parent_source_index',
                           'parent_source_index_sha256', 'pack', 'pack_sha256',
                           'parent_pack', 'parent_pack_sha256', 'content_identity',
                           'current_layouts'}
        for name, binding in parent.items():
            if name not in replaced_fields:
                require(index.get(name) == binding, f'inherited source catalog changed: {name}')
    sources = {}
    latest = {}
    for stage in recipe['stages']:
        for binding in stage['sources']:
            source = bound_source(binding)
            path = binding['path']
            sources[path] = source
            terrain_maps = source.get('maps', [])
            for i, terrain in enumerate(terrain_maps if isinstance(terrain_maps, list) else []):
                key = 'after_rows' if 'after_rows' in terrain else 'rows'
                if key in terrain:
                    latest[terrain['id']] = (path, f'/maps/{i}/{key}', stage['kind'])
            if 'rows' in source and Path(path).name != 'before.json':
                latest[source.get('map') or source['id']] = (path, '/rows', stage['kind'])
    require(set(latest) == set(index['current_native_layouts']),
            'missing or extraneous final native terrain owner')
    pack = load_pack(pack_path)
    require(any(stage['kind'] == 'safari_expedition' for stage in recipe['stages'])
            == ('current_safari_zone' in index), 'missing or extraneous Safari source owner')
    if 'current_safari_zone' in index:
        binding = index['current_safari_zone']
        stages = [stage for stage in recipe['stages'] if stage['kind'] == 'safari_expedition']
        require(len(stages) == 1 and stages[0]['kind'] == binding['stage']
                and stages[0]['sources'] == [
                    {key: binding[key] for key in ['path', 'sha256']}],
                'Safari source owner differs')
        zone = bound_source(binding)
        installed = dict(pack['data']['safari_zone'])
        installed.setdefault('fishing_flee_threshold', 64)
        require(installed == zone, 'installed Safari configuration differs')
        actors = pack['data']['maps'][zone['admission_map']]['objects']
        ids = {obj['object_identifier'] for obj in actors}
        require(not ids.intersection(binding['retired_obsolete_public_guard_ids'])
                and zone['admission_script'] in ids, 'public ranger/guard binding differs')
    def check_town_anchors(kind, owner, label):
        stages = [stage for stage in recipe['stages'] if stage['kind'] == kind]
        require(bool(stages) == (owner in index),
                f'missing or extraneous {label} anchor owner')
        if not stages:
            return 0
        binding = index[owner]
        require(len(stages) == 1
                and stages[0]['kind'] == binding['stage']
                and stages[0]['sources'] == [
                    {key: binding[key] for key in ['path', 'sha256']}],
                f'{label} anchor source owner differs')
        source = bound_source(binding)
        town = pack['data']['maps'][source['map']]
        require(town['attributes']['connections'] == source['preserved_connections'],
                f'{label} road connections changed')
        for moves, records, key in [
            (source['warp_moves'], town['events']['warps'], 'index'),
            (source.get('actor_moves', []), town['objects'], 'object_identifier'),
            (source.get('sign_moves', []), town['events']['bg_events'], 'script'),
        ]:
            for move in moves:
                identity = move.get('id', move.get('index'))
                matches = [row for row in records if row[key] == identity]
                require(len(matches) == 1
                        and [matches[0]['x'], matches[0]['y']] == move.get('to', move.get('to_tile')),
                        f'{label} installed anchor differs: {identity}')
        return sum(len(source.get(key, [])) for key in ['warp_moves', 'actor_moves', 'sign_moves'])

    rochester_moved_anchors = check_town_anchors(
        'rochester_care_green', 'current_rochester_care_green', 'Rochester')
    grand_rapids_moved_anchors = check_town_anchors(
        'grand_rapids_depot_quay', 'current_grand_rapids_depot_quay', 'Grand Rapids')
    mankato_moved_anchors = check_town_anchors(
        'mankato_terrace_pond', 'current_mankato_terrace_pond', 'Mankato')
    brainerd_moved_anchors = check_town_anchors(
        'brainerd_lakeside_loop', 'current_brainerd_lakeside_loop', 'Brainerd')
    farm_stages = [stage for stage in recipe['stages'] if stage['kind'] == 'r02_working_farmyard']
    require(bool(farm_stages) == ('current_r02_working_farmyard' in index),
            'missing or extraneous R02 garden owner')
    r02_new_actors = 0
    if farm_stages:
        binding = index['current_r02_working_farmyard']
        require(len(farm_stages) == 1 and binding['stage'] == 'r02_working_farmyard'
                and farm_stages[0]['sources'] == [
                    {k: binding[k] for k in ['path', 'sha256']}], 'R02 garden source owner differs')
        source = bound_source(binding)
        module = pack['data']['maps'][source['map']]
        spec = source['new_actor']
        actors = [o for o in module['objects'] if o['object_identifier'] == spec['id']]
        require(len(actors) == 1 and [actors[0]['x'], actors[0]['y']] == spec['tile']
                and actors[0]['sprite'] == spec['sprite'] and actors[0]['script'] == spec['id']
                and actors[0]['event_flag'] == '-1', 'R02 garden actor differs')
        require(module['attributes']['connections'] == source['preserved_external_connections'],
                'R02 town roads changed')
        for anchor in source['preserved_native_actor_positions']:
            matches = [o for o in module['objects'] if o['object_identifier'] == anchor['id']]
            require(len(matches) == 1 and [matches[0]['x'], matches[0]['y']] == anchor['tile'],
                    'R02 original actor anchor changed')
        require(source['reward']['event_flag'] in pack['data']['story_event_script_constants']['global'],
                'R02 garden claim flag missing')
        grants = [g for g in module['script_item_grants'] if g['source_script'] == spec['id']]
        require(len(grants) == 1 and grants[0]['item_id'] == source['reward']['item']
                and grants[0]['quantity'] == source['reward']['quantity'], 'R02 native Berry grant differs')
        r02_new_actors = 1
    fork_stages = [stage for stage in recipe['stages'] if stage['kind'] == 'r06_pine_fork']
    require(bool(fork_stages) == ('current_r06_pine_fork' in index),
            'missing or extraneous R06 fork owner')
    r06_new_ledge_lanes = 0
    if fork_stages:
        binding = index['current_r06_pine_fork']
        require(len(fork_stages) == 1 and binding['stage'] == 'r06_pine_fork'
                and fork_stages[0]['sources'] == [
                    {key: binding[key] for key in ['path', 'sha256']}], 'R06 fork source owner differs')
        source = bound_source(binding)
        module = pack['data']['maps']['MnR06C00R00']
        attrs = module['attributes']
        collision = pack['data']['tilesets'][attrs['tileset_name']]['collision']
        def fork_kind(point):
            x, y = point
            block = module['blocks'][y//2*attrs['width']+x//2]
            return collision[f'{block:02x}'][y%2*2+x%2]
        for lane in source['new_ledge']['lanes']:
            require([fork_kind(lane[key]) for key in ['upper', 'lip', 'reverse_wall', 'landing']]
                    == ['FLOOR', 'HOP_DOWN', 'WALL', 'FLOOR'], 'R06 native drop differs')
            r06_new_ledge_lanes += 1
        require(r06_new_ledge_lanes == 2 and all(fork_kind([x,27]) == 'WALL' for x in [41,44]),
                'R06 drop lanes/flanks differ')
        cut = source['retained_cut_gate']
        x,y = cut['metatile']
        require(module['blocks'][y*attrs['width']+x] == int(cut['original'],16),
                'R06 existing Cut gate differs')
    checks = []
    for name, binding in index['current_native_layouts'].items():
        require((binding['path'], binding['rows_pointer'], binding['stage']) == latest[name],
                f'{name}: historical terrain cannot supersede its reviewed successor')
        source = bound_source(binding)
        require(source == sources[binding['path']], f'{name}: recipe/source disagreement')
        width, height = binding['dimensions_metatiles']
        rows = [[int(word, 16) for word in row.split()]
                for row in pointer(source, binding['rows_pointer'])]
        require(len(rows) == height and all(len(row) == width for row in rows),
                f'{name}: complete native rows required')
        module = pack['data']['maps'][name]
        attrs = module['attributes']
        require([attrs['width'], attrs['height']] == [width, height]
                and attrs['tileset_name'] == binding['tileset']
                and module['blocks'] == [block for row in rows for block in row],
                f'{name}: installed terrain or alias differs')
        if 'border_block' in binding:
            border = binding['border_block']
            border = int(border, 16) if isinstance(border, str) else border
            require(attrs['border_block'] == border, f'{name}: border differs')
        checks.append(dict(map=name, source=binding['path'], rows=binding['rows_pointer'],
                           tileset=binding['tileset'], metatiles=width * height))

    groups = {}
    for name in latest:
        match = re.fullmatch(r'MnR(\d\d)C\d\dR\d\d', name)
        if match:
            groups.setdefault('MN_R' + match[1], []).append(name)
        else:
            require(index['current_layouts'][name]
                    == dict(source_kind='source_bound_native_rows',
                            **index['current_native_layouts'][name]),
                    f'{name}: current layout points at older terrain')
    for name, maps in groups.items():
        require(index['current_layouts'][name]
                == dict(source_kind='current_native_layout_group', native_maps=sorted(maps)),
                f'{name}: current route group differs')
    replaced = set(groups) | (set(latest) - {m for maps in groups.values() for m in maps})
    for name, binding in parent['current_layouts'].items():
        if name not in replaced:
            require(index['current_layouts'][name] == binding,
                    f'{name}: unrelated parent source binding changed')
    return dict(status='SOURCE_AND_INSTALLED_TERRAIN_MATCH', pack=str(pack_path),
                source_index=str(index_path),
                source_index_sha256=hashlib.sha256(index_path.read_bytes()).hexdigest(),
                pack_sha256=index['pack_sha256'], native_maps=len(checks),
                route_groups=len(groups), source_files=len(sources), checks=checks,
                rochester_moved_anchors=rochester_moved_anchors,
                grand_rapids_moved_anchors=grand_rapids_moved_anchors,
                mankato_moved_anchors=mankato_moved_anchors,
                brainerd_moved_anchors=brainerd_moved_anchors,
                r02_new_actors=r02_new_actors, r06_new_ledge_lanes=r06_new_ledge_lanes,
                scope='Source provenance, complete installed rows/aliases/borders, Safari '
                      'configuration and source-owned town anchors/connections; '
                      'no earned progression or visual acceptance claim.')


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--pack', type=Path, required=True)
    parser.add_argument('--index', type=Path, default=DEFAULT_INDEX)
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    result = audit(args.pack, args.index)
    args.output.mkdir(parents=True, exist_ok=False)
    (args.output / 'receipt.json').write_text(json.dumps(result, indent=2) + '\n')
    print(json.dumps({key: value for key, value in result.items() if key != 'checks'}))


if __name__ == '__main__':
    main()
