#!/usr/bin/env python3
"""Read-only exact-art planning renderer. Never writes pack assets or game packs.

Usage: python render_reference.py PACK OUTPUT
Dependencies: Pillow, cbor2. Coordinates in authoring layouts are 32px metatiles.
"""
from render_archive import save_render
import io
import json
import re
import sys
from pathlib import Path

import cbor2
from PIL import Image, ImageDraw


def load_pack(path):
    raw = Path(path).read_bytes()
    assert raw[:12] == b'CRYSTALPACK\0', 'not a Crystal pack'
    assert int.from_bytes(raw[14:18], 'big') == len(raw)-22, 'invalid frame length'
    value = 2166136261
    for byte in raw[22:]:
        value = ((value ^ byte) * 16777619) & 0xffffffff
    assert value == int.from_bytes(raw[18:22], 'big'), 'invalid frame hash'
    return cbor2.loads(raw[22:])


def npc_palette_pack(pack, group):
    """Adapt the existing oriented-frame helper to an unchanged native bank."""
    if group == 'day':
        return pack
    raw = bytes(pack['runtime_files']['gfx/overworld/npc_sprites.pal']).decode()
    parts = re.split(r'^;\s*(morn|day|nite|dark)\s*$', raw, flags=re.M)
    banks = {parts[i]: parts[i+1] for i in range(1, len(parts), 2)}
    assert group in banks, 'Native NPC palette bank is missing'
    adapted = dict(pack)
    adapted['runtime_files'] = dict(pack['runtime_files'])
    adapted['runtime_files']['gfx/overworld/npc_sprites.pal'] = (
        '; day\n' + banks[group] + '\n; nite\n').encode()
    return adapted


class SourceArt:
    def __init__(self, pack):
        self.pack = pack
        self.cache = {}

    def blocks(self, tileset, group='day', palette_override=None):
        assert palette_override is None or 0 <= palette_override < 8, 'native palette index'
        key = tileset, group, palette_override
        if key in self.cache:
            return self.cache[key]
        files = self.pack['runtime_files']
        own = f'gfx/tilesets/{tileset}.pal'
        special = own in files and not (tileset == 'ice_path' and group == 'indoor')
        text = bytes(files[own] if special else files['gfx/tilesets/bg_tiles.pal']).decode()
        colors = []
        current = 'default'
        for raw in text.splitlines():
            line = raw.strip()
            if line.startswith(';') and 'RGB' not in line.upper():
                current = line[1:].strip().lower().replace(' ', '_')
            line = line.split(';')[0].strip()
            if line.upper().startswith('RGB') and (special or current == group):
                values = list(map(int, re.findall(r'\d+', line)))
                colors.extend(tuple((v << 3) | (v >> 2) if v <= 31 else v
                                    for v in values[i:i+3]) for i in range(0,len(values),3))
        palettes = [colors[i:i+4] for i in range(0,32,4)]
        sheet = Image.open(io.BytesIO(bytes(files[f'gfx/tilesets/{tileset}.png']))).convert('RGB')
        count = sheet.width*sheet.height//64
        pmap = self.pack['data']['tilesets'][tileset]['palette_map']
        foregrounds = json.loads(bytes(files.get(f'data/tilesets/{tileset}_foregrounds.json', b'{}')))
        tiles = []
        for index in range(256):
            value = pmap[index] if index < len(pmap) else 0
            if palette_override is not None:
                value = (value & 8) | palette_override
            bank = (value >> 3) & 1
            source = index
            if bank and count%2 == 0 and (index&127)+count//2 < count:
                source = (index&127)+count//2
            elif bank and (index&127)+128 < count:
                source = (index&127)+128
            elif source >= count:
                source = index-128 if index >= 128 and index-128 < count else 0
            x,y = source%(sheet.width//8)*8, source//(sheet.width//8)*8
            tile = sheet.crop((x,y,x+8,y+8))
            palette = palettes[value&7]
            if str(index) in foregrounds:
                palette = [palettes[0][0], *[tuple(color) for color in foregrounds[str(index)]]]
            tile.putdata([palette[0 if p[0]>=213 else 1 if p[0]>=128 else 2 if p[0]>=43 else 3]
                          for p in tile.getdata()])
            tiles.append(tile)
        layout = bytes(files[f'data/tilesets/{tileset}_metatiles.bin'])
        result = []
        for i in range(len(layout)//16):
            block = Image.new('RGB',(32,32))
            for j,tile in enumerate(layout[i*16:i*16+16]):
                block.paste(tiles[tile],(j%4*8,j//4*8))
            result.append(block)
        self.cache[key] = result
        return result

    def render(self, tileset, width, height, blocks, group='day', derived=None):
        art = list(self.blocks(tileset,group))
        for source in derived or []:
            assert int(source['id'],16)==len(art),'derived blocks must append in order'
            block=self.blocks(source['source_tileset'],group,source.get('palette_override'))[int(source['source_block'],16)]
            if 'tile_row_order' in source:
                order=source['tile_row_order']
                assert len(order)==4 and all(isinstance(row,int) and 0<=row<4 for row in order),'derived glyph rows must select four exact source rows'
                arranged=Image.new('RGB',(32,32))
                for row,original in enumerate(order):
                    arranged.paste(block.crop((0,original*8,32,original*8+8)),(0,row*8))
                block=arranged
            if 'tile_row_sources' in source:
                rows=source['tile_row_sources']
                assert len(rows)==4,'derived glyph needs four exact source rows'
                arranged=Image.new('RGB',(32,32))
                for row,entry in enumerate(rows):
                    original=entry['source_row'];assert 0<=original<4,'source glyph row'
                    image=self.blocks(entry.get('source_tileset',source['source_tileset']),group,entry.get('palette_override',source.get('palette_override')))[int(entry['source_block'],16)]
                    arranged.paste(image.crop((0,original*8,32,original*8+8)),(0,row*8))
                block=arranged
            art.append(block)
        assert len(blocks) == width*height
        result = Image.new('RGB',(width*32,height*32))
        for i,block in enumerate(blocks):
            assert 0 <= block < len(art)
            result.paste(art[block],(i%width*32,i//width*32))
        return result


def room_sheet(rooms, columns=3):
    """Arrange exact room pixels at integer 2x scale for visual review."""
    width=max(im.width for _,im in rooms)*2+24
    height=max(im.height for _,im in rooms)*2+42
    sheet=Image.new('RGB',(width*columns,height*((len(rooms)+columns-1)//columns)),(19,29,40))
    draw=ImageDraw.Draw(sheet)
    for i,(label,im) in enumerate(rooms):
        x=i%columns*width+12;y=i//columns*height+30
        draw.text((x,y-18),label,fill='white')
        sheet.paste(im.resize((im.width*2,im.height*2),Image.Resampling.NEAREST),(x,y))
    return sheet


def source_reference(pack, output):
    output = Path(output)
    output.mkdir(parents=True,exist_ok=True)
    art = SourceArt(pack)
    for tileset in ['johto','johto_modern','kanto','forest','cave','port','park']:
        blocks = art.blocks(tileset)
        atlas = Image.new('RGB',(16*72,((len(blocks)+15)//16)*86),(19,29,40))
        draw = ImageDraw.Draw(atlas)
        for i,block in enumerate(blocks):
            x,y = i%16*72, i//16*86
            atlas.paste(block.resize((64,64),Image.Resampling.NEAREST),(x,y))
            draw.text((x+2,y+66),f'{i:02x}',fill='white')
        save_render(atlas, output/f'{tileset}-atlas.png')
    names=['Route29','Route30','Route34','Route42','Route45','NewBarkTown',
           'GoldenrodCity','OlivineCity','IlexForest','NationalPark']
    for name in names:
        m=pack['data']['maps'][name];a=m['attributes']
        save_render(art.render(a['tileset_name'],a['width'],a['height'],m['blocks'],a['time_of_day'] or 'day'), output/f'{name}.png')
    print(f'Rendered {len(names)} native-scale source maps and seven metatile references to {output}')


if __name__ == '__main__':
    source_reference(load_pack(sys.argv[1]),sys.argv[2])


def layout_blocks(layout):
    """Rasterize explicitly authored strokes; no terrain inference or random seed."""
    w,h=layout['dimensions_metatiles']
    palette={'floor':0x01,'lawn':0x02,'encounter':0x03,'flowers':0x04,
             'forest':0x05,'water':0x35,'boulder':0x0a,'sign':0x45}
    palette.update({key:int(value,16) for key,value in layout.get('materials',{}).items()})
    cliff_map={int(key,16):int(value,16) for key,value in layout.get('cliff_blocks',{}).items()}
    cliff_ids=set(cliff_map.values()) if cliff_map else set(range(0x68,0x74))
    canvas=Image.new('I',(w,h),palette['forest'])
    draw=ImageDraw.Draw(canvas)
    for layer in layout['layers']:
        kind=layer['kind']
        if kind=='polygon':
            draw.polygon([tuple(p) for p in layer['points']],fill=palette[layer['material']])
        elif kind=='rect':
            x0,y0,x1,y1=layer['bounds']
            draw.rectangle((x0,y0,x1-1,y1-1),fill=palette[layer['material']])
        elif kind=='path':
            if layout.get('reject_cliff_cuts'):
                mask=Image.new('1',(w,h));ImageDraw.Draw(mask).line([tuple(p) for p in layer['points']],fill=1,width=layer.get('width_metatiles',1))
                cuts=[(x,y) for y in range(h) for x in range(w) if mask.getpixel((x,y)) and canvas.getpixel((x,y)) in cliff_ids]
                assert not cuts,f'{layer.get("name","path")} cuts a cliff at {cuts[:8]}'
            draw.line([tuple(p) for p in layer['points']],fill=palette[layer.get('material','floor')],width=layer.get('width_metatiles',1))
        elif kind=='cliff_polygon':
            mask=Image.new('1',(w,h));ImageDraw.Draw(mask).polygon([tuple(p) for p in layer['points']],fill=1)
            def rock(x,y):return 0<=x<w and 0<=y<h and bool(mask.getpixel((x,y)))
            for y in range(h):
                for x in range(w):
                    if not rock(x,y):continue
                    n,s,west,e=rock(x,y-1),rock(x,y+1),rock(x-1,y),rock(x+1,y)
                    b=(0x6a if not n and not west else 0x6b if not n and not e else
                       0x6c if not s and not west else 0x6d if not s and not e else
                       0x70 if not n else 0x72 if not s else 0x68 if not west else 0x69 if not e else
                       0x6e if not rock(x-1,y+1) else 0x6f if not rock(x+1,y+1) else 0x71)
                    canvas.putpixel((x,y),cliff_map.get(b,b))
        elif kind=='plateau':
            x0,y0,x1,y1=layer['bounds']
            draw.rectangle((x0,y0,x1-1,y1-1),fill=0x71)
            draw.line((x0+1,y0,x1-2,y0),fill=0x70)
            draw.line((x0,y0+1,x0,y1-2),fill=0x68)
            draw.line((x1-1,y0+1,x1-1,y1-2),fill=0x69)
            draw.line((x0+1,y1-1,x1-2,y1-1),fill=0x72)
            for x,y,b in [(x0,y0,0x6a),(x1-1,y0,0x6b),(x0,y1-1,0x6c),(x1-1,y1-1,0x6d)]:
                canvas.putpixel((x,y),b)
        elif kind=='house':
            x,y=layer['origin']
            for dy,row in enumerate([[0x18,0x19],[0x16,0x1e]]):
                for dx,b in enumerate(row):canvas.putpixel((x+dx,y+dy),b)
        elif kind=='point':
            canvas.putpixel(tuple(layer['position']),int(layer['block'],16))
        elif kind=='block_rect':
            x0,y0,x1,y1=layer['bounds']
            draw.rectangle((x0,y0,x1-1,y1-1),fill=int(layer['block'],16))
        elif kind=='block_remap':
            x0,y0,x1,y1=layer['bounds']
            for y in range(y0,y1):
                for x in range(x0,x1):
                    if canvas.getpixel((x,y))==int(layer['from'],16):
                        canvas.putpixel((x,y),int(layer['to'],16))
        else:
            raise ValueError(f'Unknown authored stroke {kind}')
    if layout.get('shore_fitting')=='explicit_source_edges':return list(canvas.getdata())
    # Water edge variants are still WATER collision and only replace water cells.
    original=list(canvas.getdata())
    extra_water={int(b,16) for b in layout.get('additional_water_blocks',[])}
    if layout.get('shore_fitting')=='forest_8_neighbors':
        water_ids={0x02,0x35}|extra_water
        def land(x,y):return 0<=x<w and 0<=y<h and original[y*w+x] not in water_ids
        for y in range(h):
            for x in range(w):
                if original[y*w+x]!=0x02:continue
                n,west,e=land(x,y-1),land(x-1,y),land(x+1,y)
                block=(0x07 if n and west else 0x0d if n and e else
                       0x0a if west else 0x0b if e else 0x03 if n else
                       0x38 if land(x-1,y-1) else 0x39 if land(x+1,y-1) else 0x02)
                canvas.putpixel((x,y),block)
        return list(canvas.getdata())
    if layout.get('shore_fitting')=='kanto_8_neighbors':
        water_id=palette['water']
        def land(x,y):return 0<=x<w and 0<=y<h and original[y*w+x] not in ({water_id,0x78}|extra_water)
        for y in range(h):
            for x in range(w):
                if original[y*w+x]!=water_id:continue
                n,west,e=land(x,y-1),land(x-1,y),land(x+1,y)
                block=(0x1d if n and west else 0x1f if n and e else
                       0x65 if west else 0x64 if e else 0x1e if n else
                       0x2e if land(x-1,y-1) else 0x2d if land(x+1,y-1) else water_id)
                canvas.putpixel((x,y),block)
        return list(canvas.getdata())
    if layout.get('shore_fitting')=='johto_8_neighbors':
        def land(x,y):
            return 0<=x<w and 0<=y<h and original[y*w+x] not in ({0x35,0x06,0x80}|extra_water)
        for y in range(h):
            for x in range(w):
                if original[y*w+x]!=0x35:continue
                n,west,e=land(x,y-1),land(x-1,y),land(x+1,y)
                block=(0x54 if n and west else 0x55 if n and e else
                       0x58 if west else 0x59 if e else 0x76 if n else
                       0x79 if land(x-1,y-1) else 0x7a if land(x+1,y-1) else 0x35)
                canvas.putpixel((x,y),block)
        return list(canvas.getdata())
    def water(x,y):return 0<=x<w and 0<=y<h and original[y*w+x]==0x35
    for y in range(h):
        for x in range(w):
            if not water(x,y):continue
            n,s,e,west=water(x,y-1),water(x,y+1),water(x+1,y),water(x-1,y)
            block=0x35
            if not west:block=0x54 if not n else 0x58
            elif not e:block=0x55 if not n else 0x59
            elif not n:block=0x79
            elif not s:block=0x76
            canvas.putpixel((x,y),block)
    return list(canvas.getdata())


def render_layouts(pack,planning_root,only=None):
    """Render original authored brush layouts with unchanged packed source pixels."""
    import json
    root=Path(planning_root);out=root/'renders'/'routes';out.mkdir(parents=True,exist_ok=True)
    art=SourceArt(pack)
    count=0
    for file in sorted((root/'routes').glob('*.layout.json')):
        layout=json.loads(file.read_text())
        if only and layout['id']!=only:continue
        w,h=layout['dimensions_metatiles']
        blocks=layout_blocks(layout)
        im=art.render(layout['tileset'],w,h,blocks,derived=layout.get('derived_blocks'))
        save_render(im, out/f'{layout["id"]}.png')
        factor=max(1,(max(im.size)+1599)//1600)
        preview=im.resize((im.width//factor,im.height//factor),Image.Resampling.NEAREST)
        save_render(preview, out/f'{layout["id"]}-preview.png')
        count+=1
    print(f'Rendered {count} source-art route blockouts to {out}')
