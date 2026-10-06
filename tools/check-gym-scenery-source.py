#!/usr/bin/env python3
"""Compile and exercise the production Gym matcher with an external Crystalpack.
No Cargo, source images, serialized fixture catalogs, or game files are written
into the repository. Temporary source-identity fixtures are removed on exit.
Usage: python tools/check-gym-scenery-source.py --pack content-packs/core-modular.browser.crystalpack
"""
import argparse,os,shutil,struct,subprocess,tempfile
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
ROOT=Path(__file__).resolve().parents[1]
TEST_BODY=r'''
fn check(map: &str, cells: Vec<Identity<'static>>, w: usize,h: usize, expected: [usize;5]) {
 let blocked=vec![false;cells.len()]; let out=source::resolve(map,&cells,w,h,[0,0],&blocked);
 let counts=|items:&[source::Match]| { let mut c=[0;5];for p in items {let k=match p.kind{Kind::PlanterLow=>0,Kind::PlanterRound=>1,Kind::CentralTree=>2,Kind::Hedge=>3,Kind::Maze=>4};c[k]+=p.indices(w).count();}c};
 assert_eq!(counts(&out),expected,"native source coverage: {map}");
 let mut ownership=vec![false;cells.len()];for p in &out {for i in p.indices(w){assert!(!ownership[i]);ownership[i]=true;}}
 assert!(source::resolve("UnrelatedSharedAtlasMap",&cells,w,h,[0,0],&blocked).is_empty());
 let mut missing_ground=cells.clone();for s in &mut missing_ground{if source::valid_ground(map,s){s.tile=0xffff;}}
 assert!(source::resolve(map,&missing_ground,w,h,[0,0],&blocked).is_empty());
 // Exercise every distinct native source drawing and every independent maze.
 let mut tested=std::collections::HashSet::new();
 for p in &out {
  assert!(p.coherent(&cells,w,h));
  let i=p.indices(w).next().unwrap(); let s=cells[i];
  if p.kind!=Kind::Maze && !tested.insert((s.metatile,s.column,s.row,s.tile)){continue;}
  for mode in 0..5 {
   let mut changed=cells.clone();let mut override_cells=blocked.clone();
   match mode {0=>changed[i].tile^=1,1=>changed[i].column^=1,2=>changed[i].metatile^=1,
    3=>changed[i].tileset="unrelated_atlas",_=>override_cells[i]=true}
   if mode<4 {assert!(!p.coherent(&changed,w,h),"stale resolved source accepted");}
   let changed_out=source::resolve(map,&changed,w,h,[0,0],&override_cells);
   assert!(!changed_out.iter().any(|q|q.indices(w).any(|j|j==i)),"changed or custom-owned drawing survived: {map} mode {mode}");
   if p.kind==Kind::Maze {assert!(changed_out.iter().all(|q| !q.indices(w).any(|j|p.indices(w).any(|k|j==k))),"partial maze network survived");}
  }
  if p.kind==Kind::Maze {
   let gap=(0..p.height).flat_map(|y|(0..p.width).map(move|x|(x,y))).find(|&(x,y)|!p.owns(x,y)).unwrap();
   let gap_i=(p.row+gap.1)*w+p.column+gap.0; let mut changed=cells.clone();changed[gap_i].tile^=1;
   let after=source::resolve(map,&changed,w,h,[0,0],&blocked);
   assert!(!after.iter().any(|q|q.indices(w).any(|j|j==i)),"altered native opening was ignored");
   let mut gap_override=blocked.clone();gap_override[gap_i]=true;
   let after=source::resolve(map,&cells,w,h,[0,0],&gap_override);
   assert!(!after.iter().any(|q|q.indices(w).any(|j|j==i)),"custom opening override was ignored");
   for y in 0..p.height {for x in 0..p.width {if p.owns(x,y){
    let mask=p.open_mask(x,y);
    if p.owns(x+1,y){assert_eq!(mask&2,0);assert_eq!(p.open_mask(x+1,y)&8,0);}
    if p.owns(x,y+1){assert_eq!(mask&4,0);assert_eq!(p.open_mask(x,y+1)&1,0);}
   }}}
  }
 }
 // Padding and scrolling move canvas indices, never source-map identity.
 let pad=3;let pw=w+2*pad;let ph=h+2*pad;let blank=Identity{tileset:"none",metatile:0,column:0,row:0,tile:0};let mut padded=vec![blank;pw*ph];
 for y in 0..h{padded[(y+pad)*pw+pad..(y+pad)*pw+pad+w].copy_from_slice(&cells[y*w..(y+1)*w]);}
 let padded_out=source::resolve(map,&padded,pw,ph,[-(pad as i32),-(pad as i32)],&vec![false;padded.len()]);
 assert_eq!(padded_out.iter().map(|p|p.indices(pw).count()).sum::<usize>(),expected.iter().sum::<usize>());
 for p in &out {
  // Clip one rightmost guard column from this object/network while retaining
  // unrelated full drawings. The clipped target must disappear atomically.
  let cw=p.column+p.width-1;if cw==0{continue;}let cropped:Vec<_>=(0..h).flat_map(|y|cells[y*w..y*w+cw].iter().copied()).collect();
  let crop_out=source::resolve(map,&cropped,cw,h,[0,0],&vec![false;cropped.len()]);
  assert!(!crop_out.iter().any(|q|q.kind==p.kind&&q.column==p.column&&q.row==p.row),"clipped drawing survived");
 }
 assert!(source::resolve(map,&cells,w+1,h,[0,0],&blocked).is_empty());
 println!("PASS {map}: {:?}; whole drawing, custom, atlas, phase, clipping, padding and native-gap guards",expected);
}
'''
def main():
 parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('--pack',type=Path,required=True);parser.add_argument('--rustc',default=os.getenv('RUSTC')or shutil.which('rustc'));args=parser.parse_args();assert args.rustc,'set RUSTC or pass --rustc'
 pack=load(args.pack);module=ROOT/'crates/crystal-voxel-view/src/mesh/gym_scenery_source.rs'
 code=f'#[path={str(module)!r}] mod source;\n'.replace("'",'"')+'use source::{Identity,Kind};\n'+TEST_BODY+'\nfn main(){\n'
 cases={'AzaleaGym':[104,112,9,0,0],'GoldenrodGym':[316,188,0,0,0],'CeladonGym':[0,0,0,168,0],'ViridianGym':[0,0,0,0,356]}
 for name,expected in cases.items():
  m=pack['data']['maps'][name];ts=m['attributes']['tileset_name'];w=m['attributes']['width']*4;h=m['attributes']['height']*4;layout=pack['runtime_files'][f'data/tilesets/{ts}_metatiles.bin'];assert not m['script_block_changes'],'new native dynamic state needs explicit review'
  code+='check("'+name+'",vec![\n'
  for y in range(h):
   for x in range(w):
    b=m['blocks'][(y//4)*(w//4)+x//4];tile=layout[b*16+(y%4)*4+x%4];code+=f'Identity{{tileset:"{ts}",metatile:{b},column:{x%4},row:{y%4},tile:{tile}}},\n'
  code+=f'],{w},{h},{expected});\n'
 code+='}\n'
 with tempfile.TemporaryDirectory(prefix='geothite-gym-source-')as temporary:
  path=Path(temporary);(path/'check.rs').write_text(code);subprocess.run([args.rustc,'--edition=2024','-O','-A','dead_code',str(path/'check.rs'),'-o',str(path/'check')],check=True);subprocess.run([str(path/'check')],check=True)
if __name__=='__main__':main()
