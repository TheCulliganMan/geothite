#!/usr/bin/env python3
"""Audit the six southern replacements on the compiled pack without playing."""

import argparse
from collections import deque
from itertools import combinations
import json
from pathlib import Path
import sys

sys.path.insert(0, str(Path(__file__).resolve().parents[2] / "tools"))
from render_reference import load_pack  # noqa: E402

STEPS = ((1, 0), (-1, 0), (0, 1), (0, -1))
WALK = {"FLOOR", "UP_WALL", "HOP_DOWN"}
WARP = {"LADDER", "WARP_CARPET_DOWN", "CAVE"}


def position(event):
    return event["x"], event["y"]


def adjacent(at):
    return {(at[0] + dx, at[1] + dy) for dx, dy in STEPS}


def graph(terrain, bodies, surf=False, jumps=True):
    permitted = WALK | ({"WATER"} if surf else set())
    edges = {}
    for at, source in terrain.items():
        if source not in permitted or at in bodies:
            continue
        x, y = at
        valid = set()
        for dx, dy in STEPS:
            target = (x + dx, y + dy)
            if jumps and source == "HOP_DOWN" and dy == 1:
                target = (x, y + 2)
            destination = terrain.get(target)
            if destination not in permitted or target in bodies:
                continue
            if dy == 1 and destination == "UP_WALL":
                continue
            if dy == -1 and (source == "UP_WALL" or destination == "HOP_DOWN"):
                continue
            valid.add(target)
        edges[at] = valid
    return edges


def distances(edges, start):
    assert start in edges, ("unusable standing tile", start)
    reached = {start: 0}
    pending = deque([start])
    while pending:
        at = pending.popleft()
        for target in edges[at]:
            if target not in reached:
                reached[target] = reached[at] + 1
                pending.append(target)
    return reached


def reverse(edges):
    back = {at: set() for at in edges}
    for source, links in edges.items():
        for target in links:
            back[target].add(source)
    return back


def terrain_for(pack, room):
    a = room["attributes"]
    tiles = pack["data"]["tilesets"][a["tileset_name"]]["collision"]
    w, h = a["width"], a["height"]
    blocks = room["blocks"]
    return {
        (x, y): tiles[f"{blocks[y // 2 * w + x // 2]:02x}"][2 * (y % 2) + x % 2]
        for x in range(2 * w)
        for y in range(2 * h)
    }


def audit_castorock(terrain, actors, backgrounds, arrivals, occupied, wet):
    rock = actors["MN_R03_JAW_BOULDER"]
    x, y = rock
    late = {
        backgrounds["MnJawPillar"], backgrounds["MnJawBed"],
        actors["MN_R03_JAW_CASTOROCK"],
    }
    assert not any(adjacent(target) & wet.keys() for target in late), "Surf bypasses Strength"
    assert all(terrain[x + dx, y] == "FLOOR" for dx in range(-3, 4)), "blocked push lane"
    other = occupied - {rock}
    push = graph(terrain, other | {(x + 2, y)})
    opened = distances(push, (x + 1, y))
    assert all(adjacent(target) & opened.keys() for target in late)
    assert arrivals <= opened.keys(), "opened chamber has no return"
    # From the east, restoring the original boulder permits a push back west.
    # The ledge still returns to the ladder with the boulder displaced west.
    return_west = graph(terrain, other | {(x - 2, y)})
    assert arrivals <= distances(return_west, (x - 1, y)).keys()


def audit_compiled(pack, layouts):
    maps = pack["data"]["maps"]
    constants = {m["attributes"]["map_constant"]: m for m in maps.values()}
    for layout in layouts:
        name = layout["map"]
        room = maps[name]
        a = room["attributes"]
        assert [a["width"], a["height"]] == layout["dimensions_metatiles"], name
        assert room["blocks"] == [int(x, 16) for row in layout["rows"] for x in row.split()], name
        terrain = terrain_for(pack, room)
        arrivals = {tuple(at) for at in layout["arrivals"]}
        warps = {position(e) for e in room["events"]["warps"] if terrain[position(e)] in WARP}
        assert warps == {at for at, permission in terrain.items() if permission in WARP}, name
        for event in room["events"]["warps"]:
            target_map = maps.get(event["target_map"]) or constants[event["target_map"]]
            paired = [w for w in target_map["events"]["warps"] if w["index"] == event["target_warp_id"]]
            assert len(paired) == 1, (name, "missing linked warp", event)
            assert position(paired[0]) in terrain_for(pack, target_map), (name, "off-map linked warp")
        for field, block, identifier in (
            ("warps", room["events"]["warps"], "index"),
            ("actors", room["objects"], "script"),
            ("backgrounds", room["events"]["bg_events"], "script"),
        ):
            current = {record[identifier]: position(record) for record in block}
            assert all(current[move[identifier]] == tuple(move["to"]) for move in layout.get(field, [])), (name, field)
        for change in layout.get("script_patches", []):
            label, index = change["script"], change["index"]
            assert room["scripts"][label][index]["args"] == change["after_args"], (name, label)
            assert room["script_text_bodies"][label]["commands"][index]["args"] == change["after_args"], (name, label, "uncompiled native text")

        actors = {o["script"]: position(o) for o in room["objects"]}
        backgrounds = {e["script"]: position(e) for e in room["events"]["bg_events"]}
        assert all(terrain[at] == "FLOOR" for at in actors.values()), name
        drops = {tuple(row["from"]): tuple(row["to"]) for row in layout.get("drops", [])}
        assert drops.keys() == {p for p, t in terrain.items() if t == "HOP_DOWN"}, name
        pile = {position(o) for o in room["objects"] if o["object_type"] == "OBJECTTYPE_ITEMBALL"}
        fixed = set(actors.values()) - pile
        river = name == "MnR03Cave1F"
        for take in range(len(pile) + 1):
            for picked in combinations(sorted(pile), take):
                occupied = fixed | (pile - set(picked))
                foot_edges = graph(terrain, occupied)
                wet_edges = graph(terrain, occupied, surf=True)
                for arrival in arrivals:
                    foot = distances(foot_edges, arrival)
                    wet = distances(wet_edges, arrival)
                    assert arrivals <= foot.keys(), (name, arrival, "disconnected arrival")
                    assert foot.keys() <= distances(reverse(foot_edges), arrival).keys(), (name, arrival, "walking trap")
                    assert wet.keys() <= distances(reverse(wet_edges), arrival).keys(), (name, arrival, "Surf trap")
                    assert all(adjacent(warp) & foot.keys() for warp in warps), (name, arrival, "inaccessible exit")
                    for script, at in actors.items():
                        if not (river and script == "MN_R03_JAW_CASTOROCK"):
                            assert adjacent(at) & foot.keys(), (name, arrival, script)
                    for script, at in backgrounds.items():
                        if river and script in {"MnJawPillar", "MnJawBed"}:
                            continue
                        if river and script == "MN_HIDDEN_R03CAVE1F":
                            assert adjacent(at) & wet.keys() and not adjacent(at) & foot.keys(), (name, "island cache")
                        else:
                            assert adjacent(at) & foot.keys(), (name, arrival, script)
                    if river:
                        audit_castorock(terrain, actors, backgrounds, arrivals, occupied, wet)
                ordinary = graph(terrain, occupied, jumps=False)
                for lip, landing in drops.items():
                    assert landing == (lip[0], lip[1] + 2) and terrain[lip[0], lip[1] + 1] == "WALL", (name, lip)
                    assert distances(foot_edges, lip)[landing] == 1, (name, lip)
                    assert distances(ordinary, lip)[landing] >= 8, (name, lip, "trivial bypass")
                    assert distances(foot_edges, landing)[lip] >= 8, (name, lip, "trivial reverse")
        if name == "MnR01Cave1F":
            rookie = next(o for o in room["objects"] if o["script"] == "MN_CAVE_DRIFTLESS_ROOKIE")
            assert rookie["sightline_direction_override"] == "LEFT" and rookie["radius"] == 3
            ray = {(rookie["x"] - i, rookie["y"]) for i in range(1, 4)}
            assert all(terrain[at] == "FLOOR" for at in ray)
            blocked_ray = graph(terrain, set(actors.values()) | ray)
            assert all(arrivals <= distances(blocked_ray, at).keys() for at in arrivals), "new forced battle"
        print(f"{name}: {a['width']}×{a['height']} metatiles, {len(arrivals)} arrivals, {len(drops)} one-way drops")


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("candidate", type=Path)
    args = parser.parse_args()
    audit_compiled(load_pack(args.candidate), json.loads(Path(__file__).with_name("south.json").read_text()))
