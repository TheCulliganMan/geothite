#!/usr/bin/env python3
"""Check stored duck anatomy, real web topology and visible fitted faces."""
import json,unittest
from pathlib import Path
from duck_sculpt import sculpture,HEIGHT
from sculpt_test_support import assert_closed_models,assert_model_shading

ROOT=Path(__file__).resolve().parents[1]/'crates/crystal-voxel-view/models/battle_species'

class DuckTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):cls.models={n:json.loads((ROOT/(n+'.mesh.json')).read_bytes()) for n in ('psyduck','golduck')}

    def test_closed_connected_oriented_anatomy(self):assert_closed_models(self,self.models)
    def test_shading_matches_real_triangle_winding(self):assert_model_shading(self,self.models)

    def test_recipe_grounding_height_and_single_web_parts(self):
        for n,m in self.models.items():
            self.assertEqual(m,sculpture(n));names=[p['part'] for p in m['primitives']]
            self.assertEqual(len(names),len(set(names)))
            self.assertEqual(sum(x.startswith('Body /') for x in names),1)
            self.assertEqual(sum(x.startswith('Foot / single') for x in names),2)
            self.assertEqual(min(v for p in m['primitives'] for v in p['positions'][1::3]),0.)
            self.assertAlmostEqual(max(v for p in m['primitives'] for v in p['positions'][1::3]),HEIGHT,places=6)
            self.assertLess(sum(len(p['indices'])//3 for p in m['primitives']),6000)
            for p in m['primitives']:
                if p['part'].startswith('Foot / single'):self.assertEqual(min(p['positions'][1::3]),0.)
                if p['part'].startswith('Bill /') and 'shell' in p['part']:
                    # The bill is a flattened spatulate volume, not a head-size ball.
                    extent=[max(p['positions'][i::3])-min(p['positions'][i::3]) for i in range(3)]
                    self.assertGreater(extent[0],extent[1]*3.)

    def test_eye_surfaces_face_outward_and_follow_head(self):
        for n,m in self.models.items():
            body=next(p for p in m['primitives'] if p['part'].startswith('Body /'))
            points=[body['positions'][i:i+3] for i in range(0,len(body['positions']),3)]
            triangles=[[points[j] for j in body['indices'][i:i+3]] for i in range(0,len(body['indices']),3)]
            def head_z(x,y):
                zs=[]
                for a,b,c in triangles:
                    den=(b[1]-c[1])*(a[0]-c[0])+(c[0]-b[0])*(a[1]-c[1])
                    if abs(den)<1e-12:continue
                    u=((b[1]-c[1])*(x-c[0])+(c[0]-b[0])*(y-c[1]))/den
                    v=((c[1]-a[1])*(x-c[0])+(a[0]-c[0])*(y-c[1]))/den
                    if min(u,v,1-u-v)>=-1e-6:zs.append(u*a[2]+v*b[2]+(1-u-v)*c[2])
                self.assertTrue(zs,(n,x,y))
                return max(zs)
            for p in m['primitives']:
                if p['part'].startswith(('Eye /','Forehead /')):
                    # Actual stored front center uses +Z-facing shading. This
                    # catches reversed patch closure despite a positive volume.
                    self.assertGreater(p['normals'][2],0.,(n,p['part']))
                    for i in range(0,len(p['positions']),3):
                        x,y,z=p['positions'][i:i+3]
                        self.assertLess(abs(z-head_z(x,y)),.035,(n,p['part'],i//3))
            self.assertEqual(sum(p['part'].startswith('Eye / fitted') for p in m['primitives']),2)

    def test_species_identity_is_anatomical(self):
        a=self.models['psyduck'];b=self.models['golduck']
        for m,counts in [(a,{'Hair /':3,'Finger / temple':6,'Hand / fitted':2,'Crown /':0,'Hand claw /':0}),
                         (b,{'Hair /':0,'Finger / temple':0,'Hand / three':2,'Crown /':4,'Hand claw /':6,'Foot claw /':6,'Foot membrane /':2,'Hand membrane /':4})]:
            for prefix,count in counts.items():self.assertEqual(sum(p['part'].startswith(prefix) for p in m['primitives']),count,prefix)
        width=lambda m:max(abs(v) for p in m['primitives'] if p['part'].startswith('Body /') for v in p['positions'][::3])
        self.assertGreater(width(a),width(b)*1.4)
        tails=[next(p for p in m['primitives'] if p['part'].startswith('Tail /')) for m in (a,b)]
        self.assertGreater(abs(min(tails[1]['positions'][2::3])),abs(min(tails[0]['positions'][2::3]))*1.3)

if __name__=='__main__':unittest.main()
