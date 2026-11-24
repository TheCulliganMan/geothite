#!/usr/bin/env python3
"""Check the four authored northern cave replacements against compiled collision."""

from collections import deque
from copy import deepcopy
from itertools import combinations
import json
from pathlib import Path
import sys

sys.path.insert(0, str(Path(__file__).resolve().parents[2] / "tools"))
from render_reference import load_pack

STEPS = ((0, -1), (1, 0), (0, 1), (-1, 0))
FACING = dict(zip(("UP", "RIGHT", "DOWN", "LEFT"), STEPS))
HOPS = {"HOP_DOWN": FACING["DOWN"], "HOP_LEFT": FACING["LEFT"]}
FLOOR = {"FLOOR", "UP_WALL", *HOPS}
CASTS = {
    "MnR20Cave1F": ((12, 12), (13, 12)),
    "MnR12Cave1F": ((14, 10), (15, 10)),
    "MnR10Cave1F": ((14, 40), (15, 40)),
    "MnInnerBasin": ((10, 21), (11, 21)),
}
WET_CACHE = {"MN_R12_GROTTO_REVIVE", "MN_CAVE_R20_SEEP_CACHE"}
FRACTURE = "MN_R10_SURVEY_FRACTURE"
SEAM = "MN_R10_STAR_SEAM"
BASIN = "MnInnerBasin"


def adjacent(tile):
    x, y = tile
    return {(x + dx, y + dy) for dx, dy in STEPS}


def terrain(module, palette, blocks=None):
    attrs = module["attributes"]
    width, height = attrs["width"], attrs["height"]
    blocks = module["blocks"] if blocks is None else blocks
    return {
        (x, y): palette[f"{blocks[y // 2 * width + x // 2]:02x}"][2 * (y % 2) + (x % 2)]
        for x in range(2 * width) for y in range(2 * height)
    }


def edges(tiles, bodies, drops, surf=False, jump=True):
    walk = FLOOR | ({"WATER"} if surf else set())
    graph = {p: set() for p, k in tiles.items() if k in walk and p not in bodies}
    for (x, y), targets in graph.items():
        for dx, dy in STEPS:
            at = (x + dx, y + dy)
            if jump and HOPS.get(tiles[x, y]) == (dx, dy):
                at = drops[x, y]
            if at not in graph:
                continue
            if dy == 1 and tiles[at] == "UP_WALL" or dy == -1 and tiles[x, y] == "UP_WALL":
                continue
            targets.add(at)
    return graph


def distance(graph, start, reverse=False):
    if reverse:
        backward = {p: set() for p in graph}
        for p, targets in graph.items():
            for target in targets:
                backward[target].add(p)
        graph = backward
    assert start in graph, start
    todo = deque([start])
    found = {start: 0}
    while todo:
        p = todo.popleft()
        for target in graph[p]:
            if target not in found:
                found[target] = found[p] + 1
                todo.append(target)
    return found


def check_outside(pack, room, base, warp_):
    maps = pack["data"]["maps"]
    target = warp_["target_map"]
    if target == room["attributes"]["map_constant"]:
        return
    outside = maps.get(target) or next(m for m in maps.values() if m["attributes"]["map_constant"] == target)
    collision = pack["data"]["tilesets"][outside["attributes"]["tileset_name"]]["collision"]
    surface = terrain(outside, collision)
    landing, = [w for w in outside["events"]["warps"] if w["index"] == warp_["target_warp_id"]]
    position = (landing["x"], landing["y"])
    assert surface[position] in {"FLOOR", "CAVE"}, position
    outside_mouths = adjacent(position) | {position}
    assert any(
        (w["x"], w["y"]) in outside_mouths
        and surface[w["x"], w["y"]] == "CAVE"
        and w["target_map"] in {room["id"], room["attributes"]["map_constant"]}
        and w["target_warp_id"] == (2 if room["id"] == BASIN else warp_["index"])
        for w in outside["events"]["warps"]
    ), (room["id"], "outside warp to the new landing")
    assert (warp_["target_map"], warp_["target_warp_id"]) == (base["target_map"], base["target_warp_id"])


def check_room(pack, parent, layout):
    name = layout["map"]
    module = pack["data"]["maps"][name]
    base = parent["data"]["maps"][name]
    blocks = [int(b, 16) for row in layout["rows"] for b in row.split()]
    assert module["blocks"] == blocks, (name, "replacement missing")
    assert module["attributes"]["tileset_name"] == "cave"
    width, height = layout["dimensions_metatiles"]
    assert [module["attributes"]["width"], module["attributes"]["height"]] == [width, height]
    old_events, new_events = base["events"], module["events"]
    for field, key, old, new in (
        ("warps", "index", old_events["warps"], new_events["warps"]),
        ("actors", "script", base["objects"], module["objects"]),
        ("backgrounds", "script", old_events["bg_events"], new_events["bg_events"]),
    ):
        expected = deepcopy(old)
        for move in layout[field]:
            row, = [r for r in expected if r[key] == move[key] and [r["x"], r["y"]] == move["from"]]
            row["x"], row["y"] = move["to"]
        assert new == expected, (name, "only declared anchor moves", field)
    scripts = deepcopy(base["scripts"])
    for patch in layout["script_patches"]:
        command = scripts[patch["script"]][patch["index"]]
        assert command["args"] == patch["before_args"]
        command["args"] = patch["after_args"]
    assert module["scripts"] == scripts, (name, "only declared script operands")
    palette = pack["data"]["tilesets"]["cave"]["collision"]
    closed = terrain(module, palette)
    if name == BASIN:
        drained = blocks.copy()
        assert len(module["script_block_changes"]) == 9
        changes = {(r["x"], r["y"], r["block_id"]) for r in module["script_block_changes"]}
        assert changes == {
            tuple(map(int, s["after_args"])) for s in layout["script_patches"]
            if s["script"] == "MnBasinRestoreTerrain"
        }
        for x, y, b in changes:
            assert closed[x, y] in {"WALL", "WATER"}
            drained[y // 2 * width + x // 2] = b
        opened = terrain(module, palette, drained)
    else:
        opened = closed
    for surface in (closed, opened):
        assert all(
            surface[x, y] == "WALL"
            for x, y in surface
            if x in (0, 2 * width - 1) or y in (0, 2 * height - 1)
        ), (name, "sealed cave perimeter")
    actors = {(a["x"], a["y"]): a for a in module["objects"]}
    items = sorted(at for at, a in actors.items() if a["object_type"] == "OBJECTTYPE_ITEMBALL")
    posts = {(e["x"], e["y"]): e["script"] for e in module["events"]["bg_events"]}
    warps = module["events"]["warps"]
    assert {w["index"] for w in warps} == {w["index"] for w in base["events"]["warps"]}
    entrances = set()
    for w in warps:
        at = (w["x"], w["y"])
        origin, = [o for o in base["events"]["warps"] if o["index"] == w["index"]]
        check_outside(pack, module, origin, w)
        if name == BASIN and w["index"] == 2:
            assert closed[at] == "FLOOR" and list(at) == layout["arrivals"][0]
        else:
            entrances.add(at)
    assert {p for p, k in closed.items() if k == "LADDER"} == entrances
    drops = {tuple(d["from"]): tuple(d["to"]) for d in layout.get("drops", [])}
    assert {p for p, k in closed.items() if k in HOPS} == set(drops)
    for (x, y), landing in drops.items():
        dx, dy = HOPS[closed[x, y]]
        assert landing == (x + 2 * dx, y + 2 * dy)
        assert closed[x + dx, y + dy] == "WALL" and closed[landing] == "FLOOR"
    starts = [tuple(at) for at in layout["arrivals"]]
    assert all(closed[p] == "FLOOR" and p not in actors for p in starts)
    assert all(opened[p] == "FLOOR" for p in actors)
    areas = [False, True] if name in {BASIN, "MnR10Cave1F"} else [False]
    sums = []
    for revealed in areas:
        surface = opened if revealed else closed
        for count in range(len(items) + 1):
            for claimed in combinations(items, count):
                bodies = {
                    p for p, actor in actors.items() if p not in claimed
                    and not (revealed and actor["script"] == FRACTURE)
                }
                foot = edges(surface, bodies, drops)
                swim = edges(surface, bodies, drops, surf=True)
                for start in starts:
                    walk = distance(foot, start)
                    wet = distance(swim, start)
                    assert set(starts) <= walk.keys()
                    assert walk.keys() <= distance(foot, start, reverse=True).keys()
                    assert wet.keys() <= distance(swim, start, reverse=True).keys()
                    assert all(adjacent(at) & walk.keys() for at in entrances)
                    bank, water = CASTS[name]
                    assert bank in walk and surface[bank] == "FLOOR"
                    assert water in wet and water in adjacent(bank) and surface[water] == "WATER"
                    for at, actor in actors.items():
                        visible = name != BASIN or revealed
                        assert bool(adjacent(at) & walk.keys()) == visible, (name, actor["script"])
                        if not visible:
                            assert not adjacent(at) & wet.keys()
                        if actor["sightline_direction_override"]:
                            dx, dy = FACING[actor["sightline_direction_override"]]
                            ray = {(at[0] + dx * step, at[1] + dy * step) for step in range(1, actor["radius"] + 1)}
                            assert ray <= walk.keys(), (name, actor["script"], ray - walk.keys())
                    for at, script in posts.items():
                        assert bool(adjacent(at) & wet.keys()) == (script != SEAM or revealed), script
                        assert bool(adjacent(at) & walk.keys()) == (script not in WET_CACHE and (script != SEAM or revealed)), script
                if name == "MnR10Cave1F" and not revealed:
                    # Once a player enters the seam, restoring the rock must not trap them.
                    assert foot.keys() == distance(foot, starts[0], reverse=True).keys()
                if name == "MnR20Cave1F":
                    west, east = starts
                    shortcut = distance(foot, east)[west]
                    long_way = distance(edges(surface, bodies, drops, jump=False), east)[west]
                    assert shortcut + 16 <= long_way
                    if not claimed:
                        sums.append((distance(foot, west)[east], shortcut))
                if name == BASIN and not claimed:
                    sums.append(len(distance(foot, starts[0])))
    return dict(map=name, metatiles=[width, height], distances=sums)


def verify(parent, candidate):
    spec = json.loads(Path(__file__).with_name("north.json").read_text())
    return [check_room(candidate, parent, layout) for layout in spec]


if __name__ == "__main__":
    print(json.dumps(verify(load_pack(Path(sys.argv[1])), load_pack(Path(sys.argv[2]))), indent=2))
