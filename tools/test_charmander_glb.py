#!/usr/bin/env python3
"""Independent stored Charmander topology and articulated anatomy checks."""
import math
from pathlib import Path
import unittest
import charmander_glb as author
from sculpt_test_support import assert_closed_models,assert_model_shading
from skin_glb_test_support import IDENTITY
from test_gengar_glb import Model,winding_alignment
ASSET=Path(__file__).resolve().parents[1]/'crates/crystal-voxel-view/models/battle_species/charmander.glb'

class CharmanderTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):cls.blob=ASSET.read_bytes();cls.m=Model(cls.blob)
    def test_canonical_single_source_and_bounded_schema(self):
        self.assertEqual(self.blob,author.export_charmander())
        self.assertEqual(len(self.m.parts),43)
        self.assertEqual(sum(len(v) for v in self.m.vertices),13250)
        self.assertEqual(sum(len(v)//3 for v in self.m.indices),21086)
        self.assertLess(len(self.blob),1024*1024)
        self.assertLessEqual(len(self.m.doc['accessors']),256)
        self.assertFalse(ASSET.with_suffix('.mesh.json').exists())
    def test_closed_outward_solids_and_smooth_normals(self):
        m=self.m.neutral_model();assert_closed_models(self,{'charmander':m});assert_model_shading(self,{'charmander':m})
    def test_anatomy_and_physical_height(self):
        for prefix,count in [('Arm /',2),('Hand /',2),('Finger /',6),('Foot claw /',6),('Foot /',2),('Fang /',2),('Tail /',1),('Flame /',5)]:
            self.assertEqual(sum(n.startswith(prefix) for n in self.m.names),count,prefix)
        points=[p for v in self.m.vertices for p in v]
        self.assertEqual(min(p[1] for p in points),0.)
        self.assertAlmostEqual(max(p[1] for p in points),1.,places=6)
        self.assertGreater(max(p[0] for p in points)-min(p[0] for p in points),.60)
    def test_hierarchy_inverse_binds_and_rigid_face(self):
        m=self.m
        self.assertEqual(m.parents[:-1],[p for _,p,_ in author.JOINTS])
        self.assertEqual([n['name'] for n in m.doc['nodes'][:-1]],['charmander/'+n for n,_,_ in author.JOINTS])
        for matrix in m.matrices():self.assertLess(max(abs(a-b) for a,b in zip(matrix,IDENTITY)),1e-7)
        for ids,weights in zip(m.ids,m.weights):
            for js,ws in zip(ids,weights):
                self.assertAlmostEqual(sum(ws),1.,places=7)
                self.assertTrue(all(j<len(author.JOINTS) for j in js))
        for p,name in enumerate(m.names):
            if name.startswith(('Eye /','Head /','Eye glint /','Iris /','Pupil /','Mouth /','Tongue /','Nostril /','Fang /')):
                self.assertTrue(all(js==[2,0,0,0] for js in m.ids[p]))
    def test_planted_soles_floor_and_animation_strain(self):
        m=self.m;fixed=[p for p,n in enumerate(m.names) if n.startswith(('Foot /','Foot claw /'))]
        for clip in m.clips:
            for step in range(17):
                mats=m.matrices(clip,step/16)
                for p in fixed:self.assertEqual(m.posed(p,mats),m.vertices[p])
                for p in range(len(m.parts)):
                    posed=m.posed(p,mats);self.assertGreaterEqual(min(v[1] for v in posed),-1e-7,(clip,step,m.names[p]))
                    if step%4==0:
                        normals=m.posed(p,mats,True)
                        for i in range(0,len(m.indices[p]),3):
                            tri=m.indices[p][i:i+3];cosine,area=winding_alignment(posed,normals,tri)
                            self.assertGreater(cosine,0.,(clip,step,m.names[p],i));self.assertGreater(area,1e-12)
                            for a,b in zip(tri,tri[1:]+tri[:1]):
                                ratio=math.dist(posed[a],posed[b])/math.dist(m.vertices[p][a],m.vertices[p][b])
                                self.assertTrue(.7<ratio<1.3,(clip,step,m.names[p],ratio))
    def test_curved_tail_and_fire_are_bound_to_the_tail_chain(self):
        m=self.m
        tail=m.names.index('Tail / continuous tapering curved orange lizard tail')
        self.assertTrue(all(all(j in (0,7,8,9) for j in ids) for ids in m.ids[tail]))
        self.assertTrue(any(9 in ids for ids in m.ids[tail]))
        for p,name in enumerate(m.names):
            if name.startswith('Flame /'):
                self.assertTrue(all(all(j in (0,9,10) for j in ids) for ids in m.ids[p]))
                self.assertTrue(any(10 in ids for ids in m.ids[p]))
        self.assertGreater(abs(m.rotations('attack',.28)[10][2]),abs(m.rotations('attack',.18)[10][2]))
    def test_neutral_endpoints_and_early_torso_brace(self):
        m=self.m
        for clip in m.clips:
            for t in (0.,1.):
                for matrix in m.matrices(clip,t):self.assertLess(max(abs(a-b) for a,b in zip(matrix,IDENTITY)),1e-7)
        for joint in (1,2):
            values=[m.rotations('attack',i/100)[joint][0] for i in range(101)]
            self.assertGreater(values[10],max(values)*.4);self.assertGreater(values[20],max(values)*.9)
        for animation in m.doc['animations']:
            idle=animation['name'].endswith('.idle')
            self.assertEqual(animation['extras']['loopSuggested'],idle)
            self.assertEqual(animation['extras']['cueRelative'],not idle)
            self.assertEqual({c['target']['path'] for c in animation['channels']},{'rotation'})

if __name__=='__main__':unittest.main()
