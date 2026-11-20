#!/usr/bin/env python3
"""Check the Winona research award, its pack mirrors and the untouched world."""
import argparse
import base64
import io
import json
from pathlib import Path

from PIL import Image
import sys

REGION = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(REGION / "tools"))
from audit_story_flow import instructions, native_pages
from render_reference import SourceArt, load_pack

STATION = "MnWinonaFieldStation"
HINT = "MnRewardFinderBank"
FULL = "MnRewardAideFull"
CLAIM = "MnRewardDexClaim1"
TEXT = {HINT + "Text", FULL + "Text", CLAIM + "Text"}


def changed(a, b):
    return {k for k in a.keys() | b.keys() if k not in a or k not in b or a[k] != b[k]}


def riverbank(parent, child):
    key = "johto_glacier_rail"
    atlas = f"data/tilesets/{key}_metatiles.bin"
    collision = f"data/tilesets/{key}.json"
    a, b = parent["data"], child["data"]
    old, new = a["maps"]["MnWinona"], b["maps"]["MnWinona"]
    assert changed(old, new) == {"blocks"}
    label = old["attributes"]["blocks_label"]
    assert changed(a["map_blocks"], b["map_blocks"]) == {label}
    assert base64.b64decode(b["map_blocks"][label]) == bytes(new["blocks"])
    assert changed(a["tilesets"], b["tilesets"]) == {key}
    before, after = a["tilesets"][key], b["tilesets"][key]
    assert changed(before, after) == {"collision", "palette_map"}
    first, last = parent["runtime_files"], child["runtime_files"]
    sheet = f"gfx/tilesets/{key}.2bpp"
    png = f"gfx/tilesets/{key}.png"
    palettes = f"data/tilesets/{key}_palette_map.json"
    assert changed(first, last) == {atlas, collision, sheet, png, palettes}
    colors = after["palette_map"]
    assert json.loads(bytes(last[palettes])) == colors
    original_sheet, new_sheet = bytes(first[sheet]), bytes(last[sheet])
    assert len(original_sheet) == len(new_sheet) == 192 * 16
    def address(glyph, palette):
        if glyph < len(palette) and palette[glyph] & 8 and (glyph & 127) + 96 < 192:
            return (glyph & 127) + 96
        return glyph - 128 if glyph >= 192 else glyph
    assert {i for i, (x, y) in enumerate(zip(before["palette_map"], colors, strict=True))
            if x != y} <= set(range(0xc4, 0xc9))
    assert {i for i in range(192) if original_sheet[16*i:16*i+16] != new_sheet[16*i:16*i+16]} == set(range(0xa4, 0xa9))
    for glyph in set(first[atlas]) | {0x03, 0x14}:
        assert before["palette_map"][glyph] == colors[glyph]
        addr = 16 * address(glyph, colors)
        assert original_sheet[addr:addr+16] == new_sheet[addr:addr+16]
    pix = Image.open(io.BytesIO(bytes(last[png]))).convert("L")
    assert pix.size == (128, 96)
    for y in range(pix.height):
        for x in range(pix.width):
            row = 16 * ((y // 8) * 16 + x // 8) + 2 * (y % 8)
            bit = 7 - x % 8
            ink = ((new_sheet[row] >> bit) & 1) | (((new_sheet[row+1] >> bit) & 1) << 1)
            assert pix.getpixel((x, y)) == 255 - 85 * ink
    older, extended = bytes(first[atlas]), bytes(last[atlas])
    assert extended.startswith(older) and len(older) < len(extended) <= 4096
    assert len(extended) % 16 == 0
    original_art, revised_art = SourceArt(parent), SourceArt(child)
    for period in ("morn", "day", "nite"):
        old_pixels, new_pixels = (art.blocks(key, period) for art in (original_art, revised_art))
        assert all(tile.tobytes() == new_pixels[i].tobytes() for i, tile in enumerate(old_pixels))
    assert json.loads(bytes(last[collision])) == after["collision"]
    assert after["collision"] | before["collision"] == after["collision"]
    width = old["attributes"]["width"]
    placements = set()
    for i, (prior, current) in enumerate(zip(old["blocks"], new["blocks"], strict=True)):
        assert before["collision"][f"{prior:02x}"] == after["collision"][f"{current:02x}"]
        if prior != current:
            placements.add((i % width, i // width))
    assert all(12 <= x <= 14 and 2 <= y <= 8 for x, y in placements), placements
    assert len(placements) == 15 and len(extended) - len(older) == 7 * 16
    assert (12, 4) in placements and (13, 4) in placements
    # The west bank under the fir is exactly where a player's dry foot becomes a keel.
    def glyph(x, y):
        placed = new["blocks"][y // 4 * width + x // 4]
        return extended[placed * 16 + y % 4 * 4 + x % 4]
    for y in range(16, 24):
        assert glyph(51, y) != 0x14 and all(glyph(x, y) == 0x14 for x in range(52, 56))
    return placements


def compare(parent, child):
    assert changed(parent, child) == {"data", "report", "identity", "runtime_files"}
    assert changed(parent["report"], child["report"]) == {"manifests"}
    assert child["report"]["manifests"] == parent["report"]["manifests"] + ["northstar-finder-lesson-v1"]
    assert changed(parent["identity"], child["identity"]) == {"runtime_modpack_id", "content_hash"}
    a, b = parent["data"], child["data"]
    assert changed(a, b) == {"maps", "map_scripts", "map_blocks", "tilesets"}
    assert changed(a["maps"], b["maps"]) == {STATION, "MnWinona"}
    placements = riverbank(parent, child)
    old, new = a["maps"][STATION], b["maps"][STATION]
    claims = {f"MnRewardDexClaim{i}" for i in range(5)}
    programs = claims | {HINT, FULL}
    assert changed(old["scripts"], new["scripts"]) == changed(a["map_scripts"], b["map_scripts"]) == programs | TEXT
    assert changed(old["script_text_bodies"], new["script_text_bodies"]) == TEXT
    tables = {"script_text_commands", "script_control_commands", "script_flag_commands"}
    assert changed(old, new) == tables | {"scripts", "script_text_bodies"}
    for key in tables:
        # No typed rows for an unrelated scene can disappear or change.
        keep = lambda room: [row for row in room[key] if row["source_script"] not in programs]
        assert keep(old) == keep(new), key
    for label in claims:
        before, after = instructions(old, label), instructions(new, label)
        assert before[4] == ("iffalse", "MnRewardStationFull")
        expected = list(before)
        expected[4] = ("iffalse", FULL)
        if label == CLAIM:
            assert before[3] == ("giveitem", "ITEMFINDER", "1")
            assert before[5] == ("setevent", "EVENT_MN_DEX_REWARD_1")
            expected[10:10] = [("checkevent", "EVENT_MN_HIDDEN_WINONA"), ("iffalse", HINT)]
        assert after == expected, label
    for label in HINT, FULL:
        assert instructions(new, label) == [
            ("writetext", label + "Text"), ("waitbutton",), ("closetext",), ("end",)]
    assert ("iffalse", "MnRewardStationFull") in instructions(new, "MnRewardTamarack")
    pages = json.loads((REGION / "story.json").read_text())["dialogue_overhaul"]["texts"][STATION]
    for label in TEXT:
        assert native_pages(new["script_text_bodies"][label]["commands"]) == pages[label]
        assert b["map_scripts"][label] == new["scripts"][label]
    assert instructions(b["maps"]["MnWinona"], "MN_HIDDEN_WINONA") == [("hiddenitem", "POKE_BALL", "EVENT_MN_HIDDEN_WINONA")]
    print(f"{len(b['maps'])} maps checked. Winona award, advice and riverbank verified ({len(placements)} placed metatiles).")


if __name__ == "__main__":
    cli = argparse.ArgumentParser(description=__doc__)
    cli.add_argument("parent", type=Path)
    cli.add_argument("child", type=Path)
    args = cli.parse_args()
    compare(load_pack(args.parent), load_pack(args.child))
