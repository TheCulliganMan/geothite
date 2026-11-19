#!/usr/bin/env python3
"""Trace the six Fen search squares, saved observations and the dry path to Mitwist."""

import argparse
from dataclasses import dataclass, field
from itertools import permutations, product
import json
from pathlib import Path
import runpy
import sys

HERE = Path(__file__).resolve().parent
REGION = HERE.parents[1]
sys.path.insert(0, str(REGION / "tools"))
from audit_story_flow import native_pages, require
from render_reference import load_pack

Trail = runpy.run_path(str(HERE.parent / "northern-trails-v01/check-voyageurs.py"))["Trail"]
FEN, WOODS = "MnLayeredFen", "MnBemidjiNorthwoods"
ACTOR = "MN_FEN_FLARKINDLE"  # Existing save/script ID; the installed Pokémon is MITWIST.
P = "EVENT_MN_FEN_"
READS = [P + name + "_READ" for name in ("ROOT", "POOL", "RIB")]
COMPARED, OPEN, CAUGHT = (P + s for s in ("LAYERS_COMPARED", "FOLDED_PATH_REVEALED", "CAUGHT"))
ROOT, POOL, SEEN = (P + s for s in ("SEQUENCE_ROOT", "SEQUENCE_POOL", "OBSERVER_READ"))
PAGES = ("root", "pool", "rib")
HINTS = ("observer_partial", "early", "wrong_order")
SITES = [
    ("MnFenReadRoot", (12, 12), (11, 12)), ("MnFenChooseRoot", (12, 15), (12, 16)),
    ("MnFenReadPool", (12, 18), (12, 17)), ("MnFenChoosePool", (12, 20), (11, 20)),
    ("MnFenReadRib", (30, 12), (31, 12)), ("MnFenChooseRib", (28, 14), (29, 14)),
]
CONTACTS = [(SITES[i][0], SITES[i + 1][0]) for i in range(0, len(SITES), 2)]
CUT = ("changeblock", "26", "8", "1")


@dataclass
class Visit:
    flags: set = field(default_factory=set)
    visible: set = field(default_factory=set)


def follow(room, label, visit, yes=False, caught=False):
    """Follow installed field instructions; reject unfamiliar commands and nonterminating paths."""
    stack, trace, cond = [(label, 0)], [], None
    while stack:
        require(len(trace) < 120, f"infinite native branch from {label}")
        name, i = stack[-1]
        row = room["scripts"][name][i]
        op, args = row["command"], row["args"]
        trace.append((op, *args))
        stack[-1] = (name, i + 1)
        if op in ("end", "endcallback", "jumptext"):
            stack.pop()
        elif op == "scall":
            stack.append((args[0], 0))
        elif op in ("sjump", "iftrue", "iffalse"):
            if op == "sjump" or cond == (op == "iftrue"):
                stack[-1] = (args[0], 0)
        elif op == "checkevent":
            cond = args[0] in visit.flags
        elif op == "setevent":
            visit.flags.add(args[0])
        elif op == "clearevent":
            visit.flags.discard(args[0])
        elif op == "appear":
            visit.visible.add(args[0])
        elif op == "disappear":
            visit.visible.discard(args[0])
        elif op == "yesorno":
            cond = yes
        elif op == "special":
            require(args == ["CheckCaughtCelebi"], f"unexpected special {args}")
            cond = caught
        else:
            require(op in {"opentext", "writetext", "promptbutton", "closetext", "faceplayer",
                           "changeblock", "refreshmap", "cry", "loadvar", "loadwildmon",
                           "startbattle", "reloadmapafterbattle"}, f"untraced native {name}/{op}")
    return trace


def printed(trace):
    return [op[1].removeprefix("MnFenText_") for op in trace if op[0] in ("jumptext", "writetext")]


def geometry(data):
    room = data["maps"][FEN]
    require(room["events"]["warps"] == room["events"]["coord_events"] == [], "unexpected Fen ingress")
    event_set = {(e["script"], (e["x"], e["y"]), e["event_type"]) for e in room["events"]["bg_events"]}
    require(event_set == {(label, xy, "BGEVENT_READ") for label, xy, _ in SITES}, "changed six field contacts")
    trail = Trail(data, [FEN, WOODS])
    start = (FEN, 39, 16)
    for y in (16, 17):
        require(trail.step((FEN, 39, y), "RIGHT") == (WOODS, 0, y)
                and trail.step((WOODS, 0, y), "LEFT") == (FEN, 39, y), "broken physical seam")
    print("Dry walking steps from (39,16) in and out with actors present")
    for label, (x, y), (sx, sy) in SITES:
        require(abs(x - sx) + abs(y - sy) == 1 and (x, y) not in trail.bodies[FEN], f"no facing A contact {label}")
        print(f"  {label} ({x},{y}) from ({sx},{sy}): {trail.both(start, (FEN,sx,sy), dry=True)}")
    require(trail.both(start, (FEN, 36, 16), dry=True) == (3, 3), "guide no longer accessible")
    actor = [o for o in room["objects"] if o["script"] == ACTOR]
    require(len(actor) == 1 and (actor[0]["sprite"], actor[0]["x"], actor[0]["y"]) == ("SPRITE_MITWIST", 32, 5),
            "wrong wild Pokémon, position or identity")
    adjacent = [(31, 5), (33, 5), (32, 4), (32, 6)]
    for x, y in adjacent:
        try:
            trail.path(start, (FEN, x, y), dry=True)
        except ValueError:
            continue
        raise ValueError(f"premature dry access to Mitwist from ({x},{y})")
    typed, = room["script_block_changes"]
    require(typed == {"x": 26, "y": 8, "block_id": 1, "source_script": "MnFenRestoreTerrain", "command_index": 2}
            and follow(room, "MnFenTiles", Visit({OPEN})).count(CUT) == 1
            and CUT not in follow(room, "MnFenTiles", Visit()), "terrain callback disagrees with actual crossing")
    blocks = room["blocks"].copy()
    i = room["attributes"]["width"] * (typed["y"] // 2) + typed["x"] // 2
    require(blocks[i] == 5, "closed crossing lost the forest")
    blocks[i] = typed["block_id"]
    opened_data = dict(data, maps={FEN: dict(room, blocks=blocks), WOODS: data["maps"][WOODS]})
    opened = Trail(opened_data, [FEN, WOODS])
    require(opened.both(start, (FEN, 31, 5), dry=True) == (31, 31), "changed open path or its return")
    print("  Two east crossings, closed hollow inaccessible on foot; revealed cut (26,8) gives 31 steps each way.")


def callbacks(room, visit, reentry):
    bindings = {row["args"][0]: row["args"][1] for row in room["map_script_section_commands"]
                if row["command"] == "callback"}
    kinds = ("NEWMAP", "TILES", "OBJECTS") if reentry else ("TILES",)
    return [op for k in kinds for op in follow(room, bindings["MAPCALLBACK_" + k], visit)]


def encounters(room):
    before = Visit(visible={ACTOR})
    require(CUT not in callbacks(room, before, True) and ACTOR not in before.visible, "premature actor")
    require(("yesorno",) not in follow(room, ACTOR, before, yes=True), "premature battle")
    for caught in (False, True):
        visit = Visit({OPEN})
        require(CUT in callbacks(room, visit, True) and ACTOR in visit.visible, "absent revealed Pokémon")
        decline = follow(room, ACTOR, visit)
        require(printed(decline)[-2:] == ["ask", "decline"] and ("startbattle",) not in decline
                and ACTOR in visit.visible and visit.flags == {OPEN}, "declining changes discovery")
        result = follow(room, ACTOR, visit, yes=True, caught=caught)
        require(("loadwildmon", "MITWIST", "55") in result and result.count(("startbattle",)) == 1
                and (CAUGHT in visit.flags) == caught and ACTOR not in visit.visible, "wrong optional encounter")
        require(CUT in callbacks(room, visit, False) and ACTOR not in visit.visible, "wrong immediate reload")
        require(CUT in callbacks(room, visit, True) and (ACTOR in visit.visible) != caught, "wrong visit after battle")


def observations(room):
    # All six orders, with an independent choice of primary or old contact on each of six visits.
    count = 0
    for order in permutations(range(3)):
        for incoming in product(range(2), repeat=3):
            read = Visit()
            for step, site in enumerate(order):
                result = follow(room, CONTACTS[site][incoming[site]], read)
                require(printed(result) == [PAGES[site]] + (["layers_settle"] if step == 2 else [])
                        and read.flags == {READS[s] for s in order[:step + 1]} | ({COMPARED} if step == 2 else set()),
                        "observations failed to teach sequence at the actual landmark")
            for outgoing in product(range(2), repeat=3):
                v = Visit(read.flags.copy())
                for site in range(3):
                    old = v.flags.copy()
                    require(printed(follow(room, "MnFenObserver", v)) == [HINTS[site]] and v.flags == old,
                            "observer lost next step after comparison")
                    for other in {0, 1, 2} - {site}:
                        for label in CONTACTS[other]:
                            require(printed(follow(room, label, v)) == [HINTS[site]] and v.flags == old,
                                    f"out-of-order or repeated touch destroyed progress at {label}")
                    result = follow(room, CONTACTS[site][outgoing[site]], v)
                    expected = [PAGES[site] + "_selected"] + (["folded_path"] if site == 2 else [])
                    require(printed(result) == expected, f"incorrect next action {site}")
                    require(v.flags == [old | {ROOT}, old | {POOL}, read.flags | {OPEN}][site], "bad saved step")
                require(v.visible == {ACTOR} and result.count(CUT) == 1, "no physical crossing or actor at reveal")
                for pair in CONTACTS:
                    for label in pair:
                        old = v.flags.copy()
                        require(printed(follow(room, label, v)) == [PAGES[CONTACTS.index(pair)]]
                                and v.flags == old, "completed landmark no longer readable")
                count += 1
    for site, pair in enumerate(CONTACTS):
        revisit = Visit()
        follow(room, pair[0], revisit)
        require(printed(follow(room, pair[1], revisit)) == [PAGES[site]]
                and revisit.flags == {READS[site]}, "rereading an alias counted as a second observation")
    # Simulate saves with a subset of observations and first/return conversation, including
    # legacy saves with all three read but still awaiting the observer's original comparison.
    for subset in product((False, True), repeat=3):
        had = {flag for flag, seen in zip(READS, subset) if seen}
        missing = [i for i in range(3) if not subset[i]]
        for met in (False, True):
            v = Visit(had | ({SEEN} if met else set()))
            pages = printed(follow(room, "MnFenObserver", v))
            if not missing:
                require(pages == ["observer_compare"] and v.flags == had | {COMPARED} | ({SEEN} if met else set()),
                        "legacy all-read save lost comparison")
            else:
                expected = HINTS[missing[0]] if met or had else "observer_intro"
                require(pages == [expected] and v.flags == had | {SEEN}, "wrong missing feature for existing observations")
                for i in missing:
                    follow(room, CONTACTS[i][0], v)
                require(v.flags == set(READS) | {SEEN, COMPARED}, "partial visit cannot finish at original marks")
    print(f"  {count} complete search/trigger combinations, 16 partial-save guide contexts and all 3 mistake phases passed.")


def baseline(room):
    visit = Visit()
    for read, _ in CONTACTS:
        follow(room, read, visit)
    require(visit.flags == set(READS), "parent no longer requires the observer after all observations")
    follow(room, "MnFenObserver", visit)
    follow(room, "MnFenReadRoot", visit)
    require(visit.flags == set(READS) | {COMPARED}, "parent's original root already operates the sequence")
    follow(room, "MnFenChooseRoot", visit)
    require(ROOT in visit.flags, "parent's offset root no longer operates sequence")
    follow(room, "MnFenChooseRib", visit)
    require(ROOT not in visit.flags, "parent's erroneous touch no longer resets prior progress")
    print("  Reproduced parent: return to guide needed, original root cannot act, offset root + wrong moss erases step.")


def check_pages(data, room):
    story = json.loads((REGION / "story.json").read_text())["dialogue_overhaul"]["texts"][FEN]
    rare = json.loads((REGION / "pokemon/acquisition-v01/rare-discoveries.json").read_text())
    source, = (q["dialogue_pages"] for q in rare["quests"] if q["id"] == "fen-memory")
    keys = ["observer_intro", "observer_partial", "observer_compare", "observer_after", *PAGES,
            "root_selected", "pool_selected", "rib_selected", "folded_path", "early", "wrong_order", "layers_settle"]
    for key in keys:
        label = "MnFenText_" + key
        rows = room["script_text_bodies"][label]["commands"]
        require(story[label] == source[key] == native_pages(rows), f"divergent prompt {key}")
        require(all(len(line) <= 18 and line.isascii() for page in native_pages(rows) for line in page), "bad text fit")
        require([dict(command=r["command"], args=r["args"]) for r in rows] == room["scripts"][label]
                == data["map_scripts"][label], f"unpublished page {key}")
    # Verify all derived script indices still point to the corresponding native op, including reveal/appear.
    for table in ("script_control_commands", "script_flag_commands", "script_text_commands",
                  "script_object_commands", "script_runtime_commands"):
        for row in room[table]:
            label = row["source_script"]
            raw = room["scripts"][label][row["command_index"]]
            require(raw["command"] == row["command"] and data["map_scripts"][label] == room["scripts"][label],
                    f"misindexed native {table} / {label}")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("pack", type=Path)
    parser.add_argument("--baseline", action="store_true")
    parser.add_argument("--parent", type=Path)
    args = parser.parse_args()
    data = load_pack(args.pack)["data"]
    room = data["maps"][FEN]
    geometry(data)
    encounters(room)
    if args.baseline:
        baseline(room)
    else:
        check_pages(data, room)
        observations(room)
    if args.parent:
        parent = load_pack(args.parent)["data"]
        prior = parent["maps"][FEN]
        require(all(prior[k] == room[k] for k in ("events", "objects", "scripted_wild_battles",
                    "script_block_changes", "map_script_section_commands")), "changed standing event/catch/terrain metadata")
        for label in [ACTOR, "MnFenCaught", "MnFenDeclined", "MnFenNewVisit", "MnFenObjects", "MnFenTiles",
                      "MnFenRestoreTerrain", "MnFenOpenPath", "MnFenText_ask", "MnFenText_reveal", "MnFenText_caught"]:
            require(prior["scripts"][label] == room["scripts"][label], f"changed unrelated encounter {label}")
        for label in ("event_flags", "story_event_script_constants"):
            if label in data:
                require(parent[label] == data[label], f"changed old flag namespace {label}")
        baseline(prior)
    print("Static compiled field instructions, physical approaches, level-55 offer, refusal, capture, retreat and reentry passed.")


if __name__ == "__main__":
    main()
