#!/usr/bin/env python3
"""Whole city/R04/depot review using original pixels and explicit story overlays."""
import argparse
import json
from pathlib import Path

from PIL import Image, ImageChops

from render_archive import archive_existing, save_render
from render_reference import SourceArt, layout_blocks, load_pack
from render_stages import render_stage
from review_north_shore import oriented_sprite

ROOT = Path(__file__).resolve().parents[3]
PLAN = ROOT / "planning/minnesota"
OUT = PLAN / "renders/courier-story"


def scene(pack, art, name):
    m = pack["data"]["maps"][name]
    a = m["attributes"]
    terrain = art.render(a["tileset_name"], a["width"], a["height"], m["blocks"],
                         group="indoor" if name == "MnStPaulUnionDepot" else "day")
    save_render(terrain, OUT / f"{name}-terrain.png")
    result = render_stage(terrain, "convert", OUT, name, "RGBA")
    empty = result.copy()
    for actor in m["objects"]:
        facing = {"UP": "north", "LEFT": "west", "RIGHT": "east"}.get(
            actor["spritemovedata"].rsplit("_", 1)[-1], "south")
        result.alpha_composite(oriented_sprite(pack, actor["sprite"], facing),
                               (actor["x"] * 16, actor["y"] * 16))
    save_render(result, OUT / f"{name}-whole.png")
    return result, empty


def review(pack_path, base_path):
    archive_existing(OUT)
    pack, base = load_pack(pack_path), load_pack(base_path)
    maps, old = pack["data"]["maps"], base["data"]["maps"]
    for name in ["MnStPaulUnionDepot", "MnR04C00R00", "MnStPaul"]:
        assert maps[name]["blocks"] == old[name]["blocks"], (name, "unexpected terrain edit")
    expected = old["MnMinneapolis"]["blocks"].copy()
    assert expected[12 * 32 + 31] == 5
    expected[12 * 32 + 31] = 1
    assert maps["MnMinneapolis"]["blocks"] == expected
    source = json.loads((PLAN / "transit/MnStPaulUnionDepot.json").read_text())
    assert maps["MnStPaulUnionDepot"]["blocks"] == [int(v, 16) for row in source["rows"] for v in row.split()]
    art = SourceArt(pack)
    origins = {"MnMinneapolis": (0, 0), "MnR04C00R00": (1024, 96), "MnStPaul": (1920, 0)}
    world = Image.new("RGBA", (2944, 768), "#17232c")
    ground = world.copy()
    city_scene = None
    for name, origin in origins.items():
        actors, terrain = scene(pack, art, name)
        world.alpha_composite(actors, origin)
        ground.alpha_composite(terrain, origin)
        if name == "MnMinneapolis":
            city_scene = actors
    save_render(world, OUT / "whole-connected-cities-r04.png")
    depot, _ = scene(pack, art, "MnStPaulUnionDepot")
    # Player-scale view of the new paper and existing courier, with the whole
    # room retained separately. The production room camera may fit at 2x.
    player = oriented_sprite(pack, "SPRITE_CHRIS", "south")
    inside = depot.copy()
    inside.alpha_composite(player, (19 * 16, 16 * 16))
    save_render(inside, OUT / "depot-source-player.png")
    for scale in [2, 3]:
        scaled = render_stage(inside, "resize", OUT, f"depot-source-{scale}x",
                              (512 * scale, 384 * scale), Image.Resampling.NEAREST)
        save_render(scaled, OUT / f"depot-source-{scale}x.png")
    # The native camera presents the active map's objects; adjacent terrain
    # appears through the seam before that neighbor becomes the active map.
    gate = ground.copy()
    gate.alpha_composite(city_scene, (0, 0))
    gate.alpha_composite(player, (59 * 16, 24 * 16))
    crop = render_stage(gate, "crop", OUT, "gate-camera", (59 * 16 - 256, 24 * 16 - 192,
                                                           59 * 16 + 256, 24 * 16 + 192))
    scaled = render_stage(crop, "resize", OUT, "gate-camera", (1536, 1152), Image.Resampling.NEAREST)
    reference = render_stage(scaled, "crop", OUT, "gate-camera", (128, 176, 1408, 976))
    reference = render_stage(reference, "convert", OUT, "gate-camera", "RGB")
    save_render(reference, OUT / "gate-camera-reference.png")
    native = OUT / "native-current-gate.png"
    if native.exists():
        actual = render_stage(Image.open(native), "convert", OUT, "gate-native", "RGB")
        difference = ImageChops.difference(actual, reference)
        save_render(difference, OUT / "gate-camera-difference.png")
        print("Current gate differing pixels:", sum(p != (0, 0, 0) for p in difference.getdata()), "of", 1280 * 800)
    padded = Image.new("RGBA", (1024, 768), inside.getpixel((0, 383)))
    padded.alpha_composite(inside, (256, 192))
    save_render(padded, OUT / "depot-source-padded.png")
    crop = render_stage(padded, "crop", OUT, "depot-camera", (304, 256, 816, 640))
    scaled = render_stage(crop, "resize", OUT, "depot-camera", (1536, 1152), Image.Resampling.NEAREST)
    reference = render_stage(scaled, "crop", OUT, "depot-camera", (128, 176, 1408, 976))
    reference = render_stage(reference, "convert", OUT, "depot-camera", "RGB")
    save_render(reference, OUT / "depot-camera-reference.png")
    native = OUT / "native-depot.png"
    if native.exists():
        actual = render_stage(Image.open(native), "convert", OUT, "depot-native", "RGB")
        difference = ImageChops.difference(actual, reference)
        save_render(difference, OUT / "depot-camera-difference.png")
        print("Depot differing pixels:", sum(p != (0, 0, 0) for p in difference.getdata()), "of", 1280 * 800)
    print("PASS: one declared forest mouth; unchanged R04, St Paul and depot terrain. Whole maps and all render generations retained.")


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--pack", type=Path, default=ROOT / "content-packs/minnesota-northstar-courier-story.crystalpack")
    parser.add_argument("--base", type=Path, default=ROOT / "content-packs/minnesota-northstar-river-story.crystalpack")
    args = parser.parse_args()
    review(args.pack, args.base)
