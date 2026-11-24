#!/usr/bin/env python3
"""Check the southern cave details using exact old and new compiled rooms."""

import argparse
import json
from pathlib import Path
import sys

SOURCE = Path(__file__).resolve().parent
BASE_SOURCE = SOURCE.parent / "crystal-replacement-v02"
sys.path.insert(0, str(BASE_SOURCE))
from review_south import audit_compiled, distances, graph, position, terrain_for  # noqa: E402

sys.path.insert(0, str(SOURCE.parents[1] / "tools"))
from render_reference import load_pack  # noqa: E402


def block_at(room, x, y):
    width = room["attributes"]["width"]
    return room["blocks"][(y // 2) * width + x // 2]


def review(parent, candidate, rows, contracts):
    old_maps = parent["data"]["maps"]
    new_maps = candidate["data"]["maps"]
    expected = {room["map"] for room in contracts}
    assert len(rows) == len(expected) and {room["map"] for room in rows} == expected
    contract_by_name = {room["map"]: room for room in contracts}
    layouts = [{**contract_by_name[room["map"]], **room} for room in rows]
    audit_compiled(candidate, layouts)

    for layout in layouts:
        name = layout["map"]
        old, current = old_maps[name], new_maps[name]
        expected_blocks = [int(cell, 16) for row in layout["rows"] for cell in row.split()]
        assert current == {**old, "blocks": expected_blocks}
        terrain = terrain_for(candidate, current)
        retained = terrain_for(parent, old)
        for event in current["events"]["bg_events"]:
            at = position(event)
            assert terrain[at] == retained[at], (name, event["script"], "cue collision")
        for event in current["events"]["warps"]:
            at = position(event)
            assert terrain[at] == retained[at], (name, at, "warp collision")

        occupied = {position(actor) for actor in current["objects"]}
        first = tuple(layout["arrivals"][0])
        old_foot = distances(graph(retained, occupied), first)
        foot_routes = graph(terrain, occupied)
        foot = distances(foot_routes, first)
        old_wet = distances(graph(retained, occupied, surf=True), first)
        wet = distances(graph(terrain, occupied, surf=True), first)
        count = sum(a != b for a, b in zip(old["blocks"], current["blocks"]))
        print(f"{name}: {count} changed blocks, {len(old_foot)} → {len(foot)} dry tiles, "
              f"{len(old_wet)} → {len(wet)} with Surf")

        if name == "MnDriftlessLampGallery":
            relief = next(e for e in current["events"]["bg_events"] if e["script"] == "MnLampFossils")
            x, y = position(relief)
            assert (x - 1, y) in foot and block_at(current, x, y) == 0x4B
            assert any(
                command["command"] == "checkflag" and command["args"] == ["STATUSFLAGS_FLASH"]
                for command in current["scripts"]["MnLampFossils"]
            )
        if name == "MnR01Cave1F":
            assert foot_routes.keys() <= foot.keys(), "isolated Hollow walkable terrain"
            west, _, east = map(tuple, layout["arrivals"])
            elbow = (23, 19)
            bend = distances(foot_routes, elbow)
            back = distances(foot_routes, east)
            assert foot[elbow] + bend[east] == foot[east], "lower eastward crossing"
            assert back[elbow] + bend[west] == back[west], "lower westward crossing"
        if name in {"MnR01Cave1F", "MnR05RockCut"}:
            lip = 0x4C if name == "MnR01Cave1F" else 0x63
            mouths = [(x, y) for (x, y), kind in terrain.items() if kind == "WARP_CARPET_DOWN"]
            assert len(mouths) == 2 and all(y % 2 == 1 for _, y in mouths)
            assert all(block_at(current, x, y + 1) == lip for x, y in mouths)
            assert all((x, y - 1) in foot and block_at(current, x, y) == 0x24 for x, y in mouths)

    bank = "mn_r05_rock_cut"
    metatiles = f"data/tilesets/{bank}_metatiles.bin"
    old_native = bytes(parent["runtime_files"][metatiles])
    new_native = bytes(candidate["runtime_files"][metatiles])
    recipe = json.loads((SOURCE / "south-art.json").read_text())[0]["metatiles"][0]
    assert recipe["id"] == 0x63 and len(old_native) == 0x63 * 16
    assert new_native == old_native + bytes(recipe["glyphs"])
    assert recipe["glyphs"][2:4] == [0x1A, 0x1B]
    collision = candidate["data"]["tilesets"][bank]["collision"]
    assert collision["63"] == ["WALL"] * 4


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("parent", type=Path)
    parser.add_argument("candidate", type=Path)
    args = parser.parse_args()
    authored = json.loads((SOURCE / "south.json").read_text())
    previous = json.loads((BASE_SOURCE / "south.json").read_text())
    review(load_pack(args.parent), load_pack(args.candidate), authored, previous)
