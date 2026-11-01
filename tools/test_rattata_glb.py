#!/usr/bin/env python3
"""Stored Rattata topology, fitted face, planted paws and curled-tail skin checks."""
import math
from pathlib import Path
import unittest
import rattata_glb as author
from sculpt_test_support import assert_closed_models,assert_model_shading
from skin_glb_test_support import IDENTITY
from test_gengar_glb import Model,winding_alignment
ASSET=Path(__file__).resolve().parents[1]/'crates/crystal-voxel-view/models/battle_species/rattata.glb'

class RattataTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):cls.blob=ASSET.read_bytes();cls.m=Model(cls.blob)
    def test_single_canonical_bounded_skin(self):
        self.assertEqual(self.blob,author.export_rattata())
        self.assertEqual(len(self.m.parts),42)
        self.assertEqual(sum(len(v) for v in self.m.vertices),11494)
        self.assertEqual(sum(len(v)//3 for v in self.m.indices),20432)
        self.assertLess(len(self.blob),1024*1024)
        self.assertLessEqual(len(self.m.doc['accessors']),256)
        self.assertFalse(ASSET.with_suffix('.mesh.json').exists())
    def test_closed_outward_solids_and_smooth_normals(self):
        m=self.m.neutral_model();assert_closed_models(self,{'rattata':m});assert_model_shading(self,{'rattata':m})
    def test_rodent_anatomy_height_and_curled_tail(self):
        for prefix,count in [('Body /',1),('Head /',1),('Muzzle /',2),('Incisor /',2),('Whisker /',6),
            ('Paw /',4),('Toes /',4),('Leg /',4),('Ear /',2),('Ear lining /',2),('Tail /',1)]:
            self.assertEqual(sum(n.startswith(prefix) for n in self.m.names),count,prefix)
        points=[p for v in self.m.vertices for p in v]
        self.assertEqual(min(p[1] for p in points),0.)
        self.assertAlmostEqual(max(p[1] for p in points),.82,places=6)
        body=self.m.vertices[next(i for i,n in enumerate(self.m.names) if n.startswith('Body /'))]
        self.assertGreater(max(v[2] for v in body)-min(v[2] for v in body),1.5*(max(v[1] for v in body)-min(v[1] for v in body)))
        self.assertGreater(max(p[1] for p in author.TAIL),author.TAIL[-1][1], 'tail returns downward inside its curl')
        self.assertGreater(max(p[0] for p in author.TAIL),author.TAIL[-1][0], 'tip returns toward the curl center')
    def test_hierarchy_inverse_binds_and_coherent_face(self):
        m=self.m
        self.assertEqual(m.parents[:-1],[p for _,p,_ in author.JOINTS])
        self.assertEqual([n['name'] for n in m.doc['nodes'][:-1]],['rattata/'+n for n,_,_ in author.JOINTS])
        for matrix in m.matrices():self.assertLess(max(abs(a-b) for a,b in zip(matrix,IDENTITY)),1e-7)
        for ids,weights in zip(m.ids,m.weights):
            for js,ws in zip(ids,weights):
                self.assertAlmostEqual(sum(ws),1.,places=7)
                self.assertTrue(all(j<len(author.JOINTS) for j in js))
        for i,name in enumerate(m.names):
            if name.startswith(('Eye /','Pupil /','Eye glint /','Brow /','Muzzle /','Nose /','Incisor /','Whisker /','Jaw /','Mouth cavity /','Head /')):
                self.assertTrue(all(js==[3,0,0,0] for js in m.ids[i]),name)
    def test_planted_paws_floor_and_animation_strain(self):
        m=self.m;fixed=[(p,i) for p,v in enumerate(m.vertices) if m.names[p].startswith(('Paw /','Toes /','Leg /')) for i in range(len(v))]
        for clip in m.clips:
            for step in range(17):
                mats=m.matrices(clip,step/16)
                for p,i in fixed:self.assertEqual(m.point(p,i,mats),m.vertices[p][i])
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
    def test_three_tail_joints_and_fitted_ear_bowls(self):
        m=self.m;tail=next(i for i,n in enumerate(m.names) if n.startswith('Tail /'))
        self.assertEqual({j for ids,ws in zip(m.ids[tail],m.weights[tail]) for j,w in zip(ids,ws) if w>0.},{1,6,7,8})
        for i,name in enumerate(m.names):
            if name.startswith('Ear lining /'):
                side=-1 if name.endswith('-1') else 1
                shell=next(k for k,n in enumerate(m.names) if n.startswith('Ear /') and n.endswith(' '+str(side)))
                owner=4 if side<0 else 5
                self.assertTrue(all(ids==[owner,0,0,0] for ids in m.ids[i]))
                self.assertTrue(all(ids==[owner,0,0,0] for ids in m.ids[shell]))
                from goldfish_sculpt import surface
                skin=surface(m.neutral_model()['primitives'][shell])
                for x,y,z in m.vertices[i][::12]:self.assertLess(abs(z-skin(y,x)),.01)
        self.assertGreater(m.rotations('attack',.28)[8][1],m.rotations('attack',.18)[8][1])
    def test_neutral_endpoints_and_early_head_brace(self):
        m=self.m
        for clip in m.clips:
            for t in (0.,1.):
                for matrix in m.matrices(clip,t):self.assertLess(max(abs(a-b) for a,b in zip(matrix,IDENTITY)),1e-7)
        for joint in (1,2,3):
            values=[m.rotations('attack',i/100)[joint][0] for i in range(101)]
            self.assertGreater(values[10],max(values)*.4);self.assertGreater(values[20],max(values)*.9)
        for animation in m.doc['animations']:
            idle=animation['name'].endswith('.idle')
            self.assertEqual(animation['extras']['loopSuggested'],idle)
            self.assertEqual(animation['extras']['cueRelative'],not idle)
            self.assertEqual({c['target']['path'] for c in animation['channels']},{'rotation'})

if __name__=='__main__':unittest.main()
