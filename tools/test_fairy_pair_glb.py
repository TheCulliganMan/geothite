#!/usr/bin/env python3
"""Independent decoding, fitted anatomy and pose checks for both large fairies."""
import copy
import math
import unittest
from pathlib import Path
import fairy_pair_glb as author
from test_gengar_glb import Model, winding_alignment
from skin_glb_test_support import IDENTITY
from sculpt_test_support import assert_closed_models, assert_model_shading, parts_overlap
from sculpt_test_support import indexed_ray_intersections as rays
import test_clefable_sculpt as clefable_tests
import test_wigglytuff_sculpt as wigglytuff_tests


class FairyPairTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        root = Path(__file__).resolve().parents[1]/'crates/crystal-voxel-view/models/battle_species'
        cls.blobs = {s:(root/(s+'.glb')).read_bytes() for s in author.SPECIES}
        for s,b in cls.blobs.items():
            if b!=author.export_fairy_pair(s):raise AssertionError('stored fairy differs from canonical exporter: '+s)
        cls.runtime = {s:Model(b) for s,b in cls.blobs.items()}
        cls.models = {s:m.neutral_model() for s,m in cls.runtime.items()}

    def test_exact_geometry_schema_budgets_hierarchy_and_inverse_binds(self):
        for s,m in self.runtime.items():
            self.assertEqual(author.geometry_digest(self.models[s]),author.REST_DIGESTS[s])
            self.assertEqual(m.parents[:len(author.JOINTS[s])],[p for n,p,pivot in author.JOINTS[s]])
            self.assertEqual(len(m.parts),32 if s=='clefable' else 24)
            self.assertLessEqual(sum(len(v) for v in m.vertices),20000)
            self.assertLessEqual(sum(len(i)//3 for i in m.indices),40000)
            self.assertLessEqual(len(m.doc['accessors']),256)
            self.assertLess(len(self.blobs[s]),1024*1024)
            for matrix in m.matrices():
                self.assertLess(max(abs(a-b) for a,b in zip(matrix,IDENTITY)),1e-7)

    def test_stored_closed_connected_outward_solids_and_shading(self):
        assert_closed_models(self,self.models)
        assert_model_shading(self,self.models)

    def test_coherent_face_ear_caps_and_actual_sided_controls(self):
        for s,m in self.runtime.items():
            used = {j for ids,weights in zip(m.ids,m.weights) for js,ws in zip(ids,weights) for j,w in zip(js,ws) if w}
            self.assertEqual(used,set(range(len(author.JOINTS[s]))))
            for i,title in enumerate(m.names):
                if title.startswith(('Body /','Belly /','Eye /','Eye border /','Eye white /','Eye glint /','Iris /','Iris sheen /','Mouth /','Tongue /','Cheek /')):
                    self.assertTrue(all(js==[1,0,0,0] for js in m.ids[i]))
                if title.startswith(('Foot /','Toe /')):
                    self.assertTrue(all(js==[0,0,0,0] for js in m.ids[i]))
                if title.startswith(('Ear /','Ear tip /','Ear lining /')):
                    owner = 2+(int(title.split()[-1])>0)
                    self.assertTrue(all(js==[owner,0,0,0] for js in m.ids[i]))
                if title.startswith(('Tuft /','Tuft crease /')):
                    self.assertTrue(all(js==[6,0,0,0] for js in m.ids[i]))
                if title.startswith(('Ear /','Arm /','Wing /')):
                    joint = m.ids[i][0][0]
                    self.assertGreater(sum(v[0] for v in m.vertices[i])*author.JOINTS[s][joint][2][0],0.,(s,title))

    def test_all_controls_move_independently_and_both_arms_swing(self):
        for s,m in self.runtime.items():
            rotations = m.rotations('attack',.34)
            for joint in range(1,len(author.JOINTS[s])):
                isolated = [[0.,0.,0.,1.] for _ in m.doc['nodes']];isolated[joint] = rotations[joint]
                matrices = m.matrices(rotations=isolated)
                self.assertGreater(max(math.dist(p,q) for i,vs in enumerate(m.vertices) for p,q in zip(vs,m.posed(i,matrices))),.0005,(s,joint))
            matrices = m.matrices('attack',.34)
            for side in (-1,1):
                i = next(i for i,n in enumerate(m.names) if n.startswith('Arm /') and n.split()[-1]==str(side))
                self.assertGreater(max(math.dist(p,q) for p,q in zip(m.vertices[i],m.posed(i,matrices))),.025,(s,side))

    def test_planted_feet_floor_winding_strain_and_neutral_endpoints(self):
        for s,m in self.runtime.items():
            for clip in m.clips:
                for step in range(9):
                    matrices = m.matrices(clip,step/8)
                    for i,title in enumerate(m.names):
                        posed = m.posed(i,matrices);normals = m.posed(i,matrices,True)
                        self.assertGreaterEqual(min(v[1] for v in posed),-1e-7,(s,clip,title,step))
                        if title.startswith(('Foot /','Toe /')):
                            self.assertLess(max(math.dist(p,q) for p,q in zip(posed,m.vertices[i])),1e-7)
                        for k in range(0,len(m.indices[i]),3):
                            tri = m.indices[i][k:k+3];cosine,area = winding_alignment(posed,normals,tri)
                            self.assertGreater(cosine,0.,(s,clip,title,step));self.assertGreater(area,1e-11)
                            for a,b in zip(tri,tri[1:]+tri[:1]):
                                ratio = math.dist(posed[a],posed[b])/math.dist(m.vertices[i][a],m.vertices[i][b])
                                self.assertTrue(.75<ratio<1.3,(s,clip,title,step,ratio))
                    if step in (0,8):
                        for matrix in matrices:
                            self.assertLess(max(abs(a-b) for a,b in zip(matrix,IDENTITY)),1e-7)

    def test_occupied_volume_roots_survive_actual_clips(self):
        for s,m in self.runtime.items():
            for clip in m.clips:
                for t in (0.,.18,.34,.46,.67,.83):
                    matrices = m.matrices(clip,t)
                    parts = [dict(p,positions=sum(m.posed(i,matrices),[])) for i,p in enumerate(self.models[s]['primitives'])]
                    body = next(p for p in parts if p['part'].startswith('Body /'))
                    for p in parts:
                        if p['part'].startswith(('Arm /','Ear /','Wing /','Tail /','Tuft /','Foot /')):
                            self.assertTrue(parts_overlap(body,p),(s,clip,t,p['part']))
                        if p['part'].startswith(('Hand claw /','Hand finger /')):
                            side = p['part'].split()[-2]
                            arm = next(q for q in parts if q['part'].startswith('Arm /') and q['part'].split()[-1]==side)
                            self.assertTrue(parts_overlap(arm,p),(s,clip,t,p['part']))

    def test_fitted_vertices_edges_and_centres_clear_actual_posed_parents(self):
        for s,m in self.runtime.items():
            helper = clefable_tests.ClefableTests() if s=='clefable' else wigglytuff_tests.WigglytuffTests()
            helper.ps = self.models[s]['primitives'];original = list(helper.pairs())
            for clip in m.clips:
                for t in (0.,.34,.67):
                    matrices = m.matrices(clip,t)
                    helper.ps = [dict(p,positions=sum(m.posed(i,matrices),[])) for i,p in enumerate(self.models[s]['primitives'])]
                    for (old,_),(child,parent) in zip(original,helper.pairs()):
                        # Cast along the parent's transformed authored front,
                        # rather than world Z. A rigid rear cap can project past
                        # the ear's world-XY silhouette without losing support.
                        parent_index = m.names.index(parent['part'])
                        child_index = m.names.index(child['part'])
                        owner = m.ids[parent_index][0][0]
                        for index in (parent_index,child_index):
                            self.assertTrue(all(js==[owner,0,0,0] for js in m.ids[index]))
                            self.assertTrue(all(ws==[1.,0.,0.,0.] for ws in m.weights[index]))
                        matrix = matrices[owner]
                        rear = ' rear ' in child['part']
                        def carrier_frame(part):
                            positions = []
                            for i in range(0,len(part['positions']),3):
                                point = part['positions'][i:i+3]
                                if rear:point = [-point[0],point[1],-point[2]]
                                delta = [point[k]-matrix[12+k] for k in range(3)]
                                local = [sum(matrix[axis*4+k]*delta[k] for k in range(3)) for axis in range(3)]
                                if rear:local = [-local[0],local[1],-local[2]]
                                positions.extend(local)
                            return dict(part,positions=positions)
                        child,parent = carrier_frame(child),carrier_frame(parent)
                        vs = [child['positions'][i:i+3] for i in range(0,len(child['positions']),3)]
                        oldvs = [old['positions'][i:i+3] for i in range(0,len(old['positions']),3)]
                        for k in range(0,len(child['indices']),3):
                            ids = child['indices'][k:k+3];a,b,c = [oldvs[i] for i in ids]
                            if (b[0]-a[0])*(c[1]-a[1])-(b[1]-a[1])*(c[0]-a[0])<=1e-12:continue
                            tri = [vs[i] for i in ids]
                            for weights in ((1/3,1/3,1/3),(.5,.5,0),(.5,0,.5),(0,.5,.5),(1,0,0),(0,1,0),(0,0,1)):
                                x,y,z = [sum(weights[j]*tri[j][axis] for j in range(3)) for axis in range(3)]
                                hits = rays(parent,x,y);self.assertTrue(hits,(s,child['part']))
                                gap = z-max(hits)
                                self.assertGreater(gap,.0001,(s,clip,t,child['part'],gap))
                                self.assertLess(gap,.017,(s,clip,t,child['part'],gap))

    def test_canonical_reader_roundtrip_and_strict_mutation_rejection(self):
        for s,b in self.blobs.items():
            self.assertEqual(author.geometry_digest(author.decode_fairy_pair(b,s)),author.REST_DIGESTS[s])
            mutated = bytearray(b);mutated[-4] ^= 1
            with self.assertRaises(ValueError):author.decode_fairy_pair(bytes(mutated),s)
            model = copy.deepcopy(author.authoring_model(s));model['primitives'][0]['positions'][0] += .001
            with self.assertRaises(ValueError):author.export_fairy_pair(s,model)
            with self.assertRaises(ValueError):author.decode_fairy_pair(b,next(n for n in author.SPECIES if n!=s))


if __name__ == '__main__':unittest.main()
