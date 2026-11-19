#!/usr/bin/env python3
"""Verify the four resident returns and the fair prize against their actual parent archive."""

import argparse
from itertools import product
import json
from pathlib import Path
import sys

HERE = Path(__file__).resolve().parent
REGION = HERE.parents[1]
sys.path.insert(0, str(REGION / "tools"))
from audit_story_flow import instructions, native_pages, read_branch, require
from render_reference import load_pack

FAIR = "MnStateFairgrounds"
ITEM = "FAIR_CORN_STICK"
GAME = "MnFairGame"
PENDING = "EVENT_MN_FAIR_GAME_PENDING"
CLAIMED = "EVENT_MN_FAIR_GAME_PRIZE"
FAIR_TEXTS = {GAME + name + "Text" for name in ("Invite", "Bag", "Win")}
WORKER = "EVENT_MN_REED_WORKER_RELEASED"
WATER = "EVENT_MN_SURVEY_RESTORED"
TITLE = "EVENT_MN_LEAGUE_CHAMPION"
RESIDENTS = {
    "MnStCloud": (
        ("MN_ST_CLOUD_STONECUTTER", (18, 13), WORKER, "MnTownStonecutterAfterText"),
        ("MN_ST_CLOUD_RIVER_WATCHER", (29, 24), WATER, "MnTownRiverWatcherAfterText"),
    ),
    "MnDuluth": (
        ("MN_DULUTH_X_DOCK_HAND", (20, 73), WATER, "MnTownDockHandAfterText"),
        ("MnDuluth_MN_DULUTH_ISLAND_ARTIST", (76, 58), WATER, "MnTownPainterAfterText"),
    ),
}


def pages(data, name, label):
    room = data["maps"][name]
    rows = room["scripts"][label]
    require(data["map_scripts"][label] == rows, f"{name}/{label} global program")
    seen = native_pages(rows)
    require(seen == native_pages(room["script_text_bodies"][label]["commands"]), f"{label} native text")
    return seen


def ring_toss(room, flags=(), *, accept=True, roll=0, fits=True):
    flags = set(flags)
    script = GAME + "Host"
    receipt, spoken = [], []
    condition = None
    for _ in range(12):
        for op, *args in instructions(room, script):
            if op == "checkevent":
                condition = args[0] in flags
            elif op in ("iftrue", "iffalse", "sjump"):
                if op == "sjump" or condition == (op == "iftrue"):
                    script = args[0]
                    break
            elif op == "yesorno":
                condition = accept
            elif op == "random":
                require(args == ["2"], "toss has changed odds")
                condition = bool(roll)
            elif op == "giveitem":
                require(args == [ITEM, "1"], "unexpected fair prize")
                condition = fits
                if fits:
                    receipt.append(ITEM)
            elif op == "setevent":
                flags.add(args[0])
            elif op == "clearevent":
                flags.discard(args[0])
            elif op in ("writetext", "jumptextfaceplayer"):
                spoken.append(args[0])
                if op == "jumptextfaceplayer":
                    return flags, receipt, spoken
            elif op == "end":
                return flags, receipt, spoken
            else:
                require(
                    op in ("faceplayer", "opentext", "waitbutton", "closetext"), f"unreviewed toss {op}"
                )
        else:
            raise ValueError(f"{script} fell through")
    raise ValueError("ring toss did not end")


def rewards(room):
    def labels(*parts):
        return [GAME + name + "Text" for name in parts]

    require(ring_toss(room, accept=False) == (set(), [], labels("Invite")), "declining")
    require(ring_toss(room, roll=1) == (set(), [], labels("Invite", "Miss")), "free retry")
    state, items, spoken = ring_toss(room, fits=False)
    require((state, items, spoken) == ({PENDING}, [], labels("Invite", "Bag")), "pending win")
    require(ring_toss(room, state, roll=1, fits=False) == ({PENDING}, [], labels("Bag")), "still full")
    require(
        ring_toss(room, state, accept=False, roll=1) == ({CLAIMED}, [ITEM], labels("Win")),
        "collect held prize",
    )
    require(ring_toss(room) == ({CLAIMED}, [ITEM], labels("Invite", "Win")), "immediate award")
    require(ring_toss(room, {CLAIMED}) == ({CLAIMED}, [], labels("Repeat")), "duplicate award")


def check_fair(data, revised):
    room = data["maps"][FAIR]
    item = data["items"][ITEM]
    require(item["name"] == "SWEET CORN" and item["parameter"] == 20, "Pack name and recovery")
    (actor,) = [o for o in room["objects"] if o["script"] == GAME + "Host"]
    require((actor["x"], actor["y"], actor["event_flag"]) == (17, 24, "-1"), "midway host")
    new_text = json.loads((REGION / "story.json").read_text())["dialogue_overhaul"]["texts"][FAIR]
    for label in FAIR_TEXTS:
        actual = pages(data, FAIR, label)
        words = " ".join(line for page in actual for line in page)
        if revised:
            require(
                actual == new_text[label] and "CORN STICK" not in words and item["name"] in words, label
            )
        else:
            require("CORN STICK" in words and item["name"] not in words, f"parent preimage {label}")
    rewards(room)


def check_residents(data, reviewed):
    accepted = json.loads((HERE.parent / "town-responses-v01/dialogue.json").read_text())
    for name, speakers in RESIDENTS.items():
        room = data["maps"][name]
        for script, pos, flag, after in speakers:
            first, branch = script + "Text", script + "After"
            (actor,) = [o for o in room["objects"] if o["script"] == script]
            require(
                (actor["x"], actor["y"]) == pos and actor["event_flag"] == "-1", f"{script} position"
            )
            initial = pages(data, name, first)
            if reviewed:
                require(
                    initial == accepted[name][first]
                    and pages(data, name, after) == accepted[name][after],
                    f"{script} reviewed conversations",
                )
                require(
                    instructions(room, script)
                    == [("faceplayer",), ("checkevent", flag), ("iftrue", branch), ("jumptext", first)]
                    and instructions(room, branch) == [("jumptext", after)],
                    f"{script} controls",
                )
                for key, cmd, idx, column, value in (
                    ("script_flag_commands", "checkevent", 1, "flag_id", flag),
                    ("script_control_commands", "iftrue", 2, "resolved_target_script", branch),
                    ("script_text_commands", "jumptext", 3, "text_label", first),
                ):
                    (record,) = [
                        r
                        for r in room[key]
                        if r["source_script"] == script and r["command_index"] == idx
                    ]
                    require(record["command"] == cmd and record[column] == value, f"{script}/{key}")
                for label in (script, branch):
                    require(
                        room["scripts"][label] == data["map_scripts"][label], f"{label} split program"
                    )
            else:
                require(
                    instructions(room, script) == [("jumptextfaceplayer", first)]
                    and after not in room["scripts"]
                    and initial != accepted[name][first],
                    f"{script} original Intro reader",
                )
            for bits in product((False, True), repeat=3):
                flags = {f for f, bit in zip((WORKER, WATER, TITLE), bits) if bit}
                _, spoken, unchanged = read_branch(room, script, flags)
                require(
                    spoken == [after if reviewed and flag in flags else first] and unchanged == flags,
                    f"{script}/{sorted(flags)}",
                )


def changed_keys(before, after):
    return {k for k in before.keys() | after.keys() if before.get(k) != after.get(k)}


def compare(parent, child, field):
    old, new = parent["data"], child["data"]
    require(old["items"][ITEM] == new["items"][ITEM], "fair food changed")
    old_room, new_room = old["maps"][FAIR], new["maps"][FAIR]
    require(
        changed_keys(old_room, new_room) == {"scripts", "script_text_bodies"},
        "changes to fair behavior or terrain",
    )
    for key in ("scripts", "script_text_bodies"):
        require(changed_keys(old_room[key], new_room[key]) == FAIR_TEXTS, f"fair {key} diff")
    for name, speakers in RESIDENTS.items():
        old_room, new_room = old["maps"][name], new["maps"][name]
        if field:
            require(old_room == new_room, f"Field town changed: {name}")
            continue
        labels = {
            l
            for script, _, _, after in speakers
            for l in (script, script + "Text", script + "After", after)
        }
        require(
            changed_keys(old_room["scripts"], new_room["scripts"]) == labels,
            f"{name} Intro script scope",
        )
        parsed = {
            "script_flag_commands",
            "script_control_commands",
            "script_text_commands",
            "script_object_commands",
        }
        require(
            changed_keys(old_room, new_room) == {"scripts", "script_text_bodies", *parsed},
            f"{name} native changes",
        )
        require(
            changed_keys(old_room["script_text_bodies"], new_room["script_text_bodies"])
            == {s for s in labels if s.endswith("Text")},
            f"{name} native page changes",
        )
        for key in parsed:
            untouched = [
                [r for r in room[key] if r["source_script"] not in labels]
                for room in (old_room, new_room)
            ]
            require(untouched[0] == untouched[1], f"{name}/{key} changed another speaker")


def main():
    cli = argparse.ArgumentParser(description=__doc__)
    cli.add_argument("parent", type=Path)
    cli.add_argument("child", type=Path, nargs="?", help="Omit to reproduce the original edition")
    args = cli.parse_args()
    old = load_pack(args.parent)
    field = "northstar-field-review-v2" in old["report"]["manifests"]
    check_fair(old["data"], False)
    check_residents(old["data"], field)
    if args.child:
        new = load_pack(args.child)
        check_fair(new["data"], True)
        check_residents(new["data"], True)
        compare(old, new, field)
    print(
        f"{'Field' if field else 'Intro'}: four residents × eight milestone combinations; ring-toss decline, miss, win, full Pack, retry and repeat passed."
    )


if __name__ == "__main__":
    main()
