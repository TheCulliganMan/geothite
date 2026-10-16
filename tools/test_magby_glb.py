#!/usr/bin/env python3
"""Decoded Magby skin preserves open bill, fitted layers and planted claw supports."""
import math,unittest
from pathlib import Path
import magby_glb as author
import test_gengar_glb as skin
import test_donphan_sculpt as joins
from skin_glb_test_support import IDENTITY
ROOT=Path(__file__).resolve().parents[1]/'crates/crystal-voxel-view/models/battle_species/magby.glb'
class MagbySkinTests(unittest.TestCase):
 @classmethod
 def setUpClass(cls):cls.blob=ROOT.read_bytes();cls.m=skin.Model(cls.blob)
 def test_canonical_anatomy_bind_and_budgets(self):
  m=self.m;self.assertEqual(self.blob,author.export_magby(author.decode_magby(self.blob)))
  self.assertEqual((len(m.parts),sum(map(len,m.vertices)),sum(len(p)//3 for p in m.indices),len(m.joints)),(38,19764,35032,11))
  self.assertLess(len(self.blob),1024*1024);self.assertLessEqual(len(m.doc['accessors']),256)
  for matrix in m.matrices():self.assertLess(max(abs(a-b) for a,b in zip(matrix,IDENTITY)),1e-7)
 def test_local_normal_palette(self):
  m=self.m
  for p,n in enumerate(m.names):
   rgb=(0,0,0) if n.startswith(('Collar /','Eye socket /','Pupil /','Mouth back /')) else (255,255,255) if n.startswith(('Eye /','Glint /','Hand claw /','Toe claw /')) else (255,148,74) if n.startswith(('Bill /','Belly /')) else (189,16,90)
   color=m.doc['materials'][m.parts[p]['material']]['pbrMetallicRoughness']['baseColorFactor']
   for v,a in zip(rgb,color):
    x=v/255;expected=x/12.92 if x<=.04045 else ((x+.055)/1.055)**2.4;self.assertAlmostEqual(a,expected,places=7)
 def test_coherent_crown_bill_fitted_gaze_and_claw_owners(self):
  m=self.m
  for p,n in enumerate(m.names):
   owner=0 if n.startswith(('Foot /','Toe claw /')) else 3 if n.startswith(('Head /','Eye socket /','Eye /','Pupil /','Glint /','Bill /','Mouth back /')) else 4 if n.startswith('Crown lobe /') else (5 if (n.split()[-2] if n.startswith('Hand claw /') else n.split()[-1])=='-1' else 6) if n.startswith(('Hand /','Hand claw /')) else None
   if owner is not None:self.assertTrue(all(ids==[owner,0,0,0] for ids in m.ids[p]),n)
 def test_floor_fixed_claws_endpoints_and_articulation(self):
  m=self.m;moved=set()
  for clip in m.clips:
   for step in range(13):
    mats=m.matrices(clip,step/12)
    for p,n in enumerate(m.names):
     posed=m.posed(p,mats);self.assertGreaterEqual(min(v[1] for v in posed),-1e-7,(clip,n))
     if n.startswith(('Foot /','Toe claw /')):self.assertEqual(posed,m.vertices[p])
     if max(math.dist(a,b) for a,b in zip(posed,m.vertices[p]))>.0005:moved.add(n.split(' /')[0])
    if step in (0,12):
     for matrix in mats:self.assertLess(max(abs(a-b) for a,b in zip(matrix,IDENTITY)),1e-7)
  self.assertTrue({'Body','Belly','Head','Eye','Bill','Crown lobe','Arm','Hand','Hand claw','Tail','Leg'}<=moved,moved)
 def test_winding_and_edge_strain(self):
  m=self.m
  for clip in m.clips:
   for t in (.18,.31,.55,.80):
    mats=m.matrices(clip,t)
    for p,n in enumerate(m.names):
     posed=m.posed(p,mats);normals=m.posed(p,mats,True)
     for k in range(0,len(m.indices[p]),3):
      tri=m.indices[p][k:k+3];cosine,area=skin.winding_alignment(posed,normals,tri);self.assertGreater(cosine,0.,(clip,t,n,k));self.assertGreater(area,1e-12)
      for a,b in zip(tri,tri[1:]+tri[:1]):
       ratio=math.dist(posed[a],posed[b])/math.dist(m.vertices[p][a],m.vertices[p][b]);self.assertTrue(.80<ratio<1.20,(clip,t,n,ratio))
 def test_actual_posed_crown_neck_limbs_claws_and_tail_roots(self):
  m=self.m
  def find(prefix):return next(i for i,n in enumerate(m.names) if n.startswith(prefix))
  body=find('Body /');head=find('Head /');collar=find('Collar /');pairs=[(head,collar),(collar,body),(find('Bill /'),head),(find('Tail /'),body)]
  for p,n in enumerate(m.names):
   if n.startswith('Crown lobe /'):pairs.append((p,head))
   elif n.startswith(('Arm /','Leg /')):pairs.append((p,body))
   elif n.startswith('Hand /'):pairs.append((p,find('Arm / rooted raised red live-coal limb '+n.split()[-1])))
   elif n.startswith('Hand claw /'):pairs.append((p,find('Hand / rooted red live-coal claw junction '+n.split()[-2])))
   elif n.startswith('Foot /'):pairs.append((p,find('Leg / rooted red live-coal shin '+n.split()[-1])))
   elif n.startswith('Toe claw /'):pairs.append((p,find('Foot / grounded small red live-coal support '+n.split()[-2])))
  for clip in m.clips:
   for t in (0.,.18,.55):
    mats=m.matrices(clip,t)
    for a,b in pairs:
     def part(p):return {'positions':[v for point in m.posed(p,mats) for v in point],'indices':m.indices[p]}
     self.assertTrue(joins.DonphanTests.overlap(self,part(a),part(b)),(clip,t,m.names[a],m.names[b]))
 def test_noncanonical_data_rejected(self):
  with self.assertRaises(ValueError):author.decode_magby(self.blob[:-1])
if __name__=='__main__':unittest.main()
