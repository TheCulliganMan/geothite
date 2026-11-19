#!/usr/bin/env python3
"""Check the northern pine gate, first sight of Mitwist and both directions out of its hollow."""

import argparse
from itertools import permutations, product
from pathlib import Path
import runpy
import sys

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE.parents[1] / "tools"))
from audit_story_flow import native_pages, read_branch, require
from render_reference import SourceArt, load_pack

journey = runpy.run_path(str(HERE.parent / "trail-battles-v01/check-fen-journey.py"))
Area, FEN, ENTRY, ACTOR, OPEN, CAUGHT, SPECIES = (journey[k] for k in ("Area", "FEN", "ENTRY", "ACTOR", "OPEN", "CAUGHT", "SPECIES"))
Visit, follow, callbacks = (journey[k] for k in ("Visit", "follow", "callbacks"))
OLD, NEW = ("changeblock", "26", "8", "1"), ("changeblock", "26", "8", "62")
PAGES = {
    "MnFenText_hollow_open": [["The pines part at", "the north end."], ["Young firs shelter", "the hollow beyond."]],
    "MnFenText_hollow_return": [["One wet boot is", "plenty for me."], ["See that gap in", "the north pines?"], ["I'm heading east,", "back to the woods."]],
}
# Full aperture (blocks 13..17; north to south). Numbers refer to original Johto art.
ROWS = [
    "05 05 05 05 05",
    "05 05 5c 3c 62",
    "05 5d 02 3f 05",
    "05 02 3e 05 05",
    "05 05 05 05 05",
]


def shown(trace):
    return [row[1] for row in trace if row[0] in {"jumptext", "writetext"}]


def art(parent, child, room):
    grid = room["blocks"]
    require(room["attributes"]["width"] == 20 and room["attributes"]["tileset_name"] == "johto", "wrong native bank")
    wanted = [[int(k, 16) for k in row.split()] for row in ROWS]
    for y, row in enumerate(wanted, 1):
        require(grid[20*y+13:20*y+18] == row, f"wrong northern foliage in row {y}")
    original = parent["data"]["maps"][FEN]
    require(sum(grid[20*y+x] != original["blocks"][20*y+x] for y in range(1,6) for x in range(13,18)) == 13, "incorrect hollow footprint")
    stock = {v for row in wanted for v in row}
    a, b = SourceArt(parent), SourceArt(child)
    for v in stock:
        before = parent["data"]["tilesets"]["johto"]["collision"][f"{v:02x}"]
        require(child["data"]["tilesets"]["johto"]["collision"][f"{v:02x}"] == before and set(before) <= {"FLOOR", "WALL", "HEADBUTT_TREE"}, "introduced a Cut or other gate")
    for period in ("morn", "day", "nite"):
        previous, actual = a.blocks("johto", period), b.blocks("johto", period)
        require(all(previous[v].tobytes() == actual[v].tobytes() for v in stock), f"{period} hollow differs from original Johto pixels")


def encounter(room):
    for order in permutations(range(3)):
        for aliases in product(range(2), repeat=3):
            visit = Visit()
            for i, feature in enumerate(order):
                traces = follow(room, journey["CONTACTS"][feature][aliases[i]], visit)
                require(traces.count(NEW) == int(i == 2) and (ACTOR in visit.visible) == (i == 2), "opened at wrong observation")
                if i == 2:
                    require(shown(traces)[-1] == "MnFenText_hollow_open", "wrong live opening text")
            require(callbacks(room, visit, True).count(NEW) == 1, "revisit did not restore the opening")
    require(NEW not in callbacks(room, Visit(), True), "new visitor entered early")
    for saved in product((False, True), repeat=3):
        visit = Visit({flag for flag, was_read in zip(journey["READS"], saved) if was_read})
        for i, was_read in enumerate(saved):
            if not was_read:
                follow(room, journey["CONTACTS"][i][0], visit)
        if all(saved):
            follow(room, "MnFenObserver", visit)
        require(OPEN in visit.flags and NEW in callbacks(room, visit, True), "partial save cannot finish")
    for caught in (False, True):
        visit = Visit({OPEN})
        again = callbacks(room, visit, True)
        require(NEW in again and ACTOR in visit.visible, "lost a previously revealed hollow")
        refused = follow(room, ACTOR, visit)
        require(ACTOR in visit.visible and ("startbattle",) not in refused and visit.flags == {OPEN}, "declining closed encounter")
        result = follow(room, ACTOR, visit, yes=True, caught=caught)
        require(("loadwildmon", "MITWIST", "55") in result and result.count(("startbattle",)) == 1, "wrong wild partner")
        require((CAUGHT in visit.flags) == caught and ACTOR not in visit.visible, "wrong battle outcome")
        require(NEW in callbacks(room, visit, True) and (ACTOR in visit.visible) != caught, "wrong return after catch or escape")
    for label, pages in PAGES.items():
        require(native_pages(room["script_text_bodies"][label]["commands"]) == pages, f"wrong native {label}")
        require(all(len(line) <= 18 for page in pages for line in page), "overflowing text page")
    (cache,) = room["script_field_pickups"]
    flag = cache["event_flag"]
    for found in (False, True):
        _, texts, _ = read_branch(room, "MnFenObserver", {OPEN} | ({flag} if found else set()))
        require(texts == ["MnFenText_hollow_return"] + ([] if found else ["MnFenSouthSpareBallText"]), "wrong observer directions or lost extra clue")


def geometry(data, room):
    closed, opened, after = Area(data, FEN), Area(data, FEN, True), Area(data, FEN, True, True)
    require(room["script_block_changes"] == [{"x":26, "y":8, "block_id":62, "source_script":"MnFenRestoreTerrain", "command_index":2}], "stale tile-change index")
    require(OLD not in follow(room, "MnFenRestoreTerrain", Visit({OPEN})), "old gravel unexpectedly won")
    for x in (26,27):
        for y in (8,9):
            require(closed.kind((x,y)) == "WALL", "closed gate can be entered")
            require(opened.kind((x,y)) == ("HEADBUTT_TREE" if (x,y)==(26,9) else "FLOOR"), "wrong open shoulder")
    require(closed.kind((32,4)) == "HEADBUTT_TREE" and closed.kind(SPECIES) == "FLOOR", "Mitwist is not sitting at the pine roots")
    require(not journey["adjacent"](SPECIES) & closed.paths(ENTRY, True).keys(), "premature walk or Surf into the hollow")
    west = (31,5)
    alternatives = journey["adjacent"](SPECIES) & opened.paths(ENTRY).keys()
    require(west in alternatives and len(alternatives) >= 2, "no legible encounter stance")
    door = (27,8)
    for from_ in (door, west, *alternatives):
        require(from_ in opened.paths(ENTRY) and ENTRY in opened.paths(from_), "lost returning path")
    require(door in after.paths(SPECIES) and SPECIES in after.paths(door), "capture blocks the clearing")
    require(opened.actors == closed.actors | {SPECIES} and after.actors == closed.actors, "wrong walking actors")
    newly_open = opened.paths(ENTRY).keys() - closed.paths(ENTRY).keys()
    print(f"The forest gate to Mitwist's western stance takes {opened.paths(door)[west]} dry steps. Interaction positions {sorted(alternatives)}. Opening adds {len(newly_open)} walker squares. The closed hollow resists Surf; after capture the floor stays public.")


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("parent", type=Path)
    parser.add_argument("candidate", type=Path)
    args = parser.parse_args()
    parent, child = load_pack(args.parent), load_pack(args.candidate)
    room = child["data"]["maps"][FEN]
    art(parent, child, room)
    geometry(child["data"], room)
    formerly = parent["data"]["maps"][FEN]
    for label in ("MnFenText_folded_path", "MnFenText_observer_after"):
        require(room["scripts"][label] == formerly["scripts"][label], "changed historical page")
        require(not any(
            label in instruction["args"] for body in room["scripts"].values() for instruction in body
        ), f"stale live page reference {label}")
    for label in (*PAGES, "MnFenTrailOpened", "MnFenObserverAfter", "MnFenSouthBallFound", "MnFenRestoreTerrain"):
        require(room["scripts"][label] == child["data"]["map_scripts"][label], "stale global script " + label)
    encounter(room)
    print("13 original Johto block placements at three times of day; all 48 observation orders and aliases, cache directions, decline, capture, escape and revisit.")
