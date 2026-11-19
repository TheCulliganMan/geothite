#!/usr/bin/env python3
"""Read the dock pitches, follow both League approaches, and compare the installed stores."""
from argparse import ArgumentParser
from pathlib import Path
import runpy
import sys

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE.parents[1] / "tools"))
from audit_story_flow import instructions, native_pages
from render_reference import load_pack

rail = runpy.run_path(str(HERE.parent / "headwaters-journey-v01/check-rail.py"))
league = runpy.run_path(str(HERE.parent / "league-journey-v01/check-approach.py"))
ROOM = "MnDuluthDockSupplies"
CLERK = "MN_DULUTH_X_DOCKSUPPLIES_F1_HOST"
CUSTOMER = "MN_DL_INTERIOR_DOCK_CUSTOMER"
SHOP = f"MART_{CLERK}"
OLD = ["FULL_HEAL", "HYPER_POTION", "REVIVE"]
NEW = OLD + ["ESCAPE_ROPE", "POTION"]


def two_way(data, name, start, goal):
    blocked = {rail["coords"](o) for o in data["maps"][name]["objects"]}
    return tuple(rail["paths"](data, name, a, blocked)[b] for a, b in [(start, goal), (goal, start)])


def trace_dock(data, before):
    maps = data["maps"]
    room = maps[ROOM]
    source = maps["MnDuluth"]
    door = rail["find_warp"](source, 61)
    assert (rail["coords"](door), door["target_map_constant"], door["target_warp_id"]) == (
        (12, 77), "MN_DULUTH_X_DOCKSUPPLIES", 3)
    assert rail["quadrant"](data, "MnDuluth", 12, 77) == "DOOR"
    landing = rail["find_warp"](room, 3)
    assert rail["coords"](landing) == (6, 9)
    assert {w["target_warp_id"] for w in room["events"]["warps"] if w["index"] in [1, 2]} == {62}
    assert rail["coords"](rail["find_warp"](source, 62)) == (12, 78)
    assert two_way(data, "MnDuluth", (48, 32), (12, 78)) == (100, 100)
    assert two_way(data, ROOM, (6, 9), (5, 5)) == (5, 5)
    assert two_way(data, ROOM, (6, 9), (7, 7)) == (3, 3)
    assert rail["quadrant"](data, ROOM, 4, 5) == "COUNTER"
    assert {o["script"]: rail["coords"](o) for o in room["objects"]} == {
        CLERK: (3, 5), CUSTOMER: (8, 7)}
    sales = instructions(room, CLERK)
    assert sales[2:5] == [("writetext", f"{CLERK}Text"), ("waitbutton",),
                           ("pokemart", "MARTTYPE_STANDARD", SHOP)]
    assert room["script_shop_commands"] == [
        {"command": "pokemart", "mart_type": "MARTTYPE_STANDARD", "mart_id": SHOP,
         "source_script": CLERK, "command_index": 4}]
    for actor, phrase in [(CLERK, "Buying rope?"), (CUSTOMER, "One POTION for the boat")]:
        label = next(a[1] for a in instructions(room, actor) if a[0] == "writetext")
        shown = " ".join(" ".join(page) for page in native_pages(room["scripts"][label]))
        assert phrase in shown, (actor, shown)
    assert data["marts"][SHOP] == (OLD if before else NEW)
    rope, potion = (data["items"][item] for item in ("ESCAPE_ROPE", "POTION"))
    assert (rope["price"], rope["escape_rope_mode"]) == (550, "DIG_WARP")
    assert (potion["price"], potion["effect"], potion["parameter"]) == (300, "RestoreHPEffect", 20)
    assert all(i["field_usable"] and i["consumable"] for i in (rope, potion))


def preparations(data):
    marts = data["marts"]
    assert set(marts["MART_MN_ELY"]) == set(marts["MART_INDIGO_PLATEAU"])
    assert {"FULL_RESTORE", "MAX_POTION", "FULL_HEAL", "REVIVE", "MAX_REPEL"} <= set(marts["MART_MN_ELY"])
    for destination in [(27, 28), (11, 22)]:
        assert two_way(data, "MnEly", (25, 14), destination) == (22, 22)
    for routes, start, expected in [
        (league["APPROACH"], ("MnBoundaryWaters", 32, 42), (225, 30)),
        (league["SHORE"], ("MnDuluth", 48, 32), (243, 24)),
    ]:
        trail = league["open_trail"](data, routes)
        node, grass = start, 0
        leg = trail.path(start, ("MnEly", 25, 14), dry=True)
        for direction in leg:
            node = trail.step(node, direction, dry=True)
            if trail.kind(*node) == "TALL_GRASS":
                wild = data["wild_encounters"][node[0]]
                assert all(wild["grass_rates"][time] == 10 for time in ["morning", "day", "night"])
                grass += 1
        assert (len(leg), grass) == expected
    print("Ely has the seven Indigo Plateau items. Clinic and Mart each 22 steps from door.")
    print("Boundary–Ely: 225 steps, 30 grass. Duluth–Ely: 243 steps, 24 grass.")


if __name__ == "__main__":
    parser = ArgumentParser(description=__doc__)
    parser.add_argument("gym_walk", type=Path)
    parser.add_argument("cave_returns", type=Path, nargs="?")
    args = parser.parse_args()
    old = load_pack(args.gym_walk)["data"]
    trace_dock(old, True)
    preparations(old)
    if args.cave_returns:
        new = load_pack(args.cave_returns)["data"]
        trace_dock(new, False)
        preparations(new)
        assert new["marts"] == {**old["marts"], SHOP: NEW}
        assert new["items"] == old["items"]
        assert new["maps"][ROOM] == old["maps"][ROOM]
        print("Only the dock's stock changes. Rope 550, Potion 300. Both native speakers and counter intact.")
    else:
        print("Parent reproduced: rope and basic Potion requested here, neither in the clerk’s stock.")
