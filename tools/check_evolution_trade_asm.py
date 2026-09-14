#!/usr/bin/env python3
"""Compare canonical exported evolution and NPC trade tables with Crystal ASM."""
import csv
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
    data = repository / "apps/web/assets/data"
    manifest = json.loads((data / "content-packs/core-modular.generated.json").read_text())

    def category(name):
        result = {}
        for filename in manifest["files"][name]:
            payload = json.loads((data / filename).read_text())
            if set(result) & set(payload):
                raise ValueError("Duplicate exported keys: " + filename)
            result.update(payload)
        return result

    rows = category("evolutions")
    if any(key != row["species"] for key, row in rows.items()):
        raise ValueError("Evolution payload species does not match its key")
    actual = {key: row["evolutions"] for key, row in rows.items()}
    if actual != expected:
        differences = sorted(key for key in set(actual) | set(expected)
                             if actual.get(key) != expected.get(key))
        raise ValueError("ASM evolution mismatch: " + ", ".join(differences))
    print("All {} species and {} evolution rules exactly match ASM.".format(
        len(expected), sum(map(len, expected.values()))))


    trade_constants = (asm / "constants/npc_trade_constants.asm").read_text()
    trade_ids = re.findall(r"^\s*const (NPC_TRADE_\w+)", trade_constants, re.M)
    trade_lines = re.findall(r"^\s*npctrade (.*)",
                            (asm / "data/events/npc_trades.asm").read_text(), re.M)
    if len(trade_ids) != len(trade_lines):
        raise ValueError("NPC trade constants and records differ in length")
    trades = {}
    for trade_id, line in zip(trade_ids, trade_lines):
        args = next(csv.reader([line], skipinitialspace=True))
        args = [arg.strip() for arg in args]
        if len(args) != 10:
            raise ValueError("Unexpected NPC trade operands: " + line)
        trades[trade_id] = dict(
            dialogSet=args[0], requestedSpecies=args[1], offeredSpecies=args[2],
            nickname=args[3], dvs=[int(args[4].replace("$", "0x"), 0),
                                   int(args[5].replace("$", "0x"), 0)],
            heldItem=args[6], originalTrainerId=int(args[7]),
            originalTrainerName=args[8], genderRequirement=args[9],
        )
    if category("npc_trades") != trades:
        raise ValueError("Exported NPC trade records differ from ASM")
    print("All {} NPC trade records exactly match ASM.".format(len(trades)))


if __name__ == "__main__":
    check(Path(__file__).resolve().parents[2])
