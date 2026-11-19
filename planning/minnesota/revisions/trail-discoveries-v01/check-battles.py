#!/usr/bin/env python3
"""Audit trainer parties referenced by Minnesota map scripts and the five-battle League."""

import argparse
from collections import Counter, defaultdict
import json
from pathlib import Path
import sys

HERE = Path(__file__).resolve().parent
REGION = HERE.parents[1]
sys.path.insert(0, str(REGION / "tools"))
from audit_story_flow import instructions, require
from render_reference import load_pack

REED = "MN_REED_REFINEMENT"
KITE = "MN_R20_T09"
LEAGUE = [f"MN_LEAGUE_{number}" for number in range(1, 5)] + ["MN_CHAMPION"]


def bindings(data):
    result = defaultdict(list)
    for name, room in data["maps"].items():
        if not name.startswith("Mn"):
            continue
        for label, rows in room["scripts"].items():
            if not isinstance(rows, list):
                continue
            for row in rows:
                if isinstance(row, dict) and row.get("command") in ("trainer", "loadtrainer"):
                    trainer_class, trainer_id = row["args"][:2]
                    result[trainer_id].append((name, label, row["command"], trainer_class))
    return result


def moveset(data, member):
    if member["moves"]:
        for move in member["moves"]:
            definition = data["moves"][move["name"]]
            require(
                move["current_pp"] == definition["pp"] and move["pp_ups"] == 0,
                f"unexpected starting PP {move}",
            )
        return [move["name"] for move in member["moves"]]
    # Trainer materialization uses the last four distinct eligible level moves, in order.
    slots = []
    for level, move in data["learnsets"][member["species"]]:
        if level <= member["level"]:
            if move in slots:
                slots.remove(move)
            slots.append(move)
    return slots[-4:]


def check_parties(data, refs, baseline):
    defs = json.loads((REGION / "pokemon/roster.json").read_text())["families"]
    retired = json.loads((REGION / "pokemon/roster-curation.json").read_text())["retired_families"]
    retired_ids = {family["id"] for family in retired}
    roster = {m["name"].upper() for f in defs if f["id"] not in retired_ids for m in f["members"]}
    teams = data["trainers"]["trainers"]
    require(set(refs) == {k for k in teams if k.startswith("MN_")}, "unbound or missing Minnesota team")
    require(
        data["competitive_battles"] and not data.get("minnesota_trainer_scaling", False),
        "changed trainer rules",
    )
    weak_returns, empty_drums = set(), set()
    count = 0
    for name, sources in refs.items():
        team = teams[name]
        require(all(s[3] == team["trainer_class"] for s in sources), f"wrong trainer class {name}")
        require(1 <= len(team["party"]) <= 6, f"invalid team size {name}")
        require(
            all(k is None or k in data["items"] for k in team["items"]), f"unknown trainer item {name}"
        )
        for member in team["party"]:
            species = member["species"]
            require(
                species in roster and 1 <= member["level"] <= 100,
                f"unavailable or invalid partner {name}/{species}",
            )
            require(
                member["item"] is None or member["item"] in data["items"],
                f"bad held item {name}/{species}",
            )
            moves = moveset(data, member)
            damage = [m for m in moves if data["moves"][m]["power"]]
            require(
                1 <= len(moves) == len(set(moves)) <= 4 and damage, f"unusable moves {name}/{species}"
            )
            if "RETURN" in moves and "QUICK_ATTACK" in moves:
                weak_returns.add((name, species))
            if "BELLY_DRUM" in moves and all(
                data["type_categories"]["moves"][m] != "physical" for m in damage
            ):
                empty_drums.add((name, species))
            count += 1
    require(
        weak_returns == ({(REED, "GREYTAG")} if baseline else set()),
        f"28-power Return beside Quick Attack: {weak_returns}",
    )
    require(
        empty_drums == ({(KITE, "CHORIP")} if baseline else set()),
        f"Belly Drum with no physical attack: {empty_drums}",
    )
    counts = Counter(s[2] for sources in refs.values() for s in sources)
    print(
        f"{len(refs)} Minnesota teams, {count} party members; {dict(counts)}. All 20 numbered routes included."
    )
    covered = {
        int(name[3:5])
        for sources in refs.values()
        for name, _, _, _ in sources
        if name.startswith("MnR") and name[3:5].isdigit()
    }
    require(covered == set(range(1, 21)), "missing numbered route in battle inventory")


def check_partners(data, refs, baseline):
    teams = data["trainers"]["trainers"]
    station, label, op, _ = refs[REED][0]
    require(
        len(refs[REED]) == 1
        and (station, label, op) == ("MnStCloudStation", "MN_ARC_AFTERMATH_REED", "loadtrainer"),
        "Reed rematch binding",
    )
    room = data["maps"][station]
    require(
        any((o["x"], o["y"], o["script"]) == (15, 16, label) for o in room["objects"]),
        "Reed station location",
    )
    actions = instructions(room, label)
    for event in ("EVENT_MN_LEAGUE_CHAMPION", "EVENT_MN_REED_CHAMPION_DEFEATED"):
        require(("checkevent", event) in actions[:5], f"rematch lost {event}")
    require(
        actions.index(("yesorno",)) < actions.index(("startbattle",)), "Reed rematch must be optional"
    )
    member = teams[REED]["party"][3]
    require((member["species"], member["level"]) == ("GREYTAG", 57), "Reed's fourth partner")
    require("STRENGTH" in data["pokemon"]["GREYTAG"]["tmhm_learnset"], "illegal Greytag move")
    reed_moves = ["STRENGTH", "LOW_KICK", "QUICK_ATTACK", "ENCORE"]
    design = json.loads((REGION / "story-pokemon.json").read_text())["refinement_reed_challenge"]
    require(design["party"][3]["moves"] == reed_moves, "Reed source diverged")
    require(
        moveset(data, member) == (["RETURN", *reed_moves[1:]] if baseline else reed_moves),
        "Reed's move slot",
    )
    route, label, op, _ = refs[KITE][0]
    require(
        len(refs[KITE]) == 1 and (route, label, op) == ("MnR20C01R00", KITE, "trainer"),
        "Kite route binding",
    )
    (actor,) = [o for o in data["maps"][route]["objects"] if o["script"] == KITE]
    require(
        (actor["x"], actor["y"], actor["radius"], actor["object_type"])
        == (43, 27, 4, "OBJECTTYPE_TRAINER"),
        "Kite's sight encounter",
    )
    team = teams[KITE]["party"]
    require(team[0]["species"] == "REEDRAFT" and "RAIN_DANCE" in moveset(data, team[0]), "rain opener")
    design = json.loads((HERE / "route-trainer.json").read_text())
    require(design["trainer"] == KITE and design["party_slot"] == 2, "Kite design binding")
    partner = design["member"]
    require(
        (team[1]["species"], team[1]["level"])
        == (partner["species"], partner["level"])
        == ("CHORIP", 46),
        "Kite's second partner",
    )
    require([22, "WATER_GUN"] in data["learnsets"]["CHORIP"], "Chorip's native Water Gun")
    require(
        partner["moves"] == ["AMNESIA", "SLUDGE_BOMB", "PSYCHIC_M", "WATER_GUN"], "Kite design moves"
    )
    require(
        moveset(data, team[1])
        == ([*partner["moves"][:3], "BELLY_DRUM"] if baseline else partner["moves"]),
        "Kite rain move",
    )


def award(data, member):
    rules = data["battle_reward_rules"]
    wild = data["pokemon"][member["species"]]["base_exp"] * member["level"] // rules["wild_exp_divisor"]
    return wild * rules["trainer_exp_numerator"] // rules["trainer_exp_denominator"]


def purse(team):
    # Trainer completion pays four shares of the final party slot's reward.
    return 4 * team["base_reward"] * team["party"][-1]["level"]


def check_course(data, refs):
    teams, rooms = data["trainers"]["trainers"], data["maps"]
    station = rooms["MnWinonaFieldStation"]
    starters = {
        (op[1], op[2])
        for label in ("MN_W_CHIKORITA", "MN_W_CYNDAQUIL", "MN_W_TOTODILE")
        for op in instructions(station, label)
        if op[0] == "givepoke"
    }
    require(
        starters == {(species, "5") for species in ("POUCHID", "KINDLIT", "BIRCHIT")}, "opening party"
    )
    story = {
        "MN_REED_WINONA": "MnWinonaFieldStation",
        "MN_REED_ROCHESTER": "MnRochesterPokecenter1F",
        "MN_REED_RELAY": "MnIceRelayFloor",
        "MN_ICE_ENFORCER": "MnMinneapolisDispatchOffice",
        "MN_LODE_COURIER": "MnStPaulUnionDepot",
        "MN_REED_STPAUL": "MnStPaulUnionDepot",
        "MN_LODE_FOREMAN": "MnR09Cave1F",
        "MN_REED_MINE": "MnIronDrillCore",
        "MN_LODE_CHIEF": "MnBoundaryWatersSurveyShelter",
        "MN_REED_BOUNDARY": "MnBoundaryIntake",
    }
    story_ranges = []
    for name, room in story.items():
        require(
            any(r == room and command == "loadtrainer" for r, _, command, _ in refs[name]),
            f"missing {name} in {room}",
        )
        levels = [member["level"] for member in teams[name]["party"]]
        story_ranges.append(f"{name.removeprefix('MN_')} {min(levels)}–{max(levels)}")
    print("Story encounters: " + ", ".join(story_ranges))
    gym_ids = [f"MN_GYM_{i}" for i in range(1, 9)]
    aces = [max(z["level"] for z in teams[k]["party"]) for k in gym_ids]
    require(aces == [12, 16, 22, 25, 30, 35, 40, 45], "review changed first-title gym curve")
    for i, trainer in enumerate(LEAGUE, 1):
        room = "MnLeagueChampion" if i == 5 else f"MnLeagueRoom{i}"
        require(
            any(n == room and op == "loadtrainer" for n, _, op, _ in refs[trainer]),
            f"missing {trainer} in {room}",
        )
        for label in rooms[room]["scripts"]:
            for op, *args in instructions(rooms[room], label):
                require(
                    op not in ("healparty", "pokemart")
                    and not (
                        op in ("jumpstd", "special")
                        and any("heal" in a.lower() or "nurse" in a.lower() for a in args)
                    ),
                    f"new in-run service {room}/{label}",
                )
    require(
        instructions(rooms["MnElyPokecenter1F"], "MnElyNurse") == [("jumpstd", "PokecenterNurseScript")],
        "Ely clinic",
    )
    require(
        ("pokemart", "MARTTYPE_STANDARD", "MART_MN_ELY")
        in instructions(rooms["MnElyMart"], "MnElyClerk"),
        "Ely store",
    )
    require(
        {"REVIVE", "HYPER_POTION", "FULL_HEAL"} <= set(data["marts"]["MART_MN_ELY"]),
        "Ely preparation stock",
    )
    require(
        ("clearevent", "EVENT_MN_LEAGUE_RUN_ACTIVE")
        in instructions(rooms["MnLeagueLobby"], "MnLeagueLobbyResetRun"),
        "retreat to lobby",
    )
    counts = [len(teams[k]["party"]) for k in LEAGUE]
    require(counts == [4, 4, 4, 4, 6], "League party lengths")
    gains = [award(data, member) for k in LEAGUE for member in teams[k]["party"]]
    rematches = [max(z["level"] for z in teams[k + "_REMATCH"]["party"]) for k in gym_ids]
    print(
        f"Gym aces {aces}; League {[max(z['level'] for z in teams[k]['party']) for k in LEAGUE]}; gym rematch aces {rematches}."
    )
    print(
        f"League {len(gains)} opponents: {sum(gains)} active EXP, {sum(x // 2 for x in gains)} for each healthy untraded reserve without an EXP Share."
    )
    print(
        f"Gross prizes: {sum(purse(teams[k]) for k in LEAGUE[:-1])} before Casey; {sum(purse(teams[k]) for k in LEAGUE)} including Casey."
    )
    approach = [teams[k] for k in refs if k.startswith("MN_R13_T")]
    kit = sum(data["items"][key]["price"] for key in ("HYPER_POTION", "REVIVE", "FULL_HEAL"))
    print(
        f"Route 13: {len(approach)} trainers, {sum(map(purse, approach))} gross prize money. Ely Hyper Potion + Revive + Full Heal: {kit}."
    )


def compare(old, new):
    original = old["trainers"]["trainers"]
    for trainer, partner, move_slot, before, after in (
        (REED, 3, 0, "RETURN", "STRENGTH"),
        (KITE, 1, 3, "BELLY_DRUM", "WATER_GUN"),
    ):
        moves = original[trainer]["party"][partner]["moves"]
        require(moves[move_slot]["name"] == before, f"unexpected {trainer} parent move")
        moves[move_slot] = {"name": after, "current_pp": new["moves"][after]["pp"], "pp_ups": 0}
    require(old["trainers"] == new["trainers"], "another trainer slot, item, AI or prize changed")
    for key in (
        "items",
        "moves",
        "learnsets",
        "pokemon",
        "type_categories",
        "battle_reward_rules",
        "marts",
        "wild_encounters",
    ):
        require(old[key] == new[key], f"unrelated battle data changed: {key}")


def main():
    cli = argparse.ArgumentParser(description=__doc__)
    cli.add_argument("pack", type=Path)
    mode = cli.add_mutually_exclusive_group()
    mode.add_argument("--baseline", action="store_true", help="verify the two original defects")
    mode.add_argument(
        "--parent", type=Path, help="verify exactly two move changes against this prior edition"
    )
    args = cli.parse_args()
    data = load_pack(args.pack)["data"]
    refs = bindings(data)
    check_parties(data, refs, args.baseline)
    check_partners(data, refs, args.baseline)
    check_course(data, refs)
    if args.parent:
        compare(load_pack(args.parent)["data"], data)
    print("All selected battle checks passed.")


if __name__ == "__main__":
    main()
