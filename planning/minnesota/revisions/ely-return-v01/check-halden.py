#!/usr/bin/env python3
"""Check Halden's invitation against the preceding archive and native script tables."""
import argparse
import json
from pathlib import Path
import sys

REGION = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(REGION / "tools"))
from audit_story_flow import instructions, native_pages
from render_reference import load_pack

ROOM = "MnLeagueRoom4"
ASK = "MnLeagueRoom4Challenge"
READY = "MnHaldenReady"
WAIT = "MnHaldenWaits"
BATTLE = "MnLeagueRoom4BattleStart"
FIRST = "MnHaldenFirstBattle"
FLAG = "EVENT_MN_HALDEN_GAVE_TIME"
TEXT = {"MnHaldenReadyText", "MnHaldenWaitText"}
PROGRAMS = {ASK, READY, WAIT}
TABLES = {
    "script_text_commands": {"opentext", "writetext", "promptbutton", "closetext", "yesorno", "waitbutton"},
    "script_flag_commands": {"setevent", "checkevent"},
    "script_control_commands": {"sjump", "iftrue", "iffalse", "ifequal", "ifgreater", "ifnotequal", "end"},
}
FIGHT_TABLES = {"scripted_trainer_battles", "script_block_changes", "script_map_commands",
                "script_variable_commands", "script_control_commands", "script_runtime_commands"}


def changed(a, b):
    return {key for key in a.keys() | b.keys() if key not in a or key not in b or a[key] != b[key]}


def indexed(rows):
    # The native generator regroups programs by label. Instruction order within a program is significant.
    return sorted(rows, key=lambda row: row["source_script"])


def first_battle(value):
    if isinstance(value, dict):
        return {k: first_battle(v) for k, v in value.items()}
    if isinstance(value, list):
        return [first_battle(v) for v in value]
    if value == BATTLE:
        return FIRST
    if value == "MN_LEAGUE_4":
        return "MN_HALDEN_FIRST"
    return value


def verify(parent, child):
    before = parent["maps"][ROOM]
    old = instructions(before, ASK)
    assert old[-5:] == [("opentext",), ("writetext", "MnLeagueRoom4BeforeText"),
                        ("promptbutton",), ("closetext",), ("sjump", BATTLE)]
    assert old[-7:-5] == [("checkevent", "EVENT_MN_LEAGUE_CHAMPION"), ("iftrue", "MnLeagueRoom4RematchIntro")]
    assert FLAG not in parent["story_event_script_constants"]["global"]
    if child is None:
        print("Parent confirms that Halden currently starts the fourth battle on acknowledgment.")
        return
    room = child["maps"][ROOM]
    assert changed(before, room) == {"scripts", "script_text_bodies", *TABLES, *FIGHT_TABLES}
    assert changed(before["scripts"], room["scripts"]) == PROGRAMS | TEXT | {FIRST}
    assert changed(before["script_text_bodies"], room["script_text_bodies"]) == TEXT
    assert instructions(room, ASK) == old[:-5] + [
        ("opentext",), ("checkevent", FLAG), ("iftrue", READY),
        ("writetext", "MnLeagueRoom4BeforeText"), ("promptbutton",), ("sjump", READY)]
    assert instructions(room, READY) == [
        ("writetext", "MnHaldenReadyText"), ("yesorno",), ("iffalse", WAIT),
        ("closetext",), ("sjump", FIRST)]
    assert instructions(room, WAIT) == [
        ("setevent", FLAG), ("writetext", "MnHaldenWaitText"), ("waitbutton",),
        ("closetext",), ("end",)]
    assert all(room["scripts"][label] == child["map_scripts"][label] for label in room["scripts"])
    canonical = json.loads((REGION / "story.json").read_text())["dialogue_overhaul"]["texts"][ROOM]
    for label in TEXT:
        assert native_pages(room["script_text_bodies"][label]["commands"]) == canonical[label]
    flags = child["story_event_script_constants"]["global"]
    assert [key for key, val in flags.items() if key.startswith("EVENT_") and val == flags[FLAG]] == [FLAG]
    assert room["scripts"][FIRST] == first_battle(before["scripts"][BATTLE])
    for table in FIGHT_TABLES:
        rows = [row for row in room[table] if row["source_script"] == FIRST]
        baseline = [row for row in before[table] if row["source_script"] == BATTLE]
        assert baseline and rows == first_battle(baseline), table
        if table not in TABLES:
            assert [r for r in room[table] if r["source_script"] != FIRST] == before[table], table
    for table, commands in TABLES.items():
        assert indexed(row for row in before[table] if row["source_script"] not in PROGRAMS) == indexed(
            row for row in room[table] if row["source_script"] not in PROGRAMS | {FIRST})
        for label in PROGRAMS:
            source = instructions(room, label)
            rows = [row for row in room[table] if row["source_script"] == label]
            assert [(row["command_index"], row["command"]) for row in rows] == [
                (index, action[0]) for index, action in enumerate(source) if action[0] in commands]
            for row in rows:
                op, *args = source[row["command_index"]]
                if table == "script_control_commands" and op != "end":
                    assert row["resolved_target_script"] == args[-1]
                elif table == "script_flag_commands":
                    assert row["flag_id"] == args[0]
                elif op == "writetext":
                    assert row["text_label"] == args[0]
    print("Halden retains the three-win gate, rematch and room geometry. "
          "No marks the greeting heard and returns control. Yes starts the new first-challenge team "
          "and preserves the original win, loss and door callbacks.")


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("parent", type=Path)
    parser.add_argument("child", type=Path, nargs="?")
    args = parser.parse_args()
    verify(load_pack(args.parent)["data"], load_pack(args.child)["data"] if args.child else None)
