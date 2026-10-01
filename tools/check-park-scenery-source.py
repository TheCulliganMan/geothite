#!/usr/bin/env python3
"""Compile and exercise the production park matcher with an external Crystalpack.
No Cargo, source images, serialized fixture catalogs, or game files are written
into the repository. Temporary source-identity fixtures are removed on exit.
Usage: python tools/check-park-scenery-source.py --pack content-packs/core-modular.browser.crystalpack
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
TEST_BODY=r"""
fn check(map: &str,cells:Vec<Identity<'static>>,w:usize,h:usize,expected:[usize;3]) {
 let blocked=vec![false;cells.len()];let out=source::resolve(map,&cells,w,h,[0,0],&blocked);
 let count=|items:&[source::Match]|{let mut counts=[0;3];for p in items{counts[p.kind.asset_index()]+=p.indices(w).count();}counts};
 assert_eq!(count(&out),expected,"native source counts {map}");
 assert!(source::resolve("UnknownParkAtlasMap",&cells,w,h,[0,0],&blocked).is_empty());
 assert!(source::resolve(map,&cells,w,h,[1,0],&blocked).is_empty());
 assert!(source::resolve(map,&cells,w+1,h,[0,0],&blocked).is_empty());
 for p in &out {
  assert!(p.coherent(&cells,w,h));
  for i in p.guard_indices(w) {
   for mode in 0..5 {
    let mut changed=cells.clone();let mut custom=blocked.clone();
    match mode {0=>changed[i].tile^=1,1=>changed[i].column^=1,2=>changed[i].metatile^=1,3=>changed[i].tileset="unrelated_atlas",_=>custom[i]=true}
    if mode<4 {assert!(!p.coherent(&changed,w,h));}
    assert!(!source::resolve(map,&changed,w,h,[0,0],&custom).iter().any(|q|q.column==p.column&&q.row==p.row&&q.kind==p.kind),"partial/stale fixture accepted {map} {i} {mode}");
   }
  }
  // Clipping the guard backing removes the whole prop, even if its visible
  // sculpture cells all survive. Ordinary ground never receives ownership.
  let cw=p.column+p.guard_width()-1;
  let crop:Vec<_>=(0..h).flat_map(|y|cells[y*w..y*w+cw].iter().copied()).collect();
  assert!(!source::resolve(map,&crop,cw,h,[0,0],&vec![false;crop.len()]).iter().any(|q|q.column==p.column&&q.row==p.row));
  assert_eq!(p.indices(w).count(),match p.kind{Kind::LitterBin=>4,Kind::PedestalFountain=>6,Kind::PondBasin=>4});
  for (local,_)in p.indices(w).enumerate(){assert!(p.guard_indices(w).any(|i|i==p.ground(local,w)));}
 }
 let pad=3;let pw=w+6;let ph=h+6;let blank=Identity{tileset:"none",metatile:0,column:0,row:0,tile:0};let mut padded=vec![blank;pw*ph];
 for y in 0..h{padded[(y+pad)*pw+pad..(y+pad)*pw+pad+w].copy_from_slice(&cells[y*w..(y+1)*w]);}
 let after=source::resolve(map,&padded,pw,ph,[-3,-3],&vec![false;padded.len()]);
 assert_eq!(after.iter().map(|p|p.indices(pw).count()).sum::<usize>(),expected.iter().sum::<usize>());
 println!("PASS {map}: {:?}; whole art, phase, backing, custom, clipping, atlas, scrolling guards",expected);
}
"""
def main():
 parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('--pack',type=Path,required=True);parser.add_argument('--rustc',default=os.getenv('RUSTC')or shutil.which('rustc'));args=parser.parse_args();assert args.rustc,'set RUSTC or pass --rustc'
 pack=load(args.pack);module=ROOT/'crates/crystal-voxel-view/src/mesh/park_scenery_source.rs'
 code=f'#[path={str(module)!r}] mod source;\n'.replace("'",'"')+'use source::{Identity,Kind};\n'+TEST_BODY+'\nfn main(){\n'
 cases={'NationalPark':[8,6,4],'NationalParkBugContest':[8,6,4],'SafariZoneBeta':[4,12,0]}
 for name,expected in cases.items():
  m=pack['data']['maps'][name];ts=m['attributes']['tileset_name'];w=m['attributes']['width']*4;h=m['attributes']['height']*4;layout=pack['runtime_files'][f'data/tilesets/{ts}_metatiles.bin'];assert not m['script_block_changes'],'new native dynamic state needs review'
  code+='check("'+name+'",vec![\n'
  for y in range(h):
   for x in range(w):
    b=m['blocks'][(y//4)*(w//4)+x//4];tile=layout[b*16+(y%4)*4+x%4];code+=f'Identity{{tileset:"{ts}",metatile:{b},column:{x%4},row:{y%4},tile:{tile}}},\n'
  code+=f'],{w},{h},{expected});\n'
 code+='}\n'
 with tempfile.TemporaryDirectory(prefix='geothite-park-source-')as temporary:
  path=Path(temporary);(path/'check.rs').write_text(code);subprocess.run([args.rustc,'--edition=2024','-O','-A','dead_code',str(path/'check.rs'),'-o',str(path/'check')],check=True);subprocess.run([str(path/'check')],check=True)
if __name__=='__main__':main()
