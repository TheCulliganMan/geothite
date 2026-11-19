#!/usr/bin/env python3
"""Check the complete release against its own parent, beyond individual feature tests."""

import argparse
import json
from pathlib import Path
import sys

HERE = Path(__file__).resolve().parent
REGION = HERE.parents[1]
sys.path.insert(0, str(REGION / "tools"))
from audit_regional_design import same_except
from audit_story_flow import require
from render_reference import load_pack

SIGNS = {"MnStPaul", "MnR11C00R00"}
INTAKE = "MnBoundaryIntake"
TAVEL = "MnRochesterGym"
MANIFEST = "northstar-campaign-quality-v1"


def compare(parent, child):
    a, b = parent["data"], child["data"]
    mothlight = "MnMothlightLoft" in a["maps"]
    same_except(parent, child, {"data", "report", "identity", "runtime_files"}, "pack")
    same_except(parent["report"], child["report"], {"manifests"}, "report")
    require(
        child["report"]["manifests"] == parent["report"]["manifests"] + [MANIFEST],
        "wrong lineage",
    )
    data_changes = {"maps", "map_scripts"}
    if mothlight:
        data_changes |= {
            "trainers",
            "pokemon",
            "sprite_palette_defaults",
            "npcs",
            "story_event_script_constants",
            "tilesets",
            "map_blocks",
            "trainer_class_names",
        }
    same_except(a, b, data_changes, "game data")
    journey = json.loads((REGION / "story.json").read_text())[
        "journey_landmark_dialogue"
    ]
    clue_dir = REGION / "discoveries/route-07-11-clues-v01"
    clues = json.loads((clue_dir / "caches.json").read_text())
    clue_rooms = {clue["map"] for clue in clues}
    banks = {a["maps"][name]["attributes"]["tileset_name"] for name in clue_rooms}
    rooms = SIGNS | {INTAKE}
    if mothlight:
        rooms |= set(journey) | clue_rooms
    require(a["maps"].keys() == b["maps"].keys(), "changed world map inventory")
    touched = same_except(a["maps"], b["maps"], rooms, "maps")
    require(touched == rooms, "missing a required room change")
    scripts = set()
    for name in touched:
        before, after = a["maps"][name], b["maps"][name]
        for label, commands in after["scripts"].items():
            if commands != before["scripts"].get(label):
                scripts.add(label)
                require(
                    b["map_scripts"].get(label) == commands,
                    f"{name}: stale global script {label}",
                )
        if name == TAVEL and mothlight:
            label = after["attributes"]["map_events_label"]
            scripts.add(label)
            expected = [
                {"command": cmd["command"], "args": cmd["args"]}
                for cmd in after["map_event_section_commands"]
            ]
            require(
                b["map_scripts"][label] == expected,
                "Timekeeper event and global index disagree",
            )
        require(
            before["attributes"] == after["attributes"]
            and before["events"] == after["events"],
            f"{name}: map size, connections or event locations changed",
        )
        if name not in clue_rooms:
            require(before["blocks"] == after["blocks"], f"{name}: terrain changed")
        if name != TAVEL:
            require(before["objects"] == after["objects"], f"{name}: actors changed")
    same_except(a["map_scripts"], b["map_scripts"], scripts, "global scripts")
    if mothlight:
        same_except(a["tilesets"], b["tilesets"], banks, "tilesets")
        mirrors = {a["maps"][name]["attributes"]["blocks_label"] for name in clue_rooms}
        same_except(a["map_blocks"], b["map_blocks"], mirrors, "block mirrors")
        old = a["story_event_script_constants"]
        new = b["story_event_script_constants"]
        same_except(old, new, {"global"}, "story constants")
        supper = "EVENT_MN_WINONA_CHAMPION_SUPPER"
        require(
            supper not in old["global"]
            and supper in new["global"]
            and new["global"][supper] not in old["global"].values(),
            "championship return flag collision",
        )
        same_except(old["global"], new["global"], {supper}, "event flags")
    files = {
        f"data/tilesets/{bank}_native_backgrounds.json"
        for bank in ("mn_iron_descent_thaw", "mn_headwaters_thaw")
    }
    if mothlight:
        for bank in banks:
            for suffix in (
                ".json",
                "_metatiles.bin",
                "_palette_map.json",
                "_foregrounds.json",
            ):
                files.add(f"data/tilesets/{bank}{suffix}")
            files.update(f"gfx/tilesets/{bank}{suffix}" for suffix in (".png", ".2bpp"))
        for folder in ("sprites", "trainers"):
            files.update(
                f"gfx/{folder}/mn_timekeeper{suffix}"
                for suffix in (".png", ".2bpp", ".gbcpal")
            )
    changed_files = same_except(
        parent["runtime_files"], child["runtime_files"], files, "runtime files"
    )
    print(
        f"{len(a['maps'])} maps compared, {len(rooms)} edited, {len(changed_files)} approved art files changed. "
        "All warps, story gates outside the edited rooms, encounter tables, audio, and other maps are intact."
    )


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("parent", type=Path)
    parser.add_argument("candidate", type=Path)
    args = parser.parse_args()
    compare(load_pack(args.parent), load_pack(args.candidate))
