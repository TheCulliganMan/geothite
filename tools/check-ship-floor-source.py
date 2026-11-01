#!/usr/bin/env python3
"""Check production ship floor selectors and coplanar materials without Cargo.
Reads only the external pack. All generated fixtures/binaries are temporary.
"""
import argparse, hashlib, json, os, shutil, struct, subprocess, tempfile
from pathlib import Path
class Cbor:
 def __init__(self,data):self.data=data;self.at=0
 def take(self,n):
  value=self.data[self.at:self.at+n];self.at+=n
  if len(value)!=n:raise ValueError('truncated CBOR')
  return value
 def read(self):
  head=self.take(1)[0];kind,arg=head>>5,head&31
  if arg<24:value=arg
  elif arg in [24,25,26,27]:value=int.from_bytes(self.take(1<<(arg-24)),'big')
  elif arg==31:
   if kind not in [4,5]:raise ValueError('unsupported indefinite CBOR')
   values=[]
   while self.data[self.at]!=255:values.append(self.read())
   self.at+=1;return values if kind==4 else dict(zip(values[::2],values[1::2]))
  else:raise ValueError('unsupported CBOR')
  if kind==0:return value
  if kind==1:return -1-value
  if kind==2:return self.take(value)
  if kind==3:return self.take(value).decode()
  if kind==4:return [self.read()for _ in range(value)]
  if kind==5:return {self.read():self.read()for _ in range(value)}
  if kind==6:return self.read()
  if kind==7:
   if arg in [20,21]:return arg==21
   if arg in [22,23]:return None
   if arg in [25,26,27]:return struct.unpack({25:'>e',26:'>f',27:'>d'}[arg],value.to_bytes(1<<(arg-24),'big'))[0]
  raise ValueError('unsupported CBOR')
def load(path):
 raw=path.read_bytes();assert raw[:12]==b'CRYSTALPACK\0' and len(raw)-22==int.from_bytes(raw[14:18],'big')
 decoder=Cbor(raw[22:]);pack=decoder.read();assert decoder.at==len(raw)-22;return pack
ROOT = Path(__file__).resolve().parents[1]
MAPS = {
    "FastShipB1F": [0, 420, 328, 48],
    "FastShipCabins_NNW_NNE_NE": [544, 0, 0, 0],
    "FastShipCabins_SE_SSE_CaptainsCabin": [588, 0, 0, 0],
    "FastShipCabins_SW_SSW_NW": [496, 0, 0, 0],
}
BODY = r'''
fn check(map: &str, cells: &[(u16,u8,u8,u16,u8)], expected: [usize;4]) {
 let mut counts = [0;4];
 for &(b,x,y,t,material) in cells {
  let actual = source::style(map,"lighthouse",b,x,y,t);
  let found = actual.map(|s|match s {ShipFloor::Cabin=>1,ShipFloor::Mess=>2,ShipFloor::Deck=>3,ShipFloor::Border=>4}).unwrap_or(0);
  assert_eq!(found,material,"native source {map} block={b:x} at {x},{y} tile={t:x}");
  if let Some(s)=actual {
   counts[(found-1)as usize]+=1;
   assert!(source::style(map,"custom-lighthouse",b,x,y,t).is_none());
   assert!(source::style(map,"lighthouse",0xff,x,y,t).is_none());
   assert!(source::style(map,"lighthouse",b,4,y,t).is_none());
   assert!(source::style(map,"lighthouse",b,x,4,t).is_none());
   assert!(source::style(map,"lighthouse",b,x,y,t^1).is_none());
   if s!=ShipFloor::Border {assert!(source::style(map,"lighthouse",b,x^1,y,t).is_none());}
  }
  for other in ["UnknownShip", "FastShipCustom", "FastShip1F", "OlivineLighthouse6F"] {
   assert!(source::style(other,"lighthouse",b,x,y,t).is_none());
  }
 }
 assert_eq!(counts,expected,"{map}");
 println!("PASS {map}: cabin/mess/deck/border {counts:?}");
}
#[derive(Default)] struct SurfaceMeshData {positions:Vec<[f32;3]>,normals:Vec<[f32;3]>,uvs:Vec<[f32;2]>,colors:Vec<[f32;4]>,indices:Vec<u32>}
fn append_quad(m:&mut SurfaceMeshData,p:[[f32;3];4],n:[f32;3],uv:[[f32;2];4],c:[f32;4]) {
 let start=m.positions.len()as u32; m.positions.extend(p);m.normals.extend([n;4]);m.uvs.extend(uv);m.colors.extend([c;4]);m.indices.extend([start,start+1,start+2,start,start+2,start+3]);
}
fn check_fixture_sources() {
 let map="FastShipCabins_NNW_NNE_NE";
 let cases = [
  ("dungeon:ship-barrel",0x35,2,2,0x48),("dungeon:ship-barrel",0x35,3,2,0x49),
  ("dungeon:ship-barrel",0x35,2,3,0x58),("dungeon:ship-barrel",0x35,3,3,0x59),
  ("dungeon:ship-barrel",0x2f,0,2,0x48),("dungeon:ship-barrel",0x2f,1,3,0x59),
  ("dungeon:ship-rack",0x2f,2,0,0x13),("dungeon:ship-rack",0x2f,3,3,0x3c),
  ("dungeon:ship-bunk",0x38,2,2,0x46),("dungeon:ship-bunk",0x38,3,3,0x57),
 ];
 for (label,b,x,y,t) in cases {
  assert!(source::fixture_source(map,label,"lighthouse",b,x,y,t));
  assert!(!source::fixture_source(map,label,"lighthouse",b,x,y,t^1));
  assert!(!source::fixture_source(map,label,"custom-lighthouse",b,x,y,t));
  assert!(!source::fixture_source(map,label,"lighthouse",0xff,x,y,t));
  assert!(!source::fixture_source(map,label,"lighthouse",b,4,y,t));
  assert!(!source::fixture_source(map,label,"lighthouse",b,x,4,t));
  assert!(!source::fixture_source(map,"custom/ship-prop","lighthouse",b,x,y,t));
  assert!(!source::fixture_source("OlivineLighthouse6F",label,"lighthouse",b,x,y,t));
 }
 for label in ["special:ship-door","special:ship-corridor-bulkhead","dungeon:ship-bunk"] {
  assert!(!source::fixture_source(map,label,"lighthouse",0x09,0,0,0x0e));
 }
 println!("PASS existing fixture backing vocabulary rejects changed art, maps, labels and navigation");
}
fn check_materials() {
 for s in [ShipFloor::Cabin,ShipFloor::Mess,ShipFloor::Deck,ShipFloor::Border] {
  for y in -9..14 {for x in -9..14 {
   let mut a=SurfaceMeshData::default();let mut b=SurfaceMeshData::default();
   ship_floor_cell(&mut a,[-4.,4.,2.,10.],3.75,s,[x,y]);ship_floor_cell(&mut b,[28.,36.,-22.,-14.],3.75,s,[x,y]);
   assert_eq!(a.indices,b.indices);assert_eq!(a.colors,b.colors);assert_eq!(a.normals,b.normals);
   assert!(a.indices.len()<=30);let mut area=0.;
   for q in a.positions.chunks_exact(4){assert!(q.iter().all(|p|p[1]==3.75&&(-4. ..=4.).contains(&p[0])&&(2. ..=10.).contains(&p[2])));area+=(q[3][0]-q[0][0])*(q[1][2]-q[0][2]);}
   assert!((area-64.0f32).abs()<0.0001);
   for (a,b) in a.positions.iter().zip(&b.positions){assert!((a[0]+32.-b[0]).abs()<0.0001&&(a[2]-24.-b[2]).abs()<0.0001&&a[1]==b[1]);}
   assert!(a.colors.iter().all(|c|c[3]==1.&&c.iter().all(|v|v.is_finite()&&(0. ..=1.).contains(v))));
  }}
 }
 println!("PASS four materials: 2,116 crop phases; exact area/height; <=10 triangles/cell");
}
'''
def main():
 ap=argparse.ArgumentParser(description=__doc__);ap.add_argument('--pack',type=Path,required=True);ap.add_argument('--rustc',default=os.getenv('RUSTC')or shutil.which('rustc'));args=ap.parse_args();assert args.rustc,'set RUSTC or pass --rustc'
 before=hashlib.sha256(args.pack.read_bytes()).hexdigest();pack=load(args.pack)
 module=ROOT/'crates/crystal-voxel-view/src/mesh/ship_floor_source.rs'
 finish=(ROOT/'crates/crystal-voxel-view/src/mesh/ship_floor.rs').read_text()
 interior=(ROOT/'crates/crystal-voxel-view/src/mesh/interior_surfaces.rs').read_text()
 material=interior[interior.index('fn material_quad('):interior.index('fn floor_cell(')]
 geometry=finish[finish.index('fn ship_floor_cell('):finish.index('#[cfg(test)]')]
 code=f'#[path="{module}"] mod source;\nuse source::Style as ShipFloor;\n'+BODY+material+geometry+'\nfn main(){check_materials();check_fixture_sources();\n'
 summary={}
 for name,expected in MAPS.items():
  m=pack['data']['maps'][name];assert not m['script_block_changes'];ts=m['attributes']['tileset_name'];assert ts=='lighthouse'
  w=m['attributes']['width']*4;h=m['attributes']['height']*4;layout=pack['runtime_files'][f'data/tilesets/{ts}_metatiles.bin'];collision=pack['data']['tilesets'][ts]['collision'];body=[];selected=set();counts=[0]*4
  for y in range(h):
   for x in range(w):
    b=m['blocks'][y//4*(w//4)+x//4];t=layout[b*16+y%4*4+x%4];c=collision[f'{b:02x}'][y%4//2*2+x%4//2]
    # Independent inventory: only native FLOOR collision and floor artwork.
    style=0
    if c=='FLOOR':
     if t in [0x0d,0x1d]:style=2 if name=='FastShipB1F'else 1
     elif name=='FastShipB1F'and t in [0x2e,0x2f]:style=3
     elif name=='FastShipB1F'and t in [0x1b,0x3f]:style=4
    if style:counts[style-1]+=1;selected.add((x,y))
    body.append(f'({b},{x%4},{y%4},{t},{style}),')
  assert counts==expected,(name,counts,expected)
  for e in m['events']['warps']:
   assert not any((e['x']*2+dx,e['y']*2+dy)in selected for dx in range(2)for dy in range(2)),(name,e)
  code+='check("'+name+'",&[\n'+'\n'.join(body)+f'\n],{expected});\n'
  summary[name]={'native_floor_cells':sum(counts),'materials':counts,'warps_preserved':len(m['events']['warps']),'objects_unchanged':len(m['objects']),'coordinate_events_unchanged':len(m['events']['coord_events']),'scripted_block_changes':len(m['script_block_changes'])}
 # The unchanged 1F existing timber deck, walls and twelve warp paths remain outside scope.
 first=pack['data']['maps']['FastShip1F'];assert not first['script_block_changes'];assert len(first['events']['warps'])==12
 code+='}\n'
 with tempfile.TemporaryDirectory(prefix='geothite-ship-floor-')as temp:
  p=Path(temp);(p/'check.rs').write_text(code)
  subprocess.run([args.rustc,'--edition=2024','-O','-A','dead_code',str(p/'check.rs'),'-o',str(p/'check')],check=True)
  subprocess.run([str(p/'check')],check=True)
 assert hashlib.sha256(args.pack.read_bytes()).hexdigest()==before
 print(json.dumps({'maps':summary,'native_floor_cells':sum(v['native_floor_cells']for v in summary.values()),'external_pack_unchanged':True,'first_floor_preserved':True},indent=2))
if __name__=='__main__':main()
