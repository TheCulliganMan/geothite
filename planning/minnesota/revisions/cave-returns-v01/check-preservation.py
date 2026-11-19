#!/usr/bin/env python3
"""Verify each Cave Returns edition against its Gym Walk parent."""

import argparse
from pathlib import Path
import sys

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE.parents[1] / "tools"))
from render_reference import load_pack

ROOMS = {"MnR03Cave1F", "MnLeagueLobby"}
SHOP = "MART_MN_DULUTH_X_DOCKSUPPLIES_F1_HOST"


def changed(old, new):
    return {key for key in old.keys() | new.keys() if old.get(key) != new.get(key)}


def verify(parent, child):
    a, b = load_pack(parent), load_pack(child)
    assert changed(a, b) == {"identity", "report", "data", "runtime_files"}
    assert changed(a["identity"], b["identity"]) == {
        "content_hash",
        "runtime_modpack_id",
    }
    assert changed(a["report"], b["report"]) == {"manifests"}
    assert b["report"]["manifests"] == a["report"]["manifests"] + [
        "northstar-cave-returns-v1"
    ]
    old, new = a["data"], b["data"]
    assert changed(old, new) == {
        "marts",
        "maps",
        "map_blocks",
        "map_scripts",
        "tilesets",
    }
    assert (
        old["maps"].keys() == new["maps"].keys()
        and changed(old["maps"], new["maps"]) == ROOMS
    )
    assert changed(old["marts"], new["marts"]) == {SHOP}
    assert new["marts"][SHOP] == old["marts"][SHOP] + ["ESCAPE_ROPE", "POTION"]
    assert changed(old["tilesets"], new["tilesets"]) == {"cave"}
    assert changed(old["map_blocks"], new["map_blocks"]) == {
        new["maps"]["MnR03Cave1F"]["attributes"]["blocks_label"]
    }
    labels = set()
    for name in ROOMS:
        before, after = old["maps"][name], new["maps"][name]
        for key in [
            "attributes",
            "events",
            "objects",
            "trainer_scripts",
            "scripted_trainer_battles",
            "scripted_wild_battles",
        ]:
            assert before[key] == after[key], (name, key)
        for label in changed(before["scripts"], after["scripts"]):
            assert new["map_scripts"][label] == after["scripts"][label], (name, label)
            labels.add(label)
    assert changed(old["map_scripts"], new["map_scripts"]) == labels
    assert changed(
        old["maps"]["MnLeagueLobby"]["scripts"], new["maps"]["MnLeagueLobby"]["scripts"]
    ) == {"MnElyRegistrationText"}
    assets = changed(a["runtime_files"], b["runtime_files"])
    expected = {
        "data/tilesets/cave.json",
        "data/tilesets/cave_metatiles.bin",
        "data/tilesets/cave_palette_map.json",
        "gfx/tilesets/cave.2bpp",
        "gfx/tilesets/cave.png",
    }
    assert (
        assets <= expected
        and {"gfx/tilesets/cave.2bpp", "gfx/tilesets/cave.png"} <= assets
    ), assets
    print(
        f"{len(old['maps'])} maps compared; only Route 3's cave, the League desk, the dock stock and the cave artwork changed."
    )


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("parent", type=Path)
    parser.add_argument("child", type=Path)
    args = parser.parse_args()
    verify(args.parent, args.child)
