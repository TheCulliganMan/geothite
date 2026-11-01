#!/usr/bin/env python3
"""Capture exact connected-route pixels using authored native neighbor maps.

Camera bounds follow the production connection-composite clamp. Source art and
native border blocks are reused; actual GPU images are never modified. Every
water glyph uses one consistent native animation phase across the full frame.
Views may explicitly request the ordinary native compositor's two-hop context.
"""
import argparse
import json
import os
from pathlib import Path
import shutil
import subprocess

from PIL import Image, ImageChops

from render_archive import retain_render_file, save_render
from render_reference import SourceArt, load_pack, room_sheet
from render_stages import render_stage
from review_city_native import phases, priority_scene
from review_north_shore import oriented_sprite
from review_passage_native import actors, digest, write

ROOT = Path(__file__).resolve().parents[3]


def falls_phases(art, regions, scene, box):
    """One consistent native three-pixel downward scroll for the whole face.

    Native cadence/offset are verified by the renderer regression. This review
    matches only the eight coherent source phases, never individual glyphs.
    """
    placements = []
    for module, ox, oy in regions:
        a = module['attributes']
        if a['tileset_name'] != 'mn_minneapolis_city' or 153 not in module['blocks']:
            continue
        tile = art.blocks(a['tileset_name'])[153].crop((0, 0, 8, 8)).convert('RGBA')
        for index, block in enumerate(module['blocks']):
            if block != 153:
                continue
            for dy in range(0, 32, 8):
                for dx in range(0, 32, 8):
                    x = ox + index % a['width'] * 32 + dx - box[0]
                    y = oy + index // a['width'] * 32 + dy - box[1]
                    if x < scene.width and x + 8 > 0 and y < scene.height and y + 8 > 0:
                        placements.append((tile, x, y))
    for stage in range(8 if placements else 1):
        image = scene.copy()
        for tile, x, y in placements:
            image.paste(ImageChops.offset(tile, 0, stage * 3 % 8), (x, y))
        yield stage, image


def sources(pack, name, depth=1):
    """Resolve authored rectangles, including explicit native two-hop context.

    The ordinary native compositor visits (map, origin) breadth first and puts
    farther rectangles below direct neighbors. Repeated reciprocal placements
    are omitted. One-hop callers keep their original comparison contract.
    """
    assert depth in {1, 2}, 'source depth must match a supported native context'
    maps = pack['data']['maps']
    pending = [(maps[name], 0, 0, 0)]; seen = {(name, 0, 0)}
    for module, ox, oy, hops in pending:
        if hops == depth:
            continue
        a = module['attributes']
        for link in a['connections']:
            target = maps[link['target_map']]; b = target['attributes']
            offset = link['offset']*32
            dx,dy = {'north':(offset,-b['height']*32), 'south':(offset,a['height']*32),
                     'west':(-b['width']*32,offset), 'east':(a['width']*32,offset)}[link['direction'].lower()]
            key = (target['id'], ox+dx, oy+dy)
            if key not in seen:
                seen.add(key); pending.append((target, ox+dx, oy+dy, hops+1))
    return [(module,x,y) for module,x,y,_ in sorted(pending, key=lambda p:(p[3] != 0, -p[3]))]


def camera_origin(regions, at):
    """Derive 8px-cell LCD clamp; no image registration or fitted translation."""
    left = min(x//8 for _,x,_ in regions); top = min(y//8 for _,_,y in regions)
    right = max((x+m['attributes']['width']*32)//8 for m,x,_ in regions)
    bottom = max((y+m['attributes']['height']*32)//8 for m,_,y in regions)
    sx = max(left,min(at[0]*2-10,max(left,right-20)))
    sy = max(top,min(at[1]*2-9,max(top,bottom-18)))
    return (320+(at[0]*2-sx-10)*24,288+(at[1]*2-sy-9)*24),[sx,sy]


def compare(pack, art, view, actual, directory):
    depth = view.get('source_depth', 1)
    regions = sources(pack,view['map'],depth); base = regions[0][0]; x,y = view['core']
    assert base['attributes']['environment'] in {'ROUTE','TOWN'}
    box = (x*16-256,y*16-192,x*16+256,y*16+192)
    # Every origin is metatile-aligned. Padding is indexed in active-map space,
    # preserving the native border's modulo phase even at negative coordinates.
    left,top = box[0]//32*32,box[1]//32*32
    right,bottom = (box[2]+31)//32*32,(box[3]+31)//32*32
    scene = Image.new('RGBA',(right-left,bottom-top))
    border = art.blocks(base['attributes']['tileset_name'])[base['attributes']['border_block']]
    save_render(border,directory/'active-native-border-block.png')
    for yy in range(0,scene.height,32):
        for xx in range(0,scene.width,32):
            scene.paste(border,(xx,yy))
    masked = 0
    # Native resolver gives the active map precedence, then the last listed
    # connected rectangle. Neighboring actor sprites are not fabricated.
    for module,ox,oy in [*regions[1:],regions[0]]:
        a = module['attributes']
        terrain = art.render(a['tileset_name'],a['width'],a['height'],module['blocks'])
        save_render(terrain,directory/(module['id']+'-whole-native-terrain.png'))
        layer = terrain.convert('RGBA')
        if module is base:
            layer = actors(pack,terrain,module)
            layer.alpha_composite(oriented_sprite(pack,'SPRITE_CHRIS','south'),(x*16,y*16))
            save_render(layer,directory/'active-actors-player-before-priority.png')
            masked = priority_scene(pack,module,layer,terrain,box)
            save_render(layer,directory/'active-actors-player-native-priority.png')
        scene.alpha_composite(layer,(ox-left,oy-top))
    save_render(scene,directory/'connected-original-native-source.png')
    crop = render_stage(scene,'crop',directory,'connected-source512',(box[0]-left,box[1]-top,box[2]-left,box[3]-top))
    # battle_entry_progress::connection_composite_viewport_origin clamps to
    # immediate neighbors. Scenery composition separately includes two hops.
    # A second-hop rectangle can supply visible pixels without extending the
    # camera bounds (for example the Rainy River outlet's southeast trees).
    origin,lcd = camera_origin(sources(pack, view['map'], 1),view['core'])
    best = actual.width*actual.height; matched = None; matched_falls = None
    ownership = []
    for i,(m,ox,oy) in enumerate(regions):
        a=m['attributes']; mask=Image.new('L',crop.size,0)
        mask.paste(255,(ox-box[0],oy-box[1],ox+a['width']*32-box[0],oy+a['height']*32-box[1]))
        # Animation must obey the same authored rectangle ownership as static
        # terrain: active map first, then reverse connection-list precedence.
        # This selects source art, never excludes any GPU comparison pixel.
        for n,nx,ny in ([] if i == 0 else [regions[0],*regions[i+1:]]):
            b=n['attributes']
            mask.paste(0,(nx-box[0],ny-box[1],nx+b['width']*32-box[0],ny+b['height']*32-box[1]))
        save_render(mask,directory/(m['id']+'-native-source-ownership.png'))
        ownership.append(mask)
    for phase in range(4):
        candidate = crop.copy()
        for i,(m,ox,oy) in enumerate(regions):
            local_box = (box[0]-ox,box[1]-oy,box[2]-ox,box[3]-oy)
            variants = list(phases(pack,m,candidate,local_box,directory/(m['id']+'-water-phases')))
            if variants:
                animated = next(image for number,image in variants if number==phase)
                candidate.paste(animated,(0,0),ownership[i])
        for fall_stage, scrolling in falls_phases(art, regions, candidate, box):
            label = f'water{phase}-falls{fall_stage}'
            save_render(scrolling,directory/f'consistent-{label}-source512.png')
            scaled = render_stage(scrolling,'resize',directory,f'{label}-source3x',(1536,1152),Image.Resampling.NEAREST)
            lx,ty=768-origin[0],576-origin[1]
            reference = render_stage(scaled,'crop',directory,f'{label}-source640',(lx,ty,lx+640,ty+576)).convert('RGB')
            delta = ImageChops.difference(actual,reference)
            save_render(delta,directory/f'{label}-difference.png')
            errors = sum(pixel!=(0,0,0) for pixel in delta.getdata())
            if errors<best:
                best,matched,matched_falls = errors,phase,fall_stage
                save_render(reference,directory/'closest-source-reference.png')
    return dict(pixel_mismatches=best,compared_pixels=640*576,consistent_water14_phase=matched,
                consistent_falls_scroll_stage=matched_falls,
                derived_player_origin=list(origin),lcd_origin_8px_cells=lcd,centered_camera_assumption_used=False,
                source_foreground_pixels_over_actors=masked,
                source_connection_depth=depth,
                immediate_native_sources=[dict(map=m['id'],origin_pixels=[ox,oy]) for m,ox,oy in regions])


def main():
    parser=argparse.ArgumentParser(description=__doc__)
    for name in ['pack','binary','views','output']:
        parser.add_argument('--'+name,type=Path,required=True)
    parser.add_argument('--resume',action='store_true')
    args=parser.parse_args(); out=args.output.resolve()
    out.mkdir(parents=True,exist_ok=args.resume)
    pack_path=args.pack.resolve(); pack=load_pack(pack_path); art=SourceArt(pack)
    old=json.loads((out/'capture-progress.json').read_text()) if args.resume else None
    binary=args.binary.resolve(); private=ROOT/'.local'/f'connected-route-capture-{os.getpid()}'
    shutil.copy2(binary,private)
    party_species = os.environ.get(
        'CRYSTAL_RENDER_TEST_PARTY_SPECIES',
        'CINDORE' if 'CINDORE' in pack['data']['pokemon'] else 'TOTODILE')
    report=dict(pack=str(pack_path),pack_sha256=digest(pack_path),binary_sha256=digest(private),views_sha256=digest(args.views),
                scope=__doc__,fixture=f'Explicit local placement with {party_species}, south-facing, hour12; no earned access claim.',captures=[])
    if old:
        assert all(old[k]==report[k] for k in ['pack_sha256','binary_sha256','views_sha256'])
    env=dict(os.environ,CRYSTAL_RENDER_TEST_HEADLESS='1',CRYSTAL_RENDER_TEST_CAPTURE_TARGET='image',
             CRYSTAL_RENDER_TEST_WINDOW_SIZE='640x576',CRYSTAL_PREVIEW_MUTE='1',
             CRYSTAL_RENDER_TEST_PARTY_SPECIES=party_species,CRYSTAL_RENDER_TEST_PARTY_SIZE='1')
    env.pop('CRYSTAL_RENDER_TEST_SECOND_VIEW',None); sheet=[]
    try:
        for view in json.loads(args.views.read_text()):
            folder=out/view['id'];folder.mkdir(exist_ok=args.resume); png=folder/'production.png'
            x,y=view['core']
            assert not any((o['x'],o['y']) == (x,y) for o in pack['data']['maps'][view['map']]['objects']), 'native camera fixture must not overlap an NPC'
            cmd=[str(private),'--pack',str(pack_path),'--map',view['map'],'--x',str(x),'--y',str(y),
                                    '--view','2d','--hour','12','--screenshot',str(png)]
            if old:
                assert digest(png)==next(row['production_sha256'] for row in old['captures'] if row['id']==view['id'])
            else:
                with (folder/'capture.log').open('wb') as log:
                    process=subprocess.run(cmd,env=env,stdout=log,stderr=subprocess.STDOUT,timeout=120)
                assert process.returncode==0,(view['id'],process.returncode)
            retain_render_file(png); actual=render_stage(Image.open(png),'convert',folder,'production-rgb','RGB')
            assert actual.size==(640,576)
            row=dict(view,command=cmd,production_sha256=digest(png),**compare(pack,art,view,actual,folder))
            raw=list((folder/'.render-archive'/png.name).glob('gpu-readback-*.json'));assert raw
            row['raw_gpu_descriptors']=[str(path.relative_to(out)) for path in raw]
            report['captures'].append(row);write(out/'capture-progress.json',report);sheet.append((view['id'],actual))
            print(view['id'],'mismatches',row['pixel_mismatches'],flush=True)
        save_render(room_sheet(sheet,3),out/'production-decisions.png')
        report['status']='PASS' if all(row['pixel_mismatches']==0 for row in report['captures']) else 'REVIEW_DIFFERENCES'
    finally:
        private.unlink();report['private_binary_removed']=True
        report['pack_unchanged']=digest(pack_path)==report['pack_sha256']
        (out/'review_connected_route_native.py.snapshot').write_bytes(Path(__file__).read_bytes())
        write(out/'capture-progress.json',report)
    assert report['status']=='PASS','Inspect retained source/native differences'


if __name__=='__main__':
    main()
