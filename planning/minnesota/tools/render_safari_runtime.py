#!/usr/bin/env python3
"""Read-only whole-preserve preview from actual current pack maps and source art.

This renders static native actor poses, not a simulated game session. Pack
publication remains Rust. Every invocation requires a fresh output directory.
"""

import argparse
from collections import deque
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


def inspect_boundaries(pack, modules):
    """Audit current native collision; the two free walking exits stay open."""
    data = pack["data"]
    cells, occupied, guards = {}, set(), set()
    for (column, row), (_, module) in modules.items():
        attr = module["attributes"]
        collision = data["tilesets"][attr["tileset_name"]]["collision"]
        width, height = attr["width"], attr["height"]
        for y in range(height * 2):
            for x in range(width * 2):
                block = module["blocks"][y // 2 * width + x // 2]
                cells[column * 64 + x, row * 48 + y] = collision[f"{block:02x}"][y % 2 * 2 + x % 2]
        for actor in module["objects"]:
            tile = column * 64 + actor["x"], row * 48 + actor["y"]
            occupied.add(tile)
            if actor["object_identifier"] in {"BWS_PENDING_GATE_WEST", "BWS_PENDING_GATE_EAST"}:
                guards.add(tile)
    if guards != {(11, 101), (12, 101)}:
        raise ValueError("Current native permit guard bodies changed")
    perimeter = {p for p in cells if p[0] in {0, 127} or p[1] in {0, 111}}
    exits = {(14, 111), (15, 111)}
    court = modules[0, 2][1]
    approach = data["maps"]["MnPreservePineEntry"]
    if not any(edge == {"direction": "south", "target_map": "MnPreservePineEntry", "offset": -1}
               for edge in court["attributes"]["connections"]):
        raise ValueError("Free Pine Entry walking connection missing")
    approach_attr = approach["attributes"]
    approach_collision = data["tilesets"][approach_attr["tileset_name"]]["collision"]
    for x, y in exits:
        # A -1-metatile south connection adds two walking tiles at its target.
        target_x = x + 2
        block = approach["blocks"][target_x // 2]
        if cells[x, y] != "FLOOR" or approach_collision[f"{block:02x}"][target_x % 2] != "FLOOR":
            raise ValueError("Free walking seam lacks paired passive FLOOR")
    bad = sorted(p for p in perimeter - exits if cells[p] not in {"WALL", "BUOY"})
    if bad:
        raise ValueError(f"Unconnected walking/Surf perimeter opening: {bad}")

    def flood(blocked):
        seen, queue = {(15, 105)}, deque([(15, 105)])
        while queue:
            x, y = queue.popleft()
            for tile in [(x - 1, y), (x + 1, y), (x, y - 1), (x, y + 1)]:
                if (tile not in seen and tile not in blocked
                        and cells.get(tile) in {"FLOOR", "TALL_GRASS", "WATER"}):
                    seen.add(tile)
                    queue.append(tile)
        return seen

    closed, admitted = flood(occupied), flood(occupied - guards)
    if any(cells[p] == "WATER" for p in closed) or (3, 90) in closed:
        raise ValueError("Unpaid walking/Surf bypass around permit guards")
    if not exits <= closed or not {(3, 89), (71, 56)} <= admitted:
        raise ValueError("Free approach, admitted lake or cabin approach disconnected")
    return {
        "perimeter_tiles": len(perimeter), "closed_wall_or_buoy_tiles": len(perimeter - exits),
        "paired_free_walking_exits": sorted(exits), "unconnected_water_exits": 0,
        "closed_court_reachable_tiles": len(closed), "unpaid_lake_bypass": False,
        "admitted_lake_and_cabin_approach_reachable": True,
        "scope": "Static FLOOR/TALL_GRASS/WATER graph with actual actor bodies; native play is separate",
    }


def render(pack_path, output, include_approach=False):
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
    boundaries = inspect_boundaries(pack, modules)
    art = SourceArt(pack)
    placed = {}
    for (column, row), (name, module) in modules.items():
        attributes = module["attributes"]
        if (attributes["width"], attributes["height"]) != (32, 8 if row == 2 else 24):
            raise ValueError(f"Unexpected native chunk dimensions: {name}")
        placed[name] = (module, (column * 64, row * 48))
    if include_approach:
        # Place the complete public trail and basecamp from their real walking
        # seams. Negative origins are preserved; no invented connector terrain.
        previous = "MnBoundarySafariC00R02"
        for name in ("MnPreservePineEntry", "MnBoundaryWaters"):
            source, (x, y) = placed[previous]
            edges = [edge for edge in source["attributes"]["connections"]
                     if edge["direction"] == "south" and edge["target_map"] == name]
            if len(edges) != 1:
                raise ValueError(f"Expected one actual south walking seam: {previous}->{name}")
            placed[name] = (pack["data"]["maps"][name],
                            (x + edges[0]["offset"] * 2, y + source["attributes"]["height"] * 2))
            previous = name
    left = min(origin[0] for _, origin in placed.values())
    top = min(origin[1] for _, origin in placed.values())
    right = max(origin[0] + module["attributes"]["width"] * 2 for module, origin in placed.values())
    bottom = max(origin[1] + module["attributes"]["height"] * 2 for module, origin in placed.values())
    terrain = Image.new("RGB", ((right - left) * 16, (bottom - top) * 16), (40, 44, 48))
    actors, connections = [], []
    zone = pack["data"].get("safari_zone")
    for name, (module, (x, y)) in placed.items():
        attributes = module["attributes"]
        width, height = attributes["width"], attributes["height"]
        if len(module["blocks"]) != width * height:
            raise ValueError(f"Incomplete native block array: {name}")
        if zone and name in zone["maps"] and zone["maps"][name]["origin_core"] != [x, y]:
            raise ValueError(f"Habitat placement differs from native chunk grid: {name}")
        terrain.paste(
            art.render(attributes["tileset_name"], width, height, module["blocks"]),
            ((x - left) * 16, (y - top) * 16),
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
            actors.append((actor, x - left + actor["x"], y - top + actor["y"], facing))
    scene = terrain.convert("RGBA")
    for actor, x, y, facing in actors:
        scene.alpha_composite(
            oriented_sprite(pack, actor["sprite"], FACING[facing]), (x * 16, y * 16 - 4)
        )
    output.mkdir(parents=True, exist_ok=False)
    terrain.save(output / "terrain-original.png")
    scene.save(output / "whole-original.png")
    scene.resize((scene.width // 2, scene.height // 2), Image.Resampling.NEAREST).save(output / "whole-preview.png")
    report = {
        "pack": str(pack_path), "pack_sha256": pack_hash, "selected": is_selected,
        "selection_observed_at_utc": selection_observed_at,
        "selected_revision_observed": world["active_local_review_revision"],
        "expedition_configured": zone is not None,
        "maps": list(placed),
        "origins_core": {name: origin for name, (_, origin) in placed.items()},
        "canvas_origin_core": [left, top], "includes_public_approach": include_approach,
        "dimensions_pixels": scene.size, "connections": connections,
        "static_actor_count": len(actors), "gameplay_verified_by_this_preview": False,
        "boundary_audit": boundaries,
        "render_ms": round((time.perf_counter() - started) * 1000, 1),
    }
    (output / "render.json").write_text(json.dumps(report, indent=2) + "\n")
    return report


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--pack", type=Path, help="Defaults to the live world selector")
    parser.add_argument("--output", type=Path)
    parser.add_argument("--include-approach", action="store_true",
                        help="Include complete Pine Entry and basecamp at actual connection offsets")
    args = parser.parse_args()
    output = args.output or ROOT / f"planning/minnesota/renders/safari-runtime-preview/run-{time.time_ns()}"
    print(json.dumps({"output": str(output), **render(args.pack, output, args.include_approach)}, indent=2))


if __name__ == "__main__":
    main()
