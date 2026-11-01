#!/usr/bin/env python3
"""Read-only exact-art review of Duluth's authored native walking seams."""
import argparse
import json
from pathlib import Path

from PIL import Image, ImageChops

from render_archive import save_render
from render_reference import SourceArt, load_pack
from render_stages import render_stage
import review_duluth

ROOT = Path(__file__).resolve().parents[3]
OUT = ROOT / 'planning/minnesota/renders/duluth-connections/expanded'
ORIGINS = {
    'MnR10C00R00': (0, 3072),
    'MnR10C01R00': (1024, 3072),
    'MnDuluth': (1536, 3072),
    **{f'MnR11C{column:02}R{row:02}': (1312 + column * 1024, row * 768)
       for row in range(4) for column in range(2)},
}


def review(path, published=False):
    pack = load_pack(path)
    source = json.loads((ROOT / 'planning/minnesota/towns/MnDuluth.connections.json').read_text())['regional_connections']
    art = SourceArt(pack)
    review_duluth.OUTPUT = OUT
    world = Image.new('RGBA', (3104, 3840))
    terrain_world = Image.new('RGBA', (3104, 3840))
    scenes = {}
    for name, origin in ORIGINS.items():
        m = pack['data']['maps'][name]
        a = m['attributes']
        before = list(m['blocks'])
        after = list(m['blocks'])
        for patch in source['patches']:
            if patch['map'] != name:
                continue
            x, y = patch['metatile']
            index = y * a['width'] + x
            old, new = int(patch['expected'], 16), int(patch['block'], 16)
            assert after[index] == (new if published else old), (name, 'stale native mouth', patch)
            before[index], after[index] = old, new
        for label, blocks in [('before', before), ('connected', after)]:
            terrain = art.render(a['tileset_name'], a['width'], a['height'], blocks, 'day')
            save_render(terrain, OUT / (name + '-' + label + '-terrain.png'))
            if label == 'before':
                continue
            scene = render_stage(terrain, 'convert', OUT, name + '-connected-scene', 'RGBA')
            actors = [o for o in m['objects'] if not o['object_identifier'].startswith(('MN_DULUTH_RANGE_GUARD_', 'MN_DULUTH_SHORE_GUARD_'))]
            if name == 'MnDuluth':
                if published:
                    relocation = source['sign_relocation']
                    signs = [b for b in m['events']['bg_events'] if b['script'] == relocation['script']]
                    assert len(signs) == 1 and [signs[0]['x'], signs[0]['y']] == relocation['destination'], 'native west sign relocation'
                for p in source['passages']:
                    for identifier, at in zip(p['identifiers'], p['bodies']):
                        facing = 'RIGHT' if p['outward'] == 'west' else 'DOWN'
                        actors.append(dict(object_identifier=identifier, x=at[0], y=at[1], sprite='SPRITE_OFFICER', spritemovedata='SPRITEMOVEDATA_STANDING_' + facing))
                if published:
                    assert {o['object_identifier']: [o['x'], o['y']] for o in actors} == {o['object_identifier']: [o['x'], o['y']] for o in m['objects']}
            for actor in actors:
                movement = actor['spritemovedata']
                facing = 'north' if movement.endswith('_UP') else 'west' if movement.endswith('_LEFT') else 'east' if movement.endswith('_RIGHT') else 'south'
                sprite = review_duluth.oriented_sprite(pack, actor['sprite'], facing)
                scene.alpha_composite(sprite, (actor['x'] * 16, actor['y'] * 16))
            save_render(scene, OUT / (name + '-connected-scene.png'))
            world.alpha_composite(scene, origin)
            terrain_rgba = render_stage(terrain, 'convert', OUT, name + '-connected-terrain-rgba', 'RGBA')
            terrain_world.alpha_composite(terrain_rgba, origin)
            scenes[name] = scene
    save_render(world, OUT / 'whole-connected-world.png')
    save_render(terrain_world, OUT / 'whole-connected-terrain.png')
    player = review_duluth.oriented_sprite(pack, 'SPRITE_CHRIS', 'south')
    player = render_stage(player, 'resize', OUT, 'reference-player', (48, 48), Image.Resampling.NEAREST)
    for label, name, at in [('range-checkpoint', 'MnDuluth', (4, 24)), ('shore-checkpoint', 'MnDuluth', (32, 4)), ('range-arrival', 'MnR10C01R00', (28, 24))]:
        origin = ORIGINS[name]
        x, y = origin[0] + at[0] * 16, origin[1] + at[1] * 16
        # The full composition retains every actor. Classic 2D presents the
        # current map's roster; neighboring terrain continues across the seam.
        # Keep the earlier all-actor comparison too, rather than masking pixels.
        old_reference = OUT / (label + '-reference.png')
        if old_reference.exists() and not (OUT / (label + '-all-actors-reference.png')).exists():
            save_render(Image.open(old_reference), OUT / (label + '-all-actors-reference.png'))
        scoped = terrain_world.copy()
        scoped.alpha_composite(scenes[name], origin)
        save_render(scoped, OUT / (label + '-classic-source-world.png'))
        crop = render_stage(scoped, 'crop', OUT, label, (x - 256, y - 192, x + 256, y + 192))
        scaled = render_stage(crop, 'resize', OUT, label, (1536, 1152), Image.Resampling.NEAREST)
        view = Image.new('RGBA', (1280, 800))
        view.paste(scaled, (-128, -176))
        view.alpha_composite(player, (640, 400))
        save_render(view, OUT / (label + '-reference-rgba.png'))
        reference = render_stage(view, 'convert', OUT, label + '-reference', 'RGB')
        save_render(reference, OUT / (label + '-reference.png'))
        native = OUT / ('native-' + label + '-2d.png')
        if native.exists():
            actual = render_stage(Image.open(native), 'convert', OUT, label + '-native', 'RGB')
            diff = ImageChops.difference(actual, reference)
            save_render(diff, OUT / (label + '-native-source-difference.png'))
            print(label, 'different source/native pixels:', sum(p != (0, 0, 0) for p in diff.getdata()), 'of', 1280 * 800)
    print('Rendered complete Duluth, R10 and R11 at one source scale; blank areas are not invented terrain.')


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--pack', default=str(ROOT / 'content-packs/minnesota-northstar-expanded-base.crystalpack'))
    parser.add_argument('--verify-published', action='store_true')
    args = parser.parse_args()
    review(args.pack, args.verify_published)
