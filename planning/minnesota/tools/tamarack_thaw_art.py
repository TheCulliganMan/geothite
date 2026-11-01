"""Read-only native-art composition for the explicit Tamarack snowline.

This mutates only the decoded review view. Native publication remains Rust.
Original logical glyphs keep native background palette addresses; appended cold
glyphs keep snow background0 without changing grass's original zero-index holes.
"""
from copy import deepcopy
import json

from inland_winter_preview_art import apply_alias, physical, remap, tile_png


def compose(pack, specification):
    full = specification['target']
    partial = specification.get('partial_target', 'mn_tamarack_thaw')
    source = specification['source']
    files = pack['runtime_files']
    original = pack['data']['tilesets'][source]
    original_layout = bytes(files[f'data/tilesets/{source}_metatiles.bin'])
    original_fg = json.loads(bytes(files.get(f'data/tilesets/{source}_foregrounds.json', b'{}')))
    original_animations = json.loads(bytes(files.get(f'data/tilesets/{source}_animations.json', b'{}')))
    animated = {3, 0x14} | {int(glyph) for glyph in original_animations}
    full_receipt = apply_alias(pack, specification)
    # Moving water and flowers keep their original native zero color and clock.
    full_backgrounds = {str(glyph): original['palette_map'][glyph] & 7 for glyph in animated}
    files[f'data/tilesets/{full}_native_backgrounds.json'] = json.dumps(full_backgrounds).encode()
    definition = deepcopy(pack['data']['tilesets'][full])
    graphics = bytearray(files[f'gfx/tilesets/{full}.2bpp'])
    full_layout = bytes(files[f'data/tilesets/{full}_metatiles.bin'])
    foregrounds = json.loads(bytes(files[f'data/tilesets/{full}_foregrounds.json']))
    backgrounds = {str(glyph): 0 if str(glyph) in original_fg else original['palette_map'][glyph] & 7
                   for glyph in set(original_layout) | animated}
    layout = bytearray(original_layout)
    used = set(original_layout) | set(full_layout) | animated
    protected = {physical(glyph, definition['palette_map'][glyph], 256) for glyph in used}
    preferred = list(range(224, 256)) + list(range(96, 128))
    candidates = preferred + [index for index in range(256) if index not in preferred]
    free = iter(slot for slot in candidates if slot not in used and slot not in protected
                and str(slot) not in foregrounds and str(slot) not in backgrounds
                and not any(graphics[slot * 16:slot * 16 + 16]))
    duplicated, shared = {}, {}

    def cold(glyph):
        if glyph in animated or str(glyph) not in backgrounds:
            return glyph
        if glyph in duplicated:
            return duplicated[glyph]
        palette = definition['palette_map'][glyph]
        slot = physical(glyph, palette, 256)
        raw = bytes(graphics[slot * 16:slot * 16 + 16])
        inks = foregrounds.get(str(glyph))
        key = raw, palette & 7, json.dumps(inks)
        if key not in shared:
            target = next(free, None)
            assert target is not None, 'mixed snowline exhausted safe native glyph slots'
            graphics[target * 16:target * 16 + 16] = raw
            definition['palette_map'][target] = (palette & 7) | (8 if target >= 128 else 0)
            if inks is not None:
                foregrounds[str(target)] = deepcopy(inks)
            shared[key] = target
        duplicated[glyph] = shared[key]
        return duplicated[glyph]

    variants = {}
    source_blocks = sorted({int(recipe['block'], 16) for recipe in specification['recipes']} | {1})
    # Include every placed cold block, even when only its native background changes.
    source_blocks = sorted(set(source_blocks) | set(specification['placed_blocks']))
    for block in source_blocks:
        target = len(layout) // 16
        assert target < 256
        variants[block] = target
        layout.extend(cold(glyph) for glyph in full_layout[block * 16:block * 16 + 16])
        definition['collision'][f'{target:02x}'] = list(original['collision'][f'{block:02x}'])
    melt_variants, melted = {}, {}
    if specification.get('include_melt_variants', False):
        # A broad frost-on-green band separates full snow from native green.
        # Grass bitplanes are never remapped: its priority holes stay native.
        grass_glyphs = {
            full_layout[block * 16 + index]
            for block in specification['placed_blocks']
            for index in range(16)
            if original['collision'][f'{block:02x}'][(index // 8) * 2 + (index % 4) // 2] == 'TALL_GRASS'
        }
        reverse = {target: original for original, target in duplicated.items()}

        def melt(glyph):
            if str(glyph) in backgrounds:
                return glyph
            if glyph in reverse:
                return reverse[glyph]
            if glyph in melted:
                return melted[glyph]
            palette = definition['palette_map'][glyph]
            slot = physical(glyph, palette, 256)
            raw = bytes(graphics[slot * 16:slot * 16 + 16])
            inks = foregrounds.get(str(glyph))
            white = (min(inks[0]) >= specification['melt_white_ink_min']
                     if inks and 'melt_white_ink_min' in specification
                     else bool(inks and list(inks[0]) == [239, 255, 255]))
            if glyph not in grass_glyphs and white:
                raw = remap(raw, [0, 0, 2, 3])
            target = next(free, None)
            assert target is not None, 'melt band exhausted safe native glyph slots'
            graphics[target * 16:target * 16 + 16] = raw
            definition['palette_map'][target] = (palette & 7) | (8 if target >= 128 else 0)
            if inks is not None:
                foregrounds[str(target)] = deepcopy(inks)
            backgrounds[str(target)] = 0
            melted[glyph] = target
            return target

        for block in source_blocks:
            target = len(layout) // 16
            assert target < 256
            melt_variants[block] = target
            cold_id = variants[block]
            layout.extend(melt(glyph) for glyph in bytes(layout[cold_id * 16:(cold_id + 1) * 16]))
            definition['collision'][f'{target:02x}'] = list(original['collision'][f'{block:02x}'])
    assert layout[:len(original_layout)] == original_layout
    assert all(definition['palette_map'][glyph] == original['palette_map'][glyph]
               for glyph in set(original_layout) | animated)
    pack['data']['tilesets'][partial] = definition
    files[f'gfx/tilesets/{partial}.2bpp'] = bytes(graphics)
    files[f'gfx/tilesets/{partial}.png'] = tile_png(graphics)
    files[f'gfx/tilesets/{partial}.pal'] = files[f'gfx/tilesets/{full}.pal']
    files[f'data/tilesets/{partial}_metatiles.bin'] = bytes(layout)
    files[f'data/tilesets/{partial}_foregrounds.json'] = json.dumps(foregrounds).encode()
    files[f'data/tilesets/{partial}_native_backgrounds.json'] = json.dumps(backgrounds).encode()
    result = {'full_alias': full_receipt, 'partial_alias': partial,
                      'duplicated_static_glyphs': duplicated,
                      'allocated_partial_glyphs': sorted(set(duplicated.values())),
                      'cold_variants': {f'{key:02x}': f'{value:02x}' for key, value in variants.items()},
                      'native_background_addresses': len(backgrounds),
                      'animated_native_glyphs_retained': sorted(animated)}
    if specification.get('include_melt_variants', False):
        result.update(melt_variants={f'{key:02x}': f'{value:02x}' for key, value in melt_variants.items()},
                      allocated_melt_glyphs=sorted(melted.values()), melted_glyphs=melted,
                      melt_static_sources=reverse)
    return variants, result
