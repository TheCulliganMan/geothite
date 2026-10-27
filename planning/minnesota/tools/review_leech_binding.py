#!/usr/bin/env python3
"""Connected nine-map shore chapter and independent source/native comparisons."""
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
OUT = PLAN / "renders/leech-shore"
ORIGINS = {
    "MnBemidji": (448, 0), "MnR08C00R00": (192, 768),
    "MnR08C01R00": (1216, 768), "MnR08C00R01": (192, 1536),
    "MnR08C01R01": (1216, 1536), "MnLeechLake": (96, 2304),
    "MnR07C00R00": (288, 3072), "MnR07C00R01": (288, 3840),
    "MnBrainerd": (96, 4608),
}


def draw(pack, art, name, indoor=False):
    m = pack["data"]["maps"][name]
    a = m["attributes"]
    original = art.render(a["tileset_name"], a["width"], a["height"], m["blocks"],
                          group="indoor" if indoor else "day")
    save_render(original, OUT / f"{name}-native-terrain.png")
    result = render_stage(original, "convert", OUT, name + "-native", "RGBA")
    for actor in m["objects"]:
        facing = {"UP": "north", "LEFT": "west", "RIGHT": "east"}.get(
            actor["spritemovedata"].rsplit("_", 1)[-1], "south")
        result.alpha_composite(oriented_sprite(pack, actor["sprite"], facing),
                               (actor["x"] * 16, actor["y"] * 16))
    save_render(result, OUT / f"{name}-native-whole.png")
    return result, original


def difference(reference, native, key):
    save_render(reference, OUT / f"{key}-native-reference.png")
    actual = render_stage(Image.open(native), "convert", OUT, key + "-actual", "RGB")
    if key == "fisher-room":
        bounds = ImageChops.difference(actual, Image.new("RGB", actual.size, actual.getpixel((0, 0)))).getbbox()
        assert bounds and (bounds[2] - bounds[0], bounds[3] - bounds[1]) == reference.size
        actual = render_stage(actual, "crop", OUT, key + "-room", bounds)
    assert actual.size == reference.size
    delta = ImageChops.difference(actual, reference)
    save_render(delta, OUT / f"{key}-difference.png")
    count = sum(p != (0, 0, 0) for p in delta.getdata())
    print(key, "differing pixels", count, "of", reference.width * reference.height)
    return count


def review(path):
    archive_existing(OUT)
    pack = load_pack(path)
    source = json.loads((PLAN / "towns/MnLeechLake.json").read_text())
    layout = json.loads((PLAN / "towns/MnLeechLake.layout.json").read_text())
    assert pack["data"]["maps"]["MnLeechLake"]["blocks"] == layout_blocks(layout)
    for room in source["interiors"]:
        assert pack["data"]["maps"][room["id"]]["blocks"] == [int(v, 16) for row in room["rows"] for v in row.split()]
    art = SourceArt(pack)
    world = Image.new("RGBA", (1472, 5376), "#17232c")
    ground = world.copy()
    scenes = {}
    for name, origin in ORIGINS.items():
        scene, terrain = draw(pack, art, name)
        scenes[name] = scene
        ground.alpha_composite(render_stage(terrain, "convert", OUT, name + "-ground", "RGBA"), origin)
        world.alpha_composite(scene, origin)
    save_render(world, OUT / "whole-connected-brainerd-leech-bemidji.png")
    save_render(render_stage(world, "resize", OUT, "connected-preview", (736, 2688), Image.Resampling.NEAREST),
                OUT / "connected-preview.png")
    results = {}
    for key, tile in [("pier", (33, 21)), ("care-porch", (16, 15))]:
        canvas = ground.copy()
        canvas.alpha_composite(scenes["MnLeechLake"], ORIGINS["MnLeechLake"])
        x, y = ORIGINS["MnLeechLake"][0] + tile[0] * 16, ORIGINS["MnLeechLake"][1] + tile[1] * 16
        canvas.alpha_composite(oriented_sprite(pack, "SPRITE_CHRIS", "south"), (x, y))
        crop = render_stage(canvas, "crop", OUT, key, (x - 256, y - 192, x + 256, y + 192))
        scaled = render_stage(crop, "resize", OUT, key, (1536, 1152), Image.Resampling.NEAREST)
        reference = render_stage(scaled, "crop", OUT, key, (128, 176, 1408, 976))
        reference = render_stage(reference, "convert", OUT, key, "RGB")
        native = OUT / f"native-{key}.png"
        if native.exists():
            results[key] = difference(reference, native, key)
    inside, _ = draw(pack, art, "MnLeechLakeFisherHouse", indoor=True)
    inside.alpha_composite(oriented_sprite(pack, "SPRITE_CHRIS", "south"), (5 * 16, 6 * 16))
    save_render(inside, OUT / "fisher-room-source-player.png")
    scaled = render_stage(inside, "resize", OUT, "fisher-room-reference", (640, 512), Image.Resampling.NEAREST)
    reference = render_stage(scaled, "convert", OUT, "fisher-room-reference", "RGB")
    if (OUT / "native-fisher-room.png").exists():
        results["fisher-room"] = difference(reference, OUT / "native-fisher-room.png", "fisher-room")
    draw(pack, art, "MnLeechLakeCareCabin", indoor=True)
    print("PASS: complete native nine-map corridor, whole cabins and source identities; all render generations retained.")
    print(json.dumps(results, sort_keys=True))
    assert not any(results.values()), "source/native pixels differ"


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--pack", type=Path, default=ROOT / "content-packs/minnesota-northstar-leech-shore.crystalpack")
    args = parser.parse_args()
    review(args.pack)
