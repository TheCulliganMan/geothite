#!/usr/bin/env python3
"""Inventory installed hidden items and trace adjacent search squares from real map entrances."""

import argparse
from collections import Counter, defaultdict, deque
import json
from pathlib import Path
import sys

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE.parents[1] / "tools"))
from audit_story_flow import instructions, native_pages, read_branch, require
from render_reference import load_pack

DIRECTIONS = {"UP": (0, -1), "DOWN": (0, 1), "LEFT": (-1, 0), "RIGHT": (1, 0)}
OPPOSITE = {"UP": "DOWN", "DOWN": "UP", "LEFT": "RIGHT", "RIGHT": "LEFT"}
SIDES = {"north": "UP", "south": "DOWN", "east": "RIGHT", "west": "LEFT"}
WARP = {"LADDER", "CAVE", "DOOR", "WARP_CARPET_UP", "WARP_CARPET_DOWN"}
GROUND = {"FLOOR", "TALL_GRASS", "LONG_GRASS", "ICE", "BRIDGE_EW_DECK", "BRIDGE_NS_DECK"}
WATER = {"WATER", "BRIDGE_EW_WATER", "BRIDGE_NS_WATER"}
MOVES = {"MnR10Cave1F": ("Rock Smash", (40, 6)), "MnR18C00R00": ("Cut", (4, 15)),
         "MnLeechLake": ("Whirlpool", (60, 20))}


def coords(row):
    return row["x"], row["y"]


def adjacent(pos):
    return {(pos[0]+dx, pos[1]+dy) for dx, dy in DIRECTIONS.values()}


def ground(role):
    return role in GROUND | WARP or role.startswith("HOP_") or (
        role.endswith("_WALL") and not role.startswith("WATER_"))


def quadrant(data, name, x, y, changes=None):
    room = data["maps"][name]
    a = room["attributes"]
    if not (0 <= x < 2*a["width"] and 0 <= y < 2*a["height"]):
        return "OUTSIDE"
    offset = (y//2)*a["width"] + x//2
    block = (changes or {}).get(offset, room["blocks"][offset])
    return data["tilesets"][a["tileset_name"]]["collision"][f"{block:02x}"][y%2*2+x%2]


def anchors(data):
    """Incoming warp destinations and pairs of land squares on declared seams."""
    maps = data["maps"]
    constants = {r["attributes"]["map_constant"]: n for n, r in maps.items()}
    entrances = defaultdict(set)
    for n, r in maps.items():
        if not n.startswith("Mn"):
            continue
        for warp in r["events"]["warps"]:
            to = constants.get(warp["target_map_constant"])
            if to:
                for arrival in maps[to]["events"]["warps"]:
                    if arrival["index"] == warp["target_warp_id"]:
                        entrances[to].add(coords(arrival))
        a = r["attributes"]
        w, h = 2*a["width"], 2*a["height"]
        for conn in a["connections"]:
            to = conn["target_map"]
            dx, dy = DIRECTIONS[SIDES[conn["direction"]]]
            b = maps[to]["attributes"]
            for i in range(h if dx else w):
                x, y = ((w-1 if dx > 0 else 0), i) if dx else (i, (h-1 if dy > 0 else 0))
                tx = (0 if dx > 0 else 2*b["width"]-1) if dx else x-2*conn["offset"]
                ty = (0 if dy > 0 else 2*b["height"]-1) if dy else y-2*conn["offset"]
                if ground(quadrant(data, n, x, y)) and ground(quadrant(data, to, tx, ty)):
                    entrances[n].add((x, y))
    return entrances


def walk(edges, starts, stops=()):
    reached = {pos: 0 for pos in starts if pos in edges}
    queue = deque(reached)
    while queue:
        pos = queue.popleft()
        for dest in edges[pos]:
            if dest not in reached:
                reached[dest] = reached[pos]+1
                if dest not in stops:
                    queue.append(dest)
    return reached


def approaches(data, name, entrances, mode):
    r = data["maps"][name]
    a = r["attributes"]
    occupied = {coords(o) for o in r["objects"]}
    changes = {}
    if name in MOVES and mode == MOVES[name][0]:
        x, y = MOVES[name][1]
        if mode == "Rock Smash":
            [rock] = [o for o in r["objects"] if coords(o) == (x, y)]
            require(rock["spritemovedata"] == "SPRITEMOVEDATA_SMASHABLE_ROCK"
                    and instructions(r, rock["script"]) == [("jumpstd", "SmashRockScript")], "ore gate rock")
            occupied.remove((x, y))
        else:
            index = (y//2)*a["width"]+x//2
            block = r["blocks"][index]
            replacement = data["field_moves"][mode.lower()]["replacements"][a["tileset_name"]][block]
            expect = "CUT_TREE" if mode == "Cut" else "WHIRLPOOL"
            require(quadrant(data, name, x, y) == expect, f"{name} missing field-move target")
            changes[index] = replacement["replacement_block_id"]
    g = {(x, y): quadrant(data, name, x, y, changes)
         for y in range(2*a["height"]) for x in range(2*a["width"])}
    allowed = {p for p, role in g.items() if ground(role) or (mode in ("Surf", "Whirlpool") and role in WATER)}
    allowed -= occupied
    portals = {coords(o) for o in r["events"]["warps"] if g[coords(o)] in WARP} & allowed
    edges, reverse = {p: [] for p in allowed}, {p: [] for p in allowed}

    def step(pos, direction):
        role = g[pos]
        if role.endswith("_WALL") and direction in role.split("_"):
            return None
        dx, dy = DIRECTIONS[direction]
        length = 2 if role.startswith("HOP_") and direction in role.split("_") else 1
        dest = (pos[0]+dx*length, pos[1]+dy*length)
        if dest not in allowed:
            return None
        target = g[dest]
        if target.endswith("_WALL") and OPPOSITE[direction] in target.split("_"):
            return None
        if target.startswith("WARP_CARPET_") and not target.endswith(direction):
            return None
        return dest

    for pos in allowed:
        for direction in DIRECTIONS:
            dest = step(pos, direction)
            if dest is None:
                continue
            while g[dest] == "ICE":
                onward = step(dest, direction)
                if onward is None:
                    break
                dest = onward
            edges[pos].append(dest)
            reverse[dest].append(pos)
    outward = walk(edges, entrances, portals)
    homeward = walk(reverse, portals | (entrances & allowed), portals)
    return outward.keys() & homeward.keys() - portals


def audit(pack, expect_clue):
    data = pack["data"]
    maps, flags = data["maps"], data["story_event_script_constants"]["global"]
    entrances = anchors(data)
    counts, items, receipts = Counter(), {}, set()
    for name, room in maps.items():
        events = [e for e in room["events"]["bg_events"] if e["event_type"] == "BGEVENT_ITEM"]
        if not name.startswith("Mn") or not events:
            continue
        modes = ["walk", "Surf"] + ([MOVES[name][0]] if name in MOVES else [])
        access = {mode: approaches(data, name, entrances[name], mode) for mode in modes}
        if name in MOVES:
            tool, loc = MOVES[name]
            prior = "Surf" if tool == "Whirlpool" else "walk"
            require(adjacent(loc) & access[prior], f"{name} cannot operate {tool} from public footing")
        for e in events:
            site, label = coords(e), e["script"]
            require(all(coords(o) != site for o in room["objects"]), f"{name}/{site} covered by actor")
            require([f for f in room["events"]["bg_events"] if coords(f) == site] == [e], f"{name}/{site} overlap")
            [row] = [p for p in room["script_field_pickups"] if p["command"] == "hiddenitem" and p["source_script"] == label]
            token, flag = row["item_id"], row["event_flag"]
            require(instructions(room, label) == [("hiddenitem", token, flag)] and row["quantity"] == 1
                    and token in data["items"] and flag in flags and flags[flag] not in receipts
                    and data["map_scripts"][label] == room["scripts"][label], f"{name}/{label} reward, flag or wire")
            receipts.add(flags[flag])
            method = next((m for m in modes if adjacent(site) & access[m]), None)
            require(method is not None, f"{name}/{site} cannot face {token} and return to an entrance")
            stands = sorted(adjacent(site) & access[method])
            counts[method] += 1
            items[name, label] = (site, token, flag)
            print(f"{name} {site} {token} — {method}, search from {stands}")
    room = maps["MnDuluth"]
    flag = "EVENT_MN_HIDDEN_DULUTH"
    require(items["MnDuluth", "MN_HIDDEN_DULUTH"] == ((68, 81), "NUGGET", flag), "original Duluth reward")
    require({(x, y) for y in range(78, 84) for x in range(68, 74)
             if quadrant(data, "MnDuluth", x, y) == "FLOOR"}
            == {(x, y) for x in range(69, 73) for y in range(79, 83)}, "east float footprint")
    if expect_clue:
        pages = json.loads((HERE / "duluth-float-clue.json").read_text())
        require(native_pages(room["scripts"]["MnDuluthFloatSnagText"]) == pages, "Duluth directions")
        for state, text in [(set(), "MnDuluthFloatSnagText"), ({flag}, "MnDuluthFloatingFisher2Text")]:
            _, texts, result = read_branch(room, "MnDuluthFloatingFisher2", state)
            require(texts == [text] and result == state, "fisher changed receipt or repeated old clue")
    else:
        require(instructions(room, "MnDuluthFloatingFisher2") == [
            ("jumptextfaceplayer", "MnDuluthFloatingFisher2Text")], "expected original angler")
    reference = maps["Route32"]
    expected = {(12, 67): "GREAT_BALL", (11, 40): "SUPER_POTION"}
    require({coords(e): instructions(reference, e["script"])[0][1] for e in reference["events"]["bg_events"]
             if e["event_type"] == "BGEVENT_ITEM"} == expected, "original Crystal Route 32 comparison")
    print(f"{len(items)} distinct hidden rewards across {len({n for n, _ in items})} maps: {dict(counts)}.")
    print("Static map-entry reach and return with authored actors in place. Public connections, incoming warp "
          "arrivals, ice sliding, normal landings and the three specific field gates were checked. "
          "This does not establish badge acquisition, boat ownership, trainer movement or full-bag behavior.")
    return items


if __name__ == "__main__":
    cli = argparse.ArgumentParser(description=__doc__)
    cli.add_argument("pack", type=Path)
    cli.add_argument("--parent", type=Path, help="also require unchanged reward coordinates and receipts")
    cli.add_argument("--baseline", action="store_true", help="require the original Duluth conversation")
    args = cli.parse_args()
    old = audit(load_pack(args.parent), False) if args.parent else None
    result = audit(load_pack(args.pack), not args.baseline)
    if old is not None:
        require(result == old, "a hidden item's coordinate, reward, flag or identity changed")
