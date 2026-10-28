#!/usr/bin/env python3
"""Capture the expanded fair through production Bevy and compare native pixels."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess

from PIL import Image, ImageChops, ImageDraw

from render_archive import retain_render_file, save_render
from render_reference import SourceArt, load_pack, room_sheet
from review_city_native import priority_scene
from review_connected_route_native import sources
from review_north_shore import oriented_sprite
from review_passage_native import actors

ROOT = Path(__file__).resolve().parents[3]


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def player_origin(actual, player):
    """Require one exact native CHRIS frame; no terrain/image registration."""
    matches = []
    for scale in (4, 3):
        frame = player.resize((16 * scale, 16 * scale), Image.Resampling.NEAREST)
        opaque = [(x, y, frame.getpixel((x, y))[:3])
                  for y in range(frame.height) for x in range(frame.width)
                  if frame.getpixel((x, y))[3]]
        seed = next(v for v in opaque if v[2][0] > v[2][2] + 30)
        for y in range(actual.height - frame.height + 1):
            for x in range(actual.width - frame.width + 1):
                if actual.getpixel((x + seed[0], y + seed[1])) == seed[2] and all(
                    actual.getpixel((x + u, y + v)) == rgb for u, v, rgb in opaque
                ):
                    matches.append((scale, x, y, len(opaque)))
    assert len(matches) == 1, ("one exact standing CHRIS frame required", matches)
    return matches[0]


def compare(pack, art, view, actual, folder):
    name = view.get("map", "MnStateFairgrounds")
    x, y = view["core"]
    regions = sources(pack, name)
    module = regions[0][0]
    player = oriented_sprite(pack, "SPRITE_CHRIS", "south")
    scale, px, py, pixels = player_origin(actual, player)
    box = (x * 16 - 256, y * 16 - 192, x * 16 + 256, y * 16 + 192)
    left, top = box[0] // 32 * 32, box[1] // 32 * 32
    right, bottom = (box[2] + 31) // 32 * 32, (box[3] + 31) // 32 * 32
    scene = Image.new("RGBA", (right - left, bottom - top))
    attr = module["attributes"]
    border = art.blocks(attr["tileset_name"])[attr["border_block"]]
    for by in range(0, scene.height, 32):
        for bx in range(0, scene.width, 32):
            scene.paste(border, (bx, by))
    masked = 0
    for source, ox, oy in [*regions[1:], regions[0]]:
        a = source["attributes"]
        ground = art.render(a["tileset_name"], a["width"], a["height"], source["blocks"])
        layer = ground.convert("RGBA")
        if source is module:
            starting_poses = dict(source, objects=[
                dict(actor, spritemovedata="SPRITEMOVEDATA_STANDING_RIGHT")
                if actor["spritemovedata"] in {"SPRITEMOVEDATA_PATROL_CLOCKWISE",
                                               "SPRITEMOVEDATA_PATROL_CLOCKWISE_SLOW",
                                               "SPRITEMOVEDATA_SPINCLOCKWISE"} else actor
                for actor in source["objects"]])
            layer = actors(pack, ground, starting_poses)
            layer.alpha_composite(player, (x * 16, y * 16))
            masked = priority_scene(pack, module, layer, ground, box)
        scene.alpha_composite(layer, (ox - left, oy - top))
    save_render(scene, folder / "connected-native-source.png")
    crop = scene.crop((box[0] - left, box[1] - top, box[2] - left, box[3] - top))
    save_render(crop, folder / "native-source512.png")
    scaled = crop.resize((512 * scale, 384 * scale), Image.Resampling.NEAREST)
    lx, ty = 256 * scale - px, 192 * scale - py
    expected = scaled.crop((lx, ty, lx + 640, ty + 576)).convert("RGB")
    difference = ImageChops.difference(actual, expected)
    save_render(expected, folder / "source-reference640.png")
    save_render(difference, folder / "difference.png")
    # Keep the untouched frame and full difference. A moving actor cannot be
    # compared with its source-event starting pose; exclude only its authored
    # loop rectangle from the static-pixel assertion and report those bounds.
    motion = Image.new("1", actual.size)
    painter = ImageDraw.Draw(motion)
    regions = []
    for actor in module["objects"]:
        if actor["spritemovedata"] not in {"SPRITEMOVEDATA_PATROL_CLOCKWISE",
                                         "SPRITEMOVEDATA_PATROL_CLOCKWISE_SLOW",
                                         "SPRITEMOVEDATA_SPINCLOCKWISE"}:
            continue
        ax, ay = actor["x"], actor["y"]
        bounds = [int(((ax * 16 - 8) - box[0]) * scale - lx),
                  int(((ay * 16 - 8) - box[1]) * scale - ty),
                  int((((ax + actor["move_range_x"] + 1) * 16 + 8) - box[0]) * scale - lx),
                  int((((ay + actor["move_range_y"] + 1) * 16 + 8) - box[1]) * scale - ty)]
        painter.rectangle(bounds, fill=1)
        regions.append(dict(id=actor["object_identifier"], bounds=bounds))
    static = difference.copy()
    static.paste((0, 0, 0), mask=motion)
    save_render(static, folder / "static-difference.png")
    excluded = sum(bool(p) for p in motion.getdata())
    return dict(native_scale=scale, exact_player_origin=[px, py],
                exact_opaque_player_pixels=pixels, compared_pixels=640 * 576 - excluded,
                pixel_mismatches=sum(p != (0, 0, 0) for p in static.getdata()),
                full_frame_pixel_mismatches=sum(p != (0, 0, 0) for p in difference.getdata()),
                motion_region_pixels_excluded=excluded, motion_regions=regions,
                source_foreground_pixels_over_actors=masked)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--pack", type=Path, required=True)
    parser.add_argument("--binary", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--source", type=Path, default=ROOT / "planning/minnesota/zones/state-fairgrounds.json")
    args = parser.parse_args()
    pack_path = args.pack.resolve()
    out = args.output.resolve()
    out.mkdir(parents=True, exist_ok=False)
    pack = load_pack(pack_path)
    art = SourceArt(pack)
    views = json.loads(args.source.read_text())["expansion_contract"]["review_views"]
    private = ROOT / ".local" / f"fair-native-capture-{os.getpid()}"
    shutil.copy2(args.binary, private)
    report = dict(pack=str(pack_path), pack_sha256=digest(pack_path), binary_sha256=digest(private),
                  source_sha256=digest(args.source), fixture="Explicit local position, empty party, south-facing, noon; no earned access claim.", captures=[])
    env = dict(os.environ, CRYSTAL_RENDER_TEST_HEADLESS="1", CRYSTAL_RENDER_TEST_CAPTURE_TARGET="image",
               CRYSTAL_RENDER_TEST_WINDOW_SIZE="640x576", CRYSTAL_PREVIEW_MUTE="1")
    env.pop("CRYSTAL_RENDER_TEST_SECOND_VIEW", None)
    sheet = []
    try:
        for view in views:
            folder = out / view["id"]
            folder.mkdir()
            x, y = view["core"]
            png = folder / "production.png"
            command = [str(private), "--pack", str(pack_path), "--map", "MnStateFairgrounds", "--x", str(x), "--y", str(y), "--view", "2d", "--hour", "12", "--screenshot", str(png)]
            with (folder / "capture.log").open("wb") as log:
                result = subprocess.run(command, env=env, stdout=log, stderr=subprocess.STDOUT, timeout=120)
            assert result.returncode == 0, (view, result.returncode)
            retain_render_file(png)
            actual = Image.open(png).convert("RGB")
            assert actual.size == (640, 576)
            row = dict(view, command=command, production_sha256=digest(png), **compare(pack, art, view, actual, folder))
            raw = list((folder / ".render-archive" / png.name).glob("gpu-readback-*.json"))
            assert raw, "retain raw GPU descriptors"
            row["raw_gpu_descriptors"] = [str(p.relative_to(out)) for p in raw]
            report["captures"].append(row)
            (out / "capture-progress.json").write_text(json.dumps(report, indent=2) + "\n")
            sheet.append((view["id"], actual))
            print(view["id"], "mismatches", row["pixel_mismatches"], flush=True)
        save_render(room_sheet(sheet, columns=3), out / "production-decisions.png")
        report["status"] = "PASS" if all(v["pixel_mismatches"] == 0 for v in report["captures"]) else "REVIEW_DIFFERENCES"
    finally:
        private.unlink()
        report["pack_unchanged"] = digest(pack_path) == report["pack_sha256"]
        (out / "capture-progress.json").write_text(json.dumps(report, indent=2) + "\n")
        (out / "review_state_fair_native.py.snapshot").write_bytes(Path(__file__).read_bytes())
    assert report["status"] == "PASS", "inspect retained source/native differences"


if __name__ == "__main__":
    main()
