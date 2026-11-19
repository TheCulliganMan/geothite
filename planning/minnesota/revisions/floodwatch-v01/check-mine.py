#!/usr/bin/env python3
"""Trace Q06 on installed walking tiles and native scripts, including returns after delivery."""

import argparse
from collections import deque
from pathlib import Path
import sys

TOOLS = Path(__file__).resolve().parents[2] / "tools"
sys.path.insert(0, str(TOOLS))
from audit_story_flow import instructions, native_pages, read_branch, require
from render_reference import load_pack

PUBLIC, APPROACH, HAUL, VENT, SUMP, PUMP, CORE = (
    "MnR09Cave1F", "MnIronDrillApproach", "MnIronHaulage", "MnIronVentilation",
    "MnIronSump", "MnIronPumpGallery", "MnIronDrillCore",
)
TOWN, DEPOT, DESCENT = "MnGrandRapids", "MnGrandRapidsSurveyDepot", "MnR10C00R00"
MOUTH = ("MnR09C00R00", 40, 42)
MAPS = (PUBLIC, APPROACH, HAUL, VENT, SUMP, PUMP, CORE, TOWN, DEPOT, DESCENT,
        "MnR09C00R00", "MnR09C01R00", "MnR09C00R01", "MnR09C01R01")
DIRS = {"north": (0, -1), "south": (0, 1), "west": (-1, 0), "east": (1, 0)}
ACTIVE = {"CAVE", "DOOR", "LADDER", "WARP_CARPET_DOWN"}
LAND = {"FLOOR", "TALL_GRASS", "UP_WALL", "HOP_LEFT", "HOP_DOWN"} | ACTIVE
FINN = "MN_NDRILL_FINN"
FOREMAN, HP, VP, FEED, SEEN, WON, OFF, METER, DELIVERY = [
    "EVENT_MN_" + suffix for suffix in (
        "DEPOT_FOREMAN_DEFEATED", "IRON_HAUL_PRESSURE_OFF", "IRON_VENT_PRESSURE_OFF",
        "DRILL_FEED_ISOLATED", "DRILL_TEST_WITNESSED", "REED_MINE_DEFEATED",
        "DRILL_STOPPED", "MINE_INSTRUMENT_RECOVERED", "DEPOT_CLEARED",
    )
]
VALVES = ((HAUL, "MnIronPressureHAUL", HP, (29, 12)),
          (VENT, "MnIronPressureVENT", VP, (13, 16)),
          (PUMP, "MnNDrillGalleryFeed", FEED, (25, 18)))


def callbacks(room, kind):
    return [r["args"][1] for r in room["map_script_section_commands"]
            if r["command"] == "callback" and r["args"][0] == "MAPCALLBACK_" + kind]


def restore(room, kind, state):
    """Execute installed map callbacks, including the native side effect of actor visibility."""
    flags = set(state)
    actors = {o["object_identifier"]: (o["x"], o["y"]) for o in room["objects"]
              if o["event_flag"] not in flags}
    positions = {o["object_identifier"]: (o["x"], o["y"]) for o in room["objects"]}
    bound = {o["object_identifier"]: o["event_flag"] for o in room["objects"]}
    edits = {}
    for label in callbacks(room, kind):
        for _ in range(8):
            for op, *args in instructions(room, label):
                if op == "checkevent":
                    condition = args[0] in flags
                elif op in ("iftrue", "iffalse"):
                    if condition == (op == "iftrue"):
                        label = args[0]
                        break
                elif op in ("appear", "disappear"):
                    actor = args[0]
                    if op == "appear":
                        actors[actor] = positions[actor]
                        flags.discard(bound[actor])
                    else:
                        actors.pop(actor, None)
                        if bound[actor] != "-1":
                            flags.add(bound[actor])
                elif op == "moveobject":
                    positions[args[0]] = tuple(map(int, args[1:]))
                elif op == "changeblock":
                    x, y, block = map(int, args)
                    edits[x, y] = block
                elif op in ("end", "endcallback"):
                    label = None
                    break
                else:
                    raise ValueError(f"Unhandled callback instruction {room['id']}/{label}/{op}")
            if label is None:
                break
        else:
            raise ValueError(f"Unterminated callback {room['id']}/{label}")
    return flags, actors, edits


class Workings:
    """Native metatile quadrants, occupancy, active warps, arrivals and seams for this journey."""

    def __init__(self, data, flags):
        self.roles, self.links, self.limits = {}, {}, {}
        self.warps, self.occupied = {}, set()
        maps = data["maps"]
        names = {room["attributes"]["map_constant"]: n for n, room in maps.items()}
        for name in MAPS:
            room = maps[name]
            a = room["attributes"]
            w, h = a["width"], a["height"]
            self.limits[name] = (2*w, 2*h)
            self.links[name] = {c["direction"]: c for c in a["connections"]}
            _, people, _ = restore(room, "OBJECTS", flags)
            _, _, edits = restore(room, "TILES", flags)
            self.occupied.update((name, *pos) for pos in people.values())
            collision = data["tilesets"][a["tileset_name"]]["collision"]
            for idx, base in enumerate(room["blocks"]):
                x, y = 2*(idx % w), 2*(idx // w)
                block = edits.get((x, y), base)
                for q, role in enumerate(collision[f"{block:02x}"]):
                    self.roles[name, x + q%2, y + q//2] = role
            for warp in room["events"]["warps"]:
                dst = names[warp["target_map_constant"]]
                if dst not in MAPS:
                    continue
                landing, = [p for p in maps[dst]["events"]["warps"]
                            if p["index"] == warp["target_warp_id"]]
                self.warps[name, warp["x"], warp["y"]] = (dst, landing["x"], landing["y"])

    def step(self, pos, direction):
        name, x, y = pos
        dx, dy = DIRS[direction]
        role = self.roles[pos]
        if role == "UP_WALL" and dy < 0:
            return None
        # Native ledges jump from the lip, over the blocked square, to the landing.
        stride = 2 if (role, direction) in {("HOP_LEFT", "west"), ("HOP_DOWN", "south")} else 1
        x, y = x + stride*dx, y + stride*dy
        if (name, x, y) not in self.roles:
            con = self.links[name].get(direction)
            if not con or con["target_map"] not in self.limits:
                return None
            name = con["target_map"]
            w, h = self.limits[name]
            if dx:
                x, y = (0 if dx > 0 else w-1), y-2*con["offset"]
            else:
                x, y = x-2*con["offset"], (0 if dy > 0 else h-1)
        dest = (name, x, y)
        role = self.roles.get(dest)
        if dest in self.occupied or role not in LAND or (role == "UP_WALL" and dy > 0):
            return None
        if role in ACTIVE:
            if role == "WARP_CARPET_DOWN" and direction != "south":
                return None
            return self.warps.get(dest)
        return dest

    def distances(self, start):
        reached = {start: 0}
        queue = deque([start])
        while queue:
            pos = queue.popleft()
            for direction in DIRS:
                dest = self.step(pos, direction)
                if dest is not None and dest not in reached:
                    reached[dest] = reached[pos] + 1
                    queue.append(dest)
        return reached


def check_inputs(room, label, stand):
    event, = [e for e in room["events"]["bg_events"] if e["script"] == label]
    require(event["event_type"] == "BGEVENT_READ" and
            (event["x"], event["y"] + 1) == stand, f"unbound control {label}")


def native_write(room, label, initial, outcome):
    for accept in (False, True):
        actions, _, final = read_branch(room, label, initial, accept)
        require(final == set(initial) | ({outcome} if accept else set()), f"{label} yes/no")
        if accept:
            write = actions.index(("setevent", outcome))
            require(("promptbutton",) in actions[:write], f"{label} unacknowledged write")
        else:
            require(not any(a[0] == "changeblock" for a in actions), f"{label} opens on refusal")
    require(read_branch(room, label, set(initial) | {outcome})[2] == set(initial) | {outcome},
            f"{label} repeat")


def check_rescue(room, terrain, reachable, legacy):
    """Trace both control approaches and the crew movement from the published movement table."""
    directions = {"UP": "north", "DOWN": "south", "LEFT": "west", "RIGHT": "east"}
    control = (16, 2)
    approaches = [(CORE, control[0]+dx, control[1]+dy) for dx, dy in DIRS.values()]
    approaches = [p for p in approaches if p in reachable]
    require(set(approaches) == {(CORE, 15, 2), (CORE, 16, 3)}, "motor's accessible sides changed")
    shutdown = instructions(room, "MnNDrillStop")
    original = ("applymovement", "PLAYER", "MnNDrillPlayerToCrew")
    if legacy:
        require(original in shutdown, "expected original unconditional shutdown walk")
    else:
        require(("scall", "MnMineWalkToCrew") in shutdown and
                instructions(room, "MnMineWalkToCrew") == [
                    ("readvar", "VAR_XCOORD"), ("ifnotequal", "15", "MnMineFromMotorFront"),
                    ("applymovement", "PLAYER", "MnMineRoundMotor"), ("sjump", "MnMineFromMotorFront"),
                ] and instructions(room, "MnMineFromMotorFront") == [original, ("end",)],
                "motor approach dispatch")
        cond, = [r for r in room["script_control_commands"] if r["source_script"] == "MnMineWalkToCrew"
                 and r["command"] == "ifnotequal"]
        require(cond["compare_value"] == "15" and cond["resolved_target_script"] == "MnMineFromMotorFront",
                "unresolved approach branch")

    def walk(pos, label):
        movement, = [m for m in room["script_movements"] if m["label"] == label]
        rows = [(s["command"], s["direction"]) for s in movement["steps"]]
        require(rows[-1] == ("step_end", None) and
                instructions(room, label) == [(op, d) for op, d in rows[:-1]] + [("step_end",)],
                f"{label} movement wire mismatch")
        blocked = []
        for op, facing in rows[:-1]:
            require(op == "step", f"unreviewed {label} movement {op}")
            direction = directions[facing]
            dx, dy = DIRS[direction]
            target = (CORE, pos[1]+dx, pos[2]+dy)
            if terrain.step(pos, direction) != target:
                blocked.append(target)
            pos = target
        return pos, blocked

    for start in approaches:
        pos, blocks = start, []
        if not legacy and pos[1] == 15:
            pos, blocks = walk(pos, "MnMineRoundMotor")
            require(pos == (CORE, 16, 3) and not blocks, "cutting through the side of the motor")
        pos, blocks = walk(pos, "MnNDrillPlayerToCrew")
        if legacy and start[1] == 15:
            require(pos == (CORE, 8, 8) and {(CORE, 8, 5), (CORE, 8, 8)} <= set(blocks),
                    "old west approach did not cross the engineer and Reed")
        else:
            require(pos == (CORE, 9, 9) and not blocks, f"invalid walk from motor {start}: {blocks}")
    pos, blocks = walk((CORE, 15, 9), "MnNDrillCrewOut")
    require(pos == (CORE, 8, 7) and not blocks, "Finn cannot emerge beside the player")
    print("Motor's front and west approaches:", "reproduced west walk through engineer and Reed."
          if legacy else "both reach the free (9,9) watching square; Finn reaches (8,7).")


def journey(pack, legacy):
    data, rooms = pack["data"], pack["data"]["maps"]
    core = rooms[CORE]
    actor, = [o for o in core["objects"] if o["script"] == FINN]
    row, = [o for o in core["map_event_section_commands"]
            if o["command"] == "object_event" and o["args"][11] == FINN]
    require(data["npcs"][CORE] == core["objects"] and actor["event_flag"] == row["args"][12],
            "split NPC or native Finn flag mismatch")
    program = "MnNDrillCoreActors"
    require(data["map_scripts"][program] == core["scripts"][program]
            and callbacks(core, "OBJECTS") == [program], "unbound Finn callback")
    writers = {(name, op["source_script"], op["command"])
               for name, room in rooms.items() if name.startswith("Mn")
               for op in room["script_flag_commands"]
               if op["flag_id"] == DELIVERY and op["command"] in ("setevent", "clearevent")}
    require(writers == {(DEPOT, "MN_DEPOT_RECIPIENT", "setevent")}, f"receipt writers: {writers}")
    for name, room in rooms.items():
        if not name.startswith("Mn"):
            continue
        linked = {o["object_identifier"] for o in room["objects"] if o["event_flag"] == DELIVERY}
        switches = [op for op in room["script_object_commands"]
                    if op["command"] in ("appear", "disappear") and op["object_id"] in linked]
        require(not switches or (legacy and name == CORE and linked == {FINN}),
                f"{name} actor also writes the receipt")
    for initial in (set(), {OFF}, {OFF, METER}, {OFF, METER, DELIVERY}):
        result, actors, _ = restore(core, "OBJECTS", initial)
        if legacy:
            expected = initial - {DELIVERY} if OFF in initial else initial | {DELIVERY}
            require(actor["event_flag"] == DELIVERY and result == expected, "legacy receipt behavior")
        else:
            present = OFF in initial and DELIVERY not in initial
            require(result == initial, f"drill-room entry rewrote receipt: {initial} -> {result}")
            require(actors.get(FINN) == ((8, 7) if present else None), f"Finn after {initial}")
    print("Finn's entry callback:", "reproduced premature credit and erased delivery." if legacy else
          "receipt preserved before rescue, before and after pickup, and after depot delivery.")

    badge, report = "ENGINE_GLACIERBADGE", "EVENT_MN_HEADWATERS_REPORT"
    public = rooms[PUBLIC]
    for held in (set(), {badge}, {report}, {badge, report}):
        actions, _, flags = read_branch(public, "MN_R09_DEPOT_FOREMAN", held)
        permitted = held == {badge, report}
        require((FOREMAN in flags) == permitted and (("startbattle",) in actions) == permitted,
                f"foreman prerequisites {held}")
        if permitted:
            require(actions.index(("startbattle",)) < actions.index(("setevent", FOREMAN)),
                    "foreman unlocked before battle win")
    for room, label, flag, stand in VALVES:
        check_inputs(rooms[room], label, stand)
        native_write(rooms[room], label, set(), flag)
        commands = read_branch(rooms[room], label, (), True)[0]
        immediate = {(int(x), int(y)): int(b) for op, *args in commands if op == "changeblock"
                     for x, y, b in [args]}
        expected = {HAUL: {(18, 16): 13}, VENT: {(18, 12): 7}, PUMP: {(24, 6): 13, (14, 16): 13}}
        require(immediate == expected[room] == restore(rooms[room], "TILES", {flag})[2],
                f"{label} changed on room reentry")
    foreman_open = restore(public, "TILES", {FOREMAN})[2]
    require(foreman_open == {(18, 10): 7} and not restore(public, "TILES", ())[2],
            "public mine shaft not gated by foreman")
    require(Workings(data, ()).roles[PUBLIC, 19, 11] == "WALL", "public shaft already open")
    approach = (APPROACH, 6, 21)
    require(approach not in Workings(data, ()).distances(MOUTH), "foreman bypassed")
    for valves, reply in [(set(), "MnIronSumpGateText"), ({HP}, "MnIronSumpHaulOnlyText"),
                          ({VP}, "MnIronSumpVentOnlyText"), ({HP, VP}, "MnIronSumpOpenText")]:
        flags = {FOREMAN} | valves
        terrain = Workings(data, flags)
        dist = terrain.distances(MOUTH)
        require(approach in dist, "cannot descend from the public mine")
        for name, _, _, stand in VALVES[:2]:
            require((name, *stand) in dist, f"cannot reach {name} valve with {valves}")
        require(read_branch(rooms[SUMP], "MnIronSumpGateRead", flags)[1] == [reply],
                f"sump gauges disagree with {valves}")
        check_inputs(rooms[SUMP], "MnIronSumpGateRead", (32, 8))
        edits = restore(rooms[SUMP], "TILES", flags)[2]
        require(edits == ({(32, 6): 13} if len(valves) == 2 else {}), "sump gate state")
        require(((PUMP, 25, 18) in dist) == (len(valves) == 2), "pressure shutter bypass")
        require((CORE, 4, 15) not in dist, "pump feed bypass")
        if len(valves) == 2:
            print(f"Public doorstep to HAUL / VENT / pump controls: {dist[HAUL,29,12]} / "
                  f"{dist[VENT,13,16]} / {dist[PUMP,25,18]} on-foot inputs.")

    flags = {FOREMAN, HP, VP, FEED}
    terrain = Workings(data, flags)
    start = (CORE, 4, 15)
    require(start in terrain.distances(MOUTH), "core unreachable after isolation")
    witness = core["events"]["coord_events"]
    require({(c["x"], c["y"]) for c in witness} == {(x, 12) for x in range(3, 21)},
            "witness row incomplete")
    for c in witness:
        actions, _, after = read_branch(core, c["script_name"], flags)
        require(after == flags | {SEEN} and ("earthquake", "12") in actions, "missing drill test")
        again, _, after = read_branch(core, c["script_name"], flags | {SEEN})
        require(after == flags | {SEEN} and ("earthquake", "12") not in again, "witness repeats")
    witness_tiles = {(CORE, c["x"], c["y"]) for c in witness}
    old = terrain.occupied
    terrain.occupied = old | witness_tiles
    bypass = terrain.distances(start)
    require((CORE, 8, 9) not in bypass and (CORE, 16, 3) not in bypass, "working drill bypass")
    terrain.occupied = old
    seen = terrain.distances(start)
    require((CORE, 8, 9) in seen and (CORE, 16, 3) in seen, "Reed or motor inaccessible")
    check_rescue(core, terrain, seen, legacy)
    require(any(o["script"] == "MN_NDRILL_REED" and (o["x"], o["y"]) == (8, 8)
                for o in core["objects"]), "wrong rival stand")
    check_inputs(core, "MnNDrillStop", (16, 3))
    for missing in (SEEN, WON, FEED):
        incomplete = (flags | {SEEN, WON}) - {missing}
        require(read_branch(core, "MnNDrillStop", incomplete, True)[2] == incomplete,
                f"motor shutdown without {missing}")
    actions, _, flag_after = read_branch(core, "MN_NDRILL_REED", flags | {SEEN})
    require(flag_after == flags | {SEEN, WON} and
            actions.index(("startbattle",)) < actions.index(("setevent", WON)), "Reed battle")
    flags |= {SEEN, WON}
    native_write(core, "MnNDrillStop", flags, OFF)
    flags.add(OFF)
    native_write(core, FINN, flags, METER)
    flags.add(METER)
    depot = rooms[DEPOT]
    recipient = "MN_DEPOT_RECIPIENT"
    for missing in (METER, FOREMAN):
        require(read_branch(depot, recipient, flags - {missing})[1] == ["MnGrandRapidsDepotMissingText"],
                f"depot accepted without {missing}")
    native_write(depot, recipient, flags, DELIVERY)
    pages = native_pages(depot["scripts"]["MnGrandRapidsDepotReadText"])
    require("DULUTH" in " ".join(word for page in pages for word in page), "missing next destination")
    terrain = Workings(data, flags)
    finn_stand, depot_stand, next_route = (CORE, 9, 7), (DEPOT, 11, 10), (DESCENT, 1, 10)
    require(any(o["script"] == recipient and (o["x"], o["y"]) == (11, 9)
                for o in depot["objects"]), "wrong technician stand")
    require(finn_stand in terrain.distances((CORE, 9, 9)), "cannot speak to the rescued Finn")
    rescue = terrain.distances(finn_stand)
    require(depot_stand in rescue and MOUTH in rescue and next_route in rescue, "no route out")
    delivered = Workings(data, flags | {DELIVERY}).distances(depot_stand)
    require(next_route in delivered and start in delivered, "no Route 10 or return to core")
    print(f"Finn to mine mouth / technician: {rescue[MOUTH]} / {rescue[depot_stand]} inputs. "
          f"After delivery, technician to Route 10 / return core: {delivered[next_route]} / {delivered[start]}.")


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("pack", type=Path)
    parser.add_argument("--legacy", action="store_true", help="require the observed pre-fix receipt corruption")
    args = parser.parse_args()
    journey(load_pack(args.pack), args.legacy)
