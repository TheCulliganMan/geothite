#!/usr/bin/env python3
"""Inspect installed replacement caves and draw full floorplans from native art."""

import argparse
from base64 import b64decode
from copy import deepcopy
import hashlib
import json
from pathlib import Path
import sys

sys.path.insert(0, str(Path(__file__).resolve().parents[2] / "tools"))
from render_archive import save_render  # noqa: E402
from render_reference import SourceArt, load_pack, room_sheet  # noqa: E402

SOURCE = Path(__file__).parent
SAVE_POLICY = "data/northstar-save-compatibility.json"


def layouts():
    return [room for name in ("south", "north", "mines", "ice")
            for room in json.loads((SOURCE / f"{name}.json").read_text())]


def check_room(parent, result, layout, *, require_new_footprint=True):
    name = layout["map"]
    before, after = [p["data"]["maps"][name] for p in (parent, result)]
    original = before["attributes"]
    attributes = after["attributes"]
    width, height = layout["dimensions_metatiles"]
    if require_new_footprint:
        assert [width, height] != [original["width"], original["height"]], (name, "reused footprint")
    expected = {**original, "width": width, "height": height}
    if "time_of_day" in layout:
        expected["time_of_day"] = layout["time_of_day"]
        expected["palette"] = "PALETTE_" + layout["time_of_day"].upper()
    if "border_block" in layout:
        expected["border_block"] = layout["border_block"]
    assert attributes == expected, name
    rows = [row.split() for row in layout["rows"]]
    assert len(rows) == height and all(len(row) == width for row in rows), name
    blocks = [int(cell, 16) for row in rows for cell in row]
    assert blocks == after["blocks"], name
    assert hashlib.sha256(bytes(before["blocks"])).hexdigest() == layout["before_blocks_sha256"], name
    payload = result["data"]["map_blocks"][attributes["blocks_label"]]
    assert bytes(blocks) == b64decode(payload, validate=True), name
    assert result["data"]["map_attributes"][name] == attributes, name
    constant = attributes["map_constant"]
    assert result["data"]["map_dimensions"][constant] == {"width": width, "height": height}, name

    # A relocated event must retain its target, flag and script identity. Check
    # the full structs, including positions that the design elected to keep.
    event_types = (
        ("warps", "index", before["events"]["warps"], after["events"]["warps"]),
        ("actors", "script", before["objects"], after["objects"]),
        ("backgrounds", "script", before["events"]["bg_events"], after["events"]["bg_events"]),
        ("coordinates", "script_name", before["events"]["coord_events"], after["events"]["coord_events"]),
    )
    for key, field, old_events, actual in event_types:
        expected = deepcopy(old_events)
        for move in layout.get(key, []):
            index, = (i for i, event in enumerate(old_events)
                      if [event["x"], event["y"]] == move["from"]
                      and event[field] == move.get("index", move.get("script")))
            expected[index]["x"], expected[index]["y"] = move["to"]
        assert expected == actual, (name, key)
    scripts = deepcopy(before["scripts"])
    for patch in layout.get("script_patches", []):
        command = scripts[patch["script"]][patch["index"]]
        assert command["args"] == patch["before_args"], (name, patch)
        command["args"] = patch["after_args"]
    assert scripts == after["scripts"], name
    for section in ("scenes", "trainer_scripts", "scripted_wild_battles", "scripted_trainer_battles",
                    "script_item_grants", "script_field_pickups", "gift_pokemon_scripts",
                    "script_flag_commands", "map_script_section_commands"):
        assert before[section] == after[section], (name, section)
    return before, after


def review(parent, result, output):
    batch = layouts()
    names = {layout["map"] for layout in batch}
    old, new = parent["data"], result["data"]
    assert len(batch) == len(names) == 19
    assert old["maps"].keys() == new["maps"].keys()
    assert names == {name for name in old["maps"] if old["maps"][name] != new["maps"][name]}
    assert old.keys() == new.keys()
    for key in old.keys() - {"maps", "map_blocks", "map_attributes", "map_dimensions", "map_scripts", "npcs", "runtime_map_metadata"}:
        assert old[key] == new[key], key
    for key in ("audio_manifest", "compiled_audio", "audio_compression"):
        assert parent[key] == result[key], key
    assert {name: raw for name, raw in parent["runtime_files"].items() if name != SAVE_POLICY} == result["runtime_files"]

    art = SourceArt(parent)
    scenes = []
    warps = 0
    constants = {room["attributes"]["map_constant"]: name for name, room in new["maps"].items()}
    for layout in batch:
        versions = check_room(parent, result, layout)
        updated = versions[1]
        for warp in updated["events"]["warps"]:
            peer = constants.get(warp["target_map_constant"], warp["target_map"])
            arrivals = new["maps"][peer]["events"]["warps"]
            assert any(event["index"] == warp["target_warp_id"] for event in arrivals), (layout["map"], warp)
            warps += 1
        images = []
        for tag, room in zip(("Fair floorplan", "Replacement floorplan"), versions):
            attrs = room["attributes"]
            group = attrs["time_of_day"] or "day"
            if group in {"dark", "auto"}:
                group = "nite"
            image = art.render(attrs["tileset_name"], attrs["width"], attrs["height"], room["blocks"], group)
            images.append((tag, image))
        save_render(room_sheet(images, columns=2), output / (layout["map"] + ".png"))
        scenes.append((layout["map"], images[1][1]))
    save_render(room_sheet(scenes, columns=4), output / "replacement-floorplans.png")
    print(f"{len(batch)} new footprints and {warps} resolved cave warp records")


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ("parent", "candidate", "output"):
        parser.add_argument(name, type=Path)
    args = parser.parse_args()
    review(load_pack(args.parent), load_pack(args.candidate), args.output)
