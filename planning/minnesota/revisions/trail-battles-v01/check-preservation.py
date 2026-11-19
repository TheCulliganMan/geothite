#!/usr/bin/env python3
"""Compare the two complete archives, shared tile artwork and authored trainer parties."""
import argparse
import base64
import json
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE.parents[1] / "tools"))
from audit_story_flow import require
from render_reference import SourceArt, load_pack

FEN = "MnLayeredFen"
WOODS = "MnBemidjiNorthwoods"
GUARDS = {"MnIceCargoBasement", "MnIceRelayFloor"}
ROOMS = GUARDS | {FEN, WOODS, "MnMinneapolisGym", "MnR05C00R02"}
MARKER = "northstar-trail-battles-v1"


def differences(left, right):
    return left.keys() ^ right.keys() | {k for k in left.keys() & right.keys() if left[k] != right[k]}


def exact(left, right, expected, what):
    changed = differences(left, right)
    require(changed == expected, f"{what}: {changed ^ expected}")


def teams(before, after):
    exact(before, after, {"trainers"}, "trainer settings")
    a, b = before["trainers"], after["trainers"]
    authored = json.loads((HERE / "city-parties.json").read_text())
    designs = {k: team for city in authored.values() for k, team in city.items()}
    exact(a, b, designs.keys(), "changed trainers")
    for key, team in designs.items():
        exact(a[key], b[key], {"party"}, f"{key} metadata")
        require(len(a[key]["party"]) == len(b[key]["party"]) == len(team["party"]), key)
        for old, new, design in zip(a[key]["party"], b[key]["party"], team["party"], strict=True):
            require(differences(old, new) <= {"species", "moves"}, f"{key} party settings")
            require(old["species"] == design["from"] and new["species"] == design["species"]
                    and old["level"] == design["level"]
                    and [m["name"] for m in new["moves"]] == design["moves"], f"{key} party design")


def check(parent, candidate):
    old_pack, new_pack = load_pack(parent), load_pack(candidate)
    require(differences(old_pack, new_pack) <= {"data", "identity", "report", "runtime_files"},
            "audio, format or storage changed")
    exact(old_pack["report"], new_pack["report"], {"manifests"}, "report")
    require(new_pack["report"]["manifests"] == old_pack["report"]["manifests"] + [MARKER], "lineage")
    exact(old_pack["identity"], new_pack["identity"], {"runtime_modpack_id", "content_hash"}, "identity")
    a, b = old_pack["data"], new_pack["data"]
    required = {"maps", "map_scripts", "map_blocks", "npcs", "trainers"}
    changed = differences(a, b)
    require(required <= changed <= required | {"tilesets"}, f"unexpected game data: {changed}")
    exact(a["maps"], b["maps"], ROOMS, "rooms")
    require(a["maps"].keys() == b["maps"].keys(), "lost room")
    exact(a["npcs"], b["npcs"], GUARDS, "actor registry")
    layouts, programs = set(), set()
    for name in ROOMS:
        old, new = a["maps"][name], b["maps"][name]
        require(old["attributes"] == new["attributes"], f"{name} connections or map identity")
        require(old["events"]["warps"] == new["events"]["warps"], f"{name} doors")
        for label in differences(old["scripts"], new["scripts"]):
            programs.add(label)
            require(new["scripts"][label] == b["map_scripts"][label], f"split script {label}")
        if old["map_event_section_commands"] != new["map_event_section_commands"]:
            label = new["attributes"]["map_events_label"]
            programs.add(label)
            expected = [{k: cmd[k] for k in ("command", "args")} for cmd in new["map_event_section_commands"]]
            require(expected == b["map_scripts"][label], f"split events {label}")
        if name in (FEN, WOODS):
            label = new["attributes"]["blocks_label"]
            layouts.add(label)
            require(bytes(new["blocks"]) == base64.b64decode(b["map_blocks"][label]), label)
        else:
            require(old["blocks"] == new["blocks"], f"{name} terrain")
    exact(a["map_scripts"], b["map_scripts"], programs, "split script registry")
    exact(a["map_blocks"], b["map_blocks"], layouts, "terrain registry")
    teams(a["trainers"], b["trainers"])
    if "tilesets" in changed:
        exact(a["tilesets"], b["tilesets"], {"johto"}, "tileset data")
        old, new = (v["tilesets"]["johto"] for v in (a, b))
        require(all(new["collision"][k] == v for k, v in old["collision"].items()), "existing collision")
        before, after = SourceArt(old_pack), SourceArt(new_pack)
        for clock in ("morn", "day", "nite"):
            x, y = before.blocks("johto", clock), after.blocks("johto", clock)
            require(len(y) >= len(x) and all(i.tobytes() == j.tobytes() for i, j in zip(x, y)),
                    f"original Johto artwork at {clock}")
    files = differences(old_pack["runtime_files"], new_pack["runtime_files"])
    require(all(k.startswith(("gfx/tilesets/johto.", "data/tilesets/johto_", "data/tilesets/johto."))
                for k in files), f"unrelated artwork, animation or runtime file: {files}")
    print(f"{candidate.name}: {len(b['maps'])} maps, exactly six revised rooms; "
          "two terrain layouts, two guards and 25 designed parties. Original regional encounters, "
          "doors, audio and existing tile graphics preserved.")


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("parent", type=Path)
    parser.add_argument("candidate", type=Path)
    args = parser.parse_args()
    check(args.parent, args.candidate)
