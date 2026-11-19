#!/usr/bin/env python3
"""Trace the optional Granite Passage on the actual SQ03 stamp trip, including its ice fork."""

import argparse
from collections import deque
from pathlib import Path
import json
import runpy
import sys

HERE = Path(__file__).resolve().parent
REGION = HERE.parents[1]
sys.path.insert(0, str(REGION / "tools"))
from audit_story_flow import BADGES, instructions, native_pages, read_branch, require
from render_reference import load_pack

winter = runpy.run_path(str(HERE.parent / "northern-trails-v01" / "check-voyageurs.py"))
DIRECTIONS = winter["DIRECTIONS"]
WEST, EAST, CAMP = winter["WEST"], winter["EAST"], winter["CAMP"]
GRANITE, FIRST, BASEMENT = "MnR20Cave1F", "MnVoyageursIce1F", "MnVoyageursIceB1F"
SURF_REWARD = "MN_CAVE_R20_SEEP_CACHE"
ARRIVED = "EVENT_MN_FALLS_VOYAGEURS_ARRIVAL"
REPORT = "EVENT_MN_BORDER_REPORT"
SIGNAL = "EVENT_MN_LIGHTHOUSE_SIGNAL"


def at(row):
    return row["x"], row["y"]


def neighbor(p, delta, length=1):
    return p[0] + delta[0] * length, p[1] + delta[1] * length


def faces(p, choices):
    return {neighbor(p, delta) for delta in DIRECTIONS.values()} & set(choices)


class Cave:
    """Directed ice slides and west/drop ledges, including their unusable reverse faces."""

    def __init__(self, data, name, surf=False, ice=True, avoid=(), taken=False):
        r = data["maps"][name]
        terrain = winter["Trail"](data, {name}).grid[name]
        self.g = {
            (x, y): role for y, row in enumerate(terrain) for x, role in enumerate(row)
        }
        self.ports = {at(w) for w in r["events"]["warps"] if self.g[at(w)] == "LADDER"}
        barriers = set(avoid) | {
            at(o)
            for o in r["objects"]
            if not (taken and o["object_type"] == "OBJECTTYPE_ITEMBALL")
        }
        self.nodes = {
            p
            for p, role in self.g.items()
            if role != "WALL" and (surf or role != "WATER") and (ice or role != "ICE")
        } - barriers
        require(
            set(self.g.values())
            <= {
                "WALL",
                "FLOOR",
                "UP_WALL",
                "WATER",
                "HOP_LEFT",
                "HOP_DOWN",
                "LADDER",
                "ICE",
            },
            name,
        )
        self.edges = {p: [] for p in self.nodes}
        for p in self.nodes:
            for direction, delta in DIRECTIONS.items():
                hops = self.g[p] == "HOP_" + direction
                v = neighbor(p, delta, 2 if hops else 1)
                if (
                    v not in self.nodes
                    or (direction == "UP" and self.g[p] == "UP_WALL")
                    or (direction == "DOWN" and self.g[v] == "UP_WALL")
                ):
                    continue
                while self.g[v] == "ICE":
                    following = neighbor(v, delta)
                    if following not in self.nodes or (
                        direction == "DOWN" and self.g[following] == "UP_WALL"
                    ):
                        break
                    v = following
                self.edges[p].append(v)

    def distances(self, start):
        seen = {start: 0}
        todo = deque([start])
        while todo:
            p = todo.popleft()
            if p in self.ports and p != start:
                continue
            for v in self.edges[p]:
                if v not in seen:
                    seen[v] = seen[p] + 1
                    todo.append(v)
        return seen

    def all_return(self, entrance):
        seen = self.distances(entrance)
        reachable = self.ports.intersection(seen)
        incoming = {v: [] for v in seen}
        for p in seen:
            for v in self.edges[p]:
                if v in incoming:
                    incoming[v].append(p)
        queue = deque(reachable)
        while queue:
            for p in incoming[queue.popleft()]:
                if p not in reachable:
                    reachable.add(p)
                    queue.append(p)
        require(
            reachable == set(seen), "reachable cave square without a route to any exit"
        )
        return seen


def doors(data):
    maps = data["maps"]
    trail = winter["Trail"](data, {WEST, EAST, GRANITE, FIRST, BASEMENT})
    # Read only active entrance quadrants; adjacent FLOOR warp records are return landings.
    pairs = [
        (WEST, 1, GRANITE, 1),
        (EAST, 1, GRANITE, 2),
        (EAST, 3, FIRST, 3),
        (FIRST, 2, BASEMENT, 2),
    ]
    for src, src_index, dest, dest_index in pairs:

        def entry(n, index):
            return next(w for w in maps[n]["events"]["warps"] if w["index"] == index)

        w, t = entry(src, src_index), entry(dest, dest_index)
        require(
            w["target_map"] in (dest, maps[dest]["attributes"]["map_constant"])
            and w["target_warp_id"] == dest_index
            and trail.kind(src, *at(w)) in {"CAVE", "LADDER"},
            f"{src} active {src_index} must enter {dest}/{dest_index}",
        )
        ret = t if dest == GRANITE else entry(dest, 1)
        landing = entry(src, ret["target_warp_id"])
        require(
            ret["target_map"] in (src, maps[src]["attributes"]["map_constant"])
            and trail.kind(dest, *at(ret)) == "LADDER"
            and trail.kind(src, *at(landing)) == "FLOOR"
            and at(landing) == (w["x"], w["y"] + 1),
            f"{dest} must return to its own entrance",
        )


def room_checks(data):
    room = data["maps"][GRANITE]
    w, e = (5, 21), (23, 19)
    dry = Cave(data, GRANITE)
    wet = Cave(data, GRANITE, surf=True)
    forward = dry.all_return(w)
    back = dry.all_return(e)
    for p in [(13, 12), (13, 13)]:
        landing = (11, p[1])
        require(
            landing in dry.edges[p] and p not in dry.edges[landing],
            "west lip must drop one way",
        )
        require(
            20 <= dry.distances(landing)[p] <= 40,
            "ordinary walking loop must recover the upper shelf",
        )
    expected = {
        "MN_HIDDEN_R20CAVE1F": "REVIVE",
        "MN_CAVE_R20_UPPER_CACHE": "MAX_REVIVE",
        SURF_REWARD: "FULL_RESTORE",
    }
    for event in room["events"]["bg_events"]:
        name = event["script"]
        rows = instructions(room, name)
        require(
            event["event_type"] == "BGEVENT_ITEM"
            and len(rows) == 1
            and rows[0][:2] == ("hiddenitem", expected.pop(name)),
            name,
        )
        for start in (w, e):
            require(
                bool(faces(at(event), dry.distances(start))) == (name != SURF_REWARD),
                name,
            )
            require(
                faces(at(event), wet.all_return(start)), f"{name} Surf reach and return"
            )
    require(not expected, "missing hidden reward")
    guide = next(
        o for o in room["objects"] if o["script"] == "MN_INTERIOR_R20_CLEFT_SCOUT"
    )
    ball = next(o for o in room["objects"] if o["object_type"] == "OBJECTTYPE_ITEMBALL")
    require(
        faces(at(guide), forward)
        and faces(at(ball), forward)
        and instructions(room, ball["script"]) == [("itemball", "HYPER_POTION", "1")],
        "scout and Hyper Potion",
    )
    trainer = next(
        o for o in room["objects"] if o["object_type"] == "OBJECTTYPE_TRAINER"
    )
    require(
        at(trainer) == (21, 17)
        and trainer["radius"] == 3
        and trainer["spritemovedata"] == "SPRITEMOVEDATA_STANDING_DOWN"
        and trainer["sightline_direction_override"] in (None, "DOWN")
        and instructions(room, trainer["script"])[0][0] == "trainer",
        "Leif's battle",
    )
    sight = [(21, y) for y in (18, 19, 20)]
    require(
        all(p in back and dry.g[p] == "FLOOR" for p in sight),
        "reachable trainer sightline",
    )
    require(
        e in Cave(data, GRANITE, avoid=sight).distances(w),
        "an optional fight must have a dry alternative",
    )
    print(
        f"Granite Passage uses {forward[e]} eastbound and {back[w]} westbound inputs. Leif sees three shelf squares; a dry route avoids them. Surf reaches the Full Restore."
    )
    for n, start, other, item in [
        (FIRST, (5, 19), (23, 5), "FULL_HEAL"),
        (BASEMENT, (5, 19), (5, 21), "PP_UP"),
    ]:
        interior = Cave(data, n)
        reached = interior.all_return(start)
        require(
            other in reached and other in Cave(data, n, taken=True).all_return(start), n
        )
        if n == FIRST:
            require(
                other not in Cave(data, n, ice=False).distances(start),
                "ice puzzle can be walked around",
            )
        hidden = data["maps"][n]["events"]["bg_events"]
        require(
            len(hidden) == 1
            and instructions(data["maps"][n], hidden[0]["script"])[0][:2]
            == ("hiddenitem", item)
            and faces(at(hidden[0]), reached),
            f"{n} hidden {item}",
        )
        for obj in data["maps"][n]["objects"]:
            require(faces(at(obj), reached), f"{n} {obj['script']}")
        print(f"{n} has {len(reached)} reachable stopping positions; all can return.")


def trail_checks(data):
    trail = winter["Trail"](data, {WEST, EAST, "MnR20C00R01", "MnR20C01R01", CAMP})
    road_start = (WEST, 0, 45)
    foot_w, foot_e = (WEST, 62, 32), (EAST, 12, 32)
    west = trail.both(road_start, foot_w)
    between = trail.both(foot_w, foot_e)
    branch = trail.both(foot_e, (EAST, 6, 34))
    for y in (24, 25):
        end = (EAST, 63, y + 12)
        east = trail.both(foot_e, end)
        require(
            trail.step(end, "RIGHT") == (CAMP, 0, y)
            and trail.step((CAMP, 0, y), "LEFT") == end,
            "paired camp stamp seam",
        )
        print(
            f"Route 20 west entry to granite west {west}, east granite to stamp approach {east}, stamp arrival (0,{y}) facing right."
        )
    print(
        f"Outdoor alternative between granite landings {between}; east granite to the ice branch {branch}."
    )


def stamp_check(data):
    room = data["maps"][CAMP]
    label = "MnFallsVoyageursArrival"
    require(
        any(
            r["command"] == "callback" and r["args"] == ["MAPCALLBACK_NEWMAP", label]
            for r in room["map_script_section_commands"]
        ),
        "installed arrival callback",
    )
    require(
        instructions(room, label)
        == [
            ("readvar", "VAR_XCOORD"),
            ("ifnotequal", "0", "MnFallsArrivalEnd"),
            ("readvar", "VAR_YCOORD"),
            ("ifequal", "24", "MnFallsArrivalQualify"),
            ("ifnotequal", "25", "MnFallsArrivalEnd"),
            ("sjump", "MnFallsArrivalQualify"),
        ],
        "stamp coordinates",
    )
    rows = instructions(room, "MnFallsArrivalQualify")
    require(
        rows[:2]
        == [("readvar", "VAR_FACING"), ("ifnotequal", "RIGHT", "MnFallsArrivalEnd")]
        and rows[-2:] == [("setevent", ARRIVED), ("endcallback",)],
        "arrival heading",
    )
    require(
        {arg for op, arg in rows[2:-2:2] if op in ("checkevent", "checkflag")}
        == BADGES | {REPORT, SIGNAL}
        and all(row == ("iffalse", "MnFallsArrivalEnd") for row in rows[3:-2:2]),
        "stamp prerequisites",
    )
    clinic = data["maps"]["MnInternationalFallsPokecenter1F"]
    prize = instructions(clinic, "MN_FALLS_RANGER")
    idx = prize.index(("giveitem", "PP_UP", "1"))
    require(
        prize[idx + 1 : idx + 3]
        == [
            ("iffalse", "MnFallsFull"),
            ("setevent", "EVENT_MN_BORDER_CIRCUIT_REWARDED"),
        ],
        "full Pack retry",
    )
    for n in (GRANITE, FIRST, BASEMENT):
        require(
            all(
                ("setevent", ARRIVED) not in instructions(data["maps"][n], label)
                for label in data["maps"][n]["scripts"]
            ),
            "cave must not award stamp",
        )


def signs(data, baseline):
    design = json.loads((REGION / "routes/MN_R20.json").read_text())
    for n, key in [(WEST, "MN_R20_SIGN_GRANITE"), (EAST, "MN_R20_SIGN_MEADOW")]:
        room = data["maps"][n]
        [src] = [s for s in design["signs"] if s["id"] == key]
        [event] = [e for e in room["events"]["bg_events"] if e["script"] == key]
        require(
            event["event_type"] == "BGEVENT_READ"
            and at(event) == (src["tile"][0] % 64, src["tile"][1] % 48),
            "original post position",
        )
        label = key + "Text"
        expected = (
            (
                ["GRANITE NARROWS", "Two mouths connect", "beneath the stone."]
                if n == WEST
                else ["SHELTERED MEADOW", "Watch for rare", "night partners."]
            )
            if baseline
            else src["lines"]
        )
        rows = room["scripts"][label]
        require(
            instructions(room, key) == [("jumptext", label)]
            and sum(native_pages(rows), []) == expected
            and data["map_scripts"][label] == rows
            and [dict(r, command_index=i) for i, r in enumerate(rows)]
            == room["script_text_bodies"][label]["commands"],
            label,
        )
        require(all(len(line) <= 18 for line in expected), label)
        print(
            f"{n} {at(event)}: " + " / ".join(" ".join(p) for p in native_pages(rows))
        )


def main():
    cli = argparse.ArgumentParser(description=__doc__)
    cli.add_argument("pack", type=Path)
    cli.add_argument(
        "--baseline", action="store_true", help="check the two original sign messages"
    )
    args = cli.parse_args()
    data = load_pack(args.pack)["data"]
    ski = winter["walk_checks"](data)
    outbound, homeward = winter["SITES"]
    winter["exercise_guide"](data, ski, outbound, homeward)
    winter["exercise_guide"](data, ski, homeward, outbound)
    clinic = data["maps"]["MnInternationalFallsPokecenter1F"]
    initial = BADGES | {SIGNAL}
    _, heard, active = read_branch(clinic, "MN_FALLS_RANGER", initial, True)
    require(
        REPORT in active and "MnFallswaiting_linesText" in heard,
        "register for the circuit",
    )
    _, heard, active = read_branch(clinic, "MN_FALLS_RANGER", initial, False)
    require(
        REPORT not in active and "MnFallsdecline_linesText" in heard,
        "decline the circuit",
    )
    doors(data)
    trail_checks(data)
    room_checks(data)
    stamp_check(data)
    signs(data, args.baseline)
    print(
        "Two through-cave mouths, the separate two-floor ice cavern, search and return, both stamp arrivals and original clinic reward checked."
    )


if __name__ == "__main__":
    main()
