#!/usr/bin/env python3
"""Independent stored Bulbasaur topology and articulated anatomy checks."""
import math
from pathlib import Path
import unittest
import bulbasaur_glb as author
from sculpt_test_support import assert_closed_models,assert_model_shading
from skin_glb_test_support import IDENTITY
from test_gengar_glb import Model,winding_alignment
ASSET=Path(__file__).resolve().parents[1]/'crates/crystal-voxel-view/models/battle_species/bulbasaur.glb'

class BulbasaurTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):cls.blob=ASSET.read_bytes();cls.m=Model(cls.blob)
    def test_canonical_single_source_and_bounded_schema(self):
        self.assertEqual(self.blob,author.export_bulbasaur())
        self.assertEqual(len(self.m.parts),44)
        self.assertEqual(sum(len(v) for v in self.m.vertices),19587)
        self.assertEqual(sum(len(v)//3 for v in self.m.indices),31896)
        self.assertLess(len(self.blob),1024*1024)
        self.assertLessEqual(len(self.m.doc['accessors']),256)
        self.assertFalse(ASSET.with_suffix('.mesh.json').exists())
    def test_closed_outward_solids_and_smooth_normals(self):
        m=self.m.neutral_model();assert_closed_models(self,{'bulbasaur':m});assert_model_shading(self,{'bulbasaur':m})
    def test_anatomy_and_physical_height(self):
        for prefix,count in [('Ear /',2),('Foreleg /',2),('Hindleg /',2),('Forefoot /',2),('Hindfoot /',2),('Foreclaw /',6),('Hindclaw /',6),('Head spot /',3),('Bulb /',1)]:
            self.assertEqual(sum(n.startswith(prefix) for n in self.m.names),count,prefix)
        points=[p for v in self.m.vertices for p in v]
        self.assertEqual(min(p[1] for p in points),0.)
        self.assertAlmostEqual(max(p[1] for p in points),.95,places=6)
        self.assertGreater(max(p[0] for p in points)-min(p[0] for p in points),.60)
    def test_hierarchy_inverse_binds_and_rigid_face(self):
        m=self.m
        self.assertEqual(m.parents[:-1],[p for _,p,_ in author.JOINTS])
        self.assertEqual([n['name'] for n in m.doc['nodes'][:-1]],['bulbasaur/'+n for n,_,_ in author.JOINTS])
        for matrix in m.matrices():self.assertLess(max(abs(a-b) for a,b in zip(matrix,IDENTITY)),1e-7)
        for ids,weights in zip(m.ids,m.weights):
            for js,ws in zip(ids,weights):
                self.assertAlmostEqual(sum(ws),1.,places=7)
                self.assertTrue(all(j<len(author.JOINTS) for j in js))
        for p,name in enumerate(m.names):
            if name.startswith(('Eye /','Head /','Ear /','Iris /','Head spot /','Eye glint /','Mouth /','Tongue /','Nostril /','Fang /')):
                self.assertTrue(all(js==[2,0,0,0] for js in m.ids[p]))
    def test_planted_soles_floor_and_animation_strain(self):
        m=self.m;fixed=[p for p,n in enumerate(m.names) if n.startswith(('Forefoot /','Hindfoot /','Foreclaw /','Hindclaw /'))]
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
    def test_bulb_is_lobed_and_follows_the_chest_with_delayed_tip(self):
        m=self.m;p=m.names.index('Bulb / continuous six-lobed folded green seed')
        self.assertGreater(len(m.vertices[p]),1000)
        # Twelve alternating bulges/valleys are present around the wide ring;
        # this is a shaped six-lobed seed rather than a sphere with painted lines.
        row=[v for v in m.vertices[p] if abs(v[1]-.71)<.012]
        self.assertEqual(len(row),48)
        cx=sum(v[0] for v in row)/48;cz=sum(v[2] for v in row)/48
        row.sort(key=lambda v:math.atan2(v[0]-cx,v[2]-cz))
        widths=[math.hypot(x-cx,z-cz) for x,y,z in row]
        self.assertGreater(max(widths)-min(widths),.035)
        self.assertEqual(sum(widths[i]>widths[i-1] and widths[i]>widths[(i+1)%48] for i in range(48)),6)
        self.assertEqual(sum(widths[i]<widths[i-1] and widths[i]<widths[(i+1)%48] for i in range(48)),6)
        self.assertGreater(abs(m.rotations('attack',.28)[8][0]),abs(m.rotations('attack',.18)[8][0]))
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
