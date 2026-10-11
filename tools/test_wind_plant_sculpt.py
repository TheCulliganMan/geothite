#!/usr/bin/env python3
"""Stored wind-plant anatomy and fitted botanical detail regressions."""
import json,unittest
from pathlib import Path
from wind_plant_sculpt import sculpture,SPECIES,HEIGHT
from sculpt_test_support import assert_closed_models,assert_model_shading
from test_axolotl_sculpt import ray_intersections

ROOT=Path(__file__).resolve().parents[1]/'crates/crystal-voxel-view/models/battle_species'
class WindPlantTests(unittest.TestCase):
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
            self.assertLess(sum(len(p['indices'])//3 for p in ps),11500)
            self.assertLess(len(json.dumps(m,separators=(',',':'))),550000)
    def test_distinct_crown_and_eye_anatomy(self):
        for s,m in self.models.items():
            names=[p['part'] for p in m['primitives']]
            self.assertEqual(sum(n.startswith('Leaf / continuous serrated') for n in names),2 if s=='hoppip' else 0)
            self.assertEqual(sum(n.startswith('Flower petal /') for n in names),8 if s=='skiploom' else 0)
            self.assertEqual(sum('continuous seed sphere' in n for n in names),3 if s=='jumpluff' else 0)
            self.assertEqual(sum('fine seed bristle' in n for n in names),84 if s=='jumpluff' else 0)
            for p in m['primitives']:
                if p['part'].startswith('Foot /'):self.assertLess(min(p['positions'][1::3]),.003)
        eyes={s:next(p['base_color'] for p in m['primitives'] if p['part'].startswith('Eye / fitted')) for s,m in self.models.items()}
        self.assertEqual(eyes['skiploom'],eyes['jumpluff']);self.assertNotEqual(eyes['hoppip'],eyes['jumpluff'])
    def test_faces_and_leaf_ear_insets_follow_real_skin(self):
        for s,m in self.models.items():
            ps=m['primitives'];body=ps[0]
            for p in ps:
                if p['part'].startswith(('Mouth /','Tongue /')):skin=body;limit=.014
                elif p['part'].startswith('Ear / fitted'):
                    side=p['part'].split()[-1];skin=next(q for q in ps if q['part']=='Ear / pointed pink shell '+side);limit=.009
                else:continue
                for i in range(0,len(p['positions']),3):
                    x,y,z=p['positions'][i:i+3];hits=ray_intersections(skin,x,y)
                    self.assertTrue(hits,(s,p['part']));self.assertLess(abs(z-max(hits)),limit,(s,p['part']))
    def test_real_supporting_foot_contact_and_seed_stem_overlap(self):
        for s,m in self.models.items():
            body=m['primitives'][0]
            for p in m['primitives']:
                if not p['part'].startswith('Foot /'):continue
                x=(max(p['positions'][::3])+min(p['positions'][::3]))/2
                lo=max(min(p['positions'][1::3]),min(body['positions'][1::3]));hi=min(max(p['positions'][1::3]),max(body['positions'][1::3]));overlap=False
                for k in range(1,15):
                    y=lo+(hi-lo)*k/15;a=ray_intersections(p,x,y);b=ray_intersections(body,x,y)
                    if a and b and min(max(a),max(b))-max(min(a),min(b))>.004:overlap=True;break
                self.assertTrue(overlap,(s,p['part']))
        ps=self.models['jumpluff']['primitives']
        for suffix in ('lower -1','lower 1','crown'):
            globe=next(p for p in ps if p['part']==f'Seed head / {suffix} / continuous seed sphere')
            stem=next(p for p in ps if p['part']==('Seed stem / lower globe '+suffix.split()[-1] if suffix!='crown' else 'Seed stem / tall crown peduncle'))
            x=(max(min(globe['positions'][::3]),min(stem['positions'][::3]))+min(max(globe['positions'][::3]),max(stem['positions'][::3])))/2
            lo=max(min(globe['positions'][1::3]),min(stem['positions'][1::3]));hi=min(max(globe['positions'][1::3]),max(stem['positions'][1::3]));overlap=False
            for k in range(1,15):
                y=lo+(hi-lo)*k/15;a=ray_intersections(globe,x,y);b=ray_intersections(stem,x,y)
                if a and b and min(max(a),max(b))-max(min(a),min(b))>.003:overlap=True;break
            self.assertTrue(overlap,suffix)

if __name__=='__main__':unittest.main()
