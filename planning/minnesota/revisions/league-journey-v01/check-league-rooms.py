#!/usr/bin/env python3
"""Walk the Ely League's installed collision grid and trace its entry, exits, rewards and register."""

import argparse
import base64
from collections import deque
from pathlib import Path
import runpy
import sys

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE.parents[1] / "tools"))
from audit_story_flow import instructions, require
from render_reference import load_pack

admission = runpy.run_path(str(HERE.parent / "lake-observations-v01/check-league.py"))
ROOMS, ACTIVE, COUNT, TITLE = (
    admission[k] for k in ("ROOMS", "ACTIVE", "RUN", "TITLE")
)
HALL, LOBBY, CLINIC = "MnLeagueHallOfFame", "MnLeagueLobby", "MnElyPokecenter1F"
SOUTH = ["FLOOR", "FLOOR", "WARP_CARPET_DOWN", "WARP_CARPET_DOWN"]


def floorplan(data, m, changes=()):
    a = m["attributes"]
    w, h = a["width"], a["height"]
    blocks = m["blocks"].copy()
    for x, y, b in changes:
        blocks[(y // 2) * w + x // 2] = b
    col = data["tilesets"][a["tileset_name"]]["collision"]
    return {
        (x, y): col[f"{blocks[(y//2)*w+x//2]:02x}"][2 * (y % 2) + x % 2]
        for y in range(2 * h)
        for x in range(2 * w)
    }


def adjacent(x, y):
    return ((x - 1, y), (x + 1, y), (x, y - 1), (x, y + 1))


def walk(grid, room, origin):
    actors = {(o["x"], o["y"]) for o in room["objects"]}
    require(
        grid[origin] == "FLOOR" and origin not in actors,
        f"not a clean arrival in {room['id']}",
    )
    distances = {origin: 0}
    q = deque([origin])
    while q:
        p = q.popleft()
        for target in adjacent(*p):
            kind = grid.get(target)
            if (
                target not in distances
                and target not in actors
                and (kind == "FLOOR" or (kind or "").startswith("WARP_CARPET"))
            ):
                distances[target] = 1 + distances[p]
                q.append(target)
    return distances


def resolve(data, m, xy):
    (source,) = [w for w in m["events"]["warps"] if (w["x"], w["y"]) == xy]
    (dest,) = [
        r
        for r in data["maps"].values()
        if r["attributes"]["map_constant"] == source["target_map_constant"]
    ]
    to = dest["events"]["warps"][source["target_warp_id"] - 1]
    return dest, (to["x"], to["y"])


def art(parent, child):
    old, new = parent["runtime_files"], child["runtime_files"]
    path = "data/tilesets/champions_room_metatiles.bin"
    base, updated = bytes(old[path]), bytes(new[path])
    require(
        len(base) == 64 * 16
        and updated
        == base + base[0x31 * 16 : 0x31 * 16 + 8] + base[0x08 * 16 + 8 : 0x09 * 16],
        "ceremonial runner did not preserve its original border and stock foot",
    )
    for key in old:
        if key != path and ("champions_room" in key or "elite_four_room" in key):
            require(old[key] == new[key], "changed stock League art " + key)
    o = parent["data"]["tilesets"]["champions_room"]
    c = child["data"]["tilesets"]["champions_room"]
    require(
        c == dict(o, collision=dict(o["collision"], **{"40": SOUTH})),
        "altered stock Champion tiles or permissions",
    )
    for n, slot in (("MnLeagueRoom3", 0x1C), ("MnLeagueChampion", 0x40)):
        a, b = (p["data"]["maps"][n] for p in (parent, child))
        layout = a["blocks"].copy()
        layout[-3] = slot
        require(
            b == dict(a, blocks=layout)
            and base64.b64decode(
                child["data"]["map_blocks"][b["attributes"]["blocks_label"]],
                validate=True,
            )
            == bytes(layout),
            n + " changed more than its threshold",
        )
    for n in ("LancesRoom", "WillsRoom"):
        require(
            parent["data"]["maps"][n] == child["data"]["maps"][n],
            "changed original Johto League room",
        )


def course(data, fixed):
    maps = data["maps"]
    amounts = []
    require(data["currency_constants"]["MAX_MONEY"] == 999999, "unexpected wallet cap")
    lobby = maps[LOBBY]
    require(
        lobby["attributes"]["environment"] == "GATE"
        and all(
            maps[name]["attributes"]["environment"] == "INDOOR"
            for name in [*ROOMS, HALL]
        ),
        "one of the challenge rooms acquired an outdoor or escape-permitting environment",
    )
    desk = lobby["objects"][0]
    for entry in ((1, 4), (1, 5)):
        require(
            floorplan(data, lobby)[(5, 3)] == "COUNTER"
            and (5, 4) in walk(floorplan(data, lobby), lobby, entry)
            and (desk["x"], desk["y"]) == (5, 2),
            "registration unreachable",
        )
    for i, n in enumerate(ROOMS):
        m = maps[n]
        last = i == 4
        door_y = 1 if last else 2
        bottom = 2 * m["attributes"]["height"] - 1
        (person,) = m["objects"]
        from_pos = (4, bottom - 1)
        require(
            (person["x"], person["y"], person["script"]) == (5, 7, n + "Challenge"),
            n + " challenge spot",
        )
        callback = instructions(m, n + "RefreshDoor")
        require(
            callback[:4]
            == [
                ("checkevent", ACTIVE),
                ("iffalse", n + "CloseDoor"),
                ("readmem", COUNT),
                ("ifless", str(i + 1), n + "CloseDoor"),
            ],
            n + " door changed ownership",
        )

        def delta(label):
            (row,) = [
                x for x in m["script_block_changes"] if x["source_script"] == n + label
            ]
            return row["x"], row["y"], row["block_id"]

        closed, won = delta("CloseDoor"), delta("RefreshDoor")
        require(won == delta("BattleStart"), n + " win does not match saved-open gate")
        for opened in (False, True):
            grid = floorplan(data, m, [won if opened else closed])
            reachable = walk(grid, m, from_pos)
            require(
                (5, 8) in reachable
                and (4, bottom - 1) in reachable
                and (5, bottom - 1) in reachable,
                n + " player cannot reach challenger or return",
            )
            for x in (4, 5):
                door = (x, door_y)
                toe = (x, bottom)
                require(
                    grid[door] == ("LADDER" if opened else "WALL")
                    and (x, door_y + 1) in reachable,
                    n + " northern gate allows bypass or blocks earned ascent",
                )
                want = LOBBY if i == 0 else ROOMS[i - 1]
                dest, place = resolve(data, m, toe)
                require(
                    dest["id"] == want
                    and place == ((1, 4 + x - 4) if i == 0 else (x, 4)),
                    n + " wrong backward room or lane",
                )
                if fixed or n not in ("MnLeagueRoom3", "MnLeagueChampion"):
                    require(
                        grid[toe] == "WARP_CARPET_DOWN"
                        and toe in reachable
                        and (x, bottom + 1) not in grid,
                        n + " cannot deliberately leave via " + str(toe),
                    )
                else:
                    known = "LADDER" if n == "MnLeagueRoom3" and x == 5 else "FLOOR"
                    require(grid[toe] == known, "unexpected parent return permission")
                dest, place = resolve(data, m, door)
                require(
                    dest["id"] == (HALL if last else ROOMS[i + 1])
                    and place[0] == x
                    and floorplan(data, dest)[place] == "FLOOR",
                    n + " forward warp unsafe",
                )
        battle = m["scripted_trainer_battles"][0]
        wire = instructions(m, battle["source_script"])
        require(
            wire[battle["loadtrainer_command_index"]]
            == (
                "loadtrainer",
                battle["request"]["trainer_class"],
                battle["request"]["trainer_id"],
            )
            and wire[battle["startbattle_command_index"]] == ("startbattle",)
            and not any(
                q[0] in ("loadvar", "setevent", "giveitem", "healparty", "givemoney")
                for q in wire
            ),
            n + " changed ordinary loss or prizes",
        )
        t = data["trainers"]["trainers"][battle["request"]["trainer_id"]]
        require(
            len(t["party"]) == (6 if last else 4)
            and t["trainer_class"] == battle["request"]["trainer_class"],
            n + " party length or trainer class",
        )
        amounts.append(t["base_reward"] * t["party"][-1]["level"] * 4)
    require(amounts == [2304, 1568, 1568, 2448, 5300], "changed base prizes")
    if fixed:
        print(
            "Five closed gates, five earned gates, ten forward and ten deliberate return lanes."
        )
    else:
        print(
            "Parent reproduces both missing Casey return lanes and Veil's asymmetric return."
        )
    print("Ordinary five-match base prizes:", amounts)


def induction(data):
    m = data["maps"][HALL]
    run = instructions(m, "MnElyRecordVictory")
    require(
        run[1:5]
        == [
            ("checkevent", ACTIVE),
            ("iffalse", "MnElyRecordDenied"),
            ("readmem", COUNT),
            ("ifnotequal", "5", "MnElyRecordDenied"),
        ],
        "induction access",
    )
    require(
        run[-5:]
        == [
            ("setevent", TITLE),
            ("clearevent", ACTIVE),
            ("loadmem", COUNT, "0"),
            ("halloffame",),
            ("end",),
        ],
        "missing title or wrong induction order",
    )
    bg = {e["script"]: (e["x"], e["y"]) for e in m["events"]["bg_events"]}
    require(
        instructions(m, "MnElyReadRegister")[-1] == ("sjump", "MnElyRecordVictory")
        and bg["MnElyReadRegister"] == (6, 4),
        "book bypasses recorder",
    )
    for x in (4, 5):
        public = walk(floorplan(data, m), m, (x, 12))
        require(
            (5, 6) in public and (6, 5) in public and (11, 7) in public,
            "Hall register, recorder or portrait inaccessible",
        )
    target = data["story_event_script_constants"]["maps"][HALL][
        "GEOTHITE_POST_CREDITS_SPAWN"
    ]
    spot = data["runtime_spawn_points"][str(target)]
    require(
        (spot["mapName"], spot["tileX"], spot["tileY"]) == (CLINIC, 3, 5),
        "credits send the player to the wrong map",
    )
    for name in (LOBBY, CLINIC):
        room = data["maps"][name]
        l = name + "ResetRun"
        require(
            any(
                row["command"] == "callback"
                and row["args"] == ["MAPCALLBACK_NEWMAP", l]
                for row in room["map_script_section_commands"]
            ),
            "lost loss/exit callback " + name,
        )
        require(
            instructions(room, l)
            == [
                ("clearevent", ACTIVE),
                ("setval", "0"),
                ("writemem", COUNT),
                ("endcallback",),
            ],
            "loss/withdrawal also changes a badge, supplies or title",
        )
    print(
        "Recording requires five registered wins, writes the title once, keeps the party's record, and binds Continue to the Ely clinic (3,5)."
    )


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("parent", type=Path)
    parser.add_argument("candidate", type=Path, nargs="?")
    args = parser.parse_args()
    parent = load_pack(args.parent)
    child = load_pack(args.candidate) if args.candidate else parent
    if args.candidate:
        art(parent, child)
    admission["verify"](child["data"])
    course(child["data"], bool(args.candidate))
    induction(child["data"])
