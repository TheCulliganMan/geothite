#!/usr/bin/env python3
"""Bounded source-art preview for snow-ground native signs/Headbutt edges."""
import argparse
import copy
import hashlib
import io
import json
import re
from pathlib import Path

from PIL import Image,ImageDraw
import preview_winter_art_v02 as winter


def canonical(value):
    return json.dumps(value,sort_keys=True,separators=(',',':')).encode()


def day_bank(pack):
    text=bytes(pack['runtime_files']['gfx/tilesets/bg_tiles.pal']).decode()
    text=text.split('; day')[1].split('; nite')[0]
    colors=[]
    for line in text.splitlines():
        if line.strip().startswith('RGB'):
            values=list(map(int,re.findall(r'\d+',line.split(';')[0])))
            colors += [tuple((v<<3)|(v>>2) for v in values[i:i+3]) for i in range(0,len(values),3)]
    return [colors[i:i+4] for i in range(0,32,4)]


def remap(raw):
    output=bytearray(16);mapping=[1,2,3,3]
    for y in range(8):
        for x in range(8):
            bit=7-x;level=(raw[y*2]>>bit&1)|((raw[y*2+1]>>bit&1)<<1);value=mapping[level]
            output[y*2] |= (value&1)<<bit;output[y*2+1] |= (value>>1)<<bit
    return bytes(output)


def derive_edges(pack,spec):
    assert spec['schema_version']==1
    for name,expected in spec['source_file_sha256'].items():
        assert hashlib.sha256(bytes(pack['runtime_files'][name])).hexdigest()==expected,name
    for name,expected in spec['source_definition_sha256'].items():
        assert hashlib.sha256(canonical(pack['data']['tilesets'][name])).hexdigest()==expected,name
    out=copy.deepcopy(pack);bank=day_bank(pack);receipts=[]
    for target in spec['targets']:
        name=target['tileset'];source=target['source'];d=out['data']['tilesets'][name]
        old=bytes(pack['runtime_files'][f'data/tilesets/{name}_metatiles.bin']);graphics=bytearray(pack['runtime_files'][f'gfx/tilesets/{name}.2bpp'])
        foregrounds=json.loads(bytes(pack['runtime_files'][f'data/tilesets/{name}_foregrounds.json']))
        assert len(old)==target['expected_blocks']*16 and len(graphics)==256*16
        assert hashlib.sha256(old).hexdigest()==target['metatiles_sha256']
        assert hashlib.sha256(graphics).hexdigest()==target['graphics_sha256']
        assert hashlib.sha256(canonical(foregrounds)).hexdigest()==target['foregrounds_sha256']
        assert d==pack['data']['tilesets'][name]
        blocks=bytearray(old);used=set(old);free=iter(g for g in range(224,256) if g not in used)
        shared={}
        for glyph,color in foregrounds.items():
            glyph=int(glyph);slot=winter.physical(glyph,d['palette_map'][glyph],256)
            shared[(bytes(graphics[slot*16:(slot+1)*16]),tuple(map(tuple,color)))]=glyph
        snow=old[int(target['snow_block'],16)*16:(int(target['snow_block'],16)+1)*16]
        canopy=old[int(target['canopy_block'],16)*16:(int(target['canopy_block'],16)+1)*16]
        white=foregrounds[str(snow[0])][0];tree_colors=foregrounds[str(canopy[0])]
        native=pack['data']['tilesets'][source];native_blocks=bytes(pack['runtime_files'][f'data/tilesets/{source}_metatiles.bin']);native_gfx=bytes(pack['runtime_files'][f'gfx/tilesets/{source}.2bpp'])
        new_glyphs=[]
        for recipe in target['blocks']:
            block=int(recipe['source_block'],16);collision=native['collision'][f'{block:02x}']
            assert 'CUT_TREE' not in collision and collision==recipe['collision']
            assert int(recipe['id'],16)==len(blocks)//16
            glyphs=native_blocks[block*16:(block+1)*16]
            for position,glyph in enumerate(glyphs):
                if glyph==int(target['ground_glyph'],16):
                    blocks.append(snow[position]);continue
                attribute=native['palette_map'][glyph];palette=attribute&7
                slot=winter.physical(glyph,attribute,len(native_gfx)//16)
                raw=remap(native_gfx[slot*16:(slot+1)*16])
                colors=tree_colors if palette==2 else [white,list(bank[palette][1]),list(bank[palette][3])]
                key=raw,tuple(map(tuple,colors))
                if key not in shared:
                    replacement=next(free);shared[key]=replacement;new_glyphs.append(replacement)
                    graphics[replacement*16:(replacement+1)*16]=raw;d['palette_map'][replacement]=11;foregrounds[str(replacement)]=colors
                blocks.append(shared[key])
            d['collision'][recipe['id']]=collision.copy()
        assert d['palette_map'][:224]==pack['data']['tilesets'][name]['palette_map'][:224]
        for glyph in used:assert d['palette_map'][glyph]==pack['data']['tilesets'][name]['palette_map'][glyph]
        files=out['runtime_files'];files[f'data/tilesets/{name}_metatiles.bin']=bytes(blocks);files[f'gfx/tilesets/{name}.2bpp']=bytes(graphics)
        files[f'data/tilesets/{name}_foregrounds.json']=canonical(foregrounds)
        sheet=Image.new('L',(128,128));pix=sheet.load()
        for n in range(256):
            for y in range(8):
                for x in range(8):
                    bit=7-x;value=(graphics[n*16+y*2]>>bit&1)|((graphics[n*16+y*2+1]>>bit&1)<<1)
                    pix[n%16*8+x,n//16*8+y]=[255,170,85,0][value]
        buf=io.BytesIO();sheet.save(buf,format='PNG');files[f'gfx/tilesets/{name}.png']=buf.getvalue()
        for time in ['morn','day','nite','dark']:
            previous=winter.SourceArt(pack).blocks(name,time);current=winter.SourceArt(out).blocks(name,time)
            assert all(a.tobytes()==b.tobytes() for a,b in zip(previous,current)),name+time
        receipts.append({'tileset':name,'new_blocks':len(target['blocks']),'new_glyphs':[f'{g:02x}' for g in new_glyphs],
            'metatiles_sha256':hashlib.sha256(blocks).hexdigest(),'graphics_sha256':hashlib.sha256(graphics).hexdigest()})
    assert out['data']['maps']==pack['data']['maps']
    return out,receipts


def render(pack,spec,output):
    preview,receipts=derive_edges(pack,spec);output.mkdir(parents=True,exist_ok=True)
    source=winter.SourceArt(pack);after=winter.SourceArt(preview)
    count=max(len(t['blocks']) for t in spec['targets']);image=Image.new('RGB',(count*144,356),(23,31,42));draw=ImageDraw.Draw(image)
    for target in spec['targets']:
        row=0 if target['source']=='mn_border_pines' else 1
        for i,r in enumerate(target['blocks']):
            x=i*144;y=row*178;draw.text((x+4,y+4),target['source']+':'+r['source_block'],fill='white')
            original=source.blocks(target['source'])[int(r['source_block'],16)];new=after.blocks(target['tileset'])[int(r['id'],16)]
            image.paste(original.resize((64,64),Image.Resampling.NEAREST),(x+4,y+28));image.paste(new.resize((64,64),Image.Resampling.NEAREST),(x+76,y+28));draw.text((x+76,y+100),'snow '+r['id'],fill='white')
    winter.save_render(image,output/'native-mixed-edge-contact-2x.png')
    # A hand-arranged sign/Headbutt verge at native player scale; no route claim.
    rows=[[0x3f]*5,[0x3f,0x46,0x3c,0x47,0x3f],[0x3f,0x3c,0x41,0x3c,0x3f],[0x3f,0x48,0x3c,0x42,0x3f],[0x3f]*5]
    scene=after.render('mn_winter_outdoor',5,5,sum(rows,[])).crop((0,0,160,144)).convert('RGBA')
    from review_north_shore import oriented_sprite
    assert preview['data']['tilesets']['mn_winter_outdoor']['collision']['41'][1]=='FLOOR'
    scene.alpha_composite(oriented_sprite(pack,'SPRITE_CHRIS','south'),(80,60))
    winter.save_render(scene,output/'snow-sign-headbutt-source-player160.png');winter.save_render(scene.resize((640,576),Image.Resampling.NEAREST),output/'snow-sign-headbutt-source-player-4x.png')
    proof={'status':'SOURCE_ONLY_PASS','receipts':receipts,'old_prefix_all_four_timebanks_equal':True,'maps_equal':True,'cut_recipes_added':0,'production_published':False,'spec_canonical_sha256':hashlib.sha256(canonical(spec)).hexdigest(),'source_player_core':[5,4]}
    (output/'proof.json').write_text(json.dumps(proof,indent=2)+'\n');return proof


if __name__=='__main__':
    parser=argparse.ArgumentParser();parser.add_argument('pack',type=Path);parser.add_argument('output',type=Path);args=parser.parse_args()
    pack=winter.load_pack(args.pack)
    if 'mn_winter_outdoor' not in pack['data']['tilesets']:
        pack,_=winter.derive(pack,json.loads((Path(__file__).parent/'winter-art-v01.json').read_text()))
    spec=json.loads((Path(__file__).parent/'winter-mixed-edges-v01.json').read_text())
    print(json.dumps(render(pack,spec,args.output),indent=2))
