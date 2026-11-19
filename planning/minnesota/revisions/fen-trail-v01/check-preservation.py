#!/usr/bin/env python3
"""Compare a lake investigations archive to its Town Returns parent, including shared map art."""
import argparse
import base64
import hashlib
import json
from pathlib import Path
import sys

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE.parents[1] / "tools"))
from audit_story_flow import require
from render_reference import SourceArt, load_pack

FEN = "MnLayeredFen"
PEPIN = "MnMississippiLowerWoodsC00R01"
FIELD = "MnFieldMargin"
ROOMS = {FEN, PEPIN, FIELD, "MnR16C01R00", "MnMississippiLowerWoodsC01R00", "MnMississippiLowerWoodsC01R01"}
MARKER = "northstar-fen-trail-v1"


def differing(a, b):
    return (a.keys() ^ b.keys()) | {k for k in a.keys() & b.keys() if a[k] != b[k]}


def scope(a, b, expected, what):
    actual = differing(a, b)
    require(actual == expected, f"{what}: {sorted(actual)}; expected {sorted(expected)}")


def check(parent, child):
    a, b = load_pack(parent), load_pack(child)
    receipts = [json.loads(p.read_text()) for p in HERE.glob("*-build.json")]
    with parent.open("rb") as fp, child.open("rb") as fc:
        ph, ch = (hashlib.file_digest(f, "sha256").hexdigest() for f in (fp, fc))
    require(any(r["parent_pack_sha256"] == ph and r["expected_pack_sha256"] == ch
                for r in receipts), "archive pair")
    scope(a, b, {"data", "runtime_files", "identity", "report"}, "archive")
    scope(a["report"], b["report"], {"manifests"}, "report")
    require(b["report"]["manifests"] == a["report"]["manifests"] + [MARKER], "lineage")
    scope(a["identity"], b["identity"], {"runtime_modpack_id", "content_hash"}, "identity")
    require(b["identity"]["runtime_modpack_id"] == a["identity"]["runtime_modpack_id"] + "+" + MARKER, "lineage id")
    old, new = a["data"], b["data"]
    scope(old, new, {"tilesets", "maps", "map_blocks", "map_scripts"}, "game data, flags, actors and encounters")
    scope(old["maps"], new["maps"], ROOMS, "room changes")
    scope(old["tilesets"], new["tilesets"], {"johto"}, "tileset changes")
    scripts, layouts = set(), set()
    allowed = {"scripts", "script_flag_commands", "script_control_commands", "script_text_commands",
               "script_text_bodies", "script_runtime_commands", "script_object_commands"}
    for name in ROOMS:
        m, n = (d["maps"][name] for d in (old, new))
        others = {"blocks"} if name in (FEN, PEPIN) else {"events", "map_event_section_commands"} if name == FIELD else set()
        require(differing(m, n) <= (allowed | others), f"unrelated changes to {name}: {differing(m, n)}")
        modified = differing(m["scripts"], n["scripts"])
        scripts |= modified
        for key in modified:
            require(key in n["scripts"] and n["scripts"][key] == new["map_scripts"][key], f"local/global {key}")
        if name == FIELD:
            scope(m["events"], n["events"], {"bg_events", "coord_events"}, "field events")
            for key in ("bg_events", "coord_events"):
                require(n["events"][key][:len(m["events"][key])] == m["events"][key], "existing field events")
            section = n["attributes"]["map_events_label"]
            commands = [{"command": c["command"], "args": c["args"]} for c in n["map_event_section_commands"]]
            require(new["map_scripts"][section] == commands, "split field events")
            scripts.add(section)
        if name in (FEN, PEPIN):
            attr = m["attributes"]
            index = lambda x, y: y * attr["width"] + x
            blocks = list(m["blocks"])
            replacements = ([(6, 6, 0x8a), (6, 9, 0x8b), (15, 6, 0x8c)] if name == FEN else [(12, 5, 0x8d)])
            for x, y, block in replacements:
                i = index(x, y)
                require(old["tilesets"]["johto"]["collision"][f"{blocks[i]:02x}"]
                        == new["tilesets"]["johto"]["collision"][f"{block:02x}"], f"{name} lost field movement")
                blocks[i] = block
            require(blocks == n["blocks"], f"unexpected {name} terrain")
            key = attr["blocks_label"]
            require(bytes(blocks) == base64.b64decode(new["map_blocks"][key]), f"{name} split terrain")
            layouts.add(key)
    scope(old["map_scripts"], new["map_scripts"], scripts, "scripts beyond investigations")
    scope(old["map_blocks"], new["map_blocks"], layouts, "split terrain")
    ob, nb = (d["tilesets"]["johto"] for d in (old, new))
    scope(ob, nb, {"collision", "palette_map"}, "Johto bank")
    scope(ob["collision"], nb["collision"], {"8a", "8b", "8c", "8d"}, "new permissions")
    ga, gb = a["runtime_files"], b["runtime_files"]
    files = {"gfx/tilesets/johto.2bpp", "gfx/tilesets/johto.png"}
    files.update("data/tilesets/johto" + suffix for suffix in (
        ".json", "_metatiles.bin", "_native_backgrounds.json", "_animations.json", "_palette_map.json"))
    scope(ga, gb, files, "native assets")
    native = "data/tilesets/johto_metatiles.bin"
    original = bytes(ga[native])
    require(bytes(gb[native])[:len(original)] == original and len(gb[native]) == len(original) + 64,
            "changes to existing Johto metatiles")
    src, out = SourceArt(a), SourceArt(b)
    for clock in ("morn", "day", "nite"):
        pa, pb = src.blocks("johto", clock), out.blocks("johto", clock)
        require(all(x.tobytes() == y.tobytes() for x, y in zip(pa, pb)), f"Johto pixels changed at {clock}")
    print(f"{len(old['maps'])} maps compared, {len(ROOMS)} investigation maps changed. Four new metatiles; original Johto pixels and movement preserved.")


if __name__ == "__main__":
    cli = argparse.ArgumentParser(description=__doc__)
    cli.add_argument("parent", type=Path)
    cli.add_argument("child", type=Path)
    args = cli.parse_args()
    check(args.parent, args.child)
