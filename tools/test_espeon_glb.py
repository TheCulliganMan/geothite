#!/usr/bin/env python3
"""Sun-cat anatomy, fitted ruby lens, planted paws and occupied independent fork tips."""
import copy,math,unittest
from pathlib import Path
import espeon_glb as author
import test_gengar_glb as skin
from skin_glb_test_support import IDENTITY
from sculpt_test_support import assert_closed_models,assert_model_shading,parts_overlap,indexed_ray_intersections as rays
ROOT=Path(__file__).resolve().parents[1]/'crates/crystal-voxel-view/models/battle_species'
ASSET=ROOT/'espeon.glb'
class EspeonTests(unittest.TestCase):
 @classmethod
 def setUpClass(cls):
  cls.blob=ASSET.read_bytes();cls.m=skin.Model(cls.blob);cls.parts=cls.m.neutral_model()['primitives']
 def find(self,prefix,ending=None):return next(p for p in self.parts if p['part'].startswith(prefix) and (ending is None or p['part'].split()[-len(ending):]==ending))
 def test_canonical_schema_limits_and_distinctive_anatomy(self):
  m=self.m;self.assertEqual(self.blob,author.export_espeon());self.assertEqual(len(m.parts),44);self.assertEqual(len(m.joints),15)
  self.assertFalse((ROOT/'espeon.mesh.json').exists())
  self.assertLessEqual(sum(map(len,m.vertices)),20000);self.assertLessEqual(sum(map(len,m.indices)),120000);self.assertLessEqual(len(m.doc['accessors']),256);self.assertLess(len(self.blob),1024*1024)
  points=[p for v in m.vertices for p in v];self.assertEqual(min(p[1] for p in points),0.);self.assertAlmostEqual(max(p[1] for p in points),1.0,places=6)
  for prefix,count in [('Ear /',2),('Ear inset /',2),('Paw /',4),('Leg /',4),('Haunch /',2),('Shoulder /',2),('Cheek wisp /',2),('Gem /',1),('Tail /',1),('Tail hub /',1),('Tail fork /',2)]:self.assertEqual(sum(n.startswith(prefix) for n in m.names),count)
  for matrix in m.matrices():self.assertLess(max(abs(a-b) for a,b in zip(matrix,IDENTITY)),1e-7)
 def test_closed_outward_solids(self):assert_closed_models(self,{'espeon':self.m.neutral_model()})
 def test_actual_face_normals(self):assert_model_shading(self,{'espeon':self.m.neutral_model()})
 def test_fitted_eyes_ear_insets_and_sparkles(self):
  for mark in self.parts:
   n=mark['part'];side=[n.split()[-1]]
   if n.startswith('Eye /'):parent=self.find('Head /')
   elif n.startswith('Ear inset /'):parent=self.find('Ear /',side)
   elif n.startswith('Iris /'):parent=self.find('Eye /',side)
   elif n.startswith('Glint /'):parent=self.find('Iris /',side)
   elif n.startswith('Gem /'):parent=self.find('Head /')
   elif n.startswith('Gem glint /'):parent=self.find('Gem /')
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
   elif n.startswith(('Body /','Haunch /','Shoulder /')):self.assertEqual(owners,{1},n)
   elif n.startswith(('Ear /','Ear inset /')):self.assertEqual(owners,{4 if n.endswith('-1') else 5},n)
   elif n.startswith('Leg /'):
    side,row=map(int,n.split()[-2:]);self.assertEqual(owners,{0,1,11+(0 if side<0 else 2)+row})
   elif n.startswith('Neck /'):self.assertEqual(owners,{1,2,3})
   elif n.startswith('Tail hub /'):self.assertEqual(owners,{8},n)
   elif n.startswith('Tail fork /'):self.assertEqual(owners,{8,9 if n.endswith('-1') else 10},n)
   elif n.startswith('Tail /'):self.assertEqual(owners,{1,6,7,8},n)
   else:self.assertEqual(owners,{3},n)
  self.assertEqual({(math.copysign(1.,p[0]),math.copysign(1.,p[2])) for i,n in enumerate(m.names) if n.startswith('Paw /') for p in m.vertices[i] if p[1]<1e-7},{(-1.,-1.),(-1.,1.),(1.,-1.),(1.,1.)})
 def test_occupied_neck_shoulders_ears_fork_hub_and_leg_roots_survive_motion(self):
  m=self.m;body=self.find('Body /')['part'];head=self.find('Head /')['part'];neck=self.find('Neck /')['part'];hub=self.find('Tail hub /')['part']
  pairs=[(neck,body),(neck,head),(self.find('Muzzle /')['part'],head),(self.find('Tail /')['part'],body),(hub,self.find('Tail /')['part'])]
  for p in self.parts:
   n=p['part']
   if n.startswith(('Ear /','Cheek wisp /')):pairs.append((n,head))
   elif n.startswith('Tail fork /'):pairs.append((n,hub))
   elif n.startswith(('Haunch /','Back quill /','Rump quill /')):pairs.append((n,body))
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
  self.assertTrue({'Head','Ear','Leg','Tail','Tail fork'}<=moved,moved)
 def test_fork_tips_articulate_independently_from_common_hub(self):
  m=self.m;mats=m.matrices('attack',.31)
  for i,n in enumerate(m.names):
   if not n.startswith('Tail fork /'):continue
   self.assertGreater(max(math.dist(m.point(i,j,mats),skin.transform(mats[8],p)) for j,p in enumerate(m.vertices[i])),.003,n)
 def test_source_cue_contract(self):
  for clip in self.m.doc['animations']:
   idle=clip['name'].endswith('.idle');self.assertEqual(clip['extras']['loopSuggested'],idle);self.assertEqual(clip['extras']['cueRelative'],not idle);self.assertEqual({c['target']['path'] for c in clip['channels']},{'rotation'})
if __name__=='__main__':unittest.main()
