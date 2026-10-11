#!/usr/bin/env python3
"""Stored slime anatomy, physical hands, true mouth rims and supported face/folds."""
import json,os,unittest
from pathlib import Path
from slime_sculpt import sculpture,SPECIES,HEIGHT
from sculpt_test_support import assert_closed_models,assert_model_shading
from test_axolotl_sculpt import ray_intersections as unindexed_ray_intersections
from bovine_sculpt import reverse_front
import test_donphan_sculpt as joins
ROOT=Path(os.environ.get('SLIME_TEST_MODELS',Path(__file__).resolve().parents[1]/'crates/crystal-voxel-view/models/battle_species'))

_RAY_INDEX={}
def ray_intersections(parent,x,y):
 # Independent projection index over stored triangles. Every candidate keeps
 # the same barycentric test as the original unindexed ray oracle.
 key=id(parent)
 if key not in _RAY_INDEX:
  vs=[parent['positions'][i:i+3] for i in range(0,len(parent['positions']),3)]
  xmin=min(v[0] for v in vs)-1e-6;xmax=max(v[0] for v in vs)+1e-6;ymin=min(v[1] for v in vs)-1e-6;ymax=max(v[1] for v in vs)+1e-6
  nx=12;ny=20;bins=[[] for _ in range(nx*ny)]
  for k in range(0,len(parent['indices']),3):
   a,b,c=[vs[i] for i in parent['indices'][k:k+3]];den=(b[1]-c[1])*(a[0]-c[0])+(c[0]-b[0])*(a[1]-c[1])
   if abs(den)<1e-12:continue
   ix0=max(0,min(nx-1,int((min(a[0],b[0],c[0])-1e-6-xmin)/(xmax-xmin)*nx)));ix1=max(0,min(nx-1,int((max(a[0],b[0],c[0])+1e-6-xmin)/(xmax-xmin)*nx)))
   iy0=max(0,min(ny-1,int((min(a[1],b[1],c[1])-1e-6-ymin)/(ymax-ymin)*ny)));iy1=max(0,min(ny-1,int((max(a[1],b[1],c[1])+1e-6-ymin)/(ymax-ymin)*ny)))
   for ix in range(ix0,ix1+1):
    for iy in range(iy0,iy1+1):bins[ix*ny+iy].append((a,b,c,den))
  _RAY_INDEX[key]=(parent,xmin,xmax,ymin,ymax,nx,ny,bins)
 _,xmin,xmax,ymin,ymax,nx,ny,bins=_RAY_INDEX[key]
 if not xmin<=x<=xmax or not ymin<=y<=ymax:return []
 ix=min(nx-1,int((x-xmin)/(xmax-xmin)*nx));iy=min(ny-1,int((y-ymin)/(ymax-ymin)*ny));hits=[]
 for a,b,c,den in bins[ix*ny+iy]:
  u=((b[1]-c[1])*(x-c[0])+(c[0]-b[0])*(y-c[1]))/den;v=((c[1]-a[1])*(x-c[0])+(a[0]-c[0])*(y-c[1]))/den
  if min(u,v,1-u-v)>=-1e-7:hits.append(u*a[2]+v*b[2]+(1-u-v)*c[2])
 return hits
class SlimeTests(unittest.TestCase):
 @classmethod
 def setUpClass(cls):cls.models={s:json.loads((ROOT/(s+'.mesh.json')).read_bytes()) for s in SPECIES}
 def find(self,ps,prefix,suffix=None):return next(p for p in ps if p['part'].startswith(prefix) and (suffix is None or p['part'].endswith(' '+str(suffix))))
 def test_indexed_rays_match_original_triangle_oracle(self):
  for species,m in self.models.items():
   for parent in [self.find(m['primitives'],'Body /'),self.find(m['primitives'],'Mouth /')]:
    xl=min(parent['positions'][::3]);xh=max(parent['positions'][::3]);yl=min(parent['positions'][1::3]);yh=max(parent['positions'][1::3])
    for i in range(7):
     for j in range(9):
      x=xl+(xh-xl)*(i+.371)/7;y=yl+(yh-yl)*(j+.619)/9;self.assertEqual(sorted(ray_intersections(parent,x,y)),sorted(unindexed_ray_intersections(parent,x,y)),(species,parent['part'],x,y))
 def test_closed_connected_outward_surfaces(self):assert_closed_models(self,self.models)
 def test_finite_unit_normals_follow_actual_winding(self):assert_model_shading(self,self.models)
 def test_recipe_parity_height_grounding_and_budget(self):
  for s,m in self.models.items():
   with self.subTest(species=s):
    self.assertEqual(m,sculpture(s));ps=m['primitives'];self.assertEqual(min(v for p in ps for v in p['positions'][1::3]),0);self.assertAlmostEqual(max(v for p in ps for v in p['positions'][1::3]),HEIGHT[s],places=6);self.assertLess(sum(len(p['indices'])//3 for p in ps),50000);self.assertEqual(len(ps),len({p['part'] for p in ps}))
    b=self.find(ps,'Body /');ground={tuple(b['positions'][i:i+3]) for i in range(0,len(b['positions']),3) if b['positions'][i+1]==0};self.assertGreater(len(ground),20);self.assertGreater(max(x for x,y,z in ground)-min(x for x,y,z in ground),.3);self.assertGreater(max(z for x,y,z in ground)-min(z for x,y,z in ground),.2)
 def test_species_have_distinct_sludge_and_transform_anatomy(self):
  for s,m in self.models.items():
   ps=m['primitives'];self.assertEqual(sum(p['part'].startswith('Body /') for p in ps),1)
   for prefix,n in ((('Eye /',2),('Body sheen /',3),('Smile interior /',1)) if s=='ditto' else (('Arm /',2),('Hand /',2),('Finger /',6),('Eye white /',2),('Pupil /',2),('Lip /',1),('Tongue /',1),('Body fold /',6))):self.assertEqual(sum(p['part'].startswith(prefix) for p in ps),n,(s,prefix))
   if s=='ditto':self.assertFalse(any(p['part'].startswith(('Arm /','Hand /','Lip /','Finger /')) for p in ps))
 def pairs(self,s):
  ps=self.models[s]['primitives']
  for p in ps:
   n=p['part'];sign=n.split()[-1];parent=None
   if n.startswith(('Eye /','Eye border /','Mouth /','Body sheen /')):parent=self.find(ps,'Body /')
   elif n.startswith('Eye white /'):parent=self.find(ps,'Eye border /',sign)
   elif n.startswith('Pupil /'):parent=self.find(ps,'Eye white /',sign)
   elif n.startswith(('Tongue /','Smile interior /')):parent=self.find(ps,'Mouth /')
   if parent:yield p,parent
 def test_marking_vertices_fit_actual_stored_parents(self):
  for s in SPECIES:
   for p,parent in self.pairs(s):
    for x,y,z in {tuple(p['positions'][i:i+3]) for i in range(0,len(p['positions']),3)}:
     h=ray_intersections(parent,x,y);self.assertTrue(h,(s,p['part'],x,y));self.assertLess(abs(z-max(h)),.023,(s,p['part']))
 def test_front_marking_triangles_clear_actual_support(self):
  for s in SPECIES:
   for p,parent in self.pairs(s):
    for k in range(0,len(p['indices']),3):
     a,b,c=[p['positions'][i*3:i*3+3] for i in p['indices'][k:k+3]]
     if (b[0]-a[0])*(c[1]-a[1])-(b[1]-a[1])*(c[0]-a[0])<1e-10:continue
     x,y,z=[(a[j]+b[j]+c[j])/3 for j in range(3)];h=ray_intersections(parent,x,y);self.assertTrue(h);self.assertGreater(z-max(h),.0001,(s,p['part'],k//3,z-max(h)))
 def test_arms_palms_and_each_digit_have_physical_roots(self):
  for s in ('grimer','muk'):
   ps=self.models[s]['primitives'];b=self.find(ps,'Body /')
   for sign in (-1,1):
    arm=self.find(ps,'Arm /',sign);hand=self.find(ps,'Hand /',sign);self.assertTrue(joins.DonphanTests.overlap(self,b,arm),(s,arm['part']));self.assertTrue(joins.DonphanTests.overlap(self,arm,hand),(s,hand['part']))
    for p in ps:
     if p['part'].startswith('Finger /') and p['part'].split()[-2]==str(sign):self.assertTrue(joins.DonphanTests.overlap(self,hand,p),(s,p['part']))
 def test_sludge_mouth_rims_have_true_open_centers(self):
  for s in ('grimer','muk'):
   ps=self.models[s]['primitives'];p=self.find(ps,'Lip /');x=sum(p['positions'][::3])/(len(p['positions'])//3);y=sum(p['positions'][1::3])/(len(p['positions'])//3)
   self.assertFalse(ray_intersections(p,x,y),s);self.assertTrue(ray_intersections(self.find(ps,'Mouth /'),x,y),s)
   parent=self.find(ps,'Body /')
   for x,y,z in {tuple(p['positions'][i:i+3]) for i in range(0,len(p['positions']),3)}:
    h=ray_intersections(parent,x,y);self.assertTrue(h);self.assertLess(abs(z-max(h)),.036)
 def test_front_and_rear_sludge_folds_follow_actual_body(self):
  for s in ('grimer','muk'):
   ps=self.models[s]['primitives'];b=self.find(ps,'Body /')
   for p in ps:
    if not p['part'].startswith('Body fold /'):continue
    rear=p['part'].split()[-2]=='1';parent=reverse_front(b) if rear else b;mark=reverse_front(p) if rear else p
    for x,y,z in {tuple(mark['positions'][i:i+3]) for i in range(0,len(mark['positions']),3)}:
     h=ray_intersections(parent,x,y);self.assertTrue(h);self.assertLess(abs(z-max(h)),.014,(s,p['part']))
 def test_muk_long_arm_and_ditto_single_soft_silhouette(self):
  ps=self.models['muk']['primitives'];right=self.find(ps,'Arm /',1);left=self.find(ps,'Arm /',-1);self.assertGreater(max(right['positions'][::3]),max(abs(x) for x in left['positions'][::3])*1.25)
  ps=self.models['ditto']['primitives'];self.assertEqual(len(ps),8);b=self.find(ps,'Body /');upper=[b['positions'][i] for i in range(0,len(b['positions']),3) if HEIGHT['ditto']*.55<b['positions'][i+1]<HEIGHT['ditto']*.78];self.assertGreater(max(upper),abs(min(upper))*1.04)
if __name__=='__main__':unittest.main()
