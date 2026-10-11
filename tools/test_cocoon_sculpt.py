#!/usr/bin/env python3
"""Verify canonical cocoon contours and fitted faceted anatomy."""
import json,unittest
from pathlib import Path
from cocoon_sculpt import sculpture,SPECIES,HEIGHT
from sculpt_test_support import assert_closed_models,assert_model_shading
from abra_sculpt import surface

ROOT=Path(__file__).resolve().parents[1]/'crates/crystal-voxel-view/models/battle_species'

class CocoonTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):cls.models={s:json.loads((ROOT/(s+'.mesh.json')).read_bytes()) for s in SPECIES}
    def test_closed_connected_oriented_parts(self):assert_closed_models(self,self.models)
    def test_normals_match_winding(self):assert_model_shading(self,self.models)

    def test_canonical_recipe_and_physical_bounds(self):
        for s,m in self.models.items():
            self.assertEqual(m,sculpture(s));parts=m['primitives'];names=[p['part'] for p in parts]
            self.assertEqual(len(names),len(set(names)))
            self.assertEqual(sum(n.startswith('Shell /') for n in names),1)
            self.assertEqual(min(v for p in parts for v in p['positions'][1::3]),0.)
            self.assertAlmostEqual(max(v for p in parts for v in p['positions'][1::3]),HEIGHT[s],places=6)
            self.assertLess(sum(len(p['indices'])//3 for p in parts),1000)

    def test_faces_and_plates_follow_actual_shell(self):
        for s,m in self.models.items():
            skin=surface(m['primitives'][0])
            for p in m['primitives'][1:]:
                if p['part'].startswith(('Eye /','Brow /','Arm fold /','Shell fold /')):
                    for i in range(0,len(p['positions']),3):
                        x,y,z=p['positions'][i:i+3]
                        self.assertLess(abs(z-skin(y,x)),.023,(s,p['part']))

    def test_metapod_has_a_continuous_bent_front_profile(self):
        skin=surface(self.models['metapod']['primitives'][0])
        self.assertGreater(skin(.43,0)-skin(.25,0),.17)
        self.assertGreater(skin(.43,0)-skin(.71,0),.13)
        self.assertGreater(skin(.43,0)-skin(.77,0),.20)

if __name__=='__main__':unittest.main()
