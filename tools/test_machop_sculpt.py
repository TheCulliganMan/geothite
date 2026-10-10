#!/usr/bin/env python3
"""Stored-model proof of distinct, closed and grounded fighting-family anatomy."""
import json,math,unittest
from pathlib import Path
from machop_sculpt import sculpture,HEIGHTS
from sculpt_test_support import assert_closed_models,assert_model_shading

ROOT=Path(__file__).resolve().parents[1]/'crates/crystal-voxel-view/models/battle_species'

class FightingFamilyTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):cls.models={n:json.loads((ROOT/(n+'.mesh.json')).read_bytes()) for n in HEIGHTS}

    def test_exact_recipe_grounding_height_and_geometry_budget(self):
        for name,m in self.models.items():
            with self.subTest(species=name):
                self.assertEqual(m,sculpture(name));parts=m['primitives']
                self.assertEqual(min(v for p in parts for v in p['positions'][1::3]),0.)
                self.assertAlmostEqual(max(v for p in parts for v in p['positions'][1::3]),HEIGHTS[name],places=6)
                self.assertLess(sum(len(p['indices'])//3 for p in parts),7000)
                self.assertEqual(len(parts),len({p['part'] for p in parts}))

    def test_every_part_is_a_connected_closed_outward_solid(self):
        assert_closed_models(self,self.models)

    def test_actual_stored_normals_agree_with_triangle_winding(self):
        assert_model_shading(self,self.models)

    def test_stage_specific_anatomy_and_fitted_face(self):
        for name,m in self.models.items():
            names=[p['part'] for p in m['primitives']];four=name=='machamp';young=name=='machop'
            expected=4 if four else 2
            for prefix,count in [('Body /',1),('Arm /',expected),('Hand /',expected),('Finger /',expected*3),('Thumb /',expected),('Crest /',3),('Foot /',2),('Toe /',6),('Eye / fitted',2),('Eye / narrow',2)]:
                self.assertEqual(sum(n.startswith(prefix) for n in names),count,(name,prefix))
            self.assertEqual(sum(n.startswith('Tail /') for n in names),int(young))
            self.assertEqual(sum(n.startswith('Trunks /') for n in names),int(not young))
            self.assertEqual(sum(n.startswith('Arm marking /') for n in names),4 if name=='machoke' else 0)
            for p in m['primitives']:
                if p['part'].startswith(('Foot /','Toe /')):self.assertEqual(min(p['positions'][1::3]),0.)
                if p['part'].startswith('Eye /'):
                    self.assertLess(max(p['positions'][2::3])-min(p['positions'][2::3]),HEIGHTS[name]*.05)
        # Distinguishing silhouette, not merely the same mesh with another tint.
        widths={n:max(abs(v) for p in m['primitives'] if p['part'].startswith('Body /') for v in p['positions'][::3])/HEIGHTS[n] for n,m in self.models.items()}
        self.assertGreater(widths['machamp'],widths['machoke']*1.12)
        self.assertGreater(widths['machoke'],widths['machop']*1.10)

if __name__=='__main__':unittest.main()
