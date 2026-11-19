#!/usr/bin/env python3
"""Trace the Tall Tale gate, six clue orders, log cutset and independent creature encounters."""

import argparse
from dataclasses import dataclass, field
from itertools import permutations, product
from pathlib import Path
import runpy
import sys

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE.parents[1] / "tools"))
from audit_story_flow import instructions, native_pages, require
from render_reference import load_pack

walk = runpy.run_path(str(HERE.parent / "floodwatch-v01/check-moorhead.py"))
explore = runpy.run_path(str(HERE.parent / "lake-observations-v01/check-hidden.py"))
OUTSIDE = "MnBemidjiNorthwoods"
GATE = "MnTallTaleForestGate"
TRAIL = "MnTallTaleTrail"
PREFIX = "EVENT_MN_TALL_TALE_"
OX = "MN_TALE_BABEHEMOTH"
KEEPER = "MN_TALE_BUNJANT"
HOMES = {OX: (18, 13), KEEPER: (30, 8)}
REVEALED = {OX: "OX_REVEALED", KEEPER: "CAMP_KEEPER_REVEALED"}
CLUES = {
    "MnTaleCulvert": ("CULVERT_READ", (23, 29), (22, 29)),
    "MnTalePrints": ("PRINTS_READ", (17, 15), (16, 15)),
    "MnTaleStack": ("STACK_READ", (29, 20), (29, 21)),
}


@dataclass
class Visit:
    flags: set = field(default_factory=set)
    visible: set = field(default_factory=set)
    positions: dict = field(default_factory=lambda: HOMES.copy())
    block: int = 0


def run(room, label, visit, *, yes=False, caught=False):
    """Read the installed opcodes, preserving the visible shell's normal end-from-scall return."""
    frames = [(label, 0)]
    events = []
    test = False
    for _ in range(160):
        if not frames:
            return events
        name, index = frames[-1]
        op, *args = instructions(room, name)[index]
        frames[-1] = (name, index + 1)
        if op == "checkevent":
            test = args[0].removeprefix(PREFIX) in visit.flags
        elif op == "setevent":
            require(args[0].startswith(PREFIX), f"unexpected quest receipt {args}")
            visit.flags.add(args[0][len(PREFIX) :])
        elif op in ("iftrue", "iffalse", "sjump"):
            if op == "sjump" or test == (op == "iftrue"):
                frames[-1] = (args[0], 0)
        elif op == "scall":
            frames.append((args[0], 0))
        elif op in ("end", "endcallback", "return"):
            frames.pop()
        elif op in ("writetext", "jumptext"):
            require(args[0] in room["scripts"], f"missing {args[0]}")
            events.append(("page", args[0]))
            if op == "jumptext":
                frames.pop()
        elif op == "yesorno":
            test = yes
        elif op in ("appear", "disappear"):
            (actor,) = args
            if op == "appear":
                visit.visible.add(actor)
            else:
                visit.visible.discard(actor)
            events.append((op, actor, visit.positions[actor]))
        elif op == "moveobject":
            visit.positions[args[0]] = tuple(map(int, args[1:]))
        elif op == "changeblock":
            require(args[:2] == ["28", "16"], "log callback changed another location")
            visit.block = int(args[2])
        elif op == "special":
            require(args == ["CheckCaughtCelebi"], f"untraced special {args}")
            test = caught
        elif op in (
            "cry",
            "loadvar",
            "loadwildmon",
            "startbattle",
            "reloadmapafterbattle",
            "turnobject",
        ):
            events.append((op, *args))
        elif op not in (
            "faceplayer",
            "opentext",
            "closetext",
            "promptbutton",
            "waitbutton",
            "refreshmap",
        ):
            raise ValueError(f"untraced {name}/{op} {args}")
    raise ValueError(f"nonterminating {label}")


def gate(data):
    maps = data["maps"]
    neighbors = ["MnBemidji", "MnR17C00R02", "MnLayeredFen"]
    trail = walk["Trail"](data, [OUTSIDE, GATE, TRAIL, *neighbors])
    for xy, side, dest, back in [
        ((8, 27), "DOWN", ("MnBemidji", 24, 0), "UP"),
        ((8, 0), "UP", ("MnR17C00R02", 32, 15), "DOWN"),
        ((0, 16), "LEFT", ("MnLayeredFen", 39, 16), "RIGHT"),
    ]:
        at = (OUTSIDE, *xy)
        require(
            trail.step(at, side, True) == dest and trail.step(dest, back, True) == at,
            f"Northwoods approach {dest}",
        )
        trail.both(at, (OUTSIDE, 13, 12), True)

    def warp(name, xy, target, landing):
        (source,) = [w for w in maps[name]["events"]["warps"] if (w["x"], w["y"]) == xy]
        require(
            source["target_map_constant"] == maps[target]["attributes"]["map_constant"],
            f"wrong destination {name}/{xy}",
        )
        (end,) = [w for w in maps[target]["events"]["warps"] if w["index"] == source["target_warp_id"]]
        require(
            (end["x"], end["y"]) == landing and trail.kind(target, *landing) == "FLOOR",
            f"unsafe arrival {target}/{landing}",
        )

    for town, door, arrival in ((OUTSIDE, (13, 11), (1, 4)), (TRAIL, (3, 33), (8, 4))):
        require(trail.kind(town, *door) == "DOOR", f"gate façade {town}")
        warp(town, door, GATE, arrival)
    for y in (4, 5):
        for x, side, town, landing in [(0, "LEFT", OUTSIDE, (13, 12)), (9, "RIGHT", TRAIL, (3, 34))]:
            require(trail.kind(GATE, x, y) == "WARP_CARPET_" + side, "gate floor arrow")
            warp(GATE, (x, y), town, landing)
        trail.both((GATE, 1, y), (GATE, 8, y), True)
    trail.both((GATE, 1, 4), (GATE, 5, 4), True)
    require(trail.kind(GATE, 5, 3) == "COUNTER", "attendant cannot be reached across the counter")
    require(
        any(
            o["script"] == "MnTaleGateAttendant" and (o["x"], o["y"]) == (5, 2)
            for o in maps[GATE]["objects"]
        ),
        "gate attendant",
    )
    require(
        instructions(maps[GATE], "MnTaleGateAttendant") == [("jumptext", "MnTaleGateHint")], "gate hint"
    )
    hint = native_pages(maps[GATE]["scripts"]["MnTaleGateHint"])
    require(hint == native_pages(data["map_scripts"]["MnTaleGateHint"]), "gate hint mirror")
    words = " ".join(line for page in hint for line in page).lower()
    require("road crew" in words and "old camp" in words, "road crew introduction")


def geography(data, baseline):
    gate(data)
    room = data["maps"][TRAIL]
    if not baseline:
        for label, landmarks in {
            "find_pipe": ("pipe", "east", "rocks"),
            "find_print": ("pool north", "east bank", "print"),
            "find_stack": ("logs", "road bends", "pile"),
            "find_path": ("loose log", "trail north"),
            "find_mark": ("old camp", "cut end", "marks"),
        }.items():
            name = "MnTaleText_" + label
            pages = native_pages(room["scripts"][name])
            require(
                pages
                == native_pages(data["map_scripts"][name])
                == native_pages(room["script_text_bodies"][name]["commands"]),
                f"worker reminder mirrors {name}",
            )
            words = " ".join(line for page in pages for line in page).lower()
            require(all(hint in words for hint in landmarks), f"unusable {label} directions: {words}")
    for who in ("ox", "keeper"):
        label = f"MnTaleText_{who}_ask"
        pages = native_pages(room["scripts"][label])
        require(
            pages
            == native_pages(data["map_scripts"][label])
            == native_pages(room["script_text_bodies"][label]["commands"]),
            "encounter prompt mirrors",
        )
        require(
            pages[-1][-1].endswith("Go closer?" if baseline else "Start a battle?"),
            f"unexpected {who} prompt",
        )
    changes = {
        (entry["source_script"], entry["command_index"]): (entry["x"], entry["y"], entry["block_id"])
        for entry in room["script_block_changes"]
    }
    raw = {
        (label, index): tuple(map(int, args))
        for label in room["scripts"]
        for index, (command, *args) in enumerate(instructions(room, label))
        if command == "changeblock"
    }
    require(raw == changes, "script_block_changes diverged from native changeblock opcodes")
    if not baseline:
        for label in ("MnTaleKeeperAtLog", "MnTaleKeeperReturnsToCamp"):
            for index, (command, *args) in enumerate(instructions(room, label)):
                if command not in ("moveobject", "appear", "disappear", "turnobject"):
                    continue
                (entry,) = [
                    r
                    for r in room["script_object_commands"]
                    if r["source_script"] == label and r["command_index"] == index
                ]
                require(
                    entry["command"] == command and entry["object_id"] == args[0],
                    "stale keeper object opcode",
                )
                if command == "moveobject":
                    require(
                        (entry["x"], entry["y"]) == tuple(map(int, args[1:])), "stale keeper coordinates"
                    )
                if command == "turnobject":
                    require(str(entry["direction"]).upper() == args[1], "stale keeper facing")
    bg = {(r["x"], r["y"]): (r["script"], r["event_type"]) for r in room["events"]["bg_events"]}
    signs = {at: label for label, (_, at, _) in CLUES.items()}
    signs.update({(28, 16): "MnTalePath", (28, 8): "MnTaleLog", (30, 12): "MnTaleCampRecord"})
    if not baseline:
        signs.update({(29, 16): "MnTalePath", (28, 20): "MnTaleStack"})
    require(
        all(bg.get(at) == (name, "BGEVENT_READ") for at, name in signs.items()),
        "unreadable trail marker",
    )
    event_label = room["attributes"]["map_events_label"]
    section = [{"command": r["command"], "args": r["args"]} for r in room["map_event_section_commands"]]
    require(
        data["map_scripts"][event_label] == room["scripts"].get(event_label, section) == section,
        "trail event section mirrors",
    )
    compiled_bg = {
        (int(r["args"][0]), int(r["args"][1])): (r["args"][3], r["args"][2])
        for r in section
        if r["command"] == "bg_event"
    }
    require(bg == compiled_bg, "trail BGEVENT mirrors")
    original = {o["object_identifier"]: (o["x"], o["y"]) for o in room["objects"]}
    require(original == {"MnTaleWorker": (10, 34), **HOMES}, "quest actor homes")
    closed = Visit()
    run(room, "MnTaleTerrain", closed)
    require(
        room["blocks"][8 * room["attributes"]["width"] + 14] == closed.block,
        "raw loose log differs from closed callback",
    )
    trail = walk["Trail"](data, [TRAIL])
    trail.bodies[TRAIL].difference_update(HOMES.values())
    origin = (TRAIL, 3, 34)
    closed_paths = walk["paths"](trail, origin)
    wet = explore["approaches"](data, TRAIL, {origin[1:]}, "Surf")
    require(
        (28, 17) in wet and (28, 9) not in wet and (30, 13) not in wet,
        "pool bypasses closed logging trail",
    )
    order = [(10, 35), *(stand for _, _, stand in CLUES.values()), (28, 17), (29, 17), (28, 21)]
    for xy in order:
        trail.both(origin, (TRAIL, *xy), True)
    for xy in [(28, 9), (30, 13), HOMES[KEEPER], (29, 15)]:
        require((TRAIL, *xy) not in closed_paths, f"camp bypass at {xy}")
    for xy in ((28, 16), (29, 16), (28, 20), (29, 20), (28, 8)):
        require(
            trail.kind(TRAIL, *xy) == ("HEADBUTT_TREE" if baseline else "WALL"),
            f"wrong log surface {xy}",
        )
    require(trail.kind(TRAIL, 29, 8) == "HEADBUTT_TREE", "live tree next to the marked log")
    visits = Visit(flags={"CAMP_PATH_ACKNOWLEDGED"})
    run(room, "MnTaleTerrain", visits)
    bank = data["tilesets"][room["attributes"]["tileset_name"]]
    four = bank["collision"][f"{visits.block:02x}"]
    require(visits.block == 1 and four == ["FLOOR"] * 4, "open log tiles")
    require(
        (trail.kind(TRAIL, *HOMES[OX]), trail.kind(TRAIL, *HOMES[KEEPER]), trail.kind(TRAIL, 29, 15))
        == ("FLOOR", "FLOOR", "FLOOR"),
        "creature left on water or wall",
    )
    for i, kind in enumerate(four):
        trail.grid[TRAIL][16 + i // 2][28 + i % 2] = kind
    for ox_visible, keeper_visible in product((False, True), repeat=2):
        trail.bodies[TRAIL] = {(10, 34)}
        trail.bodies[TRAIL].update(
            HOMES[a] for a, v in zip((OX, KEEPER), (ox_visible, keeper_visible)) if v
        )
        for xy in [(28, 9), (30, 13), (18, 14), (30, 9), *order]:
            trail.both(origin, (TRAIL, *xy), True)
        for x in (28, 29):
            require(
                trail.both((TRAIL, x, 17), (TRAIL, x, 15), True) == (2, 2), f"bad camp throat at {x}"
            )
    return room


def page(events):
    return [v[1].removeprefix("MnTaleText_") for v in events if v[0] == "page"]


def quest(room, baseline):
    for order in permutations(CLUES):
        visit = Visit()
        for i in range(4):
            flags = visit.flags.copy()
            seen = {CLUES[k][0] for k in order[:i]}
            response = page(run(room, "MnTaleWorker", visit))
            if not i or (baseline and i < 3):
                want = "worker_introduction"
            elif i == 3:
                want = "worker_comparison"
            else:
                want = next(
                    label
                    for flag, label in (
                        ("CULVERT_READ", "find_pipe"),
                        ("PRINTS_READ", "find_print"),
                        ("STACK_READ", "find_stack"),
                    )
                    if flag not in seen
                )
            require(
                response == [want] and visit.flags == flags | ({"CAMP_DIRECTIONS"} if i == 3 else set()),
                f"worker after {order[:i]}",
            )
            if i < 3:
                for task in ("MnTalePath", "MnTaleLog"):
                    run(room, task, visit, yes=True)
                    require(visit.flags == flags and not visit.visible, "premature clearing or account")
                run(room, order[i], visit)
                require(visit.flags == flags | {CLUES[order[i]][0]}, "observation receipt")
        require(
            page(run(room, "MnTaleWorker", visit)) == ["worker_comparison" if baseline else "find_path"],
            "loose log guidance",
        )
        for answer in (False, True):
            original = visit.flags.copy()
            scene = run(room, "MnTalePath", visit, yes=answer)
            require(
                visit.flags == original | ({"CAMP_PATH_ACKNOWLEDGED"} if answer else set()), "log choice"
            )
            require(visit.visible == set() and visit.positions == HOMES, "keeper stranded at log")
            if answer:
                pose = HOMES[KEEPER] if baseline else (29, 15)
                require(
                    ("appear", KEEPER, pose) in scene
                    and ("disappear", KEEPER, pose) in scene
                    and visit.block == 1,
                    "keeper pose/clear sequence",
                )
        require(run(room, "MnTalePath", visit, yes=True) == [], "open path still offers to roll a log")
        prior = visit.flags.copy()
        require(
            page(run(room, "MnTaleCampRecord", visit)) == ["camp_record"] and visit.flags == prior,
            "camp post replaced the marked log",
        )
        require(
            page(run(room, "MnTaleWorker", visit)) == ["worker_comparison" if baseline else "find_mark"],
            "marked log guidance",
        )
        run(room, "MnTaleLog", visit)
        require("LOG_ACCOUNT_READ" in visit.flags and not visit.visible, "premature reveal")
        require(page(run(room, "MnTaleWorker", visit)) == ["worker_log_account"], "account handoff")
        require(set(REVEALED.values()) <= visit.flags and visit.visible == set(HOMES), "missing reveal")
        require(page(run(room, "MnTaleWorker", visit)) == ["worker_after"], "lost aftermath")
    return visit


def encounters(room, revealed):
    callbacks = {
        r["args"][0]: r["args"][1]
        for r in room["map_script_section_commands"]
        if r["command"] == "callback"
    }
    require(
        callbacks
        == {
            "MAPCALLBACK_OBJECTS": "MnTaleObjects",
            "MAPCALLBACK_NEWMAP": "MnTaleNewVisit",
            "MAPCALLBACK_TILES": "MnTaleTerrain",
        },
        "quest callback bindings",
    )
    for ox, keeper, ox_caught, keeper_caught in product((False, True), repeat=4):
        stages = ((OX, ox, ox_caught), (KEEPER, keeper, keeper_caught))
        flags = {
            flag
            for actor, ready, caught in stages
            for flag, use in [
                (REVEALED[actor], ready),
                (actor.removeprefix("MN_TALE_") + "_CAUGHT", caught),
            ]
            if use
        }
        expected = {actor for actor, ready, caught in stages if ready and not caught}
        visit = Visit(flags.copy(), set(HOMES))
        run(room, "MnTaleNewVisit", visit)
        require(visit.visible == expected and visit.flags == flags, "fresh visit actors")
        for visible in (set(), set(HOMES)):
            visit.visible = visible.copy()
            run(room, "MnTaleObjects", visit)
            require(visit.visible == visible & expected, "Continue revived a departed partner")
    for order in permutations(HOMES):
        visit = Visit(revealed.flags.copy(), set(HOMES))
        for actor in order:
            species = actor.removeprefix("MN_TALE_")
            old = visit.flags.copy()
            remaining = visit.visible.copy()
            no = run(room, actor, visit, yes=False)
            require(
                not any(e[0] == "startbattle" for e in no)
                and visit.flags == old
                and visit.visible == remaining,
                "refusal altered encounter",
            )
            # Escaping and defeating the wild partner both leave CheckCaughtCelebi false.
            for outcome in ("escape", "knockout", "capture"):
                caught = outcome == "capture"
                sequence = run(room, actor, visit, yes=True, caught=caught)
                battle = [
                    e
                    for e in sequence
                    if e[0] in ("cry", "loadvar", "loadwildmon", "startbattle", "reloadmapafterbattle")
                ]
                require(
                    battle
                    == [
                        ("cry", species),
                        ("loadvar", "VAR_BATTLETYPE", "BATTLETYPE_OPTIONAL_LEGENDARY"),
                        ("loadwildmon", species, "60"),
                        ("startbattle",),
                        ("reloadmapafterbattle",),
                    ],
                    "wrong encounter",
                )
                require(
                    visit.flags == old | ({species + "_CAUGHT"} if caught else set())
                    and visit.visible == remaining - {actor},
                    "capture receipts crossed",
                )
                require(("retry" in page(sequence)) != caught, "missing retry feedback")
                run(room, "MnTaleObjects", visit)
                require(actor not in visit.visible, "respawned immediately")
                run(room, "MnTaleNewVisit", visit)
                require((actor in visit.visible) != caught, "retry or persistent capture lost")


def main():
    cli = argparse.ArgumentParser(description=__doc__)
    cli.add_argument("pack", type=Path)
    cli.add_argument("--baseline", action="store_true")
    args = cli.parse_args()
    data = load_pack(args.pack)["data"]
    room = geography(data, args.baseline)
    encounters(room, quest(room, args.baseline))
    print(
        "Three Northwoods approaches; two gate lanes each way; closed/open camp, six clue orders, eight clue subsets, two encounter orders and 16 reveal/capture states passed."
    )


if __name__ == "__main__":
    main()
