#!/usr/bin/env python3
"""Independent stored anatomy, real jaw opening, wing roots and supported details."""
import json,os,unittest
from pathlib import Path
from aerodactyl_sculpt import sculpture,HEIGHT
from sculpt_test_support import assert_closed_models,assert_model_shading
from sculpt_test_support import indexed_ray_intersections as ray_intersections
import test_gligar_sculpt as joins
ROOT=Path(__file__).resolve().parents[1]/'crates/crystal-voxel-view/models/battle_species'
def rotate(p,up):
 q=dict(p);q['positions']=[v for i in range(0,len(p['positions']),3) for v in ((p['positions'][i],-p['positions'][i+2],p['positions'][i+1]) if up else (p['positions'][i],p['positions'][i+2],-p['positions'][i+1]))];return q
class AerodactylTests(unittest.TestCase):
 @classmethod
 def setUpClass(cls):
  from aerodactyl_glb import read_aerodactyl
  cls.m=read_aerodactyl(Path(os.environ.get('CRYSTAL_AERODACTYL_TEST_GLB',str(ROOT/'aerodactyl.glb'))));cls.ps=cls.m['primitives']
 def find(self,prefix):return next(p for p in self.ps if p['part'].startswith(prefix))
 overlap=joins.GligarTests.overlap
 def test_closed_connected_outward_anatomy(self):assert_closed_models(self,{'aerodactyl':self.m})
 def test_finite_unit_normals_follow_actual_faces(self):assert_model_shading(self,{'aerodactyl':self.m})
 def test_recipe_parity_grounding_height_budget_and_identity(self):
  from aerodactyl_glb import geometry_digest
  self.assertEqual(geometry_digest(self.m),geometry_digest(sculpture()));self.assertEqual(min(v for p in self.ps for v in p['positions'][1::3]),0.)
  self.assertAlmostEqual(max(v for p in self.ps for v in p['positions'][1::3]),HEIGHT,places=6);self.assertLessEqual(sum(len(p['positions'])//3 for p in self.ps),20000);self.assertLessEqual(sum(len(p['indices'])//3 for p in self.ps),40000);self.assertEqual(len(self.ps),len({p['part'] for p in self.ps}))
  for prefix,n in (('Skull /',1),('Jaw /',1),('Jaw hinge /',2),('Skull horn /',2),('Tooth /',12),('Wing /',2),('Wing arm /',2),('Wing finger /',4),('Wing claw /',6),('Foot claw /',6),('Tail tip /',1)):
   self.assertEqual(sum(p['part'].startswith(prefix) for p in self.ps),n)
 def test_skull_jaw_cheeks_and_swept_horns_have_real_joins(self):
  skull=self.find('Skull /');jaw=self.find('Jaw /');self.assertTrue(self.overlap(skull,self.find('Body /')))
  for sign in (-1,1):
   cheek=self.find('Jaw hinge / rooted rear cheek '+str(sign));self.assertTrue(self.overlap(cheek,skull));self.assertTrue(self.overlap(cheek,jaw))
   self.assertTrue(self.overlap(skull,self.find('Skull horn / long swept backward fossil crest '+str(sign))))
 def test_open_jaw_is_actual_empty_space_between_stored_solids(self):
  skull=rotate(self.find('Skull /'),True);jaw=rotate(self.find('Jaw /'),True)
  # Interior vertical clearance at three independently chosen positions.
  for z in (.24,.32,.40):
   scale=max(self.find('Skull /')['positions'][2::3])/.585
   x=0;y=-z*scale;upper=ray_intersections(skull,x,y);lower=ray_intersections(jaw,x,y);self.assertTrue(upper and lower)
   self.assertGreater(min(upper)-max(lower),.12*HEIGHT)
   middle=(min(upper)+max(lower))/2
   for p in self.ps:
    hits=ray_intersections(rotate(p,True),x,y)
    if hits:self.assertFalse(min(hits)-1e-6<middle<max(hits)+1e-6,p['part'])
 def test_closed_mouth_negative_control_has_no_open_clearance(self):
  upper=rotate(self.find('Skull /'),True);lower=dict(self.find('Jaw /'))
  lower['positions']=[v+HEIGHT*.3 if i%3==1 else v for i,v in enumerate(lower['positions'])];lower=rotate(lower,True)
  scale=max(self.find('Skull /')['positions'][2::3])/.585
  a=ray_intersections(upper,0,-.32*scale);b=ray_intersections(lower,0,-.32*scale)
  self.assertTrue(a and b);self.assertLess(min(a)-max(b),.12*HEIGHT)
 def test_arrowhead_has_visible_projected_area_in_front_view(self):
  arrow=self.find('Tail tip /');area=0.
  for i in range(0,len(arrow['indices']),3):
   a,b,c=[arrow['positions'][3*j:3*j+3] for j in arrow['indices'][i:i+3]]
   area+=abs((b[0]-a[0])*(c[1]-a[1])-(b[1]-a[1])*(c[0]-a[0]))/2
  self.assertGreater(area,.015*HEIGHT**2)
 def test_upper_and_lower_teeth_are_rooted_in_their_actual_jaws(self):
  for p in self.ps:
   if p['part'].startswith('Tooth /'):
    parent=self.find('Skull /' if ' upper ' in p['part'] else 'Jaw /');self.assertTrue(self.overlap(p,parent),p['part'])
 def fitted_pairs(self):
  for p in self.ps:
   name=p['part'];sign=name.split()[-1];parent=None;up=None
   if name.startswith(('Eye socket /','Nostril /')):parent=self.find('Skull /')
   elif name.startswith('Eye white /'):parent=self.find('Eye socket / supported stern dark wedge '+sign)
   elif name.startswith('Pupil /'):parent=self.find('Eye white / fitted narrow ivory wedge '+sign)
   elif name.startswith('Mouth ceiling /'):parent=self.find('Skull /');up=False
   elif name.startswith(('Mouth floor /','Tongue /')):parent=self.find('Jaw /');up=True
   if parent:yield (rotate(p,up),rotate(parent,up)) if up is not None else (p,parent)
 def test_fitted_face_palate_and_tongue_vertices_follow_stored_surfaces(self):
  for p,parent in self.fitted_pairs():
   for x,y,z in {tuple(p['positions'][i:i+3]) for i in range(0,len(p['positions']),3)}:
    hits=ray_intersections(parent,x,y);self.assertTrue(hits,p['part']);self.assertLess(abs(z-max(hits)),.020,p['part'])
 def test_fitted_front_triangles_clear_actual_parent(self):
  for p,parent in self.fitted_pairs():
   for k in range(0,len(p['indices']),3):
    pts=[p['positions'][3*i:3*i+3] for i in p['indices'][k:k+3]];a,b,c=pts
    if (b[0]-a[0])*(c[1]-a[1])-(b[1]-a[1])*(c[0]-a[0])<1e-10:continue
    x,y,z=[sum(v[q] for v in pts)/3 for q in range(3)];hits=ray_intersections(parent,x,y);self.assertTrue(hits,p['part']);self.assertGreater(z-max(hits),.0001,(p['part'],k//3))
 def test_both_scalloped_sails_and_branching_spars_are_attached(self):
  body=self.find('Body /')
  for sign in (-1,1):
   wing=self.find('Wing / broad scalloped violet fossil membrane '+str(sign));arm=self.find('Wing arm / continuous bent leading fossil arm '+str(sign));self.assertTrue(self.overlap(wing,body));self.assertTrue(self.overlap(wing,arm));self.assertTrue(self.overlap(arm,body))
   for p in self.ps:
    if p['part'].startswith('Wing finger /') and p['part'].split()[-2]==str(sign):self.assertTrue(self.overlap(p,wing),p['part']);self.assertTrue(self.overlap(p,arm),p['part'])
    if p['part'].startswith('Wing claw /') and p['part'].split()[-2]==str(sign):self.assertTrue(self.overlap(p,arm),p['part'])
 def test_grounded_pads_talons_and_arrow_tail_are_rooted(self):
  body=self.find('Body /')
  for sign in (-1,1):
   leg=self.find('Leg / bent fossil hind limb '+str(sign));foot=self.find('Foot / grounded narrow fossil pad '+str(sign));self.assertTrue(self.overlap(leg,body));self.assertTrue(self.overlap(leg,foot));self.assertEqual(min(foot['positions'][1::3]),0.)
   for p in self.ps:
    if p['part'].startswith('Foot claw /') and p['part'].split()[-2]==str(sign):self.assertTrue(self.overlap(p,foot),p['part'])
  tail=self.find('Tail /');self.assertTrue(self.overlap(tail,body));self.assertTrue(self.overlap(tail,self.find('Tail tip /')))
if __name__=='__main__':unittest.main()
