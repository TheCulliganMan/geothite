#!/usr/bin/env python3
"""Independently compare the installed shoreline and cliff revision against its exact parent."""
import argparse
import base64
import io
import json
import sys
from collections import Counter, defaultdict
from pathlib import Path

from PIL import Image

HERE = Path(__file__).resolve().parent
REGION = HERE.parents[1]
sys.path.insert(0, str(REGION / "tools"))
from render_reference import SourceArt, load_pack

SECTIONS = ("north", "south", "places", "waterfronts")
TIMBER = {"mn_duluth_expanded": (0xc0, 0xce), "mn_boundary_safari": (0xc8, 0xd0)}
MANIFEST = "northstar-cliff-faces-v1"


def read(name):
    return json.loads((HERE / (name + ".json")).read_text())


def require(condition, message):
    if not condition:
        raise ValueError(message)


def equal(actual, expected, message):
    require(actual == expected, message)


def exact_except(old, new, fields, message):
    equal(old.keys(), new.keys(), message + " keys")
    for key in old.keys() - set(fields):
        equal(new[key], old[key], f"{message} {key}")


def program(pages):
    cmds = []
    for i, page in enumerate(pages):
        for j, line in enumerate(page):
            cmds.append({"command": "line" if j else ("para" if i else "text"),
                         "args": [json.dumps(line, ensure_ascii=False)]})
    return cmds + [{"command": "done", "args": []}]


def body(label, commands):
    return {"label": label, "commands": [dict(cmd, command_index=n) for n, cmd in enumerate(commands)]}


def check_text(old, new):
    changes = defaultdict(dict)
    scripts = dict(old["map_scripts"])
    story = json.loads((REGION / "story.json").read_text())
    for row in read("dock-dialogue"):
        room, label, source = row["map"], row["label"], row["source"]
        if source == "dialogue_overhaul":
            pages = story["dialogue_overhaul"]["texts"][room][label]
        elif source == "story_flow":
            pages = story["story_flow_revision"]["dialogue"][room][label]
        else:
            route = json.loads((REGION / "routes" / source).read_text())
            lines = next(sign["lines"] for sign in route["signs"] if sign["id"] + "Text" == label)
            pages = [lines[n:n+2] for n in range(0, len(lines), 2)]
        original, revised = program(row["previous"]), program(pages)
        require(label not in changes[room] and original != revised, "duplicate or unchanged text " + label)
        equal(scripts[label], original, label + " parent global text")
        equal(old["maps"][room]["scripts"][label], original, label + " parent local text")
        equal(old["maps"][room]["script_text_bodies"][label], body(label, original), label + " parent cache")
        changes[room][label] = scripts[label] = revised
    equal(scripts, new["map_scripts"], "global script/text mirrors")
    return changes


def physical(glyph, color, count):
    return count//2 + (glyph & 127) if color & 8 else glyph


def check_timber(before, after, bank, paths):
    a, b = before["runtime_files"], after["runtime_files"]
    colors = before["data"]["tilesets"]
    src = a["gfx/tilesets/kanto.2bpp"]
    binary = f"gfx/tilesets/{bank}.2bpp"
    png = f"gfx/tilesets/{bank}.png"
    palfile = f"data/tilesets/{bank}_palette_map.json"
    bits = bytearray(a[binary])
    count = len(bits)//16
    palette = list(colors[bank]["palette_map"])
    layout = a[f"data/tilesets/{bank}_metatiles.bin"]
    used = {physical(g, palette[g], count) for g in layout}
    start, ends = TIMBER[bank]
    imports = zip([*range(0x60, 0x68), 0x6e, 0x6f], [*range(start, start+8), ends, ends+1])
    for source, target in imports:
        color = colors["kanto"]["palette_map"][source]
        pos = physical(source, color, len(src)//16)*16
        dest = physical(target, 8 | (color & 7), count)
        require(target not in layout and dest < count and dest not in used, bank + " timber overwrites old art")
        used.add(dest)
        bits[16*dest:16*dest+16] = src[pos:pos+16]
        palette[target] = 8 | (color & 7)
    equal(bytes(bits), bytes(b[binary]), bank + " original graphics and Kanto timber")
    equal(palette, after["data"]["tilesets"][bank]["palette_map"], bank + " native palette")
    equal(palette, json.loads(bytes(b[palfile])), bank + " palette mirror")
    atlas = Image.open(io.BytesIO(bytes(b[png]))).convert("L")
    equal(atlas.size, (128, count//16*8), bank + " atlas shape")
    levels = (255, 170, 85, 0)
    pixels = atlas.load()
    for i in range(count):
        for dy in range(8):
            lo, hi = bits[16*i+dy*2:16*i+dy*2+2]
            for dx in range(8):
                shift = 7-dx
                color = ((lo >> shift) & 1) | (((hi >> shift) & 1) << 1)
                equal(pixels[i%16*8+dx, i//16*8+dy], levels[color], bank + " PNG/2bpp mirror")
    paths.update((binary, png, palfile))


def check_art(before, after):
    a, b = before["runtime_files"], after["runtime_files"]
    old, new = before["data"]["tilesets"], after["data"]["tilesets"]
    equal(a.keys(), b.keys(), "runtime file inventory")
    equal(old.keys(), new.keys(), "native tileset inventory")
    ids, changes, allowed = {}, defaultdict(list), set()
    for section in SECTIONS:
        file = HERE / f"{section}-art.json"
        if file.is_file():
            for recipe in json.loads(file.read_text()):
                changes[recipe["tileset"]].append(recipe)
    for bank in old:
        fields = {"palette_map"} if bank in TIMBER else set()
        if bank in changes:
            fields.add("collision")
        exact_except(old[bank], new[bank], fields, bank + " tileset")
        if bank in TIMBER:
            check_timber(before, after, bank, allowed)
        if bank not in changes:
            continue
        meta = f"data/tilesets/{bank}_metatiles.bin"
        jsonfile = f"data/tilesets/{bank}.json"
        layout = bytearray(a[meta])
        collisions = dict(old[bank]["collision"])
        require(len(layout) % 16 == 0, bank + " broken native metatiles")
        for recipe in changes[bank]:
            donor = collisions[f"{recipe['collision_from']:02x}"]
            native = len(layout)//16
            key = bank, recipe["id"]
            require(key not in ids and f"{native:02x}" not in collisions, bank + " duplicate artwork")
            ids[key] = native
            layout.extend(recipe["tiles"])
            collisions[f"{native:02x}"] = recipe.get("collision", donor)
        equal(bytes(layout), bytes(b[meta]), bank + " original and appended native metatiles")
        equal(collisions, new[bank]["collision"], bank + " original and appended collision")
        equal(collisions, json.loads(bytes(b[jsonfile])), bank + " collision mirror")
        allowed.update((meta, jsonfile))
    for path in a.keys() - allowed:
        equal(a[path], b[path], "unexpected runtime file change: " + path)
    return ids, set(changes) | TIMBER.keys()


def read_placements(old):
    plans = defaultdict(dict)
    groups = [row for section in SECTIONS for row in read(section)]
    supplement = []
    for family in ("field", "mothlight"):
        rows = read("south-" + family)
        matches = all(old[r["map"]]["blocks"][c["at"][1]*old[r["map"]]["attributes"]["width"] + c["at"][0]]
                      == c["from"] for r in rows for c in r["placements"])
        if matches:
            supplement.append(rows)
    require(len(supplement) == 1, "expected exactly one lineage-specific southern preimage")
    for r in groups + supplement[0]:
        name = r["map"]
        equal(old[name]["attributes"]["tileset_name"], r["tileset"], name + " parent tileset")
        for cell in r["placements"]:
            pos = tuple(cell["at"])
            require(pos not in plans[name], f"duplicate {name} edit at {pos}")
            plans[name][pos] = cell
    return plans


def check_grass(old, new, before, after, modified_banks):
    art_a, art_b = SourceArt(before), SourceArt(after)
    total = 0
    quadrants = set()
    for name, m in old["maps"].items():
        if not name.startswith(("Mn", "MN_")):
            continue
        bank = m["attributes"]["tileset_name"]
        kinds = old["tilesets"][bank]["collision"]
        revised = new["tilesets"][bank]["collision"]
        for (left, right), n in Counter(zip(m["blocks"], new["maps"][name]["blocks"])).items():
            left_key, right_key = f"{left:02x}", f"{right:02x}"
            for q, (a, b) in enumerate(zip(kinds[left_key], revised[right_key])):
                equal(a == "TALL_GRASS", b == "TALL_GRASS", f"{name} encounter quadrant")
                if a == "TALL_GRASS":
                    total += n
                    if bank in modified_banks or left != right:
                        quadrants.add((bank, left, right, q))
    for bank, left, right, q in sorted(quadrants):
        x, y = 16*(q%2), 16*(q//2)
        bounds = x, y, x+16, y+16
        for light in ("morn", "day", "nite"):
            a = art_a.blocks(bank, light)[left].crop(bounds).tobytes()
            b = art_b.blocks(bank, light)[right].crop(bounds).tobytes()
            equal(a, b, f"{bank} {left:02x}/{right:02x} grass pixels at {light}")
    return total


def verify(before, after):
    old, new = before["data"], after["data"]
    exact_except(before, after, {"data", "identity", "runtime_files", "report"}, "pack, including audio")
    exact_except(before["report"], after["report"], {"manifests"}, "pack report")
    equal(after["report"]["manifests"], before["report"]["manifests"] + [MANIFEST], "manifest lineage")
    require(before["identity"] != after["identity"], "unchanged pack identity")
    exact_except(old, new, {"maps", "map_blocks", "tilesets", "map_scripts"}, "game catalog")
    ids, modified_banks = check_art(before, after)
    texts = check_text(old, new)
    plans = read_placements(old["maps"])
    revised = plans.keys() | texts.keys()
    equal(old["maps"].keys(), new["maps"].keys(), "map inventory")
    packed = dict(old["map_blocks"])
    for name, m in old["maps"].items():
        child = new["maps"][name]
        if name not in revised:
            equal(child, m, name + " unrelated map")
            continue
        exact_except(m, child, {"blocks", "scripts", "script_text_bodies"}, name + " map events, actors and warps")
        attrs = m["attributes"]
        blocks = list(m["blocks"])
        for (x, y), cell in plans.get(name, {}).items():
            require(0 <= x < attrs["width"] and 0 <= y < attrs["height"], name + " coordinates")
            i = y*attrs["width"] + x
            equal(cell["from"], blocks[i], f"{name} native preimage at {(x,y)}")
            dest = ids.get((attrs["tileset_name"], cell["to"]), cell["to"])
            require(blocks[i] != dest, f"{name} redundant edit at {(x,y)}")
            blocks[i] = dest
        equal(blocks, child["blocks"], name + " only the authored placements")
        label = attrs["blocks_label"]
        equal(base64.b64decode(old["map_blocks"][label]), bytes(m["blocks"]), name + " parent block mirror")
        packed[label] = base64.b64encode(bytes(blocks)).decode("ascii")
        scripts, cached = dict(m["scripts"]), dict(m["script_text_bodies"])
        for label, commands in texts.get(name, {}).items():
            scripts[label] = commands
            cached[label] = body(label, commands)
        equal(scripts, child["scripts"], name + " native script programs")
        equal(cached, child["script_text_bodies"], name + " cached native text")
    equal(packed, new["map_blocks"], "all installed block mirrors")
    deltas = {"up": (0, -1), "down": (0, 1), "left": (-1, 0), "right": (1, 0)}
    for section in SECTIONS:
        for shore in read(section + "-surf"):
            x, y = shore["from"]
            dx, dy = deltas[shore["direction"]]
            actors = {(o["x"], o["y"]) for o in new["maps"][shore["map"]]["objects"]}
            require(actors.isdisjoint({(x, y), (x + dx, y + dy)}),
                    f"{shore['map']} shore position or landing occupied at {(x, y)}")
    grass = check_grass(old, new, before, after, modified_banks)
    print(f"{sum(map(len, plans.values()))} authored placements across {len(plans)} exteriors; "
          f"{sum(map(len, texts.values()))} text programs match all three mirrors. "
          f"{grass} original encounter quadrants retain their position and native morning/day/night artwork. "
          "Original maps, events, actors, warps, encounters, moves, animations, palettes and audio validated.")


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("parent", type=Path)
    parser.add_argument("candidate", type=Path)
    args = parser.parse_args()
    verify(load_pack(args.parent), load_pack(args.candidate))
