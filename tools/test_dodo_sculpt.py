#!/usr/bin/env python3
"""Independent stored runner-bird anatomy, solid roots and fitted eyes."""
import json,math,os,unittest
from pathlib import Path
from dodo_sculpt import sculpture,SPECIES,HEIGHT,POSES
from sculpt_test_support import assert_closed_models,assert_model_shading
from test_slime_sculpt import ray_intersections
import test_magnet_sculpt as joins
ROOT=Path(os.environ.get('DODO_TEST_MODELS',Path(__file__).resolve().parents[1]/'crates/crystal-voxel-view/models/battle_species'))
class DodoTests(unittest.TestCase):
 @classmethod
 def setUpClass(cls):cls.models={s:json.loads((ROOT/(s+'.mesh.json')).read_bytes()) for s in SPECIES}
 def find(self,ps,prefix,ending=None):return next(p for p in ps if p['part'].startswith(prefix) and (ending is None or p['part'].endswith(' '+str(ending))))
 def test_closed_connected_outward_geometry(self):assert_closed_models(self,self.models)
 def test_finite_unit_normals_and_actual_winding(self):assert_model_shading(self,self.models)
 def test_recipe_grounding_height_budget(self):
  for s,m in self.models.items():
   self.assertEqual(m,sculpture(s));ps=m['primitives'];self.assertEqual(min(v for p in ps for v in p['positions'][1::3]),0);self.assertAlmostEqual(max(v for p in ps for v in p['positions'][1::3]),HEIGHT,places=6);self.assertLess(sum(len(p['indices'])//3 for p in ps),65000 if s=='doduo' else 95000);self.assertEqual(len(ps),len({p['part'] for p in ps}))
 def test_species_specific_heads_feet_crests_and_tail(self):
  for s,m in self.models.items():
   ps=m['primitives'];n=2 if s=='doduo' else 3
   counts=(('Body /',1),('Head /',n),('Neck /',n),('Upper beak /',n),('Lower beak /',n),('Eye /',2*n),('Leg /',2),('Foot /',2),('Toe /',6),('Claw /',6),('Hind toe /',2),('Hind claw /',2),('Crest /',0 if n==2 else 6),('Tail /',0 if n==2 else 3),('Underbelly /',0 if n==2 else 1),('Glint /',4 if n==2 else 0),('Pupil /',0 if n==2 else 6),('Tongue /',0 if n==2 else 3))
   for prefix,count in counts:self.assertEqual(sum(p['part'].startswith(prefix) for p in ps),count,(s,prefix))
 def test_heads_necks_beaks_crests_and_tail_have_actual_solid_roots(self):
  for s,m in self.models.items():
   ps=m['primitives'];body=self.find(ps,'Body /');pairs=[]
   for j in range(2 if s=='doduo' else 3):
    h=self.find(ps,'Head /',j);neck=self.find(ps,'Neck /',j);pairs += [(body,neck),(neck,h),(h,self.find(ps,'Upper beak /',j)),(h,self.find(ps,'Lower beak /',j))]
    if s=='dodrio':pairs += [(h,self.find(ps,'Crest /',str(j)+' '+str(k))) for k in (0,1)]
   if s=='dodrio':pairs += [(body,p) for p in ps if p['part'].startswith(('Tail /','Underbelly /'))]
   for a,b in pairs:self.assertTrue(joins.MagnetTests.overlap(self,a,b),(s,a['part'],b['part']))
 def test_long_legs_join_feet_and_all_clawed_digits(self):
  for s,m in self.models.items():
   ps=m['primitives'];body=self.find(ps,'Underbelly /') if s=='dodrio' else self.find(ps,'Body /');pairs=[]
   for sign in (-1,1):
    leg=self.find(ps,'Leg /',sign);foot=self.find(ps,'Foot /',sign);pairs += [(body,leg),(leg,foot)]
    for k in range(3):
     suffix=str(sign)+' '+str(k);toe=self.find(ps,'Toe /',suffix);pairs += [(foot,toe),(toe,self.find(ps,'Claw /',suffix))]
    toe=self.find(ps,'Hind toe /',sign);pairs += [(foot,toe),(toe,self.find(ps,'Hind claw /',sign))]
   for a,b in pairs:self.assertTrue(joins.MagnetTests.overlap(self,a,b),(s,a['part'],b['part']))
 def markings(self):
  for s,m in self.models.items():
   ps=m['primitives']
   for p in ps:
    n=p['part'];parent=None
    if not n.startswith(('Eye /','Pupil /','Glint /')):continue
    j=int(n.split()[-2]);side=n.split()[-1];parent=self.find(ps,'Head /',j) if n.startswith('Eye /') else self.find(ps,'Eye /',str(j)+' '+side)
    yaw,pitch=POSES[s][j][-2:];a,b=math.cos(yaw),math.sin(yaw);c,t=math.cos(pitch),math.sin(pitch)
    def unturn(q):
     r=dict(q);pos=[]
     for i in range(0,len(q['positions']),3):
      x,y,z=q['positions'][i:i+3];x,z=x*a-z*b,z*a+x*b;pos.extend((x,y*c-z*t,z*c+y*t))
     r['positions']=pos;return r
    yield s,unturn(p),unturn(parent)
 def test_eye_vertices_follow_actual_stored_parents(self):
  for s,p,parent in self.markings():
   for i in range(0,len(p['positions']),3):
    x,y,z=p['positions'][i:i+3];h=ray_intersections(parent,x,y);self.assertTrue(h,(s,p['part']));self.assertLess(abs(z-max(h)),.017,(s,p['part']))
 def test_front_marking_triangles_clear_actual_support(self):
  for s,p,parent in self.markings():
   for k in range(0,len(p['indices']),3):
    a,b,c=[p['positions'][i*3:i*3+3] for i in p['indices'][k:k+3]]
    u=[b[j]-a[j] for j in range(3)];v=[c[j]-a[j] for j in range(3)];normal=(u[1]*v[2]-u[2]*v[1],u[2]*v[0]-u[0]*v[2],u[0]*v[1]-u[1]*v[0]);area=math.sqrt(sum(t*t for t in normal))
    # Undoing a quantized 3-D head turn leaves tiny XY error on vertical walls.
    # Classify cap faces by their relative projected area, not that wall noise.
    if normal[2]<=1e-10 or normal[2]/area<1e-4:continue
    x,y,z=[(a[j]+b[j]+c[j])/3 for j in range(3)];h=ray_intersections(parent,x,y);self.assertTrue(h);self.assertGreater(z-max(h),.0001,(s,p['part']))
if __name__=='__main__':unittest.main()
