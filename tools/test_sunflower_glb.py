#!/usr/bin/env python3
"""Independent decoded seedling and sunflower support/leaf/ray skin checks."""
import math,unittest
from pathlib import Path
import sunflower_glb as author
from test_gengar_glb import Model,winding_alignment
from skin_glb_test_support import IDENTITY
ROOT=Path(__file__).resolve().parents[1]/'crates/crystal-voxel-view/models/battle_species'
class SunflowerSkinTests(unittest.TestCase):
 @classmethod
 def setUpClass(cls):cls.models={s:Model((ROOT/(s+'.glb')).read_bytes()) for s in author.SPECIES}
 def test_canonical_bind_geometry_and_budget(self):
  for s,m in self.models.items():
   blob=(ROOT/(s+'.glb')).read_bytes();decoded=author.decode_sunflower(s,blob)
   self.assertEqual(blob,author.export_sunflower(s,decoded));self.assertLess(len(blob),1024*1024)
   expected=(18,11694,16524,7,96) if s=='sunkern' else (30,9135,13696,25,195)
   self.assertEqual((len(m.parts),sum(map(len,m.vertices)),sum(len(p)//3 for p in m.indices),len(m.joints),len(m.doc['accessors'])),expected)
   for matrix in m.matrices():self.assertLess(max(abs(a-b) for a,b in zip(matrix,IDENTITY)),1e-7)
   for ids,weights in zip(m.ids,m.weights):
    for js,ws in zip(ids,weights):self.assertAlmostEqual(sum(ws),1.,places=7);self.assertTrue(all(j<len(m.joints) for j in js))
 def test_flower_petals_keep_broad_rounded_distal_caps(self):
  m=self.models['sunflora'];face=m.vertices[next(i for i,n in enumerate(m.names) if n.startswith('Face /'))]
  center_y=(min(v[1] for v in face)+max(v[1] for v in face))/2
  for p,n in enumerate(m.names):
   if not n.startswith('Petal /'):continue
   angle=int(n.rsplit(' ',1)[1])*math.tau/8;c=math.cos(angle);s=math.sin(angle)
   xy=[(x*c+(y-center_y)*s,-x*s+(y-center_y)*c) for x,y,z in m.vertices[p]]
   low=min(y for x,y in xy);high=max(y for x,y in xy);cap=[x for x,y in xy if y>low+.8*(high-low)]
   self.assertGreater((max(cap)-min(cap))/(max(x for x,y in xy)-min(x for x,y in xy)),.70,n)
 def test_pack_primary_materials_and_white_disk(self):
  for s,m in self.models.items():
   for p,n in enumerate(m.names):
    if s=='sunkern':rgb=(239,231,8) if n.startswith('Body /') else (107,148,66) if n.startswith(('Sprout /','Leaf /','Coat /')) else (0,0,0) if n.startswith(('Eye /','Mouth /')) else None
    else:rgb=(82,173,0) if n.startswith(('Body /','Leg /','Foot /','Leaf arm /')) else (214,181,24) if n.startswith(('Petal /','Bract /')) else (255,255,255) if n.startswith('Face /') else None
    if rgb is not None:
     expected=[(v/255)/12.92 if v/255<=.04045 else ((v/255+.055)/1.055)**2.4 for v in rgb]
     actual=m.doc['materials'][m.parts[p]['material']]['pbrMetallicRoughness']['baseColorFactor']
     for a,b in zip(actual,expected):self.assertAlmostEqual(a,b,places=7)
 def test_fixed_supports_floor_winding_and_strain(self):
  for s,m in self.models.items():
   moved=set()
   for clip in m.clips:
    for step in range(13):
     mats=m.matrices(clip,step/12)
     for p,n in enumerate(m.names):
      posed=m.posed(p,mats);normals=m.posed(p,mats,True)
      self.assertGreaterEqual(min(v[1] for v in posed),-1e-7,(s,clip,step,n))
      for i,(before,after) in enumerate(zip(m.vertices[p],posed)):
       if n.startswith('Foot /') or (s=='sunkern' and before[1]<.001):self.assertEqual(before,after)
      if max(math.dist(a,b) for a,b in zip(posed,m.vertices[p]))>.0005:moved.add(n.split(' /')[0])
      if step%6==0:
       for k in range(0,len(m.indices[p]),3):
        tri=m.indices[p][k:k+3];cosine,area=winding_alignment(posed,normals,tri)
        self.assertGreater(cosine,0.,(s,clip,step,n,k));self.assertGreater(area,1e-12)
        for a,b in zip(tri,tri[1:]+tri[:1]):
         ratio=math.dist(posed[a],posed[b])/math.dist(m.vertices[p][a],m.vertices[p][b])
         self.assertTrue(.80<ratio<1.20,(s,clip,step,n,ratio))
     if step in (0,12):
      for matrix in mats:self.assertLess(max(abs(a-b) for a,b in zip(matrix,IDENTITY)),1e-7)
   self.assertTrue(({'Body','Sprout','Leaf','Leaf vein'} if s=='sunkern' else {'Body','Face','Leaf arm','Leaf vein','Petal','Bract'})<=moved,(s,moved))
 def test_face_and_radial_rays_have_coherent_rigid_owners(self):
  m=self.models['sunflora']
  for p,n in enumerate(m.names):
   owner=9+int(n.rsplit(' ',1)[1]) if n.startswith('Petal /') else 17+int(n.rsplit(' ',1)[1]) if n.startswith('Bract /') else 4 if n.startswith(('Face /','Eye /','Mouth /','Tongue /')) else None
   if owner is not None:self.assertTrue(all(ids==[owner,0,0,0] for ids in m.ids[p]),n)
 def test_delayed_crown_response_rotation_only_and_root_stationary(self):
  for s,m in self.models.items():
   for a in m.doc['animations']:
    self.assertEqual(a['extras']['loopSuggested'],a['name'].endswith('.idle'));self.assertEqual(a['extras']['cueRelative'],not a['name'].endswith('.idle'))
    self.assertEqual({c['target']['path'] for c in a['channels']},{'rotation'});self.assertNotIn(0,{c['target']['node'] for c in a['channels']})
  m=self.models['sunflora'];body=[m.rotations('attack',i/100)[1][0] for i in range(101)];disk=[m.rotations('attack',i/100)[4][0] for i in range(101)]
  self.assertGreater(disk.index(max(disk)),body.index(max(body)))
if __name__=='__main__':unittest.main()
