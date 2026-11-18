#!/usr/bin/env python3
"""Verify the published beach design in a pair of compiled Minnesota packs."""
import argparse
import json
import re
import sys
from pathlib import Path

import cbor2
from PIL import Image, ImageDraw

SOURCE = Path(__file__).resolve().parent
sys.path.insert(0, str(SOURCE.parents[1] / "tools"))
from audit_regional_design import Terrain
from render_reference import SourceArt
from review_north_shore import oriented_sprite

GYM = "MnBrainerdGym"
SHEET = "mn_gym_6"


def load(path):
    with path.open("rb") as file:
        header = file.read(22)
        assert header[:12] == b"CRYSTALPACK\0" and int.from_bytes(header[14:18], "big") == path.stat().st_size - 22
        return cbor2.load(file)


def room_image(pack):
    room = pack["data"]["maps"][GYM]
    art = SourceArt(pack).render(SHEET, 9, 7, room["blocks"], "day")
    actors = art.copy()
    for obj in room["objects"]:
        direction = {"LEFT": "west", "RIGHT": "east", "DOWN": "south"}[obj["spritemovedata"].split("_")[-1]]
        sprite = oriented_sprite(pack, obj["sprite"], direction)
        actors.paste(sprite, (obj["x"] * 16, obj["y"] * 16 - 4), sprite)
    return art, actors


def cameras(manifest, revised, output):
    output.mkdir(parents=True, exist_ok=True)
    canvas = Image.new("RGB", (664, 978), "#222e2c")
    labels = ImageDraw.Draw(canvas)
    for col, (pack, title) in enumerate(((manifest, "Crossings"), (revised, "Lakeside"))):
        _, room = room_image(pack)
        room.resize((576, 448), Image.Resampling.NEAREST).save(output / f"{title.lower()}-room.png")
        player = oriented_sprite(pack, "SPRITE_CHRIS", "north")
        for row, (x, y, caption) in enumerate(((8, 12, "Entrance"), (6, 9, "West shore"), (10, 5, "Far sandbar"))):
            frame = room.copy()
            frame.paste(player, (x * 16, y * 16 - 4), player)
            left = max(0, min(room.width - 160, x * 16 - 72))
            top = max(0, min(room.height - 144, y * 16 - 64))
            view = frame.crop((left, top, left + 160, top + 144))
            canvas.paste(view.resize((320, 288), Image.Resampling.NEAREST), (8 + col * 328, row * 326 + 28))
            labels.text((8 + col * 328, row * 326 + 6), f"{title} · {caption}", fill="white")
    canvas.save(output / "brainerd-camera-comparison.png")


def audit(old, new):
    a, b = Terrain(old, GYM), Terrain(new, GYM)
    assert a.tiles == b.tiles and len(b.tiles) == 252
    assert a.map["objects"] == b.map["objects"] and a.map["events"] == b.map["events"]
    assert len(b.occupied) == 4 and len(b.warps) == 3
    walkable = b.reach([(8, 11)])
    assert all(b.tiles[8, y] == "FLOOR" and (8, y) not in b.occupied for y in range(4, 13))
    assert all((8, y) in walkable for y in range(4, 12))
    sight = set()
    trainer_scripts = set()
    for obj in b.map["objects"]:
        if obj["radius"]:
            assert obj["object_type"] == "OBJECTTYPE_TRAINER" and obj["radius"] == 2
            step = {"LEFT": -1, "RIGHT": 1}[obj["spritemovedata"].split("_")[-1]]
            sight.update((obj["x"] + step * n, obj["y"]) for n in (1, 2))
            trainer_scripts.add(obj["script"])
    assert len(trainer_scripts) == 2 and len(sight) == 4 and sight <= walkable
    assert b.map["scripts"]["MnBrainerdGymV2Qualify"][-1] == {"command":"sjump", "args":["MnBrainerdGymV2QualifyAccepted"]}
    assert any(cmd["command"] == "callback" and cmd["args"] == ["MAPCALLBACK_TILES", "MnBrainerdGymRefreshGates"]
               for cmd in b.map["map_script_section_commands"])

    spec = json.loads((SOURCE / "shallows.json").read_text())
    assert a.blocks == spec["prior"] and b.blocks == spec["room"]
    assert len(spec["glyphs"]) == 155 and len(spec["blocks"]) == 38
    old_files, new_files = old["runtime_files"], new["runtime_files"]
    tilename = f"gfx/tilesets/{SHEET}.2bpp"
    before, after = bytes(old_files[tilename]), bytes(new_files[tilename])
    assert len(before) == len(after) and before[:27 * 16] == after[:27 * 16] and before[182 * 16:] == after[182 * 16:]
    palette = new["data"]["tilesets"][SHEET]["palette_map"]
    for index, glyph in enumerate(spec["glyphs"], start=27):
        assert 0 <= palette[index] == glyph["palette"] < 8
        pixels = "".join(str(((after[index*16+y*2] >> x) & 1) | (((after[index*16+y*2+1] >> x) & 1) << 1))
                         for y in range(8) for x in reversed(range(8)))
        assert pixels == "".join(glyph["rows"])
    packed = bytes(new_files[f"data/tilesets/{SHEET}_metatiles.bin"])
    assert packed[:16*16] == bytes(old_files[f"data/tilesets/{SHEET}_metatiles.bin"])
    assert packed[16*16:] == bytes(tile for block in spec["blocks"] for tile in block["glyphs"])
    palette_bytes = bytes(new_files[f"gfx/tilesets/{SHEET}.pal"])
    assert palette_bytes == (SOURCE / "lake.pal").read_bytes()
    colors = [list(map(int, re.findall(r"\d+", line.split(";")[0]))) for line in palette_bytes.decode().splitlines()]
    assert len(colors) == 8 and all(len(group) == 12 and all(0 <= v < 32 for v in group) for group in colors)
    sand = tuple((v << 3) | (v >> 2) for v in colors[5][:3])
    art, _ = room_image(new)
    for x, y in b.occupied:
        feet = art.crop((x * 16, y * 16 + 8, (x+1) * 16, (y+1) * 16))
        sand_pixels = sum(count for count, color in feet.getcolors() if color == sand)
        assert sand_pixels > 64, (x, y, "NPC footing should be visibly sand")
    print("252 collision squares, 3 warps, 4 dry character positions, 4 reachable trainer sight squares, "
          "8 straight steps to Sivren, 155 physical tiles, 38 native metatiles, 8 RGB555 palettes.")


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("parent", type=Path)
    parser.add_argument("revised", type=Path)
    parser.add_argument("--render-dir", type=Path)
    args = parser.parse_args()
    parent, revised = load(args.parent), load(args.revised)
    audit(parent, revised)
    if args.render_dir:
        cameras(parent, revised, args.render_dir)
