#!/usr/bin/env python3
"""Independent playback of stored Gyarados skins and rotation clips."""
import math
import os
from pathlib import Path
import unittest
import gyarados_glb as author
from skin_glb_test_support import IDENTITY,slerp
from test_gengar_glb import Model,winding_alignment
ASSET=Path(os.environ.get('GYARADOS_GLB',Path(__file__).resolve().parents[1]/'crates/crystal-voxel-view/models/battle_species/gyarados.glb'))


class GyaradosSkinTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):cls.blob=ASSET.read_bytes();cls.m=Model(cls.blob)
    def test_canonical_geometry_skin_schema_and_rejection(self):
        self.assertEqual(self.blob,author.export_gyarados())
        self.assertEqual(author.geometry_digest(self.m.neutral_model()),'555f2c6aace148f0fa0e3412cdb5912581b5f6d6f60341c6eeaecdcb7d4ac045')
        self.assertEqual(len(self.m.parts),40)
        self.assertEqual(sum(len(v) for v in self.m.vertices),9187)
        self.assertEqual(sum(len(v)//3 for v in self.m.indices),11666)
        self.assertLess(len(self.m.doc['accessors']),256)
        self.assertLess(len(self.blob),author.MAX_BYTES)
        with self.assertRaises(ValueError):author.decode_gyarados(self.blob[:-1]+bytes([self.blob[-1]^1]))
    def test_hierarchy_normalized_weights_and_neutral_binds(self):
        m=self.m
        self.assertEqual([n['name'] for n in m.doc['nodes'][:-1]],['gyarados/'+n for n,_,_ in author.JOINTS])
        self.assertEqual(m.parents[:-1],[p for _,p,_ in author.JOINTS])
        for matrix in m.matrices():self.assertLess(max(abs(a-b) for a,b in zip(matrix,IDENTITY)),1e-7)
        for ids,weights in zip(m.ids,m.weights):
            for js,ws in zip(ids,weights):
                self.assertAlmostEqual(sum(ws),1.,places=7)
                self.assertTrue(all(j<len(author.JOINTS) for j in js))
        for p,name in enumerate(m.names):
            if name.startswith(('Eye /','Crest /','Cheek fin /')):
                self.assertTrue(all(js==[3,0,0,0] for js in m.ids[p]))
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
    def test_grounded_coil_floor_clearance_winding_and_strain(self):
        m=self.m;contacts=[i for i,p in enumerate(m.vertices[0]) if p[1]<.035]
        self.assertGreater(len(contacts),5)
        for clip in m.clips:
            for step in range(33):
                mats=m.matrices(clip,step/32)
                for i in contacts:self.assertEqual(m.point(0,i,mats),m.vertices[0][i])
                for p in range(len(m.parts)):
                    posed=m.posed(p,mats)
                    self.assertGreaterEqual(min(v[1] for v in posed),-1e-7,(clip,step,m.names[p]))
                    self.assertTrue(all(math.isfinite(v) for point in posed for v in point))
                    if step%4==0:
                        normals=m.posed(p,mats,True)
                        for i in range(0,len(m.indices[p]),3):
                            tri=m.indices[p][i:i+3];cosine,area=winding_alignment(posed,normals,tri)
                            self.assertGreater(cosine,0.,(clip,step,m.names[p],i))
                            self.assertGreater(area,1e-12)
                            for a,b in zip(tri,tri[1:]+tri[:1]):
                                ratio=math.dist(posed[a],posed[b])/math.dist(m.vertices[p][a],m.vertices[p][b])
                                self.assertTrue(.70<ratio<1.30,(clip,step,m.names[p],ratio))
        for a,b in (('idle','attack'),('attack','hit'),('hit','idle')):
            for step in range(17):
                q=[slerp(x,y,step/16) for x,y in zip(m.rotations(a,.18),m.rotations(b,.28))]
                mats=m.matrices(rotations=q)
                for i in contacts:self.assertEqual(m.point(0,i,mats),m.vertices[0][i])
    def test_early_brace_jaw_opening_and_delayed_appendages(self):
        m=self.m
        for joint in (1,2,3):
            samples=[m.rotations('attack',i/100)[joint][0] for i in range(101)]
            self.assertTrue(all(x>0 for x in samples[1:26]))
            self.assertGreater(samples[10],max(samples)*.4)
            self.assertGreater(samples[20],max(samples)*.9)
        jaw=[m.rotations('attack',i/100)[4][0] for i in range(101)]
        self.assertGreater(jaw[10],max(jaw)*.7)
        early=m.rotations('attack',.18);late=m.rotations('attack',.28)
        for joint in (5,6):self.assertGreater(abs(late[joint][0]),abs(early[joint][0]))
        self.assertGreater(abs(late[8][1]),abs(early[8][1]))
        p=m.names.index('Jaw / forward lower mandible');mats=m.matrices('attack',.12)
        from skin_glb_test_support import transform
        head_only=[transform(mats[3],v) for v in m.vertices[p]]
        self.assertGreater(max(math.dist(a,b) for a,b in zip(m.posed(p,mats),head_only)),.01)


if __name__=='__main__':unittest.main()
