#!/usr/bin/env python3
"""Check the two Rocket plants against the external pack and production matcher.

No Cargo/Blender, exported source catalogs, or repository-generated fixtures.
The optional --source path permits verification of a staged pure Rust module.
"""
import argparse
import importlib.util
import json
import os
from pathlib import Path
import shutil
import subprocess
import tempfile

ROOT = Path(__file__).resolve().parents[1]
BODY = r'''
fn check(cells: Vec<Identity<'static>>, w: usize, h: usize) {
    let free = vec![false; cells.len()];
    let out = source::resolve(source::MAP, &cells, w, h, [0, 0], &free);
    assert_eq!(out.len(), 2);
    assert_eq!(out.iter().map(|p| p.indices(w).count()).sum::<usize>(), 12);
    assert!(source::resolve("TeamRocketBaseB2F", &cells, w, h, [0, 0], &free).is_empty());
    assert!(source::resolve(source::MAP, &cells, w, h, [1, 0], &free).is_empty());
    assert!(source::resolve(source::MAP, &cells, w + 1, h, [0, 0], &free).is_empty());
    assert!(source::resolve(source::MAP, &cells, w, h, [0, 0], &free[..free.len()-1]).is_empty());
    let mut owned = vec![false; cells.len()];
    for p in &out {
        for i in p.indices(w) { assert!(!owned[i]); owned[i] = true; }
        assert_eq!(cells[p.ground(w)].tile, 0x10);
        for i in p.guard_indices(w) {
            for mode in 0..6 {
                let mut changed = cells.clone(); let mut blocked = free.clone();
                match mode {
                    0 => changed[i].tile ^= 1, 1 => changed[i].metatile ^= 1,
                    2 => changed[i].column ^= 1, 3 => changed[i].row ^= 1,
                    4 => changed[i].tileset = "custom", _ => blocked[i] = true,
                }
                if mode < 5 { assert!(!p.coherent(&changed, w, h)); }
                let after = source::resolve(source::MAP, &changed, w, h, [0, 0], &blocked);
                assert!(!after.iter().any(|q| q.column == p.column && q.row == p.row));
                assert_eq!(after.len(), 1, "the other complete plant remains independent");
            }
        }
        // Clip each edge of the full block, including the native floor donor.
        for side in 0..4 {
            let x0 = if side == 0 { p.column + 1 } else { 0 };
            let y0 = if side == 1 { p.row + 1 } else { 0 };
            let x1 = if side == 2 { p.column + 3 } else { w };
            let y1 = if side == 3 { p.row + 3 } else { h };
            let cw = x1 - x0; let ch = y1 - y0;
            let crop: Vec<_> = (y0..y1).flat_map(|y| cells[y*w+x0..y*w+x1].iter().copied()).collect();
            let after = source::resolve(source::MAP, &crop, cw, ch, [x0 as i32, y0 as i32], &vec![false; crop.len()]);
            assert!(!after.iter().any(|q| q.column + x0 == p.column && q.row + y0 == p.row));
        }
    }
    // A real crop and a padded/scrolling canvas retain the same map anchors.
    let crop: Vec<_> = (24..28).flat_map(|y| cells[y*w+24..y*w+32].iter().copied()).collect();
    assert_eq!(source::resolve(source::MAP, &crop, 8, 4, [24,24], &vec![false;32]).len(), 2);
    let blank = Identity { tileset:"none", metatile:0, column:0, row:0, tile:0 };
    let (pw, ph) = (w + 6, h + 6); let mut padded = vec![blank; pw*ph];
    for y in 0..h { padded[(y+3)*pw+3..(y+3)*pw+3+w].copy_from_slice(&cells[y*w..(y+1)*w]); }
    let p = source::resolve(source::MAP, &padded, pw, ph, [-3,-3], &vec![false; padded.len()]);
    assert_eq!(p.iter().map(|p|p.indices(pw).count()).sum::<usize>(), 12);
    println!("PASS 2 plants / 12 owned cells: every source/custom/ground/phase/crop guard");
}
'''

def main():
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument('--pack', type=Path, required=True)
    ap.add_argument('--repo', type=Path, default=ROOT)
    ap.add_argument('--source', type=Path)
    ap.add_argument('--rustc', default=os.getenv('RUSTC') or shutil.which('rustc'))
    args = ap.parse_args()
    assert args.rustc, 'pass --rustc'
    spec = importlib.util.spec_from_file_location('pack_reader', args.repo / 'tools/check-traditional-room-source.py')
    reader = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(reader)
    pack = reader.load(args.pack)
    m = pack['data']['maps']['TeamRocketBaseB1F']
    assert m['attributes']['tileset_name'] == 'underground'
    assert (m['attributes']['width'], m['attributes']['height']) == (15, 9)
    assert not m['script_block_changes']
    w, h = 60, 36
    layout = pack['runtime_files']['data/tilesets/underground_metatiles.bin']
    collision = pack['data']['tilesets']['underground']['collision']
    assert [(i % 15, i // 15, b) for i,b in enumerate(m['blocks']) if b in (0x29,0x2a)] == [(6,6,0x29),(7,6,0x2a)]
    owned = {(x,y) for x in range(26,30) for y in range(24,27)}
    assert len(owned) == 12
    for x,y in owned:
        b = m['blocks'][y//4*15+x//4]
        assert collision[f'{b:02x}'][y%4//2*2+x%4//2] == 'WALL'
    # Native actor, coordinate, interaction and warp boundaries stay outside
    # the 12 source cells. These assertions read, never edit, game state.
    for event in m['objects'] + sum(m['events'].values(), []):
        assert all((event['x']*2+dx,event['y']*2+dy) not in owned for dy in range(2) for dx in range(2))
    for b, offset in [(0x29,2),(0x2a,0)]:
        for y in range(4):
            for x in range(4):
                expected = [[0x1e,0x1f],[0x2e,0x2f],[0x3e,0x3f]][y][x-offset] if y<3 and offset<=x<offset+2 else 0x10
                assert layout[b*16+y*4+x] == expected
    module = args.source or args.repo / 'crates/crystal-voxel-view/src/mesh/rocket_plants_source.rs'
    code = f'#[path={json.dumps(str(module.resolve()))}] mod source;\nuse source::Identity;\n' + BODY + '\nfn main() { check(vec![\n'
    for y in range(h):
        for x in range(w):
            b = m['blocks'][y//4*15+x//4]
            t = layout[b*16+y%4*4+x%4]
            code += f'Identity{{tileset:"underground",metatile:{b},column:{x%4},row:{y%4},tile:{t}}},\n'
    code += f'],{w},{h}); }}\n'
    with tempfile.TemporaryDirectory(prefix='geothite-rocket-plants-') as tmp:
        root = Path(tmp)
        (root / 'check.rs').write_text(code)
        subprocess.run([args.rustc,'--edition=2024','-O','-A','dead_code',str(root/'check.rs'),'-o',str(root/'check')],check=True)
        subprocess.run([str(root/'check')],check=True)
    print(json.dumps({'map':'TeamRocketBaseB1F','plants':2,'owned_cells':12,'guard_cells':32,
        'unchanged_actors':len(m['objects']), 'unchanged_events':{k:len(v)for k,v in m['events'].items()},
        'native_collision':'WALL','underlay':'underground tile 0x10 within each guarded block',
        'native_review':'pending; source proof is not visual approval'},indent=2))

if __name__ == '__main__':
    main()
