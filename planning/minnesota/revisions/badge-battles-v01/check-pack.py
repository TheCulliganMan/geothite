#!/usr/bin/env python3
"""Compare a Fen Waterline archive with its Badge Battles child, including guide directions."""

import argparse
import json
import sys
from copy import deepcopy
from pathlib import Path

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE.parents[1] / "tools"))
from audit_regional_design import Terrain
from render_reference import load_pack

GYM = "MnBrainerdGym"
GUIDE = "MnBrainerdGymCircuitBeforeChampion"
BADGE = "MnBadgeBattleGuideBadge"
REPORT = "MnBadgeBattleGuideReport"
CONTROLS = {GUIDE, BADGE, REPORT}
TEXT = GYM + "GuideText"
AFTER_BADGE = GYM + "GuideAfterBadgeText"
AFTER_REPORT = GYM + "GuideAfterReportText"
MANIFEST = "northstar-badge-battles-v1"


def only(before, after, keys, where):
    changed = before.keys() ^ after.keys()
    changed |= {k for k in before.keys() & after.keys() if before[k] != after[k]}
    assert changed == keys, (where, "unexpected", changed - keys, "missing", keys - changed)


def command(name, *args):
    return {"command": name, "args": list(args)}


def text_commands(pages):
    return [
        command("line" if line_index else "para" if page_index else "text", json.dumps(line))
        for page_index, page in enumerate(pages)
        for line_index, line in enumerate(page)
    ] + [command("done")]


def typed_rows(label, script):
    result = {"script_control_commands": [], "script_flag_commands": [], "script_text_commands": []}
    for i, row in enumerate(script):
        op, (arg,) = row["command"], row["args"]
        record = {"source_script": label, "command_index": i, "command": op}
        if op == "iftrue":
            result["script_control_commands"].append(
                dict(record, compare_value=None, target_label=arg, resolved_target_script=arg)
            )
        elif op == "checkevent":
            result["script_flag_commands"].append(dict(record, flag_id=arg))
        else:
            assert op == "jumptextfaceplayer", (label, op)
            result["script_text_commands"].append(dict(record, text_label=arg))
    return result


def guide(before, after):
    edits = json.loads((HERE / "lake-guide.json").read_text())
    texts = {e["label"] for e in edits}
    assert len(edits) == 3 and texts == {TEXT, AFTER_BADGE, AFTER_REPORT}
    only(before["maps"], after["maps"], {GYM}, "maps")
    only(before["map_scripts"], after["map_scripts"], texts | CONTROLS, "global scripts")
    a, b = before["maps"][GYM], after["maps"][GYM]
    fields = {"script_flag_commands", "script_control_commands", "script_text_commands"}
    only(a, b, fields | {"scripts", "script_text_bodies"}, GYM)
    only(a["scripts"], b["scripts"], texts | CONTROLS, "room scripts")
    only(a["script_text_bodies"], b["script_text_bodies"], texts, "parsed text bodies")
    for edit in edits:
        label = edit["label"]
        assert edit["map"] == GYM
        for source, room, pages in ((before, a, edit["before"]), (after, b, edit["after"])):
            if pages is None:
                assert label not in source["map_scripts"] and label not in room["scripts"]
                assert label not in room["script_text_bodies"]
                continue
            commands = text_commands(pages)
            assert source["map_scripts"][label] == room["scripts"][label] == commands, label
            assert room["script_text_bodies"][label] == {
                "label": label,
                "commands": [dict(row, command_index=i) for i, row in enumerate(commands)],
            }, label
    original = {GUIDE: [command("jumptextfaceplayer", TEXT)]}
    revised = {
        GUIDE: [command("checkevent", "EVENT_MN_GYM_6_DEFEATED"),
                command("iftrue", BADGE), command("jumptextfaceplayer", TEXT)],
        BADGE: [command("checkevent", "EVENT_MN_BRAINERD_REPORT"),
                command("iftrue", REPORT), command("jumptextfaceplayer", AFTER_BADGE)],
        REPORT: [command("jumptextfaceplayer", AFTER_REPORT)],
    }
    for field in fields:
        assert [r for r in a[field] if r["source_script"] not in CONTROLS] == [
            r for r in b[field] if r["source_script"] not in CONTROLS
        ], ("unrelated parsed commands", field)
    for source, room, scripts in ((before, a, original), (after, b, revised)):
        for label in CONTROLS:
            entries = scripts.get(label)
            assert room["scripts"].get(label) == source["map_scripts"].get(label) == entries, label
            expected = typed_rows(label, entries or [])
            for field in fields:
                assert [r for r in room[field] if r["source_script"] == label] == expected[field], (label, field)


def parties(before, after):
    recipe = json.loads((HERE / "league.json").read_text())["parties"]
    assert len(recipe) == 3 and all(len(t["move_edits"]) == 1 for t in recipe)
    trainers = {t["trainer_id"] for t in recipe}
    assert trainers == {"MN_LEAGUE_3", "MN_LEAGUE_4", "MN_GYM_8_REMATCH"}
    a, b = before["trainers"], after["trainers"]
    only(a, b, {"trainers"}, "trainer catalog")
    only(a["trainers"], b["trainers"], trainers, "trainer records")
    for trainer in recipe:
        trainer_id = trainer["trainer_id"]
        expected = deepcopy(a["trainers"][trainer_id])
        for edit in trainer["move_edits"]:
            member, = (m for m in expected["party"] if (m["species"], m["level"]) == (edit["species"], edit["level"]))
            move = member["moves"][edit["slot"]]
            assert move["name"] == edit["before"] and move["pp_ups"] == 0, (trainer_id, move)
            assert move["current_pp"] == before["moves"][edit["before"]]["pp"]
            move.update(name=edit["after"], current_pp=before["moves"][edit["after"]]["pp"])
        assert b["trainers"][trainer_id] == expected, ("unexpected party or trainer change", trainer_id)


def directions(pack):
    city = Terrain(pack, "MnBrainerd")
    coast = Terrain(pack, "MnR18C00R00")
    trail = Terrain(pack, "MnR07C00R01")
    callback = "MnPublicRoadsBrainerdOpen"
    assert any(r["command"] == "callback" and r["args"] == ["MAPCALLBACK_TILES", callback]
               for r in city.map["map_script_section_commands"])
    script = city.map["scripts"][callback]
    assert script[-1] == command("endcallback") and all(r["command"] == "disappear" for r in script[:-1])
    hidden = {r["args"][0] for r in script[:-1]}
    city.occupied -= {(o["x"], o["y"]) for o in city.map["objects"] if o["object_identifier"] in hidden}
    entrances = {(w["x"], w["y"]): w["target_map"] for w in city.map["events"]["warps"]}
    assert entrances[9, 22] == "MN_BRAINERD_POKECENTER_1F" and entrances[12, 36] == "MN_BRAINERD_GYM"
    roads = city.reach([(9, 23)])
    assert {(12, 37), (63, 24), (63, 25), (32, 0), (33, 0)} <= roads, "clinic, gym and public roads"
    for here, heading, there, offset in (
        (city, "east", coast, 7), (coast, "west", city, -7),
        (city, "north", trail, 6), (trail, "south", city, -6),
    ):
        assert {"direction": heading, "target_map": there.map["id"], "offset": offset} in here.map["attributes"]["connections"]
    shore = coast.reach([(0, 10)])
    grass = {(14, 10), (14, 11)}
    assert grass | {(0, 11)} <= shore and all(coast.tiles[q] == "TALL_GRASS" for q in grass)
    atlas = bytes(pack["runtime_files"][f"data/tilesets/{coast.tileset}_metatiles.bin"])
    grass_ink = bytes(pack["runtime_files"][f"gfx/tilesets/{coast.tileset}.2bpp"])[64:80]
    assert any(grass_ink) and grass_ink == bytes(pack["runtime_files"]["gfx/tilesets/johto.2bpp"])[64:80]
    for x, y in grass:
        i = coast.block(x, y) * 16 + y % 2 * 8 + x % 2 * 2
        assert all(atlas[i + shift] == 0x04 for shift in (0, 1, 4, 5)), "visible tall grass"
    data = pack["data"]
    wild = data["wild_encounters"][coast.map["id"]]
    assert wild["grass_rates"]["day"] > 0
    assert wild["grass"]["day"][0] == {"species": "REEDRAFT", "level": 29}
    assert data["encounter_slot_tables"]["tables"]["grass"][0] == {"threshold": 30, "slot": 0}
    assert [24, "MEGA_DRAIN"] in data["learnsets"]["REEDRAFT"]
    reachable = trail.reach([(20, 47)])
    assert (21, 47) in reachable and (22, 43) in reachable
    sign, = (e for e in trail.map["events"]["bg_events"] if e["script"] == "MN_R07_SOUTH_SIGN")
    assert (sign["x"], sign["y"]) == (22, 42)
    assert trail.map["scripts"][sign["script"]] == [command("jumptext", "MN_R07_SOUTH_SIGNText")]
    assert command("text", '"LEECH LAKE TRAIL"') in trail.map["scripts"]["MN_R07_SOUTH_SIGNText"]


def verify(parent, child):
    only(parent, child, {"data", "identity", "report"}, "archive")
    only(parent["identity"], child["identity"], {"content_hash", "runtime_modpack_id"}, "identity")
    only(parent["report"], child["report"], {"manifests"}, "report")
    lineage = parent["report"]["manifests"]
    assert lineage[-1] == "northstar-fen-waterline-v1"
    assert child["report"]["manifests"] == lineage + [MANIFEST]
    assert child["identity"]["runtime_modpack_id"] == "+".join(lineage + [MANIFEST])
    a, b = parent["data"], child["data"]
    only(a, b, {"maps", "map_scripts", "trainers"}, "game data")
    parties(a, b)
    guide(a, b)
    directions(child)
    print(f"Verified {len(b['maps'])} maps, {len(b['trainers']['trainers'])} trainers, three move replacements, the guide and both roads.")


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("parent", type=Path)
    parser.add_argument("child", type=Path)
    args = parser.parse_args()
    verify(load_pack(args.parent), load_pack(args.child))
