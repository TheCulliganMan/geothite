#!/usr/bin/env python3
"""Stored genetic anatomy, fitted lenses, planted feet and articulated heavy tail."""
import copy,math,unittest
from pathlib import Path
import mewtwo_glb as author
import test_gengar_glb as skin
from skin_glb_test_support import IDENTITY
from sculpt_test_support import assert_closed_models,assert_model_shading,parts_overlap,indexed_ray_intersections as rays,crossed_faces
ROOT=Path(__file__).resolve().parents[1]/'crates/crystal-voxel-view/models/battle_species'
class MewtwoTests(unittest.TestCase):
 @classmethod
 def setUpClass(cls):
  cls.blob=(ROOT/'mewtwo.glb').read_bytes();cls.m=skin.Model(cls.blob);cls.parts=cls.m.neutral_model()['primitives']
 def find(self,prefix,side=None):return next(p for p in self.parts if p['part'].startswith(prefix) and (side is None or p['part'].split()[-1]==str(side)))
 def test_schema_floor_and_anatomy(self):
  m=self.m;self.assertEqual(self.blob,author.export_mewtwo());self.assertEqual(len(m.parts),52);self.assertEqual(len(m.joints),17);self.assertFalse((ROOT/'mewtwo.mesh.json').exists())
  self.assertLessEqual(sum(map(len,m.vertices)),20000);self.assertLessEqual(sum(map(len,m.indices)),120000);self.assertLessEqual(len(m.doc['accessors']),256);self.assertLess(len(self.blob),1024*1024)
  points=[p for v in m.vertices for p in v];self.assertEqual(min(p[1] for p in points),0.);self.assertAlmostEqual(max(p[1] for p in points),1.45,places=6)
  for prefix,count in [('Horn /',2),('Finger /',6),('Fingertip /',6),('Foot /',2),('Toe /',6),('Tail /',1),('Back tube /',1)]:self.assertEqual(sum(n.startswith(prefix) for n in m.names),count)
 def test_closed_outward_solids(self):assert_closed_models(self,{'mewtwo':self.m.neutral_model()})
 def test_actual_face_normals(self):assert_model_shading(self,{'mewtwo':self.m.neutral_model()})
 def test_fitted_lenses_clear_stored_parents(self):
  for mark in self.parts:
   n=mark['part'];side=n.split()[-1]
   if n.startswith(('Eye socket /','Nostril /')):parent=self.find('Head /')
   elif n.startswith('Eye /'):parent=self.find('Eye socket /',side)
   elif n.startswith('Iris /'):parent=self.find('Eye /',side)
   elif n.startswith(('Pupil /','Glint /')):parent=self.find('Iris /',side)
   else:continue
   for k in range(0,len(mark['indices']),3):
    a,b,c=[mark['positions'][3*j:3*j+3] for j in mark['indices'][k:k+3]]
    if (b[0]-a[0])*(c[1]-a[1])-(b[1]-a[1])*(c[0]-a[0])<=1e-14:continue
    samples=[tuple((a[j]+b[j]+c[j])/3 for j in range(3)),*[tuple((u[j]+v[j])/2 for j in range(3)) for u,v in [(a,b),(b,c),(c,a)]]]
    for q in samples:
     hits=rays(parent,q[0],q[1]);self.assertTrue(hits,n);self.assertGreater(q[2]-max(hits),.00005,(n,q))
 def test_occupied_anatomical_roots_survive_motion(self):
  find=lambda p,s=None:self.find(p,s)['part'];pairs=[(find('Neck /'),find('Body /')),(find('Head /'),find('Neck /')),(find('Back tube /'),find('Body /')),(find('Back tube /'),find('Head /')),(find('Tail /'),find('Pelvis /')),(find('Tail tip /'),find('Tail /'))]
  for side in (-1,1):
   pairs += [(find('Horn /',side),find('Head /')),(find('Upper arm /',side),find('Body /')),(find('Forearm /',side),find('Upper arm /',side)),(find('Hand /',side),find('Forearm /',side)),(find('Haunch /',side),find('Body /')),(find('Shin /',side),find('Haunch /',side)),(find('Foot /',side),find('Shin /',side))]
   for j in range(3):
    f=next(n for n in self.m.names if n.startswith('Finger /') and n.split()[-2:]==[str(side),str(j)]);tip=next(n for n in self.m.names if n.startswith('Fingertip /') and n.split()[-2:]==[str(side),str(j)])
    pairs += [(f,find('Hand /',side)),(tip,f)]
  for clip in self.m.clips:
   for t in (0.,.18,.55):
    mats=self.m.matrices(clip,t);parts={p['part']:dict(p,positions=sum(self.m.posed(i,mats),[])) for i,p in enumerate(self.parts)}
    for a,b in pairs:self.assertTrue(parts_overlap(parts[a],parts[b]),(clip,t,a,b))
  tip=copy.deepcopy(self.find('Fingertip /'));tip['positions']=[v+(1. if i%3==0 else 0.) for i,v in enumerate(tip['positions'])];self.assertFalse(parts_overlap(tip,self.find('Finger /')))
 def test_planted_pads_neutral_endpoints_and_motion(self):
  m=self.m;moved=set()
  for clip in m.clips:
   for step in range(17):
    mats=m.matrices(clip,step/16)
    for i,n in enumerate(m.names):
     points=m.posed(i,mats);self.assertGreaterEqual(min(q[1] for q in points),-1e-7,(clip,n))
     if max(math.dist(a,b) for a,b in zip(points,m.vertices[i]))>.002:moved.add(n.split(' /')[0])
     if n.startswith(('Foot /','Toe /')):self.assertLess(max(math.dist(a,b) for a,b in zip(points,m.vertices[i])),1e-7)
    if step in (0,16):
     for matrix in mats:self.assertLess(max(abs(a-b) for a,b in zip(matrix,IDENTITY)),1e-7)
  self.assertTrue({'Body','Neck','Head','Upper arm','Forearm','Hand','Tail','Tail tip'}<=moved,moved)
 def test_winding_strain_and_nonintersecting_anatomy(self):
  m=self.m
  for p in self.parts:
   if p['part'].startswith(('Head /','Horn /','Foot /','Tail /','Upper arm /','Shin /')):self.assertIsNone(crossed_faces(p),p['part'])
  for clip in m.clips:
   for t in (.18,.31,.55,.80):
    mats=m.matrices(clip,t)
    for i,n in enumerate(m.names):
     posed=m.posed(i,mats);normals=m.posed(i,mats,True)
     for k in range(0,len(m.indices[i]),3):
      tri=m.indices[i][k:k+3];cosine,area=skin.winding_alignment(posed,normals,tri);self.assertGreater(cosine,0.,(clip,n));self.assertGreater(area,1e-12)
      for a,b in zip(tri,tri[1:]+tri[:1]):self.assertTrue(.75<math.dist(posed[a],posed[b])/math.dist(m.vertices[i][a],m.vertices[i][b])<1.30,(clip,n))
 def test_source_cue_contract(self):
  for clip in self.m.doc['animations']:
   idle=clip['name'].endswith('.idle');self.assertEqual(clip['extras']['loopSuggested'],idle);self.assertEqual(clip['extras']['cueRelative'],not idle);self.assertEqual({c['target']['path'] for c in clip['channels']},{'rotation'})
if __name__=='__main__':unittest.main()
