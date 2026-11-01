#!/usr/bin/env python3
"""Read-only native winter-art mirror. It never serializes a game pack."""
import argparse
import copy
import hashlib
import io
import json
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[4]
sys.path.insert(0, str(ROOT / 'planning/minnesota/tools'))
from render_archive import save_render
from render_reference import SourceArt, load_pack
from PIL import Image, ImageDraw


def physical(glyph, palette, count):
    return glyph if not palette & 8 else (glyph & 127) + count // 2


def derive(pack, spec):
    """Apply only in memory; return cloned preview pack and exact art receipts."""
    assert spec['schema_version'] == 1
    assert [r['index_map'] for r in spec['recipes']] == [[1,2,3,3],[0,1,2,3],[1,2,1,3],[1,2,3,3],[1,2,3,3]], 'Native grass priority holes/ice stripes must survive'
    for name,expected in spec['source_definition_sha256'].items():
        assert hashlib.sha256(json.dumps(pack['data']['tilesets'][name],sort_keys=True,separators=(',',':')).encode()).hexdigest()==expected, name
    for path, expected in spec['source_file_sha256'].items():
        assert hashlib.sha256(bytes(pack['runtime_files'][path])).hexdigest() == expected, path
    out = copy.deepcopy(pack)
    receipts = []
    for target in spec['targets']:
        source, name, first = target['source'], target['target'], target['expected_blocks']
        assert name not in out['data']['tilesets']
        definition = copy.deepcopy(pack['data']['tilesets'][source])
        original = bytes(pack['runtime_files'][f'gfx/tilesets/{source}.2bpp'])
        oldblocks = bytes(pack['runtime_files'][f'data/tilesets/{source}_metatiles.bin'])
        assert len(original) == 192 * 16 and len(oldblocks) == first * 16
        graphics = bytearray(256 * 16)
        graphics[:96*16] = original[:96*16]
        graphics[128*16:224*16] = original[96*16:]
        blocks = bytearray(oldblocks)
        used = set(blocks)
        assert len(definition['palette_map']) == 224
        definition['palette_map'].extend([0] * 32)
        free = iter(range(224,256))
        foregrounds = {}; shared = {}; records = []
        for i, recipe in enumerate(spec['recipes']):
            donor, block = recipe['source_tileset'], int(recipe['source_block'], 16)
            donor_def = pack['data']['tilesets'][donor]
            collision = donor_def['collision'][f'{block:02x}']
            assert collision == recipe['collision'], recipe['name']
            donor_graphics = bytes(pack['runtime_files'][f'gfx/tilesets/{donor}.2bpp'])
            native = bytes(pack['runtime_files'][f'data/tilesets/{donor}_metatiles.bin'])[block*16:(block+1)*16]
            for glyph in native:
                slot = physical(glyph, donor_def['palette_map'][glyph], len(donor_graphics)//16)
                pixels = bytearray(donor_graphics[slot*16:(slot+1)*16])
                for row in range(8):
                    low,high=pixels[row*2:row*2+2]; pixels[row*2:row*2+2]=b'\0\0'
                    for col in range(8):
                        bit=7-col;index=((low>>bit)&1)|(((high>>bit)&1)<<1)
                        mapped=recipe['index_map'][index]
                        pixels[row*2] |= (mapped&1)<<bit
                        pixels[row*2+1] |= ((mapped>>1)&1)<<bit
                pixels=bytes(pixels)
                key = (pixels, tuple(tuple(c) for c in recipe['foregrounds']))
                if key not in shared:
                    new = next(free); shared[key] = new
                    graphics[new*16:(new+1)*16] = pixels
                    definition['palette_map'][new] = 3 | (8 if new >= 128 else 0)
                    foregrounds[str(new)] = recipe['foregrounds']
                blocks.append(shared[key])
            definition['collision'][f'{first+i:02x}'] = collision.copy()
            records.append({'id':f'{first+i:02x}', 'name':recipe['name'], 'collision':collision})
        out['data']['tilesets'][name] = definition
        files=out['runtime_files']
        files[f'data/tilesets/{name}_metatiles.bin'] = bytes(blocks)
        files[f'gfx/tilesets/{name}.2bpp'] = bytes(graphics)
        sheet = Image.new('L',(128,128)); pixels=sheet.load()
        for n in range(256):
            for y in range(8):
                for x in range(8):
                    bit=7-x; value=(graphics[n*16+y*2]>>bit&1) | ((graphics[n*16+y*2+1]>>bit&1)<<1)
                    pixels[n%16*8+x,n//16*8+y]=[255,170,85,0][value]
        buf=io.BytesIO();sheet.save(buf,format='PNG');files[f'gfx/tilesets/{name}.png']=buf.getvalue()
        files[f'data/tilesets/{name}_foregrounds.json']=json.dumps(foregrounds).encode()
        receipts.append({'source':source,'target':name,'recipes':records,'new_glyphs':len(shared),
                         'metatiles_sha256':hashlib.sha256(blocks).hexdigest(),
                         'graphics_2bpp_sha256':hashlib.sha256(graphics).hexdigest()})
        old_art=SourceArt(pack).blocks(source)
        new_art=SourceArt(out).blocks(name)
        assert all(a.tobytes()==b.tobytes() for a,b in zip(old_art,new_art)), 'Original source pixels changed'
        for period in ['morn','nite','dark']:
            previous=SourceArt(pack).blocks(source,period);current=SourceArt(out).blocks(name,period)
            assert all(a.tobytes()==b.tobytes() for a,b in zip(previous,current)), period
    assert out['data']['maps'] == pack['data']['maps']
    assert all(out['runtime_files'][k]==v for k,v in pack['runtime_files'].items())
    return out, receipts


def render(pack, spec, output):
    preview, receipts = derive(pack, spec)
    output.mkdir(parents=True,exist_ok=True)
    native=SourceArt(pack); art=SourceArt(preview)
    # Native original references versus authored selective color: 1x and 4x.
    swatch=Image.new('RGB',(576,200),(23,31,42));d=ImageDraw.Draw(swatch)
    names=['snow / FLOOR','grass / TALL_GRASS','striped ice / ICE']
    for i,recipe in enumerate(spec['recipes'][:3]):
        x=16+i*184
        donor=native.blocks(recipe['source_tileset'])[int(recipe['source_block'],16)]
        variant=art.blocks('mn_winter_outdoor')[60+i]
        d.text((x,8),names[i],fill='white');d.text((x,26),'native donor',fill='white')
        swatch.paste(donor.resize((64,64),Image.Resampling.NEAREST),(x,42))
        d.text((x+80,26),'outdoor winter',fill='white')
        swatch.paste(variant.resize((64,64),Image.Resampling.NEAREST),(x+80,42))
        swatch.paste(variant,(x+80,120))
    save_render(swatch,output/'native-winter-swatch.png')
    # Hand-authored material-study clearing. Not a promised playable region.
    rows=[
      'PPPPPPPPPPPP','PPPPSSSSPPPP','PSSSSSSSSSPP','PSSGGGSSSSSP',
      'PSSGGGSSISSP','PSSSSSIIIISP','PPSSSSIIIISP','PPSSSSSSSSPP','PPPPPPPPPPPP']
    # Optional snowy native05 canopy; every quadrant remains WALL.
    vocabulary={'P':63,'S':60,'G':61,'I':62}
    values=[vocabulary[c] for row in rows for c in row]
    study=art.render('mn_winter_outdoor',12,9,values)
    save_render(study,output/'material-study-native32.png')
    save_render(study.resize((768,576),Image.Resampling.NEAREST),output/'material-study-2x.png')
    for name,box in [('snow-grass-ice',(96,80,256,224)),('ice-stop-bank',(192,112,352,256))]:
        crop=study.crop(box);save_render(crop,output/f'{name}-source160.png')
        from review_north_shore import oriented_sprite
        sprite=oriented_sprite(pack,'SPRITE_CHRIS','south')
        player=crop.convert('RGBA');player.alpha_composite(sprite,(72,64));save_render(player,output/f'{name}-player-source160.png')
        save_render(crop.resize((640,576),Image.Resampling.NEAREST),output/f'{name}-4x.png')
    from review_city_native import priority_scene
    from review_north_shore import oriented_sprite
    sprite=oriented_sprite(pack,'SPRITE_CHRIS','south')
    priority=[]; views=[]
    for label,selected,tileset,block in [('native-grass',pack,'johto',3),('icy-grass',preview,'mn_winter_outdoor',61)]:
        ground=SourceArt(selected).render(tileset,5,5,[block]*25).convert('RGBA')
        scene=ground.copy();scene.alpha_composite(sprite,(72,64))
        before=sum(a!=b for a,b in zip(scene.getdata(),ground.getdata()))
        module={'attributes':{'tileset_name':tileset,'width':5,'height':5},'blocks':[block]*25}
        masked=priority_scene(selected,module,scene,ground,(0,0,160,144))
        visible=sum(a!=b for a,b in zip(scene.getdata(),ground.getdata()))
        assert visible>0 and visible<before
        view=scene.crop((0,0,160,144));save_render(view,output/f'{label}-native-priority-player160.png');views.append((label,view))
        priority.append({'name':label,'sprite_pixels_before_priority':before,'masked_pixels':masked,'visible_pixels':visible})
    assert priority[0]['visible_pixels']==priority[1]['visible_pixels'], priority
    comparison=Image.new('RGB',(664,316),(23,31,42));d=ImageDraw.Draw(comparison)
    for i,(label,view) in enumerate(views):
        d.text((i*332+6,6),label+' native priority',fill='white');comparison.paste(view.resize((320,288),Image.Resampling.NEAREST),(i*332+6,22))
    save_render(comparison,output/'grass-priority-comparison-2x.png')
    (output/'proof.json').write_text(json.dumps({'status':'SOURCE_ONLY_PASS','receipts':receipts,
       'original_maps_equal':True,'all_original_runtime_files_equal':True,
       'original_prefix_same_all_four_timebanks_pixels':True,'production_published':False,'grass_priority_comparisons':priority,
       'spec_sha256':hashlib.sha256(json.dumps(spec,indent=2).encode()+b'\n').hexdigest()},indent=2)+'\n')
    return receipts


if __name__=='__main__':
    parser=argparse.ArgumentParser();parser.add_argument('pack',type=Path);parser.add_argument('output',type=Path)
    args=parser.parse_args();spec=json.loads((Path(__file__).parent/'winter-art-v01.json').read_text())
    print(json.dumps(render(load_pack(args.pack),spec,args.output),indent=2))
    proof_path=args.output/'proof.json';proof=json.loads(proof_path.read_text())
    proof['base_pack']=str(args.pack.resolve());proof['base_pack_sha256']=hashlib.sha256(args.pack.read_bytes()).hexdigest()
    proof['preview_helper_sha256']=hashlib.sha256(Path(__file__).read_bytes()).hexdigest()
    proof_path.write_text(json.dumps(proof,indent=2)+'\n')
