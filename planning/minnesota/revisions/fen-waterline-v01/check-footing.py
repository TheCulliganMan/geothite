#!/usr/bin/env python3
"""Check the archived Fen's shoreline against the native walkers, canoes and water animation."""

import argparse
import base64
import io
import json
import runpy
import sys
from collections import Counter
from functools import cache
from pathlib import Path

from PIL import Image

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE.parents[1] / "tools"))
from audit_regional_design import Terrain
from render_reference import SourceArt, load_pack

sprite_support = runpy.run_path(str(HERE.parent / "shore-watch-v01/check-route.py"))["sprite_support"]
FEN = "MnLayeredFen"
DIRS = ((0, -1), (0, 1), (-1, 0), (1, 0))
ATLAS = "data/tilesets/johto_metatiles.bin"
COLLISIONS = "data/tilesets/johto.json"
PALETTES = "data/tilesets/johto_palette_map.json"
SHEET = "gfx/tilesets/johto.2bpp"
PNG = "gfx/tilesets/johto.png"
WATER = "gfx/tilesets/water/water.2bpp"
RIPPLE = 0x14


def changed(a, b):
    return {k for k in a.keys() | b.keys() if a.get(k) != b.get(k)}


def screen(pack, terrain, clock):
    return SourceArt(pack).render(
        terrain.tileset, terrain.width, terrain.map["attributes"]["height"], terrain.blocks, clock
    )


def installed_graphics(pack):
    files = pack["runtime_files"]
    data = pack["data"]["tilesets"]["johto"]
    assert json.loads(bytes(files[COLLISIONS])) == data["collision"]
    assert json.loads(bytes(files[PALETTES])) == data["palette_map"]
    sheet = bytes(files[SHEET])
    png = Image.open(io.BytesIO(bytes(files[PNG]))).convert("L")
    assert png.size == (128, 128) and len(sheet) == 4096
    for y in range(png.height):
        for x in range(png.width):
            offset = ((y // 8) * 16 + x // 8) * 16 + (y % 8) * 2
            bit = 7 - x % 8
            ink = (sheet[offset] >> bit & 1) | ((sheet[offset + 1] >> bit & 1) << 1)
            assert png.getpixel((x, y)) == 255 - ink * 85, ("PNG and native ink differ", x, y)


def preservation(parent, child):
    assert changed(parent, child) == {"identity", "report", "data", "runtime_files"}
    assert changed(parent["identity"], child["identity"]) == {"runtime_modpack_id", "content_hash"}
    assert changed(parent["report"], child["report"]) == {"manifests"}
    assert child["report"]["manifests"] == parent["report"]["manifests"] + ["northstar-fen-waterline-v1"]
    old, new = parent["data"], child["data"]
    assert changed(old, new) == {"map_blocks", "maps", "tilesets"}
    assert changed(old["maps"], new["maps"]) == {FEN}
    assert changed(old["maps"][FEN], new["maps"][FEN]) == {"blocks"}
    label = new["maps"][FEN]["attributes"]["blocks_label"]
    assert changed(old["map_blocks"], new["map_blocks"]) == {label}
    assert base64.b64decode(new["map_blocks"][label]) == bytes(new["maps"][FEN]["blocks"])
    assert changed(old["tilesets"], new["tilesets"]) == {"johto"}
    files = changed(parent["runtime_files"], child["runtime_files"])
    assert {ATLAS, COLLISIONS} <= files <= {ATLAS, COLLISIONS, PALETTES, SHEET, PNG}, files
    original = bytes(parent["runtime_files"][ATLAS])
    extended = bytes(child["runtime_files"][ATLAS])
    assert len(extended) <= 256 * 16 and extended.startswith(original)
    assert len(extended) % 16 == 0
    old_bank, bank = (p["data"]["tilesets"]["johto"]["collision"] for p in (parent, child))
    assert len(bank) == len(extended) // 16 and bank | old_bank == bank
    assert set(range(len(original) // 16, len(extended) // 16)) <= set(new["maps"][FEN]["blocks"])
    old_palette, palette = (p["data"]["tilesets"]["johto"]["palette_map"] for p in (parent, child))
    assert palette[: len(old_palette)] == old_palette
    old_sheet, sheet = (bytes(p["runtime_files"][SHEET]) for p in (parent, child))
    added_glyphs = set(extended[len(original) :]) - set(original)
    for index in range(256):
        start = index * 16
        a, b = old_sheet[start : start + 16], sheet[start : start + 16]
        assert a == b or (index in added_glyphs and not any(a)), ("overwrote native art", index)
    installed_graphics(child)
    before, after = Terrain(parent, FEN), Terrain(child, FEN)
    assert before.tiles == after.tiles and before.reach([(39, 16)]) == after.reach([(39, 16)])
    lakes = {p for p, role in before.tiles.items() if role == "WATER"}
    shore = {(x + dx, y + dy) for x, y in lakes for dx in (-1, 0, 1) for dy in (-1, 0, 1)}
    # No earlier Johto block may look different anywhere that already uses it.
    # Within the Fen even newly selected tiles must preserve the woodland and clues away from the shore.
    art = SourceArt(parent), SourceArt(child)
    for clock in ("morn", "day", "nite"):
        a, b = (renderer.blocks("johto", clock) for renderer in art)
        assert all(block.tobytes() == b[i].tobytes() for i, block in enumerate(a))
        for (x, y), kind in before.tiles.items():
            i, j = x % 2 * 16, y % 2 * 16
            prior, current = a[before.block(x, y)], b[after.block(x, y)]
            if (x, y) not in shore:
                box = (i, j, i + 16, j + 16)
                assert prior.crop(box).tobytes() == current.crop(box).tobytes(), (
                    "unrelated terrain pixels",
                    clock,
                    x,
                    y,
                )
            if kind == "TALL_GRASS":
                # A gravel lip can overlap the patch, but encounters must still have visibly tall grass.
                patches = [(u, v, u + 8, v + 8) for u in (i, i + 8) for v in (j, j + 8)]
                assert any(prior.crop(p).tobytes() == current.crop(p).tobytes() for p in patches), (
                    "lost native encounter grass",
                    clock,
                    x,
                    y,
                )
    return len(new["maps"])


def check_feet(pack):
    terrain = Terrain(pack, FEN)
    assert terrain.tileset == "johto"
    lake = {p for p, role in terrain.tiles.items() if role == "WATER"}
    dry = terrain.reach([(39, 16)])
    assert len(lake) == 205 and len(dry) == 274 and (17, 5) in lake
    image = screen(pack, terrain, "day")
    atlas = bytes(pack["runtime_files"][ATLAS])
    ripple = bytes(pack["runtime_files"][WATER])
    frames = [ripple[i : i + 16] for i in range(0, len(ripple), 16)]
    assert len(frames) == 4 and frames[1] == frames[3] and len(set(frames)) == 3
    for y in range(8):
        assert all(
            any((ripple[f + 2 * y] | ripple[f + 2 * y + 1]) & (1 << x) for f in (0, 16, 32, 48))
            for x in range(8)
        )
    water_colors = {
        rgb
        for _, rgb in image.crop((24 * 16, 19 * 16, 25 * 16, 20 * 16)).getcolors()
        if rgb[2] > rgb[1] and rgb[2] > rgb[0]
    }
    assert water_colors, "the canoe channel contains no visible blue"

    @cache
    def glyph(px, py):
        cell = terrain.block(px // 16, py // 16)
        return atlas[16 * cell + 4 * ((py // 8) % 4) + (px // 8) % 4]

    @cache
    def wet(px, py):
        # Foam is white for part of the cycle. The actual animated source identifies those pixels.
        return glyph(px, py) == RIPPLE or image.getpixel((px, py)) in water_colors

    feet = sprite_support(pack, ["CHRIS", "KRIS"])
    keel = sprite_support(pack, ["MN_CANOE_CHRIS", "MN_CANOE_KRIS"])
    errors = Counter()
    examples = {}

    def expect(ok, label, where):
        if not ok:
            errors[label] += 1
            examples.setdefault(label, where)

    rock = {(198, 148, 57), (165, 123, 24)}  # Sunlit and shaded native Johto gravel in daylight.

    def stones(x, y):
        return any(
            rgb in rock for _, rgb in image.crop((16 * x, 16 * y, 16 * x + 16, 16 * y + 16)).getcolors()
        )

    approaches = Counter()
    banks = set()
    strides = 0
    for x, y in sorted(lake | dry):
        afloat = (x, y) in lake
        support = keel if afloat else feet
        expect(
            all(wet(16 * x + i, 16 * y + j) == afloat for i, j in support), "parked soles or keel", (x, y)
        )
        if afloat:
            # Include the one-square inlet tip as well as the stepping squares in the strait.
            expect(
                all(glyph(16 * x + i, 16 * y + j) == RIPPLE for i in (0, 8) for j in (0, 8)),
                "missing native ripple",
                (x, y),
            )
        for dx, dy in DIRS:
            target = x + dx, y + dy
            if not afloat and target in lake:
                approaches[y < 13, dx, dy] += 1
                if stones(x, y):
                    banks.add((y < 13, dx, dy))
                # Departure and return cross the identical pixel. The runtime owns the new pose
                # as soon as the bottom center of the sprite enters the destination square.
                for t in range(17):
                    px, py = 16 * x + 8 + dx * t, 16 * y + 15 + dy * t
                    expect(
                        wet(px, py) == ((px // 16, py // 16) == target),
                        "launch/landing",
                        ((x, y), target, t),
                    )
            elif target in (lake if afloat else dry) and (dx, dy) in ((0, 1), (1, 0)):
                strides += 1
                for t in range(1, 16):
                    expect(
                        all(wet(16 * x + i + t * dx, 16 * y + j + t * dy) == afloat for i, j in support),
                        "walking/paddling stride",
                        ((x, y), target, t),
                    )
    # The north side of the upper inlet is backed by unwalkable firs.
    assert sum(n for (north, _, _), n in approaches.items() if north) == 17
    assert sum(n for (north, _, _), n in approaches.items() if not north) == 68
    expect(banks == approaches.keys(), "missing brown stone on reachable shores", approaches.keys() - banks)
    return sum(approaches.values()), strides, errors, examples


if __name__ == "__main__":
    cli = argparse.ArgumentParser(description=__doc__)
    cli.add_argument("parent", type=Path)
    cli.add_argument("child", type=Path, nargs="?", help="omit to measure the historical footing failures")
    args = cli.parse_args()
    source = load_pack(args.parent)
    pack = load_pack(args.child) if args.child else source
    if args.child:
        print(
            f"Verified all {preservation(source, pack)} maps, archived content, collision, native mirrors and routes."
        )
    crossings, strides, errors, examples = check_feet(pack)
    for label, count in errors.items():
        print(f"{label}: {count} mismatches; first {examples[label]}")
    if args.child:
        assert not errors, "shoreline footing regression"
    print(f"205 water squares, {crossings} approach pairs, and {strides} same-surface strides checked.")
