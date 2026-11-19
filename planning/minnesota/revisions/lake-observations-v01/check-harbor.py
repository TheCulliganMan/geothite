#!/usr/bin/env python3
"""Trace the shore investigation and its public return paths in a compiled Minnesota edition."""

import argparse
import runpy
import sys
from collections import deque
from pathlib import Path

HERE = Path(__file__).resolve().parent
REGION = HERE.parents[1]
sys.path.insert(0, str(REGION / "tools"))
from audit_story_flow import native_pages, read_branch, require
from render_reference import load_pack

support = runpy.run_path(str(HERE.parent / "northern-trails-v01/check-voyageurs.py"))
Trail, DIRECTIONS = support["Trail"], support["DIRECTIONS"]
DULUTH, SHOP = "MnDuluth", "MnDuluthHarborWorkshop"
MARAIS, CEDAR, GATE = "MnGrandMarais", "MnMaraisCedarApproach", "MnGrandMaraisShoreGate"
BASE, SHELTER, INTAKE = (
    "MnBoundaryWaters",
    "MnBoundaryWatersSurveyShelter",
    "MnBoundaryIntake",
)
KETTLE = "MnDevilsKettle"
DEPOT, SIGNAL = "EVENT_MN_DEPOT_CLEARED", "EVENT_MN_LIGHTHOUSE_SIGNAL"
CHIEF, LINK = "EVENT_MN_SURVEY_CHIEF_DEFEATED", "EVENT_MN_ARC_LINK_SENT"
LANDING, REED = "EVENT_MN_VERDIGRIM_COUNTER_CURRENT", "EVENT_MN_REED_BOUNDARY_DEFEATED"
DRILL, SUMP = "EVENT_MN_VERDIGRIM_DRILL_STOPPED", "EVENT_MN_VERDIGRIM_OUTLET_CLOSED"
RELEASED, RESTORED = "EVENT_MN_KETTLURK_RELEASED", "EVENT_MN_SURVEY_RESTORED"
CAUGHT, LAMP = "EVENT_MN_DEVILS_KETTLE_CAUGHT", "EVENT_MN_GRAND_MARAIS_SIGNAL"
DIRECTIONS_OPPOSITE = {"LEFT": "RIGHT", "RIGHT": "LEFT", "UP": "DOWN", "DOWN": "UP"}
SIMPLE = {
    "FLOOR",
    "TALL_GRASS",
    "BRIDGE",
    "DOOR",
    "LADDER",
    "CAVE",
    "WARP_CARPET_LEFT",
    "WARP_CARPET_RIGHT",
    "WARP_CARPET_DOWN",
    "BRIDGE_EW_DECK",
    "BRIDGE_NS_DECK",
}


def rows(room, label):
    return [(row["command"], *row["args"]) for row in room["scripts"][label]]


def run(room, script, flags, accept=None):
    return read_branch(room, script, flags, accept)


def check_bindings(room, entry):
    """Ensure changed commands and jump targets occur in the decoded runtime tables."""
    indexes = {
        (row["source_script"], row["command_index"]): row
        for table in (
            "script_flag_commands",
            "script_control_commands",
            "script_text_commands",
        )
        for row in room[table]
    }
    for i, (op, *args) in enumerate(rows(room, entry)):
        if op in ("checkevent", "iftrue", "iffalse", "jumptext", "jumptextfaceplayer"):
            row = indexes[entry, i]
            field = (
                "flag_id"
                if op == "checkevent"
                else (
                    "resolved_target_script"
                    if op in ("iftrue", "iffalse")
                    else "text_label"
                )
            )
            require(
                row["command"] == op and args == [row[field]],
                f"stale native {entry} command {i}",
            )
            if field == "text_label":
                text = args[0]
                require(
                    [
                        (c["command"], *c["args"])
                        for c in room["script_text_bodies"][text]["commands"]
                    ]
                    == rows(room, text),
                    f"stale native text {text}",
                )


def road_flag_tests(data, baseline):
    maps = data["maps"]
    harbor, shelter, intake = maps[SHOP], maps[SHELTER], maps[INTAKE]
    stable = {DEPOT}
    _, before, denied = run(harbor, "MN_HARBOR_RECEIVER", {LAMP})
    require(
        before == ["MnOverhaulHarborMissingLogText"] and SIGNAL not in denied,
        "the optional lamp cannot replace the depot log",
    )
    _, initial, stable = run(harbor, "MN_HARBOR_RECEIVER", stable)
    require(
        stable == {DEPOT, SIGNAL} and initial[-1] == "MnOverhaulHarborDirectionsText",
        "harbor call must work with the depot log and no badge or lamp",
    )
    _, earlier, _ = run(shelter, "MN_SURVEY_CHIEF", set())
    require(
        earlier == ["MnBoundarySurveyChiefSignalDeniedText"],
        "Hale should name the Duluth engineer",
    )
    battle, _, stable = run(shelter, "MN_SURVEY_CHIEF", stable)
    require(
        CHIEF in stable
        and battle.index(("loadtrainer", "EXECUTIVEM", "MN_LODE_CHIEF"))
        < battle.index(("startbattle",))
        < battle.index(("setevent", CHIEF)),
        "reserve the mic until Hale loses",
    )
    _, _, declined = run(shelter, "MN_BOUNDARY_SURVEY_RECEIVER", stable, False)
    require(declined == stable, "declined radio is retryable")
    _, _, stable = run(shelter, "MN_BOUNDARY_SURVEY_RECEIVER", stable, True)
    require(
        LINK in stable and RESTORED not in stable,
        "the call opens the hatch, not the League",
    )
    for entry, early, renewed, final in [
        (
            "MN_HARBOR_RECEIVER",
            "MN_HARBOR_RECEIVERDoneText",
            "MnHarborIntakeReturnText",
            "MnHarborSurveyReturnText",
        ),
        (
            "MN_BOUNDARY_SURVEY_ASSISTANT",
            "MnWayStopDriveText",
            "MnShelterCrewBelowText",
            "MnWayCrewCountText",
        ),
        (
            "MN_BOUNDARY_SURVEY_RECEIVER",
            "MnArcLinkAfterText",
            "MnShelterRadioBelowText",
            "MnBoundarySurveyReceiverAlreadyText",
        ),
    ]:
        room = harbor if entry == "MN_HARBOR_RECEIVER" else shelter
        first = stable - {LINK} if room is harbor else stable
        flag = LINK if room is harbor else LANDING
        for state, want in [
            (first, early),
            (first | {flag}, early if baseline else renewed),
            (stable | {LANDING, REED, DRILL, SUMP, RESTORED}, final),
        ]:
            _, visible, after = run(room, entry, state)
            require(
                visible == [want] and after == state,
                f"{room['id']}/{entry} rejoin: {visible} expected {want}",
            )
    entered = run(intake, "MnArcIntakeEnter4", stable)[2]
    require(
        LANDING in entered and "EVENT_MN_INTAKE_FIRST_SEEN" in entered,
        "landing must fire after the opened hatch",
    )
    _, _, refused_reed = run(intake, "MN_ARC_INTAKE_REED", entered, False)
    fight, _, after_reed = run(intake, "MN_ARC_INTAKE_REED", entered, True)
    require(
        REED not in refused_reed and REED in after_reed and ("startbattle",) in fight,
        "Reed refusal and fight",
    )
    revisits = [stable, entered, after_reed]
    for control, flag in [("MnArcIntakeStop", DRILL), ("MnVerdigrimOutlet", SUMP)]:
        _, _, declined = run(intake, control, after_reed, False)
        _, _, advanced = run(intake, control, after_reed, True)
        require(declined == after_reed and flag in advanced, f"retryable {control}")
        after_reed = advanced
        revisits.append(advanced)
    _, _, declined = run(intake, "MnVerdigrimBypass", after_reed, False)
    _, _, completed = run(intake, "MnVerdigrimBypass", after_reed, True)
    require(
        declined == after_reed and {RELEASED, RESTORED} <= completed,
        "acknowledged bypass releases Kettlurk and restores the shore",
    )
    optional = maps[KETTLE]
    for flags in [set(), {RESTORED}, {RELEASED}, {RELEASED, RESTORED, CAUGHT}]:
        actions, _, _ = run(optional, "MN_DEVILS_KETTLURK", flags)
        require(
            ("startbattle",) not in actions,
            "separate release, restoration and uncaught prerequisites",
        )
        require(
            callback(optional, "MnKettleNewVisit", flags)
            == [("disappear", "MN_DEVILS_KETTLURK")],
            "early visit or previous catch must hide Kettlurk",
        )
    require(
        callback(optional, "MnKettleNewVisit", completed)
        == [("appear", "MN_DEVILS_KETTLURK")],
        "an uncaught Kettlurk must return on a fresh visit",
    )
    actions, _, refused = run(optional, "MN_DEVILS_KETTLURK", completed, False)
    require(
        refused == completed, "declining the Kettle encounter must not consume a flag"
    )
    require(("startbattle",) not in actions, "declined optional capture")
    program = rows(optional, "MN_DEVILS_KETTLURK")
    require(
        ("loadwildmon", "KETTLURK", "60") in program
        and ("special", "CheckCaughtCelebi") in program
        and ("disappear", "MN_DEVILS_KETTLURK") in program,
        "distinct battle, catch, and uncaught retreat",
    )
    require(
        ("appear", "MN_DEVILS_KETTLURK") in rows(optional, "MnKettleNewVisit")
        and ("setevent", CAUGHT) in rows(optional, "MnKettleCaught"),
        "next visit or permanent capture",
    )
    if not baseline:
        for room, entries in [
            (harbor, ("MN_HARBOR_RECEIVERDone", "MnHarborIntakeReturn")),
            (
                shelter,
                (
                    "MnWayStopDrive",
                    "MnShelterCrewBelow",
                    "MnArcLinkAfter",
                    "MnShelterRadioBelow",
                ),
            ),
            (maps[BASE], ("MN_BOUNDARY_SOUTH_TRAIL", "MnWayLeagueTrail")),
        ]:
            for entry in entries:
                check_bindings(room, entry)
        pages = lambda label: " ".join(
            " ".join(page) for page in native_pages(maps[BASE]["scripts"][label])
        )
        require(
            "ROUTE 13 to ELY" in pages("MN_BOUNDARY_SOUTH_TRAILText")
            and "eight BADGES" in pages("MnWayLeagueTrailText")
            and "letting" not in pages("MnWayLeagueTrailText"),
            "road sign or restored ranger still advertises a closed public road",
        )

    return [*revisits, completed]


class ShorePaths(Trail):
    def __init__(self, data):
        names = {
            BASE,
            SHELTER,
            INTAKE,
            SHOP,
            DULUTH,
            MARAIS,
            GATE,
            CEDAR,
            KETTLE,
            "MnR12RangerHut",
            "MnR13C00R00",
        }
        names.update(n for n in data["maps"] if n.startswith(("MnR11C", "MnR12C00R")))
        super().__init__(data, names)
        self.dest = {}
        by_constant = {data["maps"][n]["attributes"]["map_constant"]: n for n in names}
        for name in names:
            room = data["maps"][name]
            # Public-road callbacks explicitly remove the historical road guards on map load.
            section = data["map_scripts"][room["attributes"]["map_scripts_label"]]
            for row in section:
                if (
                    row["command"] == "callback"
                    and row["args"][0] == "MAPCALLBACK_OBJECTS"
                    and row["args"][1].startswith("MnPublicRoads")
                ):
                    program = rows(room, row["args"][1])
                    require(
                        all(c[0] in ("disappear", "endcallback") for c in program),
                        "unexpected conditional road",
                    )
                    for _, obj in (c for c in program if c[0] == "disappear"):
                        (target,) = [
                            a for a in room["objects"] if a["object_identifier"] == obj
                        ]
                        self.bodies[name].discard((target["x"], target["y"]))
            for w in room["events"]["warps"]:
                n = by_constant.get(w["target_map_constant"])
                if n is None or name == n:
                    continue
                landing = data["maps"][n]["events"]["warps"][w["target_warp_id"] - 1]
                self.dest[name, w["x"], w["y"]] = (n, landing["x"], landing["y"])
            for c in room["events"]["coord_events"]:
                if c["script_name"] in ("MnKettleTrailEntry", "MnKettleTrailExit"):
                    program = rows(room, c["script_name"])
                    require(program[0][0] == "warp", "Kettle stair trigger")
                    _, const, x, y = program[0]
                    self.dest[name, c["x"], c["y"]] = (
                        by_constant[const],
                        int(x),
                        int(y),
                    )
        self.forced = {
            (CEDAR, 12, 8),
            (CEDAR, 13, 8),
            (KETTLE, 28, 32),
            (KETTLE, 28, 33),
        }

    def square(self, at, direction):
        n, x, y = at
        dx, dy = DIRECTIONS[direction]
        x, y = x + dx, y + dy
        if self.kind(n, x, y) == "OUTSIDE":
            link = self.conns[n].get(support["SIDES"][direction])
            if not link or link["target_map"] not in self.maps:
                return None
            n = link["target_map"]
            width, height = len(self.grid[n][0]), len(self.grid[n])
            shift = 2 * link["offset"]
            x, y = (
                ((0 if dx > 0 else width - 1), y - shift)
                if dx
                else (x - shift, (0 if dy > 0 else height - 1))
            )
        src, dst = self.kind(*at), self.kind(n, x, y)
        if (x, y) in self.bodies[n]:
            return None
        for k, d in [(src, direction), (dst, DIRECTIONS_OPPOSITE[direction])]:
            if k.endswith("_WALL") and d in k.split("_"):
                return None
            if k == "BRIDGE_EW_DECK" and d in ("UP", "DOWN"):
                return None
            if k == "BRIDGE_NS_DECK" and d in ("LEFT", "RIGHT"):
                return None
        if dst not in SIMPLE and not dst.endswith("_WALL"):
            return None
        point = n, x, y
        return (
            self.dest.get(point, point)
            if (
                point in self.forced
                or dst in {"DOOR", "CAVE", "LADDER"}
                or dst == "WARP_CARPET_" + direction
            )
            else point
        )

    def reach(self, start):
        seen = {start}
        q = deque([start])
        while q:
            at = q.popleft()
            for direction in DIRECTIONS:
                nxt = self.square(at, direction)
                if nxt is not None and nxt not in seen:
                    seen.add(nxt)
                    q.append(nxt)
        return seen


def callback(room, entry, flags):
    """Follow the native return callbacks and retain their terrain and actor commands."""
    label = entry
    taken = []
    cond = None
    for _ in range(32):
        for op, *args in rows(room, label):
            if op == "checkevent":
                cond = args[0] in flags
            elif (
                op == "sjump"
                or (op == "iftrue" and cond)
                or (op == "iffalse" and not cond)
            ):
                label = args[0]
                break
            elif op in ("iftrue", "iffalse"):
                pass
            elif op in ("end", "endcallback"):
                return taken
            elif op in ("moveobject", "appear", "disappear", "changeblock"):
                taken.append((op, *args))
            else:
                raise ValueError(f"unhandled return operation {label}: {op}")
        else:
            raise ValueError(f"unterminated callback {label}")
    raise ValueError(f"loop in return callback {entry}")


def intake_returns(data, visits):
    room = data["maps"][INTAKE]
    section = data["map_scripts"][room["attributes"]["map_scripts_label"]]
    callbacks = [
        (row["args"][0], row["args"][1])
        for row in section
        if row["command"] == "callback"
    ]
    require(
        set(callbacks)
        == {
            ("MAPCALLBACK_TILES", "MnArcIntakeActors"),
            ("MAPCALLBACK_NEWMAP", "MnVerdigrimNewVisit"),
        },
        "intake callbacks must run on real re-entry",
    )
    kinds = data["tilesets"][room["attributes"]["tileset_name"]]["collision"]
    for flags in visits:
        trail = ShorePaths(data)
        actors = {a["object_identifier"]: (a["x"], a["y"]) for a in room["objects"]}
        present = set(actors)
        for label in ("MnVerdigrimNewVisit", "MnArcIntakeActors"):
            for op, *args in callback(room, label, flags):
                if op == "changeblock":
                    x, y, num = map(int, args)
                    for i, k in enumerate(kinds[f"{num:02x}"]):
                        trail.grid[INTAKE][y + i // 2][x + i % 2] = k
                elif op == "moveobject":
                    actors[args[0]] = tuple(map(int, args[1:]))
                elif op == "appear":
                    present.add(args[0])
                else:
                    present.discard(args[0])
        trail.bodies[INTAKE] = {actors[key] for key in present}
        # Measure only within the intake so an exit cannot disguise an impassable aisle.
        trail.dest = {}
        start = (INTAKE, 4, 19)
        ground = trail.reach(start)
        if RESTORED in flags:
            require(
                {"MN_ARC_KETTLURK", "MN_ARC_INTAKE_REED"}.isdisjoint(present)
                and actors["MN_ARC_INTAKE_FINN"] == (11, 15)
                and actors["MN_ARC_INTAKE_IVY"] == (9, 15),
                "departed captive and returning crew",
            )
            goal = (INTAKE, 11, 16)
        elif SUMP in flags:
            goal = (INTAKE, 13, 12)
        elif DRILL in flags:
            goal = (INTAKE, 21, 20)
        elif REED in flags:
            goal = (INTAKE, 17, 12)
        elif LANDING in flags:
            goal = (INTAKE, 15, 10)
        else:
            goal = (INTAKE, 4, 17)
        require(
            goal in ground and start in trail.reach(goal),
            f"intake leave and rejoin at {goal}",
        )


def routes(data):
    t = ShorePaths(data)
    start = SHOP, 10, 5
    needs = {
        (DULUTH, 72, 42),
        (MARAIS, 18, 1),
        (CEDAR, 18, 14),
        (GATE, 1, 4),
        ("MnR12RangerHut", 4, 8),
        (BASE, 42, 23),
        (SHELTER, 8, 5),
        (SHELTER, 12, 4),
        (BASE, 32, 43),
        ("MnR13C00R00", 32, 0),
        (KETTLE, 20, 13),
    }
    before = t.reach(start)
    require(
        needs <= before,
        f"unreachable public shore destinations {sorted(needs - before)}",
    )
    require(
        (INTAKE, 4, 19) not in before and t.kind(SHELTER, 13, 8) == "WALL",
        "intake hatch before the radio",
    )
    r = data["maps"][SHELTER]
    event = "MnArcShelterHatch"
    require(
        ("callback", "MAPCALLBACK_TILES", event)
        in [
            (v["command"], *v["args"])
            for v in data["map_scripts"][r["attributes"]["map_scripts_label"]]
        ],
        "hatch must reopen on map entry",
    )
    change = ("changeblock", "12", "8", "10")
    require(
        change in rows(r, event) and change in rows(r, "MN_BOUNDARY_SURVEY_RECEIVER"),
        "immediate and repeat hatch opening",
    )
    for i, k in enumerate(
        data["tilesets"][r["attributes"]["tileset_name"]]["collision"]["0a"]
    ):
        t.grid[SHELTER][8 + i // 2][12 + i % 2] = k
    after = t.reach(start)
    needs.update(
        {
            (INTAKE, 4, 19),
            (INTAKE, 4, 17),
            (INTAKE, 11, 10),
            (INTAKE, 17, 12),
            (INTAKE, 21, 20),
            (INTAKE, 13, 12),
        }
    )
    require(needs <= after, f"unreachable intake or return {sorted(needs - after)}")
    # A reverse traversal over the actual directed steps covers every checkpoint at once.
    reverse = {}
    for a in after:
        for direction in DIRECTIONS:
            b = t.square(a, direction)
            if b in after:
                reverse.setdefault(b, []).append(a)
    seen = {start}
    q = deque([start])
    while q:
        for a in reverse.get(q.popleft(), ()):
            if a not in seen:
                seen.add(a)
                q.append(a)
    require(needs <= seen, f"trapped after leaving a story stop {sorted(needs - seen)}")


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("pack", type=Path)
    parser.add_argument(
        "--baseline",
        action="store_true",
        help="check the previously misleading return prompts",
    )
    args = parser.parse_args()
    data = load_pack(args.pack)["data"]
    revisits = road_flag_tests(data, args.baseline)
    routes(data)
    intake_returns(data, revisits)
    print(
        "Duluth workshop, North Shore road, Grand Marais, Gunflint ranger hut, free shelter, intake, Route 13 and optional Kettlurk shore checked in both directions. First, interrupted and restored replies checked against quest gates."
    )
