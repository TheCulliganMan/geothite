#!/usr/bin/env python3
"""Review authored shop terrain using unchanged native art; never write a pack."""

import argparse
import json
from pathlib import Path

from PIL import Image

from render_archive import save_render
from render_reference import SourceArt, load_pack

ROOT = Path(__file__).resolve().parents[3]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--source", type=Path, default=ROOT / "planning/minnesota/landmarks/north-shore-shops.json")
    parser.add_argument("--pack", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--group", choices=["morn", "day", "nite"], default="day")
    parser.add_argument("--published", action="store_true")
    args = parser.parse_args()
    source = json.loads(args.source.read_text())
    pack = load_pack(args.pack)
    art = SourceArt(pack)
    maps = pack["data"]["maps"]
    revisions = {}
    for shop in source["shops"]:
        module = maps[shop["exterior_map"]]
        blocks = list(module["blocks"])
        width = module["attributes"]["width"]
        for cell in shop["cells"]:
            x, y = cell["metatile"]
            before, after = (int(cell[k], 16) for k in ["before", "after"])
            assert blocks[y * width + x] == (after if args.published else before), cell
            blocks[y * width + x] = after
        revisions[shop["exterior_map"]] = blocks
        room = shop["room"]
        room_blocks = [int(b, 16) for row in room["rows"] for b in row.split()]
        template = maps[room["template"]]
        if args.published:
            assert maps[shop["id"]]["blocks"] == room_blocks
        tileset = template["attributes"]["tileset_name"]
        rw, rh = room["dimensions_metatiles"]
        save_render(art.render(tileset, rw, rh, room_blocks, group="indoor"), args.output / f'{shop["id"]}-interior.png')
        image = art.render("mn_north_shore", width, module["attributes"]["height"], blocks, group=args.group)
        save_render(image, args.output / f'{shop["exterior_map"]}-whole.png')
        x, y = shop["porch"]
        save_render(image.crop((x * 16 - 80, y * 16 - 64, x * 16 + 80, y * 16 + 80)), args.output / f'{shop["id"]}-door-camera.png')
        # Include the complete site and its tie to the existing road.
        xs = [c["metatile"][0] for c in shop["cells"]]
        ys = [c["metatile"][1] for c in shop["cells"]]
        crop = image.crop(((min(xs) - 2) * 32, max(0, min(ys) - 2) * 32,
                           (max(xs) + 3) * 32, min(24, max(ys) + 3) * 32))
        save_render(crop.resize((crop.width * 2, crop.height * 2), Image.Resampling.NEAREST), args.output / f'{shop["id"]}-site.png')
    whole = Image.new("RGBA", (56 * 32, 96 * 32))
    for name, module in maps.items():
        if not name.startswith("MnR11C"):
            continue
        column, row = int(name[6:8]), int(name[9:11])
        attributes = module["attributes"]
        image = art.render(attributes["tileset_name"], attributes["width"], attributes["height"], revisions.get(name, module["blocks"]), group=args.group)
        whole.paste(image, (column * 1024, row * 768))
    save_render(whole, args.output / "whole-north-shore.png")
    print(args.output)


if __name__ == "__main__":
    main()
