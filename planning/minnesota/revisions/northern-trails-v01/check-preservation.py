#!/usr/bin/env python3
"""Compare an entire northern-trails pack with its actual parent, including native payloads."""

import argparse
from pathlib import Path
import sys

sys.path.insert(0, str(Path(__file__).resolve().parents[2] / "tools"))
from audit_regional_design import same_except
from audit_story_flow import require
from render_reference import load_pack

SKI = {"MnInternationalFalls", "MnRainyRiverIcefields"}
BLOCKS = SKI | {"MnInnerBasin", "MnR12Cave1F"}
ACTORS = SKI | {"MnR02C00R00"}
SCRIPTS = SKI | {
    "MnR20C00R00",
    "MnDriftlessLampGallery",
    "MnInternationalFallsPokecenter1F",
    "MnBoundaryWaters",
}
ROOMS = BLOCKS | ACTORS | SCRIPTS
ART = {
    "data/tilesets/cave.json",
    "data/tilesets/cave_palette_map.json",
    "data/tilesets/cave_metatiles.bin",
    "gfx/tilesets/cave.png",
    "gfx/tilesets/cave.2bpp",
}


def compare(before, after):
    same_except(
        before, after, {"data", "runtime_files", "identity", "report"}, "entire pack"
    )
    same_except(before["report"], after["report"], {"manifests"}, "report")
    require(
        after["report"]["manifests"]
        == before["report"]["manifests"] + ["northstar-northern-trails-v1"],
        "lineage",
    )
    require(
        same_except(
            before["runtime_files"], after["runtime_files"], ART, "rendering and media"
        )
        == ART,
        "missing native cave art",
    )
    old, new = before["data"], after["data"]
    same_except(
        old,
        new,
        {"maps", "map_blocks", "tilesets", "npcs", "map_scripts"},
        "regional and original game data",
    )
    require(old["maps"].keys() == new["maps"].keys(), "world inventory")
    require(
        same_except(old["maps"], new["maps"], ROOMS, "world maps") == ROOMS,
        "missing updated area",
    )
    same_except(old["tilesets"], new["tilesets"], {"cave"}, "tilesets")
    same_except(
        old["map_blocks"],
        new["map_blocks"],
        {new["maps"][name]["attributes"]["blocks_label"] for name in BLOCKS},
        "terrain mirrors",
    )
    same_except(old["npcs"], new["npcs"], ACTORS, "placed NPC mirrors")
    allowed_scripts = set()
    for name in ROOMS:
        a, b = old["maps"][name], new["maps"][name]
        require(
            a["attributes"] == b["attributes"] and a["events"] == b["events"],
            f"{name} changed map edge, warp or event",
        )
        if name not in BLOCKS:
            require(a["blocks"] == b["blocks"], f"{name} terrain")
        if name not in ACTORS:
            require(a["objects"] == b["objects"], f"{name} actors")
        if name not in SCRIPTS:
            require(a["scripts"] == b["scripts"], f"{name} scripts")
        for label, program in b["scripts"].items():
            if a["scripts"].get(label) != program:
                allowed_scripts.add(label)
                require(
                    new["map_scripts"].get(label) == program,
                    f"{name}/{label} global mirror",
                )
        if name in ACTORS:
            event = b["attributes"]["map_events_label"]
            allowed_scripts.add(event)
            require(
                new["map_scripts"][event]
                == [
                    {"command": row["command"], "args": row["args"]}
                    for row in b["map_event_section_commands"]
                ],
                f"{name} compiled actors",
            )
    same_except(
        old["map_scripts"], new["map_scripts"], allowed_scripts, "global scripts"
    )
    print(
        f"{len(new['maps'])} maps compared, {len(ROOMS)} updated. All existing warps, map edges, trainer teams, "
        "encounter tables, regional species and audio preserved."
    )


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("parent", type=Path)
    parser.add_argument("candidate", type=Path)
    args = parser.parse_args()
    compare(load_pack(args.parent), load_pack(args.candidate))
