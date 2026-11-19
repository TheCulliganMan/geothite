#!/usr/bin/env python3
"""Verify the installed Rock Cut against its selected parent and audit the two-way cave route."""
import argparse
import base64
import hashlib
import io
import json
import runpy
import sys
from collections import deque
from pathlib import Path

from PIL import Image

HERE = Path(__file__).resolve().parent
BROWN = HERE.parent / "brown-shore-edges-v01"
PREVIOUS_CAVES = HERE.parent / "shore-rocks-cave-mouths-v01"
sys.path.insert(0, str(HERE.parents[1] / "tools"))
from render_reference import SourceArt, load_pack

previous = runpy.run_path(str(BROWN / "review.py"))
require, exact_except, records = previous["require"], previous["exact_except"], previous["records"]
ROOM = "MnR05RockCut"
NORTH, SOUTH = "MnR05C00R00", "MnR05C00R01"
MANIFEST = "northstar-rock-cut-v1"
BANK_MANIFEST = "northstar-rock-cut-stone-v1"
BANK = "mn_r05_rock_cut"
STEPS = {"UP": (0, -1), "DOWN": (0, 1), "LEFT": (-1, 0), "RIGHT": (1, 0)}
UPPER, LOWER = (11, 4), (5, 16)
STAIR = (9, 7)
LIPS = {(6, 6), (7, 6)}
PORTALS = {(5, 17), (11, 5)}


def source_bank(parent, child):
    """Construct only the authorized private copy of the original Crystal cave bank."""
    original = parent["data"]["tilesets"]["cave"]
    old_files, new_files = parent["runtime_files"], child["runtime_files"]
    require(BANK not in parent["data"]["tilesets"], "private cave tileset already exists")
    mirrored = dict(old_files)
    for template in ("data/tilesets/{}_metatiles.bin", "data/tilesets/{}.json", "gfx/tilesets/{}.2bpp"):
        mirrored[template.format(BANK)] = old_files[template.format("cave")]
    palette = f"data/tilesets/{BANK}_palette_map.json"
    require(json.loads(bytes(new_files[palette])) == original["palette_map"], "private cave palette changed")
    mirrored[palette] = new_files[palette]
    label = f"gfx/tilesets/{BANK}.png"
    for_new = Image.open(io.BytesIO(bytes(new_files[label]))).convert("RGBA")
    for_old = Image.open(io.BytesIO(bytes(old_files["gfx/tilesets/cave.png"]))).convert("RGBA")
    require(for_new.size == for_old.size and for_new.tobytes() == for_old.tobytes(), "cave glyph pixels changed")
    mirrored[label] = new_files[label]
    return dict(parent, runtime_files=mirrored,
                data=dict(parent["data"], tilesets={**parent["data"]["tilesets"], BANK: original}))


def check_delta(parent, child):
    """Apply the authored block list to the parent and require exact equality everywhere else."""
    exact_except(parent, child, {"data", "runtime_files", "identity", "report"}, "pack")
    exact_except(parent["report"], child["report"], {"manifests"}, "report")
    require(child["report"]["manifests"] == parent["report"]["manifests"] + [BANK_MANIFEST, MANIFEST],
            "revision lineage")
    old_id, new_id = parent["identity"], child["identity"]
    exact_except(old_id, new_id, {"content_hash", "runtime_modpack_id"}, "identity")
    require(new_id["runtime_modpack_id"] == "+".join((old_id["runtime_modpack_id"], BANK_MANIFEST, MANIFEST)),
            "unexpected runtime lineage")
    require(new_id["content_hash"] != old_id["content_hash"] and len(new_id["content_hash"]) == 64,
            "missing new pack identity")
    recipes = records(HERE, "exterior-art") + records(HERE, "interior-art")
    ids = previous["added_route_art"](source_bank(parent, child), child, recipes)
    old, new = parent["data"], child["data"]
    exact_except(old, new, {"maps", "map_blocks", "map_attributes", "tilesets"}, "game data")
    require(old["maps"].keys() == new["maps"].keys(), "map inventory")
    rooms = records(HERE, "exteriors") + records(HERE, "interior")
    names = [r["map"] for r in rooms]
    require(len(set(names)) == len(names) and set(names) <= {NORTH, SOUTH, ROOM}, "invalid map scope")
    require(ROOM in names, "no Rock Cut interior in revision")
    mirrors = dict(old["map_blocks"])
    changes = 0
    habitat = set()
    protected = previous["cells"](PREVIOUS_CAVES, previous["SHORES"])
    for name in old["maps"].keys() - set(names):
        require(old["maps"][name] == new["maps"][name], name + " changed outside Rock Cut")
    for design in rooms:
        name, bank = design["map"], design["tileset"]
        room, installed = old["maps"][name], new["maps"][name]
        if name == ROOM:
            exact_except(room, installed, {"blocks", "attributes"}, name)
            attrs = dict(room["attributes"], tileset_name=BANK, border_block=0)
            require(attrs == installed["attributes"] and room["attributes"]["border_block"] == 46,
                    "private cave bank and cave perimeter")
        else:
            exact_except(room, installed, {"blocks"}, name)
            attrs = room["attributes"]
        require(attrs["tileset_name"] == bank, name + " tileset")
        blocks, used = list(room["blocks"]), set()
        for change in design["placements"]:
            x, y = change["at"]
            require(0 <= x < attrs["width"] and 0 <= y < attrs["height"] and (x,y) not in used,
                    f"{name}: invalid or repeated placement {(x,y)}")
            used.add((x, y))
            require((name, x, y) not in protected, f"{name} {(x,y)}: protected original shoreline")
            index = y*attrs["width"] + x
            target = ids.get((bank, change["to"]), change["to"])
            require(blocks[index] == change["from"] and target != blocks[index], f"{name} {(x,y)}: stale placement")
            if name != ROOM:
                baseline = blocks[index]
                previous_mask = old["tilesets"][bank]["collision"][f"{baseline:02x}"]
                final_mask = new["tilesets"][bank]["collision"][f"{target:02x}"]
                for q, (left,right) in enumerate(zip(previous_mask, final_mask)):
                    if left in previous["PRESERVED_TERRAIN"] or right in previous["PRESERVED_TERRAIN"]:
                        require(left == right, f"{name} {(x,y)}: changed water or grass footprint")
                        habitat.add((bank, baseline, target, q))
            blocks[index] = target
            changes += 1
        require(blocks == installed["blocks"], f"{name}: installed map differs from the declared changes")
        mirrors[attrs["blocks_label"]] = base64.b64encode(bytes(blocks)).decode("ascii")
    require(mirrors == new["map_blocks"], "packed blocks do not match their map mirrors")
    attributes = dict(old["map_attributes"])
    require(attributes[ROOM] == old["maps"][ROOM]["attributes"], "parent cave attribute mirror is stale")
    attributes[ROOM] = new["maps"][ROOM]["attributes"]
    require(attributes == new["map_attributes"], "split attribute cache differs from installed room attributes")
    old_art, new_art = SourceArt(parent), SourceArt(child)
    for bank, left, right, q in habitat:
        x, y = q%2*16, q//2*16
        box = (x,y,x+16,y+16)
        for time in ("morn","day","nite"):
            require(old_art.blocks(bank,time)[left].crop(box).tobytes() ==
                    new_art.blocks(bank,time)[right].crop(box).tobytes(), f"{bank}: native habitat artwork changed")
    return changes


def terrain(pack, name):
    data = pack["data"]
    room = data["maps"][name]
    attrs = room["attributes"]
    collision = data["tilesets"][attrs["tileset_name"]]["collision"]
    w = attrs["width"]
    return {(x, y): collision[f"{room['blocks'][y//2*w + x//2]:02x}"][y%2*2 + x%2]
            for y in range(attrs["height"]*2) for x in range(w*2)}


def graph(tiles, occupied, hops=True):
    """Single-step graph of the cave's installed land and cardinal wall/ledge permissions."""
    barriers = {"WALL", "WATER", "VOID"}
    allowed = {"FLOOR", "HOP_DOWN", "LADDER", "CAVE", "DOOR", "WARP_CARPET_DOWN", "WARP_CARPET_UP"}
    allowed.update(role for role in tiles.values() if role.endswith("_WALL") and role != "WALL"
                   and all(side in STEPS for side in role[:-5].split("_")))
    require(set(tiles.values()) <= (allowed | barriers), f"unreviewed cave collision: {set(tiles.values()) - allowed - barriers}")
    vertices = {p for p, role in tiles.items() if role in allowed} - occupied
    edges = {p: set() for p in vertices}
    for (x, y), neighbors in edges.items():
        start = tiles[x, y]
        for direction, (dx, dy) in STEPS.items():
            if start.endswith("_WALL") and direction in start[:-5].split("_"):
                continue
            jumping = direction == "DOWN" and start == "HOP_DOWN"
            scale = 2 if jumping else 1
            dest = (x+scale*dx, y+scale*dy)
            if dest not in edges or (jumping and not hops):
                continue
            role = tiles[dest]
            if role.endswith("_WALL") and any(STEPS[s] == (-dx,-dy) for s in role[:-5].split("_")):
                continue
            neighbors.add(dest)
    return edges


def reachable(edges, start):
    if start not in edges:
        return {}
    distances = {start: 0}
    queue = deque([start])
    while queue:
        at = queue.popleft()
        for step in edges[at] - distances.keys():
            distances[step] = distances[at] + 1
            queue.append(step)
    return distances


def access(pack):
    room = pack["data"]["maps"][ROOM]
    tiles = terrain(pack, ROOM)
    actors = {(o["x"], o["y"]): o for o in room["objects"]}
    require(actors[(5,13)]["script"] == "MN_R05_CUT_WALKER"
            and actors[(5,5)]["script"] == "MN_R05_ITEM_03", "healer or Repel moved")
    programs = room["scripts"]
    require(any(c["command"] == "special" and c["args"] == ["HealParty"]
                for c in programs["MN_R05_CUT_WALKER"]), "healer lost party treatment")
    require(programs["MN_R05_ITEM_03"] == [{"command": "itemball", "args": ["REPEL", "1"]}],
            "lost Repel reward")
    require(any((e["x"], e["y"], e["event_type"], e["script"])
                == (10, 9, "BGEVENT_ITEM", "MN_HIDDEN_R05ROCKCUT") for e in room["events"]["bg_events"])
            and programs["MN_HIDDEN_R05ROCKCUT"] == [
                {"command": "hiddenitem", "args": ["HARD_STONE", "EVENT_MN_HIDDEN_R05ROCKCUT"]}],
            "hidden Hard Stone lost its original pickup or event")
    require({(w["x"],w["y"]) for w in room["events"]["warps"]} == PORTALS, "unexpected indoor exits")
    require({p for p, token in tiles.items() if token in {"LADDER","CAVE","DOOR"}
             or token.startswith("WARP_CARPET_")} == PORTALS, "unbound threshold or missing exit")
    require({p for p, role in tiles.items() if role == "HOP_DOWN"} == LIPS, "shelf takeoffs moved")
    xmax, ymax = max(x for x,y in tiles), max(y for x,y in tiles)
    require(all(role == "WALL" for (x,y), role in tiles.items() if x in (0,xmax) or y in (0,ymax)),
            "walkable room edge reaches the unfitted outer rock")
    distances = {}
    for picked_up in (False, True):
        bodies = set(actors)
        if picked_up:
            bodies.remove((5, 5))
        open_tiles = graph(tiles, bodies)
        # Walk off the actual arrival tile and walk back onto each threshold from the cave.
        # Directional carpets additionally require the outward button while standing on them.
        for mouth, clear in (((5,17), LOWER), ((11,5), UPPER)):
            role = tiles[mouth]
            require(role == "WARP_CARPET_DOWN", f"{mouth}: horizontal cave exit needs a ground-level threshold")
            require(clear in open_tiles[mouth] and mouth in open_tiles[clear], f"blocked arrival {mouth}")
            approach = mouth[0], mouth[1]-1
            require(approach in open_tiles[mouth] and mouth in open_tiles[approach],
                    f"no floor-facing approach to directional threshold {mouth}")
        blocked = bodies | PORTALS
        walk = graph(tiles, blocked)
        north, south = reachable(walk, UPPER), reachable(walk, LOWER)
        require(walk.keys() == north.keys() == south.keys(), "stranded accessible cave floor or dead end")
        inverse = {at: set() for at in walk}
        for at, neighbors in walk.items():
            for dest in neighbors:
                inverse[dest].add(at)
        require(reachable(inverse, LOWER).keys() == walk.keys(), "one-way cave trap without a return")
        for target in ((6,13), (6,5), (10,8)):
            require(target in north and target in south, f"cannot face original service/reward from {target}")
        for x, y in LIPS:
            landing = x, y+2
            require(tiles[x,y+1] == "WALL" and landing in walk[x,y]
                    and (x,y) not in walk[landing], f"broken one-way shelf at {(x,y)}")
        plain = graph(tiles, blocked, hops=False)
        require(UPPER in reachable(plain, LOWER) and LOWER in reachable(plain, UPPER), "no ordinary stair return")
        no_stair = graph(tiles, blocked | {STAIR}, hops=False)
        require(UPPER not in reachable(no_stair, LOWER), "upper shelf bypasses the fitted stair")
        if not picked_up:
            distances = north[LOWER], south[UPPER], len(walk)
    return distances


def visible_landmarks(pack):
    room = pack["data"]["maps"][ROOM]
    attrs = room["attributes"]
    art = SourceArt(pack)
    original = art.blocks("cave", "nite")
    actual = art.blocks(BANK, "nite")

    def square(block, x, y, height=16):
        left, top = 16*(x%2), 16*(y%2)
        return block.crop((left, top, left+16, top+height)).tobytes()

    # Native walking squares include the stair treads, the hidden item's boulder and the light
    # directly beyond each outdoor threshold. An unchanged collision token cannot prove this.
    for (x,y), block, q, height in ((STAIR,0x12,3,16), ((10,9),0x22,3,16),
                                   ((5,18),0x23,1,8), ((11,6),0x23,1,8)):
        selected = room["blocks"][y//2*attrs["width"] + x//2]
        require(square(actual[selected],x,y,height) == square(original[block],q%2,q//2,height),
                f"Rock Cut {(x,y)}: missing native stair, boulder or threshold light")
    upper = room["blocks"][2*attrs["width"] + 4]
    require(square(actual[room["blocks"][0]],0,0) != square(actual[upper],8,5),
            "dark solid exterior is visually identical to the walkable upper gallery")


def exterior_access(pack):
    """Check dry paths with actors in place and count the grass squares on shortest routes."""
    anchors = {
        NORTH: [((24,28),(30,28),6), ((30,28),(37,29),10), ((37,24),(37,29),5)],
        SOUTH: [((13,36),(13,29),7), ((13,36),(16,30),9), ((13,36),(12,31),6), ((13,36),(18,31),10)],
    }
    for name, tests in anchors.items():
        tiles = terrain(pack, name)
        room = pack["data"]["maps"][name]
        actors = {(o["x"],o["y"]) for o in room["objects"]}
        ordinary = {p for p,t in tiles.items() if t in {"FLOOR","TALL_GRASS","LONG_GRASS"}} - actors
        edges = {p: {(p[0]+dx,p[1]+dy) for dx,dy in STEPS.values() if (p[0]+dx,p[1]+dy) in ordinary}
                 for p in ordinary}
        for start, target, steps in tests:
            distance = reachable(edges, start)
            require(distance.get(target) == steps, f"{name}: ordinary path {start} to {target} changed")
            if name == SOUTH:
                grass = {start: 0}
                for p, moves in distance.items():
                    if moves == 0 or moves > steps:
                        continue
                    cost = int(tiles[p] in {"TALL_GRASS","LONG_GRASS"})
                    previous_steps = [grass[q] for q in edges[p] if distance.get(q) == moves-1]
                    grass[p] = min(previous_steps)+cost
                require(grass[target] == 4, f"{name}: minimum grass on shortest route to {target}: {grass[target]}")
        if name == SOUTH:
            signs = {(e["x"],e["y"]) for e in room["events"]["bg_events"] if e["event_type"] == "BGEVENT_READ"}
            require({(12,30),(18,30)} <= signs and all(tiles[p] == "WALL" for p in ((12,30),(18,30))),
                    "southern road or cave sign lost its interaction face")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("parent", type=Path)
    parser.add_argument("candidate", type=Path)
    args = parser.parse_args()
    supported = {records(BROWN, name+"-build")["expected_pack_sha256"] for name in ("field", "mothlight")}
    require(hashlib.sha256(args.parent.read_bytes()).hexdigest() in supported, "expected a selected brown-shore parent")
    parent, candidate = load_pack(args.parent), load_pack(args.candidate)
    changes = check_delta(parent, candidate)
    runpy.run_path(str(PREVIOUS_CAVES / "check-caves.py"))["check_caves"](candidate)
    previous["route_connections"](candidate)
    exterior_access(candidate)
    down, up, count = access(candidate)
    visible_landmarks(candidate)
    print(f"Verified {changes} declared placements. Both cave exits, the two drop lanes, fitted stair, healer, "
          f"Repel and hidden Hard Stone passed. All {count} resting tiles are reachable before pickup. "
          f"North to south takes {down} moves and south to north takes {up}.")


if __name__ == "__main__":
    main()
