"""Independent anatomical constraints for the Suicune authoring draft."""
import math,unittest
from copy import deepcopy
from suicune_sculpt import sculpture
from sculpt_test_support import assert_closed_models,assert_model_shading,ray_intersections

class SuicuneSculptTests(unittest.TestCase):
 @classmethod
 def setUpClass(cls):cls.model=sculpture();cls.parts=cls.model['primitives']
 def test_closed_outward_solids_and_shading(self):
  assert_closed_models(self,{'suicune':self.model});assert_model_shading(self,{'suicune':self.model})
 def test_crest_has_a_real_opening_and_ribbons_are_broad(self):
  crest=next(p for p in self.parts if p['part'].startswith('Crest /'))
  vertices=[tuple(crest['positions'][i:i+3]) for i in range(0,len(crest['positions']),3)];edges=set()
  for i in range(0,len(crest['indices']),3):
   a,b,c=[vertices[j] for j in crest['indices'][i:i+3]]
   for x,y in ((a,b),(b,c),(c,a)):edges.add(tuple(sorted((x,y))))
  self.assertEqual(len(set(vertices))-len(edges)+len(crest['indices'])//3,0)
  y=(min(v[1] for v in vertices)+max(v[1] for v in vertices))/2
  self.assertEqual(ray_intersections(crest,0.,y),[])
  ribbons=[p for p in self.parts if p['part'].startswith('Ribbon /')];self.assertEqual(len(ribbons),2)
  self.assertTrue(all(len(p['indices'])//3>500 for p in ribbons))
  for ribbon in ribbons:
   points=[ribbon['positions'][i:i+3] for i in range(0,len(ribbon['positions']),3)]
   cap=[points[i] for i in set(ribbon['indices'][:30])]
   width=max(math.dist(a[:2],b[:2]) for a in cap for b in cap)
   depth=max(p[2] for p in cap)-min(p[2] for p in cap)
   self.assertGreater(width,depth*3.)
 def test_source_palette_and_planted_silhouette(self):
  for prefix,rgb555 in [('Body /',(11,23,31)),('Mane /',(14,2,20))]:
   p=next(p for p in self.parts if p['part'].startswith(prefix));rgb=[((v<<3)|(v>>2))/255 for v in rgb555]
   for actual,c in zip(p['base_color'],rgb):self.assertAlmostEqual(actual,c/12.92 if c<=.04045 else ((c+.055)/1.055)**2.4,places=7)
  self.assertLess(sum(len(p['positions'])//3 for p in self.parts),20000)
  self.assertAlmostEqual(max(v for p in self.parts for v in p['positions'][1::3]),1.6,places=6)
  feet=[p for p in self.parts if p['part'].startswith('Foot /')];self.assertEqual(len(feet),4)
  for p in self.parts:self.assertGreaterEqual(min(p['positions'][1::3]),0.)
  for p in feet:self.assertEqual(min(p['positions'][1::3]),0.)
 def test_markings_and_eye_layers_clear_parent_triangles(self):
  def side_view(p,side):
   p=deepcopy(p);vs=p['positions'];p['positions']=[v for i in range(0,len(vs),3) for v in (-side*vs[i+2],vs[i+1],side*vs[i])];return p
  for p in self.parts:
   name=p['part'];child=p;parent=None;side=name.split()[-1]
   if name.startswith('Body diamond /'):
    sign=int(name.split()[-2]);child=side_view(p,sign);parent=side_view(next(q for q in self.parts if q['part'].startswith('Body /')),sign)
   elif name.startswith('Leg diamond /'):
    sign=int(side);child=side_view(p,sign);parent=side_view(next(q for q in self.parts if q['part'].startswith('Leg /') and 'rear' in q['part'] and q['part'].endswith(' '+side)),sign)
   else:
    prefix=next((b for a,b in {'Bib /':'Neck /','Eye socket /':'Head /','Eye /':'Eye socket /','Iris /':'Eye /','Pupil /':'Iris /'}.items() if name.startswith(a)),None)
    if prefix:parent=next(q for q in self.parts if q['part'].startswith(prefix) and (prefix in ('Neck /','Head /') or q['part'].endswith(' '+side)))
   if parent is None:continue
   vs=[child['positions'][i:i+3] for i in range(0,len(child['positions']),3)];checked=0
   for i in range(0,len(child['indices']),3):
    a,b,c=[vs[j] for j in child['indices'][i:i+3]]
    if (b[0]-a[0])*(c[1]-a[1])-(b[1]-a[1])*(c[0]-a[0])<=1e-10:continue
    mid=[sum(v[k] for v in (a,b,c))/3 for k in range(3)];hits=ray_intersections(parent,*mid[:2]);self.assertTrue(hits,name);self.assertGreater(mid[2],max(hits),name);checked+=1
   self.assertGreater(checked,8,name)
if __name__=='__main__':unittest.main()
