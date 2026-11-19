#!/usr/bin/env python3
"""Compare the entire Badge Trails archive against its own Shore Watch parent."""

import argparse
import base64
from pathlib import Path
import sys

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE.parents[1] / "tools"))
from render_reference import load_pack

HOLLOW = "MnR01Cave1F"
LABELS = {
    HOLLOW: {"MN_R01_HOLLOW_GUIDEText", "MN_R01_HOLLOW_GUIDEFieldHMGivenText"},
    "MnStPaulGym": {
        "MnStPaulGymLeader",
        "MnStPaulGymCourtRequirements",
        "MnStPaulGymRequirementsText",
        "MnStPaulGymRematchBeforeChampion",
        "MnBadgeBrannicMina",
        "MnBadgeBrannicMinaText",
        "MnBadgeBrannicBreeze",
        "MnBadgeBrannicBreezeText",
        "MnBadgeBrannicNextGym",
        "MnBadgeBrannicWindText",
    },
    "MnBrainerdGym": {
        "MnBrainerdGymRematchBeforeChampion",
        "MnBadgeSivrenItasca",
        "MnBadgeSivrenItascaText",
        "MnBadgeSivrenOfficer",
        "MnBadgeSivrenOfficerText",
        "MnBadgeSivrenWake",
        "MnBadgeSivrenWakeText",
    },
    "MnDuluthGym": {"MnDuluthGymRequirementsText"},
}


def changed(before, after):
    return {
        key for key in before.keys() | after.keys() if before.get(key) != after.get(key)
    }


def verify(parent, child):
    a, b = parent["data"], child["data"]
    assert changed(parent, child) == {"data", "identity", "report", "runtime_files"}
    assert changed(parent["report"], child["report"]) == {"manifests"}
    assert child["report"]["manifests"] == [
        *parent["report"]["manifests"],
        "northstar-badge-trails-v1",
    ]
    assert changed(parent["identity"], child["identity"]) == {
        "runtime_modpack_id",
        "content_hash",
    }
    assert changed(a, b) == {"maps", "map_scripts", "map_blocks", "tilesets"}
    assert changed(parent["runtime_files"], child["runtime_files"]) == {
        "data/tilesets/cave_metatiles.bin",
        "data/tilesets/cave.json",
    }
    assert changed(a["tilesets"], b["tilesets"]) == {"cave"}
    assert changed(a["tilesets"]["cave"], b["tilesets"]["cave"]) == {"collision"}
    assert changed(
        a["tilesets"]["cave"]["collision"], b["tilesets"]["cave"]["collision"]
    ) == {"4c"}
    assert changed(a["maps"], b["maps"]) == set(LABELS)
    assert changed(a["map_blocks"], b["map_blocks"]) == {HOLLOW + "_Blocks"}
    assert base64.b64decode(
        b["map_blocks"][HOLLOW + "_Blocks"], validate=True
    ) == bytes(b["maps"][HOLLOW]["blocks"])
    assert changed(a["map_scripts"], b["map_scripts"]) == set.union(*LABELS.values())
    for name, labels in LABELS.items():
        old, new = a["maps"][name], b["maps"][name]
        fields = {"scripts", "script_text_bodies"}
        if name == HOLLOW:
            fields.add("blocks")
        if name in {"MnStPaulGym", "MnBrainerdGym"}:
            fields.update(
                {
                    "script_control_commands",
                    "script_flag_commands",
                    "script_text_commands",
                }
            )
        assert changed(old, new) == fields, name
        assert changed(old["scripts"], new["scripts"]) == labels, name
        for label in labels:
            assert new["scripts"][label] == b["map_scripts"][label], (name, label)
        assert changed(old["script_text_bodies"], new["script_text_bodies"]) == {
            label for label in labels if label.endswith("Text")
        }, name
        for field in fields & {
            "script_flag_commands",
            "script_text_commands",
            "script_control_commands",
        }:
            assert [r for r in old[field] if r["source_script"] not in labels] == [
                r for r in new[field] if r["source_script"] not in labels
            ], (name, field)
    print(
        f"{len(b['maps'])} maps checked. Exactly four changed, with 20 changed or new scripts. "
        "All warps, parties, encounter tables, audio, animations and existing save flags are retained."
    )


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("parent", type=Path)
    parser.add_argument("candidate", type=Path)
    args = parser.parse_args()
    verify(load_pack(args.parent), load_pack(args.candidate))
