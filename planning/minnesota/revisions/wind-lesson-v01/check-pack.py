#!/usr/bin/env python3
"""Check Saven's three-page greeting, battle registration and complete archive preservation."""
import argparse
import json
from pathlib import Path
import runpy

HERE = Path(__file__).resolve().parent
REGION = HERE.parents[1]
support = runpy.run_path(str(HERE.parent / "ely-return-v01/check-pack.py"))
read_pack, only = support["read_pack"], support["only"]
GYM = "MnStCloudGym"
LABEL = "MnStCloudGymLeaderSeenText"
LEADER = "MnStCloudGymLeader"
MANIFEST = "northstar-wind-lesson-v1"
OLD = [["SAVEN: Feel that?", "The wind turned."], ["A good flier", "turns with it."],
       ["Let me see how", "quickly you adapt!"]]
NEW = [OLD[0], ["If BANKWING digs,", "send in a flier."], OLD[2]]


def text_program(pages):
    rows = []
    for page_index, lines in enumerate(pages):
        assert 1 <= len(lines) <= 2 and all(line.isascii() and 1 <= len(line) <= 18 for line in lines)
        for line_index, line in enumerate(lines):
            opcode = "line" if line_index else ("para" if page_index else "text")
            rows.append({"command": opcode, "args": [json.dumps(line)]})
    return [*rows, {"command": "done", "args": []}]


def lesson_before_battle(data):
    room = data["maps"][GYM]
    [saven] = [o for o in room["objects"] if o["script"] == LEADER]
    assert (saven["x"], saven["y"], saven["object_type"]) == (3, 3, "OBJECTTYPE_SCRIPT")
    prefix = [(row["command"], *row["args"]) for row in room["scripts"][LEADER][:14]]
    assert prefix == [
        ("faceplayer",), ("checkevent", "EVENT_MN_GYM_5_DEFEATED"), ("iftrue", "MnStCloudGymLeaderReward"),
        ("checkflag", "ENGINE_FOGBADGE"), ("iffalse", "MnStCloudGymRequirements"),
        ("checkevent", "EVENT_MN_GYM_5_COURT_OPEN"), ("iffalse", "MnStCloudGymRequirements"),
        ("opentext",), ("writetext", LABEL), ("waitbutton",), ("closetext",),
        ("winlosstext", "MnStCloudGymLeaderWonText", "MnStCloudGymLeaderLostText"),
        ("loadtrainer", "MN_LEADER_SAVEN", "MN_GYM_5"), ("startbattle",),
    ]
    [challenge] = [b for b in room["scripted_trainer_battles"] if b["source_script"] == LEADER]
    assert (challenge["request"]["battle_type"], challenge["request"]["trainer_id"],
            challenge["startbattle_command_index"]) == ("BATTLETYPE_TRAINER", "MN_GYM_5", 13)
    [cue] = [c for c in room["script_text_commands"] if c["source_script"] == LEADER and c["text_label"] == LABEL]
    assert (cue["command"], cue["command_index"]) == ("writetext", 8)

    [bankwing] = [m for m in data["trainers"]["trainers"]["MN_GYM_5"]["party"] if m["species"] == "BANKWING"]
    assert bankwing["level"] == 29 and any(move["name"] == "DIG" for move in bankwing["moves"])
    assert (data["moves"]["DIG"]["type"], data["moves"]["DIG"]["effect"]) == ("GROUND", "FLY")
    [matchup] = [m for m in data["type_effectiveness"]["matchups"]
                 if (m["attacker"], m["defender"]) == ("GROUND", "FLYING")]
    assert matchup["multiplier"]["numerator"] == 0
    assert "FLYING" in (data["pokemon"]["FLOSSAIL"]["type1"], data["pokemon"]["FLOSSAIL"]["type2"])
    for route, slot, level in (("MnR05C00R00", 1, 22), ("MnR05C00R01", 0, 21)):
        assert data["wild_encounters"][route]["grass"]["day"][slot] == {"level": level, "species": "FLOSSAIL"}
        slots = data["encounter_slot_tables"]["tables"]["grass"]
        assert slots[slot]["threshold"] - (slots[slot - 1]["threshold"] if slot else 0) == 30


def verify(parent, child):
    only(parent, child, {"identity", "report", "data"}, "pack envelope")
    only(parent["identity"], child["identity"], {"runtime_modpack_id", "content_hash"}, "identity")
    only(parent["report"], child["report"], {"manifests"}, "report")
    chain = parent["report"]["manifests"]
    assert chain[-1] == "northstar-quarry-landings-v1"
    assert child["report"]["manifests"] == chain + [MANIFEST]
    assert child["identity"]["runtime_modpack_id"] == "+".join(chain + [MANIFEST])
    a, b = parent["data"], child["data"]
    only(a, b, {"maps", "map_scripts"}, "game data")
    only(a["maps"], b["maps"], {GYM}, "maps")
    only(a["map_scripts"], b["map_scripts"], {LABEL}, "global scripts")
    old, new = a["maps"][GYM], b["maps"][GYM]
    only(old, new, {"scripts", "script_text_bodies"}, GYM)
    for key in ("scripts", "script_text_bodies"):
        only(old[key], new[key], {LABEL}, key)
    story = json.loads((REGION / "story.json").read_text())
    assert story["gym_overhaul"]["dialogue"][GYM][LABEL] == OLD
    assert story["dialogue_overhaul"]["texts"][GYM][LABEL] == NEW
    for data, room, pages in ((a, old, OLD), (b, new, NEW)):
        rows = text_program(pages)
        assert data["map_scripts"][LABEL] == room["scripts"][LABEL] == rows
        assert room["script_text_bodies"][LABEL] == {
            "label": LABEL, "commands": [dict(row, command_index=i) for i, row in enumerate(rows)]}
    lesson_before_battle(b)
    print("Saven's one middle page matches all three compiled copies. Gates, battle and regional team access verified.")


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("archives", nargs="+", type=Path, metavar="PARENT CHILD")
    args = parser.parse_args()
    if len(args.archives) % 2:
        parser.error("supply each Quarry Landings parent followed by its Wind Lesson child")
    approved = {json.loads((HERE.parent / "quarry-landings-v01" / f"{ed}-build.json").read_text())["expected_pack_sha256"]
                for ed in ("field", "intro")}
    for before, after in zip(args.archives[::2], args.archives[1::2]):
        print(before.name, "→", after.name, flush=True)
        verify(read_pack(before, approved), read_pack(after))
