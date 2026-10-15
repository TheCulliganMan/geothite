#!/usr/bin/env python3
"""Verify stored amphibian contours, fitted faces and real limb contacts."""
import json,unittest
from pathlib import Path
from axolotl_sculpt import sculpture,SPECIES,HEIGHT
from battle_model_assets import read_species_model,species_paths
from sculpt_test_support import assert_closed_models,assert_model_shading
from wooper_glb import geometry_digest

ROOT=Path(__file__).resolve().parents[1]/'crates/crystal-voxel-view/models/battle_species'


def ray_intersections(p,x,y):
    # Independent intersections with actual stored triangles, not recipe math.
    ps=[p['positions'][i:i+3] for i in range(0,len(p['positions']),3)];hits=[]
    for i in range(0,len(p['indices']),3):
        a,b,c=[ps[j] for j in p['indices'][i:i+3]]
        den=(b[1]-c[1])*(a[0]-c[0])+(c[0]-b[0])*(a[1]-c[1])
        if abs(den)<1e-12:continue
        u=((b[1]-c[1])*(x-c[0])+(c[0]-b[0])*(y-c[1]))/den
        v=((c[1]-a[1])*(x-c[0])+(a[0]-c[0])*(y-c[1]))/den
        if min(u,v,1-u-v)>=-1e-7:hits.append(u*a[2]+v*b[2]+(1-u-v)*c[2])
    return hits


class AxolotlTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):cls.models={s:read_species_model(species_paths(ROOT)[s]) for s in SPECIES}
    def test_closed_connected_oriented_parts(self):assert_closed_models(self,self.models)
    def test_normals_agree_with_real_winding(self):assert_model_shading(self,self.models)

    def test_canonical_recipe_dimensions_and_budget(self):
        for s,m in self.models.items():
            recipe=sculpture(s)
            self.assertEqual(geometry_digest(m),geometry_digest(recipe));parts=m['primitives'];names=[p['part'] for p in parts]
            self.assertEqual(names,[p['part'] for p in recipe['primitives']])
            self.assertEqual(len(names),len(set(names)))
            self.assertEqual(sum(n.startswith('Body /') for n in names),1)
            self.assertEqual(min(v for p in parts for v in p['positions'][1::3]),0.)
            self.assertAlmostEqual(max(v for p in parts for v in p['positions'][1::3]),HEIGHT[s],places=6)
            self.assertLess(sum(len(p['indices'])//3 for p in parts),7000)
            self.assertLess(len(json.dumps(recipe,separators=(',',':'))),400000)

    def test_facial_dots_and_belly_bars_follow_actual_skin(self):
        for s,m in self.models.items():
            body=m['primitives'][0]
            for p in m['primitives'][1:]:
                if not p['part'].startswith(('Eye /','Nostril /','Belly marking /')):continue
                for i in range(0,len(p['positions']),3):
                    x,y,z=p['positions'][i:i+3];hits=ray_intersections(body,x,y)
                    self.assertTrue(hits,(s,p['part']))
                    self.assertLess(abs(z-max(hits)),.007,(s,p['part']))

    def test_supporting_legs_overlap_body_and_grounded_soles(self):
        for s,m in self.models.items():
            body=m['primitives'][0]
            for side in (-1,1):
                foot=next(p for p in m['primitives'] if p['part']==f'Foot / grounded three-lobed pad {side}')
                leg=next(p for p in m['primitives'] if p['part']==f'Leg / rounded supporting haunch {side}')
                self.assertEqual(min(foot['positions'][1::3]),0.)
                x=(min(foot['positions'][::3])+max(foot['positions'][::3]))/2
                for a,b in ((foot,leg),(leg,body)):
                    low=max(min(a['positions'][1::3]),min(b['positions'][1::3]));high=min(max(a['positions'][1::3]),max(b['positions'][1::3]))
                    self.assertGreater(high-low,.003,(s,side))
                    overlaps=False
                    for k in range(1,10):
                        y=low+(high-low)*k/10;aa=ray_intersections(a,x,y);bb=ray_intersections(b,x,y)
                        if aa and bb and min(max(aa),max(bb))-max(min(aa),min(bb))>.003:overlaps=True;break
                    self.assertTrue(overlaps,(s,side,a['part'],b['part']))

    def test_distinct_body_proportions_and_flattened_paddle_tail(self):
        ratios={}
        for s,m in self.models.items():
            names=[p['part'] for p in m['primitives']]
            self.assertEqual(sum(n.startswith('Gill /') for n in names),2 if s=='wooper' else 0)
            self.assertEqual(sum(n.startswith('Gill branch /') for n in names),8 if s=='wooper' else 0)
            self.assertEqual(sum(n.startswith('Belly marking /') for n in names),3 if s=='wooper' else 0)
            self.assertEqual(sum(n.startswith('Dorsal fin /') for n in names),1 if s=='quagsire' else 0)
            points=[m['primitives'][0]['positions'][i:i+3] for i in range(0,len(m['primitives'][0]['positions']),3)]
            def width(y):
                row=min({p[1] for p in points},key=lambda yy:abs(yy-y))
                return max(p[0] for p in points if p[1]==row)-min(p[0] for p in points if p[1]==row)
            ratios[s]=width(HEIGHT[s]*.81)/width(HEIGHT[s]*.45)
            tail=next(p for p in m['primitives'] if p['part'].startswith('Tail /'))
            tailpoints=[tail['positions'][i:i+3] for i in range(0,len(tail['positions']),3)]
            # At the widest section the paddle is flattened in Y, with real
            # thickness. This catches regression back to a circular tube.
            row=min(tailpoints,key=lambda p:p[0])[2];section=[p for p in tailpoints if abs(p[2]-row)<1e-6]
            width_x=max(p[0] for p in section)-min(p[0] for p in section);depth_y=max(p[1] for p in section)-min(p[1] for p in section)
            self.assertGreater(width_x/depth_y,1.65)
        self.assertGreater(ratios['wooper'],1.35)
        self.assertLess(ratios['quagsire'],1.14)

    def test_adult_pads_have_volume_and_digit_folds_follow_actual_skin(self):
        m=self.models['quagsire'];parts={p['part']:p for p in m['primitives']}
        for side in (-1,1):
            foot=parts[f'Foot / grounded three-lobed pad {side}']
            self.assertGreater(max(foot['positions'][1::3]),.035)
            upper=dict(foot);upper['positions']=[v for i in range(0,len(foot['positions']),3)
                for v in (foot['positions'][i],foot['positions'][i+2],foot['positions'][i+1])]
            arm=parts[f'Arm / fleshy hanging paddle {side}']
            for j in range(2):
                for name,parent,swap in [(f'Toe crease / fitted pad fold {side} {j}',upper,True),
                                         (f'Finger crease / short palm fold {side} {j}',arm,False)]:
                    fold=parts[name]
                    for i in range(0,len(fold['positions']),3):
                        x,y,z=fold['positions'][i:i+3]
                        hits=ray_intersections(parent,x,z if swap else y);self.assertTrue(hits,name)
                        self.assertLess(abs((y if swap else z)-max(hits)),.006,name)

if __name__=='__main__':unittest.main()
