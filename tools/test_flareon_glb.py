#!/usr/bin/env python3
"""Flame-fox anatomy, planted paws and occupied flexible cream plume checks."""
import copy,math,unittest
from pathlib import Path
import flareon_glb as author
import test_gengar_glb as skin
from skin_glb_test_support import IDENTITY
from sculpt_test_support import assert_closed_models,assert_model_shading,parts_overlap,indexed_ray_intersections as rays
ROOT=Path(__file__).resolve().parents[1]/'crates/crystal-voxel-view/models/battle_species'
ASSET=ROOT/'flareon.glb'
class FlareonTests(unittest.TestCase):
 @classmethod
 def setUpClass(cls):
  cls.blob=ASSET.read_bytes();cls.m=skin.Model(cls.blob);cls.parts=cls.m.neutral_model()['primitives']
 def find(self,prefix,ending=None):return next(p for p in self.parts if p['part'].startswith(prefix) and (ending is None or p['part'].split()[-len(ending):]==ending))
 def test_canonical_schema_limits_and_distinctive_anatomy(self):
  m=self.m;self.assertEqual(self.blob,author.export_flareon());self.assertEqual(len(m.parts),46);self.assertEqual(len(m.joints),14)
  self.assertFalse((ROOT/'flareon.mesh.json').exists())
  self.assertLessEqual(sum(map(len,m.vertices)),20000);self.assertLessEqual(sum(map(len,m.indices)),120000);self.assertLessEqual(len(m.doc['accessors']),256);self.assertLess(len(self.blob),1024*1024)
  points=[p for v in m.vertices for p in v];self.assertEqual(min(p[1] for p in points),0.);self.assertAlmostEqual(max(p[1] for p in points),.97,places=6)
  for prefix,count in [('Ear /',2),('Ear inset /',2),('Paw /',4),('Leg /',4),('Haunch /',2),('Ruff /',1),('Ruff tuft /',3),('Crown tuft /',3),('Tail /',1)]:self.assertEqual(sum(n.startswith(prefix) for n in m.names),count)
  for matrix in m.matrices():self.assertLess(max(abs(a-b) for a,b in zip(matrix,IDENTITY)),1e-7)
 def test_closed_outward_solids(self):assert_closed_models(self,{'flareon':self.m.neutral_model()})
 def test_actual_face_normals(self):assert_model_shading(self,{'flareon':self.m.neutral_model()})
 def test_fitted_eyes_ear_insets_and_sparkles(self):
  for mark in self.parts:
   n=mark['part'];side=[n.split()[-1]]
   if n.startswith('Eye /'):parent=self.find('Head /')
   elif n.startswith('Ear inset /'):parent=self.find('Ear /',side)
   elif n.startswith(('Iris /','Pupil /')):parent=self.find('Eye /',side)
   elif n.startswith('Glint /'):parent=self.find('Pupil /',side)
   elif n.startswith('Glint small /'):parent=self.find('Iris /',side)
   else:continue
   vs=[mark['positions'][i:i+3] for i in range(0,len(mark['positions']),3)]
   for k in range(0,len(mark['indices']),3):
    a,b,c=[vs[j] for j in mark['indices'][k:k+3]]
    if (b[0]-a[0])*(c[1]-a[1])-(b[1]-a[1])*(c[0]-a[0])<=1e-14:continue
    q=[(a[j]+b[j]+c[j])/3 for j in range(3)];hits=rays(parent,q[0],q[1]);self.assertTrue(hits,n);self.assertGreater(q[2]-max(hits),.0001,(n,q))
 def test_anatomical_owners_and_four_planted_paws(self):
  m=self.m
  for i,n in enumerate(m.names):
   owners={j for js,ws in zip(m.ids[i],m.weights[i]) for j,w in zip(js,ws) if w}
   if n.startswith(('Paw /','Toe crease /')):self.assertEqual(owners,{0},n)
   elif n.startswith(('Body /','Haunch /')):self.assertEqual(owners,{1},n)
   elif n.startswith(('Ruff /','Ruff tuft /')):self.assertEqual(owners,{13},n)
   elif n.startswith(('Ear /','Ear inset /')):self.assertEqual(owners,{4 if n.endswith('-1') else 5},n)
   elif n.startswith('Leg /'):
    side,row=map(int,n.split()[-2:]);self.assertEqual(owners,{0,1,9+(0 if side<0 else 2)+row})
   elif n.startswith('Neck /'):self.assertEqual(owners,{1,2,3})
   elif not n.startswith(('Tail /','Tail tip /')):self.assertEqual(owners,{3},n)
  self.assertEqual({(math.copysign(1.,p[0]),math.copysign(1.,p[2])) for i,n in enumerate(m.names) if n.startswith('Paw /') for p in m.vertices[i] if p[1]<1e-7},{(-1.,-1.),(-1.,1.),(1.,-1.),(1.,1.)})
 def test_occupied_neck_ruff_ear_and_leg_roots_survive_motion(self):
  m=self.m;body=self.find('Body /')['part'];head=self.find('Head /')['part'];neck=self.find('Neck /')['part'];ruff=self.find('Ruff /')['part']
  pairs=[(neck,body),(neck,head),(ruff,neck),(self.find('Muzzle /')['part'],head),(self.find('Tail /')['part'],body)]
  for p in self.parts:
   n=p['part']
   if n.startswith(('Ear /','Crown tuft /')):pairs.append((n,head))
   elif n.startswith('Ruff tuft /'):pairs.append((n,ruff))
   elif n.startswith('Haunch /'):pairs.append((n,body))
   elif n.startswith('Leg /'):
    side,row=n.split()[-2:];pairs.append((n,self.find('Paw /',[side,row])['part']));pairs.append((n,body if row=='0' else self.find('Haunch /',[side])['part']))
  for clip in m.clips:
   for t in (0.,.31):
    mats=m.matrices(clip,t);parts={p['part']:dict(p,positions=sum(m.posed(i,mats),[])) for i,p in enumerate(self.parts)}
    for a,b in pairs:self.assertTrue(parts_overlap(parts[a],parts[b]),(clip,t,a,b))
  detached=copy.deepcopy(self.find('Muzzle /'));detached['positions']=[v+(1. if i%3==0 else 0.) for i,v in enumerate(detached['positions'])];self.assertFalse(parts_overlap(detached,self.find('Head /')))
 def test_floor_neutral_endpoints_winding_strain_and_independent_motion(self):
  m=self.m;moved=set()
  for clip in m.clips:
   for step in range(9):
    mats=m.matrices(clip,step/8)
    for i,n in enumerate(m.names):
     posed=m.posed(i,mats);normals=m.posed(i,mats,True);self.assertGreaterEqual(min(q[1] for q in posed),-1e-7,(clip,n))
     displacement=max(math.dist(a,b) for a,b in zip(posed,m.vertices[i]))
     if n.startswith(('Paw /','Toe crease /')):self.assertLess(displacement,1e-7,n)
     elif displacement>.002:moved.add(n.split(' /')[0])
     for k in range(0,len(m.indices[i]),3):
      tri=m.indices[i][k:k+3];cosine,area=skin.winding_alignment(posed,normals,tri);self.assertGreater(cosine,0.,(clip,n));self.assertGreater(area,1e-12)
      for a,b in zip(tri,tri[1:]+tri[:1]):self.assertTrue(.75<math.dist(posed[a],posed[b])/math.dist(m.vertices[i][a],m.vertices[i][b])<1.3,(clip,n))
    if step in (0,8):
     for matrix in mats:self.assertLess(max(abs(a-b) for a,b in zip(matrix,IDENTITY)),1e-7)
  self.assertTrue({'Head','Ear','Leg','Tail','Ruff'}<=moved,moved)
 def test_source_cue_contract(self):
  for clip in self.m.doc['animations']:
   idle=clip['name'].endswith('.idle');self.assertEqual(clip['extras']['loopSuggested'],idle);self.assertEqual(clip['extras']['cueRelative'],not idle);self.assertEqual({c['target']['path'] for c in clip['channels']},{'rotation'})
if __name__=='__main__':unittest.main()
