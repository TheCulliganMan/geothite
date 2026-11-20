#!/usr/bin/env python3
"""Check Marais Supplies against the exact preceding archive of either edition."""
import argparse
from copy import deepcopy
import json
from pathlib import Path
import runpy
import sys

HERE = Path(__file__).resolve().parent
REGION = HERE.parents[1]
ROOT = REGION.parents[1]
sys.path.insert(0, str(REGION / "tools"))
from audit_story_flow import native_pages

COMMON = runpy.run_path(str(HERE.parent / "ely-return-v01" / "check-pack.py"))
read_pack, only = COMMON["read_pack"], COMMON["only"]
ROOM = "MnGrandMaraisFieldStation"
WINONA = "MnWinonaFieldStation"
ACTOR, DOOR = "MnGrandMaraisWelcome", "MnMaraisDoorwayPart"
SPRITE = "SPRITE_MN_PROF_TAMARACK"
MANIFEST = "northstar-marais-supplies-v1"
FLAGS = {"EVENT_MN_MARAIS_" + n for n in ("SUPPLIES_OPEN", "BALLS_RECEIVED", "POTIONS_RECEIVED")}
OLD = {"MnRewardMachinePartOriginal", "MnMaraisGiftAfterRepair", "MnMaraisPartnerGift",
       "MnMaraisPartnerAfterRepair"}
NEW = {DOOR} | {"MnMarais" + n for n in (
    "RecordPartner", "PartnerBox", "PartnerDeclined", "SupplyReturn", "TravelSupplies",
    "TravelMedicine", "CheckSupplies", "CheckWaitingBalls", "HoldPotions", "HoldBoth")}
TEXT = {"MnMarais" + n + "Text" for n in (
    "PartnerDeclined", "PartnerBox", "BallsWaiting", "PotionsWaiting", "BothWaiting",
    "BallsReceived", "PotionsReceived")}
PROGRAMS = OLD | NEW
TABLES = {
    "script_control_commands": {"sjump", "scall", "end", "iftrue", "iffalse", "ifequal", "ifnotequal"},
    "script_text_commands": {"opentext", "closetext", "writetext", "jumptextfaceplayer",
                             "waitbutton", "promptbutton", "yesorno"},
    "script_flag_commands": {"setevent", "clearevent", "checkevent"},
    "script_scene_commands": {"setscene"},
    "script_item_grants": {"giveitem", "verbosegiveitem"},
    "script_item_checks": {"checkitem"},
}


def verify_indexes(before, after):
    # Every changed raw operation with a typed representation must have exactly
    # the native operands and index. Retain ordering of every unrelated row.
    for table, ops in TABLES.items():
        prior = [row for row in before[table] if row["source_script"] not in PROGRAMS]
        assert prior == [row for row in after[table] if row["source_script"] not in PROGRAMS], table
        expected = []
        for script in sorted(PROGRAMS):
            for index, ins in enumerate(after["scripts"][script]):
                op, args = ins["command"], ins["args"]
                if op not in ops:
                    continue
                row = dict(command=op, source_script=script, command_index=index)
                if table == "script_control_commands":
                    label = args[-1] if args else None
                    assert label is None or label in after["scripts"], (script, label)
                    row.update(target_label=label, resolved_target_script=label,
                               compare_value=args[0] if len(args) == 2 else None)
                elif table == "script_text_commands":
                    row["text_label"] = args[0] if args else None
                    assert not args or args[0] in after["script_text_bodies"], (script, args)
                elif table == "script_flag_commands":
                    row["flag_id"] = args[0]
                elif table == "script_scene_commands":
                    row.update(scene_id=args[0], map_id=None)
                elif table == "script_item_grants":
                    row.update(item_id=args[0], quantity=int(args[1]), verbose=op == "verbosegiveitem")
                else:
                    row["item_id"] = args[0]
                expected.append(row)
        actual = [row for row in after[table] if row["source_script"] in PROGRAMS]
        assert sorted(actual, key=lambda r: (r["source_script"], r["command_index"])) == expected, table


def station(old, new, runtime_files):
    before, after = old["maps"][ROOM], new["maps"][ROOM]
    changes = {"objects", "events", "map_event_section_commands", "scripts", "script_text_bodies"}
    only(before, after, changes | TABLES.keys(), "station fields")
    assert before["gift_pokemon_scripts"] == after["gift_pokemon_scripts"]
    assert len(before["gift_pokemon_scripts"]) == 1
    only(before["scripts"], after["scripts"], PROGRAMS | TEXT, "local programs")
    assert set(after["scripts"]) - set(before["scripts"]) == NEW | TEXT
    only(before["script_text_bodies"], after["script_text_bodies"], TEXT, "printed pages")
    texts = json.loads((REGION / "story.json").read_text())["dialogue_overhaul"]["texts"][ROOM]
    for label in TEXT:
        rows = after["scripts"][label]
        assert native_pages(rows) == texts[label], label
        assert after["script_text_bodies"][label] == {
            "label": label, "commands": [dict(row, command_index=i) for i, row in enumerate(rows)]}
    verify_indexes(before, after)
    assert len(before["objects"]) == 1 and before["objects"][0]["sprite"] == "SPRITE_SCIENTIST"
    assert after["objects"] == [dict(before["objects"][0], sprite=SPRITE)]
    assert new["npcs"][ROOM] == after["objects"] and old["npcs"][ROOM] == before["objects"]
    assert any(a["sprite"] == SPRITE and a["pal"] == after["objects"][0]["pal"]
               for a in old["maps"]["MnWinonaFieldStation"]["objects"])
    assert SPRITE in old["sprite_palette_defaults"]
    for suffix in ("png", "2bpp", "gbcpal"):
        assert runtime_files[f"gfx/sprites/mn_prof_tamarack.{suffix}"]
    events = deepcopy(before["events"])
    assert [(e["x"], e["y"], e["scene_id"], e["script_name"]) for e in events["coord_events"]] == [
        (x, 6, "SCENE_MN_FIELD_STATION", ACTOR) for x in (2, 3)]
    for e in events["coord_events"]:
        e["script_name"] = DOOR
    assert events == after["events"]
    raw_events = deepcopy(before["map_event_section_commands"])
    for row in raw_events:
        if row["command"] == "coord_event":
            assert row["args"][3] == ACTOR
            row["args"][3] = DOOR
        elif row["command"] == "object_event":
            assert row["args"][:3] == ["5", "3", "SPRITE_SCIENTIST"]
            row["args"][2] = SPRITE
    assert after["map_event_section_commands"] == raw_events
    event_label = after["attributes"]["map_events_label"]
    for label in PROGRAMS | TEXT:
        assert new["map_scripts"][label] == after["scripts"][label], label
    assert new["map_scripts"][event_label] == [
        {"command": r["command"], "args": r["args"]} for r in raw_events]


def verify(parent, child):
    only(parent, child, {"data", "identity", "report"}, "archive sections")
    only(parent["identity"], child["identity"], {"content_hash", "runtime_modpack_id"}, "identity")
    only(parent["report"], child["report"], {"manifests"}, "build report")
    chain = parent["report"]["manifests"]
    assert chain[-1] == "northstar-ely-return-v1" and child["report"]["manifests"] == chain + [MANIFEST]
    assert child["identity"]["runtime_modpack_id"] == "+".join(chain + [MANIFEST])
    old, new = parent["data"], child["data"]
    only(old, new, {"maps", "map_scripts", "npcs", "story_event_script_constants"}, "gameplay")
    only(old["maps"], new["maps"], {ROOM, WINONA}, "maps")
    only(old["npcs"], new["npcs"], {ROOM}, "NPC index")
    a, b = old["story_event_script_constants"], new["story_event_script_constants"]
    only(a, b, {"global"}, "event tables")
    only(a["global"], b["global"], FLAGS, "new receipt flags")
    highest = max(v for k, v in a["global"].items() if k.startswith("EVENT_"))
    assert FLAGS.isdisjoint(a["global"]) and {b["global"][k] for k in FLAGS} == set(range(highest + 1, highest + 4))
    station(old, new, parent["runtime_files"])
    aide = runpy.run_path(str(HERE / "check-aide.py"))
    aide["check"](old["maps"][WINONA], new["maps"][WINONA])
    winona_labels = set(aide["LABELS"])
    only(old["maps"][WINONA]["scripts"], new["maps"][WINONA]["scripts"], winona_labels,
         "Winona aide programs and text")
    event_label = new["maps"][ROOM]["attributes"]["map_events_label"]
    only(old["map_scripts"], new["map_scripts"], PROGRAMS | TEXT | {event_label} | winona_labels,
         "regional script index")
    for label in winona_labels:
        assert new["map_scripts"][label] == new["maps"][WINONA]["scripts"][label], label
    runpy.run_path(str(HERE / "check-outcomes.py"))["verify"](old["maps"], new["maps"])
    print(f"{len(old['maps'])} maps and {len(parent['runtime_files'])} assets checked; "
          "remaining maps, roster, trainers, item definitions, audio and graphics match the parent.")


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("packs", nargs="*", type=Path)
    args = parser.parse_args()
    receipts = [json.loads((HERE.parent / "ely-return-v01" / f"{edition}-build.json").read_text())
                for edition in ("intro", "field")]
    if len(args.packs) == 2:
        pairs = [args.packs]
    elif not args.packs:
        pairs = [(ROOT / r["output_pack"], ROOT / r["output_pack"].replace("-ely-return-", "-marais-supplies-"))
                 for r in receipts]
    else:
        parser.error("supply a parent and child together")
    for p, c in pairs:
        print(f"Comparing {c.name} to {p.name}", flush=True)
        verify(read_pack(p, {r["expected_pack_sha256"] for r in receipts}), read_pack(c))
