#!/usr/bin/env python3
"""Trace Mina's date, Sera's intervention and the return to the St. Cloud depot."""

import argparse
import json
import runpy
import sys
from collections import deque
from pathlib import Path

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE.parents[1] / "tools"))
from audit_story_flow import native_pages, read_branch, require
from render_reference import load_pack

harbor = runpy.run_path(str(HERE.parent / "lake-observations-v01/check-harbor.py"))
Trail, Roads = harbor["Trail"], harbor["ShorePaths"]
DIRECTIONS, rows, callback = harbor["DIRECTIONS"], harbor["rows"], harbor["callback"]
CITY, DEPOT, STATION = "MnStCloud", "MnStPaulUnionDepot", "MnStCloudStation"
PAUL, MINNEAPOLIS, BRAINERD = "MnStPaul", "MnMinneapolis", "MnBrainerd"
NOTE = "EVENT_MN_DISPATCH_NOTE"
MINER = "EVENT_MN_COURIER_DEFEATED"
PAUL_WIN = "EVENT_MN_REED_STPAUL_DEFEATED"
DATE = "EVENT_MN_DEEP_BORE_MANIFEST_READ"
REPORT = "EVENT_MN_BRAINERD_REPORT"
HELP, RADIO, TOLD = (
    "EVENT_MN_REED_WORKER_RELEASED",
    "EVENT_MN_SERA_RADIO_SENT",
    "EVENT_MN_SERA_RADIO_REPORTED",
)
DRILL, RESTORED, CHAMP = (
    "EVENT_MN_DRILL_STOPPED",
    "EVENT_MN_SURVEY_RESTORED",
    "EVENT_MN_LEAGUE_CHAMPION",
)
LAST_WIN = "EVENT_MN_REED_CHAMPION_DEFEATED"
SERA, REED = "MN_ARC_SC_SERA", "MN_ARC_SC_REED"
CLERK, AFTERMATH = "MN_ST_CLOUD_STATION_CLERK", "MN_ARC_AFTERMATH_REED"


class CentralRoads(Roads):
    def __init__(self, data):
        names = {CITY, DEPOT, STATION, PAUL, MINNEAPOLIS, BRAINERD}
        names.update(
            n for n in data["maps"] if n.startswith(("MnR04C", "MnR05C", "MnR06C"))
        )
        Trail.__init__(self, data, names)
        self.dest, self.forced = {}, set()
        constants = {data["maps"][n]["attributes"]["map_constant"]: n for n in names}
        for n in names:
            room = data["maps"][n]
            # bridge::layer_permission maps either authored deck to ordinary
            # FLOOR for walkers. The axis in its token belongs to water below.
            for line in self.grid[n]:
                for x, kind in enumerate(line):
                    if kind in {"BRIDGE_EW_DECK", "BRIDGE_NS_DECK"}:
                        line[x] = "FLOOR"
            for w in room["events"]["warps"]:
                other = constants.get(w["target_map_constant"])
                if other and other != n:
                    end = data["maps"][other]["events"]["warps"][
                        w["target_warp_id"] - 1
                    ]
                    self.dest[n, w["x"], w["y"]] = other, end["x"], end["y"]
            calls = [
                r["args"][1]
                for r in data["map_scripts"][room["attributes"]["map_scripts_label"]]
                if r["command"] == "callback"
                and r["args"][0] == "MAPCALLBACK_OBJECTS"
                and r["args"][1].startswith("MnPublicRoads")
            ]
            for label in calls:
                for cmd in rows(room, label):
                    if cmd[0] == "disappear":
                        (actor,) = [
                            a
                            for a in room["objects"]
                            if a["object_identifier"] == cmd[1]
                        ]
                        self.bodies[n].discard((actor["x"], actor["y"]))

    def distance(self, start, end):
        todo = deque([(start, 0)])
        visited = {start}
        while todo:
            at, count = todo.popleft()
            if at == end:
                return count
            for direction in DIRECTIONS:
                nxt = self.square(at, direction)
                if nxt is not None and nxt not in visited:
                    visited.add(nxt)
                    todo.append((nxt, count + 1))
        raise ValueError(f"no walking path from {start} to {end}")


def actor_state(room, label, flags):
    initial = {a["object_identifier"]: (a["x"], a["y"]) for a in room["objects"]}
    positions = dict(initial)
    visible = set(positions)
    for op, *args in callback(room, label, flags):
        if op == "disappear":
            visible.remove(args[0])
        elif op == "appear":
            visible.add(args[0])
        else:
            require(op == "moveobject", f"unsupported actor callback {op}")
            positions[args[0]] = tuple(map(int, args[1:]))
    return {k: v for k, v in positions.items() if k in visible}


def visit_actors(room, label, flags):
    # The standard callback reader handles relocation and visibility. Facing
    # only changes the portrait orientation and cannot open a walking square.
    room = dict(
        room,
        scripts={
            k: [r for r in v if r["command"] != "turnobject"]
            for k, v in room["scripts"].items()
        },
    )
    return actor_state(room, label, flags)


def depot_approaches(room, roads):
    miner = "MN_ST_PAUL_DEPOT_COURIER"
    entry = rows(room, "MnOverhaulDepotEntry")
    require(
        entry[:6]
        == [
            ("checkevent", MINER),
            ("iftrue", "MnOverhaulDepotEnd"),
            ("checkflag", "ENGINE_PLAINBADGE"),
            ("iffalse", "MnOverhaulDepotEnd"),
            ("checkevent", NOTE),
            ("iffalse", "MnOverhaulDepotEnd"),
        ],
        "wrong depot entry conditions",
    )
    starts = set()
    for _, column, column_label in (c for c in entry if c[0] == "ifequal"):
        for _, row, label in (c for c in rows(room, column_label) if c[0] == "ifequal"):
            x, y = int(column), int(row)
            starts.add((x, y))
            path = rows(room, label)
            for motion in (c[2] for c in path if c[:2] == ("applymovement", "PLAYER")):
                (movement,) = [
                    m for m in room["script_movements"] if m["label"] == motion
                ]
                for step in movement["steps"]:
                    if step["command"] == "step_end":
                        break
                    require(step["command"] == "step", f"unhandled move in {motion}")
                    dx, dy = DIRECTIONS[step["direction"]]
                    x, y = x + dx, y + dy
                    require(
                        roads.kind(DEPOT, x, y) in {"FLOOR", "WARP_CARPET_DOWN"}
                        and (x, y) not in roads.bodies[DEPOT],
                        f"blocked depot entrance {label}: {(x, y)}",
                    )
            require(
                (x, y) == (19, 16) and path[-1] == ("sjump", miner),
                f"wrong miner approach {label}",
            )
    physical = {(w["x"], w["y"]) for w in room["events"]["warps"]}
    physical.update((e["x"], e["y"]) for e in room["events"]["coord_events"])
    require(
        physical <= starts and (17, 10) in starts and len(starts) == 13,
        "depot must handle every door, train arrival and floor trigger",
    )


def story(data, baseline):
    maps = data["maps"]
    depot, city, station = (maps[k] for k in (DEPOT, CITY, STATION))
    miner, mina = "MN_ST_PAUL_DEPOT_COURIER", "MN_NSTP_MINA"

    def visit(room, label, flags=(), accept=None):
        return read_branch(room, label, flags, accept)

    require(
        visit(depot, miner)[1] == ["MnSPCourierNoteText"],
        "miner must point back to Ellis",
    )
    fight, _, won = visit(depot, miner, {NOTE})
    require(
        fight.index(("loadtrainer", "GRUNTM", "MN_LODE_COURIER"))
        < fight.index(("startbattle",))
        < fight.index(("setevent", MINER))
        and won == {NOTE, MINER},
        "miner victory receipt",
    )
    require(
        visit(depot, mina, won, False)[1][-1] == "MnArcSPDeclinedText", "Reed refusal"
    )
    _, _, later = visit(depot, "MnArcSPBattle", won, False)
    require(
        later == won | {PAUL_WIN},
        "declining Mina's date after Reed must preserve her offer",
    )
    _, text, accepted = visit(depot, mina, later, True)
    require(
        accepted == later | {DATE}
        and text[-1] == "MnNSTPMinaQuestText"
        and visit(depot, mina, accepted)[1] == ["MnNSTPMinaAfterText"],
        "Mina retry and reminder",
    )
    came_back = later | {REPORT}
    ask = "MnNSTPMinaReadyText" if baseline else "MnCentralMinaCompareOfferText"
    handover = "MnNSTPMinaQuestText" if baseline else "MnCentralMinaCompareGivenText"
    acknowledgment = "MnNSTPMinaAfterText" if baseline else "MnCentralMinaLakeLogText"
    _, refusal, declined = visit(depot, mina, came_back, False)
    require(
        refusal == [ask, "MnNSTPMinaDeclinedText"] and declined == came_back,
        "keep the date after refusal",
    )
    _, received, with_both = visit(depot, mina, declined, True)
    require(
        received == [ask, handover]
        and with_both == came_back | {DATE}
        and visit(depot, mina, with_both)[1] == [acknowledgment],
        "Mina should read the lake log on return",
    )
    require(
        visit(depot, mina, with_both | {RESTORED})[1] == ["MnArcMinaAfterText"],
        "preserve Mina's restored-world reply about Finn",
    )
    if not baseline:
        authored = json.loads((HERE.parents[1] / "story.json").read_text())
        for name in (ask, handover, acknowledgment):
            require(
                native_pages(depot["script_text_bodies"][name]["commands"])
                == authored["central_mina_dialogue"][name],
                f"Mina's installed pages disagree with the current story: {name}",
            )
        require(
            native_pages(city["script_text_bodies"]["MnArcSCDeclinedText"]["commands"])
            == authored["dialogue_overhaul"]["texts"][CITY]["MnArcSCDeclinedText"],
            "Sera's installed refusal disagrees with the current story",
        )
    ranger = maps[BRAINERD]
    require(
        ("setevent", "EVENT_MN_BRAINERD_REPORT") in rows(ranger, "MN_BRAINERD_RANGER")
        and not any(
            "winch" in str(v).lower() or "SERA" in str(v)
            for v in ranger["scripts"].values()
        ),
        "review the ranger's actual date/winch branches",
    )
    text = " ".join(
        line
        for page in native_pages(
            city["script_text_bodies"]["MnArcSCDeclinedText"]["commands"]
        )
        for line in page
    ).lower()
    require(
        ("winch" in text and "brainerd" in text)
        if baseline
        else (
            "stay here" in text and "stand beside me" in text and "brainerd" not in text
        ),
        "Sera's refusal should describe the local retry",
    )
    late = {PAUL_WIN, DRILL, RESTORED}
    for flags, placed in [(set(), False), ({PAUL_WIN}, True), (late, baseline)]:
        town_actors = visit_actors(city, "MnArcSCActors", flags)
        require(
            (REED in town_actors) == placed, f"street Reed before/after rescue {flags}"
        )
    require(
        AFTERMATH in visit_actors(station, "MnArcAftermathActor", late)
        and AFTERMATH
        not in visit_actors(station, "MnArcAftermathActor", {PAUL_WIN, DRILL}),
        "station Reed's restored-world appearance",
    )
    for flags, visible in [
        ({PAUL_WIN, DRILL}, "MnRefineSeraMineSafeText"),
        (late, "MnRefineSeraRestoredText"),
    ]:
        if baseline:
            require(
                visit(city, SERA, flags, False)[1][0] == "MnArcSCBeforeText",
                "baseline late first meeting",
            )
        else:
            require(
                visit_actors(city, "MnArcSCActors", flags).get(SERA) == (38, 6)
                and visit(city, SERA, flags)[1] == [visible],
                "stale emergency after Finn is safe",
            )
            for entry in ("MnArcSCEntry24", "MnArcSCEntry25", "MnArcSCScene"):
                commands, lines, after = visit(city, entry, flags)
                require(
                    not lines
                    and after == flags
                    and not any(r[0] == "applymovement" for r in commands),
                    f"obsolete road scene or invented help receipt at {entry}",
                )
    early = {PAUL_WIN, HELP}
    require(
        visit(city, SERA, early)[1] == ["MnRefineSeraHandoffText"],
        "give the clerk and shaft contacts",
    )
    require(
        visit(station, CLERK, early, False)[2] == early, "radio refusal changed state"
    )
    _, sent, done = visit(station, CLERK, early, True)
    require(
        sent[-1] == "MnRefineRadioSentText"
        and done == early | {RADIO}
        and visit(station, CLERK, done)[1] == ["MnRefineRadioRepeatText"],
        "radio and retry",
    )
    _, report, back = visit(city, SERA, done)
    require(
        report == ["MnRefineSeraHeardText"]
        and back == done | {TOLD}
        and visit(city, SERA, back)[1] == ["MnRefineSeraWaitingText"],
        "report to Sera once",
    )
    require(
        visit(station, AFTERMATH, late)[1] == ["MnArcAftermathText"], "pre-title Reed"
    )
    require(
        LAST_WIN not in visit(station, AFTERMATH, late | {CHAMP}, False)[2],
        "postgame refusal",
    )
    fight, _, after = visit(station, AFTERMATH, late | {CHAMP}, True)
    require(
        fight.index(("loadtrainer", "MN_RIVAL_REED", "MN_REED_REFINEMENT"))
        < fight.index(("startbattle",))
        < fight.index(("setevent", LAST_WIN))
        and visit(station, AFTERMATH, after)[1] == ["MnRefineReedAfterText"],
        "Champion's optional Reed encounter and subsequent return",
    )
    team = data["trainers"]["trainers"]["MN_REED_REFINEMENT"]["party"]
    require(
        len(team) == 6
        and team[0]["species"] == "CROOKVEIL"
        and {m["level"] for m in team} == {57, 58, 59},
        "postgame Reed's offer disagrees with his team",
    )


def travel(data, baseline):
    roads = CentralRoads(data)
    maps = data["maps"]
    city = maps[CITY]
    before = visit_actors(city, "MnArcSCActors", set())
    roads.bodies[CITY].difference_update({(24, 5), (25, 5)})
    require(SERA not in before and REED not in before, "early road should be open")
    # Walk from the platform to the physical Brainerd ranger before the St Paul
    # rival battle. This registers an ordinary destination for a later Fly.
    source, ranger = (DEPOT, 15, 20), (BRAINERD, 25, 20)
    out, back = roads.distance(source, ranger), roads.distance(ranger, source)
    require(
        ("setflag", "ENGINE_FLYPOINT_MN_BRAINERD")
        in rows(maps[BRAINERD], "MnBrainerdUnlockFlypoint"),
        "early Brainerd visit must register a flypoint",
    )
    require(
        ("setflag", "ENGINE_STORMBADGE")
        in rows(maps["MnStCloudGym"], "MnStCloudGymLeader")
        and ("verbosegiveitem", "HM_FLY", "1")
        in rows(city, "MN_ST_CLOUD_DEPOT_GUIDEFieldHM"),
        "the Wind Badge and St Cloud HM should allow the alternate return",
    )
    depot_approaches(maps[DEPOT], roads)
    print(
        f"St Paul depot to Brainerd ranger and back over Routes 4, 5 and 6: {out}/{back} walking steps"
    )
    for label, start, player_end in [
        ("MnArcSCEntry24", (24, 6), (24, 7)),
        ("MnArcSCEntry25", (25, 6), (24, 7)),
        (SERA, (24, 4), (24, 4)),
    ]:
        at = {
            k: v
            for k, v in visit_actors(city, "MnArcSCActors", {PAUL_WIN}).items()
            if k not in {"MN_SC_NORTH_WARDEN_WEST", "MN_SC_NORTH_WARDEN_EAST"}
        }
        at["PLAYER"] = start
        commands, texts, flags = read_branch(city, label, {PAUL_WIN}, True)
        require(
            flags == {PAUL_WIN, HELP} and texts[-1] == "MnArcSCReleasedText",
            "release from both lanes",
        )
        for cmd in commands:
            if cmd[0] == "disappear":
                del at[cmd[1]]
            if cmd[0] != "applymovement":
                continue
            _, actor, movement_label = cmd
            (motion,) = [
                m for m in city["script_movements"] if m["label"] == movement_label
            ]
            for step in motion["steps"]:
                if step["command"] == "step_end":
                    break
                require(step["command"] == "step", f"unhandled {movement_label} step")
                dx, dy = DIRECTIONS[step["direction"]]
                xx, yy = at[actor]
                nxt = xx + dx, yy + dy
                require(
                    roads.kind(CITY, *nxt) == "FLOOR"
                    and nxt not in {v for k, v in at.items() if k != actor},
                    f"{actor} crosses {nxt} from entry {start}",
                )
                at[actor] = nxt
        final = (24, 9) if baseline else (38, 6)
        require(
            at[SERA] == final == visit_actors(city, "MnArcSCActors", flags)[SERA]
            and at["PLAYER"] == player_end
            and REED not in at,
            "scene and revisit disagree",
        )
        require(
            read_branch(city, label, {PAUL_WIN}, False)[2] == {PAUL_WIN},
            "decline consumed intervention",
        )
        if not baseline:
            require(
                commands.index(("writetext", "MnArcSCReleasedText"))
                < commands.index(("applymovement", SERA, "MnArcSeraLeaves")),
                "Sera should say goodbye while she is still next to the player",
            )
    roads.bodies[CITY].add(final)
    home = (CITY, final[0], final[1] + 1)
    desk = (STATION, 4, 13)
    print(
        f"Sera's return {final}, clerk and return: {roads.distance(home, desk)}/{roads.distance(desk, home)} steps"
    )
    for goal in [(CITY, 24, 4), (CITY, 25, 4), (STATION, 15, 15)]:
        roads.distance(home, goal)
    require(
        (CITY, 37, 5) in roads.dest and roads.dest[CITY, 37, 5][0] == STATION,
        "depot door",
    )
    if not baseline:
        require(
            abs(final[0] - 37) + abs(final[1] - 5) == 2,
            "Sera should wait next to the station entrance",
        )


if __name__ == "__main__":
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument("pack", type=Path)
    p.add_argument(
        "--baseline",
        action="store_true",
        help="reproduce the original confrontation and handoffs",
    )
    args = p.parse_args()
    d = load_pack(args.pack)["data"]
    story(d, args.baseline)
    travel(d, args.baseline)
    print(
        "Mina and Reed, both road entries, late arrival, repeat radio and optional Champion match verified."
    )
