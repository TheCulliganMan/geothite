#!/usr/bin/env python3
"""Decoded complete coral forks, attached rounded tips and four planted pads."""
import math,unittest
from pathlib import Path
import corsola_glb as author
import test_gengar_glb as skin
from skin_glb_test_support import IDENTITY
from sculpt_test_support import parts_overlap,ray_intersections
ROOT=Path(__file__).resolve().parents[1]/'crates/crystal-voxel-view/models/battle_species'
class CorsolaSkinTests(unittest.TestCase):
 @classmethod
 def setUpClass(cls):cls.blob=(ROOT/'corsola.glb').read_bytes();cls.m=skin.Model(cls.blob)
 def find(self,prefix):return next(i for i,n in enumerate(self.m.names) if n.startswith(prefix))
 def part(self,p,mats):return {'positions':[v for point in self.m.posed(p,mats) for v in point],'indices':self.m.indices[p]}
 def test_canonical_original_geometry_bind_and_budgets(self):
  m=self.m;self.assertEqual(self.blob,author.export_corsola(author.decode_corsola(self.blob)));self.assertEqual((len(m.parts),sum(map(len,m.vertices)),sum(len(p)//3 for p in m.indices),len(m.joints)),(37,7359,14180,9));self.assertLess(len(self.blob),1024*1024);self.assertLessEqual(len(m.doc['accessors']),256)
  for matrix in m.matrices():self.assertLess(max(abs(a-b) for a,b in zip(matrix,IDENTITY)),1e-7)
 def test_complete_branch_arm_tip_and_fitted_face_owners(self):
  m=self.m
  for p,n in enumerate(m.names):
   owner=0 if n.startswith('Foot /') else 2+int(n.split()[-1]) if n.startswith('Coral /') else (7 if n.endswith('-1') else 8) if n.startswith('Side arm /') else 1
   self.assertTrue(all(ids==[owner,0,0,0] for ids in m.ids[p]),n)
 def test_neutral_endpoints_fixed_pads_floor_clearance_and_articulation(self):
  m=self.m;moved=set();feet=[p for p,n in enumerate(m.names) if n.startswith('Foot /')];self.assertEqual(len(feet),4)
  for clip in m.clips:
   for step in range(13):
    mats=m.matrices(clip,step/12)
    for p,n in enumerate(m.names):
     posed=m.posed(p,mats);self.assertGreaterEqual(min(v[1] for v in posed),-1e-7,(clip,n))
     if max(math.dist(a,b) for a,b in zip(posed,m.vertices[p]))>.003:moved.add(n.split(' /')[0])
     if p in feet:
      for a,b in zip(posed,m.vertices[p]):self.assertLess(math.dist(a,b),1e-7)
    if step in (0,12):
     for matrix in mats:self.assertLess(max(abs(a-b) for a,b in zip(matrix,IDENTITY)),1e-7)
  self.assertTrue({'Body','Belly','Coral','Side arm','Eye','Mouth','Belly fleck'}<=moved,moved)
 def test_rigid_branches_keep_winding_and_exact_edge_lengths(self):
  m=self.m
  for clip in m.clips:
   for t in (.18,.31,.55,.80):
    mats=m.matrices(clip,t)
    for p,n in enumerate(m.names):
     posed=m.posed(p,mats);normals=m.posed(p,mats,True)
     for k in range(0,len(m.indices[p]),3):
      tri=m.indices[p][k:k+3];cosine,area=skin.winding_alignment(posed,normals,tri);self.assertGreater(cosine,0.,(clip,t,n,k));self.assertGreater(area,1e-12)
      for a,b in zip(tri,tri[1:]+tri[:1]):self.assertAlmostEqual(math.dist(posed[a],posed[b])/math.dist(m.vertices[p][a],m.vertices[p][b]),1.,places=6)
 def test_actual_posed_crown_bowl_feet_branch_forks_eyes_and_arm_joins(self):
  m=self.m;f=self.find;body=f('Body /');belly=f('Belly /');pairs=[(body,belly)]
  for p,n in enumerate(m.names):
   if n.startswith('Foot /'):pairs.append((p,belly))
  for side in ('-1','1'):
   arm=f('Side arm / pink drooping branch '+side);eye=f('Eye / glossy oval '+side)
   pairs += [(arm,body),(arm,f('Side arm / rounded tip '+side)),(eye,body),(eye,f('Eye / white glint '+side))]
  for j in range(5):
   branch=f('Coral / rounded primary branch '+str(j));fork=f('Coral / thick lateral fork '+str(j))
   pairs += [(branch,body),(branch,f('Coral / smooth primary tip '+str(j))),(branch,fork),(fork,f('Coral / rounded fork tip '+str(j)))]
  for clip in m.clips:
   for t in (0.,.18,.55):
    mats=m.matrices(clip,t)
    for a,b in pairs:self.assertTrue(parts_overlap(self.part(a,mats),self.part(b,mats)),(clip,t,m.names[a],m.names[b]))
 def test_fitted_smile_and_white_flecks_stay_on_the_moving_crown(self):
  m=self.m;body=self.find('Body /')
  for clip in m.clips:
   for t in (.18,.55):
    mats=m.matrices(clip,t);a=mats[1]
    def local(p):
     points=[]
     for v in m.posed(p,mats):
      d=[v[k]-a[12+k] for k in range(3)];points.extend(sum(a[4*j+k]*d[k] for k in range(3)) for j in range(3))
     return {'positions':points,'indices':m.indices[p]}
    support=local(body)
    for p,n in enumerate(m.names):
     if not n.startswith(('Belly fleck /','Mouth /')):continue
     mark=local(p)
     for i in range(0,len(mark['positions']),3):
      x,y,z=mark['positions'][i:i+3];hits=ray_intersections(support,x,y);self.assertTrue(hits,n);self.assertLess(abs(z-max(hits)),.023,n)
 def test_each_stem_and_arm_moves_independently_of_the_crown(self):
  m=self.m;mats=m.matrices('attack',.18)
  for prefix in ['Coral / smooth primary tip '+str(j) for j in range(5)]+['Side arm / rounded tip '+s for s in ('-1','1')]:
   p=self.find(prefix);threshold=.025 if prefix.startswith('Side arm /') else .005;self.assertTrue(any(math.dist(m.point(p,k,mats),skin.transform(mats[1],v))>threshold for k,v in enumerate(m.vertices[p])),prefix)
 def test_noncanonical_data_rejected(self):
  with self.assertRaises(ValueError):author.decode_corsola(self.blob[:-1])
if __name__=='__main__':unittest.main()
