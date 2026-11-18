#!/usr/bin/env python3
"""Compare installed northern route contours and their directed movement without playing."""
import argparse
import json
import sys
from pathlib import Path

SOURCE = Path(__file__).resolve().parent
sys.path.insert(0, str(SOURCE.parents[1] / "tools"))

from audit_regional_design import DIRS, Terrain, check, coords
from render_reference import SourceArt, load_pack
from review_r12_depth import walk


def resting_graph(terrain, *, allow_ice=True, surf=False, remove_rock=False):
    """One directional input per edge, including ledge landings and forced ice slides."""
    surfaces = {"FLOOR", "TALL_GRASS", "UP_WALL", "HOP_DOWN"}
    if allow_ice:
        surfaces.add("ICE")
    if surf:
        surfaces.add("WATER")
    occupied = terrain.occupied.copy()
    if remove_rock:
        rock, = [o for o in terrain.map["objects"] if o["script"] == "MN_R12_RAVEN_ROCK_FRACTURE"]
        occupied.remove(coords(rock))
    passable = {p for p, kind in terrain.tiles.items() if kind in surfaces} - occupied
    # The shelter's second warp records a safe return on ordinary floor.
    passable -= {p for p in terrain.warps if terrain.tiles[p] in {"CAVE", "DOOR", "LADDER"}}

    def step(at, dx, dy):
        current = terrain.tiles[at]
        distance = 2 if dy == 1 and current == "HOP_DOWN" else 1
        target = at[0] + distance * dx, at[1] + distance * dy
        if target not in passable:
            return None
        if (dy == -1 and current == "UP_WALL") or (dy == 1 and terrain.tiles[target] == "UP_WALL"):
            return None
        return target

    edges = {}
    for start in passable:
        edges[start] = []
        for dx, dy in DIRS.values():
            stop = step(start, dx, dy)
            if stop is None:
                continue
            while terrain.tiles[stop] == "ICE":
                next_tile = step(stop, dx, dy)
                if next_tile is None:
                    break
                stop = next_tile
            edges[start].append(stop)
    return edges


def check_paths(pack):
    raven = Terrain(pack, "MnR12C00R00")
    base, summit, lookout, clearing = (35, 14), (42, 9), (49, 8), (12, 12)
    intact = walk(resting_graph(raven, surf=True), base)
    check(lookout in intact and summit not in intact and clearing not in intact, "Raven Rock gates")
    check(lookout not in walk(resting_graph(raven, allow_ice=False), base), "free dry shortcut")
    broken = resting_graph(raven, allow_ice=False, remove_rock=True)
    dry = walk(broken, base)
    check({summit, lookout, (48, 6)} <= dry and clearing not in dry, "Rock Smash dry route")
    check((48, 8) in broken[(48, 6)] and (48, 6) not in broken[(48, 8)], "Raven Rock drop")
    check({base, lookout} <= walk(resting_graph(raven), summit), "Raven Rock exit after regrowth")

    tor = Terrain(pack, "MnR13C00R00")
    graph = resting_graph(tor)
    lip, landing = (39, 24), (39, 26)
    check(tor.tiles[lip] == "HOP_DOWN" and tor.tiles[(39, 25)] == "WALL", "Wind Tor face")
    check(landing in graph[lip] and lip not in graph[landing], "Wind Tor one-way drop")
    uphill = walk(graph, landing, lip)
    check(uphill and {(47, 24), (47, 25)} <= set(uphill), "Wind Tor eastern stair return")
    check(len(uphill) - 1 == 18, "Wind Tor uphill distance")

    cedar = Terrain(pack, "MnR13C00R01")
    for terrain, mouth, door, number in [
        (raven, (40, 15), "MN_R12_CAVE_1F", 1), (cedar, (14, 5), "MnR13Cave1F", 1),
        (cedar, (14, 6), "MnR13Cave1F", 2),
    ]:
        check(any(coords(w) == mouth and w["target_map"] == door and w["index"] == number
                  and w["target_warp_id"] == 1 for w in terrain.map["events"]["warps"]), door)
    approaches = resting_graph(cedar)
    check(walk(approaches, (14, 6), (20, 6)) and walk(approaches, (20, 6), (14, 6)),
          "Cedar Shelter doorway and trail")


def original_grass(pack):
    """Record encounter quadrants, shared block IDs and their rendered native palettes."""
    art = SourceArt(pack)
    footprints, sprites = {}, {}
    for name in pack["data"]["maps"]:
        if not (name.startswith(("MnR11C", "MnR12C", "MnR13C")) and name[6:8].isdigit()):
            continue
        terrain = Terrain(pack, name)
        occupied = {xy: terrain.block(*xy) for xy, kind in terrain.tiles.items() if kind == "TALL_GRASS"}
        footprints[name] = occupied
        blocks = set(occupied.values())
        if name == "MnR12C00R00":
            # The green-topped drop is ordinary ledge footing in this tileset.
            check(terrain.block(48, 6) == 0x4d, "Raven Rock grassy crest")
            blocks.add(0x4d)
        for block in blocks:
            for period in ("morn", "day", "nite"):
                key = terrain.tileset, block, period
                if key not in sprites:
                    sprites[key] = art.blocks(terrain.tileset, period)[block].tobytes()
    check(sum(name.startswith(("MnR12", "MnR13")) for name in footprints) == 5
          and {"MnR11C00R02", "MnR11C00R03"} <= footprints.keys(), "northern route exteriors")
    return footprints, sprites


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("parent", type=Path)
    parser.add_argument("candidate", type=Path)
    args = parser.parse_args()
    parents = load_pack(args.parent)
    art = json.loads((SOURCE / "northern-art.json").read_text())
    layouts = [r for name in ("northern-routes.json", "shore-north.json")
               for r in json.loads((SOURCE / name).read_text())]
    banks = {recipe["tileset"] for recipe in art}
    next_id = {bank: len(parents["runtime_files"][f"data/tilesets/{bank}_metatiles.bin"]) // 16
               for bank in banks}
    native_ids = {}
    for recipe in art:
        bank = recipe["tileset"]
        native_ids[bank, recipe["id"]] = next_id[bank]
        next_id[bank] += 1
    grass = original_grass(parents)
    shore_cases = [
        ("MnR11C00R02", [((24, 24), (23, 24))], ((25, 24), (26, 24))),
        ("MnR11C00R03", [((x, 12), (x, 11)) for x in range(18, 22)] + [((18, 12), (17, 12))],
         ((23, 12), (24, 12))),
    ]
    for name, blocked, low in shore_cases:
        parent_terrain = Terrain(parents, name)
        for water, bank in [*blocked, low]:
            check((parent_terrain.tiles[water], parent_terrain.tiles[bank]) == ("WATER", "FLOOR"),
                  f"{name} original dismount from {water} to {bank}")
    expected = {}
    for route in layouts:
        terrain = Terrain(parents, route["map"])
        check(terrain.tileset == route["tileset"], route["map"])
        blocks = list(terrain.blocks)
        for placement in route["placements"]:
            x, y = placement["at"]
            index = y * terrain.width + x
            check(blocks[index] == placement["from"], f'{route["map"]} parent {(x, y)}')
            blocks[index] = native_ids.get((terrain.tileset, placement["to"]), placement["to"])
        expected[route["map"]] = blocks
    del parents
    candidate = load_pack(args.candidate)
    for name, blocks in expected.items():
        check(candidate["data"]["maps"][name]["blocks"] == blocks, f"{name} installed blocks")
    for row in art:
        tileset = row["tileset"]
        native_id = native_ids[tileset, row["id"]]
        start = native_id * 16
        native = candidate["runtime_files"][f"data/tilesets/{tileset}_metatiles.bin"]
        check(list(native[start:start + 16]) == row["tiles"], f"{tileset} terrain art {native_id}")
        collision = candidate["data"]["tilesets"][tileset]["collision"]
        check(collision[f'{native_id:02x}'] == collision[f'{row["collision_from"]:02x}'],
              f"{tileset} terrain collision {native_id}")
    check(original_grass(candidate) == grass,
          "Route 11/12/13 native grass blocks, appearance and encounter quadrants")
    for name, blocked, (launch, low) in shore_cases:
        terrain = Terrain(candidate, name)
        for water, cliff in blocked:
            check((terrain.tiles[water], terrain.tiles[cliff]) == ("WATER", "WALL"),
                  f"{name} fitted wall at {cliff}")
        check((terrain.tiles[launch], terrain.tiles[low]) == ("WATER", "FLOOR"),
              f"{name} ordinary low bank at {low}")
    check_paths(candidate)
    print("Northern contours passed. Both ledges, the 18-step stair return, the Rock Smash "
          "and ice alternatives and both shelter approaches use their installed terrain. "
          "The two low banks remain open and all Route 11–13 grass matches the parent.")


if __name__ == "__main__":
    main()
