#!/usr/bin/env python3
"""Check both League approaches, the original Cut reward and the final snowy grove."""

import argparse
import base64
import json
from pathlib import Path
import runpy
import sys

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE.parents[1] / "tools"))
from audit_story_flow import BADGES, instructions
from render_reference import SourceArt, load_pack

Trail = runpy.run_path(str(HERE.parent / "northern-trails-v01/check-voyageurs.py"))[
    "Trail"
]
FINAL = "MnR13C00R02"
ELY = "MnEly"
CAMP = "MnBoundaryWaters"
BANK = "mn_winter_north_shore"
ATLAS = f"data/tilesets/{BANK}_metatiles.bin"
APPROACH = {CAMP, "MnR13C00R00", "MnR13C00R01", FINAL, ELY}
SHORE = {"MnDuluth", "MnR11C00R03", "MnR11C00R02", "MnElyShorePass", ELY}


def open_trail(data, names):
    trail = Trail(data, names)
    # The two attendants actually step aside when approached. Other outdoor actors stay in place.
    for name, label in [
        (CAMP, "MnBoundaryLeagueCheck"),
        ("MnDuluth", "MnDuluthShoreCheck"),
    ]:
        if name not in names:
            continue
        room = data["maps"][name]
        dismissed = [
            cmd[1] for cmd in instructions(room, label) if cmd[0] == "disappear"
        ]
        guards = [o for o in room["objects"] if o["script"] == label]
        assert len(guards) == 2 and {o["object_identifier"] for o in guards} == set(
            dismissed
        )
        assert instructions(room, label) == [("disappear", g) for g in dismissed] + [
            ("end",)
        ]
        for o in guards:
            trail.bodies[name].remove((o["x"], o["y"]))
    return trail


def cannot_reach(trail, start, end):
    try:
        trail.path(start, end, dry=True)
    except ValueError:
        return True
    return False


def approaches(pack, revised):
    data = pack["data"]
    gym = instructions(data["maps"]["MnDuluthGym"], "MnDuluthGymLeader")
    assert gym.index(("startbattle",)) < gym.index(("setflag", "ENGINE_RISINGBADGE"))
    door = next(
        w
        for w in data["maps"]["MnDuluth"]["events"]["warps"]
        if (w["x"], w["y"]) == (48, 31)
    )
    assert door["target_map"] == "MN_DULUTH_GYM"
    assert any(
        w["target_map"] == "MN_DULUTH" and w["target_warp_id"] == 4
        for w in data["maps"]["MnDuluthGym"]["events"]["warps"]
    )
    return_tile = data["maps"]["MnDuluth"]["events"]["warps"][3]
    assert (return_tile["index"], return_tile["x"], return_tile["y"]) == (4, 48, 32)
    route = open_trail(data, APPROACH)
    by_shore = open_trail(data, SHORE)
    room = data["maps"][FINAL]
    finish = (FINAL, 34, 31)
    north = (CAMP, 32, 42)
    assert route.both(north, finish, dry=True) == (195, 195)
    for x in (34, 35):
        town = (ELY, x - 10, 0)
        assert route.step((FINAL, x, 31), "DOWN") == town
        assert route.step(town, "UP") == (FINAL, x, 31)
        for door_x in (25, 26):
            apron = (ELY, door_x, 14)
            assert all(n <= 32 for n in route.both(town, apron, dry=True))
            assert all(
                n <= 245 for n in by_shore.both(("MnDuluth", 48, 32), apron, dry=True)
            )
            assert route.kind(ELY, door_x, 13) == "DOOR"
            lodge = next(
                w
                for w in data["maps"][ELY]["events"]["warps"]
                if (w["x"], w["y"]) == (door_x, 13)
            )
            assert lodge["target_map"] == "MN_LEAGUE_LOBBY"
    false_lanes = [
        x
        for x in range(64)
        if route.kind(FINAL, x, 31) == "FLOOR"
        and route.step((FINAL, x, 31), "DOWN") is None
    ]
    assert false_lanes == ([] if revised else [30, 31, 32, 33])
    for x in (42, 43):
        for y in range(18, 26):
            assert cannot_reach(route, finish, (FINAL, x, y))
            assert route.kind(FINAL, x, y) == ("WALL" if revised else "FLOOR")
    for x in (28, 29):
        assert route.kind(FINAL, x, 16) == "HEADBUTT_TREE"
        stance = (FINAL, x, 17)
        if revised:
            assert max(route.both(finish, stance, dry=True)) <= 30
        else:
            assert cannot_reach(route, finish, stance)
    assert all(route.kind(FINAL, x, 21) == "FLOOR" for x in range(33, 37))
    tomas = next(o for o in room["objects"] if o["script"] == "MN_R13_T10")
    assert (
        tomas["x"],
        tomas["y"],
        tomas["sightline_direction_override"],
        tomas["radius"],
    ) == (37, 21, "LEFT", 4)
    assert route.both(finish, (FINAL, 36, 21), dry=True)

    # Ivy and the Revive remain in a separate grove reachable only through the original Cut tree.
    assert route.kind(FINAL, 18, 7) == "CUT_TREE"
    item = next(o for o in room["objects"] if o["script"] == "MN_R13_ITEM_04")
    assert (item["x"], item["y"]) == (21, 13)
    assert ("itemball", "REVIVE", "1") in instructions(room, item["script"])
    assert route.both(finish, (FINAL, 18, 6), dry=True)
    assert cannot_reach(route, finish, (FINAL, 20, 13))
    block = room["blocks"][3 * 32 + 9]
    replacement = data["field_moves"]["cut"]["replacements"][BANK][block][
        "replacement_block_id"
    ]
    roles = data["tilesets"][BANK]["collision"][f"{replacement:02x}"]
    for q, role in enumerate(roles):
        route.grid[FINAL][6 + q // 2][18 + q % 2] = role
    assert route.both(finish, (FINAL, 20, 13), dry=True)
    assert route.both(finish, (FINAL, 18, 11), dry=True)
    register = instructions(data["maps"]["MnLeagueLobby"], "MnElyRegister")
    assert {c[1] for c in register if c[0] == "checkflag"} == BADGES
    assert ("checkevent", "EVENT_MN_SURVEY_RESTORED") in register
    assert ("yesorno",) in register and (
        "setevent",
        "EVENT_MN_LEAGUE_RUN_ACTIVE",
    ) in register


def compare(parent, child):
    before, after = (p["data"]["maps"][FINAL] for p in (parent, child))
    assert {k: v for k, v in before.items() if k != "blocks"} == {
        k: v for k, v in after.items() if k != "blocks"
    }
    old, new = (bytes(p["runtime_files"][ATLAS]) for p in (parent, child))
    slot = len(old) // 16
    assert (
        len(old) % 16 == 0
        and new == old + old[0x89 * 16 : 0x89 * 16 + 8] + old[0x83 * 16 + 8 : 0x84 * 16]
    )
    changes = {(14, 8): (0x5D, slot), (14, 9): (0x8B, 0x83), (20, 12): (0x0A, 0x8B)}
    changes.update({(21, y): (0x83, 0x8B) for y in range(9, 13)})
    changes.update({xy: (0x83, 0x8B) for xy in [(15, 14), (15, 15), (16, 15)]})
    assert len(before["blocks"]) == len(after["blocks"]) == 32 * 16
    assert {
        (i % 32, i // 32): (a, b)
        for i, (a, b) in enumerate(zip(before["blocks"], after["blocks"]))
        if a != b
    } == changes
    label = after["attributes"]["blocks_label"]
    assert base64.b64decode(child["data"]["map_blocks"][label]) == bytes(
        after["blocks"]
    )
    a, b = (p["data"]["tilesets"][BANK] for p in (parent, child))
    assert b == {
        **a,
        "collision": {**a["collision"], f"{slot:02x}": a["collision"]["5d"]},
    }
    assert (
        json.loads(bytes(child["runtime_files"][f"data/tilesets/{BANK}.json"]))
        == b["collision"]
    )
    for key in parent["runtime_files"]:
        if BANK in key and key not in (ATLAS, f"data/tilesets/{BANK}.json"):
            assert parent["runtime_files"][key] == child["runtime_files"][key], key
    for key in ("wild_encounters", "field_encounters", "field_moves"):
        assert parent["data"][key] == child["data"][key], key
    trees = child["data"]["field_encounters"][FINAL]["tables"]["headbutt"]
    assert {s["level"] for pool in trees.values() for s in pool} == {46}
    drawn = SourceArt(child)
    check_town_trailhead(parent, child, drawn)
    check_shore_pass(parent, child)
    for clock in ("morn", "day", "nite"):
        crown = drawn.blocks("mn_winter_headwaters_stairs", clock)[0x5D]
        bank = drawn.blocks(BANK, clock)
        assert (
            bank[slot].crop((0, 0, 32, 16)).tobytes()
            == crown.crop((0, 0, 32, 16)).tobytes()
        )
        assert (
            bank[slot].crop((0, 16, 32, 32)).tobytes()
            == bank[0x83].crop((0, 16, 32, 32)).tobytes()
        )


def check_shore_pass(parent, child):
    name = "MnElyShorePass"
    guide = name + "TrailGuide"
    new_label = "MnAfterChampionShoreGuideText"
    old, new = (p["data"]["maps"][name] for p in (parent, child))
    for key, previous in old.items():
        if key in ("scripts", "script_text_bodies"):
            allowed = {new_label, guide} if key == "scripts" else {new_label}
            assert {k: v for k, v in previous.items() if k not in allowed} == {
                k: v for k, v in new[key].items() if k not in allowed
            }
            assert new_label not in previous and new_label in new[key]
        elif key == "script_text_commands":
            assert new[key] == [
                {**row, "text_label": new_label}
                if row["source_script"] == guide
                else row
                for row in previous
            ]
        else:
            assert new[key] == previous, key
    assert instructions(new, guide) == [("jumptextfaceplayer", new_label)]
    assert all(
        new["scripts"][label] == child["data"]["map_scripts"][label]
        for label in (new_label, guide)
    )


def check_town_trailhead(parent, child, art):
    old, new = (p["data"]["maps"][ELY] for p in (parent, child))
    slot = new["blocks"][12]
    assert old["blocks"][12] == 0x31 and slot != 0x31
    assert new["blocks"] == old["blocks"][:12] + [slot] + old["blocks"][13:]
    assert base64.b64decode(
        child["data"]["map_blocks"][new["attributes"]["blocks_label"]]
    ) == bytes(new["blocks"])
    name = "mn_ely_snow"
    old_bank, bank = (p["data"]["tilesets"][name] for p in (parent, child))
    assert bank["collision"] == {**old_bank["collision"], f"{slot:02x}": ["FLOOR"] * 4}
    files = child["runtime_files"]
    base = f"data/tilesets/{name}"
    assert json.loads(bytes(files[base + ".json"])) == bank["collision"]
    assert json.loads(bytes(files[base + "_palette_map.json"])) == bank["palette_map"]
    atlas = bytes(files[base + "_metatiles.bin"])
    assert atlas[: slot * 16] == bytes(parent["runtime_files"][base + "_metatiles.bin"])
    assert len(atlas) == (slot + 1) * 16
    used = set(atlas[slot * 16 :]) - {0x39}
    assert used == set(range(0x66, 0x70)) and used.isdisjoint(atlas[: slot * 16])
    expected_pal = old_bank["palette_map"].copy()
    for i in used:
        expected_pal[i] = 0
    assert bank == {
        **old_bank,
        "collision": bank["collision"],
        "palette_map": expected_pal,
    }

    # Read each actual rendered column independently of the author's mask.
    # Snow continues across the map boundary, then recedes towards the center
    # of the road. Every pixel touches its original texture in world phase.
    for clock in ("morn", "day", "nite"):
        road = art.blocks(name, clock)[0x31]
        snow = art.blocks(BANK, clock)[0x83]
        mixed = art.blocks(name, clock)[slot]
        road_colors = {c for _, c in road.getcolors()}
        depths = []
        for x in range(32):
            depth = next(y for y in range(32) if mixed.getpixel((x, y)) in road_colors)
            depths.append(depth)
            for y in range(32):
                actual = mixed.getpixel((x, y))
                if y >= depth:
                    assert actual == road.getpixel((x, y))
                else:
                    original = snow.getpixel((x, y))
                    assert actual == original or (original, actual) == (
                        (181, 214, 222),
                        (206, 231, 239),
                    )
        assert depths[0] == 23 and min(depths) == 7 and depths[-1] == 14
        assert max(abs(a - b) for a, b in zip(depths, depths[1:])) <= 1
        bottom = next(i for i, n in enumerate(depths) if n == min(depths))
        assert depths[: bottom + 1] == sorted(depths[: bottom + 1], reverse=True)
        assert depths[bottom:] == sorted(depths[bottom:])
        for x in range(0, 32, 8):
            for y in range(0, 32, 8):
                assert len(mixed.crop((x, y, x + 8, y + 8)).getcolors()) <= 4


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("parent", type=Path)
    parser.add_argument("child", nargs="?", type=Path)
    args = parser.parse_args()
    parent = load_pack(args.parent)
    approaches(parent, False)
    if args.child:
        child = load_pack(args.child)
        compare(parent, child)
        approaches(child, True)
        print(
            "Both Ely lanes match, 16 sealed snow squares removed, and both Lv46 trees accessible. "
            "Native winter crowns and the four-color melting road verified at three times of day."
        )
    print(
        "Both Lodge approaches, return trips, Tomas, Cut and registration checked against installed maps."
    )


if __name__ == "__main__":
    main()
