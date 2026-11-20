#!/usr/bin/env python3
"""Compare both Ely Return editions to their respective Finder Lesson parents."""
import argparse
from copy import deepcopy
import hashlib
import json
from pathlib import Path
import runpy

import cbor2

HERE = Path(__file__).resolve().parent
REGION = HERE.parents[1]
ROOT = REGION.parents[1]
HALDEN = runpy.run_path(str(HERE / "check-halden.py"))
ROOM = "MnLeagueRoom4"
OLD_TEAM, FIRST_TEAM = "MN_LEAGUE_4", "MN_HALDEN_FIRST"
FLAG = "EVENT_MN_HALDEN_GAVE_TIME"
MANIFEST = "northstar-ely-return-v1"


def read_pack(path, allowed_digests=None):
    raw = path.read_bytes()
    if allowed_digests is not None:
        assert hashlib.sha256(raw).hexdigest() in allowed_digests, "expected a Finder Lesson release parent"
    assert raw[:12] == b"CRYSTALPACK\0" and len(raw) >= 22, "invalid archive header"
    version = int.from_bytes(raw[12:14], "big")
    payload = memoryview(raw)[22:]
    assert len(payload) == int.from_bytes(raw[14:18], "big"), "invalid archive length"
    checksum = 2166136261
    for byte in payload:
        checksum = ((checksum ^ byte) * 16777619) & 0xffffffff
    assert checksum == int.from_bytes(raw[18:22], "big"), "invalid archive checksum"
    # An ordinary dictionary decoder would hide a second archived trainer or
    # event definition occupying the very same key.
    pack = cbor2.loads(payload, allow_duplicate_keys=False)
    assert pack["format_version"] == version
    return pack


def only(old, new, expected, name):
    changed = {key for key in old.keys() | new.keys()
               if key not in old or key not in new or old[key] != new[key]}
    assert changed == expected, (name, "missing", expected - changed, "unexpected", changed - expected)


def first_team(old, new):
    only(old["trainers"], new["trainers"], {"trainers"}, "trainer catalog")
    before, after = old["trainers"]["trainers"], new["trainers"]["trainers"]
    only(before, after, {FIRST_TEAM}, "trainer records")
    assert FIRST_TEAM not in before
    expected = deepcopy(before[OLD_TEAM])
    expected["trainer_id"] = FIRST_TEAM
    member = expected["party"][1]
    assert (member["species"], member["level"]) == ("MUSKETUNGE", 50)
    member["species"] = "MARSHRACK"
    moves = ["MEGA_DRAIN", "BUBBLEBEAM", "STOMP", "GROWTH"]
    eligible = {move for level, move in old["learnsets"][member["species"]] if level <= member["level"]}
    assert set(moves) <= eligible
    member["moves"] = [{"name": move, "current_pp": old["moves"][move]["pp"], "pp_ups": 0}
                       for move in moves]
    assert after[FIRST_TEAM] == expected, "first fight diverges from Halden beyond the agreed partner"
    for trainer in ["MN_CHAMPION", "MN_REED_REFINEMENT", *(f"MN_GYM_{i}_REMATCH" for i in range(1, 9))]:
        assert trainer in before, trainer


def first_battle(old, new):
    HALDEN["verify"](old, new)
    before, after = old["maps"][ROOM], new["maps"][ROOM]
    labels = HALDEN["PROGRAMS"] | HALDEN["TEXT"] | {HALDEN["FIRST"]}
    only(old["map_scripts"], new["map_scripts"], labels, "global executable and text index")
    for table in HALDEN["FIGHT_TABLES"] | HALDEN["TABLES"].keys():
        excluded = {HALDEN["FIRST"]}
        if table in HALDEN["TABLES"]:
            excluded |= HALDEN["PROGRAMS"]
        assert [row for row in before[table] if row["source_script"] not in excluded] == [
            row for row in after[table] if row["source_script"] not in excluded
        ], f"existing {table} rows changed or moved"
    for label in HALDEN["TEXT"]:
        rows = after["scripts"][label]
        assert after["script_text_bodies"][label] == {
            "label": label, "commands": [dict(row, command_index=i) for i, row in enumerate(rows)]}

    # Every pre-existing eligibility check still has its original compiled arguments.
    ask = HALDEN["ASK"]
    cutoff = len(before["scripts"][ask]) - 5
    for table in HALDEN["TABLES"]:
        def existing(room):
            return [row for row in room[table] if row["source_script"] == ask and row["command_index"] < cutoff]
        assert existing(before) == existing(after), table
        for row in after[table]:
            label, index, op = row["source_script"], row["command_index"], row["command"]
            if label not in HALDEN["PROGRAMS"] or (label == ask and index < cutoff):
                continue
            args = after["scripts"][label][index]["args"]
            expected = dict(source_script=label, command_index=index, command=op)
            if table == "script_control_commands":
                target = args[-1] if op != "end" else None
                expected.update(target_label=target, resolved_target_script=target, compare_value=None)
            elif table == "script_text_commands":
                expected["text_label"] = args[0] if op == "writetext" else None
            else:
                expected["flag_id"] = args[0]
            assert row == expected, (table, label, index)


def verify(parent, child):
    only(parent, child, {"data", "report", "identity"}, "archive sections")
    only(parent["report"], child["report"], {"manifests"}, "build report")
    only(parent["identity"], child["identity"], {"content_hash", "runtime_modpack_id"}, "pack identity")
    chain = parent["report"]["manifests"]
    assert chain[-1] == "northstar-finder-lesson-v1"
    assert child["report"]["manifests"] == chain + [MANIFEST]
    assert child["identity"]["runtime_modpack_id"] == "+".join(chain + [MANIFEST])
    old, new = parent["data"], child["data"]
    only(old, new, {"maps", "map_scripts", "story_event_script_constants", "trainers"}, "game data")
    only(old["maps"], new["maps"], {ROOM}, "map modules")
    before, after = old["story_event_script_constants"], new["story_event_script_constants"]
    only(before, after, {"global"}, "event constant tables")
    only(before["global"], after["global"], {FLAG}, "global event constants")
    assert FLAG not in before["global"]
    assert after["global"][FLAG] == max(v for k, v in before["global"].items() if k.startswith("EVENT_")) + 1
    first_team(old, new)
    first_battle(old, new)
    graphics = sum(key.startswith("gfx/") for key in parent["runtime_files"])
    print(f"Verified {len(old['maps'])} maps, all {len(old['trainers']['trainers'])} existing trainers, "
          f"the new first battle, original rematch, {len(parent['runtime_files'])} embedded files "
          f"including {graphics} graphics, and compiled audio.")


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("paths", type=Path, nargs="*", metavar="PACK",
                        help="an explicit parent and child (default: both standard edition pairs)")
    args = parser.parse_args()
    parents = [json.loads((HERE.parent / "finder-lesson-v01" / f"{edition}-build.json").read_text())
               for edition in ("intro", "field")]
    if len(args.paths) == 2:
        pairs = [args.paths]
    elif not args.paths:
        pairs = [(ROOT / p["output_pack"], ROOT / p["output_pack"].replace("-finder-lesson-", "-ely-return-"))
                 for p in parents]
    else:
        parser.error("supply a parent and child together")
    digests = {p["expected_pack_sha256"] for p in parents}
    for parent, child in pairs:
        print(f"Comparing {child.name} to {parent.name}", flush=True)
        verify(read_pack(parent, digests), read_pack(child))
