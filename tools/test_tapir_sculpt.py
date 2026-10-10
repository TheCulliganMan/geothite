#!/usr/bin/env python3
"""Stored tapir anatomy, continuous material seam, ruff/pendulum and grounding."""
import copy,json,unittest
from pathlib import Path
from tapir_sculpt import sculpture,HEIGHT
from sculpt_test_support import assert_closed_models,assert_model_shading
ROOT=Path(__file__).resolve().parents[1]/'crates/crystal-voxel-view/models/battle_species'

class TapirTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):cls.models={n:json.loads((ROOT/(n+'.mesh.json')).read_bytes()) for n in ('drowzee','hypno')}
    def test_closed_anatomy_with_welded_body_materials(self):
        m=copy.deepcopy(self.models);a,b=m['drowzee']['primitives'][:2]
        joined={'part':'Body / joined material regions','positions':a['positions']+b['positions'],'normals':a['normals']+b['normals'],'indices':a['indices']+[i+len(a['positions'])//3 for i in b['indices']]}
        m['drowzee']['primitives']=[joined]+m['drowzee']['primitives'][2:]
        assert_closed_models(self,m)
    def test_actual_triangle_shading(self):assert_model_shading(self,self.models)
    def test_recipe_ground_and_species_identity(self):
        for n,m in self.models.items():
            self.assertEqual(m,sculpture(n));names=[p['part'] for p in m['primitives']]
            self.assertEqual(len(names),len(set(names)))
            self.assertEqual(min(v for p in m['primitives'] for v in p['positions'][1::3]),0.)
            self.assertAlmostEqual(max(v for p in m['primitives'] for v in p['positions'][1::3]),HEIGHT,places=6)
            self.assertLess(sum(len(p['indices'])//3 for p in m['primitives']),7200)
            adult=n=='hypno'
            for prefix,count in {'Ear / rounded':0 if adult else 2,'Ear / pointed':2 if adult else 0,'Ruff /':1 if adult else 0,'Pendulum /':3 if adult else 0,'Hand / curved':6,'Foot / yellow':4 if adult else 2}.items():
                self.assertEqual(sum(x.startswith(prefix) for x in names),count,(n,prefix))
            for p in m['primitives']:
                if p['part'].startswith('Eye /'):self.assertGreater(p['normals'][2],0.)
            if adult:
                body=next(p for p in m['primitives'] if p['part'].startswith('Body /'))
                self.assertGreater(body['base_color'][0],body['base_color'][2]*5)
    def test_body_seam_is_wavy_welded_and_shares_normals(self):
        a,b=self.models['drowzee']['primitives'][:2]
        def positions(p):return {tuple(p['positions'][i:i+3]):tuple(p['normals'][i:i+3]) for i in range(0,len(p['positions']),3)}
        pa,pb=positions(a),positions(b);seam=pa.keys()&pb.keys()
        self.assertEqual(len(seam),32)
        self.assertGreater(max(x[1] for x in seam)-min(x[1] for x in seam),.045)
        for v in seam:self.assertEqual(pa[v],pb[v])
        self.assertNotEqual(a['base_color'],b['base_color'])
    def test_ruff_neck_hole_and_held_pendulum(self):
        m=self.models['hypno'];parts={p['part']:p for p in m['primitives']}
        r=parts['Ruff / closed serrated ivory collar']
        ps=[r['positions'][i:i+3] for i in range(0,len(r['positions']),3)]
        self.assertGreater(min((x*x+z*z)**.5 for x,y,z in ps),.07)
        # Closed annulus has both inner and outer radial surfaces, not a disk.
        self.assertGreater(max((x*x+z*z)**.5 for x,y,z in ps),.26)
        cord=parts['Pendulum / hanging dark cord'];hand=parts['Hand / broad palm 1']
        center=[sum(cord['positions'][i:24:3])/8 for i in range(3)]
        for axis in range(3):
            self.assertGreaterEqual(center[axis],min(hand['positions'][axis::3])-.01)
            self.assertLessEqual(center[axis],max(hand['positions'][axis::3])+.01)
        self.assertGreater(max(cord['positions'][1::3])-min(cord['positions'][1::3]),.35)
        ring=parts['Pendulum / open metal ring'];points=[ring['positions'][i:i+3] for i in range(0,len(ring['positions']),3)]
        cx=(min(ring['positions'][::3])+max(ring['positions'][::3]))/2;cy=(min(ring['positions'][1::3])+max(ring['positions'][1::3]))/2
        self.assertGreater(min(((x-cx)**2+(y-cy)**2)**.5 for x,y,z in points),.035)
if __name__=='__main__':unittest.main()
