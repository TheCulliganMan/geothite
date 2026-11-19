#!/usr/bin/env python3
"""Audit the fossil, field kit and mine return in one installed parent or a parent/child pair."""

import argparse
import json
import sys
from pathlib import Path

REGION = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(REGION / "tools"))
from render_reference import load_pack

GALLERY = "MnDriftlessLampGallery"
FOSSIL = "MnLampFossilsText"
KIT = "MnLampEastShelfText"
READ = "EVENT_MN_LAMP_GALLERY_READ"
CLAIMED = "EVENT_MN_LAMP_GALLERY_CACHE"
FLASH = "STATUSFLAGS_FLASH"
PUMP = "MnIronPumpGallery"
FEED = "EVENT_MN_DRILL_FEED_ISOLATED"
STOPPED = "EVENT_MN_DRILL_STOPPED"


def require(ok, problem):
    if not ok:
        raise ValueError(problem)


def read(room, entry, flags, *, stop_at_prompt=False):
    """Trace the installed read-only switch/hint branches, including nested returns."""
    flags = set(flags)
    tables = {}
    for key in ("script_flag_commands", "script_control_commands", "script_text_commands",
                "script_object_commands"):
        for op in room[key]:
            tables[op["source_script"], op["command_index"]] = op
    label = entry
    shown, appeared = [], []
    tested = None
    for _ in range(12):
        for index, cmd in enumerate(room["scripts"][label]):
            op, args = cmd["command"], cmd["args"]
            if op == "end":
                return shown, flags, appeared
            if op in ("opentext", "closetext", "waitbutton"):
                continue
            row = tables[label, index]
            require(row["command"] == op, f"{label}[{index}]: stale native opcode")
            if op in ("checkflag", "checkevent", "setevent"):
                require(args == [row["flag_id"]], f"{label}: wrong decoded flag")
                if op == "setevent":
                    flags.add(row["flag_id"])
                else:
                    tested = row["flag_id"] in flags
            elif op in ("iftrue", "iffalse", "sjump"):
                require(op == "sjump" or tested is not None, f"{label}: no prior flag check")
                if op == "sjump" or tested == (op == "iftrue"):
                    label = row["resolved_target_script"]
                    require(args == [label], "stale native branch destination")
                    break
            elif op in ("writetext", "jumptext"):
                text = row["text_label"]
                require(args == [text] and text in room["script_text_bodies"], f"{label}: lost page")
                shown.append(text)
                if op == "jumptext" or stop_at_prompt:
                    return shown, flags, appeared
            elif op == "appear":
                require(args == [row["object_id"]], f"{label}: wrong native object")
                appeared.extend(args)
            else:
                raise ValueError(f"unexpected {entry} instruction {label}/{op}")
        else:
            raise ValueError(f"{label} did not close or branch")
    raise ValueError(f"loop in installed {entry} script")


def lines(room, label):
    return [json.loads(op["args"][0]) for op in room["script_text_bodies"][label]["commands"]
            if op["command"] in ("text", "line", "para", "cont", "next")]


def check_pack(pack, *, revised):
    rooms = pack["data"]["maps"]
    gallery, pump = rooms[GALLERY], rooms[PUMP]
    design = json.loads((REGION / "caves/driftless-fossil-gallery-v01/gallery.json").read_text())
    source = json.loads((REGION / "story.json").read_text())["dialogue_overhaul"]["texts"][GALLERY]
    require([(e["x"], e["y"], e["script"]) for e in gallery["events"]["bg_events"]]
            == [(6, 4, "MnLampFossils")], "lost western fossil interaction")
    kit, = [o for o in gallery["objects"] if o["script"] == "MN_LAMP_GALLERY_CACHE"]
    require((kit["x"], kit["y"]) == (19, 9), "lost eastern field kit")
    require(any(e["x"] == 25 and e["y"] == 17 and e["script"] == "MnNDrillGalleryFeed"
                for e in pump["events"]["bg_events"]), "lost pump panel")
    for flags, expected, words in [
        (set(), "MnNDrillGalleryText", "Cut power"),
        ({FEED}, "MnNDrillGalleryAfterText", "still pounding"),
        ({FEED, STOPPED}, "MnNDrillGalleryStoppedText", "boots on the ladder"),
    ]:
        spoken, after, objects = read(pump, "MnNDrillGalleryFeed", flags, stop_at_prompt=True)
        require(spoken == [expected] and words in " ".join(lines(pump, expected)),
                f"incorrect mine panel while {sorted(flags)}: {spoken}")
        require(after == flags and not objects, "reading the mine panel changed progress")

    pages = lines(gallery, FOSSIL)
    if revised:
        require(pages == [line for page in source[FOSSIL] for line in page], "fossil pages differ")
        require(pages[0] == "On the west shelf," and lines(gallery, KIT) == design["read_pages"][-1],
                "the fossil and kit should refer to separate shelves")
    else:
        require(pages[0] == "By the east shelf," and KIT not in gallery["scripts"],
                "unexpected campaign-quality parent description")
    for initial in [set(), {FLASH}, {READ, FLASH}, {READ, CLAIMED, FLASH}, {READ, CLAIMED}]:
        spoken, after, objects = read(gallery, "MnLampFossils", initial)
        if FLASH not in initial:
            require(spoken == ["MnLampTooDarkText"] and after == initial and not objects,
                    "unlit fossil changed discovery state or exposed the cache")
        else:
            hint = [KIT] if revised and CLAIMED not in initial else []
            require(spoken == [FOSSIL] + hint and after == initial | {READ},
                    f"wrong lit fossil response for {sorted(initial)}: {spoken}")
            require(objects == (["MN_LAMP_GALLERY_CACHE"] if CLAIMED not in initial else []),
                    "field kit rediscovery or premature disappearance")


def check_pair(parent, child):
    before = parent["data"]["maps"]
    after = child["data"]["maps"]
    require(before[PUMP] == after[PUMP], "the responsive mine panel already worked in the parent")
    allowed = {"scripts", "script_text_commands", "script_control_commands", "script_text_bodies"}
    for key, value in before[GALLERY].items():
        if key not in allowed:
            require(after[GALLERY][key] == value, f"gallery geometry or unrelated field changed: {key}")
    old, new = before[GALLERY]["scripts"], after[GALLERY]["scripts"]
    require(new.keys() - old.keys() == {KIT}, "unexpected new field interaction")
    for label, body in old.items():
        if label not in (FOSSIL, "MnLampFossils"):
            require(new[label] == body, f"changed field-kit reward or existing action: {label}")
    for label, body in new.items():
        require(child["data"]["map_scripts"][label] == body, f"stale global field script: {label}")


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("parent", type=Path)
    parser.add_argument("candidate", type=Path, nargs="?")
    args = parser.parse_args()
    previous = load_pack(args.parent)
    check_pack(previous, revised=False)
    print(f"{args.parent.name}: three mine stages and five parent fossil states passed.")
    if args.candidate is not None:
        current = load_pack(args.candidate)
        check_pack(current, revised=True)
        check_pair(previous, current)
        print(f"{args.candidate.name}: correct west fossil, conditional east kit and intact mine returns.")
