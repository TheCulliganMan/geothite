#!/usr/bin/env python3
"""Compare archived scenes, encounters, saves' event names and shared art to the selected parent."""
import argparse
import json
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE.parents[1] / "tools"))
from audit_story_flow import require
from render_reference import SourceArt, load_pack

FEN = "MnLayeredFen"
WOODS = "MnBemidjiNorthwoods"
BANKS = {"johto", "mn_winter_bemidji_northwoods"}


def different(a, b):
    return a.keys() ^ b.keys() | {k for k in a.keys() & b.keys() if a[k] != b[k]}


def check(parent, child):
    before, after = load_pack(parent), load_pack(child)
    require(different(before, after) == {"data", "runtime_files", "identity", "report"}, "archive scope")
    require(different(before["report"], after["report"]) == {"manifests"} and
            after["report"]["manifests"] == before["report"]["manifests"] + ["northstar-fen-clearings-v1"], "lineage")
    a, b = before["data"], after["data"]
    require(different(a, b) == {"maps", "map_scripts", "map_blocks", "story_event_script_constants",
                                "wild_encounters", "tilesets"}, "unexpected regional game data")
    flags, new_flags = (v["story_event_script_constants"] for v in (a, b))
    require(different(flags, new_flags) == {"global"} and
            set(new_flags["global"]) == set(flags["global"]) | {"EVENT_MN_HIDDEN_FEN_SOUTH_BALL"},
            "lost or unrelated event flag")
    require(different(a["maps"], b["maps"]) == {FEN, WOODS} and a["maps"].keys() == b["maps"].keys(),
            "unrelated rooms")
    require(different(a["wild_encounters"], b["wild_encounters"]) == {FEN}, "wildlife changes")
    require(different(a["tilesets"], b["tilesets"]) == BANKS, "unrelated tilesets")
    for k in ("audio", "moves", "pokemon", "learnsets", "trainers", "items", "fishing", "npcs",
              "map_attributes", "map_dimensions", "runtime_map_metadata"):
        require(a[k] == b[k], f"{k} changed")
    scripts = set()
    for name in (FEN, WOODS):
        old, new = a["maps"][name], b["maps"][name]
        require(old["attributes"] == new["attributes"] and old["objects"] == new["objects"],
                f"{name} identity or actors changed")
        require(old["events"]["warps"] == new["events"]["warps"], f"{name} warps")
        for key in different(old["scripts"], new["scripts"]):
            require(new["scripts"][key] == b["map_scripts"][key], f"{name}/{key} split script")
            scripts.add(key)
        if old["map_event_section_commands"] != new["map_event_section_commands"]:
            scripts.add(new["attributes"]["map_events_label"])
    require(different(a["map_scripts"], b["map_scripts"]) == scripts, "unrelated global script")
    src, out = SourceArt(before), SourceArt(after)
    for bank in BANKS:
        x, y = (v["tilesets"][bank] for v in (a, b))
        for k, v in x["collision"].items():
            require(y["collision"][k] == v, f"{bank} original collision {k}")
        for clock in ("morn", "day", "nite"):
            original, revised = src.blocks(bank, clock), out.blocks(bank, clock)
            require(len(revised) >= len(original) and all(j.tobytes() == k.tobytes()
                    for j,k in zip(original, revised)), f"{bank} original tiles at {clock}")
    files = different(before["runtime_files"], after["runtime_files"])
    require(all(any(key.startswith((f"gfx/tilesets/{bank}.",f"data/tilesets/{bank}_",f"data/tilesets/{bank}."))
                    for bank in BANKS) for key in files), f"unrelated artwork or data {files}")
    require(all("animations" not in key for key in files), "animation frames or timing changed")
    for bank in BANKS:
        p = f"data/tilesets/{bank}_palette_map.json"
        require(json.loads(bytes(after["runtime_files"][p])) == b["tilesets"][bank]["palette_map"],
                f"split palette {bank}")
    print(f"{child.name}: compared {len(b['maps'])} rooms. Only Fen and Northwoods revised. "
          "All previous tile art at three clocks, encounter grass, exits, actors, trainers, rods and audio preserved.")


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("parent", type=Path)
    parser.add_argument("child", type=Path)
    args = parser.parse_args()
    check(args.parent, args.child)
