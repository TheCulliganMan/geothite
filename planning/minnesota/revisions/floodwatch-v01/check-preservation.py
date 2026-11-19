#!/usr/bin/env python3
"""Compare every Floodwatch payload with its exact parent, including maps, art, and sound."""

import argparse
import base64
import io
import json
from pathlib import Path
import sys

from PIL import Image

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE.parents[1] / "tools"))
from audit_regional_design import same_except
from audit_story_flow import require
from render_reference import SourceArt, load_pack
from check_garden_roofs import check_garden_roofs

CITY = "MnMoorhead"
CORE = "MnIronDrillCore"
PAID = "EVENT_MN_PRAIRIE_CIRCUIT_REWARDED"
COURSE = {"MnPrairie" + label for label in (
    "NeedLakeText", "IntroText", "LyleText", "JuneText", "FinchText", "GaugeText", "PaidText",
    "FullText", "Missing", "CheckLyle", "CheckJune", "CheckFinch", "DoneListing", "Finished",
    "NeedLake", "Gauge", "Full",
)}
SCRIPTS = {
    CITY: COURSE | {"MN_MOORHEAD_LEVEE_HELPER"},
    "MnR16C00R00": {"MN_R16_RIVER_WARDENText", "MN_R16_T01Seen", "MN_R16_T04Seen",
                    "MN_R16_T04Won", "MN_R16_T04Lost", "MN_R16_T04ReturnText"},
    "MnR16C01R00": {"MN_R16_SIGN_SEEDText", "MN_R16_T09Seen"},
    "MnMinneapolisDispatchOffice": {"MnNICEPlayerWatchesRight"},
    "MnIceOperations": {"MnIceOperationsWorkerText"},
    "MnIceRecords": {"MnIceHQCardGivenText", "MnIceHQCardAfterText"},
    CORE: {"MnNDrillStop", "MnNDrillCoreActors", "MnMineWalkToCrew", "MnMineFromMotorFront",
           "MnMineRoundMotor"},
}
TYPED = {
    CITY: {"script_control_commands", "script_flag_commands", "script_text_commands",
           "script_text_bodies", "script_item_grants", "script_runtime_commands",
           "script_audio_commands", "script_object_commands"},
    CORE: {"script_control_commands", "script_flag_commands", "script_object_commands",
           "script_variable_commands", "script_movements"},
    "MnMinneapolisDispatchOffice": {"script_movements"},
}


def records_except(a, b, labels, kind):
    if isinstance(a, dict):
        same_except(a, b, labels, kind)
    else:
        def unchanged(rows):
            return [r for r in rows if r.get("source_script") not in labels and r.get("label") not in labels]
        require(unchanged(a) == unchanged(b), f"{kind}: changed an unrelated native instruction")


def inspect_maps(old, new):
    require(same_except(old["maps"], new["maps"], SCRIPTS, "all maps") == SCRIPTS.keys(),
            "missing a revised map")
    changed_scripts = set()
    for name, labels in SCRIPTS.items():
        a, b = old["maps"][name], new["maps"][name]
        typed = TYPED.get(name, {"script_text_bodies"})
        extra = {"attributes", "blocks"} if name == CITY else set()
        if name == CORE:
            extra |= {"objects", "map_event_section_commands"}
        same_except(a, b, typed | extra | {"scripts"}, name)
        require(same_except(a["scripts"], b["scripts"], labels, name + " scripts") == labels,
                f"{name} missing or unexpected native program")
        changed_scripts |= labels
        for label in labels:
            require(new["map_scripts"][label] == b["scripts"][label], f"{name}/{label} split script")
        for key in typed:
            records_except(a[key], b[key], labels, f"{name}/{key}")
        if name != CORE:
            continue
        require(len(a["objects"]) == len(b["objects"]), "drill crew count")
        for left, right in zip(a["objects"], b["objects"]):
            if left["object_identifier"] == "MN_NDRILL_FINN":
                require({**left, "event_flag": "-1"} == right, "Finn's remaining actor properties")
            else:
                require(left == right, "other mine resident changed")
        expected = [dict(row, args=[*row["args"][:-1], "-1"])
                    if row["command"] == "object_event" and row["args"][11] == "MN_NDRILL_FINN"
                    else row for row in a["map_event_section_commands"]]
        require(expected == b["map_event_section_commands"], "drill room native events")
        event = b["attributes"]["map_events_label"]
        changed_scripts.add(event)
        require(new["map_scripts"][event] == [
            {"command": row["command"], "args": row["args"]} for row in expected
        ], "drill room split events")
    same_except(old["map_scripts"], new["map_scripts"], changed_scripts, "all global scripts")
    same_except(old["npcs"], new["npcs"], {CORE}, "all placed actors")
    require(new["npcs"][CORE] == new["maps"][CORE]["objects"], "drill room split actors")
    a, b = old["maps"][CITY], new["maps"][CITY]
    same_except(a["attributes"], b["attributes"], {"tileset_name"}, "Moorhead map attributes")
    same_except(old["map_attributes"], new["map_attributes"], {CITY}, "all map attributes")
    require(new["map_attributes"][CITY] == b["attributes"], "Moorhead split attributes")
    label = b["attributes"]["blocks_label"]
    same_except(old["map_blocks"], new["map_blocks"], {label}, "all map block files")
    require(base64.b64decode(new["map_blocks"][label]) == bytes(b["blocks"]), "Moorhead split terrain")


def compare(parent, child):
    same_except(parent, child, {"identity", "report", "data", "runtime_files"}, "entire pack")
    same_except(parent["report"], child["report"], {"manifests"}, "report and counts")
    require(child["report"]["manifests"] == parent["report"]["manifests"] + ["northstar-floodwatch-v1"],
            "wrong parent or manifest")
    same_except(parent["identity"], child["identity"], {"content_hash", "runtime_modpack_id"}, "identity")
    require(parent["identity"]["content_hash"] != child["identity"]["content_hash"]
            and child["identity"]["runtime_modpack_id"] == "+".join(child["report"]["manifests"]), "stale identity")
    old, new = parent["data"], child["data"]
    same_except(old, new, {"maps", "map_blocks", "map_attributes", "npcs", "map_scripts", "tilesets",
                           "trainers", "story_event_script_constants"}, "all Pokémon game data")
    inspect_maps(old, new)
    same_except(old["trainers"], new["trainers"], {"trainers"}, "trainer data")
    original, revised = old["trainers"]["trainers"], new["trainers"]["trainers"]
    teams = {"MN_R16_T01", "MN_R16_T09"}
    require(same_except(original, revised, teams, "all regional and original teams") == teams,
            "changed trainer parties")
    for key in teams:
        a, b = original[key], revised[key]
        same_except(a, b, {"party"}, key)
        require(len(a["party"]) == len(b["party"]), f"{key} party size")
        for ma, mb in zip(a["party"], b["party"]):
            same_except(ma, mb, {"species", "moves"}, key + " member")
    sc, sd = old["story_event_script_constants"], new["story_event_script_constants"]
    same_except(sc, sd, {"global"}, "story registries")
    same_except(sc["global"], sd["global"], {PAID}, "all native constants")
    largest = max(v for k, v in sc["global"].items() if k.startswith("EVENT_"))
    require(PAID not in sc["global"] and sd["global"][PAID] == largest + 1, "new reward flag aliases another")
    check_garden(parent, child)
    check_garden_roofs(child)
    print(f"Compared {len(new['maps'])} maps, {len(revised)} trainer teams and "
          f"{len(parent['runtime_files'])} original runtime files. All wildlife, animations, "
          "audio assets and unrelated gameplay data match.")


def check_garden(parent, child):
    old, new = parent["data"], child["data"]
    a, b = old["maps"][CITY], new["maps"][CITY]
    width = a["attributes"]["width"]
    changed = {(i % width, i // width) for i, (x, y) in enumerate(zip(a["blocks"], b["blocks"])) if x != y}
    require(len(a["blocks"]) == len(b["blocks"]) and changed == {(11, 8), (12, 8)}, "Moorhead garden extent")
    require([a["blocks"][8*width+x] for x in (11, 12)] == [8, 8], "expected two disconnected gatehouse roofs")
    bank = "johto_modern"
    require(a["attributes"] == b["attributes"] and b["attributes"]["tileset_name"] == bank,
            "Moorhead must keep Crystal's water and flower animation bank")
    same_except(old["tilesets"], new["tilesets"], {bank}, "all original collision and palette tables")
    prefix = f"data/tilesets/{bank}"
    meta, png, bits = prefix+"_metatiles.bin", f"gfx/tilesets/{bank}.png", f"gfx/tilesets/{bank}.2bpp"
    files, revised = parent["runtime_files"], child["runtime_files"]
    media = {meta, png, bits}
    require(same_except(files, revised, media, "all graphics, sounds and animations") == media,
            "garden files were not all updated")
    orig, out = bytes(files[meta]), bytes(revised[meta])
    garden = json.loads((HERE / "porch-garden.json").read_text())
    require(len(orig) == 128*16 and out == orig + bytes(garden["blocks"][0] + garden["blocks"][1]),
            "the existing Crystal terrain dictionary changed")
    require([b["blocks"][8*width+x] for x in (11, 12)] == [128, 129], "new garden tile ordering")
    require(all(i < 128 for name, r in new["maps"].items() if name != CITY
                and r["attributes"]["tileset_name"] == bank for i in r["blocks"]),
            "garden escaped Moorhead")
    t, u = old["tilesets"][bank], new["tilesets"][bank]
    require(u["collision"] == {**t["collision"], "80": t["collision"]["08"], "81": t["collision"]["08"]},
            "garden permissions or preexisting collision changed")
    same_except(t, u, {"palette_map", "collision"}, "Moorhead native bank")
    fresh = set(out[len(orig):]) - set(orig)
    authored = {tile["glyph"]: tile for tile in garden["tiles"]}
    require(fresh == authored.keys(), "unreferenced or unaccounted garden glyph")
    colors = t["palette_map"].copy()
    for glyph, tile in authored.items():
        colors[glyph] = 8 | tile["palette"]
    require(colors == u["palette_map"], "garden or inherited native palette")
    if prefix+".json" in revised:
        require(json.loads(bytes(revised[prefix+".json"])) == u["collision"], "garden split collision")
    if prefix+"_palette_map.json" in revised:
        require(json.loads(bytes(revised[prefix+"_palette_map.json"])) == u["palette_map"], "garden split palette")
    before_2bpp, after_2bpp = bytes(files[bits]), bytes(revised[bits])
    require(len(before_2bpp) == len(after_2bpp), "native VRAM address split moved")
    size = len(before_2bpp)//16
    def physical(tile, bank_data):
        mapping = bank_data["palette_map"]
        return size//2 + (tile & 127) if tile < len(mapping) and mapping[tile] & 8 else tile
    old_uses = {physical(i, t) for i in orig}
    slots = {physical(i, u) for i in fresh}
    changed = {i for i in range(size) if before_2bpp[16*i:16*i+16] != after_2bpp[16*i:16*i+16]}
    require(changed and changed <= slots and not changed & old_uses, "existing 2bpp artwork changed")
    for glyph, drawing in authored.items():
        slot = physical(glyph, u)
        tile = after_2bpp[16*slot:16*slot+16]
        decoded = ["".join(str(((tile[2*v]>>(7-k))&1) + 2*((tile[2*v+1]>>(7-k))&1))
                           for k in range(8)) for v in range(8)]
        require(decoded == drawing["rows"], f"garden glyph {glyph:02x} differs from its drawing")
    left, right = [Image.open(io.BytesIO(bytes(s[png]))).convert("L") for s in (files, revised)]
    require(left.size == right.size, "garden PNG resized")
    for i in range(size):
        x, y = 8*(i % (left.width//8)), 8*(i // (left.width//8))
        box = (x, y, x+8, y+8)
        if i not in changed:
            require(left.crop(box).tobytes() == right.crop(box).tobytes(), "original PNG glyph changed")
            continue
        tile = after_2bpp[16*i:16*i+16]
        expected = bytes(255-85*(((tile[2*v]>>(7-k))&1) + 2*((tile[2*v+1]>>(7-k))&1))
                         for v in range(8) for k in range(8))
        require(right.crop(box).tobytes() == expected, "native PNG and 2bpp garden pixels disagree")
    original_art, garden_art = SourceArt(parent), SourceArt(child)
    for group in ("morn", "day", "nite"):
        old_images = original_art.blocks(bank, group)
        new_images = garden_art.blocks(bank, group)
        require(all(x.tobytes() == y.tobytes() for x, y in zip(old_images, new_images)),
                f"existing {bank} {group} terrain changed appearance")


if __name__ == "__main__":
    cli = argparse.ArgumentParser(description=__doc__)
    cli.add_argument("parent", type=Path)
    cli.add_argument("candidate", type=Path)
    args = cli.parse_args()
    compare(load_pack(args.parent), load_pack(args.candidate))
