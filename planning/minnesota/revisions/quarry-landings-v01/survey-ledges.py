#!/usr/bin/env python3
"""Report every explicit Minnesota ledge direction against its standing, on-foot landing."""
import argparse
from collections import Counter
from pathlib import Path
import runpy
import sys

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE.parents[1] / "tools"))
from audit_regional_design import DIRS, Terrain

read_pack = runpy.run_path(str(HERE.parent / "ely-return-v01/check-pack.py"))["read_pack"]
SIDES = {"LEFT": "west", "RIGHT": "east", "UP": "north", "DOWN": "south"}


def classify(terrain, landing, direction, stationary):
    kind = terrain.tiles.get(landing)
    if kind is None:
        links = terrain.map["attributes"]["connections"]
        return "declared connection" if any(c["direction"] == SIDES[direction] for c in links) else "map edge"
    if kind in {"WATER", "WALL"}:
        return kind.lower()
    # These are the complete set of walking landings in the surveyed editions.
    # Surface extensions should get an explicit movement check before reporting.
    if kind not in {"FLOOR", "TALL_GRASS", "ICE"}:
        raise ValueError(f"review new terrain {kind} at {terrain.map['id']} {landing} {direction}")
    if landing in stationary:
        return "fixed actor"
    return "open terrain"


def inventory(pack):
    maps = 0
    entries = {}
    for name, room in pack["data"]["maps"].items():
        if not (room["attributes"]["map_group_constant"] or "").startswith("MAPGROUP_MINNESOTA"):
            continue
        maps += 1
        terrain = Terrain(pack, name)
        mounts = [(xy, kind) for xy, kind in terrain.tiles.items() if kind.startswith("HOP_")]
        if not mounts:
            continue
        stationary = {
            (o["x"], o["y"]): o["script"] for o in room["objects"]
            if o["event_flag"] == "-1" and o["spritemovedata"].startswith("SPRITEMOVEDATA_STANDING_")
        }
        for (x, y), kind in mounts:
            # Crystal's corner is a pair of cardinal inputs. Each has its own two-step landing.
            for direction in kind.removeprefix("HOP_").split("_"):
                dx, dy = DIRS[direction]
                landing = (x + 2 * dx, y + 2 * dy)
                status = classify(terrain, landing, direction, stationary)
                entries[name, (x, y), direction] = (landing, status, stationary.get(landing))
    return maps, entries


def survey(pack, details=False):
    maps, entries = inventory(pack)
    mounts = Counter((name, at) for name, at, direction in entries)
    results = Counter(status for landing, status, actor in entries.values())
    if details:
        for key, (landing, status, actor) in entries.items():
            if status != "open terrain":
                print(*key, landing, status, actor or "")
    totals = {"maps": maps, "maps with mounts": len({name for name, at in mounts}),
              "mounts": len(mounts), "corners": sum(n > 1 for n in mounts.values())}
    return totals, dict(results)


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("archives", nargs="+", type=Path)
    parser.add_argument("--details", action="store_true")
    args = parser.parse_args()
    for path in args.archives:
        print(path.name, *survey(read_pack(path), args.details))
