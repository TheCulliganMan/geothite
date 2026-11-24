#!/usr/bin/env python3
"""Verify the six v05 southern caves against their compiled route contracts."""

import argparse
from collections import Counter
import json
from pathlib import Path
import sys

SOURCE = Path(__file__).resolve().parent
BASE_CONTRACT = SOURCE.parent / "crystal-replacement-v02" / "south.json"
sys.path.insert(0, str(BASE_CONTRACT.parent))
from review_south import adjacent, audit_compiled, distances, graph, position, terrain_for

sys.path.insert(0, str(SOURCE.parents[1] / "tools"))
from render_reference import load_pack


def minimum_approach(edges, start, target):
    reached = distances(edges, start)
    return min(reached[at] for at in adjacent(target) if at in reached)


def check_surface_pairs(pack, maps, inside):
    for event in inside["events"]["warps"]:
        outside = maps[event["target_map"]]
        if outside["attributes"]["environment"] == "CAVE":
            continue
        bank = next(warp for warp in outside["events"]["warps"] if warp["index"] == event["target_warp_id"])
        x, y = position(bank)
        terrain = terrain_for(pack, outside)
        assert terrain[x, y] == "FLOOR"
        assert terrain[x, y - 1] == "CAVE"
        door = next(warp for warp in outside["events"]["warps"] if position(warp) == (x, y - 1))
        assert maps[door["target_map"]] == inside
        assert door["target_warp_id"] == event["index"]


def check_room(pack, contract):
    name = contract["map"]
    keyed_maps = pack["data"]["maps"]
    lookup = {**keyed_maps, **{m["attributes"]["map_constant"]: m for m in keyed_maps.values()}}
    room = keyed_maps[name]
    start = tuple(contract["arrivals"][0])
    terrain = terrain_for(pack, room)
    occupied = {position(actor) for actor in room["objects"]}
    actors = {actor["script"]: position(actor) for actor in room["objects"]}
    markers = {event["script"]: position(event) for event in room["events"]["bg_events"]}
    foot = graph(terrain, occupied)
    wet = graph(terrain, occupied, surf=True)
    reached = distances(foot, start)
    wet_reached = distances(wet, start)
    width, height = 2 * room["attributes"]["width"], 2 * room["attributes"]["height"]
    assert all(
        kind == "WALL"
        for (x, y), kind in terrain.items()
        if x in (0, width - 1) or y in (0, height - 1)
    ), (name, "unsealed cave boundary")
    check_surface_pairs(pack, lookup, room)

    if name == "MnR01Cave1F":
        guide = actors["MN_R01_HOLLOW_GUIDEFieldHM"]
        antidote = actors["MN_R01_ITEM_03"]
        rookie_x, rookie_y = actors["MN_CAVE_DRIFTLESS_ROOKIE"]
        ray = {(rookie_x - offset, rookie_y) for offset in (1, 2, 3)}
        alt = graph(terrain, occupied | ray)
        gallery, east = map(tuple, contract["arrivals"][1:])
        assert len(reached) == len(foot)
        assert reached[gallery] <= 26
        assert minimum_approach(foot, start, guide) <= 5
        assert minimum_approach(foot, start, antidote) <= 26
        assert east in distances(alt, start)
        assert (27, 20) in distances(alt, start)
        lower_crossing = {(x, y) for x in range(20, 27) for y in (20, 21, 22)}
        assert east in distances(graph(terrain, occupied | lower_crossing), start)
        assert east not in distances(graph(terrain, occupied | ray | lower_crossing), start)
        assert gallery in distances(alt, start)
    elif name == "MnDriftlessLampGallery":
        relief = markers["MnLampFossils"]
        assert relief == (14, 8)
        assert reached.get((13, 8), 99) <= 30
        assert minimum_approach(foot, start, actors["MN_LAMP_GALLERY_CACHE"]) <= 35
        assert any(
            command["command"] == "checkflag" and command["args"] == ["STATUSFLAGS_FLASH"]
            for command in room["scripts"]["MnLampFossils"]
        )
        assert len(reached) == len(foot)
    elif name == "MnR03Cave1F":
        boulder = actors["MN_R03_JAW_BOULDER"]
        late_destinations = [actors["MN_R03_JAW_CASTOROCK"], markers["MnJawPillar"], markers["MnJawBed"]]
        assert not any(adjacent(at) & wet_reached.keys() for at in late_destinations)
        unlocked_bodies = occupied - {boulder} | {(boulder[0] + 2, boulder[1])}
        unlocked = graph(terrain, unlocked_bodies, surf=True)
        assert unlocked.keys() <= distances(unlocked, start).keys()
        island = markers["MN_HIDDEN_R03CAVE1F"]
        assert adjacent(island) & wet_reached.keys()
        assert not adjacent(island) & reached.keys()
    elif name == "MnR13Cave1F":
        harper, ether = actors["MN_R13_HARPER"], actors["MN_R13_SHELTER_ETHER"]
        assert minimum_approach(foot, start, harper) <= 9
        assert minimum_approach(wet, start, ether) < minimum_approach(foot, start, ether)
        assert len(wet_reached) == len(wet)
    else:
        assert len(wet_reached) == len(wet)

    if name in {"MnR01Cave1F", "MnR05RockCut"}:
        marker = 0x4C if name == "MnR01Cave1F" else 0x63
        mouths = sorted(at for at, tile in terrain.items() if tile == "WARP_CARPET_DOWN")
        assert len(mouths) == 2
        for x, y in mouths:
            assert (x, y - 1) in reached
            index = ((y + 1) // 2) * room["attributes"]["width"] + x // 2
            assert room["blocks"][index] == marker

    return_departures = {
        "MnDriftlessLampGallery": (13, 8),
        "MnR05RockCut": (19, 8),
        "MnR17Cave1F": (17, 6),
    }
    if name in return_departures:
        departure = return_departures[name]
        fast = distances(foot, departure)[start]
        ordinary = distances(graph(terrain, occupied, jumps=False), departure)[start]
        assert ordinary - fast >= 6, (name, "return ledge fails to shorten the trip")

    kinds = Counter(terrain.values())
    print(f"{name} · {len(reached)} dry, {len(wet_reached)} with Surf, {kinds['WATER']} water tiles")


def verify(pack):
    baseline = {record["map"]: record for record in json.loads(BASE_CONTRACT.read_text())}
    authored = json.loads((SOURCE / "south.json").read_text())
    assert len(authored) == len(baseline)
    assert {record["map"] for record in authored} == baseline.keys()
    contracts = []
    for record in authored:
        name = record["map"]
        contract = {**baseline[name], **record}
        if name == "MnDriftlessLampGallery":
            contract["drops"] = [
                {"from": [16, 12], "to": [16, 14]},
                {"from": [17, 12], "to": [17, 14]},
            ]
        contracts.append(contract)
    audit_compiled(pack, contracts)
    for contract in contracts:
        check_room(pack, contract)


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("candidate", type=Path)
    verify(load_pack(parser.parse_args().candidate))
