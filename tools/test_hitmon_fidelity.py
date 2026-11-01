#!/usr/bin/env python3
"""Independent stored-volume guards for the three fighting-pose corrections."""
import os
import unittest
from pathlib import Path

from sculpt_test_support import horizontal_section_bounds, indexed_ray_intersections
from test_gengar_glb import Model

ROOT = Path(os.environ.get(
    'CRYSTAL_HITMON_FIDELITY_TEST_MODELS',
    Path(__file__).resolve().parents[1] / 'crates/crystal-voxel-view/models/battle_species'))
HEIGHTS = {'hitmonchan': 1.30, 'hitmonlee': 1.24, 'hitmontop': 1.25}


def bounds(part):
    return [(min(part['positions'][k::3]), max(part['positions'][k::3]))
            for k in range(3)]


def upper_at_absolute_x(part, x):
    """Cut the actual eye triangles vertically; do not reuse the author outline."""
    points = [(abs(a), b, c) for a, b, c in zip(
        part['positions'][::3], part['positions'][1::3], part['positions'][2::3])]
    heights = []
    for k in range(0, len(part['indices']), 3):
        tri = [points[j] for j in part['indices'][k:k+3]]
        for a, b in zip(tri, tri[1:]+tri[:1]):
            if a[0] <= x < b[0] or b[0] <= x < a[0]:
                heights.append(a[1]+(b[1]-a[1])*(x-a[0])/(b[0]-a[0]))
    return max(heights)


class HitmonFidelityTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.models = {name: Model((ROOT/(name+'.glb')).read_bytes()).neutral_model()
                      for name in HEIGHTS}

    def part(self, name, prefix, side=None):
        return next(p for p in self.models[name]['primitives']
                    if p['part'].startswith(prefix)
                    and (side is None or p['part'].endswith(' '+str(side))))

    def test_hitmonchan_has_a_round_skull_and_full_boots_in_profile(self):
        name = 'hitmonchan'
        head = self.part(name, 'Head /')
        low, high = bounds(head)[1]
        for fraction in (.25, .40):
            a, b = horizontal_section_bounds(head, low+fraction*(high-low))
            self.assertGreater(b[1]-a[1], .17*HEIGHTS[name], 'skull is a crown plate')
        for side in (-1, 1):
            boot = bounds(self.part(name, 'Boot /', side))
            self.assertGreater(boot[2][1]-boot[2][0], .17*HEIGHTS[name])
            self.assertGreater(boot[1][1]-boot[1][0], .065*HEIGHTS[name])

    def test_hitmonchan_shins_are_vertical_and_hem_has_an_empty_interior(self):
        name = 'hitmonchan'
        for side in (-1, 1):
            leg = self.part(name, 'Leg /', side)
            low, high = bounds(leg)[1]
            centers = []
            for fraction in (.25, .50, .75):
                a, b = horizontal_section_bounds(leg, low+fraction*(high-low))
                centers.append((a[1]+b[1])/2)
            self.assertLess(max(centers)-min(centers), .004*HEIGHTS[name])
        hem = self.part(name, 'Skirt /')
        y = bounds(hem)[1][0]+.02*HEIGHTS[name]
        hits = sorted(set(round(z, 6) for z in indexed_ray_intersections(hem, 0., y)))
        self.assertEqual(len(hits), 4, 'cloth must surround an open hem')
        self.assertLess(hits[1]-hits[0], .015*HEIGHTS[name])
        self.assertLess(hits[3]-hits[2], .015*HEIGHTS[name])
        self.assertGreater(hits[2]-hits[1], .20*HEIGHTS[name])

    def test_hitmonlee_eye_corners_rise_outward_for_the_game_stare(self):
        for side in (-1, 1):
            eye = self.part('hitmonlee', 'Eye /', side)
            xs = [abs(x) for x in eye['positions'][::3]]
            low, high = min(xs), max(xs)
            inner = upper_at_absolute_x(eye, low+.20*(high-low))
            outer = upper_at_absolute_x(eye, low+.75*(high-low))
            self.assertGreater(outer-inner, .025*HEIGHTS['hitmonlee'])

    def test_hitmonlee_has_heavy_upright_coils_and_broad_upper_body(self):
        name = 'hitmonlee'
        body = bounds(self.part(name, 'Body /'))
        self.assertGreater(body[0][1]-body[0][0], .40*HEIGHTS[name])
        for side in (-1, 1):
            leg = self.part(name, 'Leg spring /', side)
            low, high = bounds(leg)[1]
            centers = []
            for fraction in (.25, .50, .75):
                a, b = horizontal_section_bounds(leg, low+fraction*(high-low))
                centers.append((a[1]+b[1])/2)
                self.assertGreater(b[1]-a[1], .11*HEIGHTS[name])
            self.assertLess(max(centers)-min(centers), .004*HEIGHTS[name])

    def test_hitmontop_has_two_thick_swept_helmet_flanges_and_six_folds(self):
        name = 'hitmontop'
        parts = self.models[name]['primitives']
        self.assertEqual(sum(p['part'].startswith('Helmet crease /') for p in parts), 6)
        head = bounds(self.part(name, 'Head /'))
        for side in (-1, 1):
            flange = bounds(self.part(name, 'Helmet flange /', side))
            self.assertGreater(max(abs(x) for x in flange[0]),
                               max(abs(x) for x in head[0])+.12*HEIGHTS[name])
            self.assertGreater(flange[2][1]-flange[2][0], .085*HEIGHTS[name])

    def test_hitmontop_keeps_compact_raised_limbs_over_a_full_body(self):
        name = 'hitmontop'
        body = bounds(self.part(name, 'Body /'))
        self.assertGreater(body[1][1]-body[1][0], .30*HEIGHTS[name])
        for side in (-1, 1):
            leg = bounds(self.part(name, 'Leg /', side))
            foot = bounds(self.part(name, 'Foot /', side))
            self.assertLess(leg[1][1]-leg[1][0], .19*HEIGHTS[name])
            self.assertGreater(foot[1][0], body[1][1])
            self.assertLess((sum(foot[1])-sum(body[1]))/2, .28*HEIGHTS[name])


if __name__ == '__main__':
    unittest.main()
