#!/usr/bin/env python3
"""Audit installed cave terrain, source art, and the native progression graph."""

import argparse
from base64 import b64decode
from io import BytesIO
import json
from pathlib import Path
import sys

from PIL import Image

SOURCE = Path(__file__).resolve().parent
REPLACEMENT = SOURCE.with_name("crystal-replacement-v02")
sys.path.insert(0, str(SOURCE.parents[1] / "tools"))
sys.path.insert(0, str(REPLACEMENT))

from render_reference import load_pack  # noqa: E402
import check_north  # noqa: E402
import check_south_detail  # noqa: E402
import review_ice  # noqa: E402
import review_mine_flow  # noqa: E402
import review_mines  # noqa: E402

GROUPS = ("south", "north", "mines", "ice")


def read(path):
    return json.loads(path.read_text())


def layouts():
    groups = {}
    for group in GROUPS:
        prior = {layout["map"]: layout for layout in read(REPLACEMENT / f"{group}.json")}
        revised = read(SOURCE / f"{group}.json")
        assert set(prior) == {room["map"] for room in revised}
        groups[group] = [dict(prior[room["map"]], rows=room["rows"]) for room in revised]
    return groups


def audit_art(parent, installed, *, source=SOURCE, group_order=("mines", "south", "north", "ice")):
    authored = {}
    for group in group_order:
        for file in read(source / f"{group}-art.json"):
            bank = authored.setdefault(file["tileset"], {"metatiles": [], "glyphs": []})
            bank["metatiles"].extend(file["metatiles"])
            bank["glyphs"].extend(file["glyphs"])
    old_tilesets, new_tilesets = [
        pack["data"]["tilesets"] for pack in (parent, installed)
    ]
    before, after = [pack["runtime_files"] for pack in (parent, installed)]
    allowed_files = set()
    for tileset, bank in authored.items():
        art = f"gfx/tilesets/{tileset}"
        definition = f"data/tilesets/{tileset}"
        layout = definition + "_metatiles.bin"
        collision = definition + ".json"
        allowed_files.update((layout, collision))

        expected = bytes(before[layout])
        permissions = dict(old_tilesets[tileset]["collision"])
        for block in bank["metatiles"]:
            assert len(expected) == block["id"] * 16
            expected += bytes(block["glyphs"])
            permissions[f"{block['id']:02x}"] = block["collision"]
        assert expected == bytes(after[layout]), tileset
        assert json.loads(bytes(after[collision])) == permissions, tileset

        palettes = old_tilesets[tileset]["palette_map"].copy()
        if bank["glyphs"]:
            encoded, png = art + ".2bpp", art + ".png"
            palette_json = definition + "_palette_map.json"
            allowed_files.update((encoded, png, palette_json))
            ink = bytearray(before[encoded])
            new_slots = {}
            for glyph in bank["glyphs"]:
                slot = glyph["id"]
                new_slots[slot] = bytes([255, 170, 85, 0][int(value)]
                                        for row in glyph["rows"] for value in row)
                palettes[slot] = glyph["palette"]
                for y, row in enumerate(glyph["rows"]):
                    for plane in range(2):
                        ink[(slot * 8 + y) * 2 + plane] = sum(
                            ((int(value) >> plane) & 1) << (7 - x)
                            for x, value in enumerate(row)
                        )
            assert bytes(after[encoded]) == ink, tileset
            assert json.loads(bytes(after[palette_json])) == palettes, tileset
            with Image.open(BytesIO(bytes(before[png]))) as original, Image.open(
                BytesIO(bytes(after[png]))
            ) as actual:
                assert original.size == actual.size, tileset
                original, actual = original.convert("L"), actual.convert("L")
                for slot in range((actual.width // 8) * (actual.height // 8)):
                    x, y = slot % (actual.width // 8) * 8, slot // (actual.width // 8) * 8
                    extent = (x, y, x + 8, y + 8)
                    expected = (new_slots[slot] if slot in new_slots else
                                original.crop(extent).tobytes())
                    assert actual.crop(extent).tobytes() == expected, (tileset, slot, "PNG glyph")
        assert new_tilesets[tileset] == dict(
            old_tilesets[tileset], palette_map=palettes, collision=permissions
        ), tileset
    assert new_tilesets.keys() == old_tilesets.keys()
    assert {k for k in old_tilesets if old_tilesets[k] != new_tilesets[k]} == authored.keys()
    assert before.keys() <= after.keys()
    assert {k for k in after if before.get(k) != after[k]} <= allowed_files


def audit_maps(parent, installed, groups):
    old, new = [pack["data"] for pack in (parent, installed)]
    for key in old.keys() - {"maps", "tilesets", "map_blocks"}:
        assert old[key] == new[key], key
    assert old["maps"].keys() == new["maps"].keys()
    assert old["map_blocks"].keys() == new["map_blocks"].keys()
    assert old.keys() == new.keys()
    assert installed["audio_manifest"] == parent["audio_manifest"]
    assert installed["compiled_audio"] == parent["compiled_audio"]
    assert installed["audio_compression"] == parent["audio_compression"]
    report = dict(parent["report"], manifests=parent["report"]["manifests"] + ["northstar-cave-detail-v4"])
    assert installed["report"] == report
    authored = set()
    block_labels = set()
    changed_blocks = 0
    for group in groups.values():
        for layout in group:
            name = layout["map"]
            authored.add(name)
            room = new["maps"][name]
            previous = old["maps"][name]
            terrain = [int(b, 16) for row in layout["rows"] for b in row.split()]
            assert room == dict(previous, blocks=terrain), name
            label = room["attributes"]["blocks_label"]
            assert b64decode(new["map_blocks"][label], validate=True) == bytes(terrain)
            block_labels.add(label)
            assert len(previous["blocks"]) == len(terrain)
            changed_blocks += sum(a != b for a, b in zip(previous["blocks"], terrain))
    assert len(authored) == 19
    assert {k for k in new["maps"] if old["maps"][k] != new["maps"][k]} == authored
    assert {k for k in new["map_blocks"] if old["map_blocks"][k] != new["map_blocks"][k]} == block_labels
    print(f"{len(authored)} authored interiors, {changed_blocks} changed metatiles")


def verify(fair, parent, installed):
    groups = layouts()
    audit_art(parent, installed)
    audit_maps(parent, installed, groups)
    check_south_detail.review(
        parent, installed, groups["south"], read(REPLACEMENT / "south.json")
    )
    for layout in groups["north"]:
        print(check_north.check_room(installed, fair, layout))
    print(review_mines.audit(fair, installed, groups["mines"]))
    print(review_mine_flow.audit(parent, installed))
    for layout in groups["ice"]:
        review_ice.inspect(installed, installed["data"]["maps"][layout["map"]], layout)


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ("fair", "parent", "installed"):
        parser.add_argument(name, type=Path)
    args = parser.parse_args()
    verify(load_pack(args.fair), load_pack(args.parent), load_pack(args.installed))
