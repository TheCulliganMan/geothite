#!/usr/bin/env python3
"""Stored seahorse anatomy, real bore, spiral and supported fin/face details."""
import json,os,unittest
from pathlib import Path
from seahorse_sculpt import sculpture,HEIGHT
from sculpt_test_support import assert_closed_models,assert_model_shading
from test_slime_sculpt import ray_intersections
from bovine_sculpt import reverse_front
import test_magnet_sculpt as joins
ROOT=Path(os.environ.get('SEAHORSE_TEST_MODELS',Path(__file__).resolve().parents[1]/'crates/crystal-voxel-view/models/battle_species'))
class SeahorseTests(unittest.TestCase):
 @classmethod
 def setUpClass(cls):
  from seahorse_glb import read_seahorse
  cls.models={n:(json.loads((ROOT/(n+'.mesh.json')).read_bytes()) if os.environ.get('SEAHORSE_TEST_MODELS') and not (ROOT/(n+'.glb')).exists() else read_seahorse(ROOT/(n+'.glb'),n)) for n in ('horsea','seadra')}
 def find(self,ps,prefix,suffix=None):return next(p for p in ps if p['part'].startswith(prefix) and (suffix is None or p['part'].endswith(' '+str(suffix))))
 def test_closed_outward_anatomy(self):assert_closed_models(self,self.models)
 def test_actual_stored_shading(self):assert_model_shading(self,self.models)
 def test_recipe_identity_grounding_and_budget(self):
  for n,m in self.models.items():
   from seahorse_glb import geometry_digest
   self.assertEqual(geometry_digest(m),geometry_digest(sculpture(n)));ps=m['primitives'];names=[p['part'] for p in ps];self.assertEqual(len(names),len(set(names)))
   self.assertEqual(min(y for p in ps for y in p['positions'][1::3]),0.)
   self.assertAlmostEqual(max(y for p in ps for y in p['positions'][1::3]),HEIGHT,places=6)
   self.assertLessEqual(sum(len(p['indices'])//3 for p in ps),40000);self.assertLessEqual(sum(len(p['positions'])//3 for p in ps),20000)
   for prefix,count in {'Back fin /':2,'Belly fold /':5,'Snout /':3,'Pupil /':2,'Head fin /':2 if n=='seadra' else 6,'Armor /':6 if n=='seadra' else 0,'Iris /':0 if n=='seadra' else 2}.items():self.assertEqual(sum(x.startswith(prefix) for x in names),count)
 def test_anatomical_solid_connections(self):
  for n,m in self.models.items():
   ps=m['primitives'];body=self.find(ps,'Body /');head=self.find(ps,'Head /');snout=self.find(ps,'Snout / hollow');bore=self.find(ps,'Snout / recessed');floor=self.find(ps,'Snout / deep')
   pairs=[(body,head),(body,self.find(ps,'Tail /')),(head,snout),(snout,bore),(bore,floor)]
   pairs += [(body,p) for p in ps if p['part'].startswith(('Back fin /','Armor /'))]
   pairs += [(head,p) for p in ps if p['part'].startswith('Head fin /')]
   for a,b in pairs:self.assertTrue(joins.MagnetTests.overlap(self,a,b),(n,a['part'],b['part']))
 def markings(self):
  for n,m in self.models.items():
   ps=m['primitives']
   for p in ps:
    name=p['part'];tail=name.split()[-1];parent=None
    if name.startswith('Belly /'):parent=self.find(ps,'Body /')
    elif name.startswith('Belly fold /'):parent=self.find(ps,'Belly /')
    elif name.startswith('Eye socket /'):parent=self.find(ps,'Head /')
    elif name.startswith('Eye white /'):parent=self.find(ps,'Eye socket /',tail)
    elif name.startswith('Iris /'):parent=self.find(ps,'Eye white /',tail)
    elif name.startswith('Pupil /'):parent=self.find(ps,'Eye white /' if n=='seadra' else 'Iris /',tail)
    elif name.startswith('Glint /'):parent=self.find(ps,'Pupil /',tail)
    elif name.startswith(('Fin tip /','Fin ray /')):parent=self.find(ps,'Back fin /',name.split()[-2])
    if parent:yield n,p,parent
 def test_details_follow_actual_parent_triangles(self):
  for n,p,parent in self.markings():
   for i in range(0,len(p['positions']),3):
    x,y,z=p['positions'][i:i+3];h=ray_intersections(parent,x,y);self.assertTrue(h,(n,p['part'],x,y))
    gap=min(abs(z-max(h)),abs(z-min(h))) if p['part'].startswith('Fin tip /') else abs(z-max(h))
    self.assertLess(gap,.018,(n,p['part']))
 def test_front_faces_clear_stored_parent_surfaces(self):
  for n,p,parent in self.markings():
   for k in range(0,len(p['indices']),3):
    a,b,c=[p['positions'][i*3:i*3+3] for i in p['indices'][k:k+3]]
    if (b[0]-a[0])*(c[1]-a[1])-(b[1]-a[1])*(c[0]-a[0])<1e-10:continue
    x,y,z=[(a[j]+b[j]+c[j])/3 for j in range(3)];h=ray_intersections(parent,x,y);self.assertTrue(h);self.assertGreater(z-max(h),.0001,(n,p['part'],x,y))
 def test_ivory_sail_tips_clear_actual_rear_surfaces(self):
  for n,p,parent in self.markings():
   if not p['part'].startswith('Fin tip /'):continue
   q=reverse_front(p);support=reverse_front(parent)
   for k in range(0,len(q['indices']),3):
    a,b,c=[q['positions'][i*3:i*3+3] for i in q['indices'][k:k+3]]
    if (b[0]-a[0])*(c[1]-a[1])-(b[1]-a[1])*(c[0]-a[0])<1e-10:continue
    x,y,z=[(a[j]+b[j]+c[j])/3 for j in range(3)];h=ray_intersections(support,x,y);self.assertTrue(h);self.assertGreater(z-max(h),.0001,(n,p['part']))
 def test_muzzles_have_real_openings_and_deep_wells(self):
  for n,m in self.models.items():
   ps=m['primitives'];wall=self.find(ps,'Snout / hollow');bore=self.find(ps,'Snout / recessed');floor=self.find(ps,'Snout / deep')
   zmax=max(wall['positions'][2::3]);y=(min(wall['positions'][1::3])+max(wall['positions'][1::3]))/2
   # An axial ray meets only the recessed floor, never the annular muzzle wall.
   self.assertFalse(ray_intersections(wall,0,y));self.assertFalse(ray_intersections(bore,0,y))
   hits=ray_intersections(floor,0,y);self.assertTrue(hits);self.assertGreater(zmax-max(hits),.14)
 def test_spiral_has_visible_inner_turn_and_grounded_volume(self):
  for n,m in self.models.items():
   tail=self.find(m['primitives'],'Tail /');xs=tail['positions'][::3];ys=tail['positions'][1::3]
   self.assertGreater(max(xs)-min(xs),.27);self.assertGreater(max(ys)-min(ys),.25)
   # Stored curve contains multiple distinct cross sections and an inward tip.
   self.assertGreater(len(xs),1000);self.assertEqual(min(ys),0.)
if __name__=='__main__':unittest.main()
