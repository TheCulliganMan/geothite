#!/usr/bin/env python3
"""Audit the result checks on Minnesota's compiled map-script Pokémon, Egg and item gifts."""
import argparse
from collections import Counter
from pathlib import Path
import runpy

HERE = Path(__file__).resolve().parent
read_pack = runpy.run_path(str(HERE / "check-pack.py"))["read_pack"]
GIFTS = {"giveitem", "verbosegiveitem", "verbosegiveitemvar", "givepoke", "giveegg"}
MAR = "MnGrandMaraisFieldStation"
BASE_DEFECTS = {(MAR, "MnMaraisPartnerGift", index) for index in [2, 3]}


def event_programs(room):
    """Follow local script references from authored objects, tiles, scenes and callbacks."""
    scripts = room["scripts"]
    pending = {obj["script"] for obj in room["objects"]}
    for row in room["map_event_section_commands"] + room["map_script_section_commands"]:
        pending.update(arg for arg in row["args"] if arg in scripts)
    pending.intersection_update(scripts)
    visited = set()
    while pending:
        label = pending.pop()
        visited.add(label)
        for row in scripts[label]:
            pending.update(arg for arg in row["args"] if arg in scripts and arg not in visited)
    return visited


def verify(pack):
    d = pack["data"]
    child = "northstar-marais-supplies-v1" in pack["report"]["manifests"]
    rooms = {name: m for name, m in d["maps"].items()
             if (m["attributes"]["map_group_constant"] or "").startswith("MAPGROUP_MINNESOTA")}
    counts, unchecked, deferred = Counter(), set(), set()
    for name, room in rooms.items():
        reached = event_programs(room)
        for label, rows in room["scripts"].items():
            for index, ins in enumerate(rows):
                op = ins["command"]
                if op not in GIFTS:
                    continue
                counts[op] += 1
                where = (name, label, index)
                assert label in reached, ("gift has no local event entry", where)
                definitions = room["gift_pokemon_scripts"] if op in ("giveegg", "givepoke") else room["script_item_grants"]
                assert sum(r["source_script"] == label and r["command_index"] == index
                           for r in definitions) == 1, ("unindexed gift", where)
                # A same-map sjump carries the most recent script result to the
                # shared success/failure continuation without modifying it.
                owner, pos = label, index + 1
                visited = set()
                while (owner, pos) not in visited:
                    visited.add((owner, pos))
                    probe = room["scripts"][owner][pos]
                    if probe["command"] != "sjump":
                        break
                    deferred.add(where)
                    owner, pos = probe["args"][0], 0
                if op == "givepoke":
                    good = probe["command"] == "ifequal" and probe["args"][0] == "2"
                elif op == "giveegg":
                    good = probe["command"] == "ifnotequal" and probe["args"][0] == "2"
                else:
                    good = probe["command"] == "iffalse"
                if not good:
                    unchecked.add(where)
                    continue
                target = probe["args"][-1]
                assert target in room["scripts"]
                match = [r for r in room["script_control_commands"]
                         if r["source_script"] == owner and r["command_index"] == pos]
                assert len(match) == 1 and match[0]["resolved_target_script"] == target, where
    assert unchecked == (set() if child else BASE_DEFECTS), unchecked
    assert {(n, k) for n, k, _ in deferred} == {
        ("MnR01C00R00", "MnFossilEarlyExchangeOffer"), ("MnR01C00R00", "MnFossilExchangeSpire"),
        ("MnRochesterFossilLab", "MnLabSecondLens"), ("MnRochesterFossilLab", "MnLabSecondOffer")}
    print(f"{len(rooms)} Minnesota maps, {sum(counts.values())} event-linked gift operations: "
          f"{sum(counts[k] for k in ('giveitem', 'verbosegiveitem', 'verbosegiveitemvar'))} item, "
          f"{counts['givepoke']} Pokémon, {counts['giveegg']} Egg. "
          f"{len(unchecked)} unguarded item results, {len(deferred)} shared fossil result branches.")
    return unchecked


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("packs", nargs="+", type=Path)
    for path in parser.parse_args().packs:
        print(path.name, flush=True)
        verify(read_pack(path))
