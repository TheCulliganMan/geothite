#!/usr/bin/env python3
"""Read-only whole-preserve preview from actual current pack maps and source art.

This renders static native actor poses, not a simulated game session. Pack
publication remains Rust. Every invocation requires a fresh output directory.
"""

import argparse
from datetime import datetime, timezone
import hashlib
import json
from pathlib import Path
import re
import time

from PIL import Image

from render_reference import SourceArt, load_pack
from review_north_shore import oriented_sprite


ROOT = Path(__file__).resolve().parents[3]
OPPOSITE = {"north": "south", "south": "north", "east": "west", "west": "east"}
FACING = {"UP": "north", "DOWN": "south", "LEFT": "west", "RIGHT": "east"}


def render(pack_path, output):
    started = time.perf_counter()
    world = json.loads((ROOT / "planning/minnesota/world.json").read_text())
    selection_observed_at = datetime.now(timezone.utc).isoformat()
    selected = world[world["active_local_review_revision"]]
    selected_path = ROOT / selected["pack"]
    pack_path = (pack_path or selected_path).resolve()
    pack_hash = hashlib.sha256(pack_path.read_bytes()).hexdigest()
    is_selected = pack_path == selected_path.resolve()
    if is_selected and pack_hash != selected["pack_sha256"]:
        raise ValueError("Selected pack bytes differ from the world selector")
    pack = load_pack(pack_path)
    modules = {}
    for name, module in pack["data"]["maps"].items():
        match = re.fullmatch(r"MnBoundarySafariC(\d{2})R(\d{2})", name)
        if match:
            modules[tuple(map(int, match.groups()))] = (name, module)
    if set(modules) != {(column, row) for column in range(2) for row in range(3)}:
        raise ValueError("Expected the current six clipped preserve chunks")
    art = SourceArt(pack)
    terrain = Image.new("RGB", (2048, 1792))
    actors, connections = [], []
    zone = pack["data"].get("safari_zone")
    for (column, row), (name, module) in sorted(modules.items()):
        attributes = module["attributes"]
        width, height = attributes["width"], attributes["height"]
        if (width, height) != (32, 8 if row == 2 else 24):
            raise ValueError(f"Unexpected native chunk dimensions: {name}")
        if len(module["blocks"]) != width * height:
            raise ValueError(f"Incomplete native block array: {name}")
        if zone and zone["maps"][name]["origin_core"] != [column * 64, row * 48]:
            raise ValueError(f"Habitat placement differs from native chunk grid: {name}")
        terrain.paste(
            art.render(attributes["tileset_name"], width, height, module["blocks"]),
            (column * 1024, row * 768),
        )
        for edge in attributes["connections"]:
            back = pack["data"]["maps"][edge["target_map"]]["attributes"]["connections"]
            if not any(
                item["direction"] == OPPOSITE[edge["direction"]]
                and item["target_map"] == name
                and item["offset"] == -edge["offset"]
                for item in back
            ):
                raise ValueError(f"Nonreciprocal native connection: {name}: {edge}")
            connections.append({"map": name, **edge})
        for actor in module["objects"]:
            facing = actor["spritemovedata"].rsplit("_", 1)[-1]
            if actor["pal"] != 0 or facing not in FACING:
                raise ValueError(f"Actor needs a deliberate palette/pose review: {name}")
            actors.append((actor, column * 64 + actor["x"], row * 48 + actor["y"], facing))
    scene = terrain.convert("RGBA")
    for actor, x, y, facing in actors:
        scene.alpha_composite(
            oriented_sprite(pack, actor["sprite"], FACING[facing]), (x * 16, y * 16 - 4)
        )
    output.mkdir(parents=True, exist_ok=False)
    terrain.save(output / "terrain-original.png")
    scene.save(output / "whole-original.png")
    scene.resize((1024, 896), Image.Resampling.NEAREST).save(output / "whole-preview.png")
    report = {
        "pack": str(pack_path), "pack_sha256": pack_hash, "selected": is_selected,
        "selection_observed_at_utc": selection_observed_at,
        "selected_revision_observed": world["active_local_review_revision"],
        "expedition_configured": zone is not None,
        "maps": [name for name, _ in modules.values()],
        "dimensions_pixels": scene.size, "connections": connections,
        "static_actor_count": len(actors), "gameplay_verified_by_this_preview": False,
        "render_ms": round((time.perf_counter() - started) * 1000, 1),
    }
    (output / "render.json").write_text(json.dumps(report, indent=2) + "\n")
    return report


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--pack", type=Path, help="Defaults to the live world selector")
    parser.add_argument("--output", type=Path)
    args = parser.parse_args()
    output = args.output or ROOT / f"planning/minnesota/renders/safari-runtime-preview/run-{time.time_ns()}"
    print(json.dumps({"output": str(output), **render(args.pack, output)}, indent=2))


if __name__ == "__main__":
    main()
