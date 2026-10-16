#!/usr/bin/env python3
"""Stored boxer clothing, connected crown, glove roots and fitted facial checks."""
import copy,json,os,unittest
from collections import defaultdict
from pathlib import Path
from hitmonchan_glb import read_hitmonchan,geometry_digest
from hitmonchan_sculpt import sculpture,HEIGHT
from sculpt_test_support import assert_closed_models,assert_model_shading
from test_kingdra_sculpt import crossed_faces
from sculpt_geometry import translate
from chikorita_sculpt import cross,sub,dot
import test_nidorina_sculpt as attachment_checks
ROOT=Path(os.environ.get('CRYSTAL_HITMONCHAN_TEST_MODELS',Path(__file__).resolve().parents[1]/'crates/crystal-voxel-view/models/battle_species'))

class HitmonchanTests(unittest.TestCase):
 @classmethod
 def setUpClass(cls):cls.m=read_hitmonchan(ROOT/'hitmonchan.glb');cls.ps=cls.m['primitives']
 find=attachment_checks.NidorinaTests.find
 overlap=attachment_checks.NidorinaTests.overlap
 test_fitted_layers_follow_actual_stored_parent=attachment_checks.NidorinaTests.test_fitted_layers_follow_actual_stored_parent
 test_fitted_face_centers_and_edges_clear_parent=attachment_checks.NidorinaTests.test_fitted_face_centers_and_edges_clear_parent
 def test_closed_connected_outward_solids(self):assert_closed_models(self,{'hitmonchan':self.m})
 def test_shading_tracks_actual_faces(self):assert_model_shading(self,{'hitmonchan':self.m})
 def test_front_crown_cap_has_no_internal_normal_seams(self):
  head=self.find('Head /');normals=defaultdict(list)
  for k in range(0,len(head['indices']),3):
   ids=head['indices'][k:k+3];points=[head['positions'][3*j:3*j+3] for j in ids];face=cross(sub(points[1],points[0]),sub(points[2],points[0]))
   if face[2]<=0 or face[2]*face[2]<.49*dot(face,face):continue
   for j,p in zip(ids,points):normals[tuple(p)].append(head['normals'][3*j:3*j+3])
  shared=[ns for ns in normals.values() if len(ns)>1];self.assertGreater(len(shared),100)
  for ns in shared:
   for n in ns[1:]:self.assertGreater(dot(ns[0],n),.99999)
 def test_parity_floor_height_budget_and_boxing_anatomy(self):
  self.assertEqual(geometry_digest(self.m),geometry_digest(sculpture()));self.assertEqual(len(self.ps),len({p['part'] for p in self.ps}));self.assertLess(sum(len(p['indices'])//3 for p in self.ps),100000)
  self.assertEqual(min(v for p in self.ps for v in p['positions'][1::3]),0);self.assertAlmostEqual(max(v for p in self.ps for v in p['positions'][1::3]),HEIGHT,places=6)
  for prefix,n in (('Head /',1),('Tunic /',1),('Skirt /',1),('Belt /',1),('Shoulder guard /',2),('Eye /',2),('Glove /',2),('Glove thumb /',2),('Glove cuff /',2),('Glove seam /',2),('Boot /',2),('Boot cuff /',2),('Boot seam /',2)):
   self.assertEqual(sum(p['part'].startswith(prefix) for p in self.ps),n,prefix)
  h=self.find('Head /');self.assertGreater(max(h['positions'][1::3])-min(h['positions'][1::3]),.49)
  skirt=self.find('Skirt /');low=[abs(skirt['positions'][i]) for i in range(0,len(skirt['positions']),3) if skirt['positions'][i+1]<.42];high=[abs(skirt['positions'][i]) for i in range(0,len(skirt['positions']),3) if skirt['positions'][i+1]>.51];self.assertGreater(max(low),max(high)*1.45)
  self.assertFalse(any('Crownlike hair' in p['part'] for p in self.ps))
 def test_clothes_neck_head_and_shoulders_have_real_volume_roots(self):
  torso=self.find('Tunic /');neck=self.find('Neck /');head=self.find('Head /');skirt=self.find('Skirt /');belt=self.find('Belt /')
  for a,b in ((neck,torso),(head,neck),(skirt,torso),(belt,torso),(belt,skirt)):self.assertTrue(self.overlap(a,b),a['part'])
  for s in (-1,1):self.assertTrue(self.overlap(self.find('Shoulder guard /',s),torso),'shoulder '+str(s))
 def test_both_bent_arms_mitts_thumbs_and_cuffs_have_real_roots(self):
  for s in (-1,1):
   arm=self.find('Arm /',s);shoulder=self.find('Shoulder guard /',s);cuff=self.find('Glove cuff /',s);glove=self.find('Glove /',s);thumb=self.find('Glove thumb /',s)
   for a,b in ((arm,shoulder),(arm,self.find('Tunic /')),(cuff,arm),(glove,cuff),(thumb,glove)):self.assertTrue(self.overlap(a,b),a['part'])
 def test_both_legs_boots_and_ankle_cuffs_are_supported_and_grounded(self):
  for s in (-1,1):
   leg=self.find('Leg /',s);boot=self.find('Boot /',s);cuff=self.find('Boot cuff /',s)
   for a,b in ((leg,self.find('Skirt /')),(boot,leg),(cuff,leg),(cuff,boot)):self.assertTrue(self.overlap(a,b),a['part'])
   self.assertLess(min(boot['positions'][1::3]),.002)
 def pairs(self):
  for p in self.ps:
   n=p['part'];s=n.split()[-1];parent=None
   for child,base in (('Eye socket /','Head /'),('Eye /','Eye socket /'),('Iris /','Eye /'),('Pupil /','Iris /'),('Glint /','Pupil /'),('Glove seam /','Glove /'),('Boot seam /','Boot /')):
    if n.startswith(child):parent=self.find(base,s if base!='Head /' else None)
   if n.startswith('Mouth /'):parent=self.find('Head /')
   if parent:yield p,parent
 def test_crown_costume_and_bent_limbs_do_not_cross_themselves(self):
  for p in self.ps:
   if p['part'].startswith(('Head /','Tunic /','Skirt /','Belt /','Shoulder guard /','Arm /','Leg /','Boot cuff /','Glove cuff /')):self.assertIsNone(crossed_faces(p),p['part'])
 def test_detached_head_glove_thumb_boot_and_skirt_are_rejected(self):
  for prefix,parent in (('Head /','Neck /'),('Glove /','Glove cuff /'),('Glove thumb /','Glove /'),('Boot /','Leg /'),('Skirt /','Tunic /')):
   self.assertFalse(self.overlap(translate(copy.deepcopy(self.find(prefix)),2,0,0),self.find(parent)))

if __name__=='__main__':unittest.main()
