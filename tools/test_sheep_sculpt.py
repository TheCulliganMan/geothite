#!/usr/bin/env python3
"""Stored fleece geometry, limb contact and fitted sheep details."""
import json,unittest
from pathlib import Path
from sheep_sculpt import sculpture,SPECIES,HEIGHT
from sculpt_test_support import assert_closed_models,assert_model_shading
from test_axolotl_sculpt import ray_intersections

ROOT=Path(__file__).resolve().parents[1]/'crates/crystal-voxel-view/models/battle_species'
def vertices(p):return {tuple(p['positions'][i:i+3]) for i in range(0,len(p['positions']),3)}

class SheepTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):cls.models={s:json.loads((ROOT/(s+'.mesh.json')).read_bytes()) for s in SPECIES}
    def test_closed_connected_oriented_parts(self):assert_closed_models(self,self.models)
    def test_normals_match_stored_triangles(self):assert_model_shading(self,self.models)
    def test_recipe_heights_and_budget(self):
        for s,m in self.models.items():
            self.assertEqual(m,sculpture(s));ps=m['primitives']
            self.assertEqual(len(ps),len({p['part'] for p in ps}))
            self.assertEqual(min(v for p in ps for v in p['positions'][1::3]),0.)
            self.assertAlmostEqual(max(v for p in ps for v in p['positions'][1::3]),HEIGHT[s],places=6)
            self.assertLess(sum(len(p['indices'])//3 for p in ps),19000)
            self.assertLess(len(json.dumps(m,separators=(',',':'))),900000)
    def test_distinct_fleece_limbs_and_curled_crowns(self):
        for s,m in self.models.items():
            names=[p['part'] for p in m['primitives']]
            self.assertEqual(sum(n.startswith('Fleece /') for n in names),1)
            self.assertEqual(sum(n.startswith('Crown /') for n in names),2 if s=='flaaffy' else 1)
            self.assertEqual(sum(n.startswith('Crown groove /') for n in names),2 if s=='flaaffy' else 1)
            self.assertEqual(sum(n.startswith('Leg /') for n in names),2 if s=='flaaffy' else 4)
            self.assertEqual(sum(n.startswith('Arm /') for n in names),2 if s=='flaaffy' else 0)
            if s=='mareep':
                for p in m['primitives']:
                    if p['part'].startswith('Leg /'):self.assertEqual(min(p['positions'][1::3]),0.)
            else:
                for p in m['primitives']:
                    if p['part'].startswith('Foot /'):self.assertEqual(min(p['positions'][1::3]),0.)
    def test_fitted_curls_nostrils_and_toe_caps(self):
        for s,m in self.models.items():
            ps=m['primitives'];head=next(p for p in ps if p['part'].startswith('Head /'))
            for p in ps:
                if p['part'].startswith('Nostril /'):skin=head
                elif p['part'].startswith('Crown groove /'):
                    suffix=p['part'].split()[-1]
                    skin=next(q for q in ps if q['part'].startswith('Crown /') and (s=='mareep' or q['part'].endswith(" "+suffix)))
                elif p['part'].startswith('Toe /'):
                    suffix=p['part'].split()[-1];skin=next(q for q in ps if q['part'].startswith('Foot /') and q['part'].endswith(" "+suffix))
                else:continue
                for x,y,z in vertices(p):
                    hits=ray_intersections(skin,x,y);self.assertTrue(hits,(s,p['part']))
                    self.assertLess(abs(z-max(hits)),.013,(s,p['part']))
    def test_shared_tail_and_ear_rings_and_real_support_contacts(self):
        for s,m in self.models.items():
            ps={p['part']:p for p in m['primitives']}
            for prefix,count in [('Tail / curved alternating segment',4)]+[(f'Ear / {side}',3) for side in (-1,1)]:
                for k in range(count-1):self.assertEqual(len(vertices(ps[f'{prefix} band {k}'])&vertices(ps[f'{prefix} band {k+1}'])),14)
            body=next(p for n,p in ps.items() if n.startswith('Body /' if s=='flaaffy' else 'Fleece /'))
            for p in [p for n,p in ps.items() if n.startswith('Leg /')]:
                x=(max(p['positions'][::3])+min(p['positions'][::3]))/2
                lo=max(min(p['positions'][1::3]),min(body['positions'][1::3]));hi=min(max(p['positions'][1::3]),max(body['positions'][1::3]));overlap=False
                for k in range(1,15):
                    y=lo+(hi-lo)*k/15;a=ray_intersections(p,x,y);b=ray_intersections(body,x,y)
                    if a and b and min(max(a),max(b))-max(min(a),min(b))>.005:overlap=True;break
                self.assertTrue(overlap,(s,p['part']))

if __name__=='__main__':unittest.main()
