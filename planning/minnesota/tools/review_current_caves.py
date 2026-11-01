#!/usr/bin/env python3
"""Review selected cave terrain and mine floors with source owners and real links.

This is a read-only source review, not publication, live script-state simulation,
native camera acceptance or an earned-access claim. Rendered actors are authored
anchors; runtime callbacks may hide or move them. Darkness remains dark.
"""
import argparse
from copy import deepcopy
import hashlib
import json
from pathlib import Path

from render_archive import save_render
from render_progress_atlas import native_room
from render_reference import load_pack, room_sheet

ROOT = Path(__file__).resolve().parents[3]
MINE_TILESETS = {'mn_iron_haulage_rock', 'mn_iron_sump_rock',
                 'mn_iron_vent_rock', 'mn_iron_core_rock'}
KNOWN_MINES = {'MnIronHaulage', 'MnIronSump', 'MnIronVentilation',
               'MnIronPumpGallery', 'MnIronDrillCore'}


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def bound(binding):
    path = Path(binding['path'])
    assert not path.is_absolute() and '..' not in path.parts
    path = ROOT / path
    assert sha(path) == binding['sha256'], ('source changed', str(path))
    return json.loads(path.read_text())


def owner(index, name):
    for field in ['current_native_layouts', 'current_caves', 'current_story_floors',
                  'current_ice_caves', 'current_discovery_sites']:
        binding = index.get(field, {}).get(name)
        if binding is None:
            continue
        source = bound(binding)
        if 'rows_pointer' in binding:
            rows = source
            for part in binding['rows_pointer'][1:].split('/'):
                rows = rows[int(part)] if isinstance(rows, list) else rows[part]
            spec = binding
        elif 'rooms' in source:
            spec, = [r for r in source['rooms'] if r['id'] == name]
            rows = spec['rows']
        else:
            spec, rows = source, source['rows']
        return dict(binding), deepcopy(rows), spec['dimensions_metatiles'], field
    # The regional mine adds a story-gated shaft to the original terrain.
    # Keep that explicit overlay; never restore its old unguarded floor merely
    # to obtain a source match. The production controller covers this case too.
    assert name == 'MnR09Cave1F', ('missing cave terrain owner', name)
    binding = dict(path='planning/minnesota/caves/johto-quality-v01/MnR09Cave1F.json',
                   sha256='de255bc83a433a3c962a9e97d0b794146f3224bd33dfd809f65e4287431f9b64')
    source = bound(binding)
    rows = [line.split() for line in source['rows']]
    assert rows[5][9] == '02'
    rows[5][9] = '2d'
    return binding, [' '.join(row) for row in rows], source['dimensions_metatiles'], 'native_story_shaft_overlay'


def inspect(pack, index):
    maps = pack['data']['maps']
    expected = (set(index['current_caves']) | set(index['current_ice_caves'])
                | KNOWN_MINES | {'MnR09Cave1F'}
                | {name for name, binding in index['current_native_layouts'].items()
                   if binding['tileset'] == 'cave'})
    assert expected <= maps.keys(), ('missing cave map', sorted(expected - maps.keys()))
    constants = {m['attributes']['map_constant']: name for name, m in maps.items()}
    rooms = []
    for name, module in sorted(maps.items()):
        attrs = module['attributes']
        cave = attrs['environment'] == 'CAVE'
        mine = attrs['environment'] == 'DUNGEON' and attrs['tileset_name'] in MINE_TILESETS
        if name in expected:
            assert cave or mine, ('cave classification changed', name)
        if not name.startswith('Mn') or not (cave or mine):
            continue
        binding, rows, dimensions, role = owner(index, name)
        assert dimensions == [attrs['width'], attrs['height']], ('cave dimensions', name)
        assert len(rows) == attrs['height'] and all(len(row.split()) == attrs['width'] for row in rows)
        assert [int(b, 16) for row in rows for b in row.split()] == module['blocks'], ('cave rows', name)
        if role == 'native_story_shaft_overlay':
            assert module['scripts']['MnNDrillPublicTiles'] == [
                dict(command='checkevent', args=['EVENT_MN_DEPOT_FOREMAN_DEFEATED']),
                dict(command='iffalse', args=['MnNDrillPublicEnd']),
                dict(command='changeblock', args=['18', '10', '7']),
                dict(command='endcallback', args=[])]
            assert any(row['command'] == 'callback' and row['args'] == ['MAPCALLBACK_TILES', 'MnNDrillPublicTiles']
                       for row in module['map_script_section_commands'])
        links = []
        for warp in module['events']['warps']:
            target = constants.get(warp['target_map_constant'], warp['target_map'])
            assert target in maps, ('missing cave link', name, target)
            arrivals = [w for w in maps[target]['events']['warps'] if w['index'] == warp['target_warp_id']]
            assert len(arrivals) == 1, ('missing cave arrival', name, target, warp['target_warp_id'])
            arrival, = arrivals
            links.append(dict(index=warp['index'], from_tile=[warp['x'], warp['y']],
                              target=target, target_index=arrival['index'],
                              target_tile=[arrival['x'], arrival['y']]))
        rooms.append(dict(map=name, environment=attrs['environment'], tileset=attrs['tileset_name'],
                          dimensions_metatiles=dimensions, time_of_day=attrs['time_of_day'],
                          border_block=attrs['border_block'], terrain_owner=binding,
                          owner_role=role, actor_anchors=len(module['objects']),
                          hidden_item_or_read_posts=len(module['events']['bg_events']),
                          authored_links=links))
    assert expected <= {r['map'] for r in rooms}, 'include every source-owned cave and rock mine'
    return rooms


def main():
    cli = argparse.ArgumentParser(description=__doc__)
    cli.add_argument('--output', type=Path, required=True)
    args = cli.parse_args()
    assert not args.output.exists(), 'retain all earlier review epochs'
    world_path = ROOT / 'planning/minnesota/world.json'
    world = json.loads(world_path.read_text())
    revision = world[world['active_local_review_revision']]
    pack_path, index_path = ROOT / revision['pack'], ROOT / revision['authoring_source_index']
    assert sha(pack_path) == revision['pack_sha256']
    assert sha(index_path) == revision['authoring_source_index_sha256']
    inputs = {str(path): sha(path) for path in [world_path, pack_path, index_path]}
    index = json.loads(index_path.read_text())
    pack = load_pack(pack_path)
    rooms = inspect(pack, index)
    args.output.mkdir(parents=True)
    sheet = []
    for room in rooms:
        name = room['map']
        image = native_room(pack['data']['maps'][name], pack,
                            archive=args.output / name, native_origin=True)
        save_render(image, args.output / name / 'whole-original.png')
        sheet.append((name + ' — ' + room['environment'], image))
    save_render(room_sheet(sheet, columns=3), args.output / 'all-current-caves-and-mines-original.png')
    assert all(sha(Path(path)) == digest for path, digest in inputs.items()), 'review inputs changed'
    receipt = dict(status='SOURCE_TERRAIN_AND_LINKS_MATCH', selected_revision=world['active_local_review_revision'],
                   world_sha256=sha(world_path), pack=revision['pack'], pack_sha256=sha(pack_path),
                   source_index=revision['authoring_source_index'], source_index_sha256=sha(index_path),
                   cave_environment_count=sum(r['environment'] == 'CAVE' for r in rooms),
                   rock_dungeon_count=sum(r['environment'] == 'DUNGEON' for r in rooms),
                   rooms=rooms, inputs_unchanged=True, tool_sha256=sha(Path(__file__)), scope=__doc__)
    (args.output / 'receipt.json').write_text(json.dumps(receipt, indent=2) + '\n')
    (args.output / 'review_current_caves.py.snapshot').write_bytes(Path(__file__).read_bytes())
    print(json.dumps({k: v for k, v in receipt.items() if k not in ['rooms', 'scope']}))


if __name__ == '__main__':
    main()
