#!/usr/bin/env python3
"""Check the rewritten Iron Range against its retained stage and scene contracts."""

import argparse
from collections import deque
from itertools import product
import json
from pathlib import Path
import sys

sys.path.insert(0, str(Path(__file__).resolve().parents[2] / "tools"))
from render_reference import load_pack  # noqa: E402

DIRECTIONS = {"UP": (0, -1), "RIGHT": (1, 0), "DOWN": (0, 1), "LEFT": (-1, 0)}
LAND = {"FLOOR", "UP_WALL", "HOP_DOWN", "HOP_LEFT"}


def position(record):
    return record["x"], record["y"]


def neighbor(at, delta, length=1):
    return at[0] + delta[0] * length, at[1] + delta[1] * length


def surface(room, collision, mutations):
    attrs = room["attributes"]
    width, height = attrs["width"], attrs["height"]
    blocks = room["blocks"].copy()
    for (x, y), tile in mutations.items():
        blocks[y // 2 * width + x // 2] = tile
    return {
        (x, y): collision[f"{blocks[y // 2 * width + x // 2]:02x}"][y % 2 * 2 + x % 2]
        for y in range(height * 2) for x in range(width * 2)
    }


def walking(terrain, bodies, surf):
    permitted = LAND | ({"WATER"} if surf else set())
    graph = {}
    for at, cell in terrain.items():
        if cell not in permitted or at in bodies:
            continue
        graph[at] = set()
        for direction, delta in DIRECTIONS.items():
            target = neighbor(at, delta, 2 if cell == f"HOP_{direction}" else 1)
            destination = terrain.get(target)
            if target in bodies or destination not in permitted:
                continue
            if cell == "UP_WALL" and direction == "UP":
                continue
            if destination == "UP_WALL" and direction == "DOWN":
                continue
            graph[at].add(target)
    return graph


def reached(graph, start):
    assert start in graph, (start, "arrival is obstructed")
    seen = {start}
    queue = deque([start])
    while queue:
        for point in graph[queue.popleft()] - seen:
            seen.add(point)
            queue.append(point)
    return seen


def approaches(at, tiles):
    return any(neighbor(at, direction) in tiles for direction in DIRECTIONS.values())


def frame_states(before, after):
    """Pair each scripted block group with its reviewed new position and all states."""
    old, new = before["script_block_changes"], after["script_block_changes"]
    groups = {}
    assert len(old) == len(new)
    for prior, current in zip(old, new):
        assert (prior["source_script"], prior["command_index"], prior["block_id"]) == (
            current["source_script"], current["command_index"], current["block_id"]
        )
        key = position(prior), position(current)
        groups.setdefault(key, {None}).add(prior["block_id"])
    for values in product(*[sorted(blocks, key=lambda v: -1 if v is None else v) for blocks in groups.values()]):
        yield tuple({
            positions[side]: value for positions, value in zip(groups, values) if value is not None
        } for side in (0, 1))


def core_motion(room, tiles):
    actors = {actor["script"]: position(actor) for actor in room["objects"]}
    witness = [event for event in room["events"]["coord_events"] if event["script_name"].startswith("MnNDrillEnter")]
    controls, = [event for event in room["events"]["bg_events"] if event["script"] == "MnNDrillStop"]
    # Translate the retained movement programs from the re-authored event starts.
    # This covers the control-panel turning branch as well as the straight path.
    assert room["scripts"]["MnMineWalkToCrew"][1]["args"] == ["17", "MnMineFromMotorFront"]

    def follow(at, program):
        assert tiles[at] == "FLOOR", (program, at)
        for command in room["scripts"][program]:
            if command["command"] != "step" or command["args"][0] not in DIRECTIONS:
                continue
            at = neighbor(at, DIRECTIONS[command["args"][0]])
            assert tiles[at] == "FLOOR", (program, at)
        return at

    at = position(controls)
    follow(neighbor(at, (0, 1)), "MnNDrillPlayerToCrew")
    assert follow(neighbor(at, (-1, 0)), "MnMineRoundMotor") == neighbor(at, (0, 1))
    follow(actors["MN_NDRILL_FINN"], "MnNDrillCrewOut")
    assert len(witness) == 18
    for event in witness:
        movement = room["scripts"][event["script_name"]][2]["args"][1]
        assert follow(position(event), movement) == (11, 11), event
    # The player cannot reach the drill panel or the scene's talk actors
    # without crossing a coordinate trigger where the first arrival sees it.
    scene_line = {position(event) for event in witness}
    actor_blocks = set(actors.values())
    start = position(next(warp for warp in room["events"]["warps"] if warp["index"] == 2))
    before_triggers = reached(walking(tiles, actor_blocks | scene_line, False), start)
    for target in (at, actors["MN_NDRILL_REED"], actors["MN_NDRILL_ENGINEER"]):
        assert not approaches(target, before_triggers), ("witness bypass", target)


def audit(parent, candidate, layouts):
    old_maps, maps = parent["data"]["maps"], candidate["data"]["maps"]
    constants = {record["attributes"]["map_constant"]: name for name, record in maps.items()}
    counts = {"rooms": 0, "shaft records": 0, "arrival and capability states": 0, "core movement variants": 0}
    for layout in layouts:
        name = layout["map"]
        old, room = old_maps[name], maps[name]
        width, height = layout["dimensions_metatiles"]
        assert [width, height] == [room["attributes"]["width"], room["attributes"]["height"]]
        assert [width, height] != [old["attributes"]["width"], old["attributes"]["height"]]
        assert room["blocks"] == [int(cell, 16) for row in layout["rows"] for cell in row.split()]
        collision = candidate["data"]["tilesets"][room["attributes"]["tileset_name"]]["collision"]
        old_warps, new_warps = old["events"]["warps"], room["events"]["warps"]
        for prior, warp in zip(old_warps, new_warps):
            assert (warp["index"], warp["target_map"], warp["target_warp_id"]) == (
                prior["index"], prior["target_map"], prior["target_warp_id"]
            )
            target = maps[constants.get(warp["target_map"], warp["target_map"])]
            landing, = [
                row for row in target["events"]["warps"] if row["index"] == warp["target_warp_id"]
            ]
            assert landing["target_map"] in (name, room["attributes"]["map_constant"]), (name, warp)
            counts["shaft records"] += 1
        old_actors, actors = old["objects"], room["objects"]
        assert [a["script"] for a in old_actors] == [a["script"] for a in actors]
        old_anchors = [*old["events"]["bg_events"], *old_actors]
        anchors = [*room["events"]["bg_events"], *actors]
        start_warps = [(position(prior), position(warp)) for prior, warp in zip(old_warps, new_warps)]
        if name == "MnR09Cave1F":
            start_warps.append(((13, 18), (5, 16)))
        for old_changes, new_changes in frame_states(old, room):
            before, after = surface(old, collision, old_changes), surface(room, collision, new_changes)
            assert all(
                after[x, y] == "WALL" for x, y in after
                if x in (0, width * 2 - 1) or y in (0, height * 2 - 1)
            ), (name, "outer boundary")
            old_active = {warp["index"] for warp in old_warps if before[position(warp)] == "LADDER"}
            new_active = {warp["index"] for warp in new_warps if after[position(warp)] == "LADDER"}
            assert old_active == new_active, (name, "ladder activation")
            assert {at for at, cell in after.items() if cell == "LADDER"} == {
                position(warp) for warp in new_warps if warp["index"] in new_active
            }, (name, "unpaired visible ladder")
            for actor in actors:
                at = position(actor)
                assert after[at] == "FLOOR", (name, actor["script"], at)
                facing = actor["sightline_direction_override"]
                if facing:
                    for n in range(1, actor["radius"] + 1):
                        assert after[neighbor(at, DIRECTIONS[facing], n)] == "FLOOR", (name, actor["script"])
            for surf in (False, True):
                old_graph = walking(before, {position(a) for a in old_actors}, surf)
                graph = walking(after, {position(a) for a in actors}, surf)
                back = {at: set() for at in graph}
                for at, moves in graph.items():
                    for target in moves:
                        back[target].add(at)
                for previous, arrival in start_warps:
                    if previous not in old_graph:
                        assert arrival not in graph
                        continue
                    old_reach, new_reach = reached(old_graph, previous), reached(graph, arrival)
                    assert new_reach <= reached(back, arrival), (name, arrival, "no return")
                    old_exits = {
                        w["index"] for w in old_warps if w["index"] in old_active
                        and approaches(position(w), old_reach)
                    }
                    new_exits = {
                        w["index"] for w in new_warps if w["index"] in new_active
                        and approaches(position(w), new_reach)
                    }
                    assert old_exits == new_exits, (name, arrival, old_exits, new_exits)
                    for prior, anchor in zip(old_anchors, anchors):
                        assert approaches(position(prior), old_reach) == approaches(position(anchor), new_reach), (
                            name, arrival, anchor["script"], surf
                        )
                    counts["arrival and capability states"] += 1
            if name == "MnIronDrillCore":
                core_motion(room, after)
                counts["core movement variants"] += 21
        counts["rooms"] += 1
    return counts


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("parent", type=Path)
    parser.add_argument("candidate", type=Path)
    args = parser.parse_args()
    layout = json.loads(Path(__file__).with_name("mines.json").read_text())
    print(json.dumps(audit(load_pack(args.parent), load_pack(args.candidate), layout), indent=2))
