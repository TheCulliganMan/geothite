#!/usr/bin/env python3
"""Read back native ambient clips, verify static scenery, and retain animation previews."""
import argparse
import copy
import io
import json
from pathlib import Path

from PIL import Image

from render_archive import save_render
from render_reference import SourceArt, load_pack


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--pack", type=Path, required=True)
    parser.add_argument("--base", type=Path, required=True)
    parser.add_argument("--recipe", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    recipe = json.loads(args.recipe.read_text())
    pack, base = load_pack(args.pack), load_pack(args.base)
    assert pack["data"] == base["data"], "Animation changed gameplay data"
    for path, payload in base["runtime_files"].items():
        assert pack["runtime_files"][path] == payload, path
    assert pack.get("audio_manifest") == base.get("audio_manifest")
    files = pack["runtime_files"]
    animation_path = "data/tilesets/mn_campfire_animations.json"
    assert set(files) - set(base["runtime_files"]) == {animation_path}
    clips = json.loads(bytes(files[animation_path]))
    layout = bytes(files["data/tilesets/mn_campfire_metatiles.bin"])
    palette_map = pack["data"]["tilesets"]["mn_campfire"]["palette_map"]
    original = json.loads((args.output / "frame-0.indices.json").read_text())
    base_sheet = Image.open(io.BytesIO(bytes(files["gfx/tilesets/mn_campfire.png"]))).convert("RGB")
    ground_art = SourceArt(base)
    frames = {group: [] for group in ("morn", "day", "nite")}
    for phase, source_index in enumerate(recipe["frame_order"]):
        indices = original.copy()
        sheet = base_sheet.copy()
        for glyph_text, clip in clips.items():
            glyph = int(glyph_text)
            assert clip["frame_ticks"] == recipe["frame_ticks"]
            assert len(clip["frames"]) == len(recipe["frame_order"])
            assert layout.count(glyph) == 1, "Shared scenery glyph animates"
            tile = layout[128 * 16:129 * 16].index(glyph)
            assert tile in (1, 2, 5, 6), "Animation escapes the flame"
            physical = (glyph & 127) + (palette_map[glyph] >> 3) * 128
            sx, sy = physical % 16 * 8, physical // 16 * 8
            encoded = clip["frames"][phase]
            for y in range(8):
                lo, hi = encoded[y * 2:y * 2 + 2]
                for x in range(8):
                    value = ((lo >> (7 - x)) & 1) | (((hi >> (7 - x)) & 1) << 1)
                    indices[(tile // 4 * 8 + y) * 32 + tile % 4 * 8 + x] = value
                    gray = (255, 170, 85, 0)[value]
                    sheet.putpixel((sx + x, sy + y), (gray,) * 3)
        expected = json.loads((args.output / f"frame-{source_index}.indices.json").read_text())
        assert indices == expected, "Published clip does not match imported native frame"
        x, y, width, height = recipe["region"]
        for n, value in enumerate(indices):
            if not (x <= n % 32 < x + width and y <= n // 32 < y + height):
                assert value == original[n], "Stone, log or background moved"
        encoded_png = io.BytesIO()
        sheet.save(encoded_png, format="PNG")
        adapted = copy.copy(pack)
        adapted["runtime_files"] = dict(files)
        adapted["runtime_files"]["gfx/tilesets/mn_campfire.png"] = encoded_png.getvalue()
        art = SourceArt(adapted)
        for group in frames:
            blocks = art.blocks("mn_campfire", group)
            old = ground_art.blocks("mn_campfire", group)
            for block in range(128):
                assert blocks[block].tobytes() == old[block].tobytes(), "Native source scenery changed"
            cutout = blocks[128].convert("RGBA")
            cutout.putdata([(*rgb[:3], 255) if value else (0, 0, 0, 0)
                           for rgb, value in zip(cutout.getdata(), indices)])
            frames[group].append(cutout)
            save_render(cutout, args.output / f"{group}.phase-{phase}.1x.png")
            save_render(cutout.resize((256, 256), Image.Resampling.NEAREST),
                        args.output / f"{group}.phase-{phase}.8x.png")
    duration = recipe["frame_ticks"] * 1000 / 60
    for group, images in frames.items():
        enlarged = [im.resize((256, 256), Image.Resampling.NEAREST) for im in images]
        save_render(enlarged[0], args.output / f"{group}.animated.png", save_all=True,
                    append_images=enlarged[1:], duration=duration, loop=0, disposal=0, blend=0)
        colors = sorted({rgb for im in images for rgb in im.getdata() if rgb[3]})
        assert len(colors) < 255
        palette = [0, 0, 0] + [v for rgb in colors for v in rgb[:3]]
        palette += [0] * (768 - len(palette))
        lookup = {rgb: index + 1 for index, rgb in enumerate(colors)}
        gif_frames = []
        for frame in enlarged:
            gif = Image.new("P", frame.size)
            gif.putpalette(palette)
            gif.putdata([lookup[rgb] if rgb[3] else 0 for rgb in frame.getdata()])
            gif_frames.append(gif)
        save_render(gif_frames[0], args.output / f"{group}.animated.gif", save_all=True,
                    append_images=gif_frames[1:], duration=duration, loop=0,
                    transparency=0, disposal=2, optimize=False)
        contact = Image.new("RGBA", (len(enlarged) * 256, 256))
        for phase, frame in enumerate(enlarged):
            contact.paste(frame, (phase * 256, 0))
        save_render(contact, args.output / f"{group}.frames.png")
    print(f'Verified {len(clips)} native glyph clips, four phases, stationary scenery, original runtime/audio/data, and morning/day/night colors.')


if __name__ == "__main__":
    main()
