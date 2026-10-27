#!/usr/bin/env python3
"""Read-only written-pack preservation and exact-source art proof for three cities.

Pass --city CONTRACT LAYOUT three times. Inputs are never promoted or exported.
This checks stored data and source pixels, not controller play or a GPU capture.
Requires the existing planning renderer's Pillow and cbor2 dependencies.
"""

import argparse
import base64
from collections import Counter
from copy import deepcopy
import hashlib
import io
import json
from pathlib import Path

import cbor2
from PIL import Image, ImageDraw

from audit_winding_shelter_541 import raw_identity
from render_archive import save_render
from render_reference import SourceArt, layout_blocks, load_pack


ROOT = Path(__file__).resolve().parents[3]
CITIES = {"MnMinneapolis", "MnDuluth", "MnStPaul"}
OPPOSITE = {"north": "south", "south": "north", "west": "east", "east": "west"}
FACING = {"north": "UP", "south": "DOWN", "west": "LEFT", "east": "RIGHT"}
FAIR_TEXTS = {
    "MN_ST_PAUL_FAIR_STEWARDText", "MN_ST_PAUL_FAIR_GUARD_WESTText",
    "MN_ST_PAUL_FAIR_GUARD_EASTText", "MN_ST_PAUL_SIGN_FAIRText",
}


def sha(raw):
    return hashlib.sha256(raw).hexdigest()


def fingerprint(value):
    return sha(cbor2.dumps(value, canonical=True))


def module_hash(value):
    return sha(json.dumps(value, sort_keys=True, separators=(",", ":"), ensure_ascii=False).encode())


def command(name, *args):
    return dict(command=name, args=list(args))


def text_body(lines):
    """Expected 18-column source text records; this does not run game scripts."""
    rows = []
    for line in lines:
        row = ""
        for word in line.split():
            if row and len(row) + 1 + len(word) > 18:
                rows.append(row)
                row = ""
            row += (" " if row else "") + word
        if row:
            rows.append(row)
    return [command("text" if i == 0 else "para" if i % 2 == 0 else "line", '"' + row + '"')
            for i, row in enumerate(rows)] + [command("done")]


def expected_scripts(source, clinic):
    scripts = {}
    for actor in source.get("residents", []):
        label, service = actor["id"] + "Text", actor.get("service")
        if service != "pokemart":
            scripts[label] = text_body(actor["lines"])
        if service == "pokemart":
            body = [command("faceplayer"), command("opentext"),
                    command("pokemart", "MARTTYPE_STANDARD", "MART_" + actor["id"]),
                    command("closetext"), command("end")]
        elif service == "heal_party":
            body = [command("faceplayer"), command("opentext"), command("writetext", label),
                    command("waitbutton"), command("special", "HealParty"),
                    command("blackoutmod", clinic), command("closetext"), command("end")]
        else:
            assert service is None, "unsupported activity source service"
            body = [command("jumptextfaceplayer", label)]
        scripts[actor["id"]] = body
    for sign in source.get("signs", []):
        label = sign["id"] + "Text"
        scripts[label] = text_body(sign["lines"])
        scripts[sign["id"]] = [command("jumptext", label)]
    for trainer in source.get("trainers", []):
        name = trainer["id"]
        seen, won, lost, after = [name + suffix for suffix in ["Seen", "Won", "Lost", "After"]]
        for label, field in [(seen, "before_lines"), (won, "after_lines"), (lost, "loss_lines")]:
            scripts[label] = text_body(trainer[field])
        scripts[name] = [command("trainer", trainer["trainer_class"], name, trainer["defeated_flag"],
                                 seen, won, lost, after)]
        scripts[after] = [command("endifjustbattled"), command("jumptextfaceplayer", won)]
    for item in source.get("items", []):
        scripts[item["id"]] = [command("itemball", item["item"], str(item["quantity"]))]
    scripts.update(source.get("programs", {}))
    return scripts


def expected_objects(before, source):
    objects = []
    for key in ["residents", "trainers", "items"]:
        for entry in source.get(key, []):
            obj = deepcopy(before["data"]["maps"]["NewBarkTown"]["objects"][0])
            obj.update(sprite=entry.get("sprite", "SPRITE_POKE_BALL"), x=entry["tile"][0],
                       y=entry["tile"][1], spritemovedata="SPRITEMOVEDATA_STANDING_DOWN",
                       move_range_x=0, move_range_y=0, pal=0, script=entry["id"],
                       object_identifier=entry["id"], event_flag="-1")
            if key != "items":
                obj["spritemovedata"] = "SPRITEMOVEDATA_STANDING_" + FACING[entry["facing"]]
            if key == "trainers":
                obj.update(object_type="OBJECTTYPE_TRAINER", radius=entry["sight_range"])
            elif key == "items":
                obj.update(object_type="OBJECTTYPE_ITEMBALL", event_flag=entry["flag"])
            objects.append(obj)
    return objects


def native_warp(index, tile, target, target_warp):
    return dict(index=index, x=tile[0], y=tile[1], target_map_constant=target,
                target_map=target, target_warp_id=target_warp)


def relocated(records, kind, plan):
    result = deepcopy(records)
    overrides = {(v["kind"], v["index"]): v for v in plan.get("anchor_overrides", [])}
    for index, record in enumerate(result):
        old = [record["x"], record["y"]]
        proposal = overrides.get((kind, index))
        if proposal:
            assert proposal["before"] == old, (plan["city"], kind, index, "stale matrix")
            at = proposal["after"]
        else:
            at = [old[i] + plan["translation_core"][i] for i in range(2)]
        record["x"], record["y"] = at
    return result


def section_rows(module):
    rows = [command("db", "0", "0"), command("def_warp_events")]
    for warp in module["events"]["warps"]:
        rows.append(command("warp_event", str(warp["x"]), str(warp["y"]),
                            warp["target_map_constant"], str(warp["target_warp_id"])))
    rows.append(command("def_coord_events"))
    for event in module["events"]["coord_events"]:
        rows.append(command("coord_event", str(event["x"]), str(event["y"]),
                            event["scene_id"] or "-1", event["script_name"]))
    rows.append(command("def_bg_events"))
    for event in module["events"]["bg_events"]:
        rows.append(command("bg_event", str(event["x"]), str(event["y"]), event["event_type"], event["script"]))
    rows.append(command("def_object_events"))
    for obj in module["objects"]:
        rows.append(command("object_event", *[str(obj[key]) for key in [
            "x", "y", "sprite", "spritemovedata", "move_range_x", "move_range_y", "hram_x", "hram_y",
            "pal", "object_type", "radius", "script", "event_flag"]]))
    return rows


def kind(pack, module, at):
    x, y = at
    width = module["attributes"]["width"]
    block = module["blocks"][y // 2 * width + x // 2]
    return pack["data"]["tilesets"][module["attributes"]["tileset_name"]]["collision"][f"{block:02x}"][y % 2 * 2 + x % 2]


def audit(before, after, inputs):
    checks = []

    def check(label, valid, detail=None):
        row = dict(check=label, passed=bool(valid))
        if detail is not None:
            row["detail"] = detail
        checks.append(row)

    def equal(label, expected, actual):
        valid = expected == actual
        check(label, valid, None if valid else dict(expected_sha256=fingerprint(expected), actual_sha256=fingerprint(actual)))

    a, b = before["data"], after["data"]
    plans = {plan["city"]: (plan, layout) for plan, layout in inputs}
    assert set(plans) == CITIES, "explicitly supply exactly these three cities"
    rooms = {room["id"]: (plan, room) for plan, _ in inputs for room in plan["rooms"]}
    neighbors = {link["target"] for plan, _ in inputs for link in plan["connections"]}
    owned = CITIES | neighbors
    check("baseline exactly543 maps and expected five unique reciprocal neighbors", len(a["maps"]) == 543 and len(neighbors) == 5)
    equal("exact contract-derived new room roster", set(a["maps"]) | set(rooms), set(b["maps"]))
    changed = sorted(name for name, old in a["maps"].items() if b["maps"].get(name) != old)
    equal("exact three city/five reciprocal-neighbor changed roster", owned, set(changed))
    for name, old in a["maps"].items():
        if name not in owned:
            equal(name + " complete original module exact", old, b["maps"].get(name))
    revised = {}
    allowed_old_globals = set()
    all_expected_scripts = {}
    for city, (plan, layout) in plans.items():
        old, new = a["maps"][city], b["maps"][city]
        check(city + " entire installed preimage certified by contract", module_hash(old) == plan["native_module_sha256"])
        overrides = plan.get("anchor_overrides", [])
        limits = dict(object=len(old["objects"]), warp=len(old["events"]["warps"]),
                      background=len(old["events"]["bg_events"]), coordinate=len(old["events"]["coord_events"]))
        check(city + " unique bounded anchor override identities", len({(v["kind"], v["index"]) for v in overrides}) == len(overrides)
              and all(v["kind"] in limits and 0 <= v["index"] < limits[v["kind"]] for v in overrides))
        expected_attr = deepcopy(old["attributes"])
        expected_attr.update(width=plan["dimensions_metatiles"][0], height=plan["dimensions_metatiles"][1],
                             tileset_name=layout["compiled_tileset"])
        for link in plan["connections"]:
            matches = [v for v in expected_attr["connections"] if v["direction"] == link["direction"] and v["target_map"] == link["target"]]
            assert len(matches) == 1 and matches[0]["offset"] == link["old_offset"]
            matches[0]["offset"] = link["offset"]
        equal(city + " only contracted native attributes differ", expected_attr, new["attributes"])
        equal(city + " ENTIRE authored source terrain grid", layout_blocks(layout), new["blocks"])
        equal(city + " original objects only move by exact indexed matrix", relocated(old["objects"], "object", plan), new["objects"][:len(old["objects"])])
        equal(city + " exact additional source objects", expected_objects(before, plan), new["objects"][len(old["objects"]):])
        event_expected = {key: relocated(old["events"][key], direction, plan) for key, direction in
                          [("warps", "warp"), ("bg_events", "background"), ("coord_events", "coordinate")]}
        for door in plan["doors"]:
            target = b["maps"][door["target"]]["attributes"]["map_constant"]
            event_expected["warps"].append(native_warp(door["warp_index"], door["tile"], target, door["target_warp"]))
            event_expected["warps"].append(native_warp(door["return_warp_index"], door["return_tile"], old["attributes"]["map_constant"], door["return_warp_index"]))
        event_expected["bg_events"] += [dict(x=v["tile"][0], y=v["tile"][1], event_type="BGEVENT_READ", script=v["id"]) for v in plan["signs"]]
        equal(city + " complete events preserve old identities and exact new paired doors", event_expected, new["events"])
        scripts = expected_scripts(plan, plan["clinic_constant"])
        all_expected_scripts[city] = scripts
        for revision in plan.get("text_revisions", []):
            label = revision["label"]
            check(city + " narrowly authorized existing fair text " + label, city == "MnStPaul" and label in FAIR_TEXTS)
            equal(label + " exact old text preimage", text_body(revision["before"]), old["scripts"].get(label))
            revised[label] = text_body(revision["after"])
        expected_raw = dict(old["scripts"])
        expected_raw.update({key: value for key, value in revised.items() if key in expected_raw})
        expected_raw.update(scripts)
        equal(city + " all old story/raw programs exact plus declared source programs", expected_raw, new["scripts"])
        # Original derived programs remain exact prefixes; additions may not replace them.
        exceptions = {"attributes", "blocks", "objects", "events", "scripts", "map_event_section_commands"}
        for field, value in old.items():
            if field in exceptions:
                continue
            actual = new[field]
            if isinstance(value, list):
                equal(city + " original typed prefix " + field, value, actual[:len(value)])
                additions = actual[len(value):]
                if field.startswith("script_"):
                    check(city + " added typed records belong only to declared source labels " + field,
                          all(row.get("source_script") in scripts for row in additions))
                    check(city + " added typed instructions reference exact raw command indices " + field,
                          all("command_index" not in row or
                              (0 <= row["command_index"] < len(scripts[row["source_script"]])
                               and scripts[row["source_script"]][row["command_index"]]["command"] == row["command"])
                              for row in additions if row.get("source_script") in scripts))
            elif isinstance(value, dict):
                expected = deepcopy(value)
                if field == "script_text_bodies":
                    for label, raw in revised.items():
                        if label in expected:
                            expected[label] = dict(label=label, commands=[dict(row, command_index=i) for i, row in enumerate(raw)])
                equal(city + " original typed records " + field, expected, {key: actual.get(key) for key in expected})
                if field in ["script_text_bodies", "trainer_scripts"]:
                    check(city + " added typed dictionary belongs only to declared source labels " + field,
                          set(actual) - set(expected) <= set(scripts))
            else:
                equal(city + " exact original field " + field, value, actual)
        allowed_old_globals.add(old["attributes"]["map_events_label"])
    equal("exact four approved old wayfinding text exceptions", FAIR_TEXTS, set(revised))
    for name in neighbors:
        expected = deepcopy(a["maps"][name])
        for city, (plan, _) in plans.items():
            for link in plan["connections"]:
                if link["target"] == name:
                    matches = [v for v in expected["attributes"]["connections"] if v["target_map"] == city and v["direction"] == OPPOSITE[link["direction"]]]
                    assert len(matches) == 1 and matches[0]["offset"] == -link["old_offset"]
                    matches[0]["offset"] = -link["offset"]
        equal(name + " entire reciprocal module only contracted offsets differ", expected, b["maps"][name])
    for name, (plan, room) in rooms.items():
        new = b["maps"][name]
        equal(name + " complete source room blocks", [int(v, 16) for row in room["rows"] for v in row.split()], new["blocks"])
        expected_attr = deepcopy(a["maps"][room["template"]]["attributes"])
        expected_attr.update(width=room["dimensions_metatiles"][0], height=room["dimensions_metatiles"][1],
                             tileset_name=room["tileset"], border_block=room["border_block"],
                             map_constant=room["native_constant"], map_group_constant=b["maps"][plan["city"]]["attributes"]["map_group_constant"],
                             location=b["maps"][plan["city"]]["attributes"]["location"], blocks_label=name + "_Blocks",
                             map_scripts_label=name + "_MapScripts", map_events_label=name + "_MapEvents",
                             connections=[], connection_flags=None)
        equal(name + " exact source/template attributes", expected_attr, new["attributes"])
        equal(name + " exact source residents/trainers/items", expected_objects(before, room), new["objects"])
        expected_events = dict(warps=[native_warp(v["index"], v["tile"], v["target"], v["target_warp"]) for v in room["warps"]], coord_events=[],
                               bg_events=[dict(x=v["tile"][0], y=v["tile"][1], event_type="BGEVENT_READ", script=v["id"]) for v in room["signs"]])
        equal(name + " every source indexed warp and reading exact", expected_events, new["events"])
        all_expected_scripts[name] = expected_scripts(room, plan["clinic_constant"])
        equal(name + " exact source ordinary/service/quest raw programs", all_expected_scripts[name], new["scripts"])
        check(name + " explicit arrival is native ordinary Floor", kind(after, new, room["arrival_core"]) == "FLOOR")
        for warp in room["warps"]:
            if warp["passive"]:
                check(name + " passive floor warp " + str(warp["index"]), kind(after, new, warp["tile"]) == "FLOOR")
    # One full normalization proves every otherwise-unmentioned GameDataSet field.
    normalized = dict(b)
    for field in ["maps", "map_attributes", "npcs"]:
        table = dict(b[field])
        for name in rooms:
            table.pop(name, None)
        for name in (owned if field != "npcs" else CITIES):
            table[name] = a[field][name]
        normalized[field] = table
    for field in ["map_dimensions", "runtime_map_metadata", "map_blocks"]:
        table = dict(b[field])
        for name in rooms:
            key = b["maps"][name]["attributes"]["blocks_label" if field == "map_blocks" else "map_constant"]
            table.pop(key, None)
        for city in CITIES:
            key = a["maps"][city]["attributes"]["blocks_label" if field == "map_blocks" else "map_constant"]
            if field == "runtime_map_metadata":
                expected = dict(a[field][key], width=b["maps"][city]["attributes"]["width"], height=b["maps"][city]["attributes"]["height"])
                equal(city + " only metadata extent changes", expected, b[field][key])
            elif field == "map_dimensions":
                equal(city + " native dimension record", dict(width=64, height=48), b[field][key])
            table[key] = a[field][key]
        normalized[field] = table
    # New map identities must stay in the existing city group and be unique.
    metadata_ids = [(v["groupId"], v["mapId"]) for v in b["runtime_map_metadata"].values()]
    check("all native map group/ID pairs unique", len(metadata_ids) == len(set(metadata_ids)))
    expected_map_ids = {}
    for plan, _ in inputs:
        group = a["runtime_map_metadata"][a["maps"][plan["city"]]["attributes"]["map_constant"]]["groupId"]
        used = {v["mapId"] for v in a["runtime_map_metadata"].values() if v["groupId"] == group}
        for room in plan["rooms"]:
            identifier = next(value for value in range(1, 256) if value not in used)
            expected_map_ids[room["id"]] = identifier
            used.add(identifier)
    for name, (plan, room) in rooms.items():
        meta = b["runtime_map_metadata"][room["native_constant"]]
        city_meta = b["runtime_map_metadata"][b["maps"][plan["city"]]["attributes"]["map_constant"]]
        expected = dict(city_meta, name=name, constant=room["native_constant"], width=room["dimensions_metatiles"][0],
                        height=room["dimensions_metatiles"][1], environment=b["maps"][name]["attributes"]["environment"],
                        phoneService=b["maps"][name]["attributes"]["phone_service"], mapId=expected_map_ids[name])
        equal(name + " exact city-group native metadata", expected, meta)
        check(name + " native map ID supports whole room roster", 1 <= meta["mapId"] <= 255)
    expected_new_globals = {}
    for name, scripts in all_expected_scripts.items():
        expected_new_globals.update({k: v for k, v in scripts.items() if k not in a["map_scripts"]})
        for attr, section in [("map_scripts_label", "map_script_section_commands"), ("map_events_label", "map_event_section_commands")]:
            label = b["maps"][name]["attributes"][attr]
            if label not in a["map_scripts"]:
                expected_new_globals[label] = [{k: v for k, v in row.items() if k != "command_index"} for row in b["maps"][name][section]]
    equal("exact source-derived additional global program/section label roster", set(expected_new_globals), set(b["map_scripts"]) - set(a["map_scripts"]))
    for label, expected in expected_new_globals.items():
        equal("new global program exact " + label, expected, b["map_scripts"].get(label))
    scripts = dict(b["map_scripts"])
    for label in expected_new_globals:
        scripts.pop(label, None)
    for label in allowed_old_globals | set(revised):
        scripts[label] = a["map_scripts"][label]
    normalized["map_scripts"] = scripts
    marts = {"MART_" + actor["id"]: actor["stock"] for plan, _ in inputs
             for source in [plan] + plan["rooms"] for actor in source["residents"] if actor.get("service") == "pokemart"}
    equal("exact new source vendor stock catalog", marts, {key: value for key, value in b["marts"].items() if key not in a["marts"]})
    normalized["marts"] = {key: value for key, value in b["marts"].items() if key not in marts}
    trainers = {trainer["id"]: trainer for plan, _ in inputs for source in [plan] + plan["rooms"] for trainer in source["trainers"]}
    equal("exact new optional trainer roster", set(trainers), set(b["trainers"]["trainers"]) - set(a["trainers"]["trainers"]))
    for name, source in trainers.items():
        native = b["trainers"]["trainers"][name]
        equal(name + " exact trainer class/name/source party", [source["trainer_class"], source["display_name"].upper(), [(p["species"], p["level"]) for p in source["party"]]],
              [native["trainer_class"], native["name"], [(p["species"], p["level"]) for p in native["party"]]])
        check(name + " original native species/moves only", all(member["species"] in a["pokemon"] and all(move["name"] in a["moves"] for move in member["moves"]) for member in native["party"]))
    normalized["trainers"] = dict(b["trainers"], trainers={key: value for key, value in b["trainers"]["trainers"].items() if key not in trainers})
    event_ids = [flag for plan, _ in inputs for flag in plan["event_flags"]]
    constants = b["story_event_script_constants"]["global"]
    equal("exact fresh event constant roster", set(event_ids), set(constants) - set(a["story_event_script_constants"]["global"]))
    old_max = max(value for key, value in a["story_event_script_constants"]["global"].items() if key.startswith("EVENT_"))
    equal("fresh event IDs append without overwriting source IDs", set(range(old_max + 1, old_max + 1 + len(event_ids))), {constants[flag] for flag in event_ids})
    for plan, _ in inputs:
        values = [constants[flag] for flag in plan["event_flags"]]
        check(plan["city"] + " exact ordered contiguous event allocation", values == list(range(values[0], values[0] + len(values))))
    normalized["story_event_script_constants"] = dict(b["story_event_script_constants"])
    normalized["story_event_script_constants"]["global"] = {key: value for key, value in constants.items() if key not in event_ids}
    spawns = dict(b["runtime_spawn_points"])
    for key, old in a["runtime_spawn_points"].items():
        expected = deepcopy(old)
        if old["mapName"] in plans:
            plan = plans[old["mapName"]][0]
            matching = [v for v in plan.get("spawn_overrides", []) if v["identifier"] == old["identifier"]]
            at = matching[0]["after"] if matching else [old["tileX"] + plan["translation_core"][0], old["tileY"] + plan["translation_core"][1]]
            if matching:
                check(key + " exact spawn override preimage", matching[0]["before"] == [old["tileX"], old["tileY"]])
            expected.update(tileX=at[0], tileY=at[1], metatileX=at[0] // 2, metatileY=at[1] // 2, subtileX=at[0] % 2, subtileY=at[1] % 2)
        equal(key + " exact original HOME/recovery or explicit exterior relocation", expected, spawns.get(key))
        spawns[key] = old
    additions = {key: value for key, value in spawns.items() if key not in a["runtime_spawn_points"]}
    clinic_plans = [plan for plan, _ in inputs if plan.get("clinic_recovery_core") is not None]
    check("only the declared Duluth clinic adds recovery", len(additions) == len(clinic_plans) == 1)
    for plan in clinic_plans:
        matching = [(key, value) for key, value in additions.items() if value["mapConstant"] == plan["clinic_constant"]]
        check(plan["city"] + " clinic recovery bound to original clinic", len(matching) == 1)
        if matching:
            key, recovery = matching[0]
            x, y = plan["clinic_recovery_core"]
            meta = b["runtime_map_metadata"][plan["clinic_constant"]]
            identifier = max(v["identifier"] for v in a["runtime_spawn_points"].values()) + 1
            expected = dict(identifier=identifier, mapName=meta["name"], mapConstant=meta["constant"], groupName=meta["groupName"], groupId=meta["groupId"], mapId=meta["mapId"], tileX=x, tileY=y, metatileX=x // 2, metatileY=y // 2, subtileX=x % 2, subtileY=y % 2)
            equal("exact appended clinic recovery record", expected, recovery)
            check("new clinic recovery identifier key exact", key == str(identifier))
            spawns.pop(key)
    normalized["runtime_spawn_points"] = spawns
    landmarks = deepcopy(b["pokegear_landmarks"])
    expected_landmarks = {name: b["maps"][plan["city"]]["attributes"]["location"] for name, (plan, _) in rooms.items()}
    equal("only new rooms acquire existing city landmark aliases", expected_landmarks, {key: value for key, value in landmarks["map_to_landmark"].items() if key not in a["pokegear_landmarks"]["map_to_landmark"]})
    for name in rooms:
        landmarks["map_to_landmark"].pop(name, None)
    normalized["pokegear_landmarks"] = landmarks
    aliases = {layout["compiled_tileset"]: layout for _, layout in inputs if layout.get("derived_blocks")}
    equal("exact city art alias roster", set(aliases), set(b["tilesets"]) - set(a["tilesets"]))
    normalized["tilesets"] = {key: value for key, value in b["tilesets"].items() if key not in aliases}
    moves = deepcopy(b["field_moves"])
    for alias, layout in aliases.items():
        equal(alias + " exact inherited Cut replacements", a["field_moves"]["cut"]["replacements"][layout["tileset"]], moves["cut"]["replacements"].get(alias))
        moves["cut"]["replacements"].pop(alias, None)
    normalized["field_moves"] = moves
    equal("ENTIRE GameDataSet exact after only explicitly checked city changes", a, normalized)
    # All stored split mirrors remain coherent. Existing text-body disagreements
    # are compared as an exact preimage rather than silently accepted anew.
    disagreements = {}
    for stage, data in [("before", a), ("after", b)]:
        disagreements[stage] = sorted((name, label, fingerprint(raw), fingerprint(data["map_scripts"].get(label)))
            for name, module in data["maps"].items() for label, raw in module["scripts"].items() if data["map_scripts"].get(label) != raw)
    equal("only the exact pre-existing text-body mirror disagreements retained", disagreements["before"], disagreements["after"])
    for name, module in b["maps"].items():
        equal(name + " attribute mirror", module["attributes"], b["map_attributes"].get(name))
        equal(name + " NPC mirror", module["objects"], b["npcs"].get(name))
        equal(name + " encoded block mirror", module["blocks"], list(base64.b64decode(b["map_blocks"][module["attributes"]["blocks_label"]], validate=True)))
        for attr, field in [("map_scripts_label", "map_script_section_commands"), ("map_events_label", "map_event_section_commands")]:
            expected = [{k: v for k, v in row.items() if k != "command_index"} for row in module[field]]
            equal(name + " global raw " + field, expected, b["map_scripts"].get(module["attributes"][attr]))
            check(name + " sequential raw indices " + field, [row["command_index"] for row in module[field]] == list(range(len(expected))))
        if name in CITIES or name in rooms:
            equal(name + " raw events describe actual stored actors and warps", section_rows(module), [{k: v for k, v in row.items() if k != "command_index"} for row in module["map_event_section_commands"]])
    for plan, _ in inputs:
        for link in plan["connections"]:
            city, neighbor = b["maps"][plan["city"]], b["maps"][link["target"]]
            for index, (town, route) in enumerate(zip(link["town_edge"], link["route_edge"])):
                check(plan["city"] + " reciprocal seam dry Floor lane " + link["direction"] + str(index), kind(after, city, town) == kind(after, neighbor, route) == "FLOOR")
                if link["direction"] in ["north", "south"]:
                    aligned = town[0] == route[0] + 2 * link["offset"]
                    boundary = town[1] == (0 if link["direction"] == "north" else city["attributes"]["height"] * 2 - 1)
                    boundary &= route[1] == (neighbor["attributes"]["height"] * 2 - 1 if link["direction"] == "north" else 0)
                else:
                    aligned = town[1] == route[1] + 2 * link["offset"]
                    boundary = town[0] == (0 if link["direction"] == "west" else city["attributes"]["width"] * 2 - 1)
                    boundary &= route[0] == (neighbor["attributes"]["width"] * 2 - 1 if link["direction"] == "west" else 0)
                check(plan["city"] + " exact native seam alignment " + link["direction"] + str(index), aligned and boundary)
    for key in before:
        if key not in ["identity", "data", "runtime_files", "report"]:
            equal("entire top-level field exact " + key, before[key], after[key])
    original_file_errors = [name for name, raw in before["runtime_files"].items() if after["runtime_files"].get(name) != raw]
    check("every original native art/source/audio file byte exact", not original_file_errors, original_file_errors)
    expected_files = {path for alias in aliases for path in [f"data/tilesets/{alias}.json", f"data/tilesets/{alias}_metatiles.bin", f"data/tilesets/{alias}_palette_map.json", f"gfx/tilesets/{alias}.png", f"gfx/tilesets/{alias}.2bpp"]}
    equal("only exact five runtime files per new art alias", expected_files, set(after["runtime_files"]) - set(before["runtime_files"]))
    expected_manifests = [manifest for plan, layout in inputs for manifest in
        ([plan["city"].lower() + "-expanded-city-art"] if layout.get("derived_blocks") else [])
        + [room["id"].lower() + "-city-room" for room in plan["rooms"]] + [plan["city"].lower() + "-expanded-city"]]
    report = deepcopy(after["report"])
    equal("compile-report original manifest prefix exact", before["report"]["manifests"], report["manifests"][:len(before["report"]["manifests"])])
    suffix = report["manifests"][len(before["report"]["manifests"]):]
    equal("compile report exact source-derived additions, no duplicates", Counter(expected_manifests), Counter(suffix))
    equal("compile-report map count", len(b["maps"]), report["maps"])
    report["maps"], report["manifests"] = before["report"]["maps"], before["report"]["manifests"]
    for key in ["reachable_maps", "solvable_maps"]:
        equal("compile-report exact existing-plus-room " + key, sorted(set(before["report"][key]) | set(rooms)), report[key])
        report[key] = before["report"][key]
    equal("whole remaining compile report exact", before["report"], report)
    identity = deepcopy(after["identity"])
    equal("native runtime identity exact ordered manifest suffix", before["identity"]["runtime_modpack_id"] + "+" + "+".join(suffix), identity["runtime_modpack_id"])
    identity.update(runtime_modpack_id=before["identity"]["runtime_modpack_id"], content_hash=before["identity"]["content_hash"], maps=before["identity"]["maps"])
    equal("remaining identity fields exact", before["identity"], identity)
    return checks, dict(original_maps=len(a["maps"]), final_maps=len(b["maps"]), additional_rooms=len(rooms),
                        changed_original_maps=changed, fresh_flags=len(event_ids), new_trainers=len(trainers), new_marts=len(marts),
                        original_runtime_files=len(before["runtime_files"]), new_art_aliases=sorted(aliases),
                        retained_old_mirror_disagreements=disagreements["before"])


def audit_art(before, after, inputs, output=None):
    checks, witnesses = [], []
    old_art, new_art = SourceArt(before), SourceArt(after)

    def check(label, valid):
        checks.append(dict(check=label, passed=bool(valid)))

    for plan, layout in inputs:
        recipes = layout.get("derived_blocks", [])
        if not recipes:
            continue
        alias, source = layout["compiled_tileset"], layout["tileset"]
        old_definition, definition = before["data"]["tilesets"][source], after["data"]["tilesets"][alias]
        old_layout = bytes(before["runtime_files"][f"data/tilesets/{source}_metatiles.bin"])
        native_layout = bytes(after["runtime_files"][f"data/tilesets/{alias}_metatiles.bin"])
        check(alias + " inherited metatile glyph references exact", native_layout[:len(old_layout)] == old_layout)
        check(alias + " exact contiguous recipe extent", len(native_layout) == len(old_layout) + 16 * len(recipes))
        expected_collision = deepcopy(old_definition["collision"])
        for recipe in recipes:
            assert recipe.get("tile_row_order") is None and recipe.get("tile_row_sources") is None
            expected_collision[recipe["id"]] = before["data"]["tilesets"][recipe["source_tileset"]]["collision"][recipe["source_block"]]
            check(alias + " exact source collision recipe " + recipe["id"], recipe["collision"] == expected_collision[recipe["id"]])
        check(alias + " entire collision catalog exact", definition["collision"] == expected_collision)
        check(alias + " collision runtime JSON exact", json.loads(bytes(after["runtime_files"][f"data/tilesets/{alias}.json"])) == expected_collision)
        check(alias + " palette-map runtime JSON exact", json.loads(bytes(after["runtime_files"][f"data/tilesets/{alias}_palette_map.json"])) == definition["palette_map"])
        check(alias + " no other tileset definition fields change", {k: v for k, v in definition.items() if k not in ["palette_map", "collision"]} == {k: v for k, v in old_definition.items() if k not in ["palette_map", "collision"]})
        # SourceArt uses packed PNG glyphs. Independently decode native 2bpp so
        # a PNG-only preview cannot hide a different runtime bitmap.
        sheet = Image.open(io.BytesIO(bytes(after["runtime_files"][f"gfx/tilesets/{alias}.png"]))).convert("RGB")
        raw = bytes(after["runtime_files"][f"gfx/tilesets/{alias}.2bpp"])
        errors = 0
        for index in range(len(raw) // 16):
            for y in range(8):
                lo, hi = raw[index * 16 + y * 2:index * 16 + y * 2 + 2]
                for x in range(8):
                    color = ((lo >> (7 - x)) & 1) | (((hi >> (7 - x)) & 1) << 1)
                    value = [255, 170, 85, 0][color]
                    errors += sheet.getpixel((index % (sheet.width // 8) * 8 + x, index // (sheet.width // 8) * 8 + y)) != (value, value, value)
        check(alias + " PNG SourceArt glyph pixels match exact native 2bpp", errors == 0)
        for group in ["morn", "day", "nite", "dark"]:
            originals, installed = old_art.blocks(source, group), new_art.blocks(alias, group)
            for index, block in enumerate(originals):
                check(f"{alias} {group} inherited block {index:02x} exact RGB pixels", block.tobytes() == installed[index].tobytes())
            panel = Image.new("RGB", (440, max(1, len(recipes)) * 44), (18, 25, 34))
            draw = ImageDraw.Draw(panel)
            for row, recipe in enumerate(recipes):
                index = int(recipe["id"], 16)
                expected = old_art.blocks(recipe["source_tileset"], group, recipe.get("palette_override"))[int(recipe["source_block"], 16)]
                actual = installed[index]
                valid = expected.tobytes() == actual.tobytes()
                check(f"{alias} {group} imported block {recipe['id']} exact recipe RGB pixels", valid)
                panel.paste(expected, (4, row * 44 + 4))
                panel.paste(actual, (44, row * 44 + 4))
                draw.text((84, row * 44 + 4), f"{group} {recipe['id']} <- {recipe['source_tileset']}:{recipe['source_block']}", fill="white")
                draw.text((84, row * 44 + 18), "SOURCE RECONSTRUCTION " + ("MATCH" if valid else "MISMATCH"), fill="white")
                witnesses.append(dict(alias=alias, group=group, block=recipe["id"], source_tileset=recipe["source_tileset"], source_block=recipe["source_block"],
                                      source_rgb_sha256=sha(expected.tobytes()), installed_rgb_sha256=sha(actual.tobytes()), collision=recipe["collision"]))
            if output:
                save_render(panel, output / f"{alias}-{group}-source-reconstruction.png")
    return checks, witnesses


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--before", type=Path, required=True)
    parser.add_argument("--pack", type=Path, required=True)
    parser.add_argument("--city", nargs=2, type=Path, action="append", required=True, metavar=("CONTRACT", "LAYOUT"))
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--negative-checks", action="store_true")
    args = parser.parse_args()
    args.output.mkdir(parents=True, exist_ok=False)
    inputs = [(json.loads(contract.read_bytes()), json.loads(layout.read_bytes())) for contract, layout in args.city]
    paths = [args.before, args.pack] + [path for pair in args.city for path in pair]
    hashes = {str(path): sha(path.read_bytes()) for path in paths}
    before, after = load_pack(args.before), load_pack(args.pack)
    checks, summary = audit(before, after, inputs)
    art_checks, witnesses = audit_art(before, after, inputs, args.output)
    checks += art_checks
    identities = {str(path): raw_identity(path) for path in [args.before, args.pack]}
    checks.append(dict(check="exact stored CBOR identities independently recomputed", passed=all(value["exact"] for value in identities.values())))
    negatives = []
    if args.negative_checks:
        def damaged_map(name, mutate):
            damaged = dict(after)
            damaged["data"] = dict(after["data"])
            damaged["data"]["maps"] = dict(after["data"]["maps"])
            damaged["data"]["maps"][name] = deepcopy(after["data"]["maps"][name])
            mutate(damaged["data"]["maps"][name])
            return damaged

        for label, damaged in [
            ("original gym party/script cache changed", damaged_map("MnMinneapolisGym", lambda module: module["scripts"].clear())),
            ("old city anchor moved outside explicit matrix", damaged_map("MnMinneapolis", lambda module: module["objects"][0].__setitem__("x", module["objects"][0]["x"] + 1))),
            ("new source terrain pixel block changed", damaged_map("MnMinneapolis", lambda module: module["blocks"].__setitem__(0, module["blocks"][0] ^ 1))),
            ("new room reward/raw program changed", damaged_map("MnMinneapolisPrintShop", lambda module: module["scripts"]["MN_MP_PRINTSHOP_HOST"].clear())),
            ("new cached command attaches to original story label", damaged_map("MnMinneapolis", lambda module: module["script_flag_commands"].append(dict(module["script_flag_commands"][0])))),
        ]:
            failed = [row["check"] for row in audit(before, damaged, inputs)[0] if not row["passed"]]
            negatives.append(dict(corruption=label, rejected=bool(failed), failed_checks=failed[:12]))
        # Mutating only alias glyph pixels catches renderer-only/native divergence.
        damaged = dict(after)
        damaged["runtime_files"] = dict(after["runtime_files"])
        alias = next(layout["compiled_tileset"] for _, layout in inputs if layout.get("derived_blocks"))
        path = f"gfx/tilesets/{alias}.2bpp"
        raw = bytearray(damaged["runtime_files"][path])
        raw[0] ^= 1
        damaged["runtime_files"][path] = bytes(raw)
        failed = [row["check"] for row in audit_art(before, damaged, inputs)[0] if not row["passed"]]
        negatives.append(dict(corruption="native alias bitmap differs from SourceArt PNG", rejected=bool(failed), failed_checks=failed[:12]))
        damaged = dict(after)
        damaged["data"] = dict(after["data"])
        damaged["data"]["tilesets"] = dict(after["data"]["tilesets"])
        damaged["data"]["tilesets"][alias] = deepcopy(after["data"]["tilesets"][alias])
        recipe = next(layout["derived_blocks"][0] for _, layout in inputs if layout.get("derived_blocks"))
        damaged["data"]["tilesets"][alias]["collision"][recipe["id"]][0] = "WATER"
        failed = [row["check"] for row in audit_art(before, damaged, inputs)[0] if not row["passed"]]
        negatives.append(dict(corruption="native imported collision differs from declared source", rejected=bool(failed), failed_checks=failed[:12]))
    checks.append(dict(check="all input bytes stable throughout independent audit", passed=hashes == {str(path): sha(path.read_bytes()) for path in paths}))
    report = dict(status="PASS" if all(row["passed"] for row in checks) and all(row["rejected"] for row in negatives) else "FAIL",
                  scope="Read-only stored-native preservation, declared contract/source geometry and exact SourceArt RGB/2bpp/collision comparison. No GPU, production-controller, earning/progression, PCM resynthesis or uncached-runtime claim.",
                  input_sha256=hashes, exact_native_content_identities=identities, tool_sha256=sha(Path(__file__).read_bytes()),
                  summary=summary, checks=checks, corruption_negatives=negatives, source_art_witnesses=witnesses)
    raw = (json.dumps(report, indent=2) + "\n").encode()
    path = args.output / ("proof-" + sha(raw) + ".json")
    path.write_bytes(raw)
    (args.output / "audit_city_expansions.py.snapshot").write_bytes(Path(__file__).read_bytes())
    print(json.dumps(dict(status=report["status"], report=str(path), checks=len(checks), negative_checks=len(negatives),
                          failed=[row for row in checks if not row["passed"]]), indent=2))
    raise SystemExit(0 if report["status"] == "PASS" else 1)


if __name__ == "__main__":
    main()
