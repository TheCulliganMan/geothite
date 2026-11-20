"""Verify the Winona aide's four phases and the unchanged lab around them."""
import json
from pathlib import Path

SOURCE = Path(__file__).resolve().parents[2] / "story.json"
MAP = "MnWinonaFieldStation"
AIDE = "MnKindlitResearchAide"
MEET = "MnWinonaAideMeet"
MEET_TEXT = "MnWinonaAideMeetText"
READINGS_TEXT = "MnWinonaAideReadingsText"
LABELS = {AIDE, MEET, MEET_TEXT, READINGS_TEXT}
STARTER = "EVENT_MN_STARTER_RECEIVED"
PARCEL = "EVENT_MN_R01_PARCEL_RECEIVED"
DELIVERY = "EVENT_MN_DELIVERED_ROCHESTER_PARCEL"


def program(rows):
    return [(row["command"], *row["args"]) for row in rows]


def check(before, after):
    old, new = before["scripts"], after["scripts"]
    assert {k for k in old.keys() | new.keys() if old.get(k) != new.get(k)} == LABELS
    assert program(old[AIDE]) == [("scall", "MN_W_AIDE"), ("sjump", "MnEarlyPartners")]
    assert program(new[AIDE]) == [
        ("checkevent", STARTER), ("iffalse", MEET), ("scall", "MN_W_AIDE"),
        ("checkevent", DELIVERY), ("iftrue", "MnEarlyPartners"),
        ("checkevent", PARCEL), ("iftrue", "MnEarlyPartners"),
        ("jumptextfaceplayer", READINGS_TEXT),
    ]
    assert program(new[MEET]) == [("jumptextfaceplayer", MEET_TEXT)]
    assert program(new["MnEarlyPartners"])[:2] == [
        ("checkevent", DELIVERY), ("iffalse", "MnEarlyPartnerWait"),
    ]
    assert program(new["MnEarlyPartnerWait"]) == [("jumptext", "MnEarlyPartnerWaitText")]
    assert program(new["MnWChoose"]) == [("jumptextfaceplayer", "MnWChooseText")]
    texts = json.loads(SOURCE.read_text())["dialogue_overhaul"]["texts"][MAP]
    tables = {"scripts", "script_text_bodies", "script_control_commands",
              "script_flag_commands", "script_text_commands"}
    assert {k for k in before.keys() | after.keys() if before.get(k) != after.get(k)} == tables
    bodies = after["script_text_bodies"]
    assert {k for k in bodies if bodies[k] != before["script_text_bodies"].get(k)} == {MEET_TEXT, READINGS_TEXT}
    assert bodies.keys() == before["script_text_bodies"].keys() | {MEET_TEXT, READINGS_TEXT}
    for label in [MEET_TEXT, READINGS_TEXT]:
        lines = [("text" if page == 0 and line == 0 else "para" if line == 0 else "line",
                  '"' + words + '"')
                 for page, pair in enumerate(texts[label]) for line, words in enumerate(pair)]
        expected = lines + [("done",)]
        assert program(new[label]) == program(bodies[label]["commands"]) == expected
        assert [r["command_index"] for r in bodies[label]["commands"]] == list(range(len(expected)))
    for table, commands, target in [
        ("script_control_commands", {"scall", "iftrue", "iffalse", "sjump"}, "target_label"),
        ("script_flag_commands", {"checkevent"}, "flag_id"),
        ("script_text_commands", {"jumptextfaceplayer"}, "text_label"),
    ]:
        original = [r for r in before[table] if r["source_script"] not in LABELS]
        assert [r for r in after[table] if r["source_script"] not in LABELS] == original, table
        actual = [r for r in after[table] if r["source_script"] in LABELS]
        tuples = [(r["source_script"], r["command_index"], r["command"], r[target]) for r in actual]
        expected = [(label, i, op, args[0])
                    for label in (AIDE, MEET)
                    for i, (op, *args) in enumerate(program(new[label])) if op in commands]
        assert sorted(tuples) == sorted(expected), (table, tuples)
        if table == "script_control_commands":
            assert all(r["resolved_target_script"] == r[target] and r["compare_value"] is None for r in actual)
    assert [a["script"] for a in after["objects"] if a["object_identifier"] == "MN_W_AIDE"] == [AIDE]
