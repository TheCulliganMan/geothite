#!/usr/bin/env python3
"""Stored support assemblies must keep foot/shin roles and one walking limb."""
import os,unittest
from pathlib import Path
from test_gengar_glb import Model
ROOT=Path(os.environ.get('CRYSTAL_WORLD_SUPPORT_TEST_MODELS',Path(__file__).resolve().parents[1]/'crates/crystal-voxel-view/models/battle_species'))
class WorldSupportPackingTests(unittest.TestCase):
 def check(self,species,prefixes):
  m=Model((ROOT/(species+'.glb')).read_bytes());checked=0
  for primitive,meta in zip(m.parts,m.doc['extras']['source_part_ranges']):
   names=[p['part'] for p in meta.get('source_parts',[])] or [primitive['extras']['part']]
   source=[n for n in names if n.startswith(prefixes)]
   if not source:continue
   checked+=1;sides={int(n.split()[-2] if n.startswith('Foot claw /') else n.split()[-1]) for n in source}
   with self.subTest(species=species,part=primitive['extras']['part']):
    self.assertEqual(len(sides),1,'walking primitive spans both legs')
    kinds={n.split(' /')[0] for n in source}
    self.assertFalse({'Greave','Foot'}<=kinds,'shin armor and rigid foot must retain distinct walking roles')
  self.assertGreater(checked,0)
 def test_lickitung_thigh_markings_stay_per_leg(self):self.check('lickitung',('Thigh ring /','Thigh center /'))
 def test_mime_hip_and_knee_shells_stay_per_leg(self):self.check('mr__mime',('Hip /','Knee /'))
 def test_scizor_armor_pads_and_nails_keep_per_leg_roles(self):self.check('scizor',('Greave /','Foot /','Toe cap /'))
 def test_scyther_armor_pads_and_claws_keep_per_leg_roles(self):self.check('scyther',('Greave /','Foot /','Foot claw /'))
if __name__=='__main__':unittest.main()
