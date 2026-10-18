#!/usr/bin/env python3
"""Independent stored Oddish topology and articulated anatomy checks."""
import math
from pathlib import Path
import unittest
import oddish_glb as author
from sculpt_test_support import assert_closed_models,assert_model_shading,crossed_faces
from skin_glb_test_support import IDENTITY
from test_gengar_glb import Model,winding_alignment
ASSET=Path(__file__).resolve().parents[1]/'crates/crystal-voxel-view/models/battle_species/oddish.glb'

class OddishTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):cls.blob=ASSET.read_bytes();cls.m=Model(cls.blob)
    def test_canonical_single_source_and_bounded_schema(self):
        self.assertEqual(self.blob,author.export_oddish())
        self.assertEqual(len(self.m.parts),19)
        self.assertEqual(sum(len(v) for v in self.m.vertices),8845)
        self.assertEqual(sum(len(v)//3 for v in self.m.indices),14620)
        self.assertLess(len(self.blob),1024*1024)
        self.assertLessEqual(len(self.m.doc['accessors']),256)
        self.assertFalse(ASSET.with_suffix('.mesh.json').exists())
    def test_closed_outward_solids_and_smooth_normals(self):
        m=self.m.neutral_model();assert_closed_models(self,{'oddish':m});assert_model_shading(self,{'oddish':m})
    def test_leaf_caps_do_not_cross_at_rest_or_through_animation(self):
        m=self.m
        for clip in m.clips:
            for t in (0.,.18,.34,.52,.76,1.):
                mats=m.matrices(clip,t)
                for i,name in enumerate(m.names):
                    if name.startswith('Leaf '):
                        part=dict(positions=[v for point in m.posed(i,mats) for v in point],indices=m.indices[i])
                        self.assertIsNone(crossed_faces(part),(clip,t,name))
    def test_legacy_concave_fan_negative_control_crosses_leaf_walls(self):
        from pitcher_sculpt import leaf
        from oddish_sculpt import LEAVES,GREEN
        for j,(start,end,width) in enumerate(LEAVES):
            blade,_=leaf('legacy',start,end,width,GREEN,bend=.035 if j%2==0 else -.012)
            self.assertIsNotNone(crossed_faces(blade),j)
    def test_cap_fix_preserves_boundary_positions_and_other_anatomy(self):
        from unittest.mock import patch
        import oddish_sculpt
        from pitcher_sculpt import leaf
        def legacy(*args,**kwargs):
            kwargs['solid_caps']=False
            return leaf(*args,**kwargs)
        with patch.object(oddish_sculpt,'leaf',legacy):
            old=oddish_sculpt.sculpture()
            prior=Model(author.export_oddish())
        current=self.m.neutral_model()
        for before,after in zip(old['primitives'],current['primitives']):
            def points(p):
                import struct
                values=struct.unpack('<'+'f'*len(p['positions']),struct.pack('<'+'f'*len(p['positions']),*p['positions']))
                return set(zip(values[::3],values[1::3],values[2::3]))
            self.assertEqual(points(before),points(after),before['part'])
            if not before['part'].endswith('cupped blade'):
                self.assertEqual(author.geometry_digest({'primitives':[before]}),author.geometry_digest({'primitives':[after]}))
        self.assertEqual(prior.doc['nodes'],self.m.doc['nodes'])
        self.assertEqual(prior.doc['materials'],self.m.doc['materials'])
        for i in range(len(self.m.parts)):
            def bindings(model):
                return {(tuple(p),tuple(js),tuple(ws)) for p,js,ws in zip(model.vertices[i],model.ids[i],model.weights[i])}
            self.assertEqual(bindings(prior),bindings(self.m),self.m.names[i])
        from skin_glb_test_support import values
        self.assertEqual(prior.doc['animations'],self.m.doc['animations'])
        for animation in self.m.doc['animations']:
            for sampler in animation['samplers']:
                for field in ('input','output'):
                    self.assertEqual(values(prior.doc,prior.binary,sampler[field]),values(self.m.doc,self.m.binary,sampler[field]))
    def test_anatomy_and_physical_height(self):
        for prefix,count in [('Foot /',2),('Eye /',2),('Eye glint /',2),('Mouth /',1),('Tongue /',1),('Leaf ',10)]:
            self.assertEqual(sum(n.startswith(prefix) for n in self.m.names),count,prefix)
        points=[p for v in self.m.vertices for p in v]
        self.assertEqual(min(p[1] for p in points),0.)
        self.assertAlmostEqual(max(p[1] for p in points),1.00,places=6)
        self.assertGreater(max(p[0] for p in points)-min(p[0] for p in points),.60)
        leaves=[v for part,name in enumerate(self.m.names) if name.startswith('Leaf ') for v in self.m.vertices[part]]
        self.assertGreater(max(v[2] for v in leaves)-min(v[2] for v in leaves),.50, 'leaf fan must remain broad from a side view')
    def test_hierarchy_inverse_binds_and_rigid_face(self):
        m=self.m
        self.assertEqual(m.parents[:-1],[p for _,p,_ in author.JOINTS])
        self.assertEqual([n['name'] for n in m.doc['nodes'][:-1]],['oddish/'+n for n,_,_ in author.JOINTS])
        for matrix in m.matrices():self.assertLess(max(abs(a-b) for a,b in zip(matrix,IDENTITY)),1e-7)
        for ids,weights in zip(m.ids,m.weights):
            for js,ws in zip(ids,weights):
                self.assertAlmostEqual(sum(ws),1.,places=7)
                self.assertTrue(all(j<len(author.JOINTS) for j in js))
        for p,name in enumerate(m.names):
            if name.startswith(('Eye /','Eye glint /','Mouth /','Tongue /')):
                self.assertTrue(all(js==[1,0,0,0] for js in m.ids[p]))
    def test_planted_soles_floor_and_animation_strain(self):
        m=self.m;fixed=[(p,i) for p,v in enumerate(m.vertices) if m.names[p].startswith('Foot /') for i,point in enumerate(v)]
        for clip in m.clips:
            for step in range(17):
                mats=m.matrices(clip,step/16)
                for p,i in fixed:self.assertEqual(m.posed(p,mats)[i],m.vertices[p][i])
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
    def test_five_leaf_chains_and_delayed_tips(self):
        m=self.m
        for p,name in enumerate(m.names):
            if name.startswith('Leaf '):
                j=int(name.split()[1]);base=2+2*j
                self.assertTrue(all(all(bone in (0,base,base+1) for bone in ids) for ids in m.ids[p]))
                self.assertTrue(any(base in ids for ids in m.ids[p]))
                self.assertTrue(any(base+1 in ids for ids in m.ids[p]))
        self.assertGreater(m.rotations('attack',.28)[3][0],m.rotations('attack',.18)[3][0])
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
