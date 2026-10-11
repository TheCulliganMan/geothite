#!/usr/bin/env python3
"""Stored canine proportions, bone armor and real joint contacts."""
import json,unittest
from pathlib import Path
from hellhound_sculpt import sculpture,SPECIES,HEIGHT
from sculpt_test_support import assert_closed_models,assert_model_shading
from test_axolotl_sculpt import ray_intersections
ROOT=Path(__file__).resolve().parents[1]/'crates/crystal-voxel-view/models/battle_species'
class HellhoundTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):cls.models={s:json.loads((ROOT/(s+'.mesh.json')).read_bytes()) for s in SPECIES}
    def test_closed_connected_oriented_parts(self):assert_closed_models(self,self.models)
    def test_normals_match_stored_triangles(self):assert_model_shading(self,self.models)
    def test_regeneration_height_budget_and_distinct_anatomy(self):
        for s,m in self.models.items():
            self.assertEqual(m,sculpture(s));ps=m['primitives'];big=s=='houndoom'
            self.assertEqual(len(ps),len({p['part'] for p in ps}))
            self.assertEqual(min(v for p in ps for v in p['positions'][1::3]),0.)
            self.assertAlmostEqual(max(v for p in ps for v in p['positions'][1::3]),HEIGHT,places=6)
            self.assertLess(sum(len(p['indices'])//3 for p in ps),23000)
            for prefix,count in (('Leg /',4),('Paw / grounded',4),('Cuff /',8 if big else 4),('Armor / broad',3 if big else 2),('Horn /',2 if big else 0),('Ear /',0 if big else 2),('Claw /',12 if big else 0),('Tail / pointed adult',1 if big else 0)):
                self.assertEqual(sum(p['part'].startswith(prefix) for p in ps),count,(s,prefix))
    def assert_overlap(self,a,b,xs,ys):
        overlap=False
        for x in xs:
            for y in ys:
                aa=ray_intersections(a,x,y);bb=ray_intersections(b,x,y)
                if aa and bb and min(max(aa),max(bb))-max(min(aa),min(bb))>.001:overlap=True
        self.assertTrue(overlap,(a['part'],b['part']))
    def test_real_paw_leg_body_support(self):
        for s,m in self.models.items():
            ps=m['primitives'];body=ps[0]
            for side in (-1,1):
                for row in range(2):
                    foot=next(p for p in ps if p['part']==f'Paw / grounded rounded pad {side} {row}');leg=next(p for p in ps if p['part']==f'Leg / sculpted canine support {side} {row}')
                    self.assertEqual(min(foot['positions'][1::3]),0.)
                    x=(max(foot['positions'][::3])+min(foot['positions'][::3]))/2
                    self.assert_overlap(foot,leg,(x-.015,x,x+.015),(.045,.065,.085,.10))
                    self.assert_overlap(leg,body,[side*x for x in (.055,.08,.11,.14)],(.40,.43,.46,.49,.52,.55))
    def test_fitted_features_use_real_skin(self):
        for s,m in self.models.items():
            ps=m['primitives'];roof=dict(ps[0]);roof['positions']=[v for i in range(0,len(roof['positions']),3) for v in (roof['positions'][i],roof['positions'][i+2],roof['positions'][i+1])]
            for p in ps:
                name=p['part'];side=name.split()[-1];top=False
                if name.startswith('Eye / fitted dark'):skin=ps[2]
                elif name.startswith('Eye / fitted ivory'):skin=next(q for q in ps if q['part']=='Eye / fitted dark angled socket '+side)
                elif name.startswith('Eye / fitted red'):skin=next(q for q in ps if q['part']=='Eye / fitted ivory canine white '+side)
                elif name.startswith('Eye / thin'):skin=next(q for q in ps if q['part']=='Eye / fitted red narrow iris '+side)
                elif name.startswith(('Armor / fitted forehead','Muzzle / fitted orange')):skin=ps[2]
                elif name.startswith('Armor / dark'):skin=next(q for q in ps if q['part'].startswith('Armor / fitted forehead'))
                elif name.startswith('Armor / broad'):skin=roof;top=True
                elif name.startswith('Chest /'):skin=ps[1]
                elif name.startswith('Charm / fitted'):skin=next(q for q in ps if q['part'].startswith('Charm / flat'))
                elif name.startswith('Paw crease /'):skin=next(q for q in ps if q['part']=='Paw / grounded rounded pad '+' '.join(name.split()[-3:-1]))
                else:continue
                for i in range(0,len(p['positions']),3):
                    x,y,z=p['positions'][i:i+3];hits=ray_intersections(skin,x,z if top else y)
                    self.assertTrue(hits,(s,name));self.assertLess(abs((y if top else z)-max(hits)),.012,(s,name))
    def test_cuffs_claws_and_spade_have_real_support(self):
        for s,m in self.models.items():
            ps=m['primitives']
            for p in ps:
                name=p['part']
                if name.startswith('Cuff /'):
                    side,row,j=name.split()[-3:];support=next(q for q in ps if q['part']==f'Leg / sculpted canine support {side} {row}')
                elif name.startswith('Claw /'):
                    side,row,j=name.split()[-3:];support=next(q for q in ps if q['part']==f'Paw / grounded rounded pad {side} {row}')
                elif name.startswith('Tail / pointed'):support=next(q for q in ps if q['part'].startswith('Tail / long'))
                else:continue
                cx=(min(p['positions'][::3])+max(p['positions'][::3]))/2
                low=max(min(p['positions'][1::3]),min(support['positions'][1::3]));high=min(max(p['positions'][1::3]),max(support['positions'][1::3]))
                self.assert_overlap(p,support,[cx-.007,cx,cx+.007],[low+(high-low)*k/12 for k in range(1,12)])
    def test_muzzle_nose_and_adult_bone_collar_contacts(self):
        for s,m in self.models.items():
            ps=m['primitives'];muzzle=next(p for p in ps if p['part'].startswith('Muzzle / projecting'));nose=next(p for p in ps if p['part'].startswith('Nose /'))
            pairs=[(muzzle,ps[2]),(nose,muzzle)]
            if s=='houndoom':
                collar=next(p for p in ps if p['part'].startswith('Collar /'));charm=next(p for p in ps if p['part'].startswith('Charm / flat'))
                pairs.extend(((collar,ps[1]),(charm,collar)))
            for a,b in pairs:
                xlo=max(min(a['positions'][::3]),min(b['positions'][::3]));xhi=min(max(a['positions'][::3]),max(b['positions'][::3]))
                ylo=max(min(a['positions'][1::3]),min(b['positions'][1::3]));yhi=min(max(a['positions'][1::3]),max(b['positions'][1::3]))
                self.assert_overlap(a,b,[xlo+(xhi-xlo)*k/6 for k in range(1,6)],[ylo+(yhi-ylo)*k/10 for k in range(1,10)])
    def test_head_neck_and_appendage_roots_connect(self):
        for s,m in self.models.items():
            ps=m['primitives'];big=s=='houndoom';body,neck,head=ps[:3]
            self.assert_overlap(neck,body,(-.025,0,.025),(.45,.50,.55,.60))
            self.assert_overlap(head,neck,(-.025,0,.025),(.80,.83,.86,.89) if big else (.75,.78,.81,.84))
            for p in ps:
                if p['part'].startswith('Horn /'):
                    side=int(p['part'].split()[-1]);self.assert_overlap(p,head,[side*x for x in (.08,.10,.12)],(.96,1.00,1.04))
                elif p['part'].startswith('Ear /'):
                    side=int(p['part'].split()[-1]);self.assert_overlap(p,head,[side*x for x in (.09,.11,.13)],(.93,.95,.97))
                elif p['part'].startswith(('Tail / long','Tail / short')):self.assert_overlap(p,body,(-.01,0,.01),(.44,.47,.50,.53,.56))
if __name__=='__main__':unittest.main()
