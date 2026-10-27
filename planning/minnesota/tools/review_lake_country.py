#!/usr/bin/env python3
"""Whole connected lake-country maps and matched native checkpoint cameras."""
import argparse
import json
from pathlib import Path

from PIL import Image, ImageChops

from render_archive import archive_existing, save_render
from render_reference import SourceArt, load_pack
from render_stages import render_stage
from review_north_shore import oriented_sprite

ROOT = Path(__file__).resolve().parents[3]
PLAN = ROOT / "planning/minnesota"
OUT = PLAN / "renders/lake-country-story"
ORIGINS = {
    "MnBrainerd": (448, 0),
    "MnR06C00R00": (288, 768),
    "MnR06C00R01": (288, 1536),
    "MnStCloud": (160, 2176),
    "MnR05C00R00": (160, 2752),
    "MnR05C00R01": (160, 3520),
    "MnR05C00R02": (160, 4288),
    "MnMinneapolis": (0, 4544),
}
VIEWS = {
    "two-city-gate": ("MnMinneapolis", (32, 4)),
    "wind-gate": ("MnStCloud", (24, 4)),
    "brainerd-south": ("MnBrainerd", (32, 46)),
    "ranger": ("MnBrainerd", (25, 20)),
}


def scene(pack, art, name):
    module = pack["data"]["maps"][name]
    a = module["attributes"]
    terrain = art.render(a["tileset_name"], a["width"], a["height"], module["blocks"])
    save_render(terrain, OUT / f"{name}-terrain.png")
    result = render_stage(terrain, "convert", OUT, name, "RGBA")
    for actor in module["objects"]:
        facing = {"UP": "north", "LEFT": "west", "RIGHT": "east"}.get(
            actor["spritemovedata"].rsplit("_", 1)[-1], "south")
        result.alpha_composite(oriented_sprite(pack, actor["sprite"], facing),
                               (actor["x"] * 16, actor["y"] * 16))
    save_render(result, OUT / f"{name}-whole.png")
    return result, terrain


def review(pack_path, base_path):
    archive_existing(OUT)
    pack, base = load_pack(pack_path), load_pack(base_path)
    spec = json.loads((PLAN / "playable/lake-country-story.json").read_text())
    for name in ORIGINS:
        expected = base["data"]["maps"][name]["blocks"].copy()
        width = pack["data"]["maps"][name]["attributes"]["width"]
        for cap in spec["caps"]:
            if cap["map"] == name:
                x, y = cap["metatile"]
                assert expected[y * width + x] == cap["previous"]
                expected[y * width + x] = cap["replacement"]
        if name == "MnMinneapolis":
            assert expected[16] == 5
            expected[16] = 1
        assert expected == pack["data"]["maps"][name]["blocks"], name
    art = SourceArt(pack)
    world = Image.new("RGBA", (1472, 5312), "#17232c")
    ground = world.copy()
    scenes = {}
    for name, origin in ORIGINS.items():
        actors, terrain = scene(pack, art, name)
        scenes[name] = actors
        world.alpha_composite(actors, origin)
        ground.alpha_composite(terrain.convert("RGBA"), origin)
    save_render(world, OUT / "whole-connected-lake-country.png")
    preview = render_stage(world, "resize", OUT, "connected-preview", (736, 2656), Image.Resampling.NEAREST)
    save_render(preview, OUT / "connected-preview.png")
    results = {}
    for key, (name, tile) in VIEWS.items():
        # Neighbor terrain is visible before activation, but only the active
        # map's actors appear in the native camera.
        camera = ground.copy()
        camera.alpha_composite(scenes[name], ORIGINS[name])
        x, y = ORIGINS[name][0] + tile[0] * 16, ORIGINS[name][1] + tile[1] * 16
        camera.alpha_composite(oriented_sprite(pack, "SPRITE_CHRIS", "south"), (x, y))
        crop = render_stage(camera, "crop", OUT, key, (x - 256, y - 192, x + 256, y + 192))
        scaled = render_stage(crop, "resize", OUT, key, (1536, 1152), Image.Resampling.NEAREST)
        reference = render_stage(scaled, "crop", OUT, key, (128, 176, 1408, 976))
        reference = render_stage(reference, "convert", OUT, key, "RGB")
        save_render(reference, OUT / f"{key}-reference.png")
        native = OUT / f"native-{key}.png"
        if native.exists():
            actual = render_stage(Image.open(native), "convert", OUT, key + "-native", "RGB")
            assert actual.size == reference.size
            difference = ImageChops.difference(actual, reference)
            save_render(difference, OUT / f"{key}-difference.png")
            count = sum(pixel != (0, 0, 0) for pixel in difference.getdata())
            results[key] = count
            print(key, "differing pixels", count, "of", 1280 * 800)
    print("PASS: one new mouth/nine visible caps; all eight whole maps and every render generation retained.")
    if results:
        print(json.dumps(results, sort_keys=True))


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--pack", type=Path, default=ROOT / "content-packs/minnesota-northstar-lake-country.crystalpack")
    parser.add_argument("--base", type=Path, default=ROOT / "content-packs/minnesota-northstar-courier-story.crystalpack")
    args = parser.parse_args()
    review(args.pack, args.base)
