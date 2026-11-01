#!/usr/bin/env python3
"""Swept moth wings, coherent veins, fitted gaze and tucked insect leg checks."""
import copy,math,unittest
from pathlib import Path
import venomoth_glb as author
import test_gengar_glb as skin
from skin_glb_test_support import IDENTITY
from sculpt_test_support import assert_closed_models,assert_model_shading,parts_overlap,indexed_ray_intersections as rays
ROOT=Path(__file__).resolve().parents[1]/'crates/crystal-voxel-view/models/battle_species'
ASSET=ROOT/'venomoth.glb'
class VenomothTests(unittest.TestCase):
 @classmethod
 def setUpClass(cls):
  cls.blob=ASSET.read_bytes();cls.m=skin.Model(cls.blob);cls.parts=cls.m.neutral_model()['primitives']
 def find(self,prefix,ending=None):return next(p for p in self.parts if p['part'].startswith(prefix) and (ending is None or p['part'].split()[-len(ending):]==ending))
 def test_canonical_schema_limits_and_distinctive_anatomy(self):
  m=self.m;self.assertEqual(self.blob,author.export_venomoth());self.assertEqual(len(m.parts),53);self.assertEqual(len(m.joints),17)
  self.assertFalse((ROOT/'venomoth.mesh.json').exists())
  self.assertLessEqual(sum(map(len,m.vertices)),20000);self.assertLessEqual(sum(map(len,m.indices)),120000);self.assertLessEqual(len(m.doc['accessors']),256);self.assertLess(len(self.blob),1024*1024)
  points=[p for v in m.vertices for p in v];self.assertEqual(min(p[1] for p in points),0.);self.assertAlmostEqual(max(p[1] for p in points),1.16,places=6)
  for prefix,count in [('Eye /',2),('Pupil /',2),('Glint /',2),('Antenna /',3),('Mandible /',2),('Wing /',4),('Wing vein /',20),('Leg /',6),('Foot /',6),('Abdomen band /',3)]:self.assertEqual(sum(n.startswith(prefix) for n in m.names),count)
  for matrix in m.matrices():self.assertLess(max(abs(a-b) for a,b in zip(matrix,IDENTITY)),1e-7)
 def test_closed_outward_solids(self):assert_closed_models(self,{'venomoth':self.m.neutral_model()})
 def test_actual_face_normals(self):assert_model_shading(self,{'venomoth':self.m.neutral_model()})
 def test_fitted_round_pupils_and_glints(self):
  for mark in self.parts:
   n=mark['part'];side=[n.split()[-1]]
   if n.startswith('Pupil /'):parent=self.find('Eye /',side)
   elif n.startswith('Glint /'):parent=self.find('Pupil /',side)
   else:continue
   vs=[mark['positions'][i:i+3] for i in range(0,len(mark['positions']),3)]
   for k in range(0,len(mark['indices']),3):
    a,b,c=[vs[j] for j in mark['indices'][k:k+3]]
    if (b[0]-a[0])*(c[1]-a[1])-(b[1]-a[1])*(c[0]-a[0])<=1e-14:continue
    q=[(a[j]+b[j]+c[j])/3 for j in range(3)];hits=rays(parent,q[0],q[1]);self.assertTrue(hits,n);self.assertGreater(q[2]-max(hits),.0001,n)
 def test_anatomical_owners_and_coherent_wing_veins(self):
  m=self.m
  for i,n in enumerate(m.names):
   owners={j for js,ws in zip(m.ids[i],m.weights[i]) for j,w in zip(js,ws) if w}
   if n.startswith('Thorax /'):expected={1}
   elif n.startswith(('Abdomen /','Abdomen band /')):expected={3}
   elif n.startswith('Antenna /'):expected={8 if 'central' in n else 9 if n.endswith('-1') else 10}
   elif n.startswith(('Wing /','Wing vein /')):
    side,rear=map(int,n.split()[-4:-2] if n.startswith('Wing vein /') else n.split()[-2:]);expected={4+(0 if side<0 else 2)+rear}
   elif n.startswith(('Leg /','Foot /')):
    side,j=map(int,n.split()[-2:]);expected={11+(0 if side<0 else 3)+j}
    if n.startswith('Leg /'):expected.add(1)
   else:expected={2}
   self.assertEqual(owners,expected,n)
 def test_occupied_anatomical_roots_survive_motion(self):
  m=self.m;body=self.find('Thorax /')['part'];head=self.find('Head /')['part'];abdomen=self.find('Abdomen /')['part'];pairs=[(head,body),(abdomen,body)]
  for p in self.parts:
   n=p['part']
   if n.startswith(('Eye /','Antenna /','Mandible /')):pairs.append((n,head))
   if n.startswith(('Wing /','Leg /')):pairs.append((n,body))
   if n.startswith('Foot /'):pairs.append((n,self.find('Leg /',n.split()[-2:])['part']))
  for clip in m.clips:
   for t in (0.,.31):
    mats=m.matrices(clip,t);parts={p['part']:dict(p,positions=sum(m.posed(i,mats),[])) for i,p in enumerate(self.parts)}
    for a,b in pairs:self.assertTrue(parts_overlap(parts[a],parts[b]),(clip,t,a,b))
  detached=copy.deepcopy(self.find('Head /'));detached['positions']=[v+(1. if i%3==0 else 0.) for i,v in enumerate(detached['positions'])];self.assertFalse(parts_overlap(detached,self.find('Thorax /')))
 def test_veins_and_zigzag_bands_are_seated_on_stored_surfaces(self):
  m=self.m
  for i,n in enumerate(m.names):
   if n.startswith('Wing vein /'):
    side,rear,face,j=n.split()[-4:];parent=self.find('Wing /',[side,rear]);depths=[]
    for q in m.vertices[i]:
     hits=rays(parent,q[0],q[1]);self.assertTrue(hits,n)
     depths.append(q[2]-max(hits) if face=='1' else min(hits)-q[2])
    self.assertLess(max(depths),.005);self.assertLess(min(depths),.0001)
   elif n.startswith('Abdomen band /'):
    self.assertTrue(parts_overlap(self.parts[i],self.find('Abdomen /')),n)
 def test_floor_neutral_endpoints_winding_strain_and_independent_motion(self):
  m=self.m;moved=set()
  for clip in m.clips:
   for step in range(9):
    mats=m.matrices(clip,step/8)
    for i,n in enumerate(m.names):
     posed=m.posed(i,mats);normals=m.posed(i,mats,True);self.assertGreaterEqual(min(q[1] for q in posed),-1e-7,(clip,n))
     displacement=max(math.dist(a,b) for a,b in zip(posed,m.vertices[i]))
     if displacement>.002:moved.add(n.split(' /')[0])
     for k in range(0,len(m.indices[i]),3):
      tri=m.indices[i][k:k+3];cosine,area=skin.winding_alignment(posed,normals,tri);self.assertGreater(cosine,0.,(clip,n));self.assertGreater(area,1e-12)
      for a,b in zip(tri,tri[1:]+tri[:1]):self.assertTrue(.75<math.dist(posed[a],posed[b])/math.dist(m.vertices[i][a],m.vertices[i][b])<1.3,(clip,n))
    if step in (0,8):
     for matrix in mats:self.assertLess(max(abs(a-b) for a,b in zip(matrix,IDENTITY)),1e-7)
  self.assertTrue({'Wing','Antenna','Head','Leg'}<=moved,moved)
 def test_source_cue_contract(self):
  for clip in self.m.doc['animations']:
   idle=clip['name'].endswith('.idle');self.assertEqual(clip['extras']['loopSuggested'],idle);self.assertEqual(clip['extras']['cueRelative'],not idle);self.assertEqual({c['target']['path'] for c in clip['channels']},{'rotation'})
if __name__=='__main__':unittest.main()
