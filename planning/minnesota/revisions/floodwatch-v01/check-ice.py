#!/usr/bin/env python3
"""Trace the Minneapolis dispatch rescue using the installed actors, tiles, warps and scripts."""

import argparse
from collections import deque
import json
from pathlib import Path
import runpy
import sys

HERE = Path(__file__).resolve().parent
REGION = HERE.parents[1]
sys.path.insert(0, str(REGION / "tools"))
from audit_story_flow import native_pages, require
from render_reference import load_pack

trail = runpy.run_path(str(HERE.parent / "northern-trails-v01" / "check-voyageurs.py"))
Trail, DIRECTIONS = trail["Trail"], trail["DIRECTIONS"]
NETWORK = json.loads((REGION / "lairs/ice-headquarters/network.json").read_text())
OFFICE, CITY, HOME = (
    "MnMinneapolisDispatchOffice",
    "MnMinneapolis",
    "MnMinneapolisDispatcherHouse",
)
CARGO, OPS, RECORDS = "MnIceCargoBasement", "MnIceOperations", "MnIceRecords"
TRANSFER, MAINT, RELAY = "MnIceTransfer", "MnIceMaintenance", "MnIceRelayFloor"
INDOORS = {OFFICE, CARGO, OPS, RECORDS, TRANSFER, MAINT, RELAY, HOME}
WALK = {"FLOOR", "LADDER", "WARP_PANEL", "WARP_CARPET_DOWN"}


def c(op, *args):
    return {"command": op, "args": list(args)}


class Rescue:
    def __init__(self, data):
        self.data = data
        self.rooms = data["maps"]
        self.trail = Trail(data, INDOORS | {CITY})
        self.exits = {}
        names = {
            self.rooms[n]["attributes"]["map_constant"]: n for n in INDOORS | {CITY}
        }
        for name in INDOORS | {CITY}:
            for w in self.rooms[name]["events"]["warps"]:
                target = names.get(w["target_map_constant"])
                if target:
                    landing = self.rooms[target]["events"]["warps"][
                        w["target_warp_id"] - 1
                    ]
                    self.exits[(name, w["x"], w["y"])] = (
                        target,
                        landing["x"],
                        landing["y"],
                    )

    def blocks(self, name, label):
        room = self.rooms[name]
        collision = self.data["tilesets"][room["attributes"]["tileset_name"]][
            "collision"
        ]
        commands = [
            row for row in room["scripts"][label] if row["command"] == "changeblock"
        ]
        require(commands, f"{name}/{label} changes terrain")
        for cmd in commands:
            x, y, block = map(int, cmd["args"])
            for i, token in enumerate(collision[f"{block:02x}"]):
                self.trail.grid[name][y + i // 2][x + i % 2] = token

    def reach(self, start, rooms, panels=True):
        seen = {start}
        queue = deque([start])
        while queue:
            name, x, y = queue.popleft()
            for dx, dy in DIRECTIONS.values():
                xx, yy = x + dx, y + dy
                at = name, xx, yy
                kind = self.trail.kind(*at)
                if (xx, yy) in self.trail.bodies[name] or kind not in WALK:
                    continue
                if kind == "WARP_PANEL" and not panels:
                    continue
                if (
                    kind in ("LADDER", "WARP_PANEL", "WARP_CARPET_DOWN")
                    and at in self.exits
                ):
                    at = self.exits[at]
                if at[0] in rooms and at not in seen:
                    seen.add(at)
                    queue.append(at)
        return seen

    def track(self, name, label, start, occupied):
        x, y = start
        steps = self.rooms[name]["scripts"][label]
        require(
            steps == self.data["map_scripts"][label], f"{label} global movement mirror"
        )
        require(steps[-1] == c("step_end"), f"{label} ends movement")
        mirrors = [
            r["steps"]
            for r in self.rooms[name]["script_movements"]
            if r["label"] == label
        ]
        require(
            mirrors
            and all(
                [(r["command"], r["direction"]) for r in rows]
                == [(row["command"], next(iter(row["args"]), None)) for row in steps]
                for rows in mirrors
            ),
            f"{label} installed movement program",
        )
        collisions = []
        for step in steps[:-1]:
            require(step["command"] == "step", f"{label} native step")
            dx, dy = DIRECTIONS[step["args"][0]]
            x, y = x + dx, y + dy
            kind = self.trail.kind(name, x, y)
            if (x, y) in occupied or kind not in WALK:
                collisions.append((x, y, kind))
        return (x, y), collisions


def neighbor(area, name, coord):
    x, y = coord
    return any((name, x + dx, y + dy) in area for dx, dy in DIRECTIONS.values())


def scene_and_gate(r, baseline):
    room = r.rooms[OFFICE]
    programs = room["scripts"]
    cast = r.trail.bodies[OFFICE]
    triggers = room["events"]["coord_events"]
    require(
        sorted((v["x"], v["y"]) for v in triggers) == [(10, 13), (11, 13)],
        "two entrance lanes",
    )
    for origin in triggers:
        script = programs[origin["script_name"]]
        require(
            script[0] == c("checkevent", "EVENT_MN_REED_JOINED_ICE")
            and script[2] == c("appear", "MN_NICE_REED")
            and script[3]["args"][0] == "PLAYER",
            "recruitment staging",
        )
        end, collisions = r.track(
            OFFICE, script[3]["args"][1], (origin["x"], origin["y"]), cast
        )
        expected = [(11, 9, "FLOOR")] if baseline and origin["x"] == 11 else []
        require(
            end == (11, 8) and collisions == expected,
            f"{origin['script_name']} player crossed {collisions}",
        )
    require(
        r.trail.kind(OFFICE, 5, 2) == "WALL", "cargo stairs closed before recruitment"
    )
    r.blocks(OFFICE, "MnNICEOfficeTiles")
    reed = (11, 9)
    end, obstacles = r.track(
        OFFICE, "MnNICEReedToStair", reed, (cast - {reed}) | {(11, 8)}
    )
    require(
        end == (5, 2) and not obstacles and r.trail.kind(OFFICE, *end) == "LADDER",
        "Reed exits on the actual cargo stair",
    )
    cast.remove(reed)
    local = r.reach((OFFICE, 11, 8), {OFFICE, CARGO})
    require(
        neighbor(local, CARGO, (23, 17)) and not neighbor(local, CARGO, (23, 4)),
        "cargo breaker accessible before its stair opens",
    )
    r.blocks(CARGO, "MnNICEBreaker")
    require(
        r.exits[(OFFICE, 5, 2)] == (CARGO, 4, 19)
        and r.exits[(CARGO, 5, 18)] == (OFFICE, 5, 4),
        "office stair return",
    )


def headquarters(r, baseline):
    links = {}
    for link in NETWORK["links"]:
        for near, far in [(link["a"], link["b"]), (link["b"], link["a"])]:
            source = (near["map"], *near["active"])
            landing = (far["map"], *far["arrival"])
            require(
                r.exits[source] == landing
                and r.trail.kind(*landing) == "FLOOR"
                and r.trail.kind(*source)
                == ("WARP_PANEL" if link["kind"] == "panel" else "LADDER"),
                f"{link['id']} reciprocal active port and safe arrival",
            )
            links[(near["map"], link["id"])] = source
            if link["kind"] == "panel":
                panel = next(
                    b["script"]
                    for b in r.rooms[near["map"]]["events"]["bg_events"]
                    if b["script"].endswith("Panel" + link["id"])
                )
                label = next(
                    c["args"][0]
                    for c in r.rooms[near["map"]]["scripts"][panel]
                    if c["command"] in ("jumptext", "writetext")
                )
                text = " ".join(
                    sum(
                        native_pages(
                            r.rooms[near["map"]]["script_text_bodies"][label][
                                "commands"
                            ]
                        ),
                        [],
                    )
                )
                require(
                    link["id"] in text
                    and far["map"].removeprefix("MnIce").upper() in text,
                    f"{panel} names physical destination",
                )
    start = (OPS, 4, 19)
    before = r.reach(start, INDOORS)
    ada = (RECORDS, 24, 5)
    require(
        ada in before and ada not in r.reach(start, INDOORS, panels=False),
        "card requires the moving pads",
    )
    require((RELAY, 4, 15) not in before, "relay sealed before card")
    ira_floor = r.reach(start, {OPS})
    ada_floor = r.reach(ada, {RECORDS})
    require(
        neighbor(ira_floor, OPS, (5, 15))
        and not neighbor(ira_floor, OPS, links[OPS, "H"][1:]),
        "H is across the locked return shutter from Ira",
    )
    require(
        not neighbor(ada_floor, RECORDS, links[RECORDS, "E"][1:]),
        "E is in a separate records wing from Ada",
    )
    if baseline:
        for name, label, words in [
            (OPS, "MnIceOperationsWorkerText", "through on pad H"),
            (RECORDS, "MnIceHQCardGivenText", "pad B or E"),
            (RECORDS, "MnIceHQCardAfterText", "Pad B or E"),
        ]:
            pages = native_pages(r.rooms[name]["script_text_bodies"][label]["commands"])
            require(words in " ".join(sum(pages, [])), f"original directions {label}")
    # Follow what these particular speakers can offer from their side of the wall.
    at = start
    for name, next_port in [
        (OPS, "records_stair"),
        (RECORDS, "workshop_stair"),
        (MAINT, "A"),
        (TRANSFER, "B"),
    ]:
        port = links[name, next_port]
        require(
            neighbor(r.reach(at, {name}), name, port[1:]),
            f"{name} can use the advertised {next_port}",
        )
        at = r.exits[port]
    require(
        ada in r.reach(at, {RECORDS}), "arrive beside Ada beginning on the legal route"
    )
    h = links[RECORDS, "H"]
    require(
        neighbor(r.reach(ada, {RECORDS}), RECORDS, h[1:]), "Ada can actually point to H"
    )
    at = r.exits[h]
    for door in ["RETURN", "OPERATIONS"]:
        design = next(d for d in NETWORK["doors"] if d["id"] == door)
        x, y = design["metatile"]
        near = r.reach(at, {OPS})
        entry = "MnIceHQDoor" + door
        require(
            any(
                neighbor(near, OPS, (2 * x + dx, 2 * y + dy))
                for dx in range(2)
                for dy in range(2)
            ),
            f"{entry} approachable from H",
        )
        prog = r.rooms[OPS]["scripts"][entry]
        require(
            c("checkevent", "EVENT_MN_ICE_HQ_CARD_RECEIVED") in prog
            and c("checkitem", "CARD_KEY") in prog
            and c("setevent", design["flag"]) in prog
            and c("changeblock", str(2 * x), str(2 * y), "7") in prog,
            f"{entry} gated by held card",
        )
        require(
            c("checkevent", design["flag"])
            in r.rooms[OPS]["scripts"][entry + "Restore"],
            f"{entry} persists on return",
        )
        r.blocks(OPS, entry)
    require(
        neighbor(r.reach(at, {OPS}), OPS, links[OPS, "relay_stair"][1:]),
        "two card shutters open the relay stair",
    )
    power = (RELAY, 20, 2)
    require(
        neighbor(r.reach(start, INDOORS), RELAY, power[1:]),
        "reach the cell power control",
    )
    relay = r.rooms[RELAY]["scripts"]["MnNICEHolding"]
    require(
        c("checkevent", "EVENT_MN_REED_RELAY_DEFEATED") in relay
        and c("checkevent", "EVENT_MN_ICE_CARGO_FEED_OFF") in relay
        and c("setevent", "EVENT_MN_ICE_HOLDING_POWER_OFF") in relay,
        "relay needs Reed and the cargo breaker",
    )
    require(
        (OFFICE, 11, 6) in r.reach((RELAY, 20, 3), INDOORS),
        "return from relay to public side of latch",
    )
    if not baseline:
        for name, label, fragments in [
            (
                OPS,
                "MnIceOperationsWorkerText",
                ["north stairs", "MAINTENANCE", "pad A", "Pad B"],
            ),
            (
                RECORDS,
                "MnIceHQCardGivenText",
                ["pad H", "RETURN shutter", "relay stairs"],
            ),
            (
                RECORDS,
                "MnIceHQCardAfterText",
                ["Pad H", "RETURN shutter", "OPERATIONS door", "relay stairs"],
            ),
        ]:
            body = r.rooms[name]["script_text_bodies"][label]["commands"]
            native = [{k: c[k] for k in ("command", "args")} for c in body]
            require(
                native
                == r.rooms[name]["scripts"][label]
                == r.data["map_scripts"][label],
                f"{label} native mirrors",
            )
            pages = native_pages(body)
            canon = json.loads((REGION / "story.json").read_text())[
                "dialogue_overhaul"
            ]["texts"][name][label]
            require(
                pages == canon
                and all(w in " ".join(sum(pages, [])) for w in fragments),
                f"installed directions {label}",
            )


def rescue_and_return(r):
    room = r.rooms[OFFICE]
    scripts = room["scripts"]
    actor = "MN_ICE_CAPTIVE_DISPATCHER"
    latch = scripts["MnMPCellLatch"]
    note = scripts["MnMPDispatchCopy"]
    rescued = "EVENT_MN_ICE_CAPTIVE_RESCUED"
    require(
        c("checkevent", "EVENT_MN_ICE_HOLDING_POWER_OFF") in latch
        and c("checkevent", "EVENT_MN_ICE_ENFORCER_DEFEATED") in latch
        and c("yesorno") in latch
        and c("setevent", rescued) in latch
        and c("setevent", "EVENT_MN_DISPATCH_NOTE") not in latch,
        "capture of Ellis and evidence remain separate",
    )
    r.blocks(OFFICE, "MnMPCellLatch")
    start = (15, 5)
    occupied = (r.trail.bodies[OFFICE] - {start}) | {(11, 6)}
    for command in latch:
        if command["command"] == "applymovement" and command["args"][0] == actor:
            start, hits = r.track(OFFICE, command["args"][1], start, occupied)
            require(not hits, f"Ellis crosses {hits} leaving the cell")
    require(
        start == (11, 15) and c("disappear", actor) in latch,
        "Ellis exits the south door",
    )
    r.trail.bodies[OFFICE].remove((15, 5))
    require(
        c("checkevent", rescued) in note
        and c("setevent", "EVENT_MN_DISPATCH_NOTE") in note
        and neighbor(r.reach((OFFICE, 11, 6), {OFFICE}), OFFICE, (9, 11)),
        "the stamp can be recovered from the physical desk",
    )
    outside = r.exits[OFFICE, 11, 15]
    doorstep = r.exits[HOME, 5, 7]
    require(
        outside[0] == doorstep[0] == CITY and r.trail.both(outside, doorstep, dry=True),
        "city walk between dispatch office and Ellis's home",
    )
    require(
        r.exits[CITY, 25, 33] == (OFFICE, 10, 14)
        and r.exits[CITY, 23, 41] == (HOME, 5, 6),
        "returning from both thresholds",
    )
    require(
        neighbor(r.reach((HOME, 5, 6), {HOME}), HOME, (7, 5)),
        "Ellis is approachable at home",
    )


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("pack", type=Path)
    parser.add_argument(
        "--baseline",
        action="store_true",
        help="verify the diagnosed pre-revision staging and route",
    )
    args = parser.parse_args()
    rescue = Rescue(load_pack(args.pack)["data"])
    scene_and_gate(rescue, args.baseline)
    headquarters(rescue, args.baseline)
    rescue_and_return(rescue)
    print(
        "Two entrance scenes, Reed's exit, cargo breaker, 12 reciprocal stair/pad pairs, 16 panel signs, Ada's route, both required shutters, relay return, Ellis's exit, separate evidence desk and walk home verified."
    )
