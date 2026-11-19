#!/usr/bin/env python3
"""Compare a Trail Discoveries release with its exact Field or local Intro parent."""

import argparse
import base64
import hashlib
import io
import json
from pathlib import Path
import runpy
import sys

from PIL import Image

HERE = Path(__file__).resolve().parent
REGION = HERE.parents[1]
sys.path.insert(0, str(REGION / "tools"))
from audit_story_flow import native_pages, require
from render_reference import SourceArt, load_pack

PARENTS = {
    "0fae2552c52d5b4b4854f07a63a1e7699a93b5f6cc645d37f70c1ab916422572": "Field",
    "b5decf90a6a45cd87549a5f26820962ef4a521bac1246e423ccfba5ae8394afb": "Intro",
}
TRAIL = "MnTallTaleTrail"
CABIN = "MnBoundaryRootBeerCabin"
PRAIRIE = "MnR02C00R00"
RIDGE = "MnR13C00R00"
WINTER = "mn_outdoor_winter_stairs"
MANIFEST = "northstar-trail-discoveries-v1"
TEXT = {
    CABIN: {"MN_INTERIOR_DOROTHYInteriorText"},
    "MnBoundarySafariC00R02": {"BWS_REED_RETURN_RANGERText"},
    "MnBoundarySafariC01R00": {"BWS_SIGN_05Text"},
    "MnR08C00R00": {"MnDiscoveryR08ClueText"},
    TRAIL: {
        f"MnTaleText_{s}"
        for s in ("find_mark", "find_path", "find_pipe", "find_print", "find_stack", "keeper_ask", "ox_ask")
    },
}
RARE = runpy.run_path(str(HERE / "check-rare.py"))["SITES"]
for name, (_, prompt, *_) in RARE.items():
    TEXT.setdefault(name, set()).add(prompt)

TRAIL_LOGIC = {
    "MnTale" + s
    for s in (
        "FindMark", "FindPath", "FindPipe", "FindPrint", "FindStack", "KeeperAtLog",
        "KeeperReturnsToCamp", "MissingPipe", "Path", "Terrain", "Worker",
    )
}
LANDMARKS = {(11, 14), (8, 7), (14, 10), (14, 4), (14, 8), (15, 6)}


def changes(old, new):
    return (old.keys() ^ new.keys()) | {k for k in old.keys() & new.keys() if old[k] != new[k]}


def scope(old, new, allowed, context):
    diff = changes(old, new)
    require(diff == allowed, f"{context}: missing {sorted(allowed - diff)}, unreviewed {sorted(diff - allowed)}")


def verify_art(parent, child):
    pf, cf = parent["runtime_files"], child["runtime_files"]
    allowed = {
        f"{prefix}/{bank}{suffix}"
        for bank in ("johto", WINTER)
        for prefix, suffix in (("gfx/tilesets", ".2bpp"), ("gfx/tilesets", ".png"), ("data/tilesets", "_metatiles.bin"))
    }
    allowed |= {"data/tilesets/johto_native_backgrounds.json"}
    allowed |= {f"data/tilesets/{WINTER}{s}" for s in (".json", "_foregrounds.json", "_palette_map.json")}
    scope(pf, cf, allowed, "runtime files")
    require(cf.keys() - pf.keys() == {"data/tilesets/johto_native_backgrounds.json"} and not pf.keys() - cf.keys(),
            "only the Johto background lookup may be added")
    ptiles, ctiles = parent["data"]["tilesets"], child["data"]["tilesets"]
    scope(ptiles, ctiles, {"johto", WINTER}, "terrain banks")
    parent_art, child_art = SourceArt(parent), SourceArt(child)
    for bank, original, total, tile_count, count in (("johto", 128, 138, 192, 32), (WINTER, 142, 143, 256, 4)):
        data, gfx = "data/tilesets/" + bank, "gfx/tilesets/" + bank
        old, new = ptiles[bank], ctiles[bank]
        scope(old, new, {"collision", "palette_map"}, bank)
        old_layout, layout = (bytes(f[data + "_metatiles.bin"]) for f in (pf, cf))
        require(len(old_layout) == original * 16 and len(layout) == total * 16 and layout.startswith(old_layout),
                f"{bank} original glyph arrangements")
        scope(old["collision"], new["collision"], {f"{n:02x}" for n in range(original, total)}, bank + " quadrants")
        old_pal, pal = old["palette_map"], new["palette_map"]
        ids = set(range(0xc0, 0xc0 + count))
        require(len(pal) == len(old_pal) and {i for i, (a, b) in enumerate(zip(old_pal, pal)) if a != b} <= ids
                and ids.isdisjoint(old_layout) and ids <= set(layout[original * 16:])
                and all(pal[i] & 8 for i in ids), bank + " spare glyph addresses")
        def physical(i, palette):
            return (i & 127) + tile_count // 2 if palette[i] & 8 else i
        writable = {physical(i, pal) for i in ids}
        require(writable.isdisjoint({physical(i, old_pal) for i in old_layout}), bank + " native VRAM overlap")
        before, after = (bytes(f[gfx + ".2bpp"]) for f in (pf, cf))
        require(len(before) == len(after) == tile_count * 16 and all(
            before[i*16:(i+1)*16] == after[i*16:(i+1)*16] for i in range(tile_count) if i not in writable
        ), bank + " unrelated tile planes")
        sheet = Image.open(io.BytesIO(bytes(cf[gfx + ".png"]))).convert("L")
        width = sheet.width // 8
        require(sheet.width * sheet.height == tile_count * 64, bank + " sheet dimensions")
        for tile in range(tile_count):
            for y in range(8):
                lo, hi = after[tile * 16 + y * 2 : tile * 16 + y * 2 + 2]
                for x in range(8):
                    ink = ((lo >> (7-x)) & 1) | (((hi >> (7-x)) & 1) << 1)
                    require(sheet.getpixel(((tile % width) * 8 + x, (tile // width) * 8 + y)) == 255 - 85 * ink,
                            f"{bank}: indexed PNG / 2bpp mismatch at tile {tile:02x} pixel {x},{y}")
        for clock in ("morn", "day", "nite"):
            a, b = parent_art.blocks(bank, clock), child_art.blocks(bank, clock)
            require(len(a) == original and len(b) == total and all(x.tobytes() == y.tobytes() for x, y in zip(a, b)),
                    f"{bank}: existing terrain at {clock}")
        if bank == WINTER:
            require(json.loads(bytes(cf[data + ".json"])) == new["collision"]
                    and json.loads(bytes(cf[data + "_palette_map.json"])) == pal
                    and new["collision"]["8e"] == old["collision"]["88"], "winter data / runtime mirrors")
            tones = json.loads(bytes(cf[data + "_foregrounds.json"]))
            scope(json.loads(bytes(pf[data + "_foregrounds.json"])), tones, {str(i) for i in ids}, "winter foregrounds")
            require(all(tones[str(i)] == tones[str(0xf6)] for i in ids), "cairn palette donor")
        else:
            shades = json.loads(bytes(cf[data + "_native_backgrounds.json"]))
            require(shades and all(int(i) in ids and v == 2 for i, v in shades.items()), "Johto background donor")


def raw_rows(rows):
    return [{"command": row["command"], "args": row["args"]} for row in rows]


def verify_event_section(room, global_scripts):
    rows = room["map_event_section_commands"]
    label = room["attributes"]["map_events_label"]
    raw = raw_rows(rows)
    require([r["command_index"] for r in rows] == list(range(len(rows)))
            and raw == global_scripts[label] == room["scripts"].get(label, raw), room["id"] + " event section")
    backgrounds = [{"x": int(a[0]), "y": int(a[1]), "event_type": a[2], "script": a[3]}
                   for row in raw if row["command"] == "bg_event" for a in [row["args"]]]
    require(backgrounds == room["events"]["bg_events"], room["id"] + " raw / typed background events")
    return label


def verify_events(name, old, new):
    if name == CABIN:
        require(len(old["objects"]) == len(new["objects"]), "cabin actors")
        for a, b in zip(old["objects"], new["objects"]):
            if a["script"] == "MN_INTERIOR_DOROTHY":
                require(a["sprite"] == "SPRITE_GRANNY" and b == {**a, "sprite": "SPRITE_NURSE"}
                        and (b["x"], b["y"]) == (5, 3), "cabin healer appearance")
            else:
                require(a == b, "unrelated cabin actor")
        def uniform(row):
            if row["command"] == "object_event" and row["args"][11] == "MN_INTERIOR_DOROTHY":
                return {**row, "args": row["args"][:2] + ["SPRITE_NURSE"] + row["args"][3:]}
            return row
        require([uniform(row) for row in old["map_event_section_commands"]] == new["map_event_section_commands"],
                "unrelated cabin actor raw operands")
        return
    scope(old["events"], new["events"], {"bg_events"}, name + " events")
    rows = old["events"]["bg_events"]
    if name == PRAIRIE:
        authored = json.loads((REGION / "routes/MN_R02.json").read_text())["signs"]
        positions = {s["id"]: tuple(s["tile"]) for s in authored}
        require(len(positions) == 5, "expected five Prairie Farm Road boards")
        expected = []
        for row in rows:
            if row["script"] in positions:
                require((row["x"] + 1, row["y"]) == positions[row["script"]] and row["event_type"] == "BGEVENT_READ",
                        "original paper offset " + row["script"])
                row = {**row, "x": row["x"] + 1}
            expected.append(row)
        require(positions.keys() <= {row["script"] for row in rows}, "missing sign")
    else:
        expected = rows + [
            {"x": 29, "y": 16, "event_type": "BGEVENT_READ", "script": "MnTalePath"},
            {"x": 28, "y": 20, "event_type": "BGEVENT_READ", "script": "MnTaleStack"},
        ]
    require(expected == new["events"]["bg_events"], name + " reviewed reads")
    prune = lambda commands: [row for row in raw_rows(commands) if row["command"] not in ("bg_event", "def_bg_events")]
    require(prune(old["map_event_section_commands"]) == prune(new["map_event_section_commands"]),
            name + " unrelated raw event operands")


def verify_blocks(name, old, new, data):
    tileset = new["attributes"]["tileset_name"]
    collision = data["tilesets"][tileset]["collision"]
    width = old["attributes"]["width"]
    old_blocks, new_blocks = old["blocks"], new["blocks"]
    require(len(old_blocks) == len(new_blocks), name + " size")
    delta = {(i % width, i // width): (a, b) for i, (a, b) in enumerate(zip(old_blocks, new_blocks)) if a != b}
    if name == RIDGE:
        require(delta == {(19, 16): (0x88, 0x8e)}, "Cedar Ridge")
    else:
        pond = {(i % width, i // width) for i, v in enumerate(old_blocks)
                if v == 0x35 and 3 <= i % width <= 7 and 3 <= i // width <= 7}
        require(len(pond) == 22 and delta.keys() == pond | LANDMARKS, "Tall Tale shore and field observations")
        for at, (a, b) in delta.items():
            quadrants = list(collision[f"{a:02x}"])
            if at in ((14, 8), (14, 10)):
                quadrants[:2] = ["WALL", "WALL"]
            if at == (14, 4):
                quadrants[0] = "WALL"
            require(collision[f"{b:02x}"] == quadrants, f"unrelated collision at {at}")
        changes = new["script_block_changes"]
        typed = {(r["source_script"], r["command_index"]): (r["x"], r["y"], r["block_id"]) for r in changes}
        raw = {(label, i): tuple(map(int, r["args"])) for label, rows in new["scripts"].items()
               for i, r in enumerate(rows) if r["command"] == "changeblock"}
        gate = new_blocks[8 * width + 14]
        require(len(typed) == len(changes) and raw == typed and gate >= 0x80
                and {v for (label, _), v in typed.items() if label == "MnTaleTerrain"} == {(28, 16, gate)}
                and set(typed.values()) == {(28, 16, gate), (28, 16, 1)}, "closed trunk / cleared footpath")
    key = new["attributes"].get("block_label") or name + "_Blocks"
    require(base64.b64decode(data["map_blocks"][key]) == bytes(new_blocks), name + " raw block mirror")
    return key


def verify_trail_commands(old, new):
    # Raw opcodes and the typed tables are consumed by different native paths.
    # All four families already occur in this map; moveobject is new to the keeper scene.
    for group in ("flag", "control", "object", "text"):
        key = f"script_{group}_commands"
        before, end = old[key], new[key]
        require([r for r in before if r["source_script"] not in TRAIL_LOGIC]
                == [r for r in end if r["source_script"] not in TRAIL_LOGIC],
                f"Tall Tale {group}: unrelated binding")
        opcodes = {r["command"] for r in before}
        if group == "object":
            opcodes.update(("moveobject", "turnobject"))
        expected = {(label, i): row for label, rows in new["scripts"].items() for i, row in enumerate(rows)
                    if row["command"] in opcodes}
        indices = [(r["source_script"], r["command_index"]) for r in end]
        require(len(indices) == len(set(indices)) and set(indices) == expected.keys(), f"Tall Tale {group} indices")
        for index, typed in zip(indices, end):
            raw = expected[index]
            op, args = raw["command"], raw["args"]
            require(typed["command"] == op, f"Tall Tale {group} opcode at {index}")
            if group == "flag":
                require(args == [typed["flag_id"]], f"Tall Tale flag operand at {index}")
            elif group == "control" and typed["target_label"] is not None:
                target = typed["target_label"]
                require(args[-1] == target and target == typed["resolved_target_script"]
                        and target in new["scripts"], f"Tall Tale branch at {index}")
            elif group == "text" and typed["text_label"] is not None:
                require(args == [typed["text_label"]] and args[0] in new["scripts"], f"Tall Tale text at {index}")
            elif group == "object" and index[0] in TRAIL_LOGIC and typed["object_id"] is not None:
                require(args[0] == typed["object_id"], f"Tall Tale actor at {index}")
                if op == "moveobject":
                    require([typed["x"], typed["y"]] == list(map(int, args[1:])), f"keeper move at {index}")
                if op == "turnobject":
                    require(typed["direction"].upper() == args[1], f"keeper turn at {index}")


def verify_maps(a, b, texts):
    names = set(texts) | {PRAIRIE, RIDGE}
    scope(a["maps"], b["maps"], names, "affected maps")
    story = json.loads((REGION / "story.json").read_text())
    all_scripts, all_blocks = set(), set()
    for name in sorted(names):
        old, new = a["maps"][name], b["maps"][name]
        pages = texts.get(name, set())
        fields = {"scripts", "script_text_bodies"} if pages else set()
        if name in (TRAIL, RIDGE):
            fields.add("blocks")
            all_blocks.add(verify_blocks(name, old, new, b))
        if name in (PRAIRIE, TRAIL, CABIN):
            fields.add("map_event_section_commands")
            fields.add("objects" if name == CABIN else "events")
            verify_events(name, old, new)
            all_scripts.add(verify_event_section(new, b["map_scripts"]))
        if name == TRAIL:
            verify_trail_commands(old, new)
            fields.update(("script_flag_commands", "script_object_commands", "script_control_commands",
                           "script_text_commands", "script_block_changes"))
        scope(old, new, fields, name)
        scope(old["script_text_bodies"], new["script_text_bodies"], pages, name + " dialogue")
        for label in pages:
            body = new["script_text_bodies"][label]
            rows = body["commands"]
            require(body["label"] == label and [r["command_index"] for r in rows] == list(range(len(rows)))
                    and raw_rows(rows) == new["scripts"][label], name + "/" + label + " native body")
            sources = (story["story_flow_revision"]["dialogue"] if label == "MnDiscoveryR08ClueText"
                       else story["dialogue_overhaul"]["texts"])
            require(native_pages(rows) == sources[name][label], label + " authored pages")
            if name in RARE and label == RARE[name][1]:
                species = RARE[name][2]
                require(native_pages(old["scripts"][label])[-1][-1].endswith("closer?")
                        and native_pages(rows)[-1][-1] == f"Battle {species}?", label + " voluntary encounter")
        labels = pages | (TRAIL_LOGIC if name == TRAIL else set())
        scope(old["scripts"], new["scripts"], labels, name + " native scripts")
        for label in labels:
            require(new["scripts"][label] == b["map_scripts"][label], label + " global script mirror")
        all_scripts.update(labels)
    scope(a["map_scripts"], b["map_scripts"], all_scripts, "global scripts")
    scope(a["map_blocks"], b["map_blocks"], all_blocks, "global block mirrors")
    scope(a["npcs"], b["npcs"], {CABIN}, "global actor mirrors")
    require(b["npcs"][CABIN] == b["maps"][CABIN]["objects"], "cabin actors")
    return len(names), len(all_scripts)


def verify_trail_palette_limit(pack):
    """Include the partial border tiles of a scrolling 160×144 map camera."""
    room = pack["data"]["maps"][TRAIL]
    width = room["attributes"]["width"]
    bank = pack["data"]["tilesets"]["johto"]["palette_map"]
    files = pack["runtime_files"]
    blocks = bytes(files["data/tilesets/johto_metatiles.bin"])
    backgrounds = json.loads(bytes(files["data/tilesets/johto_native_backgrounds.json"]))
    require("data/tilesets/johto_foregrounds.json" not in files, "unexpected extra Johto foreground palettes")
    rows = []
    for y in range(4 * room["attributes"]["height"]):
        row = []
        for x in range(4 * width):
            block = room["blocks"][(y // 4) * width + x // 4]
            glyph = blocks[16 * block + 4 * (y % 4) + x % 4]
            palette = bank[glyph] & 7
            row.append((palette, backgrounds.get(str(glyph), palette)))
        rows.append(row)
    maximum = max(
        len({entry for row in rows[y : y + 19] for entry in row[x : x + 21]})
        for y in range(len(rows) - 18)
        for x in range(len(rows[0]) - 20)
    )
    require(maximum <= 8, f"Tall Tale camera needs {maximum} background palettes")
    print(f"Tall Tale uses at most {maximum} four-shade background palettes per scrolling GBC camera.")


def verify(parent, child, texts):
    verify_trail_palette_limit(child)
    a, b = parent["data"], child["data"]
    scope(parent, child, {"data", "runtime_files", "report", "identity"}, "archive")
    scope(parent["identity"], child["identity"], {"runtime_modpack_id", "content_hash"}, "identity")
    require(child["identity"]["runtime_modpack_id"] == parent["identity"]["runtime_modpack_id"] + "+" + MANIFEST,
            "runtime pack lineage")
    scope(parent["report"], child["report"], {"manifests"}, "report")
    require(child["report"]["manifests"] == parent["report"]["manifests"] + [MANIFEST], "parent manifests")
    scope(a, b, {"maps", "map_scripts", "map_blocks", "tilesets", "npcs", "trainers"}, "game data")
    maps, scripts = verify_maps(a, b, texts)
    verify_art(parent, child)
    # compare() substitutes precisely the two reviewed known moves in its input, then compares every trainer field.
    runpy.run_path(str(HERE / "check-battles.py"))["compare"](a, b)
    print(f"{len(a['maps'])} maps, {len(a['trainers']['trainers'])} trainer teams, {len(b['pokemon'])} species, "
          f"{len(b['moves'])} moves, {len(b['items'])} items, {len(b)} game-data tables and "
          f"{len(child['runtime_files'])} runtime assets checked.")
    print(f"{maps} maps, {scripts} global map labels, two trainer move slots, ten native terrain files changed. "
          "128 Johto and 142 winter metatiles retain all original permissions and pixels morning, day and night. "
          "All original audio, sprite art, animations, wildlife, prices, starting flags and spawn inputs match.")


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("parent", type=Path)
    parser.add_argument("child", type=Path)
    args = parser.parse_args()
    digest = hashlib.sha256(args.parent.read_bytes()).hexdigest()
    require(digest in PARENTS, f"unreviewed parent {digest}")
    print(f"{PARENTS[digest]} parent {digest}; child {hashlib.sha256(args.child.read_bytes()).hexdigest()}", flush=True)
    verify(load_pack(args.parent), load_pack(args.child), TEXT)
