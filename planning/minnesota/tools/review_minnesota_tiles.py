#!/usr/bin/env python3
"""Review original props through the existing packed-source art renderer."""
import argparse
import copy
import json
from pathlib import Path

from PIL import Image, ImageDraw

from render_archive import save_render
from render_reference import SourceArt, load_pack

ROOT = Path(__file__).resolve().parents[3]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--pack", type=Path, required=True)
    parser.add_argument("--base", type=Path)
    parser.add_argument("--catalog", type=Path, default=ROOT / "planning/minnesota/tiles/northwoods-v1/catalog.json")
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    catalog = json.loads(args.catalog.read_text())
    pack = load_pack(args.pack)
    art = SourceArt(pack)
    tileset = catalog["tileset_id"]
    if args.base:
        base = load_pack(args.base)
        for name, original in base["runtime_files"].items():
            assert pack["runtime_files"][name] == original, name
        normalized = copy.deepcopy(pack["data"])
        del normalized["tilesets"][tileset]
        for action in ("cut", "whirlpool"):
            normalized["field_moves"][action]["replacements"].pop(tileset, None)
        assert normalized == base["data"], "Unrelated game data changed"
        for name in ("audio_manifest", "audio_compression"):
            assert pack.get(name) == base.get(name), name
    args.output.mkdir(parents=True, exist_ok=True)
    source = Image.open(args.catalog.parent / "source.png")
    for item in catalog["items"]:
        x, y, width, height = item["crop"]
        save_render(source.crop((x, y, x + width, y + height)),
                    args.output / f'{item["id"]}.source-crop.png')
    files = pack["runtime_files"]
    layout = bytes(files[f"data/tilesets/{tileset}_metatiles.bin"])
    original_layout = bytes(files["data/tilesets/johto_metatiles.bin"])
    graphics = bytes(files[f"gfx/tilesets/{tileset}.2bpp"])
    palette_map = pack["data"]["tilesets"][tileset]["palette_map"]
    for index, item in enumerate(catalog["items"]):
        expected = json.loads((args.output.parent / f'{item["id"]}.indices.json').read_text())
        assert len(expected) == 1024
        for tile in range(16):
            tx, ty = tile % 4 * 8, tile // 4 * 8
            ink = [expected[(ty + y) * 32 + tx + x] for y in range(8) for x in range(8)]
            glyph = layout[(128 + index) * 16 + tile]
            tile_palette = item.get("tile_palettes", [item["palette"]] * 16)[tile]
            if not any(ink):
                ground = 2 if tile_palette == 2 else 1
                assert glyph == original_layout[ground * 16 + tile]
                continue
            palette = palette_map[glyph]
            assert palette & 7 == tile_palette
            physical = (glyph & 127) + (palette >> 3) * (len(graphics) // 32)
            actual = []
            for y in range(8):
                lo, hi = graphics[physical * 16 + y * 2:physical * 16 + y * 2 + 2]
                actual.extend(((lo >> bit) & 1) | (((hi >> bit) & 1) << 1) for bit in range(7, -1, -1))
            assert actual == ink, (item["id"], tile)
    for group in ("morn", "day", "nite"):
        original = art.blocks("johto", group)
        blocks = art.blocks(tileset, group)
        assert len(blocks) == 128 + len(catalog["items"])
        for index in range(128):
            assert original[index].tobytes() == blocks[index].tobytes(), (group, index)
        # Each item sits in a three-block-wide vignette at unchanged map scale.
        columns = min(3, len(catalog["items"]))
        rows = (len(catalog["items"]) + columns - 1) // columns
        width, height = columns * 3, rows * 3
        gallery = art.render(tileset, width, height, [1] * (width * height), group)
        collisions = gallery.copy()
        draw = ImageDraw.Draw(collisions)
        for index, item in enumerate(catalog["items"]):
            x, y = (index % columns * 3 + 1) * 32, (index // columns * 3 + 1) * 32
            if item["palette"] == 2:
                for row in range(3):
                    for column in range(3):
                        pos = ((index % columns * 3 + column) * 32, (index // columns * 3 + row) * 32)
                        gallery.paste(blocks[2], pos)
                        collisions.paste(blocks[2], pos)
            gallery.paste(blocks[128 + index], (x, y))
            collisions.paste(blocks[128 + index], (x, y))
            for quadrant, collision in enumerate(item["collision"]):
                qx, qy = x + quadrant % 2 * 16, y + quadrant // 2 * 16
                draw.rectangle((qx, qy, qx + 15, qy + 15), outline="#ff6666" if collision == "WALL" else "#44eeee")
        save_render(gallery, args.output / f"{group}.native.png")
        save_render(gallery.resize((width * 128, height * 128), Image.Resampling.NEAREST), args.output / f"{group}.4x.png")
        save_render(collisions.resize((width * 128, height * 128), Image.Resampling.NEAREST), args.output / f"{group}.collision.4x.png")
    print(f'Verified {len(catalog["items"])} published prop bitplanes against importer indices and native ground margins; all 128 original blocks match Johto in morning, day and night.')


if __name__ == "__main__":
    main()
