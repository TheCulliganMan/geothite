"""Read-only native winter art composition for explicit reviewed alias recipes.

This only mutates a decoded pack in memory for SourceArt. It never serializes a
crystalpack, exports game programs or invents map terrain. Native publication
belongs to the guarded Rust builders.
"""
from copy import deepcopy
import io
import json
import re

from PIL import Image


def physical(glyph, palette, count):
    return glyph if not palette & 8 else (glyph & 127) + count // 2


def native_day(raw):
    parts = re.split(r'^;\s*(morn|day|nite|dark|indoor)\s*$',
                     bytes(raw).decode(), flags=re.M)
    banks = {parts[i]: parts[i + 1] for i in range(1, len(parts), 2)}
    values = []
    for line in banks['day'].splitlines():
        line = line.split(';')[0].strip()
        if line.upper().startswith('RGB'):
            values.extend(int(value) for value in re.findall(r'\d+', line))
    assert len(values) >= 96
    return [values[i:i + 3] for i in range(0, 96, 3)]


def remap(raw, mapping):
    result = bytearray(16)
    for y in range(8):
        for x in range(8):
            bit = 7 - x
            value = ((raw[y * 2] >> bit) & 1) | (((raw[y * 2 + 1] >> bit) & 1) << 1)
            mapped = mapping[value]
            result[y * 2] |= (mapped & 1) << bit
            result[y * 2 + 1] |= (mapped >> 1) << bit
    return bytes(result)


def tile_png(graphics):
    assert len(graphics) % (16 * 16) == 0
    image = Image.new('L', (128, len(graphics) // 32))
    pixels = image.load()
    for tile in range(len(graphics) // 16):
        for y in range(8):
            for x in range(8):
                bit = 7 - x
                offset = tile * 16 + y * 2
                value = ((graphics[offset] >> bit) & 1) | (((graphics[offset + 1] >> bit) & 1) << 1)
                pixels[tile % 16 * 8 + x, tile // 16 * 8 + y] = 255 - value * 85
    buffer = io.BytesIO()
    image.save(buffer, format='PNG')
    return buffer.getvalue()


def apply_alias(pack, specification):
    """Compose one explicit alias, preserving block IDs and collision quadrants."""
    source, target = specification['source'], specification['target']
    assert target not in pack['data']['tilesets'], 'private alias is already occupied'
    files = pack['runtime_files']
    original = pack['data']['tilesets'][source]
    definition = deepcopy(original)
    old_graphics = bytes(files[f'gfx/tilesets/{source}.2bpp'])
    count = len(old_graphics) // 16
    assert count in (192, 256)
    old_metatiles = bytes(files[f'data/tilesets/{source}_metatiles.bin'])
    assert len(old_metatiles) // 16 == specification['source_blocks']
    metatiles = bytearray(old_metatiles)
    graphics = bytearray(256 * 16)
    if count == 192:
        graphics[:96 * 16] = old_graphics[:96 * 16]
        graphics[128 * 16:224 * 16] = old_graphics[96 * 16:]
    else:
        graphics[:] = old_graphics
    definition['palette_map'] += [0] * (256 - len(definition['palette_map']))
    inherited = json.loads(bytes(files.get(f'data/tilesets/{source}_foregrounds.json', b'{}')))
    foregrounds = deepcopy(inherited)
    used = set(old_metatiles)
    protected = {physical(glyph, definition['palette_map'][glyph], 256) for glyph in used}
    preferred = list(range(224, 256)) + list(range(96, 128))
    candidates = preferred + [slot for slot in range(256) if slot not in preferred]
    free = iter(slot for slot in candidates
                if slot not in used and slot not in protected and str(slot) not in foregrounds
                and not any(graphics[slot * 16:slot * 16 + 16]))
    shared = {}
    for name, colors in foregrounds.items():
        glyph = int(name)
        slot = physical(glyph, definition['palette_map'][glyph], 256)
        shared[bytes(graphics[slot * 16:slot * 16 + 16]), tuple(map(tuple, colors))] = glyph
    day = native_day(files['gfx/tilesets/bg_tiles.pal'])
    rgb = [tuple((value << 3) | (value >> 2) for value in color) for color in day]

    def donor(tileset, block):
        own = pack['data']['tilesets'][tileset]
        raw = bytes(files[f'gfx/tilesets/{tileset}.2bpp'])
        layout = bytes(files[f'data/tilesets/{tileset}_metatiles.bin'])
        colors = json.loads(bytes(files.get(f'data/tilesets/{tileset}_foregrounds.json', b'{}')))
        result = []
        for glyph in layout[block * 16:block * 16 + 16]:
            palette = own['palette_map'][glyph]
            slot = physical(glyph, palette, len(raw) // 16)
            inks = colors.get(str(glyph), rgb[(palette & 7) * 4 + 1:(palette & 7) * 4 + 4])
            result.append((raw[slot * 16:slot * 16 + 16], tuple(map(tuple, inks))))
        return result

    canopy = donor('mn_winter_north_shore', 0x8b)[0][1]
    frost = donor('mn_winter_outdoor', 0x3d)[0][1]
    ground = int(specification['ground_reference_block'], 16)
    ground_glyphs = set(old_metatiles[ground * 16:ground * 16 + 16])
    allocated = []
    for recipe in specification['recipes']:
        block = int(recipe['block'], 16)
        assert recipe['collision'] == original['collision'][f'{block:02x}']
        if recipe['kind'] == 'native_copy':
            tileset = recipe['donor_tileset']
            native_block = int(recipe['donor_block'], 16)
            assert pack['data']['tilesets'][tileset]['collision'][f'{native_block:02x}'] == recipe['collision']
            parts = donor(tileset, native_block)
        elif recipe['kind'] == 'frost_flowers':
            parts = [(raw, frost) for raw, _ in donor(source, block)]
        else:
            assert recipe['kind'] == 'mixed_foliage'
            parts = []
            for glyph in old_metatiles[block * 16:block * 16 + 16]:
                if original['palette_map'][glyph] & 7 != 2:
                    parts.append(glyph)
                    continue
                slot = physical(glyph, original['palette_map'][glyph], count)
                raw = old_graphics[slot * 16:slot * 16 + 16]
                is_ground = glyph in ground_glyphs
                parts.append((remap(raw, [0, 1, 2, 3] if is_ground else [1, 2, 3, 3]),
                              frost if is_ground else canopy))
        assert len(parts) == 16
        for offset, part in enumerate(parts):
            if isinstance(part, int):
                metatiles[block * 16 + offset] = part
                continue
            raw, inks = part
            key = raw, inks
            if key not in shared:
                glyph = next(free, None)
                assert glyph is not None, 'native private alias exhausted safe blank glyph slots'
                graphics[glyph * 16:glyph * 16 + 16] = raw
                definition['palette_map'][glyph] = 3 | (8 if glyph >= 128 else 0)
                foregrounds[str(glyph)] = inks
                shared[key] = glyph
                allocated.append(glyph)
            metatiles[block * 16 + offset] = shared[key]
    for index in range(0, 32, 4):
        day[index] = [29, 31, 31]
    palette = '; Native day inks with snow background0.\n' + '\n'.join(
        'RGB ' + ', '.join(str(value) for color in day[index:index + 4] for value in color)
        for index in range(0, 32, 4)) + '\n'
    files[f'gfx/tilesets/{target}.2bpp'] = bytes(graphics)
    files[f'gfx/tilesets/{target}.png'] = tile_png(graphics)
    files[f'gfx/tilesets/{target}.pal'] = palette.encode()
    files[f'data/tilesets/{target}_metatiles.bin'] = bytes(metatiles)
    files[f'data/tilesets/{target}_foregrounds.json'] = json.dumps(foregrounds).encode()
    pack['data']['tilesets'][target] = definition
    assert definition['collision'] == original['collision'], 'snow must not change traversal'
    return {'source': source, 'target': target, 'allocated_glyphs': allocated,
            'original_block_ids_preserved': True, 'all_collision_quadrants_exact': True}
