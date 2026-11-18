#!/usr/bin/env python3
"""Check three installed public fishing piers and the separate high quay and low cove in Duluth."""
import argparse
import json
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE.parents[1] / "tools"))

from audit_regional_design import Terrain, check, same_except
from render_reference import load_pack

BANK = "mn_outdoor_johto_stairs"
MAP = "MnBrainerd"
WATER_GLYPH = 0x14


def glyph(pack, bank, index):
    raw = pack["runtime_files"][f"gfx/tilesets/{bank}.2bpp"]
    palette = pack["data"]["tilesets"][bank]["palette_map"][index]
    physical = (index & 127) + len(raw) // 32 if palette & 8 else index
    return bytes(raw[16*physical:16*physical+16]), palette & 7


def verify(parent, child):
    plan = json.loads((HERE / "brainerd-pier.json").read_text())
    before, after = Terrain(parent, MAP), Terrain(child, MAP)
    same_except(before.map, after.map, {"blocks"}, MAP)
    check(before.tileset == after.tileset == BANK, "Brainerd outdoor tileset")
    check(parent["data"]["maps"]["MnBrainerdGym"] == child["data"]["maps"]["MnBrainerdGym"],
          "Brainerd Gym must retain its lake, people and progression")
    layout = bytes(child["runtime_files"][f"data/tilesets/{BANK}_metatiles.bin"])
    collision = child["data"]["tilesets"][BANK]["collision"]
    definitions = {d["name"]: d for d in plan["metatiles"]}
    expected = list(before.blocks)
    for cell in [*plan["placements"], *plan["open_water"]]:
        x, y = cell["at"]
        i = y * before.width + x
        check(expected[i] == cell["from"], f"previous Brainerd block at {(x, y)}")
        block = after.blocks[i]
        if "deck" in cell:
            deck = definitions[cell["deck"]]
            check(list(layout[16*block:16*block + 16]) == deck["tiles"], f"native deck art at {(x, y)}")
            check(collision[f"{block:02x}"] == deck["collision"], f"native deck footing at {(x, y)}")
            for index, kind in enumerate(deck["collision"]):
                qx, qy = 2*(index % 2), 2*(index // 2)
                ids = {deck["tiles"][(qy+dy)*4 + qx+dx] for dx in (0, 1) for dy in (0, 1)}
                if kind == "WATER":
                    check(ids == {WATER_GLYPH}, "animated water quadrant")
                else:
                    check(kind == "FLOOR" and all(0xc0 <= g <= 0xcf for g in ids), "open timber quadrant")
        else:
            check(block == 0x35, "obsolete stone shoreline remains")
        expected[i] = block
    check(expected == after.blocks, "unexpected Brainerd terrain changes")
    check(glyph(parent, BANK, WATER_GLYPH) == glyph(child, BANK, WATER_GLYPH), "original water ink")
    animations = [key for key in parent["runtime_files"] if key.startswith("gfx/tilesets/water/")]
    check("gfx/tilesets/water/water.2bpp" in animations and
          all(parent["runtime_files"][key] == child["runtime_files"][key] for key in animations),
          "original native water animation frames")
    for source in [*range(0x60, 0x68), 0x6e, 0x6f]:
        check(glyph(parent, "kanto", source) == glyph(child, BANK, source + 0x60),
              f"native timber glyph {source:02x}")
    for x in (47, 48):
        check(all(after.tiles[x, y] == "FLOOR" for y in range(13, 21)), "gap in the pier approach")
        check(after.tiles[x, 12] == "WATER", "blocked north end")
    check(all(after.tiles[x, y] == "WATER" for x in (46, 49) for y in range(13, 20)),
          "blocked fishing edge")
    check((48, 16) in after.occupied and all((47, y) not in after.occupied for y in range(13, 21)),
          "Fisher and passing lane")
    print("Brainerd matches its parent outside fifteen dock and shore blocks. Two timber lanes, "
          "both sides, the north tip, the fisher, and native water animation passed.")


def verify_city_piers(parent, child):
    plans = json.loads((HERE / "city-piers.json").read_text())
    for plan in plans:
        name, bank = plan["map"], plan["tileset"]
        before, after = Terrain(parent, name), Terrain(child, name)
        same_except(before.map, after.map, {"blocks"}, name)
        check(before.tileset == after.tileset == bank, f"{name} tileset")
        metatiles = bytes(child["runtime_files"][f"data/tilesets/{bank}_metatiles.bin"])
        collision = child["data"]["tilesets"][bank]["collision"]
        recipes = {r["name"]: r for r in plan["metatiles"]}
        start = 0xb0 if name == "MnBemidji" else 0xd0
        timber = set(range(start, start + 10))
        for source, dest in zip([*range(0x60, 0x68), 0x6e, 0x6f], range(start, start + 10)):
            check(glyph(parent, "kanto", source) == glyph(child, bank, dest),
                  f"{name} timber {source:02x}")
        expected = list(before.blocks)
        for cell in [*plan["placements"], *plan["shore"]]:
            x, y = cell["at"]
            index = y*before.width + x
            check(expected[index] == cell["from"], f"{name} parent block at {(x, y)}")
            block = after.blocks[index]
            if "deck" in cell:
                recipe = recipes[cell["deck"]]
                tiles = list(metatiles[16*block:16*block+16])
                check(tiles == recipe["tiles"] and collision[f"{block:02x}"] == recipe["collision"],
                      f"{name} installed art and collision at {(x, y)}")
                for n, surface in enumerate(recipe["collision"]):
                    ax, ay = n%2*2, n//2*2
                    ids = {tiles[4*(ay+dy)+ax+dx] for dy in (0, 1) for dx in (0, 1)}
                    if surface == "FLOOR":
                        check(ids <= timber, f"{name} obstruction on deck")
                    else:
                        check(surface == "WATER" and WATER_GLYPH in ids and
                              ids <= ({WATER_GLYPH, 0x3d} if name == "MnBemidji" else {WATER_GLYPH}),
                              f"{name} original water and shoreline at pier")
            else:
                check(block == cell["to"], f"{name} lake edge at {(x, y)}")
            expected[index] = block
        check(expected == after.blocks, f"{name} unexpected terrain edit")
        bounds = plan["deck"]
        left, top, right, bottom = [bounds[k] for k in ("left", "top", "right", "bottom")]
        for x in range(left, right+1):
            check(all(after.tiles[x, y] == "FLOOR" for y in range(top, bottom+1)),
                  f"{name} dry continuous deck")
            check(all(after.tiles[x, y] == "WATER" for y in [top-1, bottom+1]), f"{name} open sides")
        road, water = (left-1, right+1) if plan["open_end"] == "east" else (right+1, left-1)
        for y in range(top, bottom+1):
            check(after.tiles[road, y] == "FLOOR" and after.tiles[water, y] == "WATER",
                  f"{name} entrance and open tip")
        actor = tuple(plan["actor"]["at"])
        check(actor in after.occupied and after.tiles[actor] == "FLOOR", f"{name} dry angler")
        check(all((x, (bottom if actor[1] == top else top)) not in after.occupied for x in range(left, right+1)),
              f"{name} dry passing lane")
        check(glyph(parent, bank, WATER_GLYPH) == glyph(child, bank, WATER_GLYPH),
              f"{name} original lake ripples")
        print(f"{name} installed artwork, dry angler, two-lane approach and every public fishing edge passed.")

    before, after = Terrain(parent, "MnDuluth"), Terrain(child, "MnDuluth")
    check(before.map == after.map, "Duluth high quay and low-cove map")
    dry_approach = after.reach([(47, 59)])
    check(all((x, 57) in dry_approach and after.tiles[x, 57] == "FLOOR"
              and after.tiles[x, 58] == "WATER" for x in (60, 61)),
          "Duluth's low cove must allow two southward launches from the public pier approach")
    check(all(after.tiles[x, 57] == "WALL" for x in (62, 63)),
          "Duluth's boulder marks the east end of the cove")
    check(all(after.tiles[x, y] == "WALL" for x, y in [(46, 62), (49, 62), (47, 64), (48, 64)]),
          "Duluth high quay retains its protective walls")
    print("Duluth's original high quay and signed low harbor launch remain distinct.")


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("parent", type=Path)
    parser.add_argument("candidate", type=Path)
    args = parser.parse_args()
    parent, candidate = load_pack(args.parent), load_pack(args.candidate)
    verify(parent, candidate)
    verify_city_piers(parent, candidate)
