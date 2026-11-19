#!/usr/bin/env python3
"""Check the installed Boundary intake directions at each stage of the rescue."""

import argparse
import json
import sys
from pathlib import Path

REGION = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(REGION / "tools"))
from render_reference import load_pack

ROOM = "MnBoundaryIntake"
FINN = "MN_ARC_INTAKE_FINN"
CONTROLS = ("MnArcIntakeStop", "MnVerdigrimOutlet", "MnVerdigrimBypass")
RADIO = "EVENT_MN_ARC_LINK_SENT"
LANDING = "EVENT_MN_VERDIGRIM_COUNTER_CURRENT"
REED = "EVENT_MN_REED_BOUNDARY_DEFEATED"
DRILL = "EVENT_MN_VERDIGRIM_DRILL_STOPPED"
SUMP = "EVENT_MN_VERDIGRIM_OUTLET_CLOSED"
BYPASS = "EVENT_MN_VERDIGRIM_CLEAN_BYPASS"
RESTORED = "EVENT_MN_SURVEY_RESTORED"


def require(condition, message):
    if not condition:
        raise ValueError(message)


def first_text(room, entry, flags):
    """Read the compiled command tables through the first visible text or prompt."""
    indexes = {}
    for table in ("script_flag_commands", "script_control_commands", "script_text_commands"):
        for row in room[table]:
            indexes[row["source_script"], row["command_index"]] = row
    label = entry
    tested = None
    for _ in range(10):
        for index, raw in enumerate(room["scripts"][label]):
            op = raw["command"]
            if op in ("faceplayer", "opentext"):
                continue
            row = indexes[label, index]
            require(row["command"] == op, f"{label}[{index}] has a stale decoded command")
            if op == "checkevent":
                require(raw["args"] == [row["flag_id"]], f"{label} flag mismatch")
                tested = row["flag_id"] in flags
            elif op in ("iffalse", "iftrue"):
                require(tested is not None, f"{label} branches without a condition")
                if tested == (op == "iftrue"):
                    label = row["resolved_target_script"]
                    require(raw["args"] == [label], f"{label} has a stale target")
                    break
            elif op in ("jumptext", "writetext"):
                text = row["text_label"]
                require(raw["args"] == [text] and text in room["script_text_bodies"],
                        f"{label} has a missing text body")
                return text
            else:
                raise ValueError(f"{entry} reached unexpected command {op} before a hint")
        else:
            raise ValueError(f"{entry} ended without a hint at {label}")
    raise ValueError(f"{entry} did not resolve a hint")


def check_intake(parent, candidate):
    previous = parent["data"]["maps"][ROOM]
    room = candidate["data"]["maps"][ROOM]
    changes = {"scripts", "script_flag_commands", "script_control_commands",
               "script_text_commands", "script_text_bodies"}
    for field, value in previous.items():
        if field not in changes:
            require(value == room[field], f"unrelated intake data changed: {field}")
    for label, old in previous["scripts"].items():
        if label not in (FINN, "MnVerdigrimDenied", "MnVerdigrimDeniedText"):
            require(room["scripts"][label] == old, f"changed original action: {label}")
    events = {(e["x"], e["y"], e["event_type"]): e["script"] for e in room["events"]["bg_events"]}
    require([events[x, y, "BGEVENT_UP"] for x, y in ((17, 11), (21, 19), (13, 11))] == list(CONTROLS),
            "a named direction points to the wrong native control")
    reed, = [o for o in room["objects"] if o["script"] == "MN_ARC_INTAKE_REED"]
    require(abs(reed["x"] - 17) + abs(reed["y"] - 11) <= 4, "Reed is no longer beside the drill")
    shelter = candidate["data"]["maps"]["MnBoundaryWatersSurveyShelter"]
    require(any(e["script"] == "MN_BOUNDARY_SURVEY_RECEIVER" for e in shelter["events"]["bg_events"]),
            "the shelter radio has no interaction")
    for action_room, action, flag in [
        (shelter, "MN_BOUNDARY_SURVEY_RECEIVER", RADIO),
        (room, "MnVerdigrimExposure", LANDING),
        (room, "MnArcFinalBattle", REED),
        (room, CONTROLS[0], DRILL),
        (room, CONTROLS[1], SUMP),
        (room, CONTROLS[2], RESTORED),
    ]:
        require(any(c["source_script"] == action and c["command"] == "setevent" and c["flag_id"] == flag
                    for c in action_room["script_flag_commands"]), f"{action} cannot grant {flag}")

    story = json.loads((REGION / "story.json").read_text())
    texts = dict(story["intake_control_dialogue"])
    texts["MnVerdigrimDeniedText"] = story["dialogue_overhaul"]["texts"][ROOM]["MnVerdigrimDeniedText"]
    for label, pages in texts.items():
        expected = []
        for page_index, page in enumerate(pages):
            require(1 <= len(page) <= 2, f"{label}: invalid page size")
            for index, line in enumerate(page):
                require(line.isascii() and 0 < len(line) <= 18, f"{label}: invalid line {line}")
                opcode = "line" if index else ("para" if page_index else "text")
                expected.append((opcode, [f'"{line}"']))
        expected.append(("done", []))
        commands = room["script_text_bodies"][label]["commands"]
        require([(c["command"], c["args"]) for c in commands] == expected, f"{label}: visible pages differ")
    for label, commands in room["scripts"].items():
        require(candidate["data"]["map_scripts"][label] == commands, f"stale global program {label}")

    cases = [
        (set(),     ["MnIntakeRadioHintText"] * 4),
        ({RADIO},   ["MnIntakeLandingHintText"] * 4),
        ({LANDING}, ["MnVerdigrimDeniedText"] * 4),
        ({REED},    ["MnVerdigrimDrillAskText", "MnIntakeDrillHintText",
                     "MnIntakeDrillHintText", "MnArcIntakeFinnText"]),
        ({DRILL},   ["MnVerdigrimAlreadyText", "MnVerdigrimOutletAskText",
                     "MnIntakeOutletHintText", "MnFlowFinnOutletText"]),
        ({SUMP},    ["MnVerdigrimAlreadyText", "MnVerdigrimAlreadyText",
                     "MnVerdigrimBypassAskText", "MnFlowFinnBypassText"]),
        ({BYPASS, RESTORED}, ["MnVerdigrimAlreadyText"] * 3 + ["MnFlowFinnRestoredText"]),
    ]
    flags = set()
    for gained, expected in cases:
        flags.update(gained)
        for entry, want in zip((*CONTROLS, FINN), expected, strict=True):
            got = first_text(room, entry, flags)
            require(got == want, f"{entry} with {sorted(flags)}: {got} instead of {want}")
    print("Boundary intake: all 28 installed control and crew responses passed.")


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("parent", type=Path)
    parser.add_argument("candidate", type=Path)
    args = parser.parse_args()
    check_intake(load_pack(args.parent), load_pack(args.candidate))
