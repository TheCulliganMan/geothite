#!/usr/bin/env python3
"""Compare a written Fair Table edition with its exact Lakeside Stops parent."""
import argparse
import hashlib
import json
from pathlib import Path
import runpy

HERE = Path(__file__).resolve().parent
common = runpy.run_path(str(HERE.parent / "arcade-frontage-v01" / "check-pack.py"))
read_pack, only, program, indexed = (common[k] for k in ("read_pack", "only", "program", "indexed"))
FAIR, PIE = "MnStateFairgrounds", "MnNorthShorePieShop"
SLICE = "MN_BLUEBERRY_SLICE"
BAKERY = "MART_MN_SPX_WESTSEVENTHBAKERY_HOST"
KITCHEN = "MART_MN_DULUTH_X_COMMUNITYKITCHEN_F1_HOST"
SPECIALS = {
    "MN_BUTTER_ROLL": ("BUTTER ROLL", 200, 20),
    "MN_CARDAMOM_BUN": ("CARDAMOM BUN", 350, 50),
    "MN_WILD_RICE_SOUP": ("RICE SOUP", 500, 60),
}
WARNING, SELLER = "MnFairTwoBeersText", "MnFairRootBeerVendor"
PAVILION, GUEST = "MnFairPavilionSignText", "MN_SHORE_PIE_GUESTText"

HOSTEL = "MnDuluthHillsideHostel1F"
COOK = "MN_DL_INTERIOR_HOSTEL_COOK"
RECEIPT = "EVENT_MN_DULUTH_HOSTEL_BUTTER_ROLL"
RESIDENTS = {
    "MnMinneapolisBridgeHome": ("MN_MP_BRIDGEHOME_HOST", "MnFairTableBridgeWalkText"),
    "MnBrainerdMart": ("MN_BRAINERD_MART_CUSTOMER", "MnFairTableMartLunchText"),
}
GIFT_BRANCHES = {"MnHostelRollRepeat", "MnHostelRollLater", "MnHostelRollFull"}
# These editions have fixed archive hashes and fixed dialogue.
PAGES = json.loads((HERE / "expected-food-dialogue.json").read_text())


def check_pages(old, room, labels):
    only(old["script_text_bodies"], room["script_text_bodies"], labels, room["id"] + " text")
    assert labels == set(PAGES[room["id"]])
    for label in labels:
        commands = []
        for page_number, page in enumerate(PAGES[room["id"]][label]):
            assert 1 <= len(page) <= 2 and all(0 < len(line) <= 18 for line in page)
            for n, line in enumerate(page):
                command = "line" if n else ("para" if page_number else "text")
                commands.append((command, json.dumps(line, ensure_ascii=False)))
        assert program(room, label) == commands + [("done",)]
        assert room["script_text_bodies"][label] == dict(label=label, commands=indexed(room["scripts"][label]))



def check_food(old, new):
    stalls = new["maps"][FAIR]["script_shop_commands"]
    stocks = [[item for item in new["marts"][s["mart_id"]] if item.startswith("FAIR_")] for s in stalls]
    assert len(stalls) == 32 and sum(bool(stock) for stock in stocks) == 13
    fair = {item for stock in stocks for item in stock}
    assert len(fair) == 16 and fair == {key for key in new["items"] if key.startswith("FAIR_")}
    prepared = fair | {SLICE} | SPECIALS.keys()
    only(old["items"], new["items"], prepared, "all item definitions including berries and bottles")
    assert set(new["items"]) == set(old["items"]) | SPECIALS.keys()
    for key in prepared:
        item = new["items"][key]
        if key in SPECIALS:
            assert (item["name"], item["price"], item["parameter"]) == SPECIALS[key]
            only(new["items"][SLICE], item, {"name", "script_name", "description"} |
                 ({"price", "parameter"} if key != "MN_CARDAMOM_BUN" else set()), key)
        else:
            only(old["items"][key], item, {"description", "held_effect"}, key)
        assert item["pocket"] == "ITEM" and item["script_name"] == key
        assert item["consumable"] and item["field_usable"] and item["battle_usable"]
        assert item["field_menu"] == item["battle_menu"] == "ITEMMENU_PARTY"
        assert item["held_effect"] == "HELD_BERRY" and 0 < item["parameter"] <= 100
        lines = item["description"].splitlines()
        assert len(lines) == 2 and all(0 < len(line) <= 18 for line in lines)
        assert f'{item["parameter"]} HP (use/hold).' in lines
        if key == "FAIR_BEER":
            assert item["effect"] == "RESTORE_HP_CONFUSE" and item["confusion_inflict_turns"] == 4
            assert "confusion" in lines[1]
        else:
            assert item["effect"] == "RestoreHPEffect" and item.get("confusion_inflict_turns") is None
    only(old["marts"], new["marts"], {BAKERY, KITCHEN}, "every shop inventory")
    assert new["marts"][BAKERY] == ["MN_BUTTER_ROLL", "MN_CARDAMOM_BUN"] + old["marts"][BAKERY]
    assert new["marts"][KITCHEN] == ["MN_WILD_RICE_SOUP"] + old["marts"][KITCHEN]


def check_locations(old, new):
    only(old["maps"], new["maps"], {FAIR, PIE, HOSTEL, *RESIDENTS}, "maps")
    labels_by_room = {FAIR: {PAVILION, WARNING, SELLER}, PIE: {GUEST}, HOSTEL: set(PAGES[HOSTEL]) | GIFT_BRANCHES | {COOK}}
    for name, (actor, label) in RESIDENTS.items():
        labels_by_room[name] = {actor, label}
    only(old["map_scripts"], new["map_scripts"], set().union(*labels_by_room.values()), "global scripts")
    for name, labels in labels_by_room.items():
        before, room = old["maps"][name], new["maps"][name]
        only(before["scripts"], room["scripts"], labels, name)
        for label in labels:
            assert new["map_scripts"][label] == room["scripts"][label]
        if name in RESIDENTS:
            actor, label = RESIDENTS[name]
            only(before, room, {"scripts", "script_text_bodies", "script_text_commands"}, name)
            assert program(room, actor) == [("jumptextfaceplayer", label)]
            check_pages(before, room, {label})
    before, room = old["maps"][HOSTEL], new["maps"][HOSTEL]
    only(before, room, {"scripts", "script_text_bodies", "script_text_commands", "script_item_grants",
                        "script_flag_commands", "script_control_commands", "script_object_commands"}, HOSTEL)
    ordered = lambda rows: sorted(rows, key=lambda row: (row["source_script"], row["command_index"]))
    assert ordered(before["script_object_commands"]) == ordered(room["script_object_commands"])
    for section in ("script_text_commands", "script_item_grants", "script_flag_commands", "script_control_commands"):
        unrelated = lambda m: ordered([row for row in m[section] if row["source_script"] not in labels_by_room[HOSTEL]])
        assert unrelated(before) == unrelated(room), section
    check_pages(before, room, set(PAGES[HOSTEL]))
    assert program(room, COOK) == [
        ("faceplayer",), ("checkevent", RECEIPT), ("iftrue", "MnHostelRollRepeat"), ("opentext",),
        ("writetext", "MnHostelButterRollOfferText"), ("yesorno",), ("iffalse", "MnHostelRollLater"),
        ("verbosegiveitem", "MN_BUTTER_ROLL", "1"), ("iffalse", "MnHostelRollFull"),
        ("setevent", RECEIPT), ("closetext",), ("end",)]
    assert program(room, "MnHostelRollRepeat") == [("jumptext", "MnHostelButterRollRepeatText")]
    for branch in ("Later", "Full"):
        assert program(room, "MnHostelRoll" + branch) == [
            ("writetext", "MnHostelButterRoll" + branch + "Text"), ("waitbutton",), ("closetext",), ("end",)]
    tables = "story_event_script_constants"
    only(old[tables], new[tables], {"global"}, "event tables")
    a, b = old[tables]["global"], new[tables]["global"]
    only(a, b, {RECEIPT}, "event receipt")
    assert b[RECEIPT] == 1 + max(value for key, value in a.items() if key.startswith("EVENT_"))
    old_pie, pie = old["maps"][PIE], new["maps"][PIE]
    only(old_pie, pie, {"scripts", "script_text_bodies"}, PIE)
    only(old_pie["scripts"], pie["scripts"], {GUEST}, PIE)
    check_pages(old_pie, pie, {GUEST})
    before, fair = old["maps"][FAIR], new["maps"][FAIR]
    allowed = {"scripts", "script_text_bodies", "script_shop_commands", "script_text_commands", "script_control_commands"}
    only(before, fair, allowed, FAIR)
    only(before["scripts"], fair["scripts"], {WARNING, PAVILION, SELLER}, FAIR)
    check_pages(before, fair, {WARNING, PAVILION})
    assert program(fair, SELLER) == [
        ("faceplayer",), ("opentext",), ("writetext", WARNING), ("waitbutton",),
        ("pokemart", "MARTTYPE_STANDARD", "MART_MnFairRootBeerVendor"), ("closetext",), ("end",)]
    for section in ("script_shop_commands", "script_text_commands", "script_control_commands"):
        others = lambda room: [r for r in room[section] if r["source_script"] != SELLER]
        assert others(before) == others(fair), section
        for row in fair[section]:
            if row["source_script"] == SELLER:
                assert fair["scripts"][SELLER][row["command_index"]]["command"] == row["command"]
    for room, labels in ((fair, (PAVILION, SELLER, WARNING)), (pie, (GUEST,))):
        for label in labels:
            assert new["map_scripts"][label] == room["scripts"][label], label


def check(parent, child):
    only(parent, child, {"identity", "data", "report"}, "archive and artwork")
    only(parent["data"], child["data"], {"maps", "map_scripts", "items", "marts", "story_event_script_constants"}, "game data")
    only(parent["report"], child["report"], {"manifests", "items"}, "report")
    only(parent["identity"], child["identity"], {"runtime_modpack_id", "content_hash", "items"}, "identity")
    manifests = parent["report"]["manifests"] + ["northstar-fair-table-v1"]
    assert child["report"]["manifests"] == manifests
    assert child["identity"]["runtime_modpack_id"] == "+".join(manifests)
    assert child["identity"]["items"] == child["report"]["items"] == len(child["data"]["items"]) == parent["report"]["items"] + 3
    check_food(parent["data"], child["data"])
    check_locations(parent["data"], child["data"])


if __name__ == "__main__":
    args = argparse.ArgumentParser(description=__doc__)
    args.add_argument("parent", type=Path)
    args.add_argument("child", type=Path)
    paths = args.parse_args()
    parent_sha = hashlib.sha256(paths.parent.read_bytes()).hexdigest()
    recipes = (json.loads(p.read_text()) for p in HERE.glob("*-build.json"))
    recipe, = [r for r in recipes if r["parent_pack_sha256"] == parent_sha]
    assert hashlib.sha256(paths.child.read_bytes()).hexdigest() == recipe["expected_pack_sha256"]
    parent, child = read_pack(paths.parent), read_pack(paths.child)
    assert parent["identity"]["content_hash"] == recipe["parent_content_id"]
    assert child["identity"]["content_hash"] == recipe["expected_content_id"]
    check(parent, child)
    print(f"{paths.child.name}: 20 usable and holdable preparations, 13 specialty stalls, two city menus, the hostel gift and five locations verified.")
