#!/usr/bin/env python3
"""Retain headless city captures and compare full frames to installed native art."""
import argparse
import io
import json
import os
from pathlib import Path
import re
import shutil
import subprocess

from PIL import Image, ImageChops

from render_archive import retain_render_file, save_render
from render_reference import SourceArt, load_pack, room_sheet
from render_stages import render_stage
from review_north_shore import oriented_sprite
from review_passage_native import actors, digest, write

ROOT = Path(__file__).resolve().parents[3]


def native_scene_crop(art, module, scene, box):
    """Include the active map's repeated native border outside its extent."""
    left, top, right, bottom = box
    border = art.blocks(module['attributes']['tileset_name'], 'day')[
        module['attributes']['border_block']]
    crop = Image.new('RGBA', (right - left, bottom - top))
    for y in range(top // 32 * 32, bottom, 32):
        for x in range(left // 32 * 32, right, 32):
            crop.paste(border, (x - left, y - top))
    crop.paste(scene, (-left, -top))
    return crop


def priority_scene(pack, module, scene, ground, box):
    """Mirror native nonzero-index foreground; never modify the GPU capture."""
    attr = module['attributes']; name = attr['tileset_name']
    tileset = pack['data']['tilesets'][name]; files = pack['runtime_files']
    sheet = Image.open(io.BytesIO(bytes(files[f'gfx/tilesets/{name}.png']))).convert('RGB')
    count = sheet.width * sheet.height // 64
    layout = bytes(files[f'data/tilesets/{name}_metatiles.bin'])
    tokens = {'TALL_GRASS','LONG_GRASS','LONG_GRASS_1C','GRASS_48','GRASS_49',
              'GRASS_4A','GRASS_4B','GRASS_4C','BOOKSHELF','COUNTER','COUNTER_98',
              'INCENSE_BURNER','MART_SHELF','PC'}
    masked = 0
    for i, block in enumerate(module['blocks']):
        bx,by = i%attr['width']*32,i//attr['width']*32
        if bx >= box[2] or bx+32 <= box[0] or by >= box[3] or by+32 <= box[1]:
            continue
        collision = tileset['collision'][f'{block:02x}']
        foreground = collision[:2] == ['FLOOR','FLOOR'] and all(v == 'WALL' or v in tokens for v in collision[2:])
        for j,glyph in enumerate(layout[block*16:(block+1)*16]):
            sx,sy = j%4,j//4
            if collision[(sy//2)*2+sx//2] not in tokens and not (foreground and sy >= 1):
                continue
            bank = (tileset['palette_map'][glyph] >> 3)&1; source = glyph
            if bank and count%2 == 0 and (glyph&127)+count//2 < count:
                source = (glyph&127)+count//2
            elif bank and (glyph&127)+128 < count:
                source = (glyph&127)+128
            elif source >= count:
                source = glyph-128 if glyph >= 128 and glyph-128 < count else 0
            for v in range(4 if foreground and sy == 1 else 0,8):
                for u in range(8):
                    if sheet.getpixel((source%(sheet.width//8)*8+u,source//(sheet.width//8)*8+v))[0] < 213:
                        p = bx+sx*8+u,by+sy*8+v
                        if scene.getpixel(p)[:3] != ground.getpixel(p)[:3]:
                            masked += 1
                        scene.putpixel(p,ground.getpixel(p))
    return masked


def phases(pack, module, scene, box, folder):
    """One genuine outdoor water phase across every visible water glyph."""
    attr = module['attributes']; name = attr['tileset_name']
    palette_map = pack['data']['tilesets'][name]['palette_map']
    source = bytes(pack['runtime_files']['gfx/tilesets/bg_tiles.pal']).decode()
    current = ''; colors = []
    for line in source.splitlines():
        if line.strip().startswith(';'):
            current = line.strip()[1:].strip().lower()
        if line.strip().startswith('RGB') and current == 'day':
            values = list(map(int, re.findall(r'\d+', line.split(';')[0])))
            colors.extend(tuple((v << 3) | (v >> 2) for v in values[i:i+3]) for i in range(0,len(values),3))
    palette = colors[(palette_map[0x14]&7)*4:(palette_map[0x14]&7)*4+4]
    assert len(palette) == 4
    raw = bytes(pack['runtime_files']['gfx/tilesets/water/water.2bpp'])
    layout = bytes(pack['runtime_files'][f'data/tilesets/{name}_metatiles.bin'])
    at = []
    for i, block in enumerate(module['blocks']):
        for j, glyph in enumerate(layout[block*16:(block+1)*16]):
            x,y = i%attr['width']*32+j%4*8, i//attr['width']*32+j//4*8
            if glyph == 0x14 and x < box[2] and x+8 > box[0] and y < box[3] and y+8 > box[1]:
                at.append((x-box[0],y-box[1]))
    if not at:
        return
    for frame in range(4):
        tile = Image.new('RGBA',(8,8))
        for y in range(8):
            lo,hi = raw[frame*16+y*2:frame*16+y*2+2]
            for x in range(8):
                bit = 1 << (7-x)
                tile.putpixel((x,y),palette[(2 if hi&bit else 0)|(1 if lo&bit else 0)]+(255,))
        save_render(tile,folder/f'native-water14-phase{frame}-8.png')
        image = scene.copy()
        for point in at:
            image.paste(tile,point)
        yield frame,image


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ['pack','binary','views','output']:
        parser.add_argument('--'+name,type=Path,required=True)
    parser.add_argument('--resume',action='store_true')
    args = parser.parse_args()
    pack_path,binary,out = args.pack.resolve(),args.binary.resolve(),args.output.resolve()
    views = json.loads(args.views.read_text()); pack = load_pack(pack_path)
    assert views and isinstance(views,list)
    out.mkdir(parents=True,exist_ok=args.resume)
    old = json.loads((out/'capture-progress.json').read_text()) if args.resume else None
    if old:
        assert old['pack_sha256'] == digest(pack_path) and old['binary_sha256'] == digest(binary)
        assert old['views_sha256'] == digest(args.views)
    private = ROOT/'.local'/f'city-native-{os.getpid()}'
    shutil.copy2(binary,private)
    report = dict(pack=str(pack_path),pack_sha256=digest(pack_path),binary=str(binary),binary_sha256=digest(private),
                  views_sha256=digest(args.views),fixture='Explicit empty-party placement, south-facing, hour12. No progression claim.',
                  captures=[],personally_visually_reviewed=False)
    env = dict(os.environ,CRYSTAL_RENDER_TEST_HEADLESS='1',CRYSTAL_RENDER_TEST_CAPTURE_TARGET='image',
               CRYSTAL_RENDER_TEST_WINDOW_SIZE='640x576',CRYSTAL_PREVIEW_MUTE='1')
    env.pop('CRYSTAL_RENDER_TEST_SECOND_VIEW',None)
    art = SourceArt(pack); cached = {}; terrain = {}; sheet = []
    try:
        for view in views:
            name,label,(x,y) = view['map'],view['id'],view['core']
            module = pack['data']['maps'][name]; attr = module['attributes']
            assert attr['environment'] in ['TOWN','ROUTE'], 'exterior native camera contract'
            if name not in cached:
                ground = art.render(attr['tileset_name'],attr['width'],attr['height'],module['blocks'],group='day')
                cached[name] = actors(pack,ground,module)
                terrain[name] = ground
                save_render(ground,out/f'{name}-whole-terrain.png')
                save_render(cached[name],out/f'{name}-whole-native-source.png')
            folder = out/label; folder.mkdir(exist_ok=args.resume)
            png = folder/'production.png'
            cmd = [str(private),'--pack',str(pack_path),'--map',name,'--x',str(x),'--y',str(y),
                   '--view','2d','--hour','12','--screenshot',str(png)]
            if old:
                previous = next(v for v in old['captures'] if v['id'] == label)
                assert digest(png) == previous['production_sha256']
            else:
                with (folder/'capture.log').open('wb') as log:
                    result = subprocess.run(cmd,env=env,stdout=log,stderr=subprocess.STDOUT,timeout=120)
                assert result.returncode == 0,(label,result.returncode)
            retain_render_file(png)
            actual = render_stage(Image.open(png),'convert',folder,'production','RGB')
            assert actual.size == (640,576)
            scene = cached[name].copy(); scene.alpha_composite(oriented_sprite(pack,'SPRITE_CHRIS','south'),(x*16,y*16))
            box = (x*16-256,y*16-192,x*16+256,y*16+192)
            foreground_pixels = priority_scene(pack,module,scene,terrain[name],box)
            crop = native_scene_crop(art, module, scene, box)
            save_render(crop, folder/'source512-with-native-border.png')
            best = actual.width*actual.height; matched = None
            for phase, candidate in [(None,crop),*list(phases(pack,module,crop,box,folder))]:
                key = 'static' if phase is None else str(phase)
                save_render(candidate,folder/f'phase-{key}-source512.png')
                scaled = render_stage(candidate,'resize',folder,f'phase-{key}-source3x',(1536,1152),Image.Resampling.NEAREST)
                reference = render_stage(scaled,'crop',folder,f'phase-{key}-source640',(448,288,1088,864)).convert('RGB')
                difference = ImageChops.difference(actual,reference)
                save_render(difference,folder/f'phase-{key}-difference.png')
                errors = sum(pixel != (0,0,0) for pixel in difference.getdata())
                if errors < best:
                    best,matched = errors,phase
                    save_render(reference,folder/'closest-source-reference.png')
            raw = list((folder/'.render-archive'/png.name).glob('gpu-readback-*.json'))
            assert raw,'retain actual raw GPU capture'
            report['captures'].append(dict(id=label,map=name,core=[x,y],command=cmd,production_sha256=digest(png),
                pixel_mismatches=best,compared_pixels=640*576,consistent_water14_phase=matched,
                source_foreground_pixels_over_actors=foreground_pixels,
                raw_gpu_descriptors=[str(path.relative_to(out)) for path in raw]))
            write(out/'capture-progress.json',report); sheet.append((label,actual))
            print(label,'mismatches',best,flush=True)
        save_render(room_sheet(sheet,columns=3),out/'production-decisions.png')
        report['status'] = 'PASS' if all(v['pixel_mismatches'] == 0 for v in report['captures']) else 'FAIL'
    finally:
        private.unlink(); report['private_binary_removed'] = not private.exists()
        report['pack_unchanged'] = digest(pack_path) == report['pack_sha256']
        write(out/'capture-progress.json',report)
        (out/'review_city_native.py.snapshot').write_bytes(Path(__file__).read_bytes())
    assert report['status'] == 'PASS','inspect retained native differences'


if __name__ == '__main__':
    main()
