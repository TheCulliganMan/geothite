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
            widths[x], heights[y] = chunks[x, y].size
    image = Image.new("RGB", (sum(widths.values()), sum(heights.values())), "#242629")
    for (x, y), chunk in chunks.items():
        image.paste(chunk, (sum(widths[i] for i in range(x)), sum(heights[i] for i in range(y))))
    return image


def review(before, after):
    layouts = [r for file in ("driftless", "river-bluffs", "iron-range", "iron-descent", "northern-routes")
               for r in read(file + ".json")]
    expected = {r["map"] for r in layouts}
    a, b = before["data"], after["data"]
    equal(before["audio_manifest"], after["audio_manifest"], "audio manifest")
    equal(a.keys(), b.keys(), "game catalogs")
    for k in a.keys() - {"maps", "map_blocks", "tilesets", "field_moves"}:
        equal(a[k], b[k], k)
    equal(a["maps"].keys(), b["maps"].keys(), "map inventory")
    equal({k for k in a["maps"] if a["maps"][k] != b["maps"][k]}, expected, "changed maps")

    recipes = []
    for ink in read("iron-ink.json"):
        for bank in ink.get("additional_banks", []):
            recipes.append({**ink["design"], **bank})
        recipes.append(ink["design"])
    recipes += [r for file in ("iron-art", "northern-art", "driftless-art", "descent-art") for r in read(file + ".json")]
    ids, next_id = {}, {}
    for recipe in recipes:
        t = recipe["tileset"]
        bank = f"data/tilesets/{t}_metatiles.bin"
        old, new = before["runtime_files"][bank], after["runtime_files"][bank]
        i = next_id.get(t, len(old) // 16)
        equal(old, new[:len(old)], t + " original metatiles")
        if "tiles" in recipe:
            equal(list(new[i*16:(i+1)*16]), recipe["tiles"], t + " metatile recipe")
        equal(a["tilesets"][t]["collision"][f"{recipe['collision_from']:02x}"],
              b["tilesets"][t]["collision"][f"{i:02x}"], t + " collision")
        ids[t, recipe.get("id", recipe.get("block"))] = i
        next_id[t] = i + 1
    moves = copy.deepcopy(a["field_moves"])
    for recipe in recipes:
        if "cut_to" in recipe:
            t = recipe["tileset"]
            native = ids[t, recipe["id"]]
            bare = ids.get((t, recipe["cut_to"]), recipe["cut_to"])
            moves["cut"]["replacements"][t][native] = {"variant": "grass", "replacement_block_id": bare}
    equal(moves, b["field_moves"], "native Cut replacements and unaffected field moves")
    for t, count in next_id.items():
        equal(len(after["runtime_files"][f"data/tilesets/{t}_metatiles.bin"]), count * 16, t + " art extent")
    count = 0
    for r in layouts:
        name, tileset = r["map"], r["tileset"]
        old, new = a["maps"][name], b["maps"][name]
        blocks = list(old["blocks"])
        for cell in r["placements"]:
            x, y = cell["at"]
            i = y * old["attributes"]["width"] + x
            equal(blocks[i], cell["from"], name + " preimage")
            blocks[i] = ids.get((tileset, cell["to"]), cell["to"])
        count += len(r["placements"])
        equal(blocks, new["blocks"], name + " installed blocks")
        equal(bytes(blocks), base64.b64decode(b["map_blocks"][old["attributes"]["blocks_label"]]), name + " packed mirror")
        equal({**old, "blocks": blocks}, new, name + " events, actors and scripts")
    print(f"{count} metatiles across {len(expected)} of {len(a['maps'])} maps; scripts, warps, actors, encounters and audio match parent.")


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
            for route in (1, 5, 9, 10, 12, 13):
                whole(pack, art, route).save(args.output / f"route-{route:02}-{suffix}.png")


if __name__ == "__main__":
    main()
