#!/usr/bin/env python3
"""Compare a written Arcade Frontage edition to its matching Wind Lesson archive."""
import argparse
import base64
from collections import Counter
import hashlib
import json
from pathlib import Path

import cbor2

HERE = Path(__file__).resolve().parent
REGION = HERE.parents[1]
CITY, CEDAR = "MnMinneapolis", "MnR08C00R01"
ROOM, DESK, INN = "MnMinneapolisCornerHome", "MnMinneapolisArrivalInfo", "MnMinneapolisRiverInn"
LABELS = {"MnCornerWelcomeText", "MnCornerRulesText", "MnCornerCoinsBoughtText", "MnCornerNeedCase",
          "MnCornerItemsText", "MnMinneapolisCornerHomePatronText",
          *(f"MnCorner{counter}Confirm{i}" for counter in ("Items", "Tms") for i in range(3))}
PRIZES = [("QUICK_CLAW", 300), ("RARE_CANDY", 500), ("PP_UP", 1000),
          ("TM_THUNDER", 5500), ("TM_BLIZZARD", 5500), ("TM_FIRE_BLAST", 5500)]


def changed(a, b):
    return {k for k in a.keys() | b.keys() if k not in a or k not in b or a[k] != b[k]}


def only(old, new, expected, name):
    actual = changed(old, new)
    assert actual == expected, (name, "missing", expected - actual, "unexpected", actual - expected)


def read_pack(path, allowed=None):
    raw = path.read_bytes()
    if allowed is not None:
        assert hashlib.sha256(raw).hexdigest() in allowed, f"unexpected Wind Lesson parent {path}"
    assert raw[:12] == b"CRYSTALPACK\0" and len(raw) >= 22, path
    payload = memoryview(raw)[22:]
    assert len(payload) == int.from_bytes(raw[14:18], "big"), path
    checksum = 2166136261
    for byte in payload:
        checksum = ((checksum ^ byte) * 16777619) & 0xffffffff
    assert checksum == int.from_bytes(raw[18:22], "big"), path
    pack = cbor2.loads(payload, allow_duplicate_keys=False)
    assert pack["format_version"] == int.from_bytes(raw[12:14], "big"), path
    return pack


def program(room, label):
    return [(row["command"], *row["args"]) for row in room["scripts"][label]]


def indexed(commands):
    return [dict(row, command_index=i) for i, row in enumerate(commands)]


def check_pages(old, room, labels):
    name = room["id"]
    only(old["script_text_bodies"], room["script_text_bodies"], labels, name + " text")
    story = json.loads((REGION / "story.json").read_text())["dialogue_overhaul"]["texts"][name]
    for label in labels:
        expected = []
        for page_no, page in enumerate(story[label]):
            assert 1 <= len(page) <= 2
            for line_no, line in enumerate(page):
                command = "line" if line_no else ("para" if page_no else "text")
                expected.append((command, json.dumps(line, ensure_ascii=False)))
        assert program(room, label) == expected + [("done",)], label
        assert room["script_text_bodies"][label] == {
            "label": label, "commands": indexed(room["scripts"][label])}, label


def check_menu(room, source, index, header, data, options):
    menu = room["script_vertical_menus"][f"{source}:{index}"]
    assert menu == dict(source_script=source, loadmenu_command_index=index-1,
                        verticalmenu_command_index=index, header_label=header,
                        data_label=data, options=options, two_dimensional=False,
                        rows=None, columns=None, spacing=None)
    assert program(room, source)[index-1:index+2] == [
        ("loadmenu", header), ("verticalmenu",), ("closewindow",)]
    assert program(room, data) == [("db", "STATICMENU_CURSOR"), ("db", str(len(options))),
                                   *(("db", json.dumps(option + "@")) for option in options)]
    for label in (header, data):
        assert room["script_menu_definitions"][label] == {
            "label": label, "commands": indexed(room["scripts"][label])}
    assert ("dw", data) in program(room, header)


def check_terrain(old, new, name, expected):
    a, b = old["maps"][name], new["maps"][name]
    only(a, b, {"blocks", "events", "map_event_section_commands"}, name)
    width = a["attributes"]["width"]
    assert len(a["blocks"]) == len(b["blocks"])
    edits = {(i % width, i // width): (left, right)
             for i, (left, right) in enumerate(zip(a["blocks"], b["blocks"])) if left != right}
    assert edits == expected, (name, edits)
    only(a["events"], b["events"], {"bg_events"}, name)
    tileset = a["attributes"]["tileset_name"]
    terrain_deltas = set()
    for i, (left, right) in enumerate(zip(a["blocks"], b["blocks"])):
        ca = old["tilesets"][tileset]["collision"][f"{left:02x}"]
        cb = new["tilesets"][tileset]["collision"][f"{right:02x}"]
        for q, (before, after) in enumerate(zip(ca, cb)):
            if before != after:
                terrain_deltas.add(((i % width)*2 + q%2, (i//width)*2 + q//2, before, after))
    assert terrain_deltas == ({(32, 4, "FLOOR", "WALL")} if name == CEDAR else set()), (name, terrain_deltas)


def check_counter(data, oldroom):
    room = data["maps"][ROOM]
    only(oldroom, room, {"scripts", "script_text_bodies", "script_item_grants",
                        "script_vertical_menus", "script_menu_definitions"}, ROOM)
    menus = {"MnCornerItems": ["QUICK CLAW 300", "RARE CANDY 500", "PP UP 1000", "CANCEL"],
             "MnCornerTms": ["THUNDER 5500", "BLIZZARD 5500", "FIRE BLAST 5500", "CANCEL"]}
    data_labels = {source + "Data" for source in menus}
    only(oldroom["scripts"], room["scripts"], LABELS | data_labels | {"MnCornerItemsPrize0"}, "arcade programs")
    only(oldroom["script_menu_definitions"], room["script_menu_definitions"], data_labels, "prize menu data")
    only(oldroom["script_vertical_menus"], room["script_vertical_menus"], {source + ":7" for source in menus}, "prize menus")
    for source, options in menus.items():
        check_menu(room, source, 7, source + "Menu", source + "Data", options)
    check_pages(oldroom, room, LABELS)
    grants = [dict(row, item_id="QUICK_CLAW") if row["source_script"] == "MnCornerItemsPrize0"
              and row["command_index"] == 5 and row["item_id"] == "MOON_STONE" else row
              for row in oldroom["script_item_grants"]]
    assert grants == room["script_item_grants"]
    games = Counter(event["script"] for event in room["events"]["bg_events"])
    assert games == {"MnCornerSlots": 18, "MnCornerCards": 12}
    for i, (item, price) in enumerate(PRIZES):
        counter = "MnCornerItems" if i < 3 else "MnCornerTms"
        label = f"{counter}Prize{i%3}"
        confirm = f"{counter}Confirm{i%3}"
        page = program(room, confirm)
        assert [row[0] for row in page] == ["text", "line", "done"], confirm
        visible = " ".join(json.loads(row[1]) for row in page[:2])
        name = item.replace("_", " ") if i < 3 else ("TM25 THUNDER", "TM14 BLIZZARD", "TM38 FIRE BLAST")[i-3]
        assert name in visible and f"{price} coins?" in visible, (confirm, visible)
        rows = [(c["command"], *c["args"]) for c in room["scripts"][label]]
        assert rows == [("checkcoins", str(price)), ("ifequal", "HAVE_LESS", "MnCornerNotEnough"),
                        ("writetext", confirm), ("yesorno",),
                        ("iffalse", "MnCornerClose"), ("giveitem", item, "1"),
                        ("iffalse", "MnCornerNoRoom"), ("takecoins", str(price)), ("sjump", "MnCornerReceived")]
    source = REGION / "pokemon"
    families = json.loads((source / "roster.json").read_text())["families"]
    retired = {f["id"] for f in json.loads((source / "roster-curation.json").read_text())["retired_families"]}
    roster = {m["name"].upper() for f in families if f["id"] not in retired for m in f["members"]}
    for (item, _), number in zip(PRIZES[3:], (22, 23, 31)):
        move = data["items"][item]["tmhm_move"]
        assert sum(move in data["pokemon"][p]["tmhm_learnset"] for p in roster) == number
    assert data["items"]["QUICK_CLAW"]["held_effect"] == "HELD_QUICK_CLAW"
    assert data["items"]["QUICK_CLAW"]["parameter"] == 60


def check_interaction_indexes(old, room, actors):
    tables = {"script_text_commands": {"opentext", "writetext", "yesorno", "waitbutton", "closetext"},
              "script_control_commands": {"ifequal", "iffalse", "sjump", "end"},
              "script_runtime_commands": {"loadmenu", "verticalmenu", "closewindow", "special"},
              "script_object_commands": {"faceplayer"}}
    for table, ops in tables.items():
        assert [r for r in old[table] if r["source_script"] not in actors] == [
            r for r in room[table] if r["source_script"] not in actors], table
        expected = {(source, i, op): args for source in actors
                    for i, (op, *args) in enumerate(program(room, source)) if op in ops}
        rows = [r for r in room[table] if r["source_script"] in actors]
        assert len(rows) == len(expected), (room["id"], table, list(expected), rows)
        for row in rows:
            op = row["command"]
            args = expected.pop((row["source_script"], row["command_index"], op))
            if table == "script_runtime_commands":
                assert row["args"] == args
            if op == "writetext":
                assert row["text_label"] == args[0]
            if op in {"ifequal", "iffalse", "sjump"}:
                assert row["target_label"] == row["resolved_target_script"] == args[-1]
                assert args[-1] in room["scripts"]
            if op == "ifequal":
                assert row["compare_value"] == args[0]


def check_directory(old, room):
    host, header, menu = "MN_MP_ARRIVALINFO_HOST", "MnSouthDeskHeader", "MnSouthDeskMenu"
    destinations = json.loads((HERE / "arrival-directory.json").read_text())
    options = [d["choice"] for d in destinations] + ["CANCEL"]
    assert options == ["CLINIC", "GAME CORNER", "SOUTH MARKET", "CANCEL"]
    branches = [d["label"] for d in destinations]
    programs = {host, *branches}
    pages = {label + "Text" for label in programs}
    only(old, room, {"scripts", "script_menu_definitions", "script_vertical_menus", "script_control_commands",
                     "script_object_commands", "script_runtime_commands", "script_text_commands", "script_text_bodies"}, DESK)
    only(old["scripts"], room["scripts"], programs | pages | {header, menu}, "directory programs")
    only(old["script_menu_definitions"], room["script_menu_definitions"], {menu, header}, "directory menu data")
    only(old["script_vertical_menus"], room["script_vertical_menus"], {host + ":4"}, "directory menus")
    assert program(room, host) == [
        ("faceplayer",), ("opentext",), ("writetext", host + "Text"), ("loadmenu", header),
        ("verticalmenu",), ("closewindow",),
        *(("ifequal", str(i), label) for i, label in enumerate(branches, 1)),
        ("closetext",), ("end",)]
    for branch in branches:
        assert program(room, branch) == [("writetext", branch + "Text"), ("waitbutton",), ("closetext",), ("end",)]
    check_menu(room, host, 4, header, menu, options)
    check_pages(old, room, pages)
    check_interaction_indexes(old, room, programs)


def block_art(pack, tileset):
    """Resolve each metatile to its sixteen native (palette, two-bit pixels) glyphs."""
    files = pack["runtime_files"]
    palette = pack["data"]["tilesets"][tileset]["palette_map"]
    ink = bytes(files[f"gfx/tilesets/{tileset}.2bpp"])
    size = len(ink) // 16
    def glyph(tile):
        attribute = palette[tile]
        low = tile & 127
        banked = [low + size//2, low + 128] if attribute & 8 else []
        candidates = banked + [tile, low, 0]
        physical = next(i for i in candidates if i < size)
        return attribute & 7, ink[physical*16:(physical+1)*16]
    raw = files[f"data/tilesets/{tileset}_metatiles.bin"]
    assert len(raw) % 16 == 0
    return [tuple(glyph(tile) for tile in raw[i:i+16]) for i in range(0, len(raw), 16)]


def check_inn(parent, child):
    a, b = parent["data"], child["data"]
    old, room = a["maps"][INN], b["maps"][INN]
    base, private = "traditional_house", "mn_mill_inn"
    only(old, room, {"scripts", "script_object_commands", "script_control_commands", "script_text_commands",
                     "script_text_bodies", "script_runtime_commands", "attributes", "blocks", "events",
                     "map_event_section_commands"}, INN)
    assert private not in a["tilesets"]
    assert old["attributes"]["tileset_name"] == base
    assert room["attributes"] == dict(old["attributes"], tileset_name=private) == b["map_attributes"][INN]
    assert {name for name, m in b["maps"].items() if m["attributes"]["tileset_name"] == private} == {INN}
    native = block_art(parent, base)
    donor = block_art(parent, "players_house")[0x22]
    bed = len(native)
    wall_bed = tuple(glyph for row in range(0, 16, 4)
                     for glyph in (native[0x04][row:row+2] + donor[row:row+2]))
    assert block_art(child, private) == native + [wall_bed], "inn native bed, floor and other tile pixels"
    original = a["tilesets"][base]["collision"]
    private_tiles = b["tilesets"][private]
    assert private_tiles["collision"] == original | {f"{bed:02x}": ["FLOOR", "WALL", "FLOOR", "WALL"]}
    for key, suffix in [("collision", ""), ("palette_map", "_palette_map")]:
        raw = child["runtime_files"][f"data/tilesets/{private}{suffix}.json"]
        assert json.loads(bytes(raw)) == private_tiles[key], (private, key)
    assert old["blocks"][7] == 4 and room["blocks"] == old["blocks"][:7] + [bed] + old["blocks"][8:]
    host, rest, leave, rested = "MN_MP_RIVERINN_HOST", "MnMillInnOfferRest", "MnMillInnLeave", "MnMillInnRestedText"
    assert not old["events"]["bg_events"]
    assert room["events"] == dict(old["events"], bg_events=[
        dict(x=7, y=y, event_type="BGEVENT_READ", script=rest) for y in (2, 3)])
    actors, pages = {host, rest, leave}, {host + "Text", rested}
    only(old["scripts"], room["scripts"], actors | pages, "inn programs")
    assert program(room, host) == [("faceplayer",), ("sjump", rest)]
    assert program(room, rest) == [("opentext",), ("writetext", host + "Text"), ("yesorno",), ("iffalse", leave),
                                   ("special", "HealParty"), ("writetext", rested), ("waitbutton",), ("closetext",), ("end",)]
    assert program(room, leave) == [("closetext",), ("end",)]
    check_pages(old, room, pages)
    check_interaction_indexes(old, room, actors)


def verify(parent, child):
    only(parent, child, {"identity", "data", "report", "runtime_files"}, "pack")
    only(parent["report"], child["report"], {"manifests"}, "report")
    manifests = parent["report"]["manifests"] + ["mn_mill_inn-native-v1", "northstar-arcade-frontage-v1"]
    assert child["report"]["manifests"] == manifests
    assert child["identity"]["runtime_modpack_id"] == "+".join(manifests)
    only(parent["identity"], child["identity"], {"content_hash", "runtime_modpack_id"}, "identity")
    a, b = parent["data"], child["data"]
    only(a, b, {"maps", "map_attributes", "map_scripts", "map_blocks", "tilesets"}, "region")
    only(a["map_attributes"], b["map_attributes"], {INN}, "regional map attributes")
    only(a["maps"], b["maps"], {CITY, CEDAR, ROOM, DESK, INN}, "affected maps")
    only(a["tilesets"], b["tilesets"], {"mn_minneapolis_city", "mn_headwaters_thaw", "mn_mill_inn"}, "tilesets")
    only(parent["runtime_files"], child["runtime_files"], {
        "data/tilesets/mn_headwaters_thaw.json", "data/tilesets/mn_headwaters_thaw_metatiles.bin",
        "data/tilesets/mn_minneapolis_city.json", "data/tilesets/mn_minneapolis_city_metatiles.bin",
        "data/tilesets/mn_minneapolis_city_palette_map.json", "gfx/tilesets/mn_minneapolis_city.2bpp",
        "gfx/tilesets/mn_minneapolis_city.png", "data/tilesets/mn_mill_inn.json",
        "data/tilesets/mn_mill_inn_metatiles.bin", "data/tilesets/mn_mill_inn_palette_map.json",
        "gfx/tilesets/mn_mill_inn.2bpp", "gfx/tilesets/mn_mill_inn.png"}, "authored tiles")
    for tileset in ("mn_minneapolis_city", "mn_headwaters_thaw"):
        original = block_art(parent, tileset)
        revised = block_art(child, tileset)
        assert len(revised) == len(original) + 1 and revised[:-1] == original, tileset
        only(a["tilesets"][tileset]["collision"], b["tilesets"][tileset]["collision"], {"b9"}, tileset)
        if tileset == "mn_headwaters_thaw":
            assert revised[-1] == tuple(original[0x45 if i in (0, 1, 4, 5) else 0x86][i] for i in range(16))
        collision_file = child["runtime_files"][f"data/tilesets/{tileset}.json"]
        assert json.loads(bytes(collision_file)) == b["tilesets"][tileset]["collision"]
    check_terrain(a, b, CITY, {(26, 24): (5,128), (27,24): (128,130),
                               (26,25): (5,131), (27,25): (184,185)})
    check_terrain(a, b, CEDAR, {(11,3): (184,138), (16,2): (134,185)})
    for name, label, coords in [(CITY, "MnCornerSign", [(55,51), (54,51)]),
                                (CEDAR, "MN_R08_SIGN_02", [(32,4)])]:
        old_events, new_events = [d["maps"][name]["events"]["bg_events"] for d in (a,b)]
        assert [e for e in old_events if e["script"] != label] == [e for e in new_events if e["script"] != label]
        assert [e for e in new_events if e["script"] == label] == [
            dict(x=x, y=y, event_type="BGEVENT_READ", script=label) for x,y in coords]
    check_counter(b, a["maps"][ROOM])
    check_inn(parent, child)
    check_directory(a["maps"][DESK], b["maps"][DESK])
    scripts = set()
    for name in (CITY, CEDAR, ROOM, DESK, INN):
        old, new = a["maps"][name], b["maps"][name]
        assert old["events"]["warps"] == new["events"]["warps"] and old["objects"] == new["objects"]
        assert bytes(new["blocks"]) == base64.b64decode(b["map_blocks"][new["attributes"]["blocks_label"]])
        event_label = new["attributes"]["map_events_label"]
        commands = b["map_scripts"][event_label]
        assert new["map_event_section_commands"] == indexed(commands)
        assert [r for r in commands if r["command"] != "bg_event"] == [
            r for r in a["map_scripts"][event_label] if r["command"] != "bg_event"]
        assert [r["args"] for r in commands if r["command"] == "bg_event"] == [
            [str(e["x"]), str(e["y"]), e["event_type"], e["script"]] for e in new["events"]["bg_events"]]
        for label in changed(old["scripts"], new["scripts"]):
            scripts.add(label)
            assert b["map_scripts"][label] == new["scripts"][label]
        if old["map_event_section_commands"] != new["map_event_section_commands"]:
            scripts.add(new["attributes"]["map_events_label"])
    only(a["map_scripts"], b["map_scripts"], scripts, "regional script index")
    block_labels = {b["maps"][name]["attributes"]["blocks_label"] for name in (CITY, CEDAR, INN)
                    if a["maps"][name]["blocks"] != b["maps"][name]["blocks"]}
    only(a["map_blocks"], b["map_blocks"], block_labels, "regional block index")


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("parent", type=Path)
    parser.add_argument("child", type=Path)
    args = parser.parse_args()
    allowed = {json.loads((HERE.parent / "wind-lesson-v01" / f"{edition}-build.json").read_text())["expected_pack_sha256"]
               for edition in ("intro", "field")}
    verify(read_pack(args.parent, allowed), read_pack(args.child))
    print(f"{args.child.name}: arcade prizes and machines, Cedar trail board, arrival directory, native inn bed and healing, "
          "all other maps and archive content passed")
