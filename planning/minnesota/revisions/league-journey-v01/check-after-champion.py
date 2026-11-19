#!/usr/bin/env python3
"""Check Ely's news about Reed and the border watch's report in an installed game."""
import argparse
from itertools import product
import json
from pathlib import Path
import runpy
import sys

HERE = Path(__file__).resolve().parent
REGION = HERE.parents[1]
sys.path.insert(0, str(REGION / "tools"))
from audit_story_flow import BADGES, instructions, native_pages, read_branch, require
from render_reference import load_pack

league = runpy.run_path(str(HERE / "check-league-rooms.py"))
winter = runpy.run_path(str(HERE.parent / "northern-trails-v01/check-voyageurs.py"))
CAMP, ELY, STATION = "MnBoundaryWaters", "MnEly", "MnStCloudStation"
CLINIC = "MnInternationalFallsPokecenter1F"
CHAMP, SAVED = "EVENT_MN_LEAGUE_CHAMPION", "EVENT_MN_SURVEY_RESTORED"
WON = "EVENT_MN_REED_CHAMPION_DEFEATED"
REPORT, ARRIVED, PAID = "EVENT_MN_BORDER_REPORT", "EVENT_MN_FALLS_VOYAGEURS_ARRIVAL", "EVENT_MN_BORDER_CIRCUIT_REWARDED"
RANGER, REED = "MN_BOUNDARY_BORDER_RANGER", "MN_ARC_AFTERMATH_REED"
NEW = {ELY: ["MnAfterChampionReedText"], CAMP: ["MnAfterChampionWatchCrossingText", "MnAfterChampionWatchHomeText"]}


def speak(room, actor, flags):
    _, pages, after = read_branch(room, actor, flags)
    require(after == set(flags), f"directions changed progress at {actor}")
    return pages


def choices(maps, revised):
    for champion, restored, won in product((False, True), repeat=3):
        flags = {f for f, v in [(CHAMP, champion), (SAVED, restored), (WON, won)] if v}
        expected = NEW[ELY][0] if revised and champion and restored and not won else "MnElyRangerText"
        require(speak(maps[ELY], "MnElyRanger", flags) == [expected], f"Ely {flags}")

    for champion, restored, registered, arrived, paid in product((False, True), repeat=5):
        flags = {f for f, v in [(CHAMP, champion), (SAVED, restored), (REPORT, registered),
                               (ARRIVED, arrived), (PAID, paid)] if v}
        if arrived and not paid:
            expected = ["MnFallsWatchArrivedText"]
            if restored and not champion:
                expected.append("MnWayLeagueTrailText")
        elif champion:
            expected = [NEW[CAMP][1] if revised and paid else NEW[CAMP][0] if revised and registered
                        else "MnWayNorthWatchText"]
        elif restored:
            expected = ["MnWayLeagueTrailText"]
        else:
            expected = ["MN_BOUNDARY_BORDER_RANGERText"]
        require(speak(maps[CAMP], RANGER, flags) == expected, f"west watch {flags}, wanted {expected}")


def verify_shore(data, revised):
    name = "MnElyShorePass"
    room = data["maps"][name]
    guide = "MnElyShorePassTrailGuide"
    wanted = "MnAfterChampionShoreGuideText" if revised else "MnElyShorePassTrailGuideText"
    require(speak(room, guide, set()) == [wanted] and speak(room, guide, {CHAMP}) == [wanted],
            "actual two-way shore guide")
    require(any(o["script"] == guide and (o["x"], o["y"]) == (8, 14) for o in room["objects"]),
            "standing guide on the snowy border")
    coast = "MnR11C00R02"
    trail = winter["Trail"](data, {name, coast})
    kinds = set().union(*[set(row) for row in trail.grid[name]])
    require(kinds == {"FLOOR", "WALL", "TALL_GRASS"}, "shore road has no slides, ledges or stairs")
    links = {e["direction"]: e["target_map"] for e in room["attributes"]["connections"]}
    require(links == {"west": "MnEly", "east": "MnR11C00R02"}, "real east and west roads")
    east = data["maps"][coast]
    for dest in ("MN_SPLIT_ROCK_POKECENTER_1F", "MN_R11_SPLIT_ROCK_LIGHTHOUSE"):
        door = next(w for w in east["events"]["warps"] if w["target_map"] == dest
                    and trail.kind(coast, w["x"], w["y"]) == "DOOR")
        goal = (coast, door["x"], door["y"] + 1)
        require(max(trail.both((name, 8, 13), goal)) < 140, f"guide's directions must reach {dest}")
    if revised:
        pages = native_pages(room["script_text_bodies"][wanted]["commands"])
        canonical = json.loads((REGION / "story.json").read_text())["dialogue_overhaul"]["texts"][name]
        require(pages == canonical[wanted] and "SPLIT ROCK" in " ".join(sum(pages, []))
                and room["scripts"][wanted] == data["map_scripts"][wanted]
                and room["scripts"][guide] == data["map_scripts"][guide]
                and room["script_text_commands"].count(dict(source_script=guide, command_index=0,
                    command="jumptextfaceplayer", text_label=wanted)) == 1, "installed shore directions")
    return room["scripts"]["MnElyShorePassTrailGuideText"]


def gym_exit(data, city):
    town = data["maps"]["Mn" + city]
    gym = data["maps"]["Mn" + city + "Gym"]
    inside = league["floorplan"](data, gym)
    outside = league["floorplan"](data, town)
    doors = [w for w in town["events"]["warps"]
             if w["target_map_constant"] == gym["attributes"]["map_constant"]
             and outside[w["x"], w["y"]] == "DOOR"]
    require(len(doors) == 1, f"{city} public gym door")
    door = doors[0]
    arrival = gym["events"]["warps"][door["target_warp_id"] - 1]
    reached = league["walk"](inside, gym, (arrival["x"], arrival["y"]))
    carpets = {pos for pos, role in inside.items() if role == "WARP_CARPET_DOWN"}
    require(len(carpets) == 2 and all(0 < reached.get(pos, 99) <= 3 for pos in carpets),
            f"{city} short walk to both halves of the exit")
    occupied = {(o["x"], o["y"]) for o in town["objects"]}
    for carpet in carpets:
        destination, landing = league["resolve"](data, gym, carpet)
        require(destination["id"] == town["id"] and landing == (door["x"], door["y"] + 1)
                and landing not in occupied and outside[landing] == "FLOOR",
                f"{city} exit should return to the same public doorway")


def offers(data):
    maps = data["maps"]
    hall = instructions(maps["MnLeagueHallOfFame"], "MnElyRecordVictory")
    require(("ifnotequal", "5", "MnElyRecordDenied") in hall and hall.index(("setevent", CHAMP)) < hall.index(("halloffame",)),
            "five wins, then register the title")
    visibility = instructions(maps[STATION], "MnArcAftermathActor")
    require(visibility[:3] == [("checkevent", SAVED), ("iffalse", "MnArcAftermathHide"), ("appear", REED)],
            "Reed's actual station appearance")
    _, before, _ = read_branch(maps[STATION], REED, {SAVED})
    no, declined, retry = read_branch(maps[STATION], REED, {SAVED, CHAMP}, False)
    yes, accepted, victory = read_branch(maps[STATION], REED, retry, True)
    done, repeat, _ = read_branch(maps[STATION], REED, victory)
    require(before == ["MnArcAftermathText"] and declined == ["MnRefineReedOfferText", "MnRefineReedDeclinedText"]
            and ("startbattle",) not in no and WON not in retry, "Reed before title and declined battle")
    require(accepted[0] == "MnRefineReedOfferText" and ("loadtrainer", "MN_RIVAL_REED", "MN_REED_REFINEMENT") in yes
            and yes.index(("reloadmapafterbattle",)) < yes.index(("setevent", WON))
            and ("startbattle",) not in done and repeat == ["MnRefineReedAfterText"], "Reed won/return paths")
    team = data["trainers"]["trainers"]["MN_REED_REFINEMENT"]["party"]
    require(len(team) == 6 and min(p["level"] for p in team) == 57 and max(p["level"] for p in team) == 59
            and any(p["species"] == "CROOKVEIL" for p in team), "actual offered team")
    for i, city in enumerate(("Rochester", "Mankato", "Minneapolis", "StPaul", "StCloud",
                              "Brainerd", "Bemidji", "Duluth"), 1):
        gym_exit(data, city)
        room = f"Mn{city}Gym"
        won, gift = f"EVENT_MN_GYM_{i}_REMATCH_WON", f"EVENT_MN_GYM_{i}_REMATCH_GIFT"
        yes, _, _ = read_branch(maps[room], room + "LeaderAfter", {CHAMP, won, gift}, True)
        require(("startbattle",) in yes and ("giveitem", "RARE_CANDY", "1") not in yes,
                f"{room} repeatable match without second candy")
        require(("giveitem", "LEFTOVERS", "1") in instructions(maps[room], room + "CircuitClaim"),
                f"{room} guide's circuit prize")
    clinic = instructions(maps[CLINIC], "MN_FALLS_RANGER")
    require(("checkevent", PAID) in clinic and ("iftrue", "MnFallsRepeat") in clinic and
            clinic.index(("giveitem", "PP_UP", "1")) < clinic.index(("iffalse", "MnFallsFull")) <
            clinic.index(("setevent", PAID)), "the clinic crosses off only a delivered PP Up")
    stamp = instructions(maps[CAMP], "MnFallsArrivalQualify")
    require({a[1] for a in stamp if a[0] == "checkflag"} == BADGES and ("checkevent", REPORT) in stamp
            and ("checkevent", "EVENT_MN_LIGHTHOUSE_SIGNAL") in stamp and ("setevent", ARRIVED) in stamp,
            "stamp requires a registered, qualified eastbound arrival")


def access(data):
    t = winter["Trail"](data, {CAMP, ELY, STATION, "MnStCloud"})
    maps = data["maps"]
    require(any(a["args"] == ["MAPCALLBACK_OBJECTS", "MnPublicRoadsBoundaryWatersOpen"]
                for a in maps[CAMP]["map_script_section_commands"]), "camp gate reconciliation")
    hidden = {a[1] for a in instructions(maps[CAMP], "MnPublicRoadsBoundaryWatersOpen") if a[0] == "disappear"}
    for actor in maps[CAMP]["objects"]:
        if actor["object_identifier"] in hidden:
            t.bodies[CAMP].discard((actor["x"], actor["y"]))
    spawn_id = data["fly_destinations"]["ENGINE_FLYPOINT_MN_ST_CLOUD"]["destination_spawn_identifier"]
    spawn = data["runtime_spawn_points"][str(spawn_id)]
    city = maps["MnStCloud"]
    station_door = next(w for w in city["events"]["warps"] if w["target_map"] == "MN_ST_CLOUD_STATION")
    inside = next(w for w in maps[STATION]["events"]["warps"] if w["index"] == station_door["target_warp_id"])
    doorstep = next(w for w in city["events"]["warps"] if w["index"] == inside["target_warp_id"])
    fly = (spawn["mapName"], spawn["tileX"], spawn["tileY"])
    require(fly == ("MnStCloud", 24, 32) and (inside["x"], inside["y"]) == (11, 17)
            and t.kind("MnStCloud", station_door["x"], station_door["y"]) == "DOOR", "St. Cloud Fly and depot door")
    print("St. Cloud Fly landing to station doorstep and back", t.both(fly, ("MnStCloud", doorstep["x"], doorstep["y"])))
    for room, actor, start, end in [(ELY, "MnElyRanger", (11, 23), (19, 21)), (STATION, REED, (inside["x"], inside["y"]), (15, 17)),
                                    (CAMP, RANGER, (0, 24), (10, 26)), (CAMP, RANGER, (0, 25), (10, 26))]:
        obj = next(o for o in maps[room]["objects"] if o["script"] == actor)
        require(abs(obj["x"] - end[0]) + abs(obj["y"] - end[1]) == 1 and t.kind(room, *end) == "FLOOR",
                f"stand beside {actor}")
        print(room, start, "to", end, "and back", t.both((room, *start), (room, *end)))


def pages(data):
    canonical = json.loads((REGION / "story.json").read_text())["dialogue_overhaul"]["texts"]
    for name, labels in NEW.items():
        room = data["maps"][name]
        for label in labels:
            commands = room["scripts"][label]
            require(commands == data["map_scripts"][label] and native_pages(commands) == canonical[name][label]
                    and room["script_text_bodies"][label]["commands"] ==
                    [dict(c, command_index=i) for i, c in enumerate(commands)], f"{name}/{label} published pages")
        for label in labels + (["MnElyRanger", "MnAfterChampionSnow"] if name == ELY else
                               ["MnWayNorthWatch", "MnAfterChampionWatchCrossing", "MnAfterChampionWatchHome"]):
            require(room["scripts"][label] == data["map_scripts"][label], f"split program {label}")
            for i, (op, *args) in enumerate(instructions(room, label)):
                kind = {"checkevent": ("script_flag_commands", "flag_id"), "iffalse": ("script_control_commands", "resolved_target_script"),
                        "iftrue": ("script_control_commands", "resolved_target_script"),
                        "jumptextfaceplayer": ("script_text_commands", "text_label")}.get(op)
                if kind:
                    key, field = kind
                    require(any(r["source_script"] == label and r["command_index"] == i and
                                r["command"] == op and r[field] == args[0] for r in room[key]), f"typed {label}/{i}")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("parent", type=Path)
    parser.add_argument("candidate", nargs="?", type=Path)
    args = parser.parse_args()
    parent = load_pack(args.parent)["data"]
    choices(parent["maps"], False)
    old_shore = verify_shore(parent, False)
    data = load_pack(args.candidate)["data"] if args.candidate else parent
    if args.candidate:
        choices(data["maps"], True)
        pages(data)
        require(verify_shore(data, True) == old_shore, "historical shore text was overwritten")
        for n in (STATION, CLINIC):
            require(parent["maps"][n] == data["maps"][n], f"source quest or battle changed in {n}")
    offers(data)
    access(data)
    print("40 ranger states, eight repeatable gym matches and sixteen exit carpets, Reed's invitation/return, clinic receipt and walkable approaches passed.")


if __name__ == "__main__":
    main()
