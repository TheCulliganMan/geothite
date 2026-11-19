#!/usr/bin/env python3
"""Trace fixed Minnesota creature actors to their native art, dialogue, pictures, cries and encounters."""

import argparse
from collections import Counter
from hashlib import sha256
import json
from pathlib import Path
import re
import runpy
import sys

HERE = Path(__file__).resolve().parent
REGION = HERE.parents[1]
sys.path.insert(0, str(REGION / "tools"))
from audit_story_flow import instructions, require
from render_reference import load_pack

SPECIES_OPS = {"cry", "loadwildmon", "givepoke", "giveegg", "pokepic"}
JUMPS = {
    "scall",
    "farscall",
    "sjump",
    "farsjump",
    "iftrue",
    "iffalse",
    "ifequal",
    "ifnotequal",
    "writetext",
    "jumptext",
    "jumptextfaceplayer",
}
CLINIC = ("MnRochesterPokecenter1F", "MN_ARC_GILLPUP")
STATION = "MnWinonaFieldStation"
CAPSULES = {"MN_W_CHIKORITA": "POUCHID", "MN_W_CYNDAQUIL": "KINDLIT", "MN_W_TOTODILE": "BIRCHIT"}


def roster():
    curation = json.loads((REGION / "pokemon/roster-curation.json").read_text())
    retired_ids = {r["id"] for r in curation["retired_families"]}
    live, retired = {}, {}
    for f in json.loads((REGION / "pokemon/roster.json").read_text())["families"]:
        (retired if f["id"] in retired_ids else live).update(
            (m["name"].upper(), m["id"]) for m in f["members"]
        )
    return live, retired


def expected_actors():
    sites = runpy.run_path(str(HERE.parent / "trail-discoveries-v01/check-rare.py"))["SITES"]
    result = {(n, s[0]): s[2] for n, s in sites.items()}
    result.update(
        {
            CLINIC: "LOOBIT",
            ("MnBoundaryIntake", "MN_ARC_KETTLURK"): "KETTLURK",
            ("MnWinona", "MN_RESTORE_WINONA_GILLPUP"): "CHORIP",
            ("MnLeechLake", "MN_RESTORE_LEECH_GILLPUP"): "CHORIP",
            ("MnTallTaleTrail", "MN_TALE_BABEHEMOTH"): "BABEHEMOTH",
            ("MnTallTaleTrail", "MN_TALE_BUNJANT"): "BUNJANT",
        }
    )
    fair = json.loads((REGION / "zones/state-fairgrounds.json").read_text())["native_map"]
    for room in [fair, *fair["interiors"]]:
        for o in room["residents"]:
            if (o["id"].startswith("MnFairExhibit") and "Judge" not in o["id"]) or o["id"] in {
                "MnFairBarnMoomoo",
                "MnFairButterArtist",
            }:
                result[room["id"], o["id"]] = o["sprite"].removeprefix("SPRITE_")
    return result


def speaker(room, start):
    """Follow every local jump, call and text reference, including refusal and return pages."""
    queue, seen = [start], set()
    words, ops = set(), set()
    while queue:
        label = queue.pop()
        if label in seen:
            continue
        seen.add(label)
        for op, *args in instructions(room, label):
            if op in JUMPS:
                queue.append(args[-1])
            elif op in SPECIES_OPS:
                ops.add((op, args[0]))
            elif op in ("text", "para", "line", "cont"):
                words.update(re.findall(r"\b[A-Z][A-Z0-9-]+\b", args[0].upper()))
    return words, ops


def map_operands(room, active, baseline, counts):
    raw, records = {}, {}
    for label in room["scripts"]:
        for i, (op, *args) in enumerate(instructions(room, label)):
            if op in SPECIES_OPS:
                counts[op] += 1
                require(
                    args[0] in active
                    or (baseline and (room["id"], label, op, args[0]) == (*CLINIC, "cry", "GILLPUP")),
                    f"{room['id']}/{label} uses {op} {args[0]}",
                )
                raw[label, i, op] = args[0]
            elif (
                "cry" in op.lower()
                or op == "variablesprite"
                or (op == "special" and any(t in args[0].lower() for t in ("cry", "sprite")))
            ):
                raise ValueError(
                    f"A new dynamic creature or cry needs review: {room['id']}/{label} {op} {args}"
                )
    for r in room["script_audio_commands"]:
        if r["command"] == "cry":
            records[r["source_script"], r["command_index"], "cry"] = r["audio_id"]
    for r in room["scripted_wild_battles"]:
        records[r["source_script"], r["loadwildmon_command_index"], "loadwildmon"] = r["request"][
            "species"
        ]
    for r in room["gift_pokemon_scripts"]:
        records[r["source_script"], r["command_index"], "giveegg" if r["egg"] else "givepoke"] = r[
            "species_id"
        ]
    for r in room["script_runtime_commands"]:
        if r["command"] == "pokepic":
            records[r["source_script"], r["command_index"], "pokepic"] = r["args"][0]
    require(raw == records, f"{room['id']} native species commands differ from script sources")


def audit(pack, baseline):
    data = pack["data"]
    catalog = data["pokemon"]
    active, retired = roster()
    all_ids = set(catalog) | active.keys() | retired.keys()
    regional = {
        row["name"]
        for row in data["runtime_map_metadata"].values()
        if row["groupName"].startswith("MINNESOTA")
    }
    require(
        regional == {n for n in data["maps"] if n.startswith("Mn")},
        "regional metadata/map-name disagreement",
    )
    source = json.loads((REGION / "pokemon/overworld/20261027-v1/accepted.json").read_text())["species"]
    accepted = {s["name"].upper(): s["runtime_files"] for s in source}
    expected = expected_actors()
    actors = {}
    capsules = set()
    art = set()
    counts = Counter()
    subs = 0
    for name in sorted(regional):
        room = data["maps"][name]
        map_operands(room, active, baseline, counts)
        for e in room["script_text_bodies"].values():
            text = " ".join(a["args"][0] for a in e["commands"] if a["args"])
            names = set(re.findall(r"\b[A-Z][A-Z0-9-]+\b", text.upper()))
            require(
                not names & retired.keys(),
                f"{name}/{e['label']} names retired species {names & retired.keys()}",
            )
            subs += 1
        for o in room["objects"]:
            key = (name, o["script"])
            species = o["sprite"].removeprefix("SPRITE_")
            is_capsule = name == STATION and o["script"] in CAPSULES
            if species not in all_ids and key not in expected and not is_capsule:
                continue
            mirrored = [
                row
                for row in data["npcs"][name]
                if row["script"] == o["script"] and row["x"] == o["x"] and row["y"] == o["y"]
            ]
            require(
                len(mirrored) == 1 and mirrored[0]["sprite"] == o["sprite"], f"actor mirror at {key}"
            )
            if is_capsule:
                offered = CAPSULES[o["script"]]
                rows = instructions(room, o["script"])
                preview = [(op, args[0]) for op, *args in rows if op in SPECIES_OPS]
                question = next(args[0] for op, *args in rows if op == "writetext")
                words, _ = speaker(room, question)
                require(
                    species == "POKE_BALL"
                    and offered in words
                    and preview == [(command, offered) for command in ("pokepic", "cry", "givepoke")],
                    f"starter mismatch {key}",
                )
                capsules.add(o["script"])
                continue
            words, ops = speaker(room, o["script"])
            require(
                species in active and species == expected.get(key, species),
                f"unexpected creature art at {key}: {species}",
            )
            if baseline and key == CLINIC:
                require(ops == {("cry", "GILLPUP")}, "clinic cry preimage")
            else:
                require(
                    all(s == species for op, s in ops),
                    f"appearance/encounter mismatch: {key} {species} {ops}",
                )
            require(
                species in words or ("pokepic", species) in ops,
                f"{key} species not identified in text or picture",
            )
            actors[key] = species
            if species in art:
                continue
            path = f"gfx/pokemon/{species.lower()}/overworld.2bpp"
            (recipe,) = [r for r in accepted[species] if r["path"].endswith("/" + path)]
            blob = bytes(pack["runtime_files"][path])
            require(
                len(blob) == recipe["bytes"] and sha256(blob).hexdigest() == recipe["sha256"],
                f"{species} native graphics",
            )
            art.add(species)
    require(
        expected.items() <= actors.items() and capsules == CAPSULES.keys(), "missing creature or starter"
    )
    for species in art | set(CAPSULES.values()):
        cry = "CRY_" + active[species]
        require(
            data["pokemon_cries"][species]["cry"] == cry and cry in pack["audio_manifest"]["cries"],
            f"{species} does not resolve to its own published cry",
        )
    print(
        f"{len(regional)} regional maps, {len(actors)} fixed creatures, {len(art)} verified native walking banks and {len(CAPSULES)} starter stands. {subs} displayed text bodies and {dict(sorted(counts.items()))}."
    )


if __name__ == "__main__":
    cli = argparse.ArgumentParser(description=__doc__)
    cli.add_argument("pack", type=Path)
    cli.add_argument(
        "--baseline",
        action="store_true",
        help="Expect the original Rochester Loobit / Gillpup discrepancy.",
    )
    a = cli.parse_args()
    audit(load_pack(a.pack), a.baseline)
