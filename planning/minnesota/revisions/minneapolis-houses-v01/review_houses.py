#!/usr/bin/env python3
"""Detached Minneapolis house study using unchanged native 8px glyphs.

This renders source only. It never writes a crystalpack or canonical town data.
"""
import argparse
from copy import deepcopy
import hashlib
import json
from pathlib import Path
import sys
import time

from PIL import Image, ImageDraw

ROOT = Path(__file__).resolve().parents[4]
sys.path.insert(0, str(ROOT / "planning/minnesota/tools"))
from render_reference import SourceArt, load_pack
from review_north_shore import oriented_sprite
from render_archive import save_render


def digest(value):
    return hashlib.sha256(value).hexdigest()


def source_block(block, tileset="kanto"):
    return [dict(tileset=tileset, block=f"{block:02x}", glyph=[i % 4, i // 4])
            for i in range(16)]


def copy_quadrant(target, origin, destination, source):
    tx, ty = destination
    sx, sy = source
    for dy in range(2):
        for dx in range(2):
            target[(ty * 2 + dy) * 4 + tx * 2 + dx] = origin[
                (sy * 2 + dy) * 4 + sx * 2 + dx]


def render(pack, folder):
    started = time.perf_counter()
    folder.mkdir(parents=True, exist_ok=False)
    art = SourceArt(pack)
    module = pack["data"]["maps"]["MnMinneapolis"]
    tileset = module["attributes"]["tileset_name"]
    native = art.blocks(tileset)
    before = list(module["blocks"])
    assert len(native) == 0x9d and len(before) == 40 * 32
    recipes = []

    def recipe(name, glyphs, collision):
        key = dict(glyphs=glyphs, collision=collision)
        for old in recipes:
            if old["glyphs"] == glyphs and old["collision"] == collision:
                return int(old["id"], 16)
        block = dict(id=f"{0x9d + len(recipes):02x}", name=name,
                     glyphs=glyphs, collision=collision)
        recipes.append(block)
        return int(block["id"], 16)

    wall = ["WALL"] * 4
    roof_gable = [recipe("native diamond gable left", source_block(0x0c), wall),
                  recipe("native diamond gable right", source_block(0x0e), wall)]
    roof_shallow = [recipe("native shallow pitch left", source_block(0x38), wall),
                    recipe("native shallow pitch right", source_block(0x39), wall)]
    # Remove the source house's original doorway using an exact native window.
    # Put that same complete 16px doorway at each existing exterior warp.
    left = source_block(0x3c)
    copy_quadrant(left, source_block(0x3d), (1, 1), (0, 1))
    left_id = recipe("weatherboard left with two native windows", left, wall)
    right_ids = {}
    for quadrant, index in [((0, 1), 2), ((1, 1), 3)]:
        right = source_block(0x3d)
        copy_quadrant(right, source_block(0x3c), quadrant, (1, 1))
        collision = list(wall)
        collision[index] = "DOOR"
        right_ids[index] = recipe("weatherboard right, fixed doorway quadrant "
                                  + str(index), right, collision)

    wood_ground = [recipe("native Ecruteak timber house left", source_block(0x26, "johto"), wall),
                   recipe("native Ecruteak timber house right doorway",
                          source_block(0x27, "johto"), ["WALL", "WALL", "DOOR", "WALL"])]
    studio_roof = []
    for source in [0x12, 0x13]:
        glyphs = source_block(source, "johto_modern")
        glyphs[8:] = source_block(0x20, "johto_modern")[:8]
        studio_roof.append(recipe("studio blue eave over full upper glazing", glyphs, wall))
    studio_ground = []
    for source in [0x10, 0x11]:
        glyphs = source_block(source, "johto_modern")
        collision = list(wall)
        if source == 0x11:
            copy_quadrant(glyphs, source_block(0x1a, "johto_modern"), (1, 1), (1, 1))
            collision[3] = "DOOR"
        studio_ground.append(recipe("studio cyan brick ground window band", glyphs, collision))

    houses = [
        dict(id="NortheastHome", bounds=[36, 3, 38, 5], door_core=[74, 9],
             roof=roof_gable, ground=[left_id, right_ids[2]], purpose="Front gable home beside the library"),
        dict(id="RiverNaturalist", bounds=[26, 7, 28, 9], door_core=[55, 17],
             roof=roof_shallow, ground=[left_id, right_ids[3]], purpose="Lower pitched cottage beside the garden street"),
        dict(id="GardenFlat", bounds=[29, 7, 31, 9], door_core=[60, 17],
             roof=[0x05, 0x05], ground=wood_ground,
             purpose="Native warm timber cottage against its existing solid pine back strip"),
        dict(id="PatternStudio", bounds=[32, 7, 34, 9], door_core=[67, 17],
             roof=studio_roof, ground=studio_ground,
             purpose="Two glazed storeys under a short blue eave; bookbinder retains masonry"),
    ]
    after = list(before)
    for house in houses:
        x, y, x1, y1 = house["bounds"]
        assert (x1 - x, y1 - y) == (2, 2)
        rows = [house["roof"], house["ground"]]
        for dy, row in enumerate(rows):
            for dx, block in enumerate(row):
                after[(y + dy) * 40 + x + dx] = block

    colors = {}
    for block in recipes:
        image = Image.new("RGB", (32, 32))
        for i, glyph in enumerate(block["glyphs"]):
            src = art.blocks(glyph["tileset"])[int(glyph["block"], 16)]
            x, y = glyph["glyph"]
            image.paste(src.crop((x * 8, y * 8, x * 8 + 8, y * 8 + 8)),
                        (i % 4 * 8, i // 4 * 8))
        colors[int(block["id"], 16)] = image

    def whole(grid):
        result = Image.new("RGB", (1280, 1024))
        for i, block in enumerate(grid):
            result.paste(colors[block] if block in colors else native[block],
                         (i % 40 * 32, i // 40 * 32))
        return result

    old_image, new_image = whole(before), whole(after)
    save_render(old_image, folder / "whole-city-before.png")
    save_render(new_image, folder / "whole-city-candidate.png")
    # Player cameras are fixed core positions and native 160x144 pixels.
    cameras = [("northeast-home", (74, 10)), ("garden-pair", (58, 18)),
               ("pattern-studio", (67, 18))]
    contacts = Image.new("RGB", (640, len(cameras) * 318), "#14212b")
    draw = ImageDraw.Draw(contacts)
    for row, (name, (x, y)) in enumerate(cameras):
        for column, (tag, source) in enumerate([("before", old_image), ("candidate", new_image)]):
            scene = source.convert("RGBA")
            scene.alpha_composite(oriented_sprite(pack, "SPRITE_CHRIS", "north"), (x * 16, y * 16))
            left = min(max(x * 16 - 80, 0), 1280 - 160)
            top = min(max(y * 16 - 72, 0), 1024 - 144)
            crop = scene.crop((left, top, left + 160, top + 144))
            save_render(crop, folder / f"{name}-{tag}-native160.png")
            contacts.paste(crop.resize((320, 288), Image.Resampling.NEAREST), (column * 320, row * 318 + 26))
            draw.text((column * 320 + 8, row * 318 + 8), f"{name}: {tag}", fill="white")
    save_render(contacts, folder / "player-camera-before-candidate.png")
    crop_box = (25 * 32, 0, 39 * 32, 11 * 32)
    cluster = Image.new("RGB", (896, 378), "#14212b")
    draw = ImageDraw.Draw(cluster)
    for i, (title, source) in enumerate([("Native installed before", old_image), ("Detached source candidate", new_image)]):
        cluster.paste(source.crop(crop_box), (i * 448, 26))
        draw.text((i * 448 + 8, 8), title, fill="white")
    save_render(cluster, folder / "northeast-cluster-before-candidate.png")
    # Keep the real four native palette banks visible in the art decision.
    bank_examples = [("gable", roof_gable, [left_id, right_ids[2]]),
                     ("shallow", roof_shallow, [left_id, right_ids[3]]),
                     ("timber", [0x05, 0x05], wood_ground),
                     ("studio", studio_roof, studio_ground)]
    bank_sheet = Image.new("RGB", (4 * 176, len(bank_examples) * 166), "#14212b")
    bank_draw = ImageDraw.Draw(bank_sheet)
    for column, group in enumerate(["morn", "day", "nite", "dark"]):
        for row, (label, roof, ground) in enumerate(bank_examples):
            glyph_rows = [roof, ground]
            house_image = Image.new("RGB", (64, 64))
            for by, glyph_row in enumerate(glyph_rows):
                for bx, block_id in enumerate(glyph_row):
                    if block_id < 0x9d:
                        house_image.paste(art.blocks(tileset, group)[block_id], (bx * 32, by * 32))
                        continue
                    block = next(b for b in recipes if int(b["id"], 16) == block_id)
                    for i, glyph in enumerate(block["glyphs"]):
                        src = art.blocks(glyph["tileset"], group)[int(glyph["block"], 16)]
                        x, y = glyph["glyph"]
                        house_image.paste(src.crop((x * 8, y * 8, x * 8 + 8, y * 8 + 8)),
                                          (bx * 32 + i % 4 * 8, by * 32 + i // 4 * 8))
            bank_sheet.paste(house_image.resize((128, 128), Image.Resampling.NEAREST),
                             (column * 176 + 20, row * 166 + 26))
            bank_draw.text((column * 176 + 8, row * 166 + 8),
                           group + ": " + label, fill="white")
    save_render(bank_sheet, folder / "four-native-palette-banks.png")
    ts = pack["data"]["tilesets"][tileset]
    changes = []
    for i, (old, new) in enumerate(zip(before, after)):
        if old == new:
            continue
        collision = (ts["collision"][f"{new:02x}"] if new < 0x9d else
                     next(b["collision"] for b in recipes if int(b["id"], 16) == new))
        assert collision == ts["collision"][f"{old:02x}"], (i, old, new)
        changes.append(dict(metatile=[i % 40, i // 40], before=f"{old:02x}", after=f"{new:02x}", collision=collision))
    warps = module["events"]["warps"]
    for house in houses:
        assert any([w["x"], w["y"]] == house["door_core"] for w in warps)
    spec = dict(schema_version=1, status="DETACHED_SOURCE_CANDIDATE_NOT_NATIVE_PUBLICATION",
                parent_pack="content-packs/minnesota-city-river-bend-v01.crystalpack",
                expected_parent_file_sha256="57eefb675b9cb32ac66a01819f56961392e51678a6f9dd959888fbb9427e4e62",
                map="MnMinneapolis", tileset=tileset, expected_existing_metatiles=157,
                new_metatiles=recipes, map_patches=changes, houses=houses,
                preserve="All old glyph pixels, palette banks, metatiles, collisions, maps, warps, scripts, objects, rooms, services, audio, recovery, HOME and reports exactly except declared new art and 16 block identifiers plus their native source mirror.")
    (folder / "house-exteriors.contract.json").write_text(json.dumps(spec, indent=2) + "\n")
    (folder / "receipt.json").write_text(json.dumps(dict(status=spec["status"], changed_metatiles=len(changes), appended_metatiles=len(recipes), identical_collision_quadrants=40*32*4, door_coordinates_preserved=[h["door_core"] for h in houses], camera_positions=cameras, warm_render_seconds=round(time.perf_counter()-started,3), source_script_sha256=digest(Path(__file__).read_bytes()), source_pack_sha256=spec["expected_parent_file_sha256"]), indent=2) + "\n")
    print(json.dumps(dict(output=str(folder), changes=len(changes), recipes=len(recipes), seconds=time.perf_counter()-started)))


if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    pack_path = ROOT / "content-packs/minnesota-city-river-bend-v01.crystalpack"
    assert digest(pack_path.read_bytes()) == "57eefb675b9cb32ac66a01819f56961392e51678a6f9dd959888fbb9427e4e62"
    render(load_pack(pack_path), args.output)
