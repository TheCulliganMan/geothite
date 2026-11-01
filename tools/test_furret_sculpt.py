"""Independent closed-band, seam, palette and face constraints for Furret."""
import math,unittest
from furret_sculpt import sculpture,LIGHT,DARK
from sculpt_test_support import assert_closed_models,assert_model_shading,ray_intersections

def seam_sections(parts,prefix):
 sections=[p for p in parts if p['part'].startswith(prefix)]
 if prefix=='Body /':
  coat=next(p for p in parts if p['part'].startswith('Back coat /'))
  i=next(i for i,p in enumerate(sections) if p['part']=='Body / joined cream neck front')
  q=sections[i];sections[i]=dict(q,positions=q['positions']+coat['positions'],normals=q['normals']+coat['normals'])
 return sections

class FurretSculptTests(unittest.TestCase):
 @classmethod
 def setUpClass(cls):cls.model=sculpture();cls.parts=cls.model['primitives']
 def test_closed_outward_geometry_and_shading(self):
  assert_closed_models(self,{'furret':self.model});assert_model_shading(self,{'furret':self.model})
 def test_colored_sections_share_exact_seam_positions_and_normals(self):
  for prefix in ('Body /','Tail /'):
   sections=seam_sections(self.parts,prefix)
   self.assertGreater(len(sections),4)
   def points(p):
    result={}
    for i in range(0,len(p['positions']),3):result.setdefault(tuple(p['positions'][i:i+3]),set()).add(tuple(p['normals'][i:i+3]))
    return result
   for left,right in zip(sections,sections[1:]):
    a,b=points(left),points(right);shared=a.keys()&b.keys();self.assertGreaterEqual(len(shared),24)
    for point in shared:self.assertTrue(a[point]&b[point],(prefix,point))
 def test_source_palette_floor_and_budget(self):
  for word,rgb in [('cream',LIGHT),('brown',DARK)]:
   expected=[v/12.92 if v<=.04045 else ((v+.055)/1.055)**2.4 for v in rgb]
   for p in self.parts:
    if p['part'].startswith(('Body /','Tail /')) and word in p['part']:
     for a,b in zip(p['base_color'],expected):self.assertAlmostEqual(a,b,places=7)
  self.assertLess(sum(len(p['positions'])//3 for p in self.parts),20000)
  for p in self.parts:self.assertGreaterEqual(min(p['positions'][1::3]),0.)
  for p in self.parts:
   if p['part'].startswith('Foot /'):self.assertEqual(min(p['positions'][1::3]),0.)
 def test_face_layers_fit_actual_colored_body(self):
  body={'positions':[],'indices':[]}
  for p in self.parts:
   if p['part'].startswith('Body /'):
    offset=len(body['positions'])//3;body['positions']+=p['positions'];body['indices'] += [i+offset for i in p['indices']]
  for p in self.parts:
   name=p['part'];parent=body if name.startswith(('Eye /','Nose /','Mouth /','Face mark /')) else next((q for q in self.parts if q['part'].startswith('Eye /') and q['part'].endswith(' '+name.split()[-1])),None) if name.startswith('Glint /') else None
   if parent is None:continue
   vs=[p['positions'][i:i+3] for i in range(0,len(p['positions']),3)];checked=0
   for i in range(0,len(p['indices']),3):
    a,b,c=[vs[j] for j in p['indices'][i:i+3]]
    if (b[0]-a[0])*(c[1]-a[1])-(b[1]-a[1])*(c[0]-a[0])<=1e-10:continue
    mid=[sum(v[k] for v in (a,b,c))/3 for k in range(3)];hits=ray_intersections(parent,*mid[:2]);self.assertTrue(hits,name);self.assertGreater(mid[2],max(hits),name);checked+=1
   self.assertGreater(checked,0,name)
if __name__=='__main__':unittest.main()
