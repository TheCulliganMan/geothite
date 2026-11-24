#!/usr/bin/env python3
"""Check compiled northern cave routes and render the edited decisions."""

import argparse
from copy import deepcopy
import json
from pathlib import Path
import sys

SOURCE = Path(__file__).resolve().parent
sys.path[:0] = [
    str(SOURCE.with_name("crystal-detail-v04")),
    str(SOURCE.with_name("crystal-replacement-v02")),
]

from check_north import check_room, distance, edges  # noqa: E402
from review_north_detail import (  # noqa: E402
    BASIN, CLEFT, GROTTO, ORE, SourceArt, actor_bodies, approach, comparison,
    floor_square, in_frame, load_pack, movement, scene,
)

CAMERAS = {
    CLEFT: {
        "West stair and lower return": (7, 14),
        "Scout and the marked seep rock": (27, 7),
        "Low seep overlook": (22, 11),
        "East return among the banks": (34, 16),
    },
    GROTTO: {
        "Otta's bench and the near spring inlet": (9, 6),
        "Dry contour below the camp": (8, 13),
        "Far angler's bend": (24, 21),
        "Recessed supply pocket": (30, 6),
    },
    ORE: {
        "Lower bluff around the surveyor": (10, 19),
        "Fault gate": (17, 8),
        "Fork above the spring": (23, 33),
        "Cut between the lower shores": (22, 36),
        "Far supply shore": (25, 42),
    },
    BASIN: {
        "Lower dial above the arrival": (7, 23),
        "Center dial ledge": (20, 17),
        "Lever and the stone perch beyond": (21, 12),
        "Kensigard's perch": (24, 8),
        "Outer bank above the lake": (32, 17),
    },
}


def review_layout(parent, metadata, revised):
    layout = deepcopy(metadata)
    name = layout["map"]
    old_room = parent["data"]["maps"][name]
    layout["rows"] = revised["rows"]
    for kind in ("actors", "warps", "backgrounds"):
        layout[kind] = revised.get(kind, [])
    patches = {
        (patch["script"], patch["index"]): patch
        for patch in revised.get("script_patches", [])
    }
    for change in old_room["script_block_changes"]:
        name, index = change["source_script"], change["command_index"]
        args = old_room["scripts"][name][index]["args"]
        patches.setdefault((name, index), {
            "script": name, "index": index, "before_args": args, "after_args": args,
        })
    layout["script_patches"] = list(patches.values())
    return layout


def interaction(room, script, kind):
    collection = room["objects"] if kind == "actor" else room["events"]["bg_events"]
    event, = [event for event in collection if event["script"] == script]
    return event["x"], event["y"]


def measure(pack, name, layout, opened, revised):
    room, tiles, bodies, drops = movement(pack, name, layout, opened)
    start = tuple(layout["arrivals"][0])
    dry, wet = edges(tiles, bodies, drops), edges(tiles, bodies, drops, surf=True)
    walk, swim = distance(dry, start), distance(wet, start)
    undiscovered = wet.keys() - swim.keys()
    if opened or name in {CLEFT, GROTTO}:
        assert not undiscovered, (name, undiscovered)
    elif name == BASIN:
        assert all(y < 10 for _, y in undiscovered), undiscovered
    else:
        assert all(y < 8 for _, y in undiscovered), undiscovered
    background = {(item["x"], item["y"]) for item in room["events"]["bg_events"]}
    width, focus = floor_square(tiles, walk, bodies | background)
    picture = tuple(32 * v for v in layout["dimensions_metatiles"])
    result = {
        "walkable": len(walk), "with_surf": len(swim),
        "largest_empty_square": width, "largest_empty_camera": focus,
    }
    if name == CLEFT:
        end = tuple(layout["arrivals"][1])
        west = distance(dry, end)[start]
        ordinary = distance(edges(tiles, bodies, drops, jump=False), end)[start]
        rock = interaction(room, "MN_CAVE_R20_SEEP_CACHE", "background")
        assert approach(walk, rock) is None and approach(swim, rock) is not None
        result.update(eastbound=walk[end], westbound=west, ledge_savings=ordinary - west)
        if revised:
            scout_west, lookout, west_water = (27, 7), (22, 11), (23, 10)
            assert scout_west in walk and lookout in walk and west_water in swim
            assert tiles[west_water] == "WATER"
            assert in_frame(picture, scout_west, rock)
            assert in_frame(picture, lookout, rock)
    elif name == GROTTO:
        cache = interaction(room, "MN_R12_CAVE_CACHE", "actor")
        result.update(far_cache_walk=approach(walk, cache), far_cache_surf=approach(swim, cache))
        if revised:
            bank, water = (12, 7), (13, 7)
            assert bank in walk and water in wet[bank] and tiles[water] == "WATER"
            assert in_frame(picture, (9, 6), water)
            assert not in_frame(picture, (28, 11), cache)
            for stair in ((15, 23), (22, 23)):
                blocked = bodies | {stair}
                assert approach(distance(edges(tiles, blocked, drops), start), cache) is None
                assert approach(distance(edges(tiles, blocked, drops, surf=True), start), cache) is not None
    elif name == ORE:
        cache = interaction(room, "MN_R10_ITEM_04", "actor")
        seam = interaction(room, "MN_R10_STAR_SEAM", "background")
        result.update(
            spring_supply_walk=approach(walk, cache), spring_supply_surf=approach(swim, cache),
            star_seam=approach(walk, seam),
        )
        assert (result["star_seam"] is not None) == opened
        if not opened:
            escape = (26, 5)
            result["restored_fracture_egress"] = distance(dry, escape)[start]
            assert start not in distance(edges(tiles, bodies, drops, jump=False), escape)
        if revised:
            assert result["spring_supply_walk"] - result["spring_supply_surf"] >= 10
    else:
        creature = interaction(room, "MN_BASIN_STRANDRAK", "actor")
        lever = interaction(room, "MnBasinLever", "background")
        result.update(lever=approach(walk, lever), sanctuary=approach(walk, creature))
        assert (result["sanctuary"] is not None) == opened
        assert (approach(swim, creature) is not None) == opened
        if revised:
            assert room["attributes"]["time_of_day"] == "nite"
            trigger = (21, 12)
            assert trigger in walk and lever == (trigger[0] + 1, trigger[1])
            assert in_frame(picture, trigger, creature)
            stairs = (21, 11)
            assert (stairs in walk) == opened and (stairs in swim) == opened
            if opened:
                approach_steps = (trigger, stairs, (21, 10), (21, 9), (22, 9), (23, 9), (23, 8), (24, 8))
                assert all(right in dry[left] for left, right in zip(approach_steps, approach_steps[1:]))
                empty = actor_bodies(room, opened) - {creature}
                after_capture = edges(tiles, empty, drops, surf=True)
                assert distance(after_capture, start).keys() == after_capture.keys()
    return result


def review(parent, candidate, output):
    previous = SOURCE.with_name("crystal-replacement-v02") / "north.json"
    watched = {row["map"]: row for row in json.loads(previous.read_text())}
    authored = json.loads((SOURCE / "north.json").read_text())
    output.mkdir(parents=True, exist_ok=False)
    data = (parent, candidate)
    art = tuple(SourceArt(pack) for pack in data)
    summary = {}
    for authored_room in authored:
        name = authored_room["map"]
        layout = review_layout(parent, watched[name], authored_room)
        check_room(candidate, parent, layout)
        states = (False, True) if name in {ORE, BASIN} else (False,)
        phases = {}
        for opened in states:
            state = "opened" if opened else "closed"
            results = [measure(pack, name, layout, opened, bool(i)) for i, pack in enumerate(data)]
            phases[state] = dict(zip(("v04", "v05"), results))
            cameras = dict(CAMERAS[name])
            cameras["Previous largest empty floor"] = results[0]["largest_empty_camera"]
            cameras["Rebuilt largest empty floor"] = results[1]["largest_empty_camera"]
            for period in ("day", "nite") if name == BASIN else ("nite",):
                images = [
                    (f"{version}-{period}-{state}", scene(pack, texture, name, opened, period))
                    for version, pack, texture in zip(("v04", "v05"), data, art)
                ]
                contact_sheet = comparison(output, name, images, cameras)
                contact_sheet.save(output / f"{name}-{period}-{state}-cameras.png")
        summary[name] = phases
    (output / "measurements.json").write_text(json.dumps(summary, indent=2) + "\n")
    print(json.dumps(summary, indent=2))


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    for argument in ("parent", "candidate", "output"):
        parser.add_argument(argument, type=Path)
    args = parser.parse_args()
    review(load_pack(args.parent), load_pack(args.candidate), args.output)
