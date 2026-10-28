#!/usr/bin/env python3
"""Render all installed city populations from the actual published pack.

Read-only pack access; native source art is composed in memory. Originals and
player-scale views are versioned by render_archive before any replacement.
"""
import argparse
from pathlib import Path
from PIL import Image, ImageDraw
from render_archive import save_render
from render_reference import SourceArt, load_pack, npc_palette_pack
from review_north_shore import oriented_sprite


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--pack', required=True)
    parser.add_argument('--output', required=True)
    args = parser.parse_args()
    pack = load_pack(args.pack)
    art = SourceArt(pack)
    output = Path(args.output)
    output.mkdir(parents=True, exist_ok=True)
    players = []
    for name, module in pack['data']['maps'].items():
        cast = [a for a in module['objects'] if a['script'].startswith(name + 'Life')]
        if not cast:
            continue
        attr = module['attributes']
        group = attr['time_of_day'] or 'day'
        scene = art.render(attr['tileset_name'], attr['width'], attr['height'], module['blocks'], group).convert('RGBA')
        sprites = npc_palette_pack(pack, group)
        for actor in module['objects']:
            facing = {'UP':'north','RIGHT':'east','LEFT':'west'}.get(actor['spritemovedata'].rsplit('_',1)[-1], 'south')
            scene.alpha_composite(oriented_sprite(sprites, actor['sprite'], facing), (actor['x']*16,actor['y']*16))
        save_render(scene, output / f'{name}-whole.png')
        for label, actor in [('trader',cast[0]),('walker',next(a for a in cast if 'WALK_' in a['spritemovedata']))]:
            x, y = actor['x'], actor['y']+1
            # These explicit review fixtures start facing down at noon, like
            # render_at_location. They show the same map state before any input.
            camera = Image.new('RGBA',(320,288),(0,0,0,255))
            left, top = x*16-144, y*16-128
            camera.alpha_composite(scene,(-left,-top))
            camera.alpha_composite(oriented_sprite(sprites,'SPRITE_CHRIS','south'),(144,128))
            save_render(camera, output / f'{name}-{label}-source.png')
            if label == 'trader':
                players.append((name,camera))
    sheet = Image.new('RGB',(320*4,310*((len(players)+3)//4)),(19,29,40))
    draw = ImageDraw.Draw(sheet)
    for index,(name,scene) in enumerate(players):
        x,y=index%4*320,index//4*310
        draw.text((x+6,y+4),name,fill='white')
        sheet.paste(scene,(x,y+20))
    save_render(sheet,output/'all-traders.png')
    print(f'Rendered complete maps and player views for {len(players)} installed town populations.')


if __name__ == '__main__':
    main()
