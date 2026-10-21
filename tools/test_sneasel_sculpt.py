#!/usr/bin/env python3
"""Stored claw-weasel skin, rooted anatomy and silhouette checks."""
import os,json,unittest
from pathlib import Path
from sneasel_sculpt import sculpture,HEIGHT
from sculpt_test_support import assert_closed_models,assert_model_shading,ray_intersections
from sneasel_glb import read_sneasel,geometry_digest
ROOT=Path(__file__).resolve().parents[1]/'crates/crystal-voxel-view/models/battle_species/sneasel.glb'
ROOT=Path(os.environ.get('SNEASEL_GLB',str(ROOT)))
class SneaselTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):cls.model=read_sneasel(ROOT);cls.ps=cls.model['primitives']
    def test_closed_connected_oriented_parts(self):assert_closed_models(self,{'sneasel':self.model})
    def test_normals_match_stored_triangles(self):assert_model_shading(self,{'sneasel':self.model})
    def test_regeneration_bounds_counts_and_budget(self):
        self.assertEqual(geometry_digest(self.model),geometry_digest(sculpture()));ps=self.ps
        self.assertEqual(len(ps),len({p['part'] for p in ps}))
        self.assertEqual(min(v for p in ps for v in p['positions'][1::3]),0.)
        self.assertAlmostEqual(max(v for p in ps for v in p['positions'][1::3]),HEIGHT,places=6)
        self.assertLess(sum(len(p['indices'])//3 for p in ps),13500)
        for prefix,count in (('Ear /',2),('Plume /',1),('Tail / swept',3),('Talon / hooked',4),('Talon / paired',4),('Gem /',2)):
            self.assertEqual(sum(p['part'].startswith(prefix) for p in ps),count)
    def test_rooted_anatomy_and_flat_supports(self):
        from sculpt_test_support import parts_overlap
        ps=self.ps
        def find(prefix):return next(p for p in ps if p['part'].startswith(prefix))
        pairs=[(find('Head /'),find('Body /')),(find('Tail / short'),find('Body /'))]
        for p in ps:
            n=p['part'];side=n.split()[-1]
            if n.startswith(('Ear /','Plume /')):pairs.append((p,find('Head /')))
            elif n.startswith('Tail / swept'):pairs.append((p,find('Tail / short')))
            elif n.startswith(('Arm /','Leg /')):pairs.append((p,find('Body /')))
            elif n.startswith('Hand /'):pairs.append((p,find('Arm / curved reaching forelimb '+side)))
            elif n.startswith('Talon / hooked'):pairs.append((p,find('Hand / broad claw-bearing palm '+n.split()[-2])))
            elif n.startswith('Foot /'):
                pairs.append((p,find('Leg / upright supporting hock '+side)));self.assertEqual(min(p['positions'][1::3]),0.)
            elif n.startswith('Talon / paired'):pairs.append((p,find('Foot / broad grounded teal pad '+n.split()[-2])))
        for a,b in pairs:self.assertTrue(parts_overlap(a,b),(a['part'],b['part']))
    def test_fitted_facial_features_and_gems_follow_stored_skin(self):
        for p in self.ps:
            name=p['part'];side=name.split()[-1]
            if name.startswith('Eye / fitted dark'):skin=self.ps[1]
            elif name.startswith('Eye / fitted ivory'):skin=next(q for q in self.ps if q['part']=='Eye / fitted dark slanted socket '+side)
            elif name.startswith('Eye / narrow'):skin=next(q for q in self.ps if q['part']=='Eye / fitted ivory angular white '+side)
            elif name.startswith('Eye / vertical'):skin=next(q for q in self.ps if q['part']=='Eye / narrow fitted red iris '+side)
            elif name.startswith('Gem /'):skin=self.ps[1] if name.endswith('forehead') else self.ps[0]
            elif name.startswith('Mouth / fitted narrow'):skin=self.ps[1]
            elif name.startswith('Mouth / small'):skin=self.ps[1]
            else:continue
            for i in range(0,len(p['positions']),3):
                x,y,z=p['positions'][i:i+3];hits=ray_intersections(skin,x,y)
                self.assertTrue(hits,name);self.assertLess(abs(z-max(hits)),.008,name)
if __name__=='__main__':unittest.main()
