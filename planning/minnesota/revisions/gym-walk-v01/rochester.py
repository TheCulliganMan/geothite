#!/usr/bin/env python3
"""Check physical stair cutsets and the first-visit choices in Rochester and Mankato."""

import argparse
import base64
import json
import runpy
from pathlib import Path

HERE = Path(__file__).resolve().parent
BADGES = runpy.run_path(str(HERE.parent / "badge-trails-v01/check-gyms.py"))
ROCK = runpy.run_path(str(HERE.parent / "rock-cut-v01/review.py"))
RUN, COORDS = BADGES["lanes"]["run"], BADGES["coords"]
R, M = "MnRochesterGym", "MnMankatoGym"
SWITCH = "MnMankatoGymV2Gate1Open"


def graph(pack, name, wins, avoid=(), shifted=None):
    room = pack["data"]["maps"][name]
    a = room["attributes"]
    (callback,) = [
        r["args"][1]
        for r in room["map_script_section_commands"]
        if r["command"] == "callback" and r["args"][0] == "MAPCALLBACK_TILES"
    ]
    blocks = room["blocks"].copy()
    flags = {f"EVENT_MN_GYM_{1 if name == R else 2}_TRAINER_{i}_DEFEATED" for i in wins}
    RUN(room, callback, flags, blocks)
    collision = pack["data"]["tilesets"][a["tileset_name"]]["collision"]
    terrain = {
        (x, y): collision[f"{blocks[y // 2 * a['width'] + x // 2]:02x}"][
            2 * (y % 2) + x % 2
        ]
        for y in range(2 * a["height"])
        for x in range(2 * a["width"])
    }
    occupied = {(shifted or {}).get(COORDS(o), COORDS(o)) for o in room["objects"]}
    return ROCK["graph"](terrain, occupied | set(avoid))


def distance(pack, name, wins, start, goal, avoid=(), shifted=None):
    return ROCK["reachable"](graph(pack, name, wins, avoid, shifted), start).get(goal)


def trails(pack, original):
    maps = pack["data"]["maps"]
    start = BADGES["public_door"](pack["data"], maps[R])
    assert start == (12, 15)
    point = (12, 5)
    assert distance(pack, R, [], start, (12, 14)) == 1
    assert distance(pack, R, [], start, point) is None
    west, east = (
        ({(4, 4), (4, 5), (5, 4), (5, 5)}, {(14, 4), (14, 5), (15, 4), (15, 5)})
        if original
        else ({(5, 7)}, {(14, 7)})
    )
    bypass = distance(pack, R, [1], start, point, west | east)
    assert (bypass is not None) == original, (
        "Rochester reaches a high court without either staircase"
    )
    if not original:
        for moved in [
            {},
            {(11, 14): (12, 14)},
            {(16, 7): (15, 7)},
            {(11, 14): (12, 14), (16, 7): (15, 7)},
        ]:
            west_length = 26 if (11, 14) in moved else 24
            assert distance(pack, R, [1], start, point, west, moved) == 14
            assert (
                distance(pack, R, [1], start, point, east | {(15, 7)}, moved)
                == west_length
            )
            assert distance(pack, R, [1], point, start, west, moved) == 14
            assert (
                distance(pack, R, [1], point, start, east | {(15, 7)}, moved)
                == west_length
            )
        optional = next(o for o in maps[R]["objects"] if COORDS(o) == (16, 7))
        assert (
            BADGES["lanes"]["bearing"](optional) == "LEFT" and optional["radius"] == 2
        )
        assert distance(pack, R, [1], start, (14, 7)) is not None
    start = BADGES["public_door"](pack["data"], maps[M])
    assert start == (6, 17)
    assert distance(pack, M, [], start, (3, 16)) is not None
    assert distance(pack, M, [], start, (6, 6)) is None
    west_treads = {(2, 7), (3, 7)}
    assert (distance(pack, M, [1], start, (6, 6), west_treads) is not None) == original
    east_treads = {(12, 5), (13, 5)} if original else {(12, 7), (13, 7)}
    assert (
        distance(pack, M, [1, 2], (12, 8), (11, 4), east_treads) is not None
    ) == original
    assert distance(pack, M, [1], start, (6, 6)) is not None
    for wins in ([1], [1, 2]):
        assert distance(pack, M, wins, (6, 6), (6, 8)) == 1
        assert distance(pack, M, wins, (6, 8), (6, 6)) != 1
        assert distance(pack, M, wins, (6, 8), start) == 8
    assert distance(pack, M, [1], (6, 8), (12, 8)) is not None
    assert distance(pack, M, [1], (12, 8), (11, 4)) is None
    assert distance(pack, M, [1, 2], (12, 8), (11, 4)) is not None
    assert distance(pack, M, [1, 2], (11, 4), start) is not None


def check_delta(parent, child, name):
    old, new = (p["data"]["maps"][name] for p in (parent, child))
    dirty = {"blocks"} if name == R else {"blocks", "scripts", "script_block_changes"}
    assert {k for k in old.keys() | new.keys() if old.get(k) != new.get(k)} == dirty, (
        name
    )
    changes = [
        (x % old["attributes"]["width"], x // old["attributes"]["width"], b, a)
        for x, (b, a) in enumerate(zip(old["blocks"], new["blocks"]))
        if a != b
    ]
    expected = (
        [(2, 2, 23, 22), (7, 2, 23, 24), (2, 3, 7, 25), (7, 3, 1, 26), (2, 4, 6, 7)]
        if name == R
        else [(6, 2, 16, 17), (0, 3, 11, 19), (3, 3, 10, 20), (5, 3, 13, 20)]
    )
    assert changes == expected, (name, changes)
    mirror = child["data"]["map_blocks"][new["attributes"]["blocks_label"]]
    assert bytes(new["blocks"]) == base64.b64decode(mirror)
    key = old["attributes"]["tileset_name"]
    path = f"data/tilesets/{key}_metatiles.bin"
    original, actual = (bytes(p["runtime_files"][path]) for p in (parent, child))
    assert actual[:-32] == original
    before, after = (p["data"]["tilesets"][key] for p in (parent, child))
    n = len(original) // 16
    for new_id in (n, n + 1):
        assert set(actual[new_id * 16 : new_id * 16 + 16]) <= set(original)
    assert after == {**before, "collision": after["collision"]}
    assert after["collision"] == json.loads(
        bytes(child["runtime_files"][f"data/tilesets/{key}.json"])
    )
    assert {k: v for k, v in after["collision"].items() if int(k, 16) < n} == before[
        "collision"
    ]
    for f in parent["runtime_files"]:
        if key in f and f not in (path, f"data/tilesets/{key}.json"):
            assert parent["runtime_files"][f] == child["runtime_files"][f], f
    if name == R:
        return
    assert {k for k in old["scripts"] if old["scripts"][k] != new["scripts"][k]} == {
        SWITCH
    }
    change, end = new["scripts"][SWITCH]
    assert change == {"command": "changeblock", "args": ["12", "6", "16"]} and end == {
        "command": "end",
        "args": [],
    }
    assert new["scripts"][SWITCH] == child["data"]["map_scripts"][SWITCH]
    (entry,) = [c for c in new["script_block_changes"] if c["source_script"] == SWITCH]
    assert (entry["x"], entry["y"], entry["block_id"]) == (12, 6, 16)
    # Both upper lips start on upper sandstone and finish on the lower quarry's stone row.
    assert actual[(n + 1) * 16 : (n + 1) * 16 + 12] == original[17 * 16 : 17 * 16 + 12]
    assert actual[(n + 1) * 16 + 12 : (n + 2) * 16] == original[13 * 16 + 12 : 14 * 16]


if __name__ == "__main__":
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("parent", type=Path)
    ap.add_argument("child", type=Path)
    a = ap.parse_args()
    parent, child = BADGES["load_pack"](a.parent), BADGES["load_pack"](a.child)
    trails(parent, True)
    trails(child, False)
    for name in (R, M):
        check_delta(parent, child, name)
    print(
        "Rochester: west stairs 24 steps, east stairs 14, and no third way onto the court. Mankato: both upper climbs require their treads, the directed quarry drop and eight-step retreat remain available before Jorel. All actors, rewards, sight and warp records match."
    )
