#!/usr/bin/env python3
"""Check the two native quarry edits, their published mirrors and the two stair gates."""
import argparse
import base64
from collections import deque
from copy import deepcopy
from pathlib import Path
import runpy

HERE = Path(__file__).resolve().parent
preflight = runpy.run_path(str(HERE.parent / "ely-return-v01/check-pack.py"))
read_pack, only = preflight["read_pack"], preflight["only"]
ledge_tools = runpy.run_path(str(HERE / "survey-ledges.py"))
Terrain, DIRS = ledge_tools["Terrain"], ledge_tools["DIRS"]
PARENTS = {"7f581c82ed1d15be9a3275197ceb1b77f13064f8c7f75a94fa7ff465470c1b4d",
           "c41a0e9bf5b9e64b11f15a48fd89148849b8377cdada5c113fadecf034c5eb98"}
GYM = "MnMankatoGym"
MANIFEST = "northstar-quarry-landings-v1"
ENTRANCE = (6, 17)
MOUNTS = {(x, 14) for x in range(6, 14)}


def routes(pack, lower_open, court_open):
    terrain = Terrain(pack, GYM)
    tiles = terrain.tiles.copy()
    for gate, opened in enumerate((lower_open, court_open)):
        label = f"MnMankatoGymV2Gate{gate}{'Open' if opened else 'Closed'}"
        [change] = [row for row in terrain.map["scripts"][label] if row["command"] == "changeblock"]
        x, y, block = map(int, change["args"])
        quadrants = pack["data"]["tilesets"][terrain.tileset]["collision"][f"{block:02x}"]
        for i, token in enumerate(quadrants):
            tiles[x + i % 2, y + i // 2] = token
    assert set(tiles.values()) == {"FLOOR", "HOP_DOWN", "WALL", "WARP_CARPET_DOWN"}
    occupied = terrain.occupied

    def move(start, direction):
        x, y = start
        dx, dy = DIRS[direction]
        distance = 2 if tiles[start] == "HOP_DOWN" and direction == "DOWN" else 1
        end = (x + distance * dx, y + distance * dy)
        if end not in occupied and tiles.get(end) in {"FLOOR", "HOP_DOWN", "WARP_CARPET_DOWN"}:
            return end
        return None

    reachable = {ENTRANCE}
    queue = deque(reachable)
    while queue:
        point = queue.popleft()
        for direction in DIRS:
            dest = move(point, direction)
            if dest is not None and dest not in reachable:
                reachable.add(dest)
                queue.append(dest)
    return reachable, move


def verify(parent, child):
    only(parent, child, {"identity", "report", "data"}, "pack envelope")
    only(parent["identity"], child["identity"], {"runtime_modpack_id", "content_hash"}, "identity")
    only(parent["report"], child["report"], {"manifests"}, "build report")
    chain = parent["report"]["manifests"]
    assert chain[-1] == "northstar-leech-lake-trainers-v1"
    assert child["report"]["manifests"] == chain + [MANIFEST]
    assert child["identity"]["runtime_modpack_id"] == "+".join(chain + [MANIFEST])
    a, b = parent["data"], child["data"]
    only(a, b, {"maps", "map_blocks", "npcs", "map_scripts"}, "game data")
    src, dst = a["maps"][GYM], b["maps"][GYM]
    blocks, events = (src["attributes"][key] for key in ("blocks_label", "map_events_label"))
    for table, key in (("maps", GYM), ("npcs", GYM), ("map_blocks", blocks), ("map_scripts", events)):
        only(a[table], b[table], {key}, table)

    expected = deepcopy(src)
    assert expected["blocks"][64:72] == [5, 18, 18, 18, 18, 18, 6, 9]
    expected["blocks"][70:72] = [18, 6]
    [guide] = [o for o in expected["objects"] if o["script"] == "MnMankatoGymGuide"]
    assert (guide["x"], guide["y"], guide["spritemovedata"], guide["event_flag"]) == (
        9, 16, "SPRITEMOVEDATA_STANDING_DOWN", "-1")
    guide.update(y=17, spritemovedata="SPRITEMOVEDATA_STANDING_LEFT")
    [row] = [r for r in expected["map_event_section_commands"] if guide["script"] in r["args"]]
    assert row["command"] == "object_event" and row["args"][:4] == [
        "9", "16", "SPRITE_GYM_GUIDE", "SPRITEMOVEDATA_STANDING_DOWN"]
    row["args"][1], row["args"][3] = "17", guide["spritemovedata"]
    assert expected == dst, "room differs beyond two metatiles and the guide position/facing"
    assert b["npcs"][GYM] == dst["objects"]
    assert base64.b64decode(b["map_blocks"][blocks]) == bytes(dst["blocks"])
    assert b["map_scripts"][events] == [
        {"command": r["command"], "args": r["args"]} for r in dst["map_event_section_commands"]]

    for phase, lower, court in (("arrival", False, False), ("first win", True, False), ("both wins", True, True)):
        before, old_move = routes(parent, lower, court)
        after, new_move = routes(child, lower, court)
        for visited in (before, after):
            assert bool(MOUNTS & visited) == lower and (not lower or MOUNTS <= visited), phase
            assert ((11, 4) in visited) == court, (phase, "leader audience")
        assert (8, 17) in after and new_move((8, 17), "RIGHT") is None
        assert (7, 17) == new_move(ENTRANCE, "RIGHT") and (8, 17) == new_move((7, 17), "RIGHT")
        old_drops = {x for x, y in MOUNTS if old_move((x, y), "DOWN") == (x, 16)}
        assert old_drops == {6, 7, 8, 10, 11, 12} and all(new_move(p, "DOWN") == (p[0], 16) for p in MOUNTS)
        assert all(new_move((x, 16), "UP") is None for x in range(6, 14)), phase
    old_maps, old_inputs = ledge_tools["inventory"](parent)
    new_maps, new_inputs = ledge_tools["inventory"](child)
    assert old_maps == new_maps and old_inputs.keys() == new_inputs.keys()
    changed = {key for key in old_inputs if old_inputs[key] != new_inputs[key]}
    assert changed == {(GYM, (9, 14), "DOWN"), (GYM, (13, 14), "DOWN")}
    assert old_inputs[GYM, (9, 14), "DOWN"] == ((9, 16), "fixed actor", guide["script"])
    assert old_inputs[GYM, (13, 14), "DOWN"] == ((13, 16), "wall", None)
    for key in changed:
        assert new_inputs[key] == (old_inputs[key][0], "open terrain", None)
    print(f"All {new_maps} Minnesota maps, {len(new_inputs)} directional landings. Exactly two open.")
    print("Two native metatiles, one guide, all split mirrors. Lower lip 6/8 → 8/8. Both stairs retain eligibility.")


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("archives", nargs="+", type=Path, metavar="PARENT CHILD")
    args = parser.parse_args()
    if len(args.archives) % 2:
        parser.error("supply each parent followed by its child")
    for before, after in zip(args.archives[::2], args.archives[1::2]):
        print(before.name, "→", after.name, flush=True)
        verify(read_pack(before, PARENTS), read_pack(after))
