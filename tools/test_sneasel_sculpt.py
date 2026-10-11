#!/usr/bin/env python3
"""Stored claw-weasel skin, rooted anatomy and silhouette checks."""
import json,unittest
from pathlib import Path
from sneasel_sculpt import sculpture,HEIGHT
from sculpt_test_support import assert_closed_models,assert_model_shading
from test_axolotl_sculpt import ray_intersections
ROOT=Path(__file__).resolve().parents[1]/'crates/crystal-voxel-view/models/battle_species/sneasel.mesh.json'
class SneaselTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):cls.model=json.loads(ROOT.read_bytes());cls.ps=cls.model['primitives']
    def test_closed_connected_oriented_parts(self):assert_closed_models(self,{'sneasel':self.model})
    def test_normals_match_stored_triangles(self):assert_model_shading(self,{'sneasel':self.model})
    def test_regeneration_bounds_counts_and_budget(self):
        self.assertEqual(self.model,sculpture());ps=self.ps
        self.assertEqual(len(ps),len({p['part'] for p in ps}))
        self.assertEqual(min(v for p in ps for v in p['positions'][1::3]),0.)
        self.assertAlmostEqual(max(v for p in ps for v in p['positions'][1::3]),HEIGHT,places=6)
        self.assertLess(sum(len(p['indices'])//3 for p in ps),13500)
        for prefix,count in (('Ear /',1),('Plume /',1),('Tail / swept',3),('Talon / hooked',4),('Talon / paired',4),('Gem /',2)):
            self.assertEqual(sum(p['part'].startswith(prefix) for p in ps),count)
    def assert_contact(self,a,b,xs,ys):
        overlap=False
        for x in xs:
            for y in ys:
                aa=ray_intersections(a,x,y);bb=ray_intersections(b,x,y)
                if aa and bb and min(max(aa),max(bb))-max(min(aa),min(bb))>.001:overlap=True
        self.assertTrue(overlap,(a['part'],b['part']))
    def test_rooted_head_ears_and_forelimbs(self):
        body=self.ps[0];head=self.ps[1]
        self.assert_contact(head,body,(-.015,0,.015),(.535,.55,.565))
        for side in (-1,1):
            ear=next(p for p in self.ps if p['part'].startswith('Ear /' if side<0 else 'Plume /'))
            self.assert_contact(ear,head,[side*x for x in (.09,.105,.12)],(.77,.785,.80))
            arm=next(p for p in self.ps if p['part']=='Arm / curved reaching forelimb '+str(side))
            self.assert_contact(arm,body,[side*x for x in (.07,.09,.11)],(.48,.50,.52,.54))
            hand=next(p for p in self.ps if p['part']=='Hand / broad claw-bearing palm '+str(side))
            cx=(min(hand['positions'][::3])+max(hand['positions'][::3]))/2;cy=(min(hand['positions'][1::3])+max(hand['positions'][1::3]))/2
            self.assert_contact(hand,arm,[cx-.015,cx,cx+.015],[cy-.015,cy,cy+.015])
            for p in self.ps:
                if p['part'].startswith('Talon / hooked') and p['part'].split()[-2]==str(side):self.assert_contact(p,hand,[cx-.025,cx-.010,cx+.010,cx+.025],[cy-.01,cy,cy+.01])
    def test_three_tail_feathers_contact_rooted_fan_stem(self):
        stem=next(p for p in self.ps if p['part'].startswith('Tail / short'))
        self.assert_contact(stem,self.ps[0],(-.01,0,.01),(.23,.24,.25,.26))
        for p in self.ps:
            if p['part'].startswith('Tail / swept'):
                self.assert_contact(p,stem,(-.015,-.007,0,.007,.015),(.23,.24,.245,.25,.255,.26))
    def test_feet_legs_and_curved_toe_contacts(self):
        for side in (-1,1):
            leg=next(p for p in self.ps if p['part']=='Leg / curved outward hock '+str(side));foot=next(p for p in self.ps if p['part']=='Foot / broad grounded teal pad '+str(side))
            self.assertEqual(min(foot['positions'][1::3]),0.)
            self.assert_contact(leg,self.ps[0],[side*x for x in (.07,.09,.11)],(.23,.25,.27))
            self.assert_contact(foot,leg,[side*x for x in (.16,.18,.20)],(.035,.05,.07))
            for p in self.ps:
                if p['part'].startswith('Talon / paired') and p['part'].split()[-2]==str(side):self.assert_contact(p,foot,[side*x for x in (.16,.17,.18,.19,.20)],(.025,.04,.05))
    def test_fitted_facial_features_and_gems_follow_stored_skin(self):
        for p in self.ps:
            name=p['part'];side=name.split()[-1]
            if name.startswith('Eye / fitted dark'):skin=self.ps[1]
            elif name.startswith('Eye / fitted ivory'):skin=next(q for q in self.ps if q['part']=='Eye / fitted dark slanted socket '+side)
            elif name.startswith('Eye / narrow'):skin=next(q for q in self.ps if q['part']=='Eye / fitted ivory angular white '+side)
            elif name.startswith('Eye / vertical'):skin=next(q for q in self.ps if q['part']=='Eye / narrow fitted red iris '+side)
            elif name.startswith('Gem /'):skin=self.ps[1] if name.endswith('forehead') else self.ps[0]
            elif name.startswith('Mouth / fitted open'):skin=self.ps[1]
            elif name.startswith(('Mouth / fitted pink','Mouth / small')):skin=next(q for q in self.ps if q['part'].startswith('Mouth / fitted open'))
            else:continue
            for i in range(0,len(p['positions']),3):
                x,y,z=p['positions'][i:i+3];hits=ray_intersections(skin,x,y)
                self.assertTrue(hits,name);self.assertLess(abs(z-max(hits)),.008,name)
if __name__=='__main__':unittest.main()
