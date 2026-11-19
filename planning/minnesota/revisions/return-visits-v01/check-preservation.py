#!/usr/bin/env python3
"""Compare both full game archives, including the pixels of every original house block."""
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

GYM = "MnDuluthGym"
BOOK = "MnDuluthUnionHall2F"
FLAG = "EVENT_MN_INTERIOR_UNION_NAMES_ENTERED"
MARKER = "northstar-town-returns-v1"
FILES = {"data/tilesets/house_metatiles.bin", "gfx/tilesets/house.png", "gfx/tilesets/house.2bpp"}


def changes(a, b):
    return (a.keys() ^ b.keys()) | {k for k in a.keys() & b.keys() if a[k] != b[k]}


def scope(a, b, expected, label):
    actual = changes(a, b)
    require(actual == expected, f"{label} changed {sorted(actual)}; expected {sorted(expected)}")


def book(a, b):
    bank, newer = (p["data"]["tilesets"]["house"] for p in (a, b))
    scope(bank, newer, {"collision", "palette_map"}, "house tileset")
    scope(bank["collision"], newer["collision"], {"40"}, "house collision")
    require(newer["collision"]["40"] == bank["collision"]["21"], "book changed furniture collision")
    old, new = (p["runtime_files"] for p in (a, b))
    atlas, sheet = "data/tilesets/house_metatiles.bin", "gfx/tilesets/house.2bpp"
    arrays = bytes(new[atlas])
    require(len(arrays) == 65 * 16 and arrays[:-16] == bytes(old[atlas]), "house arrangements")
    require(all(n == bank["palette_map"][i] for i, n in enumerate(newer["palette_map"])
                if not 0xa0 <= i <= 0xa3), "unrelated palette changed")
    orig, after = bytes(old[sheet]), bytes(new[sheet])
    require(len(orig) == len(after) == 3072 and orig[:2048] == after[:2048]
            and orig[2112:] == after[2112:], "unrelated house glyph modified")
    # Compare all 64 old block images and all the unaffected pixels of the table itself.
    before, after = SourceArt(a), SourceArt(b)
    rows = json.loads((HERE / "crew-book.json").read_text())
    for clock in ("morn", "day", "nite"):
        first, second = before.blocks("house", clock), after.blocks("house", clock)
        require(len(first) == 64 and len(second) == 65 and
                all(x.tobytes() == y.tobytes() for x, y in zip(first, second)), f"original house art at {clock}")
        old_table, ledger = first[0x21], second[0x40]
        for y in range(32):
            for x in range(32):
                if x < 16 or y < 16:
                    require(old_table.getpixel((x, y)) == ledger.getpixel((x, y)), "outside the book square")
        colors = {}
        for y, row in enumerate(rows):
            for x, ink in enumerate(row):
                pixel = ledger.getpixel((16 + x, 16 + y))
                require(colors.setdefault(ink, pixel) == pixel, "native book drawing differs")
        require(len(set(colors.values())) == 4, "book requires four distinct native shades")


def check(old_path, new_path):
    parent_hash = hashlib.sha256(old_path.read_bytes()).hexdigest()
    receipt = next((r for r in (json.loads(p.read_text()) for p in HERE.glob("*-build.json"))
                    if r["parent_pack_sha256"] == parent_hash), None)
    require(receipt and receipt["expected_pack_sha256"] == hashlib.sha256(new_path.read_bytes()).hexdigest(),
            "expected the paired parent and child from a published build receipt")
    old, new = load_pack(old_path), load_pack(new_path)
    scope(old, new, {"data", "runtime_files", "identity", "report"}, "archive")
    scope(old["report"], new["report"], {"manifests"}, "build report")
    require(new["report"]["manifests"] == old["report"]["manifests"] + [MARKER], "parent lineage")
    scope(old["identity"], new["identity"], {"runtime_modpack_id", "content_hash"}, "identity")
    require(new["identity"]["runtime_modpack_id"] == old["identity"]["runtime_modpack_id"] + "+" + MARKER,
            "runtime lineage")
    scope(old["runtime_files"], new["runtime_files"], FILES, "native files")
    a, b = old["data"], new["data"]
    scope(a, b, {"tilesets", "maps", "map_scripts", "map_blocks", "story_event_script_constants"}, "game data")
    scope(a["tilesets"], b["tilesets"], {"house"}, "tilesets")
    book(old, new)
    cg, ng = (d["story_event_script_constants"] for d in (a, b))
    scope(cg, ng, {"global"}, "story constants")
    scope(cg["global"], ng["global"], {FLAG}, "story flags")
    require(ng["global"][FLAG] > max(v for k, v in cg["global"].items() if k.startswith("EVENT_")),
            "existing event address reused")
    allowed = {BOOK, GYM, "MnStateFairgrounds", "MnDuluthUnionHall1F", "MnDuluthOreWarehouse2F", "MnRochesterPokecenter1F"}
    if "northstar-field-review-v2" not in old["report"]["manifests"]:
        allowed |= {"MnStCloud", "MnDuluth"}
    scope(a["maps"], b["maps"], allowed, "rooms")
    global_labels = set()
    for name in sorted(allowed):
        p, q = (d["maps"][name] for d in (a, b))
        static = {k for k in p.keys() | q.keys() if not k.startswith("script_") and k != "scripts"}
        for key in static - ({"blocks"} if name == BOOK else set()):
            require(p[key] == q[key], f"{name}: changed {key}")
        labels = changes(p["scripts"], q["scripts"])
        for label in labels:
            require(q["scripts"].get(label) == b["map_scripts"].get(label), f"local/global {name}/{label}")
        global_labels |= labels
    scope(a["map_scripts"], b["map_scripts"], global_labels, "global scripts")
    old_book, new_book = (d["maps"][BOOK] for d in (a, b))
    at = 3 * old_book["attributes"]["width"] + 4
    blocks = old_book["blocks"].copy()
    require(blocks[at] == 0x21, "original table")
    blocks[at] = 0x40
    require(new_book["blocks"] == blocks, "other Union Hall terrain changed")
    key = old_book["attributes"]["blocks_label"]
    scope(a["map_blocks"], b["map_blocks"], {key}, "split map terrain")
    require(base64.b64decode(b["map_blocks"][key]) == bytes(blocks), "room / split book block mismatch")
    print(f"{len(b['maps'])} maps, {len(b['trainers']['trainers'])} trainers, {len(b['audio'])} audio assets. "
          f"{len(allowed)} rooms, {len(global_labels)} script labels, one table and one new event. "
          "All previous event addresses and all 64 original house blocks at three times of day match.")


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("parent", type=Path)
    parser.add_argument("child", type=Path)
    opts = parser.parse_args()
    check(opts.parent, opts.child)
