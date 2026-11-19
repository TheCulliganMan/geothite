#!/usr/bin/env python3
"""Verify region-wide archive scope, native palette banks, animation, actors and edition-specific piers."""

import argparse
import base64
import io
import json
from pathlib import Path
import sys
from PIL import Image

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE.parents[1] / "tools"))
from audit_story_flow import require
from render_reference import load_pack, SourceArt

BANK = "mn_headwaters"
LAKE = "MnLakeItasca"
RANGER = "MN_BRAINERD_RANGER"


def changed(a, b):
    return {key for key in set(a) | set(b) if a.get(key) != b.get(key)}


def verify(parent, child):
    a, b = parent["data"], child["data"]
    require(
        changed(parent, child) == {"identity", "report", "runtime_files", "data"},
        "unrelated archive mutation",
    )
    require(
        changed(parent["report"], child["report"]) == {"manifests"}
        and child["report"]["manifests"]
        == parent["report"]["manifests"] + ["northstar-headwaters-journey-v1"],
        "archive lineage",
    )
    require(
        changed(a, b) <= {"maps", "map_scripts", "tilesets", "map_blocks", "npcs"},
        "non-map data changed",
    )
    require(changed(a["tilesets"], b["tilesets"]) == {BANK}, "unrelated tileset")
    old, new = a["tilesets"][BANK], b["tilesets"][BANK]
    require(
        changed(old, new) == {"palette_map", "collision"}
        and new["palette_map"] == old["palette_map"] + [13, 13],
        "two palette entries for the staff",
    )
    added = {
        **{str(n): ["WATER"] * 4 for n in (93, 94, 95)},
        "90": ["WATER", "WATER", "WALL", "WATER"],
        "91": ["WATER", "WATER", "BRIDGE_NS_DECK", "BRIDGE_NS_DECK"],
        "92": ["BRIDGE_NS_DECK", "BRIDGE_NS_DECK", "WATER", "WATER"],
    }
    require(
        changed(old["collision"], new["collision"]) == added.keys()
        and all(new["collision"][k] == v for k, v in added.items()),
        "only the six Itasca blocks may be new",
    )
    old_files, new_files = parent["runtime_files"], child["runtime_files"]
    gfx = "gfx/tilesets/" + BANK
    prefix = "data/tilesets/" + BANK
    require(
        changed(old_files, new_files)
        == {
            gfx + ".2bpp",
            gfx + ".png",
            prefix + "_metatiles.bin",
            prefix + ".json",
            prefix + "_palette_map.json",
        },
        "wildlife, audio or other runtime asset changed",
    )
    require(
        json.loads(bytes(new_files[prefix + ".json"])) == new["collision"]
        and json.loads(bytes(new_files[prefix + "_palette_map.json"]))
        == new["palette_map"],
        "split tileset records",
    )
    raw = bytes(old_files[gfx + ".2bpp"])
    larger = bytes(new_files[gfx + ".2bpp"])
    require(
        len(raw) == 192 * 16
        and len(larger) == 256 * 16
        and raw[: 96 * 16] == larger[: 96 * 16]
        and raw[96 * 16 :] == larger[128 * 16 : 224 * 16],
        "original 2bpp graphics moved or recolored",
    )
    atlas = bytes(old_files[prefix + "_metatiles.bin"])
    after = bytes(new_files[prefix + "_metatiles.bin"])
    require(
        after[:-96] == atlas and set(after[-96:]) - set(atlas) == {224, 225},
        "new blocks use the two new staff glyphs",
    )
    original_art, after_art = SourceArt(parent), SourceArt(child)
    for tod in ["morn", "day", "nite"]:
        x, y = original_art.blocks(BANK, tod), after_art.blocks(BANK, tod)
        require(
            len(x) == 144
            and len(y) == 150
            and all(i.tobytes() == j.tobytes() for i, j in zip(x, y)),
            f"original terrain pixels at {tod}",
        )
    # The two new tiles in the split PNG must decode to precisely the 2bpp graphics.
    png = Image.open(io.BytesIO(bytes(new_files[gfx + ".png"]))).convert("L")
    width = png.width // 8
    for tile in (224, 225):
        for row in range(8):
            lo, hi = larger[tile * 16 + row * 2 : tile * 16 + row * 2 + 2]
            for col in range(8):
                shift = 7 - col
                shade = (lo >> shift & 1) + 2 * (hi >> shift & 1)
                require(
                    png.getpixel((tile % width * 8 + col, tile // width * 8 + row))
                    == 255 - 85 * shade,
                    "staff tile plane mismatch",
                )
    expected = {
        LAKE,
        "MnItascaGatehouse",
        "MnStCloud",
        "MnStPaulUnionDepot",
        "MnBrainerd",
        "MnR07C00R00",
    }
    expected.update(
        n
        for n, r in a["maps"].items()
        if n.startswith("MnR20")
        and any(
            e["script"] in ("MN_R20_SIGN_GRANITE", "MN_R20_SIGN_MEADOW")
            for e in r["events"]["bg_events"]
        )
    )
    wrong_tree = old["collision"]["88"][2] == "HEADBUTT_TREE"
    if wrong_tree:
        expected |= {"MnLeechLake", "MnMilleLacs"}
    require(
        changed(a["maps"], b["maps"]) == expected,
        "unexpected set of affected regional rooms",
    )
    event_sections = set()
    world_scripts = set()
    block_labels = set()
    for n in expected:
        before, end = a["maps"][n], b["maps"][n]
        require(
            before["attributes"] == end["attributes"],
            f"attributes, dimensions and exits {n}",
        )
        require(
            before["events"]["warps"] == end["events"]["warps"]
            and before["events"]["coord_events"] == end["events"]["coord_events"],
            f"warps and coordinate triggers {n}",
        )
        allowable = {
            "scripts",
            "script_text_bodies",
            "script_text_commands",
            "script_control_commands",
            "script_movements",
            "script_object_commands",
            "script_flag_commands",
            "script_runtime_commands",
        }
        if n == LAKE:
            allowable |= {"blocks", "events", "map_event_section_commands"}
            old_bg, new_bg = before["events"]["bg_events"], end["events"]["bg_events"]
            require(
                new_bg[:-1] == old_bg
                and new_bg[-1]
                == {
                    "x": 22,
                    "y": 21,
                    "event_type": "BGEVENT_READ",
                    "script": "MN_ITASCA_MARKER",
                },
                "new gauge interaction",
            )
        elif n == "MnBrainerd":
            allowable |= {"objects", "map_event_section_commands"}
            require(len(before["objects"]) == len(end["objects"]), "actor count")
            for left, right in zip(before["objects"], end["objects"]):
                if left["object_identifier"] == RANGER:
                    require(
                        changed(left, right) == {"x", "y"}
                        and (right["x"], right["y"]) == (49, 20),
                        "sole ranger move",
                    )
                else:
                    require(left == right, "unrelated Brainerd actor")
        elif n in ("MnLeechLake", "MnMilleLacs"):
            allowable = {"blocks"}
        require(
            changed(before, end) <= allowable,
            f"unexpected changed tables in {n}: {changed(before, end) - allowable}",
        )
        delta = changed(before["scripts"], end["scripts"])
        world_scripts.update(delta)
        for label in delta:
            require(
                end["scripts"].get(label) == b["map_scripts"].get(label),
                f"split script {label}",
            )
        if before["map_event_section_commands"] != end["map_event_section_commands"]:
            label = end["attributes"]["map_events_label"]
            event_sections.add(label)
            require(
                b["map_scripts"][label]
                == [
                    {k: r[k] for k in ("command", "args")}
                    for r in end["map_event_section_commands"]
                ],
                f"split event section {n}",
            )
        if before["blocks"] != end["blocks"]:
            w = before["attributes"]["width"]
            require(len(before["blocks"]) == len(end["blocks"]), "map size")
            delta = {
                (i % w, i // w, old_id, new_id)
                for i, (old_id, new_id) in enumerate(
                    zip(before["blocks"], end["blocks"])
                )
                if old_id != new_id
            }
            if n == LAKE:
                require(
                    delta
                    == {
                        (11, 10, 0x58, 0x90),
                        (11, 12, 0x82, 0x91),
                        (11, 13, 0x81, 0x92),
                        (11, 14, 0x54, 0x93),
                        (12, 10, 0x59, 0x94),
                        (11, 18, 0x58, 0x95),
                        *((11, y, 0x58, 0x93) for y in (11, 15, 16, 17)),
                    },
                    "headwaters staff, deck and east channel edge",
                )
            else:
                xs, y = (
                    (range(17, 21), 10) if n == "MnLeechLake" else (range(10, 15), 11)
                )
                require(
                    delta == {(x, y, 0x88, 0x89) for x in xs},
                    "correct existing timber donor",
                )
            label = (
                end["attributes"]["block_label"]
                if "block_label" in end["attributes"]
                else n + "_Blocks"
            )
            block_labels.add(label)
            require(
                base64.b64decode(b["map_blocks"][label]) == bytes(end["blocks"]),
                "split blocks",
            )
    require(
        changed(a["map_scripts"], b["map_scripts"]) == world_scripts | event_sections,
        "unrelated global script changes",
    )
    require(
        changed(a["map_blocks"], b["map_blocks"]) == block_labels,
        "unrelated global block changes",
    )
    require(
        changed(a["npcs"], b["npcs"]) == {"MnBrainerd"}
        and b["npcs"]["MnBrainerd"] == b["maps"]["MnBrainerd"]["objects"],
        "unrelated global actor changes",
    )
    print(
        f"Compared all {len(a['maps'])} maps, {len(a['trainers']['trainers'])} trainer teams and {len(old_files)} runtime files. "
        f"{len(expected)} maps changed. All original 144 headwaters blocks retain their pixels at three times of day; all animation programs and audio are identical."
    )


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("parent", type=Path)
    parser.add_argument("child", type=Path)
    args = parser.parse_args()
    verify(load_pack(args.parent), load_pack(args.child))
