#!/usr/bin/env python3
"""Stored geometry checks for the two lava mollusks."""
import json,unittest
from pathlib import Path
from lava_sculpt import sculpture,SPECIES,HEIGHT
from sculpt_test_support import assert_closed_models,assert_model_shading
from test_axolotl_sculpt import ray_intersections

ROOT=Path(__file__).resolve().parents[1]/'crates/crystal-voxel-view/models/battle_species'

class LavaTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):cls.models={s:json.loads((ROOT/(s+'.mesh.json')).read_bytes()) for s in SPECIES}
    def test_closed_connected_oriented_parts(self):assert_closed_models(self,self.models)
    def test_normals_match_stored_triangles(self):assert_model_shading(self,self.models)
    def test_regeneration_dimensions_and_budget(self):
        for s,m in self.models.items():
            self.assertEqual(m,sculpture(s));ps=m['primitives']
            self.assertEqual(len(ps),len({p['part'] for p in ps}))
            self.assertEqual(sum(p['part'].startswith('Body /') for p in ps),1)
            self.assertEqual(min(v for p in ps for v in p['positions'][1::3]),0.)
            self.assertAlmostEqual(max(v for p in ps for v in p['positions'][1::3]),HEIGHT,places=6)
            self.assertLess(sum(len(p['indices'])//3 for p in ps),11000)
            self.assertLess(len(json.dumps(m,separators=(',',':'))),650000)
    def test_shell_anatomy_and_open_vent(self):
        for s,m in self.models.items():
            names=[p['part'] for p in m['primitives']]
            self.assertEqual(sum(n.startswith('Shell ridge /') for n in names),2 if s=='magcargo' else 0)
            self.assertEqual(sum(n.startswith('Mantle /') for n in names),2 if s=='slugma' else 0)
        ps=self.models['magcargo']['primitives'];vent=next(p for p in ps if p['part']=='Chimney / hollow stone vent')
        floor=next(p for p in ps if p['part']=='Chimney / recessed molten floor')
        self.assertGreater(max(vent['positions'][1::3])-max(floor['positions'][1::3]),.085)
        top=dict(vent);top['positions']=[v for i in range(0,len(vent['positions']),3) for v in (vent['positions'][i],vent['positions'][i+2],vent['positions'][i+1])]
        x=(max(floor['positions'][::3])+min(floor['positions'][::3]))/2
        z=(max(floor['positions'][2::3])+min(floor['positions'][2::3]))/2
        self.assertFalse(ray_intersections(top,x,z))
    def test_fitted_skin_details_and_shell_ridges(self):
        for s,m in self.models.items():
            body=m['primitives'][0]
            upper=dict(body);upper['positions']=[v for i in range(0,len(body['positions']),3) for v in (body['positions'][i],body['positions'][i+2],body['positions'][i+1])]
            for p in m['primitives']:
                if p['part'].startswith('Mantle /'):
                    for i in range(0,len(p['positions']),3):
                        x,y,z=p['positions'][i:i+3];hits=ray_intersections(upper,x,z)
                        self.assertTrue(hits);self.assertLess(abs(y-max(hits)),.035)
                if not p['part'].startswith(('Lava /','Mouth /')):continue
                for i in range(0,len(p['positions']),3):
                    x,y,z=p['positions'][i:i+3];hits=ray_intersections(body,x,y)
                    self.assertTrue(hits,(s,p['part']));self.assertLess(abs(z-max(hits)),.012)
        ps=self.models['magcargo']['primitives'];shell=next(p for p in ps if p['part'].startswith('Shell /'))
        for side in (-1,1):
            rotated=dict(shell);rotated['positions']=[v for i in range(0,len(shell['positions']),3) for v in (shell['positions'][i+2],shell['positions'][i+1],side*shell['positions'][i])]
            ridge=next(p for p in ps if p['part']==f'Shell ridge / complete fitted spiral {side}')
            for i in range(0,len(ridge['positions']),15):
                x,y,z=ridge['positions'][i:i+3];hits=ray_intersections(rotated,z,y)
                self.assertTrue(hits);self.assertLess(abs(side*x-max(hits)),.045)
    def test_stalks_and_shell_overlap_the_body(self):
        for s,m in self.models.items():
            body=m['primitives'][0]
            contacts=[p for p in m['primitives'] if p['part'].startswith(('Eye stalk /','Shell /'))]
            for p in contacts:
                x=(max(p['positions'][::3])+min(p['positions'][::3]))/2
                lo=max(min(p['positions'][1::3]),min(body['positions'][1::3]));hi=min(max(p['positions'][1::3]),max(body['positions'][1::3]))
                overlap=False
                for k in range(1,20):
                    y=lo+(hi-lo)*k/20;a=ray_intersections(p,x,y);b=ray_intersections(body,x,y)
                    if a and b and min(max(a),max(b))-max(min(a),min(b))>.002:overlap=True;break
                self.assertTrue(overlap,(s,p['part']))

if __name__=='__main__':unittest.main()
