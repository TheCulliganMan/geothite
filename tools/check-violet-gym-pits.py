#!/usr/bin/env python3
"""Audit the production Violet recess matcher/geometry against an external pack.

No source pack, image, map dump or serialized source catalog is saved in the
checkout. Fixtures exist only in a temporary rustc harness. This focused check
is separate from native game screenshot QA. Optional PLY output is actual
production geometry for a temporary model preview, never a shipped asset.
"""
import argparse
from pathlib import Path
import os
import runpy
import shutil
import subprocess
import tempfile

ROOT = Path(__file__).resolve().parents[1]
BODY = r'''
fn verify(cells: Vec<Identity<'static>>, blocked: Vec<bool>, w: usize, h: usize) {
    let empty = vec![false; cells.len()];
    let p = source::resolve("VioletGym", &cells, w, h, &empty);
    assert_eq!(p.cells.len(), 176);
    let mut per_block = [0usize; 2];
    let mut owned = vec![false; cells.len()];
    for &i in &p.cells {
        assert!(blocked[i], "passable native source claimed at {i}");
        assert!(!owned[i]); owned[i] = true;
        per_block[usize::from(cells[i].metatile == 0x19)] += 1;
    }
    assert_eq!(per_block, [96, 80]);
    let mut sides = [0usize; 4];
    for e in &p.edges {
        sides[e.side] += 1;
        assert!(owned[e.cell]);
        let (x,y) = (e.cell % w, e.cell / w);
        let neighbor = match e.side {
            0 => y.checked_sub(1).map(|y|y*w+x),
            1 => (x+1<w).then_some(e.cell+1),
            2 => (y+1<h).then_some(e.cell+w),
            _ => x.checked_sub(1).map(|x|y*w+x),
        };
        assert!(!neighbor.is_some_and(|i|owned[i]), "internal pit wall");
    }
    assert_eq!(sides, [40,28,40,28]);
    assert_eq!(p.edges.iter().filter(|e|e.rim).count(),108);
    assert_eq!((176*4-p.edges.len())/2,284);
    let mut remaining=owned.clone();let mut sizes=vec![];
    while let Some(first)=remaining.iter().position(|&v|v) {
        remaining[first]=false;let mut todo=vec![first];let mut size=0;
        while let Some(i)=todo.pop() {
            size+=1;let(x,y)=(i%w,i/w);
            for j in [y.checked_sub(1).map(|y|y*w+x),(x+1<w).then_some(i+1),
                (y+1<h).then_some(i+w),x.checked_sub(1).map(|x|y*w+x)].into_iter().flatten() {
                if remaining[j] {remaining[j]=false;todo.push(j);}
            }
        } sizes.push(size);
    }
    assert_eq!(sizes,vec![88,88]);
    assert!(source::resolve("MahoganyGym",&cells,w,h,&empty).cells.is_empty());
    assert!(source::resolve("VioletGym",&cells,w+1,h,&empty).cells.is_empty());
    assert!(source::resolve("VioletGym",&cells,w,h,&[]).cells.is_empty());
    // Every independent whole source drawing is atomic, including its floor.
    for (i,s) in cells.iter().enumerate().filter(|(_,s)|matches!(s.metatile,0x17|0x19)&&s.column==0&&s.row==0) {
        let count=if s.metatile==0x17{16}else{8};
        for mode in 0..5 {
            let mut changed=cells.clone();let mut custom=empty.clone();
            match mode {0=>changed[i].tile^=1,1=>changed[i].column=1,2=>changed[i].metatile^=1,
                3=>changed[i].tileset="custom",_=>custom[i]=true}
            let after=source::resolve("VioletGym",&changed,w,h,&custom);
            assert_eq!(after.cells.len(),176-count,"whole source guard {mode}");
        }
    }
    // Viewport padding changes indices only. Source-phase proof stays exact.
    let pad=3;let pw=w+pad*2;let ph=h+pad*2;
    let blank=Identity{tileset:"none",metatile:0,column:0,row:0,tile:0};
    let mut padded=vec![blank;pw*ph];
    for y in 0..h {padded[(y+pad)*pw+pad..(y+pad)*pw+pad+w].copy_from_slice(&cells[y*w..(y+1)*w]);}
    let pp=source::resolve("VioletGym",&padded,pw,ph,&vec![false;padded.len()]);
    assert_eq!(pp.cells.len(),176);assert_eq!(pp.edges.len(),136);
    assert_eq!(pp.edges.iter().filter(|e|e.rim).count(),108);
    for &i in &pp.cells {assert!(owned[(i/pw-pad)*w+i%pw-pad]);}
    // Clipping a block cannot replace half of its authored drawing.
    let cw=w-1;let cropped=(0..h).flat_map(|y|cells[y*w..y*w+cw].iter().copied()).collect::<Vec<_>>();
    let pc=source::resolve("VioletGym",&cropped,cw,h,&vec![false;cw*h]);
    assert_eq!(pc.cells.len(),120);
    let g=GridGeometry{width:w,height:h,tile_width:8.,tile_height:8.,origin_x:0.,origin_z:0.};
    let mut mesh=SurfaceMeshData::default();append_geometry(&mut mesh,&g,&p);
    assert_eq!(mesh.positions.len(),(176+136*6+108)*4);
    assert_eq!(mesh.indices.len()/3,2200);
    assert_eq!(mesh.positions.iter().map(|p|p[1]).fold(f32::INFINITY,f32::min),-24.);
    assert!(mesh.positions.iter().all(|p|p.iter().all(|v|v.is_finite())&&p[1]>=-24.&&p[1]<=0.));
    for face in mesh.positions.chunks_exact(4).zip(mesh.normals.chunks_exact(4)) {
        let(v,n)=face;let a=[v[1][0]-v[0][0],v[1][1]-v[0][1],v[1][2]-v[0][2]];
        let b=[v[2][0]-v[0][0],v[2][1]-v[0][1],v[2][2]-v[0][2]];
        let cross=[a[1]*b[2]-a[2]*b[1],a[2]*b[0]-a[0]*b[2],a[0]*b[1]-a[1]*b[0]];
        assert!(cross.iter().zip(n[0]).map(|(a,b)|a*b).sum::<f32>()>0.,"face winding");
        let x=v.iter().map(|p|p[0]).sum::<f32>()/4.;let z=v.iter().map(|p|p[2]).sum::<f32>()/4.;
        let cx=(x/8.).floor().clamp(0.,(w-1)as f32)as usize;
        let cy=(z/8.).floor().clamp(0.,(h-1)as f32)as usize;
        // Boundary walls may lie exactly on the adjacent platform coordinate;
        // a small step along their inward normal must remain inside the pit.
        let xx=((x+n[0][0]*0.001)/8.).floor().clamp(0.,(w-1)as f32)as usize;
        let yy=((z+n[0][2]*0.001)/8.).floor().clamp(0.,(h-1)as f32)as usize;
        assert!(owned[cy*w+cx]||owned[yy*w+xx],"geometry outside source footprint");
    }
    if let Some(path)=std::env::args().nth(1) {
        use std::io::Write;let mut f=std::fs::File::create(path).unwrap();
        writeln!(f,"ply\nformat ascii 1.0\nelement vertex {}\nproperty float x\nproperty float y\nproperty float z\nproperty uchar red\nproperty uchar green\nproperty uchar blue\nelement face {}\nproperty list uchar int vertex_indices\nend_header",mesh.positions.len(),mesh.indices.len()/3).unwrap();
        for(p,c)in mesh.positions.iter().zip(&mesh.colors){writeln!(f,"{} {} {} {} {} {}",p[0],p[1],p[2],(c[0]*255.)as u8,(c[1]*255.)as u8,(c[2]*255.)as u8).unwrap();}
        for t in mesh.indices.chunks_exact(3){writeln!(f,"3 {} {} {}",t[0],t[1],t[2]).unwrap();}
    }
    println!("PASS VioletGym: 176 WALL-only cells = two 88-cell recesses; 136 exposed edges (108 platform rims); 284 joined edges have no internal faces; 2200 triangles at depth 24; exact source/custom/padding/crop/winding/footprint guards");
}
'''

def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--pack',type=Path,required=True)
    parser.add_argument('--rustc',default=os.getenv('RUSTC') or shutil.which('rustc'))
    parser.add_argument('--mesh-output',type=Path)
    args=parser.parse_args()
    assert args.rustc,'Set RUSTC or pass --rustc'
    item=runpy.run_path(str(ROOT/'tools/check-gym-floor.py'))['item']
    load=runpy.run_path(str(ROOT/'tools/check-gym-scenery-source.py'))['load']
    pack=load(args.pack);m=pack['data']['maps']['VioletGym'];ts=m['attributes']['tileset_name']
    assert ts=='elite_four_room' and not m['script_block_changes']
    w,h=m['attributes']['width']*4,m['attributes']['height']*4
    layout=pack['runtime_files'][f'data/tilesets/{ts}_metatiles.bin']
    collisions=pack['data']['tilesets'][ts]['collision']
    src=(ROOT/'crates/crystal-voxel-view/src/mesh.rs').read_text()
    module=ROOT/'crates/crystal-voxel-view/src/mesh/violet_gym_pit_source.rs'
    geometry=(ROOT/'crates/crystal-voxel-view/src/mesh/violet_gym_pits.rs').read_text()
    code='#![allow(dead_code)]\nmod audit {\n'
    code+=f'#[path="{module}"] mod source;\nuse source::Identity;\n'
    code+='const SOURCE_TILE_HEIGHT:f32=8.;const DEPTH_PIXELS:f32=24.;\n'
    code+='#[derive(Default)] struct SurfaceMeshData{positions:Vec<[f32;3]>,normals:Vec<[f32;3]>,uvs:Vec<[f32;2]>,colors:Vec<[f32;4]>,indices:Vec<u32>}\n'
    code+='#[derive(Clone,Copy)]\n'+item(src,'struct GridGeometry')+item(src,'impl GridGeometry')
    for name in ['append_solid_quad','append_quad','append_quad_colors']:code+=item(src,'fn '+name+'(')
    for name in ['color','append_geometry']:code+=item(geometry,'fn '+name+'(')
    code+=BODY+'pub fn run(){verify(vec![\n'
    collision=[]
    for y in range(h):
        for x in range(w):
            b=m['blocks'][(y//4)*(w//4)+x//4];tile=layout[b*16+(y%4)*4+x%4]
            code+=f'Identity{{tileset:"{ts}",metatile:{b},column:{x%4},row:{y%4},tile:{tile}}},\n'
            collision.append(collisions[f'{b:02x}'][(y%4)//2*2+(x%4)//2]=='WALL')
    code+='],vec!['+','.join(str(v).lower() for v in collision)+f'],{w},{h});}}}}\nfn main(){{audit::run();}}\n'
    with tempfile.TemporaryDirectory(prefix='geothite-violet-pits-') as temporary:
        path=Path(temporary);(path/'check.rs').write_text(code)
        subprocess.run([args.rustc,'--edition=2024',str(path/'check.rs'),'-o',str(path/'check')],check=True)
        command=[str(path/'check')]
        if args.mesh_output:
            args.mesh_output.parent.mkdir(parents=True,exist_ok=True);command.append(str(args.mesh_output.resolve()))
        subprocess.run(command,check=True)
    print('PASS external pack remained read-only; native battle/walking screenshot QA is a separate check')

if __name__=='__main__':main()
