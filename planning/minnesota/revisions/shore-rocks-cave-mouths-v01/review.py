#!/usr/bin/env python3
"""Compare installed rock pixels, collision, cave warp mirrors and preserved game content."""
import argparse
import base64
import copy
import io
import json
import sys
from collections import defaultdict
from pathlib import Path

from PIL import Image

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE.parents[1] / "tools"))
from render_reference import SourceArt, load_pack

SECTIONS = ("routes", "places", "caves", "route-landings", "places-landings")
WATER = {"WATER", "WATER_21", "WHIRLPOOL", "WATERFALL", "BUOY"}
EDGE = {"left": (-1, 0, (1, 5)), "right": (1, 0, (0, 4)),
        "up": (0, -1, (4, 5)), "down": (0, 1, (0, 1))}


def read(name):
    return json.loads((HERE / (name + ".json")).read_text())


def same(actual, expected, message):
    if actual != expected:
        raise ValueError(message)


def exact_except(first, second, excluded, label):
    same(first.keys(), second.keys(), label + " inventory")
    for key in first.keys() - set(excluded):
        same(second[key], first[key], f"{label}: {key}")


def address(glyph, palette, count):
    return count // 2 + (glyph & 127) if palette & 8 else glyph


def check_ink(bank, recipes, before, after, files, palette, layout):
    path = f"gfx/tilesets/{bank}.2bpp"
    bits = bytearray(before[path])
    count = len(bits) // 16
    occupied = {address(glyph, palette[glyph], count) for glyph in layout}
    added = set()
    for recipe in recipes:
        glyph = recipe["glyph"]
        color = (8 * recipe.get("bank", 1)) | (palette[recipe["palette_from"]] & 7)
        pos = address(glyph, color, count)
        same(glyph in layout or glyph in added or pos in occupied or pos >= count, False,
             f"{bank}: new glyph aliases existing artwork")
        added.add(glyph)
        occupied.add(pos)
        palette[glyph] = color
        for y, row in enumerate(recipe["rows"]):
            for plane in (0, 1):
                bits[16 * pos + 2 * y + plane] = sum(
                    ((int(pixel) >> plane) & 1) << (7 - x) for x, pixel in enumerate(row))
    same(bytes(after[path]), bytes(bits), f"{bank}: native indexed graphics")
    palette_path = f"data/tilesets/{bank}_palette_map.json"
    same(json.loads(bytes(after[palette_path])), palette, bank + " palette mirror")
    png_path = f"gfx/tilesets/{bank}.png"
    atlas = Image.open(io.BytesIO(bytes(after[png_path]))).convert("L")
    same(atlas.size, (128, count // 16 * 8), bank + " native atlas dimensions")
    for slot in range(count):
        for y in range(8):
            lo, hi = bits[slot * 16 + 2 * y : slot * 16 + 2 * y + 2]
            for x in range(8):
                shift = 7 - x
                color = ((lo >> shift) & 1) | (((hi >> shift) & 1) << 1)
                same(atlas.getpixel((slot % 16 * 8 + x, slot // 16 * 8 + y)),
                     255 - 85 * color, bank + " atlas and two-bit graphics differ")
    for suffix in ("native_backgrounds", "foregrounds"):
        key = f"data/tilesets/{bank}_{suffix}.json"
        if key in before:
            original = json.loads(bytes(before[key]))
            overrides = dict(original)
            for tile in recipes:
                donor, target = str(tile["palette_from"]), str(tile["glyph"])
                if donor in original:
                    overrides[target] = copy.deepcopy(original[donor])
                    if suffix == "foregrounds" and "snow_light_from" in tile:
                        overrides[target][0] = original[str(tile["snow_light_from"])][0]
                else:
                    overrides.pop(target, None)
            same(json.loads(bytes(after[key])), overrides, bank + " native shade inheritance")
            files.add(key)
    files.update((path, png_path, palette_path))


def check_art(parent, child):
    a, b = parent["runtime_files"], child["runtime_files"]
    same(a.keys(), b.keys(), "runtime file inventory")
    old, new = parent["data"]["tilesets"], child["data"]["tilesets"]
    same(old.keys(), new.keys(), "native tileset inventory")
    art, ink = defaultdict(list), defaultdict(list)
    for section in SECTIONS:
        for recipe in read(section + "-art"):
            art[recipe["tileset"]].append(recipe)
        if (HERE / (section + "-ink.json")).is_file():
            for recipe in read(section + "-ink"):
                ink[recipe["tileset"]].append(recipe)
    ids, changed = {}, set()
    for bank in old.keys():
        expected = copy.deepcopy(old[bank])
        path = f"data/tilesets/{bank}_metatiles.bin"
        if bank in ink:
            check_ink(bank, ink[bank], a, b, changed, expected["palette_map"], a[path])
        if bank in art:
            blocks = bytearray(a[path])
            collisions = expected["collision"]
            for recipe in art[bank]:
                index = len(blocks) // 16
                key = (bank, recipe["id"])
                same(key in ids or f"{index:02x}" in collisions, False, bank + " duplicate metatile")
                ids[key] = index
                blocks.extend(recipe["tiles"])
                collisions[f"{index:02x}"] = recipe.get(
                    "collision", collisions[f"{recipe['collision_from']:02x}"])
            same(bytes(b[path]), bytes(blocks), bank + " native metatiles")
            collision_path = f"data/tilesets/{bank}.json"
            same(json.loads(bytes(b[collision_path])), collisions, bank + " collision mirror")
            changed.update((path, collision_path))
        same(new[bank], expected, bank + " tileset definition")
    for path in a.keys() - changed:
        same(b[path], a[path], path)
    return ids


def check_sections(parent, child, ids):
    old, new = parent["data"], child["data"]
    plans, warps = defaultdict(list), defaultdict(list)
    for section in SECTIONS:
        for room in read(section):
            for placement in room["placements"]:
                plans[room["map"]].append((section, room["tileset"], placement))
    for warp in read("cave-warps"):
        warps[warp["map"]].append(warp)
    exact_except(old, new, {"maps", "map_blocks", "map_scripts", "tilesets"}, "game data")
    same(new["maps"].keys(), old["maps"].keys(), "map inventory")
    mirrors, scripts = dict(old["map_blocks"]), dict(old["map_scripts"])
    painted = set()
    stones = 0
    banks = 0
    grass = {"TALL_GRASS", "LONG_GRASS", "LONG_GRASS_1C", "TALL_GRASS_10", "CUT_08", "CUT_28",
             "08", "10", "14", "18", "1C", "28", "GRASS_48", "GRASS_49", "GRASS_4A", "GRASS_4B", "GRASS_4C"}
    for name, base in old["maps"].items():
        if name not in plans:
            same(base, new["maps"][name], name + " untouched map")
            continue
        m = copy.deepcopy(base)
        attrs = m["attributes"]
        used = set()
        for section, bank, cell in plans[name]:
            x, y = cell["at"]
            same(bank, attrs["tileset_name"], name + " tileset binding")
            same((x, y) in used, False, name + " duplicate placement")
            used.add((x, y))
            i = y * attrs["width"] + x
            l, r = m["blocks"][i], ids.get((bank, cell["to"]), cell["to"])
            same(l, cell["from"], f"{name} stale block at {(x, y)}")
            left = old["tilesets"][bank]["collision"][f"{l:02x}"]
            right = new["tilesets"][bank]["collision"][f"{r:02x}"]
            if section != "caves":
                same(left, right, f"{name} changed shoreline access at {(x, y)}")
                if section in ("places", "routes"):
                    stones += 1
                else:
                    banks += 1
            graphic = f"data/tilesets/{bank}_metatiles.bin"
            for q, (lk, rk) in enumerate(zip(left, right)):
                if lk in grass or rk in grass:
                    same(rk, lk, name + " encounter square")
                    painted.add((bank, l, r, q))
                if lk in WATER or rk in WATER:
                    same(rk, lk, name + " water permission")
                    k = q // 2 * 8 + q % 2 * 2
                    for offset in (k, k + 1, k + 4, k + 5):
                        prior = parent["runtime_files"][graphic][l * 16 + offset]
                        after = child["runtime_files"][graphic][r * 16 + offset]
                        if prior == 0x14:
                            same(after, prior, name + " original animated water square")
            m["blocks"][i] = r
        for change in warps.pop(name, []):
            index = change["index"] - 1
            w = m["events"]["warps"][index]
            same([w["x"], w["y"]], change["from"], name + " previous warp position")
            w["x"], w["y"] = change["to"]
            event = [e for e in m["map_event_section_commands"] if e["command"] == "warp_event"][index]
            same(event["args"][:2], list(map(str, change["from"])), name + " warp mirror preimage")
            event["args"][:2] = list(map(str, change["to"]))
            scripts[attrs["map_events_label"]] = [
                {"command": c["command"], "args": c["args"]} for c in m["map_event_section_commands"]]
        same(m, new["maps"][name], name + " installed blocks, actors, warps and scripts")
        mirrors[attrs["blocks_label"]] = base64.b64encode(bytes(m["blocks"])).decode("ascii")
    same(dict(warps), {}, "relocated warp lacks accompanying exterior")
    same(new["map_blocks"], mirrors, "packed map block mirrors")
    same(new["map_scripts"], scripts, "global script and event mirrors")
    before_art, after_art = SourceArt(parent), SourceArt(child)
    for bank, left, right, q in sorted(painted):
        x, y = (q % 2) * 16, (q // 2) * 16
        box = (x, y, x + 16, y + 16)
        for light in ("morn", "day", "nite"):
            same(after_art.blocks(bank, light)[right].crop(box).tobytes(),
                 before_art.blocks(bank, light)[left].crop(box).tobytes(), bank + " grass artwork")
    print(f"{len(plans)} changed exteriors, {stones} shore rock placements and {banks} waterline "
          "placements with identical movement permissions.")


def check_launch_edges(pack):
    contacts = read("route-landing-edges") + read("places-landing-edges")
    files = pack["runtime_files"]
    water = bytes(files["gfx/tilesets/water/water.2bpp"])
    if len(water) % 16 or len({water[i:i + 16] for i in range(0, len(water), 16)}) < 2:
        raise ValueError("missing distinct native water animation frames")
    narrows = json.loads(bytes(files["data/tilesets/mn_shiner_narrows_animations.json"]))
    for edge in contacts:
        name = edge["map"]
        room = pack["data"]["maps"][name]
        attrs = room["attributes"]
        dx, dy, offsets = EDGE[edge["direction"]]
        x, y = edge["from"]
        x, y = x + dx, y + dy
        if not (0 <= x < attrs["width"] * 2 and 0 <= y < attrs["height"] * 2):
            raise ValueError(f"{name}: launch crosses map boundary at {(x, y)}")
        bank = attrs["tileset_name"]
        block = room["blocks"][y // 2 * attrs["width"] + x // 2]
        quad = y % 2 * 2 + x % 2
        same(pack["data"]["tilesets"][bank]["collision"][f"{block:02x}"][quad], "WATER",
             f"{name}: expected launchable water at {(x, y)}")
        native = files[f"data/tilesets/{bank}_metatiles.bin"]
        start = block * 16 + (y % 2) * 8 + (x % 2) * 2
        for offset in offsets:
            glyph = native[start + offset]
            if bank == "mn_shiner_narrows":
                same(glyph in (0x91, 0x92, 0x93), True, f"{name}: missing local water ripple")
                ripple = narrows[str(glyph)]
                same(ripple["frame_ticks"], 22, name + " local water animation timing")
                same(len({bytes(frame) for frame in ripple["frames"]}), 4, name + " local water frames")
            else:
                same(glyph, 0x14, f"{name}: false bank in front of {edge['from']}")
    print(f"{len(contacts)} open landing edges meet animated native water at the player's feet.")
    corners = read("route-open-water")
    for point in corners:
        room = pack["data"]["maps"][point["map"]]
        attrs = room["attributes"]
        bank = attrs["tileset_name"]
        x, y = point["at"]
        block = room["blocks"][y // 2 * attrs["width"] + x // 2]
        q = y % 2 * 2 + x % 2
        same(pack["data"]["tilesets"][bank]["collision"][f"{block:02x}"][q], "WATER",
             point["map"] + " open-water bend")
        tiles = files[f"data/tilesets/{bank}_metatiles.bin"]
        start = block * 16 + y % 2 * 8 + x % 2 * 2
        same([tiles[start + offset] for offset in (0, 1, 4, 5)], [0x14] * 4,
             f"{point['map']}: detached bank corner at {(x, y)}")
    print(f"{len(corners)} shoreline squares have continuous native water around their landings.")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("parent", type=Path)
    parser.add_argument("child", type=Path)
    args = parser.parse_args()
    parent, child = load_pack(args.parent), load_pack(args.child)
    exact_except(parent, child, {"data", "runtime_files", "identity", "report"}, "pack")
    exact_except(parent["report"], child["report"], {"manifests"}, "report")
    same(child["report"]["manifests"], parent["report"]["manifests"] + ["northstar-rocky-shores-v1"],
         "revision lineage")
    check_sections(parent, child, check_art(parent, child))
    check_launch_edges(child)


if __name__ == "__main__":
    main()
