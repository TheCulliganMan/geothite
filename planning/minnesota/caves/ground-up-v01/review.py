#!/usr/bin/env python3
"""Check and render the authored cave candidates against the verified hosted pack."""

import argparse
from collections import deque
import gzip
import hashlib
from itertools import product
import json
from pathlib import Path
import sys

import cbor2
from PIL import Image, ImageDraw

sys.path.insert(0, str(Path(__file__).resolve().parents[2] / "tools"))
from render_reference import SourceArt, npc_palette_pack  # noqa: E402
from review_north_shore import oriented_sprite  # noqa: E402


PARENT_SHA256 = "f123f566ed0b70cd9e551a2c88673faac132555e610c3eb5e3ecc95816d1bc57"
UP = (0, -1)
DOWN = (0, 1)
DIRECTIONS = (UP, (1, 0), DOWN, (-1, 0))
WARP_TILES = {"LADDER", "WARP_CARPET_DOWN"}
SURF_CACHES = {"MN_CAVE_R20_SEEP_CACHE", "MN_R12_GROTTO_REVIVE"}
CASTOROCK = (21, 11)
JAW_BOULDER = (13, 6)
FRACTURE_ROCK = (40, 6)
STAR_SEAM = (44, 10)
FACING = {"UP": UP, "RIGHT": (1, 0), "DOWN": DOWN, "LEFT": (-1, 0)}


def neighbors(tile):
    x, y = tile
    return [(x + dx, y + dy) for dx, dy in DIRECTIONS]


def distances(terrain, start, blockers, drops, surf=False):
    permitted = {"FLOOR", "UP_WALL", "HOP_DOWN"}
    if surf:
        permitted.add("WATER")
    assert terrain.get(start) in permitted and start not in blockers, start
    reached = {start: 0}
    pending = deque([start])
    while pending:
        at = pending.popleft()
        for step, target in zip(DIRECTIONS, neighbors(at), strict=True):
            if step == DOWN and at in drops:
                target = drops[at]
            kind = terrain.get(target)
            if kind not in permitted or target in blockers or target in reached:
                continue
            if (kind == "UP_WALL" and step == DOWN) or (
                terrain[at] == "UP_WALL" and step == UP
            ):
                continue
            reached[target] = reached[at] + 1
            pending.append(target)
    return reached


def check_outside(pack, room, warp):
    maps = pack["data"]["maps"]
    target = warp["target_map"]
    outside = maps.get(target)
    if outside is None:
        outside = next(m for m in maps.values() if m["attributes"]["map_constant"] == target)
    attributes = outside["attributes"]
    collision = pack["data"]["tilesets"][attributes["tileset_name"]]["collision"]

    def collision_at(event):
        x, y = event["x"], event["y"]
        block = outside["blocks"][y // 2 * attributes["width"] + x // 2]
        return collision[f"{block:02x}"][y % 2 * 2 + x % 2]

    outside_warps = outside["events"]["warps"]
    landing = next(event for event in outside_warps if event["index"] == warp["target_warp_id"])
    assert collision_at(landing) in {"CAVE", "FLOOR"}, "unsafe outside arrival"
    assert any(
        event["target_map"] in (room["id"], room["attributes"]["map_constant"])
        and event["target_warp_id"] == warp["index"]
        and abs(event["x"] - landing["x"]) + abs(event["y"] - landing["y"]) <= 1
        and collision_at(event) == "CAVE"
        for event in outside_warps
    ), "outdoor entrance and safe return no longer pair"


def check_castorock(terrain, arrival, bodies, dry, wet):
    assert JAW_BOULDER in bodies and CASTOROCK in bodies
    assert {(3, 5), (5, 10)} <= dry.keys()
    assert (4, 11) not in dry and (4, 11) in wet
    east_approaches = {(19, 4), (23, 12), (21, 10)}
    assert east_approaches.isdisjoint(wet)
    assert all(terrain[x, 6] == "FLOOR" for x in range(10, 17))
    # Stand behind the rock for the two pushes, then check the opened clues
    # with the rock in its authored third state and the creature still visible.
    assert (12, 6) in dry
    stationary = bodies - {JAW_BOULDER}
    opened = stationary | {(15, 6)}
    eastern = distances(terrain, (14, 6), opened, {})
    assert east_approaches | {arrival, (16, 6)} <= eastern.keys()
    returned = stationary | {(10, 6)}
    assert arrival in distances(terrain, (11, 6), returned, {})


def check_layout(pack, layout):
    maps = pack["data"]["maps"]
    room = maps[layout["map"]]
    attributes = room["attributes"]
    width, height = layout["dimensions_metatiles"]
    assert (attributes["width"], attributes["height"]) == (width, height)
    assert attributes["tileset_name"] == "cave" and not attributes["connections"]
    assert not room["script_block_changes"] and not room["events"]["coord_events"]
    assert hashlib.sha256(bytes(room["blocks"])).hexdigest() == layout["before_blocks_sha256"]
    rows = [row.split() for row in layout["rows"]]
    assert len(rows) == height and all(len(row) == width for row in rows)
    blocks = [int(block, 16) for row in rows for block in row]
    collision = pack["data"]["tilesets"]["cave"]["collision"]
    def terrain_for(values):
        return {
            (x, y): collision[f"{values[y // 2 * width + x // 2]:02x}"][y % 2 * 2 + x % 2]
            for y in range(height * 2)
            for x in range(width * 2)
        }

    terrain = terrain_for(blocks)
    old_terrain = terrain_for(room["blocks"])
    drops = {tuple(drop["from"]): tuple(drop["to"]) for drop in layout.get("drops", [])}
    kinds = set(terrain.values())
    assert kinds <= {"WALL", "FLOOR", "UP_WALL", "WATER", "HOP_DOWN"} | WARP_TILES, kinds
    assert {p for p, kind in terrain.items() if kind == "HOP_DOWN"} == drops.keys()
    for (x, y), landing in drops.items():
        assert landing == (x, y + 2)
        assert terrain[x, y + 1] == "WALL" and terrain[landing] == "FLOOR"
    warps = room["events"]["warps"]
    mouths = {(warp["x"], warp["y"]) for warp in warps}
    assert {p for p, kind in terrain.items() if kind in WARP_TILES} == mouths
    assert all(
        kind == old_terrain[x, y]
        for (x, y), kind in terrain.items()
        if x in (0, width * 2 - 1) or y in (0, height * 2 - 1)
    )
    arrivals = [tuple(tile) for tile in layout["arrivals"]]
    for arrival, warp in zip(arrivals, warps, strict=True):
        assert arrival in neighbors((warp["x"], warp["y"]))
        check_outside(pack, room, warp)
    actors = {(actor["x"], actor["y"]): actor for actor in room["objects"]}
    assert all(terrain[tile] == "FLOOR" for tile in actors)
    castorock_gate = layout["map"] == "MnR03Cave1F"
    ore_fracture = layout["map"] == "MnR10Cave1F"
    if ore_fracture:
        assert actors[FRACTURE_ROCK]["script"] == "MN_R10_SURVEY_FRACTURE"
        assert {p for p, kind in terrain.items() if kind == "WATER"} == {
            p for p, kind in old_terrain.items() if kind == "WATER"
        }
    fracture_states = (False, True) if ore_fracture else (False,)
    for smashed, collected in product(fracture_states, (False, True)):
        bodies = {
            tile for tile, actor in actors.items()
            if not (collected and actor["object_type"] == "OBJECTTYPE_ITEMBALL")
            and not (smashed and tile == FRACTURE_ROCK)
        }
        for arrival in arrivals:
            dry = distances(terrain, arrival, bodies, drops)
            wet = distances(terrain, arrival, bodies, drops, surf=True)
            assert set(arrivals) <= dry.keys()
            accessible_drops = drops.keys()
            if ore_fracture and not smashed:
                accessible_drops = accessible_drops - {(42, 16), (43, 16)}
            assert accessible_drops <= dry.keys() and set(drops.values()) <= dry.keys()
            assert all(any(p in dry for p in neighbors(mouth)) for mouth in mouths)
            accessible_bodies = bodies - {CASTOROCK} if castorock_gate else bodies
            assert all(any(p in dry for p in neighbors(body)) for body in accessible_bodies)
            if castorock_gate:
                check_castorock(terrain, arrival, bodies, dry, wet)
            for target in room["events"]["bg_events"]:
                at = (target["x"], target["y"])
                dry_approaches = set(neighbors(at)) & dry.keys()
                wet_approaches = set(neighbors(at)) & wet.keys()
                if target["script"] in SURF_CACHES:
                    assert terrain[at] == old_terrain[at]
                    assert not dry_approaches and wet_approaches
                elif target["script"] in {"MnJawPillar", "MnJawBed"}:
                    assert terrain[at] == old_terrain[at] and not wet_approaches
                elif ore_fracture and at == STAR_SEAM:
                    assert terrain[at] == old_terrain[at]
                    assert bool(dry_approaches) == bool(wet_approaches) == smashed
                else:
                    assert dry_approaches, target["script"]
                    if target["script"] in {
                        "MN_CAVE_CEDAR_MAX_REPEL", "MN_CAVE_R20_UPPER_CACHE",
                        "MN_HIDDEN_R17CAVE1F",
                    }:
                        assert terrain[at] == "WALL", target["script"]
            dry_return_points = dry.keys()
            if ore_fracture:
                # A save from inside the fracture chamber may load with the
                # rock restored. Its ledge must still get the player back.
                dry_return_points = {
                    p for p, kind in terrain.items() if kind in {"FLOOR", "UP_WALL", "HOP_DOWN"}
                } - bodies
                if smashed:
                    assert dry_return_points <= dry.keys()
            assert all(arrival in distances(terrain, p, bodies, drops)
                       for p in dry_return_points), "stranded dry position"
            assert all(
                arrival in distances(terrain, point, bodies, drops, surf=True)
                for point in wet
            ), "stranded cave position"
            for (x, y), actor in actors.items():
                direction = actor["sightline_direction_override"]
                if direction:
                    dx, dy = FACING[direction]
                    ray = {(x + n * dx, y + n * dy) for n in range(1, actor["radius"] + 1)}
                    assert ray <= dry.keys(), actor["script"]
            if not collected and not smashed:
                print(f'{layout["map"]} from {arrival}: {len(dry)} dry, {len(wet)} with Surf')
                for other in arrivals:
                    if other != arrival:
                        print(f"  dry crossing to {other}: {dry[other]} steps")
    return room, blocks


def render(pack, layouts, output):
    art = SourceArt(pack)
    output.mkdir(parents=True, exist_ok=True)
    for layout in layouts:
        room, blocks = check_layout(pack, layout)
        width, height = layout["dimensions_metatiles"]
        group = room["attributes"]["time_of_day"] or "day"
        sprites = npc_palette_pack(pack, group)
        versions = []
        for tiles in (room["blocks"], blocks):
            scene = art.render("cave", width, height, tiles, group=group).convert("RGBA")
            for actor in room["objects"]:
                direction = actor["spritemovedata"].rsplit("_", 1)[-1]
                facing = {"UP": "north", "RIGHT": "east", "LEFT": "west"}.get(direction, "south")
                sprite = oriented_sprite(sprites, actor["sprite"], facing)
                scene.alpha_composite(sprite, (actor["x"] * 16, actor["y"] * 16))
            versions.append(scene.convert("RGB"))
        picture_width, picture_height = versions[0].size
        panel = Image.new("RGB", (picture_width * 2 + 36, picture_height + 38), "#ece7da")
        draw = ImageDraw.Draw(panel)
        for i, (label, image) in enumerate(zip(("Installed", "Redesign"), versions, strict=True)):
            x = i * (picture_width + 12) + 12
            draw.text((x, 6), label, fill="#262223")
            panel.paste(image, (x, 26))
        panel.save(output / f'{layout["map"]}.png')
        cameras = layout["cameras"]
        closeups = Image.new("RGB", (352, len(cameras) * 170), "#ece7da")
        draw = ImageDraw.Draw(closeups)
        for i, (label, (x, y)) in enumerate(cameras.items()):
            top = i * 170
            draw.text((12, top + 4), label, fill="#262223")
            for column, scene in enumerate(versions):
                left = max(0, min(x * 16 - 72, picture_width - 160))
                upper = max(0, min(y * 16 - 64, picture_height - 144))
                crop = scene.crop((left, upper, left + 160, upper + 144))
                closeups.paste(crop, (column * 172 + 12, top + 20))
        closeups.save(output / f'{layout["map"]}-cameras.png')


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("pack", type=Path)
    parser.add_argument("output", type=Path)
    args = parser.parse_args()
    raw = args.pack.read_bytes()
    if raw.startswith(b"\x1f\x8b"):
        raw = gzip.decompress(raw)
    assert hashlib.sha256(raw).hexdigest() == PARENT_SHA256, "unexpected hosted parent"
    assert raw[:12] == b"CRYSTALPACK\0"
    source = Path(__file__).parent
    layouts = json.loads((source / "layouts.json").read_text(encoding="utf-8"))
    for filename in ("ore-line.json", "gunflint.json"):
        layouts.append(json.loads((source / filename).read_text(encoding="utf-8")))
    render(cbor2.loads(raw[22:]), layouts, args.output)


if __name__ == "__main__":
    main()
