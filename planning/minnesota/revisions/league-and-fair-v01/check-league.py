#!/usr/bin/env python3
"""Check the four authored League appearances, their normal battles and original game assets."""
import argparse
from copy import deepcopy
from io import BytesIO
import json
from pathlib import Path
import struct
import sys

from PIL import Image

REGION = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(REGION / "tools"))
from audit_regional_design import same_except
from audit_story_flow import require
from render_reference import load_pack

# Accepted battle portrait and independently accepted later walking revision.
CAST = [
    ("10-fern", 2, 3, "COOLTRAINERF"),
    ("11-flint", 3, 2, "HIKER"),
    ("12-veil", 3, 3, "SAGE"),
    ("13-halden", 2, 4, "COOLTRAINERM"),
]
POSES = ["down_idle", "up_idle", "left_idle", "down_step", "up_step", "left_step"]


def source(*path):
    return json.loads((REGION / "npcs").joinpath(*path).read_text())


def check_pixels(files, folder, stem, rows, palette):
    prefix = f"gfx/{folder}/{stem}"
    width, height = (16, 96) if folder == "sprites" else (56, 56)
    require(len(rows) == height and all(len(row) == width for row in rows), f"{prefix} accepted grid")
    inks = bytes(".123".index(c) for row in rows for c in row)
    pal = struct.unpack("<4H", bytes(files[prefix + ".gbcpal"]))
    require(list(pal) == palette, f"{prefix} native palette")
    encoded = bytes(files[prefix + ".2bpp"])
    require(len(encoded) == width * height // 4, f"{prefix} tile count")
    # Decode the shipped tile bank, independently of the game's encoder.
    decoded = bytearray(width * height)
    for y in range(height):
        for x in range(width):
            row = ((y // 8) * (width // 8) + x // 8) * 16 + (y % 8) * 2
            bit = 7 - x % 8
            decoded[y * width + x] = ((encoded[row] >> bit) & 1) | (((encoded[row + 1] >> bit) & 1) << 1)
    require(decoded == inks, f"{prefix} native 2bpp pixels")
    if folder == "sprites":
        colors = [(v, v, v, 255 if i else 0) for i, v in enumerate([255, 170, 85, 0])]
        mode = "RGBA"
    else:
        def expand(v):
            return (v << 3) | (v >> 2)
        colors = [tuple(expand((ink >> shift) & 31) for shift in (0, 5, 10)) for ink in pal]
        mode = "RGB"
    with Image.open(BytesIO(bytes(files[prefix + ".png"]))) as png:
        require(png.size == (width, height) and png.mode == mode, f"{prefix} PNG layout")
        require(png.tobytes() == b"".join(bytes(colors[i]) for i in inks), f"{prefix} displayed pixels")
    return {prefix + suffix for suffix in (".png", ".2bpp", ".gbcpal")}


def check_room(old, new, actor, trainer, old_class, private):
    name, script, tile = actor["map"], actor["script"], actor["tile"]
    room = deepcopy(old["maps"][name])
    final = new["maps"][name]
    sprite = "SPRITE_" + private
    (obj,) = [o for o in room["objects"] if o["script"] == script and [o["x"], o["y"]] == tile]
    require(obj["sprite"] == actor["previous_sprite"] and obj["sprite_has_facings"], f"{name} original actor")
    obj["sprite"] = sprite
    (event,) = [e for e in room["map_event_section_commands"] if e["command"] == "object_event" and e["args"][11] == script]
    require(event["args"][:3] == [*map(str, tile), actor["previous_sprite"]], f"{name} event preimage")
    event["args"][2] = sprite
    require(room["objects"] == new["npcs"][name], f"{name} native NPC mirror")
    event_label = room["attributes"]["map_events_label"]
    records = [{k: command[k] for k in ("command", "args")} for command in room["map_event_section_commands"]]
    require(new["map_scripts"][event_label] == records, f"{name} world event mirror")

    (battle,) = room["scripted_trainer_battles"]
    battle_label = battle["source_script"]
    require({"command": "sjump", "args": [battle_label]} in room["scripts"][script], f"{name} actor starts challenge")
    start = room["scripts"][battle_label]
    ix = battle["loadtrainer_command_index"]
    require(start[ix] == {"command": "loadtrainer", "args": [old_class, trainer]}, f"{name} load preimage")
    require(start[battle["startbattle_command_index"]]["command"] == "startbattle", f"{name} starts normal battle")
    start[ix]["args"][0] = private
    request = battle["request"]
    require((request["trainer_id"], request["trainer_class"]) == (trainer, old_class), f"{name} parsed battle")
    request["trainer_class"] = private
    (runtime,) = [c for c in room["script_runtime_commands"] if c["source_script"] == battle_label and c["command_index"] == ix]
    require(runtime["args"] == [old_class, trainer], f"{name} native loadtrainer")
    runtime["args"][0] = private
    require(new["map_scripts"][battle_label] == start, f"{name} battle program mirror")
    # Republishing reparses these two derived indexes in script-map order.
    # The candidate must contain the same original records in that exact order.
    for key in ("script_flag_commands", "script_control_commands"):
        room[key].sort(key=lambda row: (row["source_script"], row["command_index"]))
    require(room == final, f"{name} changed encounter geometry, progression, dialogue, or additional script data")


def check_cafe(parent, candidate):
    old, new = parent["data"], candidate["data"]
    lodge = "MnMothlightLodge"
    for name in (lodge, "MnMothlightLoft", "MnMaraisCedarApproach"):
        require(old["maps"][name] == new["maps"][name], f"regional Café room {name}")
    cook = "MN_MOTHLIGHT_CAFE"
    team = old["trainers"]["trainers"][cook]
    require(new["trainers"]["trainers"][cook] == team, "Café cook's complete roster and rules")
    party = [(slot["species"], slot["level"]) for slot in team["party"]]
    require(team["name"] == "COOK" and party == [("MOTHOSAIC", 36), ("MUFFIEND", 36)], "native Café team")
    today, pending = "EVENT_MN_MOTHLIGHT_CAFE_WON_TODAY", "EVENT_MN_MOTHLIGHT_CAFE_PRIZE_PENDING"
    for flag, expected in ((today, 2813), (pending, 2814)):
        require(old["story_event_script_constants"]["global"][flag] ==
                new["story_event_script_constants"]["global"][flag] == expected, flag)
    scripts = old["maps"][lodge]["scripts"]
    cook_commands = [(c["command"], c["args"]) for c in scripts["MnMothlightCook"]]
    require(cook_commands[1:5] == [("checkevent", [pending]), ("iftrue", ["MnCafePrize"]),
                                  ("checkevent", [today]), ("iftrue", ["MnCafeToday"])] and
            ("loadtrainer", ["FISHER", cook]) in cook_commands and
            cook_commands[-3:] == [("setevent", [today]), ("setevent", [pending]), ("sjump", ["MnCafePrize"])],
            "Café offer and once-a-day win")
    prize = [(c["command"], c["args"]) for c in scripts["MnCafePrize"]]
    require(prize == [("giveitem", ["BERRY_JUICE", "1"]), ("iffalse", ["MnCafeFull"]),
                      ("clearevent", [pending]), ("jumptext", ["MnCafePrizeText"])], "pending Café prize")
    for label in ("MnMothlightCook", "MnCafePrize", "MnCafeFull", "MnCafeToday", "MnCafeDeclined"):
        require(old["map_scripts"][label] == new["map_scripts"][label] == scripts[label], f"Café field program {label}")
    art = {path: data for path, data in parent["runtime_files"].items() if "mn_mothlight" in path}
    require(len(art) == 17 and all(candidate["runtime_files"][k] == v for k, v in art.items()), "Café and Lodge native art")
    print(f"Café parent retained: Lodge, loft and cedar approach, {len(art)} native art files, level-36 Mothosaic and Muffiend, daily win flag 2813, and one Berry Juice with full-Pack retry.")


def audit(parent, candidate=None):
    old = parent["data"]
    ancestry = parent["report"]["manifests"]
    trails = "northstar-northern-trails-v1"
    cafe = ancestry[-2:] == [trails, "mothlight-battle-cafe-v1"]
    require(ancestry[-1] == trails or (cafe and parent["identity"]["content_hash"] ==
            "701440cf37077155a8c690d54fddc48d91aadb3bf6f7ffcee52525dc034f04a8"), "wrong League parent")
    expected_trainers = deepcopy(old["trainers"])
    expected_classes = dict(old["trainer_class_names"])
    expected_palettes = dict(old["sprite_palette_defaults"])
    paths = set()
    for i, (slug, front_version, field_version, shared) in enumerate(CAST, 1):
        spec = source("regional-trainers-v01", "story-bindings", slug + ".json")
        (trainer,) = spec["trainer_ids"]
        (actor,) = spec["actors"]
        private = spec["trainer_class"]
        require(trainer == f"MN_LEAGUE_{i}" and actor["map"] == f"MnLeagueRoom{i}", f"{slug} binding")
        t = expected_trainers["trainers"][trainer]
        require(t["trainer_class"] == shared, f"{slug} parent battle class")
        original = old["maps"][actor["map"]]
        (obj,) = [o for o in original["objects"] if o["script"] == actor["script"]]
        require(obj["sprite"] == actor["previous_sprite"], f"{slug} parent field art")
        stem = private.lower()
        require(not any(stem in p for p in parent["runtime_files"]), f"{slug} shared parent has private art")
        t["trainer_class"] = private
        expected_classes[private] = spec["class_name"]
        expected_palettes["SPRITE_" + private] = 0
        portrait = source("cast", slug, f"native-pixels-v{front_version}.json")
        field = source("regional-trainers-v01", "story-field-repairs", slug, f"native-pixels-v{field_version}.json")
        require(portrait["id"] == field["id"] == stem and portrait["palette_rgb555"] == field["palette_rgb555"], f"{slug} identities")
        frames = field["overworld_frames"]
        require([f["pose"] for f in frames] == POSES and all(frames[d]["rows"] != frames[d+3]["rows"] for d in range(3)), f"{slug} six native poses")
        if candidate is not None:
            check_room(old, candidate["data"], actor, trainer, shared, private)
            for folder, rows in [("trainers", portrait["battle_rows"]), ("sprites", [row for f in frames for row in f["rows"]])]:
                paths |= check_pixels(candidate["runtime_files"], folder, stem, rows, portrait["palette_rgb555"])
    flint = expected_trainers["trainers"]["MN_LEAGUE_2"]["party"]
    ace = flint[-1]
    old_moves = ["SANDSTORM", "ROCK_SLIDE", "BARRIER", "PSYCHIC_M"]
    require(ace["species"] == "FORTIGARD" and ace["level"] == 49 and [m["name"] for m in ace["moves"]] == old_moves, "Fortigard preimage")
    require([m["name"] for m in flint[0]["moves"]] == old_moves, "Bandrake still opens with its original moves")
    for slot, move in [(0, "DEFENSE_CURL"), (2, "ROLLOUT")]:
        require(any(lv <= ace["level"] and m == move for lv, m in old["learnsets"]["FORTIGARD"]), f"native {move}")
        ace["moves"][slot] = {"name": move, "current_pp": old["moves"][move]["pp"], "pp_ups": 0}
    if candidate is None:
        print("Parent confirmed: four shared League actors, four accepted source pairs, original Flint lineup.")
        return
    if cafe:
        check_cafe(parent, candidate)
    new = candidate["data"]
    require(new["trainers"] == expected_trainers, "unrelated team/level/AI/reward or Flint move/PP drift")
    require(new["trainer_class_names"] == expected_classes and new["sprite_palette_defaults"] == expected_palettes, "shared class/palette changed")
    require(same_except(parent["runtime_files"], candidate["runtime_files"], paths, "native art and media") == paths and len(paths) == 24, "24 private rendering files")
    same_except(parent, candidate, {"data", "runtime_files", "identity", "report"}, "audio and envelope")
    same_except(old, new, {"trainers", "trainer_class_names", "sprite_palette_defaults", "maps", "map_blocks", "map_scripts", "npcs"}, "other game catalogs")
    league_rooms = {f"MnLeagueRoom{i}" for i in range(1, 5)}
    rooms = league_rooms | {"MnLeagueChampion", "MnStPaul", "MnStPaulCapitolHall", "MnStPaulFairInfoHouse", "MnR05C00R02"}
    require(same_except(old["maps"], new["maps"], rooms, "world maps") == rooms, "nine revised rooms")
    require(old["map_blocks"] == new["map_blocks"], "all terrain blocks")
    require(same_except(old["npcs"], new["npcs"], league_rooms, "world actors") == league_rooms, "four League actors")
    allowed_programs = set()
    for name in rooms:
        before, after = old["maps"][name], new["maps"][name]
        require(before["events"] == after["events"] and before["attributes"] == after["attributes"], f"{name} warps, edges and callbacks")
        allowed_programs.update(key for key in after["scripts"] if before["scripts"].get(key) != after["scripts"][key])
        if name in league_rooms:
            allowed_programs.add(after["attributes"]["map_events_label"])
    same_except(old["map_scripts"], new["map_scripts"], allowed_programs, "all world programs")
    same_except(parent["report"], candidate["report"], {"manifests"}, "report")
    markers = ["native-character-art-mn_elite_" + slug.split("-")[1] + "-v1" for slug, *_ in CAST]
    require(candidate["report"]["manifests"] == parent["report"]["manifests"] + markers + ["northstar-league-and-fair-v1"], "revision ancestry")
    print(f"{len(new['maps'])} maps compared. Only nine rooms changed. Every warp, map edge and terrain block is intact.")
    print(f"Four native actors and battle scripts, 24 walking poses, four 56×56 portraits and all 24 art resources match. All {len(expected_trainers['trainers'])} trainer records checked, with only the four private classes and two Fortigard moves revised. Original media and audio preserved.")


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("parent", type=Path)
    parser.add_argument("candidate", type=Path, nargs="?", help="omit to check the parent and accepted sources")
    args = parser.parse_args()
    audit(load_pack(args.parent), load_pack(args.candidate) if args.candidate else None)
