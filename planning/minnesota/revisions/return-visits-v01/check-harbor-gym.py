#!/usr/bin/env python3
"""Trace the two regular battles, optional branch and earned west ice return in the installed Harbor Gym."""

import argparse
from collections import deque
from itertools import product
from pathlib import Path
import sys

REGION = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(REGION / "tools"))
from audit_story_flow import instructions, native_pages, require
from render_reference import load_pack

GYM, CITY = "MnDuluthGym", "MnDuluth"
ARRIVAL, EXITS = (6, 17), {(6, 19), (7, 19)}
COURT = ((7, 3), (8, 2), (9, 3))
DIRECTIONS = {"U": (0, -1), "R": (1, 0), "D": (0, 1), "L": (-1, 0)}
LANE = {(1, 4): (5, 7), (1, 5): (1, 8), (1, 8): (1, 7), (2, 8): (1, 4)}
TEACHERS = tuple(f"EVENT_MN_GYM_8_TRAINER_{i}_DEFEATED" for i in range(1, 4))
WIN = "EVENT_MN_GYM_8_DEFEATED"
WEST = "EVENT_MN_GYM_8_WEST_RETURN_OPEN"
OPEN = "EVENT_MN_GYM_8_COURT_OPEN"
VICTORY = "MnDuluthGymLeaderWonText"
REFRESH = "MnDuluthGymRefreshGates"
SIGHT = ({(3, 11)}, {(5, 6), (6, 6)}, {(3, 7)})


def coords(row):
    return row["x"], row["y"]


def refresh(room, states, stale=None):
    blocks = list(room["blocks"])
    width = room["attributes"]["width"]
    for (x, y), tile in (stale or {}).items():
        blocks[y * width + x] = tile
    flags = set(states)
    frames = [(REFRESH, 0)]
    condition = False
    for _ in range(80):
        if not frames:
            return blocks, flags
        label, i = frames.pop()
        op, *args = instructions(room, label)[i]
        frames.append((label, i + 1))
        if op == "checkevent":
            condition = args[0] in flags
        elif op == "setevent":
            flags.add(args[0])
        elif op == "clearevent":
            flags.discard(args[0])
        elif op == "scall":
            frames.append((args[0], 0))
        elif op in ("sjump", "iftrue", "iffalse"):
            if op == "sjump" or condition == (op == "iftrue"):
                frames[-1] = (args[0], 0)
        elif op == "changeblock":
            x, y, tile = map(int, args)
            blocks[y // 2 * width + x // 2] = tile
        elif op == "end":
            frames.pop()
        else:
            raise ValueError(f"untraced Harbor gate command {op}")
    raise ValueError("Harbor gate scripts did not terminate")


class Rink:
    def __init__(self, pack, room, blocks):
        a = room["attributes"]
        self.actors = {coords(o) for o in room["objects"]}
        collision = pack["data"]["tilesets"][a["tileset_name"]]["collision"]
        self.tiles = {(x, y): collision[f"{blocks[y//2*a['width']+x//2]:02x}"][(y%2)*2+x%2]
                      for y in range(2*a["height"]) for x in range(2*a["width"])}
        require(set(self.tiles.values()) == {"WALL", "FLOOR", "ICE", "WARP_CARPET_DOWN"}, "unknown rink movement")

    def slide(self, at, button):
        dx, dy = DIRECTIONS[button]
        steps = []
        while True:
            target = at[0] + dx, at[1] + dy
            if target in self.actors or self.tiles.get(target, "WALL") == "WALL":
                return steps
            at = target
            steps.append(at)
            if self.tiles[at] != "ICE":
                return steps

    def paths(self, start, avoid=()):
        paths = {start: ""}
        queue = deque([start])
        while queue:
            at = queue.popleft()
            if at in EXITS:
                continue  # The carpet leaves the room; it cannot be used as a turning stop.
            for direction in DIRECTIONS:
                trajectory = self.slide(at, direction)
                if not trajectory or any(tile in avoid for tile in trajectory):
                    continue
                end = trajectory[-1]
                if end not in paths:
                    paths[end] = paths[at] + direction
                    queue.append(end)
        return paths


def audit(pack, baseline):
    data = pack["data"]
    room = data["maps"][GYM]
    city = data["maps"][CITY]
    (incoming,) = [w for w in city["events"]["warps"] if w["target_map"] == "MN_DULUTH_GYM"]
    doorstep = city["events"]["warps"][3]
    arrivals = room["events"]["warps"]
    require(coords(incoming) == (48, 31) and coords(arrivals[incoming["target_warp_id"] - 1]) == ARRIVAL
            and coords(doorstep) == (48, 32) and {coords(w) for w in arrivals[:2]} == EXITS
            and all((w["target_map"], w["target_warp_id"]) == ("MN_DULUTH", 4) for w in arrivals), "gym entrance/return")
    leader = instructions(room, GYM + "Leader")
    tail = [("setevent", WIN), ("scall", REFRESH), ("sjump", GYM + "LeaderReward")]
    require(leader[-3:] == tail and ("checkflag", "ENGINE_GLACIERBADGE") in leader and ("checkevent", OPEN) in leader
            and ("setflag", "ENGINE_RISINGBADGE") in leader, "Norek's challenge and reward order")
    require(any(r["args"] == ["MAPCALLBACK_TILES", REFRESH] for r in room["map_script_section_commands"]), "reentry refresh")
    body = room["script_text_bodies"][VICTORY]["commands"]
    require(native_pages(body) == native_pages(data["map_scripts"][VICTORY]) == native_pages(room["scripts"][VICTORY]),
            "victory dialogue mirrors")
    if not baseline:
        require(native_pages(body)[1:] == [["The west lane is", "open now."], ["Skate south, then", "east at the stop."]],
                "victory directions")
    actors = {a["script"]: a for a in room["objects"]}
    for i, ((xy, direction, radius), receipt) in enumerate(zip(
        [((3, 10), "DOWN", 1), ((4, 6), "RIGHT", 2), ((2, 7), "RIGHT", 1)], TEACHERS
    ), 1):
        label = f"MN_GYM_8_TRAINER_{i}"
        actor = actors[label]
        require(coords(actor) == xy and actor["sightline_direction_override"] == direction and actor["radius"] == radius
                and actor["object_type"] == "OBJECTTYPE_TRAINER" and instructions(room, label)[0][3] == receipt,
                "native on-sight battle " + label)
    require(actors[GYM+"Leader"]["radius"] == 0 and coords(actors[GYM+"Leader"]) == (8, 3), "leader interaction")
    typed = {(q["source_script"], q["command_index"]): (q["x"], q["y"], q["block_id"]) for q in room["script_block_changes"]}
    raw = {(name, i): tuple(map(int, r["args"])) for name, rows in room["scripts"].items() for i, r in enumerate(rows)
           if r["command"] == "changeblock"}
    require(raw == typed and len(typed) == len(room["script_block_changes"]), "typed gate changeblocks")
    early = []
    for flags, name, count in [(set(), "entry", 33), ({TEACHERS[0]}, "Livra", 53), (set(TEACHERS[:2]), "court", 76),
                                ({WIN, *TEACHERS[:2]}, "badge", 79 if baseline else 67)]:
        blocks, results = refresh(room, flags)
        rink = Rink(pack, room, blocks)
        paths = rink.paths(ARRIVAL)
        stops = paths.keys() - EXITS
        if name != "badge":
            early.append((blocks, dict(paths)))
        require(len(stops) == count and EXITS <= paths.keys() and all(EXITS & rink.paths(s).keys() for s in stops), name + " exit from every stop")
        require(all((x in stops) == (name in ("court", "badge")) for x in COURT), name + " court access")
        if name == "entry":
            require(not (SIGHT[1] | SIGHT[2]) & stops and rink.paths(ARRIVAL)[(3, 11)] == "UULUR", "first rough turn")
        if name == "Livra":
            require((5, 6) in rink.paths(ARRIVAL, SIGHT[2]) and (3, 7) in rink.paths(ARRIVAL, SIGHT[1]), "optional Anver branch")
        if name == "badge":
            for obj in room["objects"]:
                x, y = coords(obj)
                require(any((x+dx, y+dy) in stops for dx, dy in DIRECTIONS.values()), obj["script"] + " badge revisit")
            lengths = [min(len(rink.paths(t)[e]) for e in EXITS) for t in COURT]
            require(lengths == ([14, 16, 18] if baseline else [9, 11, 13]), f"after badge: {lengths}")
            if not baseline:
                start = COURT[0]
                for dr, end in zip("DDDLDRDDD", [(7,4), (7,5), (7,8), (2,8), (2,16), (6,16), (6,17), (6,18), (6,19)]):
                    require(rink.slide(start, dr)[-1] == end, "wrong landing " + str(end))
                    start = end
            print(f"After victory from {COURT}: {lengths} directional inputs to either exit.")
        require((WEST in results) == (WIN in flags), name + " gate receipt")
        print(f"{name}: {len(stops)} reachable indoor resting squares, both carpets reachable.")
    if not baseline:
        width = room["attributes"]["width"]
        for values in product((False, True), repeat=5):
            flags = {f for f, active in zip((*TEACHERS, WIN, WEST), values) if active}
            won = WIN in flags
            stale = {xy: tiles[not won] for xy, tiles in LANE.items()}
            blocks, results = refresh(room, flags, stale)
            require(all(blocks[y*width+x] == LANE[x,y][won] for x,y in LANE)
                    and (WEST in results) == won and (OPEN in results) == (won or set(TEACHERS[:2]) <= flags),
                    "stale victory/teacher flags or persisted blocks")
        old_stops = early[-1][1].keys() - EXITS
        blocks, _ = refresh(room, {*TEACHERS[:2], WIN})
        rink = Rink(pack, room, blocks)
        require(all(rink.paths(s).keys() & EXITS for s in old_stops),
                "pre-win resting square stranded after victory")
    return early


if __name__ == "__main__":
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument("pack", type=Path)
    p.add_argument("--baseline", action="store_true")
    p.add_argument("--parent", type=Path)
    args = p.parse_args()
    pack = load_pack(args.pack)
    current = audit(pack, args.baseline)
    if args.parent:
        parent = load_pack(args.parent)
        before, now = (p["data"]["maps"][GYM] for p in (parent, pack))
        require(all(before[k] == now[k] for k in ("blocks", "events", "attributes", "objects", "map_event_section_commands")),
                "gym map dimensions, static terrain, warps or actors changed")
        labels = before["scripts"].keys() | now["scripts"].keys()
        require({k for k in labels if before["scripts"].get(k) != now["scripts"].get(k)} ==
                {GYM+"V2Gate1Open", GYM+"V2Gate1Closed", VICTORY},
                "unexpected change to a teacher, leader, rematch or gift script")
        banks = [p["data"]["tilesets"]["mn_gym_8"] for p in (parent, pack)]
        require(banks[0] == banks[1] and all(pack["runtime_files"].get(k) == v for k, v in parent["runtime_files"].items()
                                             if "/mn_gym_8" in k), "private gym art, animation or collision changed")
        require(current == audit(parent, True), "changed pre-badge geometry or slide choices")
        print("Identical first three courses in the parent and child, including all shortest routes, ends and trainer positions.")
