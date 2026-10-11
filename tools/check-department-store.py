#!/usr/bin/env python3
"""Check closed authored geometry, optional native source coverage, and editable source.

python3 tools/check-department-store.py [--pack content-packs/core-modular.browser.crystalpack]
The external pack is read in memory only. No source catalog or audit is exported.
"""
from pathlib import Path
from johto_art_sources import read_source
from model_asset_storage import read_model_json as model
from model_asset_storage import stored_model_size
import argparse,collections,importlib.util,json,math,re,struct,subprocess,tempfile
ROOT=Path(__file__).resolve().parent.parent
spec=importlib.util.spec_from_file_location('department_art',ROOT/'tools/build-department-store.py');kit=importlib.util.module_from_spec(spec);spec.loader.exec_module(kit)
def check_models():
 count=tris=stored=0
 for a in kit.assets():
  path=ROOT/'crates/crystal-voxel-view/models/department_store'/f'{a.name}.mesh.json';doc=model(path)
  assert doc==json.loads(json.dumps(kit.document(a))),'runtime must reproduce the editable geometry generator'
  assert doc['bounds']['min']==[0,0,0]
  for p in doc['primitives']:
   vs=[tuple(p['positions'][i:i+3])for i in range(0,len(p['positions']),3)];ns=[tuple(p['normals'][i:i+3])for i in range(0,len(p['normals']),3)]
   assert len(vs)==len(ns)and all(math.isfinite(v)for vec in vs+ns for v in vec)
   assert all(0<=v<=1 for v in p['base_color'])and p['base_color'][3]==1
   edges=collections.Counter();directions=collections.Counter();volume=0
   for i in range(0,len(p['indices']),3):
    ids=p['indices'][i:i+3];tri=[vs[j]for j in ids];n=kit.normal(*tri)
    for j in ids:assert sum(a*b for a,b in zip(n,ns[j]))>1-1e-8
    volume+=sum(a*b for a,b in zip(tri[0],kit.cross(tri[1],tri[2])))/6
    for a,b in zip(tri,tri[1:]+tri[:1]):assert a!=b;edges[tuple(sorted((a,b)))]+=1;directions[(a,b)]+=1
   assert volume>0,(p['name'],volume);assert set(edges.values())=={2},p['name'];assert all(directions[(b,a)]==1 for a,b in directions),p['name']
   if doc['name']=='closed_u_display':
    # Convex prism AABBs cannot enter the native upper 32x16 floor inset.
    x0,x1=min(v[0]for v in vs),max(v[0]for v in vs);z0,z1=min(v[2]for v in vs),max(v[2]for v in vs)
    assert x1<=16 or x0>=48 or z1<=16 or z0>=32,(p['name'],'covers floor inset')
   count+=1
  tris+=doc['triangle_count'];stored+=stored_model_size(path)
 # Lower centre is a closed supporting body, regardless of its dark lid finish.
 body=next(p for p in kit.u_display().parts if p[0]=='Closed lower display sealed body');vs=body[2]
 assert min(v[2]for v in vs)==32 and max(v[2]for v in vs)==63.5 and max(v[1]for v in vs)==7.6
 print(f'Geometry: 8 deterministic models, {count} closed positive-volume parts, {tris:,} triangles, {stored:,} stored bytes; upper floor inset and closed lower centre verified')
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
    p=[]
    while self.b[self.i]!=255:p.append(self.item())
    self.i+=1;return(b''if m==2 else'').join(p)
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
 b=path.read_bytes();assert b[:12]==b'CRYSTALPACK\0'and len(b)-22==int.from_bytes(b[14:18],'big');d=Decoder(b[22:]);p=d.item();assert d.i==len(b)-22;return p

def check_basement_crops(raw, guards):
 source=(ROOT/'crates/crystal-voxel-view/src/mesh/dungeon_extension.rs').read_text()
 functions='fn basement_wall_tile'+source.split('fn basement_wall_tile',1)[1].split('fn cerulean_bridge_detail',1)[0]
 data=','.join(f'({b},[{",".join(map(str,raw[b*16:b*16+16]))}])'for b in sorted(guards))
 harness='''#[derive(Clone)] struct VisualTileSource { tileset_id: std::sync::Arc<str>, metatile_id:u16, subtile_column:u8, subtile_row:u8, tile_index:u16 }
#[derive(Clone)] struct VisualTile { source:VisualTileSource }
struct GridGeometry { width:usize,height:usize }
'''+functions+f'fn main() {{ let native:[(u16,[u16;16]);{len(guards)}]=[{data}];'+'''
let mut crops=0;
for (block,drawing) in native {
 for y0 in 0..4 {for x0 in 0..4 {for height in 1..=4-y0 {for width in 1..=4-x0 {
  let g=GridGeometry{width,height};
  let mut t:Vec<_>=(0..height).flat_map(|y|(0..width).map(move|x|(x+x0,y+y0))).map(|(x,y)|VisualTile{source:VisualTileSource{tileset_id:"underground".into(),metatile_id:block,subtile_column:x as u8,subtile_row:y as u8,tile_index:drawing[y*4+x]}}).collect();
  assert!((0..t.len()).all(|i|basement_wall_block(&t.iter().collect::<Vec<_>>(),&g,i)),"native block {block:x}, crop {x0},{y0} {width}x{height}");
  t[0].source.tile_index^=1;
  assert!((0..t.len()).all(|i|!basement_wall_block(&t.iter().collect::<Vec<_>>(),&g,i)),"changed crop {block:x}");
  crops+=1;
 }}}}
}
println!("Production basement matcher: {crops} native crops accepted; every altered visible drawing rejected.");
}'''
 # Compile the actual production matcher. Native fixture identities exist only
 # in this temporary verification directory and are removed on completion.
 with tempfile.TemporaryDirectory(prefix='geothite-basement-crops-')as scratch:
  source_file=Path(scratch)/'check.rs';exe=Path(scratch)/'check';source_file.write_text(harness)
  subprocess.run(['rustc','--edition=2021',str(source_file),'-o',str(exe)],check=True)
  subprocess.run([str(exe)],check=True)

def check_pack(path):
 p=load_pack(path);raw=p['runtime_files']['data/tilesets/mart_metatiles.bin'];coll=p['data']['tilesets']['mart']['collision']
 source=(ROOT/'crates/crystal-voxel-view/src/mesh/department_store_bindings.rs').read_text()
 hashes=[int(v,16)for v in re.findall(r'fingerprint:\s*0x([0-9a-f]+)',source)]
 masks=[[int(v.strip(),0)for v in s.split(',')if v.strip()]for s in re.findall(r'rows:\s*&\[(.*?)\]',source,re.S)]
 assert len(hashes)==len(masks)==31
 def cells(m,rect):
  x0,y0,w,h=rect
  for y in range(y0,y0+h):
   for x in range(x0,x0+w):
    block=m['blocks'][y//4*8+x//4];yield x,y,block,x%4,y%4,raw[block*16+(y%4)*4+x%4]
 def identity(m,rect):
  h=0xcbf29ce484222325
  for x,y,b,sx,sy,t in cells(m,rect):
   for v in[b&255,b>>8,sx,sy,t&255,t>>8]:h=((h^v)*0x100000001b3)&((1<<64)-1)
  return h
 wall=u=legacy=stairs=lift=counter=0
 flights=0
 networks=re.findall(r'Network\s*\{(.*?)\n    \},',source,re.S)
 assert len(networks)==31
 networks=networks[:30]
 for region in['Celadon','Goldenrod']:
  for f in range(1,7):
   name=f'{region}DeptStore{f}F';m=p['data']['maps'][name];assert m['attributes']['width']==8 and m['attributes']['height']==4
   assert identity(m,(0,0,32,4))==hashes[f-1],name
   owned={(x,y)for y,r in enumerate(masks[f-1])for x in range(32)if r&(1<<x)}
   assert len(owned)==[56,28,52,36,52,36][f-1]
   for x,y,b,sx,sy,t in cells(m,(0,0,32,4)):
    collision=coll[f'{b:02x}'][sy//2*2+sx//2]
    if (x,y)in owned:assert collision=='WALL',(name,x,y,collision)
    rows=3 if b==5 else 2 if b in[1,2,3,8,9,10,23,24,25,47]else 0
    if sy<rows:
     legacy+=1
     if(x,y)not in owned:
      if b==5:lift+=1
      else:assert collision=='STAIRCASE';stairs+=1
   for warp in m['events']['warps']:
    assert not owned.intersection((warp['x']*2+dx,warp['y']*2+dy)for dx in range(2)for dy in range(2))
   # Every non-lift north warp is a complete modeled stair drawing. Read
   # identity and collision from the external pack; no fixture catalog ships.
   for j,network in enumerate(networks[7:18],7):
    nf=int(re.search(r'floor:\s*(\d+)',network)[1])
    if nf!=f:continue
    x,y=map(int,re.search(r'anchor:\s*\[(\d+),\s*(\d+)\]',network).groups())
    assert identity(m,(x,y,2,2))==hashes[j]
    assert masks[j]==[3,3]
    assert all(coll[f'{b:02x}'][sy//2*2+sx//2]=='STAIRCASE' for _,_,b,sx,sy,_ in cells(m,(x,y,2,2)))
    # Celadon's shared 6F layout draws the roof flight but has no roof warp.
    # Modeling its existing drawing must not create a gameplay connection.
    assert any(w['x']*2==x and w['y']*2==y for w in m['events']['warps']) or (name=='CeladonDeptStore6F' and x==26)
    flights+=1
   # Lift fronts own the portal and tread only. The call-panel housing has
   # separate wall ownership; the warp's original two-cell aperture stays open.
   network=networks[17+f]
   assert int(re.search(r'floor:\s*(\d+)',network)[1])==f
   assert identity(m,(4,0,4,4))==hashes[17+f]
   assert masks[17+f]==[3,3,15,0]
   assert all(coll[f'{b:02x}'][sy//2*2+sx//2]=='STAIRCASE' for _,_,b,sx,sy,_ in cells(m,(4,0,2,2)))
   assert all(coll[f'{b:02x}'][sy//2*2+sx//2]=='FLOOR' for _,_,b,sx,sy,_ in cells(m,(4,2,4,1)))
   assert any(w['x']==2 and w['y']==0 and 'ELEVATOR' in w['target_map'] for w in m['events']['warps'])
   for j,network in enumerate(networks[24:28],24):
    if int(re.search(r'floor:\s*(\d+)',network)[1])!=f:continue
    x,y=map(int,re.search(r'anchor:\s*\[(\d+),\s*(\d+)\]',network).groups())
    assert identity(m,(x,y,2,2))==hashes[j]
    assert masks[j]==[3,3]
    assert all(coll[f'{b:02x}'][sy//2*2+sx//2]=='COUNTER' for _,_,b,sx,sy,_ in cells(m,(x,y,2,2)))
   if f==2:
    assert identity(m,(24,8,8,8))==hashes[28]
    assert masks[28]==[255,255,3,3,3,3,255,255]
    for x,y,b,sx,sy,t in cells(m,(24,8,8,8)):
     collision=coll[f'{b:02x}'][sy//2*2+sx//2]
     if masks[28][y-8]&(1<<(x-24)):assert collision=='COUNTER'
     else:assert collision in['FLOOR','MART_SHELF']
   if f==4:
    assert identity(m,(24,8,8,4))==hashes[29]
    assert masks[29]==[255,255,3,3]
    for x,y,b,sx,sy,t in cells(m,(24,8,8,4)):
     collision=coll[f'{b:02x}'][sy//2*2+sx//2]
     assert collision==('COUNTER' if masks[29][y-8]&(1<<(x-24)) else 'FLOOR')
   wall+=len(owned)
   if f==5:
    assert identity(m,(12,8,8,8))==hashes[6]
    owned={(x+12,y+8)for y,r in enumerate(masks[6])for x in range(8)if r&(1<<x)}
    for x,y,b,sx,sy,t in cells(m,(12,8,8,8)):
     collision=coll[f'{b:02x}'][sy//2*2+sx//2]
     if(x,y)in owned:
      assert collision in['COUNTER','WALL']
      if collision=='COUNTER':counter+=1
      else:u+=1
     else:assert collision=='FLOOR'
 assert(legacy,wall,lift,stairs,u,counter)==(704,520,96,88,64,48)
 assert flights==22
 # Authenticate the only scoped runtime wall variant through its packed callback.
 cm=p['data']['maps']['CeladonDeptStore6F'];changes=cm['script_block_changes']
 assert any(c['source_script']=='CeladonDeptStore6FHideRooftopStairsCallback' and (c['x'],c['y'],c['block_id'])==(12,0,3) for c in changes)
 runtime=dict(cm);runtime['blocks']=cm['blocks'].copy();runtime['blocks'][6]=3
 assert identity(runtime,(0,0,32,4))==hashes[30]
 assert masks[30]==[0x3f00ffcf,0x3f00ffcf,0,0]
 assert sum(bin(r).count("1") for r in masks[30])==40
 for x,y,b,sx,sy,t in cells(runtime,(0,0,32,4)):
  if masks[30][y]&(1<<x):assert coll[f'{b:02x}'][sy//2*2+sx//2]=='WALL'
 print('Runtime source: exact Celadon 6F rooftop-stair callback wall owns 40 wall cells')
 # Basement geometry must consume every native wall course, not merely
 # render successfully through a fallback. Keep floor, crates and lift art out.
 bm=p['data']['maps']['GoldenrodDeptStoreB1F'];br=p['runtime_files']['data/tilesets/underground_metatiles.bin']
 bs=(ROOT/'crates/crystal-voxel-view/src/mesh/dungeon_extension.rs').read_text().split('fn basement_wall_block',1)[1].split('fn cerulean_bridge_detail',1)[0]
 guards={int(k,16):int(v,16) for k,v in re.findall(r'(0x[0-9a-f]+)\s*=>\s*(0x[0-9a-f]+)',bs)}
 boundary=0
 for b in bm['blocks']:
  drawing=br[b*16:b*16+16]
  wall_tiles={0x03,0x04,0x05,0x0c,0x0d,0x13,0x14,0x15}
  count=sum(t in wall_tiles for t in drawing)
  if not count:continue
  assert b in guards,('unmodeled basement wall block',b)
  h=0xcbf29ce484222325
  for y in range(4):
   for x in range(4):
    for v in[b&255,b>>8,x,y,drawing[y*4+x],0]:h=((h^v)*0x100000001b3)&((1<<64)-1)
  assert h==guards[b],('basement guard mismatch',b)
  boundary+=count
 assert boundary==492
 check_basement_crops(br,guards)
 assert [(i%bm['attributes']['width']*4,i//bm['attributes']['width']*4)for i,b in enumerate(bm['blocks'])if b==3]==[(32,4)]
 assert list(br[3*16:3*16+16])[2:4]==[0x2c,0x2d]
 assert list(br[3*16:3*16+16])[6:8]==[0x3c,0x3d]
 for x,y in [(18,8),(19,8),(20,8),(21,8),(18,9),(19,9),(20,9),(21,9)]:
  b=bm['blocks'][y//4*bm['attributes']['width']+x//4]
  assert b==(0x25 if x<20 else 0x26)
  assert br[b*16+y%4*4+x%4]==(0x02 if y==8 else 0x12)
 assert {(w['x'],w['y'])for w in bm['events']['warps']if w['target_map']=='GOLDENROD_DEPT_STORE_ELEVATOR'}=={(9,4),(10,4)}
 assert any(w['x']==17 and w['y']==2 and w['target_map']=='GOLDENROD_UNDERGROUND_WAREHOUSE'for w in bm['events']['warps'])
 print('Basement fixture source: separate 8-cell lift opening and 4-cell warehouse stair agree with their original warps')
 print(f'Basement source: all {boundary} wall-course cells match complete drawing guards; floor, crates and lift artwork excluded')
 print('Native source: 12 exact wall layouts, 520 architectural cells; 22 complete stair flights own 88 cells; 12 open lift fronts own 96 door/threshold cells. Eight complete service-counter ends own 32 cells. Two full 2F service loops own 80 cells with clerk floors and shelves preserved. Two 4F service returns own 40 cells with clerk floors preserved. Two complete 5F islands own 112 cells; all 16 native floor-inset cells preserved')
def check_source():
 manifest=ROOT/'art/johto/source/manifest.json';entry_file=ROOT/'source-manifest-entry.json'
 if manifest.exists():entry=next(s for s in json.loads(manifest.read_text())['sources']if s['file']=='department-store.blend')
 elif entry_file.exists():entry=json.loads(entry_file.read_text())
 else:raise AssertionError('editable Blender source has not been stored')
 payload=read_source(ROOT/'art/johto/source',entry)
 print(f'Editable Blender source: {entry["file"]}, {len(payload):,} compressed bytes')
if __name__=='__main__':
 p=argparse.ArgumentParser(description=__doc__);p.add_argument('--pack',type=Path);p.add_argument('--skip-source',action='store_true');args=p.parse_args();check_models()
 if args.pack:check_pack(args.pack)
 if not args.skip_source:check_source()
