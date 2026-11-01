#!/usr/bin/env python3
"""Read-only independent preservation audit for the native Safari overlay."""

import argparse
from collections import deque
from copy import deepcopy
from datetime import datetime, timezone
import hashlib
import json
from pathlib import Path

from render_reference import load_pack


GUARDS = ("MN_BOUNDARY_PENDING_GATE_WEST", "MN_BOUNDARY_PENDING_GATE_EAST")


def audit_thicket(pack, previous, zone):
    """Check the reward change and the entire closed/open walking/Surf graph."""
    assert {key: value for key, value in previous.items() if key != "habitats"} == {
        key: value for key, value in zone.items() if key != "habitats"}
    data, cells, bodies = pack["data"], {}, set()
    for name, placement in zone["maps"].items():
        if placement.get("origin_core") is None:
            continue
        ox, oy = placement["origin_core"]
        module = data["maps"][name]
        width, height = module["attributes"]["width"], module["attributes"]["height"]
        collision = data["tilesets"][module["attributes"]["tileset_name"]]["collision"]
        for y in range(height * 2):
            for x in range(width * 2):
                block = module["blocks"][y // 2 * width + x // 2]
                cells[ox + x, oy + y] = collision[f"{block:02x}"][y % 2 * 2 + x % 2]
        bodies.update((ox + actor["x"], oy + actor["y"]) for actor in module["objects"]
                      if actor["object_identifier"] not in zone["gate_court"]["guard_identifiers"])

    def at(config, point):
        x, y = point
        habitats = [h for h in config["habitats"]
                    if h["bounds_core"][0][0] <= x < h["bounds_core"][1][0]
                    and h["bounds_core"][0][1] <= y < h["bounds_core"][1][1]]
        assert len(habitats) == 1, f"habitat coverage at {point}"
        return habitats[0]

    def table(habitat):
        return {key: value for key, value in habitat.items() if key not in {"id", "bounds_core"}}

    grass, outside = set(), 0
    for point, kind in cells.items():
        before, after = at(previous, point), at(zone, point)
        if after["id"] == "PINE_THICKET":
            assert after["bounds_core"] == [[0, 30], [16, 38]]
            assert before["id"] == "PINE_PORTAGE"
            assert kind != "WATER", "new odds must not leak into the adjacent pocket lake"
            if kind == "TALL_GRASS":
                grass.add(point)
        else:
            assert table(before) == table(after), f"outside encounter behavior changed at {point}"
            outside += 1
    assert len(grass) == 52 and (9, 35) in grass
    thicket = at(zone, (9, 35))
    assert thicket["grass_rate"] == 48 and thicket["water_rate"] == 12
    weights = {slot["species"]: slot["weight"] for slot in thicket["grass"]}
    assert weights == {"PINIT": 15, "NOTCHBILL": 10, "SHEDSTAG": 45, "GOUGECREST": 30}
    old = at(previous, (9, 35))
    assert thicket["water"] == old["water"]
    for slot in thicket["grass"]:
        original = next(s for s in old["grass"] if s["species"] == slot["species"])
        assert {k: v for k, v in slot.items() if k != "weight"} == {
            k: v for k, v in original.items() if k != "weight"}

    def flood(grid):
        seen, pending = {(3, 89)}, deque([(3, 89)])
        while pending:
            x, y = pending.popleft()
            adjacent = [(x - 1, y), (x + 1, y), (x, y - 1), (x, y + 1)]
            if grid[x, y] == "HOP_DOWN":
                adjacent.append((x, y + 2))
            for point in adjacent:
                if point not in seen and point not in bodies and grid.get(point) in {
                    "FLOOR", "TALL_GRASS", "WATER", "HOP_DOWN"}:
                    seen.add(point)
                    pending.append(point)
        return seen

    assert cells[10, 31] == "CUT_TREE"
    assert not (flood(cells) & grass), "uncut thicket has a land/Surf/ledge bypass"
    replacement = data["field_moves"]["cut"]["replacements"]["mn_boundary_safari"][0x63]
    assert replacement == {"replacement_block_id": 0x3f, "variant": "tree"}
    collision = data["tilesets"]["mn_boundary_safari"]["collision"]
    assert all(collision["63"][i] == collision["3f"][i] for i in [0, 1, 3])
    opened = cells.copy()
    for index, kind in enumerate(collision["3f"]):
        opened[10 + index % 2, 30 + index // 2] = kind
    assert grass <= flood(opened), "native Cut does not open every thicket grass tile"

    root = Path(__file__).resolve().parents[3]
    roster = json.loads((root / "planning/minnesota/pokemon/roster.json").read_text())
    curation = json.loads((root / "planning/minnesota/pokemon/roster-curation.json").read_text())
    retired = {family["id"] for family in curation["retired_families"]}
    active = {member["name"].upper() for family in roster["families"]
              if family["id"] not in retired for member in family["members"]}
    species = {slot["species"] for habitat in zone["habitats"]
               for surface in ["grass", "water"] for slot in habitat[surface]}
    assert species | {"GILLPUP", "VOYAGEAR"} <= active, "non-active Minnesota species"
    return {"passed": True, "gated_grass_tiles": len(grass), "outside_cells_preserved": outside,
            "native_cut": "63→3f; other three quadrants unchanged",
            "species_weights": weights, "active_minnesota_species_checked": sorted(species),
            "scope": "Conservative static land/Surf/down-hop graph; native controller proof is separate"}


def audit(parent, candidate, zone):
    old, new = parent["data"], candidate["data"]
    assert old.get("safari_zone") is None, "parent already admits expeditions"
    compiled_zone = new["safari_zone"].copy()
    # Rust omits the native default threshold from its serialized form.
    compiled_zone.setdefault("fishing_flee_threshold", 64)
    assert compiled_zone == zone, "published configuration differs from source"
    labels = {f"{name}Text" for name in zone["gate_court"]["guard_identifiers"]}
    labels |= {f"{name}Text" for name in zone["retire_scripts"]}
    labels |= {f"{name}Text" for name in GUARDS}
    labels |= {
        "BWS_SIGN_01Text", "BWS_SIGN_06Text", "MN_COHESION_PRESERVE_NOTICEText",
        "MN_BOUNDARY_GATE_NOTICEText", f'{zone["admission_script"]}Text',
    }
    assert len(labels) == 12
    restored = deepcopy(candidate)
    if "safari_zone" in old:
        restored["data"]["safari_zone"] = None
    else:
        restored["data"].pop("safari_zone")
    changed_maps, changed_labels = [], set()
    assert old["maps"].keys() == new["maps"].keys()
    for name, before in old["maps"].items():
        after = new["maps"][name]
        if before == after:
            continue
        changed_maps.append(name)
        expected = deepcopy(before)
        assert before["scripts"].keys() == after["scripts"].keys()
        assert before["script_text_bodies"].keys() == after["script_text_bodies"].keys()
        for label in before["scripts"]:
            if before["scripts"][label] == after["scripts"][label]:
                continue
            assert label in labels, f"unrelated program changed: {name}/{label}"
            assert before["script_text_bodies"][label] != after["script_text_bodies"][label]
            changed_labels.add(label)
            expected["scripts"][label] = after["scripts"][label]
            expected["script_text_bodies"][label] = after["script_text_bodies"][label]
            assert new["map_scripts"][label] == after["scripts"][label]
            restored["data"]["map_scripts"][label] = old["map_scripts"][label]
        if name == zone["admission_map"]:
            assert [o["script"] for o in before["objects"][3:5]] == list(GUARDS)
            assert [(o["x"], o["y"]) for o in before["objects"][3:5]] == [(32, 6), (33, 6)]
            expected["objects"] = before["objects"][:3] + before["objects"][5:]
            assert after["objects"] == expected["objects"], "retained NPC changed or lost"
            assert after["objects"][2] == before["objects"][2], "ranger index changed"
            rows = [r.copy() for r in before["map_event_section_commands"]
                    if not (r["command"] == "object_event" and r["args"][11] in GUARDS)]
            assert len(rows) == len(before["map_event_section_commands"]) - 2
            for index, row in enumerate(rows):
                row["command_index"] = index
            expected["map_event_section_commands"] = rows
        assert expected == after, f"unrelated map field changed: {name}"
        restored["data"]["maps"][name] = before
    assert changed_labels == labels, "expected guidance was not updated exactly once"
    header = f'{zone["admission_map"]}_MapEvents'
    if header in old["map_scripts"]:
        expected = [row for row in old["map_scripts"][header]
                    if not (row["command"] == "object_event" and row["args"][11] in GUARDS)]
        assert len(expected) == len(old["map_scripts"][header]) - 2
        assert new["map_scripts"][header] == expected
        restored["data"]["map_scripts"][header] = old["map_scripts"][header]
    assert candidate["report"]["manifests"] == (
        parent["report"]["manifests"] + ["native-safari-expedition-v4"])
    restored["report"]["manifests"] = parent["report"]["manifests"]
    assert candidate["identity"] != parent["identity"]
    restored["identity"] = parent["identity"]
    assert restored == parent, "unrelated data, art, audio, or report changed"
    return {
        "passed": True, "whole_maps_unchanged": len(old["maps"]) - len(changed_maps),
        "changed_maps": changed_maps, "text_labels": sorted(changed_labels),
        "removed_basecamp_guards": list(GUARDS),
        "preserved": "All terrain, retained actors, events, executable programs, non-Safari data, art and audio",
    }


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ("parent", "candidate", "zone", "output"):
        parser.add_argument(f"--{name}", type=Path, required=True)
    parser.add_argument("--previous-zone", type=Path,
                        help="Verify the isolated thicket reward against earlier encounter behavior")
    args = parser.parse_args()
    assert not args.output.exists(), "preserve earlier receipts"
    candidate = load_pack(args.candidate)
    zone = json.loads(args.zone.read_text())
    result = audit(load_pack(args.parent), candidate, zone)
    if args.previous_zone:
        result["thicket"] = audit_thicket(candidate, json.loads(args.previous_zone.read_text()), zone)
    result["recorded_utc"] = datetime.now(timezone.utc).isoformat()
    result["inputs"] = {
        name: {"path": str(path), "sha256": hashlib.sha256(path.read_bytes()).hexdigest()}
        for name, path in (("parent", args.parent), ("candidate", args.candidate), ("zone", args.zone))
    }
    if args.previous_zone:
        root = Path(__file__).resolve().parents[3]
        for name, path in [
            ("previous_zone", args.previous_zone),
            ("roster", root / "planning/minnesota/pokemon/roster.json"),
            ("curation", root / "planning/minnesota/pokemon/roster-curation.json"),
        ]:
            result["inputs"][name] = {
                "path": str(path), "sha256": hashlib.sha256(path.read_bytes()).hexdigest()}
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(json.dumps(result, indent=2) + "\n")
    print(json.dumps({key: result[key] for key in ("passed", "whole_maps_unchanged", "changed_maps")}))


if __name__ == "__main__":
    main()
