#!/usr/bin/env python3
"""Resolve each surface cave entrance, active indoor exit, outdoor facing and public return."""

import argparse
import json
import runpy
import sys
from collections import Counter, deque
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / "tools"))
from render_reference import load_pack

MOUTHS = ROOT / "revisions/shore-rocks-cave-mouths-v01"
PUBLIC = runpy.run_path(str(ROOT / "revisions/crystal-cave-faces-v01/review.py"))[
    "public_approaches"
]
ROLES = {"FLOOR", "TALL_GRASS", "LONG_GRASS", "LADDER", "WARP_CARPET_DOWN"}
NEIGHBORS = ((0, -1), (1, 0), (0, 1), (-1, 0))


def require(ok, message):
    if not ok:
        raise ValueError(message)


def point(w):
    return w["x"], w["y"]


def near(p):
    return [(p[0] + x, p[1] + y) for x, y in NEIGHBORS]


def role(data, room, p):
    x, y = p
    a = room["attributes"]
    if not 0 <= x < a["width"] * 2 or not 0 <= y < a["height"] * 2:
        return "off-map"
    b = room["blocks"][y // 2 * a["width"] + x // 2]
    return data["tilesets"][a["tileset_name"]]["collision"][f"{b:02x}"][
        2 * (y % 2) + x % 2
    ]


def access(data, room, arrival, exit_at):
    """Find an ordinary interior step off the arrival and a legal return to this particular exit."""
    occupied = {point(o) for o in room["objects"]}
    doors = {
        point(w)
        for w in room["events"]["warps"]
        if role(data, room, point(w)) == "LADDER"
    }
    q = deque([arrival])
    seen = {arrival}
    while q:
        p = q.popleft()
        for nxt in near(p):
            if nxt not in seen | doors | occupied and role(data, room, nxt) in ROLES:
                seen.add(nxt)
                q.append(nxt)
    north, east, south, west = near(exit_at)
    r = role(data, room, exit_at)
    if r == "WARP_CARPET_DOWN":
        require(
            role(data, room, south) in {"WALL", "off-map"},
            room["id"] + " outward carpet needs a south bump",
        )
        require(
            north in seen, room["id"] + " cannot approach return carpet from inside"
        )
    else:
        require(
            r == "LADDER"
            and any(p in seen - {exit_at} for p in (north, east, south, west)),
            room["id"] + " cannot step onto return ladder",
        )
    require(len(seen) >= 3, room["id"] + " cannot leave arrival")


def check(pack):
    d = pack["data"]
    maps = d["maps"]
    names = {m["attributes"]["map_constant"]: n for n, m in maps.items()}
    inventory = json.loads((MOUTHS / "cave-checks.json").read_text())["mouths"]
    expected = {(n["map"], tuple(n["mouth"])): n for n in inventory}
    discoveries = set()
    returns = Counter()
    for name, room in maps.items():
        if not name.startswith("Mn") or room["attributes"]["environment"] not in {
            "ROUTE",
            "TOWN",
        }:
            continue
        for w in room["events"]["warps"]:
            if role(d, room, point(w)) != "CAVE":
                continue
            pos = point(w)
            discoveries.add((name, pos))
            expected_row = expected.get((name, pos))
            require(expected_row, name + " unclassified cave")
            doorstep = (pos[0], pos[1] + 1)
            inside = maps[names[w["target_map_constant"]]]
            require(
                inside["id"] == expected_row["interior"], name + " cave destination"
            )
            e = inside["events"]["warps"][w["target_warp_id"] - 1]
            entry = point(e)
            found = []
            for ex in inside["events"]["warps"]:
                if names[ex["target_map_constant"]] != name or role(
                    d, inside, point(ex)
                ) not in {"WARP_CARPET_DOWN", "LADDER"}:
                    continue
                target = room["events"]["warps"][ex["target_warp_id"] - 1]
                actual = point(target)
                if actual not in (pos, doorstep):
                    continue
                access(d, inside, entry, point(ex))
                require(
                    role(d, room, doorstep) in {"FLOOR", "TALL_GRASS"}
                    and doorstep not in {point(o) for o in room["objects"]},
                    name + " occupied return landing",
                )
                # CAVE forces DOWN on the next input frame; a FLOOR anchor already stands clear.
                require(
                    role(d, room, actual)
                    == ("CAVE" if actual == pos else role(d, room, doorstep)),
                    name + " unexpected arrival behavior",
                )
                found.append(actual)
            require(
                len(found) == 1,
                f"{name}{pos}: expected one active indoor exit, found {found}",
            )
            returns["automatic south step" if found[0] == pos else "dry landing"] += 1
    require(discoveries == expected.keys(), "outdoor cave inventory differs")
    require(PUBLIC(pack) == 15, "public routes to cave entrances and fossil stair")
    for name, tiles, inside, xy in [
        ("Route42", (10, 5), "MountMortar1FOutside", (3, 33)),
        ("Route45", (2, 5), "DarkCaveBlackthornEntrance", (23, 3)),
        ("AzaleaTown", (31, 7), "SlowpokeWellB1F", (17, 15)),
    ]:
        r = maps[name]
        w = next(v for v in r["events"]["warps"] if point(v) == tiles)
        other = maps[inside]
        require(
            names[w["target_map_constant"]] == inside and role(d, r, tiles) == "CAVE",
            name + " reference cave",
        )
        require(
            point(other["events"]["warps"][w["target_warp_id"] - 1]) == xy
            and role(d, other, xy) in {"LADDER", "WARP_CARPET_DOWN"},
            inside + " reference arrival",
        )
    print(
        f"{len(discoveries)} surface mouths, {returns['dry landing']} direct dry returns, {returns['automatic south step']} automatic south step, "
        "15 public approaches including the fossil stair, and 3 original Crystal references."
    )


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("pack", type=Path)
    check(load_pack(parser.parse_args().pack))
