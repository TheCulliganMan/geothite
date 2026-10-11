#!/usr/bin/env python3
"""Independent stored geometry checks for Miltank and Tauros."""
import json,unittest
from pathlib import Path
from bovine_sculpt import sculpture,SPECIES,HEIGHTS
from sculpt_test_support import assert_closed_models,assert_model_shading
from test_axolotl_sculpt import ray_intersections
ROOT=Path(__file__).resolve().parents[1]/'crates/crystal-voxel-view/models/battle_species'
class BovineTests(unittest.TestCase):
 @classmethod
 def setUpClass(cls):cls.models={s:json.loads((ROOT/(s+'.mesh.json')).read_bytes()) for s in SPECIES}
 def test_closed_connected_oriented_parts(self):assert_closed_models(self,self.models)
 def test_normals_match_stored_triangles(self):assert_model_shading(self,self.models)
 def test_regeneration_scale_and_anatomy(self):
  for s,m in self.models.items():
   self.assertEqual(m,sculpture(s));ps=m['primitives']
   self.assertEqual(len(ps),len({p['part'] for p in ps}))
   self.assertEqual(min(v for p in ps for v in p['positions'][1::3]),0.)
   self.assertAlmostEqual(max(v for p in ps for v in p['positions'][1::3]),HEIGHTS[s],places=6)
   self.assertLess(sum(len(p['indices'])//3 for p in ps),21000)
   counts=(('Teat /',4),('Leg /',2),('Foot / grounded',2),('Hand / dark',2),('Tail / rounded',1),('Spot /',3)) if s=='miltank' else (('Leg /',4),('Hoof /',4),('Tail / separate',3),('Tail / pointed',3),('Rivet / gray',3))
   for prefix,count in counts:self.assertEqual(sum(p['part'].startswith(prefix) for p in ps),count,(s,prefix))
   self.assertEqual(sum(p['part'].startswith('Horn /') for p in ps),2)
 def overlap(self,a,b):
  xl=max(min(a['positions'][::3]),min(b['positions'][::3]));xh=min(max(a['positions'][::3]),max(b['positions'][::3]))
  yl=max(min(a['positions'][1::3]),min(b['positions'][1::3]));yh=min(max(a['positions'][1::3]),max(b['positions'][1::3]))
  good=False
  for i in range(1,12):
   for j in range(1,16):
    x=xl+(xh-xl)*i/12;y=yl+(yh-yl)*j/16;aa=ray_intersections(a,x,y);bb=ray_intersections(b,x,y)
    if aa and bb and min(max(aa),max(bb))-max(min(aa),min(bb))>.001:good=True;break
   if good:break
  self.assertTrue(good,(a['part'],b['part']))
 def test_real_udder_and_four_teat_contacts(self):
  ps=self.models['miltank']['primitives'];udder=next(p for p in ps if p['part'].startswith('Udder /'))
  self.overlap(udder,ps[0])
  for p in ps:
   if p['part'].startswith('Teat /'):self.overlap(p,udder)
 def test_grounded_hooves_and_limb_support(self):
  for s,m in self.models.items():
   ps=m['primitives']
   for p in ps:
    name=p['part']
    if name.startswith('Leg /'):self.overlap(p,ps[0])
    elif name.startswith('Arm /'):self.overlap(p,ps[0])
    elif name.startswith(('Foot / grounded','Hoof / grounded','Hand / dark')):
     suffix=' '.join(name.split()[-2:]) if s=='tauros' else name.split()[-1]
     prefix='Arm /' if name.startswith('Hand /') else 'Leg /'
     support=next(q for q in ps if q['part'].startswith(prefix) and ' '.join(q['part'].split()[-len(suffix.split()):])==suffix);self.overlap(p,support)
     if not name.startswith('Hand /'):self.assertEqual(min(p['positions'][1::3]),0.)
 def test_head_mane_horns_and_tail_attachments(self):
  for s,m in self.models.items():
   ps=m['primitives'];head=ps[1] if s=='miltank' else ps[2]
   self.overlap(head,ps[0] if s=='miltank' else ps[1])
   if s=='tauros':self.overlap(ps[1],ps[0])
   for p in ps:
    name=p['part']
    if name.startswith('Horn /'):self.overlap(p,head)
    elif name.startswith(('Tail / separate','Tail / thin')):self.overlap(p,ps[0])
    elif name.startswith('Tail / pointed'):
     q=next(q for q in ps if q['part']=='Tail / separate curved brown whip '+name.split()[-1]);self.overlap(p,q)
    elif name.startswith('Tail / rounded'):self.overlap(p,next(q for q in ps if q['part'].startswith('Tail / thin')))
 def test_ears_muzzles_and_surface_markings(self):
  for s,m in self.models.items():
   ps=m['primitives'];head=ps[1] if s=='miltank' else ps[2]
   for p in ps:
    if p['part'].startswith(('Ear / broad','Muzzle /')):self.overlap(p,head)
    elif p['part'].startswith('Nose /'):
     support=next(q for q in ps if q['part'].startswith('Muzzle /')) if s=='miltank' else head
     self.overlap(p,support)
    elif p['part'].startswith('Spot /'):
     rear=dict(ps[0]);rear['positions']=[-v if i%3!=1 else v for i,v in enumerate(rear['positions'])]
     for i in range(0,len(p['positions']),3):
      x,y,z=p['positions'][i:i+3];hits=ray_intersections(rear,-x,y)
      self.assertTrue(hits);self.assertLess(abs(-z-max(hits)),.006)
 def test_facial_features_are_fitted_to_actual_skin(self):
  for s,m in self.models.items():
   ps=m['primitives'];head=ps[1] if s=='miltank' else ps[2]
   for p in ps:
    name=p['part'];side=name.split()[-1]
    if name.startswith('Face /'):skin=head
    elif name.startswith('Eye / fitted dark'):skin=next(q for q in ps if q['part'].startswith('Face /')) if s=='miltank' else head
    elif name.startswith('Eye / fitted ivory'):skin=next(q for q in ps if q['part']=='Eye / fitted dark socket '+side)
    elif name.startswith('Eye / fitted blue'):skin=next(q for q in ps if q['part']=='Eye / fitted ivory white '+side)
    elif name.startswith('Eye / dark'):skin=next(q for q in ps if q['part']==('Eye / fitted blue iris ' if s=='miltank' else 'Eye / fitted ivory white ')+side)
    elif name.startswith('Rivet / dark'):skin=head
    elif name.startswith('Rivet / gray'):skin=next(q for q in ps if q['part']=='Rivet / dark forehead rim '+side)
    else:continue
    for i in range(0,len(p['positions']),3):
     x,y,z=p['positions'][i:i+3];hits=ray_intersections(skin,x,y);self.assertTrue(hits,(s,name));self.assertLess(abs(z-max(hits)),.028 if name.startswith('Face /') else .016,(s,name))
if __name__=='__main__':unittest.main()
