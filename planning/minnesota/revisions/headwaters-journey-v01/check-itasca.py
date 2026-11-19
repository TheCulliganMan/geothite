#!/usr/bin/env python3
"""Trace the two routes to Itasca's reading, the report and the actual mine checkpoint."""

import argparse
from collections import deque
from functools import cache
from itertools import product
from pathlib import Path
import runpy
import sys

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE.parents[1] / "tools"))
from audit_story_flow import instructions, native_pages, read_branch, require
from render_reference import load_pack

base = runpy.run_path(str(HERE.parent / "northern-trails-v01/check-voyageurs.py"))
Trail, DIRECTIONS = base["Trail"], base["DIRECTIONS"]
LAKE, GATE, TOWN, MINE = "MnLakeItasca", "MnItascaGatehouse", "MnBemidji", "MnR09Cave1F"
OUT = "MnR14C00R00"
MARKER, OFFICER, FOREMAN = (
    "MN_ITASCA_MARKER",
    "MN_BEMIDJI_OFFICER",
    "MN_R09_DEPOT_FOREMAN",
)
READ, REPORT, RANGER, NOTE = [
    "EVENT_MN_" + key
    for key in (
        "HEADWATERS_READING",
        "HEADWATERS_REPORT",
        "BRAINERD_REPORT",
        "PRAIRIE_NOTEBOOK",
    )
]
PINE = "ENGINE_GLACIERBADGE"


def trace(data, revised):
    rooms = data["maps"]
    names = {LAKE, GATE, TOWN, MINE}
    names.update(n for n in rooms if n.startswith(("MnR09C0", "MnR14C0")))
    trail = Trail(data, names)
    warps = {}
    for n in names:
        r = rooms[n]
        require(not r["events"]["coord_events"], f"unaccounted road trigger {n}")
        for w in r["events"]["warps"]:
            dest = w["target_map"]
            if dest not in rooms:
                dest = data["runtime_map_metadata"][dest]["name"]
            if dest not in names:
                continue
            (landing,) = [
                l
                for l in rooms[dest]["events"]["warps"]
                if l["index"] == w["target_warp_id"]
            ]
            warps[n, w["x"], w["y"]] = (dest, landing["x"], landing["y"])
    # Entered-room callbacks vacate road wardens even without a report or badge.
    for n in (LAKE, TOWN):
        label = "MnPublicRoads" + ("LakeItasca" if n == LAKE else "Bemidji") + "Open"
        callbacks = {
            (r["args"][0], r["args"][1])
            for r in rooms[n]["map_script_section_commands"]
            if r["command"] == "callback"
        }
        require(
            {("MAPCALLBACK_OBJECTS", label), ("MAPCALLBACK_TILES", label)} <= callbacks,
            "public roads",
        )
        for op, *args in instructions(rooms[n], label):
            if op == "disappear":
                (actor,) = [
                    o for o in rooms[n]["objects"] if o["object_identifier"] == args[0]
                ]
                trail.bodies[n].remove((actor["x"], actor["y"]))
            else:
                require(op == "endcallback", "unexpected public road predicate")

    # A direct boundary crossing is an actual move. Only directional carpet and
    # cave/ladder collisions fire portals. Plain floor arrival records never fire.
    def step(here, direction):
        name, x, y = here
        dx, dy = DIRECTIONS[direction]
        at = (name, x + dx, y + dy)
        if at in warps:
            kind = trail.kind(*at)
            if (
                (kind == "WARP_CARPET_" + direction)
                or (kind == "CAVE" and direction == "UP")
                or kind == "LADDER"
            ):
                return warps[at]
        return trail.move(here, direction, True)

    @cache
    def distances(start):
        reached = {start: 0}
        queue = deque([start])
        while queue:
            here = queue.popleft()
            for direction in DIRECTIONS:
                at = step(here, direction)
                if at is not None and at not in reached:
                    reached[at] = reached[here] + 1
                    queue.append(at)
        return reached

    for y in (4, 5):
        a, b = (GATE, 1, y), (GATE, 8, y)
        lake = (LAKE, 42, 20 + y)
        route = (OUT, 5, 38 + y)
        require(
            step(lake, "RIGHT") == a
            and step(a, "LEFT") == lake
            and step(route, "LEFT") == b
            and step(b, "RIGHT") == route
            and b in distances(a)
            and a in distances(b),
            "two-way gatehouse",
        )
    points = {
        "Bemidji gym doorstep": (TOWN, 10, 10),
        "field officer": (TOWN, 21, 20),
        "Itasca gate entrance": (OUT, 5, 42),
        "west gatehouse door": (GATE, 1, 4),
        "posted reading": (LAKE, 19, 22),
        "mine foreman": (MINE, 9, 6),
    }
    if revised:
        points["striped gauge"] = (LAKE, 21, 21)
    for description, pos in points.items():
        require(
            trail.kind(*pos) == "FLOOR" and pos[1:] not in trail.bodies[pos[0]],
            description,
        )
    triggers = {
        (r["x"], r["y"])
        for r in rooms[LAKE]["events"]["bg_events"]
        if r["script"] == MARKER and r["event_type"] == "BGEVENT_READ"
    }
    require(
        triggers == ({(19, 21), (22, 21)} if revised else {(19, 21)}),
        "readable marker and staff",
    )
    for key in ("field officer", "mine foreman"):
        n, x, y = points[key]
        (actor,) = [
            o
            for o in rooms[n]["objects"]
            if o["script"] == (OFFICER if n == TOWN else FOREMAN)
        ]
        require((x, y - 1) == (actor["x"], actor["y"]), "stand must face the actor")
    for start in ("Bemidji gym doorstep", "field officer", "Itasca gate entrance"):
        for witness in (
            ("posted reading", "striped gauge") if revised else ("posted reading",)
        ):
            order = [
                start,
                "west gatehouse door",
                witness,
                "field officer",
                "mine foreman",
                start,
            ]
            lengths = []
            for a, b in zip(order, order[1:]):
                first, second = points[a], points[b]
                require(
                    second in distances(first) and first in distances(second),
                    f"{a} to {b} and back",
                )
                lengths.append(distances(first)[second])
            print(
                f"{start} via {witness}: "
                + " + ".join(map(str, lengths))
                + " walking inputs."
            )
    # Actual brook width is one metatile. Both lanes need dry ground to either bank.
    for y in (25, 26):
        for x in range(21, 24):
            require(
                step((LAKE, x, y), "RIGHT") == (LAKE, x + 1, y)
                and step((LAKE, x + 1, y), "LEFT") == (LAKE, x, y),
                "bridge deck",
            )
    if revised:
        require(
            all(
                trail.kind(LAKE, x, y) == "WATER"
                for x in (22, 23)
                for y in (24, 27, 28, 29)
            ),
            "water above and below the deck",
        )
        require(
            trail.kind(LAKE, 22, 21) == "WALL" and trail.kind(LAKE, 23, 21) == "WATER",
            "bank-mounted staff",
        )


def story(data, revised):
    lake, town, mine = (data["maps"][key] for key in (LAKE, TOWN, MINE))
    n = 0
    for ranger, notebook, badge, officer_first in product((False, True), repeat=4):
        flags = {
            flag
            for flag, on in ((RANGER, ranger), (NOTE, notebook), (PINE, badge))
            if on
        }
        if officer_first:
            _, pages, after = read_branch(town, OFFICER, flags)
            require(
                pages[-1] == "MnBemidjiNeedReadingText" and after == flags,
                "premature signature",
            )
        events, pages, flags = read_branch(lake, MARKER, flags)
        require(
            pages[0]
            == ("MN_ITASCA_MARKERText" if ranger else "MnFlowItascaOwnDateText")
            and events.index(("promptbutton",)) < events.index(("setevent", READ)),
            "dated reading",
        )
        _, pages, again = read_branch(lake, MARKER, flags)
        require(
            pages == ["MnItascaMarkerAfterText"] and again == flags,
            "reread before officer",
        )
        _, _, before_report = read_branch(mine, FOREMAN, flags)
        require(
            "EVENT_MN_DEPOT_FOREMAN_DEFEATED" not in before_report,
            "raw reading opened mine encounter",
        )
        events, pages, flags = read_branch(town, OFFICER, flags)
        require(
            REPORT in flags
            and ("MnBemidjiNotebookMatchText" in pages) == notebook
            and events.index(("promptbutton",)) < events.index(("setevent", REPORT)),
            "officer comparison",
        )
        _, pages, again = read_branch(lake, MARKER, flags)
        require(
            pages == ["MnOverhaulItascaComparedText"] and again == flags,
            "signed reading reunion",
        )
        events, pages, _ = read_branch(mine, FOREMAN, flags)
        require(
            (("loadtrainer", "GRUNTM", "MN_LODE_FOREMAN") in events) == badge,
            "actual badge gate at foreman",
        )
        if not badge:
            require(pages == ["MnIronRangeForemanBadgeDeniedText"], "worksite refusal")
        n += 1
    print(
        f"{n} ranger, notebook, badge and visit orders; unsigned evidence, repeat and actual foreman gate passed."
    )
    if revised:
        words = lambda room, key: " ".join(
            line for page in native_pages(room["scripts"][key]) for line in page
        ).lower()
        require(
            "marks" in words(lake, "MN_ITASCA_MARKERText")
            and "rungs" not in words(lake, "MN_ITASCA_MARKERText"),
            "gauge vocabulary",
        )
        require(
            "bridge" in words(lake, "MnWayStoneReturnText")
            and "bridge" in words(lake, "MnItascaMarkerAfterText"),
            "return directs to visible crossing",
        )


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("pack", type=Path)
    parser.add_argument("--baseline", action="store_true")
    args = parser.parse_args()
    data = load_pack(args.pack)["data"]
    trace(data, not args.baseline)
    story(data, not args.baseline)
