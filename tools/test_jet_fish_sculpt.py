#!/usr/bin/env python3
"""Actual arm support, hollow siphon and streamlined fish regressions."""
import json,unittest
from pathlib import Path
from jet_fish_sculpt import sculpture,SPECIES,HEIGHT,side_rotate
from sculpt_test_support import assert_closed_models,assert_model_shading
from test_axolotl_sculpt import ray_intersections
ROOT=Path(__file__).resolve().parents[1]/'crates/crystal-voxel-view/models/battle_species'
class JetFishTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):cls.models={s:json.loads((ROOT/(s+'.mesh.json')).read_bytes()) for s in SPECIES}
    def test_closed_connected_oriented_parts(self):assert_closed_models(self,self.models)
    def test_shading_matches_stored_faces(self):assert_model_shading(self,self.models)
    def test_regeneration_height_and_budget(self):
        for s,m in self.models.items():
            self.assertEqual(m,sculpture(s));ps=m['primitives']
            self.assertEqual(len(ps),len({p['part'] for p in ps}))
            self.assertEqual(min(v for p in ps for v in p['positions'][1::3]),0)
            self.assertAlmostEqual(max(v for p in ps for v in p['positions'][1::3]),HEIGHT,places=6)
            self.assertLess(sum(len(p['indices'])//3 for p in ps),22000)
    def test_remoraid_tail_is_behind_elongated_body(self):
        ps=self.models['remoraid']['primitives'];body=ps[0]
        width=max(body['positions'][::3])-min(body['positions'][::3]);length=max(body['positions'][2::3])-min(body['positions'][2::3])
        self.assertGreater(length/width,1.7)
        tails=[p for p in ps if p['part'].startswith('Tail /')];self.assertEqual(len(tails),2)
        for p in tails:self.assertLess(max(p['positions'][2::3]),-.25)
        self.assertEqual(sum(p['part'].startswith('Stripe /') for p in ps),4)
    def test_eight_curled_arms_contact_authoritative_mantle(self):
        ps=self.models['octillery']['primitives'];body=ps[0];arms=[p for p in ps if p['part'].startswith('Arm /')]
        self.assertEqual(len(arms),8)
        for arm in arms:
            overlap=False
            for k in range(1,20):
                # Probe the arm root, whose first rings live in the mantle.
                x=arm['positions'][0];y=.17+k*.004
                aa=ray_intersections(arm,x,y);bb=ray_intersections(body,x,y)
                if aa and bb and min(max(aa),max(bb))-max(min(aa),min(bb))>.005:overlap=True;break
            self.assertTrue(overlap,arm['part'])
            self.assertGreater(max(arm['positions'][1::3])-min(arm['positions'][1::3]),.20)
        self.assertEqual(sum(p['part'].startswith('Sucker / raised') for p in ps),16)
        self.assertEqual(sum(p['part'].startswith('Sucker / recessed') for p in ps),16)
    def test_stripes_smiles_and_lids_follow_actual_skin(self):
        for s,m in self.models.items():
            ps=m['primitives'];body=ps[0]
            for p in ps:
                name=p['part']
                if name.startswith('Stripe /'):
                    side=int(name.split()[-2]);skin=side_rotate(body,-side);p=side_rotate(p,-side)
                elif name.startswith('Mouth /'):skin=body
                elif name.startswith('Eye / fitted'):
                    side=name.split()[-1];skin=next(q for q in ps if q['part']=='Eye / sleepy ivory oval '+side)
                else:continue
                for i in range(0,len(p['positions']),3):
                    x,y,z=p['positions'][i:i+3];hits=ray_intersections(skin,x,y)
                    self.assertTrue(hits,(s,name));self.assertLess(abs(z-max(hits)),.014,(s,name))
    def test_suckers_are_supported_by_stored_arm_surfaces(self):
        ps=self.models['octillery']['primitives']
        for i in range(8):
            arm=dict(next(p for p in ps if p['part']=='Arm / smooth curling tentacle '+str(i)))
            arm['positions']=[v for k in range(0,len(arm['positions']),3) for v in (arm['positions'][k],arm['positions'][k+2],arm['positions'][k+1])]
            for j in range(2):
                rim=next(p for p in ps if p['part']==f'Sucker / raised yellow rim {i} {j}')
                well=next(p for p in ps if p['part']==f'Sucker / recessed gold well {i} {j}')
                x=(min(rim['positions'][::3])+max(rim['positions'][::3]))/2
                z=(min(rim['positions'][2::3])+max(rim['positions'][2::3]))/2
                hits=ray_intersections(arm,x,z);self.assertTrue(hits)
                top=max(hits)
                self.assertLess(min(rim['positions'][1::3]),top)
                self.assertLess(max(rim['positions'][1::3])-top,.028)
                self.assertLess(min(well['positions'][1::3]),top)
                self.assertGreater(max(well['positions'][1::3]),top)
    def test_siphon_has_open_lip_and_depth(self):
        ps=self.models['octillery']['primitives'];tube=next(p for p in ps if p['part'].startswith('Siphon / real'))
        vertices=[tube['positions'][i:i+3] for i in range(0,len(tube['positions']),3)]
        depth=max(q[2] for q in vertices)-min(q[2] for q in vertices);self.assertGreater(depth,.18)
        front=max(q[2] for q in vertices);rim=[q for q in vertices if abs(q[2]-front)<1e-6]
        cx=sum(q[0] for q in rim)/len(rim);cy=sum(q[1] for q in rim)/len(rim)
        # The center ray stops at a deeper part of the wall, never at the lip.
        hits=ray_intersections(tube,cx,cy)
        self.assertTrue(hits);self.assertLess(max(hits),front-.05)
if __name__=='__main__':unittest.main()
