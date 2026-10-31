#!/usr/bin/env python3
"""Read-only whole Ely review; source previews reuse installed native art.

Supply --source and --activities together for the quick authoring loop, or omit
both to inspect an installed pack. Every run requires a fresh output directory.
"""
import argparse
from copy import deepcopy
import hashlib
import json
from pathlib import Path
import time

from PIL import Image

from render_archive import save_render
from render_reference import SourceArt, load_pack, npc_palette_pack, room_sheet
from review_passage_native import actors


def main():
    cli = argparse.ArgumentParser(description=__doc__)
    for name in ['pack', 'output']:
        cli.add_argument('--' + name, type=Path, required=True)
    for name in ['source', 'activities']:
        cli.add_argument('--' + name, type=Path)
    a = cli.parse_args()
    assert bool(a.source) == bool(a.activities), 'source and activities form one preview'
    a.output.mkdir(parents=True, exist_ok=False)
    pack = load_pack(a.pack)
    m = deepcopy(pack['data']['maps']['MnEly'])
    assert [m['attributes']['width'], m['attributes']['height']] == [20, 18]
    if a.source:
        source = json.loads(a.source.read_text())
        assert source['id'] == 'MnEly' and source['tileset'] == 'mn_ely_snow'
        m['blocks'] = [int(v, 16) for row in source['rows'] for v in row.split()]
        activities = json.loads(a.activities.read_text())
        assert len(m['objects']) == 1, 'preview must start from the original town'
        for actor in activities['residents'] + [
            {**item, 'sprite': 'SPRITE_POKE_BALL', 'facing': 'south'}
            for item in activities['items']
        ]:
            obj = deepcopy(m['objects'][0])
            obj.update(x=actor['tile'][0], y=actor['tile'][1], sprite=actor['sprite'],
                       script=actor['id'], spritemovedata='SPRITEMOVEDATA_STANDING_' +
                       {'south': 'DOWN', 'north': 'UP', 'east': 'RIGHT', 'west': 'LEFT'}[actor['facing']])
            m['objects'].append(obj)
    started = time.perf_counter()
    art = SourceArt(pack)
    scene = actors(npc_palette_pack(pack, 'day'), art.render('mn_ely_snow', 20, 18, m['blocks']), m)
    save_render(scene, a.output / 'whole-original.png')
    save_render(scene.resize((1280, 1152), Image.Resampling.NEAREST), a.output / 'whole-2x.png')
    views = []
    for label, at in [('north-arrival', [24, 2]), ('supply-nook', [34, 5]),
                      ('lodge-clearing', [27, 16]), ('pine-loop', [33, 10]),
                      ('clinic-garden', [13, 23]), ('mart-windbreak', [29, 29])]:
        x, y = at
        crop = scene.crop((x*16-80, y*16-72, x*16+80, y*16+72))
        save_render(crop, a.output / (label + '-original.png'))
        views.append((label, crop.resize((640, 576), Image.Resampling.NEAREST)))
    save_render(room_sheet(views, 3), a.output / 'decisions.png')
    (a.output / 'receipt.json').write_text(json.dumps(dict(
        pack=str(a.pack), pack_sha256=hashlib.sha256(a.pack.read_bytes()).hexdigest(),
        source=str(a.source) if a.source else 'installed pack',
        warm_render_seconds=time.perf_counter()-started,
        scope='Native static source review; falling snow and gameplay require production review.'
    ), indent=2) + '\n')


if __name__ == '__main__':
    main()
