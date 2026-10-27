#!/usr/bin/env python3
"""Review the entire current shore and Ryan's cabin with original source art."""
import argparse
import json
from pathlib import Path
from PIL import Image, ImageChops
from render_archive import save_render
from render_stages import render_stage
from render_reference import SourceArt, layout_blocks, load_pack
from review_north_shore import oriented_sprite
from review_league_approach import inspect_league

ROOT = Path(__file__).resolve().parents[3]
OUT = ROOT / 'planning/minnesota/renders/ryans-house'

def review(path, published=False):
    pack = load_pack(path)
    design = json.loads((ROOT/'planning/minnesota/routes/MN_R11.json').read_text())
    layout = json.loads((ROOT/'planning/minnesota/routes/MN_R11.layout.json').read_text())
    source = json.loads((ROOT/'planning/minnesota/playable/ryans-house.json').read_text())
    art = SourceArt(pack)
    blocks = layout_blocks(layout)
    report, kind, seen = inspect_league(design, layout, pack, blocks)
    assert tuple(source['door']['return_tile']) in seen
    assert kind(*source['door']['tile']) == 'DOOR'
    assert kind(*source['door']['return_tile']) == 'FLOOR'
    assert [blocks[y*56+9] for y in [10,11,12]] == [0x80]*3
    print('Static full-route walking, river edges, ledges, door and porch checks pass.')
    if published:
        for row in range(4):
            for column in range(2):
                m=pack['data']['maps'][f'MnR11C{column:02}R{row:02}']
                expected=[blocks[y*56+x] for y in range(row*24,(row+1)*24) for x in range(column*32,min(56,(column+1)*32))]
                assert m['blocks'] == expected, 'source/native shore terrain mismatch'
        assert pack['data']['maps']['MnRyansHouse']['blocks'] == [int(v,16) for row in source['room']['rows'] for v in row.split()]
    terrain=art.render(layout['tileset'],56,96,blocks,derived=layout.get('derived_blocks'))
    save_render(terrain,OUT/'whole-shore-terrain.png')
    scene=render_stage(terrain,'convert',OUT,'whole-shore','RGBA')
    for actor in design['trainers']+design['residents']+design['items']:
        image=oriented_sprite(pack,actor.get('sprite','SPRITE_POKE_BALL'),actor.get('facing','south'))
        scene.alpha_composite(image,(actor['tile'][0]*16,actor['tile'][1]*16))
    save_render(scene,OUT/'whole-shore-scene.png')
    detail=render_stage(scene,'crop',OUT,'baptism-cabin',(7*32,7*32,17*32,17*32))
    save_render(render_stage(detail,'resize',OUT,'baptism-cabin',(960,960),Image.Resampling.NEAREST),OUT/'baptism-cabin.png')
    room=source['room']
    inside=render_stage(art.render('traditional_house',5,4,[int(v,16) for row in room['rows'] for v in row.split()],group='indoor'),'convert',OUT,'house-room','RGBA')
    for actor in room['residents']:
        inside.alpha_composite(oriented_sprite(pack,actor['sprite'],actor['facing']),(actor['tile'][0]*16,actor['tile'][1]*16))
    save_render(inside,OUT/'house-room.png')
    save_render(render_stage(inside,'resize',OUT,'house-room',(640,512),Image.Resampling.NEAREST),OUT/'house-room-4x.png')
    # Native 160x144 framing proves the complete existing cascade is visible
    # on the normal walking approach, beside the cabin.
    x,y=source['porch_view_core']
    view=render_stage(scene,'crop',OUT,'porch-camera',(x*16-80,y*16-72,x*16+80,y*16+72))
    save_render(render_stage(view,'resize',OUT,'porch-camera',(640,576),Image.Resampling.NEAREST),OUT/'porch-camera-4x.png')
    player=oriented_sprite(pack,'SPRITE_CHRIS','south')
    for label,background,at in [('porch',scene,[23,24]),('interior',inside,[5,6])]:
        x,y=at
        if label=='porch':
            crop=render_stage(background,'crop',OUT,'native-reference-'+label,(x*16-256,y*16-192,x*16+256,y*16+192))
            scaled=render_stage(crop,'resize',OUT,'native-reference-'+label,(1536,1152),Image.Resampling.NEAREST)
            reference=Image.new('RGBA',(1280,800));reference.paste(scaled,(-128,-176))
            reference.alpha_composite(render_stage(player,'resize',OUT,'reference-player',(48,48),Image.Resampling.NEAREST),(640,400))
        else:
            reference=background.copy();reference.alpha_composite(player,(x*16,y*16))
            reference=render_stage(reference,'resize',OUT,'native-reference-'+label,(640,512),Image.Resampling.NEAREST)
        reference=render_stage(reference,'convert',OUT,'native-reference-'+label,'RGB')
        save_render(reference,OUT/(label+'-native-reference.png'))
        native=OUT/('native-'+label+'-2d.png')
        if native.exists():
            actual=render_stage(Image.open(native),'convert',OUT,label+'-native','RGB')
            if label=='interior':
                bounds=ImageChops.difference(actual,Image.new('RGB',actual.size,actual.getpixel((0,0)))).getbbox()
                assert bounds is not None and (bounds[2]-bounds[0],bounds[3]-bounds[1])==(640,512),'native full-room framing changed'
                actual=render_stage(actual,'crop',OUT,'native-interior-room',bounds)
            difference=ImageChops.difference(actual,reference)
            save_render(difference,OUT/(label+'-native-source-difference.png'))
            pixels=sum(pixel != (0,0,0) for pixel in difference.getdata())
            print(label,'source/native different pixels:',pixels,'of',difference.width*difference.height)
            assert pixels==0,'source/native visual mismatch'
    print('Whole shore and complete cabin rendered; all generations are retained.')

if __name__=='__main__':
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--pack',default=str(ROOT/'content-packs/minnesota-northstar-drainage-ice.crystalpack'))
    parser.add_argument('--verify-published',action='store_true')
    args=parser.parse_args();review(args.pack,args.verify_published)
