#!/usr/bin/env python3
"""Measure permitted trips, unpaid access, sign bearings and real capture surfaces."""

import argparse
import json
import runpy
import sys
from collections import Counter, deque
from copy import deepcopy
from pathlib import Path

HERE = Path(__file__).resolve().parent
TOOLS = HERE.parents[1] / "tools"
sys.path.insert(0, str(TOOLS))
from audit_story_flow import native_pages, require
from render_reference import load_pack
from render_safari_runtime import inspect_boundaries

HARBOR = runpy.run_path(str(HERE.parent / "lake-observations-v01/check-harbor.py"))
LAYOUT = (
    HERE.parent
    / "route-grass-v01/current-boundary-safari-successor-v01/current-native-v02.layout.json"
)
CAMP = "MnBoundaryWaters"
PINE = "MnPreservePineEntry"
CABIN = "MnBoundaryRootBeerCabin"
COURT = "MnBoundarySafariC00R02"
GRANITE = "MnBoundarySafariC01R00"
ISLE = "MnBoundarySafariC01R01"
DIRECTIONS = ((1, 0), (-1, 0), (0, 1), (0, -1))
DRY = {"FLOOR", "TALL_GRASS", "UP_WALL"}


def adj(pos):
    x, y = pos
    return [(x + dx, y + dy) for dx, dy in DIRECTIONS]


def pages(room, label):
    native = room["script_text_bodies"][label]["commands"]
    require(
        [(op["command"], op["args"]) for op in native]
        == [(op["command"], op["args"]) for op in room["scripts"][label]],
        f"stale pages {label}",
    )
    return " ".join(line for pair in native_pages(native) for line in pair).lower()


class Preserve:
    def __init__(self, data):
        self.data, self.zone = data, data["safari_zone"]
        self.rooms = {name: data["maps"][name] for name in self.zone["maps"]}
        self.tiles, self.actors, self.signs, self.origins = {}, {}, {}, {}
        self.squares = HARBOR["Trail"](data, set(self.rooms))
        for name, room in self.rooms.items():
            origin = self.zone["maps"][name]["origin_core"]
            if origin is None:
                require(name == CABIN, "unexpected seventh Safari map")
                continue
            ox, oy = self.origins[name] = tuple(origin)
            for y, row in enumerate(self.squares.grid[name]):
                for x, token in enumerate(row):
                    pos = ox + x, oy + y
                    require(pos not in self.tiles, "overlapping chunks")
                    self.tiles[pos] = token
            for actor in room["objects"]:
                self.actors[actor["object_identifier"]] = (
                    ox + actor["x"],
                    oy + actor["y"],
                )
            for sign in room["events"]["bg_events"]:
                self.signs[sign["script"]] = ox + sign["x"], oy + sign["y"]
        for name, (ox, oy) in self.origins.items():
            room = self.rooms[name]
            width, height = (
                2 * room["attributes"]["width"],
                2 * room["attributes"]["height"],
            )
            for side, origin in [
                ("east", (ox + width, oy)),
                ("south", (ox, oy + height)),
            ]:
                neighbors = [k for k, v in self.origins.items() if v == origin]
                if neighbors:
                    link = self.squares.conns[name][side]
                    opposite = {"east": "west", "south": "north"}[side]
                    require(
                        link["target_map"] == neighbors[0]
                        and link["offset"] == 0
                        and self.squares.conns[neighbors[0]][opposite]["target_map"]
                        == name,
                        "seam does not match habitat positions",
                    )
        self.bodies = set(self.actors.values())
        self.guards = {
            self.actors[k] for k in self.zone["gate_court"]["guard_identifiers"]
        }
        e = self.zone["entry"]
        ox, oy = self.origins[e["map_name"]]
        self.start = ox + e["tile"]["x"], oy + e["tile"]["y"]

    def distances(self, start, *, surface=None, paid=True, cut=False):
        allowed = (
            {"WATER"}
            if surface == "water"
            else DRY
            if surface == "land"
            else DRY | {"WATER"}
        )
        blocks = self.bodies - self.guards if paid else self.bodies
        distances = {start: 0}
        todo = deque([start])
        while todo:
            x, y = todo.popleft()
            current = self.tiles[x, y]
            for target in adj((x, y)):
                token = self.tiles.get(target)
                if cut and target == (10, 31):
                    token = "FLOOR"
                if token not in allowed or target in blocks or target in distances:
                    continue
                if (
                    current == "UP_WALL"
                    and target[1] < y
                    or token == "UP_WALL"
                    and target[1] > y
                ):
                    continue
                distances[target] = distances[x, y] + 1
                todo.append(target)
        return distances


def public_access(data, p, baseline):
    z = p.zone
    maps = data["maps"]
    court = maps[COURT]
    board = pages(court, "BWS_REED_RETURN_RANGERText")
    require(
        ("take you back" in board)
        if baseline
        else (
            "south" in board and "basecamp" in board and "take you back" not in board
        ),
        "unpaid Reed Landing instructions",
    )
    require(
        z["admission_map"] == CAMP
        and z["admission_actor_core"] == [30, 9]
        and z["return_to"]
        == {"map_name": CAMP, "tile": {"x": 30, "y": 10}, "facing": "up"},
        "admission or safe return moved",
    )
    require(
        set(z["retire_scripts"])
        == {
            "BWS_REED_RETURN_RANGER",
            "BWS_GRANITE_RETURN_RANGER",
            "BWS_QUIET_RETURN_RANGER",
        },
        "paid retirement roles",
    )
    require(
        set(z["maps"]).isdisjoint({CAMP, PINE, HARBOR["SHELTER"], HARBOR["INTAKE"]}),
        "story area incorrectly requires a permit",
    )
    modules = {
        (ox // 64, oy // 48): (name, maps[name]) for name, (ox, oy) in p.origins.items()
    }
    result = inspect_boundaries({"data": data}, modules)
    closed = p.distances((14, 111), paid=False)
    require(
        len(closed) == result["closed_court_reachable_tiles"] == 54
        and (10, 104) in closed
        and (3, 89) not in closed,
        "closed court must reach the ranger but not the water",
    )
    travel = HARBOR["ShorePaths"](data)
    extra = HARBOR["Trail"](data, {COURT, PINE})
    for key in ["maps", "grid", "bodies", "conns"]:
        getattr(travel, key).update(getattr(extra, key))
    reached = travel.reach((COURT, 14, 15))
    require(
        {(CAMP, 30, 10), (HARBOR["SHELTER"], 8, 5), (HARBOR["SHELTER"], 12, 4)}
        <= reached,
        "unpaid shelter, radio or check-in blocked",
    )
    flags = HARBOR["road_flag_tests"](data, False)
    HARBOR["intake_returns"](data, flags)
    HARBOR["routes"](data)
    print(
        "Free court, shelter and complete interrupted/resolved rescue checked. Three paid return rangers; 54 unpaid court squares."
    )


def record_destinations(p, cut, layout):
    reach = p.distances(p.start, cut=cut)
    goals = {}
    for sign, position in p.signs.items():
        faces = [at for at in adj(position) if at in reach and p.tiles[at] in DRY]
        require(faces, f"unreadable sign {sign}")
        goals[sign] = min(faces, key=reach.get)
    for name in p.zone["retire_scripts"]:
        goals[name] = min(
            (pt for pt in adj(p.actors[name]) if pt in reach and p.tiles[pt] in DRY),
            key=reach.get,
        )
    goals["Cabin doorstep"] = (71, 56)
    for camp in layout["campsites"]:
        goals[camp["name"]] = tuple(camp["center"])
    if cut:
        goals["Thicket grass"] = (9, 35)
    for label, point in goals.items():
        require(
            point in reach and reach[point] < 500,
            f"{label} not reachable on one permit",
        )
    print(
        "Metered travel to named stops:",
        ", ".join(
            f"{k} {reach[v]}" for k, v in goals.items() if not k.startswith("BWS_")
        ),
    )
    return goals, reach


def circuit(p, goals):
    # Four camps, three named overlooks and the cabin in one paid outing.
    itinerary = [
        "Reed Landing Camp",
        "BWS_PALISADE_VIEW",
        "Pine Portage Camp",
        "BWS_NORTHWIND_VIEW",
        "Granite Bay Camp",
        "Cabin doorstep",
        "BWS_EAGLE_POINT_VIEW",
        "Quiet Island Camp",
        "BWS_QUIET_RETURN_RANGER",
    ]
    total = 0
    previous = p.start
    for name in itinerary:
        target = goals[name]
        total += p.distances(previous)[target]
        previous = target
    # From outside (71,56) enter (71,55), arrive at (5,6), then reach
    # Dorothy on the north side of the cabin and walk back out through (5,7).
    room = p.rooms[CABIN]
    door = p.rooms[ISLE]
    doorwarp = next(w for w in door["events"]["warps"] if (w["x"], w["y"]) == (7, 7))
    arrival = room["events"]["warps"][doorwarp["target_warp_id"] - 1]
    require(
        (arrival["x"], arrival["y"]) == (5, 6)
        and p.tiles[71, 55] == "DOOR"
        and p.squares.kind(CABIN, 5, 6) == "FLOOR"
        and p.squares.kind(CABIN, 5, 7) == "WARP_CARPET_DOWN",
        "cabin active door and passive interior return",
    )
    carpet = room["events"]["warps"][0]
    outside = door["events"]["warps"][carpet["target_warp_id"] - 1]
    require(
        carpet["target_map_constant"] == door["attributes"]["map_constant"]
        and (outside["x"], outside["y"]) == (7, 8),
        "cabin walk-out return",
    )
    for y in (4, 5, 6):
        require(
            p.squares.kind(CABIN, 5, y) == "FLOOR"
            and (5, y) not in p.squares.bodies[CABIN],
            "cabin aisle occupied",
        )
    total += 6  # door, two steps up to greet, three back to the active exit
    print(
        "Combined discovery circuit, ending with Quiet Island ranger:",
        total,
        "of 500 travel steps",
    )
    require(total < 500, "combined discovery circuit exhausts permit")


def terrain(data, p, layout, baseline):
    old_drop = (107, 33)
    stairs = (111, 33)
    sign = p.signs["BWS_SIGN_05"]
    ranger = p.actors["BWS_GRANITE_RETURN_RANGER"]
    shoreline = (119, 43)
    require(
        p.tiles[old_drop] == "WALL"
        and not any(t.startswith("HOP_") for t in p.tiles.values()),
        "reaudit the named local descent",
    )
    require(
        stairs[0] < sign[0] < ranger[0]
        and stairs[1] < sign[1]
        and ranger[1] < sign[1] < shoreline[1],
        "Granite sign bearings",
    )
    walk = p.distances((sign[0], sign[1] + 1), surface="land")
    require(
        {stairs, (111, 31), (111, 34), shoreline} <= walk.keys()
        and p.tiles[(120, 43)] == "WATER",
        "Granite staircase does not reach the dock",
    )
    granite_text = pages(p.rooms[GRANITE], "BWS_SIGN_05Text")
    require(
        "drop" in granite_text
        if baseline
        else (
            "stairs" in granite_text
            and "northwest" in granite_text
            and "northeast" in granite_text
            and "drop" not in granite_text
        ),
        "Granite sign must name its real descent",
    )
    for course in layout["portages"]:
        a, b, wa, wb = (
            tuple(course[k])
            for k in ("west_land", "east_land", "west_water", "east_water")
        )
        require(
            wb in adj(b) and wa in adj(a) and {p.tiles[wa], p.tiles[wb]} == {"WATER"},
            f"portage landings {course['id']}",
        )
        forward = p.distances(a, surface="land")[b]
        reverse = p.distances(b, surface="land")[a]
        paddle = p.distances(wa, surface="water").get(wb)
        require(
            paddle is None or max(forward, reverse) + 2 < paddle,
            f"paid walking shortcut {course['id']}",
        )
        print(
            course["id"],
            "walk both directions",
            forward,
            reverse,
            "paddle",
            paddle or "separate lake",
        )
    base = p.distances(p.start)
    cut = p.distances(p.start, cut=True)
    require(
        (10, 30) in base
        and p.tiles[10, 31] == "CUT_TREE"
        and (9, 35) not in base
        and (9, 35) in cut,
        "thicket needs its actual tree",
    )
    replacement = data["field_moves"]["cut"]["replacements"]["mn_boundary_safari"][0x63]
    require(
        replacement["replacement_block_id"] == 0x3F
        and data["tilesets"]["mn_boundary_safari"]["collision"]["3f"][2] == "FLOOR",
        "temporary native Cut replacement",
    )
    species = set()
    weights = {}
    for habitat in p.zone["habitats"]:
        (x0, y0), (x1, y1) = habitat["bounds_core"]
        inside = {
            at: t for at, t in p.tiles.items() if x0 <= at[0] < x1 and y0 <= at[1] < y1
        }
        if habitat["id"] == "PINE_THICKET":
            grass = {at for at, token in inside.items() if token == "TALL_GRASS"}
            require(
                len(grass) == 52
                and not grass.intersection(base)
                and grass <= cut.keys()
                and "WATER" not in inside.values(),
                "Cut must give access to every thicket grass square and no spurious water table",
            )
        for token, surface in [("TALL_GRASS", "grass"), ("WATER", "water")]:
            if token not in inside.values():
                continue
            sites = [cut[at] for at, t in inside.items() if t == token and at in cut]
            require(sites and min(sites) < 500, "unavailable habitat")
            table = {entry["species"]: entry["weight"] for entry in habitat[surface]}
            require(sum(table.values()) == 100, "invalid Safari percentages")
            species.update(table)
            weights[habitat["id"], surface] = table
    pokemon_dir = HERE.parents[1] / "pokemon"
    roster = json.loads((pokemon_dir / "roster.json").read_text())
    curation = json.loads((pokemon_dir / "roster-curation.json").read_text())
    retired = {family["id"] for family in curation["retired_families"]}
    active = {
        m["name"].upper()
        for family in roster["families"]
        if family["id"] not in retired
        for m in family["members"]
    }
    require(species <= active, f"inactive Safari species: {sorted(species - active)}")
    require(
        len(species) == 43
        and weights["PINE_THICKET", "grass"]["CROOKVEIL"] == 45
        and weights["PINE_THICKET", "grass"]["LATCHQUILL"] == 30
        and weights["QUIET_WATER", "water"]["LATCHUSK"] == 3,
        "encounter clues need review",
    )
    require(
        len(p.zone["habitats"]) == 7
        and len(p.origins) == 6
        and Counter(p.tiles.values())["TALL_GRASS"] == 316,
        "habitat coverage changed",
    )
    print(
        "Seven habitats, 43 distinct grass/swimming species; 52 Cut-gated grass squares, 75% Crookveil or Latchquill conditional on thicket encounter."
    )


def cabin_and_dialogue(pack, baseline):
    data = pack["data"]
    rooms = data["maps"]
    inside = rooms[CABIN]
    yard = rooms[ISLE]
    indoor = "MN_INTERIOR_DOROTHY"
    label = indoor + "InteriorText"
    role = "SPRITE_GRANNY" if baseline else "SPRITE_NURSE"
    (healer,) = [o for o in inside["objects"] if o["script"] == indoor]
    (host,) = [
        o for o in yard["objects"] if o["object_identifier"] == "BWS_ROOT_BEER_LADY"
    ]
    require(
        (healer["x"], healer["y"], healer["sprite"]) == (5, 3, role)
        and (host["x"], host["y"], host["sprite"]) == (5, 11, "SPRITE_GRANNY"),
        "indoor relief medic and outdoor Dorothy",
    )
    require(
        healer in data["npcs"][CABIN] and host in data["npcs"][ISLE],
        "split actor identity",
    )
    for catalog in (
        inside["map_event_section_commands"],
        data["map_scripts"][inside["attributes"]["map_events_label"]],
    ):
        (binding,) = [
            c
            for c in catalog
            if c["command"] == "object_event" and c["args"][11] == indoor
        ]
        require(binding["args"][:3] == ["5", "3", role], "indoor native event sprite")
    require(
        bytes(pack["runtime_files"]["gfx/sprites/granny.2bpp"])
        != bytes(pack["runtime_files"]["gfx/sprites/nurse.2bpp"]),
        "independent native nurse and Dorothy frames",
    )
    hf = HARBOR["rows"](inside, indoor)
    require(
        hf
        == [
            ("faceplayer",),
            ("opentext",),
            ("writetext", label),
            ("waitbutton",),
            ("special", "HealParty"),
            ("closetext",),
            ("end",),
        ],
        "cabin party healing",
    )
    introduction = pages(inside, label)
    require(
        "i'm dorothy" in introduction
        if baseline
        else ("i'm the medic" in introduction and "rest on the bench" in introduction),
        "cabin introduction",
    )
    sale = host["script"]
    require(
        (
            "i'm dorothy" in pages(yard, sale + "FirstText")
            and ("giveitem", "ROOT_BEER", "1") in HARBOR["rows"](yard, sale)
            and ("pokemart", "MARTTYPE_STANDARD", "MART_MN_ROOT_BEER")
            in HARBOR["rows"](yard, sale + "Shop")
            and ("special", "HealParty") in HARBOR["rows"](yard, sale + "Choices")
        ),
        "Dorothy’s original outdoor services",
    )
    canonical = json.loads((HERE.parents[1] / "story.json").read_text())[
        "dialogue_overhaul"
    ]["texts"]
    for name, text in [
        (CABIN, label),
        (GRANITE, "BWS_SIGN_05Text"),
        (COURT, "BWS_REED_RETURN_RANGERText"),
    ]:
        room = rooms[name]
        require(
            room["scripts"][text] == data["map_scripts"][text],
            "global sign/dialogue mirror",
        )
        if not baseline:
            require(
                native_pages(room["script_text_bodies"][text]["commands"])
                == canonical[name][text],
                f"authored versus installed pages {text}",
            )
    print(
        "Indoor healing, outdoor first/repeat root beer, and all three native dialogue mirrors verified."
    )


def sign_bearings(p):
    for label, origin, destination, text_label, compass in [
        ("Pocket Lake west", (26, 18), (21, 21), "BWS_SIGN_03", "west"),
        (
            "North Pine east launch",
            (68, 24),
            (72, 29),
            "BWS_SIGN_NORTH_PORTAGE",
            "east",
        ),
        (
            "North Pine west portage",
            (68, 24),
            (52, 25),
            "BWS_SIGN_NORTH_PORTAGE",
            "west",
        ),
        ("Isle west cabin", (82, 58), (71, 56), "BWS_SIGN_ISLE_PORTAGE", "west"),
        ("Isle east launch", (82, 58), (86, 61), "BWS_SIGN_ISLE_PORTAGE", "east"),
        (
            "Palisade north lake",
            (30, 46),
            (36, 41),
            "BWS_PALISADE_PORTAGE_SIGN",
            "north",
        ),
        (
            "Palisade south lake",
            (30, 46),
            (36, 68),
            "BWS_PALISADE_PORTAGE_SIGN",
            "south",
        ),
        (
            "Northwind west lake",
            (80, 34),
            (75, 33),
            "BWS_NORTHWIND_PORTAGE_SIGN",
            "west",
        ),
        (
            "Northwind east lake",
            (80, 34),
            (98, 33),
            "BWS_NORTHWIND_PORTAGE_SIGN",
            "east",
        ),
    ]:
        require(p.signs[text_label] == origin, f"{label}: sign coordinate")
        (room,) = [room for room in p.rooms.values() if text_label in room["scripts"]]
        require(compass in pages(room, text_label + "Text"), label)
        if compass in ("north", "south"):
            bearing = destination[1] - origin[1]
        else:
            bearing = destination[0] - origin[0]
        require(
            (bearing < 0) == (compass in ("north", "west")) and bearing != 0,
            f"{label}: bearing",
        )
        if "lake" in label.lower() or "launch" in label.lower():
            require(p.tiles[destination] == "WATER", label)


def preserve_parent(parent, child):
    before, after = parent["data"], child["data"]
    require(
        before["safari_zone"] == after["safari_zone"],
        "permit/habitat definitions changed",
    )
    labels = {
        CABIN: "MN_INTERIOR_DOROTHYInteriorText",
        GRANITE: "BWS_SIGN_05Text",
        COURT: "BWS_REED_RETURN_RANGERText",
    }
    for name in before["safari_zone"]["maps"]:
        old = deepcopy(before["maps"][name])
        new = after["maps"][name]
        if name in labels:
            key = labels[name]
            old["scripts"][key] = new["scripts"][key]
            old["script_text_bodies"][key] = new["script_text_bodies"][key]
        if name == CABIN:
            for actor in old["objects"]:
                if actor["script"] == "MN_INTERIOR_DOROTHY":
                    actor["sprite"] = "SPRITE_NURSE"
            for row in old["map_event_section_commands"]:
                if (
                    row["command"] == "object_event"
                    and row["args"][11] == "MN_INTERIOR_DOROTHY"
                ):
                    row["args"][2] = "SPRITE_NURSE"
        differences = [k for k in set(old) | set(new) if old.get(k) != new.get(k)]
        require(not differences, f"{name} additional map changes {differences}")
        bank = old["attributes"]["tileset_name"]
        require(
            before["tilesets"][bank] == after["tilesets"][bank],
            f"{bank} collision, tiles or animations changed",
        )
        for file, blob in parent["runtime_files"].items():
            if file.startswith(
                (
                    f"gfx/tilesets/{bank}.",
                    f"data/tilesets/{bank}_",
                    "gfx/sprites/granny.",
                    "gfx/sprites/nurse.",
                )
            ):
                require(
                    child["runtime_files"].get(file) == blob,
                    f"unexpected native artwork change in {file}",
                )
    print(
        "Seven complete Safari maps and their native banks preserved outside three text records and the single indoor character."
    )


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("pack", type=Path)
    parser.add_argument(
        "--baseline",
        action="store_true",
        help="verify the historical sign, ranger and duplicate Dorothy",
    )
    parser.add_argument(
        "--parent",
        type=Path,
        help="compare all seven Safari maps with the matching incoming edition",
    )
    args = parser.parse_args()
    pack = load_pack(args.pack)
    data = pack["data"]
    p = Preserve(data)
    layout = json.loads(LAYOUT.read_text())
    cabin_and_dialogue(pack, args.baseline)
    sign_bearings(p)
    public_access(data, p, args.baseline)
    record_destinations(p, False, layout)
    goals, _ = record_destinations(p, True, layout)
    circuit(p, goals)
    terrain(data, p, layout, args.baseline)
    if args.parent:
        preserve_parent(load_pack(args.parent), pack)


if __name__ == "__main__":
    main()
