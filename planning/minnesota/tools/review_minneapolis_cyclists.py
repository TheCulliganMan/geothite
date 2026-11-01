#!/usr/bin/env python3
"""Review the published bicycle cast without modifying game content."""
import argparse
import json
from pathlib import Path

from render_reference import load_pack
from render_progress_atlas import native_room


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('base', type=Path)
    parser.add_argument('pack', type=Path)
    parser.add_argument('output', type=Path)
    args = parser.parse_args()
    args.output.mkdir(parents=True, exist_ok=False)
    base, pack = load_pack(args.base), load_pack(args.pack)
    source = json.loads((Path(__file__).resolve().parents[1] /
                         'trainers/minneapolis-cyclists.json').read_text())
    before, data = base['data'], pack['data']
    assert base['runtime_files'] == pack['runtime_files']
    assert base['audio_manifest'] == pack['audio_manifest']
    assert before['pokemon'] == data['pokemon']
    for key in before.keys() | data.keys():
        if key not in ['maps', 'map_scripts', 'npcs', 'trainers',
                       'story_event_script_constants']:
            assert before.get(key) == data.get(key), key
    for key, trainer in before['trainers']['trainers'].items():
        assert data['trainers']['trainers'][key] == trainer, key
    for key in before['trainers'].keys() | data['trainers'].keys():
        if key != 'trainers':
            assert before['trainers'].get(key) == data['trainers'].get(key), key
    for key, value in before['story_event_script_constants']['global'].items():
        assert data['story_event_script_constants']['global'][key] == value, key
    for name, module in before['maps'].items():
        if name != 'MnMinneapolis':
            assert module == data['maps'][name], name
    city = data['maps']['MnMinneapolis']
    old = before['maps']['MnMinneapolis']
    for key, program in old['scripts'].items():
        assert city['scripts'][key] == program, key
    for key, text in old['script_text_bodies'].items():
        assert city['script_text_bodies'][key] == text, key
    for key in ['blocks', 'attributes', 'events']:
        if key == 'events':
            assert old[key]['warps'] == city[key]['warps']
        else:
            assert old[key] == city[key]
    assert city['objects'][:len(old['objects'])] == old['objects']
    assert len(city['objects']) == len(old['objects']) + 18
    for row in source['trainers']:
        actor = next(o for o in city['objects'] if o['object_identifier'] == row['id'])
        assert actor['sprite'] == row['sprite']
        assert actor['spritemovedata'] == 'SPRITEMOVEDATA_PATROL_CLOCKWISE'
        assert actor['object_type'] == 'OBJECTTYPE_SCRIPT' and actor['radius'] == 0
        assert 2 * (actor['move_range_x'] + actor['move_range_y']) >= 32
        assert row['id'] not in city['trainer_scripts']
        assert any(b['request']['trainer_id'] == row['id'] for b in city['scripted_trainer_battles'])
        assert data['trainers']['trainers'][row['id']]['party']
    image = native_room(city, pack)
    image.save(args.output / 'whole-city.png')
    for name, center in [('mill', (16, 13)), ('bridge', (39, 37)),
                         ('market', (49, 46)), ('south', (27, 57))]:
        x, y = center
        image.crop((max(0, x * 16 - 160), max(0, y * 16 - 144),
                    min(image.width, x * 16 + 160),
                    min(image.height, y * 16 + 144))).resize((640, 576)).save(
                        args.output / f'{name}.png')
    receipt = dict(cyclists=18, loop_lengths=[2*sum(t["patrol_size"]) for t in source["trainers"]], existing_maps_preserved=True,
                   existing_actors_preserved=True, art_and_audio_preserved=True)
    (args.output / 'audit.json').write_text(json.dumps(receipt, indent=2) + '\n')
    print(json.dumps(receipt))


if __name__ == '__main__':
    main()
