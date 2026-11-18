#!/usr/bin/env python3
"""Render and compare the installed terrace; never starts a game or writes packs."""
import argparse
from copy import deepcopy
import json
from pathlib import Path
import sys

ROOT = Path(__file__).resolve().parents[4]
sys.path.insert(0, str(ROOT / 'planning/minnesota/tools'))
from PIL import Image
from render_archive import save_render
from render_progress_atlas import native_room
from render_reference import load_pack, room_sheet

MAP = 'MnR03C00R00'
VIEWS = [('south-fork', (39, 43)), ('east-stair', (41, 42)),
         ('supply-shelf', (40, 39)), ('return-lip', (37, 40)),
         ('river-approach', (32, 39)), ('north-meadow', (15, 16))]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--parent', type=Path, required=True)
    parser.add_argument('--pack', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    parent, pack = load_pack(args.parent), load_pack(args.pack)
    source = json.loads(Path(__file__).with_name('layout.json').read_text())
    before, after = parent['data']['maps'][MAP], pack['data']['maps'][MAP]
    changes = [(i % 24, i // 24, a, b) for i, (a, b) in enumerate(zip(before['blocks'], after['blocks'])) if a != b]
    assert sorted(changes) == sorted((*v['at'], v['from'], v['to']) for v in source['placements'])
    assert parent['runtime_files'] == pack['runtime_files']
    assert parent['compiled_audio'] == pack['compiled_audio']
    assert parent['audio_manifest'] == pack['audio_manifest']
    assert parent['data']['tilesets'] == pack['data']['tilesets']
    assert parent['data']['trainers'] == pack['data']['trainers']
    roster = json.loads((ROOT / 'planning/minnesota/pokemon/roster.json').read_text())
    curation = json.loads((ROOT / 'planning/minnesota/pokemon/roster-curation.json').read_text())
    retired = {f['id'] for f in curation['retired_families']}
    removed = {m['id'] for m in curation['removed_evolutions']}
    active = {m['name'].upper() for f in roster['families'] if f['id'] not in retired
              for m in f['members'] if m['id'] not in removed}
    trainers = pack['data']['trainers']['trainers']
    for obj in after['objects']:
        if obj['script'] in trainers:
            assert all(member['species'] in active for member in trainers[obj['script']]['party'])
    assert parent['data']['maps'].keys() == pack['data']['maps'].keys()
    assert all(m == pack['data']['maps'][name] for name, m in parent['data']['maps'].items() if name != MAP)
    assert before['attributes'] == after['attributes']
    assert before['events'] == after['events']
    expected_objects = deepcopy(before['objects'])
    for obj in expected_objects:
        if obj['script'] == source['item']['script']:
            obj['x'], obj['y'] = source['item']['to']
    assert expected_objects == after['objects']
    sign = 'MN_R03_TERRACE_SIGNText'
    assert {k:v for k,v in before['scripts'].items() if k != sign} == {k:v for k,v in after['scripts'].items() if k != sign}
    commands = after['script_text_bodies'][sign]['commands']
    lines = [json.loads(v['args'][0]) for v in commands if v['args']]
    assert lines == [line for page in source['sign_pages'] for line in page]
    assert all(len(line) <= 18 for line in lines)
    assert pack['data']['map_scripts'][sign] == after['scripts'][sign]
    collision = pack['data']['tilesets'][after['attributes']['tileset_name']]['collision']
    for old, new in zip(before['blocks'], after['blocks']):
        for a, b in zip(collision[f'{old:02x}'], collision[f'{new:02x}']):
            assert (a == 'TALL_GRASS') == (b == 'TALL_GRASS')
            assert (a in {'WATER', 'CUT_TREE'}) == (b in {'WATER', 'CUT_TREE'})
    args.output.mkdir(parents=True, exist_ok=True)
    cameras = []
    for label, data in [('before', parent), ('after', pack)]:
        whole = Image.new('RGB', (768, 1024))
        for row in range(2):
            module = data['data']['maps'][f'MnR03C00R{row:02}']
            whole.paste(native_room(module, data, native_origin=True), (0, row * 768))
        save_render(whole, args.output / f'{label}-whole.png')
        for title, (x, y) in VIEWS:
            crop = whole.crop((x * 16 - 80, y * 16 - 72, x * 16 + 80, y * 16 + 72))
            save_render(crop, args.output / f'{label}-{title}-160.png')
            cameras.append((f'{label}: {title}', crop))
    save_render(room_sheet(cameras, columns=len(VIEWS)), args.output / 'before-after-decisions.png')
    report = dict(status='PASS', changed_metatiles=changes,
                  preserved_maps=len(pack['data']['maps'])-1, sign_lines=lines,
                  terrain_scope='native collision data; core-directed paths are checked by the Rust test',
                  captures='Pack art at native scale; headless production captures retained separately.')
    (args.output / 'preservation.json').write_text(json.dumps(report, indent=2)+'\n')
    print(json.dumps(report))


if __name__ == '__main__':
    main()
