#!/usr/bin/env python3
"""Check every Glacier Line destination, boarding path and public walk from compiled data."""

import argparse
from collections import deque
import json
from pathlib import Path
import runpy
import sys

HERE = Path(__file__).resolve().parent
REGION = HERE.parents[1]
sys.path.insert(0, str(REGION / "tools"))
from audit_story_flow import instructions, require
from render_reference import load_pack

land = runpy.run_path(str(HERE.parent / "lake-observations-v01" / "check-hidden.py"))
quadrant, coords = land["quadrant"], land["coords"]
directions = land["DIRECTIONS"]
STOPS = json.loads((REGION / "transit/glacier-current-v01/service.json").read_text())[
    "stations"
]
PORTS = {
    r["id"]: r
    for r in json.loads(
        (REGION / "transit/glacier-rear-rail-v01/city-access.json").read_text()
    )["paths"]
}


def traverse(room, entry, flags=(), choice=None):
    """Follow the actual menu branches, retaining the reached menu and ordered commands."""
    label = entry
    value = None
    actions = []
    menu = None
    for _ in range(16):
        for index, action in enumerate(instructions(room, label)):
            op, *args = action
            actions.append(action)
            if op == "checkevent":
                value = args[0] in flags
            elif op == "verticalmenu":
                menu = room["script_vertical_menus"][f"{label}:{index}"]["options"]
                value = choice
            elif (
                op == "sjump"
                or (op == "iftrue" and value)
                or (op == "iffalse" and not value)
                or (op == "ifequal" and int(args[0]) == value)
            ):
                label = args[-1]
                break
            elif op in {"end", "endcallback"}:
                return actions, menu
        else:
            raise ValueError(f"{room['id']}/{label} fell through")
    raise ValueError(f"{room['id']}/{entry} did not terminate")


def blockages(room, qualified=False):
    """Reserve all authored actors, including optional passengers, and unrelated trigger squares."""
    bodies = {a["object_identifier"] or a["script"]: coords(a) for a in room["objects"]}
    events = {coords(a) for a in room["events"]["coord_events"]}
    staged = {
        (int(x), int(y))
        for script in room["scripts"].values()
        for command in script
        if command["command"] == "moveobject"
        for who, x, y in [command["args"]]
        if who in bodies and not (qualified and who.startswith("MN_W_TRAIL_WARDEN_"))
    }
    if qualified:
        starter = {"EVENT_MN_STARTER_RECEIVED"}
        entry = next(
            x["args"][1]
            for x in room["map_script_section_commands"]
            if x["command"] == "callback"
            and x["args"] == ["MAPCALLBACK_OBJECTS", "MnWTrailWardenCallback"]
        )
        actions, _ = traverse(room, entry, starter)
        for cmd, *args in actions:
            if cmd == "moveobject":
                require(args[0] in bodies, "missing trail warden")
                bodies[args[0]] = int(args[1]), int(args[2])
        for row in room["events"]["coord_events"]:
            path, _ = traverse(room, row["script_name"], starter)
            require(
                [r[0] for r in path] == ["checkevent", "iftrue", "end"],
                "Winona starter trail should be open",
            )
        events.clear()
    return set(bodies.values()) | events | staged


def paths(data, name, start, blocked):
    """Ordinary two-way footing. Other doors, ice, ledges and coordinate scenes are excluded."""
    room = data["maps"][name]
    attrs = room["attributes"]
    allowed = {
        (x, y)
        for y in range(attrs["height"] * 2)
        for x in range(attrs["width"] * 2)
        if quadrant(data, name, x, y)
        in {"FLOOR", "TALL_GRASS", "BRIDGE_NS_DECK", "BRIDGE_EW_DECK"}
    } - blocked
    require(start in allowed, f"{name} blocked start {start}")
    seen = {start: 0}
    queue = deque([start])
    while queue:
        x, y = queue.popleft()
        for dx, dy in directions.values():
            nxt = x + dx, y + dy
            if nxt in allowed and nxt not in seen:
                seen[nxt] = seen[x, y] + 1
                queue.append(nxt)
    return seen


def find_warp(room, index):
    [warp] = [w for w in room["events"]["warps"] if w["index"] == index]
    return warp


def street(data, stop):
    port = PORTS[stop["id"]]
    name, other = port["city"], port["native_connection"]["target_map"]
    room, hall = data["maps"][name], data["maps"][stop["map"]]
    edge, far = tuple(port["city_port"]), tuple(port["neighbor_port"])
    side = port["native_connection"]["direction"]
    opposite = {"west": "east", "south": "north"}[side]
    for src, dst, orientation, a, b in (
        (name, other, side, edge, far),
        (other, name, opposite, far, edge),
    ):
        attr = data["maps"][src]["attributes"]
        [link] = [e for e in attr["connections"] if e["direction"] == orientation]
        target = data["maps"][dst]["attributes"]
        if orientation in ("east", "west"):
            require(
                a[0] == (0 if orientation == "west" else 2 * attr["width"] - 1),
                f"{src} seam",
            )
            projected = (
                2 * target["width"] - 1 if orientation == "west" else 0,
                a[1] - 2 * link["offset"],
            )
        else:
            require(
                a[1] == (0 if orientation == "north" else 2 * attr["height"] - 1),
                f"{src} seam",
            )
            projected = (
                a[0] - 2 * link["offset"],
                2 * target["height"] - 1 if orientation == "north" else 0,
            )
        require(
            link["target_map"] == dst
            and projected == b
            and quadrant(data, src, *a) == "FLOOR"
            and a not in blockages(data["maps"][src], src == "MnWinona"),
            f"{src} road connection to {dst}",
        )
    door, porch = tuple(port["public_door"]), tuple(port["public_porch"])
    blocked = blockages(room, name == "MnWinona")
    road = paths(data, name, edge, blocked)
    require(
        road.get(porch) and door == (porch[0], porch[1] - 1) and door not in blocked,
        f"{name} station approach",
    )
    exterior = find_warp(room, stop["parent_entry_warp"])
    arrival = find_warp(hall, exterior["target_warp_id"])
    require(
        coords(exterior) == door
        and quadrant(data, name, *door) == "DOOR"
        and exterior["target_map_constant"] == hall["attributes"]["map_constant"]
        and coords(arrival) == tuple(stop["public_arrival_core"]),
        f"{name} street to concourse",
    )
    for index in stop["exit_warp_indices"]:
        outside = find_warp(hall, index)
        returned = find_warp(room, outside["target_warp_id"])
        require(
            outside["target_map_constant"] == room["attributes"]["map_constant"]
            and coords(returned) == porch,
            f"{name} doors return to porch",
        )
    return road[porch]


def hall(data, stop):
    name = stop["map"]
    room = data["maps"][name]
    prefix = "MnGlacier" + stop["id"]
    [conductor] = [
        o for o in room["objects"] if o["script"] == stop["conductor_script"]
    ]
    require(
        conductor["event_flag"] == "-1"
        and conductor["spritemovedata"] == "SPRITEMOVEDATA_STANDING_DOWN",
        name,
    )
    body, approach = coords(conductor), tuple(stop["approach_core"])
    require(body == (approach[0], approach[1] - 1), f"{name} speaker")
    blocked = blockages(room)
    public = paths(data, name, tuple(stop["public_arrival_core"]), blocked)
    arrivals = tuple(stop["ride_arrival_core"])
    require(approach in public and arrivals in public, f"{name} open concourse")
    require(
        {
            p
            for dx, dy in directions.values()
            if (p := (body[0] + dx, body[1] + dy)) in public
        }
        == {approach},
        f"{name} permits boarding from an unintended square",
    )
    exits = []
    for index in stop["exit_warp_indices"]:
        x, y = coords(find_warp(room, index))
        require(
            quadrant(data, name, x, y) == "WARP_CARPET_DOWN"
            and (x, y) not in blocked
            and (x, y - 1) in public,
            f"{name} exit carpet",
        )
        exits.append((x, y - 1))
    require(
        all(
            tuple(stop[k]) not in public
            for k in ("platform_core", "train_door_core", "other_train_door_core")
        ),
        f"{name} conductor bypass",
    )
    triggers = {
        coords(a) for a in room["events"]["warps"] + room["events"]["coord_events"]
    }
    require(arrivals not in triggers, f"{name} unsafe ride arrival")
    parked = {"PLAYER": approach, conductor["object_identifier"]: body}
    others = blocked - {body}
    door = tuple(stop["train_door_core"])
    choreography = [
        (conductor["object_identifier"], prefix + "ConductorOpen"),
        ("PLAYER", prefix + "PlayerBoard"),
        (conductor["object_identifier"], prefix + "ConductorReset"),
    ]
    for who, program in choreography:
        steps = instructions(room, program)
        require(steps[-1] == ("step_end",) and len(steps) > 1, program)
        for i, (op, direction) in enumerate(steps[:-1]):
            dx, dy = directions[direction]
            x, y = parked[who]
            p = x + dx, y + dy
            kind = "DOOR" if who == "PLAYER" and i == len(steps) - 2 else "FLOOR"
            require(
                op == "step"
                and quadrant(data, name, *p) == kind
                and p not in others | triggers
                and all(p != pos for actor, pos in parked.items() if actor != who),
                f"{name} {who} hits obstacle at {p}",
            )
            parked[who] = p
    require(
        parked == {"PLAYER": door, conductor["object_identifier"]: body},
        f"{name} failed to board or reset conductor",
    )
    return choreography, min(paths(data, name, arrivals, blocked)[p] + 1 for p in exits)


def check_menus(room, origin, stop, choreography):
    choices = [s for i, s in enumerate(STOPS) if i != origin]
    label, prefix = stop["conductor_script"], "MnGlacier" + stop["id"]
    [visit] = [
        row["args"][1]
        for row in room["map_script_section_commands"]
        if row["command"] == "callback" and row["args"][0] == "MAPCALLBACK_NEWMAP"
    ]
    require(
        instructions(room, visit)
        == [("setevent", stop["visited_flag"]), ("endcallback",)],
        f"{label} unlocks on hall entry",
    )
    wins = set()
    for mask in range(16):
        flags = {
            s["visited_flag"] for j, s in enumerate(choices) if mask & (1 << j)
        } | {stop["visited_flag"]}
        for selection in range(6) if mask else (None,):
            actions, menu = traverse(room, label, flags, selection)
            texts = [a[1] for a in actions if a[0] == "writetext"]
            rides = [
                a
                for a in actions
                if a[0]
                in {"applymovement", "setval", "special", "warpfacing", "newloadmap"}
            ]
            require(
                all(
                    a[0]
                    not in {
                        "setevent",
                        "clearevent",
                        "givemoney",
                        "takemoney",
                        "takeitem",
                    }
                    for a in actions
                ),
                f"{label} unexpectedly changed resources",
            )
            if mask == 0:
                require(
                    menu is None and texts[-1] == prefix + "NoStopsText" and not rides,
                    label,
                )
                continue
            expected = [
                s["menu_label"] + ("" if s["visited_flag"] in flags else " X")
                for s in choices
            ] + ["CANCEL"]
            require(menu == expected, f"{label} visited-only menu {mask}")
            if selection in (0, 5):
                require(
                    not rides and texts[-1] == prefix + "CancelText", f"{label} cancel"
                )
                continue
            dest = choices[selection - 1]
            if dest["visited_flag"] not in flags:
                require(
                    not rides and texts[-1] == prefix + "UnknownText",
                    f"{label} unvisited {dest['id']}",
                )
                continue
            x, y = dest["ride_arrival_core"]
            want = [("applymovement", *c) for c in choreography] + [
                ("setval", "TRUE" if STOPS.index(dest) < origin else "FALSE"),
                ("special", "MagnetTrain"),
                ("warpfacing", dest["constant"], str(x), str(y), "DOWN"),
                ("newloadmap", "MAPSETUP_TRAIN"),
                ("applymovement", "PLAYER", prefix + "Arrive"),
            ]
            require(
                rides == want
                and instructions(room, prefix + "Arrive")
                == [("turn_head", "DOWN"), ("step_end",)],
                f"{label} -> {dest['id']}",
            )
            wins.add(dest["id"])
    require(len(wins) == 4, f"{label} missing destination")
    return visit


def visit_sources(data):
    setters = {stop["visited_flag"]: [] for stop in STOPS}
    for name, room in data["maps"].items():
        if not name.startswith("Mn"):
            continue
        for label, rows in room["scripts"].items():
            for row in rows:
                if row["command"] == "setevent" and row["args"][0] in setters:
                    setters[row["args"][0]].append((name, label))
    return setters


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("pack", type=Path)
    data = load_pack(parser.parse_args().pack)["data"]
    flags = data["story_event_script_constants"]["global"]
    setters = visit_sources(data)
    require(
        len({flags[s["visited_flag"]] for s in STOPS}) == 5,
        "five distinct station visit flags",
    )
    for i, s in enumerate(STOPS):
        road = street(data, s)
        choreo, out = hall(data, s)
        room = data["maps"][s["map"]]
        visit = check_menus(room, i, s, choreo)
        require(
            setters[s["visited_flag"]] == [(s["map"], visit)],
            f"{visit} can unlock another way",
        )
        for name, rows in room["scripts"].items():
            if name in {visit, s["conductor_script"]} or name.startswith(
                "MnGlacier" + s["id"]
            ):
                require(
                    data["map_scripts"][name] == rows,
                    f"{s['map']}/{name} native program differs",
                )
        print(
            f"{s['menu_label']}: road ↔ porch {road} steps, train arrival → exit {out}, 4 destinations."
        )
    print(
        "20/20 directed rides, 80 visited-stop combinations, 450 menu selections and 5 first-stop messages, 5 public streets, 10 exit carpets. Static analysis; optional story triggers avoided, all passenger positions reserved."
    )


if __name__ == "__main__":
    main()
