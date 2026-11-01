#!/usr/bin/env python3
"""Independent stored armor contacts, shell topology and hollow-port checks."""
import os,unittest
from pathlib import Path
from armored_bug_sculpt import sculpture,SPECIES,HEIGHT
from armored_bug_glb import read_armored_bug,geometry_digest
from test_seahorse_glb import occupied_overlap
from sculpt_test_support import indexed_ray_intersections as rays
from sculpt_test_support import assert_closed_models,assert_model_shading
ray_intersections=rays  # Same stored-triangle barycentric oracle, with candidate indexing.
ROOT=Path(os.environ.get('CRYSTAL_ARMORED_BUG_TEST_MODELS',Path(__file__).resolve().parents[1]/'crates/crystal-voxel-view/models/battle_species'))
class ArmoredBugTests(unittest.TestCase):
 @classmethod
 def setUpClass(cls):cls.models={s:read_armored_bug(ROOT/(s+'.glb'),s) for s in SPECIES}
 def test_closed_connected_oriented_parts(self):assert_closed_models(self,self.models)
 def test_normals_match_stored_triangles(self):assert_model_shading(self,self.models)
 def test_regeneration_scale_and_distinct_anatomy(self):
  for s,m in self.models.items():
   ps=m['primitives'];self.assertEqual(geometry_digest(m),geometry_digest(sculpture(s)));self.assertEqual(min(v for p in ps for v in p['positions'][1::3]),0.)
   self.assertAlmostEqual(max(v for p in ps for v in p['positions'][1::3]),HEIGHT,places=6);self.assertEqual(len(ps),len({p['part'] for p in ps}));self.assertLess(sum(len(p['indices'])//3 for p in ps),15000)
   counts=(('Plate / navy',18),('Plate / teal',18),('Crown / navy',5),('Crown / teal',5),('Stem /',1),('Eye / narrow',2)) if s=='pineco' else (('Shell /',2),('Port /',4),('Port recess /',4),('Eye / ivory',2),('Chip / fitted',16),('Belt /',1))
   for prefix,n in counts:self.assertEqual(sum(p['part'].startswith(prefix) for p in ps),n,(s,prefix))
 def overlap(self,a,b):
  xl=max(min(a['positions'][::3]),min(b['positions'][::3]));xh=min(max(a['positions'][::3]),max(b['positions'][::3]));yl=max(min(a['positions'][1::3]),min(b['positions'][1::3]));yh=min(max(a['positions'][1::3]),max(b['positions'][1::3]));good=False
  for i in range(1,16):
   for j in range(1,20):
    x=xl+(xh-xl)*i/16;y=yl+(yh-yl)*j/20;aa=ray_intersections(a,x,y);bb=ray_intersections(b,x,y)
    if aa and bb and min(max(aa),max(bb))-max(min(aa),min(bb))>.0004:good=True;break
   if good:break
  self.assertTrue(good,(a['part'],b['part']))
 def test_pineco_armor_shares_seams_and_is_rooted(self):
  ps=self.models['pineco']['primitives']
  for p in ps:
   name=p['part']
   if name.startswith(('Plate / navy','Crown / navy','Stem /')):self.overlap(p,ps[0])
   if name.startswith(('Plate / teal','Crown / teal')):
    mate=next(q for q in ps if q['part']==name.replace('teal upper armor','navy underside').replace('teal tall plate','navy tall plate'))
    a={tuple(p['positions'][i:i+3]) for i in range(0,len(p['positions']),3)};b={tuple(mate['positions'][i:i+3]) for i in range(0,len(mate['positions']),3)}
    self.assertGreaterEqual(len(a&b),6)
 def test_forretress_shells_and_ports_have_actual_support(self):
  ps=self.models['forretress']['primitives']
  for p in ps:
   if p['part'].startswith(('Shell /','Port /','Belt /')):self.overlap(p,ps[0])
   elif p['part'].startswith('Port recess /'):
    support=next(q for q in ps if q['part']=='Port / hollow tapered red barrel '+' '.join(p['part'].split()[-2:]));self.overlap(p,support)
 def test_stone_shell_poles_are_closed_and_equator_is_open(self):
  ps=self.models['forretress']['primitives'];upper,lower=ps[1:3]
  cy=.5*HEIGHT
  a=ray_intersections(upper,0,cy);b=ray_intersections(lower,0,cy);self.assertFalse(a);self.assertFalse(b)
  roof=dict(upper);roof['positions']=[v for i in range(0,len(roof['positions']),3) for v in (roof['positions'][i],roof['positions'][i+2],roof['positions'][i+1])]
  hits=ray_intersections(roof,0,0);self.assertTrue(hits);self.assertGreater(max(hits)-min(hits),.01)
 def test_ports_and_eye_rims_have_real_unfilled_apertures(self):
  ps=self.models['forretress']['primitives']
  for p in ps:
   name=p['part']
   if name.startswith('Eye / ivory'):
    side=int(name.split()[-1]);cx=(min(p['positions'][::3])+max(p['positions'][::3]))/2;cy=(min(p['positions'][1::3])+max(p['positions'][1::3]))/2
    self.assertFalse(ray_intersections(p,cx,cy));self.assertTrue(ray_intersections(p,cx+.040,cy))
   elif name.startswith('Port /'):
    # Look down the authored barrel axis, rotating positions into that frame.
    side,j=map(int,name.split()[-2:]);axis=(side*.96,.04,.277 if j==0 else -.277)
    cx=(min(p['positions'][::3])+max(p['positions'][::3]))/2;cy=(min(p['positions'][1::3])+max(p['positions'][1::3]))/2;cz=(min(p['positions'][2::3])+max(p['positions'][2::3]))/2
    q=dict(p);q['positions']=[v for i in range(0,len(p['positions']),3) for v in ((p['positions'][i]-cx)*(-axis[2])+(p['positions'][i+2]-cz)*axis[0],p['positions'][i+1]-cy,(p['positions'][i]-cx)*axis[0]+(p['positions'][i+1]-cy)*axis[1]+(p['positions'][i+2]-cz)*axis[2])]
    self.assertFalse(ray_intersections(q,0,0));self.assertTrue(ray_intersections(q,.060,0))
 def test_fitted_eyes_and_chips_follow_real_skin(self):
  for s,m in self.models.items():
   ps=m['primitives']
   for p in ps:
    name=p['part'];side=name.split()[-1]
    if name.startswith(('Eye / fitted','Eye / broad','Slit /')):skin=ps[0];limit=.019
    elif name.startswith('Eye / narrow'):skin=next(q for q in ps if q['part']=='Eye / fitted ivory angry pine-cone eye '+side);limit=.005
    elif name.startswith('Eye / dark'):skin=next(q for q in ps if q['part']=='Eye / narrow fitted red iris '+side);limit=.004
    elif name.startswith('Eye / ivory'):skin=next(q for q in ps if q['part']=='Eye / broad dark round socket '+side);limit=.006
    elif name.startswith('Chip / fitted'):
     skin=ps[1] if name.split()[-3]=='True' else ps[2];limit=.005
     if name.split()[-2]=='True':
      skin=dict(skin);skin['positions']=[-v if i%3!=1 else v for i,v in enumerate(skin['positions'])]
    elif name.startswith('Chip / inset'):skin=next(q for q in ps if q['part']=='Chip / fitted stone depression '+' '.join(name.split()[-3:]));limit=.004
    else:continue
    for i in range(0,len(p['positions']),3):
     x,y,z=p['positions'][i:i+3]
     rear=name.startswith('Chip /') and name.split()[-2]=='True'
     if rear and name.startswith('Chip / fitted'):x=-x;z=-z
     hits=ray_intersections(skin,x,y);self.assertTrue(hits,(s,name));self.assertLess(abs(z-(min(hits) if rear and name.startswith('Chip / inset') else max(hits))),limit,(s,name))
 def attachments(self,s):
  ps=self.models[s]['primitives'];core=ps[0]
  for p in ps:
   n=p['part']
   if s=='pineco' and n.startswith(('Plate / navy','Crown / navy','Stem /')) or s=='forretress' and n.startswith(('Shell /','Port /','Belt /')):yield p,core
   elif n.startswith('Port recess /'):yield p,next(q for q in ps if q['part']=='Port / hollow tapered red barrel '+' '.join(n.split()[-2:]))
 def test_hollow_shells_armor_and_barrels_have_actual_occupied_roots(self):
  for s in self.models:
   for child,parent in self.attachments(s):self.assertTrue(occupied_overlap(child,parent),(s,child['part']))
 def test_white_gaze_is_visible_ahead_of_the_core(self):
  for s,m in self.models.items():
   core=m['primitives'][0]
   for side in (-1,1):
    eye=next(p for p in m['primitives'] if p['part'].startswith('Eye / fitted ivory' if s=='pineco' else 'Eye / ivory') and p['part'].endswith(' '+str(side)));visible=0
    for k in range(0,len(eye['indices']),3):
     a,b,c=tri=[eye['positions'][i*3:i*3+3] for i in eye['indices'][k:k+3]]
     if (b[0]-a[0])*(c[1]-a[1])-(b[1]-a[1])*(c[0]-a[0])<=1e-10:continue
     x,y,z=[sum(v[j] for v in tri)/3 for j in range(3)];hits=rays(core,x,y)
     if hits and z>max(hits)+.001:visible+=1
    self.assertGreaterEqual(visible,3,(s,side))
if __name__=='__main__':unittest.main()
