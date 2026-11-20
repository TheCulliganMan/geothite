#!/usr/bin/env python3
"""Inspect ordinary trainer registrations, Gym challenges and repeated route parties."""
import argparse
from collections import defaultdict
from pathlib import Path
import re
import runpy

HERE = Path(__file__).resolve().parent
read_pack = runpy.run_path(str(HERE.parent / "ely-return-v01/check-pack.py"))["read_pack"]


def party_signature(trainer):
    """Compare what is sent out, ignoring send order and the trainer class's DVs."""
    return tuple(sorted(
        (member["species"], member["level"], member["item"] or "",
         tuple(move["name"] for move in member["moves"]))
        for member in trainer["party"]
    ))


def explicit_challenges(scripts, entry):
    """Follow named, map-local script references to trainer declarations and loads."""
    seen = set()
    pending = [entry]
    battles = set()
    while pending:
        label = pending.pop()
        if label in seen:
            continue
        seen.add(label)
        for row in scripts.get(label, []):
            if row["command"] in {"trainer", "loadtrainer"}:
                battles.add(row["args"][1])
            pending.extend(arg for arg in row["args"] if arg in scripts and arg not in seen)
    return battles


def survey(pack):
    data = pack["data"]
    trainers = data["trainers"]["trainers"]
    flags = {}
    routes = defaultdict(lambda: defaultdict(list))
    students = defaultdict(int)
    leaders = defaultdict(set)
    for name, room in data["maps"].items():
        if not room["attributes"]["map_group_constant"].startswith("MAPGROUP_MINNESOTA"):
            continue
        route = re.fullmatch(r"MnR(\d{2})C\d{2}R\d{2}", name)
        for actor in room["objects"]:
            label = actor["script"]
            if actor["object_type"] != "OBJECTTYPE_TRAINER":
                if name.endswith("Gym"):
                    battles = explicit_challenges(room["scripts"], label)
                    if battles:
                        leaders[name].add((label, tuple(sorted(battles))))
                continue
            [command] = [row for row in room["scripts"][label] if row["command"] == "trainer"]
            group, team, victory, seen, won, lost, after = command["args"]
            assert actor["radius"] > 0, (name, label, "sight range")
            assert trainers[team]["trainer_class"] == group and trainers[team]["party"], (name, team)
            assert victory not in flags, (name, label, victory, flags.get(victory))
            flags[victory] = (name, label)
            assert all(room["script_text_bodies"][key]["commands"] for key in (seen, won, lost)), label
            assert room["scripts"][after], (name, after)
            if route:
                routes[route[1]][party_signature(trainers[team])].append(team)
            if name.endswith("Gym"):
                students[name] += 1
    assert len(students) == 8 and set(leaders) == set(students) and len(routes) == 20
    for room, entries in sorted(leaders.items()):
        [(actor, teams)] = entries
        assert actor == room + "Leader" and all(re.fullmatch(r"MN_GYM_[1-8](?:_REMATCH)?", t) for t in teams)
        print(room, students[room], "students on sight;", " / ".join(teams), "at the leader")
    repeated = []
    for route, parties in sorted(routes.items()):
        for names in parties.values():
            if len(names) > 1:
                repeated.append((route, names))
    count = sum(len(names) for parties in routes.values() for names in parties.values())
    print(f"{len(flags)} ordinary trainers with separate victory flags; "
          f"{sum(students.values())} Gym students; {count} trainers on 20 numbered routes.")
    print("Matching within-route parties (order and DVs excluded):", repeated)


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("packs", nargs="+", type=Path)
    for path in parser.parse_args().packs:
        print(path.name, flush=True)
        survey(read_pack(path))
