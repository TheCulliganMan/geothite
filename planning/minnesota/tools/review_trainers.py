#!/usr/bin/env python3
"""Read-only audit and native-art review of the Minnesota trainer expansion."""
import argparse
import collections
import copy
import hashlib
import json
import re
from pathlib import Path

from audit_outdoor_connections import collision
from render_archive import save_render
from render_progress_atlas import native_room
from render_reference import load_pack


def components(tiles):
    remaining = set(tiles)
    while remaining:
        first = remaining.pop()
        reached = {first}
        queue = collections.deque([first])
        while queue:
            x, y = queue.popleft()
            for neighbor in [(x - 1, y), (x + 1, y), (x, y - 1), (x, y + 1)]:
                if neighbor in remaining:
                    remaining.remove(neighbor)
                    reached.add(neighbor)
                    queue.append(neighbor)
        yield reached


def check_species(value, allowed):
    if isinstance(value, dict):
        for key, child in value.items():
            if key in {'species', 'day_species', 'night_species'} and child:
                assert child in allowed, f'Out-of-region or retired species: {child}'
            else:
                check_species(child, allowed)
    elif isinstance(value, list):
        for child in value:
            check_species(child, allowed)


def review(base_path, pack_path, source_path, output, render=True):
    output.mkdir(parents=True, exist_ok=False)
    base = load_pack(base_path)
    pack = load_pack(pack_path)
    source = json.loads(source_path.read_text())
    root = Path(__file__).resolve().parents[1]
    roster = json.loads((root / 'pokemon/roster.json').read_text())
    retired = {family['id'] for family in json.loads(
        (root / 'pokemon/roster-curation.json').read_text())['retired_families']}
    allowed = {member['name'].upper() for family in roster['families']
               if family['id'] not in retired for member in family['members']}
    before, data = base['data'], pack['data']
    assert data['minnesota_trainer_scaling'] is True
    rows = source['trainers']
    assert len({row['id'] for row in rows}) == len(rows)
    assert {int(re.match(r'MnR(\d\d)C', row['map'])[1]) for row in rows
            if re.match(r'MnR\d\dC', row['map'])} == set(range(1, 21))
    for key, trainer in data['trainers']['trainers'].items():
        if key.startswith('MN_'):
            check_species(trainer['party'], allowed)
    for catalog in ['wild_encounters', 'field_encounters']:
        for name, table in data[catalog].items():
            if name.startswith('Mn'):
                check_species(table, allowed)
    for name, table in data['fishing']['groups'].items():
        if name.startswith('FISHGROUP_MN_'):
            check_species(table, allowed)
    for name, table in data['fishing']['time_groups'].items():
        if name.startswith('MN_TIME_'):
            check_species(table, allowed)
    for name, module in data['maps'].items():
        if name.startswith('Mn'):
            for actor in module['objects']:
                species = actor['sprite'].removeprefix('SPRITE_')
                if species in data['pokemon']:
                    assert species in allowed, (name, species)
    residual = copy.deepcopy(data)
    residual.pop('minnesota_trainer_scaling')
    maps = collections.defaultdict(list)
    for row in rows:
        maps[row['map']].append(row)
        trainer = residual['trainers']['trainers'].pop(row['id'])
        assert trainer['trainer_class'] == row['trainer_class']
        assert [(mon['species'], mon['level']) for mon in trainer['party']] == [
            (mon['species'], mon['level']) for mon in row['party']]
        check_species(trainer['party'], allowed)
        residual['story_event_script_constants']['global'].pop(row['id'])
        residual['story_event_script_constants']['global'].pop(row['defeated_flag'])
    map_results = {}
    for name, placements in maps.items():
        original, changed = before['maps'][name], data['maps'][name]
        assert original['blocks'] == changed['blocks'], name
        assert original['attributes'] == changed['attributes'], name
        assert original['events'] == changed['events'], name
        assert changed['objects'][:len(original['objects'])] == original['objects'], name
        kinds = collision(pack, changed)
        old_bodies = {(o['x'], o['y']) for o in original['objects']}
        bodies = {(o['x'], o['y']) for o in changed['objects']}
        new_bodies = bodies - old_bodies
        safe = {tile for tile, kind in kinds.items() if kind in {'FLOOR', 'TALL_GRASS'}}
        all_modes = {tile for tile, kind in kinds.items()
                     if kind in {'FLOOR', 'TALL_GRASS', 'WATER', 'ICE', 'WATERFALL', 'WHIRLPOOL'}}
        all_mode_ids = {tile: index for index, group in enumerate(
            components(all_modes - bodies)) for tile in group}
        for group in components(all_modes - old_bodies):
            assert len({all_mode_ids[tile] for tile in group - new_bodies}) <= 1, (
                name, 'new trainers split a walking, Surf or ice component')
        after_components = list(components(safe - bodies))
        after_ids = {tile: index for index, group in enumerate(after_components) for tile in group}
        for group in components(safe - old_bodies):
            assert len({after_ids[tile] for tile in group - new_bodies}) <= 1, (
                name, 'new trainers split an existing walking path')
        for row in placements:
            actors = [o for o in changed['objects'] if o['object_identifier'] == row['id']]
            assert len(actors) == 1
            actor = actors[0]
            assert [actor['x'], actor['y']] == row['tile']
            assert actor['sprite'] == row['sprite']
            assert actor['object_type'] == 'OBJECTTYPE_TRAINER'
            assert actor['radius'] == row['sight_range']
            assert tuple(row['approach']) in safe - bodies
            request = changed['trainer_scripts'][row['id']]
            assert request['event_flag'] == row['defeated_flag']
            dx, dy = {'west': (-1, 0), 'east': (1, 0),
                      'north': (0, -1), 'south': (0, 1)}[row['facing']]
            actor_tile = tuple(row['tile'])
            for distance in range(1, row['sight_range']):
                landing = (actor_tile[0] + dx * distance, actor_tile[1] + dy * distance)
                seen = (landing[0] + dx, landing[1] + dy)
                if kinds.get(seen) not in {'FLOOR', 'TALL_GRASS', 'WATER', 'ICE'}:
                    continue
                available = all_modes - (bodies - {actor_tile})
                landing_ids = {tile: index for index, group in enumerate(
                    components(available - {landing})) for tile in group}
                for group in components(available):
                    assert len({landing_ids[tile] for tile in group - {landing}}) <= 1, (
                        row['id'], 'trainer approach blocks walking, Surf or ice access')
        residual['maps'][name] = copy.deepcopy(original)
        residual['npcs'][name] = copy.deepcopy(before['npcs'][name])
        for field in ['map_scripts_label', 'map_events_label']:
            label = original['attributes'][field]
            residual['map_scripts'][label] = copy.deepcopy(before['map_scripts'][label])
        if render:
            image = native_room(changed, pack)
            save_render(image, output / f'{name}-complete.png')
            for row in placements:
                x, y = row['tile']
                save_render(image.crop(((x - 5) * 16, (y - 4) * 16,
                                        (x + 6) * 16, (y + 5) * 16)),
                            output / f"{row['id']}-camera.png")
        map_results[name] = {'added': len(placements), 'walking_components': len(after_components)}
    for label in list(residual['map_scripts']):
        if label.startswith('MN_EXTRA_'):
            del residual['map_scripts'][label]
    assert residual == before, 'Unrelated game data changed'
    for field in ['runtime_files', 'compiled_audio', 'audio_manifest', 'audio_compression', 'format_version']:
        assert pack[field] == base[field], f'Unrelated {field} changed'
    expected_report = copy.deepcopy(base['report'])
    manifests = [name for name in pack['report']['manifests']
                 if name.startswith('minnesota-trainer-expansion-v')]
    assert len(manifests) == 1 and manifests[0] in {'minnesota-trainer-expansion-v1', 'minnesota-trainer-expansion-v2'}
    expected_report['manifests'].append(manifests[0])
    assert pack['report'] == expected_report
    report = {'pack': str(pack_path), 'content_hash': pack['identity']['content_hash'],
              'source_sha256': hashlib.sha256(source_path.read_bytes()).hexdigest(),
              'added_trainers': len(rows), 'changed_maps': len(maps),
              'minnesota_trainers': sum(name.startswith('MN_') for name in data['trainers']['trainers']),
              'active_species': len(allowed), 'maps': map_results}
    (output / 'audit.json').write_text(json.dumps(report, indent=2) + '\n')
    print(f"Verified {len(rows)} added trainers across {len(maps)} maps; native roster, all walking components and unrelated content preserved.")


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('base', type=Path)
    parser.add_argument('pack', type=Path)
    parser.add_argument('source', type=Path)
    parser.add_argument('output', type=Path)
    parser.add_argument('--no-render', action='store_true', help='audit data without repeating retained images')
    args = parser.parse_args()
    review(args.base, args.pack, args.source, args.output, render=not args.no_render)
