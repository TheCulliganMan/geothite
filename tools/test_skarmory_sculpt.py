#!/usr/bin/env python3
"""Actual steel bird anatomy, armor contacts and fitted eye regressions."""
import json,unittest
from pathlib import Path
from skarmory_sculpt import sculpture,HEIGHT
from sculpt_test_support import assert_closed_models,assert_model_shading,ray_intersections
from skarmory_glb import read_skarmory,geometry_digest
ROOT=Path(__file__).resolve().parents[1]/'crates/crystal-voxel-view/models/battle_species/skarmory.glb'
class SkarmoryTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):cls.model=read_skarmory(ROOT);cls.ps=cls.model['primitives']
    def test_closed_connected_oriented_parts(self):assert_closed_models(self,{'skarmory':self.model})
    def test_normals_match_stored_triangles(self):assert_model_shading(self,{'skarmory':self.model})
    def test_regeneration_bounds_budget_and_blade_counts(self):
        self.assertEqual(geometry_digest(self.model),geometry_digest(sculpture()));ps=self.ps
        self.assertEqual(len(ps),len({p['part'] for p in ps}))
        self.assertEqual(min(v for p in ps for v in p['positions'][1::3]),0.)
        self.assertAlmostEqual(max(v for p in ps for v in p['positions'][1::3]),HEIGHT,places=6)
        self.assertLess(sum(len(p['indices'])//3 for p in ps),8500)
        self.assertEqual(sum(p['part'].startswith('Wing / layered red') for p in ps),8)
        self.assertEqual(sum(p['part'].startswith('Wing / swept steel') for p in ps),2)
        self.assertEqual(sum(p['part'].startswith('Neck /') for p in ps),4)
    def assert_overlap(self,a,b,xs,ys):
        overlap=False
        for x in xs:
            for y in ys:
                aa=ray_intersections(a,x,y);bb=ray_intersections(b,x,y)
                if aa and bb and min(max(aa),max(bb))-max(min(aa),min(bb))>.001:overlap=True
        self.assertTrue(overlap,(a['part'],b['part']))
    def test_wing_roots_head_and_tail_connect_to_body(self):
        body=self.ps[0];head=self.ps[1]
        self.assert_overlap(head,body,(-.025,0,.025),(.87,.88,.89,.90))
        tail=next(p for p in self.ps if p['part'].startswith('Tail /'))
        self.assert_overlap(tail,body,(-.005,0,.005),(.275,.28,.285,.29,.295,.30,.305))
        for p in self.ps:
            if p['part'].startswith('Wing /'):
                side=int(p['part'].split()[-1]) if p['part'].startswith('Wing / swept') else int(p['part'].split()[-2])
                self.assert_overlap(p,body,[side*x for x in (.12,.14,.16)],(.52,.54,.56,.58,.60))
    def test_crest_and_rear_plumes_connect_to_actual_skull(self):
        head=self.ps[1]
        crest=next(p for p in self.ps if p['part'].startswith('Crest /'))
        self.assert_overlap(crest,head,(-.005,0,.005),(.94,.96,.98,1.00))
        for p in self.ps:
            if not p['part'].startswith('Nape /'):continue
            side=int(p['part'].split()[-2])
            self.assert_overlap(p,head,[side*x for x in (.025,.03,.035,.04)],(.89,.91,.93,.95,.97))
    def test_fitted_eyes_and_neck_seams_use_stored_skin(self):
        for p in self.ps:
            name=p['part']
            if name.startswith('Neck /'):skin=self.ps[0]
            elif name.startswith('Eye / fitted dark'):skin=self.ps[1]
            elif name.startswith('Eye / fitted yellow'):skin=next(q for q in self.ps if q['part']=='Eye / fitted dark angular socket '+name.split()[-1])
            elif name.startswith('Eye / dark narrow'):skin=next(q for q in self.ps if q['part']=='Eye / fitted yellow slit '+name.split()[-1])
            else:continue
            for i in range(0,len(p['positions']),3):
                x,y,z=p['positions'][i:i+3];hits=ray_intersections(skin,x,y)
                self.assertTrue(hits,name);self.assertLess(abs(z-max(hits)),.008,name)
    def test_grounded_talons_and_joint_contacts(self):
        body=self.ps[0]
        for side in (-1,1):
            thigh=next(p for p in self.ps if p['part']=='Thigh / articulated silver haunch '+str(side));leg=next(p for p in self.ps if p['part']=='Leg / bent dark steel hock '+str(side))
            claws=[p for p in self.ps if p['part'].startswith('Claw /') and (p['part'].endswith(' '+str(side)) if p['part'].startswith('Claw / hooked rear') else p['part'].split()[-2]==str(side))]
            self.assertEqual(len(claws),4)
            self.assertEqual(min(v for p in claws for v in p['positions'][1::3]),0.)
            self.assert_overlap(thigh,body,[side*x for x in (.08,.10,.12)],(.23,.25,.27,.29))
            self.assert_overlap(leg,thigh,[side*x for x in (.10,.11,.12)],(.19,.20,.21,.22))
            for claw in claws:self.assert_overlap(claw,leg,[side*x for x in (.12,.13,.14)],(.05,.06,.065,.07))
if __name__=='__main__':unittest.main()
