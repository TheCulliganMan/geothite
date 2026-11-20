"""Verify the Game Corner and Cedar Inlet plaques in the archived native tilesets."""
from io import BytesIO
import json
from pathlib import Path

from PIL import Image, ImageChops

# Map, tileset, sign script, facing target, underlying metatile, local quadrant.
PLATES = [
    ("MnMinneapolis", "mn_minneapolis_city", "MnCornerSign", 55, 51, 0x83, 16, 16),
    ("MnR08C00R01", "mn_headwaters_thaw", "MN_R08_SIGN_02", 22, 6, 0x8a, 0, 0),
]
NEW = 0xb8


def sheet(raw, packed):
    """The PNG and 2bpp must contain the same four shade indices at every pixel."""
    frame = Image.open(BytesIO(bytes(raw))).convert("L")
    assert frame.size == (128, 128) and len(packed) == 4096
    indices = []
    for tile in range(256):
        tile_rows = []
        for y in range(8):
            lo, hi = packed[tile * 16 + y * 2:tile * 16 + y * 2 + 2]
            for x in range(8):
                shift = 7 - x
                index = ((lo >> shift) & 1) + 2 * ((hi >> shift) & 1)
                assert frame.getpixel((tile % 16 * 8 + x, tile // 16 * 8 + y)) == 255 - 85 * index
                tile_rows.append(index)
        indices.append(bytes(tile_rows))
    return indices


def check(parent, child, original_art, art, only):
    p, c = parent["data"], child["data"]
    old_files, new_files = parent["runtime_files"], child["runtime_files"]
    only(p["tilesets"], c["tilesets"], {plate[1] for plate in PLATES}, "plaque tilesets")
    changed_files = set()
    for name, tileset, label, x, y, block, left, top in PLATES:
        before, after = p["maps"][name], c["maps"][name]
        a = after["attributes"]
        assert a["tileset_name"] == tileset and (x % 2 * 16, y % 2 * 16) == (left, top)
        entry = dict(x=x, y=y, event_type="BGEVENT_READ", script=label)
        assert before["events"]["bg_events"].count(entry) == after["events"]["bg_events"].count(entry) == 1
        assert before["scripts"][label] == after["scripts"][label] == c["map_scripts"][label]
        assert label + "Text" in after["script_text_bodies"]
        local = old_files[f"data/tilesets/{tileset}_metatiles.bin"]
        updated = new_files[f"data/tilesets/{tileset}_metatiles.bin"]
        assert len(local) == NEW * 16 and updated[:len(local)] == local and len(updated) == len(local) + 16
        old_quad, new_quad = local[block*16:(block+1)*16], updated[-16:]
        base = top // 8 * 4 + left // 8
        allowed = {base, base + 1, base + 4, base + 5}
        changes = {i for i, (u, v) in enumerate(zip(old_quad, new_quad)) if u != v}
        assert changes and not changes - allowed
        graphic_ids = {new_quad[i] for i in changes}
        assert graphic_ids.isdisjoint(local), (tileset, "stamp uses a previously referenced tile")
        if name == "MnR08C00R01":
            clips = json.loads(bytes(old_files["data/tilesets/mn_headwaters_animations.json"]))
            assert graphic_ids.isdisjoint(map(int, clips)), "the cedar plaque overwrites an animated tile"
        collisions = c["tilesets"][tileset]["collision"]
        only(p["tilesets"][tileset]["collision"], collisions, {"b8"}, "collision slots")
        assert collisions["b8"] == collisions[f"{block:02x}"]
        assert json.loads(bytes(new_files[f"data/tilesets/{tileset}.json"])) == collisions
        for time in ("morn", "day", "nite"):
            src = original_art.blocks(tileset, time)
            dest = art.blocks(tileset, time)
            assert len(dest) == len(src) + 1
            assert all(ImageChops.difference(u, v).getbbox() is None for u, v in zip(src, dest)), (name, time)
            box = ImageChops.difference(src[block], dest[NEW]).getbbox()
            assert box and left <= box[0] < box[2] <= left + 16 and top <= box[1] < box[3] <= top + 16
        basic = {f"gfx/tilesets/{tileset}.2bpp", f"gfx/tilesets/{tileset}.png",
                 f"data/tilesets/{tileset}.json", f"data/tilesets/{tileset}_metatiles.bin"}
        changed_files.update(basic)
        png, tiles = f"gfx/tilesets/{tileset}.png", f"gfx/tilesets/{tileset}.2bpp"
        old_indices, new_indices = sheet(old_files[png], old_files[tiles]), sheet(new_files[png], new_files[tiles])
        assert {i for i, (u, v) in enumerate(zip(old_indices, new_indices)) if u != v} == graphic_ids
        changed_files.update(check_stamp(tileset, p, c, old_files, new_files, old_indices, new_indices,
                                         old_quad, new_quad, graphic_ids, only))
    only(old_files, new_files, changed_files, "plaque runtime art")


def check_stamp(tileset, old, new, before, after, original, current, old_quad, new_quad, graphic_ids, only):
    """Check every painted pixel of the new glyphs against the local paint stencil."""
    recipes = json.loads(Path(__file__).with_name("plaques.json").read_text())
    [recipe] = [r for r in recipes if r["tileset"] == tileset]
    cedar = tileset == "mn_headwaters_thaw"
    source_block = 0x8a if cedar else 0x83
    assert recipe["block"] == NEW and recipe["background"] == recipe["collision_from"] == source_block
    assert recipe["foreground"] is None
    rows = recipe["rows"]
    assert len(rows) == 32 and all(len(row) == 32 and set(row) <= set(".0123") for row in rows)
    stamps = [(4, 0xe8, 0xc8), (5, 0xe9, 0xc9)] if cedar else [(10, 0x8f, 0xf9), (11, 0x8f, 0xfa)]
    assert graphic_ids == {g for _, _, g in stamps}
    assert [(i, u, v) for i, (u, v) in enumerate(zip(old_quad, new_quad)) if u != v] == stamps
    native = old["tilesets"][tileset]
    patched = new["tilesets"][tileset]
    only(native, patched, {"collision", "palette_map"}, tileset)
    palette = list(native["palette_map"])
    for _, source, glyph in stamps:
        assert set(original[glyph]) == {0}, (tileset, "glyph was occupied", glyph)
        palette[glyph] = native["palette_map"][source]
    assert palette == patched["palette_map"]
    palette_file = f"data/tilesets/{tileset}_palette_map.json"
    assert json.loads(bytes(after[palette_file])) == palette
    paths = {palette_file}
    if cedar:
        fg = f"data/tilesets/{tileset}_foregrounds.json"
        before_colors, after_colors = (json.loads(bytes(files[fg])) for files in (before, after))
        assert after_colors == dict(before_colors, **{str(dest): before_colors[str(src)] for _, src, dest in stamps})
        paths.add(fg)
    changed = []
    drawn = {s[0] for s in stamps}
    for y, row in enumerate(rows):
        for x, entry in enumerate(row):
            tile = y//8*4 + x//8
            i = (y % 8)*8 + x % 8
            shade = original[old_quad[tile]][i]
            if entry != ".":
                assert tile in drawn
                if shade != int(entry):
                    changed.append((x, y))
                shade = int(entry)
            assert current[new_quad[tile]][i] == shade, (tileset, x, y)
    assert len(changed) == (51 if cedar else 61)
    xs, ys = zip(*changed)
    assert (min(xs), min(ys), max(xs), max(ys)) == ((2, 8, 12, 14) if cedar else (18, 17, 29, 23))
    return paths
