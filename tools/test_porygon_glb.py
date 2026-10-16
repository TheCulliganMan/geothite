#!/usr/bin/env python3
"""Stored rigid virtual-duck cages, planted paddle swivels and physical joins."""
import math,unittest
from pathlib import Path
import porygon_glb as author
import test_gengar_glb as skin
from skin_glb_test_support import IDENTITY
from sculpt_test_support import parts_overlap,ray_intersections
from porygon_sculpt import side_view
ROOT=Path(__file__).resolve().parents[1]/'crates/crystal-voxel-view/models/battle_species'
class PorygonSkinTests(unittest.TestCase):
 @classmethod
 def setUpClass(cls):cls.blob=(ROOT/'porygon.glb').read_bytes();cls.m=skin.Model(cls.blob)
 def part(self,p,mats):return {'positions':[v for point in self.m.posed(p,mats) for v in point],'indices':self.m.indices[p]}
 def test_canonical_original_geometry_bind_and_budgets(self):
  m=self.m;self.assertEqual(self.blob,author.export_porygon(author.decode_porygon(self.blob)))
  self.assertEqual((len(m.parts),sum(map(len,m.vertices)),sum(len(p)//3 for p in m.indices),len(m.joints)),(16,4789,2008,7));self.assertLess(len(self.blob),1024*1024);self.assertLessEqual(len(m.doc['accessors']),256)
  for matrix in m.matrices():self.assertLess(max(abs(a-b) for a,b in zip(matrix,IDENTITY)),1e-7)
 def test_rigid_panel_and_fitted_eye_owners(self):
  m=self.m
  for p,n in enumerate(m.names):
   owner=5 if n.startswith('Foot /') and n.endswith('-1') else 6 if n.startswith('Foot /') else 3 if n.startswith(('Head /','Bill /','Eye border /','Eye white /','Pupil /','Eye glint /')) else 2 if n.startswith('Neck /') else 4 if n.startswith('Tail /') else 1
   self.assertTrue(all(ids==[owner,0,0,0] for ids in m.ids[p]),n)
 def test_neutral_endpoints_floor_height_and_visible_articulation(self):
  m=self.m;moved=set()
  for clip in m.clips:
   for step in range(13):
    mats=m.matrices(clip,step/12)
    for p,n in enumerate(m.names):
     posed=m.posed(p,mats);self.assertGreaterEqual(min(v[1] for v in posed),-1e-7,n)
     if max(math.dist(a,b) for a,b in zip(posed,m.vertices[p]))>.003:moved.add(n.split(' /')[0])
     if n.startswith('Foot /'):
      for a,b in zip(posed,m.vertices[p]):self.assertAlmostEqual(a[1],b[1],places=7)
    if step in (0,12):
     for matrix in mats:self.assertLess(max(abs(a-b) for a,b in zip(matrix,IDENTITY)),1e-7)
  self.assertTrue({'Body','Neck','Head','Tail','Bill','Foot','Pupil'}<=moved,moved)
 def test_all_facets_remain_rigid_without_winding_or_edge_distortion(self):
  m=self.m
  for clip in m.clips:
   for t in (.18,.31,.55,.80):
    mats=m.matrices(clip,t)
    for p,n in enumerate(m.names):
     posed=m.posed(p,mats);normals=m.posed(p,mats,True)
     for k in range(0,len(m.indices[p]),3):
      tri=m.indices[p][k:k+3];cosine,area=skin.winding_alignment(posed,normals,tri);self.assertGreater(cosine,0.,(clip,t,n,k));self.assertGreater(area,1e-12)
      for a,b in zip(tri,tri[1:]+tri[:1]):self.assertAlmostEqual(math.dist(posed[a],posed[b])/math.dist(m.vertices[p][a],m.vertices[p][b]),1.,places=6)
 def test_actual_posed_hull_connector_head_bill_tail_and_paddle_joins(self):
  m=self.m
  def find(prefix):return next(i for i,n in enumerate(m.names) if n.startswith(prefix))
  body,breast,neck,head,bill,tail=map(find,('Body /','Breast /','Neck /','Head /','Bill /','Tail /'))
  pairs=[(body,breast),(body,neck),(neck,head),(head,bill),(body,tail)]+[(breast,p) for p,n in enumerate(m.names) if n.startswith('Foot /')]
  for clip in m.clips:
   for t in (0.,.18,.31,.55):
    mats=m.matrices(clip,t)
    for a,b in pairs:self.assertTrue(parts_overlap(self.part(a,mats),self.part(b,mats)),(clip,t,m.names[a],m.names[b]))
 def test_fitted_side_panel_eyes_follow_the_actual_moving_head(self):
  m=self.m
  for clip in m.clips:
   for t in (.18,.55):
    mats=m.matrices(clip,t);a=mats[3]
    def local(p):
     q=self.part(p,mats);points=[]
     for i in range(0,len(q['positions']),3):
      d=[q['positions'][i+k]-a[12+k] for k in range(3)];points.extend(sum(a[4*j+k]*d[k] for k in range(3)) for j in range(3))
     q['positions']=points;q['normals']=[v for normal in m.normals[p] for v in normal];return q
    for p,n in enumerate(m.names):
     sign=n.split()[-1];parent=None
     for prefix,support in [('Eye border /','Head /'),('Eye white /','Eye border / fitted dark hexagonal socket '+sign),('Pupil /','Eye white / fitted ivory polygonal panel '+sign),('Eye glint /','Pupil / fitted small round digital eye '+sign)]:
      if n.startswith(prefix):parent=next(i for i,q in enumerate(m.names) if q.startswith(support));break
     if parent is None:continue
     support=side_view(local(parent),int(sign));mark=side_view(local(p),int(sign))
     for i in range(0,len(mark['positions']),3):
      x,y,z=mark['positions'][i:i+3];hits=ray_intersections(support,x,y);self.assertTrue(hits,n);self.assertLess(abs(z-max(hits)),.017,n)
 def test_attack_advances_bill_and_independently_swivels_tail_and_paddles(self):
  m=self.m;mats=m.matrices('attack',.18);bill=next(i for i,n in enumerate(m.names) if n.startswith('Bill /'))
  self.assertGreater(sum(a[2]-b[2] for a,b in zip(m.posed(bill,mats),m.vertices[bill]))/len(m.vertices[bill]),.010)
  for prefix in ('Tail /','Foot /'):
   for p,n in enumerate(m.names):
    if n.startswith(prefix):self.assertTrue(any(math.dist(m.point(p,k,mats),skin.transform(mats[1 if prefix=='Tail /' else 0],v))>.005 for k,v in enumerate(m.vertices[p])),n)
 def test_noncanonical_data_rejected(self):
  with self.assertRaises(ValueError):author.decode_porygon(self.blob[:-1])
if __name__=='__main__':unittest.main()
