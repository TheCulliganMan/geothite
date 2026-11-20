#!/usr/bin/env python3
"""Show which native surfaces players face when reading in Minnesota towns and routes."""
import argparse
from collections import defaultdict
from pathlib import Path
import sys

from PIL import Image, ImageDraw

sys.path.insert(0, str(Path(__file__).resolve().parents[2] / "tools"))
from render_reference import SourceArt, load_pack


def inspect(pack):
    art = SourceArt(pack)
    groups = defaultdict(list)
    samples = {}
    for name, room in pack["data"]["maps"].items():
        attrs = room["attributes"]
        if not (attrs["map_group_constant"] or "").startswith("MAPGROUP_MINNESOTA"):
            continue
        if attrs["environment"] not in {"TOWN", "ROUTE"}:
            continue
        for event in room["events"]["bg_events"]:
            if event["event_type"] != "BGEVENT_READ":
                continue
            x, y = event["x"], event["y"]
            block = room["blocks"][y // 2 * attrs["width"] + x // 2]
            tile = art.blocks(attrs["tileset_name"], "day")[block]
            left, top = x % 2 * 16, y % 2 * 16
            face = tile.crop((left, top, left + 16, top + 16)).convert("RGB")
            # Equal pixel patterns stay together across seasonal palette swaps.
            colors = {}
            signature = bytes(colors.setdefault(face.getpixel((col, row)), len(colors))
                              for row in range(16) for col in range(16))
            groups[signature].append((name, x, y, event["script"]))
            samples.setdefault(signature, face)
    return groups, samples


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("pack", type=Path)
    parser.add_argument("--gallery", type=Path, help="save an indexed sheet of the printed faces")
    args = parser.parse_args()
    groups, samples = inspect(load_pack(args.pack))
    keys = sorted(groups, key=lambda k: (-len(groups[k]), groups[k][0]))
    print(f"{sum(map(len, groups.values()))} outdoor reading events, {len(keys)} distinct shapes.")
    if args.gallery:
        cols = 8
        sheet = Image.new("RGB", (cols * 80, ((len(keys) + cols - 1) // cols) * 69), "#1b2522")
        draw = ImageDraw.Draw(sheet)
    for index, key in enumerate(keys, 1):
        for name, x, y, label in groups[key]:
            print(f"{index}\t{name}\t{x},{y}\t{label}")
        if args.gallery:
            x, y = (index - 1) % cols * 80, (index - 1) // cols * 69
            sheet.paste(samples[key].resize((48, 48), Image.Resampling.NEAREST), (x, y))
            draw.text((x, y + 50), f"{index} ({len(groups[key])})", fill="white")
    if args.gallery:
        sheet.save(args.gallery)


if __name__ == "__main__":
    main()
