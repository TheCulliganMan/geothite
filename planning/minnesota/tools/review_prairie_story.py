#!/usr/bin/env python3
"""Whole published prairie chapter and authored-source comparison, native art only."""
import argparse
import json
from pathlib import Path

from PIL import Image, ImageChops

from render_archive import retain_render_file, save_render
from render_reference import SourceArt, layout_blocks, load_pack
from render_stages import render_stage
from review_north_shore import oriented_sprite

ROOT = Path(__file__).resolve().parents[3]
PLAN = ROOT / "planning/minnesota"
OUT = PLAN / "renders/prairie-story"


def scene(pack, art, name):
    module = pack["data"]["maps"][name]
    attr = module["attributes"]
    terrain = art.render(attr["tileset_name"], attr["width"], attr["height"], module["blocks"])
    save_render(terrain, OUT / f"{name}-terrain.png")
    result = render_stage(terrain, "convert", OUT, name, "RGBA")
    for actor in module["objects"]:
        facing = {"SPRITEMOVEDATA_STANDING_UP": "north", "SPRITEMOVEDATA_STANDING_LEFT": "west",
                  "SPRITEMOVEDATA_STANDING_RIGHT": "east"}.get(actor["spritemovedata"], "south")
        result.alpha_composite(oriented_sprite(pack, actor["sprite"], facing),
                               (actor["x"] * 16, actor["y"] * 16))
    save_render(result, OUT / f"{name}-whole.png")
    return result


def review(pack_path, base_path):
    pack, base = load_pack(pack_path), load_pack(base_path)
    maps = pack["data"]["maps"]
    for name, patches in [("MnRochester", {(19, 9): 1, (0, 9): 1}),
                          ("MnMankato", {(23, 9): 1})]:
        layout = json.loads((PLAN / f"towns/{name}.layout.json").read_text())
        expected = layout_blocks(layout)
        width = layout["dimensions_metatiles"][0]
        for (x, y), block in patches.items():
            expected[y * width + x] = block
        assert maps[name]["blocks"] == expected, (name, "published terrain differs from source and declared mouths")
    assert maps["MnR02C00R00"]["blocks"] == base["data"]["maps"]["MnR02C00R00"]["blocks"]
    art = SourceArt(pack)
    scenes = {name: scene(pack, art, name) for name in ["MnRochester", "MnR02C00R00", "MnMankato"]}
    # Preserve both original production captures before any conversion/crop.
    for filename in ["native-care-gate.png", "native-notebook.png"]:
        retain_render_file(OUT / filename)
    background = scenes["MnMankato"]
    crop = render_stage(background, "crop", OUT, "notebook-source-camera", (160, 224, 672, 608))
    scaled = render_stage(crop, "resize", OUT, "notebook-source-camera", (1536, 1152), Image.Resampling.NEAREST)
    reference = Image.new("RGBA", (1280, 800))
    reference.paste(scaled, (-128, -176))
    player = oriented_sprite(pack, "SPRITE_CHRIS", "south")
    reference.alpha_composite(render_stage(player, "resize", OUT, "notebook-source-player", (48, 48),
                                          Image.Resampling.NEAREST), (640, 400))
    reference = render_stage(reference, "convert", OUT, "notebook-source-camera", "RGB")
    save_render(reference, OUT / "notebook-source-camera.png")
    native = OUT / "native-notebook.png"
    if native.exists():
        actual = render_stage(Image.open(native), "convert", OUT, "notebook-native-camera", "RGB")
        difference = ImageChops.difference(actual, reference)
        save_render(difference, OUT / "notebook-source-native-difference.png")
        print("Notebook full camera differing pixels:", sum(p != (0, 0, 0) for p in difference.getdata()),
              "of", difference.width * difference.height)
    print("PASS: exact authored town terrain plus declared mouths; unchanged R02 terrain. Whole maps and all render generations retained.")


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--pack", type=Path, default=ROOT / "content-packs/minnesota-northstar-prairie-story.crystalpack")
    parser.add_argument("--base", type=Path, default=ROOT / "content-packs/minnesota-northstar-prairie-base.crystalpack")
    args = parser.parse_args()
    review(args.pack, args.base)
