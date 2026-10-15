#!/usr/bin/env python3
"""Independent closed hemispheres, common equator and surface-fitted orb faces."""
import json,os,struct,unittest
from pathlib import Path
from orb_sculpt import sculpture,SPECIES,HEIGHT
from voltorb_glb import read_voltorb
from sculpt_test_support import assert_closed_models,assert_model_shading
from test_slime_sculpt import ray_intersections
ROOT=Path(os.environ.get('ORB_TEST_MODELS',Path(__file__).resolve().parents[1]/'crates/crystal-voxel-view/models'))
class OrbTests(unittest.TestCase):
 @classmethod
 def setUpClass(cls):
  cls.models={}
  for s in SPECIES:
   if s=='voltorb':
    path=ROOT/'voltorb.glb' if os.environ.get('ORB_TEST_MODELS') else ROOT/'battle_species/voltorb.glb'
    cls.models[s]=read_voltorb(path)
   else:
    path=ROOT/(s+'.mesh.json') if os.environ.get('ORB_TEST_MODELS') else ROOT/'battle_species/electrode.mesh.json'
    cls.models[s]=json.loads(path.read_bytes())
 def find(self,ps,prefix,suffix=None):return next(p for p in ps if p['part'].startswith(prefix) and (suffix is None or p['part'].endswith(' '+str(suffix))))
 def test_closed_outward_geometry(self):assert_closed_models(self,self.models)
 def test_unit_normals_and_actual_winding(self):assert_model_shading(self,self.models)
 def test_exact_recipe_grounding_height_budget(self):
  for s,m in self.models.items():
   expected=sculpture(s)
   if s=='voltorb':
    for a,b in zip(m['primitives'],expected['primitives']):
     self.assertEqual(a['part'],b['part']);self.assertEqual(a['indices'],b['indices']);self.assertEqual(a['base_color'],b['base_color'])
     for key in ('positions','normals'):self.assertEqual(struct.pack('<'+'f'*len(a[key]),*a[key]),struct.pack('<'+'f'*len(b[key]),*b[key]))
   else:self.assertEqual(m,expected)
   ps=m['primitives'];self.assertEqual(min(v for p in ps for v in p['positions'][1::3]),0);self.assertAlmostEqual(max(v for p in ps for v in p['positions'][1::3]),HEIGHT[s],places=6);self.assertLess(sum(len(p['indices'])//3 for p in ps),35000)
 def test_hemispheres_have_identical_closed_equator(self):
  for s,m in self.models.items():
   a,b=m['primitives'][:2];lo=min(a['positions'][1::3]);hi=max(b['positions'][1::3]);self.assertAlmostEqual(lo,hi,places=7)
   def rim(p,y):return {tuple(p['positions'][i:i+3]) for i in range(0,len(p['positions']),3) if abs(p['positions'][i+1]-y)<1e-7}
   self.assertEqual(rim(a,lo),rim(b,hi));self.assertEqual(len(rim(a,lo)),64)
 def test_distinct_angry_eyes_and_expressive_grin(self):
  for s,m in self.models.items():
   ps=m['primitives'];counts=(('Pupil /',2),('Upper shell /',1),('Lower shell /',1))
   counts += (('Brow /',2),('Grin border /',1),('Teeth /',1),('Tooth division /',2)) if s=='electrode' else (('Eye border /',2),('Eye white /',2))
   for prefix,n in counts:self.assertEqual(sum(p['part'].startswith(prefix) for p in ps),n)
 def test_faces_follow_actual_parent_triangles_and_clear_them(self):
  for s,m in self.models.items():
   ps=m['primitives']
   for p in ps:
    n=p['part']
    if n.startswith(('Eye border /','Brow /')) or (n.startswith('Pupil /') and s=='electrode'):parent=ps[0]
    elif n.startswith('Eye white /'):parent=self.find(ps,'Eye border /',n.split()[-1])
    elif n.startswith('Pupil /'):parent=self.find(ps,'Eye white /',n.split()[-1])
    elif n.startswith('Grin border /'):parent=ps[1]
    elif n.startswith('Teeth /'):parent=self.find(ps,'Grin border /')
    elif n.startswith('Tooth division /'):parent=self.find(ps,'Teeth /')
    else:continue
    for i in range(0,len(p['positions']),3):
     x,y,z=p['positions'][i:i+3];h=ray_intersections(parent,x,y);self.assertTrue(h,(s,n));self.assertLess(abs(z-max(h)),.020,(s,n))
    if n.startswith(('Brow /','Tooth division /')):continue
    for k in range(0,len(p['indices']),3):
     a,b,c=[p['positions'][i*3:i*3+3] for i in p['indices'][k:k+3]]
     if (b[0]-a[0])*(c[1]-a[1])-(b[1]-a[1])*(c[0]-a[0])<1e-10:continue
     x,y,z=[(a[j]+b[j]+c[j])/3 for j in range(3)];h=ray_intersections(parent,x,y);self.assertTrue(h);self.assertGreater(z-max(h),.0001,(s,n))
if __name__=='__main__':unittest.main()
