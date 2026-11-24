#!/usr/bin/env python3
"""Check compiled Voyageurs slide routes, reward approaches, and safe returns."""

import argparse
from collections import defaultdict, deque
from itertools import combinations
import json
from pathlib import Path
import sys

sys.path.insert(0, str(Path(__file__).resolve().parents[2] / "tools"))
from render_reference import load_pack  # noqa: E402
from review_voyageurs_ice import DIRECTIONS, course  # noqa: E402

ICE = {"ICE", "ICE_2B"}


def reachable(starts, edges):
    seen = set(starts)
    pending = deque(seen)
    while pending:
        for end in edges[pending.popleft()]:
            if end not in seen:
                seen.add(end)
                pending.append(end)
    return seen


def inspect(pack, room, layout):
    name = room["id"]
    geometry = {**layout, "id": name}
    arrivals = set(map(tuple, layout["arrivals"]))
    collision = pack["data"]["tilesets"]["ice_path"]["collision"]
    blocks = [int(cell, 16) for row in layout["rows"] for cell in row.split()]
    assert blocks == room["blocks"], (name, "replacement rows not installed")
    warps = {(w["x"], w["y"]) for w in room["events"]["warps"]}
    landmarks = {(e["x"], e["y"]) for e in room["events"]["bg_events"]}
    pickups = {(a["x"], a["y"]) for a in room["objects"] if a["object_type"] == "OBJECTTYPE_ITEMBALL"}
    fixed = {(a["x"], a["y"]) for a in room["objects"]} - pickups
    width, height = [2 * side for side in layout["dimensions_metatiles"]]

    def adjacent(x, y):
        return {(x + dx, y + dy) for _, dx, dy in DIRECTIONS}

    for count in range(len(pickups) + 1):
        for collected in combinations(sorted(pickups), count):
            bodies = fixed | (pickups - set(collected))
            kind, advance, _, _ = course(geometry, collision, min(arrivals), bodies)
            exits = {point for point in warps if kind(point) == "LADDER"}
            assert exits and warps == exits | arrivals, (name, "ladder and passive arrival records")
            edges, reverse, dry_edges = defaultdict(set), defaultdict(set), defaultdict(set)
            for y in range(height):
                for x in range(width):
                    at = (x, y)
                    token = kind(at)
                    if x in (0, width - 1) or y in (0, height - 1):
                        assert token == "WALL", (name, "open ice-room boundary", at)
                    if token == "LADDER":
                        assert at in exits, (name, "ladder with no warp", at)
                    if at in bodies or at in exits or token not in ICE | {"FLOOR", "UP_WALL"}:
                        continue
                    for _, dx, dy in DIRECTIONS:
                        stop, trace = advance(at, dx, dy)
                        if stop == at:
                            continue
                        edges[at].add(stop)
                        reverse[stop].add(at)
                        if all(kind(point) not in ICE for point in trace):
                            dry_edges[at].add(stop)
            reverse_reach = reachable(exits, reverse)
            for start in arrivals:
                assert kind(start) == "FLOOR", (name, "unsafe arrival", start)
                stops = reachable([start], edges)
                assert exits <= stops <= reverse_reach, (name, start, "ladder or return lost")
                assert all(adjacent(*at) & stops for at in landmarks | bodies), (name, "unreachable discovery")
            if name == "MnVoyageursIce1F":
                # The entrance and far ladder are on opposite sides of the ice.
                assert max(exits) not in reachable([min(arrivals)], dry_edges), "dry route around the upper crossing"
            else:
                deep = next((a["x"], a["y"]) for a in room["objects"] if a["script"] == "MnVoyageursIceNeverMelt")
                assert not (adjacent(*deep) & reachable(arrivals, dry_edges)), "dry route to the deep ice shelf"
    print(f"{name}: hidden caches, reward approaches and every stopping state return")


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("pack", type=Path)
    pack = load_pack(parser.parse_args().pack)
    for layout in json.loads(Path(__file__).with_name("ice.json").read_text()):
        inspect(pack, pack["data"]["maps"][layout["map"]], layout)
