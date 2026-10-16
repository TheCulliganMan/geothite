#!/usr/bin/env python3
"""Decoded hoof contacts, coherent fitted coats and physically rooted appendages."""
import math,unittest
from pathlib import Path
import fur_pig_glb as author
import test_gengar_glb as skin
from skin_glb_test_support import IDENTITY
from sculpt_test_support import parts_overlap,ray_intersections
ROOT=Path(__file__).resolve().parents[1]/'crates/crystal-voxel-view/models/battle_species'
class FurPigSkinTests(unittest.TestCase):
 @classmethod
 def setUpClass(cls):
  cls.blobs={s:(ROOT/(s+'.glb')).read_bytes() for s in author.SPECIES};cls.models={s:skin.Model(b) for s,b in cls.blobs.items()}
 def part(self,m,p,mats):return {'positions':[v for point in m.posed(p,mats) for v in point],'indices':m.indices[p]}
 def test_canonical_full_geometry_neutral_bind_and_budgets(self):
  for s,m in self.models.items():
   self.assertEqual(self.blobs[s],author.export_fur_pig(s,author.decode_fur_pig(self.blobs[s],s)))
   self.assertEqual((len(m.parts),sum(map(len,m.vertices)),sum(len(p)//3 for p in m.indices),len(m.joints)),{'swinub':(22,7044,8496,8),'piloswine':(22,5062,9432,9)}[s])
   self.assertLess(len(self.blobs[s]),1024*1024);self.assertLessEqual(len(m.doc['accessors']),256)
   for matrix in m.matrices():self.assertLess(max(abs(a-b) for a,b in zip(matrix,IDENTITY)),1e-7)
 def test_fixed_hooves_and_coherent_noses_tusks_and_stripes(self):
  for s,m in self.models.items():
   for p,n in enumerate(m.names):
    owner=0 if n.startswith('Hoof /') else 2 if n.startswith(('Snout /','Nostril /','Tusk /','Eye /','Fur crease /')) else 1 if n.startswith('Stripe /') else None
    if owner is not None:self.assertTrue(all(ids==[owner,0,0,0] for ids in m.ids[p]),(s,n))
 def test_floor_contacts_neutral_endpoints_and_visible_articulation(self):
  for s,m in self.models.items():
   moved=set()
   for clip in m.clips:
    for step in range(13):
     mats=m.matrices(clip,step/12)
     for p,n in enumerate(m.names):
      posed=m.posed(p,mats);self.assertGreaterEqual(min(v[1] for v in posed),-1e-7,(s,clip,n))
      if n.startswith('Hoof /'):self.assertEqual(posed,m.vertices[p])
      if max(math.dist(a,b) for a,b in zip(posed,m.vertices[p]))>.001:moved.add(n.split(' /')[0])
     if step in (0,12):
      for matrix in mats:self.assertLess(max(abs(a-b) for a,b in zip(matrix,IDENTITY)),1e-7)
   self.assertTrue({'Body','Snout','Nostril','Ear' if s=='piloswine' else 'Fur fringe'}<=moved,(s,moved))
 def test_winding_and_edge_strain(self):
  for s,m in self.models.items():
   for clip in m.clips:
    for t in (.18,.31,.55,.80):
     mats=m.matrices(clip,t)
     for p,n in enumerate(m.names):
      posed=m.posed(p,mats);normals=m.posed(p,mats,True)
      for k in range(0,len(m.indices[p]),3):
       tri=m.indices[p][k:k+3];cosine,area=skin.winding_alignment(posed,normals,tri);self.assertGreater(cosine,0.,(s,clip,t,n,k));self.assertGreater(area,1e-12)
       for a,b in zip(tri,tri[1:]+tri[:1]):
        ratio=math.dist(posed[a],posed[b])/math.dist(m.vertices[p][a],m.vertices[p][b]);self.assertTrue(.80<ratio<1.20,(s,clip,t,n,ratio))
 def test_actual_posed_body_leg_hoof_and_appendage_joins(self):
  for s,m in self.models.items():
   body=next(p for p,n in enumerate(m.names) if n.startswith('Body /'));pairs=[]
   for p,n in enumerate(m.names):
    if n.startswith(('Snout /','Leg /','Ear /','Tusk /','Fur fringe /')):pairs.append((p,body))
    if n.startswith('Hoof /'):pairs.append((p,m.names.index('Leg / hidden supporting shank '+' '.join(n.split()[-2:]))))
   for clip in m.clips:
    for t in (0.,.18,.55):
     mats=m.matrices(clip,t)
     for a,b in pairs:self.assertTrue(parts_overlap(self.part(m,a,mats),self.part(m,b,mats)),(s,clip,t,m.names[a],m.names[b]))
 def test_posed_eyes_stripes_fur_strands_and_nostrils_follow_actual_supports(self):
  for s,m in self.models.items():
   body=next(p for p,n in enumerate(m.names) if n.startswith('Body /'));snout=next(p for p,n in enumerate(m.names) if n.startswith('Snout /'))
   for clip in m.clips:
    for t in (.18,.55):
     mats=m.matrices(clip,t);coat=self.part(m,body,mats);nose=self.part(m,snout,mats)
     top=dict(coat);top['positions']=[v for k in range(0,len(coat['positions']),3) for v in (coat['positions'][k],coat['positions'][k+2],coat['positions'][k+1])]
     for p,n in enumerate(m.names):
      if not n.startswith(('Stripe /','Eye /','Fur crease /','Nostril /')):continue
      support=top if n.startswith('Stripe /') else nose if n.startswith('Nostril /') else coat
      for x,y,z in set(map(tuple,m.posed(p,mats))):
       hits=ray_intersections(support,x,z if n.startswith('Stripe /') else y);self.assertTrue(hits,(s,clip,t,n,x,y,z));self.assertLess(abs((y if n.startswith('Stripe /') else z)-max(hits)),.022,(s,clip,t,n))
 def test_noncanonical_data_rejected(self):
  for s,blob in self.blobs.items():
   with self.assertRaises(ValueError):author.decode_fur_pig(blob[:-1],s)
if __name__=='__main__':unittest.main()
