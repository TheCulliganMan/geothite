#!/usr/bin/env python3
"""Render and check the complete authored Rochester, without exporting content."""
import argparse
from collections import deque
import json
from pathlib import Path

from PIL import Image, ImageChops

from render_archive import save_render
from render_reference import SourceArt, layout_blocks, load_pack, room_sheet
from render_stages import render_stage
from review_north_shore import oriented_sprite

ROOT = Path(__file__).resolve().parents[3]
OUT = ROOT / "planning/minnesota/renders/rochester"


def review(path, published=False):
    pack = load_pack(path)
    art = SourceArt(pack)
    town = json.loads((ROOT / "planning/minnesota/towns/MnRochester.json").read_text())
    layout = json.loads((ROOT / "planning/minnesota/towns/MnRochester.layout.json").read_text())
    specs = [dict(town, blocks=layout_blocks(layout), start=[32, 18])]
    specs += [dict(r, blocks=[int(v, 16) for row in r["rows"] for v in row.split()],
                   start=r["passive_arrival"]["tile"]) for r in town["interiors"] if r["rows"]]
    images = []
    for s in specs:
        width, height = s["dimensions_metatiles"]
        collision = pack["data"]["tilesets"][s["tileset"]]["collision"]
        def kind(at):
            x, y = at
            block = s["blocks"][y // 2 * width + x // 2]
            return collision[f"{block:02x}"][y % 2 * 2 + x % 2]
        bodies = {tuple(a["tile"]) for a in s["residents"]}
        start = tuple(s["start"])
        assert kind(start) == "FLOOR" and start not in bodies, (s["id"], "arrival")
        todo, seen = deque([start]), {start}
        while todo:
            x, y = todo.popleft()
            for at in [(x-1, y), (x+1, y), (x, y-1), (x, y+1)]:
                if at in seen or at in bodies or not (0 <= at[0] < width*2 and 0 <= at[1] < height*2):
                    continue
                if kind(at) in {"FLOOR", "DOOR", "WARP_CARPET_DOWN"}:
                    seen.add(at)
                    if kind(at) == "FLOOR": todo.append(at)
        for a in s["residents"]:
            x, y = a["tile"]
            approaches = {(x-1,y),(x+1,y),(x,y-1),(x,y+1)}
            if a.get("counter_tile"): approaches.add(tuple(a["approach"]))
            assert kind((x,y)) == "FLOOR" and seen & approaches, (s["id"], a["id"])
        if s["id"] == town["id"]:
            for d in town["doors"]:
                assert tuple(d["tile"]) in seen and kind(d["tile"]) == "DOOR", d
                assert tuple(d["return_tile"]) in seen and kind(d["return_tile"]) == "FLOOR", d
            assert {tuple(p["landing_metatile"][i]*2 for i in [0,1]) for p in town["ports"]} <= seen
            assert all(0 < x < width*2-1 and 0 < y < height*2-1 for x,y in seen)
            assert all("CUT_TREE" not in collision[f"{b:02x}"] for b in s["blocks"])
        else:
            for e in s["exits"]:
                assert tuple(e["tile"]) in seen and kind(e["tile"]) == "WARP_CARPET_DOWN", e
            if s.get("pc"):
                assert kind(s["pc"]["tile"]) == "PC" and tuple(s["pc"]["approach"]) in seen
        if published:
            module = pack["data"]["maps"][s["id"]]
            actual = list(module["blocks"])
            # Only the declared eastern road mouth changes after city fitting.
            if s["id"] == town["id"]:
                assert actual[9*width+19] == 1 and s["blocks"][9*width+19] == 5
                actual[9*width+19] = 5
            assert actual == s["blocks"], s["id"] + " source/native terrain mismatch"
            actors = pack["data"]["npcs"][s["id"]]
            assert {a["object_identifier"]: ([a["x"], a["y"]], a["sprite"]) for a in actors} == {
                a["id"]: (a["tile"], a["sprite"]) for a in s["residents"]}, s["id"] + " source/native actors"
        terrain = art.render(s["tileset"], width, height, s["blocks"],
                             "day" if s["id"] == town["id"] else "indoor")
        save_render(terrain, OUT / (s["id"] + "-terrain.png"))
        scene = render_stage(terrain, "convert", OUT, s["id"] + "-scene", "RGBA")
        for a in s["residents"]:
            scene.alpha_composite(oriented_sprite(pack, a["sprite"], a["facing"]),
                                  (a["tile"][0]*16, a["tile"][1]*16))
        save_render(scene, OUT / (s["id"] + "-scene.png"))
        images.append((s["id"], scene))
    gym = json.loads((ROOT / "planning/minnesota/revisions/rochester-gym1.json").read_text())["gyms"][0]
    width, height = gym["dimensions_metatiles"]
    image = Image.new("RGB", (width*32, height*32))
    for y, row in enumerate(gym["rows"]):
        for x, symbol in enumerate(row):
            source, block = gym["art"][symbol]
            image.paste(art.blocks(source, "indoor")[int(block,16)], (x*32,y*32))
    save_render(image, OUT / "canonical-gym-terrain.png")
    if published:
        module = pack["data"]["maps"][gym["id"]]
        symbols = {s: i for i, s in enumerate(sorted(set("".join(gym["rows"])) | {"#"}))}
        assert module["blocks"] == [symbols[s] for row in gym["rows"] for s in row]
        native = art.render("mn_gym_1", width, height, module["blocks"], "indoor")
        save_render(native, OUT / "canonical-gym-native-terrain.png")
        difference = ImageChops.difference(image, native.convert("RGB"))
        save_render(difference, OUT / "canonical-gym-terrain-difference.png")
        assert difference.getbbox() is None, "private Gym1 glyph/palette differs from original source recipes"
    images.append((gym["id"] + " canonical terrain", image))
    save_render(room_sheet(images[1:], columns=2), OUT / "all-interiors.png")
    print("Rochester source passes five facades, four ordinary rooms, actor/counter/PC reachability, carpets, closed perimeter and no incidental Cut. Gameplay verification is separate.")


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--pack", type=Path, default=ROOT / "content-packs/minnesota-northstar-winona.crystalpack")
    parser.add_argument("--verify-published", action="store_true")
    args = parser.parse_args()
    review(args.pack, args.verify_published)
