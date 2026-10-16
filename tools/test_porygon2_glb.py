#!/usr/bin/env python3
"""Actual decoded virtual-duck neck, tail, fitted gaze and planted paddles."""
import math,unittest
from pathlib import Path
import porygon2_glb as author
import test_gengar_glb as skin
from skin_glb_test_support import IDENTITY
from sculpt_test_support import parts_overlap,ray_intersections
ROOT=Path(__file__).resolve().parents[1]/'crates/crystal-voxel-view/models/battle_species'
class Porygon2SkinTests(unittest.TestCase):
 @classmethod
 def setUpClass(cls):cls.blob=(ROOT/'porygon2.glb').read_bytes();cls.m=skin.Model(cls.blob)
 def part(self,p,mats):return {'positions':[v for point in self.m.posed(p,mats) for v in point],'indices':self.m.indices[p]}
 def test_canonical_geometry_bind_and_budgets(self):
  m=self.m;self.assertEqual(self.blob,author.export_porygon2(author.decode_porygon2(self.blob)));self.assertLess(len(self.blob),1024*1024);self.assertLess(sum(map(len,m.vertices)),20000);self.assertLessEqual(len(m.doc['accessors']),256)
  self.assertEqual((len(m.parts),sum(map(len,m.vertices)),sum(len(p)//3 for p in m.indices),len(m.joints)),(17,14189,23912,6))
  other=[p for p in m.neutral_model()["primitives"] if not p["part"].startswith(("Eye socket /","Eye white /","Pupil /","Eye glint /"))]
  self.assertEqual(len(other),9);self.assertEqual(author.geometry_digest({"primitives":other}),"4d894b9d33428b82d3b02db38aae4014af151a38302fc0e740d558250bdfcd5c")
  for matrix in m.matrices():self.assertLess(max(abs(a-b) for a,b in zip(matrix,IDENTITY)),1e-7)
 def test_coherent_fitted_face_bill_and_fixed_paddles(self):
  m=self.m
  for p,n in enumerate(m.names):
   owner=0 if n.startswith(('Foot /','Foot highlight /')) else 3 if n.startswith(('Bill /','Eye socket /','Eye white /','Pupil /','Eye glint /','Head highlight /')) else None
   if owner is not None:self.assertTrue(all(ids==[owner,0,0,0] for ids in m.ids[p]),n)
 def test_neutral_endpoints_floor_and_visible_articulation(self):
  m=self.m;moved=set()
  for clip in m.clips:
   for step in range(13):
    mats=m.matrices(clip,step/12)
    for p,n in enumerate(m.names):
     posed=m.posed(p,mats);self.assertGreaterEqual(min(v[1] for v in posed),-1e-7,n)
     if max(math.dist(a,b) for a,b in zip(posed,m.vertices[p]))>.003:moved.add(n.split(' /')[0])
     if n.startswith(('Foot /','Foot highlight /')):
      for a,b in zip(posed,m.vertices[p]):self.assertLess(math.dist(a,b),1e-7)
    if step in (0,12):
     for matrix in mats:self.assertLess(max(abs(a-b) for a,b in zip(matrix,IDENTITY)),1e-7)
  self.assertTrue({'Body','Tail','Bill','Pupil'}<=moved,moved)
 def test_winding_and_physical_edge_strain(self):
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
 def test_posed_belly_bill_tail_and_paddles_remain_attached(self):
  m=self.m
  for clip in m.clips:
   for t in (0.,.18,.55):
    mats=m.matrices(clip,t);body=self.part(0,mats);belly=self.part(1,mats)
    for p,n in enumerate(m.names):
     if n.startswith(('Belly /','Bill /','Tail /','Foot /')):self.assertTrue(parts_overlap(body,self.part(p,mats)),(clip,t,n))
     if n.startswith('Foot /'):self.assertTrue(parts_overlap(belly,self.part(p,mats)),(clip,t,n))
 def test_posed_fitted_eyes_and_highlights_remain_on_actual_support(self):
  m=self.m
  for clip in m.clips:
   for t in (.18,.55):
    mats=m.matrices(clip,t)
    for p,n in enumerate(m.names):
     sign=n.split()[-1];parent=None
     for prefix,support in [('Eye socket /','Body /'),('Head highlight /','Body /'),('Eye white /','Eye socket / thin fitted dark oval border '+sign),('Pupil /','Eye white / large fitted ivory virtual-duck oval '+sign),('Eye glint /','Pupil / fitted dark vertical curious oval '+sign),('Foot highlight /','Foot / grounded rounded splayed cyan paddle '+sign)]:
      if n.startswith(prefix):parent=next(i for i,q in enumerate(m.names) if q.startswith(support));break
     if parent is None:continue
     support=self.part(parent,mats)
     for x,y,z in set(map(tuple,m.posed(p,mats))):
      hits=ray_intersections(support,x,y);self.assertTrue(hits,(clip,n));self.assertLess(abs(z-max(hits)),.017,(clip,n))
 def test_attack_advances_bill_and_tail_moves_independently_of_body(self):
  m=self.m;mats=m.matrices('attack',.18);bill=next(i for i,n in enumerate(m.names) if n.startswith('Bill /'));tail=next(i for i,n in enumerate(m.names) if n.startswith('Tail /'))
  self.assertGreater(sum(a[2]-b[2] for a,b in zip(m.posed(bill,mats),m.vertices[bill]))/len(m.vertices[bill]),.015)
  self.assertTrue(any(math.dist(m.point(tail,k,mats),skin.transform(mats[1],v))>.01 for k,v in enumerate(m.vertices[tail])))
 def test_noncanonical_data_rejected(self):
  with self.assertRaises(ValueError):author.decode_porygon2(self.blob[:-1])
if __name__=='__main__':unittest.main()
