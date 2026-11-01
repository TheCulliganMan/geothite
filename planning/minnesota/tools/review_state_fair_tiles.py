#!/usr/bin/env python3
"""Assemble the fair art handoff from published blocks at native player scale.

This is an art gallery, not an authored or playable fairgrounds map. Run the
independent review_minnesota_tiles.py preservation/bitplane check first.
"""
import argparse
import json
from pathlib import Path

from PIL import Image, ImageChops, ImageDraw

from render_archive import save_render
from render_reference import SourceArt, load_pack

ROOT = Path(__file__).resolve().parents[3]


def compare_captures(handoff, output, paths):
    """Require every object's complete foreground to match a native capture."""
    captures = [(path, Image.open(path).convert("RGB")) for path in paths]
    proof = []
    for obj in handoff["objects"]:
        source = Image.open(output / f'{obj["id"]}.cutout.1x.png').convert("RGBA")
        ink = [(x, y, source.getpixel((x, y))[:3])
               for y in range(source.height) for x in range(source.width)
               if source.getpixel((x, y))[3] == 255]
        anchors = [ink[n * (len(ink) - 1) // 4] for n in range(5)]
        matched = None
        for path, capture in captures:
            pixels = capture.load()
            for scale in (1, 2, 3, 4):
                enlarged = source.resize((source.width * scale, source.height * scale), Image.Resampling.NEAREST)
                ax, ay, color = anchors[0]
                for y in range(capture.height):
                    for x in range(capture.width):
                        ox, oy = x - ax * scale, y - ay * scale
                        if (pixels[x, y] != color or ox < 0 or oy < 0
                                or ox + enlarged.width > capture.width
                                or oy + enlarged.height > capture.height):
                            continue
                        if any(pixels[ox + px * scale, oy + py * scale] != rgb
                               for px, py, rgb in anchors[1:]):
                            continue
                        region = capture.crop((ox, oy, ox + enlarged.width, oy + enlarged.height))
                        difference = ImageChops.difference(region, enlarged.convert("RGB"))
                        masked = ImageChops.multiply(difference, enlarged.getchannel("A").convert("RGB"))
                        if masked.getbbox() is None:
                            matched = dict(object=obj["id"], capture=str(path), origin=[ox, oy],
                                           scale=scale, native_foreground_pixels=len(ink),
                                           compared_capture_pixels=len(ink) * scale * scale)
                            break
                    if matched:
                        break
                if matched:
                    break
            if matched:
                break
        assert matched, f'Native capture does not contain exact complete {obj["id"]} pixels'
        proof.append(matched)
    (output / "verification/native-pixel-proof.json").write_text(json.dumps(proof, indent=2) + "\n")
    print(f"Every foreground pixel of all {len(proof)} objects matches a native renderer capture.")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--pack", type=Path, required=True)
    parser.add_argument("--handoff", type=Path,
                        default=ROOT / "planning/minnesota/tiles/state-fair-v1/handoff.json")
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--capture", type=Path, action="append", default=[],
                        help="Optional native captures; together they must show every complete object.")
    args = parser.parse_args()
    handoff = json.loads(args.handoff.read_text())
    pack = load_pack(args.pack)
    art = SourceArt(pack)
    tileset = handoff["tileset_id"]
    args.output.mkdir(parents=True, exist_ok=True)
    # Fixed review placements in a plain art gallery; these are not game ports.
    origins = [(1, 1), (4, 1), (6, 1), (4, 3)]
    assert len(handoff["objects"]) == len(origins)
    for group in ("morn", "day", "nite"):
        blocks = art.blocks(tileset, group)
        gallery = art.render(tileset, 8, 5, [1] * 40, group)
        for obj, (ox, oy) in zip(handoff["objects"], origins):
            width, height = obj["size_metatiles"]
            assembly = Image.new("RGB", (width * 32, height * 32))
            for y, row in enumerate(obj["blocks"]):
                for x, block in enumerate(row):
                    number = int(block, 16)
                    assert pack["data"]["tilesets"][tileset]["collision"][block] == ["WALL"] * 4
                    assembly.paste(blocks[number], (x * 32, y * 32))
                    gallery.paste(blocks[number], ((ox + x) * 32, (oy + y) * 32))
            save_render(assembly, args.output / f'{obj["id"]}.{group}.native.png')
            save_render(assembly.resize((width * 256, height * 256), Image.Resampling.NEAREST),
                        args.output / f'{obj["id"]}.{group}.8x.png')
        save_render(gallery, args.output / f"assembled.{group}.native.png")
        save_render(gallery.resize((1024, 640), Image.Resampling.NEAREST),
                    args.output / f"assembled.{group}.4x.png")
        # Original handheld viewport, cropped without resizing or extra scenery.
        view = gallery.crop((0, 0, 160, 144))
        save_render(view, args.output / f"wheel-approach.{group}.160x144.png")
        save_render(view.resize((640, 576), Image.Resampling.NEAREST),
                    args.output / f"wheel-approach.{group}.4x.png")
    contact = Image.new("RGB", (1024, 600), (38, 48, 58))
    draw = ImageDraw.Draw(contact)
    draw.text((16, 15), "STATE FAIRGROUNDS / NATIVE PIXEL ASSETS", fill="white")
    for obj, (x, y, scale) in zip(handoff["objects"],
                                [(16, 45, 8), (570, 45, 7), (800, 45, 7), (570, 315, 7)]):
        rows = obj["items"]
        cutout = Image.new("RGBA", tuple(size * 32 for size in obj["size_metatiles"]))
        for row, items in enumerate(rows):
            for column, item in enumerate(items):
                cutout.paste(Image.open(args.output / f"{item}.1x.png"), (column * 32, row * 32))
        save_render(cutout, args.output / f'{obj["id"]}.cutout.1x.png')
        enlarged = cutout.resize(tuple(size * scale for size in cutout.size), Image.Resampling.NEAREST)
        save_render(enlarged, args.output / f'{obj["id"]}.cutout.{scale}x.png')
        contact.paste(enlarged, (x, y), enlarged)
        draw.text((x, y + enlarged.height + 5), obj["name"], fill="white")
    save_render(contact, args.output / "contact.png")
    if args.capture:
        (args.output / "verification").mkdir(exist_ok=True)
        compare_captures(handoff, args.output, args.capture)
    print("Assembled four fair objects from seven published blocks at morning/day/night; saved native viewport crops and cutouts.")


if __name__ == "__main__":
    main()
