#!/usr/bin/env python3
"""Validate canonical bat topology, anatomy and stored face contact."""
import os,unittest
from pathlib import Path
from bat_glb import decode_bat
from bat_sculpt import SPECIES,HEIGHT,sculpture
from sculpt_test_support import assert_closed_models,assert_model_shading
from test_axolotl_sculpt import ray_intersections

ROOT=Path(os.environ.get('CRYSTAL_BAT_TEST_MODELS',Path(__file__).resolve().parents[1]/'crates/crystal-voxel-view/models/battle_species'))

class BatTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):cls.models={s:decode_bat((ROOT/(s+'.glb')).read_bytes(),s) for s in SPECIES}
    def test_closed_connected_oriented_solids(self):assert_closed_models(self,self.models)
    def test_normals_match_actual_triangles(self):assert_model_shading(self,self.models)
    def test_recipe_dimensions_and_budget(self):
        for s,m in self.models.items():
            from bat_glb import geometry_digest
            self.assertEqual(geometry_digest(m),geometry_digest(sculpture(s)));ps=m['primitives']
            self.assertEqual(min(v for p in ps for v in p['positions'][1::3]),0.)
            self.assertAlmostEqual(max(v for p in ps for v in p['positions'][1::3]),HEIGHT,places=6)
            self.assertEqual(len(ps),len({p['part'] for p in ps}))
            self.assertLess(sum(len(p['indices'])//3 for p in ps),6000)
            self.assertLess((ROOT/(s+'.glb')).stat().st_size,500000)
    def test_distinct_wing_and_face_anatomy(self):
        for s,m in self.models.items():
            ps=m['primitives'];names=[p['part'] for p in ps]
            wings=[p for p in ps if p['part'].startswith('Wing /')]
            self.assertEqual(len(wings),4 if s=='crobat' else 2)
            self.assertEqual(sum(n.startswith('Eye /') for n in names),0 if s=='zubat' else 4)
            self.assertEqual(sum(n.startswith('Fang /') for n in names),4)
            for p in wings:
                self.assertGreater(len(set(p['positions'][1::3])),25)
                self.assertGreater(max(p['positions'][2::3])-min(p['positions'][2::3]),.02)
    def test_open_mouth_depth_and_recessed_throat(self):
        for s in ('zubat','golbat'):
            ps=self.models[s]['primitives']
            face=next(p for p in ps if p['part'].startswith('Face /'))
            throat=next(p for p in ps if p['part']=='Mouth / recessed throat')
            self.assertGreater(max(face['positions'][2::3])-max(throat['positions'][2::3]),.10)
            x=0.;y=(min(throat['positions'][1::3])+max(throat['positions'][1::3]))/2
            # An opaque cap across the opening would intersect this center ray.
            self.assertFalse(ray_intersections(face,x,y))
            self.assertTrue(ray_intersections(throat,x,y))
    def test_fitted_eyes_stay_on_actual_face(self):
        for s in ('golbat','crobat'):
            ps=self.models[s]['primitives']
            skin=next(p for p in ps if p['part'].startswith('Face /' if s=='golbat' else 'Body /'))
            for p in ps:
                if not p['part'].startswith('Eye /'):continue
                if s=='golbat':
                    # Keep the complete eye on the front lip. Crossing the
                    # rolled rim to the rear carapace made a jagged white star.
                    self.assertLess(max(p['positions'][2::3])-min(p['positions'][2::3]),.02)
                for i in range(0,len(p['positions']),3):
                    x,y,z=p['positions'][i:i+3];hits=ray_intersections(skin,x,y)
                    self.assertTrue(hits,(s,p['part']))
                    self.assertLess(abs(z-max(hits)),.016,(s,p['part']))

if __name__=='__main__':unittest.main()
