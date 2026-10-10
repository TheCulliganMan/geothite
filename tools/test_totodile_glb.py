#!/usr/bin/env python3
"""Decoded-file proof of the bounded Totodile candidate; no Cargo or Blender.

Set TOTODILE_MESH_SOURCE for the optional original-source byte comparison.
The pinned geometry digest remains required when the retired JSON is absent.
"""
import bisect
import copy
import json
import math
import os
from pathlib import Path
import struct
import unittest

import totodile_glb as author
from skin_glb_test_support import (IDENTITY, decode, encode, matmul, raw_accessor, rows,
                             slerp, transform, trs, values)

HERE = Path(__file__).resolve().parent
ASSET = Path(os.environ.get('TOTODILE_GLB', HERE.parent / 'crates/crystal-voxel-view/models/actor_props/battle_totodile.glb'))


class Model:
    def __init__(self, blob):
        self.doc, self.binary = decode(blob)
        self.parts = self.doc['meshes'][0]['primitives']
        self.vertices, self.normals, self.ids, self.weights, self.names = [], [], [], [], []
        for part in self.parts:
            attrs = part['attributes']
            self.vertices.append(rows(values(self.doc, self.binary, attrs['POSITION']), 3))
            self.normals.append(rows(values(self.doc, self.binary, attrs['NORMAL']), 3))
            self.ids.append(rows(values(self.doc, self.binary, attrs['JOINTS_0']), 4))
            self.weights.append(rows(values(self.doc, self.binary, attrs['WEIGHTS_0']), 4))
            self.names.append(part['extras']['part'])
        self.inverse = rows(values(self.doc, self.binary, self.doc['skins'][0]['inverseBindMatrices']), 16)
        self.clips = {}
        for clip in self.doc['animations']:
            tracks = []
            for channel in clip['channels']:
                sampler = clip['samplers'][channel['sampler']]
                tracks.append((channel['target']['node'], values(self.doc, self.binary, sampler['input']),
                               rows(values(self.doc, self.binary, sampler['output']), 4)))
            self.clips[clip['name'].split('.')[-1]] = tracks

    def rotations(self, clip=None, fraction=0.):
        result = [[0., 0., 0., 1.] for _ in author.JOINTS]
        if clip is not None:
            for node, times, keys in self.clips[clip]:
                time = fraction*times[-1]
                lo = max(0, min(len(times)-2, bisect.bisect_right(times, time)-1))
                result[node] = slerp(keys[lo], keys[lo+1], (time-times[lo])/(times[lo+1]-times[lo]))
        return result

    def matrices(self, clip=None, fraction=0., rotations=None):
        rotations = rotations if rotations is not None else self.rotations(clip, fraction)
        globals_, matrices = [], []
        for i, (_, parent, _) in enumerate(author.JOINTS):
            local = trs(self.doc['nodes'][i]['translation'], rotations[i])
            global_ = matmul(globals_[parent], local) if parent is not None else local
            globals_.append(global_)
            matrices.append(matmul(global_, self.inverse[i]))
        return matrices

    def point(self, part, index, matrices, normal=False, full_weights=False):
        p = (self.normals if normal else self.vertices)[part][index]
        ids, ws = self.ids[part][index], self.weights[part][index]
        if full_weights:
            ids, ws = author.skin_weights(self.names[part], self.vertices[part][index])
        out = [0., 0., 0.]
        for joint, weight in zip(ids, ws):
            if weight:
                posed = transform(matrices[joint], p, 0. if normal else 1.)
                out = [a+weight*b for a, b in zip(out, posed)]
        return out

    def posed(self, part, matrices):
        return [self.point(part, i, matrices) for i in range(len(self.vertices[part]))]

    def neutral_model(self):
        parts = []
        for i, part in enumerate(self.parts):
            parts.append({'part': self.names[i], 'positions': sum(self.vertices[i], []),
                'normals': sum(self.normals[i], []),
                'indices': values(self.doc, self.binary, part['indices']),
                'base_color': self.doc['materials'][part['material']]['pbrMetallicRoughness']['baseColorFactor']})
        return {'name': 'battle_totodile', 'version': 1,
                'coordinate_system': author.COORDINATES, 'primitives': parts}


class TotodileTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.blob = ASSET.read_bytes()
        cls.model = Model(cls.blob)

    def test_original_geometry_bits_colors_names_and_index_order(self):
        model = self.model.neutral_model()
        self.assertEqual(author.geometry_digest(model), author.REST_DIGEST)
        self.assertEqual(self.model.names, list(author.PARTS))
        self.assertEqual(sum(len(p) for p in self.model.vertices), 5714)
        self.assertEqual(sum(len(p['indices'])//3 for p in model['primitives']), 6374)
        source_path = os.environ.get('TOTODILE_MESH_SOURCE')
        if source_path:
            source = json.loads(Path(source_path).read_bytes())
            for p, q in zip(model['primitives'], source['primitives']):
                for key, component in (('positions', 5126), ('normals', 5126), ('base_color', 5126), ('indices', 5125)):
                    self.assertEqual(author.packed(p[key], component), author.packed(q[key], component))
        points = sum(self.model.vertices, [])
        self.assertEqual(min(p[1] for p in points), 0.)
        self.assertEqual(struct.pack('<f', max(p[1] for p in points)), struct.pack('<f', 1.02))

    def test_standard_bounded_schema_one_skin_and_identity_root(self):
        doc = self.model.doc
        self.assertLess(len(self.blob), 300000)
        self.assertEqual(doc['asset']['version'], '2.0')
        self.assertFalse(doc.get('extensionsRequired'))
        self.assertEqual((len(doc['meshes']), len(doc['skins']), len(doc['scenes']), len(doc['buffers'])), (1, 1, 1, 1))
        self.assertEqual(doc['scenes'][0]['nodes'], [0, 8])
        self.assertEqual(doc['nodes'][0]['translation'], [0., 0., 0.])
        self.assertEqual(doc['nodes'][8], {'name': 'totodile', 'mesh': 0, 'skin': 0})
        self.assertEqual(doc['skins'][0]['joints'], list(range(8)))
        self.assertEqual(len(doc['materials']), 6)
        self.assertLessEqual(len(author.JOINTS), 32)
        self.assertNotIn('uri', doc['buffers'][0])
        for view in doc['bufferViews']:
            self.assertEqual(view['byteOffset'] % 4, 0)
            self.assertLessEqual(view['byteOffset']+view['byteLength'], doc['buffers'][0]['byteLength'])
        for part in self.model.parts:
            self.assertEqual(set(part['attributes']), {'POSITION', 'NORMAL', 'JOINTS_0', 'WEIGHTS_0'})
            for key in ('JOINTS_0', 'WEIGHTS_0'):
                accessor = doc['accessors'][part['attributes'][key]]
                self.assertEqual((accessor['type'], accessor['componentType']), ('VEC4', 5121))
                self.assertEqual(accessor.get('normalized', False), key == 'WEIGHTS_0')

    def test_all_influences_normalized_bounded_and_nonzero_joints_used(self):
        used = set()
        for part, ids, weights in zip(self.model.parts, self.model.ids, self.model.weights):
            raw = raw_accessor(self.model.doc, self.model.binary, part['attributes']['WEIGHTS_0'])
            self.assertTrue(all(sum(raw[i:i+4]) == 255 for i in range(0, len(raw), 4)))
            for js, ws in zip(ids, weights):
                self.assertAlmostEqual(sum(ws), 1.)
                self.assertTrue(all(type(j) is int and 0 <= j < 8 for j in js))
                self.assertTrue(all(math.isfinite(w) and 0. <= w <= 1. for w in ws))
                for j, w in zip(js, ws):
                    if w: used.add(j)
                    else: self.assertEqual(j, 0)
        self.assertEqual(used, set(range(8)))

    def test_neutral_binds_and_rotation_only_source_clock_clips(self):
        self.assertEqual(set(self.model.clips), {'idle', 'attack', 'hit'})
        for clip, (_, duration, samples) in zip(self.model.doc['animations'], author.CLIPS):
            name = clip['name'].split('.')[-1]
            self.assertEqual(clip['extras']['loopSuggested'], name == 'idle')
            self.assertEqual(clip['extras']['cueRelative'], name != 'idle')
            self.assertEqual(len(clip['channels']), 7)
            for channel in clip['channels']:
                self.assertEqual(channel['target']['path'], 'rotation')
                self.assertIn(channel['target']['node'], range(1, 8))
                self.assertEqual(clip['samplers'][channel['sampler']]['interpolation'], 'LINEAR')
            for _, times, keys in self.model.clips[name]:
                self.assertEqual(len(times), samples)
                self.assertAlmostEqual(times[-1], duration, places=6)
                self.assertEqual(times, sorted(set(times)))
                self.assertEqual(keys[0], [0., 0., 0., 1.])
                self.assertEqual(keys[-1], keys[0])
                self.assertTrue(all(abs(sum(v*v for v in q)-1.) < 2e-7 for q in keys))
            for t in (0., 1.):
                for matrix in self.model.matrices(name, t):
                    self.assertLess(max(abs(a-b) for a, b in zip(matrix, IDENTITY)), 3e-8)
        matrices = self.model.matrices()
        for i, points in enumerate(self.model.vertices):
            for j, point in enumerate(points):
                self.assertLess(math.dist(point, self.model.point(i, j, matrices)), 3e-8)
                self.assertLess(math.dist(self.model.normals[i][j], self.model.point(i, j, matrices, normal=True)), 3e-8)

    def test_planted_feet_and_all_six_claws_stay_fixed(self):
        probes = [(i, j) for i, points in enumerate(self.model.vertices)
                  for j, p in enumerate(points)
                  if p[1] <= .145 and p[2] >= -.075]
        self.assertGreater(len(probes), 300)
        self.assertEqual(len([n for n in self.model.names if n.startswith('Ivory toe claw')]), 6)
        for i, j in probes:
            self.assertEqual(self.model.ids[i][j], [0, 0, 0, 0])
            self.assertEqual(self.model.weights[i][j], [1., 0., 0., 0.])
        for clip in self.model.clips:
            for step in range(17):
                mats = self.model.matrices(clip, step/16)
                for i, j in probes[::4]:
                    self.assertEqual(self.model.point(i, j, mats), self.model.vertices[i][j])

    def test_face_jaw_cavity_and_pivots_follow_actual_anatomy(self):
        face_parts = [0, 1, 3, 4, 7, 8, 15, 16, 23, 24, 25, 26]
        for i in face_parts+[5]:
            joint = 3 if i == 5 else 2
            self.assertTrue(all(ids == [joint, 0, 0, 0] for ids in self.model.ids[i]))
            self.assertTrue(all(ws == [1., 0., 0., 0.] for ws in self.model.weights[i]))
        self.assertTrue(all(set(j for j, w in zip(js, ws) if w) <= {2, 3}
                            for js, ws in zip(self.model.ids[2], self.model.weights[2])))
        cavity_ids = self.model.ids[2]
        self.assertGreater(cavity_ids.count([2, 0, 0, 0]), 20)
        self.assertGreater(cavity_ids.count([3, 0, 0, 0]), 20)
        for clip in self.model.clips:
            for t in (.19, .43, .66, .9):
                mats = self.model.matrices(clip, t)
                for i, (_, parent, pivot) in enumerate(author.JOINTS):
                    if parent is None: continue
                    self.assertLess(math.dist(transform(mats[i], pivot), transform(mats[parent], pivot)), 1e-7)
                for i in face_parts+[5]:
                    posed = self.model.posed(i, mats)
                    self.assertAlmostEqual(math.dist(posed[0], posed[-1]),
                                           math.dist(self.model.vertices[i][0], self.model.vertices[i][-1]), places=7)

    def test_shared_continuous_body_inlay_and_plate_field(self):
        signatures = {}
        for i in (6, 9, 10, 11, 12, 13, 14):
            for p, ids, ws in zip(self.model.vertices[i], self.model.ids[i], self.model.weights[i]):
                js, full = author.body_weights(p)
                qw = author.quantized_weights(full)
                self.assertEqual(ids, [j if w else 0 for j, w in zip(js, qw)])
                self.assertEqual(ws, [w/255. for w in qw])
                signature = (tuple(ids), tuple(ws))
                if tuple(p) in signatures: self.assertEqual(signature, signatures[tuple(p)])
                signatures[tuple(p)] = signature
        for joint in (4, 5):
            self.assertGreater(sum(any(j == joint and w > .75 for j, w in zip(js, ws))
                for js, ws in zip(self.model.ids[6], self.model.weights[6])), 40)

    def test_tail_yaw_retains_low_tail_height(self):
        probes = [(i, j) for i in (6, 9, 12) for j, (ids, ws) in
                  enumerate(zip(self.model.ids[i], self.model.weights[i]))
                  if all(not w or k in (6, 7) for k, w in zip(ids, ws))]
        self.assertGreater(len(probes), 100)
        for clip in self.model.clips:
            for step in range(17):
                mats = self.model.matrices(clip, step/16)
                for i, j in probes:
                    self.assertAlmostEqual(self.model.point(i, j, mats)[1], self.model.vertices[i][j][1], places=7)

    def test_periodic_idle_tangent_and_reaction_settle(self):
        for clip in self.model.clips:
            eps = 1e-5
            first, last = author.pose_angles(clip, eps), author.pose_angles(clip, 1.-eps)
            for a, b in zip(first, last):
                for va, vb in zip(a, b):
                    if clip == 'idle': self.assertLess(abs((va+vb)/eps), .0001)
                    else: self.assertLess(max(abs(va), abs(vb))/eps, .0003)
        # The actually stored LINEAR keys also have a bounded seam tangent
        # mismatch, rather than merely checking the ideal generating curve.
        for _, times, keys in self.model.clips['idle']:
            first = [(b-a)/(times[1]-times[0]) for a, b in zip(keys[0], keys[1])]
            last = [(b-a)/(times[-1]-times[-2]) for a, b in zip(keys[-2], keys[-1])]
            self.assertLess(max(abs(a-b) for a, b in zip(first, last)), .0031)

    def test_attack_braces_forward_at_source_cue_without_reverse_windup(self):
        # Decode stored keys: the move cue starts at source animation frame
        # zero, so its first quarter must already show the crocodile's brace.
        m = self.model
        for step in range(1, 26):
            q = m.rotations('attack', step / 100)
            self.assertGreater(q[1][0], 0., step)
            self.assertGreater(q[2][0], 0., step)
        peak = max(m.rotations('attack', t / 100)[1][0] for t in range(101))
        self.assertGreater(m.rotations('attack', .10)[1][0], peak * .45)
        self.assertGreater(m.rotations('attack', .20)[1][0], peak * .90)
        # The jaw opens before the brace peaks; it is not a delayed second
        # action after the authored source emission.
        jaw_peak = max(m.rotations('attack', t / 100)[3][0] for t in range(101))
        self.assertGreater(m.rotations('attack', .10)[3][0], jaw_peak * .70)

    def test_arbitrary_crossfades_preserve_contacts_and_rigid_details(self):
        pairs = [('idle', .25, 'attack', .43), ('attack', .43, 'hit', .19),
                 ('hit', .19, 'idle', .75), ('attack', .66, 'attack', .18)]
        probes = [(i, j) for i in (2, 5, 6, 9, 12, 14, 15, 24)
                  for j in range(0, len(self.model.vertices[i]), 31)]
        for a, ta, b, tb in pairs:
            qa, qb = self.model.rotations(a, ta), self.model.rotations(b, tb)
            previous = None
            for step in range(49):
                t = step/48
                q = [slerp(x, y, t) for x, y in zip(qa, qb)]
                mats = self.model.matrices(rotations=q)
                self.assertEqual(q[0], [0., 0., 0., 1.])
                self.assertEqual(mats[0], IDENTITY)
                points = [self.model.point(i, j, mats) for i, j in probes]
                self.assertTrue(all(math.isfinite(v) for p in points for v in p))
                self.assertGreaterEqual(min(p[1] for p in points), -1e-7)
                if previous:
                    self.assertLess(max(math.dist(p, q) for p, q in zip(points, previous)), .0025)
                previous = points
                for i in (5, 15, 24):
                    self.assertAlmostEqual(math.dist(self.model.point(i, 0, mats), self.model.point(i, -1, mats)),
                        math.dist(self.model.vertices[i][0], self.model.vertices[i][-1]), places=7)
                for i in range(17, 23):
                    self.assertEqual(self.model.point(i, 0, mats), self.model.vertices[i][0])
                if step in (0, 48):
                    expected = self.model.matrices(a, ta) if step == 0 else self.model.matrices(b, tb)
                    self.assertLess(max(abs(x-y) for m, n in zip(mats, expected) for x, y in zip(m, n)), 1e-7)

    def test_compact_weight_error_stays_below_one_ten_thousandth(self):
        probes = [(i, j) for i in (2, 6, 9, 10, 11, 12, 13, 14)
                  for j in range(0, len(self.model.vertices[i]), 3)
                  if max(self.model.weights[i][j]) < 1.]
        self.assertGreater(len(probes), 250)
        worst = 0.
        for clip in self.model.clips:
            for t in (.125, .19, .25, .43, .5, .66, .75):
                mats = self.model.matrices(clip, t)
                for i, j in probes:
                    worst = max(worst, math.dist(self.model.point(i, j, mats),
                                                self.model.point(i, j, mats, full_weights=True)))
        self.assertLess(worst, .0001)

    def test_reproducible_export_and_reader_rejects_mutation(self):
        self.assertEqual(author.export_totodile(self.model.neutral_model()), self.blob)
        self.assertEqual(author.export_totodile(author.decode_totodile(self.blob)), self.blob)
        model = self.model.neutral_model()
        model['primitives'][0]['part'] = 'Wrong anatomy'
        with self.assertRaisesRegex(ValueError, 'anatomy name'):
            author.export_totodile(model)
        model['primitives'][0]['part'] = author.PARTS[0]
        model['primitives'][0]['positions'][0] += .0001
        with self.assertRaisesRegex(ValueError, 'unreviewed neutral'):
            author.export_totodile(model)
        for edit in (
            lambda d: d['nodes'][1].update(translation=[0., 1., 0.]),
            lambda d: d['nodes'][8].update(scale=[2., 2., 2.]),
            lambda d: d['animations'][0]['samplers'][0].update(interpolation='STEP'),
            lambda d: d['meshes'][0]['primitives'][0].update(targets=[]),
            lambda d: d['buffers'][0].update(uri='external.bin'),
            lambda d: d['skins'][0]['joints'].reverse(),
            lambda d: d['accessors'][1].update(count=100000000),
        ):
            doc = copy.deepcopy(self.model.doc)
            edit(doc)
            with self.assertRaises(ValueError): author.decode_totodile(encode(doc, self.model.binary))
        for blob in (self.blob[:100], self.blob+b'\0\0\0\0', b'\0'*(1024*1024+1)):
            with self.assertRaises(ValueError): author.decode_totodile(blob)


if __name__ == '__main__':
    unittest.main()
