#!/usr/bin/env python3
"""Check the installed fen drawings, original Johto tiles and inherited animation targets."""

import argparse
import base64
import io
import json
import sys
from pathlib import Path

from PIL import Image

SOURCE = Path(__file__).resolve().parent
sys.path.insert(0, str(SOURCE.parents[1] / "tools"))
from render_reference import SourceArt, load_pack

MAP = "MnLayeredFen"
JOHTO = "johto"
ATLAS = "data/tilesets/johto_metatiles.bin"
SHEET = "gfx/tilesets/johto.2bpp"
BACKGROUNDS = "data/tilesets/johto_native_backgrounds.json"
ANIMATED = (0x03, 0x14)
# These banks can inherit johto_animations.json via winter_render_behavior.
ANCESTORS = {
    JOHTO: (),
    "johto_glacier_rail": (JOHTO,),
    "mn_outdoor_johto_stairs": (JOHTO,),
    "mn_r10_rockstairs": (JOHTO,),
    "mn_winter_bemidji_northwoods": (JOHTO,),
    "mn_winter_iron_descent": ("mn_r10_rockstairs", JOHTO),
    "mn_iron_descent_thaw": ("mn_r10_rockstairs", JOHTO),
    # These inherit native water and flowers, but don't inherit Johto's added clips.
    "mn_northwoods_props": (),
    "mn_marais_shore": (),
    "mn_campfire": (),
}


def require(condition, what):
    if not condition:
        raise ValueError(what)


def glyph_rows(graphics, tile_id, palette):
    count = len(graphics) // 16
    physical = (tile_id & 127) + count // 2 if palette & 8 else tile_id
    if physical >= count and tile_id >= 128:
        physical = tile_id - 128
    offset = physical * 16
    require(offset + 16 <= len(graphics), "glyph outside native sheet")
    tile = graphics[offset : offset + 16]
    return [
        "".join(
            str((tile[2 * y] >> x & 1) + 2 * (tile[2 * y + 1] >> x & 1))
            for x in reversed(range(8))
        )
        for y in range(8)
    ]


def check_animations(old, new):
    of, nf = old["runtime_files"], new["runtime_files"]
    for file in ("water/water.2bpp", "flower/cgb_1.2bpp", "flower/cgb_2.2bpp"):
        key = "gfx/tilesets/" + file
        require(of[key] == nf[key], f"changed original animation {file}")
    seen = 0
    for name, parents in ANCESTORS.items():
        if name not in old["data"]["tilesets"]:
            continue
        bank = old["data"]["tilesets"][name]
        newbank = new["data"]["tilesets"][name]
        atlas = bytes(of[f"data/tilesets/{name}_metatiles.bin"])
        active = set(atlas)
        before = bank["palette_map"]
        after = newbank["palette_map"]
        for g in ANIMATED:
            require(before[g] == after[g], name + " built-in animation palette")
        if name != JOHTO:
            for key in (
                f"gfx/tilesets/{name}.png",
                f"gfx/tilesets/{name}.2bpp",
                f"data/tilesets/{name}_metatiles.bin",
            ):
                require(of[key] == nf[key], name + " original inherited art")
        # New Pepin frames may be inherited, but must never replace existing scenery.
        paths = [f"data/tilesets/{bank}_animations.json" for bank in (name, *parents)]
        prior, current = [
            next((json.loads(bytes(files[k])) for k in paths if k in files), {})
            for files in (of, nf)
        ]
        require(
            all(current.get(k) == v for k, v in prior.items()),
            f"{name} inherited animations changed",
        )
        for new_id in current.keys() - prior.keys():
            require(
                int(new_id) < len(after) and int(new_id) not in active,
                name + " animated preexisting scenery",
            )
        seen += sum(
            m["attributes"]["tileset_name"] == name
            for m in old["data"]["maps"].values()
        )
    return seen


def check(old, new):
    of, nf = old["runtime_files"], new["runtime_files"]
    before, after = (p["data"]["maps"][MAP] for p in (old, new))
    bank_a, bank_b = (p["data"]["tilesets"][JOHTO] for p in (old, new))
    coarse, full = bytes(of[SHEET]), bytes(nf[SHEET])
    old_atlas, new_atlas = bytes(of[ATLAS]), bytes(nf[ATLAS])
    require(
        len(coarse) == 192 * 16 and len(full) == 256 * 16, "original two Johto pages"
    )
    require(new_atlas.startswith(old_atlas), "existing Johto block changed")
    require(
        before["attributes"] == after["attributes"], "fen region, borders or map size"
    )
    require(
        base64.b64decode(new["data"]["map_blocks"][after["attributes"]["blocks_label"]])
        == bytes(after["blocks"]),
        "native map block mirror",
    )
    a_palette, b_palette = bank_a["palette_map"], bank_b["palette_map"]
    for g in set(old_atlas):
        require(
            a_palette[g] == b_palette[g]
            and glyph_rows(coarse, g, a_palette[g])
            == glyph_rows(full, g, b_palette[g]),
            f"old native Johto glyph {g:02x} changed",
        )
    for key, value in bank_a["collision"].items():
        require(bank_b["collision"].get(key) == value, f"old Johto permission {key}")
    imgs = [SourceArt(p) for p in (old, new)]
    for group in ("morn", "day", "nite"):
        aa, bb = [im.blocks(JOHTO, group) for im in imgs]
        require(
            all(a.tobytes() == b.tobytes() for a, b in zip(aa, bb)),
            f"{group} original rendered Johto blocks",
        )
    drawings = json.loads((SOURCE / "landmarks.json").read_text())
    backgrounds = json.loads(bytes(nf[BACKGROUNDS]))
    png = Image.open(io.BytesIO(bytes(nf["gfx/tilesets/johto.png"]))).convert("L")
    expected_map = list(before["blocks"])
    width = before["attributes"]["width"]
    new_glyphs = set()
    for d in drawings:
        name = d["name"]
        x, y = d["block"]
        i = y * width + x
        src, dst = before["blocks"][i], after["blocks"][i]
        require(
            src == d["from"] and dst * 16 >= len(old_atlas), name + " block identity"
        )
        expected_map[i] = dst
        require(
            bank_a["collision"][f"{src:02x}"] == bank_b["collision"][f"{dst:02x}"],
            name + " collision",
        )
        old_tiles = old_atlas[src * 16 : src * 16 + 16]
        tiles = new_atlas[dst * 16 : dst * 16 + 16]
        expected_tiles = list(old_tiles)
        for patch in d["patches"]:
            rows = patch["rows"]
            px, py = patch["at"]
            for gy in range(len(rows) // 8):
                for gx in range(len(rows[0]) // 8):
                    pos = (py + gy) * 4 + px + gx
                    g = tiles[pos]
                    new_glyphs.add(g)
                    donor = old_tiles[pos]
                    require(
                        donor not in ANIMATED and g not in old_atlas,
                        name + " original or animated tile overwritten",
                    )
                    expected_tiles[pos] = g
                    require(
                        b_palette[g] == patch["palette"]
                        and backgrounds.get(str(g)) == patch.get("background"),
                        name + " clock palette",
                    )
                    old_rows = glyph_rows(coarse, donor, a_palette[donor])
                    got = glyph_rows(full, g, b_palette[g])
                    for line in range(8):
                        source = rows[gy * 8 + line][gx * 8 : gx * 8 + 8]
                        require(
                            got[line]
                            == "".join(
                                old_rows[line][j] if v == "." else v
                                for j, v in enumerate(source)
                            ),
                            name + " native two-bit drawing",
                        )
                        for col, tone in enumerate(got[line]):
                            require(
                                png.getpixel((g % 16 * 8 + col, g // 16 * 8 + line))
                                == 255 - 85 * int(tone),
                                name + " graphical PNG/2bpp mirror",
                            )
        require(bytes(expected_tiles) == tiles, name + " changed tiles outside drawing")
    require(
        after["blocks"] == expected_map,
        "unexpected fen terrain outside three landmarks",
    )
    require(
        old["data"]["field_moves"] == new["data"]["field_moves"],
        "field move rules changed",
    )
    terrain_tiles = {
        g for b in after["blocks"] for g in new_atlas[16 * b : 16 * b + 16]
    }
    palettes = {
        (b_palette[g] & 7, backgrounds.get(str(g), b_palette[g] & 7))
        for g in terrain_tiles
    }
    require(len(palettes) <= 8, "fen terrain exceeds eight four-shade GBC palettes")
    rooms = check_animations(old, new)
    print(
        f"{len(drawings)} targets, {len(new_glyphs)} original native tiles, all {len(old_atlas) // 16} old Johto "
        f"blocks at morning/day/night, {len(palettes)} GBC palettes, original permissions and inherited animation lookups across {rooms} rooms passed."
    )


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("parent", type=Path)
    parser.add_argument("candidate", type=Path)
    args = parser.parse_args()
    check(load_pack(args.parent), load_pack(args.candidate))
