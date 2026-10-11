#!/usr/bin/env python3
"""Check stored insect anatomy, curved wings and membrane placement."""
import json,unittest
from pathlib import Path
from wing_insect_sculpt import sculpture,SPECIES,HEIGHT
from sculpt_test_support import assert_closed_models,assert_model_shading

ROOT=Path(__file__).resolve().parents[1]/'crates/crystal-voxel-view/models/battle_species'

class InsectTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):cls.models={s:json.loads((ROOT/(s+'.mesh.json')).read_bytes()) for s in SPECIES}
    def test_closed_connected_oriented_parts(self):assert_closed_models(self,self.models)
    def test_normals_match_winding(self):assert_model_shading(self,self.models)

    def test_canonical_recipe_bounds_and_budget(self):
        for s,m in self.models.items():
            self.assertEqual(m,sculpture(s));parts=m['primitives'];names=[p['part'] for p in parts]
            self.assertEqual(len(names),len(set(names)))
            self.assertEqual(min(v for p in parts for v in p['positions'][1::3]),0.)
            self.assertAlmostEqual(max(v for p in parts for v in p['positions'][1::3]),HEIGHT,places=6)
            self.assertLess(sum(len(p['indices'])//3 for p in parts),10000)
            self.assertLess(len(json.dumps(m,separators=(',',':'))),500000)

    def test_four_curved_wings_with_membranes_visible_on_both_faces(self):
        for s,m in self.models.items():
            wings=[p for p in m['primitives'] if p['part'].startswith('Wing /')]
            self.assertEqual(len(wings),8)
            for which in ('upper','lower'):
                for side in (-1,1):
                    rim=next(p for p in wings if p['part']==f'Wing / {which} curved perimeter {side}')
                    inner=next(p for p in wings if p['part']==f'Wing / {which} inset membrane {side}')
                    points=[tuple(rim['positions'][i:i+3]) for i in range(0,len(rim['positions']),3)]
                    self.assertGreater(len({round(y,5) for x,y,z in points}),20)
                    # The membrane extends beyond both perimeter faces rather
                    # than disappearing inside the opaque wing solid.
                    self.assertGreater(max(inner['positions'][2::3]),max(rim['positions'][2::3]))
                    self.assertLess(min(inner['positions'][2::3]),min(rim['positions'][2::3]))
                    w=max(rim['positions'][::3])-min(rim['positions'][::3])
                    h=max(rim['positions'][1::3])-min(rim['positions'][1::3])
                    self.assertGreater(w,.2);self.assertGreater(h,.15)
            for p in wings:
                width=max(p['positions'][::3])-min(p['positions'][::3]);depth=max(p['positions'][2::3])-min(p['positions'][2::3])
                self.assertGreater(width/depth,7.)

    def test_species_anatomy_and_bilateral_silhouette(self):
        for s,m in self.models.items():
            names=[p['part'] for p in m['primitives']]
            self.assertEqual(sum(n.startswith('Drill /') for n in names),2 if s=='beedrill' else 0)
            self.assertEqual(sum(n.startswith('Foot /') for n in names),2 if s=='butterfree' else 0)
            self.assertEqual(sum(n.startswith('Abdomen stripe /') for n in names),2 if s=='beedrill' else 0)
            self.assertEqual(sum(n.startswith('Eye / ruby') for n in names),2)
            lo=min(v for p in m['primitives'] for v in p['positions'][::3]);hi=max(v for p in m['primitives'] for v in p['positions'][::3])
            self.assertAlmostEqual(-lo,hi,places=6)

if __name__=='__main__':unittest.main()
