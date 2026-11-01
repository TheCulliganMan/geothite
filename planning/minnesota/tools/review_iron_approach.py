#!/usr/bin/env python3
"""Preview the complete native iron crosscut; no publication or gameplay claim."""
import argparse
from copy import deepcopy
import hashlib
import json
from pathlib import Path
import time

from PIL import Image
from render_reference import SourceArt, load_pack, npc_palette_pack
from review_passage_native import actors


def review(pack, art, directory, output):
    started = time.perf_counter()
    inputs = {name: (directory/name).read_bytes()
              for name in ['before.json', 'after.json', 'contract.json']}
    before, after, contract = [json.loads(inputs[name]) for name in inputs]
    original = pack['data']['maps'][contract['map']]
    canonical = json.dumps(original, sort_keys=True, separators=(',', ':'), ensure_ascii=False)
    assert hashlib.sha256(canonical.encode()).hexdigest() == contract['native_module_sha256']
    flatten = lambda room: [int(v, 16) for row in room['rows'] for v in row.split()]
    assert flatten(before) == original['blocks']
    assert after['dimensions_metatiles'] == [16, 13]
    blocks = flatten(after)
    assert len(blocks) == 16*13
    assert blocks == [0x2e if b == 1 else b for b in original['blocks']]
    collision = pack['data']['tilesets']['cave']['collision']
    assert all(collision[f'{old:02x}'] == collision[f'{new:02x}']
               for old, new in zip(original['blocks'], blocks))
    room = deepcopy(original)
    room['blocks'] = blocks
    room['attributes']['border_block'] = after['border_block']
    reward = contract['reward']
    at = reward['tile']
    assert at == [5, 5] and reward['item'] == 'ETHER' and reward['quantity'] == 1
    assert collision[f'{blocks[at[1]//2*16+at[0]//2]:02x}'][at[1]%2*2+at[0]%2] == 'FLOOR'
    item = deepcopy(original['objects'][0])
    item.update(sprite='SPRITE_POKE_BALL', sprite_has_facings=False, x=at[0], y=at[1],
                spritemovedata='SPRITEMOVEDATA_STILL', radius=0,
                object_type='OBJECTTYPE_ITEMBALL', script=reward['id'],
                object_identifier=reward['id'], event_flag=reward['flag'])
    room['objects'].append(item)
    version = hashlib.sha256(b''.join(inputs.values())).hexdigest()[:12]
    destination = output/f'{version}-{time.time_ns()}'
    destination.mkdir(parents=True, exist_ok=False)
    terrain = art.render('cave', 16, 13, blocks, group='nite')
    image = actors(npc_palette_pack(pack, 'nite'), terrain, room)
    image.save(destination/'whole-room-original.png')
    image.resize((1024, 832), Image.Resampling.NEAREST).save(destination/'whole-room-2x.png')
    for name, raw in inputs.items():
        (destination/name).write_bytes(raw)
    result = dict(status='SOURCE_PREVIEW_NOT_NATIVE_PLAY', output=str(destination),
                  collision_quadrants_preserved=832, changed_wall_metatiles=80,
                  new_optional_cache=reward, palette_group='nite',
                  warm_review_seconds=round(time.perf_counter()-started, 3))
    (destination/'source-review.json').write_text(json.dumps(result, indent=2)+'\n')
    print(json.dumps(result), flush=True)
    return version


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ['pack', 'directory', 'output']:
        parser.add_argument('--'+name, type=Path, required=True)
    parser.add_argument('--watch', action='store_true')
    args = parser.parse_args()
    pack = load_pack(args.pack)
    art = SourceArt(pack)
    previous = None
    while True:
        current = hashlib.sha256(b''.join((args.directory/name).read_bytes()
                    for name in ['before.json', 'after.json', 'contract.json'])).hexdigest()[:12]
        if current != previous:
            previous = review(pack, art, args.directory, args.output)
        if not args.watch:
            break
        time.sleep(0.25)


if __name__ == '__main__':
    main()
