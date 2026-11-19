#!/usr/bin/env python3
"""Check the signed launch, both Route 18 roads, the Surf-only Revive and native moving sand edges."""

import argparse
import base64
from collections import deque
from functools import lru_cache
import json
from pathlib import Path
import runpy
import sys

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE.parents[1] / "tools"))
from audit_story_flow import instructions
from render_reference import SourceArt, load_pack
from review_grand_marais import sprite

Trail = runpy.run_path(str(HERE.parent / "northern-trails-v01/check-voyageurs.py"))[
    "Trail"
]
DESIGN = json.loads((HERE / "route-terrain.json").read_text())
WEST, EAST = "MnR18C00R00", "MnR18C01R00"
OLD, NEW = "mn_headwaters", "mn_r18_shore"
DIRS = ((0, -1), (1, 0), (0, 1), (-1, 0))
ROAD = {"FLOOR", "TALL_GRASS"}
BALL = (61, 35)


def adjacent(point):
    x, y = point
    return [(x + dx, y + dy) for dx, dy in DIRS]


def reachable(grid, start, permitted, occupied):
    assert grid[start] in permitted and start not in occupied, f"illegal start {start}"
    distance = {start: 0}
    queue = deque([start])
    while queue:
        p = queue.popleft()
        for n in adjacent(p):
            if n not in distance and n not in occupied and grid.get(n) in permitted:
                distance[n] = distance[p] + 1
                queue.append(n)
    return distance


def surfaces(pack):
    data = pack["data"]
    neighbor = Trail(data, [WEST, EAST, "MnBrainerd", "MnMilleLacs"])
    for y in (10, 11):
        for a, b in [
            (("MnBrainerd", 63, y + 14), (WEST, 0, y)),
            ((WEST, 63, y), (EAST, 0, y)),
            ((EAST, 15, y), ("MnMilleLacs", 0, y + 2)),
        ]:
            assert neighbor.step(a, "RIGHT") == b and neighbor.step(b, "LEFT") == a, (
                a,
                b,
            )
    grid, occupied = {}, set()
    for name, offset in [(WEST, 0), (EAST, 64)]:
        for y, row in enumerate(neighbor.grid[name]):
            grid.update({(x + offset, y): role for x, role in enumerate(row)})
        occupied.update((x + offset, y) for x, y in neighbor.bodies[name])
    return grid, occupied


def trip(pack, changed):
    room = pack["data"]["maps"][WEST]
    grid, actors = surfaces(pack)
    shore = reachable(grid, (0, 10), ROAD, actors)
    assert (79, 10) in shore
    for stand in adjacent(BALL):
        assert stand not in shore, f"Revive is reachable without Surf from {stand}"
    lake = reachable(grid, (49, 20), {"WATER"}, actors)
    assert set(lake) == {p for p, v in grid.items() if v == "WATER"}
    assert all(0 < x < 79 and 0 < y < 47 for x, y in lake), (
        "water spills beyond the map"
    )
    assert (63, 29) in lake and (64, 29) in lake, "water route breaks at the map seam"
    # Both new sandy points and the old pier apron allow dismount and return.
    island = reachable(grid, (60, 35), ROAD, actors)
    assert set(island).isdisjoint(shore)
    assert set(adjacent(BALL)) <= set(island)
    island_contacts = {(a, b) for a in island for b in adjacent(a) if b in lake}
    island_stops = (
        [((56, 35), (55, 35)), ((62, 39), (62, 40)), ((66, 35), (67, 35))]
        if changed
        else [((58, 35), (57, 35)), ((62, 39), (62, 40)), ((65, 35), (66, 35))]
    )
    assert set(island_stops) <= island_contacts
    for x, y in [
        (24, 15),
        (27, 15),
        (32, 15),
        (42, 17),
        (46, 19),
        (48, 19),
        (49, 19),
        (52, 19),
        (60, 19),
        (65, 19),
        (75, 19),
    ]:
        assert (x, y) in shore and (x, y + 1) in lake, (
            f"lost signed or public launch at {x},{y}"
        )
    (ball,) = [o for o in room["objects"] if (o["x"], o["y"]) == BALL]
    assert (
        ball["sprite"] == "SPRITE_POKE_BALL"
        and ball["object_type"] == "OBJECTTYPE_ITEMBALL"
    )
    assert instructions(room, ball["script"]) == [("itemball", "REVIVE", "1")]
    (pickup,) = [
        p for p in room["script_field_pickups"] if p["source_script"] == ball["script"]
    ]
    assert pickup["event_flag"] == ball["event_flag"] == "EVENT_MN_R18_ITEM_04"
    assert (pickup["item_id"], pickup["quantity"]) == ("REVIVE", 1)
    print(
        f"{'Child' if changed else 'Parent'}: {shore[79, 10]} dry steps between towns, "
        f"{len(lake)} joined Surf tiles, {len(island)} reachable non-occupied island squares, "
        f"{len(island_contacts)} two-way island shore contacts; all 11 public launches pass."
    )
    return grid


def shade(frame, row_offset, bit):
    return int(bool(frame[row_offset] & bit)) + 2 * int(
        bool(frame[row_offset + 1] & bit)
    )


def check_native(parent, child):
    old, new = parent["data"]["tilesets"][OLD], child["data"]["tilesets"][NEW]
    source, target = parent["runtime_files"], child["runtime_files"]

    def bank(where, name, suffix):
        return bytes(where[f"data/tilesets/{name}{suffix}"])

    atlas = bank(source, OLD, "_metatiles.bin")
    first = len(atlas) // 16
    expanded = bank(target, NEW, "_metatiles.bin")
    assert expanded == atlas + b"".join(bytes(b["tiles"]) for b in DESIGN["metatiles"])
    assert new["collision"] == {
        **old["collision"],
        **{
            f"{i:02x}": b["collision"] for i, b in enumerate(DESIGN["metatiles"], first)
        },
    }
    assert json.loads(bank(target, NEW, ".json")) == new["collision"]
    assert json.loads(bank(target, NEW, "_palette_map.json")) == new["palette_map"]
    for map_name in (WEST, EAST):
        room, was = (p["data"]["maps"][map_name] for p in (child, parent))
        assert was["attributes"]["tileset_name"] == OLD
        assert room["attributes"] == dict(was["attributes"], tileset_name=NEW)
        assert room["objects"] == was["objects"]
        assert child["data"]["wild_encounters"].get(map_name) == parent["data"][
            "wild_encounters"
        ].get(map_name)
        width = room["attributes"]["width"]
        layout = was["blocks"].copy()
        for x, y, old_id, new_id in DESIGN["rooms"][map_name]:
            assert layout[y * width + x] == old_id
            layout[y * width + x] = new_id if new_id < 256 else first + new_id - 256
        assert layout == room["blocks"]
        assert bytes(layout) == base64.b64decode(
            child["data"]["map_blocks"][room["attributes"]["blocks_label"]]
        )
    for move in ("cut", "whirlpool"):
        replacements = child["data"]["field_moves"][move]["replacements"]
        assert replacements.get(NEW) == parent["data"]["field_moves"][move][
            "replacements"
        ].get(OLD)
    native = bytes(source["gfx/tilesets/water/water.2bpp"])
    assert len(native) == 64
    clips = json.loads(bank(target, NEW, "_animations.json"))
    old_clips = json.loads(bank(source, OLD, "_animations.json"))
    backgrounds = json.loads(bank(source, OLD, "_native_backgrounds.json"))
    graphics = bytearray(source[f"gfx/tilesets/{OLD}.2bpp"])
    palette = old["palette_map"].copy()
    new_ids = {str(i) for i in range(0x60, 0x60 + len(DESIGN["masks"]))}
    assert set(clips) == set(old_clips) | new_ids and set(old_clips).isdisjoint(new_ids)
    assert all(clips[k] == v for k, v in old_clips.items())
    for i, mask in enumerate(DESIGN["masks"], 0x60):
        clip = clips[str(i)]
        assert (len(clip["frames"]), clip["frame_ticks"]) == (4, 22)
        assert i not in atlas and not any(graphics[i * 16 : (i + 1) * 16])
        palette[i] = 3
        backgrounds[str(i)] = 0
        for f, frame in enumerate(clip["frames"]):
            for y in range(8):
                for bit in [1 << n for n in range(8)]:
                    source_ink = shade(native, f * 16 + 2 * y, bit)
                    wet, shallow = mask["wet"][y] & bit, mask["shallow"][y] & bit
                    assert not shallow or wet
                    expected = (
                        (1 if shallow and source_ink == 2 else source_ink) if wet else 0
                    )
                    assert shade(frame, 2 * y, bit) == expected, (i, f, y, bit)
        graphics[i * 16 : (i + 1) * 16] = bytes(clip["frames"][0])
    assert (
        graphics == bytes(target[f"gfx/tilesets/{NEW}.2bpp"])
        and palette == new["palette_map"]
    )
    assert backgrounds == json.loads(bank(target, NEW, "_native_backgrounds.json"))
    old_art, new_art = SourceArt(parent), SourceArt(child)
    for clock in ("morn", "day", "nite"):
        base, art = old_art.blocks(OLD, clock), new_art.blocks(NEW, clock)
        assert all(a.tobytes() == b.tobytes() for a, b in zip(base, art[:first]))
        for image in art[first:]:
            assert all(
                len(image.crop((x, y, x + 8, y + 8)).getcolors()) <= 4
                for x in range(0, 32, 8)
                for y in range(0, 32, 8)
            )
    print(
        f"All {first} original metatiles match at three clocks. "
        f"All {len(DESIGN['masks'])} shallow glyphs match four masked native phases."
    )


def sprite_support(pack, names):
    """Union the soles or keel in all six native frames, including mirrored strides."""
    support = {(x, y) for x in range(5, 11) for y in range(10, 15)}
    for name in names:
        for pose in range(6):
            image = sprite(pack, name, frame_offset=pose * 16)
            for y in (14, 15):
                for x in range(16):
                    if image.getpixel((x, y))[3]:
                        support.update([(x, y), (15 - x, y)])
    return support


def check_footprints(pack):
    """Recover land and water straight from native animation planes, independent of the authored recipe."""
    data, files = pack["data"], pack["runtime_files"]
    old_clips = json.loads(bytes(files[f"data/tilesets/{OLD}_animations.json"]))
    clips = json.loads(bytes(files[f"data/tilesets/{NEW}_animations.json"]))
    native = bytes(files["gfx/tilesets/water/water.2bpp"])

    def surface(frames):
        # White foam can cover water on one frame. Across the cycle, each
        # genuine water pixel has blue ink on at least one frame.
        rows = [0] * 8
        for frame in frames:
            for y in range(8):
                rows[y] |= frame[2 * y] | frame[2 * y + 1]
        return rows

    water = {0x14: surface([native[i : i + 16] for i in range(0, len(native), 16)])}
    assert water[0x14] == [255] * 8
    water.update(
        {int(k): surface(c["frames"]) for k, c in clips.items() if k not in old_clips}
    )
    assert len(water) > 1
    atlas = bytes(files[f"data/tilesets/{NEW}_metatiles.bin"])
    dry_support = sprite_support(pack, ["CHRIS", "KRIS"])
    wet_support = sprite_support(pack, ["MN_CANOE_CHRIS", "MN_CANOE_KRIS"])
    grid, occupied = surfaces(pack)
    # Both banks of both coves, the strait between route maps, and the complete
    # island circumference. Coordinates are continuous across the map boundary.
    cells = {(x, y) for x in range(45, 78) for y in range(17, 23)}
    cells.update((x, y) for x in range(54, 70) for y in range(29, 42))

    @lru_cache(maxsize=None)
    def wet_at(px, py):
        x, y = px // 16, py // 16
        room = data["maps"][WEST if x < 64 else EAST]
        assert room["attributes"]["tileset_name"] == NEW
        block = room["blocks"][y // 2 * room["attributes"]["width"] + (x % 64) // 2]
        glyph = atlas[16 * block + 4 * ((py // 8) % 4) + (px // 8) % 4]
        return bool(water.get(glyph, (0,) * 8)[py % 8] & (128 >> (px % 8)))

    bad, count = [], 0
    for x, y in sorted(cells):
        kind = grid[x, y]
        if kind not in {"FLOOR", "WATER"}:
            continue
        count += 1
        support = dry_support if kind == "FLOOR" else wet_support
        wrong = {
            (i, j)
            for i, j in support
            if wet_at(16 * x + i, 16 * y + j) != (kind == "WATER")
        }
        if wrong:
            bad.append((x, y, kind, len(wrong)))
    assert not bad, (
        f"wrong native water below boot soles or canoe keel at {len(bad)} squares: {bad}"
    )
    # Halfway across a stride, a southern-facing sprite's soles already extend
    # well past the bank. Test the same native pixel as the presentation
    # handoff on both sides of the boundary at every two-pixel movement tick.
    crossings, strides = 0, 0
    for x, y in cells - occupied:
        if grid[x, y] not in {"FLOOR", "WATER"}:
            continue
        for dx, dy in DIRS:
            destination = x + dx, y + dy
            if destination not in cells or destination in occupied:
                continue
            if grid[x, y] == "FLOOR" and grid[destination] == "WATER":
                crossings += 1
                for t in range(0, 17, 2):
                    px, py = 16 * x + 8 + t * dx, 16 * y + 15 + t * dy
                    assert wet_at(px, py) == ((px // 16, py // 16) == destination), (
                        f"launch or reverse landing at {(x, y)} to {destination}, {t}px"
                    )
            elif grid[x, y] == grid[destination] and (dx, dy) in [(1, 0), (0, 1)]:
                strides += 1
                afloat = grid[x, y] == "WATER"
                support = wet_support if afloat else dry_support
                for t in range(1, 16):
                    assert all(
                        wet_at(16 * x + i + t * dx, 16 * y + j + t * dy) == afloat
                        for i, j in support
                    ), f"wrong surface during ordinary stride {(x, y)} to {destination}"
    assert crossings and strides
    # Walk across the receded points, launch, reverse the landing and walk out.
    # The easternmost line also exercises the right-facing brown-rock beach.
    triplets = [
        ((56, 20), (56, 21), (56, 22)),
        ((58, 36), (58, 37), (58, 38)),
        ((70, 20), (70, 21), (70, 22)),
        ((65, 35), (66, 35), (67, 35)),
        ((57, 35), (56, 35), (55, 35)),
        ((63, 18), (64, 18), (64, 19)),
    ]
    for inland, shore, afloat in triplets:
        assert grid[inland] == grid[shore] == "FLOOR" and grid[afloat] == "WATER"
        assert inland in adjacent(shore) and shore in adjacent(afloat)
        assert not occupied.intersection({inland, shore, afloat})
    print(
        f"{count} shoreline squares support both walkers and both canoes through all six native "
        f"sprite poses and mirrored strides. {crossings} reversible launch edges and {15 * strides} "
        "intermediate walking or paddling positions agree with their native surface."
    )


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--footprints",
        action="store_true",
        help="check a single written Shore Watch archive",
    )
    parser.add_argument("parent", type=Path)
    parser.add_argument("child", type=Path, nargs="?")
    args = parser.parse_args()
    parent = load_pack(args.parent)
    if args.footprints:
        assert not args.child
        check_footprints(parent)
        return
    before = trip(parent, False)
    if args.child:
        child = load_pack(args.child)
        after = trip(child, True)
        for role in ("TALL_GRASS", "HEADBUTT_TREE", "CUT_TREE", "BUOY", "HOP_DOWN"):
            assert {p for p, v in before.items() if v == role} == {
                p for p, v in after.items() if v == role
            }
        check_native(parent, child)
        check_footprints(child)


if __name__ == "__main__":
    main()
