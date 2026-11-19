#!/usr/bin/env python3
"""Check Minnesota exteriors for the detached gatehouse roof once used as Moorhead's garden."""

import argparse
from functools import cache
from pathlib import Path
import sys

sys.path.insert(0, str(Path(__file__).resolve().parents[2] / "tools"))
from render_reference import load_pack


def roof_patterns(pack, bank):
    """Resolve all metatiles to palette-independent native 2bpp shapes."""
    files = pack["runtime_files"]
    sheet = bytes(files[f"gfx/tilesets/{bank}.2bpp"])
    palette = pack["data"]["tilesets"][bank]["palette_map"]
    layout = bytes(files[f"data/tilesets/{bank}_metatiles.bin"])
    count = len(sheet) // 16

    def glyph(slot):
        # Match the native renderer's VRAM bank addressing, including compact sheets.
        if slot < len(palette) and palette[slot] & 8:
            low = slot & 0x7f
            if count % 2 == 0 and low + count // 2 < count:
                return low + count // 2
            if low + 128 < count:
                return low + 128
        if slot < count:
            return slot
        return slot - 128 if 0 <= slot - 128 < count else 0

    tiles = {}
    for slot in set(layout):
        start = 16 * glyph(slot)
        tiles[slot] = sheet[start:start + 16]
    return [b"".join(tiles[s] for s in layout[i:i + 16]) for i in range(0, len(layout), 16)]


def east_neighbor(rooms, name, x, y):
    attrs = rooms[name]["attributes"]
    if x + 1 < attrs["width"]:
        return name, x + 1, y
    for link in attrs["connections"]:
        if link["direction"] == "east":
            target = link["target_map"]
            row = y - link["offset"]  # Connection offsets are in whole metatiles.
            if 0 <= row < rooms[target]["attributes"]["height"]:
                return target, 0, row
    return None


def check_garden_roofs(pack):
    rooms = pack["data"]["maps"]
    left, right = roof_patterns(pack, "johto_modern")[0x08:0x0a]

    @cache
    def halves(bank):
        patterns = roof_patterns(pack, bank)
        return (
            {i for i, p in enumerate(patterns) if p == left},
            {i for i, p in enumerate(patterns) if p == right},
        )

    errors = []
    garden = rooms["MnMoorhead"]
    a = garden["attributes"]
    garden_roofs = set.union(*halves(a["tileset_name"]))
    for x in (11, 12):
        if garden["blocks"][8 * a["width"] + x] in garden_roofs:
            errors.append(f"MnMoorhead ({x},8): the planting table contains a roof half")

    maps = matched = 0
    for name, room in rooms.items():
        a = room["attributes"]
        if a["environment"] not in ("ROUTE", "TOWN"):
            continue
        if not (name.startswith("Mn") or str(a.get("map_constant", "")).startswith("MN_")):
            continue
        maps += 1
        left_ids, _ = halves(a["tileset_name"])
        for index, block in enumerate(room["blocks"]):
            if block not in left_ids:
                continue
            matched += 1
            y, x = divmod(index, a["width"])
            neighbor = east_neighbor(rooms, name, x, y)
            if neighbor:
                other, nx, ny = neighbor
                mate = rooms[other]
                attr = mate["attributes"]
                if mate["blocks"][ny * attr["width"] + nx] in halves(attr["tileset_name"])[1]:
                    continue
            errors.append(f"{name} ({x},{y}): detached left gatehouse roof")
    if errors:
        raise ValueError("\n".join(errors))
    print(f"Checked {maps} Minnesota exteriors. All {matched} left gatehouse roofs have matching right halves.")


if __name__ == "__main__":
    cli = argparse.ArgumentParser(description=__doc__)
    cli.add_argument("pack", type=Path)
    args = cli.parse_args()
    try:
        check_garden_roofs(load_pack(args.pack))
    except ValueError as error:
        cli.exit(1, f"{error}\n")
