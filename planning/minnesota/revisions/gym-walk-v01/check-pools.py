#!/usr/bin/env python3
"""Compare both editions' actual lake permissions, exits, art and archived campaign data."""
import argparse
from collections import deque
from pathlib import Path
import json
import sys

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE.parents[1] / "tools"))
from audit_regional_design import Terrain
from render_reference import SourceArt, load_pack

GYM = "MnBrainerdGym"
ROOMS = {
    "MnRochesterGym", "MnMankatoGym", "MnMinneapolisGym",
    "MnStCloudGym", GYM, "MnBemidjiGym",
}
BANKS = {"mn_gym_1", "mn_gym_2", "mn_gym_6"}


def neighbors(point):
    x, y = point
    return {(x-1,y), (x+1,y), (x,y-1), (x,y+1)}


def connected(start, allowed):
    visited, queue = {start}, deque([start])
    while queue:
        next_squares = (neighbors(queue.popleft()) & allowed) - visited
        queue.extend(next_squares)
        visited.update(next_squares)
    return visited


def changed(a, b):
    return {k for k in a.keys() | b.keys() if a.get(k) != b.get(k)}


def check_campaign(parent, child):
    a, b = parent["data"], child["data"]
    assert changed(parent, child) == {"data","identity","report","runtime_files"}
    assert changed(parent["identity"],child["identity"]) == {"runtime_modpack_id", "content_hash"}
    assert changed(parent["report"], child["report"]) == {"manifests"}
    assert child["report"]["manifests"] == parent["report"]["manifests"] + ["northstar-gym-walk-v1"]
    assert changed(a, b) == {"map_blocks", "map_scripts", "maps", "npcs", "tilesets"}
    assert changed(a["maps"],b["maps"]) == ROOMS
    assert changed(a["npcs"],b["npcs"]) == {"MnStCloudGym"}
    assert changed(a["tilesets"],b["tilesets"]) == BANKS
    # Only Intro lacked the published Lakeside art. The Field edition starts
    # from the same design and already has its block layout and private palette.
    has_beach = len(a["tilesets"]["mn_gym_6"]["collision"]) == 54
    layouts = {"MnRochesterGym_Blocks", "MnMankatoGym_Blocks"}
    assert changed(a["map_blocks"],b["map_blocks"]) == (layouts if has_beach else layouts | {GYM + "_Blocks"})
    assets = {f"data/tilesets/{bank}{suffix}" for bank in BANKS for suffix in [".json","_metatiles.bin"]}
    assets.add("data/tilesets/mn_gym_6_animations.json")
    if not has_beach:
        assets |= {"data/tilesets/mn_gym_6_palette_map.json"} | {
            f"gfx/tilesets/mn_gym_6.{ext}" for ext in ["2bpp","pal","png"]}
    assert changed(parent["runtime_files"],child["runtime_files"]) == assets
    allowed_scripts = set()
    for room_id in ROOMS:
        before, after = a["maps"][room_id], b["maps"][room_id]
        assert before["events"]["warps"] == after["events"]["warps"]
        assert before["trainer_scripts"] == after["trainer_scripts"]
        allowed_scripts |= changed(before["scripts"], after["scripts"])
        if before["map_event_section_commands"] != after["map_event_section_commands"]:
            allowed_scripts.add(room_id + "_MapEvents")
    assert changed(a["map_scripts"],b["map_scripts"]) == allowed_scripts


def check_water(parent, child):
    before, after = Terrain(parent,GYM), Terrain(child,GYM)
    delta = {k for k in before.tiles if before.tiles[k] != after.tiles[k]}
    pools = {tuple(p) for p in json.loads((HERE/"brainerd-pools.json").read_text())["canoe_water"]}
    assert delta == pools and len(pools) == 19
    assert all(before.tiles[p]=="FLOOR" and after.tiles[p]=="WATER" for p in pools)
    dry = {p for p,t in after.tiles.items() if t=="FLOOR"} - after.occupied
    walk = connected((8,12), dry)
    assert walk == dry and all((8,y) in walk for y in range(4,13))
    components, remaining = [], pools.copy()
    while remaining:
        component = connected(next(iter(remaining)), remaining)
        components.append(component)
        remaining -= component
    assert sorted(map(len,components)) == [6,6,7]
    # On every new water square a previously positioned walker has an
    # adjacent exit, and a paddler can land without an event or trainer.
    assert all(neighbors(p) & walk for p in pools)
    room = child["data"]["maps"][GYM]
    art = SourceArt(child).render("mn_gym_6",9,7,room["blocks"],"day").convert("RGB")
    deep = {(82,173,189),(57,140,173)}
    on_dark = {p for p,t in after.tiles.items() if t in {"FLOOR","WATER"}
               and art.getpixel((16*p[0]+8,16*p[1]+15)) in deep}
    assert on_dark == pools, ("visible soles/keel depth", on_dark ^ pools)
    assert all(after.tiles[(o["x"],o["y"])] == "FLOOR" for o in room["objects"])
    print(f"{len(walk)} connected walking squares, straight wade to Sivren, three canoe pools "
          "and a walkable landing beside every water square.")


if __name__ == "__main__":
    cli=argparse.ArgumentParser(description=__doc__)
    cli.add_argument("parent",type=Path)
    cli.add_argument("child",type=Path)
    args=cli.parse_args()
    old, new = map(load_pack, [args.parent,args.child])
    check_campaign(old,new)
    check_water(old,new)
    print(f"Compared all {len(new['data']['maps'])} maps, global scripts and native assets.")
