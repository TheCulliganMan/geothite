#!/usr/bin/env python3
"""Check the Lake Badge, ranger, shoreline and public roads from St. Cloud to Bemidji."""

import argparse
from collections import Counter
from itertools import product
import json
from pathlib import Path
import runpy
import sys

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE.parents[1] / "tools"))
from audit_story_flow import instructions, native_pages, require
from inland_winter_preview_art import physical
from render_reference import load_pack

walk = runpy.run_path(str(HERE.parent / "floodwatch-v01/check-moorhead.py"))
search = runpy.run_path(str(HERE.parent / "lake-observations-v01/check-hidden.py"))
directions = walk["DIRECTIONS"]
LAKE = "MnLeechLake"
CITY = "MnBrainerd"
GYM = "MnBrainerdGym"
BIRDER = "MN_R07_BIRDERText"
SAND = "MnR07C00R00"
RANGER = "MN_BRAINERD_RANGER"
BADGE = "ENGINE_MINERALBADGE"
REPORT = "EVENT_MN_BRAINERD_REPORT"
GIFT = "EVENT_MN_SURF_GIFT_RECEIVED"
COMPARED = "EVENT_MN_HEADWATERS_REPORT"
PIERS = {LAKE: (range(17, 21), 10), "MnMilleLacs": (range(10, 15), 11)}


def ranger_talk(room, flags, copies, accept=True):
    """Execute the ranger's native report branches against the indexed HM stack."""
    active = flags.copy()
    pages = []
    condition = False
    label = RANGER
    for _ in range(20):
        for op, *args in instructions(room, label):
            if op in ("faceplayer", "opentext", "closetext", "promptbutton", "waitbutton"):
                continue
            if op in ("checkevent", "checkflag"):
                condition = args[0] in active
            elif op == "checkitem":
                require(args == ["HM_SURF"], "ranger is checking the wrong machine")
                condition = copies > 0
            elif op == "giveitem":
                require(args == ["HM_SURF", "1"], "wrong ranger gift")
                condition = accept and copies < 99
                copies += int(condition)
            elif op == "setevent":
                active.add(args[0])
            elif op == "writetext":
                pages.append(args[0])
            elif op in ("iftrue", "iffalse", "sjump"):
                if op == "sjump" or condition == (op == "iftrue"):
                    label = args[0]
                    break
            elif op in ("jumptextfaceplayer", "jumptext", "end"):
                return active, copies, pages + args
            else:
                raise ValueError(f"unexpected ranger opcode {label}/{op}")
        else:
            raise ValueError(f"unterminated ranger branch {label}")
    raise ValueError("cyclic ranger conversation")


def check_receipts(data):
    room = data["maps"][CITY]
    badge, receipt = instructions(data["maps"][GYM], "MnBrainerdGymLeader")[-4:-2]
    require(
        badge == ("setflag", BADGE) and receipt == ("setevent", "EVENT_MN_GYM_6_DEFEATED"), "Lake Badge"
    )
    require(
        ("startbattle",) in instructions(data["maps"][GYM], "MnBrainerdGymLeader")[:-4],
        "badge missing its actual leader battle",
    )
    hm = data["items"]["HM_SURF"]
    move = data["field_moves"]["surf"]
    require(
        hm["pocket"] == "TM_HM"
        and hm["tmhm_index"] == 53
        and hm["tmhm_move"] == move["move_id"] == "SURF"
        and move["badge"] == {"region": "johto", "index": 3},
        "Surf must use its indexed machine slot and the Capitol Badge's field permission",
    )
    for states in product((False, True), repeat=4):
        flags = {k for k, bit in zip((BADGE, REPORT, GIFT, COMPARED), states) if bit}
        for original in (0, 1, 99):
            final, copies, pages = ranger_talk(room, flags, original)
            if GIFT in flags:
                text = (
                    "MnOverhaulRangerComparedText" if COMPARED in flags else "MnBrainerdRangerRepeatText"
                )
                require((final, copies, pages) == (flags, original, [text]), "wrong return reading")
            elif BADGE not in flags:
                require(
                    (final, copies, pages) == (flags, original, ["MnBrainerdRangerBeforeText"]),
                    "early gift",
                )
            else:
                require(
                    final == flags | {GIFT, REPORT} and copies == max(1, original),
                    "lost or duplicated HM",
                )
                require(
                    (RANGER + "Text" in pages) == (REPORT not in flags), "reading skipped or repeated"
                )
                saved, qty, repeat = ranger_talk(room, final, copies)
                require(
                    (saved, qty) == (final, copies) and len(repeat) == 1,
                    "ranger reentry gave another item",
                )
                if not original:
                    held, qty, full = ranger_talk(room, flags, 0, accept=False)
                    require(
                        held == flags | {REPORT} and qty == 0 and full[-1] == "MnBrainerdRangerFullText",
                        "lost deferred reading",
                    )
                    require(
                        ranger_talk(room, held, 0)[:2] == (final, copies), "failed handoff cannot retry"
                    )
    print(
        "All 16 badge/report/gift/comparison states with 0, 1 and 99 Surf copies; failed delivery and retry."
    )


def check_pier(data, trail, roads, parent):
    room = data["maps"][CITY]
    ranger = walk["object_at"](data, CITY, RANGER)
    pos = ranger["x"], ranger["y"]
    expect = (25, 19) if parent else (49, 20)
    require(
        pos == expect and ranger["spritemovedata"] == "SPRITEMOVEDATA_STANDING_DOWN",
        "ranger spot or facing",
    )
    section = [{"command": r["command"], "args": r["args"]} for r in room["map_event_section_commands"]]
    label = room["attributes"]["map_events_label"]
    require(
        data["map_scripts"][label] == room["scripts"].get(label, section) == section,
        "Brainerd raw/typed events",
    )
    require(data["npcs"][CITY] == room["objects"], "Brainerd global ranger/object mirror")
    [row] = [r for r in section if r["command"] == "object_event" and r["args"][11] == RANGER]
    require(tuple(map(int, row["args"][:2])) == pos, "stale ranger event")
    stand = (CITY, pos[0], pos[1] + 1)
    require(stand in roads and trail.kind(CITY, *pos) == "FLOOR", "ranger not reachable on foot")
    for x, y in [(47, 20), (48, 20), (47, 19), (48, 19), (47, 13), (48, 13), (47, 16)]:
        require((CITY, x, y) in roads, f"pier now obstructed at {x},{y}")
    require(
        trail.kind(CITY, pos[0], pos[1] - 1) == ("FLOOR" if parent else "WATER"),
        "ranger must witness the shore after her move",
    )
    for side in ("east", "west", "south", "north"):
        a = room["attributes"]
        w, h = 2 * a["width"], 2 * a["height"]
        edge = {
            "east": [(w - 1, y) for y in range(h)],
            "west": [(0, y) for y in range(h)],
            "north": [(x, 0) for x in range(w)],
            "south": [(x, h - 1) for x in range(w)],
        }[side]
        # Eastern and western neighbors are outside this chapter. Require a real bidirectional seam.
        [link] = [c for c in a["connections"] if c["direction"] == side]
        neighbor = link["target_map"]
        path = walk["Trail"](data, [CITY, neighbor])
        require(
            any(
                (CITY, x, y) in roads
                and (dest := path.step((CITY, x, y), search["SIDES"][side], True))
                and path.step(dest, search["OPPOSITE"][search["SIDES"][side]], True) == (CITY, x, y)
                for x, y in edge
            ),
            f"no access to the ranger from the {side} approach",
        )
    return stand


def journey(data, parent):
    rooms = data["maps"]
    maps = [n for n in rooms if n.startswith(("MnR06C", "MnR07C", "MnR08C"))] + [
        CITY,
        LAKE,
        "MnBemidji",
        "MnStCloud",
    ]
    trail = walk["Trail"](data, maps)
    for name in maps:
        room = rooms[name]
        for callback in room["map_script_section_commands"]:
            if callback["command"] != "callback":
                continue
            label = callback["args"][-1]
            if label.startswith("MnPublicRoads") and label.endswith("Open"):
                hidden = {args[0] for op, *args in instructions(room, label) if op == "disappear"}
                for obj in room["objects"]:
                    if obj["object_identifier"] in hidden:
                        trail.bodies[name].discard((obj["x"], obj["y"]))
    # St. Cloud's courier encounter is complete at this Q04 departure.
    for actor in rooms["MnStCloud"]["objects"]:
        if actor["script"] in ("MN_ARC_SC_REED", "MN_ARC_SC_SERA"):
            trail.bodies["MnStCloud"].discard((actor["x"], actor["y"]))
    start = ("MnStCloud", 8, 28)
    finish = ("MnBemidji", 21, 20)
    out = walk["paths"](trail, start)
    back = walk["paths"](trail, finish)
    shared = out.keys() & back.keys()
    require(start in shared and finish in shared, "broken chapter road")
    stand = check_pier(data, trail, shared, parent)
    for point in [
        (CITY, 12, 36),
        (CITY, 23, 16),
        (CITY, 51, 24),
        (LAKE, 11, 14),
        (LAKE, 9, 36),
        (LAKE, 35, 20),
        stand,
    ]:
        require(point in shared, f"unreachable return at {point}")
    for town, door, room_name, landing in [
        (CITY, (12, 35), GYM, (8, 12)),
        (CITY, (23, 15), "MnBrainerdRangerStation", (8, 9)),
        (LAKE, (11, 13), "MnLeechLakeCareCabin", (3, 5)),
        (LAKE, (9, 35), "MnLeechLakeFisherHouse", (5, 6)),
    ]:
        doors = rooms[town]["events"]["warps"]
        [entry] = [w for w in doors if walk_pos(w) == door]
        expected = rooms[room_name]["attributes"]["map_constant"]
        require(entry["target_map_constant"] == expected, f"{room_name} entrance lost")
        returns = rooms[room_name]["events"]["warps"]
        require(
            any(w["index"] == entry["target_warp_id"] and walk_pos(w) == landing for w in returns),
            f"{room_name} arrival",
        )
        carpets = [
            w for w in returns if search["quadrant"](data, room_name, *walk_pos(w)) == "WARP_CARPET_DOWN"
        ]
        require(
            len(carpets) == 2
            and all(
                any(
                    w["index"] == c["target_warp_id"] and walk_pos(w) == (door[0], door[1] + 1)
                    for w in doors
                )
                for c in carpets
            ),
            f"{room_name} return carpets",
        )
    name = SAND
    require(trail.both((name, 17, 41), (name, 17, 44), True) == (3, 3), "birder's two-way south stair")
    require(
        not any(
            k.startswith("HOP_")
            for n in maps
            if n.startswith("MnR07")
            for row in trail.grid[n]
            for k in row
        ),
        "recheck Route 7 terrain",
    )
    page = " ".join(line for page in native_pages(rooms[name]["scripts"][BIRDER]) for line in page)
    require(("hop" in page.lower()) == parent, "old birder hop instructions")
    if not parent:
        require(
            "southeast" in page and "stone steps" in page and "sandbank" in page, "misdirected birder"
        )
        canon = json.loads((HERE.parents[1] / "story.json").read_text())["dialogue_overhaul"]["texts"][
            name
        ][BIRDER]
        require(
            native_pages(data["map_scripts"][BIRDER])
            == native_pages(rooms[name]["script_text_bodies"][BIRDER]["commands"])
            == canon,
            "birder text mirrors",
        )
    require(
        instructions(rooms["MnLeechLakeCareCabin"], "MN_LEECH_SHORE_NURSE")
        == [("jumpstd", "PokecenterNurseScript")],
        "shore nurse lacks full public care",
    )
    require(
        instructions(rooms["MnLeechLakeFisherHouse"], "MN_LEECH_SHORE_DIARIST")
        == [("jumptextfaceplayer", "MN_LEECH_SHORE_DIARISTText")],
        "fisher testimony",
    )
    print(
        f"St. Cloud Gym to the Bemidji officer on foot and back, {out[finish]}/{back[start]} steps. Ranger reached in {out[stand]}."
    )
    return trail, shared


def walk_pos(row):
    return row["x"], row["y"]


def check_trainers_and_tools(data, trail, shared):
    lake = data["maps"][LAKE]
    foothold = {(31, 27)}  # Mainland promenade, beside the marked lake launch.
    require((LAKE, 31, 27) in shared, "mainland shore")
    modes = {k: search["approaches"](data, LAKE, foothold, k) for k in ("walk", "Surf", "Whirlpool")}
    island = (53, 30)
    for label, pos, first in [
        ("island naturalist", island, "Surf"),
        ("eddy cache", (61, 20), "Whirlpool"),
    ]:
        access = [k for k, v in modes.items() if search["adjacent"](pos) & v]
        require(access and access[0] == first, f"{label} expected {first}, got {access}")
    require(instructions(lake, "MN_LEECH_EDDY_CACHE")[0][0] == "hiddenitem", "eddy cache lost")
    # Opening each optional grove uses the archive's own Cut block replacement.
    cut = {}
    for name, x, y in (("MnR06C00R00", 29, 32), ("MnR08C00R00", 6, 21)):
        room = data["maps"][name]
        attrs = room["attributes"]
        block = room["blocks"][(y // 2) * attrs["width"] + x // 2]
        require(
            trail.kind(name, x, y) == "CUT_TREE"
            and any((name, x + dx, y + dy) in shared for dx, dy in directions.values()),
            f"unusable Cut tree in {name}",
        )
        change = data["field_moves"]["cut"]["replacements"][attrs["tileset_name"]][block][
            "replacement_block_id"
        ]
        cells = data["tilesets"][attrs["tileset_name"]]["collision"][f"{change:02x}"]
        positions = [(2 * (x // 2) + i % 2, 2 * (y // 2) + i // 2) for i in range(4)]
        before = [trail.kind(name, px, py) for px, py in positions]
        for (px, py), value in zip(positions, cells):
            trail.grid[name][py][px] = value
        cut[name] = (
            walk["paths"](trail, ("MnStCloud", 8, 28)).keys()
            & walk["paths"](trail, ("MnBemidji", 21, 20)).keys()
        )
        for (px, py), value in zip(positions, before):
            trail.grid[name][py][px] = value
    sight = Counter()
    for name in trail.maps:
        if not (name.startswith(("MnR06", "MnR07", "MnR08")) or name == LAKE):
            continue
        room = data["maps"][name]
        for o in room["objects"]:
            if o["object_type"] != "OBJECTTYPE_TRAINER":
                continue
            script = o["script"]
            require(
                instructions(room, script)[0][0] == "trainer" and o["radius"] > 0,
                f"{script}: forced talk",
            )
            facing = o["sightline_direction_override"] or o["spritemovedata"].removeprefix(
                "SPRITEMOVEDATA_STANDING_"
            )
            dx, dy = directions[facing.upper()]
            x, y = walk_pos(o)
            tile = name, x + dx, y + dy
            wet = name == LAKE and walk_pos(o) == island
            if wet:
                require(tile[1:] in modes["Surf"], f"unreachable island sight square {script}")
                method = "Surf"
            elif tile in shared:
                method = "dry"
            else:
                require(tile in cut.get(name, set()), f"unreachable sight square {script} {tile}")
                method = "Cut"
            sight[method] += 1
    print(
        f"Trainer approaches {dict(sight)}. Mainland, Surf-only island and Whirlpool-only cache retained."
    )


def compare(old, new):
    before, after = old["maps"][CITY], new["maps"][CITY]
    others = lambda room: [o for o in room["objects"] if o["script"] != RANGER]
    require(others(before) == others(after), "another Brainerd actor moved")
    old_actor = dict(walk["object_at"](old, CITY, RANGER), x=49, y=20)
    require(old_actor == walk["object_at"](new, CITY, RANGER), "ranger identity changed")
    label = before["attributes"]["map_events_label"]
    for k in ("blocks", "events", "script_item_grants", "attributes"):
        require(before[k] == after[k], f"unrelated Brainerd {k}")
    require(
        {k: v for k, v in before["scripts"].items() if k != label}
        == {k: v for k, v in after["scripts"].items() if k != label},
        "ranger programs changed",
    )
    for name, (cols, y) in PIERS.items():
        orig, final = old["maps"][name], new["maps"][name]
        require(
            {k: v for k, v in orig.items() if k != "blocks"}
            == {k: v for k, v in final.items() if k != "blocks"},
            f"{name} changed beyond the dock",
        )
        width = orig["attributes"]["width"]
        donor_map = new["maps"]["MnR18C00R00"]
        donor = donor_map["blocks"][11 * donor_map["attributes"]["width"] + 8]
        expected = orig["blocks"].copy()
        for x in cols:
            expected[y * width + x] = donor
        require(final["blocks"] == expected, f"{name} changed outside the pier")
    require(old["maps"]["MnR07C00R01"] == new["maps"]["MnR07C00R01"], "the live Route 7 sapling changed")
    for name in (GYM, "MnBrainerdRangerStation", "MnLeechLakeFisherHouse", "MnLeechLakeCareCabin"):
        require(old["maps"][name] == new["maps"][name], f"unrelated {name} change")


def check_timber(pack, parent):
    data = pack["data"]
    bank = "mn_headwaters"
    filename = f"data/tilesets/{bank}_metatiles.bin"
    layout = bytes(pack["runtime_files"][filename])
    local = data["maps"]["MnR18C00R00"]
    donor = local["blocks"][11 * local["attributes"]["width"] + 8]
    require(
        layout[donor * 16 : (donor + 1) * 16] == bytes.fromhex("9f9f9f9f9d9e9d9ec3c3c3c39f9f9f9f"),
        "actual open-timber Route 18 reference",
    )
    bank_data = data["tilesets"][bank]
    require(bank_data["collision"][f"{donor:02x}"] == ["FLOOR"] * 4, "native timber permissions")

    def glyph(tileset, tile):
        sheet = pack["runtime_files"][f"gfx/tilesets/{tileset}.2bpp"]
        palette = data["tilesets"][tileset]["palette_map"][tile]
        offset = 16 * physical(tile, palette, len(sheet) // 16)
        return palette & 7, bytes(sheet[offset : offset + 16])

    for local_id, original in ((0x9F, 0x64), (0x9D, 0x60), (0x9E, 0x61), (0xC3, 0x6E)):
        require(
            glyph(bank, local_id) == glyph("kanto", original), f"incorrect timber ink {local_id:02x}"
        )
    for name, (cols, y) in PIERS.items():
        room = data["maps"][name]
        require(room["attributes"]["tileset_name"] == bank, f"{name} pier bank")
        blocks = {room["blocks"][y * room["attributes"]["width"] + x] for x in cols}
        if parent and donor != 0x88:
            require(
                blocks == {0x88}
                and bank_data["collision"]["88"] == ["FLOOR", "FLOOR", "HEADBUTT_TREE", "FLOOR"],
                "unexpected Field legacy pier",
            )
            continue
        require(blocks == {donor}, f"{name} landing contains something other than the native deck")
        if name == LAKE:
            road, stand, side, depth = (32, 47), (35, 20), (38, 21), (38, 22)
        else:
            road, stand, side, depth = (0, 12), (27, 22), (29, 23), (29, 24)
        trail = walk["Trail"](data, [name])
        for spot in (stand, side):
            trail.both((name, *road), (name, *spot), True)
        require(trail.kind(name, *depth) == "WATER", f"{name} fishing contact is dry")
    sapling = data["maps"]["MnR07C00R01"]
    block = sapling["blocks"][12 * sapling["attributes"]["width"] + 12]
    require(
        bank_data["collision"][f"{block:02x}"] == ["FLOOR", "FLOOR", "HEADBUTT_TREE", "FLOOR"],
        "authentic Route 7 sapling",
    )
    print(
        f"Checked native Route 18 deck {donor:02x}, both lake piers, and the separate live Route 7 sapling."
    )


def main():
    cli = argparse.ArgumentParser(description=__doc__)
    cli.add_argument("pack", type=Path)
    cli.add_argument("--baseline", action="store_true", help="check the original Station Return edition")
    cli.add_argument("--parent", type=Path, help="compare with this edition's Station Return parent")
    args = cli.parse_args()
    pack = load_pack(args.pack)
    data = pack["data"]
    check_timber(pack, args.baseline)
    del pack
    check_receipts(data)
    trail, shared = journey(data, args.baseline)
    check_trainers_and_tools(data, trail, shared)
    if args.parent:
        compare(load_pack(args.parent)["data"], data)


if __name__ == "__main__":
    main()
