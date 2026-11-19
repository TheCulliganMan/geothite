#!/usr/bin/env python3
"""Verify the directions a visitor can read and walk between St. Paul and the State Fair."""

import argparse
import itertools
import json
from pathlib import Path
import runpy
import sys

HERE = Path(__file__).resolve().parent
REGION = HERE.parents[1]
sys.path.insert(0, str(REGION / "tools"))
from audit_story_flow import native_pages, read_branch, require
from render_reference import load_pack

# Reuse the northern trails' collision/actor/connection-offset walk with the fair's maps.
Trail = runpy.run_path(str(HERE.parent / "northern-trails-v01" / "check-voyageurs.py"))["Trail"]
CITY = "MnStPaul"
METRO = "MnMinneapolis"
ROUTE4 = "MnR04C00R00"
ROUTE5 = "MnR05C00R02"
FAIR = "MnStateFairgrounds"
OFFICE = "MnStPaulFairInfoHouse"
CAPITOL = "MnStPaulCapitolHall"
CONTACTS = {
    CITY: {
        "MN_ST_PAUL_FAIR_GUARD_WEST": (28, 2),
        "MN_ST_PAUL_FAIR_GUARD_EAST": (29, 2),
        "MN_ST_PAUL_FAIR_STEWARD": (30, 7),
        "MN_ST_PAUL_SIGN_FAIR": (31, 7),
        "MN_ST_PAUL_CIVIC_GUIDE": (43, 13),
    },
    OFFICE: {"MN_SPX_FAIRINFOHOUSE_HOST": (5, 3)},
    CAPITOL: {"MN_ST_PAUL_RECORDS_CLERK": (4, 2)},
    ROUTE5: {"MN_R05_SOUTH_SIGN": (24, 10)},
}
FLAGS = [
    "EVENT_MN_DISPATCH_NOTE", "EVENT_MN_COURIER_DEFEATED",
    "EVENT_MN_DEEP_BORE_MANIFEST_READ", "EVENT_MN_BRAINERD_REPORT",
]


def check_dialogue(parent, child):
    canonical = json.loads((REGION / "story.json").read_text())["dialogue_overhaul"]["texts"]
    for name, contacts in CONTACTS.items():
        before, after = (pack["data"]["maps"][name] for pack in (parent, child))
        labels = {entry + "Text" for entry in contacts}
        require(before.keys() == after.keys(), f"{name} room schema")
        for key, value in before.items():
            if key in {"scripts", "script_text_bodies"}:
                other = after[key]
                require(value.keys() == other.keys(), f"{name}/{key} labels")
                require({k for k in value if value[k] != other[k]} == labels,
                        f"{name}/{key} expected exactly {sorted(labels)}")
            else:
                require(value == after[key], f"{name} changed {key}")
        sites = {o["script"]: (o["x"], o["y"]) for o in after["objects"]}
        sites.update({e["script"]: (e["x"], e["y"]) for e in after["events"]["bg_events"]
                      if e["event_type"] == "BGEVENT_READ"})
        for entry, position in contacts.items():
            label = entry + "Text"
            require(sites.get(entry) == position, f"{name} {entry} moved")
            pages = native_pages(after["script_text_bodies"][label]["commands"])
            require(pages == canonical[name][label], f"{label} native page breaks")
            require(all(1 <= len(p) <= 2 and all(l.isascii() and 1 <= len(l) <= 18 for l in p)
                        for p in pages), f"{label} printer limits")
            require(native_pages(after["scripts"][label]) == pages
                    and child["data"]["map_scripts"][label] == after["scripts"][label],
                    f"{label} split map text")
            observed = set()
            for bits in itertools.product((False, True), repeat=len(FLAGS)):
                flags = {flag for flag, bit in zip(FLAGS, bits) if bit}
                actions, spoken, final = read_branch(after, entry, flags)
                require((actions, spoken, final) == read_branch(before, entry, flags)
                        and final == flags, f"{entry} changed state selection or progress")
                observed.update(spoken)
            expected = {label}
            if entry == "MN_ST_PAUL_CIVIC_GUIDE":
                expected.update({"MnWayDepotDateText", "MnWayDepotMinerText", "MnWayDepotMinaText"})
            require(observed == expected, f"{entry} unreachable or unreviewed text {observed}")

    messages = {entry: " ".join(word for page in canonical[name][entry + "Text"] for word in page)
                for name, contacts in CONTACTS.items() for entry in contacts}
    for entry in ["MN_ST_PAUL_FAIR_GUARD_WEST", "MN_ST_PAUL_FAIR_GUARD_EAST",
                  "MN_ST_PAUL_FAIR_STEWARD", "MN_ST_PAUL_SIGN_FAIR", "MN_SPX_FAIRINFOHOUSE_HOST"]:
        words = messages[entry].lower()
        require(all(t in words for t in ("west", "minneapolis", "route 5", "north")), entry)
        require("left" in words or "west fork" in words or "turn west" in words, f"{entry} fair turn")
    words = messages["MN_R05_SOUTH_SIGN"]
    require(all(t in words for t in ("WEST TO STATE FAIR", "NORTH TO ST CLOUD", "SOUTH TO MINNEAPOLIS")),
            "Route 5 crossroads")


def check_trail(pack):
    data = pack["data"]
    rooms = data["maps"]
    trail = Trail(data, {CITY, METRO, ROUTE4, ROUTE5, FAIR, OFFICE, CAPITOL})

    def walk(name, a, b):
        require(trail.kind(name, *a) != "WALL" and trail.kind(name, *b) != "WALL", "solid walk anchor")
        return trail.both((name, *a), (name, *b), dry=True)

    def touch(name, a, direction, b):
        require(trail.step(a, direction, dry=True) == b, f"{name} broken {direction}: {a} to {b}")

    def clear_officers(script, observer):
        guards = [o for o in rooms[METRO]["objects"] if o["script"] == script]
        ids = [o["object_identifier"] for o in guards]
        require(len(guards) == 2 and len(set(ids)) == 2, f"{script} road officers")
        require({(o["x"], o["y"]) for o in guards} <= trail.bodies[METRO], f"{script} was open")
        require(any(abs(x-observer[0]) + abs(y-observer[1]) == 1
                    for x, y in [(o["x"], o["y"]) for o in guards]), f"{script} out of earshot")
        require([c["command"] for c in rooms[METRO]["scripts"][script]] == ["disappear", "disappear", "end"]
                and {c["args"][0] for c in rooms[METRO]["scripts"][script][:2]} == set(ids),
                f"{script} must open by talking without a badge, fare, or event")
        for guard in guards:
            trail.bodies[METRO].remove((guard["x"], guard["y"]))

    # The north annex is capped. Both guards, the steward, and the sign are accessible from town.
    require("north" not in trail.conns[CITY] and all(trail.kind(CITY, x, 0) == "WALL" for x in (28, 29)),
            "closed annex")
    for stand in ((28, 3), (29, 3), (29, 7), (31, 6), (42, 13)):
        walk(CITY, stand, (0, 48))
    for door, room, actor_stand in [((11, 9), OFFICE, (5, 4)), ((40, 11), CAPITOL, (4, 4))]:
        walk(CITY, (31, 6), (door[0], door[1]+1))
        warp, = [w for w in rooms[CITY]["events"]["warps"] if (w["x"], w["y"]) == door]
        require(data["runtime_map_metadata"][warp["target_map"]]["name"] == room, f"wrong {room} lobby")
        landing, = [w for w in rooms[room]["events"]["warps"] if w["index"] == warp["target_warp_id"]]
        walk(room, (landing["x"], landing["y"]), actor_stand)
    require(trail.kind(CAPITOL, 4, 3) == "COUNTER", "records clerk needs her counter")
    for local_door in ((5, 11), (17, 13)):  # Food hall and garden stand.
        walk(CITY, (29, 7), (local_door[0], local_door[1]+1))

    for dy in (0, 1):
        for a, direct, b, opposite in [
            ((CITY, 0, 48+dy), "LEFT", (ROUTE4, 55, 18+dy), "RIGHT"),
            ((ROUTE4, 0, 18+dy), "LEFT", (METRO, 79, 36+dy), "RIGHT"),
            ((METRO, 40+dy, 0), "UP", (ROUTE5, 22+dy, 15), "DOWN"),
            ((ROUTE5, 0, 8+dy), "LEFT", (FAIR, 95, 64+dy), "RIGHT"),
        ]:
            touch(a[0], a, direct, b)
            touch(b[0], b, opposite, a)

    walk(ROUTE4, (55, 18), (0, 18))
    walk(METRO, (79, 36), (78, 36))
    clear_officers("MnMPEastCheck", (78, 36))
    walk(METRO, (79, 36), (40, 3))
    clear_officers("MnMPNorthCheck", (40, 3))
    walk(METRO, (79, 36), (40, 0))
    for lane in (0, 1):
        walk(ROUTE5, (22+lane, 15), (24, 11))
        walk(ROUTE5, (24, 11), (0, 8+lane))
        walk(FAIR, (95, 64+lane), (19, 67))
    require(set(trail.conns[FAIR]) == {"east"}, "unexpected fairgrounds route")
    exit_map = rooms[FAIR]["script_text_bodies"]["MnFairGroundsMapText"]["commands"]
    require("East exit" in " ".join(row["args"][0] for row in exit_map if row["args"]),
            "fairgrounds return board")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("parent", type=Path)
    parser.add_argument("child", type=Path, nargs="?", help="omit to survey the parent walking route")
    args = parser.parse_args()
    parent = load_pack(args.parent)
    if args.child:
        new = load_pack(args.child)
        check_dialogue(parent, new)
        check_trail(new)
        print(f"{args.child.name}: 8 native cues in 4 rooms, all 16 civic guide states, 4 two-lane road crossings both ways.")
    else:
        check_trail(parent)
        print(f"{args.parent.name}: both original road lanes, signed turns, local services and walk back from the fair passed.")


if __name__ == "__main__":
    main()
