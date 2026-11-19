#!/usr/bin/env python3
"""Verify the precise archive differences for either Shore Watch release lineage."""

import argparse
import base64
import hashlib
import io
import json
from pathlib import Path
import sys

from PIL import Image

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE.parents[1] / "tools"))
from audit_story_flow import native_pages, require
from render_reference import SourceArt, load_pack

WEST, EAST, PARK = "MnR18C00R00", "MnR18C01R00", "MnMilleLacs"
BANK = "mn_headwaters"
PRIVATE = "mn_r18_shore"
FLAG = "EVENT_MN_HIDDEN_MILLE_LACS_PICNIC_LURE"
TILE = f"data/tilesets/{BANK}"
GFX = f"gfx/tilesets/{BANK}"
LEAVES = {
    WEST: {
        "MN_R18_WatchNeedsNews",
        "MN_R18_WatchHasRod",
        "MnShoreWatchMissing",
        "MnShoreWatchReeds",
        "MnShoreWatchMeadow",
        "MnShoreWatchBeach",
        "MnShoreWatchHoldLures",
        "MnShoreWatchReedsText",
        "MnShoreWatchMeadowText",
        "MnShoreWatchBeachText",
        "MnShoreWatchLuresText",
        "MnShoreWatchLuresHeldText",
    },
    EAST: set(),
    PARK: {
        "MN_ML_PARK_PICNIC",
        "MnMlPicnicPacked",
        "MnMlPicnicLure",
        "MnMlPicnicLunchText",
        "MnMlPicnicSpareBallText",
        "MnWayShoreReport",
        "MnShoreWatchParkLocations",
        "MnShoreWatchReturnText",
        "MnWayShoreReportOriginal",
        "MnShoreWatchCastingText",
    },
}
TYPED = {
    "script_flag_commands": {"checkflag", "checkevent", "setevent"},
    "script_control_commands": {"end", "iftrue", "iffalse", "sjump"},
    "script_text_commands": {
        "opentext",
        "writetext",
        "promptbutton",
        "waitbutton",
        "closetext",
        "jumptext",
        "jumptextfaceplayer",
    },
    "script_item_grants": {"giveitem"},
    "script_field_pickups": {"hiddenitem"},
    "script_object_commands": {"faceplayer"},
}


def diff(a, b):
    return a.keys() ^ b.keys() | {k for k in a.keys() & b.keys() if a[k] != b[k]}


def exact(actual, expected, name):
    require(
        actual == expected,
        f"{name}: missing {expected - actual}, extra {actual - expected}",
    )


def source(name):
    return json.loads((HERE / name).read_text())


def scripts(old, new, labels):
    exact(diff(old["scripts"], new["scripts"]), labels, new["id"] + " scripts")
    for key, ops in TYPED.items():
        before = [r for r in old[key] if r["source_script"] not in labels]
        after = [r for r in new[key] if r["source_script"] not in labels]
        require(before == after, "unrelated " + key)
        expected = {
            (l, i): (row["command"], row["args"])
            for l in labels
            for i, row in enumerate(new["scripts"][l])
            if row["command"] in ops
        }
        seen = {}
        for r in new[key]:
            l, i = r["source_script"], r["command_index"]
            if l not in labels:
                continue
            require((l, i) not in seen and (l, i) in expected, f"stale {key} {l}:{i}")
            op, args = expected[l, i]
            require(op == r["command"], f"{key} {l}:{i} expected {op}")
            if key == "script_flag_commands":
                require(args == [r["flag_id"]], "wrong typed flag")
            if op in {"iftrue", "iffalse", "sjump"}:
                require(
                    args == [r["target_label"]] == [r["resolved_target_script"]],
                    "wrong resolved jump",
                )
            if op in {"writetext", "jumptext", "jumptextfaceplayer"}:
                require(args == [r["text_label"]], "wrong resolved text")
            if op == "giveitem":
                require(
                    args == [r["item_id"], str(r["quantity"])] and not r["verbose"],
                    "wrong gift",
                )
            if op == "hiddenitem":
                require(
                    args == [r["item_id"], r["event_flag"]] and r["quantity"] == 1,
                    "wrong hidden Ball",
                )
            seen[l, i] = r
        exact(seen.keys(), expected.keys(), key + " / " + new["id"])
    text = {l for l in labels if new["scripts"][l][0]["command"] == "text"}
    exact(
        diff(old["script_text_bodies"], new["script_text_bodies"]),
        text,
        "changed native pages",
    )
    for label in text:
        rows = new["scripts"][label]
        body = new["script_text_bodies"][label]
        require(
            body["label"] == label
            and body["commands"]
            == [dict(row, command_index=i) for i, row in enumerate(rows)]
            and all(
                0 < len(line) <= 18 for page in native_pages(rows) for line in page
            ),
            "text mirror " + label,
        )


def events(old, new):
    bg = {"x": 18, "y": 34, "event_type": "BGEVENT_ITEM", "script": "MnMlPicnicLure"}
    require(
        new["events"]
        == dict(old["events"], bg_events=old["events"]["bg_events"] + [bg]),
        "park event scope",
    )
    rows = [
        {k: v for k, v in r.items() if k != "command_index"}
        for r in old["map_event_section_commands"]
    ]
    where = next(i for i, r in enumerate(rows) if r["command"] == "def_object_events")
    rows.insert(
        where,
        {"command": "bg_event", "args": ["18", "34", "BGEVENT_ITEM", "MnMlPicnicLure"]},
    )
    require(
        new["map_event_section_commands"]
        == [dict(row, command_index=i) for i, row in enumerate(rows)],
        "park native event section",
    )
    return rows


def settings(a, b):
    exact(
        diff(a, b),
        {
            "maps",
            "map_attributes",
            "field_moves",
            "map_blocks",
            "map_scripts",
            "story_event_script_constants",
            "tilesets",
        },
        "game data outside Shore Watch",
    )
    old, new = a["story_event_script_constants"], b["story_event_script_constants"]
    exact(diff(old, new), {"global"}, "scoped event symbols")
    flags = old["global"]
    next_id = 1 + max(v for k, v in flags.items() if k.startswith("EVENT_"))
    require(
        FLAG not in flags and new["global"] == dict(flags, **{FLAG: next_id}),
        "changed prior event IDs",
    )
    exact(diff(a["maps"], b["maps"]), set(LEAVES), "changed maps")
    exact(diff(a["tilesets"], b["tilesets"]), {BANK, PRIVATE}, "tileset scope")
    exact(
        diff(a["map_attributes"], b["map_attributes"]),
        {WEST, EAST},
        "route alias bindings",
    )
    old_moves, new_moves = a["field_moves"], b["field_moves"]
    exact(diff(old_moves, new_moves), {"cut", "whirlpool"}, "field move data")
    for move in ["cut", "whirlpool"]:
        x, y = old_moves[move], new_moves[move]
        replacements = x["replacements"]
        require(
            y
            == dict(
                x, replacements=dict(replacements, **{PRIVATE: replacements[BANK]})
            ),
            "unrelated field move replacement " + move,
        )
    labels = set()
    for name, allowed in LEAVES.items():
        old, new = a["maps"][name], b["maps"][name]
        allowed_keys = (
            {"blocks"}
            if not allowed
            else {"blocks", "scripts", "script_text_bodies", *TYPED}
        )
        if name != PARK:
            allowed_keys.add("attributes")
            require(
                new["attributes"]
                == dict(old["attributes"], tileset_name=PRIVATE)
                == b["map_attributes"][name],
                "incorrect route alias data",
            )
        if name == PARK:
            allowed_keys.update({"events", "map_event_section_commands"})
        require(
            diff(old, new) <= allowed_keys,
            f"changed other {name} fields: {diff(old, new) - allowed_keys}",
        )
        scripts(old, new, allowed)
        for label in allowed:
            require(
                b["map_scripts"][label] == new["scripts"][label],
                "split script " + label,
            )
        labels.update(allowed)
        if name == PARK:
            wire = new["attributes"]["map_events_label"]
            require(
                b["map_scripts"][wire] == events(old, new), "split park event section"
            )
            labels.add(wire)
        key = new["attributes"]["blocks_label"]
        require(
            base64.b64decode(b["map_blocks"][key], validate=True)
            == bytes(new["blocks"]),
            "split terrain " + name,
        )
    exact(diff(a["map_scripts"], b["map_scripts"]), labels, "split script scope")
    exact(
        diff(a["map_blocks"], b["map_blocks"]),
        {b["maps"][n]["attributes"]["blocks_label"] for n in LEAVES},
        "split block scope",
    )


def native(parent, child, route, park):
    before, after = parent["runtime_files"], child["runtime_files"]
    old_bank = parent["data"]["tilesets"][BANK]
    source_gfx = bytes(before[GFX + ".2bpp"])
    source_layout = bytes(before[TILE + "_metatiles.bin"])
    require(
        len(source_layout) == 150 * 16 and len(source_gfx) == 256 * 16,
        "expected native bank dimensions",
    )
    expected_files = {TILE + "_metatiles.bin", TILE + ".json"}
    source_art, result_art = SourceArt(parent), SourceArt(child)
    for name, recipes in [(BANK, park["recipes"]), (PRIVATE, route["metatiles"])]:
        key = f"data/tilesets/{name}"
        gfx = f"gfx/tilesets/{name}"
        new_bank = child["data"]["tilesets"][name]
        extra = b"".join(bytes(r["tiles"]) for r in recipes)
        require(
            bytes(after[key + "_metatiles.bin"]) == source_layout + extra,
            "unapproved source/arrangement " + name,
        )
        new_roles = {f"{150 + i:02x}": r["collision"] for i, r in enumerate(recipes)}
        require(
            new_bank["collision"]
            == dict(old_bank["collision"], **new_roles)
            == json.loads(bytes(after[key + ".json"])),
            "unexpected collision " + name,
        )
        for clock in ("morn", "day", "nite"):
            old, new = source_art.blocks(BANK, clock), result_art.blocks(name, clock)
            require(
                len(old) == 150
                and len(new) == 150 + len(recipes)
                and all(a.tobytes() == b.tobytes() for a, b in zip(old, new)),
                f"{name}: previous block visual changed at {clock}",
            )
        if name == BANK:
            require(
                new_bank == dict(old_bank, collision=new_bank["collision"]),
                "unrelated shared bank data",
            )
            continue
        expected_files.update(
            {
                key + ".json",
                key + "_palette_map.json",
                key + "_native_backgrounds.json",
                key + "_animations.json",
                key + "_metatiles.bin",
                gfx + ".2bpp",
                gfx + ".png",
            }
        )
        require(
            not any(f in before for f in expected_files if PRIVATE in f),
            "private bank already existed",
        )
        masks = route["masks"]
        ids = set(range(0x60, 0x60 + len(masks)))
        require(
            bool(ids)
            and max(ids) < 128
            and ids.isdisjoint(source_layout)
            and ids <= set(extra),
            "invalid shoreline mask use",
        )

        def physical(g):
            return (g & 127) + 128 if old_bank["palette_map"][g] & 8 else g

        require(
            ids.isdisjoint({physical(g) for g in source_layout}),
            "original glyph aliases a shoreline mask",
        )
        pal = old_bank["palette_map"].copy()
        for g in ids:
            pal[g] = 3
        require(
            new_bank == dict(old_bank, collision=new_bank["collision"], palette_map=pal)
            and json.loads(bytes(after[key + "_palette_map.json"])) == pal,
            "unrelated palette",
        )
        old_bg = json.loads(bytes(before[TILE + "_native_backgrounds.json"]))
        require(
            json.loads(bytes(after[key + "_native_backgrounds.json"]))
            == dict(old_bg, **{str(g): 0 for g in ids}),
            "shoreline day/night ground color",
        )
        water = bytes(before["gfx/tilesets/water/water.2bpp"])
        require(len(water) == 64, "four native water frames")
        clips = json.loads(bytes(before[TILE + "_animations.json"]))
        planes = bytearray(source_gfx)
        for j, m in enumerate(masks):
            frames = []
            for f in range(4):
                result = []
                for y, (wet, shallow) in enumerate(zip(m["wet"], m["shallow"])):
                    require(shallow & ~wet == 0, "dry sand included in shallow mask")
                    lo, hi = water[f * 16 + 2 * y : f * 16 + 2 * y + 2]
                    coast = hi & ~lo & shallow
                    result.extend([(lo & wet) | coast, (hi & wet) & ~coast])
                require(len(result) == 16, "wrong mask height")
                frames.append(result)
            i = 0x60 + j
            require(str(i) not in clips, "overwritten animation")
            clips[str(i)] = {"frame_ticks": 22, "frames": frames}
            planes[i * 16 : i * 16 + 16] = frames[0]
        require(
            bytes(after[gfx + ".2bpp"]) == planes
            and json.loads(bytes(after[key + "_animations.json"])) == clips,
            "altered original ink, animation rhythm or frame",
        )
        sheet = Image.open(io.BytesIO(bytes(after[gfx + ".png"]))).convert("L")
        require(sheet.size == (128, 128), "native sheet dimensions")
        for i in range(256):
            for y in range(8):
                lo, hi = planes[16 * i + 2 * y : 16 * i + 2 * y + 2]
                for x in range(8):
                    ink = ((lo >> (7 - x)) & 1) | (((hi >> (7 - x)) & 1) << 1)
                    require(
                        sheet.getpixel((8 * (i % 16) + x, 8 * (i // 16) + y))
                        == 255 - 85 * ink,
                        "indexed PNG and tile planes disagree",
                    )
    exact(
        diff(before, after), expected_files, "native files, sprites, sound or animation"
    )
    return len(route["metatiles"]) + len(park["recipes"])


def check(parent_path, child_path):
    parent_hash = hashlib.sha256(parent_path.read_bytes()).hexdigest()
    previous = HERE.parent / "league-journey-v01"
    require(
        parent_hash
        in {
            json.loads((previous / f"{ed}-build.json").read_text())[
                "expected_pack_sha256"
            ]
            for ed in ("field", "intro")
        },
        "not a League Journey parent",
    )
    before, after = load_pack(parent_path), load_pack(child_path)
    exact(
        diff(before, after),
        {"data", "report", "runtime_files", "identity"},
        "archive payloads",
    )
    require(
        after["report"]
        == dict(
            before["report"],
            manifests=before["report"]["manifests"]
            + ["northstar-r18-sand-shore-v1", "northstar-shore-watch-v1"],
        ),
        "archive lineage",
    )
    a, b = before["data"], after["data"]
    settings(a, b)
    r, p = source("route-terrain.json"), source("picnic-cove.json")
    count = native(before, after, r, p)
    layouts = {n: a["maps"][n]["blocks"].copy() for n in LEAVES}

    def place(n, at, old, new):
        x, y = at
        i = y * a["maps"][n]["attributes"]["width"] + x
        require(layouts[n][i] == old, f"unexpected source {n} {x},{y}")
        layouts[n][i] = new

    require(
        set(r["rooms"]) == {WEST, EAST}
        and all(
            rows and len({tuple(row[:2]) for row in rows}) == len(rows)
            for rows in r["rooms"].values()
        ),
        "route footprint",
    )
    for name, rows in r["rooms"].items():
        for x, y, old, t in rows:
            place(name, [x, y], old, t if t < 256 else 150 + t - 256)
    for at, old, t in p["placements"]:
        place(PARK, at, old, t if t < 1000 else 150 + t - 1000)
    require(
        all(layouts[n] == b["maps"][n]["blocks"] for n in LEAVES),
        "different blocks outside authored placements",
    )
    print(
        f"{len(a['maps'])} maps checked, exactly three changed. {count} appended native blocks; all original 150 metatiles pixel-identical at morning, day and night. Every previous save flag, party, encounter, fishing table, sound and animation retained."
    )


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("parent", type=Path)
    parser.add_argument("child", type=Path)
    args = parser.parse_args()
    check(args.parent, args.child)
