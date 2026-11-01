#!/usr/bin/env python3
"""Exercise production Gym surface code and external-pack FLOOR identities with rustc.
No Cargo, game pictures, persistent fixture catalogs, Blender, or GUI is needed.
The small harness supplies renderer types and a flat-source shape shim; run the
native crate tests and screenshot QA separately after integrating the patch.
"""
import argparse
from pathlib import Path
import os
import runpy
import shutil
import subprocess
import tempfile


def item(text, start):
    begin = text.index(start)
    brace = text.index('{', begin)
    depth, end = 1, brace + 1
    while depth:
        depth += (text[end] == '{') - (text[end] == '}')
        end += 1
    return text[begin:end] + '\n'


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--pack', required=True, type=Path)
    parser.add_argument('--root', type=Path, default=Path(__file__).resolve().parents[1])
    parser.add_argument('--overlay', type=Path)
    parser.add_argument('--rustc', default=os.getenv('RUSTC') or shutil.which('rustc'))
    args = parser.parse_args()
    assert args.rustc, 'Set RUSTC or pass --rustc'
    def path(rel):
        if args.overlay and (args.overlay / rel).exists():
            return args.overlay / rel
        return args.root / rel
    def read(rel):
        return path(rel).read_text()
    src = read('crates/crystal-voxel-view/src/mesh.rs')
    floor = 'crates/crystal-voxel-view/src/mesh/'
    code = '''#![allow(dead_code, private_interfaces)]
use std::{sync::Arc, collections::HashMap};
#[derive(Clone,Debug,Default,PartialEq)] struct Image;
#[derive(Clone,Debug,Default,PartialEq)] struct Handle<T=Image>(std::marker::PhantomData<T>);
#[derive(Clone,Debug)] struct VisualTileSource {tileset_id:Arc<str>,metatile_id:u16,subtile_column:u8,subtile_row:u8,tile_index:u16}
#[derive(Clone,Debug)] struct VisualTile {column:u32,row:u32,texture:Handle,priority:bool,source:VisualTileSource}
#[derive(Clone,Copy)] enum CellShape {Flat,Water,PlaneAt{height:f32}}
fn shape_for_source_on_map(_: &str,_: &VisualTileSource)->CellShape {CellShape::Flat}
mod cafe {pub fn is_cafe_map(_: &str)->bool {false}}
mod live_profiles {
#[derive(Default)] pub struct Document {pub objects:Vec<Object>,pub atmosphere:Option<()>}
#[derive(Default)] pub struct Object {
 pub name:String,pub tileset:String,pub map:Option<String>,pub maps:Option<Vec<String>>,pub metatile:u16,
 pub metatiles:Option<Vec<Vec<u16>>>,pub origin:[u8;2],pub tiles:Vec<Vec<u16>>,pub ground:u16,
 pub top_pixels:usize,pub depth_pixels:f32,pub footing_pixels:Option<f32>,pub mask:(),pub parts:Vec<()>
}
}
mod engine {
use super::*;use crate::live_profiles::Document;
const TEXTURED_SHADE:[f32;4]=[1.;4];
'''
    for name in ['SurfaceMeshData','TerrainMeshData','RepeatingBackground','TreeMeshInstance']:
        code += '#[derive(Clone,Debug,Default,PartialEq)]\n' + item(src, ('pub struct ' if name in ['SurfaceMeshData','TerrainMeshData'] else 'pub(crate) struct ') + name)
    code += '#[derive(Clone,Debug,PartialEq,Eq,Hash)]\n' + item(src, 'struct TreeMeshKey')
    code += '#[derive(Clone,Copy)]\n' + item(src,'struct GridGeometry') + item(src,'impl GridGeometry')
    for name in ['append_top','append_top_shaded','append_quad','append_quad_colors']:
        code += item(src,'fn '+name+'(')
    live = read(floor+'live.rs')
    code += 'mod live { use super::*;use crate::live_profiles::{Document,Object};\n'
    code += item(live,"pub(super) struct Placement<'a>") + item(live,"impl Placement<'_>") + item(live,"pub(super) fn resolve<'a>") + '}\n'
    gym = read(floor+'gym_scenery.rs')
    code += 'mod gym_scenery { use super::*;use crate::live_profiles::Document;\n'
    code += '#[path="'+str(path(floor+'gym_scenery_source.rs').resolve())+'"] mod source;\nuse source::Identity;\n'
    code += item(gym,'pub(super) struct Placement') + item(gym,'impl Placement')
    code += item(gym,'fn identity(').replace('crystal_render_api::VisualTileSource','VisualTileSource')
    code += item(gym,'pub(super) fn resolve(') + '}\n'
    surfaces = read(floor+'interior_surfaces.rs')
    code += surfaces[:surfaces.index('fn architecture_box(')]
    code += 'fn append_known_room_backing(_: &mut TerrainMeshData,_: &str,_: &[&VisualTile],_: &GridGeometry,_: [i32;2]) {}\n'
    for filename in ['lighthouse_floor.rs','lighthouse_chamber_floor.rs']:
        code += read(floor+filename).split('#[cfg(test)]')[0]
    code += read(floor+'gym_floor.rs')
    load = runpy.run_path(str(args.root/'tools/check-gym-scenery-source.py'))['load']
    pack = load(args.pack)
    expected = {'AzaleaGym':[128,261,0,225], 'GoldenrodGym':[912,0,0,504], 'CeladonGym':[0,152,216,168], 'ViridianGym':[0,0,0,0]}
    # Native identity fixtures are emitted only in the temporary compile folder.
    # Each tuple carries the external pack's independent collision permission.
    code += '''
fn check_native_floor(map:&str,ts:&str,w:usize,h:usize,data:&[(u16,u16,bool)],expected:[usize;4]) {
 let tiles=data.iter().enumerate().map(|(i,&(block,tile,_))|VisualTile{column:(i%w)as u32,row:(i/w)as u32,texture:Default::default(),priority:false,source:VisualTileSource{tileset_id:Arc::from(ts),metatile_id:block,subtile_column:(i%w%4)as u8,subtile_row:(i/w%4)as u8,tile_index:tile}}).collect::<Vec<_>>();
 let cells=tiles.iter().collect::<Vec<_>>();let g=GridGeometry{width:w,height:h,tile_width:8.,tile_height:8.,origin_x:0.,origin_z:0.};
 let mut counts=[0usize;4];let mut m=TerrainMeshData::default();m.authored_cells=vec![None;data.len()];
 for (i,t) in tiles.iter().enumerate() {if let Some(style)=gym_floor_source(map,&t.source) {
  assert!(data[i].2,"non-FLOOR source {map} at {i}");let k=match style{InteriorFloor::GymRose=>0,InteriorFloor::GymGreen=>1,InteriorFloor::GymTimber=>2,_=>panic!()};counts[k]+=1;
  for mode in 0..3 {let mut changed=t.source.clone();match mode{0=>changed.tile_index^=0x80,1=>changed.metatile_id=0xffff,_=>changed.tileset_id=Arc::from("custom")};assert!(gym_floor_source(map,&changed).is_none());}
  append_top(&mut m.textured,g.bounds(i%w,i/w).into(),0.,g.uv(i%w,i/w));
 }}
 if gym_floor_map(map) {for p in gym_scenery::resolve(map,&cells,&g,[0,0],None,&vec![false;data.len()]) {
  let (sample,label)=p.floor_sample_and_label();assert!(gym_floor_source(map,&cells[sample].source).is_some());
  for i in p.indices(w) {assert!(gym_floor_source(map,&cells[i].source).is_none());m.authored_cells[i]=Some(label);counts[3]+=1;append_top(&mut m.textured,g.bounds(i%w,i/w).into(),0.,g.uv(sample%w,sample/w));}
 }}
 assert_eq!(counts,expected,"{map}: [rose,green,timber,model backing]");
 let authored=m.authored_cells.clone();let finished=finish_surfaces(&mut m,map,&cells,&g,[0,0]);assert_eq!(finished,expected.iter().sum::<usize>());assert_eq!(m.authored_cells,authored);assert!(m.textured.indices.is_empty());
 println!("PASS {map}: {counts:?}; exact native FLOOR masks and complete successful-model backing");
}
#[test] fn gym_floor_external_pack_counts_and_collision_permissions() {
'''
    for name, counts in expected.items():
        m = pack['data']['maps'][name]
        ts = m['attributes']['tileset_name']
        w, h = m['attributes']['width']*4, m['attributes']['height']*4
        layout = pack['runtime_files'][f'data/tilesets/{ts}_metatiles.bin']
        collisions = pack['data']['tilesets'][ts]['collision']
        assert not m['script_block_changes'], 'New dynamic map state needs explicit review'
        code += f'check_native_floor("{name}","{ts}",{w},{h},&[\n'
        for y in range(h):
            for x in range(w):
                block = m['blocks'][y//4*(w//4)+x//4]
                tile = layout[block*16+(y%4)*4+x%4]
                is_floor = collisions[f'{block:02x}'][(y%4)//2*2+(x%4)//2] == 'FLOOR'
                code += f'({block},{tile},{str(is_floor).lower()}),'
        code += f'],{counts});\n'
    code += '}}\n'
    with tempfile.TemporaryDirectory(prefix='geothite-gym-floor-') as temporary:
        directory = Path(temporary)
        (directory/'check.rs').write_text(code)
        subprocess.run([args.rustc,'--edition=2024','--test',str(directory/'check.rs'),'-o',str(directory/'check')],check=True)
        subprocess.run([str(directory/'check'),'--nocapture'],check=True)
    print('PASS production floor/UV/profile/geometry/source guards; native app QA remains separate')

if __name__ == '__main__':
    main()
