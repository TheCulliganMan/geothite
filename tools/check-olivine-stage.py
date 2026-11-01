#!/usr/bin/env python3
"""Verify original terrace geometry and its guard against an ignored local pack.

Pack identities stay in memory or a deleted temporary Rust regression harness.
No source catalogs, artwork, collision dumps or audit manifests are delivered.
"""
import argparse
from collections import Counter
import importlib.util
import json
from pathlib import Path
import re
import runpy
import subprocess
import tempfile

ROOT = Path(__file__).resolve().parents[1]


def load_module(name, path):
    spec = importlib.util.spec_from_file_location(name, path)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--pack", type=Path, required=True)
    args = parser.parse_args()
    generator = load_module("terrace", ROOT / "tools/build-olivine-stage.py")
    model = ROOT / "crates/crystal-voxel-view/models/interiors/olivine_stage.mesh.json"
    document = json.loads(model.read_text())
    assert document == json.loads(json.dumps(generator.document()))
    validate = runpy.run_path(str(ROOT / "tools/check-dungeon-models.py"),
                             run_name="geometry_checker")["validate"]
    triangles = validate(model)
    assert document["bounds"] == {"min": [0, 0, 0], "max": [96, 8, 64]}
    assert sum(p["name"].startswith("Terrace paving") for p in document["primitives"]) == 40
    assert sum(p["name"].startswith("Cut golden retaining") for p in document["primitives"]) == 46
    for p in document["primitives"]:
        vertices = [p["positions"][i:i+3] for i in range(0, len(p["positions"]), 3)]
        assert all(0 <= x <= 96 and 0 <= y <= 8 and 0 <= z <= 64 for x, y, z in vertices)
        if p["name"] == "North full tread":
            assert min(v[1] for v in vertices) == 7.8 and max(v[1] for v in vertices) == 8
        if p["name"] == "South full tread":
            assert min(v[1] for v in vertices) == 3.8 and max(v[1] for v in vertices) == 4
    loader = load_module("pack_check", ROOT / "tools/check-department-store.py")
    pack = loader.load_pack(args.pack)
    room = pack["data"]["maps"]["OlivineGym"]
    attributes = room["attributes"]
    layout = pack["runtime_files"]["data/tilesets/champions_room_metatiles.bin"]
    collisions = pack["data"]["tilesets"]["champions_room"]["collision"]
    identities = []
    roles = {name: Counter() for name in ["deck", "stairs", "retaining", "corner"]}
    for y in range(8):
        for x in range(12):
            gx, gy = x+4, y+4
            block = room["blocks"][gy//4*attributes["width"]+gx//4]
            sx, sy = gx % 4, gy % 4
            tile = layout[block*16+sy*4+sx]
            identities.append((block, sx, sy, tile))
            role = ("deck" if tile == 0x51 else "stairs" if tile in [0x54, 0x55]
                    else "corner" if tile == 0x53 else "retaining")
            roles[role][collisions[f"{block:02x}"][sy//2*2+sx//2]] += 1
    assert roles == {"deck": Counter(FLOOR=32, UP_WALL=8), "stairs": Counter(FLOOR=8),
                     "retaining": Counter(WALL=38, UP_WALL=8), "corner": Counter(WALL=2)}
    source = (ROOT / "crates/crystal-voxel-view/src/mesh/olivine_stage.rs").read_text()
    pure = source[source.index("const DRAWING:"):source.index("#[derive(Clone, Copy)]")]
    fixture = ",".join(f"({b},{x},{y},{tile})" for b, x, y, tile in identities)
    code = '''use std::sync::Arc;
#[derive(Clone)] struct VisualTileSource { tileset_id:Arc<str>, metatile_id:u16,
subtile_column:u8, subtile_row:u8, tile_index:u16 }
''' + pure + '''
fn main() {
let sources:Vec<_> = [''' + fixture + '''].into_iter().map(|(b,x,y,t)|
VisualTileSource{tileset_id:Arc::from("champions_room"),metatile_id:b,
subtile_column:x,subtile_row:y,tile_index:t}).collect();
assert_eq!(ANCHOR,[4,4]); assert_eq!(SIZE,[12,8]);
assert_eq!(identity_hash(sources.iter()),Some(DRAWING));
for i in 0..96 { for field in 0..5 {
let mut changed=sources.clone(); let s=&mut changed[i];
match field {0=>s.metatile_id^=128,1=>s.tile_index^=128,
2=>s.subtile_column^=128,3=>s.subtile_row^=128,_=>s.tileset_id=Arc::from("other")}
assert_ne!(identity_hash(changed.iter()),Some(DRAWING));
}}
for i in 0..96 {let mut cropped=sources.clone();cropped.remove(i);
assert_eq!(identity_hash(cropped.iter()),None);}
println!("Native terrace guard: 480 altered sources and 96 incomplete drawings rejected");
}
'''
    with tempfile.TemporaryDirectory(prefix="olivine-stage-native-") as tmp:
        path = Path(tmp) / "check.rs"
        binary = Path(tmp) / "check"
        path.write_text(code)
        subprocess.run(["rustc", "--edition=2024", str(path), "-o", str(binary)], check=True)
        subprocess.run([str(binary)], check=True)
    print(f"PASS: {len(document['primitives'])} closed original parts, {triangles} triangles; "
          "deck/stairs and original directional-barrier roles verified")


if __name__ == "__main__":
    main()
