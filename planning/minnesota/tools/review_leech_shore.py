#!/usr/bin/env python3
"""Complete native-art Leech Lake shore and room review; no pack writes."""
import argparse
import json
from pathlib import Path

from render_archive import archive_existing, save_render
from render_reference import SourceArt, layout_blocks, load_pack
from render_stages import render_stage
from review_north_shore import oriented_sprite
from review_st_cloud import adjacent, flood, sampler

ROOT = Path(__file__).resolve().parents[3]
PLAN = ROOT / "planning/minnesota"
OUT = PLAN / "renders/leech-shore"


def review(path):
    archive_existing(OUT)
    pack = load_pack(path)
    design = json.loads((PLAN / "towns/MnLeechLake.json").read_text())
    layout = json.loads((PLAN / "towns/MnLeechLake.layout.json").read_text())
    blocks = layout_blocks(layout)
    assert design["dimensions_metatiles"] == layout["dimensions_metatiles"] == [32, 24]
    art = SourceArt(pack)
    terrain = art.render("mn_headwaters", 32, 24, blocks)
    save_render(terrain, OUT / "whole-terrain.png")
    scene = render_stage(terrain, "convert", OUT, "whole", "RGBA")
    for actor in design["residents"]:
        scene.alpha_composite(oriented_sprite(pack, actor["sprite"], actor["facing"]),
                              (actor["tile"][0] * 16, actor["tile"][1] * 16))
    save_render(scene, OUT / "whole-scene.png")
    k = sampler(pack, "mn_headwaters", 32, blocks)
    cells = {(x, y) for y in range(48) for x in range(64)}
    bodies = {tuple(a["tile"]) for a in design["residents"]}
    doors = {tuple(a["tile"]) for a in design["doors"]}
    assert len(bodies) == len(design["residents"])
    walk = {q for q in cells if k(q) in {"FLOOR", "DOOR", "LADDER"}}
    water = {q for q in cells if k(q) == "WATER"}
    seen = flood((32, 44), walk - bodies, doors)
    surf = flood((32, 44), (walk | water) - bodies, doors)
    outer = {q for q in cells if q[0] in [0, 63] or q[1] in [0, 47]}
    assert seen & outer == surf & outer == {(32, 0), (33, 0), (32, 47), (33, 47)}
    assert not water & outer
    assert not any(k(q) == "CUT_TREE" for q in cells)
    assert doors == {q for q in cells if k(q) == "DOOR"}
    for door in design["doors"]:
        assert tuple(door["tile"]) in seen and k(tuple(door["tile"])) == "DOOR"
        assert tuple(door["return_tile"]) in seen and k(tuple(door["return_tile"])) == "FLOOR"
        room = next(r for r in design["interiors"] if r["id"] == door["target"])
        assert door["arrival_tile"] == room["passive_arrival"]["tile"]
        assert room["passive_arrival"]["index"] == door["target_warp"] == 3
        assert [e["target_warp"] for e in room["exits"]] == [door["return_warp_index"]] * 2
    for actor in design["residents"]:
        q = tuple(actor["tile"])
        assert k(q) == "FLOOR" and any(p in seen for p in adjacent(q)), actor["id"]
    for sign in design["signs"]:
        q = tuple(sign["tile"])
        assert k(q) == "WALL" and any(p in seen for p in adjacent(q)), sign["id"]
    pier = design["pier_contract"]
    for x in range(*pier["walk_x_core"]):
        assert all(k((x, y)) == "FLOOR" for y in pier["walk_rows_core"])
        assert k((x, pier["top_rail_core_y"])) == k((x, pier["bottom_rail_core_y"])) == "WALL"
    assert all(k((pier["closed_end_core_x"], y)) == "WALL" for y in pier["walk_rows_core"])
    launch = design["launch_contract"]
    assert tuple(launch["approach_core"]) in seen
    assert k(tuple(launch["water_target_core"])) == "WATER"
    for room in design["interiors"]:
        w, h = room["dimensions_metatiles"]
        values = [int(v, 16) for row in room["rows"] for v in row.split()]
        rk = sampler(pack, room["tileset"], w, values)
        assert rk(tuple(room["passive_arrival"]["tile"])) == "FLOOR"
        for e in room["exits"]:
            assert rk(tuple(e["tile"])) == "WARP_CARPET_DOWN"
        original = art.render(room["tileset"], w, h, values, group="indoor")
        save_render(original, OUT / f'{room["id"]}-terrain.png')
        inside = render_stage(original, "convert", OUT, room["id"], "RGBA")
        for a in room["residents"]:
            inside.alpha_composite(oriented_sprite(pack, a["sprite"], a["facing"]),
                                   (a["tile"][0] * 16, a["tile"][1] * 16))
        save_render(inside, OUT / f'{room["id"]}-whole.png')
    for actor in design["residents"] + design["signs"] + [a for r in design["interiors"] for a in r["residents"]]:
        assert all(0 < len(s) <= 18 and s.isascii() for s in actor["lines"]), actor["id"]
    print("PASS: two real doors/rooms, both walking mouths, accessible witnesses/signs, closed pier, separate launch and closed Surf boundary; no native Cut quadrants.")


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--pack", type=Path, default=ROOT / "content-packs/minnesota-northstar-lake-country.crystalpack")
    args = parser.parse_args()
    review(args.pack)
