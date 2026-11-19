#!/usr/bin/env python3
"""Check native journey text, League eligibility, and the roads named on three signs."""

import argparse
from collections import deque
import json
from pathlib import Path
import sys

HERE = Path(__file__).resolve().parent
REGION = HERE.parents[1]
sys.path.insert(0, str(REGION / "tools"))
from audit_story_flow import native_pages, read_branch, require
from render_reference import load_pack

BADGES = (
    "ENGINE_ZEPHYRBADGE",
    "ENGINE_HIVEBADGE",
    "ENGINE_PLAINBADGE",
    "ENGINE_FOGBADGE",
    "ENGINE_STORMBADGE",
    "ENGINE_MINERALBADGE",
    "ENGINE_GLACIERBADGE",
    "ENGINE_RISINGBADGE",
)
RESTORED = "EVENT_MN_SURVEY_RESTORED"
REGISTERED = "EVENT_MN_LEAGUE_RUN_ACTIVE"
SUPPER = "EVENT_MN_WINONA_CHAMPION_SUPPER"


def check_native_text(data, name, label, pages):
    room = data["maps"][name]
    program = room["scripts"][label]
    require(native_pages(program) == pages, f"{name}/{label} changed pages")
    require(
        data["map_scripts"][label] == program, f"{name}/{label} global script differs"
    )
    native = room["script_text_bodies"][label]["commands"]
    require(
        native == [dict(command, command_index=i) for i, command in enumerate(program)],
        f"{name}/{label} runtime text differs",
    )


def check_league(room):
    for mask in range(256):
        owned = [flag for i, flag in enumerate(BADGES) if mask & (1 << i)]
        for restored in (False, True):
            for accept in (False, True):
                initial = owned + ([RESTORED] if restored else [])
                actions, texts, final = read_branch(
                    room, "MnElyRegister", initial, accept
                )
                can_enter = mask == 255 and restored and accept
                if mask < 255:
                    missing = next(
                        i for i, flag in enumerate(BADGES, 1) if flag not in owned
                    )
                    expected = [f"MnElyMissingBadge{missing}Text"]
                elif not restored:
                    expected = ["MnElySurveyDeniedText"]
                else:
                    expected = ["MnElyRegistrationText"] + (
                        [] if accept else ["MnElyRegistrationDeclinedText"]
                    )
                require(
                    texts == expected,
                    f"incorrect registrar instructions for {mask}/{restored}/{accept}",
                )
                require(
                    (REGISTERED in final) == can_enter,
                    "registration flag disagrees with admission",
                )
                warps = [
                    action for action in actions if action[0] in ("warp", "warpfacing")
                ]
                require(
                    warps
                    == (
                        [("warpfacing", "MN_LEAGUE_ROOM_1", "4", "16", "UP")]
                        if can_enter
                        else []
                    ),
                    "registrar sent the player to the wrong place",
                )


def check_home(room):
    flags = {"EVENT_MN_HOME_INTRO", RESTORED}
    _, reply, result = read_branch(room, "MN_W_GUARDIAN", flags)
    require(
        "MnWinonaChampionSupperText" not in reply and SUPPER not in result,
        "premature championship supper",
    )
    for previous, expected in (
        (False, "MnWinonaChampionSupperText"),
        (True, "MnWinonaChampionRestText"),
    ):
        flags.add("EVENT_MN_LEAGUE_CHAMPION")
        if previous:
            flags.add(SUPPER)
        actions, texts, final = read_branch(room, "MN_W_GUARDIAN", flags)
        require(
            texts == [expected]
            and SUPPER in final
            and ("special", "HealParty") in actions
            and ("blackoutmod", "MN_WINONA_HOME_1F") in actions,
            "Winona homecoming or heal failed",
        )


def adjacent(pos):
    x, y = pos
    return {(x - 1, y), (x + 1, y), (x, y - 1), (x, y + 1)}


def ground(data, name, withdrawn=()):
    room = data["maps"][name]
    width = room["attributes"]["width"]
    collision = data["tilesets"][room["attributes"]["tileset_name"]]["collision"]
    actors = {(o["x"], o["y"]) for o in room["objects"] if o["script"] not in withdrawn}
    walk = set()
    for i, block in enumerate(room["blocks"]):
        for q, role in enumerate(collision[f"{block:02x}"]):
            if role in {"FLOOR", "TALL_GRASS", "LONG_GRASS"}:
                walk.add((2 * (i % width) + q % 2, 2 * (i // width) + q // 2))
    return walk - actors


def walking_distances(floor, start):
    initial = adjacent(start) & floor
    require(bool(initial), f"no place to read sign at {start}")
    distances = {p: 0 for p in initial}
    todo = deque(initial)
    while todo:
        p = todo.popleft()
        for q in adjacent(p) & floor:
            if q not in distances:
                distances[q] = distances[p] + 1
                todo.append(q)
    return distances


def sign(data, name, script):
    room = data["maps"][name]
    events = [
        e
        for e in room["events"]["bg_events"]
        if e["event_type"] == "BGEVENT_READ" and e["script"] == script
    ]
    require(
        len(events) == 1
        and room["scripts"][script]
        == [{"command": "jumptext", "args": [script + "Text"]}],
        f"missing readable sign {name}/{script}",
    )
    return events[0]["x"], events[0]["y"]


def directions(data):
    city = data["maps"]["MnStPaul"]
    start = sign(data, "MnStPaul", "MN_ST_PAUL_SIGN_WAYFINDING")
    streets = walking_distances(ground(data, "MnStPaul"), start)
    for destination, west in (
        ("MN_ST_PAUL_UNION_DEPOT", True),
        ("MN_ST_PAUL_CAPITOL_HALL", False),
    ):
        (door,) = [
            w
            for w in city["events"]["warps"]
            if w["target_map_constant"] == destination
        ]
        x, y = door["x"], door["y"]
        require(
            (x, y + 1) in streets and y < start[1] and (x < start[0]) == west,
            f"wrong way to {destination}",
        )
    north = ["MnR11C00R00", "MnR11C01R00", "MnGrandMarais"]
    trail = set()
    offset = 0
    for i, name in enumerate(north):
        room = data["maps"][name]
        trail.update((x + offset, y) for x, y in ground(data, name))
        if i < 2:
            forward = room["attributes"]["connections"]
            back = data["maps"][north[i + 1]]["attributes"]["connections"]
            require(
                any(
                    c["direction"] == "east"
                    and c["target_map"] == north[i + 1]
                    and c["offset"] == 0
                    for c in forward
                )
                and any(
                    c["direction"] == "west"
                    and c["target_map"] == name
                    and c["offset"] == 0
                    for c in back
                ),
                "North Shore trail disconnected",
            )
            offset += room["attributes"]["width"] * 2
    distance = walking_distances(trail, sign(data, north[0], "MN_R11_SIGN_01"))
    require(
        any(x >= offset for x, y in distance), "Grand Marais inaccessible from the sign"
    )
    name = "MnLakeItasca"
    gate = data["maps"][name]["scripts"]["MnCohesionItascaWest"]
    require(
        [c["command"] for c in gate] == ["disappear", "disappear", "end"],
        "west trail staff no longer step aside",
    )
    officers = [
        o
        for o in data["maps"][name]["objects"]
        if o["script"] == "MnCohesionItascaWest"
    ]
    require(
        len(officers) == 2
        and {c["args"][0] for c in gate[:2]} == {o["object_identifier"] for o in officers},
        "staff did not open both trail lanes",
    )
    to_west = walking_distances(
        ground(data, name, {"MnCohesionItascaWest"}),
        sign(data, name, "MN_ITASCA_WEST_CLOSED"),
    )
    require(
        (0, 28) in to_west
        and (0, 29) in to_west
        and any(
            c["direction"] == "west" and c["target_map"] == "MnR19C00R00"
            for c in data["maps"][name]["attributes"]["connections"]
        ),
        "missing signed Itasca trail",
    )


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("candidate", type=Path)
    data = load_pack(parser.parse_args().candidate)["data"]
    story = json.loads((REGION / "story.json").read_text())
    for name, labels in story["journey_landmark_dialogue"].items():
        for label, pages in labels.items():
            check_native_text(data, name, label, pages)
    check_native_text(
        data,
        "MnStPaul",
        "MN_ST_PAUL_SIGN_WAYFINDINGText",
        [["NW - UNION DEPOT", "NE - CAPITOL HALL"], ["WEST - CITY CROSS"]],
    )
    check_native_text(
        data,
        "MnR11C00R00",
        "MN_R11_SIGN_01Text",
        [["NORTH SHORE TRAIL", "GRAND MARAIS: EAST"]],
    )
    check_league(data["maps"]["MnLeagueLobby"])
    check_home(data["maps"]["MnWinonaHome1F"])
    directions(data)
    print(
        "20 journey conversations, 1,024 registration branches, homecoming and signed roads passed."
    )


if __name__ == "__main__":
    main()
