#!/usr/bin/env python3
"""Read-only preservation, native collision and full-map harbor review."""
import argparse
import copy
import json
from pathlib import Path

from PIL import Image

from render_reference import SourceArt, layout_blocks, load_pack
from review_passage_native import actors


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--parent', type=Path, required=True)
    parser.add_argument('--pack', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    parent, pack = load_pack(args.parent), load_pack(args.pack)
    old, city = parent['data']['maps']['MnDuluth'], pack['data']['maps']['MnDuluth']
    root = Path(__file__).resolve().parents[3]
    layout = json.loads((root / 'planning/minnesota/revisions/duluth-harbor-platforms-v01/after.layout.json').read_text())
    assert city['blocks'] == layout_blocks(layout)
    for key in ['runtime_files', 'audio_manifest', 'compiled_audio', 'audio_compression']:
        assert pack.get(key) == parent.get(key), key
    allowed = {'blocks', 'objects', 'scripts', 'script_text_commands',
               'script_text_bodies', 'map_event_section_commands'}
    assert {k: v for k, v in city.items() if k not in allowed} == {
        k: v for k, v in old.items() if k not in allowed}
    assert city['objects'][:-2] == old['objects']
    for key in ['scripts', 'script_text_bodies']:
        assert all(city[key][label] == value for label, value in old[key].items())
    command_key = lambda c: (c['source_script'], c['command_index'])
    assert sorted([c for c in city['script_text_commands'] if not c['source_script'].startswith('MnDuluthFloatingFisher')], key=command_key) == sorted(old['script_text_commands'], key=command_key)
    data = copy.deepcopy(pack['data'])
    data['maps']['MnDuluth'] = old
    data['map_blocks'] = parent['data']['map_blocks']
    for key in ['map_scripts', 'npcs']:
        for label, value in parent['data'][key].items():
            if label != (old['attributes']['map_events_label'] if key == 'map_scripts' else 'MnDuluth'):
                assert data[key].get(label) == value, (key, label)
        data[key] = parent['data'][key]
    assert data == parent['data'], 'unrelated data root changed'
    assert {k: v for k, v in pack['data']['map_blocks'].items()
            if k != old['attributes']['blocks_label']} == {
        k: v for k, v in parent['data']['map_blocks'].items()
        if k != old['attributes']['blocks_label']}
    width, height = 64, 46
    border = {y * width + x for y in range(height) for x in range(width)
              if x in [0, width - 1] or y in [0, height - 1]}
    assert all(city['blocks'][i] == old['blocks'][i] for i in border)
    assert city['blocks'][33 * width + 47] == 0x35
    collision = pack['data']['tilesets']['mn_duluth_expanded']['collision']
    def kind(x, y):
        return collision[f"{city['blocks'][y // 2 * width + x // 2]:02x}"][y % 2 * 2 + x % 2]
    for actor, (x, y) in zip(city['objects'][-2:], [(55, 75), (71, 81)]):
        assert (actor['x'], actor['y']) == (x, y) and actor['sprite'] == 'SPRITE_FISHER'
        assert kind(x, y) == kind(x - 1, y) == kind(x - 3, y) == 'FLOOR'
        assert kind(x - 4, y) == 'WATER'
    args.output.mkdir(parents=True, exist_ok=False)
    terrain = SourceArt(pack).render('mn_duluth_expanded', width, height, city['blocks'])
    whole = actors(pack, terrain, city)
    whole.save(args.output / 'whole-town.png')
    for name, x, y in [('west-platform', 55, 75), ('east-platform', 71, 81), ('cleared-buoy', 95, 67)]:
        whole.crop((x * 16 - 80, y * 16 - 72, x * 16 + 80, y * 16 + 72)).resize((640, 576), Image.Resampling.NEAREST).save(args.output / f'{name}.png')
    report = dict(status='PASS', unchanged_complete_maps=len(data['maps']) - 1,
                  changed_metatiles=sum(a != b for a, b in zip(old['blocks'], city['blocks'])),
                  added_fishermen=2, perimeter_unchanged=True, art_and_audio_exact=True,
                  scope='Pack preservation and source-art review; production input is verified separately.')
    (args.output / 'preservation.json').write_text(json.dumps(report, indent=2) + '\n')
    print(json.dumps(report))


if __name__ == '__main__':
    main()
