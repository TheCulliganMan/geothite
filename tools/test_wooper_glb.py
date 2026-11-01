#!/usr/bin/env python3
"""Stored Wooper skin regressions with independent glTF matrix playback."""
import math
import os
from pathlib import Path
import unittest
import wooper_glb as author
from skin_glb_test_support import IDENTITY, slerp
from test_gengar_glb import Model, winding_alignment

ASSET=Path(os.environ.get('WOOPER_GLB',Path(__file__).resolve().parents[1]/'crates/crystal-voxel-view/models/battle_species/wooper.glb'))


class WooperTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.blob=ASSET.read_bytes();cls.m=Model(cls.blob)

    def test_canonical_storage_and_neutral_geometry(self):
        self.assertEqual(self.blob,author.export_wooper())
        self.assertEqual(author.geometry_digest(self.m.neutral_model()),'d23893a24e7dd83f9711b2309ece7d7b24094ea0184ffa86237d9ab059e0bf88')
        self.assertEqual(len(self.m.parts),22)
        self.assertEqual(sum(len(v) for v in self.m.vertices),3991)
        self.assertEqual(sum(len(v)//3 for v in self.m.indices),5532)
        self.assertLess(len(self.blob),author.MAX_BYTES)
        self.assertFalse(ASSET.with_suffix('.mesh.json').exists())
        with self.assertRaises(ValueError):author.decode_wooper(self.blob[:-1]+bytes([self.blob[-1]^1]))

    def test_joint_hierarchy_weights_and_neutral_binds(self):
        m=self.m
        self.assertEqual([n['name'] for n in m.doc['nodes'][:-1]],['wooper/'+n for n,_,_ in author.JOINTS])
        self.assertEqual(m.parents[:-1],[p for _,p,_ in author.JOINTS])
        for matrix in m.matrices():self.assertLess(max(abs(a-b) for a,b in zip(matrix,IDENTITY)),1e-7)
        for ids,weights in zip(m.ids,m.weights):
            for js,ws in zip(ids,weights):
                self.assertAlmostEqual(sum(ws),1.,places=7)
                self.assertTrue(all(j<len(author.JOINTS) for j in js))
        for i,name in enumerate(m.names):
            if name.startswith(('Eye /','Mouth /')):
                self.assertTrue(all(js==[2,0,0,0] for js in m.ids[i]))
            if name.startswith(('Gill /','Gill branch /')):
                side=name.split(' / ')[1].split()[0];joint=5 if side=='-1' else 6
                self.assertTrue(all(js==[joint,0,0,0] for js in m.ids[i]))

    def test_smile_has_occupied_roots_across_its_width_through_clips(self):
        from sculpt_test_support import parts_overlap, indexed_ray_intersections as rays
        m=self.m
        parent=next(i for i,n in enumerate(m.names) if n.startswith('Body /'))
        child=m.names.index('Mouth / fitted broad curved smile')
        self.assertEqual(len(m.vertices[child]),15*8)
        for clip in m.clips:
            for t in (0.,.31,.67):
                mats=m.matrices(clip,t)
                body={'positions':sum(m.posed(parent,mats),[]),'indices':m.indices[parent]}
                points=m.posed(child,mats)
                mouth={'positions':sum(points,[]),'indices':m.indices[child]}
                self.assertTrue(parts_overlap(body,mouth),(clip,t,'detached smile'))
                # Each closed cross-section must straddle actual stored skin,
                # including both ends; a single touching tip is insufficient.
                for ring in range(15):
                    gaps=[]
                    for x,y,z in points[ring*8:(ring+1)*8]:
                        hits=rays(body,x,y);self.assertTrue(hits,(clip,t,ring))
                        gaps.append(z-max(hits))
                    self.assertLess(min(gaps),-.0003,(clip,t,ring))
                    self.assertGreater(max(gaps),.0003,(clip,t,ring))
                    self.assertGreater(min(gaps),-.004)
                    self.assertLess(max(gaps),.007)

    def test_clips_endpoints_and_idle_seam(self):
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

    def test_contacts_floor_winding_and_strain_through_clips_and_blends(self):
        m=self.m;feet=[i for i,n in enumerate(m.names) if n.startswith('Foot /')]
        self.assertEqual(len(feet),2)
        for clip in m.clips:
            for step in range(33):
                mats=m.matrices(clip,step/32)
                for p in feet:self.assertEqual(m.posed(p,mats),m.vertices[p])
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
                                self.assertTrue(.75<ratio<1.25,(clip,step,m.names[p],ratio))
        for a,b in (('idle','attack'),('attack','hit'),('hit','idle')):
            for step in range(17):
                q=[slerp(x,y,step/16) for x,y in zip(m.rotations(a,.18),m.rotations(b,.28))]
                mats=m.matrices(rotations=q)
                for p in feet:self.assertEqual(m.posed(p,mats),m.vertices[p])

    def test_early_forward_brace_and_distinct_follow_through(self):
        m=self.m
        for joint in (1,2):
            samples=[m.rotations('attack',i/100)[joint][0] for i in range(101)]
            self.assertTrue(all(x>0 for x in samples[1:26]))
            self.assertGreater(samples[10],max(samples)*.4)
            self.assertGreater(samples[20],max(samples)*.9)
        early=m.rotations('attack',.18);late=m.rotations('attack',.28)
        self.assertGreater(abs(late[8][0]),abs(early[8][0]))
        self.assertGreater(abs(late[5][0]),abs(early[5][0]))
        self.assertGreater(math.dist(late[5],late[6]),.01)
        tail=m.names.index('Tail / broad rising paddle')
        self.assertGreater(max(math.dist(x,y) for x,y in zip(m.vertices[tail],m.posed(tail,m.matrices('attack',.28)))),.02)


if __name__=='__main__':unittest.main()
