#!/usr/bin/env python3
"""Stored feather surfaces, support contacts and sack anatomy checks."""
import json,unittest
from pathlib import Path
from delibird_sculpt import sculpture,HEIGHT
from sculpt_test_support import assert_closed_models,assert_model_shading
from test_axolotl_sculpt import ray_intersections
ROOT=Path(__file__).resolve().parents[1]/'crates/crystal-voxel-view/models/battle_species/delibird.mesh.json'
class DelibirdTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):cls.model=json.loads(ROOT.read_bytes());cls.ps=cls.model['primitives']
    def test_closed_connected_oriented_parts(self):assert_closed_models(self,{'delibird':self.model})
    def test_unit_normals_match_stored_faces(self):assert_model_shading(self,{'delibird':self.model})
    def test_regeneration_bounds_and_budget(self):
        self.assertEqual(self.model,sculpture());ps=self.ps
        self.assertEqual(len(ps),len({p['part'] for p in ps}))
        self.assertEqual(min(v for p in ps for v in p['positions'][1::3]),0.)
        self.assertAlmostEqual(max(v for p in ps for v in p['positions'][1::3]),HEIGHT,places=6)
        self.assertLess(sum(len(p['indices'])//3 for p in ps),18000)
    def test_collar_and_sack_folds_follow_actual_surfaces(self):
        body=self.ps[0];head=self.ps[1];sack=next(p for p in self.ps if p['part'].startswith('Tail sack / sagging'))
        rear=dict(sack);rear['positions']=[(-v if i%3!=1 else v) for i,v in enumerate(sack['positions'])]
        for p in self.ps:
            if p['part'].startswith('Collar /'):skins=(body,head);back=False
            elif p['part'].startswith('Tail sack / fitted'):skins=(rear,);back=True
            else:continue
            for i in range(0,len(p['positions']),3):
                x,y,z=p['positions'][i:i+3];x=-x if back else x;z=-z if back else z
                hits=[h for s in skins for h in ray_intersections(s,x,y)]
                self.assertTrue(hits,p['part']);self.assertLess(abs(z-max(hits)),.020,p['part'])
    def test_grounded_webs_have_actual_leg_and_body_support(self):
        body=self.ps[0]
        for side in (-1,1):
            foot=next(p for p in self.ps if p['part']=='Foot / grounded three-toed web '+str(side));leg=next(p for p in self.ps if p['part']=='Leg / short hidden gold support '+str(side))
            self.assertEqual(min(foot['positions'][1::3]),0.)
            x=(min(leg['positions'][::3])+max(leg['positions'][::3]))/2
            for a,b in ((foot,leg),(leg,body)):
                low=max(min(a['positions'][1::3]),min(b['positions'][1::3]));high=min(max(a['positions'][1::3]),max(b['positions'][1::3]));overlap=False
                for k in range(1,20):
                    y=low+(high-low)*k/20;aa=ray_intersections(a,x,y);bb=ray_intersections(b,x,y)
                    if aa and bb and min(max(aa),max(bb))-max(min(aa),min(bb))>.003:overlap=True
                self.assertTrue(overlap,(a['part'],b['part']))
    def test_asymmetric_flippers_and_paired_crests(self):
        for prefix in ('Wing /','Crest /'):self.assertEqual(sum(p['part'].startswith(prefix) for p in self.ps),2)
        wings=[p for p in self.ps if p['part'].startswith('Wing /')]
        self.assertGreater(abs(max(wings[0]['positions'][1::3])-max(wings[1]['positions'][1::3])),.20)
        body=self.ps[0]
        for side in (-1,1):
            wing=next(p for p in wings if p['part']=='Wing / broad tapered flipper '+str(side));overlap=False
            for x in (side*.12,side*.14,side*.16):
                for y in (.53,.55,.57,.59,.61,.63):
                    a=ray_intersections(wing,x,y);b=ray_intersections(body,x,y)
                    if a and b and min(max(a),max(b))-max(min(a),min(b))>.003:overlap=True
            self.assertTrue(overlap,wing['part'])
        head=self.ps[1]
        for side in (-1,1):
            fan=next(p for p in self.ps if p['part']=='Crest / three-point white fan '+str(side));overlap=False
            for y in (.88,.90,.92):
                a=ray_intersections(fan,side*.095,y);b=ray_intersections(head,side*.095,y)
                if a and b and min(max(a),max(b))-max(min(a),min(b))>.002:overlap=True
            self.assertTrue(overlap,fan['part'])
if __name__=='__main__':unittest.main()
