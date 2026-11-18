#!/usr/bin/env python3
"""Verify the eight packed Gym prizes, sight battles, regional learnsets and Tavel."""
import argparse
from collections import Counter
from copy import deepcopy
import json
from pathlib import Path
import sys

ROOT = Path(__file__).resolve().parents[4]
sys.path.insert(0, str(ROOT / "planning/minnesota/tools"))
from audit_regional_design import DIRS, Terrain, check, load

CITIES = ["Rochester", "Mankato", "Minneapolis", "StPaul", "StCloud", "Brainerd", "Bemidji", "Duluth"]
TIMEKEEPER = "MN_GYM_1_TRAINER_1"
CHANGES = {
    ("MN_GYM_1", "NIBBIN"): ("QUICK_ATTACK", "HEADBUTT"),
    ("MN_GYM_2", "SODCALF"): ("TAIL_WHIP", "MUD_SLAP"),
    ("MN_GYM_3", "PIPULSE"): ("SPARK", "THUNDERPUNCH"),
    ("MN_GYM_3", "SPARKVEIL"): ("THUNDERBOLT", "THUNDERSHOCK"),
    ("MN_GYM_4", "PATCHEBEE"): ("METAL_CLAW", "STEEL_WING"),
    ("MN_GYM_8", "LUTEBRISK"): ("ICE_BEAM", "ICY_WIND"),
}


def run(parent, child):
    old, new = parent["data"], child["data"]
    trainer_count = 0
    demonstrated = []
    trainers = deepcopy(old["trainers"]["trainers"])
    for (team, species), (before, after) in CHANGES.items():
        member = next(m for m in trainers[team]["party"] if m["species"] == species)
        slot = next(m for m in member["moves"] if m["name"] == before)
        slot.update(name=after, current_pp=new["moves"][after]["pp"])
    trainers[TIMEKEEPER]["trainer_class"] = "MN_TIMEKEEPER"
    check(trainers == new["trainers"]["trainers"], "broader trainer, level, AI or item change")
    for i, city in enumerate(CITIES, 1):
        name = f"Mn{city}Gym"
        room = new["maps"][name]
        team = f"MN_GYM_{i}"
        prize, = [g["item_id"][3:] for g in room["script_item_grants"] if g["item_id"].startswith("TM_")]
        check(any(m["name"] == prize for member in trainers[team]["party"] for m in member["moves"]),
              f"{city} leader has no {prize}")
        demonstrated.append(prize)
        leader, = [o for o in room["objects"] if o["script"] == name + "Leader"]
        check(leader["object_type"] == "OBJECTTYPE_SCRIPT" and leader["radius"] == 0, "leader must await conversation")
        for obj in (o for o in room["objects"] if o["script"].startswith(team + "_TRAINER_")):
            flag = room["trainer_scripts"][obj["script"]]["event_flag"]
            check(obj["object_type"] == "OBJECTTYPE_TRAINER" and obj["radius"] > 0
                  and flag == "EVENT_" + obj["script"] + "_DEFEATED", "ordinary on-sight battle")
            trainer_count += 1
    check(trainer_count == 18, "missing Gym sight trainer")
    path = Terrain(child, "MnRochesterGym")
    actor, = [o for o in path.map["objects"] if o["script"] == TIMEKEEPER]
    dx, dy = DIRS[actor["sightline_direction_override"]]
    eyes = {(actor["x"] + dx * step, actor["y"] + dy * step) for step in range(1, actor["radius"] + 1)}
    for x in (12, 13):
        check(eyes == {(12, 14), (13, 14)} and eyes <= path.reach([(x, 16)]), "first battle sight from both entrances")
    check(actor["sprite"] == "SPRITE_MN_TIMEKEEPER" and
          path.map["trainer_scripts"][TIMEKEEPER]["trainer_class"] == "MN_TIMEKEEPER", "native Tavel binding")
    for kind, byte_count in [("trainers", 784), ("sprites", 384)]:
        check(len(child["runtime_files"][f"gfx/{kind}/mn_timekeeper.2bpp"]) == byte_count, f"Tavel {kind} bank")
    pokemon = ROOT / "planning/minnesota/pokemon"
    profiles = {r["id"]: r["tmhm_moves"] for r in json.loads((pokemon/"species-profiles.json").read_text())["species"]}
    roster = json.loads((pokemon/"roster.json").read_text())["families"]
    retired = {f["id"] for f in json.loads((pokemon/"roster-curation.json").read_text())["retired_families"]}
    changes = Counter()
    for family in roster:
        if family["id"] in retired:
            continue
        for member in family["members"]:
            name = member["name"].upper()
            if name not in new["pokemon"]:
                continue
            before, after = [d["pokemon"][name]["tmhm_learnset"] for d in (old, new)]
            for machine in ("THUNDERPUNCH", "STEEL_WING"):
                check((machine in after) == (machine in profiles[member["id"]]), "profile disagrees for " + name)
                if machine in after and machine not in before:
                    changes[machine] += 1
            added = [machine for machine in after if machine not in before]
            check(after == before + added and set(added) <= {"THUNDERPUNCH", "STEEL_WING"},
                  "machine order or unrelated compatibility changed for " + name)
    check(changes == {"THUNDERPUNCH": 7, "STEEL_WING": 10}, "expected 17 additional learners")
    print(f"All eight leaders demonstrate their rewards: {', '.join(demonstrated)}.")
    print("18 sight students, 8 talk-only leaders, 17 added machine learners, 6 adjusted move slots; other trainer data matches.")


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("parent", type=Path)
    parser.add_argument("candidate", type=Path)
    args = parser.parse_args()
    _, parent = load(args.parent)
    _, child = load(args.candidate)
    run(parent, child)
