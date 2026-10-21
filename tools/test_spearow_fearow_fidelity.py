"""Independent stored geometry checks for the inspected small-bird flight poses.

CRYSTAL_SPEAROW_FEAROW_MODELS can select the immutable rejected models. These
checks measure silhouette and anatomy rather than trusting source digest pins.
"""
import math,os,unittest
from pathlib import Path
from test_gengar_glb import Model
from test_ninetales_glb import logical_model
from pidgeotto_contours import components
from sculpt_test_support import assert_closed_models,assert_model_shading,horizontal_section_bounds as section,parts_overlap

ROOT=Path(os.environ.get('CRYSTAL_SPEAROW_FEAROW_MODELS',Path(__file__).resolve().parents[1]/'crates/crystal-voxel-view/models/battle_species'))
def span(p,a):return max(p['positions'][a::3])-min(p['positions'][a::3])
def area(p,u,v):
 points=list(zip(p['positions'][::3],p['positions'][1::3],p['positions'][2::3]));total=0.
 for i in range(0,len(p['indices']),3):
  a,b,c=[points[j] for j in p['indices'][i:i+3]];total+=abs((b[u]-a[u])*(c[v]-a[v])-(b[v]-a[v])*(c[u]-a[u]))*.5
 return total
class SmallBirdFidelityTests(unittest.TestCase):
 @classmethod
 def setUpClass(cls):
  cls.models={};cls.parts={}
  for s in ('spearow','fearow'):
   m=Model((ROOT/(s+'.glb')).read_bytes());cls.models[s]=logical_model(m) if 'source_part_ranges' in m.doc['extras'] else m;cls.parts[s]=cls.models[s].neutral_model()['primitives']
 def find(self,s,prefix):
  ps=[p for p in self.parts[s] if p['part'].startswith(prefix)];self.assertTrue(ps,(s,prefix));return ps
 def test_spearow_is_made_of_closed_outward_components(self):
  ps=[dict(q,part=p['part']+' component '+str(j)) for p in self.parts['spearow'] for j,q in enumerate(components(p))]
  assert_closed_models(self,{'spearow':dict(primitives=ps)});assert_model_shading(self,{'spearow':dict(primitives=ps)})
 def test_spearow_short_shanks_are_upright_above_the_toes(self):
  p=self.find('spearow','Feet / planted')[0]
  for q in components(p):
   centers=[]
   for y in (.06*.75,.14*.75):
    lo,hi=section(q,y);centers.append((lo[1]+hi[1])*.5)
   self.assertLess(abs(centers[0]-centers[1]),.004*.75)
 def test_spearow_closed_wing_cases_are_physically_rooted(self):
  body=self.find('spearow','Torso / warm')[0]
  for label in ('left','right'):
   wing=self.find('spearow','Wing '+label+' / broad')[0];self.assertTrue(parts_overlap(body,wing))
   feathers=self.find('spearow','Wing '+label+' / four')[0]
   self.assertGreater(area(feathers,1,2),area(feathers,0,1)*3.)
   self.assertGreater(span(wing,2),.20*.75)
 def test_spearow_ivory_ends_share_actual_feather_sections_and_motion(self):
  m=self.models['spearow']
  for label in ('left','right'):
   pink=self.find('spearow','Wing '+label+' / four')[0];white=self.find('spearow','Wing '+label+' / ivory')[0]
   points=lambda p:set(zip(p['positions'][::3],p['positions'][1::3],p['positions'][2::3]))
   self.assertGreaterEqual(len(points(pink)&points(white)),16)
   ids=[m.names.index(p['part']) for p in (pink,white)]
   for i in ids:self.assertTrue(all(j==([5,0,0,0] if label=='left' else [7,0,0,0]) for j in m.ids[i]))
 def test_fearow_neck_projects_forward_below_the_raised_wings(self):
  head=self.find('fearow','Head /')[0];body=self.find('fearow','Body /')[0];wing=self.find('fearow','Wing /')[0]
  head_y=sum((min(head['positions'][1::3]),max(head['positions'][1::3])))*.5
  self.assertLess(head_y,max(body['positions'][1::3]))
  self.assertGreater(min(head['positions'][2::3]),max(body['positions'][2::3])+.12)
  self.assertGreater(max(wing['positions'][1::3])-head_y,.35)
 def test_fearow_comb_has_a_sagittal_feather_plane(self):
  p=self.find('fearow','Crown /')[0];self.assertGreater(area(p,1,2),area(p,0,1)*3.)
  self.assertGreater(span(p,2),span(p,0)*4.)
 def test_fearow_wings_have_filled_profile_area(self):
  ps=self.find('fearow','Primary /');profile=sum(area(p,1,2) for p in ps);front=sum(area(p,0,1) for p in ps)
  self.assertGreater(profile,.12*1.23**2);self.assertGreater(profile,front*.60)
 def test_fearow_nape_is_a_rooted_feather_fan(self):
  p=self.find('fearow','Mane /')[0];quills=components(p);self.assertEqual(len(quills),7)
  neck=self.find('fearow','Neck /')[0]
  for q in quills:self.assertTrue(parts_overlap(q,neck))
 def test_fearow_feet_curl_under_an_airborne_body_and_follow_leg_controls(self):
  m=self.models['fearow']
  for prefix in ('Foot /','Toe /','Talon /'):
   for p in self.find('fearow',prefix):
    i=m.names.index(p['part']);owner=15 if sum(p['positions'][::3])<0 else 16
    self.assertTrue(all(j==[owner,0,0,0] for j in m.ids[i]));self.assertEqual(m.parents[owner],1)
  for p in self.find('fearow','Toe / three'):
   self.assertGreater(span(p,1),span(p,2)*.85)
 def test_stored_idle_joins_at_production_absolute_time_offsets(self):
  for s,m in self.models.items():
   d=m.clips['idle'][0][1][-1]
   for t in (.00001/d,1.-.00001/d):
    mats=m.matrices('idle',t)
    for i,vs in enumerate(m.vertices):
     for a,b in zip(vs,m.posed(i,mats)):self.assertLess(max(abs(x-y) for x,y in zip(a,b)),.00001,(s,t,m.names[i]))
if __name__=='__main__':unittest.main()
