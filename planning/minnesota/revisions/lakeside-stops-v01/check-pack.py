#!/usr/bin/env python3
"""Check a written Lakeside Stops archive against its matching Arcade Frontage parent."""
import argparse
import base64
from collections import deque
import hashlib
import json
from pathlib import Path
import runpy

HERE = Path(__file__).resolve().parent
# These checks decode the native pack, two-bit tile graphics and canonical story pages.
common = runpy.run_path(str(HERE.parent / "arcade-frontage-v01" / "check-pack.py"))
read_pack, only, changed, indexed, program, check_pages, block_art = (
    common[key] for key in ("read_pack", "only", "changed", "indexed", "program", "check_pages", "block_art")
)

FLOORS = [f"MnMinneapolisDepartmentStore{n}F" for n in range(1, 6)]
SHOPS = {FLOORS[0], *("MnDuluth" + s for s in ("HillSupplies", "LakeOutfitter", "ToolExchange",
         "MaritimeCoop1F", "DockSupplies", "PointOutfitter")),
         *("MnMinneapolis" + s for s in ("SupplyCoop", "GardenShop", "SouthMarket")),
         *("MnStPaul" + s for s in ("FairFoodHall", "FairGardenStand", "FairBerrySnack", "FairFestivalMail",
                                    "WestSeventhBakery", "LowertownExchange", "LowertownCrafts",
                                    "LowertownCafe", "RiversideSupplies"))}
CAB, CAB_CONST = "MnMinneapolisStoreElevator", "MN_MINNEAPOLIS_STORE_ELEVATOR"
LIB, PIE, DEPOT, HALL = "MnStPaulCapitolLibrary", "MnNorthShorePieShop", "MnStPaulUnionDepot", "MnStPaulCapitolHall"
AFFECTED = SHOPS | set(FLOORS) | {LIB, PIE, DEPOT, HALL, CAB}
BANKS = {LIB: ("house", "mn_capitol_library"), PIE: ("game_corner", "mn_shore_bakery")}
ITEM, PANEL, TABLE = "MN_BLUEBERRY_SLICE", "MnStoreElevatorPanel", "MnStoreElevatorFloors"
EVENTS = {"events", "map_event_section_commands"}
PAGES = {"scripts", "script_text_bodies", "script_text_commands"}
DIRECTIONS = ((1, 0), (0, 1), (-1, 0), (0, -1))


def xy(event):
    return event["x"], event["y"]


def reading(x, y, script):
    return dict(x=x, y=y, event_type="BGEVENT_READ", script=script)


def warp(index, x, y, target, arrival):
    return dict(index=index, x=x, y=y, target_map_constant=target, target_map=target, target_warp_id=arrival)


def terrain(data, room, point):
    x, y = point
    attr = room["attributes"]
    if not (0 <= x < attr["width"] * 2 and 0 <= y < attr["height"] * 2):
        return None
    block = room["blocks"][y // 2 * attr["width"] + x // 2]
    return data["tilesets"][attr["tileset_name"]]["collision"][f"{block:02x}"][y % 2 * 2 + x % 2]


def accessible(data, room, start):
    # Normal floor can carry a return anchor. Stairs and exit carpets end a walk.
    solid = {xy(o) for o in room["objects"]}
    assert start not in solid, (room["id"], start)
    reached, todo = {start}, deque([start])
    while todo:
        x, y = todo.popleft()
        for dx, dy in DIRECTIONS:
            point = x + dx, y + dy
            if point in solid or point in reached:
                continue
            if terrain(data, room, point) in {"FLOOR", "BRIDGE_NS_DECK", "BRIDGE_EW_DECK"}:
                reached.add(point)
                todo.append(point)
    return reached


def can_talk(data, room, reached, actor):
    x, y = xy(actor)
    for dx, dy in DIRECTIONS:
        at = x + dx, y + dy
        if at in reached or (terrain(data, room, at) == "COUNTER" and (x + 2*dx, y + 2*dy) in reached):
            return True
    return False


def check_blocks(old, room, index, before, after):
    assert old["blocks"][index] == before and room["blocks"] == (
        old["blocks"][:index] + [after] + old["blocks"][index+1:]), room["id"]


def check_lift_and_shops(a, b):
    lookup = {r["attributes"]["map_constant"]: r for r in b["maps"].values()}
    cab = b["maps"][CAB]
    goldenrod = a["maps"]["GoldenrodDeptStoreElevator"]
    assert cab["blocks"] == goldenrod["blocks"] and cab["attributes"]["tileset_name"] == goldenrod["attributes"]["tileset_name"]
    first = b["maps"][FLOORS[0]]["attributes"]["map_constant"]
    assert cab["events"] == dict(warps=[warp(i, i, 3, first, -1) for i in (1, 2)],
                                 bg_events=[reading(3, 0, PANEL)], coord_events=[])
    assert program(cab, PANEL) == [("opentext",), ("elevator", TABLE), ("closetext",), ("iffalse", "MnStoreLiftDone"),
                                    ("pause", "5"), ("playsound", "SFX_ELEVATOR"), ("earthquake", "60"), ("waitsfx",), ("end",)]
    assert program(cab, "MnStoreLiftDone") == [("end",)]
    assert set(cab["scripts"]) == {PANEL, TABLE, "MnStoreLiftDone"}
    assert {(1, 2), (2, 2), (3, 1)} <= accessible(b, cab, (1, 2))
    expected_rows, expected_menu = [("db", "5")], []
    for name in sorted(SHOPS | set(FLOORS)):
        old, room = a["maps"][name], b["maps"][name]
        floor = FLOORS.index(name) + 1 if name in FLOORS else 0
        only(old, room, EVENTS | (PAGES | {"blocks"} if floor else set()), name)
        warps = old["events"]["warps"].copy()
        plates = []
        if name in SHOPS:
            left, right, previous = warps[:3]
            x = left["x"] if name == FLOORS[0] else previous["x"]
            start = x, left["y"] - 1
            assert (x in (left["x"], right["x"]) and xy(right) == (left["x"]+1, left["y"])
                    and terrain(b, room, start) == "FLOOR")
            warps[2] = dict(previous, x=x, y=start[1])
            assert warps[2]["target_warp_id"] == warps[2]["index"] == 3
            assert warps[2]["target_map_constant"] == room["attributes"]["map_constant"]
            street = lookup[left["target_map_constant"]]
            doors = [w for w in street["events"]["warps"] if w["target_map_constant"] == room["attributes"]["map_constant"]
                     and w["target_warp_id"] == 3 and terrain(b, street, xy(w)) == "DOOR"]
            assert len(doors) == 1, name
            for carpet in (left, right):
                assert terrain(b, room, xy(carpet)) == "WARP_CARPET_DOWN"
                stoop = street["events"]["warps"][carpet["target_warp_id"]-1]
                assert xy(stoop) == (doors[0]["x"], doors[0]["y"]+1) and terrain(b, street, xy(stoop)) == "FLOOR", name
                assert (stoop["x"], stoop["y"]+1) in accessible(b, street, xy(stoop)), name
                assert (carpet["x"], carpet["y"]-1) in accessible(b, room, start)
        else:
            start = (2, 1)
        reached = accessible(b, room, start)
        assert room["script_shop_commands"] == old["script_shop_commands"]
        if name == "MnStPaulLowertownExchange":
            reception, = [o for o in room["objects"] if o["script"] == "MN_SPX_EXCHANGE_RECEPTION"]
            assert can_talk(b, room, reached, reception)
        else:
            assert room["script_shop_commands"], name
        for shop in room["script_shop_commands"]:
            source = shop["source_script"]
            assert program(room, source)[shop["command_index"]] == ("pokemart", shop["mart_type"], shop["mart_id"])
            assert b["marts"][shop["mart_id"]] == a["marts"][shop["mart_id"]] and b["marts"][shop["mart_id"]]
            actor_script = source
            if name == "MnDuluthLakeOutfitter":
                actor_script += "FieldHM"
                done = actor_script + "Done"
                assert ("iftrue", done) in program(room, actor_script)
                assert program(room, done) == [("sjump", source)]
            actor, = [o for o in room["objects"] if o["script"] == actor_script]
            assert can_talk(b, room, reached, actor), (name, source)
        if floor:
            if floor == 5:
                expected = old["blocks"].copy()
                for i, before, after in ((1, 3, 5), (3, 3, 0x14), (4, 3, 0x14),
                                         (11, 0x2b, 4), (12, 0x2b, 4), (27, 0x1c, 0x40), (28, 0x1d, 0x41)):
                    assert expected[i] == before
                    expected[i] = after
                assert room["blocks"] == expected
            else:
                check_blocks(old, room, 1, 3, 5)
            assert terrain(b, room, (2, 0)) == "STAIRCASE" and (2, 1) in reached
            for stair in old["events"]["warps"]:
                if terrain(b, room, xy(stair)) == "STAIRCASE":
                    assert (stair["x"], stair["y"]+1) in reached
            idx = len(warps) + 1
            assert idx == (5, 4, 4, 4, 3)[floor-1]
            warps.append(warp(idx, 2, 0, CAB_CONST, 1))
            sign = f"MnStoreFloor{floor}Sign"
            plates = [reading(x, 0, sign) for x in ((3, 14) if floor < 5 else (3,))]
            assert all((e["x"], e["y"]+1) in reached for e in plates)
            assert program(room, sign) == [("jumptext", sign+"Text")]
            check_pages(old, room, {sign+"Text"})
            only(old["scripts"], room["scripts"], {sign, sign+"Text"}, name)
            const = room["attributes"]["map_constant"]
            expected_rows.append(("elevfloor", f"FLOOR_{floor}F", str(idx), const))
            expected_menu.append(dict(floor=f"FLOOR_{floor}F", warp=idx, target_map=name, source_script=TABLE, command_index=floor))
        assert room["events"] == dict(old["events"], warps=warps, bg_events=plates)
    assert program(cab, TABLE) == expected_rows + [("db", "-1")]
    assert cab["script_elevators"] == {PANEL+":1": dict(source_script=PANEL, elevator_command_index=1,
                                                        data_label=TABLE, floors=expected_menu)}


def check_tm_counter(data):
    roster = HERE.parent.parent / "pokemon"
    retired = {f["id"] for f in json.loads((roster / "roster-curation.json").read_text())["retired_families"]}
    families = json.loads((roster / "roster.json").read_text())["families"]
    partners = [data["pokemon"][m["name"].upper()] for f in families if f["id"] not in retired for m in f["members"]]
    offers = data["marts"]["MART_MN_MP_DEPT_5_CLERK"]
    assert len(offers) == len(set(offers)) == 13
    for item_id in offers:
        item = data["items"][item_id]
        move = item["tmhm_move"]
        assert item["price"] > 0 and move in data["moves"], item_id
        assert any(move in mon["tmhm_learnset"] for mon in partners), f"{item_id} has no eligible partner"


def check_services(parent, child):
    a, b = parent["data"], child["data"]
    # The two new MART blocks reuse glyphs from the native counter caps. Crystal's
    # base MART bank stores collision in the catalog, with only the tilemap split out.
    private_files = {"data/tilesets/mart_metatiles.bin"}
    native = block_art(parent, "mart")
    caps = []
    for donor, column, source in ((0x0c, 0, 2), (0x0e, 2, 0)):
        cap = list(native[0x0d])
        for row in (0, 4):
            cap[row+column:row+column+2] = native[donor][row+source:row+source+2]
        caps.append(tuple(cap))
    assert block_art(child, "mart") == native + caps
    original_mart = a["tilesets"]["mart"]
    footing = original_mart["collision"]["0d"]
    assert b["tilesets"]["mart"] == dict(original_mart, collision=original_mart["collision"] | {"40": footing, "41": footing})
    assert all(block < len(native) for room in b["maps"].values() if room["attributes"]["tileset_name"] == "mart"
               and room["id"] != FLOORS[-1] for block in room["blocks"])
    for name, (base, bank) in BANKS.items():
        old, room = a["maps"][name], b["maps"][name]
        assert room["attributes"] == dict(old["attributes"], tileset_name=bank)
        original, revised = block_art(parent, base), block_art(child, bank)
        assert revised[:-1] == original, bank
        assert {m["id"] for m in b["maps"].values() if m["attributes"]["tileset_name"] == bank} == {name}
        new_tile = len(original)
        if name == LIB:
            index, original_block = 21, 0x0f
            donor = block_art(parent, "radio_tower")[0x25]
            assert revised[-1] == tuple((donor if i%4 < 2 else original[original_block])[i] for i in range(16))
            collision = ["WALL", "FLOOR", "WALL", "FLOOR"]
            private_files.add(f"gfx/tilesets/{bank}.pal")
            assert child["runtime_files"][f"gfx/tilesets/{bank}.pal"] == parent["runtime_files"]["gfx/tilesets/house.pal"]
        else:
            index, original_block = 5, 0x15
            collision = a["tilesets"][base]["collision"][f"{original_block:02x}"]
            design, = json.loads((HERE.parents[1] / "landmarks" / "north-shore-pie-table.json").read_text())
            assert design["block"] == new_tile and design["background"] == original_block
            assert "foreground" not in design
            for y, line in enumerate(design["rows"]):
                for x, color in enumerate(line):
                    glyph_palette, planes = revised[-1][y//8*4 + x//8]
                    low, high = planes[(y%8)*2:(y%8)*2+2]
                    bit = 7 - x%8
                    actual = ((low >> bit) & 1) + 2*((high >> bit) & 1)
                    quadrant = y//8*4 + x//8
                    palette = 4 if quadrant in (1, 2, 5, 6) else original[original_block][quadrant][0]
                    assert glyph_palette == palette and actual == int(color), (bank, x, y)
        check_blocks(old, room, index, original_block, new_tile)
        assert b["tilesets"][bank]["collision"] == a["tilesets"][base]["collision"] | {f"{new_tile:02x}": collision}
        private_files.update([f"gfx/tilesets/{bank}.2bpp", f"gfx/tilesets/{bank}.png", f"data/tilesets/{bank}_metatiles.bin"])
        for key, suffix in [("collision", ""), ("palette_map", "_palette_map")]:
            path = f"data/tilesets/{bank}{suffix}.json"
            private_files.add(path)
            assert json.loads(bytes(child["runtime_files"][path])) == b["tilesets"][bank][key]
    only(parent["runtime_files"], child["runtime_files"], private_files, "archive art")

    old, room = a["maps"][LIB], b["maps"][LIB]
    only(old, room, EVENTS | PAGES | {"attributes", "blocks"}, LIB)
    chart = "MnCapitolRiverChart"
    assert room["events"] == dict(old["events"], bg_events=old["events"]["bg_events"] + [reading(10, y, chart) for y in (4, 5)])
    assert program(room, chart) == [("jumptext", chart+"Text")]
    lib_text = {chart+"Text", "MN_SPX_CAPITOLLIBRARY_HOSTText"}
    check_pages(old, room, lib_text)
    only(old["scripts"], room["scripts"], lib_text | {chart}, LIB)
    assert {(11, 4), (10, 6), (7, 7), (6, 3)} <= accessible(b, room, (3, 10))
    city = b["maps"]["MnStPaul"]
    west_bridge = accessible(b, city, (41, 44))
    assert {(26, 49), (26, 54), (11, 64)} <= west_bridge
    assert any(xy(w) == (11, 63) and w["target_map_constant"] == "MN_SPX_ANGLER_HOUSE" for w in city["events"]["warps"])
    for x in (26, 56):
        assert terrain(b, city, (x, 52)) in ("BRIDGE_EW_DECK", "BRIDGE_EW_RAIL")

    old, room = a["maps"][PIE], b["maps"][PIE]
    only(old, room, EVENTS | PAGES | {"attributes", "blocks", "objects", "script_control_commands",
         "script_economy_commands", "script_item_grants", "script_object_commands"}, PIE)
    baker, table = "MN_SHORE_BAKER", "MnShorePieTable"
    assert room["objects"] == [dict(o, x=7, spritemovedata="SPRITEMOVEDATA_STANDING_LEFT")
                               if o["script"] == baker else o for o in old["objects"]]
    assert room["events"] == dict(old["events"], bg_events=[reading(x, y, table) for y in (2, 3) for x in (2, 3)])
    text = {baker+"Text", table+"Text", *("MnShorePie"+s+"Text" for s in ("Poor", "Full", "Paid"))}
    check_pages(old, room, text)
    only(old["scripts"], room["scripts"], text | {baker, table, "MnShorePiePoor", "MnShorePieFull", "MnShorePieClose"}, PIE)
    assert program(room, table) == [("jumptext", table+"Text")]
    assert program(room, baker) == [("faceplayer",), ("opentext",), ("writetext", baker+"Text"), ("yesorno",),
        ("iffalse","MnShorePieClose"), ("checkmoney","YOUR_MONEY","350"), ("ifequal","HAVE_LESS","MnShorePiePoor"),
        ("giveitem",ITEM,"1"), ("iffalse","MnShorePieFull"), ("takemoney","YOUR_MONEY","350"),
        ("writetext","MnShorePiePaidText"), ("waitbutton",), ("closetext",), ("end",)]
    assert program(room, "MnShorePieClose") == [("closetext",), ("end",)]
    for label in ("MnShorePieFull", "MnShorePiePoor"):
        assert program(room, label) == [("writetext", label+"Text"), ("waitbutton",), ("closetext",), ("end",)]
    assert room["script_item_grants"] == [dict(command="giveitem", item_id=ITEM, quantity=1,
                                               source_script=baker, command_index=7, verbose=False)]
    assert room["script_economy_commands"] == [
        dict(command=op, account="YOUR_MONEY", amount_tokens=["350"], source_script=baker, command_index=i)
        for i, op in ((5,"checkmoney"), (9,"takemoney"))]
    assert b["items"][ITEM] == dict(a["items"]["FAIR_MINI_DONUTS"], name="PIE SLICE", script_name=ITEM,
                                    description="Blueberry filling.\nRestores 50 HP.")
    fields = ("price", "parameter", "effect", "consumable", "field_usable", "battle_usable")
    assert [b["items"][ITEM][key] for key in fields] == [350, 50, "RestoreHPEffect", True, True, True]
    assert (5, 3) in accessible(b, room, (3, 6)) and terrain(b, room, (6, 3)) == "COUNTER"

    old, room = a["maps"][DEPOT], b["maps"][DEPOT]
    only(old, room, {"scripts", "script_text_bodies", "blocks"}, DEPOT)
    check_blocks(old, room, 6*16+11, 0x21, 0x19)
    clerk = "MN_ST_PAUL_DEPOT_CLERKText"
    only(old["scripts"], room["scripts"], {clerk}, DEPOT)
    check_pages(old, room, {clerk})
    assert {(22, 13), (24, 15), (7, 14)} <= accessible(b, room, (15, 21))
    old, room = a["maps"][HALL], b["maps"][HALL]
    only(old, room, {"blocks"}, HALL)
    check_blocks(old, room, 6, 0x1d, 0x01)
    assert terrain(b, room, (13, 0)) == "WALL" and (13, 2) in accessible(b, room, (3, 5))


def check_indexes(a, b):
    only(a, b, {"maps", "map_scripts", "map_dimensions", "map_blocks", "map_attributes", "tilesets",
                "items", "runtime_map_metadata", "npcs", "pokegear_landmarks"}, "regional indexes")
    only(a["maps"], b["maps"], AFFECTED, "map scope")
    assert set(b["maps"]) == set(a["maps"]) | {CAB}
    only(a["tilesets"], b["tilesets"], {"mart", *(bank for _, bank in BANKS.values())}, "tilesets")
    only(a["items"], b["items"], {ITEM}, "items")
    only(a["map_attributes"], b["map_attributes"], {LIB, PIE, CAB}, "map attributes")
    for catalog in ("map_dimensions", "runtime_map_metadata"):
        only(a[catalog], b[catalog], {CAB_CONST}, catalog)
    only(a["npcs"], b["npcs"], {PIE, CAB}, "NPCs")
    only(a["pokegear_landmarks"], b["pokegear_landmarks"], {"map_to_landmark"}, "landmarks")
    old_spots, spots = (d["pokegear_landmarks"]["map_to_landmark"] for d in (a, b))
    only(old_spots, spots, {CAB}, "lift landmark")
    assert spots[CAB] == spots[FLOORS[0]]
    meta = b["runtime_map_metadata"][CAB_CONST]
    ground = b["runtime_map_metadata"][b["maps"][FLOORS[0]]["attributes"]["map_constant"]]
    expected_id = 1 + max(m["mapId"] for m in a["runtime_map_metadata"].values() if m["groupId"] == ground["groupId"])
    assert meta == dict(ground, name=CAB, constant=CAB_CONST, width=2, height=2, mapId=expected_id, environment="INDOOR")
    assert b["map_dimensions"][CAB_CONST] == {"width": 2, "height": 2}
    script_ids, block_ids = set(), set()
    for name in AFFECTED:
        new = b["maps"][name]
        old = a["maps"].get(name, {})
        attr = new["attributes"]
        assert attr == b["map_attributes"][name] and b["npcs"][name] == new["objects"]
        block_label = attr["blocks_label"]
        assert bytes(new["blocks"]) == base64.b64decode(b["map_blocks"][block_label], validate=True)
        if new["blocks"] != old.get("blocks"):
            block_ids.add(block_label)
        for label in changed(old.get("scripts", {}), new["scripts"]):
            assert b["map_scripts"][label] == new["scripts"][label]
            script_ids.add(label)
        for field, label in (("map_event_section_commands", attr["map_events_label"]),
                             ("map_script_section_commands", attr["map_scripts_label"])):
            assert new[field] == indexed(b["map_scripts"][label])
            if new[field] != old.get(field):
                script_ids.add(label)
        commands = new["map_event_section_commands"]
        assert [r["args"] for r in commands if r["command"] == "warp_event"] == [
            [str(w["x"]), str(w["y"]), w["target_map_constant"], str(w["target_warp_id"])] for w in new["events"]["warps"]]
        assert [r["args"] for r in commands if r["command"] == "bg_event"] == [
            [str(e["x"]), str(e["y"]), e["event_type"], e["script"]] for e in new["events"]["bg_events"]]
    only(a["map_scripts"], b["map_scripts"], script_ids, "regional script programs")
    only(a["map_blocks"], b["map_blocks"], block_ids, "regional block data")


def verify(parent, child):
    only(parent, child, {"identity", "report", "data", "runtime_files"}, "archive")
    manifests = parent["report"]["manifests"] + ["minneapolis-store-elevator", "mn-shore-bakery-table-v1",
                                               "mn_capitol_library-native-v1", "northstar-lakeside-stops-v1"]
    assert child["report"]["manifests"] == manifests
    only(parent["identity"], child["identity"], {"content_hash", "runtime_modpack_id", "items", "maps"}, "pack identity")
    assert child["identity"]["runtime_modpack_id"] == "+".join(manifests)
    for key in ("maps", "items"):
        assert parent["report"][key] == len(parent["data"][key])
        assert child["identity"][key] == child["report"][key] == len(child["data"][key]) == parent["report"][key]+1
    only(parent["report"], child["report"], {"maps", "items", "reachable_maps", "solvable_maps", "manifests"}, "report")
    for key in ("reachable_maps", "solvable_maps"):
        assert child["report"][key] == parent["report"][key] + [CAB]
    a, b = parent["data"], child["data"]
    check_indexes(a, b)
    check_lift_and_shops(a, b)
    check_tm_counter(b)
    check_services(parent, child)


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("parent", type=Path)
    parser.add_argument("child", type=Path)
    args = parser.parse_args()
    recipes = [json.loads((HERE / f"{e}-build.json").read_text()) for e in ("field", "intro")]
    parent_sha = hashlib.sha256(args.parent.read_bytes()).hexdigest()
    recipe, = [r for r in recipes if r["parent_pack_sha256"] == parent_sha]
    assert hashlib.sha256(args.child.read_bytes()).hexdigest() == recipe["expected_pack_sha256"], "unexpected child for this edition"
    p, q = read_pack(args.parent), read_pack(args.child)
    assert p["identity"]["content_hash"] == recipe["parent_content_id"]
    assert q["identity"]["content_hash"] == recipe["expected_content_id"]
    verify(p, q)
    print(f"{args.child.name}: {len(AFFECTED)} affected maps, 19 street doors and return carpets, 5 elevator menu destinations, "
          f"native shop inventories, bakery, river chart, waiting garden and hall verified; "
          f"{len(p['data']['maps'])} to {len(q['data']['maps'])} maps, {len(q['data']['items'])} items.")
