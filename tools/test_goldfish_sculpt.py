#!/usr/bin/env python3
"""Stored flowing fish anatomy, fitted skin and actual empty mouth regressions."""
import json,os,unittest
from pathlib import Path
from goldfish_sculpt import sculpture,SPECIES,HEIGHTS
from jet_fish_sculpt import side_rotate
from bovine_sculpt import reverse_front
from sculpt_test_support import assert_closed_models,assert_model_shading
from test_slime_sculpt import ray_intersections
from test_kingdra_sculpt import crossed_faces
from test_dragon_family_sculpt import surface_distances
ROOT=Path(os.environ.get('CRYSTAL_GOLDFISH_TEST_MODELS',Path(__file__).resolve().parents[1]/'crates/crystal-voxel-view/models/battle_species'))
class GoldfishTests(unittest.TestCase):
 @classmethod
 def setUpClass(cls):cls.models={s:json.loads((ROOT/(s+'.mesh.json')).read_bytes()) for s in SPECIES}
 def find(self,s,prefix,last=None):return next(p for p in self.models[s]['primitives'] if p['part'].startswith(prefix) and (last is None or p['part'].endswith(' '+str(last))))
 def overlap(self,a,b):
  xl=max(min(a['positions'][::3]),min(b['positions'][::3]));xh=min(max(a['positions'][::3]),max(b['positions'][::3]));yl=max(min(a['positions'][1::3]),min(b['positions'][1::3]));yh=min(max(a['positions'][1::3]),max(b['positions'][1::3]))
  if xh<=xl or yh<=yl:return False
  for i in range(1,25):
   for j in range(1,25):
    x=xl+(xh-xl)*i/25;y=yl+(yh-yl)*j/25;aa=ray_intersections(a,x,y);bb=ray_intersections(b,x,y)
    if aa and bb and min(max(aa),max(bb))-max(min(aa),min(bb))>.0002:return True
  return False
 def test_closed_connected_outward_parts(self):assert_closed_models(self,self.models)
 def test_finite_unit_normals_follow_stored_faces(self):assert_model_shading(self,self.models)
 def test_regeneration_species_height_floor_and_budget(self):
  for s,m in self.models.items():
   self.assertEqual(m,sculpture(s));self.assertEqual(len(m['primitives']),len({p['part'] for p in m['primitives']}));self.assertLess(sum(len(p['indices'])//3 for p in m['primitives']),75000)
   self.assertEqual(min(v for p in m['primitives'] for v in p['positions'][1::3]),0);self.assertAlmostEqual(max(v for p in m['primitives'] for v in p['positions'][1::3]),HEIGHTS[s],places=6)
 def test_body_has_no_nonadjacent_crossed_faces(self):
  for s in SPECIES:self.assertIsNone(crossed_faces(self.find(s,'Body /')),s)
 def test_lips_and_recessed_floors_are_rooted_in_actual_annular_snouts(self):
  for s in SPECIES:
   b=self.find(s,'Body /');self.assertTrue(self.overlap(b,self.find(s,'Lip /')),s);self.assertTrue(self.overlap(b,self.find(s,'Mouth /')),s)
 def empty_mouth(self,parts):
  lip=next(p for p in parts if p['part'].startswith('Lip /'));x=(min(lip['positions'][::3])+max(lip['positions'][::3]))/2;y=(min(lip['positions'][1::3])+max(lip['positions'][1::3]))/2;front=max(lip['positions'][2::3]);rx=(max(lip['positions'][::3])-min(lip['positions'][::3]))*.14;ry=(max(lip['positions'][1::3])-min(lip['positions'][1::3]))*.14
  for dx in (-rx,0,rx):
   for dy in (-ry,0,ry):
    hits=[z for p in parts for z in ray_intersections(p,x+dx,y+dy)]
    if not hits or max(hits)>front-.045:return False
  return True
 def test_actual_mouth_space_is_empty_across_every_stored_part(self):
  for s,m in self.models.items():self.assertTrue(self.empty_mouth(m['primitives']),s)
 def test_filled_mouth_negative_control_is_rejected(self):
  from sculpt_geometry import ellipsoid
  ps=self.models['magikarp']['primitives'];lip=self.find('magikarp','Lip /');y=(min(lip['positions'][1::3])+max(lip['positions'][1::3]))/2;z=max(lip['positions'][2::3]);filler=ellipsoid('Bad filled mouth',(.5,.5,.5),(0,y,z-.005),(.04,.06,.015),20,14)
  self.assertFalse(self.empty_mouth(ps+[filler]))
 def pairs(self):
  for s,m in self.models.items():
   for p in m['primitives']:
    name=p['part'];last=name.split()[-1];parent=None
    if name.startswith(('Eye socket /','Mask /','Bib /','Gill /')):parent=self.find(s,'Body /')
    elif name.startswith('Eye white /'):parent=self.find(s,'Eye socket /',last)
    elif name.startswith('Iris /'):parent=self.find(s,'Eye white /',last)
    elif name.startswith('Pupil /'):parent=self.find(s,'Iris /' if s=='goldeen' else 'Eye white /',last)
    elif name.startswith('Lid /'):parent=self.find(s,'Eye white /',last)
    elif name.startswith('Glint /'):parent=self.find(s,'Pupil /',last)
    elif name.startswith(('Flank /','Flank spot /')):
     sign=int(name.split()[-2] if name.startswith('Flank spot /') else last);parent=side_rotate(self.find(s,'Body /'),-sign);p=side_rotate(p,-sign)
    elif name.startswith(('Fin paint /','Fin spot /')):
     j=int(name.split()[-3] if name.startswith('Fin spot /') else name.split()[-2]);parent=[q for prefix in ('Tail /','Pectoral /','Pelvic /') for q in m['primitives'] if q['part'].startswith(prefix)][j]
     if int(last)==-1:p=reverse_front(p);parent=reverse_front(parent)
    elif name.startswith('Tail inset /'):
     parent=side_rotate(self.find(s,'Tail /'),-1);p=side_rotate(p,-1)
     if int(last)==-1:p=reverse_front(p);parent=reverse_front(parent)
    if parent:yield s,p,parent
 def test_painted_skin_and_fitted_faces_have_actual_mesh_support(self):
  for s,p,parent in self.pairs():
   for i in range(0,len(p['positions']),3):
    x,y,z=p['positions'][i:i+3];hits=ray_intersections(parent,x,y);self.assertTrue(hits,(s,p['part']));self.assertLess(abs(z-max(hits)),.02,(s,p['part']))
 def test_fitted_front_centers_and_edges_clear_stored_parents(self):
  for s,p,parent in self.pairs():
   for k in range(0,len(p['indices']),3):
    a,b,c=[p['positions'][3*i:3*i+3] for i in p['indices'][k:k+3]]
    if (b[0]-a[0])*(c[1]-a[1])-(b[1]-a[1])*(c[0]-a[0])<1e-10:continue
    for weights in ((1/3,1/3,1/3),(.5,.5,0),(0,.5,.5),(.5,0,.5)):
     x,y,z=[sum(v[q]*w for v,w in zip((a,b,c),weights)) for q in range(3)];hits=ray_intersections(parent,x,y);self.assertTrue(hits,(s,p['part']));self.assertGreater(z-max(hits),.0001,(s,p['part'],k//3))
 def test_sails_streamers_and_species_features_are_rooted(self):
  for s,m in self.models.items():
   b=self.find(s,'Body /')
   for p in m['primitives']:
    if p['part'].startswith(('Tail /','Pectoral /','Dorsal /','Pelvic /','Horn /','Whisker /')):self.assertTrue(self.overlap(p,b),(s,p['part']))
   for prefix,n in {'Tail /':1 if s=='magikarp' else 2,'Pectoral /':2,'Dorsal /':1,'Horn /':0 if s=='magikarp' else 1,'Whisker /':2 if s=='magikarp' else 0,'Lid /':2 if s=='goldeen' else 0,'Flank /':2 if s=='seaking' else 0,'Scale seam /':6 if s=='magikarp' else 0}.items():self.assertEqual(sum(p['part'].startswith(prefix) for p in m['primitives']),n,(s,prefix))
 def test_fin_rays_rims_and_carp_seams_follow_actual_parent_surfaces(self):
  for s,m in self.models.items():
   fans=[q for prefix in ('Tail /','Pectoral /','Pelvic /') for q in m['primitives'] if q['part'].startswith(prefix)]
   for p in m['primitives']:
    if p['part'].startswith('Fin ray /'):parent=fans[int(p['part'].split()[-2])]
    elif p['part'].startswith('Pectoral rim /'):parent=self.find(s,'Pectoral /',p['part'].split()[-1])
    elif p['part'].startswith('Scale seam /'):parent=self.find(s,'Body /')
    else:continue
    points={tuple(p['positions'][i:i+3]) for i in range(0,len(p['positions']),3)}
    for d in surface_distances(parent,points,.013):self.assertLess(d,.013,(s,p['part']))
if __name__=='__main__':unittest.main()
