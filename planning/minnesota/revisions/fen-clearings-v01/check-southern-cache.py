#!/usr/bin/env python3
"""Check a southern Fen hidden item using installed terrain, flags, clues and pickup programs."""
import argparse
import json
from pathlib import Path
import runpy
import sys

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE.parents[1] / "tools"))
from audit_story_flow import instructions, native_pages, read_branch, require
from render_reference import load_pack

Trail = runpy.run_path(str(HERE.parent / "northern-trails-v01/check-voyageurs.py"))["Trail"]
CACHE = json.loads((HERE / "southern-cache.json").read_text())
FEN, WOODS = "MnLayeredFen", "MnBemidjiNorthwoods"
PICKUP, HINT = "MnFenSouthSpareBall", "MnFenSouthSpareBallText"
HOME, ORIGINAL_HOME = "MnFenText_hollow_return", "MnFenText_observer_after"
OPEN = "EVENT_MN_FEN_FOLDED_PATH_REVEALED"


def geometry(data):
    trail = Trail(data, (FEN,))
    entry = FEN, 39, 16
    source = (FEN, *CACHE["approach"])
    target = (FEN, *CACHE["tile"])
    require(trail.step(source, "DOWN", dry=True) == target, "dry examination position")
    require([trail.kind(FEN, x, y) for x, y in ((7, 23), (9, 23), (8, 24), (9, 22))]
            == ["WALL", "HEADBUTT_TREE", "WALL", "HEADBUTT_TREE"], "the pocket has one facing A beside two real firs")
    direct = trail.both(entry, source, dry=True)
    southern = [(FEN, x, y) for x, y in ((39, 16), (36, 21), (34, 23), (24, 21), (20, 23), (11, 22), (8, 22))]
    legs = [trail.both(a, b, dry=True) for a, b in zip(southern, southern[1:])]
    require(all(0 < inward == outward for inward, outward in legs), "both ways around the southern shore on foot")
    return direct, sum(a for a, _ in legs)


def compare(parent, candidate):
    before = load_pack(parent)["data"]
    direct, walk = geometry(before)
    former = before["maps"][FEN]
    require(not former["script_field_pickups"] and not before["maps"][WOODS]["script_field_pickups"],
            "the former Fen and adjacent woods should have no competing pickup")
    off = before["maps"]["IlexForest"]
    for label, xy in (("IlexForestHiddenEther", (11, 7)), ("IlexForestHiddenSuperPotion", (22, 14))):
        require(any((e["x"], e["y"]) == xy and e["event_type"] == "BGEVENT_ITEM" and e["script"] == label
                    for e in off["events"]["bg_events"]), f"original Crystal reference {label}")
    if candidate is None:
        print(f"{parent.name}: empty southern pocket, {direct[0]} steps to its sole read square. "
              f"Southern walking itinerary {walk} steps. Crystal's Ilex Forest uses ordinary hidden pickups as reference.")
        return
    data = load_pack(candidate)["data"]
    room = data["maps"][FEN]
    direct, walk = geometry(data)
    x, y = CACHE["tile"]
    item, flag = CACHE["item"], CACHE["flag"]
    old_constants = before["story_event_script_constants"]["global"]
    constants = data["story_event_script_constants"]["global"]
    require(flag not in old_constants and flag in constants
            and all(constants[k] == v for k, v in old_constants.items())
            and [k for k, v in constants.items() if k.startswith("EVENT_") and v == constants[flag]] == [flag],
            "allocate exactly one distinct receipt for the cache without changing old addresses")
    new_events = room["events"]["bg_events"]
    treasure = {"x": x, "y": y, "event_type": "BGEVENT_ITEM", "script": PICKUP}
    require(all(e in new_events for e in former["events"]["bg_events"])
            and [e for e in new_events if e["event_type"] == "BGEVENT_ITEM" or (e["x"],e["y"]) == (x,y)] == [treasure],
        "the six discovery events and the solitary new cache")
    matches = [(name, pickup) for name, other in data["maps"].items() for pickup in other["script_field_pickups"]
               if pickup.get("event_flag") == flag]
    require(len(matches) == 1 and matches[0][0] == FEN and len(room["script_field_pickups"]) == 1,
            "one actual collectible owns the new receipt")
    pickup = matches[0][1]
    require((pickup["source_script"], pickup["command"], pickup["item_id"], pickup["quantity"]) ==
            (PICKUP, "hiddenitem", item, 1), "native Itemfinder pickup")
    wire = [[r["command"], *r["args"]] for r in room["map_event_section_commands"]]
    require(["bg_event", str(x), str(y), "BGEVENT_ITEM", PICKUP] in wire, "wire position")
    require(wire == [[r["command"], *r["args"]]
                     for r in data["map_scripts"][room["attributes"]["map_events_label"]]], "split event section")
    require(instructions(room, PICKUP) == [("hiddenitem", item, flag)]
            and data["map_scripts"][PICKUP] == room["scripts"][PICKUP], "one unconditional native pickup")
    for prior in (False, True):
        active = {OPEN} | ({flag} if prior else set())
        _, pages, after = read_branch(room, "MnFenObserver", active)
        require(pages == ([HOME] if prior else [HOME, HINT]) and after == active,
                "opened observer's spare ball and ordinary return")
    require(room["script_text_bodies"][ORIGINAL_HOME] == former["script_text_bodies"][ORIGINAL_HOME]
            and native_pages(room["script_text_bodies"][HINT]["commands"]) == CACHE["observer_pages"],
            "existing guide after opening and one new clue")
    require(data["map_scripts"][HINT] == room["scripts"][HINT]
            and len([row for row in room["script_flag_commands"] if row["flag_id"] == flag]) == 1,
            "new clue published to split data and hidden from the player once collected")
    print(f"{candidate.name}: {item} at ({x},{y}), native event index {constants[flag]}. "
          f"Dry return {direct}, alternate southern itinerary {walk} steps. All six quest reads and the earlier guide page retained.")


if __name__ == "__main__":
    cli = argparse.ArgumentParser(description=__doc__)
    cli.add_argument("parent", type=Path)
    cli.add_argument("candidate", nargs="?", type=Path)
    args = cli.parse_args()
    compare(args.parent, args.candidate)
