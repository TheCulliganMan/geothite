#!/usr/bin/env python3
"""Independent mesh sections for the Tyrogue, Smoochum and Jynx corrections."""
import os
import unittest
from pathlib import Path

from sculpt_test_support import horizontal_section_bounds, indexed_ray_intersections
from test_gengar_glb import Model
from test_ninetales_glb import logical_model

ROOT = Path(os.environ.get(
    'CRYSTAL_ICE_FIGHTER_FIDELITY_TEST_MODELS',
    Path(__file__).resolve().parents[1]/'crates/crystal-voxel-view/models/battle_species'))
HEIGHTS = {'tyrogue': .86, 'smoochum': .78, 'jynx': 1.28}


def bounds(part):
    return [(min(part['positions'][k::3]), max(part['positions'][k::3]))
            for k in range(3)]


class IceFighterFidelityTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.models = {}
        for name in HEIGHTS:
            model = Model((ROOT/(name+'.glb')).read_bytes())
            if name == 'jynx':
                model = logical_model(model)
            cls.models[name] = model.neutral_model()

    def part(self, name, prefix, side=None):
        return next(p for p in self.models[name]['primitives']
                    if p['part'].startswith(prefix)
                    and (side is None or p['part'].endswith(' '+str(side))))

    def test_tyrogue_has_a_round_skull_in_both_profiles(self):
        head = self.part('tyrogue', 'Head /')
        low, high = bounds(head)[1]
        for fraction in (.28, .43, .58):
            a, b = horizontal_section_bounds(head, low+fraction*(high-low))
            self.assertGreater(b[1]-a[1], .24*HEIGHTS['tyrogue'], 'skull is a plate')

    def test_tyrogue_has_short_bent_supports_full_shoes_and_an_asymmetric_guard(self):
        h = HEIGHTS['tyrogue']
        for side in (-1, 1):
            leg = self.part('tyrogue', 'Leg /', side)
            low, high = bounds(leg)[1]
            self.assertLess(high-low, .23*h)
            centers = []
            # Trace the ankle-to-knee shaft separately from the outward thigh.
            for fraction in (.15, .30, .45):
                a, b = horizontal_section_bounds(leg, low+fraction*(high-low))
                centers.append((a[1]+b[1])/2)
                self.assertGreater(b[1]-a[1], .075*h)
            self.assertLess(max(centers)-min(centers), .012*h, 'uniform forward shin slope')
            shoe = bounds(self.part('tyrogue', 'Shoe /', side))
            self.assertGreater(shoe[1][1]-shoe[1][0], .070*h)
        hands = [sum(bounds(self.part('tyrogue', 'Palm /', s))[1])/2 for s in (-1, 1)]
        self.assertGreater(hands[1]-hands[0], .22*h)

    def test_smoochum_has_a_large_head_over_a_compact_body_and_stub_feet(self):
        h = HEIGHTS['smoochum']
        head = bounds(self.part('smoochum', 'Head /'))
        body = bounds(self.part('smoochum', 'Body /'))
        self.assertGreater(head[1][1]-head[1][0], .40*h)
        self.assertLess(body[1][1]-body[1][0], .27*h)
        for side in (-1, 1):
            leg = bounds(self.part('smoochum', 'Leg /', side))
            self.assertLess(leg[1][1]-leg[1][0], .14*h)
        lip = bounds(self.part('smoochum', 'Lips /'))
        self.assertGreater(lip[0][1]-lip[0][0], 1.6*(lip[1][1]-lip[1][0]))

    def test_smoochum_fringe_curves_back_into_the_crown(self):
        fringe = self.part('smoochum', 'Bangs /')
        low, high = bounds(fringe)[1]
        x = bounds(fringe)[0][1]*.20  # Stay beside the genuine center notch.
        lower = max(indexed_ray_intersections(fringe, x, low+.12*(high-low)))
        upper = max(indexed_ray_intersections(fringe, x, low+.92*(high-low)))
        self.assertGreater(lower-upper, .045*HEIGHTS['smoochum'], 'fringe is a projecting slab')

    def test_smoochum_temples_have_occupied_fringe_to_crown_joins(self):
        fringe = self.part('smoochum', 'Bangs /')
        crown = self.part('smoochum', 'Hair crown /')
        fb = bounds(fringe)
        for direction in (-1, 1):
            x = direction*.82*fb[0][1]
            for fraction in (.40, .45, .50):
                y = fb[1][0]+fraction*(fb[1][1]-fb[1][0])
                edge = indexed_ray_intersections(fringe, x, y)
                backing = indexed_ray_intersections(crown, x, y)
                self.assertTrue(edge and backing, 'temple has an exposed gap behind the fringe')
                self.assertGreater(min(max(edge), max(backing))-max(min(edge), min(backing)),
                                   .001*HEIGHTS['smoochum'])

    def test_both_kiss_openings_are_real_recesses_in_the_head(self):
        for name in ('smoochum', 'jynx'):
            h = HEIGHTS[name]
            head = self.part(name, 'Head /')
            mouth = self.part(name, 'Mouth /')
            lip = self.part(name, 'Lips /')
            mb, lb = bounds(mouth), bounds(lip)
            cx, cy = sum(mb[0])/2, sum(mb[1])/2
            radius = (lb[0][1]-lb[0][0])/2
            for x in (cx-.20*radius, cx, cx+.20*radius):
                self.assertFalse(indexed_ray_intersections(lip, x, cy), 'lip fills aperture')
                backing = max(indexed_ray_intersections(head, x, cy))
                lining = max(indexed_ray_intersections(mouth, x, cy))
                self.assertGreater(lining-backing, .0001*h)
                self.assertLess(lining-backing, .004*h)
                for direction in (-1, 1):
                    adjacent = max(indexed_ray_intersections(head, cx+direction*radius, cy))
                    self.assertGreater(adjacent-backing, .018*h, 'dark paint covers an uncut face')
                # Sample air ahead of the real occupied backing, within the
                # opening's actual revised rim, rather than at a former mouth.
                probe = lining+.010*h
                self.assertLess(probe, lb[2][1]-.005*h)
                for part in (head, mouth, lip):
                    hits = sorted(indexed_ray_intersections(part, x, cy))
                    self.assertEqual(sum(z>probe for z in hits)%2, 0, (name, part['part']))

    def test_jynx_has_a_heavy_bell_skirt_and_two_distinct_arm_positions(self):
        h = HEIGHTS['jynx']
        gown = self.part('jynx', 'Gown /')
        a, b = horizontal_section_bounds(gown, .20*h)
        self.assertGreater(b[0]-a[0], .48*h)
        self.assertGreater(b[1]-a[1], .33*h)
        hands = [bounds(self.part('jynx', 'Palm /', side)) for side in (-1, 1)]
        self.assertGreater(abs(sum(hands[0][1])-sum(hands[1][1]))/2, .12*h)
        self.assertGreater(abs(sum(hands[0][2])-sum(hands[1][2]))/2, .10*h)


if __name__ == '__main__':
    unittest.main()
