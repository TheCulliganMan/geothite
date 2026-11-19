#!/usr/bin/env python3
"""Follow the installed native gift and movement commands for every reachable capsule side."""

import argparse
from collections import deque
from pathlib import Path
import runpy
import sys

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE.parents[1] / "tools"))
from audit_story_flow import require
from render_reference import load_pack

support = runpy.run_path(str(HERE.parent / "northern-trails-v01/check-voyageurs.py"))
Trail, DIRECTIONS = support["Trail"], support["DIRECTIONS"]
STATION = "MnWinonaFieldStation"
POUCHID = "MN_W_CHIKORITA"
CHOSEN = "EVENT_MN_STARTER_RECEIVED"
PARCEL = "EVENT_MN_R01_PARCEL_RECEIVED"
MET = "EVENT_MN_REED_FIRST_MET"
BALLS = "EVENT_MN_WINONA_BALLS_RECEIVED"
GIFTS = {
    POUCHID: ("POUCHID", {(5, 3), (6, 4)}),
    "MN_W_CYNDAQUIL": ("KINDLIT", {(7, 4)}),
    "MN_W_TOTODILE": ("BIRCHIT", {(8, 4)}),
}


def steps(room, label):
    return [(row["command"], *row["args"]) for row in room["scripts"][label]]


def in_order(rows, expected):
    rest = iter(rows)
    return all(wanted in rest for wanted in expected)


def accessible(trail, xy):
    start = (STATION, *xy)
    queue = deque([start])
    seen = {start}
    while queue:
        at = queue.popleft()
        for direction in DIRECTIONS:
            nxt = trail.step(at, direction, dry=True)
            if nxt and nxt not in seen:
                seen.add(nxt)
                queue.append(nxt)
    return {node[1:] for node in seen}


def installed_return(room, capsule, gift, x):
    """Resolve the actual native call, readvar, comparison and movement operands."""
    action = gift[gift.index(("disappear", capsule)) + 1]
    caller = capsule
    if action[0] == "scall":
        caller = action[1]
        require(any(c["source_script"] == capsule and c["command"] == "scall"
                    and c["resolved_target_script"] == caller for c in room["script_control_commands"]),
                f"{capsule}: uncompiled return call")
        rows = steps(room, caller)
        require(rows[0] == ("readvar", "VAR_XCOORD") and rows[1][0] == "ifequal",
                f"{caller}: unsupported approach branch")
        require(any(c["source_script"] == caller and c["command"] == "readvar"
                    and c["target"] == "VAR_XCOORD" for c in room["script_variable_commands"]),
                "missing native coordinate read")
        require(any(c["source_script"] == caller and c["command"] == "ifequal"
                    and c["compare_value"] == rows[1][1] and c["resolved_target_script"] == rows[1][2]
                    for c in room["script_control_commands"]), "uncompiled aisle branch")
        if x == int(rows[1][1]):
            caller = rows[1][2]
            rows = steps(room, caller)
        else:
            rows = rows[2:]
        require(len(rows) == 2 and rows[-1] == ("end",), f"{caller} must return to the gift")
        action = rows[0]
    require(action[:2] == ("applymovement", "PLAYER"), f"invalid player walk {action}")
    label = action[2]
    require(any(c["source_script"] == caller and c["object_id"] == "PLAYER"
                and c["movement"] == label for c in room["script_object_commands"]),
            f"{caller}: missing compiled movement call")
    motion, = [m for m in room["script_movements"] if m["label"] == label]
    raw = steps(room, label)
    require(raw[-1] == ("step_end",) and all(s[0] == "step" for s in raw[:-1]),
            f"unexpected return movement {label}")
    require([(s["command"], s["direction"]) for s in motion["steps"]]
            == [*raw[:-1], ("step_end", None)], f"native/raw walk disagreement in {label}")
    return [s[1] for s in raw[:-1]]


def check_first_meeting(room, trail, actors, chosen):
    """Track the three characters and the specimen case up to the battle prompt."""
    standing = {k: (a["x"], a["y"]) for k, a in actors.items()}
    standing["PLAYER"] = (4, 2)
    visible = set(standing) - {chosen, "MN_NSE_REED", "MN_NSE_EGG_CASE"}
    for op, *args in steps(room, "MnNSEReedEncounter"):
        if op == "appear":
            require(standing[args[0]] not in {standing[i] for i in visible},
                    f"{args[0]} appeared on an occupied tile")
            visible.add(args[0])
        elif op == "disappear":
            visible.remove(args[0])
        elif op == "applymovement":
            actor, label = args
            require(actor in visible, f"moving hidden actor {actor}")
            for motion in steps(room, label):
                if motion == ("step_end",):
                    break
                require(motion[0] == "step", f"{label}: unknown actor movement")
                dx, dy = DIRECTIONS[motion[1]]
                x, y = standing[actor]
                target = x + dx, y + dy
                require(trail.kind(STATION, *target) == "FLOOR"
                        and target not in {standing[i] for i in visible - {actor}},
                        f"{label}: {actor} crosses an actor or fixture at {target}")
                standing[actor] = target
        elif op == "startbattle":
            require(standing["MN_NSE_REED"] == (4, 3) and standing["MN_W_TAMARACK"] == (5, 2)
                    and standing["PLAYER"] == (4, 2), "rival challenge has lost its staging")
            return
    raise ValueError("missing first battle")


def changed(a, b):
    return {k for k in a.keys() | b.keys() if k not in a or k not in b or a[k] != b[k]}


def preservation(parent, child):
    labels = {POUCHID, "MnStationPouchidReturn", "MnStationPouchidFromAisle", "MnStationPouchidAisleSteps"}
    require(changed(parent, child) == {"data", "report", "identity"}, "other pack files changed")
    old, new = parent["report"], child["report"]
    require(changed(old, new) == {"manifests"} and old["manifests"][-1] == "northstar-lake-observations-v1"
            and new["manifests"] == old["manifests"] + ["northstar-station-return-v1"], "unexpected lineage")
    a, b = parent["data"], child["data"]
    require(changed(a, b) == {"map_scripts", "maps"} and changed(a["maps"], b["maps"]) == {STATION},
            "world data outside the station changed")
    require(changed(a["map_scripts"], b["map_scripts"]) == labels, "other split scripts changed")
    before, after = a["maps"][STATION], b["maps"][STATION]
    tables = {"script_control_commands", "script_variable_commands", "script_object_commands", "script_movements"}
    require(changed(before, after) == tables | {"scripts"} and changed(before["scripts"], after["scripts"]) == labels,
            "unexpected station content change")
    p, q = steps(before, POUCHID), steps(after, POUCHID)
    require(len(p) == len(q) and [(x, y) for x, y in zip(p, q) if x != y] == [
        (("applymovement", "PLAYER", "MnOverhaulStarterReturn0"), ("scall", "MnStationPouchidReturn"))],
        "Pouchid gift changed outside its original return instruction")
    for t in tables:
        require([r for r in before[t] if r["source_script"] not in labels]
                == [r for r in after[t] if r["source_script"] not in labels], f"unrelated native bindings in {t}")
    print(f"Compared {len(b['maps'])} rooms and {len(child['runtime_files'])} runtime files; "
          "only the capsule walk and its native bindings changed.")


def verify(pack, baseline):
    data = pack["data"]
    room = data["maps"][STATION]
    scripts = room["scripts"]
    actors = {a["object_identifier"]: a for a in room["objects"]}
    trail = Trail(data, {STATION})
    pos = lambda a: (a["x"], a["y"])
    require(pos(actors["MN_W_TAMARACK"]) == (5, 2), "Tamarack moved away from the mark")
    require(("callback", "MAPCALLBACK_OBJECTS", "MnNSEHideVisitors") in
            [(r["command"], *r["args"]) for r in data["map_scripts"][room["attributes"]["map_scripts_label"]]],
            "visitor visibility callback absent")
    for hidden in ("MN_NSE_REED", "MN_NSE_EGG_CASE"):
        require(("disappear", hidden) in steps(room, "MnNSEHideVisitors"), f"{hidden} unexpected at entry")
        trail.bodies[STATION].remove(pos(actors[hidden]))
    require(any(w["index"] == 3 and (w["x"], w["y"]) == (4, 10)
                for w in room["events"]["warps"]), "station return warp changed")
    floor = accessible(trail, (4, 10))
    for capsule, (species, expected) in GIFTS.items():
        actor = actors[capsule]
        require(actor["script"] == capsule, f"broken {species} interaction")
        at = pos(actor)
        stands = {(at[0] + dx, at[1] + dy) for dx, dy in DIRECTIONS.values()} & floor
        require(stands == expected, f"{species} reachable interaction sides {stands}")
        gift = steps(room, capsule)
        require(in_order(gift, [
            ("checkevent", CHOSEN), ("iftrue", f"MnOverhaulOtherStarter{at[0]-6}"),
            ("checkevent", "EVENT_MN_HOME_INTRO"), ("iffalse", "MnWVisitHome"),
            ("pokepic", species), ("cry", species), ("yesorno",), ("iffalse", "MnWStarterDeclined"),
            ("givepoke", species, "5"), ("ifequal", "2", "MnWStarterFull"),
            ("setevent", CHOSEN), ("setevent", f"EVENT_MN_WINONA_{capsule.removeprefix('MN_W_')}"),
            ("disappear", capsule), ("turnobject", "PLAYER", "RIGHT"),
            ("faceobject", "MN_W_TAMARACK", "PLAYER"), ("sjump", "MnWFollowup")]),
            f"{species} decision, gift, or final staging changed")
        for start in sorted(stands):
            walk = installed_return(room, capsule, gift, start[0])
            node = (STATION, *start)
            for index, direction in enumerate(walk):
                nxt = trail.step(node, direction, dry=True)
                if baseline and start == (5, 3) and index == len(walk) - 1:
                    dx, dy = DIRECTIONS[direction]
                    end = (node[1] + dx, node[2] + dy)
                    require(nxt is None and end == (3, 1) and trail.kind(STATION, *end) == "WALL",
                            "side approach no longer ends in the computer")
                    break
                require(nxt is not None, f"{species} from {start}: {direction} blocked at {node[1:]}")
                node = nxt
            else:
                require(node[1:] == (4, 2), f"{species} from {start}: wrong end {node[1:]}")
            print(f"{species:8} {start}  {' '.join(walk)}  "
                  + ("→ computer WALL (3,1)" if baseline and start == (5, 3) else "→ Tamarack (4,2)"))
    for label in ("MnWStarterDeclined", "MnWStarterFull", "MnWVisitHome"):
        require(all(s[0] in {"jumptext", "jumptextfaceplayer", "writetext", "promptbutton", "closetext", "end"}
                    for s in steps(room, label)), f"{label} accidentally consumes a choice")
    require(in_order(steps(room, "MnWFollowup"), [
        ("checkevent", PARCEL), ("iftrue", "MnWReminder"), ("setflag", "ENGINE_POKEDEX"),
        ("setevent", PARCEL), ("scall", "MnNSEReedEncounter")]), "Tamarack handoff changed")
    require(in_order(steps(room, "MnNSEReedEncounter"), [
        ("checkevent", MET), ("iftrue", "MnNSEEnd"), ("setevent", MET),
        ("loadtrainer", "MN_RIVAL_REED", "MN_REED_WINONA"), ("startbattle",),
        ("reloadmapafterbattle",), ("setevent", "EVENT_MN_REED_WINONA_DEFEATED"),
        ("moveobject", "MN_NSE_REED", "4", "3")]), "Reed's one-time battle changed")
    for capsule in GIFTS:
        check_first_meeting(room, trail, actors, capsule)
    for event in room["events"]["coord_events"]:
        require((event["x"], event["y"]) in floor, "the supply desk is inaccessible on return")
        require(in_order(steps(room, event["script_name"]), [
            ("checkevent", PARCEL), ("checkevent", BALLS), ("scall", "MnWSupplies")]),
            "return desk no longer offers the balls")
    require({(e["x"], e["y"]) for e in room["events"]["coord_events"]} == {(4, 9), (5, 9)},
            "a supply desk entrance was lost")
    require(in_order(steps(room, "MnWSupplies"), [
        ("checkevent", BALLS), ("iftrue", "MnWSuppliesDone"), ("giveitem", "POKE_BALL", "5"),
        ("iffalse", "MnWSuppliesFull"), ("setevent", BALLS)]), "supplies failed bag-full retry")
    for label in scripts:
        if label in GIFTS or label.startswith("MnStationPouchid"):
            require(data["map_scripts"][label] == scripts[label], f"global script disagrees: {label}")
    print("Three gifts and four approaches checked; choice retries, Reed and both supply desks verified.")


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("pack", type=Path)
    parser.add_argument("--baseline", action="store_true", help="require the original blocked aisle step")
    parser.add_argument("--parent", type=Path, help="deep-compare the published child against its Lake parent")
    args = parser.parse_args()
    child = load_pack(args.pack)
    verify(child, args.baseline)
    if args.parent:
        preservation(load_pack(args.parent), child)
