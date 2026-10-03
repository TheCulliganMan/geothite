#!/usr/bin/env python3
"""Validate original geometry and production pure-source matcher against an
external Crystalpack. Pack source stays in memory and temporary Rust fixtures
are removed. No Cargo, native GUI, Blender or source catalog is required.
"""
from pathlib import Path
from johto_art_sources import read_source
from model_asset_storage import read_model_json as read_model
from model_asset_storage import stored_model_size
import argparse,collections,importlib.util,json,math,os,shutil,struct,subprocess,tempfile
ROOT=Path(__file__).resolve().parent.parent
spec=importlib.util.spec_from_file_location('facility_radio_art',ROOT/'tools/build-facility-radio.py');kit=importlib.util.module_from_spec(spec);spec.loader.exec_module(kit)
def check_models():
 tris=stored=parts=0
 for a in kit.assets():
  path=ROOT/'crates/crystal-voxel-view/models/facility_radio'/f'{a.name}.mesh.json';doc=read_model(path)
  assert doc==json.loads(json.dumps(kit.document(a))),'runtime/editable generator divergence'
  assert doc['bounds']['min']==[0,0,0]
  for p in doc['primitives']:
   vs=[tuple(p['positions'][i:i+3])for i in range(0,len(p['positions']),3)];ns=[tuple(p['normals'][i:i+3])for i in range(0,len(p['normals']),3)]
   assert len(vs)==len(ns)and all(math.isfinite(v)for vec in vs+ns for v in vec)
   assert all(0<=v<=1 for v in p['base_color'])and p['base_color'][3]==1
   edges=collections.Counter();directions=collections.Counter();volume=0
   for i in range(0,len(p['indices']),3):
    tri=[vs[j]for j in p['indices'][i:i+3]];n=kit.normal(*tri)
    for j in p['indices'][i:i+3]:assert sum(a*b for a,b in zip(n,ns[j]))>1-1e-8
    volume+=sum(a*b for a,b in zip(tri[0],kit.cross(tri[1],tri[2])))/6
    for a,b in zip(tri,tri[1:]+tri[:1]):assert a!=b;edges[tuple(sorted((a,b)))]+=1;directions[(a,b)]+=1
   assert volume>0,(p['name'],volume);assert set(edges.values())=={2},p['name'];assert all(directions[(b,a)]==1 for a,b in directions),p['name']
   # Every closed part fits wholly in its drawing-owned region. A connected
   # counter must retain its true operator opening even under the countertop.
   x0,x1=min(v[0]for v in vs),max(v[0]for v in vs);z0,z1=min(v[2]for v in vs),max(v[2]for v in vs)
   if doc['name']=='radio_reception_l':assert x1<=16 or z1<=16,(p['name'],'covers L staff opening')
   if doc['name']=='radio_reception_u':assert x1<=16 or x0>=144 or z1<=16 or (x0>=80 and x1<=96 and z1<=24),(p['name'],'covers U staff opening')
   parts+=1
  tris+=doc['triangle_count'];stored+=stored_model_size(path)
 print(f'PASS geometry: 6 deterministic models, {parts} closed positive-volume parts, {tris} triangles, {stored} compact bytes; native staff spaces empty')
class Decoder:
 def __init__(self,b):self.b=b;self.i=0
 def read(self,n):v=self.b[self.i:self.i+n];self.i+=n;return v
 def item(self):
  h=self.read(1)[0];m=h>>5;a=h&31
  if a<24:n=a
  elif a in(24,25,26,27):n=int.from_bytes(self.read(1<<(a-24)),'big')
  elif a==31:n=None
  else:raise ValueError('invalid CBOR')
  if m==0:return n
  if m==1:return -1-n
  if m in(2,3):
   if n is None:
    v=[]
    while self.b[self.i]!=255:v.append(self.item())
    self.i+=1;return(b''if m==2 else'').join(v)
   v=self.read(n);return v if m==2 else v.decode()
  if m in(4,5):
   if n is None:
    v=[]
    while self.b[self.i]!=255:v.append(self.item())
    self.i+=1
   else:v=[self.item()for _ in range(n*(2 if m==5 else 1))]
   return v if m==4 else dict(zip(v[0::2],v[1::2]))
  if m==6:return self.item()
  if m==7:
   if a in(20,21):return a==21
   if a in(22,23):return None
   if a in(25,26,27):return struct.unpack({25:'>e',26:'>f',27:'>d'}[a],n.to_bytes(1<<(a-24),'big'))[0]
  raise ValueError('unsupported CBOR')
def load_pack(path):
 raw=path.read_bytes();assert raw[:12]==b'CRYSTALPACK\0'and len(raw)-22==int.from_bytes(raw[14:18],'big');d=Decoder(raw[22:]);p=d.item();assert d.i==len(raw)-22;return p
BODY=r'''
fn counts(ps:&[source::Match],w:usize)->[usize;9]{let mut out=[0;9];for p in ps{out[p.kind()as usize]+=p.indices(w).count();}out}
fn check(map:&str,cells:Vec<Identity<'static>>,w:usize,h:usize,expected:[usize;9],protected:&[(usize,usize)],warp:&[(usize,usize)]){
 let empty=vec![false;cells.len()];let out=source::resolve(map,&cells,w,h,[0,0],&empty,&empty);
 assert_eq!(counts(&out,w),expected,"{map}");
 let mut owned=vec![false;cells.len()];
 for p in &out {assert!(p.coherent(&cells,w,h));for i in p.indices(w){assert!(!owned[i]);owned[i]=true;}}
 for &(x,y)in protected {
  // Complete walkable seat pictures share the actor's source quadrant. Their
  // mesh stays north of the native foot anchor, checked against real rigs by
  // check-room-furniture-bindings.py; source ownership does not add support.
  assert!(!owned[y*w+x] || out.iter().any(|p|p.kind()==Kind::RoundStool && p.indices(w).any(|i|i==y*w+x)),
   "standing actor space claimed by non-seat furniture in {map} at {x},{y}");
 }
 for &(x,y)in warp {for dy in 0..2{for dx in 0..2{assert!(!owned[(y+dy)*w+x+dx],"warp claimed in {map}");}}}
 assert!(source::resolve("UnrelatedAtlasRoom",&cells,w,h,[0,0],&empty,&empty).is_empty());
 for mode in 0..2 {
  let mut no_ground=cells.clone();for s in &mut no_ground {
   if(s.tileset=="facility"&&matches!(s.metatile,7|0x1b))||(s.tileset=="radio_tower"&&s.metatile==1){if mode==0{s.tile=0xffff;}else{s.metatile=0xffff;}}
  }
  assert!(source::resolve(map,&no_ground,w,h,[0,0],&empty,&empty).is_empty(),"native ground identity missing: {map}");
 }

 for p in &out{
  for gy in 0..p.height(){for gx in 0..p.width(){
   let i=(p.row+gy)*w+p.column+gx;
   // Alter any guard cell: art, block, atlas and local phase all matter.
   for mode in 0..6{
    let mut altered=cells.clone();let mut custom=empty.clone();
    match mode{0=>altered[i].tile^=1,1=>altered[i].metatile^=1,2=>altered[i].column^=1,3=>altered[i].tileset="other",4=>altered[i].row^=1,_=>custom[i]=true}
    if mode<5{assert!(!p.coherent(&altered,w,h));}
    let changed=source::resolve(map,&altered,w,h,[0,0],&empty,&custom);
    assert!(!changed.iter().any(|q|q.kind()==p.kind()&&q.column==p.column&&q.row==p.row),"guard/custom change survived {map}");
   }
  }}
  let i=p.indices(w).next().unwrap();let mut reserved=empty.clone();reserved[i]=true;
  assert!(!source::resolve(map,&cells,w,h,[0,0],&reserved,&empty).iter().any(|q|q.kind()==p.kind()&&q.column==p.column&&q.row==p.row));
  for &sample in&p.ground{
   let mut changed=cells.clone();changed[sample].tile^=1;assert!(!p.coherent(&changed,w,h));
  }
  for i in p.indices(w){let s=cells[p.ground_for(i%w,i/w)];
   if s.tileset=="facility"&&s.metatile==7{assert_eq!(s.tile,if(i%w+i/w)%2==0{1}else{0x26});}
   if map=="PowerPlant"&&p.kind()==Kind::InstrumentBank{assert_eq!((s.metatile,s.tile),(0x1b,0x1c));}
  }
  let cw=p.column+p.width()-1;
  if cw>0{let clipped:Vec<_>=(0..h).flat_map(|y|cells[y*w..y*w+cw].iter().copied()).collect();let ce=vec![false;clipped.len()];
   assert!(!source::resolve(map,&clipped,cw,h,[0,0],&ce,&ce).iter().any(|q|q.kind()==p.kind()&&q.column==p.column&&q.row==p.row));}
 }
 let pad=3;let pw=w+pad*2;let ph=h+pad*2;let blank=Identity{tileset:"none",metatile:0,column:0,row:0,tile:0};let mut padded=vec![blank;pw*ph];
 for y in 0..h{padded[(y+pad)*pw+pad..(y+pad)*pw+pad+w].copy_from_slice(&cells[y*w..(y+1)*w]);}
 let pe=vec![false;padded.len()];let moved=source::resolve(map,&padded,pw,ph,[-3,-3],&pe,&pe);assert_eq!(counts(&moved,pw),expected);
 for p in&moved{for i in p.indices(pw){let s=padded[p.ground_for(i%pw,i/pw)];if s.metatile==7&&s.tileset=="facility"{assert_eq!(s.tile,if(i%pw+i/pw-6)%2==0{1}else{0x26});}}}
 assert!(source::resolve(map,&cells,w+1,h,[0,0],&empty,&empty).is_empty());
 println!("PASS {map}: {:?}; complete art/atlas/phase/custom/crop/ground/actor/warp guards",expected);
}
'''
CASES={'MrPokemonsHouse':[16,12,0,0,0,0,0,0],'PowerPlant':[112,36,0,0,0,0,0,0],'RuinsOfAlphResearchCenter':[32,12,0,0,0,0,0,0],'TeamRocketBaseB3F':[16,60,0,0,0,0,0,0],'TrainerHouseB1F':[32,0,0,0,0,0,0,0],'LavRadioTower1F':[0,0,0,0,0,24,0,0],'RadioTower1F':[0,0,0,0,58,0,0,0],'RadioTower2F':[0]*8,'RadioTower3F':[0,0,16,24,0,0,0,12],'RadioTower4F':[0,0,24,0,0,0,0,0],'RadioTower5F':[0,0,8,0,0,0,8,0]}
for name,counts in CASES.items():counts.append({'RadioTower3F':8,'RadioTower4F':12,'RadioTower5F':4}.get(name,0))
def check_pack(path,rustc):
 pack=load_pack(path);module=ROOT/'crates/crystal-voxel-view/src/mesh/facility_radio_source.rs';code=f'#[path="{module}"]mod source;\nuse source::{{Identity,Kind}};\n'+BODY+'\nfn main(){\n'
 for name,expected in CASES.items():
  m=pack['data']['maps'][name];ts=m['attributes']['tileset_name'];w=m['attributes']['width']*4;h=m['attributes']['height']*4;raw=pack['runtime_files'][f'data/tilesets/{ts}_metatiles.bin']
  states=[('base',m['blocks'])]
  by_script=collections.defaultdict(list)
  for c in m['script_block_changes']:by_script[c['source_script']].append(c)
  for script,changes in by_script.items():
   b=m['blocks'][:]
   for c in changes:b[c['y']//2*(w//4)+c['x']//2]=c['block_id']
   states.append((script,b))
  for state,blocks in states:
   code+='check("'+name+'",vec![\n'
   for y in range(h):
    for x in range(w):
     b=blocks[y//4*(w//4)+x//4];tile=raw[b*16+y%4*4+x%4];code+=f'Identity{{tileset:"{ts}",metatile:{b},column:{x%4},row:{y%4},tile:{tile}}},\n'
   feet=[(o['x']*2+1,o['y']*2+1)for o in m['objects']];warp=[(o['x']*2,o['y']*2)for o in m['events']['warps']]
   code+=f'],{w},{h},{expected},&{feet!r},&{warp!r});\n'
 code+='}\n'
 with tempfile.TemporaryDirectory(prefix='geothite-facility-source-')as temp:
  tmp=Path(temp);(tmp/'check.rs').write_text(code);subprocess.run([rustc,'--edition=2024','-O','-A','dead_code',str(tmp/'check.rs'),'-o',str(tmp/'check')],check=True);subprocess.run([str(tmp/'check')],check=True)
 print('PASS native source: 11 maps plus every relevant authored block-change state; 526 claimed cells, with live floor/warp/operator source retained')
def check_source():
 manifest=ROOT/'art/johto/source/manifest.json';entries_file=ROOT/'source-manifest-entries.json'
 if manifest.exists():entries=[s for s in json.loads(manifest.read_text())['sources']if s['file']in['facility-workbench.blend','radio-desks.blend']]
 elif entries_file.exists():entries=json.loads(entries_file.read_text())
 else:raise AssertionError('editable Blender source has not been stored; use --skip-source only while staging')
 assert {e['file']for e in entries}=={'facility-workbench.blend','radio-desks.blend'}
 for entry in entries:
  payload=read_source(ROOT/'art/johto/source',entry)
  print(f"PASS editable source: {entry['file']}, {len(payload)} compressed bytes")
def main():
 parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('--pack',type=Path);parser.add_argument('--rustc',default=os.getenv('RUSTC')or shutil.which('rustc'));parser.add_argument('--skip-source',action='store_true');args=parser.parse_args();check_models()
 if args.pack:assert args.rustc,'pass --rustc';check_pack(args.pack,args.rustc)
 if not args.skip_source:check_source()
if __name__=='__main__':main()
