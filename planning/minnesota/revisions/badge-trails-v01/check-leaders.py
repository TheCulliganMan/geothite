#!/usr/bin/env python3
"""Trace all eight badge handoffs and the two changed leaders' return conversations."""

import json
import sys
from argparse import ArgumentParser
from itertools import pairwise, product
from pathlib import Path

HERE = Path(__file__).resolve().parent
REGION = HERE.parents[1]
sys.path.insert(0, str(REGION / "tools"))
from audit_story_flow import instructions, native_pages, read_branch, require
from render_reference import load_pack

TOWNS = (
    "Rochester",
    "Mankato",
    "Minneapolis",
    "StPaul",
    "StCloud",
    "Brainerd",
    "Bemidji",
    "Duluth",
)
BADGES = ("ZEPHYR", "HIVE", "PLAIN", "FOG", "STORM", "MINERAL", "GLACIER", "RISING")
RESTORED = "EVENT_MN_SURVEY_RESTORED"
SKETCH, DATE = "EVENT_MN_DISPATCH_NOTE", "EVENT_MN_DEEP_BORE_MANIFEST_READ"
WIND = "ENGINE_STORMBADGE"
RANGER, GAUGE, REPORT = (
    "EVENT_MN_" + f
    for f in ("BRAINERD_REPORT", "HEADWATERS_READING", "HEADWATERS_REPORT")
)
CHAMP = "EVENT_MN_LEAGUE_CHAMPION"
NEW = {
    "MnStPaulGym": (
        "MnBadgeBrannicWindText",
        "MnBadgeBrannicMinaText",
        "MnBadgeBrannicBreezeText",
    ),
    "MnBrainerdGym": (
        "MnBadgeSivrenItascaText",
        "MnBadgeSivrenOfficerText",
        "MnBadgeSivrenWakeText",
    ),
}


def road(maps, direction, names):
    opposite = {"west": "east", "north": "south"}
    for a, b in pairwise(names):
        for n, target, side in ((a, b, direction), (b, a, opposite[direction])):
            require(
                any(
                    link["target_map"] == target and link["direction"] == side
                    for link in maps[n]["attributes"]["connections"]
                ),
                f"{n} {side} to {target}",
            )


def verify_destinations(data):
    m = data["maps"]
    road(m, "west", ("MnStPaul", "MnR04C00R00", "MnMinneapolis"))
    road(
        m,
        "north",
        ("MnMinneapolis", "MnR05C00R02", "MnR05C00R01", "MnR05C00R00", "MnStCloud"),
    )
    road(
        m,
        "north",
        (
            "MnBrainerd",
            "MnR07C00R01",
            "MnR07C00R00",
            "MnLeechLake",
            "MnR08C00R01",
            "MnR08C00R00",
            "MnBemidji",
        ),
    )
    road(m, "west", ("MnBemidji", "MnR14C01R00", "MnR14C00R00"))
    for n, program in (
        ("MnStPaulUnionDepot", "MN_NSTP_MINA"),
        ("MnBrainerd", "MN_BRAINERD_RANGER"),
        ("MnBemidji", "MN_BEMIDJI_OFFICER"),
    ):
        require(
            any(a["script"] == program for a in m[n]["objects"]),
            f"missing recipient {program}",
        )
    depot = m["MnStPaulUnionDepot"]
    require(
        ("checkevent", SKETCH) in instructions(depot, "MN_ST_PAUL_DEPOT_COURIER")
        and ("setevent", DATE) in instructions(depot, "MnFlowMinaOfferDate"),
        "Mina's undelivered date",
    )
    require(
        ("setevent", RANGER) in instructions(m["MnBrainerd"], "MN_BRAINERD_RANGER"),
        "ranger's own reading",
    )
    lake = m["MnLakeItasca"]
    require(
        any(
            (a["x"], a["y"], a["script"]) == (22, 21, "MN_ITASCA_MARKER")
            for a in lake["events"]["bg_events"]
        ),
        "Itasca staff by the walking bridge",
    )
    for flags in (set(), {RANGER}):
        _, _, read = read_branch(lake, "MN_ITASCA_MARKER", flags)
        require(
            GAUGE in read
            and REPORT in read_branch(m["MnBemidji"], "MN_BEMIDJI_OFFICER", read)[2],
            "can take the reading to Bemidji with or without seeing the ranger first",
        )
    party = data["trainers"]["trainers"]["MN_GYM_6"]["party"]
    require(any(p["species"] == "TORROUT" for p in party), "Sivren's partner")
    lobby = instructions(m["MnLeagueLobby"], "MnElyRegister")
    require(
        ("checkevent", RESTORED) in lobby
        and ("checkflag", "ENGINE_RISINGBADGE") in lobby,
        "the League requires the final badge and restored survey",
    )


def handoffs(parent, candidate=None):
    data = (candidate or parent)["data"]
    canonical = json.loads((REGION / "story.json").read_text())["dialogue_overhaul"][
        "texts"
    ]
    verify_destinations(data)
    for n, (town, badge) in enumerate(zip(TOWNS, BADGES), 1):
        k = "Mn" + town + "Gym"
        r = data["maps"][k]
        old = parent["data"]["maps"][k]
        leader = k + "Leader"
        require(
            any(o["script"] == leader for o in r["objects"]),
            f"physical leader in {town}",
        )
        instruction = instructions(r, leader)
        for op in (
            ("setflag", "ENGINE_" + badge + "BADGE"),
            ("setevent", f"EVENT_MN_GYM_{n}_DEFEATED"),
        ):
            require(op in instruction, f"{town} first badge")
        # Scope the comparison after the load; another component can correct eligibility directions.
        start = next(i for i, op in enumerate(instruction) if op[0] == "loadtrainer")
        old_ops = instructions(old, leader)
        first = next(i for i, op in enumerate(old_ops) if op[0] == "loadtrainer")
        require(
            instruction[start:] == old_ops[first:],
            f"{town} original battle, badge and reward",
        )
        for label in (
            "LeaderReward",
            "LeaderGiftText",
            "LeaderFull",
            "LeaderAfter",
            "RematchOffer",
            "RematchReward",
            "RematchDeclined",
        ):
            require(
                r["scripts"][k + label] == old["scripts"][k + label], f"{town}: {label}"
            )
        full = instructions(r, k + "LeaderReward")
        require(
            full[2][0] == "giveitem"
            and full[3] == ("iffalse", k + "LeaderFull")
            and full[4] == ("setevent", f"EVENT_MN_GYM_{n}_TM_RECEIVED"),
            f"{town} full-pocket retry",
        )
        if k not in NEW:
            require(
                instructions(r, k + "RematchBeforeChampion")
                == instructions(old, k + "RematchBeforeChampion"),
                f"unrelated repeat in {town}",
            )
            continue
        seen = set()
        ready = {f"EVENT_MN_GYM_{n}_DEFEATED", f"EVENT_MN_GYM_{n}_TM_RECEIVED"}
        for bits in product((False, True), repeat=4):
            first, second, third, restored = bits
            names = (
                (SKETCH, DATE, WIND, RESTORED)
                if n == 4
                else (RANGER, GAUGE, REPORT, RESTORED)
            )
            flags = ready | {f for f, on in zip(names, bits) if on}
            if not candidate:
                text = k + "LeaderAfterText"
            elif n == 4:
                text = (
                    NEW[k][1]
                    if first and not (second or restored)
                    else NEW[k][2 if third else 0]
                )
            else:
                text = (
                    NEW[k][2]
                    if third or restored
                    else NEW[k][1]
                    if second
                    else NEW[k][0]
                    if first
                    else k + "LeaderAfterText"
                )
            actions, spoken, after = read_branch(r, leader, flags)
            require(spoken == [text] and after == flags, f"{town}: {flags}, {spoken}")
            if candidate:
                seen.update(
                    a[1] for a in actions if len(a) == 2 and a[1].startswith("MnBadge")
                )
            champion = flags | {CHAMP}
            _, spoken, after = read_branch(r, leader, champion, False)
            require(
                spoken == [k + "RematchOfferText", k + "RematchDeclinedText"]
                and after == champion,
                f"{town} returning Champion can decline",
            )
        if candidate:
            for label in NEW[k]:
                require(
                    native_pages(r["script_text_bodies"][label]["commands"])
                    == canonical[k][label],
                    f"wrong native text {label}",
                )
            for label in seen | {k + "RematchBeforeChampion"}:
                require(
                    r["scripts"][label] == data["map_scripts"][label],
                    f"local/global mirror {label}",
                )
                for table, verbs, field in (
                    ("script_flag_commands", {"checkflag", "checkevent"}, "flag_id"),
                    ("script_control_commands", {"sjump", "iftrue"}, "target_label"),
                    ("script_text_commands", {"jumptext"}, "text_label"),
                ):
                    expected = [
                        (i, o[0], o[1])
                        for i, o in enumerate(instructions(r, label))
                        if o[0] in verbs
                    ]
                    actual = [
                        (a["command_index"], a["command"], a[field])
                        for a in r[table]
                        if a["source_script"] == label
                    ]
                    require(
                        expected == actual,
                        f"{table}/{label}: {expected!r} != {actual!r}",
                    )
    status = "current" if candidate else "original"
    print(
        f"All eight first badges, owed TMs and rematch pathways checked. "
        f"{status.capitalize()} St Paul/Brainerd replies passed 32 journey states and 32 Champion returns."
    )


if __name__ == "__main__":
    parser = ArgumentParser(description=__doc__)
    parser.add_argument("parent", type=Path)
    parser.add_argument("candidate", nargs="?", type=Path)
    args = parser.parse_args()
    handoffs(
        load_pack(args.parent), load_pack(args.candidate) if args.candidate else None
    )
