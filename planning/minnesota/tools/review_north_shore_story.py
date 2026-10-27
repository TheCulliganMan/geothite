#!/usr/bin/env python3
"""Read-only fast preview of explicit native Grand Marais connection patches."""
import argparse
import json
import os
from pathlib import Path
import subprocess

from PIL import Image, ImageChops, ImageDraw

from render_archive import archive_existing, retain_render_file, save_render
from render_reference import SourceArt, load_pack
from render_stages import render_stage
from review_north_shore import oriented_sprite

ROOT = Path(__file__).resolve().parents[3]
OUT = ROOT / "planning/minnesota/renders/north-shore-story"


def preview():
    source = json.loads((ROOT / "planning/minnesota/towns/MnGrandMarais.connections.json").read_text())
    pack = load_pack(ROOT / "content-packs/minnesota-northstar-itasca-story.crystalpack")
    assert pack["identity"]["content_hash"] == source["base_content_hash"]
    art = SourceArt(pack)
    native = art.blocks("johto")
    blocks = list(native)
    collisions = dict(pack["data"]["tilesets"]["johto"]["collision"])
    for recipe in source["reef_recipes"]:
        assert int(recipe["id"], 16) == len(blocks)
        base = int(recipe["base"], 16)
        image = native[base].copy()
        kinds = list(collisions[recipe["base"]])
        for change in recipe["quadrants"]:
            q = change["index"]
            x, y = q % 2 * 16, q // 2 * 16
            assert kinds[q] == "WATER"
            assert collisions[change["source"]][q] == "BUOY"
            crop = render_stage(native[int(change["source"], 16)], "crop", OUT,
                                f'reef-{recipe["id"]}-{q}', (x, y, x + 16, y + 16))
            image.paste(crop, (x, y))
            kinds[q] = "BUOY"
        blocks.append(image)
        collisions[recipe["id"]] = kinds
        save_render(image, OUT / f'reef-{recipe["id"]}-source-preview.png')
    for name in ["MnGrandMarais", "MnR11C00R00"]:
        module = pack["data"]["maps"][name]
        values = list(module["blocks"])
        a = module["attributes"]
        for patch in source["patches"]:
            if patch["map"] != name:
                continue
            x, y = patch["metatile"]
            i = y * a["width"] + x
            assert values[i] == int(patch["expected"], 16), (name, x, y)
            values[i] = int(patch["block"], 16)
        sheet = blocks if name == "MnGrandMarais" else art.blocks(a["tileset_name"])
        terrain = Image.new("RGB", (a["width"] * 32, a["height"] * 32))
        for i, block in enumerate(values):
            terrain.paste(sheet[block], (i % a["width"] * 32, i // a["width"] * 32))
        save_render(terrain, OUT / f"{name}-candidate-terrain.png")
        scene = render_stage(terrain, "convert", OUT, name + "-candidate", "RGBA")
        for actor in module["objects"]:
            facing = {"UP": "north", "LEFT": "west", "RIGHT": "east"}.get(
                actor["spritemovedata"].rsplit("_", 1)[-1], "south")
            scene.alpha_composite(oriented_sprite(pack, actor["sprite"], facing),
                                  (actor["x"] * 16, actor["y"] * 16))
        if name == "MnGrandMarais":
            for checkpoint in source["checkpoints"]:
                for x, y in checkpoint["bodies"]:
                    scene.alpha_composite(oriented_sprite(pack, "SPRITE_OFFICER", "east"), (x * 16, y * 16))
        save_render(scene, OUT / f"{name}-candidate-whole.png")
    print("Source candidate only: exact native quadrant copies; no game pack written.")


def published(path):
    """Inspect complete published geography and compare matching native cameras."""
    archive_existing(OUT)
    pack = load_pack(path)
    maps = pack["data"]["maps"]
    art = SourceArt(pack)
    scenes, terrains, animation_masks = {}, {}, {}
    names = ["MnGrandMarais", "MnGrandMaraisShoreGate"]
    names += [f"MnR11C{col:02}R{row:02}" for row in range(4) for col in range(2)]
    names += [f"MnR12C{col:02}R{row:02}" for row in range(3) for col in range(2)]
    for name in names:
        module = maps[name]
        a = module["attributes"]
        terrain = art.render(a["tileset_name"], a["width"], a["height"], module["blocks"],
                             group="indoor" if name == "MnGrandMaraisShoreGate" else "day")
        save_render(terrain, OUT / f"{name}-published-terrain.png")
        scene = render_stage(terrain, "convert", OUT, name + "-published", "RGBA")
        for actor in module["objects"]:
            facing = {"UP": "north", "LEFT": "west", "RIGHT": "east"}.get(
                actor["spritemovedata"].rsplit("_", 1)[-1], "south")
            scene.alpha_composite(oriented_sprite(pack, actor["sprite"], facing),
                                  (actor["x"] * 16, actor["y"] * 16))
        save_render(scene, OUT / f"{name}-published-whole.png")
        scenes[name], terrains[name] = scene, terrain
        mask = Image.new("L", terrain.size)
        if a["tileset_name"] in {"mn_marais_shore", "mn_north_shore", "mn_gunflint"}:
            layout = bytes(pack["runtime_files"][f'data/tilesets/{a["tileset_name"]}_metatiles.bin'])
            ink = ImageDraw.Draw(mask)
            for i, block in enumerate(module["blocks"]):
                for glyph, value in enumerate(layout[block * 16:block * 16 + 16]):
                    if value in {0x03, 0x14}:
                        x, y = i % a["width"] * 32 + glyph % 4 * 8, i // a["width"] * 32 + glyph // 4 * 8
                        ink.rectangle((x, y, x + 7, y + 7), fill=255)
        save_render(mask, OUT / f"{name}-native-animation-mask.png")
        animation_masks[name] = mask
    assert maps["MnGrandMaraisShoreGate"]["blocks"] == maps["IlexForestAzaleaGate"]["blocks"]
    atlas = Image.new("RGBA", (4992, 3136), "#17232c")
    for row in range(3):
        for col in range(2):
            atlas.alpha_composite(scenes[f"MnR12C{col:02}R{row:02}"], (col * 1024, 64 + row * 768))
    atlas.alpha_composite(scenes["MnGrandMarais"], (2048, 832))
    atlas.alpha_composite(scenes["MnGrandMaraisShoreGate"], (2752, 832))
    for row in range(4):
        for col in range(2):
            atlas.alpha_composite(scenes[f"MnR11C{col:02}R{row:02}"], (3200 + col * 1024, 64 + row * 768))
    draw = ImageDraw.Draw(atlas)
    draw.text((16, 16), "GUNFLINT / GRAND MARAIS / native Ilex-Azalea gate / NORTH SHORE", fill="white")
    draw.text((2704, 800), "PAIRED WARPS (no outdoor seam)", fill="white")
    save_render(atlas, OUT / "whole-connected-coast-and-gunflint.png")
    save_render(render_stage(atlas, "resize", OUT, "whole-connected-preview", (1248, 784), Image.Resampling.NEAREST), OUT / "whole-connected-preview.png")
    results = {}
    for key, name, tile in [("town-gate", "MnGrandMarais", (8, 18)),
                            ("gunflint-mouth", "MnGrandMarais", (4, 8)),
                            ("west-quay", "MnGrandMarais", (3, 22)),
                            ("north-shore-gate", "MnR11C00R00", (42, 4)),
                            ("harbor-water", "MnGrandMarais", (29, 34))]:
        a = maps[name]["attributes"]
        canvas = Image.new("RGBA", (a["width"] * 32 + 512, a["height"] * 32 + 512))
        animation = Image.new("L", canvas.size)
        border = art.render(a["tileset_name"], 1, 1, [a["border_block"]])
        border_animation = Image.new("L", (32, 32))
        glyphs = bytes(pack["runtime_files"][f'data/tilesets/{a["tileset_name"]}_metatiles.bin'])
        if a["tileset_name"] in {"mn_marais_shore", "mn_north_shore", "mn_gunflint"}:
            ink = ImageDraw.Draw(border_animation)
            for i, value in enumerate(glyphs[a["border_block"] * 16:a["border_block"] * 16 + 16]):
                if value in {0x03, 0x14}:
                    x, y = i % 4 * 8, i // 4 * 8
                    ink.rectangle((x, y, x + 7, y + 7), fill=255)
        for y in range(0, canvas.height, 32):
            for x in range(0, canvas.width, 32):
                canvas.paste(border, (x, y))
                animation.paste(border_animation, (x, y))
        if name == "MnGrandMarais":
            canvas.alpha_composite(render_stage(terrains["MnR12C01R00"], "convert", OUT, key + "-northwest-neighbor", "RGBA"), (-768, -512))
            canvas.alpha_composite(render_stage(terrains["MnR12C01R01"], "convert", OUT, key + "-neighbor", "RGBA"), (-768, 256))
            animation.paste(animation_masks["MnR12C01R00"], (-768, -512))
            animation.paste(animation_masks["MnR12C01R01"], (-768, 256))
        canvas.alpha_composite(scenes[name], (256, 256))
        animation.paste(animation_masks[name], (256, 256))
        x, y = 256 + tile[0] * 16, 256 + tile[1] * 16
        token = "SPRITE_SURF" if key == "harbor-water" else "SPRITE_CHRIS"
        canvas.alpha_composite(oriented_sprite(pack, token, "south"), (x, y))
        # The native LCD is 20x18 8px cells. The unconnected north edge
        # clamps R11's viewport at zero: core y=4 is half a cell above center.
        camera_y = y + (8 if key == "north-shore-gate" else 0)
        reference = render_stage(canvas, "crop", OUT, key, (x - 256, camera_y - 192, x + 256, camera_y + 192))
        reference = render_stage(reference, "resize", OUT, key, (1536, 1152), Image.Resampling.NEAREST)
        reference = render_stage(reference, "crop", OUT, key, (128, 176, 1408, 976))
        animation = render_stage(animation, "crop", OUT, key + "-animation", (x - 256, camera_y - 192, x + 256, camera_y + 192))
        animation = render_stage(animation, "resize", OUT, key + "-animation", (1536, 1152), Image.Resampling.NEAREST)
        animation = render_stage(animation, "crop", OUT, key + "-animation", (128, 176, 1408, 976))
        save_render(animation, OUT / f"{key}-camera-animation-mask.png")
        save_render(reference, OUT / f"{key}-source-camera.png")
        actual_path = OUT / f"native-{key}.png"
        if actual_path.exists():
            actual = render_stage(Image.open(actual_path), "convert", OUT, key + "-native", "RGB")
            reference = render_stage(reference, "convert", OUT, key + "-reference", "RGB")
            delta = ImageChops.difference(actual, reference)
            save_render(delta, OUT / f"{key}-difference.png")
            results[key] = dict(differing_pixels=sum(pixel != (0, 0, 0) for pixel in delta.getdata()),
                                outside_native_animation=sum(pixel != (0, 0, 0) and not mask for pixel, mask in zip(delta.getdata(), animation.getdata())))
    room = scenes["MnGrandMaraisShoreGate"].copy()
    room.alpha_composite(oriented_sprite(pack, "SPRITE_CHRIS", "south"), (128, 64))
    save_render(room, OUT / "gate-room-source-player.png")
    reference = render_stage(room, "resize", OUT, "gate-room-reference", (640, 512), Image.Resampling.NEAREST)
    save_render(reference, OUT / "gate-room-source-camera.png")
    actual = render_stage(Image.open(OUT / "native-gate-room.png"), "convert", OUT, "gate-room-native", "RGB")
    bounds = ImageChops.difference(actual, Image.new("RGB", actual.size, actual.getpixel((0, 0)))).getbbox()
    actual = render_stage(actual, "crop", OUT, "gate-room-native", bounds)
    reference = render_stage(reference, "convert", OUT, "gate-room-reference", "RGB")
    assert actual.size == reference.size
    delta = ImageChops.difference(actual, reference)
    save_render(delta, OUT / "gate-room-difference.png")
    results["gate-room"] = sum(pixel != (0, 0, 0) for pixel in delta.getdata())
    print(json.dumps(results, sort_keys=True))
    print("Whole maps, source cameras and native differences retained for visual review.")


def capture(path):
    """Ask the native Rust renderer for offscreen source-view screenshots."""
    archive_existing(OUT)
    env = dict(os.environ, CRYSTAL_RENDER_TEST_HEADLESS="1",
               CRYSTAL_RENDER_TEST_CAPTURE_TARGET="image",
               CRYSTAL_RENDER_TEST_WINDOW_SIZE="1280x800")
    for key, name, x, y in [("town-gate", "MnGrandMarais", 8, 18),
                             ("gunflint-mouth", "MnGrandMarais", 4, 8),
                             ("west-quay", "MnGrandMarais", 3, 22),
                             ("north-shore-gate", "MnR11C00R00", 42, 4),
                             ("gate-room", "MnGrandMaraisShoreGate", 8, 4),
                             ("harbor-water", "MnGrandMarais", 29, 34)]:
        image = OUT / f"native-{key}.png"
        retain_render_file(image)
        command = [str(ROOT / "target/northstar-story/wasm-dev/examples/render_at_location"),
                   "--pack", str(path), "--map", name, "--x", str(x), "--y", str(y),
                   "--view", "2d", "--hour", "12", "--screenshot", str(image)]
        with (ROOT / ".local" / f"north-shore-capture-{key}.log").open("w") as log:
            subprocess.run(command, env=env, cwd=ROOT, stdout=log, stderr=subprocess.STDOUT, check=True)
        retain_render_file(image)
        print("Native headless capture:", key, flush=True)


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--published", type=Path)
    parser.add_argument("--capture", action="store_true")
    args = parser.parse_args()
    if args.capture:
        assert args.published, "--capture requires a published pack path"
        capture(args.published.resolve())
    published(args.published) if args.published else preview()
