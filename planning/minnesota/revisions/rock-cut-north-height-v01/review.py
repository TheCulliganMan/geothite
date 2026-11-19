#!/usr/bin/env python3
"""Check the height, native rock art and connected road of Rock Cut's northern entrance."""
import argparse
import base64
import hashlib
import runpy
from pathlib import Path

HERE = Path(__file__).resolve().parent
TRAIL = HERE.parent / "rock-cut-v01"
trail = runpy.run_path(str(TRAIL / "review.py"))
brown = trail["previous"]
caves = runpy.run_path(str(trail["PREVIOUS_CAVES"] / "check-caves.py"))
require, exact_except, records = trail["require"], trail["exact_except"], trail["records"]
NAME = trail["NORTH"]
MANIFEST = "northstar-rock-cut-north-face-v1"


def extent(parent, child):
    """Reconstruct the one permitted map change, including native art and collision mirrors."""
    exact_except(parent, child, {"identity", "report", "data", "runtime_files"}, "pack")
    exact_except(parent["identity"], child["identity"], {"content_hash", "runtime_modpack_id"}, "identity")
    exact_except(parent["report"], child["report"], {"manifests"}, "report")
    require(child["report"]["manifests"] == parent["report"]["manifests"] + [MANIFEST]
            and child["identity"]["runtime_modpack_id"] == parent["identity"]["runtime_modpack_id"] + "+" + MANIFEST,
            "unexpected pack lineage")
    require(child["identity"]["content_hash"] != parent["identity"]["content_hash"], "missing new content identity")
    old, new = parent["data"], child["data"]
    exact_except(old, new, {"maps", "map_blocks", "tilesets"}, "game data")
    exact_except(old["maps"], new["maps"], {NAME}, "maps")
    before, after = old["maps"][NAME], new["maps"][NAME]
    exact_except(before, after, {"blocks"}, NAME)
    attrs = before["attributes"]
    bank = attrs["tileset_name"]
    recipes, room = records(HERE, "art"), records(HERE, "room")
    require(room["map"] == NAME and room["tileset"] == bank and all(r["tileset"] == bank for r in recipes),
            "wrong outdoor room or art bank")
    ids = brown["added_route_art"](parent, child, recipes)
    blocks = list(before["blocks"])
    seen = set()
    for placement in room["placements"]:
        x, y = placement["at"]
        require((x, y) not in seen and 0 <= x < attrs["width"] and 0 <= y < attrs["height"],
                f"repeated or out-of-range placement at {(x,y)}")
        seen.add((x, y))
        i = y*attrs["width"] + x
        old_id, new_id = blocks[i], ids.get((bank, placement["to"]), placement["to"])
        require(old_id == placement["from"] and old_id != new_id, f"stale map placement at {(x,y)}")
        for q in range(4):
            position = (2*x + q%2, 2*y + q//2)
            token, ink = caves["quadrant"](parent, before, *position)
            new_token, new_ink = caves["quadrant"](child, after, *position)
            if token in brown["PRESERVED_TERRAIN"] or new_token in brown["PRESERVED_TERRAIN"]:
                require(token == new_token and ink == new_ink, f"water or habitat changed at {position}")
        blocks[i] = new_id
    require(seen and blocks == after["blocks"], "installed room differs from declared edits")
    key = attrs["blocks_label"]
    exact_except(old["map_blocks"], new["map_blocks"], {key}, "map block mirrors")
    require(new["map_blocks"][key] == base64.b64encode(bytes(blocks)).decode("ascii"), "stale northern map mirror")
    return len(seen)


def glyph_lookup(pack):
    room = pack["data"]["maps"][NAME]
    attrs = room["attributes"]
    width = attrs["width"]
    raw = pack["runtime_files"][f"data/tilesets/{attrs['tileset_name']}_metatiles.bin"]

    def glyph(x, y):
        block = room["blocks"][(y//4)*width + x//4]
        return raw[block*16 + (y%4)*4 + x%4]

    return glyph


def floor_network(pack):
    """Check public routes and prevent an unmarked walk directly through the rock shelf."""
    tiles = trail["terrain"](pack, NAME)
    actors = {(o["x"], o["y"]) for o in pack["data"]["maps"][NAME]["objects"]}
    # The visible CAVE portal enters another map. Its passive return one tile south is a real walking square.
    roles = {"TALL_GRASS": "FLOOR", "BUOY": "WALL", "CAVE": "WALL"}
    edges = trail["graph"]({p: roles.get(role, role) for p, role in tiles.items()}, actors, hops=False)
    approach, roof = (30,28), (30,26)
    reachable = trail["reachable"]
    road = reachable(edges, approach)
    anchors = {(24,28): 6, (37,29): 8, (37,24): 16, (25,13): 20,
               (15,42): 29, (24,0): 34, (28,47): 47}
    for p, limit in anchors.items():
        require(road.get(p, limit+1) <= limit and reachable(edges, p).get(approach, limit+1) <= limit,
                f"lost two-way route from {approach} to {p}")
    # The actual stair squares straddle the rear and southern lips at these three places.
    for x, y in ((25,13), (37,25), (15,41)):
        require((x,y+1) in edges[x,y] and (x,y) in edges[x,y+1], f"severed stair at {(x,y)}")
    cap = {(x,y) for x in range(27,33) for y in range(24,27)}
    require(set(reachable(edges, roof)) == cap and cap.isdisjoint(road),
            "the rock cap leaks onto the public path")
    require(all(tiles[x,y] == "WALL" for x in (26,33) for y in range(24,28)),
            "the side of the outcrop can be crossed")
    require((28,23) not in edges[28,24] and (29,23) not in edges[29,24],
            "an undrawn northbound shortcut crosses the ridge")
    require(tiles[30,27] == "CAVE" and tiles[approach] == "FLOOR" and tiles[30,26] == "FLOOR",
            "entrance, doorstep or stone cap lost")
    return road[37,29], road[24,0], road[28,47]


def verify_face(pack):
    glyph = glyph_lookup(pack)
    # The rear sits against two full trees. These four rows are Crystal's
    # upper crown, lower crown and roots; route walkers cannot cut directly across the cap.
    for x in (54,56):
        require([[glyph(x,y),glyph(x+1,y)] for y in range(44,48)] ==
                [[0x1e,0x1f],[0x2e,0x2f],[0x2e,0x2f],[0x3e,0x3f]],
                f"missing full rooted pine at native column {x}")
    require(all(glyph(53,y) == 0x3b for y in range(48,55))
            and all(glyph(67,y) == 0x3d for y in range(49,55)),
            "the single-course side walls do not meet the front face")
    # Follow the front all the way between its west and east returns. A second
    # south-facing row halfway along a flank made the old eastern toe look taller.
    require(glyph(53,55) == 0x4b and glyph(67,55) == 0x4d, "missing either end of the low rock face")
    for x in range(54,67):
        if x not in (60,61):
            require(glyph(x,55) == 0x4c, f"gap in the continuous face at native column {x}")
    require(all(glyph(x,y) not in {0x4b,0x4c,0x4d}
                for x in range(52,72) for y in range(56,60)),
            "a second south-facing course protrudes into the low approach")
    # Native 8px addresses: the arch occupies rows 54–55. Its jambs span 32px per side.
    for x in (*range(56,60), *range(62,66)):
        require([glyph(x,y) for y in (53,54,55)] == [0x3c,0x3c,0x4c],
                f"wrong face height beside cave door at native column {x}")
    require([glyph(x,y) for y in (54,55) for x in (60,61)] == [0x46,0x47,0x56,0x57],
            "original recessed arch changed")
    room = pack["data"]["maps"][NAME]
    require(caves["check_face"](pack, room, (30,27)) == [4,4], "expected two solid one-course shoulders")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("parent", type=Path)
    parser.add_argument("child", type=Path)
    args = parser.parse_args()
    parents = {records(TRAIL, branch+"-build")["expected_pack_sha256"] for branch in ("field", "mothlight")}
    require(hashlib.sha256(args.parent.read_bytes()).hexdigest() in parents, "expected a Rock Cut Trail parent")
    parent, child = (trail["load_pack"](path) for path in (args.parent, args.child))
    changes = extent(parent, child)
    verify_face(child)
    lane, north, south = floor_network(child)
    caves["check_caves"](child)
    brown["route_connections"](child)
    print(f"{changes} northern placements. The recessed doorway retains 32px of solid, single-course "
          f"rock on either side. The trainer lane is {lane} steps from the entrance. The north/south "
          f"chunk exits are reachable in {north}/{south} steps with all roadside actors present.")


if __name__ == "__main__":
    main()
