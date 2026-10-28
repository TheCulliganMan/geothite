#!/usr/bin/env python3
"""Freeze source glyph addresses for the reviewed house study; no pack writes."""
import argparse
import hashlib
import json
from pathlib import Path
import sys

ROOT = Path(__file__).resolve().parents[4]
sys.path.insert(0, str(ROOT / "planning/minnesota/tools"))
from render_reference import SourceArt, load_pack


def sha(value):
    return hashlib.sha256(value).hexdigest()


def canonical(value):
    return json.dumps(value, sort_keys=True, separators=(",", ":"), ensure_ascii=False).encode()


def freeze(source, target):
    assert not target.exists(), "Every publication contract is immutable"
    parent = ROOT / "content-packs/minnesota-city-river-bend-v01.crystalpack"
    parent_sha = sha(parent.read_bytes())
    assert parent_sha == "57eefb675b9cb32ac66a01819f56961392e51678a6f9dd959888fbb9427e4e62"
    pack = load_pack(parent)
    art = SourceArt(pack)
    spec = json.loads(source.read_bytes())
    ts = spec["tileset"]
    files = pack["runtime_files"]
    definitions = pack["data"]["tilesets"]
    raw = bytes(files[f"data/tilesets/{ts}_metatiles.bin"])
    assert len(raw) == 157 * 16
    cache = {}

    def color_key(tileset, block, cell):
        key = tileset, block, cell
        if key not in cache:
            x, y = cell % 4 * 8, cell // 4 * 8
            cache[key] = b"".join(art.blocks(tileset, group)[block].crop(
                (x, y, x + 8, y + 8)).tobytes() for group in ["morn", "day", "nite", "dark"])
        return cache[key]

    existing = {}
    for block in range(157):
        for cell, glyph in enumerate(raw[block * 16:block * 16 + 16]):
            existing.setdefault(color_key(ts, block, cell), glyph)
    new_keys = {}
    copies, blocks, reuse = [], [], []
    for block in spec["new_metatiles"]:
        assert int(block["id"], 16) == 157 + len(blocks)
        glyphs = []
        for donor in block["glyphs"]:
            donor_ts, donor_block = donor["tileset"], int(donor["block"], 16)
            cell = donor["glyph"][1] * 4 + donor["glyph"][0]
            key = color_key(donor_ts, donor_block, cell)
            if key in existing:
                glyph = existing[key]
                reuse.append(dict(target=glyph, source=donor, four_bank_rgb_sha256=sha(key)))
            elif key in new_keys:
                glyph = new_keys[key]
            else:
                glyph = 224 + len(copies)
                assert glyph <= 255 and glyph not in raw
                new_keys[key] = glyph
                donor_layout = bytes(files[f"data/tilesets/{donor_ts}_metatiles.bin"])
                donor_glyph = donor_layout[donor_block * 16 + cell]
                attribute = definitions[donor_ts]["palette_map"][donor_glyph]
                assert attribute < 16
                bitplanes = bytes(files[f"gfx/tilesets/{donor_ts}.2bpp"])
                count = len(bitplanes) // 16
                physical = ((donor_glyph & 127) + count // 2
                            if attribute & 8 else donor_glyph)
                pixels = bitplanes[physical * 16:physical * 16 + 16]
                assert len(pixels) == 16
                assert f"gfx/tilesets/{donor_ts}.pal" not in files
                assert f"data/tilesets/{donor_ts}_foregrounds.json" not in files
                copies.append(dict(source_tileset=donor_ts, source_glyph=donor_glyph,
                                   target=glyph, source_palette=attribute,
                                   pixels_sha256=sha(pixels),
                                   four_bank_rgb_sha256=sha(key)))
            glyphs.append(glyph)
        blocks.append(dict(target=int(block["id"], 16), glyphs=glyphs,
                           collision=block["collision"]))
    assert len(copies) == 27 and len(blocks) == 13
    owned = [f"gfx/tilesets/{ts}.2bpp", f"gfx/tilesets/{ts}.png",
             f"data/tilesets/{ts}_metatiles.bin", f"data/tilesets/{ts}_palette_map.json",
             f"data/tilesets/{ts}.json"]
    optional = f"data/tilesets/{ts}_foregrounds.json"
    if optional in files:
        owned.append(optional)
    donors = sorted({item["source_tileset"] for item in copies})
    inputs = set(owned) | {"gfx/tilesets/bg_tiles.pal"}
    for donor in donors:
        inputs.update([f"gfx/tilesets/{donor}.2bpp", f"data/tilesets/{donor}_metatiles.bin"])
    module = pack["data"]["maps"]["MnMinneapolis"]
    result = dict(schema_version=1, base_pack_sha256=parent_sha,
                  base_identity=pack["identity"]["content_hash"], base_map_count=len(pack["data"]["maps"]),
                  map="MnMinneapolis", tileset=ts, expected_blocks=157,
                  base_module_sha256=sha(canonical(module)),
                  base_definition_sha256=sha(canonical(definitions[ts])),
                  donor_definition_sha256={donor:sha(canonical(definitions[donor])) for donor in donors},
                  source_files_sha256={path:sha(bytes(files[path])) for path in sorted(inputs)},
                  glyph_copies=copies, blocks=blocks,
                  cells=[dict(metatile=item["metatile"], before=int(item["before"],16),
                              after=int(item["after"],16)) for item in spec["map_patches"]])
    target.write_text(json.dumps(result, indent=2) + "\n")
    target.with_suffix(".source-review.json").write_text(json.dumps(dict(
        status="SOURCE_REVIEW_FREEZE_NOT_PUBLISHED", contract_sha256=sha(target.read_bytes()),
        source_contract_sha256=sha(source.read_bytes()), source_contract=str(source),
        new_glyphs=len(copies), metatiles=len(blocks), map_cells=len(result["cells"]),
        new_logical_slots=[f"{copy['target']:02x}" for copy in copies],
        source_rgb_reuse_witnesses=reuse, four_native_banks=["morn","day","nite","dark"]),
        indent=2) + "\n")
    print(json.dumps(dict(contract=str(target), sha256=sha(target.read_bytes()), new_glyphs=len(copies))))


if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("--source", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    freeze(args.source, args.output)
