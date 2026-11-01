#!/usr/bin/env python3
"""Read-only exact pack source checks; temp fixtures only, no Cargo."""
import argparse,os,shutil,struct,subprocess,tempfile,json
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
BODY=r'''
fn check(map:&str,c:Vec<Identity<'static>>,w:usize,h:usize,expected:[usize;3]) {
 let empty=vec![false;c.len()];let out=source::resolve(map,&c,w,h,[0,0],&empty);
 assert_eq!(out.len(),expected[0]);assert_eq!(out.iter().map(|p|p.indices(w).count()).sum::<usize>(),expected[1]);
 let objects=out.iter().map(|p|(0..p.height).map(|y|(0..p.width).filter(|&x|p.object_label(x,y).is_some()).count()).sum::<usize>()).sum::<usize>();assert_eq!(objects,expected[2]);
 let mut owned=vec![false;c.len()];for p in &out {for i in p.indices(w){assert!(!owned[i]);owned[i]=true;}}
 assert!(source::resolve("AnotherSharedAtlasRoom",&c,w,h,[0,0],&empty).is_empty());
 let mut no_ground=c.clone();for s in &mut no_ground {if source::valid_ground(map,s){s.tile=0xffff;}}
 assert!(source::resolve(map,&no_ground,w,h,[0,0],&empty).is_empty());
 let ground_reserved:Vec<_>=c.iter().map(|s|source::valid_ground(map,s)).collect();
 assert!(source::resolve(map,&c,w,h,[0,0],&ground_reserved).is_empty());
 for p in &out {
  assert!(p.coherent(&c,w,h));
  // Every source cell in each full guard, including negative passage floor,
  // rejects stale identities and custom overrides atomically.
  for i in p.guard_indices(w) {
   for mode in 0..6 {let mut bad=c.clone();let mut blocked=empty.clone();match mode {0=>bad[i].tile^=1,1=>bad[i].metatile^=1,2=>bad[i].column^=1,3=>bad[i].row^=1,4=>bad[i].tileset="custom",_=>blocked[i]=true}
    if mode<5 {assert!(!p.coherent(&bad,w,h));}
    let after=source::resolve(map,&bad,w,h,[0,0],&blocked);
    assert!(!after.iter().any(|q|q.column==p.column&&q.row==p.row));
   }
  }
  let cw=p.column+p.width-1;let cropped:Vec<_>=(0..h).flat_map(|y|c[y*w..y*w+cw].iter().copied()).collect();
  assert!(!source::resolve(map,&cropped,cw,h,[0,0],&vec![false;cropped.len()]).iter().any(|q|q.column==p.column&&q.row==p.row));
 }
 let pad=3;let pw=w+pad*2;let ph=h+pad*2;let blank=Identity{tileset:"none",metatile:0,column:0,row:0,tile:0};let mut padded=vec![blank;pw*ph];
 for y in 0..h{padded[(y+pad)*pw+pad..(y+pad)*pw+pad+w].copy_from_slice(&c[y*w..(y+1)*w]);}
 assert_eq!(source::resolve(map,&padded,pw,ph,[-3,-3],&vec![false;padded.len()]).iter().map(|p|p.indices(pw).count()).sum::<usize>(),expected[1]);
 assert!(source::resolve(map,&c,w,h,[1,0],&empty).is_empty());
 assert!(source::resolve(map,&c,w+1,h,[0,0],&empty).is_empty());
 println!("PASS {map}: {} placements, {} owned cells, {} object cells",expected[0],expected[1],expected[2]);
}
'''
def main():
 ap=argparse.ArgumentParser(description=__doc__);ap.add_argument('--pack',type=Path,required=True);ap.add_argument('--rustc',default=os.getenv('RUSTC')or shutil.which('rustc'));args=ap.parse_args();assert args.rustc,'pass --rustc'
 pack=load(args.pack);module=ROOT/'crates/crystal-voxel-view/src/mesh/traditional_room_source.rs'
 code=f'#[path="{module}"] mod source;\nuse source::Identity;\n'+BODY+'\nfn main(){\n'
 counts={};total_tatami=0;edge_tatami=0
 for name,expected in {'WiseTriosRoom':[5,40,40],'Route39Barn':[3,24,24],'DanceTheater':[1,240,72]}.items():
  m=pack['data']['maps'][name];assert not m['script_block_changes'];ts=m['attributes']['tileset_name'];w=m['attributes']['width']*4;h=m['attributes']['height']*4;layout=pack['runtime_files'][f'data/tilesets/{ts}_metatiles.bin'];collision=pack['data']['tilesets'][ts]['collision'];owned=[];body=[]
  for y in range(h):
   for x in range(w):
    b=m['blocks'][y//4*(w//4)+x//4];t=layout[b*16+y%4*4+x%4];body.append(f'Identity{{tileset:"{ts}",metatile:{b},column:{x%4},row:{y%4},tile:{t}}},')
    selected=(name=='WiseTriosRoom'and ((b in [0x28,0x37]and y%4<2)or(b in [0x37,0x38]and x%4>=2 and y%4>=2)))or(name=='Route39Barn'and b==0x26 and y%4>=2)or(name=='DanceTheater'and y<10)
    if selected:owned.append((x,y,collision[f'{b:02x}'][y%4//2*2+x%4//2]))
  assert len(owned)==expected[1];own={(x,y)for x,y,_ in owned}
  if name!='DanceTheater':assert {c for _,_,c in owned}=={'WALL'}
  else:
   assert all(c=='WALL'for x,y,c in owned if y<2)
   assert all((c=='FLOOR')==(x in [2,3,20,21])for x,y,c in owned if y==9)
  for e in m['events']['warps']:assert not any((e['x']*2+dx,e['y']*2+dy)in own for dx in range(2)for dy in range(2))
  for o in m['objects']:
   for dy in range(2):
    for dx in range(2):
     x=o['x']*2+dx;y=o['y']*2+dy
     if name=='DanceTheater':assert not((x,y)in own and(y<2 or y>=9))
     else:assert(x,y)not in own
  code+='check("'+name+'",vec![\n'+'\n'.join(body)+f'\n],{w},{h},{expected});\n';counts[name]={'placements':expected[0],'claimed_cells':len(owned),'object_cells':expected[2],'floor_finish_cells':len(owned)-expected[2]}
 for name,n in [('KurtsHouse',112),('DanceTheater',64)]:
  m=pack['data']['maps'][name];ts=m['attributes']['tileset_name'];layout=pack['runtime_files'][f'data/tilesets/{ts}_metatiles.bin'];blocks=m['blocks'];assert blocks.count(4)*16==n
  assert pack['data']['tilesets'][ts]['collision']['04']==['FLOOR']*4
  assert list(layout[4*16:5*16])==[0x44,0x45,0x45,0x46,0x54,0x55,0x55,0x56,0x45,0x46,0x44,0x45,0x55,0x56,0x54,0x55]
  total_tatami+=n;edge_tatami+=blocks.count(4)*4
 assert total_tatami==176 and edge_tatami==44
 code+='}\n'
 with tempfile.TemporaryDirectory(prefix='geothite-traditional-source-')as temp:
  path=Path(temp);(path/'check.rs').write_text(code);subprocess.run([args.rustc,'--edition=2024','-O','-A','dead_code',str(path/'check.rs'),'-o',str(path/'check')],check=True);subprocess.run([str(path/'check')],check=True)
 print(json.dumps({'maps':counts,'tatami_floor_cells':total_tatami,'tatami_new_edge_finishes':edge_tatami,'stage_support_pixels':8,'source_actor_warp_collision_checks':True},indent=2))
if __name__=='__main__':main()
