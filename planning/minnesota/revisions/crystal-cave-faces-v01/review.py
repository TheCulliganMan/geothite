#!/usr/bin/env python3
"""Find cave portals in the compiled world and compare their installed stone to original Crystal."""

import argparse
import base64
import hashlib
import io
import runpy
from collections import Counter, defaultdict, deque
from pathlib import Path

from PIL import Image

HERE = Path(__file__).resolve().parent
PREVIOUS = HERE.parent / "rock-cut-north-height-v01"
archive = runpy.run_path(str(HERE.parent / "rock-cut-v01" / "review.py"))
caves = runpy.run_path(str(archive["PREVIOUS_CAVES"] / "check-caves.py"))
brown = archive["previous"]
require, same_except, records = archive["require"], archive["exact_except"], archive["records"]
OUTSIDE = {"ROUTE", "TOWN"}
REFERENCES = (
    ("Route42", 10, 5),
    ("Route42", 28, 9),
    ("Route42", 46, 7),
    ("Route31", 34, 5),
    ("Route45", 2, 5),
    ("Route44", 56, 7),
)
MANIFEST = "northstar-crystal-cave-faces-v1"
CHANGED_PERMISSIONS = {
    ("MnFormerShores", "FLOOR", "UP_WALL"): {(22, 20), (23, 20), (24, 20)},
    ("MnFormerShores", "FLOOR", "WALL"): {(25, 20), (25, 21), (21, 22), (22, 22), (23, 22), (24, 22)},
    ("MnFormerShores", "WALL", "FLOOR"): {(22, 23), (23, 23), (24, 23), (25, 23)},
    ("MnR05C00R01", "WALL", "UP_WALL"): {(19, 24), (20, 24)},
    ("MnR05C00R01", "WALL", "FLOOR"): {
        (19, 25),
        (20, 25),
        (19, 26),
        (20, 26),
        (19, 27),
        (20, 27),
        (19, 28),
        (20, 28),
        (20, 29),
        (20, 31),
    },
    ("MnR05C00R01", "FLOOR", "WALL"): {(19, 30)},
    ("MnR09C00R00", "WALL", "FLOOR"): {(45, 41), (46, 41)},
    ("MnR09C00R00", "FLOOR", "WALL"): {(44, 42), (45, 42), (46, 42), (47, 42)},
}


def quadrant_glyphs(pack, bank, block, q):
    raw = pack["runtime_files"][f"data/tilesets/{bank}_metatiles.bin"]
    start = 16 * block + 8 * (q // 2) + 2 * (q % 2)
    return tuple(raw[start + i] for i in (0, 1, 4, 5))


def mine_frame(world):
    """The timber lintel crosses the seam at native column 80. The original aperture stays in place."""
    name = "MnR09C00R00"
    bank = world.maps[name]["attributes"]["tileset_name"]
    ink = world.bank(bank)[0]
    for y, row in (
        (81, (0xF3, 0xF4, 0xF4, 0xF4, 0xF5)),
        (82, (0x44, 0xF6, 0xF7, 0x44, 0x4C)),
        (83, (0x44, 0xF9, 0xFA, 0x44, 0x4C)),
    ):
        require(
            [world.glyph(name, x, y) for x in range(79, 84)] == [ink[t] for t in row],
            f"{name}: timber lintel, posts or recessed shaft changed on native row {y}",
        )


def game_delta(parent, child, mine_parent_id):
    same_except(parent, child, {"identity", "report", "runtime_files", "data"}, "pack")
    same_except(
        parent["identity"], child["identity"], {"content_hash", "runtime_modpack_id"}, "identity"
    )
    same_except(parent["report"], child["report"], {"manifests"}, "report")
    require(
        child["report"]["manifests"] == parent["report"]["manifests"] + [MANIFEST]
        and child["identity"]["runtime_modpack_id"]
        == parent["identity"]["runtime_modpack_id"] + "+" + MANIFEST,
        "incorrect cave revision lineage",
    )
    old, new = parent["data"], child["data"]
    same_except(old, new, {"tilesets", "maps", "map_blocks"}, "game data")
    designs = records(HERE, "rock-cut") + records(HERE, "regional")
    art = records(HERE, "rock-cut-art") + records(HERE, "regional-art")
    # Two pins of the same timbered door exist. Verify native layout and role on the matching
    # parent before substituting its own metatile index into the common design.
    source = next(r for r in designs if r["map"] == "MnR09C00R00")
    mine = old["maps"][source["map"]]
    bank = source["tileset"]
    block = mine["blocks"][20 * mine["attributes"]["width"] + 20]
    expected = bytes(
        [0x3C, 0x3C, 0x3C, 0x3C, 0xF3, 0xF4, 0xF5, 0x3C, 0xF6, 0xF7, 0xF8, 0x4C, 0xF9, 0xFA, 0xFB, 0x4C]
    )
    layouts = parent["runtime_files"][f"data/tilesets/{bank}_metatiles.bin"]
    require(
        block == mine_parent_id
        and bytes(layouts[block * 16 : block * 16 + 16]) == expected
        and old["tilesets"][bank]["collision"][f"{block:02x}"] == ["FLOOR", "FLOOR", "CAVE", "WALL"],
        "wrong family-specific timber entrance or collision donor",
    )
    opening = next(c for c in source["placements"] if c["at"] == [20, 20])
    recipe = next(a for a in art if a["tileset"] == bank and a["id"] == opening["to"])
    require(opening["from"] == recipe["collision_from"] == 0x88, "unexpected common timber source")
    opening["from"] = recipe["collision_from"] = block
    ids = brown["added_route_art"](parent, child, art)
    names = {room["map"] for room in designs}
    same_except(old["maps"], new["maps"], names, "maps")
    mirrors = dict(old["map_blocks"])
    changes = set()
    permissions = defaultdict(set)
    for name in names:
        before, after = old["maps"][name], new["maps"][name]
        same_except(before, after, {"blocks"}, name)
        attrs = before["attributes"]
        bank = attrs["tileset_name"]
        blocks = list(before["blocks"])
        for design in (r for r in designs if r["map"] == name):
            require(design["tileset"] == bank, f"{name} wrong tileset")
            for cell in design["placements"]:
                x, y = cell["at"]
                key = name, x, y
                require(
                    key not in changes and 0 <= x < attrs["width"] and 0 <= y < attrs["height"],
                    f"{key}: duplicate or out of bounds",
                )
                changes.add(key)
                i = y * attrs["width"] + x
                dest = ids.get((bank, cell["to"]), cell["to"])
                require(blocks[i] == cell["from"] and dest != blocks[i], f"{key}: stale source")
                for q in range(4):
                    pos = (2 * x + q % 2, 2 * y + q // 2)
                    old_role, old_ink = caves["quadrant"](parent, before, *pos)
                    new_role, new_ink = caves["quadrant"](child, after, *pos)
                    if old_role != new_role:
                        permissions[name, old_role, new_role].add(pos)
                    if {old_role, new_role} & (brown["PRESERVED_TERRAIN"] | {"CAVE"}):
                        require(
                            (old_role, old_ink) == (new_role, new_ink)
                            and quadrant_glyphs(parent, bank, blocks[i], q)
                            == quadrant_glyphs(child, bank, dest, q),
                            f"{name} {pos}: altered original habitat or entrance graphic",
                        )
                blocks[i] = dest
        require(blocks == after["blocks"], f"{name}: undeclared changed block")
        mirrors[attrs["blocks_label"]] = base64.b64encode(bytes(blocks)).decode("ascii")
    require(mirrors == new["map_blocks"], "room data and native block cache disagree")
    require(
        dict(permissions) == CHANGED_PERMISSIONS, f"undeclared collision changes: {dict(permissions)}"
    )
    return len(changes)


class NativeWorld:
    """Match visible 8px glyphs across tilesets, metatile boundaries, connected maps and story variants."""

    def __init__(self, pack):
        self.pack = pack
        self.banks = {}
        self.variant_sets = {}
        self.maps = pack["data"]["maps"]
        self.names = {k: n for n, r in self.maps.items() for k in (n, r["attributes"]["map_constant"])}
        self.ids = {}
        self.changed = defaultdict(set)
        self.replacements = defaultdict(lambda: defaultdict(set))
        self.script_warps = []
        self.commands = 0
        for move in pack["data"]["field_moves"].values():
            if isinstance(move, dict):
                for bank, pairs in move.get("replacements", {}).items():
                    for k, relation in pairs.items():
                        self.replacements[bank][int(k)].add(relation["replacement_block_id"])
        for name, room in self.maps.items():
            if not name.startswith("Mn"):
                continue
            authored_changes, authored_warps = set(), set()
            for label, program in room["scripts"].items():
                for order, command in enumerate(program):
                    cmd, args = command["command"], command["args"]
                    if cmd == "changeblock":
                        x, y, b = (int(s[1:], 16) if s.startswith("$") else int(s, 0) for s in args)
                        require(
                            x % 2 == 0
                            and y % 2 == 0
                            and 0 <= x // 2 < room["attributes"]["width"]
                            and 0 <= y // 2 < room["attributes"]["height"],
                            f"{name}: bad changeblock at {x, y}",
                        )
                        self.changed[name, x // 2, y // 2].add(b)
                        self.commands += 1
                        authored_changes.add((label, order, x, y, b))
                    if cmd in ("warp", "warpfacing"):
                        # The compiled command format is map,x,y[,direction]. The typed runtime
                        # record below verifies the map and facing without relying on ASM order.
                        require(
                            len(args) == (4 if cmd == "warpfacing" else 3), f"{name}: malformed {cmd}"
                        )
                        target = self.names[args[0]]
                        direction = args[3] if cmd == "warpfacing" else None
                        require(
                            direction is None or direction in {"DOWN", "LEFT", "RIGHT", "UP"},
                            f"{name}: malformed warp facing",
                        )
                        self.script_warps.append((name, label, target))
                        authored_warps.add((label, order, target, int(args[1]), int(args[2]), direction))
            applied = {
                (e["source_script"], e["command_index"], e["x"], e["y"], e["block_id"])
                for e in room["script_block_changes"]
            }
            require(
                applied == authored_changes, f"{name}: changeblock scripts and installed changes differ"
            )
            applied_warps = {
                (
                    e["source_script"],
                    e["command_index"],
                    self.names[e["target_map"]],
                    e["x"],
                    e["y"],
                    e["facing"],
                )
                for e in room["script_map_commands"]
                if e["command"] in {"warp", "warpfacing"}
            }
            require(
                applied_warps == authored_warps, f"{name}: script and installed warp destinations differ"
            )
        shared = self.pack["data"]["global_scripts"]
        require(not shared["script_block_changes"], "unclassified global map edits")
        require(
            not any(
                c.get("target_map")
                and self.names.get(c["target_map"], "").startswith("Mn")
                and self.maps[self.names[c["target_map"]]]["attributes"]["environment"] == "CAVE"
                for c in shared["script_map_commands"]
            ),
            "global script adds a Minnesota cave warp",
        )

    def bank(self, bank):
        if bank in self.banks:
            return self.banks[bank]
        files = self.pack["runtime_files"]
        sheet = (
            Image.open(io.BytesIO(bytes(files[f"gfx/tilesets/{bank}.png"])))
            .convert("RGB")
            .getchannel("R")
        )
        count = sheet.width * sheet.height // 64
        cols = sheet.width // 8
        palettes = self.pack["data"]["tilesets"][bank]["palette_map"]
        # The native render resolves logical glyphs into physical banks and four shade indices.
        shades = [0 if v >= 213 else 1 if v >= 128 else 2 if v >= 43 else 3 for v in range(256)]
        sheet = sheet.point(shades)
        lookup = []
        for tile in range(256):
            upper = tile < len(palettes) and palettes[tile] & 8
            if upper and count % 2 == 0 and (tile & 127) + count // 2 < count:
                index = (tile & 127) + count // 2
            elif upper and (tile & 127) + 128 < count:
                index = (tile & 127) + 128
            elif tile < count:
                index = tile
            else:
                index = tile - 128 if 0 <= tile - 128 < count else 0
            x, y = index % cols * 8, index // cols * 8
            pixels = sheet.crop((x, y, x + 8, y + 8)).tobytes()
            lookup.append(self.ids.setdefault(pixels, len(self.ids)))
        layout = files[f"data/tilesets/{bank}_metatiles.bin"]
        blocks = [tuple(lookup[g] for g in layout[i : i + 16]) for i in range(0, len(layout), 16)]
        self.banks[bank] = (lookup, blocks)
        return self.banks[bank]

    def resolve(self, name, x, y):
        room = self.maps[name]
        a = room["attributes"]
        w, h = 4 * a["width"], 4 * a["height"]
        if not (0 <= x < w and 0 <= y < h):
            direction = "west" if x < 0 else "east" if x >= w else "north" if y < 0 else "south"
            links = [l for l in a["connections"] if l["direction"] == direction]
            if len(links) != 1:
                return None
            link = links[0]
            target = link["target_map"]
            other = self.maps[target]["attributes"]
            offset = 4 * link["offset"]
            if direction in ("west", "east"):
                x, y = (x + 4 * other["width"] if x < 0 else x - w), y - offset
            else:
                x, y = x - offset, (y + 4 * other["height"] if y < 0 else y - h)
            if not (0 <= x < 4 * other["width"] and 0 <= y < 4 * other["height"]):
                return None
            name = target
        return (name, x // 4, y // 4), 4 * (y % 4) + x % 4

    def block(self, name, x, y):
        room = self.maps[name]
        return room["blocks"][y * room["attributes"]["width"] + x]

    def glyph(self, name, x, y):
        pos = self.resolve(name, x, y)
        if pos is None:
            return None
        key, offset = pos
        n = key[0]
        return self.bank(self.maps[n]["attributes"]["tileset_name"])[1][self.block(*key)][offset]

    def square(self, name, x, y):
        return tuple(
            self.glyph(name, 2 * x + dx, 2 * y + dy) for dx, dy in ((0, 0), (1, 0), (0, 1), (1, 1))
        )

    def variants(self, bank, original, overrides):
        key = (bank, original, overrides)
        if key in self.variant_sets:
            return self.variant_sets[key]
        options = {original, *overrides}
        pending = list(options)
        while pending:
            for other in self.replacements[bank].get(pending.pop(), ()):
                if other not in options:
                    options.add(other)
                    pending.append(other)
        self.variant_sets[key] = options
        return options

    def choices(self, key):
        bank = self.maps[key[0]]["attributes"]["tileset_name"]
        options = self.variants(bank, self.block(*key), tuple(self.changed.get(key, ())))
        return self.bank(bank)[1], options

    def possible_image(self, name, x, y, image):
        # Multiple glyphs within the same block must come from the SAME story/field-move state.
        parts = defaultdict(list)
        for (dx, dy), ink in zip(((0, 0), (1, 0), (0, 1), (1, 1)), image):
            pos = self.resolve(name, x + dx, y + dy)
            if pos is None:
                return False
            key, offset = pos
            parts[key].append((offset, ink))
        for key, part in parts.items():
            blocks, choices = self.choices(key)
            if not any(all(blocks[b][i] == ink for i, ink in part) for b in choices):
                return False
        return True

    def scan_art(self, images):
        starts = defaultdict(list)
        for image in images:
            starts[image[0]].append(image)
        found, permissions = set(), set()
        for name, room in self.maps.items():
            if not name.startswith("Mn"):
                continue
            a = room["attributes"]
            bank = a["tileset_name"]
            blocks = self.bank(bank)[1]
            collision = self.pack["data"]["tilesets"][bank]["collision"]
            offsets = {
                b: [i for i, g in enumerate(pixels) if g in starts] for b, pixels in enumerate(blocks)
            }
            for j, b in enumerate(room["blocks"]):
                x, y = j % a["width"], j // a["width"]
                _, options = self.choices((name, x, y))
                for candidate in options:
                    for q, role in enumerate(collision[f"{candidate:02x}"]):
                        if role == "CAVE":
                            permissions.add((name, 2 * x + q % 2, 2 * y + q // 2))
                    for i in offsets[candidate]:
                        gx, gy = x * 4 + i % 4, y * 4 + i // 4
                        if any(
                            self.possible_image(name, gx, gy, im) for im in starts[blocks[candidate][i]]
                        ):
                            found.add((name, gx, gy))
        return found, permissions


def cliff_geometry(world, name, x, y):
    original = world.bank(world.maps["Route42"]["attributes"]["tileset_name"])[0]
    rock = {original[t] for t in (0x2B, 0x2C, 0x2D, 0x3B, 0x3C, 0x3D, 0x4B, 0x4C, 0x4D)}
    front = {original[k] for k in (0x4B, 0x4C, 0x4D)}
    X, Y = x * 2, y * 2

    def always(dx, dy, types):
        where = world.resolve(name, X + dx, Y + dy)
        if where is None:
            return False
        key, offset = where
        blocks, options = world.choices(key)
        return all(blocks[b][offset] in types for b in options)

    for side in (range(-4, 0), range(2, 6)):
        # Crystal's tightest outer bevel leaves one whole vertical column. On a straight face the
        # full pair of native rows continues across all four columns. This rejects a one-row pebble rim.
        require(
            any(all(always(dx, dy, front) for dy in (0, 1)) for dx in side),
            f"{name} {(x, y)}: missing 16px-high double course on the {'west' if side.start < 0 else 'east'} shoulder",
        )
    # Trace contiguous native stone behind and beyond the doorway. A short 32px facade with
    # grass or forest above and behind it has no connected ridge. The wing can turn a corner;
    # at Granite Passage it can cross into the adjoining shard.
    stone = {(dx, dy) for dx in range(-10, 12) for dy in range(-8, 0) if always(dx, dy, rock)}
    seed = (0, -2)
    require(seed in stone, f"{name} {(x, y)}: no stone behind the top of the opening")
    seen, todo = {seed}, deque([seed])
    while todo:
        a, b = todo.popleft()
        for q in ((a - 1, b), (a + 1, b), (a, b - 1), (a, b + 1)):
            if q in stone and q not in seen:
                seen.add(q)
                todo.append(q)
    xmin, xmax = min(a for a, b in seen), max(a for a, b in seen)
    require(
        xmax - xmin + 1 >= 10 and min(b for a, b in seen) <= -5,
        f"{name} {(x, y)}: insufficient contiguous rock backing ({xmin}..{xmax})",
    )
    return xmax - xmin + 1, -min(b for a, b in seen)


def public_approaches(pack):
    walks = records(archive["PREVIOUS_CAVES"], "mouth-walks")
    for walk in walks:
        name = walk["map"]
        room = pack["data"]["maps"][name]
        actors = {(o["x"], o["y"]) for o in room["objects"]}
        ordinary = {
            p
            for p, role in archive["terrain"](pack, name).items()
            if role in {"FLOOR", "TALL_GRASS", "LONG_GRASS"}
        } - actors
        start, end = tuple(walk["from"]), tuple(walk["to"])
        reachable = {start} if start in ordinary else set()
        frontier = reachable.copy()
        for _ in range(walk["max_steps"]):
            frontier = {
                (x + dx, y + dy)
                for x, y in frontier
                for dx, dy in ((0, 1), (0, -1), (1, 0), (-1, 0))
                if (x + dx, y + dy) in ordinary
            } - reachable
            reachable.update(frontier)
        require(end in reachable, f"{name}: broken public path from {start} to {end}")
    return len(walks)


def discovery(pack):
    world = NativeWorld(pack)
    env = {n: r["attributes"]["environment"] for n, r in world.maps.items()}
    regional = {n for n in world.maps if n.startswith("Mn")}
    cave_warps, interiors, exterior_portals = Counter(), Counter(), set()
    building_doors, refs, alternate_refs = set(), set(), set()
    inspected = 0
    for name, room in world.maps.items():
        for warp in room["events"]["warps"]:
            dest = world.names[warp["target_map_constant"]]
            x, y = warp["x"], warp["y"]
            if name in regional:
                inspected += 1
                if "CAVE" in {env[name], env[dest]}:
                    cave_warps[env[name], env[dest]] += 1
                if env[name] in OUTSIDE and env[dest] == "DUNGEON":
                    building_doors.add(dest)
                if env[name] not in OUTSIDE and env[dest] != "CAVE" and env[name] != "CAVE":
                    continue
            elif env[name] not in OUTSIDE or env[dest] != "CAVE":
                continue
            if env[name] != "CAVE" and env[dest] != "CAVE":
                continue
            attrs = room["attributes"]
            b = world.block(name, x // 2, y // 2)
            role = pack["data"]["tilesets"][attrs["tileset_name"]]["collision"][f"{b:02x}"][
                2 * (y % 2) + x % 2
            ]
            if name in regional:
                target_records = world.maps[dest]["events"]["warps"]
                require(
                    any(r["index"] == warp["target_warp_id"] for r in target_records),
                    f"{name} {x, y}: cave connection has no arrival #{warp['target_warp_id']} on {dest}",
                )
                if role == "WALL":
                    require(
                        (name, x, y, dest) == ("MnR09Cave1F", 19, 11, "MnIronDrillApproach"),
                        f"{name} {x, y}: unclassified wall at cave entrance",
                    )
                    _, options = world.choices((name, x // 2, y // 2))
                    collision = pack["data"]["tilesets"][attrs["tileset_name"]]["collision"]
                    require(
                        any(collision[f"{i:02x}"][2 * (y % 2) + x % 2] == "LADDER" for i in options),
                        "foreman's gated drift never opens into a ladder",
                    )
                if env[name] in OUTSIDE and env[dest] == "CAVE" and role != "FLOOR":
                    exterior_portals.add((name, x, y))
                elif env[name] == "CAVE":
                    interiors[role] += 1
            elif role == "CAVE":
                refs.add((name, x, y))
            else:
                alternate_refs.add((name, x, y, role))
    require(
        not any("CAVE" in {env[n], env[t]} for n, label, t in world.script_warps),
        "a scripted entrance to or from a cave needs its own binding and terrain check",
    )
    require(
        building_doors
        == {
            "MnGrandMaraisLighthouse",
            "MnR11SplitRockLighthouse",
            "MnMinneapolisBattleClub",
            "MnMinneapolisDispatchOffice",
        },
        "unclassified outdoor dungeon",
    )
    # Natural arch from Dark Cave, the well from Azalea, plus Minnesota's timber-framed Iron Range shaft.
    shapes = {world.square(*site) for site in refs}
    shapes.add(world.square("MnR09C00R00", 40, 41))
    require(len(shapes) == 3 and set(REFERENCES) <= refs, "missing original Crystal art or mine doorway")
    require(
        alternate_refs
        == {("Route23", 9, 13, "WARP_CARPET_DOWN"), ("Route23", 10, 13, "WARP_CARPET_DOWN")},
        "unclassified original Crystal alternative entrance",
    )
    images, permissions = world.scan_art(shapes)
    expected_images = {(n, 2 * x, 2 * y) for n, x, y in exterior_portals}
    require(
        images == expected_images,
        f"unbound or missing visible aperture (8px coordinates): {images ^ expected_images}",
    )
    require(
        permissions == exterior_portals,
        f"unbound cave permission or changed story aperture: {permissions ^ exterior_portals}",
    )
    for site in REFERENCES:
        cliff_geometry(world, *site)
    for site in sorted(exterior_portals):
        cliff_geometry(world, *site)
    mine_frame(world)
    cave_rooms = {n for n in regional if env[n] == "CAVE"}
    exterior = {n for n in regional if env[n] in OUTSIDE}
    print(
        f"{len(regional)} Minnesota rooms, {len(exterior)} exterior maps, {len(cave_rooms)} caves, "
        f"{inspected} warp records, {len(world.script_warps)} scripted warps and {world.commands} changeblock commands. "
        f"Scanned natural arches, the mine shaft and wells in all map, script and field-move block states."
    )
    print(
        f"{len(images)} bound outdoor apertures; {sum(cave_warps[e, 'CAVE'] for e in OUTSIDE)} outward bindings, "
        f"{sum(cave_warps['CAVE', e] for e in OUTSIDE)} returns, {cave_warps['CAVE', 'CAVE']} cave-to-cave links, "
        f"{cave_warps['CAVE', 'DUNGEON']}/{cave_warps['DUNGEON', 'CAVE']} cave/dungeon links. "
        f"Four outdoor dungeon destinations are buildings. Indoor cave-side warp roles: {dict(sorted(interiors.items()))}."
    )
    print(
        "Compared both height courses and connected rock across the flanks and cap with six actual Crystal entrances."
    )


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("parent", type=Path)
    parser.add_argument("candidate", type=Path)
    args = parser.parse_args()
    parents = {
        records(PREVIOUS, branch + "-build")["expected_pack_sha256"]: block
        for branch, block in (("field", 0x89), ("mothlight", 0x88))
    }
    digest = hashlib.sha256(args.parent.read_bytes()).hexdigest()
    require(
        digest in parents,
        "expected the selected northern-face parent",
    )
    parent, child = (archive["load_pack"](path) for path in (args.parent, args.candidate))
    changes = game_delta(parent, child, parents[digest])
    discovery(child)
    caves["check_caves"](child)
    walks = public_approaches(child)
    brown["route_connections"](child)
    print(
        f"Verified {walks} two-way public approaches, including the Route 1 fossil stair, and {changes} placements. "
        "Unrelated gameplay, graphics and audio match the selected parent."
    )


if __name__ == "__main__":
    main()
