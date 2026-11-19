#!/usr/bin/env python3
"""Audit the eight installed gym battles, their native sightlines, prize TMs and the Timekeeper art."""

import argparse
import json
import sys
from collections import Counter
from copy import deepcopy
from pathlib import Path

REGION = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(REGION / "tools"))
from render_reference import load_pack

CITIES = ("Rochester", "Mankato", "Minneapolis", "StPaul", "StCloud", "Brainerd", "Bemidji", "Duluth")
TIMEKEEPER = "MN_GYM_1_TRAINER_1"
LESSONS = {
    ("MN_GYM_1", "NIBBIN"): ("QUICK_ATTACK", "HEADBUTT"),
    ("MN_GYM_2", "SODCALF"): ("TAIL_WHIP", "MUD_SLAP"),
    ("MN_GYM_3", "PIPULSE"): ("SPARK", "THUNDERPUNCH"),
    ("MN_GYM_3", "SPARKVEIL"): ("THUNDERBOLT", "THUNDERSHOCK"),
    ("MN_GYM_4", "PATCHEBEE"): ("METAL_CLAW", "STEEL_WING"),
    ("MN_GYM_8", "LUTEBRISK"): ("ICE_BEAM", "ICY_WIND"),
}
NEW_LEARNERS = {
    "STEEL_WING": {
        "RUSTLEBEE",
        "PATCHEBEE",
        "RIVETQUEEN",
        "VANECHICK",
        "SILOHAWK",
        "BURROWBILL",
        "BANKWING",
        "SNOWPIP",
        "SQUALLFIN",
        "WHITEWHIRL",
    },
    "THUNDERPUNCH": {
        "PIPULSE",
        "MILLIMP",
        "FLOURGEIST",
        "PUFFLARE",
        "POPUMMEL",
        "SQUEAKURD",
        "MESABLAST",
    },
}
DIRECTIONS = {"RIGHT": (1, 0), "LEFT": (-1, 0), "UP": (0, -1), "DOWN": (0, 1)}


def require(ok, message):
    if not ok:
        raise ValueError(message)


def check_teams(parent, child):
    old, new = parent["data"], child["data"]
    teams = deepcopy(old["trainers"]["trainers"])
    edits = 0
    for (team, species), (before, after) in LESSONS.items():
        (member,) = [m for m in teams[team]["party"] if m["species"] == species]
        slots = {m["name"]: m for m in member["moves"]}
        require(
            (before in slots) != (after in slots), f"{team} has unexpected lesson moves on {species}"
        )
        if before in slots:
            slots[before].update(name=after, current_pp=new["moves"][after]["pp"], pp_ups=0)
            edits += 1
        require(
            after in new["pokemon"][species]["tmhm_learnset"]
            or any(
                level <= member["level"] and move == after for level, move in new["learnsets"][species]
            ),
            f"{species} cannot legally learn {after} at level {member['level']}",
        )
    teams[TIMEKEEPER]["trainer_class"] = "MN_TIMEKEEPER"
    require(
        teams == new["trainers"]["trainers"],
        "a trainer party, level, AI, item, class or reward changed unexpectedly",
    )
    return edits


def check_learners(parent, child):
    old, new = parent["data"]["pokemon"], child["data"]["pokemon"]
    require(old.keys() == new.keys(), "changed species inventory")
    p = REGION / "pokemon"
    roster = json.loads((p / "roster.json").read_text())
    retired = {f["id"] for f in json.loads((p / "roster-curation.json").read_text())["retired_families"]}
    active = {
        m["name"].upper(): m["id"]
        for f in roster["families"]
        if f["id"] not in retired
        for m in f["members"]
    }
    require(active.keys() <= new.keys(), "active Minnesota species absent from game data")
    profiles = {
        s["id"]: s["tmhm_moves"]
        for s in json.loads((p / "species-profiles.json").read_text())["species"]
    }
    for species, profile in old.items():
        moves = profile["tmhm_learnset"].copy()
        for machine, targets in NEW_LEARNERS.items():
            if species in targets and machine not in moves:
                moves.append(machine)
        require(new[species]["tmhm_learnset"] == moves, f"{species} has undeclared machine changes")
        if species in active:
            for machine in NEW_LEARNERS:
                require(
                    (machine in moves) == (machine in profiles[active[species]]),
                    f"{species}: mismatched machine catalog",
                )
    return Counter(move for species in active for move in new[species]["tmhm_learnset"])


def check_sight(old, room, tilesets):
    attrs = room["attributes"]
    occupied = {(o["x"], o["y"]) for o in room["objects"]}
    name = room["id"]
    sight = 0
    collision = tilesets[attrs["tileset_name"]]["collision"]

    def floor(x, y):
        if not (0 <= x < 2 * attrs["width"] and 0 <= y < 2 * attrs["height"]):
            return False
        b = room["blocks"][y // 2 * attrs["width"] + x // 2]
        return collision[f"{b:02x}"][2 * (y % 2) + x % 2] == "FLOOR"

    require(len(old["objects"]) == len(room["objects"]), f"{name}: gym actor inventory changed")
    for a, b in zip(old["objects"], room["objects"]):
        if b["script"] == TIMEKEEPER:
            a = dict(a, sprite="SPRITE_MN_TIMEKEEPER")
        require(a == b, f"{name}: changed an actor's placement, visibility, script or direction")
        require(floor(b["x"], b["y"]), f"{name}: actor stands off ordinary floor")
        if b["script"].endswith("Leader"):
            require(
                b["radius"] == 0 and b["object_type"] == "OBJECTTYPE_SCRIPT",
                f"{name}: leader must await conversation",
            )
        if b["object_type"] != "OBJECTTYPE_TRAINER":
            continue
        sight += 1
        script = b["script"]
        (event,) = [e for e in room["scripts"][script] if e["command"] == "trainer"]
        binding = room["trainer_scripts"][script]
        require(
            event["args"][:3] == [binding["trainer_class"], script, "EVENT_" + script + "_DEFEATED"]
            and binding["event_flag"] == event["args"][2]
            and binding["trainer_id"] == script,
            f"{name}: {script} lost its native sight event or victory receipt",
        )
        direction = b["sightline_direction_override"]
        require(direction in DIRECTIONS and b["radius"] > 0, f"{script}: no authored sight")
        dx, dy = DIRECTIONS[direction]
        # A solid wall, a warp panel or a patch of sliding ice cannot be the only square
        # from which a student will notice a stationary player. Their unobstructed first
        # square is sufficient even when a longer line also meets a telepad or ice.
        x, y = b["x"] + dx, b["y"] + dy
        require(floor(x, y) and (x, y) not in occupied, f"{script}: no open sightline at {x, y}")
    return sight


def check_tavel(parent, child):
    source = json.loads((REGION / "gyms/battle-lessons-v01/tavel.json").read_text())
    palette = b"".join(i.to_bytes(2, "little") for i in source["palette_rgb555"])
    poses = source["overworld_frames"]
    require(
        [f["pose"] for f in poses]
        == ["down_idle", "up_idle", "left_idle", "down_step", "up_step", "left_step"],
        "incomplete native Timekeeper poses",
    )
    for group, rows in (
        ("trainers", source["battle_rows"]),
        ("sprites", [row for f in poses for row in f["rows"]]),
    ):
        stem = f"gfx/{group}/mn_timekeeper"
        encoded = bytearray()
        for top in range(0, len(rows), 8):
            for left in range(0, len(rows[0]), 8):
                for row in rows[top : top + 8]:
                    color = [int(ch) if ch != "." else 0 for ch in row[left : left + 8]]
                    for plane in (0, 1):
                        encoded.append(
                            sum(((ink >> plane) & 1) << (7 - x) for x, ink in enumerate(color))
                        )
        for suffix, expected in ((".2bpp", encoded), (".gbcpal", palette)):
            require(
                bytes(child["runtime_files"][stem + suffix]) == expected,
                f"incorrect native {group} {suffix}",
            )
        for ext in (".2bpp", ".gbcpal", ".png"):
            key = stem + ext
            if key in parent["runtime_files"]:
                require(
                    parent["runtime_files"][key] == child["runtime_files"][key],
                    f"replaced published Timekeeper {key}",
                )


def audit(parent, child):
    edits = check_teams(parent, child)
    learners = check_learners(parent, child)
    check_tavel(parent, child)
    require(
        learners["STEEL_WING"] == 10 and learners["THUNDERPUNCH"] == 9, "regional prize learner totals"
    )
    d = child["data"]
    students = 0
    for number, city in enumerate(CITIES, 1):
        name = f"Mn{city}Gym"
        old, room = parent["data"]["maps"][name], d["maps"][name]
        require(
            old["blocks"] == room["blocks"] and old["events"] == room["events"],
            f"{name}: changed native room or events",
        )
        students += check_sight(old, room, d["tilesets"])
        (leader,) = [o for o in room["objects"] if o["script"] == name + "Leader"]
        team = f"MN_GYM_{number}"
        program = room["scripts"][leader["script"]]
        require(
            any(
                c["command"] == "loadtrainer"
                and c["args"] == [d["trainers"]["trainers"][team]["trainer_class"], team]
                for c in program
            )
            and any(c["command"] == "startbattle" for c in program),
            f"{name}: leader conversation lost its battle binding",
        )
        (item_id,) = [g["item_id"] for g in room["script_item_grants"] if g["item_id"].startswith("TM_")]
        item = d["items"][item_id]
        prize = item["tmhm_move"]
        require(
            item["script_name"] == "TM_" + prize
            and item["pocket"] == "TM_HM"
            and item["tmhm_index"] > 0
            and item["field_usable"]
            and item["effect"] == "TMHMEffect"
            and prize in d["moves"],
            f"{name}: unusable prize in the bag",
        )
        party = d["trainers"]["trainers"][team]["party"]
        users = [m["species"] for m in party if any(v["name"] == prize for v in m["moves"])]
        require(users and learners[prize] > 0, f"{name}: undemonstrated or unusable prize {prize}")
        print(
            f"{city} ({leader['x']},{leader['y']}): {', '.join(users)} can demonstrate {prize}; "
            f"{learners[prize]} active regional learners."
        )
    require(students == 18, "unexpected gym student inventory")
    print(
        f"All 18 native sight battles and eight talk-only leaders verified. "
        f"{edits} corrected leader move slots; every other battle party, level and AI matches the parent."
    )


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("parent", type=Path)
    parser.add_argument("candidate", type=Path)
    args = parser.parse_args()
    audit(load_pack(args.parent), load_pack(args.candidate))
