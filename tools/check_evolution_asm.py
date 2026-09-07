#!/usr/bin/env python3
"""Compare every exported evolution rule with the vendored Crystal ASM."""
import json
from pathlib import Path
import re


def check(repository):
    asm = repository / "vendor/pokecrystal"
    constants = (asm / "constants/pokemon_constants.asm").read_text()
    species = re.findall(r"^\s*const (\w+)", constants.split("DEF NUM_POKEMON EQU")[0], re.M)
    pointers = re.findall(
        r"^\s*dw (\w+)",
        (asm / "data/pokemon/evos_attacks_pointers.asm").read_text(), re.M,
    )
    if len(species) != len(pointers):
        raise ValueError("Species constants and evolution pointers differ in length")
    blocks = dict(re.findall(
        r"^(\w+EvosAttacks):\n(.*?)(?=^\w+EvosAttacks:|\Z)",
        (asm / "data/pokemon/evos_attacks.asm").read_text(), re.M | re.S,
    ))
    expected = {}
    for mon, pointer in zip(species, pointers):
        entries = []
        for line in blocks[pointer].splitlines():
            line = line.split(";")[0].strip()
            if not line:
                continue
            if line == "db 0":
                break
            if not line.startswith("db EVOLVE_"):
                raise ValueError("Unexpected evolution instruction: " + line)
            args = [arg.strip() for arg in line[3:].split(",")]
            method = args[0].replace("EVOLVE_", "", 1)
            entry = dict(method=method, level=None, item=None, held_item=None,
                         happiness=None, stat_ratio=None, species=args[-1])
            if method not in ("LEVEL", "STAT", "ITEM", "TRADE", "HAPPINESS"):
                raise ValueError("Unknown evolution method: " + method)
            if len(args) != (4 if method == "STAT" else 3):
                raise ValueError("Unexpected evolution operands: " + line)
            if method in ("LEVEL", "STAT"):
                entry["level"] = int(args[1])
            if method == "STAT":
                entry["stat_ratio"] = args[2]
            if method == "ITEM":
                entry["item"] = args[1]
            if method == "TRADE":
                entry["held_item"] = args[1]
            if method == "HAPPINESS":
                entry["happiness"] = args[1]
            entries.append(entry)
        else:
            raise ValueError("Missing evolution terminator: " + pointer)
        expected[mon] = entries
    rows = json.loads((repository / "packages/assets/src/data/evolutions.json").read_text())
    actual = {row["species"]: row["evolutions"] for row in rows}
    if len(actual) != len(rows):
        raise ValueError("Duplicate exported evolution species")
    if actual != expected:
        differences = sorted(key for key in set(actual) | set(expected)
                             if actual.get(key) != expected.get(key))
        raise ValueError("ASM evolution mismatch: " + ", ".join(differences))
    print("All {} species and {} evolution rules exactly match ASM.".format(
        len(expected), sum(map(len, expected.values()))))


if __name__ == "__main__":
    check(Path(__file__).resolve().parents[2])
