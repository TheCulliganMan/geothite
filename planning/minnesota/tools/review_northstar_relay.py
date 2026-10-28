#!/usr/bin/env python3
"""Compare the relay's private native pixels and preserve both power states."""
import argparse
from pathlib import Path

from PIL import Image, ImageChops

from render_archive import save_render
from render_reference import SourceArt, load_pack, room_sheet
from review_boundary_basecamp_native import actor_scene

ROOT = Path(__file__).resolve().parents[3]
ROOM = "MnBoundaryWatersSurveyShelter"


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--parent", type=Path, default=ROOT / "content-packs/minnesota-northstar-relay-base.crystalpack")
    parser.add_argument("--pack", type=Path, default=ROOT / "content-packs/minnesota-northstar-relay-v2.crystalpack")
    parser.add_argument("--output", type=Path, default=ROOT / "planning/minnesota/renders/northstar-relay")
    parser.add_argument("--native-powered", type=Path)
    parser.add_argument("--native-off", type=Path)
    args = parser.parse_args()
    parent, pack = load_pack(args.parent), load_pack(args.pack)
    old_room = parent["data"]["maps"][ROOM]
    old = SourceArt(parent).render(old_room["attributes"]["tileset_name"], 8, 6, old_room["blocks"], "indoor")
    art = SourceArt(pack)
    room = pack["data"]["maps"][ROOM]
    terrain = art.render(room["attributes"]["tileset_name"], 8, 6, room["blocks"], "indoor")
    assert ImageChops.difference(old, terrain).getbbox() is None, "powered scene must preserve all native source pixels"
    change = next(c for c in room["script_block_changes"] if c["source_script"] == "MN_BOUNDARY_SURVEY_RECEIVER")
    assert (change["x"], change["y"]) == (12, 2)
    blocks = list(room["blocks"])
    blocks[14] = change["block_id"]
    off = art.render(room["attributes"]["tileset_name"], 8, 6, blocks, "indoor")
    bounds = ImageChops.difference(terrain, off).getbbox()
    assert bounds is not None and bounds[0] >= 196 and bounds[1] >= 52 and bounds[2] <= 204 and bounds[3] <= 57, bounds
    args.output.mkdir(parents=True, exist_ok=True)
    scenes = []
    for label, image in [("powered", terrain), ("off", off)]:
        save_render(image, args.output / f"{label}-terrain-original.png")
        scene = actor_scene(pack, image, room["objects"])
        save_render(scene, args.output / f"{label}-shelter-original.png")
        save_render(scene.resize((768, 576), Image.Resampling.NEAREST), args.output / f"{label}-shelter-3x.png")
        scenes.append((label, scene.convert("RGB")))
    save_render(room_sheet(scenes, columns=2), args.output / "power-states-sheet.png")
    assert bool(args.native_powered) == bool(args.native_off), "supply both matched native states"
    if args.native_powered:
        native = [Image.open(path).convert("RGB") for path in [args.native_powered, args.native_off]]
        assert all(image.size == (640, 576) for image in native), "matched (12,4), facing north, noon, 640x576 native cameras required"
        region = (383, 231, 401, 240)
        assert ImageChops.difference(*native).getbbox() == region, "only the native console screen may change"
        for expected, actual in zip([terrain, off], native):
            patch = expected.crop((197, 53, 203, 56)).resize((18, 9), Image.Resampling.NEAREST)
            assert ImageChops.difference(patch, actual.crop(region)).getbbox() is None, "native screen pixels must match the exact indoor source palette"
        print("PASS: matched native cameras differ only in 162 screen pixels; both states match the exact source pixels and indoor palette.")
    print(f"PASS: all powered source pixels match; screen-only power-down bounds {bounds}; whole shelter states archived.")


if __name__ == "__main__":
    main()
