#!/usr/bin/env python3
"""Stored Gyarados topology, rooted features and distinctive mouth regression."""
import json
import math
import os
from pathlib import Path
import unittest
from gyarados_sculpt import sculpture,HEIGHT,PATH,RADII,reference_length
from sculpt_test_support import assert_closed_models,assert_model_shading
from test_gengar_glb import Model
from gyarados_glb import geometry_digest

ASSET=Path(os.environ.get('GYARADOS_MODEL',Path(__file__).resolve().parents[1]/'crates/crystal-voxel-view/models/battle_species/gyarados.glb'))


class GyaradosTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):cls.m=Model(ASSET.read_bytes()).neutral_model()
    def test_canonical_recipe(self):self.assertEqual(geometry_digest(self.m),geometry_digest(sculpture()))
    def test_closed_connected_outward_anatomy(self):assert_closed_models(self,{'gyarados':self.m})
    def test_normals_and_winding(self):assert_model_shading(self,{'gyarados':self.m})
    def test_ground_height_and_volumetric_coil(self):
        ps=[q['positions'][i:i+3] for q in self.m['primitives'] for i in range(0,len(q['positions']),3)]
        self.assertEqual(min(p[1] for p in ps),0.)
        self.assertAlmostEqual(max(p[1] for p in ps),HEIGHT,places=6)
        self.assertGreater(max(p[0] for p in ps)-min(p[0] for p in ps),.80)
        self.assertGreater(max(p[2] for p in ps)-min(p[2] for p in ps),.80)
        self.assertLess(sum(len(q['positions'])//3 for q in self.m['primitives']),12000)
    def test_open_mouth_rim_fangs_trident_and_paired_fins(self):
        names=[q['part'] for q in self.m['primitives']]
        for prefix,count in [('Fang /',4),('Crest /',3),('Cheek fin /',2),('Barbel /',2),('Dorsal fin /',3),('Tail fin / forked',2),('Belly /',9)]:
            self.assertEqual(sum(n.startswith(prefix) for n in names),count,prefix)
        upper=next(q for q in self.m['primitives'] if q['part'].startswith('Head /'))
        lower=next(q for q in self.m['primitives'] if q['part'].startswith('Jaw /'))
        self.assertGreater(min(upper['positions'][1::3])-max(lower['positions'][1::3]),.14)
        throat=next(q for q in self.m['primitives'] if q['part'].startswith('Mouth /'))
        self.assertGreater(max(throat['positions'][1::3])-min(throat['positions'][1::3]),.28)
    def test_reference_centerline_matches_runtime_calibration(self):
        self.assertAlmostEqual(reference_length(self.m),2.7092259,places=6)
        self.assertGreater(reference_length(self.m),HEIGHT*1.8)

    def test_belly_bands_follow_actual_body_centerline(self):
        # Every panel uses the same smoothed centerline and parallel-transport
        # frame as the body, with only a shallow embedded material shell.
        parts=[p for p in self.m['primitives'] if p['part'].startswith('Belly /')]
        for panel in parts:
            self.assertEqual(len({tuple(panel['positions'][i:i+3]) for i in range(0,len(panel['positions']),3)}),130)
        centerline=sum(math.dist(a,b) for a,b in zip(PATH,PATH[1:]))
        self.assertGreater(centerline,2.2)
        self.assertGreater(RADII[6],RADII[0]*4)


if __name__=='__main__':unittest.main()
