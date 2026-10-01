#!/usr/bin/env python3
"""Check closed authored geometry, optional native source coverage, and editable source.

python3 tools/check-department-store.py [--pack content-packs/core-modular.browser.crystalpack]
The external pack is read in memory only. No source catalog or audit is exported.
"""
from pathlib import Path
import argparse,base64,collections,gzip,hashlib,importlib.util,json,math,re,struct
ROOT=Path(__file__).resolve().parent.parent
spec=importlib.util.spec_from_file_location('department_art',ROOT/'tools/build-department-store.py');kit=importlib.util.module_from_spec(spec);spec.loader.exec_module(kit)
def sha(b):return hashlib.sha256(b).hexdigest()
def model(path):
 v=json.loads(path.read_bytes());assert v['storage']=='geothite-model-gzip-v1';raw=gzip.decompress(base64.b64decode(v['data'],validate=True));assert len(raw)==v['bytes']and sha(raw)==v['sha256'];return json.loads(raw)
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
  tris+=doc['triangle_count'];stored+=path.stat().st_size
 # Lower centre is a closed supporting body, regardless of its dark lid finish.
 body=next(p for p in kit.u_display().parts if p[0]=='Closed lower display sealed body');vs=body[2]
 assert min(v[2]for v in vs)==32 and max(v[2]for v in vs)==63.5 and max(v[1]for v in vs)==7.6
 print(f'Geometry: 8 deterministic models, {count} closed positive-volume parts, {tris:,} triangles, {stored:,} compressed-text bytes; upper floor inset and closed lower centre verified')
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

def check_pack(path):
 p=load_pack(path);raw=p['runtime_files']['data/tilesets/mart_metatiles.bin'];coll=p['data']['tilesets']['mart']['collision']
 source=(ROOT/'crates/crystal-voxel-view/src/mesh/department_store_bindings.rs').read_text()
 hashes=[int(v,16)for v in re.findall(r'fingerprint:\s*0x([0-9a-f]+)',source)]
 masks=[[int(v.strip(),0)for v in s.split(',')if v.strip()]for s in re.findall(r'rows:\s*&\[(.*?)\]',source,re.S)]
 assert len(hashes)==len(masks)==7
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
 print('Native source: 12 exact wall layouts, 520 architectural cells; 184 retained existing-renderer cells = 96 lift door/threshold + 88 stair cells. Two complete 5F islands own 112 cells = 64 closed display + 48 counter; all 16 native floor-inset cells preserved')
def check_source():
 manifest=ROOT/'art/johto/source/manifest.json';entry_file=ROOT/'source-manifest-entry.json'
 if manifest.exists():entry=next(s for s in json.loads(manifest.read_text())['sources']if s['file']=='department-store.blend')
 elif entry_file.exists():entry=json.loads(entry_file.read_text())
 else:raise AssertionError('editable Blender source has not been packed')
 payload=b''.join(base64.b64decode((ROOT/'art/johto/source'/p['file']).read_bytes(),validate=True)for p in entry['chunks'])
 assert len(payload)==entry['bytes']and sha(payload)==entry['sha256'];raw=gzip.decompress(payload)
 assert raw.startswith(b'BLENDER')and len(raw)==entry['uncompressed_bytes']and sha(raw)==entry['uncompressed_sha256']
 print(f'Editable Blender source: {len(entry["chunks"])} verified chunks, {len(payload):,} compressed bytes')
if __name__=='__main__':
 p=argparse.ArgumentParser(description=__doc__);p.add_argument('--pack',type=Path);p.add_argument('--skip-source',action='store_true');args=p.parse_args();check_models()
 if args.pack:check_pack(args.pack)
 if not args.skip_source:check_source()
