#!/usr/bin/env python3
"""Check stored pitcher anatomy, cavity depth, topology and shading."""
import json,unittest
from pathlib import Path
from pitcher_sculpt import sculpture,SPECIES,HEIGHT
from sculpt_test_support import assert_closed_models,assert_model_shading

ROOT=Path(__file__).resolve().parents[1]/'crates/crystal-voxel-view/models/battle_species'

class PitcherTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):cls.models={s:json.loads((ROOT/(s+'.mesh.json')).read_bytes()) for s in SPECIES}

    def test_closed_connected_oriented_anatomy(self):assert_closed_models(self,self.models)
    def test_normals_match_triangle_winding(self):assert_model_shading(self,self.models)

    def test_recipe_dimensions_and_bounded_geometry(self):
        for s,m in self.models.items():
            self.assertEqual(m,sculpture(s))
            names=[p['part'] for p in m['primitives']]
            self.assertEqual(len(names),len(set(names)))
            self.assertEqual(min(v for p in m['primitives'] for v in p['positions'][1::3]),0.)
            self.assertAlmostEqual(max(v for p in m['primitives'] for v in p['positions'][1::3]),HEIGHT,places=6)
            self.assertLess(sum(len(p['indices'])//3 for p in m['primitives']),7000)

    def test_real_mouth_recess_and_leaf_ridges(self):
        for s,m in self.models.items():
            parts=m['primitives'];axis=1 if s=='victreebel' else 2
            rim=next(p for p in parts if 'rolled' in p['part'])
            throat=next(p for p in parts if any(x in p['part'] for x in ('recessed','shaded')))
            lip_front=max(rim['positions'][axis::3]);throat_front=max(throat['positions'][axis::3])
            self.assertGreater(lip_front-throat_front,.055)
            self.assertEqual(sum('/ raised midrib' in p['part'] for p in parts),3 if s=='victreebel' else 2)
            self.assertEqual(sum('Fang /' in p['part'] for p in parts),2 if s=='victreebel' else 0)
            self.assertEqual(sum('Eye /' in p['part'] and any(x in p['part'] for x in ('black','oval')) for p in parts),2)

    def test_pitcher_pigment_is_fitted_shallow_geometry(self):
        for s in ('weepinbell','victreebel'):
            spots=[p for p in self.models[s]['primitives'] if p['part'].startswith('Pigment /')]
            self.assertEqual(len(spots),5)
            body=next(p for p in self.models[s]['primitives'] if p['part'].startswith('Body /'))
            ps=[body['positions'][i:i+3] for i in range(0,len(body['positions']),3)]
            triangles=[[ps[j] for j in body['indices'][i:i+3]] for i in range(0,len(body['indices']),3)]
            for p in spots:
                for i in range(0,len(p['positions']),3):
                    x,y,z=p['positions'][i:i+3];zs=[]
                    for a,b,c in triangles:
                        den=(b[1]-c[1])*(a[0]-c[0])+(c[0]-b[0])*(a[1]-c[1])
                        if abs(den)<1e-12:continue
                        u=((b[1]-c[1])*(x-c[0])+(c[0]-b[0])*(y-c[1]))/den
                        v=((c[1]-a[1])*(x-c[0])+(a[0]-c[0])*(y-c[1]))/den
                        if min(u,v,1-u-v)>=-1e-6:zs.append(u*a[2]+v*b[2]+(1-u-v)*c[2])
                    self.assertTrue(zs,(s,p['part']))
                    self.assertLess(abs(z-max(zs)),.009,(s,p['part']))

if __name__=='__main__':unittest.main()
