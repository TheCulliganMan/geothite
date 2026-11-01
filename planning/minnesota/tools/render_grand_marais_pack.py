#!/usr/bin/env python3
"""Render the entire published town and verify it matches its editable design."""
from render_archive import save_render
import hashlib
import json
from pathlib import Path
import sys

import cbor2
from PIL import Image
from render_reference import SourceArt, room_sheet
from review_grand_marais import sprite

ROOT=Path(__file__).resolve().parents[3]

def dialogue_rows(lines):
    rows=[]
    for line in lines:
        row=''
        for word in line.split():
            if row and len(row)+1+len(word)>18:
                rows.append(row);row=''
            row+=(' ' if row else '')+word
        if row:rows.append(row)
    return rows

def main():
    pack_path=Path(sys.argv[1]) if len(sys.argv)>1 else ROOT/'content-packs/minnesota-grand-marais.crystalpack'
    raw=pack_path.read_bytes()
    assert raw[:12]==b'CRYSTALPACK\0' and int.from_bytes(raw[14:18],'big')==len(raw)-22,'invalid pack frame'
    pack=cbor2.loads(raw[22:])
    module=pack['data']['maps']['MnGrandMarais'];attributes=module['attributes']
    design=json.loads((ROOT/'planning/minnesota/playable/grand-marais.json').read_text())
    assert module['blocks']==[int(v,16) for row in design['rows'] for v in row.split()],'rebuild the pack: terrain differs'
    assert (attributes['width'],attributes['height'])==(design['width'],design['height'])
    assert [(w['index'],w['x'],w['y'],w['target_map_constant']) for w in module['events']['warps']]==[(v['id'],*v['tile'],v['target']) for v in design['doors']],'rebuild the pack: doors differ'
    assert [(o['x'],o['y'],o['sprite']) for o in module['objects']]==[(*v['tile'],v['sprite']) for v in design['actors']],'rebuild the pack: residents differ'
    assert [(s['x'],s['y']) for s in module['events']['bg_events']]==[tuple(v['tile']) for v in design['signs']],'rebuild the pack: signs differ'
    for index,sign in enumerate(design['signs']):
        commands=module['scripts'][f'MnGrandMaraisSign{index}Text']
        actual=[c['args'][0].removeprefix('"').removesuffix('"') for c in commands if c['command'] in {'text','line','para'}]
        assert actual==dialogue_rows(sign['lines']),'rebuild the pack: sign dialogue differs'
    for actor in design['actors']:
        script='MnGrandMarais'+actor['id']
        states=[('',actor['lines'])]
        if actor.get('signal_lines') is not None:
            states.append(('Signal',actor['signal_lines']))
            assert module['scripts'][script][0]=={'command':'checkevent','args':['EVENT_MN_GRAND_MARAIS_SIGNAL']},'rebuild the pack: resident reaction is missing'
        for suffix,lines in states:
            commands=module['scripts'][script+suffix+'Text']
            actual=[c['args'][0].removeprefix('"').removesuffix('"') for c in commands if c['command'] in {'text','line','para'}]
            assert actual==dialogue_rows(lines),'rebuild the pack: resident dialogue differs'
    for room in design['interiors']:
        actual=pack['data']['maps'][room['map']]
        assert (actual['attributes']['width'],actual['attributes']['height'])==(room['width'],room['height']),'rebuild the pack: room dimensions differ'
        assert actual['blocks']==[int(v,16) for row in room['rows'] for v in row.split()],'rebuild the pack: furniture differs'
        assert [(w['x'],w['y']) for w in actual['events']['warps']]==[tuple(v) for v in room['exits']],'rebuild the pack: exit carpets differ'
    for map_name,entries in design['dialogue'].items():
        for label,lines in entries.items():
            commands=pack['data']['maps'][map_name]['scripts'][label]
            actual=[c['args'][0].removeprefix('"').removesuffix('"') for c in commands if c['command'] in {'text','line','para'}]
            assert actual==dialogue_rows(lines),'rebuild the pack: room dialogue differs'
    output=ROOT/'planning/minnesota/renders/grand-marais'
    output.mkdir(parents=True,exist_ok=True)
    source_art=SourceArt(pack)
    terrain=source_art.render(attributes['tileset_name'],attributes['width'],attributes['height'],module['blocks'])
    save_render(terrain, output/'whole-town-from-pack-terrain-original.png')
    art=terrain.convert('RGBA')
    for actor in module['objects']:
        art.alpha_composite(sprite(pack,actor['sprite'],output/'sprite-inputs'),(actor['x']*16,actor['y']*16-4))
    save_render(art, output/'whole-town-from-pack-original-rgba.png')
    save_render(art.convert('RGB'), output/'whole-town-from-pack.png')
    doubled=art.resize((art.width*2,art.height*2),Image.Resampling.NEAREST)
    save_render(doubled, output/'whole-town-from-pack-2x-original-rgba.png')
    save_render(doubled.convert('RGB'), output/'whole-town-from-pack-2x.png')
    revision=hashlib.sha256(art.tobytes()).hexdigest()[:8]
    save_render(doubled.convert('RGB'), output/f'whole-town-{revision}.png')
    rooms=[]
    for suffix,label in [('FieldStation','Field Station'),('Pokecenter','Center'),('Mart','Mart'),('ArtistHouse','Folk School'),('FisherHouse','Fisher Cottage'),('Lighthouse','Lighthouse')]:
        room=pack['data']['maps']['MnGrandMarais'+suffix];a=room['attributes']
        terrain=source_art.render(a['tileset_name'],a['width'],a['height'],room['blocks'])
        save_render(terrain, output/('room-from-pack-'+suffix+'-terrain-original.png'))
        pixels=terrain.convert('RGBA')
        for actor in room['objects']:
            pixels.alpha_composite(sprite(pack,actor['sprite'],output/'sprite-inputs'),(actor['x']*16,actor['y']*16-4))
        save_render(pixels, output/('room-from-pack-'+suffix+'-original-rgba.png'))
        save_render(pixels.convert('RGB'), output/('room-from-pack-'+suffix+'-rgb.png'))
        rooms.append((label,pixels))
    doors={d['target']:d['tile'] for d in design['doors']}
    actors={a['id']:a['tile'] for a in design['actors']}
    station=doors['MN_GRAND_MARAIS_FIELD_STATION'];lighthouse=doors['MN_GRAND_MARAIS_LIGHTHOUSE']
    harbor=actors['HarborChild'];painter=actors['Painter']
    views=[('waterfront',(harbor[0],harbor[1]-1)),('artist-point',(painter[0],painter[1]-1)),('field-station',(station[0]+3,station[1]+3)),('lighthouse',(lighthouse[0]+2,lighthouse[1]-1))]
    for name,(x,y) in views:
        crop=art.crop((x*16-80,y*16-72,x*16+80,y*16+72))
        save_render(crop, output/('from-pack-'+name+'-native-original-rgba-160x144.png'))
        save_render(crop.convert('RGB'), output/('from-pack-'+name+'-native-rgb-160x144.png'))
        preview=crop.resize((640,576),Image.Resampling.NEAREST)
        save_render(preview, output/('from-pack-'+name+'-preview-original-rgba-640x576.png'))
        save_render(preview.convert('RGB'), output/('from-pack-'+name+'-preview-rgb-640x576.png'))
    interiors=room_sheet(rooms)
    save_render(interiors, output/'interior-review.png')
    interior_revision=hashlib.sha256(interiors.tobytes()).hexdigest()[:8]
    save_render(interiors, output/f'interiors-{interior_revision}.png')
    print(f'Image: whole-town-{revision}.png')
    print(f'Rooms: interiors-{interior_revision}.png')
    print(f'Published pack matches design: {len(module["blocks"])} metatiles, {len(module["events"]["warps"])} doors, {len(module["objects"])} residents. Entire image: {art.width} x {art.height} native pixels.')

if __name__=='__main__':main()
