#!/usr/bin/env python3
"""Stored spider anatomy, joint seams and marking contact regressions."""
import json,unittest
from pathlib import Path
from spider_sculpt import sculpture,SPECIES,HEIGHT
from sculpt_test_support import assert_closed_models,assert_model_shading
from sculpt_test_support import ray_intersections
from spider_glb import read_spinarak,read_ariados,geometry_digest

ROOT=Path(__file__).resolve().parents[1]/'crates/crystal-voxel-view/models/battle_species'
def points(p):return {tuple(p['positions'][i:i+3]) for i in range(0,len(p['positions']),3)}

class SpiderTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):cls.models={s:reader(ROOT/(s+'.glb')) for s,reader in zip(SPECIES,(read_spinarak,read_ariados))}
    def test_closed_connected_oriented_parts(self):assert_closed_models(self,self.models)
    def test_normals_match_stored_triangles(self):assert_model_shading(self,self.models)
    def test_regeneration_dimensions_and_budget(self):
        for s,m in self.models.items():
            self.assertEqual(geometry_digest(m),geometry_digest(sculpture(s)));ps=m['primitives']
            self.assertEqual(len(ps),len({p['part'] for p in ps}))
            self.assertEqual(min(v for p in ps for v in p['positions'][1::3]),0.)
            self.assertAlmostEqual(max(v for p in ps for v in p['positions'][1::3]),HEIGHT,places=6)
            self.assertLess(sum(len(p['indices'])//3 for p in ps),8500)
            self.assertLess(len(json.dumps(sculpture(s),separators=(',',':'))),600000)
    def test_six_limbs_with_shared_joint_rings_and_planted_tips(self):
        for s,m in self.models.items():
            ps={p['part']:p for p in m['primitives']};walking=2 if s=='ariados' else 3
            self.assertEqual(sum(n.startswith('Walking leg /') for n in ps),walking*2*5)
            self.assertEqual(sum(n.startswith('Raised leg /') for n in ps),6 if s=='ariados' else 0)
            for side in (-1,1):
                limbs=[(f'Walking leg / {side} {j}',5) for j in range(walking)]
                if s=='ariados':limbs.append((f'Raised leg / {side}',3))
                for name,bands in limbs:
                    for k in range(bands-1):
                        self.assertEqual(len(points(ps[f'{name} band {k}'])&points(ps[f'{name} band {k+1}'])),14)
                    if name.startswith('Walking'):
                        self.assertLess(min(ps[f'{name} band 4']['positions'][1::3]),.003)
    def test_distinct_dorsal_markings_follow_the_actual_carapace(self):
        for s,m in self.models.items():
            ps=m['primitives'];body=ps[0];names=[p['part'] for p in ps]
            self.assertEqual(sum(n.startswith('False eye /') for n in names),2 if s=='spinarak' else 0)
            self.assertEqual(sum('fitted chevron' in n for n in names),4 if s=='ariados' else 0)
            upper=dict(body);upper['positions']=[v for i in range(0,len(body['positions']),3) for v in (body['positions'][i],body['positions'][i+2],body['positions'][i+1])]
            for p in ps:
                if not p['part'].startswith(('Abdomen stripe /','False eye /','False mouth /')):continue
                for x,y,z in points(p):
                    hits=ray_intersections(upper,x,z);self.assertTrue(hits)
                    self.assertLess(abs(y-max(hits)),.022,(s,p['part']))
    def test_heads_overlap_abdomens_and_horns_have_distinct_directions(self):
        for s,m in self.models.items():
            body,head=m['primitives'][:2];overlap=False
            lo=max(min(body['positions'][1::3]),min(head['positions'][1::3]));hi=min(max(body['positions'][1::3]),max(head['positions'][1::3]))
            for k in range(1,15):
                y=lo+(hi-lo)*k/15;a=ray_intersections(body,0,y);b=ray_intersections(head,0,y)
                if a and b and min(max(a),max(b))-max(min(a),min(b))>.01:overlap=True;break
            self.assertTrue(overlap,s)
            horn=next(p for p in m['primitives'] if p['part'].startswith('Horn /'))
            dy=max(horn['positions'][1::3])-min(horn['positions'][1::3]);dz=max(horn['positions'][2::3])-min(horn['positions'][2::3])
            self.assertGreater(dz/dy,2 if s=='ariados' else 0)
            if s=='spinarak':self.assertGreater(dy/dz,2)

    def test_eye_layers_follow_the_head_and_preceding_lens(self):
        for s,m in self.models.items():
            ps={p['part']:p for p in m['primitives']}
            for side in (-1,1):
                parent=m['primitives'][1]
                names=['Eye socket / '+str(side)]
                if s=='ariados':names.append('Eye pupil / '+str(side))
                names.append('Eye glint / '+str(side))
                for name in names:
                    lens=ps[name]
                    for x,y,z in points(lens):
                        hits=ray_intersections(parent,x,y);self.assertTrue(hits,(s,name,x,y))
                        self.assertLess(abs(z-max(hits)),.022,(s,name,x,y,z))
                    parent=lens

if __name__=='__main__':unittest.main()
