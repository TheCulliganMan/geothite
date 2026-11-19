#!/usr/bin/env python3
"""Audit every Minnesota outdoor cave portal, including rock shoulders across connected map chunks."""
import argparse
import json
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE.parents[1] / "tools"))
from render_reference import load_pack


def require(value, message):
    if not value:
        raise ValueError(message)


def resolve_tile(maps, room, x, y):
    attrs = room["attributes"]
    width = 2 * attrs["width"]
    if x < 0 or x >= width:
        east = x >= width
        direction = "east" if east else "west"
        links = [c for c in attrs["connections"] if c["direction"] == direction]
        require(len(links) == 1, f"{room['id']} lacks its {direction} rock shoulder")
        link = links[0]
        neighbor = maps[link["target_map"]]
        require(any(c["target_map"] == room["id"] and c["direction"] == ("west" if east else "east")
                    for c in neighbor["attributes"]["connections"]), "disconnected cave cliff at map seam")
        require(link["offset"] == 0, "cave shoulder crosses a vertically displaced route chunk")
        x = x - width if east else x + 2*neighbor["attributes"]["width"]
        room = neighbor
    require(0 <= x < 2*room["attributes"]["width"] and 0 <= y < 2*room["attributes"]["height"],
            f"{room['id']} outside native map at {(x, y)}")
    return room, x, y


def native_glyph(pack, bank, glyph):
    bits = pack["runtime_files"][f"gfx/tilesets/{bank}.2bpp"]
    color = pack["data"]["tilesets"][bank]["palette_map"][glyph]
    physical = len(bits)//32 + (glyph & 127) if color & 8 else glyph
    return bytes(bits[16*physical:16*physical+16])


def quadrant(pack, room, x, y):
    room, x, y = resolve_tile(pack["data"]["maps"], room, x, y)
    attrs = room["attributes"]
    bank = attrs["tileset_name"]
    block = room["blocks"][(y//2)*attrs["width"] + x//2]
    definition = pack["data"]["tilesets"][bank]
    quad = 2*(y % 2) + x % 2
    token = definition["collision"][f"{block:02x}"][quad]
    raw = pack["runtime_files"][f"data/tilesets/{bank}_metatiles.bin"]
    pixels = []
    for yy in range((quad//2)*2, (quad//2)*2+2):
        for xx in range((quad%2)*2, (quad%2)*2+2):
            glyph = raw[16*block + 4*yy + xx]
            pixels.append(native_glyph(pack, bank, glyph))
    return token, pixels


def check_face(pack, room, at):
    x, y = at
    token, mouth_pixels = quadrant(pack, room, x, y)
    require(token == "CAVE" and len(set(mouth_pixels)) > 1, f"{room['id']} missing cave arch at {at}")
    if room["id"] == "MnR05C00R00" and (x, y) == (30, 27):
        bank = room["attributes"]["tileset_name"]
        cap, face = (native_glyph(pack, bank, glyph) for glyph in (0x3c, 0x4c))
        if quadrant(pack, room, x+1, y)[1][:2] == [cap, cap]:
            # The low Rock Cut wall is one native course tall. Its original deep doorway
            # cuts into the cap; the course continues at the same height on both sides.
            require(mouth_pixels == [native_glyph(pack, bank, g) for g in (0x46, 0x47, 0x56, 0x57)],
                    f"{room['id']} recessed Rock Cut arch changed")
            for dx in (-2, -1, 1, 2):
                surface, pixels = quadrant(pack, room, x+dx, y)
                require(surface == "WALL" and pixels == [cap, cap, face, face],
                        f"{room['id']} broken single-course shoulder at {(x+dx,y)}")
            return [4, 4]
    # The last glyph of the right jamb carries the local face material, even on
    # the Iron Range mine door. The paired walking squares flanking the portal
    # must be solid rock with 32 px of support. Outer bevels and original timber
    # frames may occupy three of those eight glyphs.
    _, jamb = quadrant(pack, room, x+1, y)
    face = jamb[-1]
    counts = []
    for dx in (-1, 1):
        count = []
        for d in (1, 2):
            surface, pixels = quadrant(pack, room, x+dx*d, y)
            require(surface == "WALL", f"{room['id']} gap in the cliff at {(x+dx*d, y)}: {surface}")
            count.append(pixels.count(face))
        require(min(count) > 0 and sum(count) >= 5 and max(count) == 4,
                f"{room['id']} broken rock shoulder at {at} facing {'west' if dx < 0 else 'east'}: {count}")
        counts.append(sum(count))
    return counts


def check_caves(pack):
    data = pack["data"]
    maps = data["maps"]
    names = {name: key for key, m in maps.items() for name in (key, m["attributes"]["map_constant"])}
    source = json.loads((HERE / "cave-checks.json").read_text())
    require(source["schema"] == 1, "unknown cave inventory format")
    expected = {(r["map"], tuple(r["mouth"])) for r in source["mouths"]}
    require(len(expected) == len(source["mouths"]), "duplicate exterior threshold")
    found = set()
    exterior_to_cave = set()
    cave_to_exterior = set()
    inventory = set()
    return_inventory = set()
    for name, room in maps.items():
        attrs = room["attributes"]
        if not name.startswith("Mn"):
            continue
        if attrs["environment"] == "CAVE":
            for warp in room["events"]["warps"]:
                target = names[warp["target_map_constant"]]
                if target.startswith("Mn") and maps[target]["attributes"]["environment"] in ("ROUTE", "TOWN"):
                    cave_to_exterior.add((name, warp["index"]))
        if attrs["environment"] not in ("ROUTE", "TOWN"):
            continue
        c = data["tilesets"][attrs["tileset_name"]]["collision"]
        w = attrs["width"]
        for i, block in enumerate(room["blocks"]):
            for q, role in enumerate(c[f"{block:02x}"]):
                if role == "CAVE":
                    found.add((name, (2*(i % w)+q%2, 2*(i//w)+q//2)))
        for warp in room["events"]["warps"]:
            dest = names[warp["target_map_constant"]]
            if maps[dest]["attributes"]["environment"] == "CAVE":
                exterior_to_cave.add((name, warp["index"]))
    require(found == expected, f"unbound or stale visible cave entrances: {found ^ expected}")

    def verify_link(name, record):
        room = maps[name]
        location = next((w for w in room["events"]["warps"] if w["index"] == record["index"]), None)
        require(location is not None, f"{name} missing warp {record['index']}")
        target = names[location["target_map_constant"]]
        require(target == names[location["target_map"]] == record["target_map"], f"{name} wrong destination")
        arrival = next((w for w in maps[target]["events"]["warps"] if w["index"] == record["to_index"]), None)
        require(arrival and [arrival["x"], arrival["y"]] == record["to"] and
                location["target_warp_id"] == record["to_index"] and
                [location["x"], location["y"]] == record["at"], f"{name} wrong coordinates for {record}")

    for r in source["mouths"]:
        name, at = r["map"], r["mouth"]
        room = maps[name]
        check_face(pack, room, at)
        require(r["approach"] == [at[0], at[1]+1] and
                quadrant(pack, room, *r["approach"])[0] in ("FLOOR", "TALL_GRASS"),
                f"{name} non-walkable cave doorstep")
        occupied = {(o["x"], o["y"]) for o in room["objects"]}
        require(tuple(at) not in occupied and tuple(r["approach"]) not in occupied, f"{name} blocked cave doorway")
        require(any(w["at"] == at and w["target_map"] == r["interior"] for w in r["outdoor_warps"]),
                f"{name} unbound visible portal")
        for warp in r["outdoor_warps"]:
            verify_link(name, warp)
            if maps[warp["target_map"]]["attributes"]["environment"] == "CAVE":
                inventory.add((name, warp["index"]))
        require(r["returns"], f"{name} missing cave return")
        for warp in r["returns"]:
            verify_link(r["interior"], warp)
            require(warp["target_map"] == name and warp["to"] in (at, r["approach"]), f"{name} wrong return")
            return_inventory.add((r["interior"], warp["index"]))
    require(inventory == exterior_to_cave, f"unclassified outdoor-to-cave warp IDs: {inventory ^ exterior_to_cave}")
    require(return_inventory == cave_to_exterior,
            f"unclassified cave-to-outdoor warp IDs: {return_inventory ^ cave_to_exterior}")
    print(f"{len(found)} physical cave entrances, {len(inventory)} outdoor-to-cave warp bindings and "
          f"{len(return_inventory)} interior returns passed. "
          "Both rock shoulders have 32 pixels of solid support, including across the Granite Passage map seam.")


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("candidate", type=Path)
    args = parser.parse_args()
    check_caves(load_pack(args.candidate))
