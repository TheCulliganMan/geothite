#!/usr/bin/env python3
"""Trace the installed Shore Watch volunteers, ranger receipts, shore trainers and rod alternatives."""

import argparse
from collections import Counter
from itertools import permutations, product
import json
from pathlib import Path
import runpy
import sys

HERE = Path(__file__).resolve().parent
REGION = HERE.parents[1]
sys.path.insert(0, str(REGION / "tools"))
from audit_story_flow import instructions, native_pages, require
from render_reference import load_pack

lanes = runpy.run_path(str(HERE.parent / "trail-battles-v01/check-trainers.py"))
WEST, EAST, PARK = "MnR18C00R00", "MnR18C01R00", "MnMilleLacs"
RANGER, PARK_GUIDE = "MN_R18_DOCK_RANGER", "MN_ML_PARK_RANGER"
BADGE, WIN, RECEIPT = (
    "ENGINE_MINERALBADGE",
    "EVENT_MN_GYM_6_DEFEATED",
    "EVENT_MN_SHORE_WATCH_REWARDED",
)
VOLUNTEERS = ("REED", "MEADOW", "BEACH")
REPORTS = tuple(f"EVENT_MN_SHORE_WATCH_0{i}" for i in (1, 2, 3))
HINTS = ("MnShoreWatchReedsText", "MnShoreWatchMeadowText", "MnShoreWatchBeachText")
RODS = ("OLD_ROD", "GOOD_ROD", "SUPER_ROD")


def visit(room, entry, flags, bag, capacity=True):
    """Follow actual native branches; capacity supplies the success result of an atomic grant."""
    label, index, value = entry, 0, None
    pages, gifts = [], []
    for _ in range(128):
        op, *args = instructions(room, label)[index]
        index += 1
        if op in ("checkevent", "checkflag"):
            value = args[0] in flags
        elif op == "checkitem":
            value = bag[args[0]] > 0
        elif op == "setevent":
            flags.add(args[0])
        elif op == "giveitem":
            item, count = args[0], int(args[1])
            (grant,) = [
                r
                for r in room["script_item_grants"]
                if r["source_script"] == label and r["command_index"] == index - 1
            ]
            require(
                (grant["item_id"], grant["quantity"]) == (item, count),
                "typed grant differs from installed script",
            )
            value = capacity
            if value:
                bag[item] += count
                gifts.append((item, count))
        elif op == "sjump" or (
            op in ("iftrue", "iffalse") and value == (op == "iftrue")
        ):
            label, index = args[0], 0
        elif op in ("writetext", "jumptext", "jumptextfaceplayer"):
            pages.append(args[0])
            if op != "writetext":
                return pages, gifts
        elif op == "end":
            return pages, gifts
        else:
            require(
                op
                in (
                    "iftrue",
                    "iffalse",
                    "faceplayer",
                    "closetext",
                    "opentext",
                    "promptbutton",
                    "waitbutton",
                ),
                f"unexpected {label} / {op}",
            )
    raise ValueError(f"{room['id']}/{entry} failed to terminate")


def story(data, fixed):
    rooms = data["maps"]
    west, park = rooms[WEST], rooms[PARK]
    for order in permutations(range(3)):
        flags, bag = set(), Counter()
        for j, i in enumerate(order, 1):
            room = rooms[EAST if i == 2 else WEST]
            actor = "MN_R18_WATCH_" + VOLUNTEERS[i]
            for _ in range(2):
                require(
                    visit(room, actor, flags, bag) == ([actor + "Text"], []),
                    "volunteer lost its observation",
                )
                require(
                    flags == {REPORTS[k] for k in order[:j]},
                    "repeat volunteer double-counts or consumes report",
                )
    count = 0
    for has_badge, completed, bits, rods, fits in product(
        (False, True), (False, True), range(8), range(8), (False, True)
    ):
        seen = {f for i, f in enumerate(REPORTS) if bits & (1 << i)}
        flags = set(seen)
        if has_badge:
            flags.update({BADGE, WIN})
        if completed:
            flags.add(RECEIPT)
        old = flags.copy()
        bag = Counter({r: 1 for i, r in enumerate(RODS) if rods & (1 << i)})
        before = bag.copy()
        pages, gifts = visit(west, RANGER, flags, bag, fits)
        equipped = bool(rods & 6)
        ready = has_badge and not completed and bits == 7
        item = ("LURE_BALL", 3) if equipped else ("GOOD_ROD", 1)
        earned = ready and (fits or (equipped and not fixed))
        expect_gifts = [item] if ready and fits and (not equipped or fixed) else []
        require(
            gifts == expect_gifts and flags == (old | ({RECEIPT} if earned else set())),
            f"dock receipt changed at {old}/{before}/{fits}",
        )
        expected = before.copy()
        for it, q in expect_gifts:
            expected[it] += q
        require(expected == bag, "partial or duplicate gift")
        if not has_badge:
            page = "MN_R18_WatchBadgeDeniedText"
        elif completed:
            page = "MN_R18_WatchAlreadyRewardedText"
        elif bits != 7:
            page = (
                (
                    "MN_R18_DOCK_RANGERText"
                    if bits == 0
                    else next(h for f, h in zip(REPORTS, HINTS) if f not in seen)
                )
                if fixed
                else "MN_R18_WatchNeedsNewsText"
            )
        elif equipped and fixed:
            page = "MnShoreWatchLuresText" if fits else "MnShoreWatchLuresHeldText"
        elif equipped:
            page = "MN_R18_WatchHasRodText"
        else:
            page = "MN_R18_WatchRewardGivenText" if fits else "MN_R18_WatchBagFullText"
        require(
            pages[-1] == page, f"misleading report/receipt page: {old} reads {pages}"
        )
        if ready:
            repeat = visit(west, RANGER, flags, bag, True)
            if earned:
                require(
                    repeat == (["MN_R18_WatchAlreadyRewardedText"], []),
                    "claimed gift can repeat",
                )
            else:
                require(
                    repeat[1] == [item] and RECEIPT in flags,
                    "failed grant cannot be retried",
                )
        if rods == 0 and fits:
            approach = visit(park, PARK_GUIDE, old, Counter())
            dest = (
                ("MnShoreWatchCastingText" if fixed else "MN_ML_PARK_RANGERText")
                if (not has_badge or completed)
                else (
                    "MnShoreWatchReturnText"
                    if fixed and bits == 7
                    else "MnWayShoreReportText"
                )
            )
            require(approach == ([dest], []), f"wrong directions from park {old}")
        count += 1
    print(
        f"{count} ranger states, six report orders and repeat visits; 32 park return/arrival states."
    )


def walking(data):
    rooms = data["maps"]
    anchors = lanes["anchors"](data)
    for name, ids in [(WEST, VOLUNTEERS[:2]), (EAST, VOLUNTEERS[2:])]:
        m = rooms[name]
        actors = {o["script"]: o for o in m["objects"]}
        entrances = {pos for pos in anchors[name] if pos[0] == 0}
        require(entrances, "no west arrival at " + name)
        dry = lanes["approaches"](data, name, entrances, "Walk")
        for label in ["MN_R18_WATCH_" + i for i in ids] + (
            [RANGER] if name == WEST else []
        ):
            where = lanes["coords"](actors[label])
            require(lanes["adjacent"](where) & dry, "cannot face " + label)
        require(
            any(p[0] == m["attributes"]["width"] * 2 - 1 for p in dry),
            "closed shore seam",
        )
        if name != WEST:
            continue
        occupied = {lanes["coords"](a) for a in actors.values()}
        trainers = [o for o in m["objects"] if o["object_type"] == "OBJECTTYPE_TRAINER"]
        require(len(trainers) == 6, "Route 18 trainer count")
        for a in trainers:
            script = a["script"]
            info = m["trainer_scripts"][script]
            team = data["trainers"]["trainers"][info["trainer_id"]]
            require(
                info["trainer_class"] == team["trainer_class"]
                and info["event_flag"] == "EVENT_MN_R18_TRAINER_" + script[-2:]
                and instructions(m, script)[0][0] == "trainer",
                "broken normal trainer",
            )
            sight = lanes["rays"](
                a,
                lanes["coords"](a),
                occupied,
                lambda p: lanes["quadrant"](data, name, *p),
            )
            witness = next((p for p in sight if p in dry), None)
            require(witness is not None, "no walk-up sight lane: " + script)
            print(
                f"{script} {lanes['bearing'](a)} {a['radius']}, sees {witness}, {team['name']} levels {[p['level'] for p in team['party']]}"
            )
    for name, rod in [
        ("MnDriftlessFishingGuruHouse", "OLD_ROD"),
        ("MnRiverValleyFishingGuruHouse", "GOOD_ROD"),
        ("MnPineCountryFishingGuruHouse", "SUPER_ROD"),
    ]:
        room = rooms[name]
        require(
            any(o["script"] == name + "Guru" for o in room["objects"])
            and instructions(room, name + "Guru")[:5]
            == [
                ("faceplayer",),
                ("checkitem", rod),
                ("iftrue", name + "GuruContinue"),
                ("opentext",),
                ("verbosegiveitem", rod, "1"),
            ],
            "rod alternative changed",
        )
    require(
        data["items"]["LURE_BALL"]["pocket"] == "BALL"
        and all(data["items"][r]["pocket"] == "KEY_ITEM" for r in RODS),
        "wrong destination pocket",
    )
    print(
        "All volunteers and six trainers have dry approaches from Brainerd. Three earlier rod gifts retain their actual pocket and source."
    )


def text_mirrors(data):
    source = json.loads((REGION / "story.json").read_text())["dialogue_overhaul"][
        "texts"
    ]
    for name, labels in [
        (WEST, (*HINTS, "MnShoreWatchLuresText", "MnShoreWatchLuresHeldText")),
        (PARK, ("MnShoreWatchReturnText", "MnShoreWatchCastingText")),
    ]:
        m = data["maps"][name]
        for label in labels:
            pages = native_pages(m["scripts"][label])
            require(
                pages
                == source[name][label]
                == native_pages(m["script_text_bodies"][label]["commands"])
                == native_pages(data["map_scripts"][label]),
                "stale native page " + label,
            )
            require(
                all(0 < len(p) <= 2 and all(len(x) <= 18 for x in p) for p in pages),
                "native line overflow " + label,
            )


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("pack", type=Path)
    parser.add_argument(
        "--baseline",
        action="store_true",
        help="reproduce the previous missing equipped-angler reward and report hints",
    )
    args = parser.parse_args()
    data = load_pack(args.pack)["data"]
    story(data, not args.baseline)
    walking(data)
    if not args.baseline:
        text_mirrors(data)
