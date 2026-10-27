#!/usr/bin/env python3
"""Capture the installed R12 seep shelter, retaining native pixels and phases."""
import argparse
import importlib.util
import json
import os
from pathlib import Path
import shutil
import subprocess

from PIL import Image, ImageChops

from render_archive import retain_render_file, save_render
from render_reference import SourceArt, load_pack, npc_palette_pack, room_sheet
from render_stages import render_stage
from review_north_shore import oriented_sprite
from review_passage_native import actors, digest, write

ROOT = Path(__file__).resolve().parents[3]
ROOM = "MnR12Cave1F"
VIEWS = [("mouth-basin", (17, 20)), ("seep-bank", (21, 17)),
         ("rising-throat", (19, 16)), ("upper-fork", (18, 13)),
         ("otta-roof", (6, 5)), ("cache-approach", (26, 5)),
         ("ledge-approach", (14, 13)), ("lower-return", (18, 19)),
         ("weakest-roof", (8, 5))]


def camera(scene, at, out, label, player_origin):
    x, y = at
    source = render_stage(scene, "crop", out, label + "-source512",
                          (x * 16 - 256, y * 16 - 192, x * 16 + 256, y * 16 + 192))
    scaled = render_stage(source, "resize", out, label + "-source3x",
                          (1536, 1152), Image.Resampling.NEAREST)
    left, top = 768 - player_origin[0], 576 - player_origin[1]
    return render_stage(scaled, "crop", out, label + "-reference640",
                        (left, top, left + 640, top + 576)).convert("RGB")


def measured_player(image, player):
    """Measure unique exact OAM; finite-map LCD clamping can move CHRIS."""
    scaled = player.resize((48, 48), Image.Resampling.NEAREST)
    opaque = [(x, y, scaled.getpixel((x, y))[:3]) for y in range(48) for x in range(48)
              if scaled.getpixel((x, y))[3]]
    seed = next(v for v in opaque if v[2][0] > v[2][2] + 30)
    found = []
    for y in range(image.height - 47):
        for x in range(image.width - 47):
            if image.getpixel((x + seed[0], y + seed[1])) == seed[2] and all(
                image.getpixel((x + u, y + v)) == rgb for u, v, rgb in opaque
            ):
                found.append((x, y))
    assert len(found) == 1, ("standing CHRIS must match uniquely", found)
    return found[0], len(opaque)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--pack", type=Path, required=True)
    parser.add_argument("--pack-sha", required=True)
    parser.add_argument("--source", type=Path, required=True)
    parser.add_argument("--binary", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--resume", action="store_true")
    args = parser.parse_args()
    pack_path, out = args.pack.resolve(), args.output.resolve()
    assert digest(pack_path) == args.pack_sha
    out.mkdir(parents=True, exist_ok=args.resume)
    retained = json.loads((out / "capture-progress.json").read_text()) if args.resume else None
    if retained:
        assert retained["pack_sha256"] == args.pack_sha and retained["binary_sha256"] == digest(args.binary)
    source = json.loads(args.source.read_text())
    pack = load_pack(pack_path)
    module = pack["data"]["maps"][ROOM]
    assert module["blocks"] == [int(v, 16) for row in source["rows"] for v in row.split()]
    assert module["attributes"]["time_of_day"] == "nite"
    terrain = SourceArt(pack).render("cave", 16, 12, module["blocks"], group="nite")
    sprite_pack = npc_palette_pack(pack, "nite")
    whole = actors(sprite_pack, terrain, module)
    save_render(terrain, out / "whole-native-terrain-nite.png")
    save_render(whole, out / "whole-native-source-nite.png")
    player = oriented_sprite(sprite_pack, "SPRITE_CHRIS", "south")
    save_render(player, out / "standing-chris-native-nite.png")
    helper = ROOT / "planning/minnesota/renders/caves/r10-refinement-native/cave_water_pixels.py"
    spec = importlib.util.spec_from_file_location("native_cave_water", helper)
    water = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(water)
    positions, family, water_sha = water.native_water_family(pack, module, terrain, whole, out)
    report = {"pack": str(pack_path), "pack_sha256": args.pack_sha,
              "source": str(args.source.resolve()), "source_sha256": digest(args.source),
              "water_source_sha256": water_sha, "native_water14_subtiles": len(positions),
              "fixture": "Local empty-party placement, DOWN, hour12; native forcedNITE",
              "captures": [], "personally_visually_reviewed": False}
    private = ROOT / ".local" / ("shelter-native-" + str(os.getpid()))
    shutil.copy2(args.binary, private)
    assert digest(private) == digest(args.binary)
    report["binary_sha256"] = digest(private)
    env = dict(os.environ)
    env.update(CRYSTAL_RENDER_TEST_HEADLESS="1", CRYSTAL_RENDER_TEST_CAPTURE_TARGET="image",
               CRYSTAL_RENDER_TEST_WINDOW_SIZE="640x576", CRYSTAL_PREVIEW_MUTE="1")
    env.pop("CRYSTAL_RENDER_TEST_SECOND_VIEW", None)
    images = []
    try:
        for label, at in VIEWS:
            x, y = at
            block = module["blocks"][y // 2 * 16 + x // 2]
            assert pack["data"]["tilesets"]["cave"]["collision"][f"{block:02x}"] == ["FLOOR"] * 4
            assert not any((o["x"], o["y"]) == at for o in module["objects"])
            png = out / (label + "-production.png")
            command = [str(private), "--pack", str(pack_path), "--map", ROOM,
                       "--x", str(x), "--y", str(y), "--view", "2d", "--hour", "12",
                       "--screenshot", str(png)]
            if retained:
                old = next(v for v in retained["captures"] if v["id"] == label)
                assert digest(png) == old["production_sha256"]
            else:
                with (out / (label + "-capture.log")).open("w") as log:
                    result = subprocess.run(command, env=env, stdout=log, stderr=subprocess.STDOUT,
                                            timeout=120)
                assert result.returncode == 0
            retain_render_file(png)
            actual = render_stage(Image.open(png), "convert", out, label + "-actual", "RGB")
            assert actual.size == (640, 576)
            player_origin, opaque_pixels = measured_player(actual, player)
            comparisons = []
            selected = None
            # Match one complete genuine phase across every water tile; never
            # sample individual pixels/phases to fabricate a passing image.
            for frame, scroll, tick, _, scene in family:
                expected = scene.copy()
                expected.alpha_composite(player, (x * 16, y * 16))
                reference = camera(expected, at, out / label, f"water{frame}-left{scroll}", player_origin)
                delta = ImageChops.difference(actual, reference)
                count = sum(v != (0, 0, 0) for v in delta.getdata())
                comparisons.append({"water_frame": frame, "left_scroll": scroll, "mismatches": count})
                if count == 0:
                    save_render(reference, out / (label + "-matched-reference.png"))
                    save_render(delta, out / (label + "-difference.png"))
                    selected = {"water_frame": frame, "left_scroll": scroll,
                                "genuine_schedule_example_tick": tick,
                                "actual_tick_unobserved": True, "one_consistent_phase": True}
                    break
            raw = list((out / ".render-archive" / png.name).glob("gpu-readback-*.json"))
            assert raw
            row = {"id": label, "core": list(at), "command": command,
                   "production_sha256": digest(png), "compared_pixels": 640 * 576,
                   "measured_player_origin": list(player_origin),
                   "exact_opaque_player_pixels": opaque_pixels,
                   "centered_camera_assumption_used": False,
                   "matched_native_phase": selected, "phase_comparisons": comparisons,
                   "raw_gpu_descriptors": [str(p.relative_to(out)) for p in raw]}
            report["captures"].append(row)
            write(out / "capture-progress.json", report)
            images.append((label, actual))
            print(label, "PASS" if selected else "FAIL", flush=True)
        save_render(room_sheet(images, columns=3), out / "production-decisions.png")
        report["status"] = "PASS" if all(v["matched_native_phase"] for v in report["captures"]) else "FAIL"
    finally:
        private.unlink()
        report["private_binary_removed"] = not private.exists()
        report["pack_unchanged"] = digest(pack_path) == args.pack_sha
        write(out / "capture-progress.json", report)
        (out / "review_shelter_depth_native.py.snapshot").write_bytes(Path(__file__).read_bytes())
    assert report["status"] == "PASS", "inspect retained original captures and phase diagnostics"


if __name__ == "__main__":
    main()
