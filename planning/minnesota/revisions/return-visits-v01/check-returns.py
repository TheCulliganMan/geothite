#!/usr/bin/env python3
"""Inspect the clinic's creature and the Duluth crew-book quest on compiled archives."""

import argparse
import json
from pathlib import Path
import sys

REGION = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(REGION / "tools"))
from audit_story_flow import instructions, native_pages, require
from render_reference import load_pack

CLINIC, HALL, WAREHOUSE = "MnRochesterPokecenter1F", "MnDuluthUnionHall1F", "MnDuluthOreWarehouse2F"
UPSTAIRS = "MnDuluthUnionHall2F"
HOST = "MN_DULUTH_X_UNIONHALL_F1_HOST"
CLERK = "MN_DULUTH_X_OREWAREHOUSE_F2_HOST"
RECORDER = "MN_DULUTH_X_UNIONHALL_F2_HOST"
BOOK = "MN_INTERIOR_DETAIL_RESTORED_NAMES"
ENTERED = "EVENT_MN_INTERIOR_UNION_NAMES_ENTERED"
AIDE, LOOBIT = "MN_RC_PATIENT", "MN_ARC_GILLPUP"
WELCOME, NAMES, RECEIPT = ["EVENT_MN_INTERIOR_" + suffix for suffix in ("UNION_WELCOME", "SHIFT_NAMES", "UNION_REWARD")]
LIGHT, RESTORED, TITLE = "EVENT_MN_LIGHTHOUSE_SIGNAL", "EVENT_MN_SURVEY_RESTORED", "EVENT_MN_LEAGUE_CHAMPION"


def visit(room, actor, flags, *, room_in_bag=True):
    """Follow one installed conversation, modeling only its actual event, gift and text instructions."""
    actions, flags, condition = [], set(flags), False
    frames = [(actor, 0)]
    while frames:
        require(len(actions) < 80, f"nonterminating {room['id']}/{actor}")
        label, pc = frames[-1]
        row = instructions(room, label)[pc]
        op, *args = row
        frames[-1] = label, pc + 1
        actions.append(row)
        if op == "checkevent":
            condition = args[0] in flags
        elif op == "setevent":
            flags.add(args[0])
        elif op == "giveitem":
            require(args == ["ETHER", "1"], f"unexpected reward {args}")
            condition = room_in_bag
        elif op == "scall":
            frames.append((args[0], 0))
        elif op in ("sjump", "iffalse", "iftrue"):
            if op == "sjump" or condition == (op == "iftrue"):
                frames[-1] = args[0], 0
        elif op in ("end", "jumptext"):
            frames.pop()
        elif op not in {"faceplayer", "opentext", "writetext", "waitbutton", "closetext", "cry", "waitsfx"}:
            raise ValueError(f"untraced instruction {label} {op}")
    pages = [row[1] for row in actions if row[0] in {"jumptext", "writetext"}]
    return pages, flags, actions


def check_indexes(data, room, label):
    require(data["map_scripts"][label] == room["scripts"][label], f"stale split label {label}")
    for index, (op, *args) in enumerate(instructions(room, label)):
        def typed(table):
            rows = [r for r in room[table] if r["source_script"] == label and r["command_index"] == index]
            require(len(rows) == 1 and rows[0]["command"] == op, f"stale {table}/{label} at {index}")
            return rows[0]
        if op in {"sjump", "scall", "iftrue", "iffalse"}:
            r = typed("script_control_commands")
            require(r["target_label"] == r["resolved_target_script"] == args[0], f"incorrect target {label}")
        elif op == "end":
            typed("script_control_commands")
        elif op in {"setevent", "checkevent"}:
            require(typed("script_flag_commands")["flag_id"] == args[0], f"incorrect event {label}")
        elif op in {"writetext", "jumptext"}:
            require(typed("script_text_commands")["text_label"] == args[0], f"incorrect print {label}")
        elif op == "cry":
            require(typed("script_audio_commands")["audio_id"] == args[0], f"incorrect cry {label}")


def clinic(data, baseline):
    room = data["maps"][CLINIC]
    aide, = [o for o in room["objects"] if o["script"] == AIDE]
    bird, = [o for o in room["objects"] if o["script"] == LOOBIT]
    require((aide["x"], aide["y"], aide["event_flag"]) == (8, 6, "-1")
            and (bird["x"], bird["y"], bird["event_flag"], bird["sprite"]) == (1, 6, RESTORED, "SPRITE_LOOBIT"),
            "clinic object stage")
    expected = "GILLPUP" if baseline else "LOOBIT"
    require(expected in data["pokemon"] and expected in data["pokemon_cries"], "missing native species cry")
    pages, _, sound = visit(room, LOOBIT, set())
    require(sound == [("cry", expected), ("waitsfx",), ("jumptext", "MnOverhaulGillpupText")], "bird interaction")
    check_indexes(data, room, LOOBIT)
    for flags, page in [
        (set(), "MnArcHabitatText"), ({LIGHT}, "MnOverhaulClinicAideAwayText"),
        ({LIGHT, RESTORED}, "MnArcHabitatAfterText"), ({LIGHT, RESTORED, TITLE}, "MnArcHabitatAfterText"),
    ]:
        pages, after, _ = visit(room, AIDE, flags)
        require(pages == [page] and after == flags, f"wrong aide stage {flags}")
    return {CLINIC: ["MnArcHabitatAfterText"]}


def book(data, baseline):
    hall, keeper, recorder = (data["maps"][n] for n in (HALL, WAREHOUSE, UPSTAIRS))
    first, repeat, donor = HOST+"InteriorText", HOST+"InteriorRepeatText", CLERK+"InteriorRepeatText"
    after = repeat if baseline else "MnUnionBookAfterText"
    middle = repeat if baseline else "MnUnionRecorderUpstairsText"
    mark = set() if baseline else {ENTERED}
    written = BOOK+"Text" if baseline else "MnUnionEnteredBookText"
    flags_table = data["story_event_script_constants"]["global"]
    require((ENTERED in flags_table) != baseline, "missing distinct names receipt")
    require(any((r["x"], r["y"], r["script"]) == (9, 7, BOOK) for r in recorder["events"]["bg_events"]),
            "standing crew book read moved")
    if not baseline:
        number = flags_table[ENTERED]
        require([k for k, v in flags_table.items() if k.startswith("EVENT_") and v == number] == [ENTERED],
                "occupied names event ID")
        roster = native_pages(recorder["script_text_bodies"][written]["commands"])
        require(roster[0] == ["NIGHT CREW", "Nora Berg"] and roster[1] == ["Alma Lind", "Gus Ward"],
                "missing readable list of three actual names")
    for with_title in (False, True):
        original = {LIGHT, RESTORED, TITLE} if with_title else set()
        for before_hall in (False, True):
            flags = original.copy()
            if before_hall:
                pages, flags, _ = visit(keeper, CLERK, flags)
                require(pages == [CLERK+"InteriorText"] and flags == original | {NAMES}, "out-of-order list")
                refusal, flags, _ = visit(recorder, RECORDER, flags)
                require(refusal == [RECORDER+"InteriorNeedText"] and flags == original | {NAMES},
                        "recorder bypassed first introduction")
            require(visit(recorder, BOOK, flags)[0] == [BOOK+"Text"], "premature public inscription")
            pages, flags, actions = visit(hall, HOST, flags)
            want = "MnUnionListInHandText" if before_hall and not baseline else first
            require(pages == [want] and flags == original | ({NAMES, WELCOME} if before_hall else {WELCOME}),
                    "first visit order")
            require(actions.index(("writetext", want)) < actions.index(("waitbutton",)) < actions.index(("setevent", WELCOME)),
                    "host must acknowledge the list before recording the visit")
            if not before_hall:
                require(visit(hall, HOST, flags)[0] == [repeat], "pre-list reminder")
                require(visit(recorder, RECORDER, flags)[0] == [RECORDER+"InteriorNeedText"], "recorder needs list")
                _, flags, _ = visit(keeper, CLERK, flags)
            require(visit(hall, HOST, flags)[0] == [middle] and visit(keeper, CLERK, flags)[0] == [donor],
                    "reminder before reward")
            require(visit(recorder, BOOK, flags)[0] == [BOOK+"Text"], "a copied list already changed the book")
            first_offer = [RECORDER+"InteriorText"] + ([] if baseline else ["MnUnionFirstEtherText"])
            immediate, received, immediate_actions = visit(recorder, RECORDER, flags)
            require(immediate == first_offer and received == flags | {RECEIPT} | mark
                    and immediate_actions.count(("giveitem", "ETHER", "1")) == 1, "direct receipt")
            blocked, waiting, actions = visit(recorder, RECORDER, flags, room_in_bag=False)
            require(blocked == first_offer + [RECORDER+"InteriorFullText"] and waiting == flags | mark
                    and actions.count(("giveitem", "ETHER", "1")) == 1, "full-bag recovery")
            if not baseline:
                require(actions.index(("writetext", RECORDER+"InteriorText")) < actions.index(("waitbutton",))
                        < actions.index(("setevent", ENTERED)) < actions.index(("giveitem", "ETHER", "1")),
                        "inscription must be acknowledged before the reward")
            require(visit(recorder, BOOK, waiting)[0] == [written] and
                    visit(hall, HOST, waiting)[0] == [middle if baseline else after] and
                    visit(keeper, CLERK, waiting)[0] == [donor if baseline else "MnWarehouseBookAfterText"],
                    "the book and both desks must know the names while Ether is held")
            later_offer = [RECORDER+"InteriorText"] if baseline else ["MnUnionSavedEtherText"]
            blocked, same, retry = visit(recorder, RECORDER, waiting, room_in_bag=False)
            require(blocked == later_offer + [RECORDER+"InteriorFullText"] and same == waiting
                    and ("setevent", ENTERED) not in retry, "repeat full bag transcribed the names twice")
            pages, flags, actions = visit(recorder, RECORDER, waiting)
            require(pages == later_offer and flags == received and ("setevent", RECEIPT) in actions
                    and actions.index(("giveitem", "ETHER", "1")) < actions.index(("setevent", RECEIPT)),
                    "Ether handoff")
            delivered = flags.copy()
            for place, person, answer in [(hall, HOST, after), (keeper, CLERK, donor if baseline else "MnWarehouseBookAfterText"),
                                          (recorder, RECORDER, RECORDER+"InteriorRepeatText")]:
                pages, flags, actions = visit(place, person, flags)
                require(pages == [answer] and flags == delivered and all(a[0] != "giveitem" for a in actions),
                        f"completed crew book {person}")
            require(visit(recorder, BOOK, delivered)[0] == [written], "delivered book lost names")
        # A previously earned receipt is conclusive even in a partially imported save.
        pages, flags, _ = visit(hall, HOST, original | {RECEIPT})
        require(pages == [first if baseline else "MnUnionBookAfterText"] and flags == original | {RECEIPT, WELCOME},
                "completed visit without earlier hall welcome")
        require(visit(recorder, BOOK, original | {RECEIPT})[0] == [written]
                and visit(recorder, RECORDER, original | {RECEIPT})[0] == [RECORDER+"InteriorRepeatText"],
                "older Ether receipt must imply an inscribed book and spent reward")
        legacy, actual, _ = visit(keeper, CLERK, original | {RECEIPT})
        require(legacy == [CLERK+"InteriorText" if baseline else "MnWarehouseBookAfterText"]
                and actual == original | {RECEIPT} | ({NAMES} if baseline else set()),
                "warehouse still offers the list to a returning recipient")
    for r, prefix in [(hall, "MnUnion"), (keeper, "MnWarehouse"), (recorder, "MnUnion")]:
        for label in r["scripts"]:
            if label not in r["script_text_bodies"] and label.startswith((prefix, HOST, CLERK, RECORDER, BOOK)):
                check_indexes(data, r, label)
    return {HALL: ["MnUnionListInHandText", "MnUnionRecorderUpstairsText", "MnUnionBookAfterText"],
            WAREHOUSE: [CLERK+"InteriorText", donor, "MnWarehouseBookAfterText"],
            UPSTAIRS: [RECORDER+"InteriorText", "MnUnionEnteredBookText", "MnUnionFirstEtherText", "MnUnionSavedEtherText"]}


def run(path, baseline):
    pack = load_pack(path)
    data = pack["data"]
    pages = clinic(data, baseline) | book(data, baseline)
    story = json.loads((REGION / "story.json").read_text())["dialogue_overhaul"]["texts"]
    if not baseline:
        for map_name, labels in pages.items():
            r = data["maps"][map_name]
            for label in labels:
                require(native_pages(r["script_text_bodies"][label]["commands"]) == story[map_name][label],
                        f"wrong canonical print {map_name}/{label}")
                require(native_pages(r["scripts"][label]) == story[map_name][label] and data["map_scripts"][label] == r["scripts"][label],
                        f"native raw/split print differs {label}")
    return {n: data["maps"][n] for n in pages}


def compare(parent, child):
    for name, old in parent.items():
        new = child[name]
        allowed = {
            CLINIC: {LOOBIT, "MnArcHabitatAfterText"},
            HALL: {HOST, HOST+"ReturnVisit", "MnUnionFirstVisit", "MnUnionFirstWithList", "MnUnionFirstAfter",
                   "MnUnionListInHandText", "MnUnionRecorderUpstairs", "MnUnionRecorderUpstairsText",
                   "MnUnionBookAfter", "MnUnionBookAfterText"},
            WAREHOUSE: {CLERK, CLERK+"InteriorText", CLERK+"InteriorRepeatText", CLERK+"ReturnVisit",
                        "MnWarehouseListKnown", "MnWarehouseKnowEnd", "MnWarehouseBookAfter",
                        "MnWarehouseBookAfterText"},
            UPSTAIRS: {RECORDER, RECORDER+"InteriorText", BOOK, "MnUnionReadNames", "MnUnionWriteNames",
                       "MnUnionHoldEther", "MnUnionEnteredBookText", "MnUnionFirstEtherText", "MnUnionSavedEtherText"},
        }[name]
        for field in old:
            if name == UPSTAIRS and field == "blocks":
                width = old["attributes"]["width"]
                square = (7 // 2) * width + (9 // 2)
                require(square == 28 and old[field][square] == 0x21, "original crew book tabletop")
                before, after = old[field].copy(), new[field]
                before[square] = 0x40
            elif field in {"scripts", "script_text_bodies"}:
                before = {k:v for k,v in old[field].items() if k not in allowed}
                after = {k:v for k,v in new[field].items() if k not in allowed}
            elif field in {"script_text_commands", "script_audio_commands", "script_control_commands", "script_flag_commands"}:
                before = [r for r in old[field] if r["source_script"] not in allowed]
                after = [r for r in new[field] if r["source_script"] not in allowed]
            else:
                before, after = old[field], new[field]
            require(before == after, f"unrelated change {name}/{field}")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("pack", type=Path)
    parser.add_argument("--baseline", action="store_true")
    parser.add_argument("--parent", type=Path)
    args = parser.parse_args()
    result = run(args.pack, args.baseline)
    if args.parent:
        compare(run(args.parent, True), result)
    print("Clinic before departure, during the rescue and after restoration; both Union Hall visit orders, "
          "full bag, one Ether, repeat desks, title and imported receipt passed.")


if __name__ == "__main__":
    main()
