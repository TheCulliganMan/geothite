#!/usr/bin/env python3
"""Check the Hollow's two lit surface exits, gallery ladder, trail and FLASH guide in installed packs."""

import argparse
import base64
import json
import runpy
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE.parents[1] / "tools"))
from audit_story_flow import native_pages
from render_reference import load_pack

CAVES = runpy.run_path(str(HERE.parent / "trail-battles-v01/check-caves.py"))
ROCK = runpy.run_path(str(HERE.parent / "rock-cut-v01/review.py"))
ROOM = "MnR01Cave1F"
ROUTE = "MnR01C00R00"
GALLERY = "MnDriftlessLampGallery"
ATLAS = "data/tilesets/cave_metatiles.bin"
PERMISSIONS = "data/tilesets/cave.json"
GIVEN = "MN_R01_HOLLOW_GUIDEFieldHMGivenText"
REPEAT = "MN_R01_HOLLOW_GUIDEText"
DOORS = ((9, 11), (31, 13))


def block(room, x, y):
    return room["blocks"][room["attributes"]["width"] * (y // 2) + x // 2]


def points(rows):
    return {(r["x"], r["y"]) for r in rows}


def unchanged(a, b, except_keys):
    return {k: v for k, v in a.items() if k not in except_keys} == {
        k: v for k, v in b.items() if k not in except_keys
    }


def bank(parent, child):
    old_file, new_file = parent["runtime_files"], child["runtime_files"]
    before, after = bytes(old_file[ATLAS]), bytes(new_file[ATLAS])
    old = parent["data"]["tilesets"]["cave"]
    new = child["data"]["tilesets"]["cave"]
    original_rock = before[0x2E * 16 : 0x2F * 16]
    light = before[0x23 * 16 + 2 : 0x23 * 16 + 4]
    assert original_rock == bytes([0x26]) * 16 and light == bytes([0x1A, 0x1B]), (
        "native cave donors"
    )
    assert after == before + original_rock[:2] + light + original_rock[4:], (
        "daylight spill substrate"
    )
    next_block = len(before) // 16
    expected = dict(old["collision"])
    expected[f"{next_block:02x}"] = ["WALL"] * 4
    assert new["collision"] == expected == json.loads(bytes(new_file[PERMISSIONS])), (
        "typed and raw cave collision"
    )
    assert unchanged(old, new, {"collision"}), "native cave palettes or fields changed"
    cave_files = {
        k for k in old_file.keys() | new_file.keys() if "/cave." in k or "/cave_" in k
    }
    assert all(old_file[k] == new_file[k] for k in cave_files - {ATLAS, PERMISSIONS}), (
        "unrelated cave graphics or animation"
    )
    assert parent["data"]["wild_encounters"] == child["data"]["wild_encounters"], (
        "wild encounter data"
    )
    return next_block


def terrain(parent, child, sill):
    old, new = parent["data"]["maps"][ROOM], child["data"]["maps"][ROOM]
    assert unchanged(old, new, {"blocks", "scripts", "script_text_bodies"}), (
        "unrelated Hollow attributes, rewards, encounters, events or actors"
    )
    expected = list(old["blocks"])
    for x, y in DOORS:
        i = y // 2 * old["attributes"]["width"] + x // 2
        assert [expected[i], expected[i + 20]] == [7, 0x2E], "prior freestanding ladder"
        expected[i], expected[i + 20] = 0x24, sill
    assert (
        expected
        == new["blocks"]
        == list(
            base64.b64decode(
                child["data"]["map_blocks"][new["attributes"]["blocks_label"]]
            )
        )
    ), "installed and split map blocks"
    before, after = ROCK["terrain"](parent, ROOM), ROCK["terrain"](child, ROOM)
    assert {p: (r, after[p]) for p, r in before.items() if r != after[p]} == {
        door: ("LADDER", "WARP_CARPET_DOWN") for door in DOORS
    }, "surface trigger permissions"
    assert parent["data"]["maps"][GALLERY] == child["data"]["maps"][GALLERY], (
        "gallery was changed"
    )
    for x, y in DOORS:
        assert after[x, y - 1] == "FLOOR" and after[x, y + 1] == "WALL", (
            "southward exit"
        )
    assert block(new, 11, 9) == 0x0B and after[11, 9] == "LADDER", (
        "genuine lower-floor ladder"
    )


def warp(room, pos):
    matches = [r for r in room["events"]["warps"] if (r["x"], r["y"]) == pos]
    assert len(matches) == 1, (room["id"], pos)
    return matches[0]


def arrival(pack, name, pos):
    maps = pack["data"]["maps"]
    w = warp(maps[name], pos)
    dest = next(
        n
        for n, m in maps.items()
        if m["attributes"]["map_constant"] == w["target_map_constant"]
    )
    row = maps[dest]["events"]["warps"][w["target_warp_id"] - 1]
    return dest, (row["x"], row["y"])


def paths(pack, name, anchors):
    room = pack["data"]["maps"][name]
    cells = ROCK["terrain"](pack, name)
    # Require the public journey to work both ways on foot without using the one-way ledges.
    dry = {p: "WALL" if role.startswith("HOP_") else role for p, role in cells.items()}
    warps = {p for p in points(room["events"]["warps"]) if cells[p] != "FLOOR"}
    edges = ROCK["graph"](dry, warps | points(room["objects"]))
    for i in anchors:
        reached = ROCK["reachable"](edges, i)
        assert set(anchors) <= reached.keys(), (name, i, set(anchors) - reached.keys())
    return ROCK["reachable"](edges, anchors[0])


def journey(pack):
    for inside, outside in zip(DOORS, ((18, 17), (36, 19))):
        assert arrival(pack, ROUTE, outside) == (ROOM, inside), "surface-to-Hollow"
        landing = (outside[0], outside[1] + 1)
        assert arrival(pack, ROOM, inside) == (ROUTE, landing), "dry surface return"
    assert arrival(pack, ROOM, (11, 9)) == (GALLERY, (18, 17)), "ladder down"
    assert arrival(pack, GALLERY, (19, 17)) == (ROOM, (10, 9)), "ladder up"
    mapset = pack["data"]["maps"]
    cave = mapset[ROOM]
    assert (
        next(o["script"] for o in cave["objects"] if (o["x"], o["y"]) == (5, 5))
        == "MN_R01_HOLLOW_GUIDEFieldHM"
    ), "FLASH guide"
    assert block(cave, 10, 7) == 0x30, "stone steps above the gallery ladder"
    seen = paths(
        pack,
        ROOM,
        [
            (9, 10),
            (31, 12),
            (10, 9),
            (5, 6),
            (10, 7),
            (11, 8),
            (11, 6),
            (28, 6),
            (19, 10),
            (35, 9),
        ],
    )
    assert seen[31, 12] == 26, "public reversible passage"
    gallery = paths(pack, GALLERY, [(18, 17), (19, 10)])
    assert any(p in gallery for p in CAVES["near"]((6, 4))), "fossil read position"
    for pos, role in [((18, 17), "FLOOR"), ((19, 17), "LADDER")]:
        assert CAVES["role"](pack["data"], mapset[GALLERY], pos) == role, (
            "gallery return stance"
        )


def pages(parent, child):
    data = child["data"]
    new, old = data["maps"][ROOM], parent["data"]["maps"][ROOM]
    canon = json.loads((HERE.parents[1] / "story.json").read_text())[
        "dialogue_overhaul"
    ]["texts"][ROOM]
    for key in ("scripts", "script_text_bodies"):
        assert unchanged(old[key], new[key], {GIVEN, REPEAT}), (
            "other cave dialogue changed"
        )
    for label in (GIVEN, REPEAT):
        cmds = new["script_text_bodies"][label]["commands"]
        assert list(native_pages(cmds)) == canon[label], label + " canonical pages"
        assert (
            [{k: v for k, v in c.items() if k != "command_index"} for c in cmds]
            == new["scripts"][label]
            == data["map_scripts"][label]
        ), label + " split dialogue"
        assert new["scripts"][label] != old["scripts"][label], label + " old directions"
    text = " ".join(line for pg in canon[REPEAT] for line in pg)
    assert all(
        phrase in text
        for phrase in (
            "bright gaps",
            "ROUTE 1",
            "ladder below the east steps",
            "shell bed",
            "Bring FLASH",
        )
    ), text
    directs = [
        (c["command"], c["args"]) for c in new["scripts"]["MN_R01_HOLLOW_GUIDEFieldHM"]
    ]
    assert ("checkitem", ["HM_FLASH"]) in directs and (
        "checkflag",
        ["ENGINE_ZEPHYRBADGE"],
    ) in directs
    receipt = directs.index(("verbosegiveitem", ["HM_FLASH", "1"]))
    assert directs[receipt + 1 : receipt + 3] == [
        ("iffalse", ["MN_R01_HOLLOW_GUIDEFieldHMFull"]),
        ("writetext", [GIVEN]),
    ]
    assert new["scripts"]["MN_R01_HOLLOW_GUIDE"] == [
        {"args": [REPEAT], "command": "jumptextfaceplayer"}
    ]


def references(pack):
    data = pack["data"]
    for name, doors in [
        ("UnionCave1F", [(17, 3), (17, 31)]),
        ("DarkCaveVioletEntrance", [(3, 15), (35, 33)]),
        ("MountMortar1FOutside", [(3, 33), (17, 33), (37, 33)]),
    ]:
        room = data["maps"][name]
        for x, y in doors:
            assert warp(room, (x, y))["target_map_constant"].startswith("ROUTE_")
            assert block(room, x, y) == 0x24 and block(room, x, y + 1) == 0x23
    for p in [(5, 19), (3, 33)]:
        assert CAVES["role"](data, data["maps"]["UnionCave1F"], p) == "LADDER"


def check(parent, child):
    sill = bank(parent, child)
    terrain(parent, child, sill)
    journey(child)
    pages(parent, child)
    references(child)
    CAVES["check"](child)
    print(
        "Hollow: 2 lit surface returns, 1 separate gallery ladder, 26-step reversible trail, reachable FLASH guide and fossils; 7 original Crystal doorway/sill pairs."
    )


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("parent", type=Path)
    parser.add_argument("child", type=Path)
    args = parser.parse_args()
    check(load_pack(args.parent), load_pack(args.child))
