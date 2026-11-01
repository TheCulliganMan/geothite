#!/usr/bin/env python3
"""Stored striped-canines: real roots, four supports, flush markings and uncrossed fur."""
import copy,json,math,os,unittest
from pathlib import Path
import fire_canine_sculpt as author
from fire_canine_glb import read_fire_canine,geometry_digest
from sculpt_geometry import translate
from sculpt_test_support import assert_closed_models,assert_model_shading,crossed_faces,indexed_ray_intersections as rays
ROOT=Path(os.environ.get('CRYSTAL_FIRE_CANINE_TEST_MODELS',Path(__file__).resolve().parents[1]/'crates/crystal-voxel-view/models/battle_species'))
def overlap(a,b):
 xl=max(min(a['positions'][::3]),min(b['positions'][::3]));xh=min(max(a['positions'][::3]),max(b['positions'][::3]));yl=max(min(a['positions'][1::3]),min(b['positions'][1::3]));yh=min(max(a['positions'][1::3]),max(b['positions'][1::3]))
 if xh<=xl or yh<=yl:return False
 for i in range(1,19):
  for j in range(1,23):
   x=xl+(xh-xl)*i/19;y=yl+(yh-yl)*j/23;aa=sorted(rays(a,x,y));bb=sorted(rays(b,x,y))
   if any(min(ah,bh)-max(al,bl)>.00025 for al,ah in zip(aa[::2],aa[1::2]) for bl,bh in zip(bb[::2],bb[1::2])):return True
 return False
def find(ps,prefix,side=None):return next(p for p in ps if p['part'].startswith(prefix) and (side is None or p['part'].endswith(' '+str(side))))
def fitted_pairs(ps):
 for p in ps:
  name=p['part'];side=name.split()[-1];parent=None;frame=None
  for child,owner in (('Lip /','Muzzle /'),('Eye socket /','Head /'),('Eye /','Eye socket /'),('Pupil /','Eye /'),('Eye glint /','Pupil /'),('Ear inset /','Ear /'),('Forehead stripe /','Head /')):
   if name.startswith(child):parent=find(ps,owner,side if owner in ('Eye socket /','Eye /','Pupil /','Ear /') else None)
  if name.startswith('Paw crease /'):parent=find(ps,'Paw /',' '.join(name.split()[-3:-1]))
  if name.startswith(('Flank stripe /','Leg stripe /')):
   side=int(name.split()[-2]);parent=find(ps,'Body /') if name.startswith('Flank stripe /') else find(ps,'Leg /',' '.join(name.split()[-2:]));frame=lambda p,side=side:author.rotate(p,side)
  if name.startswith('Back stripe /'):parent=find(ps,'Body /');frame=lambda p:author.rotate(p,1,roof=True)
  if parent is not None:yield (frame(p),frame(parent)) if frame else (p,parent)
class FireCanineTests(unittest.TestCase):
 @classmethod
 def setUpClass(cls):cls.models={s:read_fire_canine(ROOT/(s+'.glb'),s) for s in author.SPECIES}
 def test_closed_outward_connected_surfaces(self):assert_closed_models(self,self.models)
 def test_normals_match_actual_triangles(self):assert_model_shading(self,self.models)
 def test_recipe_palette_floor_and_unchanged_limits(self):
  for s,m in self.models.items():
   self.assertEqual(geometry_digest(m),geometry_digest(author.sculpture(s)));ps=m['primitives'];self.assertEqual(min(v for p in ps for v in p['positions'][1::3]),0.);self.assertAlmostEqual(max(v for p in ps for v in p['positions'][1::3]),author.HEIGHTS[s],places=6);self.assertLessEqual(sum(len(p['positions'])//3 for p in ps),20000);self.assertLessEqual(sum(len(p['indices'])//3 for p in ps),40000);self.assertEqual(len(ps),len({p['part'] for p in ps}))
 def test_distinct_puppy_and_guardian_anatomy(self):
  for s,m in self.models.items():
   ps=m['primitives'];big=s=='arcanine'
   for prefix,n in (('Leg /',4),('Paw /',4),('Paw crease /',8),('Flank stripe /',6 if big else 4),('Back stripe /',3 if big else 2),('Ear /',2),('Mane tuft /',10 if big else 7),('Ankle fur /',4 if big else 0),('Ankle tuft /',12 if big else 0),('Tail /',1),('Tail tuft /',3 if big else 1)):self.assertEqual(sum(p['part'].startswith(prefix) for p in ps),n,(s,prefix))
   body=find(ps,'Body /');width=max(body['positions'][::3])-min(body['positions'][::3]);length=max(body['positions'][2::3])-min(body['positions'][2::3]);self.assertGreater(length/width,2.1 if big else 1.7)
 def test_head_chest_muzzle_nose_mane_and_ear_roots(self):
  for s,m in self.models.items():
   ps=m['primitives']
   for child,parent in (('Neck /','Body /'),('Head /','Neck /'),('Muzzle /','Head /'),('Jaw /','Muzzle /'),('Nose /','Muzzle /'),('Mane /','Neck /'),('Forelock /','Head /'),('Tail /','Body /')):self.assertTrue(overlap(find(ps,child),find(ps,parent)),(s,child,parent))
   for side in (-1,1):self.assertTrue(overlap(find(ps,'Ear /',side),find(ps,'Head /')),(s,side))
 def test_four_leg_paw_contacts_and_grounded_pads(self):
  for s,m in self.models.items():
   ps=m['primitives']
   for side in (-1,1):
    for row in range(2):
     suffix=f'{side} {row}';leg=find(ps,'Leg /',suffix);paw=find(ps,'Paw /',suffix);self.assertTrue(overlap(leg,find(ps,'Body /')),(s,suffix));self.assertTrue(overlap(paw,leg),(s,suffix));self.assertEqual(min(paw['positions'][1::3]),0.)
 def test_every_fur_tuft_has_an_occupied_root(self):
  for s,m in self.models.items():
   ps=m['primitives']
   for p in ps:
    n=p['part']
    if n.startswith('Mane tuft /'):parent=find(ps,'Mane /')
    elif n.startswith('Tail tuft /'):parent=find(ps,'Tail /')
    elif n.startswith('Ankle fur /'):parent=find(ps,'Leg /',' '.join(n.split()[-2:]))
    elif n.startswith('Ankle tuft /'):parent=find(ps,'Ankle fur /',' '.join(n.split()[-3:-1]))
    else:continue
    self.assertTrue(overlap(p,parent),(s,n))
 def test_marking_front_vertices_edges_and_centers_follow_actual_parent(self):
  for s,m in self.models.items():
   for child,parent in fitted_pairs(m['primitives']):
    points=[child['positions'][i:i+3] for i in range(0,len(child['positions']),3)]
    for k in range(0,len(child['indices']),3):
     tri=[points[i] for i in child['indices'][k:k+3]];a,b,c=tri
     if (b[0]-a[0])*(c[1]-a[1])-(b[1]-a[1])*(c[0]-a[0])<=1e-12:continue
     for ws in ((1,0,0),(0,1,0),(0,0,1),(.5,.5,0),(.5,0,.5),(0,.5,.5),(1/3,1/3,1/3)):
      x,y,z=[sum(ws[j]*tri[j][axis] for j in range(3)) for axis in range(3)];hits=rays(parent,x,y);self.assertTrue(hits,(s,child['part']));gap=z-max(hits);self.assertGreater(gap,.0001,(s,child['part'],gap));self.assertLess(gap,.012,(s,child['part'],gap))
 def test_limbs_fur_ears_and_curled_tail_are_uncrossed(self):
  for s,m in self.models.items():
   for p in m['primitives']:
    if p['part'].startswith(('Leg /','Ear /','Tail /','Mane tuft /','Forelock /','Ankle tuft /','Tail tuft /')):self.assertIsNone(crossed_faces(p),(s,p['part']))
 def test_detached_appendages_are_rejected(self):
  for s,m in self.models.items():
   ps=m['primitives']
   for child,parent in (('Tail /','Body /'),('Ear /','Head /'),('Forelock /','Head /'),('Paw /','Leg /')):self.assertFalse(overlap(translate(copy.deepcopy(find(ps,child)),3,0,0),find(ps,parent)))
if __name__=='__main__':unittest.main()
