#!/usr/bin/env python3
"""Independent stored Pikachu topology, skin and continuous ear seam checks."""
import math
from pathlib import Path
import unittest
import pikachu_glb as author
from sculpt_test_support import assert_closed_models,assert_model_shading
from skin_glb_test_support import IDENTITY
from test_gengar_glb import Model,winding_alignment
ASSET=Path(__file__).resolve().parents[1]/'crates/crystal-voxel-view/models/battle_species/pikachu.glb'

class PikachuTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):cls.blob=ASSET.read_bytes();cls.m=Model(cls.blob)
    def test_canonical_single_source_and_bounded_schema(self):
        self.assertEqual(self.blob,author.export_pikachu())
        self.assertEqual(len(self.m.parts),40)
        self.assertEqual(sum(len(v) for v in self.m.vertices),11568)
        self.assertEqual(sum(len(v)//3 for v in self.m.indices),13876)
        self.assertLess(len(self.blob),1024*1024)
        self.assertLessEqual(len(self.m.doc['accessors']),256)
        self.assertFalse(ASSET.with_suffix('.mesh.json').exists())
        with self.assertRaises(ValueError):author.decode_pikachu(self.blob[:-1]+bytes([self.blob[-1]^1]))
    def test_closed_outward_solids_and_normals(self):
        m=self.m.neutral_model();assert_closed_models(self,{'pikachu':m});assert_model_shading(self,{'pikachu':m})
    def test_mouse_anatomy_and_physical_height(self):
        for prefix,count in [('Ear /',2),('Ear tip /',2),('Cheek /',2),('Finger /',8),('Thumb /',2),('Back stripe /',2),('Foot /',2),('Tail /',1),('Tail base /',1)]:
            self.assertEqual(sum(n.startswith(prefix) for n in self.m.names),count,prefix)
        points=[p for v in self.m.vertices for p in v]
        self.assertEqual(min(p[1] for p in points),0.)
        self.assertAlmostEqual(max(p[1] for p in points),1.,places=6)
        body=self.m.vertices[self.m.names.index('Body / continuous plump golden mouse pear')]
        self.assertGreater(max(p[0] for p in body)-min(p[0] for p in body),.40)
    def test_hierarchy_inverse_binds_weights_and_rigid_face(self):
        m=self.m
        self.assertEqual(m.parents[:-1],[p for _,p,_ in author.JOINTS])
        self.assertEqual([n['name'] for n in m.doc['nodes'][:-1]],['pikachu/'+n for n,_,_ in author.JOINTS])
        for matrix in m.matrices():self.assertLess(max(abs(a-b) for a,b in zip(matrix,IDENTITY)),1e-7)
        for ids,weights in zip(m.ids,m.weights):
            for js,ws in zip(ids,weights):
                self.assertAlmostEqual(sum(ws),1.,places=7)
                self.assertTrue(all(j<len(author.JOINTS) for j in js))
        for p,name in enumerate(m.names):
            if name.startswith(('Eye /','Head /','Eye glint /','Cheek /','Nose /','Mouth /')):
                self.assertTrue(all(js==[2,0,0,0] for js in m.ids[p]))
    def test_black_tips_share_the_gold_seam_through_all_clips(self):
        m=self.m
        for side in (-1,1):
            a=m.names.index('Ear / elongated gold mouse pinna '+str(side));b=m.names.index('Ear tip / continuous black terminal '+str(side))
            left={tuple(p):i for i,p in enumerate(m.vertices[a])};right={tuple(p):i for i,p in enumerate(m.vertices[b])}
            shared=left.keys()&right.keys();self.assertEqual(len(shared),20)
            for clip in m.clips:
                for step in range(17):
                    mats=m.matrices(clip,step/16)
                    for p in shared:self.assertEqual(m.point(a,left[p],mats),m.point(b,right[p],mats))
    def test_planted_pads_floor_and_animation_strain(self):
        m=self.m;fixed=[p for p,n in enumerate(m.names) if n.startswith(('Foot /','Toe crease /'))]
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
    def test_neutral_endpoints_and_early_brace_with_tail_follow_through(self):
        m=self.m
        for clip in m.clips:
            for t in (0.,1.):
                for matrix in m.matrices(clip,t):self.assertLess(max(abs(a-b) for a,b in zip(matrix,IDENTITY)),1e-7)
        for joint in (1,2):
            values=[m.rotations('attack',i/100)[joint][0] for i in range(101)]
            self.assertGreater(values[10],max(values)*.4);self.assertGreater(values[20],max(values)*.9)
        self.assertGreater(m.rotations('attack',.28)[10][1],m.rotations('attack',.18)[10][1])
        for animation in m.doc['animations']:
            idle=animation['name'].endswith('.idle')
            self.assertEqual(animation['extras']['loopSuggested'],idle)
            self.assertEqual(animation['extras']['cueRelative'],not idle)
            self.assertEqual({c['target']['path'] for c in animation['channels']},{'rotation'})

if __name__=='__main__':unittest.main()
