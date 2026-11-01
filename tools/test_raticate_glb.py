#!/usr/bin/env python3
"""Stored Raticate geometry preservation and eight-joint anatomy checks."""
import math
from pathlib import Path
import unittest
import raticate_glb as author
from skin_glb_test_support import IDENTITY
from sculpt_test_support import assert_closed_models,assert_model_shading
from test_gengar_glb import Model,winding_alignment
ASSET=Path(__file__).resolve().parents[1]/'crates/crystal-voxel-view/models/battle_species/raticate.glb'

class RaticateTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):cls.blob=ASSET.read_bytes();cls.m=Model(cls.blob)
    def test_preserved_geometry_and_bounded_shared_accessors(self):
        neutral=author.decode_raticate(self.blob)
        self.assertEqual(author.geometry_digest(neutral),'f6043d52ed1588580d3e4c8f44480d8d3cb2b8b2f268e6ccd7220287d76edd0d')
        self.assertEqual(author.export_raticate(neutral),self.blob)
        self.assertEqual(len(self.m.parts),63)
        self.assertEqual(sum(len(v) for v in self.m.vertices),5642)
        self.assertEqual(sum(len(v)//3 for v in self.m.indices),7764)
        self.assertEqual(len(self.m.doc['accessors']),247)
        self.assertLess(len(self.blob),1024*1024)
        self.assertFalse(ASSET.with_suffix('.mesh.json').exists())
        indices=[p['indices'] for p in self.m.parts]
        self.assertLess(len(set(indices)),len(indices),'identical indices share accessors without merging parts')
    def test_closed_solids_shading_and_floor(self):
        neutral=self.m.neutral_model();assert_closed_models(self,{'raticate':neutral});assert_model_shading(self,{'raticate':neutral})
        points=[p for v in self.m.vertices for p in v]
        self.assertEqual(min(v[1] for v in points),0.)
        self.assertAlmostEqual(max(v[1] for v in points),.96,places=6)
    def test_anatomical_groups_and_coherent_fitted_face(self):
        m=self.m
        for prefix,count in [('Broad upper chisel incisor',2),('Short lower chisel incisor',2),
            ('Tapered facial whisker',6),('Flared cupped rat ear',2),('Recessed ear interior',2),
            ('Long splayed rat foot',2),('Long rat toe',6),('Rat toe claw',6),('Small grasping forepaw',2),
            ('Forepaw claw',6),('Long tapered bare rat tail',1),('Subtle tail segmentation',3)]:
            self.assertEqual(sum(n.startswith(prefix) for n in m.names),count,prefix)
        for i,name in enumerate(m.names):
            if name.startswith(('Slanted almond eye','Focused small rat pupil','Natural upper eyelid',
                'Dark open mouth cavity','Lower cream jaw','Lower mouth tongue','Cream whisker cheek',
                'Broad upper chisel incisor','Short lower chisel incisor','Tapered facial whisker','Small split rat nose','Nose philtrum')):
                self.assertTrue(all(ids==[1,0,0,0] for ids in m.ids[i]),name)
    def test_hierarchy_weights_and_inverse_binds(self):
        m=self.m
        self.assertEqual(m.parents[:-1],[p for _,p,_ in author.JOINTS])
        self.assertEqual([n['name'] for n in m.doc['nodes'][:-1]],['raticate/'+n for n,_,_ in author.JOINTS])
        for matrix in m.matrices():self.assertLess(max(abs(a-b) for a,b in zip(matrix,IDENTITY)),1e-7)
        for ids,weights in zip(m.ids,m.weights):
            for js,ws in zip(ids,weights):
                self.assertAlmostEqual(sum(ws),1.,places=7)
                self.assertTrue(all(j<len(author.JOINTS) for j in js))
    def test_fixed_feet_winding_floor_and_strain_through_clips(self):
        m=self.m
        for clip in m.clips:
            for step in range(17):
                matrices=m.matrices(clip,step/16)
                for p,name in enumerate(m.names):
                    posed=m.posed(p,matrices)
                    if name.startswith(('Long splayed rat foot','Long rat toe','Rat toe claw')):
                        self.assertEqual(posed,m.vertices[p])
                    self.assertGreaterEqual(min(v[1] for v in posed),-1e-7,(clip,step,name))
                    if step%4==0:
                        normals=m.posed(p,matrices,True)
                        for i in range(0,len(m.indices[p]),3):
                            tri=m.indices[p][i:i+3];cosine,area=winding_alignment(posed,normals,tri)
                            self.assertGreater(cosine,0.,(clip,step,name,i));self.assertGreater(area,1e-12)
                            for a,b in zip(tri,tri[1:]+tri[:1]):
                                ratio=math.dist(posed[a],posed[b])/math.dist(m.vertices[p][a],m.vertices[p][b])
                                self.assertTrue(.7<ratio<1.3,(clip,step,name,ratio))
    def test_forepaw_claws_ear_bowls_and_segmented_tail_keep_owners(self):
        m=self.m
        for p,name in enumerate(m.names):
            if name.startswith(('Short bent forearm','Small grasping forepaw','Three curled forepaw digits','Forepaw claw')):
                mean_x=sum(v[0] for v in m.vertices[p])/len(m.vertices[p]);owner=4 if mean_x<-.021 else 5
                self.assertTrue(all(ids==[owner,0,0,0] for ids in m.ids[p]))
            if name.startswith(('Flared cupped rat ear','Recessed ear interior')):
                mean_x=sum(v[0] for v in m.vertices[p])/len(m.vertices[p]);owner=2 if mean_x<-.021 else 3
                self.assertTrue(all(ids==[owner,0,0,0] for ids in m.ids[p]))
            if name.startswith(('Long tapered bare rat tail','Subtle tail segmentation')):
                for point,ids,ws in zip(m.vertices[p],m.ids[p],m.weights[p]):
                    js,weights=author.skin_weights(name,point);q=author.quantized_weights(weights)
                    self.assertEqual(ids,[j if w else 0 for j,w in zip(js,q)])
                    self.assertEqual([round(w*255) for w in ws],q)
    def test_neutral_endpoints_and_early_brace(self):
        m=self.m
        for clip in m.clips:
            for t in (0.,1.):
                for matrix in m.matrices(clip,t):self.assertLess(max(abs(a-b) for a,b in zip(matrix,IDENTITY)),1e-7)
        for joint in (1,4,5):
            values=[m.rotations('attack',i/100)[joint][0] for i in range(101)]
            self.assertGreater(values[10],max(values)*.4);self.assertGreater(values[20],max(values)*.9)
        self.assertGreater(m.rotations('attack',.28)[7][1],m.rotations('attack',.18)[7][1])
        for animation in m.doc['animations']:
            idle=animation['name'].endswith('.idle')
            self.assertEqual(animation['extras']['loopSuggested'],idle)
            self.assertEqual(animation['extras']['cueRelative'],not idle)
            self.assertEqual({c['target']['path'] for c in animation['channels']},{'rotation'})

if __name__=='__main__':unittest.main()
