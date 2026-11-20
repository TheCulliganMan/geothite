#!/usr/bin/env python3
"""Check the intake restoration and border reward against their installed flag writers."""

import argparse
import json
from pathlib import Path
import sys

REGION = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(REGION / "tools"))
from audit_story_flow import BADGES, instructions, read_branch
from render_reference import load_pack


def verify(story, data):
    milestones = {m["id"]: m for m in story["milestones"]}
    intake, border = (milestones[key] for key in ("MS12", "MS17"))
    shelter = data["maps"]["MnBoundaryWatersSurveyShelter"]
    works = data["maps"]["MnBoundaryIntake"]
    crossing = data["maps"]["MnBoundaryWaters"]
    clinic = data["maps"]["MnInternationalFallsPokecenter1F"]
    assert intake["location"] == works["id"] and border["location"] == clinic["id"]
    assert intake["native_texts_location"] == shelter["id"]
    assert all(label in shelter["script_text_bodies"] for label in intake["native_texts"])

    def check_writers(milestone, sequence):
        flags = milestone["intermediate_flags"] + milestone["sets"]
        writes = {}
        for room in data["maps"].values():
            if not room["id"].startswith("Mn"):
                continue
            for command in room["script_flag_commands"]:
                flag = command["flag_id"]
                if command["command"] != "setevent" or flag not in flags:
                    continue
                script = command["source_script"]
                assert flag not in writes, f"{flag} has more than one writer"
                index = command["command_index"]
                assert instructions(room, script)[index] == ("setevent", flag)
                writes[flag] = (room["id"], script, index)
        assert [(writes[f][0], writes[f][1]) for f in flags] == [
            (room["id"], script) for room, script in sequence
        ]
        for a, b in zip(flags, flags[1:]):
            if writes[a][:2] == writes[b][:2]:
                assert writes[a][2] < writes[b][2], f"{b} precedes {a}"

    receiver = "MN_BOUNDARY_SURVEY_RECEIVER"
    bypass = "MnVerdigrimBypass"
    check_writers(intake, [(shelter, receiver), (works, "MnVerdigrimExposure"),
                           (works, "MnArcFinalBattle"), (works, "MnArcIntakeStop"),
                           (works, "MnVerdigrimOutlet")] + [(works, bypass)] * 5)
    link, counter, reed, drill, outlet, *_ = intake["intermediate_flags"]
    (restored,) = intake["sets"]
    assert read_branch(shelter, receiver, intake["requires"], True)[2] == set(intake["requires"]) | {link}
    gates = {counter, reed, drill, outlet}
    for denied in [gates - {flag} for flag in gates]:
        assert restored not in read_branch(works, bypass, denied, True)[2]
    assert restored not in read_branch(works, bypass, gates, False)[2]
    assert restored in read_branch(works, bypass, gates, True)[2]

    arrival_script, ranger = "MnFallsArrivalQualify", "MN_FALLS_RANGER"
    check_writers(border, [(crossing, arrival_script), (clinic, ranger)])
    (arrived,), (rewarded,) = border["intermediate_flags"], border["sets"]
    crossing_code = instructions(crossing, arrival_script)
    assert crossing_code[:2] == [("readvar", "VAR_FACING"), ("ifnotequal", "RIGHT", "MnFallsArrivalEnd")]
    required = [row for row in crossing_code if row[0] in {"checkflag", "checkevent"}]
    assert border["requires"] == [f"BADGE_{i}" for i in range(1, 9)] + [
        flag for action, flag in required if action == "checkevent"]
    assert {flag for action, flag in required if action == "checkflag"} == BADGES
    for gate in required:
        assert crossing_code[crossing_code.index(gate) + 1] == ("iffalse", "MnFallsArrivalEnd")
    claim = instructions(clinic, ranger)
    give = claim.index(("giveitem", "PP_UP", "1"))
    assert claim.index(("checkevent", arrived)) < give
    assert claim[give + 1:give + 3] == [("iffalse", "MnFallsFull"), ("setevent", rewarded)]
    repeat = claim.index(("checkevent", rewarded))
    assert repeat < give and claim[repeat + 1] == ("iftrue", "MnFallsRepeat")


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("pack", type=Path)
    args = parser.parse_args()
    verify(json.loads((REGION / "story.json").read_text()), load_pack(args.pack)["data"])
    print("Verified MS12 intake restoration and MS17 border reward against the installed event order.")
