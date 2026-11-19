#!/usr/bin/env python3
"""Check the beginner badge road and native third-badge dispatch advice without running the game."""

import argparse
from itertools import pairwise, product
import json
from pathlib import Path
import runpy
import sys

HERE = Path(__file__).resolve().parent
REGION = HERE.parents[1]
sys.path.insert(0, str(REGION / "tools"))
from audit_story_flow import instructions, native_pages, read_branch, require
from render_reference import load_pack

Trail = runpy.run_path(str(HERE.parent / "northern-trails-v01/check-voyageurs.py"))["Trail"]
route_connections = runpy.run_path(str(HERE.parent / "brown-shore-edges-v01/review.py"))["route_connections"]
GYM = "MnMinneapolisGym"
DISPATCH = "MnMinneapolisDispatchOffice"
ROAD = "MnR05C00R02"
REPEAT = "MnMinneapolisGymRematchBeforeChampion"
WARDEN = "MN_R05_WARDENText"
RESCUE = "EVENT_MN_ICE_CAPTIVE_RESCUED"
SKETCH = "EVENT_MN_DISPATCH_NOTE"
DATE = "EVENT_MN_DEEP_BORE_MANIFEST_READ"
RESTORED = "EVENT_MN_SURVEY_RESTORED"
IDLE = "MnFlowAvrinAfterRingText"
DESK = "MnMinneapolisGymLeaderAfterText"
FIND = "MnBadgeAvrinDispatchText"
MINA = "MnBadgeAvrinMinaText"
BADGES = ("ENGINE_ZEPHYRBADGE", "ENGINE_HIVEBADGE", "ENGINE_PLAINBADGE", "ENGINE_FOGBADGE", "ENGINE_STORMBADGE")
READY = ("EVENT_MN_GYM_3_DEFEATED", "EVENT_MN_GYM_3_TM_RECEIVED")


def pages(room, label):
    return native_pages(room["script_text_bodies"][label]["commands"])


def handoff_destinations(pack):
    rooms = pack["data"]["maps"]
    station = rooms["MnWinonaFieldStation"]
    starters = set()
    for actor in station["objects"]:
        offers = instructions(station, actor["script"])
        starters.update(row[1:] for row in offers if row[0] == "givepoke")
    require(starters == {("POUCHID", "5"), ("KINDLIT", "5"), ("BIRCHIT", "5")}, "three station choices")
    reminder = " ".join(sum(pages(station, "MnWReminderText"), []))
    require(all(k in reminder for k in ("IVY", "ROCHESTER", "clinic", "west")), "first recipient and road")
    parcel = "EVENT_MN_DELIVERED_ROCHESTER_PARCEL"
    require(any(row["command"] == "setevent" and row["flag_id"] == parcel for row in
                rooms["MnRochesterPokecenter1F"]["script_flag_commands"]), "Ivy receives the envelope")
    for n, (town, leader, badge) in enumerate(zip(
        ("Rochester", "Mankato", "Minneapolis", "StPaul", "StCloud"),
        ("VAREL", "TORVEN", "AVRIN", "BRANNIC", "SAVEN"), BADGES
    ), 1):
        name = f"Mn{town}Gym"
        room = rooms[name]
        label = name + "Leader"
        op = instructions(room, label)
        require(any(a["script"] == label for a in room["objects"]), f"{leader} on the court")
        expected = ("checkevent", parcel) if n == 1 else ("checkflag", BADGES[n-2])
        require(expected in op and ("checkevent", f"EVENT_MN_GYM_{n}_COURT_OPEN") in op
                and ("loadtrainer", "MN_LEADER_" + leader, f"MN_GYM_{n}") in op
                and ("setflag", badge) in op, f"{leader} badge handoff")
        if n == 3:
            require(all("ICE" not in row[1] for row in op if row[0] == "checkevent"),
                    "third battle available independently of dispatch rescue")
    for side, sequence in (
        ("west", ["MnWinona", "MnR01C00R00", "MnRochester", "MnR02C00R00", "MnMankato"]),
        ("north", ["MnMankato", "MnR03C00R01", "MnR03C00R00", "MnMinneapolis"]),
        ("east", ["MnMinneapolis", "MnR04C00R00", "MnStPaul"]),
        ("north", ["MnMinneapolis", "MnR05C00R02", "MnR05C00R01", "MnR05C00R00", "MnStCloud"]),
    ):
        opposite = {"east": "west", "west": "east", "north": "south"}[side]
        for a, b in pairwise(sequence):
            for origin, target, direction in ((a, b, side), (b, a, opposite)):
                require(any(c["direction"] == direction and c["target_map"] == target
                            for c in rooms[origin]["attributes"]["connections"]), f"{origin} to {target}")


def dispatch(pack, corrected):
    data = pack["data"]
    gym = data["maps"][GYM]
    office = data["maps"][DISPATCH]
    city = data["maps"]["MnMinneapolis"]
    doors = {w["target_map_constant"]: (w["x"], w["y"]) for w in city["events"]["warps"]}
    require(doors["MN_MINNEAPOLIS_DISPATCH_OFFICE"] == (25, 33)
            and doors["MN_MINNEAPOLIS_GYM"] == (64, 33), "dispatch is west of the gym")
    require(any(e["script"] == "MnMPDispatchCopy" and (e["x"], e["y"]) == (9, 11)
                for e in office["events"]["bg_events"]), "ring desk")
    flags = {RESCUE, "EVENT_MN_ICE_ENFORCER_DEFEATED", "EVENT_MN_ICE_HOLDING_POWER_OFF"}
    for absent in flags:
        require(read_branch(office, "MnMPDispatchCopy", flags - {absent})[1] == ["MnMPDispatchLockedText"],
                f"desk still locked without {absent}")
    require(SKETCH in read_branch(office, "MnMPDispatchCopy", flags)[2], "sketch obtained from the open desk")
    depot = data["maps"]["MnStPaulUnionDepot"]
    require(any(o["script"] == "MN_NSTP_MINA" for o in depot["objects"])
            and ("checkevent", SKETCH) in instructions(depot, "MN_ST_PAUL_DEPOT_COURIER")
            and ("setevent", DATE) in instructions(depot, "MnFlowMinaOfferDate"), "real destination for the sketch")
    for bits in product((False, True), repeat=4):
        active = {f for f, v in zip((RESCUE, SKETCH, DATE, RESTORED), bits) if v}
        rescue, sketch, date, restored = bits
        if corrected:
            expected = IDLE if restored or date else MINA if sketch else DESK if rescue else FIND
        else:
            expected = IDLE if sketch else DESK
        _, actual, after = read_branch(gym, "MnMinneapolisGymLeader", [*READY, *active])
        require(actual == [expected] and after == {*READY, *active}, f"Avrin at {active}: {actual}")
    require(("iffalse", REPEAT) in instructions(gym, "MnMinneapolisGymLeaderAfter")
            and ("checkevent", "EVENT_MN_LEAGUE_CHAMPION") in instructions(gym, "MnMinneapolisGymLeaderAfter"),
            "championship keeps its rematch")
    if corrected:
        check_mirrors(data, GYM, (REPEAT, "MnBadgeAvrinDispatch", "MnBadgeAvrinMina"))


def check_mirrors(data, name, labels):
    room = data["maps"][name]
    for label in labels:
        require(data["map_scripts"][label] == room["scripts"][label], f"split script {label}")
        for table, opcodes, value in (
            ("script_flag_commands", {"checkevent"}, "flag_id"),
            ("script_control_commands", {"iftrue", "iffalse"}, "target_label"),
            ("script_text_commands", {"jumptext"}, "text_label"),
        ):
            typed = [(v["command_index"], v["command"], v[value]) for v in room[table] if v["source_script"] == label]
            raw = [(i, op[0], op[1]) for i, op in enumerate(instructions(room, label)) if op[0] in opcodes]
            require(typed == raw, f"{table}/{label} branch offsets")


def river_bluffs(pack, corrected):
    rooms = pack["data"]["maps"]
    names = ("MnR05C00R00", "MnR05C00R01", ROAD)
    trail = Trail(pack["data"], names)
    require(any(o["script"] == "MN_R05_WARDEN" and (o["x"], o["y"]) == (19, 7)
                for o in rooms[ROAD]["objects"]), "the road warden stands by the southern approach")
    require(instructions(rooms[ROAD], "MN_R05_WARDEN") == [("jumptextfaceplayer", WARDEN)], "road warden speaks")
    for name, ledges in ((names[0], [(38,20), (39,20), (36,40), (37,40)]), (names[1], [(36,28), (37,28)]), (ROAD, [])):
        all_hops = {(x, y, kind) for y, row in enumerate(trail.grid[name]) for x, kind in enumerate(row) if kind.startswith("HOP_")}
        require(all_hops == {(x, y, "HOP_DOWN") for x, y in ledges}, f"{name} actual ledge directions")
        for x, y in ledges:
            require(trail.kind(name, x, y + 1) == "WALL" and trail.kind(name, x, y + 2) in {"FLOOR", "TALL_GRASS"},
                    f"{name}/{x},{y} reverse face and landing")
    for name, x, a, b in [(names[1],13,27,31), (names[0],15,39,43)]:
        require(trail.both((name,x,a), (name,x,b)) == (4,4), f"ordinary return stair at {name}/{x},{a}")
    route_connections(pack)
    if not corrected:
        old = " ".join(sum(pages(rooms[ROAD], WARDEN), []))
        require("hop down the ledges heading north" in old, "expected reversed parent guidance")


def main():
    cli = argparse.ArgumentParser(description=__doc__)
    cli.add_argument("parent", type=Path)
    cli.add_argument("candidate", type=Path, nargs="?")
    args = cli.parse_args()
    for filename, corrected in ((args.parent, False), (args.candidate, True)):
        if filename is None:
            continue
        pack = load_pack(filename)
        handoff_destinations(pack)
        dispatch(pack, corrected)
        river_bluffs(pack, corrected)
        if corrected:
            source = json.loads((REGION / "story.json").read_text())["dialogue_overhaul"]["texts"]
            for name, label in ((GYM, FIND), (GYM, MINA), (ROAD, WARDEN)):
                room = pack["data"]["maps"][name]
                require(pages(room, label) == source[name][label]
                        and pack["data"]["map_scripts"][label] == room["scripts"][label], f"native text {label}")
        print(f"{filename.name}: three starters, parcel, four badges plus St. Cloud, 16 Avrin states, six south-facing ledge squares and both stair returns.")

if __name__ == "__main__":
    main()
