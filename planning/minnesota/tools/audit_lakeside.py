"""Compare an installed Lakeside pack to its released raw or hosted Crossings parent."""
import argparse
import hashlib
import json
import struct
from collections import Counter
from pathlib import Path

from audit_regional_design import check, load, same_except

REGION = Path(__file__).resolve().parents[1]
POLICY = "data/northstar-save-compatibility.json"
PARENTS = {
    "caecd60747ee26510651509c6567a3490d919c0d6c9f8c2528d9445de2ad1599",
    "966e3a8ac71190642addc67a010ffd4e59cb289401fc748ddd36594de4820ae3",
}
GYMS = ("Rochester", "Mankato", "Minneapolis", "StPaul", "StCloud", "Brainerd", "Bemidji", "Duluth")


def read_json(path):
    return json.loads(path.read_bytes())


def sprite_inventory(old, new, root, native):
    prior, files = old["runtime_files"], new["runtime_files"]
    retired = {family["id"] for family in read_json(root / "roster-curation.json")["retired_families"]}
    roster = {family["id"]: {m["id"]: m["name"].lower() for m in family["members"]}
              for family in read_json(root / "roster.json")["families"] if family["id"] not in retired}
    palettes = read_json(root / "palettes.json")["pokemon"]
    profiles = {p["id"]: p for p in read_json(root / "species-profiles.json")["species"]}
    allowed = set()
    changed = set()
    views = 0
    for entry in read_json(root / "sprite-generation.json")["families"]:
        family = entry["id"]
        if family not in roster:
            continue
        manifest = read_json(root / entry["manifest"])
        directory = native / family
        check(hashlib.sha256((directory / "line.review.png").read_bytes()).hexdigest() ==
              manifest["visual_review"]["sheet_sha256"], f"unknown review {family}")
        for sprite in manifest["sprites"]:
            species = sprite["species_id"]
            if species not in roster[family]:
                continue
            name, pose = roster[family][species], sprite["view"]
            base = f"gfx/pokemon/{name}/"
            stem = f'{sprite["id"]}.{pose}'
            for suffix in ("2bpp", "png"):
                key = base + pose + "." + suffix
                expected = (directory / (stem + "." + suffix)).read_bytes()
                check(bytes(files[key]) == expected, "wrong native portrait " + key)
                allowed.add(key)
            if prior[base + pose + ".2bpp"] != files[base + pose + ".2bpp"]:
                changed.add((species, pose))
            colors = profiles[species]["shiny_rgb5"]
            expected = {
                "normal.gbcpal": struct.pack("<4H", *palettes[species]["gbc_rgb555_words"]),
                "shiny.pal": "".join(f"RGB {r}, {g}, {b}\n" for r, g, b in colors).encode(),
                "front.dimensions": bytes([0x77]),
            }
            for leaf, blob in expected.items():
                check(bytes(files[base + leaf]) == blob, "unmatched palette or size for " + name)
                allowed.add(base + leaf)
            views += 1
    walking = Counter()
    accepted = read_json(root / "overworld/20261027-v1/accepted.json")["species"]
    for species in accepted:
        keys = []
        for record in species["runtime_files"]:
            key = record["path"].split("/native/", 1)[1]
            check(hashlib.sha256(bytes(files[key])).hexdigest() == record["sha256"],
                  "walking sprite differs from approved bank: " + key)
            keys.append(key)
        walking["missing" if any(k not in prior for k in keys) else
                "older" if any(files[k] != prior[k] for k in keys) else "current"] += 1
        allowed.update(keys)
    check(views == 450 and len(changed) == 126 and len({s for s, _ in changed}) == 63,
          "battle portrait coverage changed")
    check(walking == {"missing": 43, "older": 6, "current": 176}, "unexpected walking coverage")
    return allowed


def compare(old_hash, old, new, root, native):
    check(old_hash in PARENTS, "unexpected Crossings parent")
    a, b = old["data"], new["data"]
    story = read_json(REGION / "story.json")["journey_landmark_dialogue"]
    changed_maps = set(story) | {"MnBrainerdGym"}
    check(same_except(a["maps"], b["maps"], changed_maps, "maps") == changed_maps,
          "expected nine edited rooms")
    same_except(old, new, {"data", "identity", "report", "runtime_files"}, "pack")
    same_except(old["report"], new["report"], {"manifests"}, "report")
    same_except(a, b, {"maps", "map_scripts", "map_blocks", "npcs", "trainers", "trainer_class_names",
                        "pokemon", "sprite_palette_defaults", "tilesets", "story_event_script_constants"},
                "game data")
    check(len(b["maps"]) == 743, "region map coverage")
    same_except(a["map_blocks"], b["map_blocks"], {"MnBrainerdGym_Blocks"}, "map blocks")
    same_except(a["tilesets"], b["tilesets"], {"mn_gym_6"}, "tilesets")
    for name in changed_maps:
        ma, mb = a["maps"][name], b["maps"][name]
        check(ma["attributes"] == mb["attributes"] and ma["events"] == mb["events"], "map exits " + name)
        if name != "MnRochesterGym":
            check(ma["objects"] == mb["objects"], "character placements " + name)
    students, leaders = 0, 0
    for town in GYMS:
        room = b["maps"][f"Mn{town}Gym"]
        for actor in room["objects"]:
            trainer = actor["script"]
            if actor["object_type"] == "OBJECTTYPE_TRAINER":
                check(actor["radius"] > 0 and room["scripts"][trainer][0]["command"] == "trainer",
                      "sight trainer misconfigured in " + town)
                students += 1
            if "Leader" in trainer and actor["object_type"] == "OBJECTTYPE_SCRIPT":
                leaders += 1
    check((students, leaders) == (18, 8), f"gym student/leader coverage {students}/{leaders}")
    for name, labels in story.items():
        for label, pages in labels.items():
            expected = []
            for index, lines in enumerate(pages):
                for line_no, line in enumerate(lines):
                    expected.append({"command": "line" if line_no else "para" if index else "text",
                                     "args": [f'"{line}"']})
            expected.append({"command": "done", "args": []})
            check(b["maps"][name]["scripts"][label] == expected, f"compiled dialogue {name}/{label}")
    same_except(a["npcs"], b["npcs"], changed_maps, "regional NPC tables")
    script_labels = set()
    for name in changed_maps:
        for room in (a["maps"][name], b["maps"][name]):
            script_labels.update(room["scripts"])
            script_labels.update(room["script_text_bodies"])
            script_labels.add(room["attributes"]["map_scripts_label"])
            script_labels.add(room["attributes"]["map_events_label"])
    same_except(a["map_scripts"], b["map_scripts"], script_labels, "shared script index")
    desk = b["maps"]["MnLeagueLobby"]["scripts"]
    registration = desk["MnElyRegister"]
    for i in range(8):
        old_test = a["maps"]["MnLeagueLobby"]["scripts"]["MnElyRegister"][1 + 2*i]
        check(registration[1 + 2*i] == old_test and old_test["command"] == "checkflag",
              f"original badge check {i+1}")
        label = f"MnElyMissingBadge{i+1}"
        check(registration[2 + 2*i] == {"command": "iffalse", "args": [label]}, label)
        check(desk[label] == [{"command": "jumptext", "args": [label + "Text"]}], label)
    check(registration[17:] == a["maps"]["MnLeagueLobby"]["scripts"]["MnElyRegister"][17:],
          "League admission after all eight badges")
    home = b["maps"]["MnWinonaHome1F"]["scripts"]
    check(home["MnWinonaRestAtHome"] == a["maps"]["MnWinonaHome1F"]["scripts"]["MnWGuardianSurveyReturn"][4:],
          "unchanged healing and blackout destination at home")
    parties = a["trainers"]["trainers"], b["trainers"]["trainers"]
    changed = same_except(*parties, {"MN_GYM_1", "MN_GYM_2", "MN_GYM_3", "MN_GYM_4",
                                     "MN_GYM_8", "MN_GYM_1_TRAINER_1"}, "trainer party")
    check(len(changed) == 6, "five leaders and Tavel")
    slots = 0
    for trainer in changed:
        left, right = (p[trainer]["party"] for p in parties)
        check(len(left) == len(right), trainer + " party size")
        for original, revised in zip(left, right):
            same_except(original, revised, {"moves"}, trainer)
            check(len(original["moves"]) == len(revised["moves"]), trainer + " move slots")
            slots += sum(x != y for x, y in zip(original["moves"], revised["moves"]))
    check(slots == 6, "leader move changes")
    learners = 0
    for token, original in a["pokemon"].items():
        revised = b["pokemon"][token]
        if original != revised:
            same_except(original, revised, {"tmhm_learnset"}, token)
            prev, now = original["tmhm_learnset"], revised["tmhm_learnset"]
            check(now[:-1] == prev and now[-1] in {"THUNDERPUNCH", "STEEL_WING"}, token)
            learners += 1
    check(learners == 17, "new machine learners")
    flags = a["story_event_script_constants"]["global"], b["story_event_script_constants"]["global"]
    same_except(*flags, {"EVENT_MN_WINONA_CHAMPION_SUPPER"}, "event numbers")
    before, after = (json.loads(bytes(p["runtime_files"][POLICY])) for p in (old, new))
    check(after["schema_version"] == 8 and after["target_manifest"] == "northstar-lakeside-v1" and
          after["inherited"] == before and {s["modpack_hash"] for s in after["sources"]} == PARENTS,
          "save inheritance")
    allowed = sprite_inventory(old, new, root, native)
    artwork = {k for k in new["runtime_files"] if "/mn_gym_6" in k or "/mn_timekeeper." in k}
    same_except(old["runtime_files"], new["runtime_files"], allowed | artwork | {POLICY}, "runtime files")
    print("743 maps, nine edited. 20 installed conversations, 18 sight trainers, eight talk-to leaders, "
          "six leader move slots and 17 machine learners. All 450 battle views and 225 six-pose banks "
          "match reviewed art. Both Crossings parents and inherited save data are recorded.")


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("parent", type=Path)
    parser.add_argument("revised", type=Path)
    parser.add_argument("--pokemon-root", type=Path, default=REGION / "pokemon")
    parser.add_argument("--native-root", type=Path, default=Path("target/minnesota-pokemon/native"))
    args = parser.parse_args()
    sha, parent = load(args.parent)
    _, revised = load(args.revised)
    compare(sha, parent, revised, args.pokemon_root, args.native_root)
