#!/usr/bin/env python3
"""Stored flowing ghost topology, curl seams, real necklace and fitted face."""
import json,math,unittest
from pathlib import Path
from misdreavus_sculpt import sculpture,HEIGHT
from misdreavus_glb import read_misdreavus,geometry_digest
from sculpt_test_support import assert_closed_models,assert_model_shading
from test_axolotl_sculpt import ray_intersections
ROOT=Path(__file__).resolve().parents[1]/'crates/crystal-voxel-view/models/battle_species'
class MisdreavusTests(unittest.TestCase):
 @classmethod
 def setUpClass(cls):cls.m=read_misdreavus(ROOT/'misdreavus.glb');cls.ps=cls.m['primitives']
 def find(self,prefix):return next(p for p in self.ps if p['part'].startswith(prefix))
 def test_closed_connected_outward_topology(self):assert_closed_models(self,{'misdreavus':self.m})
 def test_finite_unit_normals_follow_actual_faces(self):assert_model_shading(self,{'misdreavus':self.m})
 def test_exact_regeneration_height_and_feature_budget(self):
  self.assertEqual(geometry_digest(self.m),geometry_digest(sculpture()));self.assertEqual(min(v for p in self.ps for v in p['positions'][1::3]),0.)
  self.assertAlmostEqual(max(v for p in self.ps for v in p['positions'][1::3]),HEIGHT,places=6);self.assertLess(sum(len(p['indices'])//3 for p in self.ps),32000)
  self.assertEqual(len(self.ps),len({p['part'] for p in self.ps}))
  for prefix,n in (('Hair /',6),('Hair tip /',5),('Necklace gem /',5),('Necklace cord /',2),('Eye / fitted',2),('Eye / warm',2),('Gem glint /',10)):self.assertEqual(sum(p['part'].startswith(prefix) for p in self.ps),n)
 def has_overlap(self,a,b):
  xl=max(min(a['positions'][::3]),min(b['positions'][::3]));xh=min(max(a['positions'][::3]),max(b['positions'][::3]));yl=max(min(a['positions'][1::3]),min(b['positions'][1::3]));yh=min(max(a['positions'][1::3]),max(b['positions'][1::3]))
  if xh<=xl or yh<=yl:return False
  for i in range(1,19):
   for j in range(1,23):
    aa=ray_intersections(a,xl+(xh-xl)*i/19,yl+(yh-yl)*j/23);bb=ray_intersections(b,xl+(xh-xl)*i/19,yl+(yh-yl)*j/23)
    if aa and bb and min(max(aa),max(bb))-max(min(aa),min(bb))>.00025:return True
  return False
 def test_head_and_every_swept_lock_have_real_volume_support(self):
  self.assertTrue(self.has_overlap(self.ps[0],self.ps[1]))
  for p in self.ps:
   if p['part'].startswith('Hair /'):self.assertTrue(self.has_overlap(p,self.ps[1]),p['part'])
 def test_pink_curls_share_full_closed_section_with_blue_lock(self):
  for p in self.ps:
   if not p['part'].startswith('Hair tip /'):continue
   name=p['part'].removeprefix('Hair tip / tapered pink curl ');base=self.find('Hair / swept volumetric '+name)
   vertices=lambda q:{tuple(q['positions'][i:i+3]) for i in range(0,len(q['positions']),3)}
   shared=vertices(p)&vertices(base);self.assertEqual(len(shared),24,p['part'])
   center=tuple(sum(v[a] for v in shared)/len(shared) for a in range(3));vectors=[tuple(v[a]-center[a] for a in range(3)) for v in shared]
   area2=max(sum(c*c for c in (a[1]*b[2]-a[2]*b[1],a[2]*b[0]-a[0]*b[2],a[0]*b[1]-a[1]*b[0])) for a in vectors for b in vectors);self.assertGreater(area2,1e-9)
   self.assertGreater(max(p['positions'][2::3])-min(p['positions'][2::3]),.01)
 def test_necklace_has_graduated_orbs_and_real_support(self):
  gems=[p for p in self.ps if p['part'].startswith('Necklace gem /')];width=[max(p['positions'][::3])-min(p['positions'][::3]) for p in gems]
  self.assertGreater(width[2],width[1]*1.5);self.assertGreater(width[1],width[0]*1.4)
  chain=self.find('Necklace cord / curved front')
  for p in gems:
   self.assertTrue(self.has_overlap(p,self.ps[0]) or self.has_overlap(p,self.ps[1]),p['part']);self.assertTrue(self.has_overlap(p,chain),p['part'])
  self.assertTrue(self.has_overlap(self.find('Necklace cord / fitted rear'),self.ps[1]))
 def test_all_face_and_glint_layers_follow_actual_support(self):
  for p in self.ps:
   name=p['part'];support=None;s=name.split()[-1]
   if name.startswith(('Eye / fitted','Mouth /')):support=self.ps[1]
   elif name.startswith('Eye / warm'):support=self.find('Eye / fitted dark almond socket '+s)
   elif name.startswith('Iris /'):support=self.find('Eye / warm gold sclera '+s)
   elif name.startswith('Pupil /'):support=self.find('Iris / large ruby oval '+s)
   elif name.startswith('Eye glint /'):support=self.find('Pupil / dark inward vertical oval '+s)
   elif name.startswith('Gem glint /'):support=self.find('Necklace gem / graduated ruby orb '+s)
   if support:
    for x,y,z in {tuple(p['positions'][i:i+3]) for i in range(0,len(p['positions']),3)}:
     hits=ray_intersections(support,x,y);self.assertTrue(hits,(name,x,y));self.assertLess(abs(z-max(hits)),.008)
 def test_large_front_eye_and_smile_triangles_clear_actual_head(self):
  for p in self.ps:
   if not p['part'].startswith(('Eye / fitted','Mouth /')):continue
   for k in range(0,len(p['indices']),3):
    ids=p['indices'][k:k+3];points=[p['positions'][3*i:3*i+3] for i in ids];a,b,c=points
    nz=(b[0]-a[0])*(c[1]-a[1])-(b[1]-a[1])*(c[0]-a[0])
    if nz<1e-10:continue
    x,y,z=[sum(q[axis] for q in points)/3 for axis in range(3)];hits=ray_intersections(self.ps[1],x,y)
    self.assertTrue(hits);self.assertGreater(z-max(hits),.0001,(p['part'],k//3,z-max(hits)))
 def test_five_soft_hem_lobes_retain_closed_grounded_outline(self):
  body=self.ps[0];vertices={tuple(body['positions'][i:i+3]) for i in range(0,len(body['positions']),3)}
  low=[p for p in vertices if p[1]<.015];self.assertGreaterEqual(len(low),5)
  angles=sorted(math.atan2(x,z) for x,y,z in low);groups=[]
  for a in angles:
   if not groups or a-groups[-1][-1]>.25:groups.append([a])
   else:groups[-1].append(a)
  if len(groups)>1 and groups[0][0]+math.tau-groups[-1][-1]<.25:groups[0]=groups[-1]+groups[0];groups.pop()
  self.assertEqual(len(groups),5)
if __name__=='__main__':unittest.main()
