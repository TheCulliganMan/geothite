#!/usr/bin/env python3
"""Stored source proof of fox-family anatomy, species identity and oriented solids."""
import json,unittest
from pathlib import Path
from abra_sculpt import sculpture,HEIGHT
from sculpt_test_support import assert_closed_models,assert_model_shading
ROOT=Path(__file__).resolve().parents[1]/'crates/crystal-voxel-view/models/battle_species'

class FoxTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):cls.models={n:json.loads((ROOT/(n+'.mesh.json')).read_bytes()) for n in ('abra','alakazam')}
    def test_closed_connected_anatomy(self):assert_closed_models(self,self.models)
    def test_actual_triangle_shading(self):assert_model_shading(self,self.models)
    def test_recipe_height_and_ground(self):
        for n,m in self.models.items():
            self.assertEqual(m,sculpture(n))
            names=[p['part'] for p in m['primitives']];self.assertEqual(len(names),len(set(names)))
            self.assertEqual(min(v for p in m['primitives'] for v in p['positions'][1::3]),0.)
            self.assertAlmostEqual(max(v for p in m['primitives'] for v in p['positions'][1::3]),HEIGHT,places=6)
            self.assertLess(sum(len(p['indices'])//3 for p in m['primitives']),7000)
            footparts=[p for p in m['primitives'] if p['part'].startswith('Foot /')]
            for side in ('-1','1'):
                self.assertEqual(min(v for p in footparts if p['part'].split()[-1]==side for v in p['positions'][1::3]),0.)
    def test_species_identity_and_held_props(self):
        for n,m in self.models.items():
            names=[p['part'] for p in m['primitives']]
            adult=n=='alakazam'
            for prefix,count in {'Ear / pointed':2,'Eye / fitted':2 if adult else 0,'Eye / closed':0 if adult else 2,'Moustache /':2 if adult else 0,'Tail /':0 if adult else 2,'Spoon / held':2 if adult else 0,'Hand / curled':6,'Foot / splayed':6}.items():
                self.assertEqual(sum(x.startswith(prefix) for x in names),count,(n,prefix))
            self.assertFalse(any('star' in x.lower() for x in names))
            if adult:
                for side in (-1,1):
                    palm=next(p for p in m['primitives'] if p['part']=='Hand / palm '+str(side))
                    stem=next(p for p in m['primitives'] if p['part']=='Spoon / held tapered stem '+str(side))
                    # Held stem starts within the actual palm's axis bounds.
                    for axis in (0,1,2):
                        center=sum(stem['positions'][axis:36:3])/12
                        self.assertGreaterEqual(center,min(palm['positions'][axis::3])-.012)
                        self.assertLessEqual(center,max(palm['positions'][axis::3])+.012)
            head=next(p for p in m['primitives'] if p['part'].startswith('Head /'))
            # A true taper projects beyond the broad brow; no spherical snout.
            self.assertGreater(max(head['positions'][2::3])-min(head['positions'][2::3]),.35)
            for p in m['primitives']:
                if p['part'].startswith('Eye /'):self.assertGreater(p['normals'][2],0.)
if __name__=='__main__':unittest.main()
