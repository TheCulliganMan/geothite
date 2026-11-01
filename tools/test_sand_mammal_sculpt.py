#!/usr/bin/env python3
"""Geometry gates for the original canonical armored-mammal sculptures."""
import math
import unittest
from sand_mammal_sculpt import sculpture, HEIGHTS
from sculpt_test_support import assert_closed_models, assert_model_shading, parts_overlap


class SandMammalSculptTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.models = {s: sculpture(s) for s in HEIGHTS}

    def test_closed_connected_outward_solids(self):
        assert_closed_models(self, self.models)

    def test_actual_triangle_shading(self):
        assert_model_shading(self, self.models)

    def test_grounding_height_and_unchanged_budgets(self):
        for name, model in self.models.items():
            parts = model['primitives']
            self.assertEqual(min(v for p in parts for v in p['positions'][1::3]), 0.)
            self.assertAlmostEqual(max(v for p in parts for v in p['positions'][1::3]), HEIGHTS[name], places=6)
            self.assertLessEqual(len(parts), 64)
            self.assertLessEqual(sum(len(p['positions'])//3 for p in parts), 20000)
            self.assertLessEqual(sum(len(p['indices'])//3 for p in parts), 40000)
            self.assertEqual(len(parts), len({p['part'] for p in parts}))
            for p in parts:
                self.assertTrue(all(math.isfinite(v) for v in p['positions']))
                if p['part'].startswith(('Foot /', 'Toe /')):
                    self.assertLess(min(p['positions'][1::3]), .001)

    def test_stage_specific_armor_and_complete_digits(self):
        for name, model in self.models.items():
            names = [p['part'] for p in model['primitives']]
            for prefix, expected in [('Body /', 1), ('Head /', 1), ('Shell /', 1),
                                     ('Tail /', 1), ('Arm /', 2), ('Hand /', 2),
                                     ('Claw /', 4 if name=='sandslash' else 6), ('Leg /', 2), ('Foot /', 2),
                                     ('Toe /', 6), ('Eye /', 2), ('Pupil /', 2),
                                     ('Plate /', 18 if name=='sandshrew' else 0),
                                     ('Quill /', 18 if name=='sandslash' else 0)]:
                self.assertEqual(sum(n.startswith(prefix) for n in names), expected, (name, prefix))

    def test_armor_is_rooted_in_actual_shell_volume(self):
        for name, model in self.models.items():
            shell = next(p for p in model['primitives'] if p['part'].startswith('Shell /'))
            for p in model['primitives']:
                if p['part'].startswith(('Plate /', 'Quill /')):
                    self.assertTrue(parts_overlap(shell, p), (name, p['part']))

    def test_appendages_are_rooted_in_actual_parent_volume(self):
        for name, model in self.models.items():
            parts = model['primitives']
            def find(prefix):
                return next(p for p in parts if p['part'].startswith(prefix))
            for p in parts:
                title = p['part'];parent = None
                if title.startswith(('Head /', 'Haunch /', 'Arm /', 'Tail /', 'Shell /')):
                    parent = find('Body /')
                elif title.startswith(('Muzzle /', 'Ear /')):
                    parent = find('Head /')
                elif title.startswith('Claw /'):
                    side = title.split()[-2]
                    parent = next(q for q in parts if q['part'].startswith('Hand /') and q['part'].split()[-1]==side)
                elif title.startswith('Toe /'):
                    side = title.split()[-2]
                    parent = next(q for q in parts if q['part'].startswith('Foot /') and q['part'].split()[-1]==side)
                if parent is not None:
                    self.assertTrue(parts_overlap(parent, p), (name, title))


if __name__ == '__main__':
    unittest.main()
