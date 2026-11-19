#!/usr/bin/env python3
"""Verify the bounded archive changes from each Fen Clearings parent."""

import argparse
import base64
import json
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE.parents[1] / "tools"))
from audit_story_flow import require
from render_reference import SourceArt, load_pack

TERRAIN = {"MnR13C00R02", "MnLeagueRoom3", "MnLeagueChampion"}
CLINICS = {"MnElyPokecenter1F", "MnSplitRockPokecenter1F"}
RANGERS = {"MnEly", "MnBoundaryWaters", "MnElyShorePass"}


def changed(a, b):
    return (a.keys() ^ b.keys()) | {k for k in a.keys() & b.keys() if a[k] != b[k]}


def verify(parent, child):
    before, after = load_pack(parent), load_pack(child)
    require(
        changed(before, after) == {"identity", "data", "report", "runtime_files"},
        "archive changes",
    )
    require(
        changed(before["report"], after["report"]) == {"manifests"}
        and after["report"]["manifests"]
        == before["report"]["manifests"] + ["northstar-league-journey-v1"],
        "parent edition or map count",
    )
    a, b = before["data"], after["data"]
    require(
        changed(a, b) == {"maps", "map_blocks", "map_scripts", "marts", "tilesets"},
        "unexpected data change",
    )
    require(changed(a["marts"], b["marts"]) == {"MART_MN_ELY"}, "unrelated mart stock")
    rooms = changed(a["maps"], b["maps"])
    require(
        rooms == TERRAIN | CLINICS | RANGERS and a["maps"].keys() == b["maps"].keys(),
        "room scope",
    )
    labels = set()
    for name in rooms:
        old, new = a["maps"][name], b["maps"][name]
        require(
            old["attributes"] == new["attributes"] and old["objects"] == new["objects"],
            f"{name} actors or attributes changed",
        )
        if name in TERRAIN:
            require(
                changed(old, new) == {"blocks"}, f"{name} scripts or objects changed"
            )
            block_label = new["attributes"]["blocks_label"]
            require(
                base64.b64decode(b["map_blocks"][block_label]) == bytes(new["blocks"]),
                f"split block map in {name}",
            )
        elif name in CLINICS:
            require(
                changed(old, new) == {"events", "map_event_section_commands"}
                and changed(old["events"], new["events"]) == {"bg_events"},
                f"{name} clinic scope",
            )
            labels.add(new["attributes"]["map_events_label"])
        else:
            if name == "MnEly":
                key = new["attributes"]["blocks_label"]
                require(
                    base64.b64decode(b["map_blocks"][key]) == bytes(new["blocks"]),
                    "split Ely trail",
                )
                require(
                    [
                        i
                        for i, pair in enumerate(zip(old["blocks"], new["blocks"]))
                        if pair[0] != pair[1]
                    ]
                    == [12],
                    "unexpected Ely terrain outside its northern entrance",
                )
            require(
                (name == "MnEly" or old["blocks"] == new["blocks"])
                and old["events"] == new["events"],
                f"{name} geometry",
            )
            for label in changed(old["scripts"], new["scripts"]):
                require(
                    b["map_scripts"][label] == new["scripts"][label],
                    f"{name} split dialogue {label}",
                )
                labels.add(label)
    require(
        changed(a["map_scripts"], b["map_scripts"]) == labels, "split program scope"
    )
    require(
        changed(a["map_blocks"], b["map_blocks"])
        == {b["maps"][m]["attributes"]["blocks_label"] for m in TERRAIN | {"MnEly"}},
        "unrelated split blocks",
    )
    ely = "mn_ely_snow"
    banks = {"champions_room", "mn_winter_north_shore", ely}
    require(changed(a["tilesets"], b["tilesets"]) == banks, "unrelated tileset")
    files, new_files = before["runtime_files"], after["runtime_files"]
    added = {"data/tilesets/" + name + "_metatiles.bin" for name in banks}
    require(
        changed(files, new_files)
        == added
        | {
            "data/tilesets/mn_winter_north_shore.json",
            "data/tilesets/mn_ely_snow.json",
            "data/tilesets/mn_ely_snow_palette_map.json",
            "data/tilesets/mn_ely_snow_foregrounds.json",
            "gfx/tilesets/mn_ely_snow.png",
            "gfx/tilesets/mn_ely_snow.2bpp",
        },
        "unrelated native graphics or animation",
    )
    for name in banks:
        old, new = (d["tilesets"][name] for d in (a, b))
        key = "data/tilesets/" + name + "_metatiles.bin"
        require(
            changed(old, new)
            == ({"collision", "palette_map"} if name == ely else {"collision"})
            and changed(old["collision"], new["collision"])
            == {f"{len(files[key]) // 16:02x}"}
            and len(new_files[key]) == len(files[key]) + 16
            and new_files[key][: len(files[key])] == files[key],
            f"stock {name} glyphs or permissions changed",
        )
    require(
        json.loads(bytes(new_files["data/tilesets/mn_winter_north_shore.json"]))
        == b["tilesets"]["mn_winter_north_shore"]["collision"],
        "split winter collision",
    )
    source, result = SourceArt(before), SourceArt(after)
    for time in ("morn", "day", "nite"):
        old_art, new_art = source.blocks(ely, time), result.blocks(ely, time)
        require(
            len(new_art) == len(old_art) + 1
            and all(
                original.tobytes() == candidate.tobytes()
                for original, candidate in zip(old_art, new_art)
            ),
            f"existing Ely art changed at {time}",
        )
        new_block = new_art[-1]
        require(
            all(
                len(new_block.crop((x, y, x + 8, y + 8)).getcolors(64)) <= 4
                for x in range(0, 32, 8)
                for y in range(0, 32, 8)
            ),
            f"snowline exceeds four colors at {time}",
        )
    for table, suffix in (("collision", ".json"), ("palette_map", "_palette_map.json")):
        require(
            b["tilesets"][ely][table]
            == json.loads(bytes(new_files[f"data/tilesets/{ely}{suffix}"])),
            f"split {ely} {table}",
        )
    print(
        f"{child.name}: {len(a['maps'])} maps compared, eight changed. "
        "Original art, animations, encounters, move tables, teams, event flag IDs and audio preserved."
    )


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("parent", type=Path)
    parser.add_argument("child", type=Path)
    args = parser.parse_args()
    verify(args.parent, args.child)
