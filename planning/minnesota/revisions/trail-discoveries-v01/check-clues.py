#!/usr/bin/env python3
"""Survey installed hidden-item approaches and test the two altered search landmarks."""

import argparse
from collections import Counter
import json
from pathlib import Path
import runpy
import sys
from PIL import Image, ImageDraw

HERE = Path(__file__).resolve().parent
REGION = HERE.parents[1]
sys.path.insert(0, str(REGION / "tools"))
from audit_story_flow import instructions, native_pages, read_branch, require
from render_reference import SourceArt, load_pack

hidden = runpy.run_path(str(HERE.parent / "lake-observations-v01/check-hidden.py"))
winter = runpy.run_path(str(HERE.parent / "northern-trails-v01/check-voyageurs.py"))
pixels = runpy.run_path(str(HERE.parent / "campaign-quality-v01/check-clues.py"))
GROVE, SOUTH, RIDGE = "MnR08C00R00", "MnR08C00R01", "MnR13C00R00"
SEED, TONIC = (35, 42), (38, 32)
BIRCH = "MN_R08_BIRCH_WALKER"
HINT = "MnDiscoveryR08ClueText"
SIGN_SITES = {(28, 12), (32, 24), (18, 40)}


def block(room, point):
    x, y = point
    return room["blocks"][y // 2 * room["attributes"]["width"] + x // 2]


def target(box, point):
    x, y = point
    return box.crop((x % 2 * 16, y % 2 * 16, x % 2 * 16 + 16, y % 2 * 16 + 16))


def verify_drawing(pack, name, point, source):
    """Decode each installed 2bpp pixel. A dot in the drawing inherits native ground."""
    [design] = json.loads(source.read_text())
    room = pack["data"]["maps"][name]
    bank = room["attributes"]["tileset_name"]
    require(bank == design["tileset"], f"{name}: drawing bank")
    files = pack["runtime_files"]
    layouts = bytes(files[f"data/tilesets/{bank}_metatiles.bin"])
    gfx = bytes(files[f"gfx/tilesets/{bank}.2bpp"])
    definition = pack["data"]["tilesets"][bank]
    installed = block(room, point)
    require(
        definition["collision"][f"{installed:02x}"]
        == definition["collision"][f"{design['collision_from']:02x}"],
        f"{name}: search permissions",
    )
    for j in range(16):
        original = layouts[16 * design["background"] + j]
        placed = layouts[16 * installed + j]
        baseline, actual = [
            pixels["levels"](gfx[offset * 16 : offset * 16 + 16])
            for offset in [
                pixels["address"](glyph, definition["palette_map"], len(gfx) // 16)
                for glyph in (original, placed)
            ]
        ]
        expected = baseline.copy()
        for k in range(64):
            ink = design["rows"][8 * (j // 4) + k // 8][8 * (j % 4) + k % 8]
            if ink != ".":
                expected[k] = int(ink)
        require(actual == expected, f"{name}/{point}: wrong pixels in glyph {j}")
    return installed


def legal_arrivals(data, items):
    """Exclude passive Floor-only warp records before proving each public approach."""
    maps = data["maps"]
    constants = {r["attributes"]["map_constant"]: n for n, r in maps.items()}
    seam_maps = {n: r | {"events": {"warps": []}} for n, r in maps.items()}
    entrances = hidden["anchors"](data | {"maps": seam_maps})
    roles = hidden["WARP"] | {"WARP_CARPET_LEFT", "WARP_CARPET_RIGHT"}
    for name, room in maps.items():
        if not name.startswith("Mn"):
            continue
        occupied = {
            hidden["coords"](o) for o in room["objects"] if o["event_flag"] == "-1"
        }
        for warp in room["events"]["warps"]:
            point = hidden["coords"](warp)
            if point in occupied or hidden["quadrant"](data, name, *point) not in roles:
                continue
            dest = constants.get(warp["target_map_constant"])
            if dest:
                entrances[dest].update(
                    hidden["coords"](w)
                    for w in maps[dest]["events"]["warps"]
                    if w["index"] == warp["target_warp_id"]
                )
    total = Counter()
    for name in {n for n, _ in items}:
        modes = ["walk", "Surf"] + (
            [hidden["MOVES"][name][0]] if name in hidden["MOVES"] else []
        )
        access = {
            mode: hidden["approaches"](data, name, entrances[name], mode)
            for mode in modes
        }
        for (n, script), (point, _, _) in items.items():
            if name == n:
                method = next(
                    (k for k, v in access.items() if hidden["adjacent"](point) & v),
                    None,
                )
                require(
                    method is not None,
                    f"{n}/{script} has no return to an active public entrance",
                )
                total[method] += 1
    print(
        "Using only active doors, carpets, ladders, cave mouths and public seams:",
        dict(total),
    )


def false_signs(pack, art, items):
    """Find solid searchable surfaces identical to an actual reading surface in the same room."""
    maps = pack["data"]["maps"]
    matches = {}
    for (n, _), (point, _, _) in items.items():
        r = maps[n]
        if hidden["quadrant"](pack["data"], n, *point) != "WALL":
            continue
        a = r["attributes"]
        atlas = art.blocks(
            a["tileset_name"],
            "nite" if a["environment"] in {"CAVE", "DUNGEON"} else "day",
        )
        tile = target(atlas[block(r, point)], point).tobytes()
        signs = {
            hidden["coords"](e)
            for e in r["events"]["bg_events"]
            if e["event_type"] == "BGEVENT_READ"
        }
        twins = {
            p
            for p in signs
            if hidden["quadrant"](pack["data"], n, *p) == "WALL"
            and target(atlas[block(r, p)], p).tobytes() == tile
        }
        if twins:
            matches[n, point] = twins
    return matches


def routes(data):
    names = {n for n in data["maps"] if n.startswith("MnR08C")} | {
        "MnLeechLake",
        "MnBemidji",
    }
    grove = winter["Trail"](data, names)
    cache = (SOUTH, 36, 42)
    require(
        grove.step(("MnLeechLake", 32, 0), "UP") == (SOUTH, 26, 47)
        and grove.both((SOUTH, 26, 47), cache) == (21, 21)
        and grove.both((GROVE, 39, 40), cache) == (93, 93),
        "Leech Lake to grove and Birch Walker to cones",
    )
    ridge = winter["Trail"](data, {RIDGE, "MnBoundaryWaters", "MnR13C00R01"})
    require(
        ridge.step(("MnBoundaryWaters", 30, 47), "DOWN") == (RIDGE, 30, 0)
        and ridge.both((RIDGE, 30, 0), (RIDGE, 38, 33)) == (41, 41),
        "Boundary Waters to Cedar Ridge cache",
    )
    print(
        "Leech Lake to cones 21 steps. Birch Walker to cones 93. Boundary Waters to the Cedar Ridge search square 41. All three walks return."
    )


def prairie_boards(pack, art, baseline):
    data = pack["data"]
    name = "MnR02C00R00"
    road = data["maps"][name]
    attr = road["attributes"]
    signs = json.loads((REGION / "routes/MN_R02.json").read_text())["signs"]
    trail = winter["Trail"](data, {name, "MnMankato", "MnRochester"})
    east = (name, 59, 14)
    west = (name, 0, 30)
    require(
        trail.step(east, "RIGHT") == ("MnRochester", 0, 18)
        and trail.step(west, "LEFT") == ("MnMankato", 47, 18),
        "Prairie Farm Road public entrances",
    )
    hidden_pos = (18, 34)
    layout = art.blocks(attr["tileset_name"], "day")
    fence = target(layout[block(road, hidden_pos)], hidden_pos).tobytes()
    section = road["attributes"]["map_events_label"]
    wire = data["map_scripts"][section]
    require(
        len(signs) == 5 and len({s["id"] for s in signs}) == 5,
        "five distinct prairie board scripts",
    )
    distances = []
    for sign in signs:
        x, y = sign["tile"]
        left = (x - 1, y)
        face = (x, y)
        offset = 1 if sign["id"] != "MN_R02_WEST_BOARD" else -1
        stand = (name, x, y + offset)
        require(
            block(road, left) == block(road, face) == 0x56
            and hidden["quadrant"](data, name, *face) == "WALL"
            and target(layout[0x56], left).tobytes() == fence
            and target(layout[0x56], face).tobytes() != fence,
            f"posts and printed front on {sign['id']}",
        )
        [read] = [e for e in road["events"]["bg_events"] if e["script"] == sign["id"]]
        pos = hidden["coords"](read)
        require(
            pos == (left if baseline else face)
            and read["event_type"] == "BGEVENT_READ",
            sign["id"],
        )
        require(
            [
                e["script"]
                for e in road["events"]["bg_events"]
                if hidden["coords"](e) == face
            ]
            == ([] if baseline else [sign["id"]]),
            "printed board face registration",
        )
        args = [*map(str, pos), "BGEVENT_READ", sign["id"]]
        for stream in (wire, road["map_event_section_commands"]):
            require(
                sum(
                    row["command"] == "bg_event" and row["args"] == args
                    for row in stream
                )
                == 1,
                f"{sign['id']} native event stream",
            )
        script = sign["id"] + "Text"
        require(
            instructions(road, sign["id"]) == [("jumptext", script)]
            and data["map_scripts"][script] == road["scripts"][script],
            f"directions on {sign['id']}",
        )
        nearest = min(
            (trail.both(port, stand) for port in (west, east)), key=lambda pair: pair[0]
        )
        distances.append(nearest)
    require(
        distances == [(2, 2), (39, 39), (14, 14), (36, 36), (7, 7)],
        f"actual reading positions: {distances}",
    )
    seen = []
    expected = {
        "MN_R02_T01": ((51, 21), "RIGHT", 1),
        "MN_R02_T02": ((23, 19), "UP", 2),
        "MN_R02_T03": ((47, 37), "UP", 2),
    }
    trainers = [o for o in road["objects"] if o["object_type"] == "OBJECTTYPE_TRAINER"]
    require(len(trainers) == len(expected), "prairie regular trainers")
    for actor in trainers:
        xy, heading, radius = expected[actor["script"]]
        require(
            hidden["coords"](actor) == xy
            and actor["sightline_direction_override"] == heading
            and actor["radius"] == radius,
            actor["script"],
        )
        dx, dy = winter["DIRECTIONS"][heading]
        ray = [(name, xy[0] + dx * i, xy[1] + dy * i) for i in range(1, radius + 1)]
        for target_square in ray:
            require(
                trail.kind(*target_square) in winter["WALK"]
                and target_square[1:] not in trail.bodies[name],
                f"blocked trainer view {target_square}",
            )
            require(
                trail.both(east, target_square)[0] > 0,
                f"unreachable trainer sight square {target_square}",
            )
            seen.append(target_square[1:])
    print(
        "Five printed Route 2 boards with reachable reading positions:",
        distances,
        ". Trainer sight squares:",
        seen,
    )


def compare_parent(parent, child):
    before, after = parent["data"], child["data"]
    require(
        pixels["pickups"](parent) == pixels["pickups"](child),
        "a hidden item, flag or coordinate changed",
    )
    name = "MnR02C00R00"
    previous, new = before["maps"][name], after["maps"][name]
    require(
        previous["blocks"] == new["blocks"]
        and previous["attributes"] == new["attributes"]
        and previous["objects"] == new["objects"],
        "Route 2 terrain, actors, or sightlines changed",
    )
    require(
        previous["events"] | {"bg_events": new["events"]["bg_events"]} == new["events"],
        "unrelated prairie events changed",
    )
    old_events = previous["events"]["bg_events"]
    new_events = new["events"]["bg_events"]
    require(len(old_events) == len(new_events), "prairie event count")
    for old, event in zip(old_events, new_events):
        wanted = (
            old | {"x": old["x"] + 1} if old["event_type"] == "BGEVENT_READ" else old
        )
        require(event == wanted, f"{name} changed the wrong event")
    bank = "mn_outdoor_winter_stairs"
    old = SourceArt(parent)
    actual = SourceArt(child)
    for clock in ["morn", "day", "nite"]:
        perm = old.blocks(bank, clock)
        revised = actual.blocks(bank, clock)
        require(
            len(revised) == len(perm) + 1
            and all(one.tobytes() == two.tobytes() for one, two in zip(perm, revised)),
            f"old {clock} winter art changed",
        )
    print(
        "All original item receipts, Route 2 actors and sightlines, and all previously addressable winter terrain pixels match the parent."
    )


def gallery(pack, art, items, directory):
    """Native terrain centered on each event; labels give the searched square and item."""
    directory.mkdir(parents=True, exist_ok=True)
    cases = sorted((name, p, token) for (name, _), (p, token, _) in items.items())
    for offset in range(0, len(cases), 15):
        batch = cases[offset : offset + 15]
        image = Image.new("RGB", (5 * 166, ((len(batch) + 4) // 5) * 176), "#1a2428")
        pen = ImageDraw.Draw(image)
        for i, (name, (cx, cy), token) in enumerate(batch):
            r = pack["data"]["maps"][name]
            a = r["attributes"]
            atlas = art.blocks(
                a["tileset_name"],
                "nite" if a["environment"] in {"CAVE", "DUNGEON"} else "day",
            )
            screen = Image.new("RGB", (160, 144))
            for y in range(max(0, cy // 2 - 3), min(a["height"], cy // 2 + 4)):
                for x in range(max(0, cx // 2 - 3), min(a["width"], cx // 2 + 4)):
                    screen.paste(
                        atlas[r["blocks"][y * a["width"] + x]],
                        (x * 32 - cx * 16 + 72, y * 32 - cy * 16 + 64),
                    )
            x, y = i % 5 * 166, i // 5 * 176
            pen.text((x + 2, y + 1), name[2:], fill="white")
            pen.text((x + 2, y + 14), f"{token} ({cx},{cy})", fill="#dce8b5")
            image.paste(screen, (x + 2, y + 30))
        image.save(directory / f"sites-{offset}.png")


def audit(pack, baseline, directory):
    data = pack["data"]
    items = hidden["audit"](pack, True)
    require(
        len(items) == 57 and len({n for n, _ in items}) == 47,
        "hidden-item registry drift",
    )
    legal_arrivals(data, items)
    art = SourceArt(pack)
    duplicates = false_signs(pack, art, items)
    # Only the local read-sign matches are a semantic assertion. Other labels can
    # intentionally mark the same species of tree, rock, or shoreline surface.
    require(
        duplicates.get((RIDGE, TONIC), set()) == (SIGN_SITES if baseline else set()),
        "Cedar Ridge sign identity",
    )
    print("Solid hidden squares visually identical to a nearby read event:", duplicates)
    grove = data["maps"][GROVE]
    flag = items[SOUTH, "MN_HIDDEN_R08"][2]
    [walker] = [a for a in grove["objects"] if a["script"] == BIRCH]
    require(
        hidden["coords"](walker) == (39, 39)
        and instructions(grove, BIRCH)[0] == ("checkevent", flag),
        "Birch Walker clue gate",
    )
    for flags, expected in [(set(), HINT), ({flag}, BIRCH + "Text")]:
        _, labels, end = read_branch(grove, BIRCH, flags)
        require(
            labels == [expected] and end == flags,
            "Birch Walker first visit/claimed state",
        )
    old = [
        ["The thawed grove", "got flowers first."],
        ["There's a seed in", "that bed bigger"],
        ["than any of the", "flowers."],
    ]
    current = json.loads((REGION / "story.json").read_text())["story_flow_revision"][
        "dialogue"
    ][GROVE][HINT]
    rows = grove["scripts"][HINT]
    require(
        native_pages(rows) == (old if baseline else current)
        and data["map_scripts"][HINT] == rows
        and [dict(r, command_index=i) for i, r in enumerate(rows)]
        == grove["script_text_bodies"][HINT]["commands"],
        "Birch Walker's installed pages differ",
    )
    require(
        items[SOUTH, "MN_HIDDEN_R08"][:2] == (SEED, "MIRACLE_SEED")
        and items[RIDGE, "MN_HIDDEN_R13"][:2] == (TONIC, "HYPER_POTION"),
        "original cache locations",
    )
    verify_drawing(
        pack, SOUTH, SEED, REGION / "discoveries/route-07-11-clues-v01/r08-art.json"
    )
    if not baseline:
        placed = verify_drawing(pack, RIDGE, TONIC, HERE / "cedar-cairn.json")
        ridge = data["maps"][RIDGE]
        require(
            ridge["blocks"].count(placed) == 1
            and all(block(ridge, p) == 0x88 for p in SIGN_SITES),
            "cairn changed another sign",
        )
        print(
            "The original fallen cones and the two balanced stones match every authored two-bit pixel."
        )
    print(
        "Unclaimed Birch Walker:", " / ".join(" ".join(p) for p in native_pages(rows))
    )
    routes(data)
    prairie_boards(pack, art, baseline)
    print(
        "Environment totals:",
        dict(Counter(data["maps"][n]["attributes"]["environment"] for n, _ in items)),
    )
    if directory:
        gallery(pack, art, items, directory)
    return items


if __name__ == "__main__":
    cli = argparse.ArgumentParser(description=__doc__)
    cli.add_argument("pack", type=Path)
    cli.add_argument(
        "--baseline",
        action="store_true",
        help="verify the flower clue and the original unreadable sign",
    )
    cli.add_argument(
        "--gallery",
        type=Path,
        help="write native terrain crops for every hidden square",
    )
    cli.add_argument(
        "--parent",
        type=Path,
        help="compare pickup identities, the five noticeboard events and all existing winter art",
    )
    args = cli.parse_args()
    pack = load_pack(args.pack)
    audit(pack, args.baseline, args.gallery)
    if args.parent:
        compare_parent(load_pack(args.parent), pack)
