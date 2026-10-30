#!/usr/bin/env python3
"""Retained headless outdoor passage captures against installed source pixels.

Uses SourceArt and the production location tester; never generates terrain,
publishes a pack, advances progression, or modifies a source image to hide a
discrepancy. The explicit 640x576 camera contract follows fullscreen_scaling:
three physical pixels per source texel, with additional terrain around the LCD.
"""
import argparse
import hashlib
import io
import json
import os
import re
from pathlib import Path
import shutil
import subprocess

from PIL import Image, ImageChops

from render_archive import retain_render_file, save_render
from render_reference import SourceArt, load_pack, room_sheet
from render_stages import render_stage
from review_north_shore import oriented_sprite

ROOT = Path(__file__).resolve().parents[3]
VIEWS = {
    "MN_R08": [("north-approach", (24, 61)), ("lake-bend", (28, 65)),
               ("grass-choice", (18, 64)), ("shore-turnout", (28, 71)),
               ("lower-bend", (23, 73)), ("lower-exit", (19, 80))],
    "MN_R15": [("shore-walk", (47, 23)), ("cabin", (53, 15)),
               ("western-junction", (22, 26)), ("meadow-landing", (22, 21)),
               ("flower-turn", (22, 28)), ("dock-approach", (46, 18))],
    "MN_R19": [("oak-hook", (25, 41)), ("habitat-approach", (25, 46)),
               ("board-nook", (23, 45)), ("west-loop", (21, 44)),
               ("reward-join", (29, 41)), ("upper-weakness", (35, 33))],
}


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def write(path, value):
    raw = (json.dumps(value, indent=2) + "\n").encode()
    frozen = path.with_name(path.stem + "-" + hashlib.sha256(raw).hexdigest() + path.suffix)
    if not frozen.exists():
        frozen.write_bytes(raw)
    else:
        assert frozen.read_bytes() == raw
    path.write_bytes(raw)


def actors(pack, terrain, module):
    image = terrain.convert("RGBA")
    for obj in module["objects"]:
        assert obj["pal"] == 0, "review explicit native palette override first"
        # This source layer records authored starting positions. Clockwise
        # patrols start facing right; a later moving pose must still fail the
        # full-frame comparison if it enters the captured viewport.
        # Native object_event_initial_facing gives STILL actors the south pose.
        # Static paper/notebook objects use that frame too; unknown movements
        # continue to fail instead of silently inventing a pose.
        movement = obj["spritemovedata"]
        if movement == "SPRITEMOVEDATA_STILL":
            facing = "south"
        elif movement == "SPRITEMOVEDATA_PATROL_CLOCKWISE":
            facing = "east"
        else:
            facing = {"UP": "north", "DOWN": "south", "LEFT": "west", "RIGHT": "east"}[
                movement.rsplit("_", 1)[-1]]
        # The production source-art sprite origin is the gameplay-tile origin.
        image.alpha_composite(oriented_sprite(pack, obj["sprite"], facing),
                              (obj["x"] * 16, obj["y"] * 16))
    return image


def grass_priority(pack, module, at):
    """Audit the production nonzero-index grass priority at this one anchor."""
    attr = module["attributes"]
    tileset = pack["data"]["tilesets"][attr["tileset_name"]]
    block = module["blocks"][at[1] // 2 * attr["width"] + at[0] // 2]
    tokens = tileset["collision"][f"{block:02x}"]
    quadrant = at[1] % 2 * 2 + at[0] % 2
    if tokens[quadrant] != "TALL_GRASS":
        assert tokens[quadrant] == "FLOOR"
        return set()
    files = pack["runtime_files"]
    sheet = Image.open(io.BytesIO(bytes(files[f'gfx/tilesets/{attr["tileset_name"]}.png']))).convert("RGB")
    layout = bytes(files[f'data/tilesets/{attr["tileset_name"]}_metatiles.bin'])
    count = sheet.width * sheet.height // 64
    result = set()
    for y in range(16):
        for x in range(16):
            sx, sy = at[0] % 2 * 16 + x, at[1] % 2 * 16 + y
            tile = layout[block * 16 + sy // 8 * 4 + sx // 8]
            bank = (tileset["palette_map"][tile] >> 3) & 1
            source = tile
            if bank and count % 2 == 0 and (tile & 127) + count // 2 < count:
                source = (tile & 127) + count // 2
            elif bank and (tile & 127) + 128 < count:
                source = (tile & 127) + 128
            elif source >= count:
                source = tile - 128 if tile >= 128 and tile - 128 < count else 0
            value = sheet.getpixel((source % (sheet.width // 8) * 8 + sx % 8,
                                    source // (sheet.width // 8) * 8 + sy % 8))[0]
            if value < 213:
                result.add((x, y))
    return result


def water_variants(pack, modules, crop, box, output):
    """Four genuine exterior tile14 phases; one phase across the entire view."""
    files = pack["runtime_files"]
    names = {m["attributes"]["tileset_name"] for m in modules.values()}
    assert len(names) == 1
    name = next(iter(names))
    assert name in {"mn_headwaters", "kanto", "johto"}
    definition = pack["data"]["tilesets"][name]
    own = f"gfx/tilesets/{name}.pal"
    text = bytes(files.get(own, files["gfx/tilesets/bg_tiles.pal"])).decode()
    colors, current = [], "default"
    for raw in text.splitlines():
        line = raw.strip()
        if line.startswith(";") and "RGB" not in line.upper():
            current = line[1:].strip().lower().replace(" ", "_")
        line = line.split(";")[0].strip()
        if line.upper().startswith("RGB") and (own in files or current == "day"):
            values = list(map(int, re.findall(r"\d+", line)))
            colors.extend(tuple((v << 3) | (v >> 2) if v <= 31 else v for v in values[i:i+3])
                          for i in range(0, len(values), 3))
    palette_id = definition["palette_map"][0x14] & 7
    palette = colors[palette_id * 4:palette_id * 4 + 4]
    assert len(palette) == 4
    raw = bytes(files["gfx/tilesets/water/water.2bpp"])
    assert len(raw) == 64
    layout = bytes(files[f"data/tilesets/{name}_metatiles.bin"])
    positions = []
    for map_name, module in modules.items():
        w = module["attributes"]["width"]
        ox, oy = int(map_name[-5:-3]) * 1024, int(map_name[-2:]) * 768
        for i, block in enumerate(module["blocks"]):
            for j, tile in enumerate(layout[block * 16:block * 16 + 16]):
                x, y = ox + i % w * 32 + j % 4 * 8, oy + i // w * 32 + j // 4 * 8
                if tile == 0x14 and x < box[2] and x + 8 > box[0] and y < box[3] and y + 8 > box[1]:
                    positions.append((x - box[0], y - box[1]))
    if not positions:
        return []
    result = []
    for frame in range(4):
        tile = Image.new("RGBA", (8, 8))
        for y in range(8):
            lo, hi = raw[frame * 16 + y * 2:frame * 16 + y * 2 + 2]
            for x in range(8):
                bit = 1 << (7 - x)
                tile.putpixel((x, y), palette[(2 if hi & bit else 0) | (1 if lo & bit else 0)] + (255,))
        save_render(tile, output / f"native-water14-frame{frame}-original8.png")
        variant = crop.copy()
        for at in positions:
            variant.paste(tile, at)
        result.append((frame, variant))
    return result


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--pack", type=Path, required=True)
    parser.add_argument("--pack-sha", required=True)
    parser.add_argument("--binary", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--route", choices=VIEWS, action="append", required=True)
    parser.add_argument("--resume", action="store_true", help="recheck retained native originals without recapturing")
    args = parser.parse_args()
    pack_path, source_binary, out = args.pack.resolve(), args.binary.resolve(), args.output.resolve()
    assert digest(pack_path) == args.pack_sha
    out.mkdir(parents=True, exist_ok=args.resume)
    retained = json.loads((out / "capture-progress.json").read_text()) if args.resume else None
    if retained:
        assert retained["pack_sha256"] == args.pack_sha and retained["binary_sha256"] == digest(source_binary)
    pack = load_pack(pack_path)
    art = SourceArt(pack)
    player = oriented_sprite(pack, "SPRITE_CHRIS", "south")
    save_render(player, out / "standing-chris-day.png")
    private = ROOT / ".local" / ("passage-native-" + str(os.getpid()))
    shutil.copy2(source_binary, private)
    binary_sha = digest(private)
    assert binary_sha == digest(source_binary)
    report = {"pack": str(pack_path), "pack_sha256": args.pack_sha,
              "content_identity": pack["identity"]["content_hash"],
              "binary": str(source_binary), "binary_sha256": binary_sha,
              "fixture": "Explicit local empty-party placement, DOWN, hour12; no earned progression claim",
              "captures": [], "personally_visually_reviewed": False}
    env = dict(os.environ)
    env.update(CRYSTAL_RENDER_TEST_HEADLESS="1", CRYSTAL_RENDER_TEST_CAPTURE_TARGET="image",
               CRYSTAL_RENDER_TEST_WINDOW_SIZE="640x576", CRYSTAL_PREVIEW_MUTE="1")
    env.pop("CRYSTAL_RENDER_TEST_SECOND_VIEW", None)
    try:
        for route in args.route:
            folder = out / route
            folder.mkdir(exist_ok=args.resume)
            prefix = route.replace("MN_", "Mn")
            modules = {name: value for name, value in pack["data"]["maps"].items()
                       if name.startswith(prefix + "C") and len(name) == len(prefix) + 6}
            assert modules
            width = max(int(name[-5:-3]) * 1024 + m["attributes"]["width"] * 32
                        for name, m in modules.items())
            height = max(int(name[-2:]) * 768 + m["attributes"]["height"] * 32
                         for name, m in modules.items())
            ground = Image.new("RGBA", (width, height), (0, 0, 0, 255))
            terrains = {}
            for name, module in modules.items():
                attr = module["attributes"]
                terrain = art.render(attr["tileset_name"], attr["width"], attr["height"],
                                     module["blocks"], group="day")
                terrains[name] = terrain
                save_render(terrain, folder / (name + "-terrain.png"))
                ground.alpha_composite(terrain.convert("RGBA"),
                                       (int(name[-5:-3]) * 1024, int(name[-2:]) * 768))
            whole = ground.copy()
            for name, module in modules.items():
                whole.alpha_composite(actors(pack, terrains[name], module),
                                      (int(name[-5:-3]) * 1024, int(name[-2:]) * 768))
            save_render(whole, folder / "whole-native-source.png")
            sheet = []
            for label, (x, y) in VIEWS[route]:
                name = f"{prefix}C{x // 64:02}R{y // 48:02}"
                module = modules[name]
                local = (x % 64, y % 48)
                assert not any((o["x"], o["y"]) == local for o in module["objects"])
                attr = module["attributes"]
                block = module["blocks"][local[1] // 2 * attr["width"] + local[0] // 2]
                kind = pack["data"]["tilesets"][attr["tileset_name"]]["collision"][f"{block:02x}"][
                    local[1] % 2 * 2 + local[0] % 2]
                assert kind in {"FLOOR", "TALL_GRASS"}, (route, label, kind)
                png = folder / (label + "-production.png")
                cmd = [str(private), "--pack", str(pack_path), "--map", name,
                       "--x", str(local[0]), "--y", str(local[1]), "--view", "2d",
                       "--hour", "12", "--screenshot", str(png)]
                if retained:
                    old = next(v for v in retained["captures"] if v["route"] == route and v["id"] == label)
                    assert digest(png) == old["production_sha256"]
                else:
                    with (folder / (label + "-capture.log")).open("w") as log:
                        process = subprocess.run(cmd, env=env, stdout=log, stderr=subprocess.STDOUT,
                                                 timeout=120)
                    assert process.returncode == 0, (route, label, process.returncode)
                retain_render_file(png)
                actual = render_stage(Image.open(png), "convert", folder, label + "-actual", "RGB")
                assert actual.size == (640, 576)
                # Only the active map owns presented objects. Neighbor terrain
                # remains connected, but neighboring actors are not fabricated.
                scene = ground.copy()
                scene.alpha_composite(actors(pack, terrains[name], module),
                                      (x // 64 * 1024, y // 48 * 768))
                scene.alpha_composite(player, (x * 16, y * 16))
                foreground = grass_priority(pack, module, local)
                for u, v in foreground:
                    scene.putpixel((x * 16 + u, y * 16 + v), ground.getpixel((x * 16 + u, y * 16 + v)))
                save_render(scene, folder / (label + "-source-player.png"))
                box = (x * 16 - 256, y * 16 - 192, x * 16 + 256, y * 16 + 192)
                crop = render_stage(scene, "crop", folder, label + "-source512",
                                    box)
                scaled = render_stage(crop, "resize", folder, label + "-source3x",
                                      (1536, 1152), Image.Resampling.NEAREST)
                reference = render_stage(scaled, "crop", folder, label + "-source640",
                                         (448, 288, 1088, 864)).convert("RGB")
                save_render(reference, folder / (label + "-source-reference.png"))
                difference = ImageChops.difference(actual, reference)
                save_render(difference, folder / (label + "-difference.png"))
                count = sum(v != (0, 0, 0) for v in difference.getdata())
                phase = None
                if count:
                    for frame, candidate in water_variants(pack, modules, crop, box, folder / (label + "-water-phases")):
                        save_render(candidate, folder / f"{label}-water{frame}-source512.png")
                        scaled = render_stage(candidate, "resize", folder, f"{label}-water{frame}-source3x",
                                              (1536, 1152), Image.Resampling.NEAREST)
                        possible = render_stage(scaled, "crop", folder, f"{label}-water{frame}-source640",
                                                (448, 288, 1088, 864)).convert("RGB")
                        delta = ImageChops.difference(actual, possible)
                        errors = sum(v != (0, 0, 0) for v in delta.getdata())
                        save_render(delta, folder / f"{label}-water{frame}-difference.png")
                        if errors == 0:
                            count, phase = 0, frame
                            save_render(possible, folder / (label + "-matched-native-phase-reference.png"))
                            break
                raw = list((folder / ".render-archive" / png.name).glob("gpu-readback-*.json"))
                assert raw, "retain raw production readback"
                row = {"route": route, "id": label, "global_core": [x, y],
                       "map": name, "local_core": list(local), "collision": kind,
                       "command": cmd, "production_sha256": digest(png),
                       "compared_pixels": 640 * 576, "pixel_mismatches": count,
                       "native_grass_priority_nonzero_pixels": len(foreground),
                       "matched_consistent_native_water14_frame": phase,
                       "raw_gpu_descriptors": [str(p.relative_to(out)) for p in raw]}
                report["captures"].append(row)
                write(out / "capture-progress.json", report)
                sheet.append((label, actual))
                print(route, label, "mismatches", count, flush=True)
            save_render(room_sheet(sheet, columns=3), folder / "production-decisions.png")
        report["status"] = "PASS" if all(v["pixel_mismatches"] == 0 for v in report["captures"]) else "FAIL"
    finally:
        private.unlink()
        report["private_binary_removed"] = not private.exists()
        report["pack_unchanged"] = digest(pack_path) == args.pack_sha
        write(out / "capture-progress.json", report)
        (out / "review_passage_native.py.snapshot").write_bytes(Path(__file__).read_bytes())
    assert report["status"] == "PASS", "inspect retained differences before acceptance"


if __name__ == "__main__":
    main()
