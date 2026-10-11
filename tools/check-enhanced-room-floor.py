#!/usr/bin/env python3
"""Verify production room-floor selectors against an ignored external pack.

The pack stays in memory. A temporary Rust harness is deleted after checking;
no source artwork, content catalog, or audit manifest is delivered.
"""
import argparse
import importlib.util
import subprocess
import tempfile
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
MAPS = ["SproutTower1F", "SproutTower2F", "SproutTower3F", "CianwoodGym",
        "EcruteakGym", "PewterGym", "VioletGym", "CeruleanGym", "MahoganyGym", "BlackthornGym1F", "BlackthornGym2F", "OlivineGym", "GoldenrodGameCorner", "CeladonGameCorner", "CeladonGameCornerPrizeRoom"]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--pack", type=Path, required=True)
    args = parser.parse_args()
    spec = importlib.util.spec_from_file_location("department_check", ROOT / "tools/check-department-store.py")
    loader = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(loader)
    pack = loader.load_pack(args.pack)
    code = '''#![allow(dead_code)]
use std::sync::Arc;
#[derive(Clone)] struct VisualTileSource { tileset_id:Arc<str>, metatile_id:u16, subtile_column:u8, subtile_row:u8, tile_index:u16 }
#[derive(Debug,PartialEq)] enum InteriorFloor { Oak, AmberTimber, Stone, VioletBrick, TowerDepth, ArcadeChecker, PoolCeramic, MahoganyIce, MahoganyGrip, BlackthornTile, OlivineTile, OlivineRunner }
'''
    casino = (ROOT / 'crates/crystal-voxel-view/src/casino.rs').read_text()
    floor_function = casino[casino.index('pub(crate) fn is_floor_cell'):casino.index('pub(crate) fn is_game_corner_map')]
    code += 'mod casino { use super::*;\n' + floor_function + '\n}\n'
    port = (ROOT / 'crates/crystal-voxel-view/src/port.rs').read_text()
    floor_function = port[port.index('pub(crate) fn cerulean_floor_source'):port.index('pub(crate) fn port_shape')]
    code += 'mod port { use super::*;\n' + floor_function + '\n}\n'
    code += f'include!("{ROOT}/crates/crystal-voxel-view/src/mesh/enhanced_room_floor.rs");\n'
    code += 'fn main() {\n'
    for name in MAPS:
        room = pack['data']['maps'][name]
        atlas = room['attributes']['tileset_name']
        layout = pack['runtime_files'][f'data/tilesets/{atlas}_metatiles.bin']
        collision = pack['data']['tilesets'][atlas]['collision']
        identities = set()
        for block in room['blocks']:
            for y in range(4):
                for x in range(4):
                    tile = layout[block * 16 + y * 4 + x]
                    floor = collision[f'{block:02x}'][y // 2 * 2 + x // 2] == 'FLOOR'
                    ice = collision[f'{block:02x}'][y // 2 * 2 + x // 2] == 'ICE'
                    identities.add((block, x, y, tile, floor, ice))
        fixture = ','.join(f'({b},{x},{y},{t},{str(f).lower()},{str(i).lower()})' for b, x, y, t, f, i in sorted(identities))
        code += f'''{{ let map="{name}"; let mut count=0; let mut depth=0; let mut ice_count=0;
for (block,x,y,tile,floor,ice) in [{fixture}] {{
let source=VisualTileSource {{tileset_id:Arc::from("{atlas}"),metatile_id:block,subtile_column:x,subtile_row:y,tile_index:tile}};
if let Some(style)=enhanced_room_floor_source(map,&source) {{
if style==InteriorFloor::TowerDepth {{ assert!(!floor); depth+=1; }}
else if style==InteriorFloor::MahoganyIce {{ assert!(ice); ice_count+=1; }}
else {{ assert!(floor,"{{map}}: non-floor block {{block:x}} at {{x}},{{y}}"); count+=1; }}
for mode in 0..3 {{ let mut bad=source.clone(); match mode {{ 0=>bad.tile_index^=0x80,1=>bad.metatile_id=0xffff,_=>bad.tileset_id=Arc::from("custom") }};
assert!(enhanced_room_floor_source(map,&bad).is_none()); }}
}}
}}
assert!(count>0); println!("{{map}}: {{count}} floor, {{ice_count}} ice and {{depth}} intentional depth identities; source and mutation guards pass"); }}\n'''
    code += '}\n'
    with tempfile.TemporaryDirectory(prefix='geothite-room-floor-') as scratch:
        source = Path(scratch) / 'check.rs'
        binary = Path(scratch) / 'check'
        source.write_text(code)
        subprocess.run(['rustc', '--edition=2024', str(source), '-o', str(binary)], check=True)
        subprocess.run([str(binary)], check=True)


if __name__ == '__main__':
    main()
