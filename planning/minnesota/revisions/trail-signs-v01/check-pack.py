#!/usr/bin/env python3
"""Check all seven trail signs, their conversations, approaches and archive differences."""
import argparse
import base64
from copy import deepcopy
import hashlib
import json
from pathlib import Path
import runpy

from PIL import ImageChops

HERE = Path(__file__).resolve().parent
PREVIOUS = HERE.parent / "research-choices-v01"
COMMON = runpy.run_path(str(HERE.parent / "ely-return-v01" / "check-pack.py"))
WALKS = runpy.run_path(str(HERE.parent / "lake-observations-v01" / "check-hidden.py"))
from render_reference import SourceArt

only, read_pack = COMMON["only"], COMMON["read_pack"]
quadrant, adjacent, walk = (WALKS[k] for k in ("quadrant", "adjacent", "walk"))
CITY = "MnMinneapolis"
CEDAR = "MnR08C00R01"
SIGNS = {
    CITY: {"MN_MP_ARRIVALINFO_SIGN": ((40, 60), (40, 58))},
    "MnDetroitLakesApproach": {"MN_COHESION_DETROIT_SIGN": ((8, 13), (8, 12))},
    "MnPreservePineEntry": {"MN_COHESION_PRESERVE_NOTICE": ((13, 11), (12, 10))},
    "MnEly": {"MnElyTrailSign": ((20, 3), (21, 3)), "MnElyCampSign": ((14, 21), (15, 21))},
}
# Native map seams and door arrivals paired with the square the player reads from.
WALKS_BY_MAP = {
    CITY: [([(40, 63), (43, 60)], (40, 59)), ([(40, 63), (56, 52)], (55, 52))],
    "MnDetroitLakesApproach": [([(0, 18), (47, 18)], (8, 13))],
    "MnPreservePineEntry": [([(16, 0), (16, 31)], (12, 11))],
    "MnEly": [([(24, 0), (11, 22)], (21, 4)), ([(24, 0), (11, 22)], (15, 22))],
    CEDAR: [([(42, 0), (26, 47)], (22, 5))],
}
# Daytime RGB of the actual 16x16 board faces in the pinned parents.
JOHTO_BOARD = "9a2982b6d6fe5c150883a690a524859c5be96250361636b80851dfe131cd1895"
ELY_BOARD = "369adda0182a2bb9c1f7336e72c39790433a2fdbd929ff76fdf856e2d4d95732"


def check_blocks(old, new, art, changes):
    labels = set()
    for name, edits in changes.items():
        before, after = old["maps"][name], new["maps"][name]
        a = before["attributes"]
        expected = list(before["blocks"])
        for x, y, original, replacement in edits:
            index = y * a["width"] + x
            assert expected[index] == original
            expected[index] = replacement
        assert after["blocks"] == expected, name
        label = a["blocks_label"]
        labels.add(label)
        for data in (old, new):
            assert base64.b64decode(data["map_blocks"][label], validate=True) == bytes(data["maps"][name]["blocks"])
    only(old["map_blocks"], new["map_blocks"], labels, "global block files")
    a = old["maps"][CITY]["attributes"]
    sheet = old["tilesets"][a["tileset_name"]]["collision"]
    assert sheet["01"] == ["FLOOR"] * 4 and sheet["45"] == ["WALL", "FLOOR", "FLOOR", "FLOOR"]
    for time in ("morn", "day", "nite"):
        blocks = art.blocks(a["tileset_name"], time)
        assert ImageChops.difference(blocks[1], blocks[0x45]).getbbox() == (0, 1, 16, 16), time


def check_signs(old, new, name, art):
    before, after = old["maps"][name], new["maps"][name]
    fields = {"events", "map_event_section_commands"}
    only(before, after, fields | ({"blocks"} if name == CITY else set()), name)
    events = deepcopy(before["events"])
    raw = deepcopy(before["map_event_section_commands"])
    a = after["attributes"]
    art_blocks = art.blocks(a["tileset_name"])
    for script, (previous, (x, y)) in SIGNS[name].items():
        [target] = [e for e in events["bg_events"] if e["script"] == script]
        assert (target["x"], target["y"], target["event_type"]) == (*previous, "BGEVENT_READ")
        assert not any((e["x"], e["y"]) == (x, y) for e in events["bg_events"])
        assert (x, y) not in {(o["x"], o["y"]) for o in after["objects"]}
        target.update(x=x, y=y)
        assert script in after["scripts"] and new["map_scripts"][script] == after["scripts"][script]
        [line] = [r for r in raw if r["command"] == "bg_event" and r["args"][-1] == script]
        assert line["args"] == [*(str(v) for v in previous), "BGEVENT_READ", script]
        line["args"][:2] = [str(x), str(y)]
        assert quadrant(old, name, *previous) == quadrant(new, name, *previous) == "FLOOR"
        assert quadrant(new, name, x, y) == "WALL"
        block = after["blocks"][x//2 + y//2*a["width"]]
        left, top = 16*(x % 2), 16*(y % 2)
        board = art_blocks[block].crop((left, top, left + 16, top + 16))
        assert hashlib.sha256(board.tobytes()).hexdigest() == (ELY_BOARD if name == "MnEly" else JOHTO_BOARD)
    assert after["events"] == events and after["map_event_section_commands"] == raw
    key = a["map_events_label"]
    for dataset, room in ((old, before), (new, after)):
        rows = room["map_event_section_commands"]
        assert dataset["map_scripts"][key] == [dict(command=r["command"], args=r["args"]) for r in rows]
        assert [r["command_index"] for r in rows] == list(range(len(rows)))
    return key


def check_walks(new, name, entrances):
    room = new["maps"][name]
    a = room["attributes"]
    floor = {(x, y) for x in range(2 * a["width"]) for y in range(2 * a["height"])
             if quadrant(new, name, x, y) in {"FLOOR", "TALL_GRASS", "LONG_GRASS"}}
    floor -= {(o["x"], o["y"]) for o in room["objects"]}
    portals = {(e["x"], e["y"]) for e in room["events"]["warps"]}
    edges = {p: floor & adjacent(p) for p in floor}
    for origins, reader in WALKS_BY_MAP[name]:
        for origin in origins:
            assert origin in entrances and origin in floor, (name, origin)
            # Stop at a door rather than walking through the building.
            reached = walk(edges, {origin}, portals)
            assert reader not in portals and reached.get(reader, 0) > 0, (name, origin, reader)
            assert origin in walk(edges, {reader}, portals), (name, reader, origin)


def verify(parent, child):
    only(parent, child, {"data", "report", "identity", "runtime_files"}, "archive sections")
    only(parent["report"], child["report"], {"manifests"}, "build report")
    only(parent["identity"], child["identity"], {"content_hash", "runtime_modpack_id"}, "identity")
    chain = parent["report"]["manifests"]
    assert chain[-1] == "northstar-research-choices-v1"
    assert child["report"]["manifests"] == chain + ["northstar-trail-signs-v1"]
    assert child["identity"]["runtime_modpack_id"] == "+".join(child["report"]["manifests"])
    old, new = parent["data"], child["data"]
    only(old, new, {"maps", "map_scripts", "map_blocks", "tilesets"}, "game data")
    only(old["maps"], new["maps"], SIGNS.keys() | {CEDAR}, "maps")
    only(old["maps"][CEDAR], new["maps"][CEDAR], {"blocks"}, "Cedar Inlet surroundings")
    art = SourceArt(child)
    check_blocks(old, new, art, {
        CITY: [(20, 29, 0x01, 0x45), (27, 25, 0x83, 0xb8)], CEDAR: [(11, 3, 0x8a, 0xb8)]})
    runpy.run_path(str(HERE / "check-plaques.py"))["check"](parent, child, SourceArt(parent), art, only)
    entrances = WALKS["anchors"](new)
    event_labels = set()
    for name in SIGNS:
        event_labels.add(check_signs(old, new, name, art))
    for name in WALKS_BY_MAP:
        check_walks(new, name, entrances[name])
    only(old["map_scripts"], new["map_scripts"], event_labels, "global scripts")
    print(f"{len(new['maps'])} maps and {len(child['runtime_files'])} assets checked. "
          "Seven signs match their art and retain their original text. All fourteen approaches are reversible.")


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("packs", nargs="*", type=Path)
    args = parser.parse_args()
    receipts = [json.loads((PREVIOUS / f"{edition}-build.json").read_text()) for edition in ("intro", "field")]
    if len(args.packs) == 2:
        pairs = [args.packs]
    elif not args.packs:
        root = HERE.parents[3]
        pairs = [(root / r["output_pack"], root / r["output_pack"].replace("research-choices", "trail-signs"))
                 for r in receipts]
    else:
        parser.error("supply a parent and child together")
    for p, c in pairs:
        print(f"Comparing {c.name} to {p.name}", flush=True)
        verify(read_pack(p, {r["expected_pack_sha256"] for r in receipts}), read_pack(c))
