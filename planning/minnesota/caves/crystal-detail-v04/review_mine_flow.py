#!/usr/bin/env python3
"""Verify the authored Iron Range gate chain from compiled events and collision."""
import argparse
from collections import deque
from itertools import product
import io
import json
from pathlib import Path
import sys

from PIL import Image

SOURCE = Path(__file__).resolve().parent
sys.path.insert(0, str(SOURCE.parents[1] / "tools"))
sys.path.insert(0, str(SOURCE.parent / "crystal-replacement-v02"))
from render_reference import load_pack  # noqa: E402
from review_mines import DIRECTIONS, approaches, core_motion, neighbor, position, surface, walking  # noqa: E402

MINE = "MnR09Cave1F"
HAUL = "MnIronHaulage"
VENT = "MnIronVentilation"
SUMP = "MnIronSump"
PUMP = "MnIronPumpGallery"
CORE = "MnIronDrillCore"
APPROACH = "MnIronDrillApproach"
ROOMS = (MINE, APPROACH, HAUL, SUMP, VENT, PUMP, CORE)
CALLBACK = {
    MINE: "MnNDrillPublicTiles",
    HAUL: "MnIronPressureHAULRestore",
    VENT: "MnIronPressureVENTRestore",
    SUMP: "MnIronSumpGate",
    PUMP: "MnNDrillGalleryTiles",
    CORE: "MnNDrillCoreTiles",
}
FOREMAN = "EVENT_MN_DEPOT_FOREMAN_DEFEATED"
HAUL_OFF = "EVENT_MN_IRON_HAUL_PRESSURE_OFF"
VENT_OFF = "EVENT_MN_IRON_VENT_PRESSURE_OFF"
FEED = "EVENT_MN_DRILL_FEED_ISOLATED"
STOPPED = "EVENT_MN_DRILL_STOPPED"
WEST_TRACK = [(11, y) for y in range(14, 4, -1)] + [(10, 5), (9, 5)]
EAST_TRACK = [(33, 5), (32, 5), (31, 5)] + [(31, y) for y in range(6, 11)] + [
    (x, 10) for x in range(32, 42)
]
SWITCHES = {
    HAUL: "MnIronPressureHAUL",
    VENT: "MnIronPressureVENT",
    SUMP: "MnIronSumpGateRead",
    PUMP: "MnNDrillGalleryFeed",
    CORE: "MnNDrillStop",
}


def mutations(room, active_flags):
    name = room["id"]
    if name not in CALLBACK:
        return {}
    route = []
    changes = {}
    program = room["scripts"][CALLBACK[name]]
    for record in program:
        command, args = record["command"], record["args"]
        if command == "checkevent":
            route.append(args[0])
        elif command == "iffalse":
            assert all(
                row["command"] in {"end", "endcallback"}
                for row in room["scripts"][args[0]]
            ), (name, args[0], "guard may run another scripted operation")
        elif command == "changeblock":
            assert route, (name, args, "unguarded block change")
            if all(flag in active_flags for flag in route):
                x, y, block = map(int, args)
                changes[(x, y)] = block
        elif command != "endcallback":
            raise AssertionError((name, command, "callback requires explicit audit"))
    return changes


def state_components(pack, flags, surf):
    maps = pack["data"]["maps"]
    names = {maps[name]["attributes"]["map_constant"]: name for name in ROOMS}
    areas = {}
    routes = {}
    for name in ROOMS:
        room = maps[name]
        tile = pack["data"]["tilesets"][room["attributes"]["tileset_name"]]["collision"]
        terrain = surface(room, tile, mutations(room, flags))
        bodies = {position(a) for a in room["objects"]}
        if name == CORE:
            bodies.discard((17, 11))
            if STOPPED in flags:
                bodies.add((10, 9))
        graph = walking(terrain, bodies, surf)
        warps = [warp for warp in room["events"]["warps"] if terrain[position(warp)] == "LADDER"]
        for edge in warps:
            target = names.get(edge["target_map"], edge["target_map"])
            if target not in ROOMS:
                continue
            dst, = [warp for warp in maps[target]["events"]["warps"] if warp["index"] == edge["target_warp_id"]]
            at = position(edge)
            for heading, delta in DIRECTIONS.items():
                before = neighbor(at, (-delta[0], -delta[1]))
                if before not in graph or (terrain[before] == "UP_WALL" and heading == "UP"):
                    continue
                routes.setdefault((name, before), set()).add((target, position(dst)))
        areas[name] = graph, terrain
    return areas, routes


def visit(areas, routes, initial):
    distances = {initial: 0}
    pending = deque(distances)
    while pending:
        room, point = pending.popleft()
        next_tiles = {(room, neighbor) for neighbor in areas[room][0][point]}
        next_tiles.update(routes.get((room, point), ()))
        for step in next_tiles - distances.keys():
            target, landing = step
            assert landing in areas[target][0], (room, point, target, landing, "unusable mine landing")
            distances[step] = distances[(room, point)] + 1
            pending.append(step)
    return distances


def controls(maps, reach):
    state = {}
    for name, script in SWITCHES.items():
        event, = [row for row in maps[name]["events"]["bg_events"] if row["script"] == script]
        state[name] = approaches(position(event), {point for room, point in reach if room == name})
    return state


def haulage_sidings(pack):
    room = pack["data"]["maps"][HAUL]
    tileset = room["attributes"]["tileset_name"]
    data = pack["data"]["tilesets"][tileset]
    name = f"gfx/tilesets/{tileset}"
    glyphs = bytes(pack["runtime_files"][f"{name}.2bpp"])
    art = bytes(pack["runtime_files"][f"data/tilesets/{tileset}_metatiles.bin"])
    sheet = Image.open(io.BytesIO(bytes(pack["runtime_files"][f"{name}.png"]))).convert("L")
    width = room["attributes"]["width"]
    terrain = surface(room, data["collision"], {})

    def shade(at, x, y):
        native_x, native_y = at
        cell = room["blocks"][native_y // 2 * width + native_x // 2]
        glyph_x, glyph_y = native_x % 2 * 2 + x // 8, native_y % 2 * 2 + y // 8
        token = art[cell * 16 + glyph_y * 4 + glyph_x]
        slot = (token & 127) + 128 if data["palette_map"][token] & 8 else token
        row, mask = slot * 16 + y % 8 * 2, 1 << (7 - x % 8)
        color = int(bool(glyphs[row] & mask)) + 2 * int(bool(glyphs[row + 1] & mask))
        assert sheet.getpixel((slot % 16 * 8 + x % 8, slot // 16 * 8 + y % 8)) == (255, 170, 85, 0)[color]
        return color

    boundary_count = 0
    for route in (WEST_TRACK, EAST_TRACK):
        edges = {at: set() for at in route}
        assert all(terrain[at] == "FLOOR" for at in route if at != EAST_TRACK[-1])
        for a, b in zip(route, route[1:]):
            edges[a].add(b)
            edges[b].add(a)
            dx, dy = b[0] - a[0], b[1] - a[1]
            assert abs(dx) + abs(dy) == 1
            for rail in (4, 11):
                if dx:
                    left, right = (a, b) if dx > 0 else (b, a)
                    pair = shade(left, 15, rail), shade(right, 0, rail)
                else:
                    upper, lower = (a, b) if dy > 0 else (b, a)
                    pair = shade(upper, rail, 15), shade(lower, rail, 0)
                assert all(color in (0, 2) for color in pair), (a, b, rail, "discontinuous rail at metatile join")
                boundary_count += 1
        boundary = {
            "UP": [(4, 0), (11, 0)],
            "RIGHT": [(15, 4), (15, 11)],
            "DOWN": [(4, 15), (11, 15)],
            "LEFT": [(0, 4), (0, 11)],
        }
        for point, connected in edges.items():
            for direction, delta in DIRECTIONS.items():
                if neighbor(point, delta) in connected:
                    continue
                colors = [shade(point, x, y) for x, y in boundary[direction]]
                assert not all(color in (0, 2) for color in colors), (point, direction, "rail implies an unconnected route")
    assert terrain[EAST_TRACK[-1]] == "WALL", "parked car must stop through traffic"
    return boundary_count


def local_distances(pack, room, origin, target, flags, exclude=()):
    areas, _ = state_components(pack, flags, surf=False)
    graph, terrain = areas[room]
    omitted = set(exclude)
    graph = {point: edges - omitted for point, edges in graph.items() if point not in omitted}
    distances = visit({room: (graph, terrain)}, {}, (room, origin))
    return distances.get((room, target))


def shortcut_lengths(pack):
    west, east = (12, 15), (32, 5)
    assert local_distances(pack, HAUL, west, east, set()) is None, "closed Haulage crosscut bypass"
    assert local_distances(pack, HAUL, west, east, {HAUL_OFF}) is not None, "opened Haulage crosscut cannot be crossed"
    # The bottom Pump Gallery gate gives a real return after using the feed
    # panel. Compare from the original Sump arrival to its south-side bank.
    start, bank = (8, 5), (27, 16)
    inspection_turn = ((14, 11), (15, 11))
    manual = local_distances(pack, PUMP, start, bank, set())
    powered_down = local_distances(pack, PUMP, start, bank, {FEED})
    assert powered_down is not None and manual is not None and powered_down + 4 <= manual, (
        "feed must open a shorter Pump Gallery return", manual, powered_down
    )
    assert local_distances(pack, PUMP, start, bank, set(), inspection_turn) is None, "closed lower shutter bypass"
    assert local_distances(pack, PUMP, start, bank, {FEED}, inspection_turn) == powered_down
    result = {"Pump Gallery return before and after feed": [manual, powered_down]}
    for start in ((10, 8), (11, 8)):
        area, _ = state_components(pack, set(), surf=False)
        graph, terrain = area[MINE]
        landing = neighbor(start, DIRECTIONS["DOWN"], 2)
        lip = neighbor(start, DIRECTIONS["DOWN"])
        assert terrain[start] == "HOP_DOWN" and terrain[lip] == "WALL" and landing in graph[start]
        return_steps = local_distances(pack, MINE, landing, start, set())
        assert start not in graph[landing] and return_steps is not None and return_steps >= 8
        result[f"Route 9 adit ordinary uphill return at {start}"] = return_steps
    return result


def audit(parent, pack):
    states = 0
    opportunities = {}
    maps = pack["data"]["maps"]
    for caps in product((False, True), repeat=5):
        foreman, haul, vent, feed, surf = caps
        flags = {flag for enabled, flag in zip(caps[:4], (FOREMAN, HAUL_OFF, VENT_OFF, FEED)) if enabled}
        old_areas, old_routes = state_components(parent, flags, surf)
        areas, routes = state_components(pack, flags, surf)
        spawn = (MINE, (5, 16))
        before, after = visit(old_areas, old_routes, spawn), visit(areas, routes, spawn)
        assert {name for name, _ in before} == {name for name, _ in after}, (caps, "room access changed")
        assert controls(parent["data"]["maps"], before) == controls(maps, after), (caps, "control access changed")
        accessed = {name for name, _ in after}
        assert (APPROACH in accessed) == foreman
        assert (PUMP in accessed) == (foreman and haul and vent)
        assert (CORE in accessed) == (foreman and haul and vent and feed)
        for name in accessed:
            arrivals = [warp for warp in maps[name]["events"]["warps"] if (name, position(warp)) in after]
            for arrival in arrivals:
                assert spawn in visit(areas, routes, (name, position(arrival))), (caps, name, arrival, "no regional return")
        if foreman and haul and vent and feed and not surf:
            opportunities["survey entry to drill core landing"] = [
                before[(CORE, (12, 25))], after[(CORE, (12, 25))]
            ]
        states += 1
    for animated in (None, 0x12, 0x16):
        core = maps[CORE]
        collision = pack["data"]["tilesets"][core["attributes"]["tileset_name"]]["collision"]
        state = {} if animated is None else {(12, 10): animated}
        core_motion(core, surface(core, collision, state))
    return {
        "story and movement states": states,
        "core scene terrain states": 3,
        "decoded Haulage rail joins": haulage_sidings(pack),
        "walking steps before and after": opportunities | shortcut_lengths(pack),
    }


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("parent", type=Path)
    parser.add_argument("candidate", type=Path)
    args = parser.parse_args()
    print(json.dumps(audit(load_pack(args.parent), load_pack(args.candidate)), indent=2))
