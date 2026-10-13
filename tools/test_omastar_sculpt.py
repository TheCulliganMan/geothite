#!/usr/bin/env python3
"""Stored adult shell, eye visibility, volume roots and unoccupied beak cross."""
import copy
import json
import os
import unittest
from pathlib import Path

from omastar_sculpt import sculpture, HEIGHT, YELLOW, IVORY
from sculpt_geometry import linear, translate, ellipsoid
from sculpt_test_support import assert_closed_models, assert_model_shading
from test_kingdra_sculpt import crossed_faces
from test_slime_sculpt import ray_intersections
import test_nidorina_sculpt as attachments

ROOT = Path(os.environ.get('CRYSTAL_OMASTAR_TEST_MODELS',
            Path(__file__).resolve().parents[1]/'crates/crystal-voxel-view/models/battle_species'))


def open_cross(parts):
    well = next(p for p in parts if p['part'].startswith('Mouth /'))
    cy = (min(well['positions'][1::3])+max(well['positions'][1::3]))/2
    radius = max(well['positions'][::3])
    beaks = [p for p in parts if p['part'].startswith('Beak /')]
    for a in (-.65, -.30, 0., .30, .65):
        for x, y in ((a*radius, cy), (0, cy+a*radius)):
            if any(ray_intersections(p, x, y) for p in beaks):
                return False
    return True


class OmastarTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.model = json.loads((ROOT/'omastar.mesh.json').read_bytes())

    def setUp(self):
        self.m = self.model
        self.ps = self.m['primitives']

    find = attachments.NidorinaTests.find
    overlap = attachments.NidorinaTests.overlap

    def test_closed_outward_surfaces(self):
        assert_closed_models(self, {'omastar': self.m})

    def test_normals_follow_actual_faces(self):
        assert_model_shading(self, {'omastar': self.m})

    def test_recipe_height_floor_palette_and_budget(self):
        self.assertEqual(self.m, sculpture())
        self.assertEqual(len(self.ps), 43)
        self.assertEqual(len({p['part'] for p in self.ps}), len(self.ps))
        self.assertEqual(min(v for p in self.ps for v in p['positions'][1::3]), 0)
        self.assertAlmostEqual(max(v for p in self.ps for v in p['positions'][1::3]), HEIGHT, places=6)
        self.assertLess(sum(len(p['indices'])//3 for p in self.ps), 22000)
        for side in (-1, 1):
            self.assertEqual(self.find('Eye /', side)['base_color'], linear(YELLOW))
        for p in self.ps:
            if p['part'].startswith('Beak /'):
                self.assertEqual(p['base_color'], linear(IVORY))

    def test_shell_mantle_eyes_tentacles_and_beak_have_volume_roots(self):
        body = self.find('Body /')
        self.assertTrue(self.overlap(self.find('Shell /'), body))
        for p in self.ps:
            if p['part'].startswith(('Eye /', 'Tentacle /', 'Beak /')):
                self.assertTrue(self.overlap(p, body), p['part'])

    def test_spines_ribs_and_spirals_touch_actual_shell(self):
        shell = self.find('Shell /')
        for p in self.ps:
            if p['part'].startswith(('Spine /', 'Shell seam /', 'Shell spiral /')):
                self.assertTrue(self.overlap(p, shell), p['part'])
        self.assertEqual(sum(p['part'].startswith('Spine /') for p in self.ps), 6)

    def pairs(self):
        for side in (-1, 1):
            yield self.find('Pupil /', side), self.find('Eye /', side)
            yield self.find('Glint /', side), self.find('Pupil /', side)

    def test_fitted_eyes_follow_actual_stored_parents(self):
        attachments.NidorinaTests.test_fitted_layers_follow_actual_stored_parent(self)

    def test_fitted_eye_centers_and_edges_clear_actual_parents(self):
        attachments.NidorinaTests.test_fitted_face_centers_and_edges_clear_parent(self)

    def test_ten_tentacles_and_shell_details_are_uncrossed(self):
        self.assertEqual(sum(p['part'].startswith('Tentacle /') for p in self.ps), 10)
        for p in self.ps:
            if p['part'].startswith(('Tentacle /', 'Spine /', 'Shell seam /', 'Shell spiral /')):
                self.assertIsNone(crossed_faces(p), p['part'])

    def test_four_beak_quadrants_leave_actual_cross_shaped_gap(self):
        self.assertEqual(sum(p['part'].startswith('Beak /') for p in self.ps), 4)
        self.assertTrue(open_cross(self.ps))

    def test_filled_beak_negative_is_rejected(self):
        well = self.find('Mouth /')
        cy = (min(well['positions'][1::3])+max(well['positions'][1::3]))/2
        cz = max(well['positions'][2::3])
        filled = ellipsoid('Beak / invalid filled cross', IVORY,
                           (0, cy, cz+.04), (.08, .08, .025), 18, 12)
        self.assertFalse(open_cross(self.ps+[filled]))

    def test_mantle_shell_spines_and_tentacles_leave_pupils_clear(self):
        for side in (-1, 1):
            pupil = self.find('Pupil /', side)
            for x, y, z in zip(pupil['positions'][::3], pupil['positions'][1::3], pupil['positions'][2::3]):
                for p in self.ps:
                    if not p['part'].startswith(('Body /', 'Shell /', 'Spine /', 'Tentacle /')):
                        continue
                    hits = ray_intersections(p, x, y)
                    if hits:
                        self.assertLess(max(hits), z, p['part'])

    def test_detached_anatomy_is_rejected(self):
        for child, parent in (('Tentacle /', 'Body /'), ('Eye /', 'Body /'),
                              ('Spine /', 'Shell /'), ('Beak /', 'Body /')):
            self.assertFalse(self.overlap(translate(copy.deepcopy(self.find(child)), 3, 0, 0),
                                          self.find(parent)))


if __name__ == '__main__':
    unittest.main()
