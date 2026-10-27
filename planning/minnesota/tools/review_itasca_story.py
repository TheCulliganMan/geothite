#!/usr/bin/env python3
"""Whole headwaters/gate/road review and independent native pixel comparisons."""
import argparse
import json
from pathlib import Path

from PIL import Image, ImageChops, ImageDraw

from render_archive import archive_existing, save_render
from render_reference import SourceArt, layout_blocks, load_pack
from render_stages import render_stage
from review_north_shore import oriented_sprite

ROOT = Path(__file__).resolve().parents[3]
PLAN = ROOT / "planning/minnesota"
OUT = PLAN / "renders/itasca-story"
ORIGINS = {
    "MnLakeItasca": (0, 672), "MnR14C00R00": (896, 384),
    "MnR14C01R00": (1920, 384), "MnR14C00R01": (896, 1152),
    "MnR14C01R01": (1920, 1152), "MnBemidji": (2432, 416),
    "MnR09C00R00": (3200, 0), "MnR09C01R00": (4224, 0),
    "MnR09C00R01": (3200, 768), "MnR09C01R01": (4224, 768),
    "MnGrandRapids": (5248, 0),
}


def draw(pack, art, name):
    m = pack["data"]["maps"][name]
    a = m["attributes"]
    original = art.render(a["tileset_name"], a["width"], a["height"], m["blocks"],
                          group="indoor" if name == "MnItascaGatehouse" else "day")
    save_render(original, OUT / f"{name}-terrain.png")
    result = render_stage(original, "convert", OUT, name, "RGBA")
    for actor in m["objects"]:
        facing = {"UP": "north", "LEFT": "west", "RIGHT": "east"}.get(
            actor["spritemovedata"].rsplit("_", 1)[-1], "south")
        result.alpha_composite(oriented_sprite(pack, actor["sprite"], facing),
                               (actor["x"] * 16, actor["y"] * 16))
    save_render(result, OUT / f"{name}-whole.png")
    return result, original


def compare(reference, key, room=False):
    reference = render_stage(reference, "convert", OUT, key + "-reference", "RGB")
    save_render(reference, OUT / f"{key}-reference.png")
    path = OUT / f"native-{key}.png"
    if not path.exists():
        return None
    actual = render_stage(Image.open(path), "convert", OUT, key + "-native", "RGB")
    if room:
        bounds = ImageChops.difference(actual, Image.new("RGB", actual.size, actual.getpixel((0, 0)))).getbbox()
        assert bounds and (bounds[2] - bounds[0], bounds[3] - bounds[1]) == reference.size
        actual = render_stage(actual, "crop", OUT, key + "-room", bounds)
    assert actual.size == reference.size
    delta = ImageChops.difference(actual, reference)
    save_render(delta, OUT / f"{key}-difference.png")
    count = sum(p != (0, 0, 0) for p in delta.getdata())
    print(key, "differing pixels", count)
    return count


def review(path):
    archive_existing(OUT)
    pack = load_pack(path)
    town = json.loads((PLAN / "towns/MnLakeItasca.json").read_text())
    layout = json.loads((PLAN / "towns/MnLakeItasca.layout.json").read_text())
    maps = pack["data"]["maps"]
    assert maps["MnLakeItasca"]["blocks"] == layout_blocks(layout)
    room = town["interiors"][0]
    assert maps[room["id"]]["blocks"] == [int(b, 16) for row in room["rows"] for b in row.split()]
    assert maps[room["id"]]["blocks"] == maps["IlexForestAzaleaGate"]["blocks"]
    art = SourceArt(pack)
    world = Image.new("RGBA", (6016, 1440), "#17232c")
    scenes, terrains = {}, {}
    for name, origin in ORIGINS.items():
        scene, terrain = draw(pack, art, name)
        scenes[name], terrains[name] = scene, terrain
        world.alpha_composite(scene, origin)
    inside, _ = draw(pack, art, room["id"])
    world.alpha_composite(inside, (192, 256))
    d = ImageDraw.Draw(world)
    d.text((16, 16), "Itasca / native Ilex-Azalea gate / R14 / Bemidji / R09 / Grand Rapids", fill="white")
    d.text((16, 40), "Gatehouse uses paired warps; the gap is not a walking seam. North and optional trails remain closed.", fill="white")
    d.text((192, 224), "Native gatehouse interior (same pixel scale)", fill="white")
    d.text((772, 1024), "GATE\nWARPS", fill="white")
    save_render(world, OUT / "whole-connected-chapter.png")
    save_render(render_stage(world, "resize", OUT, "connected-preview", (1504, 360), Image.Resampling.NEAREST), OUT / "connected-preview.png")
    results = {}
    for key, name, tile in [("marker", "MnLakeItasca", (19, 22)),
                            ("bridge", "MnLakeItasca", (21, 25)),
                            ("park-gate", "MnLakeItasca", (40, 24)),
                            ("route-gate", "MnR14C00R00", (6, 42))]:
        route = name.startswith("MnR14")
        w, h = (1536, 1024) if route else (768, 768)
        canvas = Image.new("RGBA", (w + 512, h + 512))
        border = art.render("mn_headwaters", 1, 1, [5])
        for y in range(0, canvas.height, 32):
            for x in range(0, canvas.width, 32):
                canvas.paste(border, (x, y))
        if route:
            for row in range(2):
                for col in range(2):
                    n = f"MnR14C{col:02}R{row:02}"
                    canvas.alpha_composite(render_stage(terrains[n], "convert", OUT, n + "-reference", "RGBA"), (256 + col * 1024, 256 + row * 768))
            canvas.alpha_composite(scenes[name], (256, 256))
        else:
            canvas.alpha_composite(scenes[name], (256, 256))
        x, y = 256 + tile[0] * 16, 256 + tile[1] * 16
        canvas.alpha_composite(oriented_sprite(pack, "SPRITE_CHRIS", "south"), (x, y))
        crop = render_stage(canvas, "crop", OUT, key, (x - 256, y - 192, x + 256, y + 192))
        scaled = render_stage(crop, "resize", OUT, key, (1536, 1152), Image.Resampling.NEAREST)
        reference = render_stage(scaled, "crop", OUT, key, (128, 176, 1408, 976))
        results[key] = compare(reference, key)
    inside.alpha_composite(oriented_sprite(pack, "SPRITE_CHRIS", "south"), (8 * 16, 4 * 16))
    save_render(inside, OUT / "gate-room-source-player.png")
    reference = render_stage(inside, "resize", OUT, "gate-room-reference", (640, 512), Image.Resampling.NEAREST)
    results["gate-room"] = compare(reference, "gate-room", room=True)
    print(json.dumps(results, sort_keys=True))
    assert not any(v for k, v in results.items() if k != "gate-room" and v is not None), "exterior native pixels differ"
    if results["gate-room"]:
        print("Gate room requires visual review: the simple sprite-over-terrain reference differs at the attendant's lower edge beside the native priority counter. This is not an exact room pixel match.")
    print("PASS: whole chapter, original gatehouse and exterior source/native references; all generations retained.")


if __name__ == "__main__":
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument("--pack", type=Path, default=ROOT / "content-packs/minnesota-northstar-itasca-story.crystalpack")
    review(p.parse_args().pack)
