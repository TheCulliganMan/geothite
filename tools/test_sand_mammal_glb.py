#!/usr/bin/env python3
"""Decode stored canonical skins independently and verify their anatomy/motion."""
import copy
import math
import unittest
from pathlib import Path
import sand_mammal_glb as author
from test_gengar_glb import Model, winding_alignment
from test_ninetales_glb import logical_model
from skin_glb_test_support import IDENTITY
from sculpt_test_support import indexed_ray_intersections, parts_overlap
import test_sand_mammal_sculpt as sculpture_tests


class SandMammalSkinTests(sculpture_tests.SandMammalSculptTests):
    @classmethod
    def setUpClass(cls):
        root = Path(__file__).resolve().parents[1]/'crates/crystal-voxel-view/models/battle_species'
        cls.blobs = {s:(root/(s+'.glb')).read_bytes() for s in author.SPECIES}
        for s,blob in cls.blobs.items():
            if blob!=author.export_sand_mammal(s):raise AssertionError('stored armored mammal differs from canonical exporter: '+s)
        cls.runtime = {s:Model(blob) for s,blob in cls.blobs.items()}
        cls.logical = {s:logical_model(m) for s,m in cls.runtime.items()}
        cls.models = {s:m.neutral_model() for s,m in cls.logical.items()}

    def test_geometry_hierarchy_inverse_binds_and_weights(self):
        for s,m in self.logical.items():
            runtime = self.runtime[s]
            self.assertEqual(sum(n.startswith('Hand claw /') for n in runtime.names),2)
            self.assertFalse(any(n.startswith('Claw /') for n in runtime.names))
            self.assertEqual(author.geometry_digest(self.models[s]),author.REST_DIGESTS[s])
            self.assertEqual(m.parents[:len(author.JOINTS[s])],[p for n,p,point in author.JOINTS[s]])
            for matrix in m.matrices():
                self.assertLess(max(abs(a-b) for a,b in zip(matrix,IDENTITY)),1e-7)
            self.assertEqual({j for ids,ws in zip(m.ids,m.weights) for jj,ww in zip(ids,ws) for j,w in zip(jj,ww) if w},set(range(len(author.JOINTS[s]))))
            for p,ids in zip(self.models[s]['primitives'],m.ids):
                if p['part'].startswith(('Foot /','Toe /')):
                    self.assertTrue(all(jj==[0,0,0,0] for jj in ids))

    def test_clip_floor_feet_winding_strain_and_endpoints(self):
        for s,m in self.logical.items():
            for clip in m.clips:
                for step in range(9):
                    matrices = m.matrices(clip,step/8)
                    for i,title in enumerate(m.names):
                        posed = m.posed(i,matrices);normals = m.posed(i,matrices,True)
                        self.assertGreaterEqual(min(p[1] for p in posed),-1e-7,(s,clip,title,step))
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

    def test_every_control_moves_actual_geometry_and_both_claws_dig(self):
        for s,m in self.logical.items():
            rotations = m.rotations('attack',.44)
            for joint in range(1,len(author.JOINTS[s])):
                isolated = [[0.,0.,0.,1.] for _ in m.doc['nodes']];isolated[joint] = rotations[joint]
                matrices = m.matrices(rotations=isolated)
                self.assertGreater(max(math.dist(p,q) for i,vs in enumerate(m.vertices) for p,q in zip(vs,m.posed(i,matrices))),.0005,(s,joint))
            matrices = m.matrices('attack',.44)
            for side in (-1,1):
                i = next(i for i,n in enumerate(m.names) if n.startswith('Hand /') and n.split()[-1]==str(side))
                self.assertGreater(max(p[1]-q[1] for p,q in zip(m.vertices[i],m.posed(i,matrices))),.025,(s,side))

    def test_mutated_geometry_and_wrong_species_rejected(self):
        for s in author.SPECIES:
            self.assertEqual(author.geometry_digest(author.decode_sand_mammal(self.blobs[s],s)),author.REST_DIGESTS[s])
            mutated = bytearray(self.blobs[s]);mutated[-4] ^= 1
            with self.assertRaises(ValueError):author.decode_sand_mammal(bytes(mutated),s)
            with self.assertRaises(ValueError):author.decode_sand_mammal(self.blobs[s],next(x for x in author.SPECIES if x!=s))
            model = copy.deepcopy(author.authoring_model(s));model['primitives'][0]['positions'][0] += .001
            with self.assertRaises(ValueError):author.export_sand_mammal(s,model)
            model = copy.deepcopy(author.authoring_model(s));model['primitives'][0]['part'] = 'wrong limb'
            with self.assertRaises(ValueError):author.export_sand_mammal(s,model)
        with self.assertRaises(ValueError):author.export_sand_mammal('pikachu')

    def test_posed_roots_and_entire_shoulder_caps_stay_buried(self):
        for s,m in self.logical.items():
            for clip in m.clips:
                for t in (0.,.18,.36,.44,.67,.83):
                    matrices = m.matrices(clip,t)
                    parts = [dict(p,positions=sum(m.posed(i,matrices),[]))
                             for i,p in enumerate(self.models[s]['primitives'])]
                    body = next(p for p in parts if p['part'].startswith('Body /'))
                    shell = next(p for p in parts if p['part'].startswith('Shell /'))
                    for i,p in enumerate(parts):
                        if p['part'].startswith(('Plate /','Quill /')):
                            self.assertTrue(parts_overlap(shell,p),(s,clip,t,p['part']))
                        if p['part'].startswith('Arm /'):
                            # First twenty source vertices form the closed root cap.
                            for point in range(20):
                                x,y,z = p['positions'][point*3:point*3+3]
                                hits = sorted(indexed_ray_intersections(body,x,y))
                                self.assertTrue(any(a+.0002<z<b-.0002 for a,b in zip(hits[::2],hits[1::2])),(s,clip,t,p['part'],point))


if __name__ == '__main__':
    unittest.main()
