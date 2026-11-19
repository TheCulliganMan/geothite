#!/usr/bin/env python3
"""Audit the Fen–Northwoods seam and native graphics banks on actual parent/child archives."""

import argparse
import base64
import io
import json
import runpy
import sys
from pathlib import Path

from PIL import Image

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE.parents[1] / "tools"))
from render_reference import SourceArt, load_pack

FEN, WOODS = "MnLayeredFen", "MnBemidjiNorthwoods"
PINE = (0x1E, 0x1F, 0x2E, 0x2F, 0x3E, 0x3F)
NAMES = json.loads((HERE / "thaw.json").read_text())
Trail = runpy.run_path(str(HERE.parent / "northern-trails-v01/check-voyageurs.py"))[
    "Trail"
]


def require(good, message):
    if not good:
        raise ValueError(message)


def decoded_pixels(sheet, i):
    return [
        ((sheet[16 * i + 2 * y] >> (7 - x)) & 1)
        + 2 * ((sheet[16 * i + 2 * y + 1] >> (7 - x)) & 1)
        for y in range(8)
        for x in range(8)
    ]


def table(files, path):
    return json.loads(bytes(files.get(path, b"{}")))


def art(parent, child, edge):
    bank = edge["tileset"]
    old, new = (p["runtime_files"] for p in (parent, child))
    prefix = f"data/tilesets/{bank}"
    png_path = f"gfx/tilesets/{bank}.png"
    old_native, new_native = (bytes(f[f"gfx/tilesets/{bank}.2bpp"]) for f in (old, new))
    master = bytes(old["gfx/tilesets/johto.2bpp"])
    before, after = (p["data"]["tilesets"][bank] for p in (parent, child))
    fg0, fg1 = (table(f, prefix + "_foregrounds.json") for f in (old, new))
    bg0, bg1 = (table(f, prefix + "_native_backgrounds.json") for f in (old, new))
    changed = {}
    authored_colors = {}
    for crown in edge["crowns"]:
        for j, src in enumerate(PINE):
            k = crown["first"] + j
            changed[k] = (2, decoded_pixels(master, src))
            if "foregrounds5" in crown:
                authored_colors[str(k)] = [
                    [c << 3 | c >> 2 for c in rgb] for rgb in crown["foregrounds5"]
                ]
    snow_ground = set()
    for track in edge["track"]:
        slot = track["slot"]
        if "melt_span" in track:
            gravel = decoded_pixels(master, 6)
            require(set(gravel) == {0, 1}, "unexpected gravel source")
            widths = track["melt_span"]
            ink = [
                2 if mark else int(i % 8 < widths[i // 8])
                for i, mark in enumerate(gravel)
            ]
            authored_colors[str(slot)] = [
                [222, 255, 222],
                [173, 173, 173],
                [107, 107, 107],
            ]
            snow_ground.add(slot)
        elif "source" in track:
            ink = decoded_pixels(master, track["source"])
        else:
            ink = [int(v) for row in track["rows"] for v in row]
        changed[slot] = (track["palette"], ink)
    require(
        set(changed).isdisjoint(map(int, bg0))
        and set(changed).isdisjoint(map(int, fg0)),
        "replaced preexisting palette overrides",
    )
    require(
        bg1 == {**bg0, **{str(k): 0 for k in changed if k not in snow_ground}},
        "clock-following ground colors drifted",
    )
    require(fg1 == {**fg0, **authored_colors}, "wrong adjoining pine palette")
    require(
        table(new, prefix + "_palette_map.json") == after["palette_map"]
        and table(new, prefix + ".json") == after["collision"],
        "raw and typed palettes or permissions disagree",
    )
    require(
        len(before["palette_map"]) == len(after["palette_map"]), "old palette extent"
    )
    require(len(old_native) == len(new_native) == 4096, "native tile pages")
    png = Image.open(io.BytesIO(bytes(new[png_path]))).convert("L")
    require(png.size == (128, 128), "invalid native PNG")
    for i in range(256):
        pixels = decoded_pixels(new_native, i)
        expect = [255 - v * 85 for v in pixels]
        x, y = (i % 16) * 8, (i // 16) * 8
        require(
            list(png.crop((x, y, x + 8, y + 8)).getdata()) == expect,
            f"2bpp/PNG mismatch {bank}/{i:02x}",
        )
        if i in changed:
            palette, native = changed[i]
            require(
                pixels == native and after["palette_map"][i] == palette,
                f"wrong drawn tile {bank}/{i:02x}",
            )
        else:
            require(
                old_native[16 * i : 16 * (i + 1)] == new_native[16 * i : 16 * (i + 1)]
                and (
                    i >= len(before["palette_map"])
                    or before["palette_map"][i] == after["palette_map"][i]
                ),
                f"unrelated glyph mutation {bank}/{i:02x}",
            )
    for k in [prefix + "_animations.json", "data/tilesets/johto_animations.json"]:
        require(old.get(k) == new.get(k), "changed snow, flower or water animation")
        require(
            not set(changed).intersection(map(int, table(new, k))),
            "overwrote animated graphics",
        )
    old_atlas, new_atlas = (bytes(f[prefix + "_metatiles.bin"]) for f in (old, new))
    require(
        not set(old_atlas).intersection(changed), "old terrain draws a thaw-only glyph"
    )
    require(
        new_atlas.startswith(old_atlas)
        and all(after["collision"][k] == v for k, v in before["collision"].items()),
        "original art or movement block overwritten",
    )
    if bank == "mn_winter_bemidji_northwoods":
        room = child["data"]["maps"][WOODS]
        road_id = room["blocks"][8 * room["attributes"]["width"]]
        road = new_atlas[16 * road_id : 16 * (road_id + 1)]
        gravel = decoded_pixels(master, 6)
        edges = []
        for y in range(32):
            bare, snow = [], []
            for x in range(32):
                glyph = road[y // 8 * 4 + x // 8]
                dot = y % 8 * 8 + x % 8
                if gravel[dot]:
                    continue
                if glyph in snow_ground:
                    wet = decoded_pixels(new_native, glyph)[dot] == 1
                else:
                    require(glyph in (6, 0x6F), "unknown surface on the walking lanes")
                    wet = glyph == 0x6F
                (bare if wet else snow).append(x)
            require(
                bare and snow and max(bare) < min(snow),
                "broken shoreline of the melting snow",
            )
            edges.append(max(bare) + 1)
        require(
            edges[0] <= 8 and max(edges[15:25]) >= 21 and edges[-1] <= 14,
            "hard straight snow seam across the footpath",
        )
    return new_atlas


def camera_palette_budget(pack):
    # Recoloring a pine must still fit a 160 × 144 GBC background. Verify the
    # actual native pigments visible while crossing both road lanes at daylight.
    art = SourceArt(pack)
    scene = Image.new("RGB", (896, 416))
    for name, x in ((FEN, 0), (WOODS, 640)):
        room = pack["data"]["maps"][name]
        a = room["attributes"]
        scene.paste(
            art.render(
                a["tileset_name"], a["width"], a["height"], room["blocks"], "day"
            ),
            (x, 0),
        )
    worst = 0
    for tile_y in (16, 17):
        for tile_x in range(34, 45):
            x0, y0 = tile_x * 16 + 8 - 80, tile_y * 16 + 8 - 72
            sets = set()
            for y in range(y0, y0 + 144, 8):
                for x in range(x0, x0 + 160, 8):
                    colors = scene.crop((x, y, x + 8, y + 8)).getcolors(64)
                    inks = frozenset(rgb for _, rgb in colors)
                    require(len(inks) <= 4, "native background glyph exceeds four inks")
                    sets.add(inks)
            full = sorted(
                (s for s in sets if not any(s < other for other in sets)),
                key=lambda s: (-len(s), sorted(s)),
            )
            palettes = []
            for inks in full:
                for i, palette in enumerate(palettes):
                    if len(inks | palette) <= 4:
                        palettes[i] = inks | palette
                        break
                else:
                    palettes.append(inks)
            require(
                len(palettes) <= 8,
                f"{tile_x},{tile_y}: scene exceeds eight four-color background palettes",
            )
            worst = max(worst, len(palettes))
    return worst


def check(parent, child):
    previous, current = (p["data"] for p in (parent, child))
    terrain = []
    for edge in NAMES:
        name, bank = edge["map"], edge["tileset"]
        old, now = (d["maps"][name] for d in (previous, current))
        require(
            old["attributes"] == now["attributes"],
            "changed fishing, doors or world connections",
        )
        old_habitat, new_habitat = (
            d["wild_encounters"].get(name) for d in (previous, current)
        )
        # The separate water-life revision populates the two pools. Every dry
        # encounter remains the same on both sides of this woodland connection.
        if name == FEN:
            require(
                {
                    k: v
                    for k, v in old_habitat.items()
                    if k not in ("water", "water_rate")
                }
                == {
                    k: v
                    for k, v in new_habitat.items()
                    if k not in ("water", "water_rate")
                },
                "changed dry habitat",
            )
        else:
            require(old_habitat == new_habitat, "changed Northwoods habitat")
        require(
            old["events"]["warps"] == now["events"]["warps"], "changed travel records"
        )
        atlas = art(parent, child, edge)
        mirror = base64.b64decode(
            current["map_blocks"][old["attributes"]["blocks_label"]]
        )
        require(mirror == bytes(now["blocks"]), "native and split map disagree")
        w = now["attributes"]["width"]
        sites = set()
        for patch in edge["terrain"]:
            for x, y in patch["at"]:
                i = y * w + x
                old_id, new_id = (r["blocks"][i] for r in (old, now))
                require(
                    old_id == patch["from_block"]
                    and list(atlas[new_id * 16 : (new_id + 1) * 16]) == patch["tiles"],
                    f"incorrect terrain on {name} at ({x},{y})",
                )
                roles = previous["tilesets"][bank]["collision"][f"{old_id:02x}"]
                require(
                    current["tilesets"][bank]["collision"][f"{new_id:02x}"] == roles,
                    f"wrong travel permission on {name} at ({x},{y})",
                )
                sites.add(i)
        require(
            len(sites) == sum(len(q["at"]) for q in edge["terrain"]),
            "overlapping tree footprints",
        )
        terrain.append(len(sites))
    before, after = (Trail(d, [FEN, WOODS]) for d in (previous, current))
    for y in (16, 17):
        for direction, origin, destination in [
            ("RIGHT", (FEN, 39, y), (WOODS, 0, y)),
            ("LEFT", (WOODS, 0, y), (FEN, 39, y)),
        ]:
            require(
                after.step(origin, direction, dry=True) == destination,
                "broken eastern crossing",
            )
    sign = (WOODS, 3, 16)
    for source in [(FEN, 35, 16), (FEN, 38, 17), (WOODS, 7, 16)]:
        require(
            before.both(source, sign, dry=True) == after.both(source, sign, dry=True),
            "changed road to the sign",
        )
    a, b = (d["maps"][WOODS] for d in (previous, current))
    require(
        a["blocks"][7 * 8 + 1] == b["blocks"][7 * 8 + 1] == 0x78
        and a["events"] == b["events"]
        and a["scripts"]["MnFenTurnoff"] == b["scripts"]["MnFenTurnoff"],
        "changed turnoff board, text trigger or approach",
    )
    print(
        f"22 daylight walking cameras fit at most {camera_palette_budget(child)} four-color native background palettes."
    )
    print(
        f"{terrain[0]} Fen and {terrain[1]} Northwoods blocks joined on both walking lanes. "
        "Every original 2bpp tile, native animation, collision quadrant and the road to the existing sign pass. "
        "New pine pixels match the original silhouette; all four-shade palettes and both 2bpp/PNG mirrors agree."
    )


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    for key in ("parent", "candidate"):
        parser.add_argument(key, type=Path)
    args = parser.parse_args()
    check(load_pack(args.parent), load_pack(args.candidate))
