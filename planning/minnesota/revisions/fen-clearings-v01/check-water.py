#!/usr/bin/env python3
"""Read the Fen's Surf population, preserve its rods, and check its locally authored banks."""
import argparse
import json
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
REGION = HERE.parents[1]
sys.path.insert(0, str(REGION / "tools"))
from audit_story_flow import require
from render_reference import load_pack

FEN = "MnLayeredFen"


def check(parent, child):
    a, b = (load_pack(p)["data"] for p in (parent, child))
    old, new = (d["wild_encounters"][FEN] for d in (a, b))
    require(a["fishing"] == b["fishing"] and a["encounter_slot_tables"] == b["encounter_slot_tables"],
            "rod or regional encounter rules changed")
    require(b["maps"][FEN]["attributes"]["fishing_group"] == a["maps"][FEN]["attributes"]["fishing_group"]
            == "FISHGROUP_MN_POND", "Fen rods lost their original pond group")
    require(a["wild_encounters"].keys() == b["wild_encounters"].keys()
            and all(b["wild_encounters"][k] == v for k, v in a["wild_encounters"].items() if k != FEN),
            "regional wildlife outside the Fen changed")
    require(old["water"] is None and old["water_rate"] is None and new["water_rate"] == 3,
            "unexpected prior or current water density")
    require(old | {"water": new["water"], "water_rate": 3} == new,
            "grass, swarms or encounter zones changed")
    authorities = REGION / "pokemon/acquisition-v01"
    ecology = json.loads((authorities / "region-ecology-v01/habitats.json").read_text())
    progression = json.loads((authorities / "encounter-progression-v01/progression.json").read_text())
    native_species = set(ecology["water_profiles"]["peat-water"])
    levels, = (band["surf"] for band in progression["groups"] if FEN in band["maps"])
    common = {"morning": "RONDIVE", "day": "REEDRAFT", "night": "LEECHURGE"}
    for clock, slots in new["water"].items():
        require([s["level"] for s in slots] == levels, f"{clock} local Surf levels")
        ids = [s["species"] for s in slots]
        require(len(set(ids)) == 3 and set(ids) <= native_species and ids[0] == common[clock],
                f"{clock} Fen habitat or common species")
    print(f"{child.name}: three peat-water partners at each clock, levels {levels}, "
          "native pond rods and encounter grass preserved throughout the region.")


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("parent", type=Path)
    parser.add_argument("child", type=Path)
    args = parser.parse_args()
    check(args.parent, args.child)
