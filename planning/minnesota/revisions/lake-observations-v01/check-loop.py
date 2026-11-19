#!/usr/bin/env python3
"""Audit the Route 19 field notes, both public entrances and the ranger's reward from compiled archives."""

import argparse
import base64
from collections import deque
from functools import cache
from itertools import permutations, product
import json
from pathlib import Path
import runpy
import sys

HERE = Path(__file__).resolve().parent
REGION = HERE.parents[1]
sys.path.insert(0, str(REGION / "tools"))
from audit_story_flow import native_pages, require
from render_reference import SourceArt, load_pack

support = runpy.run_path(str(HERE.parent / "floodwatch-v01/check-moorhead.py"))
Trail, DIRS, talk = support["Trail"], support["DIRECTIONS"], support["talk"]
ITASCA, NORTH, SOUTH = "MnLakeItasca", "MnR19C00R00", "MnR19C00R01"
GATE, CITY, CABIN = "MnR19DetroitGate", "MnDetroitLakes", "MnDetroitLakesRangerCabin"
MAPS = (ITASCA, NORTH, SOUTH, GATE, CITY, CABIN)
BADGE = "ENGINE_MINERALBADGE"
PAID = "EVENT_MN_LAKE_OBSERVATIONS_REWARDED"
RANGER = "MN_DETROIT_LAKE_RANGER"
# South-to-east order, starting at the cabin in Detroit.
NOTES = [
    ("PRAIRIE", (22, 44), "EVENT_MN_LAKE_OBSERVATION_03", "MnDetroitPrairieNoteText"),
    ("WETLAND", (40, 34), "EVENT_MN_LAKE_OBSERVATION_02", "MnDetroitWetlandNoteText"),
    ("PINE", (54, 14), "EVENT_MN_LAKE_OBSERVATION_01", "MnDetroitPineNoteText"),
]
FLOWERS = [(8, 22), (9, 22), (11, 23)]
RETIRED = (
    "MnR19GateCheck",
    "MnR19GateQualify",
    "MnR19GateDone",
    "MnR19GateStepSouth",
    "MnR19GateDeniedText",
)


def resolve(data, name):
    return name if name in data["maps"] else data["runtime_map_metadata"][name]["name"]


def walk(data, revised):
    trail = Trail(data, MAPS)
    rooms = data["maps"]
    # The Itasca road's two temporary wardens can be spoken to without the Lake Badge.
    wardens = [o for o in rooms[ITASCA]["objects"] if o["script"] == "MnCohesionItascaWest"]
    rows = rooms[ITASCA]["scripts"]["MnCohesionItascaWest"]
    require(
        {(w["x"], w["y"]) for w in wardens} == {(2, 28), (2, 29)}
        and [r["command"] for r in rows] == ["disappear", "disappear", "end"]
        and {r["args"][0] for r in rows[:-1]} == {w["object_identifier"] for w in wardens},
        "Itasca west wardens",
    )
    for w in wardens:
        trail.bodies[ITASCA].remove((w["x"], w["y"]))
    require(trail.kind(ITASCA, 3, 28) == "FLOOR", "public talking square for Itasca wardens")
    for y in (28, 29):
        west = trail.step((ITASCA, 0, y), "LEFT", dry=True)
        require(
            west == (NORTH, 63, y - 16) and trail.step(west, "RIGHT", True) == (ITASCA, 0, y),
            "two-way Itasca seam",
        )
    gate = rooms[GATE]
    if revised:
        require(
            not gate["events"]["coord_events"]
            and not any(k in gate["scripts"] or k in data["map_scripts"] for k in RETIRED),
            "invisible badge stop still armed",
        )
        event_label = gate["attributes"]["map_events_label"]
        section = [{"command": r["command"], "args": r["args"]} for r in gate["map_event_section_commands"]]
        require(
            data["map_scripts"][event_label] == gate["scripts"].get(event_label, section) == section
            and not any(r["command"] == "coord_event" for r in section),
            "old checkpoint survived in a split event table",
        )
    else:
        checks = gate["events"]["coord_events"]
        require(
            {(r["x"], r["y"], r["script_name"]) for r in checks}
            == {(4, 2, "MnR19GateCheck"), (5, 2, "MnR19GateCheck")},
            "unexpected parent checkpoint",
        )
        require(
            gate["scripts"]["MnR19GateCheck"][1]["args"] == ["UP", "MnR19GateQualify"]
            and gate["scripts"]["MnR19GateQualify"][0]["args"] == [BADGE],
            "missing original one-way denial",
        )
        trail.bodies[GATE].update([(4, 2), (5, 2)])
        require(trail.path((GATE, 4, 6), (GATE, 4, 1), True), "expected parent side-lane bypass")
        trail.bodies[GATE].difference_update([(4, 2), (5, 2)])
    require(all(trail.kind(GATE, x, 0) == "WALL" for x in [0, 1, 2, 3, 6, 7, 8, 9]), "gatehouse north wall")
    for lane in (4, 5):
        require(
            all(trail.step((GATE, lane, y), "UP", True) == (GATE, lane, y - 1) for y in range(6, 1, -1)),
            f"blocked gate lane {lane}",
        )

    portals = {}
    for n in MAPS:
        for w in rooms[n]["events"]["warps"]:
            dest = resolve(data, w["target_map"])
            if dest not in MAPS:
                continue
            (tile,) = [r for r in rooms[dest]["events"]["warps"] if r["index"] == w["target_warp_id"]]
            portals[(n, w["x"], w["y"])] = (dest, tile["x"], tile["y"])
    for source, expected in [
        ((CITY, 25, 3), (GATE, 4, 6)),
        ((GATE, 4, 0), (SOUTH, 25, 8)),
        ((GATE, 5, 0), (SOUTH, 25, 8)),
        ((SOUTH, 25, 9), (GATE, 4, 1)),
        ((SOUTH, 26, 9), (GATE, 4, 1)),
        ((GATE, 4, 7), (CITY, 25, 4)),
        ((GATE, 5, 7), (CITY, 25, 4)),
        ((CITY, 41, 27), (CABIN, 4, 6)),
        ((CABIN, 3, 7), (CITY, 41, 28)),
        ((CABIN, 4, 7), (CITY, 41, 28)),
    ]:
        require(
            portals.get(source) == expected and trail.kind(*expected) == "FLOOR", f"bad portal {source}"
        )

    def step(node, direction):
        n, x, y = node
        dx, dy = DIRS[direction]
        dest = (n, x + dx, y + dy)
        trigger = {"DOOR": "UP", "WARP_CARPET_DOWN": "DOWN"}.get(trail.kind(*dest))
        if dest in portals and trigger == direction:
            return portals[dest]
        return trail.move(node, direction, True)

    @cache
    def distances(node):
        require(
            trail.kind(*node) in support["support"]["WALK"] and node[1:] not in trail.bodies[node[0]],
            f"invalid starting square {node}",
        )
        found = {node: 0}
        q = deque([node])
        while q:
            at = q.popleft()
            for direction in DIRS:
                dest = step(at, direction)
                if dest and dest not in found:
                    found[dest] = found[at] + 1
                    q.append(dest)
        return found

    start, office = (ITASCA, 3, 28), (CABIN, 6, 4)
    cabin_doorstep = (CITY, 41, 28)
    require(office in distances(cabin_doorstep) and cabin_doorstep in distances(office), "cabin round trip")
    (ranger,) = [o for o in rooms[CABIN]["objects"] if o["script"] == RANGER]
    require(
        (ranger["x"], ranger["y"]) == (6, 3) and ranger["event_flag"] == "-1", "ranger must be available"
    )
    signals = {}
    for name, pos, _, _ in NOTES:
        (event,) = [
            s for s in rooms[NORTH]["events"]["bg_events"] if s["script"] == "MN_R19_OBSERVE_" + name
        ]
        require(
            (event["x"], event["y"], event["event_type"]) == (*pos, "BGEVENT_READ"), f"{name} sign binding"
        )
        around = [(NORTH, pos[0] + dx, pos[1] + dy) for dx, dy in DIRS.values()]
        reachable = [s for s in around if s in distances(start) and s in distances(office)]
        require(reachable, f"cannot read {name} from both towns")
        signals[name] = min(reachable, key=lambda s: (distances(start)[s] + distances(office)[s], s))
    for origin in (start, office):
        trips = []
        for order in permutations(signals):
            places = [origin, *[signals[name] for name in order], office]
            require(
                all(b in distances(a) and a in distances(b) for a, b in zip(places, places[1:])),
                f"missing return or inaccessible observation sequence {order}",
            )
            trips.append(sum(distances(a)[b] for a, b in zip(places, places[1:])))
        print(
            f"{origin}: all six reading orders and return; {min(trips)}–{max(trips)} moves to the ranger."
        )
    (sign,) = [r for r in rooms[CITY]["events"]["bg_events"] if r["script"] == "MN_DETROIT_NORTH_SIGN"]
    require((sign["x"], sign["y"]) == (26, 4) and (CITY, 25, 4) in distances(office), "Detroit north sign")
    print(f"Three native observation stands {signals}; two gatehouse lanes and both cabin return carpets.")


def check_scripts(data, revised):
    rooms = data["maps"]
    cabin = rooms[CABIN]
    trail = rooms[NORTH]
    bags = support["pockets"](data, "MAX_REPEL")
    canonical = json.loads((REGION / "story.json").read_text())["dialogue_overhaul"]["texts"]
    bits = [BADGE, PAID, *[flag for _, _, flag, _ in NOTES]]
    require(
        all(k in data["story_event_script_constants"]["global"] for k in bits),
        "unregistered observation flag",
    )
    for vector in product((False, True), repeat=len(bits)):
        flags = {key for key, bit in zip(bits, vector) if bit}
        for name, _, flag, _ in NOTES:
            new, _, text, gift = talk(trail, "MN_R19_OBSERVE_" + name, flags, [])
            require(
                new == flags | {flag} and text == ["MN_R19_OBSERVE_" + name + "Text"] and not gift,
                "reading changed another flag",
            )
        missing = [clue for _, _, flag, clue in NOTES if flag not in flags]
        for bag, fits in bags:
            heard = support["check_receipt"](
                cabin,
                RANGER,
                flags,
                bag,
                BADGE in flags and not missing and PAID not in flags,
                fits,
                "MAX_REPEL",
                PAID,
            )
            if PAID in flags:
                expected = ["MnDetroitNotesAfterText"]
            elif BADGE not in flags:
                expected = ["MnDetroitNotesBadgeText"]
            elif missing:
                expected = ["MnDetroitNotesMissingText", *(missing if revised else [])]
            else:
                expected = [
                    "MnDetroitNotesReviewText",
                    "MnDetroitNotesRewardText" if fits else "MnDetroitNotesFullText",
                ]
            require(heard == expected, f"incorrect notes at desk for {flags}: {heard}")
    for name, label in [
        (CITY, "MN_DETROIT_NORTH_SIGNText"),
        (CABIN, "MnDetroitNotesMissingText"),
        (CABIN, "MnDetroitNotesReviewText"),
        *[(CABIN, clue) for *_, clue in NOTES],
    ]:
        if not revised:
            continue
        r = rooms[name]
        rows = r["scripts"][label]
        pages = native_pages(rows)
        require(
            pages == canonical[name][label]
            and native_pages(r["script_text_bodies"][label]["commands"]) == pages
            and data["map_scripts"][label] == rows,
            f"{name} split text {label}",
        )
    print(
        "32 marker/badge/receipt subsets × 4 pocket states, each reading, retry, repeat and canonical pages."
    )


def preservation(old, new):
    for n in MAPS:
        for k in ("attributes", "objects", "trainer_scripts"):
            require(old["maps"][n][k] == new["maps"][n][k], f"{n}/{k} changed")
        expected = old["maps"][n]["blocks"].copy()
        if n == NORTH:
            for x, y in FLOWERS:
                idx = y * 32 + x
                require(expected[idx] == 2, f"ground before {x},{y}")
                expected[idx] = 4
            col = new["tilesets"][new["maps"][NORTH]["attributes"]["tileset_name"]]["collision"]
            require(
                col["02"] == col["04"] == ["FLOOR"] * 4,
                "flowers obstruct field or change encounter habitat",
            )
        require(
            new["maps"][n]["blocks"] == expected
            and bytes(expected)
            == base64.b64decode(new["map_blocks"][new["maps"][n]["attributes"]["blocks_label"]]),
            f"{n}: misplaced flower or stale split blocks",
        )
        events = old["maps"][n]["events"].copy()
        if n == GATE:
            events["coord_events"] = []
        require(events == new["maps"][n]["events"], f"{n} changed door, reading or unrelated event")
        require(old["wild_encounters"].get(n) == new["wild_encounters"].get(n), f"{n} wildlife")
    for i in range(1, 7):
        key = f"MN_R19_T{i:02d}"
        require(old["trainers"]["trainers"][key] == new["trainers"]["trainers"][key], f"changed {key}")
    bank = new["maps"][NORTH]["attributes"]["tileset_name"]
    require(old["tilesets"][bank] == new["tilesets"][bank], f"shared Route 19 art bank {bank} changed")


def verified_data(path):
    pack = load_pack(path)
    data = pack["data"]
    art = SourceArt(pack)
    route = data["maps"]["Route30"]
    width = route["attributes"]["width"]
    # Crystal plants the same flowers next to the sign at (3,21).
    require(route["blocks"][10 * width + 3] == 4, "original Route 30 flower reference")
    bank = data["maps"][NORTH]["attributes"]["tileset_name"]
    for light in ("morn", "day", "nite"):
        meadow = art.blocks(bank, light)
        original = art.blocks("johto", light)
        require(
            meadow[4].tobytes() == original[4].tobytes() != meadow[2].tobytes(),
            f"{light}: meadow flower pixels differ from the native reference or disappear into grass",
        )
    return data


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("parent", type=Path)
    parser.add_argument("child", type=Path, nargs="?")
    args = parser.parse_args()
    old = verified_data(args.parent)
    print(args.parent.name)
    walk(old, False)
    check_scripts(old, False)
    if args.child:
        new = verified_data(args.child)
        print(args.child.name)
        walk(new, True)
        check_scripts(new, True)
        preservation(old, new)
        print("Three flower placements retain all floor permissions and match native Route 30 art in all three outdoor palettes.")


if __name__ == "__main__":
    main()
