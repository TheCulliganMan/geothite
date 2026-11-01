#!/usr/bin/env python3
"""Decoded crab hierarchy, closing jaws, braced limbs and fixed floor contact."""
import copy,math,unittest
from pathlib import Path
import crab_glb as author
import test_gengar_glb as skin
from skin_glb_test_support import IDENTITY,transform
from sculpt_test_support import parts_overlap
ROOT=Path(__file__).resolve().parents[1]/'crates/crystal-voxel-view/models/battle_species'
class CrabSkinTests(unittest.TestCase):
 @classmethod
 def setUpClass(cls):
  cls.blobs={s:(ROOT/(s+'.glb')).read_bytes() for s in author.SPECIES};cls.models={s:skin.Model(b) for s,b in cls.blobs.items()}
 def test_canonical_bounded_container_and_identity_bind(self):
  for s,m in self.models.items():
   self.assertEqual(self.blobs[s],author.export_crab(s));self.assertFalse((ROOT/(s+'.mesh.json')).exists());self.assertEqual(len(m.joints),12)
   self.assertLessEqual(len(m.parts),64);self.assertLessEqual(sum(map(len,m.vertices)),20000);self.assertLessEqual(sum(map(len,m.indices)),120000);self.assertLessEqual(len(m.doc['accessors']),256);self.assertLess(len(self.blobs[s]),1024*1024)
   self.assertEqual(m.parents[:12],[None,0,1,2,3,1,5,6,0,0,0,0])
   for matrix in m.matrices():self.assertLess(max(abs(a-b) for a,b in zip(matrix,IDENTITY)),1e-7)
   used=set()
   for ids,weights in zip(m.ids,m.weights):
    for js,ws in zip(ids,weights):
     self.assertAlmostEqual(sum(ws),1.);used.update(j for j,w in zip(js,ws) if w)
   self.assertEqual(used,set(range(12)))
 def test_anatomical_owners_and_planted_toes(self):
  for s,m in self.models.items():
   for i,n in enumerate(m.names):
    owners={j for js,ws in zip(m.ids[i],m.weights[i]) for j,w in zip(js,ws) if w}
    if n.startswith('Foot /'):expected={0}
    elif n.startswith(('Walking leg /','Leg knee /')):
     side,j=map(int,n.split()[-2:]);expected={0,1,8+(0 if side<0 else 2)+j}
     self.assertTrue(owners<=expected);self.assertIn(8+(0 if side<0 else 2)+j,owners);continue
    elif n.startswith('Pincer arm /'):expected={1,2 if n.endswith('-1') else 5,3 if n.endswith('-1') else 6}
    elif n.startswith(('Wrist /','Claw palm /')):expected={3 if n.endswith('-1') else 6}
    elif n.startswith('Claw jaw /'):expected={4 if n.endswith('-1') else 7}
    else:expected={1}
    self.assertEqual(owners,expected,(s,n))
 def test_actual_solid_roots_remain_occupied_during_motion(self):
  for s,m in self.models.items():
   parts=m.neutral_model()['primitives']
   def find(prefix,end=None):return next(i for i,n in enumerate(m.names) if n.startswith(prefix) and (end is None or n.split()[-len(end):]==end))
   body=find('Carapace /');belly=find('Belly /');pairs=[(body,belly)]
   for side in ('-1','1'):
    arm=find('Pincer arm /',[side]);wrist=find('Wrist /',[side]);palm=find('Claw palm /',[side]);jaw=find('Claw jaw /',[side])
    pairs += [(body,arm),(arm,wrist),(wrist,palm),(palm,jaw),(belly,find('Cheek peak /',[side]))]
    for j in ('0','1'):
     leg=find('Walking leg /',[side,j]);pairs += [(belly,leg),(leg,find('Foot /',[side,j])),(leg,find('Leg knee /',[side,j]))]
   pairs += [(body,i) for i,n in enumerate(m.names) if n.startswith('Crown /')]
   for clip in m.clips:
    for t in (0.,.31):
     mats=m.matrices(clip,t);posed=[dict(p,positions=sum(m.posed(i,mats),[])) for i,p in enumerate(parts)]
     for a,b in pairs:self.assertTrue(parts_overlap(posed[a],posed[b]),(s,clip,t,m.names[a],m.names[b]))
   detached=copy.deepcopy(parts[find('Claw jaw /')]);detached['positions']=[v+(2. if i%3==0 else 0.) for i,v in enumerate(detached['positions'])];self.assertFalse(parts_overlap(detached,parts[find('Claw palm /')]))
 def test_floor_winding_strain_endpoints_and_independent_jaws(self):
  for s,m in self.models.items():
   independent=set()
   for clip in m.clips:
    for step in range(9):
     mats=m.matrices(clip,step/8)
     for i,n in enumerate(m.names):
      points=m.posed(i,mats);normals=m.posed(i,mats,True);self.assertGreaterEqual(min(p[1] for p in points),-1e-7,(s,clip,n))
      if n.startswith('Foot /'):self.assertLess(max(math.dist(a,b) for a,b in zip(points,m.vertices[i])),1e-7)
      if n.startswith('Claw jaw /'):
       wrist=3 if n.endswith('-1') else 6
       if max(math.dist(a,transform(mats[wrist],b)) for a,b in zip(points,m.vertices[i]))>.004:independent.add(n)
      for k in range(0,len(m.indices[i]),3):
       tri=m.indices[i][k:k+3];cosine,area=skin.winding_alignment(points,normals,tri);self.assertGreater(cosine,0.,(s,clip,n));self.assertGreater(area,1e-12)
       for a,b in zip(tri,tri[1:]+tri[:1]):self.assertTrue(.75<math.dist(points[a],points[b])/math.dist(m.vertices[i][a],m.vertices[i][b])<1.3,(s,clip,n))
     if step in (0,8):
      for mat in mats:self.assertLess(max(abs(a-b) for a,b in zip(mat,IDENTITY)),1e-7)
   self.assertEqual(len(independent),2)
 def test_source_cues_own_attack_and_hit_clocks(self):
  for m in self.models.values():
   for clip in m.doc['animations']:
    idle=clip['name'].endswith('.idle');self.assertEqual(clip['extras']['loopSuggested'],idle);self.assertEqual(clip['extras']['cueRelative'],not idle);self.assertEqual({c['target']['path'] for c in clip['channels']},{'rotation'})
if __name__=='__main__':unittest.main()
