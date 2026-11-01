#!/usr/bin/env python3
"""Freeze the corrected façade recipes, keeping every bitmap glyph unchanged."""
import hashlib
import json
from pathlib import Path
import sys

ROOT=Path(__file__).resolve().parents[4]
sys.path.insert(0,str(ROOT/"planning/minnesota/tools"))
from render_reference import load_pack


def sha(value):return hashlib.sha256(value).hexdigest()
def canonical(value):return json.dumps(value,sort_keys=True,separators=(",",":"),ensure_ascii=False).encode()


def freeze(epoch,output):
    assert not output.exists()
    output.mkdir(parents=True)
    source=json.loads((epoch/"tile-join-correction.contract.json").read_bytes())
    path=ROOT/"content-packs/minnesota-city-river-bend-v01.crystalpack"
    assert sha(path.read_bytes())==source["parent_pack_sha256"]
    pack=load_pack(path);ts=source["tileset"];files=pack["runtime_files"]
    old=bytes(files[f"data/tilesets/{ts}_metatiles.bin"])
    assert len(old)==157*16
    assert len(source["new_metatiles"])==12 and len(source["map_patches"])==83
    assert all(g in old for b in source["new_metatiles"]for g in b["glyphs"])
    inputs=[f"data/tilesets/{ts}_metatiles.bin",f"data/tilesets/{ts}.json",
            f"data/tilesets/{ts}_palette_map.json",f"gfx/tilesets/{ts}.2bpp",
            f"gfx/tilesets/{ts}.png","gfx/tilesets/bg_tiles.pal"]
    contract=dict(schema_version=1,base_pack_sha256=source["parent_pack_sha256"],
                  base_identity=pack["identity"]["content_hash"],base_map_count=len(pack["data"]["maps"]),
                  map="MnMinneapolis",tileset=ts,expected_blocks=157,
                  base_module_sha256=sha(canonical(pack["data"]["maps"]["MnMinneapolis"])),
                  base_definition_sha256=sha(canonical(pack["data"]["tilesets"][ts])),
                  source_files_sha256={p:sha(bytes(files[p]))for p in sorted(inputs)},
                  blocks=[dict(target=int(b["id"],16),glyphs=b["glyphs"],collision=b["collision"])for b in source["new_metatiles"]],
                  cells=[dict(metatile=c["metatile"],before=int(c["before"],16),after=int(c["after"],16))for c in source["map_patches"]])
    target=output/"native-joins.contract.json";target.write_text(json.dumps(contract,indent=2)+"\n")
    (output/"tile-join-correction.contract.json").write_bytes((epoch/"tile-join-correction.contract.json").read_bytes())
    (output/"source-review.json").write_text(json.dumps(dict(status="CORRECTIVE_SOURCE_FROZEN_NOT_NATIVE_PUBLISHED",
        contract_sha256=sha(target.read_bytes()),source_epoch=str(epoch),new_bitmap_glyphs=0,
        old_palette_entries_changed=0,new_metatiles=12,changed_map_cells=83,audited_buildings=40,
        affected_buildings=32,grammar_assertions="No interior side caps; no upper-storey foundations; roof caps at outer edges only; ground-door frames remain native; masonry right ends closed"),indent=2)+"\n")
    print(json.dumps(dict(contract=str(target),sha256=sha(target.read_bytes()))))


if __name__=="__main__":freeze(Path(sys.argv[1]),Path(sys.argv[2]))
