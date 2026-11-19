#!/usr/bin/env python3
"""Verify Casey's installed rematch invitation and the original first-win and registration paths."""
import argparse
from copy import deepcopy
import json
from pathlib import Path
import sys

REGION = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(REGION / "tools"))
from audit_story_flow import require
from render_reference import load_pack

ROOM = "MnLeagueChampion"
INVITATION = ROOM + "RematchText"
ACTIVE = "EVENT_MN_LEAGUE_RUN_ACTIVE"
CHAMPION = "EVENT_MN_LEAGUE_CHAMPION"


def c(op, *args):
    return {"command": op, "args": list(args)}


def route(room, active, wins, returning):
    """Read the compiled guard/intro up to the first displayed page."""
    scripts = room["scripts"]
    label = ROOM + "Challenge"
    ix = 0
    value = None
    for _ in range(35):
        command = scripts[label][ix]
        op, args = command["command"], command["args"]
        ix += 1
        if op == "checkevent":
            value = {ACTIVE: active, CHAMPION: returning}[args[0]]
        elif op == "readmem":
            require(args == ["wMnLeagueRun"], "unknown League counter")
            value = wins
        elif op == "writetext":
            return args[0]
        elif op in ("iftrue", "iffalse"):
            if bool(value) == (op == "iftrue"):
                label, ix = args[0], 0
        elif op in ("ifequal", "ifnotequal", "ifgreater"):
            n = int(args[0])
            if {"ifequal": value == n, "ifnotequal": value != n, "ifgreater": value > n}[op]:
                label, ix = args[1], 0
        else:
            require(op in ("faceplayer", "opentext"), f"unexpected intro command {op}")
    raise ValueError("Casey challenge did not resolve to a displayed page")


def check(parent, child):
    orig = parent["data"]
    current = child["data"]
    old = orig["maps"][ROOM]
    room = current["maps"][ROOM]
    pages = json.loads((REGION / "story.json").read_text())["dialogue_overhaul"]["texts"][ROOM][INVITATION]
    expected = [c("text" if i == j == 0 else "line" if j else "para", f'"{line}"')
                for i, page in enumerate(pages) for j, line in enumerate(page)] + [c("done")]
    require(pages[0] == ["CASEY: Welcome", "back, CHAMPION."] and
            all(1 <= len(page) <= 2 and all(line.isascii() and 1 <= len(line) <= 18 for line in page) for page in pages),
            "native invitation pages")
    require(any("MANKATO" in line for page in pages for line in page) and "match?" in pages[-1][-1], "Casey's invitation")
    require(old["scripts"][INVITATION][4] == c("para", '"I have a new plan."'), "parent rematch claim")
    require(room["scripts"][INVITATION] == current["map_scripts"][INVITATION] == expected, "authored room and global pages")
    body = {"label": INVITATION, "commands": [dict(cmd, command_index=i) for i, cmd in enumerate(expected)]}
    revised = deepcopy(old)
    revised["scripts"][INVITATION] = expected
    revised["script_text_bodies"][INVITATION] = body
    require(revised == room, "unrelated dialogue, guard, warp, party request or reward changed in Casey's room")
    for admitted in (False, True):
        for returning in (False, True):
            for wins in range(7):
                wanted = ("NoRunText" if not admitted else "AfterText" if wins >= 5 else "NoRunText" if wins != 4
                          else "RematchText" if returning else "BeforeText")
                require(route(room, admitted, wins, returning) == ROOM + wanted, f"rematch guard {admitted, returning, wins}")
    challenge = room["scripts"]
    require(challenge[ROOM+"Challenge"][-1] == challenge[ROOM+"RematchIntro"][-1] == c("sjump", ROOM+"BattleStart"), "shared battle")
    require(c("loadtrainer", "CHAMPION", "MN_CHAMPION") in challenge[ROOM+"BattleStart"], "native Casey party")
    require(orig["trainers"]["trainers"]["MN_CHAMPION"] == current["trainers"]["trainers"]["MN_CHAMPION"], "Casey's entire team and AI")
    for name in ("MnLeagueLobby", "MnLeagueHallOfFame"):
        require(orig["maps"][name] == current["maps"][name], f"unchanged registration and induction in {name}")
    register = current["maps"]["MnLeagueLobby"]["scripts"]["MnElyRegister"]
    hall = current["maps"]["MnLeagueHallOfFame"]["scripts"]["MnElyRecordVictory"]
    require(c("yesorno") in register and len([cmd for cmd in register if cmd["command"] == "checkflag"]) == 8 and
            c("checkevent", "EVENT_MN_SURVEY_RESTORED") in register and c("setevent", ACTIVE) in register, "registration predicates")
    require(c("ifnotequal", "5", "MnElyRecordDenied") in hall and c("setevent", CHAMPION) in hall and c("halloffame") in hall, "title and induction")
    print("Casey's three native rematch pages match canonical text. All 28 title/run/progress combinations, original challenge and team, registration and Hall of Fame are intact.")


if __name__ == "__main__":
    args = argparse.ArgumentParser(description=__doc__)
    args.add_argument("parent", type=Path)
    args.add_argument("candidate", type=Path)
    paths = args.parse_args()
    check(load_pack(paths.parent), load_pack(paths.candidate))
