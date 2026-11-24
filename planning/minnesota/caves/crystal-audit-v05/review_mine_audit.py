#!/usr/bin/env python3
"""Check the Iron Range routes from the parent and rebuilt compiled packs."""
import argparse
from collections import deque
import json
from pathlib import Path
import sys

sys.path.insert(0, str(Path(__file__).resolve().parent.parent / "crystal-detail-v04"))
from review_mine_flow import (
    APPROACH, DIRECTIONS, MINE, SUMP, VENT, approaches, audit,
    load_pack, neighbor, position, state_components,
)
from review_mines import reached


def distances(graph, start, excluded=()):
    excluded = set(excluded)
    assert start in graph and start not in excluded
    result = {start: 0}
    queue = deque([start])
    while queue:
        at = queue.popleft()
        for other in graph[at] - result.keys() - excluded:
            result[other] = result[at] + 1
            queue.append(other)
    return result


def actor(pack, room, script):
    room_data = pack["data"]["maps"][room]
    actor, = [record for record in room_data["objects"] if record["script"] == script]
    mirror, = [
        record["args"] for record in room_data["map_event_section_commands"]
        if record["command"] == "object_event" and record["args"][11] == script
    ]
    assert tuple(map(int, mirror[:2])) == position(actor), script
    return actor


def check(parent, candidate):
    results = audit(parent, candidate)
    dry = state_components(candidate, set(), False)[0]
    wet = state_components(candidate, set(), True)[0]
    previous_dry = state_components(parent, set(), False)[0]

    for room, script, before, after in (
        (MINE, "MN_R09_SURVEY_CACHE", (7, 5), (5, 3)),
        (APPROACH, "MN_IRON_APPROACH_ETHER", (9, 12), (19, 18)),
    ):
        old, new = actor(parent, room, script), actor(candidate, room, script)
        assert position(old) == before and position(new) == after
        assert {**old, "x": after[0], "y": after[1]} == new
        assert dry[room][1][after] == "FLOOR"

    approach = dry[APPROACH][0]
    start, exit_ = (6, 29), (18, 5)
    main = distances(approach, start)
    ether = (19, 18)
    side = {
        neighbor(ether, direction) for direction in DIRECTIONS.values()
    } & main.keys()
    through_ether = min(main[at] + distances(approach, at)[exit_] for at in side)
    assert main[exit_] + 4 <= through_ether
    right_shelf = [(x, y) for x in range(18, 22) for y in range(16, 21)]
    assert distances(approach, start, right_shelf)[exit_] == main[exit_]

    first, last = (6, 5), (10, 19)
    west = [(x, 13) for x in range(10)]
    east = [(x, 13) for x in range(9, 15)]
    assert last not in distances(previous_dry[VENT][0], first, west)
    vent = [distances(dry[VENT][0], first, closed)[last] for closed in (west, east)]

    lower = (8, 23)
    launch, water, cache = (7, 13), (8, 13), (8, 14)
    dry_sump, dry_terrain = dry[SUMP]
    wet_sump, _ = wet[SUMP]
    assert dry_terrain[launch] == "FLOOR" and dry_terrain[water] == "WATER"
    assert launch in reached(dry_sump, lower) and water in wet_sump[launch]
    assert not approaches(cache, reached(dry_sump, lower))
    assert approaches(cache, reached(wet_sump, lower))
    result = results["walking steps before and after"]["Pump Gallery return before and after feed"]
    assert result[0] - result[1] >= 16

    return results | {
        "Drill Approach direct and through Ether": [main[exit_], through_ether],
        "Ventilation fan and west passages": vent,
        "Sump dry and Surf routes": [distances(graph, lower)[(8, 5)] for graph in (dry_sump, wet_sump)],
    }


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("parent", type=Path)
    parser.add_argument("candidate", type=Path)
    args = parser.parse_args()
    print(json.dumps(check(load_pack(args.parent), load_pack(args.candidate)), indent=2))
