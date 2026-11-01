#!/usr/bin/env python3
"""Stored-model proof of distinct, closed and grounded fighting-family anatomy."""
import json,math,unittest
from fighting_glb import read_fighting
from pathlib import Path
from machop_sculpt import sculpture,HEIGHTS
from sculpt_test_support import assert_closed_models,assert_model_shading,horizontal_section_bounds

ROOT=Path(__file__).resolve().parents[1]/'crates/crystal-voxel-view/models/battle_species'

class FightingFamilyTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):cls.models={n:read_fighting(ROOT/(n+'.glb'),n) for n in HEIGHTS}

    def test_exact_recipe_grounding_height_and_geometry_budget(self):
        for name,m in self.models.items():
            with self.subTest(species=name):
                from fighting_glb import geometry_digest
                self.assertEqual(geometry_digest(m),geometry_digest(sculpture(name)));parts=m['primitives']
                self.assertEqual(min(v for p in parts for v in p['positions'][1::3]),0.)
                self.assertAlmostEqual(max(v for p in parts for v in p['positions'][1::3]),HEIGHTS[name],places=6)
                self.assertLess(sum(len(p['indices'])//3 for p in parts),40000)
                self.assertEqual(len(parts),len({p['part'] for p in parts}))

    def test_every_part_is_a_connected_closed_outward_solid(self):
        assert_closed_models(self,self.models)

    def test_actual_stored_normals_agree_with_triangle_winding(self):
        assert_model_shading(self,self.models)

    def test_stage_specific_anatomy_and_fitted_face(self):
        for name,m in self.models.items():
            names=[p['part'] for p in m['primitives']];four=name=='machamp';young=name=='machop'
            expected=4 if four else 2
            for prefix,count in [('Body /',1),('Arm /',expected),('Hand /',expected),('Finger /',expected*(3 if young else 4)),('Thumb /',expected),('Crest /',3),('Foot /',2),('Toe /',0),('Eye / fitted',2),('Eye / red iris',2)]:
                self.assertEqual(sum(n.startswith(prefix) for n in names),count,(name,prefix))
            self.assertEqual(sum(n.startswith('Tail /') for n in names),int(young))
            self.assertEqual(sum(n.startswith('Trunks /') for n in names),int(not young))
            self.assertEqual(sum(n.startswith('Arm marking /') for n in names),12 if name=='machoke' else 0)
            for p in m['primitives']:
                if p['part'].startswith(('Foot /','Toe /')):self.assertEqual(min(p['positions'][1::3]),0.)
                if p['part'].startswith('Eye / red iris'):
                    self.assertLess(max(p['positions'][2::3])-min(p['positions'][2::3]),HEIGHTS[name]*.05)
        # Distinguishing silhouette, not merely the same mesh with another tint.
        widths={n:max(abs(v) for p in m['primitives'] if p['part'].startswith('Body /') for v in p['positions'][::3])/HEIGHTS[n] for n,m in self.models.items()}
        self.assertGreater(widths['machamp'],widths['machoke']*1.12)
        self.assertGreater(widths['machoke'],widths['machop']*1.10)

    def test_upright_shaft_direction_and_complete_flat_soles(self):
        for name,m in self.models.items():
            h=HEIGHTS[name]
            for p in m['primitives']:
                if p['part'].startswith('Leg /'):
                    cuts=[horizontal_section_bounds(p,h*f) for f in (.10,.18)]
                    centers=[(lo[1]+hi[1])/2 for lo,hi in cuts]
                    self.assertLess(abs(centers[0]-centers[1]),.008*h,(name,p['part'],'shin slopes forward'))
                if p['part'].startswith('Foot /'):
                    lo=horizontal_section_bounds(p,.005*h);mid=horizontal_section_bounds(p,.03*h)
                    self.assertGreater((lo[1][0]-lo[0][0])/(mid[1][0]-mid[0][0]),.80,(name,'sole narrows to a round bead'))

    def test_swept_crest_depth_and_juvenile_eye_shape(self):
        for name,m in self.models.items():
            for p in m['primitives']:
                if p['part'].startswith('Crest /'):
                    self.assertGreater(max(p['positions'][2::3])-min(p['positions'][2::3]),.19*HEIGHTS[name])
                if name=='machop' and p['part'].startswith('Eye / fitted'):
                    self.assertGreater((max(p['positions'][1::3])-min(p['positions'][1::3]))/(max(p['positions'][::3])-min(p['positions'][::3])),1.5)

    def test_species_specific_hand_carriage(self):
        hands={name:{p['part']:(max(p['positions'][1::3])+min(p['positions'][1::3]))/2/HEIGHTS[name] for p in m['primitives'] if p['part'].startswith('Hand /')} for name,m in self.models.items()}
        self.assertTrue(all(y>.54 for y in hands['machop'].values()))
        self.assertTrue(all(y<.40 for y in hands['machoke'].values()))
        upper=[y for n,y in hands['machamp'].items() if ' / upper ' in n];lower=[y for n,y in hands['machamp'].items() if ' / lower ' in n]
        self.assertTrue(all(y>.80 for y in upper));self.assertGreater(min(upper)-max(lower),.40)

    def test_juvenile_ribs_are_occupied_surface_relief(self):
        from sculpt_test_support import indexed_ray_intersections as rays
        from fighting_glb import authoring_scale
        body=next(p for p in self.models['machop']['primitives'] if p['part'].startswith('Body /'));scale=authoring_scale('machop')
        for x in (-.075,.075):
            for y in (.535,.573,.611):
                zs=[max(rays(body,x*scale,(y+dy)*scale)) for dy in (-.018,0.,.018)]
                self.assertGreater(zs[1]-(zs[0]+zs[2])/2,.006*HEIGHTS['machop'])

if __name__=='__main__':unittest.main()
