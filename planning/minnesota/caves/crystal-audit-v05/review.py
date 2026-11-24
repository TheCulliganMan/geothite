#!/usr/bin/env python3
"""Check the installed v05 caves against declared layouts, events, native art and links."""

import argparse
from collections import Counter
from copy import deepcopy
from importlib.util import module_from_spec, spec_from_file_location
import json
from pathlib import Path
import sys

SOURCE = Path(__file__).resolve().parent
REPLACEMENT = SOURCE.with_name("crystal-replacement-v02")
DETAIL = SOURCE.with_name("crystal-detail-v04")
sys.path.insert(0, str(REPLACEMENT))
sys.path.insert(0, str(SOURCE.parents[1] / "tools"))

from render_archive import save_render  # noqa: E402
from render_progress_atlas import native_room  # noqa: E402
from render_reference import load_pack, room_sheet  # noqa: E402
from review import check_room  # noqa: E402
from review_ice import inspect as inspect_ice  # noqa: E402

GROUPS = ("south", "north", "mines", "ice")
ART_ORDER = ("north", "south", "mines", "ice")
EVENTS = {
    "warp_event": "warps",
    "bg_event": "bg_events",
    "coord_event": "coord_events",
}


def read(path):
    return json.loads(path.read_text())


def audit_art(parent, installed):
    sys.path.insert(0, str(DETAIL))
    spec = spec_from_file_location("detail_review", DETAIL / "review.py")
    module = module_from_spec(spec)
    spec.loader.exec_module(module)
    module.audit_art(parent, installed, source=SOURCE, group_order=ART_ORDER)


def check_event_commands(before, after):
    expected = deepcopy(before["map_event_section_commands"])
    commands = {command: iter(after["events"][field]) for command, field in EVENTS.items()}
    commands["object_event"] = iter(after["objects"])
    for command in expected:
        if command["command"] in commands:
            event = next(commands[command["command"]])
            command["args"][:2] = [str(event[k]) for k in ("x", "y")]
    assert expected == after["map_event_section_commands"], after["id"]
    assert all(next(events, None) is None for events in commands.values()), after["id"]


def audit_maps(parent, installed, groups):
    old, new = (pack["data"] for pack in (parent, installed))
    names = {layout["map"] for rooms in groups.values() for layout in rooms}
    assert len(names) == sum(map(len, groups.values())) == 19
    assert old["maps"].keys() == new["maps"].keys()
    assert {name for name in old["maps"] if old["maps"][name] != new["maps"][name]} == names
    expected_splits = deepcopy({key: old[key] for key in (
        "map_attributes", "map_blocks", "map_dimensions", "runtime_map_metadata", "map_scripts", "npcs"
    )})
    for key in ("audio_manifest", "compiled_audio", "audio_compression"):
        assert parent[key] == installed[key], key
    assert installed["report"] == dict(
        parent["report"], manifests=parent["report"]["manifests"] + ["northstar-cave-audit-v5"]
    )
    assert old.keys() == new.keys()
    for key in old.keys() - expected_splits.keys() - {"tilesets", "maps"}:
        assert old[key] == new[key], key

    constants = {room["attributes"]["map_constant"]: name for name, room in new["maps"].items()}
    counts = Counter()
    for group, rooms in groups.items():
        for layout in rooms:
            name = layout["map"]
            before, after = check_room(parent, installed, layout, require_new_footprint=False)
            assert before["blocks"] != after["blocks"], name
            assert before["events"].keys() == after["events"].keys(), name
            for field in before["events"].keys() - set(EVENTS.values()):
                assert before["events"][field] == after["events"][field], (name, field)
            check_event_commands(before, after)
            attrs = after["attributes"]
            constant = attrs["map_constant"]
            label = attrs["blocks_label"]
            dims = {"width": attrs["width"], "height": attrs["height"]}
            expected_splits["map_attributes"][name] = attrs
            expected_splits["map_blocks"][label] = new["map_blocks"][label]
            expected_splits["map_dimensions"][constant] = dims
            expected_splits["runtime_map_metadata"][constant].update(dims)
            expected_splits["npcs"][name] = after["objects"]
            for label, section in (("map_scripts_label", "map_script_section_commands"),
                                   ("map_events_label", "map_event_section_commands")):
                expected_splits["map_scripts"][attrs[label]] = [
                    {key: command[key] for key in ("command", "args")} for command in after[section]
                ]
            headers = {attrs["map_scripts_label"], attrs["map_events_label"]}
            for script in after["scripts"].keys() - headers:
                assert new["map_scripts"][script] == after["scripts"][script], (name, script)
                expected_splits["map_scripts"][script] = after["scripts"][script]
            for patch in layout.get("script_patches", ()):
                if after["scripts"][patch["script"]][patch["index"]]["command"] == "changeblock":
                    block = tuple(map(int, patch["after_args"]))
                    assert any(
                        (row["x"], row["y"], row["block_id"]) == block
                        and row["source_script"] == patch["script"] for row in after["script_block_changes"]
                    ), (name, patch)
            for warp in after["events"]["warps"]:
                peer = constants.get(warp["target_map_constant"], warp["target_map"])
                arrivals = new["maps"][peer]["events"]["warps"]
                assert sum(event["index"] == warp["target_warp_id"] for event in arrivals) == 1, (name, warp)
                counts["warps"] += 1
            old_dims = [before["attributes"][key] for key in ("width", "height")]
            counts["resized"] += old_dims != layout["dimensions_metatiles"]
            counts["interiors"] += 1
            if group == "ice":
                inspect_ice(installed, after, layout)
    for key, expected in expected_splits.items():
        differences = {name for name in expected.keys() | new[key].keys()
                       if expected.get(name) != new[key].get(name)}
        assert not differences, (key, sorted(differences))
    print(dict(counts))


def render(groups, parent, installed, output):
    output.mkdir(parents=True)
    for group, rooms in groups.items():
        scenes = []
        for layout in rooms:
            name = layout["map"]
            pair = []
            for version, pack in (("v04", parent), ("v05", installed)):
                room = pack["data"]["maps"][name]
                attrs = room["attributes"]
                label = f"{name} {version}"
                if attrs["time_of_day"] == "dark":
                    room = dict(room, attributes=dict(attrs, time_of_day="nite"))
                    label += " (Flash)"
                pair.append((label, native_room(room, pack, native_origin=True)))
            scenes.extend(pair)
            save_render(room_sheet(pair, columns=2), output / f"{name}.png")
        save_render(room_sheet(scenes, columns=2), output / f"{group}.png")


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("parent", type=Path)
    parser.add_argument("installed", type=Path)
    parser.add_argument("--renders", type=Path)
    args = parser.parse_args()
    groups = {group: read(SOURCE / f"{group}.json") for group in GROUPS}
    parent, installed = map(load_pack, (args.parent, args.installed))
    audit_art(parent, installed)
    audit_maps(parent, installed, groups)
    if args.renders:
        render(groups, parent, installed, args.renders)
