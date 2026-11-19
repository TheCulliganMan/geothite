#!/usr/bin/env python3
"""Check Minnesota's actual undefeated trainer bindings and entrance-connected sight squares."""

import argparse
import runpy
import sys
from collections import Counter, deque
from functools import lru_cache
from pathlib import Path

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE.parents[1] / "tools"))
from audit_story_flow import instructions, require
from render_reference import load_pack

navigation = runpy.run_path(str(HERE.parent / "lake-observations-v01/check-hidden.py"))
D = navigation["DIRECTIONS"]
coords, ground, adjacent = [navigation[k] for k in ("coords", "ground", "adjacent")]
quadrant, anchors, approaches = [navigation[k] for k in ("quadrant", "anchors", "approaches")]
CUT = {"MnR06C00R00": (29, 32), "MnR08C00R00": (6, 21), "MnR13C00R02": (18, 7)}
GUARDS = {
    "MnIceCargoBasement": ("MN_ICE_CARGO_GUARD", (12, 12), "LEFT"),
    "MnIceRelayFloor": ("MN_ICE_RELAY_GUARD", (16, 11), "DOWN"),
}
WARP = navigation["WARP"] | {"WARP_PANEL"}


def bearing(actor):
    override = actor["sightline_direction_override"]
    if override:
        return override
    movement = actor["spritemovedata"].removeprefix("SPRITEMOVEDATA_")
    if movement.startswith("STANDING_"):
        return movement[9:]
    return {"SPINCLOCKWISE": "RIGHT", "SPINCOUNTERCLOCKWISE": "LEFT", "PATROL_CLOCKWISE": "RIGHT"}[movement]


def rays(actor, pos, occupied, kind):
    """Match core's first heading, authored radius, native viewport and optional clear-path check."""
    direction = bearing(actor)
    dx, dy = D[direction]
    native = actor["sightline_direction_override"] is not None
    radius = actor["radius"] if native else max(8, actor["radius"])
    x, y = pos
    result = []
    for i in range(1, radius + 1):
        nxt = x + dx, y + dy
        role = kind(nxt)
        if native and not (-5 <= -dx * i <= 6 and -5 <= -dy * i <= 5):
            break
        # Unpinned regional teachers can turn. Their longer line must have a walkable,
        # unobstructed approach from teacher to player. Authored fixed-facing lanes
        # follow Crystal: intermediate people or terrain do not stop its native ray.
        if not native and (
            nxt in occupied
            or role in WARP
            or not ground(role)
            or (kind((x, y)).endswith("_WALL") and direction in kind((x, y)).split("_"))
            or (role.endswith("_WALL") and navigation["OPPOSITE"][direction] in role.split("_"))
        ):
            break
        if nxt not in occupied:
            result.append(nxt)
        x, y = nxt
    return result


def river_entries(data, name):
    a = data["maps"][name]["attributes"]
    starts = set()
    for c in a["connections"]:
        target = c["target_map"]
        b = data["maps"][target]["attributes"]
        dx, dy = D[navigation["SIDES"][c["direction"]]]
        for i in range(2 * a["height" if dx else "width"]):
            x, y = (
                ((2 * a["width"] - 1 if dx > 0 else 0), i)
                if dx
                else (i, (2 * a["height"] - 1 if dy > 0 else 0))
            )
            tx = (0 if dx > 0 else 2 * b["width"] - 1) if dx else x - 2 * c["offset"]
            ty = (0 if dy > 0 else 2 * b["height"] - 1) if dy else y - 2 * c["offset"]
            if all(quadrant(data, n, *p) == "WATER" for n, p in [(name, (x, y)), (target, (tx, ty))]):
                starts.add((x, y))
    return starts


def access(data, name, entries, actors):
    dry = approaches(data, name, entries, "Walk")
    result = {"walk": dry}
    needs = {
        a["script"]
        for a in actors
        if not set(
            rays(
                a,
                coords(a),
                {coords(o) for o in data["maps"][name]["objects"]},
                lambda p: quadrant(data, name, *p),
            )
        )
        & dry
    }
    if not needs:
        return result
    result["Surf"] = approaches(data, name, entries | river_entries(data, name), "Surf")
    if name in CUT:
        room = data["maps"][name]
        pos = CUT[name]
        require(
            quadrant(data, name, *pos) == "CUT_TREE" and adjacent(pos) & dry, f"unreachable Cut on {name}"
        )
        i = room["attributes"]["width"] * (pos[1] // 2) + pos[0] // 2
        blocks = room["blocks"].copy()
        repl = data["field_moves"]["cut"]["replacements"][room["attributes"]["tileset_name"]][blocks[i]]
        require(repl["variant"] == "tree", "unexpected field move")
        blocks[i] = repl["replacement_block_id"]
        opened = dict(data, maps={**data["maps"], name: dict(room, blocks=blocks)})
        result["Cut"] = approaches(opened, name, entries, "Walk")
    return result


def run(room, label, flags, blocks, choice=1):
    """Apply an installed gym control or TILES callback, with its real dialogue/menu branch."""
    stack = [(label, 0)]
    condition, steps = None, 0
    while stack:
        name, i = stack[-1]
        row = room["scripts"][name][i]
        op, args = row["command"], row["args"]
        stack[-1] = (name, i + 1)
        steps += 1
        require(steps < 400, f"unterminated gym control {label}")
        if op in ("end", "endcallback", "jumptext"):
            stack.pop()
        elif op == "scall":
            stack.append((args[0], 0))
        elif op in ("sjump", "iftrue", "iffalse", "ifequal"):
            if (
                op == "sjump"
                or (op == "ifequal" and int(args[0]) == choice)
                or (op in ("iftrue", "iffalse") and condition == (op == "iftrue"))
            ):
                stack[-1] = (args[-1], 0)
        elif op == "checkevent":
            condition = args[0] in flags
        elif op == "setevent":
            flags.add(args[0])
        elif op == "clearevent":
            flags.discard(args[0])
        elif op == "changeblock":
            x, y, tile = map(int, args)
            blocks[room["attributes"]["width"] * (y // 2) + x // 2] = tile
        else:
            require(
                op
                in (
                    "opentext",
                    "writetext",
                    "waitbutton",
                    "closetext",
                    "loadmenu",
                    "verticalmenu",
                    "closewindow",
                ),
                f"unsupported gym control {name}/{op}",
            )


def gym_sight(data, room, actors, start):
    """Search from the public arrival through installed gates, menus, cuts, slides and paired lifts."""
    a = room["attributes"]
    table = data["tilesets"][a["tileset_name"]]["collision"]
    ground_roles = {"FLOOR", "ICE", "WARP_PANEL", "WARP_CARPET_DOWN", "WALK_LEFT", "WALK_RIGHT", "HOP_DOWN"}
    all_flags = data["story_event_script_constants"]["global"]
    require(len(start) == 1, "ambiguous public gym arrival")
    (start,) = start
    blocked = {coords(o) for o in room["objects"]} - {coords(o) for o in actors}
    others = {coords(o): o for o in room["events"]["bg_events"]}
    hedges = {coords(o): o["script_name"] for o in room["events"]["coord_events"]}
    tiles = [
        r["args"][1]
        for r in room["map_script_section_commands"]
        if r["command"] == "callback" and r["args"][0] == "MAPCALLBACK_TILES"
    ]
    require(len(tiles) == 1, "missing gym terrain callback")
    public_exits = {
        coords(w) for w in room["events"]["warps"] if w["target_map_constant"] != a["map_constant"]
    } - {start}
    index = {w["index"]: coords(w) for w in room["events"]["warps"]}
    lifts = {
        coords(w): index[w["target_warp_id"]]
        for w in room["events"]["warps"]
        if w["target_map_constant"] == a["map_constant"]
    }

    @lru_cache(None)
    def surface(flags):
        blocks = room["blocks"].copy()
        run(room, tiles[0], set(flags), blocks)
        return tuple(blocks)

    def sample(blocks, pos):
        x, y = pos
        if not (0 <= x < 2 * a["width"] and 0 <= y < 2 * a["height"]):
            return "WALL"
        return table[f"{blocks[y // 2 * a['width'] + x // 2]:02x}"][2 * (y % 2) + x % 2]

    def action(label, flags, choice=1):
        result = set(flags)
        run(room, label, result, list(surface(flags)), choice)
        return frozenset(result)

    initial = start, frozenset(), tuple(coords(o) for o in actors)
    seen, todo, caught = {initial}, deque([initial]), set()
    while todo and len(caught) < len(actors):
        pos, flags, positions = todo.popleft()
        blocks = surface(flags)

        def src(p, blocks=blocks):
            return sample(blocks, p)

        occupied = blocked | set(positions)
        successors = []
        for i, (actor, xy) in enumerate(zip(actors, positions)):
            flag = room["trainer_scripts"][actor["script"]]["event_flag"]
            if flag not in flags and pos in rays(actor, xy, occupied, src):
                require(flag in all_flags, "unknown trainer victory")
                caught.add(actor["script"])
                # The trainer walks right up to the player, stopping one tile away.
                dx, dy = D[bearing(actor)]
                moved = list(positions)
                moved[i] = (pos[0] - dx, pos[1] - dy)
                successors.append((pos, flags | {flag}, tuple(moved)))
                break
        if not successors:
            for direction, (dx, dy) in D.items():
                dest = pos[0] + dx, pos[1] + dy
                if dest in others:
                    for choice in (1, 2):
                        changed = action(others[dest]["script"], flags, choice)
                        if flags != changed:
                            successors.append((pos, changed, positions))
                if dest in hedges and src(dest) == "CUT_TREE":
                    i = dest[1] // 2 * a["width"] + dest[0] // 2
                    repl = data["field_moves"]["cut"]["replacements"][a["tileset_name"]][blocks[i]]
                    require(repl["variant"] == "tree", "gym hedge is not cuttable")
                    changed = action(hedges[dest], flags)
                    require(
                        sample(surface(changed), dest)
                        == table[f"{repl['replacement_block_id']:02x}"][2 * (dest[1] % 2) + dest[0] % 2],
                        "wrong cut landing",
                    )
                    successors.append((dest, changed, positions))
                    continue
                if src(pos) == "HOP_DOWN" and direction == "DOWN":
                    dest = dest[0] + dx, dest[1] + dy
                if dest in occupied or src(dest) not in ground_roles:
                    continue
                while src(dest) in ("ICE", "WALK_LEFT", "WALK_RIGHT"):
                    nx, ny = D[{"WALK_LEFT": "LEFT", "WALK_RIGHT": "RIGHT"}.get(src(dest), direction)]
                    nxt = dest[0] + nx, dest[1] + ny
                    if src(nxt) not in ground_roles or nxt in occupied:
                        break
                    dest = nxt
                if dest not in public_exits:
                    successors.append((lifts.get(dest, dest), flags, positions))
        for next_state in successors:
            if next_state not in seen:
                seen.add(next_state)
                todo.append(next_state)
    missing = {o["script"] for o in actors} - caught
    require(not missing, f"{room['id']}: missing gym sight {missing}")
    return len(caught)


def audit(pack, baseline):
    d = pack["data"]
    incoming = anchors(d)
    flags, tally = set(), Counter()
    for name, room in d["maps"].items():
        if not name.startswith("Mn"):
            continue
        actors = [o for o in room["objects"] if o["object_type"] == "OBJECTTYPE_TRAINER"]
        require(
            all(o["script"] in room["trainer_scripts"] for o in actors), f"{name}: missing native binding"
        )
        require(
            set(room["trainer_scripts"]) == {o["script"] for o in actors},
            f"{name}: native fight bound only to speech",
        )
        if not actors:
            continue
        tally["rooms"] += 1
        pts = {coords(o) for o in room["objects"]}
        for o in actors:
            s = o["script"]
            t = room["trainer_scripts"][s]
            require(
                o["radius"] > 0
                and (o["hram_x"], o["hram_y"], o["event_flag"]) == (-1, -1, "-1")
                and t["event_flag"] in d["story_event_script_constants"]["global"]
                and t["event_flag"] not in flags
                and t["trainer_id"] in d["trainers"]["trainers"],
                f"{name}: unavailable {s}",
            )
            require(
                instructions(room, s)[0]
                == (
                    "trainer",
                    t["trainer_class"],
                    t["trainer_id"],
                    t["event_flag"],
                    t["seen_text"],
                    t["win_text"],
                    t["loss_text"],
                    t["callback"],
                ),
                f"bad battle or victory at {s}",
            )
            flags.add(t["event_flag"])
            require(quadrant(d, name, *coords(o)) in ("FLOOR", "TALL_GRASS"), f"trainer off land {s}")
        if name.endswith("Gym"):
            (leader,) = (o for o in room["objects"] if o["script"] == name + "Leader")
            require(
                leader["object_type"] == "OBJECTTYPE_SCRIPT" and leader["radius"] == 0, "leader lost talk"
            )
            city_doors = {
                coords(w)
                for src in d["maps"].values()
                for source in src["events"]["warps"]
                if src["id"] != name and source["target_map_constant"] == room["attributes"]["map_constant"]
                for w in room["events"]["warps"]
                if w["index"] == source["target_warp_id"]
            }
            tally["gym_students"] += gym_sight(d, room, actors, city_doors)
            tally["talk_leaders"] += 1
            continue
        reachable = access(d, name, incoming[name], actors)
        for o in actors:
            spots = rays(o, coords(o), pts, lambda p, name=name: quadrant(d, name, *p))
            mode = next((m for m, area in reachable.items() if set(spots) & area), None)
            require(
                mode is not None, f"{name}/{o['script']}: no reachable first-heading sight square at {spots}"
            )
            tally[mode] += 1
    for name, (id, xy, facing) in GUARDS.items():
        r = d["maps"][name]
        (o,) = (o for o in r["objects"] if o["script"] == id)
        require(coords(o) == xy and d["npcs"][name] == r["objects"], f"{name}: object mismatch")
        if baseline:
            require(
                o["object_type"] == "OBJECTTYPE_SCRIPT"
                and o["radius"] == 0
                and id not in r["trainer_scripts"]
                and ("startbattle",) in instructions(r, id),
                f"missing original quiet guard {id}",
            )
        else:
            require(
                (o["spritemovedata"], o["sightline_direction_override"], o["radius"])
                == ("SPRITEMOVEDATA_STANDING_" + facing, facing, 3),
                "guard lost its three-square lane",
            )
            reachable = approaches(d, name, incoming[name], "Walk")
            require(
                set(
                    rays(o, xy, {coords(a) for a in r["objects"]}, lambda q, name=name: quadrant(d, name, *q))
                )
                <= reachable,
                f"unreachable guard squares {id}",
            )
            for key in (
                "script_flag_commands",
                "script_object_commands",
                "script_map_commands",
                "script_text_commands",
                "script_control_commands",
                "script_runtime_commands",
            ):
                require(
                    all(row["source_script"] != id for row in r[key]),
                    f"obsolete scripted-battle instruction in {name}/{key}",
                )
            require(
                r["trainer_scripts"][id]["event_flag"] == "EVENT_" + id + "_DEFEATED"
                and not any(b["source_script"] == id for b in r["scripted_trainer_battles"]),
                "stale guard battle",
            )
    total = sum(tally[k] for k in ("walk", "Surf", "Cut", "gym_students"))
    require(tally["talk_leaders"] == 8 and total == (301 if baseline else 303), "wrong trainer inventory")
    print(
        f"{total} ordinary trainer battles in {tally['rooms']} rooms, {tally['gym_students']} students, {tally['talk_leaders']} talk-initiated gym leaders. "
        f"Other first-facing lane witnesses: {tally['walk']} on foot, {tally['Cut']} behind a genuine Cut tree, {tally['Surf']} with water access."
    )
    print(
        "Two ordinary ICE guards "
        + (
            "required talking despite approaching down the aisle."
            if baseline
            else "now see players along all three intended corridor squares."
        )
    )


if __name__ == "__main__":
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument("pack", type=Path)
    p.add_argument("--baseline", action="store_true")
    args = p.parse_args()
    audit(load_pack(args.pack), args.baseline)
