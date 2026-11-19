#!/usr/bin/env python3
"""Verify the ranger's two conversations and the lower Tamarack Loop discovery."""

import argparse
import json
from pathlib import Path
import sys

REGION = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(REGION / "tools"))
from audit_regional_design import Terrain, same_except
from audit_story_flow import native_pages, read_branch, require
from render_reference import load_pack

RANGER_MAP = "MnR20C00R00"
LOOP = "MnR20C00R01"
RANGER = "MN_R20_RANGER"
CLUE = "MnDiscoveryR20ClueText"
FLAG = "EVENT_MN_HIDDEN_R20"


def check(pack, *, revised):
    data = pack["data"]
    room, loop = (data["maps"][name] for name in (RANGER_MAP, LOOP))
    ranger, = [a for a in room["objects"] if a["script"] == RANGER]
    require((ranger["x"], ranger["y"]) == (55, 43), "ranger moved")
    for trip in (set(), {"EVENT_MN_FALLS_VOYAGEURS_ARRIVAL"},
                 {"EVENT_MN_FALLS_VOYAGEURS_ARRIVAL", "EVENT_MN_BORDER_CIRCUIT_REWARDED"}):
        for claimed in (False, True):
            flags = trip | ({FLAG} if claimed else set())
            _, words, final = read_branch(room, RANGER, flags)
            require(words == ([RANGER + "Text"] if claimed else [CLUE]) and flags == final,
                    f"wrong ranger response or changed flags for {sorted(flags)}")
    body = room["script_text_bodies"][CLUE]["commands"]
    pages = native_pages(body)
    require(pages == native_pages(room["scripts"][CLUE]) == native_pages(data["map_scripts"][CLUE]),
            "split script or native pages disagree")
    if revised:
        canonical = json.loads((REGION / "story.json").read_text())["story_flow_revision"]["dialogue"]
        require(pages == canonical[RANGER_MAP][CLUE], "installed clue differs from the ranger's pages")
        words = " ".join(line for page in pages for line in page)
        require("lower loop" in words and "split boulder" in words and "ice in the crack" in words,
                "ranger no longer points to the frost-split erratic")
    else:
        require("in one print." in sum(pages, []), "expected the original footprint clue")
    item, = [row for row in loop["script_field_pickups"] if row["source_script"] == "MN_HIDDEN_R20"]
    event, = [e for e in loop["events"]["bg_events"] if e["script"] == item["source_script"]]
    require((event["event_type"], event["x"], event["y"]) == ("BGEVENT_ITEM", 33, 9)
            and (item["command"], item["item_id"], item["quantity"], item["event_flag"])
            == ("hiddenitem", "NEVERMELTICE", 1, FLAG), "lower-loop reward or receipt changed")
    terrain = Terrain(pack, LOOP)
    require(terrain.block(33, 9) == 0x86 and terrain.tiles[33, 9] == "WALL"
            and {(33, 10), (34, 9)} <= terrain.reach([(30, 0)]),
            "lower-loop boulder or its two dry search approaches changed")


def compare(parent, child):
    old, new = parent["data"]["maps"], child["data"]["maps"]
    require(old[LOOP] == new[LOOP], "the lower loop or its pickup changed")
    a, b = old[RANGER_MAP], new[RANGER_MAP]
    same_except(a, b, {"scripts", "script_text_bodies"}, "upper Tamarack Loop")
    for key in ("scripts", "script_text_bodies"):
        require(same_except(a[key], b[key], {CLUE}, key) == {CLUE},
                "the single clue should be the only revised page")


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("parent", type=Path)
    parser.add_argument("candidate", type=Path, nargs="?")
    args = parser.parse_args()
    previous = load_pack(args.parent)
    check(previous, revised=False)
    print(f"{args.parent.name}: footprint clue, six ranger states and two dry search approaches.")
    if args.candidate:
        current = load_pack(args.candidate)
        check(current, revised=True)
        compare(previous, current)
        print(f"{args.candidate.name}: split-boulder clue, original reward and all other responses preserved.")
