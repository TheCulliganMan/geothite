#!/usr/bin/env python3
"""Trace the public road, native trainer sights and two independent Moorhead rewards."""

import argparse
from collections import deque
from itertools import product
import json
from pathlib import Path
import runpy
import sys

HERE = Path(__file__).resolve().parent
REGION = HERE.parents[1]
sys.path.insert(0, str(REGION / "tools"))
from audit_story_flow import native_pages, require
from render_reference import load_pack

support = runpy.run_path(str(HERE.parent / "northern-trails-v01/check-voyageurs.py"))
Trail = support["Trail"]
DIRECTIONS = support["DIRECTIONS"]

BRAINERD, R15 = "MnBrainerd", "MnR15C00R00"
DETROIT, APPROACH = "MnDetroitLakes", "MnDetroitLakesApproach"
EAST, WEST = "MnR16C01R00", "MnR16C00R00"
TOWN, CLINIC = "MnMoorhead", "MnMoorheadPokecenter1F"
ROAD = (BRAINERD, R15, DETROIT, APPROACH, EAST, WEST, TOWN, CLINIC)
BORDER = (
    (BRAINERD, 0, 24, R15, 63, 24),
    (R15, 0, 24, DETROIT, 47, 18),
    (DETROIT, 0, 18, APPROACH, 47, 18),
    (APPROACH, 0, 18, EAST, 31, 24),
    (EAST, 0, 29, WEST, 63, 29),
    (WEST, 0, 24, TOWN, 31, 12),
)
HOST = "MN_MOORHEAD_LEVEE_HELPER"
BOARD = "MN_MOORHEAD_RIVER_BOARD"
KEEPER = "MN_MOORHEAD_GAUGE_KEEPER"
VOLUNTEER = "MN_MOORHEAD_CLINIC_VOLUNTEER"
READ = "EVENT_MN_MOORHEAD_GAUGE_READ"
KIT = "EVENT_MN_MOORHEAD_DRY_KIT_GIVEN"
RUN = "EVENT_MN_PRAIRIE_CIRCUIT_REWARDED"
BADGE = "ENGINE_MINERALBADGE"
RUNNERS = {"MN_R16_T01": "LYLE", "MN_R16_T04": "JUNE", "MN_R16_T09": "FINCH"}
VICTORIES = tuple("EVENT_MN_R16_TRAINER_" + key[-2:] for key in RUNNERS)


def object_at(data, map_name, script):
    (actor,) = [o for o in data["maps"][map_name]["objects"] if o["script"] == script]
    return actor


def paths(trail, start):
    require(
        trail.kind(*start) in support["WALK"] and start[1:] not in trail.bodies[start[0]],
        f"invalid starting square {start}",
    )
    found = {start: 0}
    queue = deque([start])
    while queue:
        at = queue.popleft()
        for direction in DIRECTIONS:
            step = trail.move(at, direction, dry=True)
            if step is not None and step not in found:
                found[step] = found[at] + 1
                queue.append(step)
    return found


def check_trail(data):
    trail = Trail(data, ROAD)
    rooms = data["maps"]
    require(set(trail.conns[TOWN]) == {"east"}, "unadvertised Moorhead road")
    for left, lx, ly, right, rx, ry in BORDER:
        for lane in (0, 1):
            a, b = (left, lx, ly + lane), (right, rx, ry + lane)
            require(
                trail.step(a, "LEFT", True) == b and trail.step(b, "RIGHT", True) == a,
                f"road seam {a} <-> {b}",
            )
    # The west-side Brainerd officers initially occupy both lanes and open them by talking.
    guards = [o for o in rooms[BRAINERD]["objects"] if o["script"] == "MnCohesionBrainerdWest"]
    require({(o["x"], o["y"]) for o in guards} == {(2, 24), (2, 25)}, "west gate positions")
    rows = rooms[BRAINERD]["scripts"]["MnCohesionBrainerdWest"]
    disappear = {r["args"][0] for r in rows if r["command"] == "disappear"}
    require(
        [r["command"] for r in rows] == ["disappear", "disappear", "end"]
        and disappear == {o["object_identifier"] for o in guards},
        "west gate opening",
    )
    origin = (BRAINERD, 4, 24)
    closed = paths(trail, origin)
    require(
        (BRAINERD, 3, 24) in closed and (R15, 63, 24) not in closed,
        "Brainerd gate must be reachable on the closed side",
    )
    for actor in guards:
        trail.bodies[BRAINERD].remove((actor["x"], actor["y"]))
    require(
        not any(rooms[n]["events"]["coord_events"] for n in ROAD[:-1]),
        "a road has acquired a coordinate gate; review its states",
    )
    stand = (TOWN, 20, 12)
    geometry = paths(trail, stand)
    for town, sx, sy, route, dx, dy in BORDER:
        for lane in (0, 1):
            require(
                (town, sx, sy + lane) in geometry and (route, dx, dy + lane) in geometry,
                f"disconnected lane {town} {route}",
            )
    require(
        (object_at(data, TOWN, HOST)["x"], object_at(data, TOWN, HOST)["y"]) == (20, 13)
        and stand in paths(trail, origin)
        and origin in geometry,
        "unreachable levee helper",
    )

    # Bridge decks genuinely join distinct banks, including the drainage bridge across the chunk seam.
    for x, bank in [
        (19, [(WEST, 19, 23), (WEST, 19, 24)]),
        (63, [(WEST, 63, 29), (WEST, 63, 30)]),
    ]:
        for square in bank:
            require(trail.kind(*square) == "BRIDGE_NS_DECK", f"missing deck at {square}")
            trail.bodies[WEST].add(square[1:])
        require(
            (EAST, 31, 24) not in paths(trail, (WEST, 0, 24)),
            f"unmarked bridge bypass at x{x}",
        )
        for square in bank:
            trail.bodies[WEST].remove(square[1:])

    for start in ((TOWN, 31, 12), (EAST, 31, 24), (DETROIT, 47, 18), origin):
        require(
            stand in paths(trail, start) and start in geometry,
            f"no public walk to and from {start}",
        )

    (board,) = [r for r in rooms[TOWN]["events"]["bg_events"] if r["script"] == BOARD]
    require(
        (board["x"], board["y"], board["event_type"]) == (12, 4, "BGEVENT_READ"),
        "levee marker",
    )
    require(
        (TOWN, 13, 4) in geometry and (TOWN, 17, 8) in geometry,
        "board or clinic approach missing",
    )
    warp = {(w["x"], w["y"]): w for w in rooms[TOWN]["events"]["warps"]}[(17, 7)]
    require(
        trail.kind(TOWN, 17, 7) == "DOOR"
        and warp["target_warp_id"] == 3
        and data["runtime_map_metadata"][warp["target_map"]]["name"] == CLINIC,
        "clinic entrance",
    )
    arrivals = {w["index"]: w for w in rooms[CLINIC]["events"]["warps"]}
    arrival, volunteer = (CLINIC, 3, 6), (CLINIC, 2, 4)
    require(
        (arrivals[3]["x"], arrivals[3]["y"]) == arrival[1:]
        and volunteer in paths(trail, arrival)
        and arrival in paths(trail, volunteer),
        "clinic visit",
    )
    for index, x in [(1, 3), (2, 4)]:
        w = arrivals[index]
        require(
            (w["x"], w["y"], w["target_warp_id"]) == (x, 7, 2)
            and data["runtime_map_metadata"][w["target_map"]]["name"] == TOWN
            and trail.kind(CLINIC, x, 7) == "WARP_CARPET_DOWN"
            and (CLINIC, x, 6) in paths(trail, volunteer),
            "clinic return carpet",
        )
    (outdoor,) = [w for w in rooms[TOWN]["events"]["warps"] if w["index"] == 2]
    require((outdoor["x"], outdoor["y"]) == (17, 8), "clinic exit landing")
    require(
        {(o["x"], o["y"]) for o in rooms[CLINIC]["objects"] if o["script"] == VOLUNTEER} == {(2, 3)},
        "wrong volunteer",
    )

    # While approaching one runner, stepping into any other undefeated trainer's sight would
    # start a different fight. Compute each ray against the actual objects before masking them.
    rays = {}
    for n in (R15, APPROACH, WEST, EAST):
        for actor in rooms[n]["objects"]:
            if actor["object_type"] != "OBJECTTYPE_TRAINER":
                continue
            key = actor["script"]
            require(
                actor["event_flag"] == "-1" and actor["radius"] > 0,
                f"{key}: hidden or blind",
            )
            dx, dy = DIRECTIONS[actor["sightline_direction_override"]]
            x, y = actor["x"], actor["y"]
            ray = []
            for k in range(1, actor["radius"] + 1):
                square = (n, x + k * dx, y + k * dy)
                if square[1:] in trail.bodies[n] or trail.kind(*square) not in support["WALK"]:
                    break
                ray.append(square)
            rays[key] = ray
    # Readable guidance is checked at the bound event, from an adjacent square on this road.
    town_plan = json.loads((REGION / "towns/MnMoorhead.json").read_text())
    for contact in [*town_plan["signs"], *town_plan["residents"]]:
        x, y = contact["tile"]
        sx, sy = contact["approach"]
        key = contact["id"]
        exists = any(
            o["script"] == key and (o["x"], o["y"]) == (x, y)
            for o in [*rooms[TOWN]["events"]["bg_events"], *rooms[TOWN]["objects"]]
        )
        require(
            exists and (TOWN, sx, sy) in geometry and abs(x - sx) + abs(y - sy) == 1,
            f"unreadable town event {key}",
        )
    for n, script, spot, face in [
        (WEST, "MN_R16_RIVER_WARDEN", (3, 25), (3, 24)),
        (EAST, "MN_R16_SIGN_SEED", (14, 22), (14, 23)),
    ]:
        matches = [
            o for o in [*rooms[n]["events"]["bg_events"], *rooms[n]["objects"]] if o["script"] == script
        ]
        require(
            len(matches) == 1 and (matches[0]["x"], matches[0]["y"]) == spot and (n, *face) in geometry,
            f"unreachable {script}",
        )
    path_findings = {}
    for actor_id, name in RUNNERS.items():
        other_sights = {s for key, ray in rays.items() if key != actor_id for s in ray}
        for n, x, y in other_sights:
            trail.bodies[n].add((x, y))
        safe = paths(trail, stand)
        choices = set(rays[actor_id]) & safe.keys()
        require(choices, f"{name}: no approach avoiding every other local trainer")
        sight = min(choices, key=lambda corner: (safe[corner], corner))
        back = paths(trail, sight)
        require(
            all(at in back for at in (stand, (WEST, 0, 24), (EAST, 31, 24))),
            f"{name}: both road entrances and levee return must share this sightline",
        )
        path_findings[name] = (sight, safe[sight])
        for n, x, y in other_sights:
            trail.bodies[n].remove((x, y))
    # These independent returns can be composed in all six orders, even with every other
    # initial ray blocked. A victory can only remove the corresponding sight restriction.
    print(
        f"Both bridge decks, town/checkpoint, clinic and six two-lane seams; "
        f"{geometry[origin]} dry steps from helper back to Brainerd. Trainer sights: {path_findings}"
    )


def validate_roster(data):
    files = REGION / "pokemon"
    retired = {f["id"] for f in json.loads((files / "roster-curation.json").read_text())["retired_families"]}
    active = {
        m["name"].upper()
        for f in json.loads((files / "roster.json").read_text())["families"]
        if f["id"] not in retired
        for m in f["members"]
    }
    trainers = data["trainers"]["trainers"]
    count = 0
    for n in (R15, APPROACH, WEST, EAST):
        room = data["maps"][n]
        for actor in room["objects"]:
            if actor["object_type"] != "OBJECTTYPE_TRAINER":
                continue
            label = actor["script"]
            (evt,) = [row for row in room["scripts"][label] if row["command"] == "trainer"]
            klass, team, won = evt["args"][:3]
            native = room["trainer_scripts"][label]
            require(
                (klass, team, won) == (native["trainer_class"], native["trainer_id"], native["event_flag"])
                and klass == trainers[team]["trainer_class"],
                f"trainer binding {label}",
            )
            if label in RUNNERS:
                require(
                    team == label
                    and trainers[team]["name"] == RUNNERS[label]
                    and won == "EVENT_MN_R16_TRAINER_" + label[-2:],
                    f"wrong runner {label}",
                )
            require(
                all(m["species"] in active for m in trainers[team]["party"]),
                f"foreign species {team}",
            )
            count += 1
    require(count == 14, f"expected 14 local trainers, found {count}")
    print("All 14 Route 15, Detroit approach and Route 16 trainer parties use the active Minnesota roster.")


def talk(room, entry, flags, bag):
    """Interpret only the dialogue, condition and inventory operations these residents use."""
    active = set(flags)
    contents = list(bag)
    pages, grants = [], []
    condition = None
    label = entry
    for _ in range(64):
        for row in room["scripts"][label]:
            op, args = row["command"], row["args"]
            if op in (
                "playsound",
                "opentext",
                "closetext",
                "faceplayer",
                "waitbutton",
                "promptbutton",
            ):
                continue
            if op in ("checkevent", "checkflag"):
                condition = args[0] in active
            elif op == "setevent":
                active.add(args[0])
            elif op == "giveitem":
                item, count = args
                require(count == "1", f"{label}: unexpected quantity {count}")
                slot = next(
                    (i for i, (kind, qty) in enumerate(contents) if kind == item and qty < 99),
                    None,
                )
                condition = slot is not None or len(contents) < 20
                if condition:
                    if slot is None:
                        contents.append((item, 1))
                    else:
                        contents[slot] = (item, contents[slot][1] + 1)
                    grants.append(item)
            elif op in ("jumptext", "jumptextfaceplayer", "writetext"):
                require(args[0] in room["script_text_bodies"], f"{label}: unbound text")
                pages.append(args[0])
                if op != "writetext":
                    return active, contents, pages, grants
            elif op in ("sjump", "iftrue", "iffalse"):
                if op == "sjump" or condition == (op == "iftrue"):
                    label = args[0]
                    break
            elif op == "end":
                return active, contents, pages, grants
            else:
                raise ValueError(f"unexpected {entry} command {label}/{op}")
        else:
            raise ValueError(f"unterminated script {label}")
    raise ValueError(f"{entry} did not finish")


def pockets(data, reward):
    fillers = sorted(
        k
        for k, rec in data["items"].items()
        if rec["pocket"] == "ITEM" and k not in ("FULL_HEAL", "NUGGET") and not k.startswith("ITEM_")
    )
    require(
        len(fillers) >= 20 and data["items"][reward]["pocket"] == "ITEM",
        "native item pocket",
    )
    full = [(k, 1) for k in fillers[:20]]
    return [
        ((), True),
        (full, False),
        ([*full[:19], (reward, 1)], True),
        ([*full[:19], (reward, 99)], False),
    ]


def check_receipt(room, actor, flags, bag, eligible, fits, item, receipt):
    after, contents, pages, given = talk(room, actor, flags, bag)
    expected = eligible and fits
    require(
        given == ([item] if expected else []) and after == (flags | {receipt} if expected else flags),
        f"{actor} wrong grant for {flags}: {pages}",
    )
    if expected:
        require(
            sum(q for k, q in contents if k == item) == sum(q for k, q in bag if k == item) + 1,
            f"{actor}: wrong inventory count",
        )
        rpt_flags, rpt_contents, _, rpt_gift = talk(room, actor, after, contents)
        require(
            (rpt_flags, rpt_contents, rpt_gift) == (after, contents, []),
            f"{actor} duplicate",
        )
    else:
        require(list(bag) == contents, f"{actor}: failed grant changed inventory")
        if eligible:
            rest, later_contents, _, later_gifts = talk(room, actor, after, contents[1:])
            require(
                rest == after | {receipt}
                and later_gifts == [item]
                and sum(q for k, q in later_contents if k == item)
                == sum(q for k, q in contents[1:] if k == item) + 1,
                f"{actor} retry after freeing a slot",
            )
    return pages


def check_rewards(data, revised):
    town, clinic = data["maps"][TOWN], data["maps"][CLINIC]
    clinic_bags = pockets(data, "FULL_HEAL")
    run_bags = pockets(data, "NUGGET")
    states = 0
    for bits in product((False, True), repeat=7):
        flags = {flag for flag, yes in zip((BADGE, READ, KIT, RUN, *VICTORIES), bits) if yes}
        marked, contents, msg, gifts = talk(town, BOARD, flags, ())
        require(
            marked == flags | {READ} and not contents and not gifts and msg,
            f"marker should work before either reward for {flags}",
        )
        require(talk(town, BOARD, marked, ())[0] == marked, "duplicate marker changes state")
        # The keeper's reminder and the kit have their own flag and do not depend on the trainer circuit.
        keeper_flags, _, words, keeper_gifts = talk(town, KEEPER, flags, ())
        require(
            keeper_flags == flags and not keeper_gifts and len(words) == 1,
            "keeper advanced the quest",
        )
        if READ in flags and KIT not in flags:
            require(words == ["MnWayRiverKitText"], "keeper lost the volunteer lead")
        for bag, fits in clinic_bags:
            pages = check_receipt(
                clinic,
                VOLUNTEER,
                flags,
                bag,
                READ in flags and KIT not in flags,
                fits,
                "FULL_HEAL",
                KIT,
            )
            expect = (
                "Repeat" if KIT in flags else "First" if READ not in flags else "Reward" if fits else "Full"
            )
            label = "MN_MOORHEAD_CLINIC_VOLUNTEERText" if expect == "First" else f"MnMoorheadKit{expect}Text"
            require(pages[-1] == label, f"clinic volunteer {expect} selected {pages}")
        if revised:
            ready = {BADGE, READ, *VICTORIES} <= flags and RUN not in flags
            missing = [
                name.title()
                for key, name in RUNNERS.items()
                if "EVENT_MN_R16_TRAINER_" + key[-2:] not in flags
            ]
            for bag, fits in run_bags:
                pages = check_receipt(town, HOST, flags, bag, ready, fits, "NUGGET", RUN)
                if RUN in flags:
                    expected = ["MN_MOORHEAD_LEVEE_HELPERText"]
                else:
                    if BADGE not in flags:
                        names = ["NeedLake"]
                    elif missing:
                        names = ["Intro", *missing]
                    elif READ not in flags:
                        names = ["Gauge"]
                    else:
                        names = ["Paid" if fits else "Full"]
                    expected = [f"MnPrairie{name}Text" for name in names]
                require(pages == expected, f"helper directions for {flags}: expected {expected}, got {pages}")
        else:
            after, contents, pages, gifts = talk(town, HOST, flags, ())
            require(
                flags == after and not contents and not gifts and pages,
                "parent already has a reward at the levee helper",
            )
        states += 1
    print(
        f"{states} badge/marker/two receipts/three wins states × four actual Items-pocket arrangements per gift; "
        "each full-pocket refusal, later retry, partial-stack grant and repeat checked."
    )


def check_guidance(data):
    canonical = json.loads((REGION / "story.json").read_text())["dialogue_overhaul"]["texts"]
    labels = {
        TOWN: {
            f"MnPrairie{s}Text"
            for s in (
                "NeedLake",
                "Lyle",
                "June",
                "Finch",
                "Gauge",
                "Paid",
                "Full",
                "Intro",
            )
        },
        WEST: {
            "MN_R16_RIVER_WARDENText",
            "MN_R16_T01Seen",
            "MN_R16_T04Seen",
            "MN_R16_T04Won",
            "MN_R16_T04Lost",
            "MN_R16_T04ReturnText",
        },
        EAST: {"MN_R16_SIGN_SEEDText", "MN_R16_T09Seen"},
    }
    words = {}
    for n, names in labels.items():
        room = data["maps"][n]
        for label in names:
            rows = room["scripts"][label]
            pages = native_pages(rows)
            require(
                pages == canonical[n][label]
                and data["map_scripts"][label] == rows
                and native_pages(room["script_text_bodies"][label]["commands"]) == pages,
                f"{n}/{label} changed between canonical text, local/native or global script",
            )
            require(
                all(1 <= len(page) <= 2 and all(1 <= len(row) <= 18 for row in page) for page in pages),
                f"{label}: clipped field dialogue",
            )
            words[label] = " ".join(line.lower() for page in pages for line in page)
    for lab, needles in {
        "MN_R16_SIGN_SEEDText": (
            "finch",
            "june",
            "lyle",
            "moorhead",
            "helper",
            "center",
            "mart",
        ),
        "MN_R16_RIVER_WARDENText": ("moorhead", "west", "east", "bridge"),
        "MnPrairieNeedLakeText": ("lake badge", "brainerd"),
        "MnPrairieLyleText": ("lyle", "pier", "north", "bridge"),
        "MnPrairieJuneText": ("june", "south", "hay"),
        "MnPrairieFinchText": ("finch", "east", "flowers"),
        "MnPrairieGaugeText": ("flood", "north", "levee"),
        "MnPrairiePaidText": ("nugget",),
        "MnPrairieFullText": ("pack", "full", "nugget"),
    }.items():
        require(
            all(needle in words[lab] for needle in needles),
            f"{lab}: wrong place, person or item",
        )
    require(
        words["MN_R16_RIVER_WARDENText"].index("east") < words["MN_R16_RIVER_WARDENText"].index("bridge"),
        "the warden falsely puts the river bridge west of his position",
    )
    print(
        "The eastern course sign, western bridge directions, three trainer landmarks, actual rewards "
        "and readable native pages agree with the installed actions."
    )


def check_teams(old, new):
    original = old["trainers"]["trainers"]
    teams = new["trainers"]["trainers"]
    lineups = {
        "MN_R16_T01": ["JIGLET", "CACHEEK", "RONDIVE"],
        "MN_R16_T09": ["TAPDUCK", "VANECHICK", "BOOMRUFF"],
    }
    for n in (R15, APPROACH, EAST, WEST):
        for act in new["maps"][n]["objects"]:
            if act["object_type"] != "OBJECTTYPE_TRAINER":
                continue
            key = act["script"]
            before, after = original[key], teams[key]
            if key not in lineups:
                require(before == after, f"unrelated local trainer {key} changed")
                continue
            require(
                {**before, "party": after["party"]} == after
                and [m["species"] for m in after["party"]] == lineups[key][: len(before["party"])]
                and len(after["party"]) == len(before["party"]),
                f"{key} lost identity or lineup",
            )
            for old_mon, member in zip(before["party"], after["party"]):
                require(
                    {**old_mon, "species": member["species"], "moves": member["moves"]} == member,
                    f"{key}: changed member level, held item or other attributes",
                )
                available = {
                    move for lvl, move in new["learnsets"][member["species"]] if lvl <= member["level"]
                }
                require(
                    len(member["moves"]) == len(set(m["name"] for m in member["moves"])) == 4,
                    f"{key}: four distinct battle choices required",
                )
                require(
                    all(
                        m["name"] in available
                        and m["current_pp"] == new["moves"][m["name"]]["pp"]
                        and m["pp_ups"] == 0
                        for m in member["moves"]
                    ),
                    f"{key}: illegal moves or PP",
                )


def check_native_bindings(data, revised):
    flags = data["story_event_script_constants"]["global"]
    legacy = {READ: 2443, KIT: 2444, BADGE: 31, **dict(zip(VICTORIES, (2076, 2079, 2084)))}
    require(
        all(flags.get(label) == ident for label, ident in legacy.items()), "existing receipt or Lake Badge ID"
    )
    if revised:
        require(
            RUN in flags
            and [k for k, value in flags.items() if k.startswith("EVENT_") and value == flags[RUN]] == [RUN],
            "new receipt missing or aliases an existing event",
        )
    bindings = {
        "checkevent": ("script_flag_commands", "flag_id"),
        "checkflag": ("script_flag_commands", "flag_id"),
        "setevent": ("script_flag_commands", "flag_id"),
        "sjump": ("script_control_commands", "resolved_target_script"),
        "iftrue": ("script_control_commands", "resolved_target_script"),
        "iffalse": ("script_control_commands", "resolved_target_script"),
        "writetext": ("script_text_commands", "text_label"),
        "giveitem": ("script_item_grants", "item_id"),
    }
    for n in (TOWN, CLINIC):
        room = data["maps"][n]
        todo = [BOARD, HOST, KEEPER] if n == TOWN else [VOLUNTEER]
        seen = set()
        while todo:
            key = todo.pop()
            if key in seen:
                continue
            seen.add(key)
            rows = room["scripts"][key]
            require(rows == data["map_scripts"][key], f"{n}/{key} raw script mirror")
            for i, row in enumerate(rows):
                op, args = row["command"], row["args"]
                if op not in bindings:
                    continue
                table, field = bindings[op]
                (decoded,) = [
                    rec for rec in room[table] if rec["source_script"] == key and rec["command_index"] == i
                ]
                require(decoded["command"] == op and decoded[field] == args[0], f"{n}/{key}[{i}] stale {op}")
                if op == "giveitem":
                    require(decoded["quantity"] == int(args[1]), f"{key} item quantity")
                elif field == "resolved_target_script":
                    todo.append(args[0])
                elif field == "flag_id":
                    require(args[0] in flags, f"{key} unresolved {op} {args[0]}")


def check_archive(data, revised):
    check_native_bindings(data, revised)
    validate_roster(data)
    check_trail(data)
    check_rewards(data, revised)
    if revised:
        check_guidance(data)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("parent", type=Path)
    parser.add_argument(
        "child",
        type=Path,
        nargs="?",
        help="omit to audit the original Full Heal errand",
    )
    args = parser.parse_args()
    old = load_pack(args.parent)["data"]
    print(f"Parent {args.parent.name}")
    check_archive(old, False)
    if args.child:
        new = load_pack(args.child)["data"]
        new_id = new["story_event_script_constants"]["global"].get(RUN)
        require(
            new_id is not None
            and not any(
                name.startswith("EVENT_") and v == new_id
                for name, v in old["story_event_script_constants"]["global"].items()
            ),
            "reused a native parent event ID for the trainer prize",
        )
        print(f"Child {args.child.name}")
        check_archive(new, True)
        check_teams(old, new)
        # Neither an inspection of the marker nor a clinic visit should acquire an unrelated battle prerequisite.
        for n, labels in [
            (
                CLINIC,
                (
                    VOLUNTEER,
                    "MnMoorheadKitFirst",
                    "MnMoorheadKitFull",
                    "MnMoorheadKitRepeat",
                ),
            ),
            (TOWN, (BOARD,)),
        ]:
            for label in labels:
                require(
                    old["maps"][n]["scripts"][label] == new["maps"][n]["scripts"][label],
                    f"original clinic or gauge transaction changed: {n}/{label}",
                )


if __name__ == "__main__":
    main()
