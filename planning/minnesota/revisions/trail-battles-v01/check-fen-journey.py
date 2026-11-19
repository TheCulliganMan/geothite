#!/usr/bin/env python3
"""Trace the Fen investigation in every search order and both directions at the three-way trail fork."""

import argparse
import runpy
import sys
from collections import deque
from itertools import permutations, product
from pathlib import Path

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE.parents[1] / "tools"))
from audit_story_flow import native_pages, require
from render_reference import load_pack

native = runpy.run_path(str(HERE.parent / "fen-trail-v01/check-fen.py"))
FEN, WOODS, ACTOR = native["FEN"], native["WOODS"], native["ACTOR"]
READS, OPEN, COMPARED, CAUGHT = native["READS"], native["OPEN"], native["COMPARED"], native["CAUGHT"]
CONTACTS, Visit, follow, callbacks = (native[k] for k in ("CONTACTS", "Visit", "follow", "callbacks"))
DIR = {"UP": (0, -1), "RIGHT": (1, 0), "DOWN": (0, 1), "LEFT": (-1, 0)}
OPPOSITE = {"UP": "DOWN", "DOWN": "UP", "LEFT": "RIGHT", "RIGHT": "LEFT"}
NAMES = "RPF"
ENTRY = (39, 16)
SPECIES = (32, 5)


def adjacent(pos):
    x, y = pos
    return {(x + dx, y + dy) for dx, dy in DIR.values()}


class Area:
    def __init__(self, data, name, opened=False, caught=False):
        r = data["maps"][name]
        a = r["attributes"]
        blocks = list(r["blocks"])
        if name == FEN:
            changes = [
                q
                for q in follow(r, "MnFenTiles", Visit({OPEN} if opened else set()))
                if q[0] == "changeblock"
            ]
            require(len(changes) == int(opened), "unexpected number of Fen terrain changes")
            for _, x, y, tile in changes:
                blocks[int(y) // 2 * a["width"] + int(x) // 2] = int(tile)
        col = data["tilesets"][a["tileset_name"]]["collision"]
        self.grid = {
            (x, y): col[f"{blocks[y // 2 * a['width'] + x // 2]:02x}"][2 * (y % 2) + x % 2]
            for y in range(2 * a["height"])
            for x in range(2 * a["width"])
        }
        self.reads = {(e["x"], e["y"]) for e in r["events"]["bg_events"]}
        self.actors = {
            (o["x"], o["y"]) for o in r["objects"] if o["script"] != ACTOR or opened and not caught
        }
        self.distances = {}
        self.tiles = set(self.grid.values())
        require(
            self.tiles
            <= {"WALL", "FLOOR", "TALL_GRASS", "WATER", "HEADBUTT_TREE", "DOOR", "HOP_DOWN", "UP_WALL"},
            f"untraced terrain {name}/{self.tiles}",
        )

    def kind(self, pos):
        return self.grid.get(pos, "WALL")

    def clear(self, at, wet):
        role = self.kind(at)
        return at not in self.actors and (
            role in {"FLOOR", "TALL_GRASS", "HOP_DOWN", "UP_WALL"} or wet and role == "WATER"
        )

    def step(self, at, dr, wet):
        dx, dy = DIR[dr]
        h = self.kind(at)
        if h.endswith("_WALL") and dr in h.split("_"):
            return None
        jump = 2 if h == "HOP_DOWN" and dr == "DOWN" else 1
        to = at[0] + jump * dx, at[1] + jump * dy
        arrival = self.kind(to)
        if not self.clear(to, wet) or arrival.endswith("_WALL") and OPPOSITE[dr] in arrival.split("_"):
            return None
        return to

    def paths(self, start, wet=False):
        if (start, wet) in self.distances:
            return self.distances[start, wet]
        if not self.clear(start, wet):
            return {}
        found = {start: 0}
        todo = deque([start])
        while todo:
            at = todo.popleft()
            for dr in DIR:
                to = self.step(at, dr, wet)
                if to is not None and to not in found:
                    found[to] = found[at] + 1
                    todo.append(to)
        self.distances[start, wet] = found
        return found


def water(area):
    """Count orthogonal water bodies and ensure each can be boarded and left at real open banks."""
    unknown = {p for p, v in area.grid.items() if v == "WATER"}
    shore = area.paths(ENTRY)
    result = []
    while unknown:
        start = unknown.pop()
        section = {start}
        todo = [start]
        while todo:
            new = adjacent(todo.pop()) & unknown
            section |= new
            unknown -= new
            todo.extend(new)
        launches = {
            (p, q)
            for p in shore
            for dr in DIR
            if (q := area.step(p, dr, True)) in section
            and q not in area.reads
            and area.step(q, OPPOSITE[dr], True) == p
        }
        require(launches, f"water body at {min(section)} has no open shore")
        require(section <= area.paths(ENTRY, True).keys(), "isolated water cannot be reached from the trail")
        result.append((len(section), len(launches)))
    return sorted(result)


def geometry(data, baseline):
    room = data["maps"][FEN]
    woods = Area(data, WOODS)
    require(woods.kind((0, 16)) == woods.kind((0, 17)) == "FLOOR", "blocked Northwoods seam")
    links = {
        c["direction"]: (c["target_map"], c["offset"])
        for c in data["maps"][WOODS]["attributes"]["connections"]
    }
    require(
        links == {"south": ("MnBemidji", -8), "north": ("MnR17C00R02", -12), "west": (FEN, 0)},
        "inbound trail links changed",
    )
    require(
        room["attributes"]["connections"] == [{"direction": "east", "target_map": WOODS, "offset": 0}],
        "Fen no longer returns to Northwoods",
    )
    # Pair physical map-edge squares including both native connection offsets.
    for name, local, dest in [("MnBemidji", (8, 27), (24, 0)), ("MnR17C00R02", (8, 0), (32, 15))]:
        other = data["maps"][name]
        a = other["attributes"]
        x, y = dest
        block = other["blocks"][y // 2 * a["width"] + x // 2]
        require(
            data["tilesets"][a["tileset_name"]]["collision"][f"{block:02x}"][2 * (y % 2) + x % 2]
            in ("FLOOR", "TALL_GRASS")
            and dest not in {(o["x"], o["y"]) for o in other["objects"]},
            f"blocked far side of {name} approach",
        )
        for entry in ((0, 16), (0, 17)):
            require(entry in woods.paths(local) and local in woods.paths(entry), f"one-way {name} approach")
    if not baseline:
        (sign,) = [e for e in data["maps"][WOODS]["events"]["bg_events"] if e["script"] == "MnFenTurnoff"]
        require(
            (sign["x"], sign["y"], sign["event_type"]) == (3, 15, "BGEVENT_READ")
            and woods.kind((3, 15)) == "WALL",
            "trail sign is not on the solid face of the board",
        )
        stand = (3, 16)
        for origin in ((8, 27), (8, 0), (0, 16), (0, 17)):
            require(stand in woods.paths(origin) and origin in woods.paths(stand), "trail sign inaccessible")
        r = data["maps"][WOODS]
        require(
            r["scripts"]["MnFenTurnoff"] == [{"command": "jumptext", "args": ["MnFenTurnoffText"]}]
            and native_pages(r["script_text_bodies"]["MnFenTurnoffText"]["commands"])
            == [["LAYERED FEN - WEST", "ROUTE 17 - EAST"], ["BEMIDJI - SOUTH"]],
            "changed signed destinations",
        )
        require(
            all(woods.kind((x, y)) == "FLOOR" for x in range(12) for y in (16, 17)), "sign took over road"
        )
        # The sign replaces a forest verge; its three new snow squares must not bypass
        # the existing one-way ledge or change the walk between any public approaches.
        old_blocks = r["blocks"].copy()
        shoulder = 7 * r["attributes"]["width"] + 1
        require(old_blocks[shoulder] == 0x78, "wrong native sign block")
        old_blocks[shoulder] = 0x05
        old = Area(dict(data, maps={WOODS: dict(r, blocks=old_blocks)}), WOODS)
        entrances = ((8, 27), (8, 0), (0, 16), (0, 17))
        for start in entrances:
            before, after = old.paths(start), woods.paths(start)
            require(
                set(after) - set(before) == {(2, 14), (3, 14), (2, 15)} and set(before) <= set(after),
                "the signed verge opens a new trail",
            )
            require(all(before[end] == after[end] for end in entrances), "sign changed a through journey")
    closed, opened, captured = Area(data, FEN), Area(data, FEN, True), Area(data, FEN, True, True)
    closed_land = closed.paths(ENTRY)
    opened_land = opened.paths(ENTRY)
    (actor,) = [o for o in room["objects"] if o["script"] == ACTOR]
    require((actor["x"], actor["y"]) == SPECIES and actor["sprite"] == "SPRITE_MITWIST", "wrong destination")
    target = adjacent(SPECIES)
    legal = target & opened_land.keys()
    require(
        not target & closed.paths(ENTRY, True).keys() and legal, "premature Surf access or closed destination"
    )
    require(legal <= captured.paths(ENTRY).keys(), "route closed after capture")
    require(all(ENTRY in opened.paths(p) for p in legal), "hollow has no dry return")
    contacts = {
        e["script"]: (e["x"], e["y"])
        for e in room["events"]["bg_events"]
        if e["event_type"] == "BGEVENT_READ"
    }
    require(contacts == {s: xy for s, xy, _ in native["SITES"]}, "changed search/save contacts")
    stances = {s: adjacent(p) & closed_land.keys() for s, p in contacts.items()}
    for s, from_tiles in stances.items():
        require(
            from_tiles and all(ENTRY in closed.paths(p) for p in from_tiles),
            f"lost dry search or return for {s}",
        )
    require(
        closed.kind((36, 17)) == "FLOOR" and len(closed.actors) == 1,
        "observer standing on unsuitable terrain",
    )
    require(ENTRY in opened.paths((39, 17)) and (39, 17) in closed_land, "lost public entry row")

    def excursion(order):
        costs = {ENTRY: 0}
        for site in order:
            s = CONTACTS[site][0]
            costs = {
                p: min(cost + closed.paths(q)[p] for q, cost in costs.items() if p in closed.paths(q))
                for p in stances[s]
            }
        return min(
            cost + opened.paths(q)[p] for q, cost in costs.items() for p in legal if p in opened.paths(q)
        )

    counts = {
        "".join(NAMES[i] for i in order): excursion(order + (0, 1, 2) if baseline else order)
        for order in permutations(range(3))
    }
    pools = water(closed)
    require(pools, "no actual Fen water")
    print("Dry walks from east entry via the three primary faces to Mitwist (R=root, P=pool, F=fir):", counts)
    print(
        f"{len(closed_land)} public dry squares before, {len(opened_land)} after. "
        f"Water bodies (area, reversible shore contacts) {pools}; no water route into the closed hollow."
    )
    print("Both Bemidji and Route 17 approach and return through the two Fen seam squares.")
    return counts


def progress(room, baseline):
    if baseline:
        native["observations"](room)
        return
    # One visit to each feature in any order. Alternate independent physical faces
    # and repeat any read at its other contact before the next observation.
    count = 0
    for order in permutations(range(3)):
        for aliases in product(range(2), repeat=3):
            visit = Visit()
            for step, i in enumerate(order, 1):
                movement = follow(room, CONTACTS[i][aliases[i]], visit)
                require(
                    READS[i] in visit.flags and (OPEN in visit.flags) == (step == 3),
                    f"wrong reveal on observation {i} of {order}",
                )
                if step < 3:
                    earned = visit.flags.copy()
                    again = follow(room, CONTACTS[i][1 - aliases[i]], visit)
                    require(
                        visit.flags == earned and native["CUT"] not in again, "duplicate face advanced puzzle"
                    )
                    require(
                        native["CUT"] not in callbacks(room, visit, True) and ACTOR not in visit.visible,
                        "partial visit quietly opened hurdle",
                    )
                else:
                    require(
                        ACTOR in visit.visible and movement.count(native["CUT"]) == 1,
                        "third search did not open and populate the hollow",
                    )
            require(set(READS) <= visit.flags, "unique completed observations lost")
            for s, _, _ in native["SITES"]:
                earned = visit.flags.copy()
                follow(room, s, visit)
                require(visit.flags == earned, "reinspection reset discovery")
            count += 1
    # Legacy readers and their old sequence milestones must be able to finish through
    # ordinary contacts or the observer; their existing reveal/capture always wins.
    for subset in product((False, True), repeat=3):
        owned = {flag for flag, yes in zip(READS, subset) if yes}
        historical = (
            [set()]
            if owned != set(READS)
            else [set(), {COMPARED}, {COMPARED, native["ROOT"]}, {COMPARED, native["ROOT"], native["POOL"]}]
        )
        for legacy in historical:
            visit = Visit(owned | legacy)
            missing = [i for i, f in enumerate(READS) if f not in owned]
            for i in missing:
                follow(room, CONTACTS[i][0], visit)
            if not missing:
                follow(room, "MnFenObserver", visit)
            require(OPEN in visit.flags and set(READS) <= visit.flags, "old observations cannot finish")
            require(
                native["CUT"] in callbacks(room, visit, True) and ACTOR in visit.visible,
                "broken resumed reveal",
            )
    native["encounters"](room)
    print(
        f"{count} complete one-lap observation orders/contact choices plus repeats, existing saves and both battle outcomes."
    )


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("pack", type=Path)
    parser.add_argument("--baseline", action="store_true")
    args = parser.parse_args()
    data = load_pack(args.pack)["data"]
    geometry(data, args.baseline)
    progress(data["maps"][FEN], args.baseline)
