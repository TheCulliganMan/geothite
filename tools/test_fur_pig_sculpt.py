#!/usr/bin/env python3
"""Stored fur contours, fitted pigment and real hoof/support contacts."""
import json,unittest
from pathlib import Path
from fur_pig_sculpt import sculpture,SPECIES,HEIGHT
from sculpt_test_support import assert_closed_models,assert_model_shading
from test_axolotl_sculpt import ray_intersections

ROOT=Path(__file__).resolve().parents[1]/'crates/crystal-voxel-view/models/battle_species'
class FurPigTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):cls.models={s:json.loads((ROOT/(s+'.mesh.json')).read_bytes()) for s in SPECIES}
    def test_closed_connected_oriented_parts(self):assert_closed_models(self,self.models)
    def test_normals_match_stored_triangles(self):assert_model_shading(self,self.models)
    def test_regeneration_heights_and_budget(self):
        for s,m in self.models.items():
            self.assertEqual(m,sculpture(s));ps=m['primitives']
            self.assertEqual(len(ps),len({p['part'] for p in ps}))
            self.assertEqual(min(v for p in ps for v in p['positions'][1::3]),0.)
            self.assertAlmostEqual(max(v for p in ps for v in p['positions'][1::3]),HEIGHT,places=6)
            self.assertLess(sum(len(p['indices'])//3 for p in ps),11000)
            self.assertLess(len(json.dumps(m,separators=(',',':'))),700000)
    def test_distinct_coats_and_forward_downward_tusks(self):
        ratios={}
        for s,m in self.models.items():
            ps=m['primitives'];names=[p['part'] for p in ps];body=ps[0]
            ratios[s]=(max(body['positions'][1::3])-min(body['positions'][1::3]))/(max(body['positions'][::3])-min(body['positions'][::3]))
            self.assertEqual(sum(n.startswith('Stripe /') for n in names),4 if s=='swinub' else 0)
            self.assertEqual(sum(n.startswith('Eye /') for n in names),2 if s=='swinub' else 0)
            self.assertEqual(sum(n.startswith('Ear /') for n in names),2 if s=='piloswine' else 0)
            self.assertEqual(sum(n.startswith('Tusk /') for n in names),2 if s=='piloswine' else 0)
            for p in ps:
                if not p['part'].startswith('Tusk /'):continue
                points=[p['positions'][i:i+3] for i in range(0,len(p['positions']),3)]
                tip=max(points,key=lambda q:q[2]);root=min(points,key=lambda q:q[2])
                self.assertGreater(tip[2]-root[2],.20);self.assertLess(tip[1],root[1]-.08)
        self.assertLess(ratios['swinub'],.65);self.assertGreater(ratios['piloswine'],1.25)
    def test_stripes_eyes_nostrils_and_fur_creases_follow_stored_skin(self):
        for s,m in self.models.items():
            body=m['primitives'][0];snout=next(p for p in m['primitives'] if p['part'].startswith('Snout /'))
            top=dict(body);top['positions']=[v for i in range(0,len(body['positions']),3) for v in (body['positions'][i],body['positions'][i+2],body['positions'][i+1])]
            for p in m['primitives']:
                if p['part'].startswith('Stripe /'):skin=top;upper=True
                elif p['part'].startswith('Nostril /'):skin=snout;upper=False
                elif p['part'].startswith(('Eye /','Fur crease /')):skin=body;upper=False
                else:continue
                for i in range(0,len(p['positions']),3):
                    x,y,z=p['positions'][i:i+3];hits=ray_intersections(skin,x,z if upper else y)
                    self.assertTrue(hits,(s,p['part']));self.assertLess(abs((y if upper else z)-max(hits)),.022,(s,p['part']))
    def test_grounded_hooves_and_actual_body_leg_hoof_overlap(self):
        for s,m in self.models.items():
            body=m['primitives'][0]
            for side in (-1,1):
                for row in range(2):
                    hoof=next(p for p in m['primitives'] if p['part']==f'Hoof / grounded oval pad {side} {row}')
                    leg=next(p for p in m['primitives'] if p['part']==f'Leg / hidden supporting shank {side} {row}')
                    self.assertEqual(min(hoof['positions'][1::3]),0.)
                    x=(min(hoof['positions'][::3])+max(hoof['positions'][::3]))/2
                    for a,b in ((hoof,leg),(leg,body)):
                        lo=max(min(a['positions'][1::3]),min(b['positions'][1::3]));hi=min(max(a['positions'][1::3]),max(b['positions'][1::3]));overlap=False
                        for k in range(1,15):
                            y=lo+(hi-lo)*k/15;aa=ray_intersections(a,x,y);bb=ray_intersections(b,x,y)
                            if aa and bb and min(max(aa),max(bb))-max(min(aa),min(bb))>.005:overlap=True;break
                        self.assertTrue(overlap,(s,side,row,a['part'],b['part']))

if __name__=='__main__':unittest.main()
