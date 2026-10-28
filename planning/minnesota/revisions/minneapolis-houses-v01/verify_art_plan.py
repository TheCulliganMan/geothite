#!/usr/bin/env python3
"""Independently resolve the frozen bank expansion in memory, without pack writes."""
from copy import deepcopy
import hashlib
import io
import json
from pathlib import Path
import sys

from PIL import Image, ImageChops

ROOT = Path(__file__).resolve().parents[4]
sys.path.insert(0, str(ROOT / "planning/minnesota/tools"))
from render_reference import SourceArt, load_pack
from render_archive import save_render


def sha(value):
    return hashlib.sha256(value).hexdigest()


def physical(glyph, palette, count):
    return (glyph & 127) + count // 2 if palette & 8 else glyph


def verify(folder):
    folder.mkdir(parents=True, exist_ok=False)
    source = Path(__file__).resolve().parent
    plan_path = source / "pilot-v03/native-publication.contract.json"
    spec_path = source / "pilot-v03/house-exteriors.contract.json"
    plan, spec = json.loads(plan_path.read_bytes()), json.loads(spec_path.read_bytes())
    pack_path = ROOT / "content-packs/minnesota-city-river-bend-v01.crystalpack"
    assert sha(pack_path.read_bytes()) == plan["base_pack_sha256"]
    pack = load_pack(pack_path)
    before_art = SourceArt(pack)
    target = deepcopy(pack)
    files = target["runtime_files"]
    ts = plan["tileset"]
    definition = target["data"]["tilesets"][ts]
    old_pixels = bytes(files[f"gfx/tilesets/{ts}.2bpp"])
    graphics = bytearray(256 * 16)
    graphics[:96*16] = old_pixels[:96*16]
    graphics[128*16:224*16] = old_pixels[96*16:]
    definition["palette_map"].extend([0] * 32)
    for copy in plan["glyph_copies"]:
        donor = pack["data"]["tilesets"][copy["source_tileset"]]
        palette = donor["palette_map"][copy["source_glyph"]]
        pixels = bytes(pack["runtime_files"][f"gfx/tilesets/{copy['source_tileset']}.2bpp"])
        slot = physical(copy["source_glyph"], palette, len(pixels)//16)
        pixels = pixels[slot*16:slot*16+16]
        assert sha(pixels) == copy["pixels_sha256"]
        graphics[copy["target"]*16:copy["target"]*16+16] = pixels
        definition["palette_map"][copy["target"]] = (palette & 7) | 8
    # A temporary memory-only PNG exercises SourceArt's actual bank resolver.
    image = Image.new("L", (128, 128))
    for glyph in range(256):
        raw = graphics[glyph*16:glyph*16+16]
        for y in range(8):
            for x in range(8):
                bit = 7-x
                index = ((raw[y*2] >> bit) & 1) | (((raw[y*2+1] >> bit) & 1) << 1)
                image.putpixel((glyph%16*8+x,glyph//16*8+y),[255,170,85,0][index])
    encoded = io.BytesIO()
    image.save(encoded,format="PNG")
    files[f"gfx/tilesets/{ts}.png"] = encoded.getvalue()
    metatiles = bytearray(files[f"data/tilesets/{ts}_metatiles.bin"])
    for block in plan["blocks"]:
        assert block["target"] == len(metatiles)//16
        metatiles.extend(block["glyphs"])
        definition["collision"][f"{block['target']:02x}"] = block["collision"]
    files[f"data/tilesets/{ts}_metatiles.bin"] = bytes(metatiles)
    after_art = SourceArt(target)
    old_checked = new_checked = 0
    for bank in ["morn","day","nite","dark"]:
        original, candidate = before_art.blocks(ts,bank), after_art.blocks(ts,bank)
        assert len(original) == 157 and len(candidate) == 170
        for i in range(157):
            assert ImageChops.difference(original[i],candidate[i]).getbbox() is None,(bank,i)
            old_checked += 1024
        for recipe in spec["new_metatiles"]:
            expected = Image.new("RGB",(32,32))
            for i, glyph in enumerate(recipe["glyphs"]):
                donor = before_art.blocks(glyph["tileset"],bank)[int(glyph["block"],16)]
                x,y=glyph["glyph"]
                expected.paste(donor.crop((x*8,y*8,x*8+8,y*8+8)),(i%4*8,i//4*8))
            assert ImageChops.difference(expected,candidate[int(recipe["id"],16)]).getbbox() is None,(bank,recipe["id"])
            new_checked += 1024
    blocks = list(pack["data"]["maps"]["MnMinneapolis"]["blocks"])
    for cell in plan["cells"]:
        x,y=cell["metatile"]
        assert blocks[y*40+x] == cell["before"]
        blocks[y*40+x]=cell["after"]
    scene = after_art.render(ts,40,32,blocks)
    comparison = Image.open(ROOT/"planning/minnesota/renders/minneapolis-houses-v01/pilot-v03/whole-city-candidate.png").convert("RGB")
    assert ImageChops.difference(scene,comparison).getbbox() is None
    save_render(scene,folder/"bank-resolved-source-candidate.png")
    receipt=dict(status="IN_MEMORY_ART_PLAN_PROOF_NOT_RUST_PUBLICATION_OR_GPU",
                 contract_sha256=sha(plan_path.read_bytes()), old_metatile_four_bank_pixels_equal=old_checked,
                 new_native_recipe_four_bank_pixels_equal=new_checked,whole_city_day_pixels_equal=1280*1024,
                 mismatch_pixels=0,pack_files_written=0,old_palette_entries_preserved=224,
                 expanded_physical_glyph_count=256,new_glyph_count=27)
    (folder/"receipt.json").write_text(json.dumps(receipt,indent=2)+"\n")
    print(json.dumps(receipt))


if __name__ == "__main__":
    verify(Path(sys.argv[1]))
