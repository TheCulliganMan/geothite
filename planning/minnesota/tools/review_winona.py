#!/usr/bin/env python3
"""Read-only source-art review of Winona and all five opening rooms."""
import argparse
from collections import deque
import json
from pathlib import Path

from render_archive import save_render
from render_reference import SourceArt, layout_blocks, load_pack, room_sheet
from render_stages import render_stage
from review_north_shore import oriented_sprite

ROOT = Path(__file__).resolve().parents[3]
OUT = ROOT / "planning/minnesota/renders/winona"


def review(path):
    pack = load_pack(path)
    art = SourceArt(pack)
    town = json.loads((ROOT / "planning/minnesota/towns/MnWinona.json").read_text())
    layout = json.loads((ROOT / "planning/minnesota/towns/MnWinona.layout.json").read_text())
    specs = [dict(town, tileset=town["compiled_tileset"], blocks=layout_blocks(layout), start=[14, 18])]
    specs += [dict(room, blocks=[int(v, 16) for row in room["rows"] for v in row.split()],
                   start=room["passive_arrival"]["tile"]) for room in town["interiors"]]
    rooms = []
    for spec in specs:
        width, height = spec["dimensions_metatiles"]
        collisions = pack["data"]["tilesets"][spec["tileset"]]["collision"]
        def kind(at):
            x, y = at
            return collisions[f'{spec["blocks"][y // 2 * width + x // 2]:02x}'][y % 2 * 2 + x % 2]
        bodies = {tuple(a["tile"]) for a in spec["residents"]}
        start = tuple(spec["start"])
        assert kind(start) == "FLOOR" and start not in bodies, (spec["id"], "clear arrival")
        todo, seen = deque([start]), {start}
        while todo:
            x, y = todo.popleft()
            for q in [(x-1,y),(x+1,y),(x,y-1),(x,y+1)]:
                if q in seen or q in bodies or not(0 <= q[0] < width*2 and 0 <= q[1] < height*2):
                    continue
                if kind(q) not in {"FLOOR", "DOOR", "WARP_CARPET_DOWN", "STAIRCASE"}:
                    continue
                seen.add(q)
                if kind(q) == "FLOOR":
                    todo.append(q)
        for actor in spec["residents"]:
            x, y = actor["tile"]
            approaches = {(x-1,y),(x+1,y),(x,y-1),(x,y+1)}
            if actor.get("counter_tile"):
                approaches.add(tuple(actor["approach"]))
            assert kind((x,y)) == "FLOOR" and seen & approaches, (spec["id"], actor["id"], "interaction")
        if spec["id"] == town["id"]:
            for door in town["doors"]:
                assert tuple(door["tile"]) in seen and kind(door["tile"]) == "DOOR", door["target"]
                assert tuple(door["return_tile"]) in seen and kind(door["return_tile"]) == "FLOOR"
        else:
            for exit in spec["exits"]:
                assert tuple(exit["tile"]) in seen and kind(exit["tile"]) == "WARP_CARPET_DOWN"
            if spec.get("pc"):
                assert kind(spec["pc"]["tile"]) == "PC" and tuple(spec["pc"]["approach"]) in seen
            if spec["id"] == "MnWinonaFieldStation":
                for ball in town["opening"]["starters"]:
                    assert kind(ball["tile"]) == "WALL" and tuple(ball["approach"]) in seen
            for stairs in town["internal_stairs"]:
                for key in ["from", "to"]:
                    if stairs[key+"_room"] == spec["id"]:
                        at = tuple(stairs[key+"_tile"])
                        assert kind(at) == "STAIRCASE" and at in seen, (spec["id"], "stairs")
        image = art.render(spec["tileset"], width, height, spec["blocks"],
                           "day" if spec["id"] == town["id"] else "indoor")
        save_render(image, OUT / (spec["id"] + "-terrain.png"))
        scene = render_stage(image, "convert", OUT, spec["id"] + "-scene", "RGBA")
        for actor in spec["residents"]:
            scene.alpha_composite(oriented_sprite(pack, actor["sprite"], actor["facing"]),
                                  (actor["tile"][0]*16, actor["tile"][1]*16))
        if spec["id"] == "MnWinonaFieldStation":
            for ball in town["opening"]["starters"]:
                scene.alpha_composite(oriented_sprite(pack, ball["sprite"], "south"),
                                      (ball["tile"][0]*16, ball["tile"][1]*16))
        save_render(scene, OUT / (spec["id"] + "-scene.png"))
        rooms.append((spec["id"], scene))
    save_render(room_sheet(rooms[1:], columns=2), OUT / "all-interiors.png")
    print("Source review passes all four facades, five rooms, actors, carpets, PC and both stair ends. Native gameplay remains separate.")


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--pack", default=str(ROOT / "content-packs/minnesota-northstar-city-story.crystalpack"))
    review(parser.parse_args().pack)
