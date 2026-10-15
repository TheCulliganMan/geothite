#!/usr/bin/env python3
"""Stored Sudowoodo anatomy and independent glTF skin playback checks."""
import math
import os
from pathlib import Path
import unittest
import sudowoodo_glb as author
from sudowoodo_sculpt import sculpture,HEIGHT
from sculpt_test_support import assert_closed_models,assert_model_shading
from skin_glb_test_support import IDENTITY,slerp
from test_gengar_glb import Model,winding_alignment
ASSET=Path(os.environ.get('SUDOWOODO_GLB',Path(__file__).resolve().parents[1]/'crates/crystal-voxel-view/models/battle_species/sudowoodo.glb'))


class SudowoodoTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):cls.blob=ASSET.read_bytes();cls.m=Model(cls.blob)
    def test_canonical_storage_and_schema(self):
        self.assertEqual(self.blob,author.export_sudowoodo())
        self.assertEqual(author.geometry_digest(self.m.neutral_model()),author.geometry_digest(sculpture()))
        self.assertEqual(len(self.m.parts),33)
        self.assertEqual(sum(len(v) for v in self.m.vertices),9089)
        self.assertEqual(sum(len(v)//3 for v in self.m.indices),11376)
        self.assertLess(len(self.m.doc['accessors']),256)
        self.assertLess(len(self.blob),author.MAX_BYTES)
        self.assertFalse(ASSET.with_suffix('.mesh.json').exists())
        with self.assertRaises(ValueError):author.decode_sudowoodo(self.blob[:-1]+bytes([self.blob[-1]^1]))
    def test_closed_connected_outward_parts_and_normals(self):
        model=self.m.neutral_model();assert_closed_models(self,{'sudowoodo':model});assert_model_shading(self,{'sudowoodo':model})
    def test_forked_trunk_six_green_digits_golden_spots_and_ground(self):
        names=self.m.names
        for prefix,count in [('Foot /',2),('Arm /',2),('Eye / fitted',2),('Finger orb /',6),('Finger stalk /',6),('Spot /',5),('Crown /',4)]:
            self.assertEqual(sum(n.startswith(prefix) for n in names),count,prefix)
        points=[p for part in self.m.vertices for p in part]
        self.assertEqual(min(p[1] for p in points),0.)
        self.assertAlmostEqual(max(p[1] for p in points),HEIGHT,places=6)
        self.assertGreater(max(p[0] for p in points)-min(p[0] for p in points),.9)
    def test_hierarchy_binds_and_normalized_weights(self):
        m=self.m
        self.assertEqual([n['name'] for n in m.doc['nodes'][:-1]],['sudowoodo/'+n for n,_,_ in author.JOINTS])
        self.assertEqual(m.parents[:-1],[p for _,p,_ in author.JOINTS])
        for matrix in m.matrices():self.assertLess(max(abs(a-b) for a,b in zip(matrix,IDENTITY)),1e-7)
        for ids,weights in zip(m.ids,m.weights):
            for js,ws in zip(ids,weights):
                self.assertAlmostEqual(sum(ws),1.,places=7)
                self.assertTrue(all(j<len(author.JOINTS) for j in js))
        for p,name in enumerate(m.names):
            if name.startswith(('Eye /','Smile /')):self.assertTrue(all(js==[2,0,0,0] for js in m.ids[p]))
            if name.startswith(('Finger orb /','Finger stalk /')):
                joint=4 if name.split(' / ')[1].split()[0]=='-1' else 6
                self.assertTrue(all(js==[joint,0,0,0] for js in m.ids[p]))
    def test_neutral_endpoints_periodic_idle_and_cue_contract(self):
        m=self.m
        for clip in m.clips:
            for t in (0.,1.):
                for matrix in m.matrices(clip,t):self.assertLess(max(abs(a-b) for a,b in zip(matrix,IDENTITY)),1e-7)
        a=m.rotations('idle',.0001);b=m.rotations('idle',.9999)
        self.assertLess(max(math.dist(x,y) for x,y in zip(a,b)),.0001)
        for animation in m.doc['animations']:
            idle=animation['name'].endswith('.idle')
            self.assertEqual(animation['extras']['loopSuggested'],idle)
            self.assertEqual(animation['extras']['cueRelative'],not idle)
            self.assertEqual({c['target']['path'] for c in animation['channels']},{'rotation'})
    def test_planted_pads_floor_winding_and_triangle_strain(self):
        m=self.m;feet=[i for i,n in enumerate(m.names) if n.startswith('Foot /')]
        for clip in m.clips:
            for step in range(33):
                mats=m.matrices(clip,step/32)
                for p in feet:self.assertEqual(m.posed(p,mats),m.vertices[p])
                for p in range(len(m.parts)):
                    posed=m.posed(p,mats)
                    self.assertGreaterEqual(min(v[1] for v in posed),-1e-7,(clip,step,m.names[p]))
                    if step%4==0:
                        normals=m.posed(p,mats,True)
                        for i in range(0,len(m.indices[p]),3):
                            tri=m.indices[p][i:i+3];cosine,area=winding_alignment(posed,normals,tri)
                            self.assertGreater(cosine,0.,(clip,step,m.names[p],i));self.assertGreater(area,1e-12)
                            for a,b in zip(tri,tri[1:]+tri[:1]):
                                ratio=math.dist(posed[a],posed[b])/math.dist(m.vertices[p][a],m.vertices[p][b])
                                self.assertTrue(.70<ratio<1.30,(clip,step,m.names[p],ratio))
        for a,b in (('idle','attack'),('attack','hit'),('hit','idle')):
            for step in range(17):
                mats=m.matrices(rotations=[slerp(x,y,step/16) for x,y in zip(m.rotations(a,.18),m.rotations(b,.28))])
                for p in feet:self.assertEqual(m.posed(p,mats),m.vertices[p])
    def test_early_brace_follow_through_and_visible_branch_motion(self):
        m=self.m
        for joint in (1,2):
            values=[m.rotations('attack',i/100)[joint][0] for i in range(101)]
            self.assertTrue(all(v>0 for v in values[1:26]));self.assertGreater(values[10],max(values)*.4);self.assertGreater(values[20],max(values)*.9)
        a=m.rotations('attack',.18);b=m.rotations('attack',.28)
        for joint in (4,6):self.assertGreater(abs(b[joint][0]),abs(a[joint][0]))
        for p,name in enumerate(m.names):
            if name.startswith('Finger orb /'):
                self.assertGreater(max(math.dist(a,b) for a,b in zip(m.vertices[p],m.posed(p,m.matrices('attack',.28)))),.025,name)


if __name__=='__main__':unittest.main()
