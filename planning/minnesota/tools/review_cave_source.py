#!/usr/bin/env python3
"""Render a complete cave source with native art; optionally watch for edits.

Previews preserve every source and render version. Publication and production
controller/native camera acceptance are separate from this fast source loop.
"""
import argparse
import copy
import hashlib
import json
from pathlib import Path
import time

from PIL import Image

from render_reference import SourceArt, load_pack, npc_palette_pack
from review_passage_native import actors


def render(pack, art, source, output, palette):
    started = time.perf_counter()
    raw = source.read_bytes()
    digest = hashlib.sha256(raw).hexdigest()
    version = output / (digest[:12] + '-' + str(time.time_ns()))
    version.mkdir(parents=True, exist_ok=False)
    (version / 'source.json').write_bytes(raw)
    try:
        spec = json.loads(raw)
        room = copy.deepcopy(pack['data']['maps'][spec['id']])
        width, height = spec['dimensions_metatiles']
        rows = [[int(value, 16) for value in row.split()] for row in spec['rows']]
        if len(rows) != height or any(len(row) != width for row in rows):
            raise ValueError('source rows do not match declared dimensions')
        room['blocks'] = [value for row in rows for value in row]
        room['attributes'].update(width=width, height=height)
        for proposed in spec.get('proposed_actors', []) + spec.get('proposed_items', []):
            actor = next((a for a in room['objects'] if a['script'] == proposed['id']), None)
            existing_actor = actor is not None
            if actor is None:
                actor = copy.deepcopy(room['objects'][0])
                room['objects'].append(actor)
            facing = proposed.get('facing') or next((
                trainer.get('facing') for trainer in spec.get('trainer_source', {}).get('trainers', [])
                if trainer['id'] == proposed['id']), None)
            if facing and facing not in {'north', 'south', 'east', 'west'}:
                raise ValueError('proposed actor facing must be north, south, east or west')
            movement = ('SPRITEMOVEDATA_STANDING_' + {
                'north': 'UP', 'south': 'DOWN', 'east': 'RIGHT', 'west': 'LEFT'}[facing]
                if facing else actor['spritemovedata'] if existing_actor
                else 'SPRITEMOVEDATA_STANDING_DOWN')
            actor.update(
                sprite=proposed.get('sprite', 'SPRITE_POKE_BALL'),
                sprite_has_facings='sprite' in proposed,
                x=proposed['tile'][0], y=proposed['tile'][1],
                spritemovedata=movement,
                object_identifier=proposed['id'], script=proposed['id'])
        hidden = spec.get('default_hidden_actors', [])
        assert all(any(obj['script'] == identity for obj in room['objects']) for identity in hidden), 'declared default-hidden actor missing'
        room['objects'] = [obj for obj in room['objects'] if obj['script'] not in hidden]
        attributes = room['attributes']
        preview_tileset = spec.get('base_tileset', attributes['tileset_name'])
        image = actors(npc_palette_pack(pack, palette), art.render(
            preview_tileset, width, height, room['blocks'], group=palette,
            derived=spec.get('derived_blocks')), room)
        image.save(version / 'whole-original.png')
        image.resize((image.width * 2, image.height * 2), Image.Resampling.NEAREST).save(
            version / 'whole-2x.png')
        receipt = dict(status='SOURCE_PREVIEW_ONLY', source_sha256=digest,
                       whole=str(version / 'whole-2x.png'),
                       warm_render_seconds=round(time.perf_counter() - started, 3))
    except (KeyError, TypeError, ValueError) as error:
        receipt = dict(status='SOURCE_ERROR', source_sha256=digest, error=str(error))
    (version / 'review.json').write_text(json.dumps(receipt, indent=2) + '\n')
    (output / 'latest.json').write_text(json.dumps(receipt, indent=2) + '\n')
    print(json.dumps(receipt), flush=True)
    return digest


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--pack', type=Path, required=True)
    parser.add_argument('--source', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--palette', choices=['morn', 'day', 'nite'], default='nite')
    parser.add_argument('--watch', action='store_true')
    args = parser.parse_args()
    pack = load_pack(args.pack)
    art = SourceArt(pack)
    previous = None
    while True:
        digest = hashlib.sha256(args.source.read_bytes()).hexdigest()
        if digest != previous:
            previous = render(pack, art, args.source, args.output, args.palette)
        if not args.watch:
            return
        time.sleep(0.2)


if __name__ == '__main__':
    main()
