#!/usr/bin/env python3
"""Read the winter check-in and confirm both choices on Tess's road in a compiled edition."""

import argparse
from collections import deque
import json
from pathlib import Path
import sys

REGION = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(REGION / "tools"))
from audit_story_flow import BADGES, native_pages, read_branch, require
from render_reference import load_pack

PINE = "ENGINE_GLACIERBADGE"
SIGNAL = "EVENT_MN_LIGHTHOUSE_SIGNAL"
REPORT = "EVENT_MN_BORDER_REPORT"
ARRIVED = "EVENT_MN_FALLS_VOYAGEURS_ARRIVAL"
GIVEN = "EVENT_MN_BORDER_CIRCUIT_REWARDED"
SURVEY = "EVENT_MN_SURVEY_RESTORED"
CHAMP = "EVENT_MN_LEAGUE_CHAMPION"
CLINIC = "MnInternationalFallsPokecenter1F"


def check_watch(data, parent):
    room = data["maps"][CLINIC]
    flags = sorted(BADGES)
    require(len(flags) == 8 and PINE in flags, "regional badge inventory")
    for mask in range(256):
        owned = {f for i, f in enumerate(flags) if mask & (1 << i)}
        for signal in (False, True):
            start = owned | ({SIGNAL} if signal else set())
            direction = (
                "MnFallsWatchBadgesText"
                if mask != 255
                else "MnFallsWatchHarborText"
                if not signal
                else "MnFallswaiting_linesText"
            )
            for enrolled in (False, True):
                for accept in (False, True):
                    initial = start | ({REPORT} if enrolled else set())
                    _, heard, final = read_branch(
                        room, "MN_FALLS_RANGER", initial, accept
                    )
                    expected = [direction]
                    if PINE not in owned:
                        expected = ["MnFallsbadge_linesText"]
                    elif not enrolled:
                        expected = (
                            [
                                "MnFallsregister_linesText",
                                "MnFallsregistered_linesText",
                                direction,
                            ]
                            if accept
                            else [
                                "MnFallsregister_linesText",
                                "MnFallsdecline_linesText",
                            ]
                        )
                    require(
                        heard == expected
                        and (REPORT in final)
                        == (enrolled or (PINE in owned and accept)),
                        f"unexpected clinic reply for {mask}/{signal}/{enrolled}/{accept}: {heard}",
                    )
    # Give-item retries, exact inventory quantity, and double-collection guards stay as published.
    for key in (
        "MN_FALLS_RANGER",
        "MnFallsFull",
        "MnFallsRepeat",
        "MnFallsVoyageursArrival",
        "MnFallsArrivalQualify",
        "MnFallsArrivalEnd",
    ):
        n = (
            CLINIC
            if key in ("MN_FALLS_RANGER", "MnFallsFull", "MnFallsRepeat")
            else "MnBoundaryWaters"
        )
        require(
            data["maps"][n]["scripts"][key] == parent["maps"][n]["scripts"][key], key
        )
    camp = data["maps"]["MnBoundaryWaters"]
    ranger = "MN_BOUNDARY_BORDER_RANGER"
    for i in range(16):
        state = {
            f for j, f in enumerate([ARRIVED, GIVEN, SURVEY, CHAMP]) if i & (1 << j)
        }
        _, heard, after = read_branch(camp, ranger, state)
        if ARRIVED in state and GIVEN not in state:
            expected = ["MnFallsWatchArrivedText"] + (
                ["MnWayLeagueTrailText"]
                if SURVEY in state and CHAMP not in state
                else []
            )
        else:
            expected = read_branch(parent["maps"]["MnBoundaryWaters"], ranger, state)[1]
        require(
            heard == expected and state == after,
            f"border ranger in state {state}: {heard}",
        )
    canonical = json.loads((REGION / "story.json").read_text())["dialogue_overhaul"][
        "texts"
    ]
    for name, labels in {
        CLINIC: [
            "MnFallsregister_linesText",
            "MnFallsregistered_linesText",
            "MnFallswaiting_linesText",
            "MnFallsWatchBadgesText",
            "MnFallsWatchHarborText",
        ],
        "MnBoundaryWaters": ["MnFallsWatchArrivedText"],
        "MnInternationalFalls": ["MN_FALLS_TRAIL_NOTICEText"],
    }.items():
        room = data["maps"][name]
        for key in labels:
            program = room["scripts"][key]
            require(native_pages(program) == canonical[name][key], key)
            require(
                room["script_text_bodies"][key]["commands"]
                == [dict(row, command_index=i) for i, row in enumerate(program)],
                key,
            )
            require(data["map_scripts"][key] == program, key)


def check_tess(data, parent):
    name, key = "MnR02C00R00", "MN_R02_T01"
    room = data["maps"][name]
    old = parent["maps"][name]
    change = {
        "spritemovedata": "SPRITEMOVEDATA_STANDING_RIGHT",
        "radius": 1,
        "sightline_direction_override": "RIGHT",
    }
    expected = [dict(o, **change) if o["script"] == key else o for o in old["objects"]]
    require(
        room["objects"] == expected == data["npcs"][name],
        "trainer placements and NPC mirror",
    )
    require(
        room["trainer_scripts"] == old["trainer_scripts"]
        and data["trainers"] == parent["trainers"],
        "Tess's original battle and team",
    )
    (row,) = [r for r in room["map_event_section_commands"] if key in r["args"]]
    native = data["map_scripts"][room["attributes"]["map_events_label"]][
        row["command_index"]
    ]
    require(
        row["args"][3] == change["spritemovedata"]
        and row["args"][10] == "1"
        and native["args"] == row["args"],
        "native trainer event",
    )
    source = json.loads((REGION / "routes" / "MN_R02.json").read_text())
    (tess,) = [o for o in source["trainers"] if o["id"] == key]
    (party,) = [
        v["party"]
        for v in data["trainers"]["trainers"].values()
        if v.get("trainer_id") == key
    ]
    require(
        tess["facing"] == "east"
        and tess["sight_range"] == 1
        and tess["party"] == [{k: p[k] for k in ("species", "level")} for p in party],
        "route dossier",
    )
    # Use only ordinary bidirectional walking. No ledge, field move, water or door shortcuts.
    width = room["attributes"]["width"]
    bank = data["tilesets"][room["attributes"]["tileset_name"]]["collision"]
    terrain = {}
    for i, b in enumerate(room["blocks"]):
        for q, role in enumerate(bank[f"{b:02x}"]):
            if role in ("FLOOR", "TALL_GRASS", "UP_WALL"):
                terrain[2 * (i % width) + q % 2, 2 * (i // width) + q // 2] = role
    for npc in room["objects"]:
        terrain.pop((npc["x"], npc["y"]), None)
    east = {p for p in terrain if p[0] == 2 * width - 1}
    west = {p for p in terrain if p[0] == 0}
    require(
        len(east) == len(west) == 2
        and {x["direction"]: x["target_map"] for x in room["attributes"]["connections"]}
        == {"east": "MnRochester", "west": "MnMankato"},
        "route entry lanes",
    )

    def distances(start, avoid=()):
        out = dict.fromkeys(start, 0)
        queue = deque(start)
        while queue:
            x, y = queue.popleft()
            for a, b in ((x - 1, y), (x + 1, y), (x, y - 1), (x, y + 1)):
                if (
                    (a, b) in out
                    or (a, b) in avoid
                    or (a, b) not in terrain
                    or (b < y and terrain[x, y] == "UP_WALL")
                    or (b > y and terrain[a, b] == "UP_WALL")
                ):
                    continue
                out[a, b] = out[x, y] + 1
                queue.append((a, b))
        return out

    near = (52, 21)
    require(
        terrain[near] == terrain[53, 21] == "TALL_GRASS", "original prairie habitat"
    )
    eastward, westward = distances(west), distances(east)
    cross = eastward[near] + westward[near]
    bypass = min(distances(east, {near})[p] for p in west)
    require(
        (westward[near], cross, bypass) == (13, 76, 76),
        "both lanes must be genuine walking choices",
    )


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("parent", type=Path)
    parser.add_argument("child", type=Path)
    args = parser.parse_args()
    previous = load_pack(args.parent)["data"]
    current = load_pack(args.child)["data"]
    check_watch(current, previous)
    check_tess(current, previous)
    print(
        "2,048 clinic check-ins, 16 west-watch returns, original prize and stamp scripts, and two 76-step prairie crossings passed."
    )
