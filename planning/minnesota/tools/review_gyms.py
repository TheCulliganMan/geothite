#!/usr/bin/env python3
"""Render complete authored gym rooms from external native art, without export."""
import argparse
import json
from pathlib import Path

from PIL import Image, ImageDraw
from render_archive import save_render
from gym_wind_art import marked_wind
from render_reference import SourceArt, load_pack
from review_north_shore import oriented_sprite

ROOT = Path(__file__).resolve().parents[3]
PLAN = ROOT / 'planning/minnesota'


def load_design(path):
    """Resolve current revisions while retaining pinned historical sources."""
    design = json.loads(path.read_text())
    if 'base' not in design:
        return design
    merged = load_design(path.parent / design['base'])
    for override in design['overrides']:
        revision = load_design(path.parent / override['source'])['gyms']
        assert len(revision) == 1 and revision[0]['id'] == override['map']
        indexes = [i for i, gym in enumerate(merged['gyms'])
                   if gym['id'] == override['map']]
        assert len(indexes) == 1, 'one existing gym must receive each revision'
        merged['gyms'][indexes[0]] = revision[0]
    return merged


def render(pack, design, output, verify_published=False, events=()):
    assert not (events and verify_published), 'Verify the initial published map; event views are labeled source fixtures.'
    events = set(events)
    art = SourceArt(pack)
    columns = min(2, len(design['gyms']))
    cell_width = max(g['dimensions_metatiles'][0] * 32 for g in design['gyms']) + 16
    cell_height = max(g['dimensions_metatiles'][1] * 32 for g in design['gyms']) + 34
    overview = Image.new('RGB', (columns * cell_width, ((len(design['gyms']) + columns - 1) // columns) * cell_height), '#253138')
    draw = ImageDraw.Draw(overview)
    for index, gym in enumerate(design['gyms']):
        width,height=gym['dimensions_metatiles']
        terrain = Image.new('RGB', (width*32, height*32))
        rows = [list(row) for row in gym['rows']]
        for gate in gym['gates']:
            if events and set(gate['requires']) <= events:
                x,y = gate['at']
                rows[y][x] = gate['open_symbol']
        for gate in gym.get('cut_gates', []):
            if gate['event'] in events:
                x,y=gate['tree_core'];rows[y//2][x//2]=gate['replacement_symbol']
        for y, row in enumerate(rows):
            for x, symbol in enumerate(row):
                source, block = gym['art'][symbol]
                native = art.blocks(source, 'indoor')[int(block, 16)]
                if gym.get('wind_arrows') and symbol in gym.get('winds', {}):
                    native = marked_wind(pack, native, gym['winds'][symbol])
                terrain.paste(native, (x * 32, y * 32))
        save_render(terrain, output / f'{gym["id"]}-terrain.png')
        if verify_published:
            room = pack['data']['maps'][gym['id']]
            assert [room['attributes']['width'], room['attributes']['height']] == [width,height]
            native = art.render(room['attributes']['tileset_name'],width,height,room['blocks'],group='indoor')
            save_render(native,output/f'{gym["id"]}-published-terrain.png')
            assert native.tobytes() == terrain.tobytes(), gym['id'] + ': source/native pixels differ'
            collision = pack['data']['tilesets'][room['attributes']['tileset_name']]['collision']
            for y,row in enumerate(gym['rows']):
                for x,symbol in enumerate(row):
                    source,block = gym['art'][symbol]
                    actual = collision[f'{room["blocks"][y*width+x]:02x}']
                    expected = (['WALL'] * 4 if symbol in gym.get('barriers', [])
                                else [gym['winds'][symbol]] * 4 if symbol in gym.get('winds', {})
                                else pack['data']['tilesets'][source]['collision'][block])
                    assert actual == expected, (gym['id'],x,y)
        scene = terrain.convert('RGBA')
        actors = [{'at':gym['leader_at'], 'sprite':gym['leader_sprite']},
                  {'at':gym['guide_at'], 'sprite':'SPRITE_GYM_GUIDE'}]
        actors += gym['trainers']
        for actor in actors:
            sprite = oriented_sprite(pack, actor['sprite'], 'south')
            x, y = actor['at']
            scene.alpha_composite(sprite, (x * 16, y * 16))
        save_render(scene, output / f'{gym["id"]}-whole.png')
        ox, oy = index % columns * cell_width + 8, index // columns * cell_height
        draw.text((ox,oy+4),f'{gym["slot"]}. {gym["city"]} / {gym["leader"]}',fill='white')
        overview.paste(scene.convert('RGB'),(ox,oy+26))
        for name, at in [('arrival',gym['arrival_core']),('leader',gym['leader_at'])]:
            x,y=at; cx,cy=x*16+8,y*16+8
            left=max(0,min(cx-80,width*32-160));top=max(0,min(cy-72,height*32-144))
            crop=scene.crop((left,top,left+160,top+144))
            save_render(crop,output/f'{gym["id"]}-{name}-160.png')
    save_render(overview,output/'all-gym-interiors.png')
    print(f'Rendered {len(design["gyms"])} complete gym room(s) to {output}')


def main():
    parser=argparse.ArgumentParser()
    parser.add_argument('--pack',type=Path,default=ROOT/'content-packs/text-tui.crystalpack')
    parser.add_argument('--output',type=Path,default=PLAN/'renders/gyms')
    parser.add_argument('--verify-published',action='store_true')
    parser.add_argument('--design', type=Path, default=PLAN/'gyms/current.json')
    parser.add_argument('--gym', help='Review just one authored gym by map id')
    parser.add_argument('--event', action='append', default=[], help='Seed a named source-visual fixture flag; repeat for multiple flags')
    args=parser.parse_args()
    design = load_design(args.design)
    if args.gym:
        design['gyms'] = [g for g in design['gyms'] if g['id'] == args.gym]
        assert design['gyms'], 'unknown gym map id'
    render(load_pack(args.pack),design,args.output,args.verify_published,args.event)


if __name__ == '__main__':
    main()
