#!/usr/bin/env python3
"""Verify the compiled brown shoreline restoration against both of its native source packs."""
import argparse
import base64
import hashlib
import json
import runpy
import sys
from collections import defaultdict, deque
from pathlib import Path

HERE = Path(__file__).resolve().parent
ROCKY = HERE.parent / "shore-rocks-cave-mouths-v01"
sys.path.insert(0, str(HERE.parents[1] / "tools"))
from render_reference import SourceArt, load_pack

MANIFEST = "northstar-brown-shore-edges-v1"
SHORES = ("routes", "places", "route-landings", "places-landings")
TIMES = ("morn", "day", "nite")
CAVE_RIDGE_JOIN = ("MnR05C00R00", 14, 12)
CHANNEL_BLOCKS = {(22, 0), (23, 0), (28, 0), (23, 1), (28, 1), (28, 2)}
CHANNEL_BUOYS = {(23, 0, 1), (23, 0, 3), (28, 0, 0)}
PRESERVED_TERRAIN = {
    "TALL_GRASS", "LONG_GRASS", "LONG_GRASS_1C", "TALL_GRASS_10", "CUT_08", "CUT_28",
    "08", "10", "14", "18", "1C", "28", "GRASS_48", "GRASS_49", "GRASS_4A", "GRASS_4B", "GRASS_4C",
    "WATER", "WATER_21", "WHIRLPOOL", "WATERFALL", "BUOY",
}


def require(value, label):
    if not value:
        raise ValueError(label)


def exact_except(before, after, excluded, label):
    require(before.keys() == after.keys(), f"{label}: inventory changed")
    for key in before.keys() - set(excluded):
        require(before[key] == after[key], f"{label}: {key} changed")


def records(path, stem):
    return json.loads((path / (stem + ".json")).read_text())


def cells(directory, sections):
    result = {}
    for section in sections:
        for room in records(directory, section):
            for cell in room["placements"]:
                key = (room["map"], *cell["at"])
                require(key not in result, f"{key}: duplicate placement")
                result[key] = (room["tileset"], cell["from"], cell["to"])
    return result


def signature(pack, bank, block, art):
    key = f"{block:02x}"
    metadata = pack["data"]["tilesets"][bank]
    files = pack["runtime_files"]
    tiles = files[f"data/tilesets/{bank}_metatiles.bin"][16*block:16*(block+1)]
    require(len(tiles) == 16, f"{bank}/{key}: missing native tile definition")
    raw = files[f"gfx/tilesets/{bank}.2bpp"]
    palette = metadata["palette_map"]
    ink = []
    for glyph in tiles:
        color = palette[glyph]
        physical = len(raw)//32 + (glyph & 127) if color & 8 else glyph
        ink.append((glyph, color, bytes(raw[16*physical:16*(physical+1)])))
    return (tuple(metadata["collision"][key]), ink,
            tuple(art.blocks(bank, time)[block].tobytes() for time in TIMES))


def native_shores(pack, shore):
    """Read actual placed blocks, four collision quadrants and native art in each outdoor light."""
    art = SourceArt(pack)
    native = {}
    for (name, x, y), (bank, original, _) in shore.items():
        room = pack["data"]["maps"][name]
        attrs = room["attributes"]
        require(attrs["tileset_name"] == bank and 0 <= x < attrs["width"] and 0 <= y < attrs["height"],
                f"{name} {(x, y)}: wrong bank or coordinates")
        block = room["blocks"][y*attrs["width"] + x]
        require(block == original, f"{name} {(x, y)}: expected original {original:02x}, found {block:02x}")
        if (bank, block) not in native:
            native[bank, block] = signature(pack, bank, block, art)
    # The runtime switches water frames independently of the rendered still image.
    animations = {key: value for key, value in pack["runtime_files"].items()
                  if key.startswith("gfx/tilesets/water/") or key.endswith("_animations.json")}
    return native, animations


def added_route_art(rocky, corrected, recipes):
    """Reconstruct the only permitted additions to the rocky pack's retained native art."""
    old, new = rocky["data"]["tilesets"], corrected["data"]["tilesets"]
    files, updated = rocky["runtime_files"], corrected["runtime_files"]
    groups = defaultdict(list)
    for recipe in recipes:
        groups[recipe["tileset"]].append(recipe)
    require(old.keys() == new.keys() and groups.keys() <= old.keys(), "tileset inventory")
    changed_files, logical = set(), {}
    for bank in old:
        if bank not in groups:
            require(old[bank] == new[bank], f"{bank}: unrelated native tileset changed")
            continue
        exact_except(old[bank], new[bank], {"collision"}, bank)
        base = f"data/tilesets/{bank}"
        binary, collision = base + "_metatiles.bin", base + ".json"
        layout = bytearray(files[binary])
        masks = dict(old[bank]["collision"])
        for recipe in groups[bank]:
            index = len(layout)//16
            key = bank, recipe["id"]
            require(key not in logical and index <= 255 and f"{index:02x}" not in masks,
                    f"{bank}: duplicate or out-of-range metatile {key}")
            require(len(recipe["tiles"]) == 16, f"{key}: wrong native metatile size")
            masks[f"{index:02x}"] = recipe.get("collision", masks[f"{recipe['collision_from']:02x}"])
            logical[key] = index
            layout.extend(recipe["tiles"])
        require(layout == bytes(updated[binary]), f"{bank}: unexpected native metatile graphic")
        require(new[bank]["collision"] == masks == json.loads(bytes(updated[collision])),
                f"{bank}: collision inventory mismatch")
        changed_files.update((binary, collision))
    exact_except(files, updated, changed_files, "runtime files including native art and animations")
    return logical


def route_connections(pack):
    """Follow an ordinary dry footpath across the three route parts, including the southern bridge."""
    data = pack["data"]
    names = ("MnStCloud", "MnR05C00R00", "MnR05C00R01", "MnR05C00R02", "MnMinneapolis")
    heights = []
    walk = {}
    for name in names:
        room = data["maps"][name]
        attrs = room["attributes"]
        width, height = attrs["width"], attrs["height"] * 2
        heights.append(height)
        collision = data["tilesets"][attrs["tileset_name"]]["collision"]
        absent = set()
        if name == "MnMinneapolis":
            # Arrival runs the city's installed callbacks. Retired wardens still have object records.
            for line in room["map_script_section_commands"]:
                if line["command"] == "callback" and line["args"][0] == "MAPCALLBACK_OBJECTS":
                    commands = room["scripts"][line["args"][1]]
                    if all(row["command"] in {"disappear", "endcallback"} for row in commands):
                        absent.update(row["args"][0] for row in commands if row["command"] == "disappear")
        occupied = {(o["x"], o["y"]) for o in room["objects"] if o["object_identifier"] not in absent}
        occupied.update((warp["x"], warp["y"]) for warp in room["events"]["warps"])
        # For a walker the bridge resolver returns FLOOR on either bridge deck and WALL on the rails.
        dry = {"FLOOR", "TALL_GRASS", "BRIDGE_EW_DECK", "BRIDGE_NS_DECK"}
        walk[name] = {(x, y) for x in range(width * 2) for y in range(height)
                      if (x, y) not in occupied and
                      collision[f"{room['blocks'][y//2 * width + x//2]:02x}"][2*(y%2) + x%2] in dry}
    for i, (lanes, offset) in enumerate((((24, 25), 0), ((28, 29), 0), ((22, 23), 0), ((22, 23), -9))):
        north, south = names[i:i + 2]
        for name, target, direction, shift in ((north, south, "south", offset), (south, north, "north", -offset)):
            links = data["maps"][name]["attributes"]["connections"]
            require([(l["target_map"], l["offset"]) for l in links if l["direction"] == direction]
                    == [(target, shift)], f"{name}: lost reciprocal {direction} connection to {target}")
        for x in lanes:
            landing = x - 2 * offset
            require((x, heights[i] - 1) in walk[north] and (landing, 0) in walk[south],
                    f"{north} to {south}: blocked junction at x={x}")
    require(all((x, y) in walk["MnMinneapolis"] for x in (40, 41) for y in range(4)),
            "Minneapolis north entrance does not reach the public road after arrival")
    continuous = set()
    start_y = 0
    for i in range(1, 4):
        continuous.update((x, y + start_y) for x, y in walk[names[i]])
        start_y += heights[i]
    distances = {(24, 0): 0}
    pending = deque(distances)
    while pending:
        x, y = pending.popleft()
        for point in ((x-1, y), (x+1, y), (x, y-1), (x, y+1)):
            if point in continuous and point not in distances:
                distances[point] = distances[x, y] + 1
                pending.append(point)
    for seam, lanes in ((heights[1], (28, 29)), (heights[1]+heights[2], (22, 23)), (start_y-1, (22, 23))):
        require(all((x, seam) in distances for x in lanes), f"Route 5: disconnected path at row {seam}")
    print(f"Two walkable lanes join each Route 5 section. The unobstructed dry route from the St. Cloud "
          f"end to the Minneapolis end takes {distances[22, start_y-1]} steps across the bridge.")


def channel_edges(pack):
    room = pack["data"]["maps"]["MnR05C00R00"]
    attrs = room["attributes"]
    bank = attrs["tileset_name"]
    layout = pack["runtime_files"][f"data/tilesets/{bank}_metatiles.bin"]
    collision = pack["data"]["tilesets"][bank]["collision"]

    def quadrant(x, y):
        block = room["blocks"][y//2 * attrs["width"] + x//2]
        offset = 16*block + 8*(y % 2) + 2*(x % 2)
        return collision[f"{block:02x}"][2*(y % 2) + x % 2], [layout[offset+k] for k in (0, 1, 4, 5)]

    require(quadrant(51, 1)[0] == "WALL" and quadrant(51, 2) == ("WATER", [0x4c, 0x4c, 0x14, 0x14]),
            "Route 5 north rock edge faces the wrong way")
    southern, glyphs = quadrant(49, 6)
    require(southern == "WALL" and glyphs[:2] == [0x2c, 0x2c]
            and quadrant(49, 5) == ("WATER", [0x14] * 4), "Route 5 southern bank lacks its land-side rock edge")
    for x, y in ((55, 3), (56, 3), (55, 4), (56, 4)):
        require(quadrant(x, y) == ("WATER", [0x14] * 4), f"Route 5: midstream rock at {(x, y)}")


def check_maps(rocky, corrected, shore, cave, route, logical):
    old, new = rocky["data"], corrected["data"]
    exact_except(old, new, {"maps", "map_blocks", "tilesets"}, "game data")
    require(old["maps"].keys() == new["maps"].keys(), "map inventory")
    expected = defaultdict(dict)
    for (name, x, y), (bank, original, _) in shore.items():
        expected[name][x, y] = (bank, None, original)
    for (name, x, y), (bank, prior, target) in route.items():
        require(name.startswith("MnR05C"), f"Route 5 change outside its route: {name}")
        spot = (name, x, y)
        require(spot not in shore, f"{spot}: Route 5 overlaps an original shoreline restoration")
        installed = logical.get((bank, target), target)
        if spot in cave:
            require(spot == CAVE_RIDGE_JOIN, f"{spot}: unrelated cave block changed")
            native = f"data/tilesets/{bank}_metatiles.bin"
            previous_face = rocky["runtime_files"][native][prior*16 + 4:prior*16 + 16]
            joined_face = corrected["runtime_files"][native][installed*16 + 4:installed*16 + 16]
            original = old["tilesets"][bank]["collision"][f"{prior:02x}"]
            replacement = new["tilesets"][bank]["collision"][f"{installed:02x}"]
            require(previous_face == joined_face and original == ["FLOOR", "UP_WALL", "FLOOR", "FLOOR"]
                    and replacement == ["UP_WALL", *original[1:]],
                    f"{spot}: Route 5 rear ridge altered the existing cave face or collision")
        expected[name][x, y] = (bank, prior, installed)
    require(expected.keys() <= old["maps"].keys(), "unrecognized restored map")
    mirrors = dict(old["map_blocks"])
    protected_quadrants = set()
    for name, room in old["maps"].items():
        updated = new["maps"][name]
        if name not in expected:
            require(room == updated, f"{name}: unrelated map changed")
            continue
        exact_except(room, updated, {"blocks"}, name)
        attrs = room["attributes"]
        blocks = list(room["blocks"])
        for (x, y), (bank, prior, target) in expected[name].items():
            require(attrs["tileset_name"] == bank and 0 <= x < attrs["width"] and 0 <= y < attrs["height"],
                    f"{name} {(x, y)}: invalid placement")
            index = y*attrs["width"] + x
            require(prior is None or blocks[index] == prior, f"{name} {(x, y)}: wrong Route 5 starting tile")
            require(blocks[index] != target, f"{name} {(x, y)}: declared change had no effect")
            if prior is not None:
                before = old["tilesets"][bank]["collision"][f"{prior:02x}"]
                after = new["tilesets"][bank]["collision"][f"{target:02x}"]
                for q, (left, right) in enumerate(zip(before, after)):
                    if left in PRESERVED_TERRAIN or right in PRESERVED_TERRAIN:
                        require(left == right, f"{name} {(x, y)}: Route 5 altered grass or water collision")
                        adjusted = name == "MnR05C00R00" and ((left == "WATER" and (x, y) in CHANNEL_BLOCKS)
                                   or (left == "BUOY" and (x, y, q) in CHANNEL_BUOYS))
                        if not adjusted:
                            protected_quadrants.add((bank, prior, target, q))
            blocks[index] = target
        require(blocks == updated["blocks"], f"{name}: unexpected map change; cave placement or terrain differs")
        mirrors[attrs["blocks_label"]] = base64.b64encode(bytes(blocks)).decode("ascii")
    require(new["map_blocks"] == mirrors, "packed block mirrors differ from restored maps")
    original_art, final_art = SourceArt(rocky), SourceArt(corrected)
    for bank, left, right, quadrant in protected_quadrants:
        x, y = 16*(quadrant % 2), 16*(quadrant // 2)
        box = (x, y, x + 16, y + 16)
        for time in TIMES:
            require(original_art.blocks(bank, time)[left].crop(box).tobytes()
                    == final_art.blocks(bank, time)[right].crop(box).tobytes(),
                    f"{bank}: Route 5 changed native grass or water pixels")
    # The original rocky revision installed the six cave faces and all four warp relocations.
    # This checker inventories every exterior cave connection in the corrected game as well.
    runpy.run_path(str(ROCKY / "check-caves.py"))["check_caves"](corrected)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for arg in ("cliffs", "rocky", "corrected"):
        parser.add_argument(arg, type=Path)
    args = parser.parse_args()
    known = [records(ROCKY, f"{variant}-build") for variant in ("field", "mothlight")]
    inputs = tuple(hashlib.sha256(p.read_bytes()).hexdigest() for p in (args.cliffs, args.rocky))
    require(inputs in [(r["parent_pack_sha256"], r["expected_pack_sha256"]) for r in known],
            "pre-rock and rocky inputs must be the recorded pair of original Field or Mothlight packs")
    shore = cells(ROCKY, SHORES)
    caves = cells(ROCKY, ("caves",))
    require(len(shore) == 159 and len(caves) == 31 and not shore.keys() & caves.keys(),
            "expected 159 shoreline placements apart from the 31 original cave placements")
    original = native_shores(load_pack(args.cliffs), shore)
    rocky, corrected = load_pack(args.rocky), load_pack(args.corrected)
    require(rocky["report"]["manifests"][-1] == "northstar-rocky-shores-v1", "wrong selected parent")
    exact_except(rocky, corrected, {"data", "report", "identity", "runtime_files"}, "pack")
    exact_except(rocky["report"], corrected["report"], {"manifests"}, "pack report")
    require(corrected["report"]["manifests"] == rocky["report"]["manifests"] + [MANIFEST],
            "unexpected revision lineage")
    prior_id, new_id = rocky["identity"], corrected["identity"]
    exact_except(prior_id, new_id, {"content_hash", "runtime_modpack_id"}, "pack identity")
    require(new_id["runtime_modpack_id"] == prior_id["runtime_modpack_id"] + "+" + MANIFEST,
            "unexpected runtime lineage")
    digest = new_id["content_hash"]
    require(len(digest) == 64 and all(ch in "0123456789abcdef" for ch in digest)
            and digest != prior_id["content_hash"], "missing new content identity")
    route = cells(HERE, ("route5",))
    recipes = records(HERE, "route5-art")
    logical = added_route_art(rocky, corrected, recipes)
    check_maps(rocky, corrected, shore, caves, route, logical)
    route_connections(corrected)
    channel_edges(corrected)
    require(native_shores(corrected, shore) == original,
            "restored shoreline differs in original native pixels, palette, collision or water animation")
    retained = len(caves.keys() - route.keys())
    print(f"All {len(shore)} shore placements match the pre-rock pack in native ink, morning/day/night "
          f"pixels and collision. {retained} cave blocks remain identical. All cave faces and warps pass. "
          f"Verified {len(route)} declared Route 5 changes and {len(logical)} native metatile additions.")


if __name__ == "__main__":
    main()
