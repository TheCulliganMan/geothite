"""Read-only native chevron/slat composition matching the Rust authoring recipe."""
import re


def marked_wind(pack, source, wind, group='indoor'):
    assert wind in ('WALK_RIGHT', 'WALK_LEFT')
    files = pack['runtime_files']
    glyph = bytes(files['gfx/font/font.1bpp'])[0x6d*8:0x6e*8]
    assert len(glyph) == 8
    layout = bytes(files['data/tilesets/tower_metatiles.bin'])[11*16:12*16]
    palettes = pack['data']['tilesets']['tower']['palette_map']
    indexes = {palettes[t] & 7 for t in layout}
    assert len(indexes) == 1, 'chevrons need one consistent native wood palette'
    text = bytes(files['gfx/tilesets/bg_tiles.pal']).decode()
    colors, current = [], None
    for raw in text.splitlines():
        if raw.strip().startswith(';') and 'RGB' not in raw.upper():
            current = raw[1:].strip().lower()
        line = raw.split(';')[0].strip()
        if current == group and line.upper().startswith('RGB'):
            values = list(map(int, re.findall(r'\d+', line)))
            colors += [tuple((v << 3) | (v >> 2) for v in values[i:i+3])
                       for i in range(0, len(values), 3)]
    palette = colors[next(iter(indexes))*4:next(iter(indexes))*4+4]
    assert len(palette) == 4
    image = source.copy()
    for qy in (0, 16):
        for qx in (0, 16):
            for y in range(8):
                for x in range(8):
                    sx = x if wind == 'WALK_RIGHT' else 7-x
                    ink = glyph[y] & (1 << (7-sx))
                    image.putpixel((qx+4+x, qy+4+y), palette[3 if ink else 0])
    return image
