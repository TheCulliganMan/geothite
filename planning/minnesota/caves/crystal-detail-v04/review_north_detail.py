#!/usr/bin/env python3
"""Measure the new northern paths and save native-scale frames from two compiled packs."""

import argparse
import json
from pathlib import Path
import sys

from PIL import Image, ImageDraw


SOURCE = Path(__file__).resolve().parent
sys.path[:0] = [
    str(SOURCE.with_name("crystal-replacement-v02")),
    str(SOURCE.parents[1] / "tools"),
]

from check_north import adjacent, distance, edges, terrain  # noqa: E402
from render_reference import SourceArt, load_pack, npc_palette_pack  # noqa: E402
from review_north_shore import oriented_sprite  # noqa: E402


CLEFT = "MnR20Cave1F"
GROTTO = "MnR12Cave1F"
ORE = "MnR10Cave1F"
BASIN = "MnInnerBasin"
FRACTURE = "MN_R10_SURVEY_FRACTURE"
CAMERAS = {
    CLEFT: {
        "Western shelf from the mouth": (7, 14),
        "Scout's western interaction": (26, 6),
        "Top of the lookout staircase": (22, 6),
        "Lone rock from the crest lookout": (22, 11),
        "Lone rock from the west water": (15, 14),
        "Eastern wayfinder and descent": (42, 16),
        "Westward ledge from the low route": (11, 16),
    },
    GROTTO: {
        "Otta's arrival and bench": (7, 4),
        "First view of the spring": (8, 10),
        "Foot of the western bank": (10, 18),
        "The two lower stair landings": (18, 24),
        "Angler's eastern bank": (24, 21),
        "Far supply shelf": (28, 7),
    },
    ORE: {
        "Reader and entrance": (7, 24),
        "Ordinary upper gallery": (11, 14),
        "Fracture gate": (18, 8),
        "Seam and route to the ledge": (24, 4),
        "Fault bend": (17, 28),
        "Spring landing": (16, 34),
        "Eastern supply shore": (26, 41),
    },
    BASIN: {
        "Lower dial and steps": (7, 25),
        "Center dial from its approach": (20, 17),
        "Lever from the eastern rise": (33, 14),
        "Sill and spillway": (20, 12),
        "Sanctuary from the newly opened path": (24, 7),
    },
}


def phase_blocks(room, opened):
    blocks = room["blocks"].copy()
    if opened:
        width = room["attributes"]["width"]
        for change in room["script_block_changes"]:
            at = change["y"] // 2 * width + change["x"] // 2
            blocks[at] = change["block_id"]
    return blocks


def actor_bodies(room, opened):
    return {
        (actor["x"], actor["y"]) for actor in room["objects"]
        if not (opened and actor["script"] == FRACTURE)
    }


def movement(pack, name, layout, opened):
    room = pack["data"]["maps"][name]
    tile_codes = pack["data"]["tilesets"][room["attributes"]["tileset_name"]]["collision"]
    tiles = terrain(room, tile_codes, phase_blocks(room, opened))
    drops = {tuple(drop["from"]): tuple(drop["to"]) for drop in layout.get("drops", [])}
    return room, tiles, actor_bodies(room, opened), drops


def approach(reached, point):
    return min((reached[tile] for tile in adjacent(point) if tile in reached), default=None)


def floor_square(tiles, reached, occupied):
    sizes = {}
    best = (0, (0, 0))
    for x, y in sorted(tiles, key=lambda pt: (pt[1], pt[0])):
        tile = (x, y)
        if tiles[tile] != "FLOOR" or tile not in reached or tile in occupied:
            continue
        size = 1 + min(sizes.get(p, 0) for p in [(x - 1, y), (x, y - 1), (x - 1, y - 1)])
        sizes[tile] = size
        if size > best[0]:
            best = size, (x - size // 2, y - size // 2)
    return best


def frame(draw_size, focus):
    width, height = draw_size
    x, y = focus
    left = max(0, min(x * 16 - 72, width - 160))
    top = max(0, min(y * 16 - 64, height - 144))
    return left, top, left + 160, top + 144


def in_frame(draw_size, focus, target):
    left, top, right, bottom = frame(draw_size, focus)
    x, y = (cell * 16 for cell in target)
    return left <= x and x + 16 <= right and top <= y and y + 16 <= bottom


def inspect(pack, name, layout, opened, revised):
    room, tiles, bodies, drops = movement(pack, name, layout, opened)
    start = tuple(layout["arrivals"][0])
    dry, wet = edges(tiles, bodies, drops), edges(tiles, bodies, drops, surf=True)
    walk, swim = distance(dry, start), distance(wet, start)
    hidden = {(event["x"], event["y"]) for event in room["events"]["bg_events"]}
    unvisited = wet.keys() - swim.keys()
    if opened or name in {CLEFT, GROTTO}:
        assert not unvisited, (name, unvisited)
    elif name == BASIN:
        assert all(y < 12 for (_, y) in unvisited), unvisited
    else:
        assert all(y < 8 for (_, y) in unvisited), unvisited

    widest, point = floor_square(tiles, walk, bodies | hidden)
    details = {
        "walkable_from_arrival": len(walk),
        "with_surf": len(swim),
        "largest_unoccupied_floor_square": widest,
        "wide_camera": list(point),
    }
    if name == CLEFT:
        west, east = (tuple(arrival) for arrival in layout["arrivals"])
        return_steps = distance(dry, east)[west]
        ordinary = distance(edges(tiles, bodies, drops, jump=False), east)[west]
        details["eastbound"] = walk[east]
        details["westbound"] = return_steps
        details["without_ledge"] = ordinary
        if revised:
            assert ordinary - return_steps >= 21
            scout, lookout, rock = (26, 6), (22, 11), (18, 14)
            lookout_stair, main_stair = (22, 7), (17, 7)
            west_water = (15, 14)
            picture = tuple(32 * size for size in layout["dimensions_metatiles"])
            assert tiles[lookout] == "FLOOR" and lookout in walk
            assert tiles[west_water] == "WATER" and west_water in swim
            assert approach(swim, rock) is not None and approach(walk, rock) is None
            assert in_frame(picture, scout, lookout_stair) and not in_frame(picture, scout, main_stair)
            assert in_frame(picture, lookout, rock) and in_frame(picture, west_water, rock)
            assert lookout not in distance(edges(tiles, bodies | {lookout_stair}, drops), scout)
            details["lookout_steps_from_scout"] = distance(dry, scout)[lookout]
            details["seep_restore_surf"] = approach(swim, rock)
    elif name == GROTTO:
        shelf = (29, 7)
        details["far_shelf_walk"] = approach(walk, shelf)
        details["far_shelf_surf"] = approach(swim, shelf)
        if revised:
            assert tiles[8, 6] == tiles[9, 6] == "WALL"
            assert tiles[8, 7] == tiles[9, 7] == "FLOOR"
            assert approach(walk, (10, 6)) == 4
            for blocked_stair in ((15, 23), (22, 23)):
                blockers = bodies | {blocked_stair}
                assert approach(distance(edges(tiles, blockers, drops), start), shelf) is None
                fallback = distance(edges(tiles, blockers, drops, surf=True), start)
                assert approach(fallback, shelf) is not None
    elif name == ORE:
        details["seam"] = approach(walk, (27, 5))
        details["far_spring_supply"] = approach(walk, (26, 42))
        if not opened:
            assert details["seam"] is None
            seam_approach = (26, 5)
            details["closed_fracture_escape"] = distance(dry, seam_approach)[start]
            without_drop = edges(tiles, bodies, drops, jump=False)
            assert start not in distance(without_drop, seam_approach)
    else:
        details["lever"] = approach(walk, (32, 14))
        details["sanctuary"] = approach(walk, (27, 7))
        assert (details["sanctuary"] is not None) == opened
    return details


def scene(pack, art, name, opened, period):
    room = pack["data"]["maps"][name]
    attrs = room["attributes"]
    group = attrs["time_of_day"] or period
    whole = art.render(
        attrs["tileset_name"], attrs["width"], attrs["height"], phase_blocks(room, opened), group=group,
    ).convert("RGBA")
    sprites = npc_palette_pack(pack, group)
    for actor in room["objects"]:
        if (opened and actor["script"] == FRACTURE) or (not opened and name == BASIN):
            continue
        facing = actor["spritemovedata"].rsplit("_", 1)[-1]
        direction = {"UP": "north", "RIGHT": "east", "LEFT": "west"}.get(facing, "south")
        image = oriented_sprite(sprites, actor["sprite"], direction)
        whole.alpha_composite(image, (16 * actor["x"], 16 * actor["y"]))
    return whole.convert("RGB")


def comparison(output, name, phases, cameras):
    width, height = phases[0][1].size
    sheet = Image.new("RGB", (172 * len(phases), 186 * len(cameras)), "#ede8db")
    draw = ImageDraw.Draw(sheet)
    for j, (version, whole) in enumerate(phases):
        whole.save(output / f"{name}-{version}.png")
        for i, (caption, focus) in enumerate(cameras.items()):
            if j == 0:
                draw.text((6, i * 186 + 3), caption, fill="#25212b")
            draw.text((j * 172 + 6, i * 186 + 20), version, fill="#25212b")
            crop = whole.crop(frame((width, height), focus))
            crop.save(output / f"{name}-{version}-camera-{i:02d}.png")
            sheet.paste(crop, (j * 172 + 6, i * 186 + 36))
    return sheet


def review(parent, candidate, output):
    watched = json.loads((SOURCE.with_name("crystal-replacement-v02") / "north.json").read_text())
    layouts = {layout["map"]: layout for layout in watched}
    authored = {layout["map"]: layout["rows"] for layout in json.loads((SOURCE / "north.json").read_text())}
    output.mkdir(parents=True, exist_ok=False)
    art = [SourceArt(parent), SourceArt(candidate)]
    report = {}
    for name, layout in layouts.items():
        expected = [int(v, 16) for row in authored[name] for v in row.split()]
        assert candidate["data"]["maps"][name]["blocks"] == expected, name
        phases = (False, True) if name in {ORE, BASIN} else (False,)
        detail = {}
        for opened in phases:
            label = "opened" if opened else "closed"
            summary = [inspect(pack, name, layout, opened, revised=bool(i))
                       for i, pack in enumerate((parent, candidate))]
            detail[label] = dict(zip(("wall_finish", "detail"), summary))
            cameras = {**CAMERAS[name], "Widest reachable floor": tuple(summary[1]["wide_camera"])}
            for period in (("nite", "day") if name == BASIN else ("nite",)):
                states = [
                    (f"{version}-{period}-{label}", scene(pack, asset, name, opened, period))
                    for version, pack, asset in zip(("v03", "v04"), (parent, candidate), art)
                ]
                sheet = comparison(output, name, states, cameras)
                sheet.save(output / f"{name}-{period}-{label}-cameras.png")
        report[name] = detail
    (output / "measurements.json").write_text(json.dumps(report, indent=2) + "\n")
    print(json.dumps(report, indent=2))


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    for arg in ("parent", "candidate", "new_output_directory"):
        parser.add_argument(arg, type=Path)
    args = parser.parse_args()
    review(load_pack(args.parent), load_pack(args.candidate), args.new_output_directory)
