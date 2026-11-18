#!/usr/bin/env python3
"""Compare a compiled outdoor revision with its own parent and render both route exteriors."""
import argparse
import base64
import copy
import json
import re
import sys
from pathlib import Path

from PIL import Image

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE.parents[1] / "tools"))
from render_reference import SourceArt, load_pack


ROUTE_SOURCES = ("driftless", "river-bluffs", "iron-range", "iron-descent", "northern-routes",
                 "shore-north", "lake-country", "southern-shore", "lake-piers")
ART_SOURCES = ("iron-art", "northern-art", "driftless-art", "southern-shore-art", "lake-pier-art")


def read(name):
    return json.loads((HERE / name).read_text())


def equal(actual, expected, label):
    if actual != expected:
        raise ValueError(label)


def whole(pack, art, route):
    chunks = {}
    widths, heights = {}, {}
    for name, m in pack["data"]["maps"].items():
        match = re.fullmatch(rf"MnR{route:02}C(\d+)R(\d+)", name)
        if match:
            x, y = map(int, match.groups())
            a = m["attributes"]
            chunks[x, y] = art.render(a["tileset_name"], a["width"], a["height"], m["blocks"])
            w, h = chunks[x, y].size
            widths[x] = max(w, widths.get(x, 0))
            heights[y] = max(h, heights.get(y, 0))
    image = Image.new("RGB", (sum(widths.values()), sum(heights.values())), "#242629")
    for (x, y), chunk in chunks.items():
        image.paste(chunk, (sum(widths[i] for i in range(x)), sum(heights[i] for i in range(y))))
    return image


def dock_pages(row):
    base = HERE.parents[1]
    label = row["label"]
    if row["source"] in ("dialogue_overhaul", "story_flow"):
        story = json.loads((base / "story.json").read_text())
        pages = (story["dialogue_overhaul"]["texts"] if row["source"] == "dialogue_overhaul"
                 else story["story_flow_revision"]["dialogue"])
        return pages[row["map"]][label]
    route = json.loads((base / "routes" / row["source"]).read_text())
    lines = next(sign["lines"] for sign in route["signs"] if sign["id"] == label.removesuffix("Text"))
    return [lines[i:i+2] for i in range(0, len(lines), 2)]


def text_program(pages):
    commands = []
    for page, lines in enumerate(pages):
        for index, line in enumerate(lines):
            op = "line" if index else ("para" if page else "text")
            commands.append({"command": op, "args": [json.dumps(line, ensure_ascii=False)]})
    return commands + [{"command": "done", "args": []}]


def review(before, after):
    layouts = [row for file in ROUTE_SOURCES for row in read(file + ".json")]
    cities = read("city-piers.json")
    city_names = {"MnBrainerd"} | {c["map"] for c in cities}
    expected = {r["map"] for r in layouts} | city_names
    alias = "mn_shiner_narrows"
    a, b = before["data"], after["data"]
    dialogue = read("dock-dialogue.json")
    expected.update(row["map"] for row in dialogue)
    scripts = dict(a["map_scripts"])
    revised_text = {}
    for row in dialogue:
        name, label = row["map"], row["label"]
        old_text = text_program(row["previous"])
        equal(a["map_scripts"][label], old_text, label + " old global wording")
        equal(a["maps"][name]["scripts"][label], old_text, label + " old local wording")
        revised_text[name, label] = scripts[label] = text_program(dock_pages(row))
    equal(scripts, b["map_scripts"], "global text and script programs")
    equal(before["audio_manifest"], after["audio_manifest"], "audio manifest")
    equal(a.keys(), b.keys(), "game catalogs")
    for k in a.keys() - {"maps", "map_blocks", "tilesets", "field_moves", "map_attributes", "map_scripts"}:
        equal(a[k], b[k], k)
    equal(a["maps"].keys(), b["maps"].keys(), "map inventory")
    equal({k for k in a["maps"] if a["maps"][k] != b["maps"][k]}, expected, "changed maps")

    recipes = [ink["design"] for ink in read("iron-ink.json")]
    recipes += [row for file in ART_SOURCES for row in read(file + ".json")]
    ids, next_id = {}, {}
    for recipe in recipes:
        t = recipe["tileset"]
        bank = f"data/tilesets/{t}_metatiles.bin"
        old, new = before["runtime_files"][bank], after["runtime_files"][bank]
        i = next_id.get(t, len(old) // 16)
        equal(old, new[:len(old)], t + " original metatiles")
        if "tiles" in recipe:
            equal(list(new[i*16:(i+1)*16]), recipe["tiles"], t + " metatile recipe")
        equal(recipe.get("collision", a["tilesets"][t]["collision"][f"{recipe['collision_from']:02x}"]),
              b["tilesets"][t]["collision"][f"{i:02x}"], t + " collision")
        ids[t, recipe.get("id", recipe.get("block"))] = i
        next_id[t] = i + 1
    brainerd = read("brainerd-pier.json")
    brainerd["map"] = "MnBrainerd"
    brainerd["tileset"] = a["maps"]["MnBrainerd"]["attributes"]["tileset_name"]
    brainerd["shore"] = [{**cell, "to": 0x35} for cell in brainerd["open_water"]]
    for pier in [brainerd, *cities]:
        bank_name = pier["tileset"]
        path = f"data/tilesets/{bank_name}_metatiles.bin"
        bank = after["runtime_files"][path]
        parent_layout = before["runtime_files"][path]
        equal(bank[:len(parent_layout)], parent_layout, bank_name + " original metatiles")
        deck_ids = {}
        for recipe in pier["metatiles"]:
            index = next_id.get(bank_name, len(parent_layout) // 16)
            equal(list(bank[index*16:index*16+16]), recipe["tiles"], recipe["name"] + " timbers")
            equal(b["tilesets"][bank_name]["collision"][f"{index:02x}"], recipe["collision"],
                  recipe["name"] + " deck and water")
            deck_ids[recipe["name"]] = index
            next_id[bank_name] = index + 1
        changes = [{"at": c["at"], "from": c["from"], "to": deck_ids[c["deck"]]}
                   for c in pier["placements"]]
        layouts.append({"map": pier["map"], "tileset": bank_name, "placements": changes + pier["shore"]})
    moves = copy.deepcopy(a["field_moves"])
    for name in ("cut", "whirlpool"):
        rules = moves[name]["replacements"]
        if "kanto" in rules:
            rules[alias] = rules["kanto"]
    equal(moves, b["field_moves"], "native field moves including Surf and the lake's inherited Cut rules")
    equal(b["tilesets"][alias], a["tilesets"]["kanto"], "lake native terrain and palette")
    attrs = copy.deepcopy(a["map_attributes"])
    attrs["MnR15C00R00"]["tileset_name"] = alias
    equal(attrs, b["map_attributes"], "map attribute mirrors")
    for t, count in next_id.items():
        equal(len(after["runtime_files"][f"data/tilesets/{t}_metatiles.bin"]), count * 16, t + " art extent")
    count = 0
    before_art, after_art = SourceArt(before), SourceArt(after)
    grass_count = 0
    checked_art = set()
    for name in expected - {r["map"] for r in layouts}:
        layouts.append({"map": name, "tileset": a["maps"][name]["attributes"]["tileset_name"], "placements": []})
    for r in layouts:
        name, tileset = r["map"], r["tileset"]
        old, new = a["maps"][name], b["maps"][name]
        blocks = list(old["blocks"])
        for cell in r["placements"]:
            x, y = cell["at"]
            i = y * old["attributes"]["width"] + x
            equal(blocks[i], cell["from"], name + " preimage")
            blocks[i] = cell["to"] if name in city_names else ids.get((tileset, cell["to"]), cell["to"])
        count += len(r["placements"])
        equal(blocks, new["blocks"], name + " installed blocks")
        equal(bytes(blocks), base64.b64decode(b["map_blocks"][old["attributes"]["blocks_label"]]), name + " packed mirror")
        local = {**old, "blocks": blocks, "attributes": attrs[name], "scripts": dict(old["scripts"]),
                 "script_text_bodies": dict(old["script_text_bodies"])}
        for (room, label), program in revised_text.items():
            if room == name:
                local["scripts"][label] = program
                local["script_text_bodies"][label] = {
                    "label": label, "commands": [{**cmd, "command_index": i} for i, cmd in enumerate(program)]}
        equal(local, new, name + " events, actors and scripts")
        new_bank = new["attributes"]["tileset_name"]
        for original, revised in zip(old["blocks"], new["blocks"]):
            kinds = a["tilesets"][tileset]["collision"][f"{original:02x}"]
            revised_kinds = b["tilesets"][new_bank]["collision"][f"{revised:02x}"]
            for q, (k, j) in enumerate(zip(kinds, revised_kinds)):
                equal(k == "TALL_GRASS", j == "TALL_GRASS", name + " grass moved")
                if k != "TALL_GRASS":
                    continue
                grass_count += 1
                key = tileset, new_bank, original, revised, q
                if key in checked_art:
                    continue
                checked_art.add(key)
                x, y = (q % 2)*16, (q // 2)*16
                area = x, y, x+16, y+16
                for light in ("morn", "day", "nite"):
                    equal(before_art.blocks(tileset, light)[original].crop(area).tobytes(),
                          after_art.blocks(new_bank, light)[revised].crop(area).tobytes(),
                          name + " grass artwork at " + light)
    print(f"{count} metatiles across {len(expected)} of {len(a['maps'])} maps; {grass_count} original encounter squares retain their art and positions. {len(dialogue)} dock texts match their sources. Actors, Pokémon, moves and audio match parent.")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("parent", type=Path)
    parser.add_argument("candidate", type=Path)
    parser.add_argument("--output", type=Path)
    args = parser.parse_args()
    before, after = load_pack(args.parent), load_pack(args.candidate)
    review(before, after)
    if args.output:
        args.output.mkdir(parents=True, exist_ok=True)
        for pack, suffix in [(before, "before"), (after, "after")]:
            art = SourceArt(pack)
            routes = {int(row["map"][3:5]) for name in ROUTE_SOURCES for row in read(name + ".json")}
            for route in sorted(routes):
                whole(pack, art, route).save(args.output / f"route-{route:02}-{suffix}.png")


if __name__ == "__main__":
    main()
