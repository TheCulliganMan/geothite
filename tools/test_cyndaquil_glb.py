#!/usr/bin/env python3
"""Decoded candidate regressions; no Blender, Cargo, or source JSON is required.

CYNDAQUIL_MESH_SOURCE optionally names the old migration
input for an exact attribute-by-attribute comparison. Existing common GLB proof
math is reused; this suite checks Cyndaquil's own geometry and mechanics.
"""
import bisect
import hashlib
import json
import math
import os
from pathlib import Path
import struct
import unittest

import cyndaquil_glb as author
from skin_glb_test_support import (IDENTITY, decode, encode, matmul, raw_accessor, rows,
                             slerp, transform, trs, values)

HERE = Path(__file__).resolve().parent
ASSET = Path(os.environ.get('CYNDAQUIL_GLB', HERE.parent / 'crates/crystal-voxel-view/models/actor_props/battle_cyndaquil.glb'))


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

    def rotations(self, clip, fraction):
        result = [[0., 0., 0., 1.] for _ in author.JOINTS]
        if clip is not None:
            for node, times, keys in self.clips[clip]:
                time = fraction * times[-1]
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

    def posed(self, part, matrices, full_weights=False, normal=False):
        result = []
        points = self.normals[part] if normal else self.vertices[part]
        for v, point, ids, ws in zip(self.vertices[part], points, self.ids[part], self.weights[part]):
            if full_weights:
                ids, ws = author.skin_weights(self.names[part], v)
            out = [0., 0., 0.]
            for joint, weight in zip(ids, ws):
                if weight:
                    posed = transform(matrices[joint], point, 0. if normal else 1.)
                    out = [a + weight*b for a, b in zip(out, posed)]
            result.append(out)
        return result

    def neutral_model(self):
        parts = []
        for i, part in enumerate(self.parts):
            parts.append({'part': self.names[i], 'positions': sum(self.vertices[i], []),
                'normals': sum(self.normals[i], []),
                'indices': values(self.doc, self.binary, part['indices']),
                'base_color': self.doc['materials'][part['material']]['pbrMetallicRoughness']['baseColorFactor']})
        return {'name': 'battle_cyndaquil', 'version': 1, 'coordinate_system': author.COORDINATES, 'primitives': parts}


class CyndaquilTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.blob = ASSET.read_bytes()
        cls.model = Model(cls.blob)

    def test_original_geometry_bits_colors_names_and_index_order(self):
        model = self.model.neutral_model()
        self.assertEqual(author.geometry_digest(model), author.REST_DIGEST)
        self.assertEqual(self.model.names, list(author.PARTS))
        self.assertEqual(sum(len(p) for p in self.model.vertices), 5128)
        self.assertEqual(sum(len(p['indices'])//3 for p in model['primitives']), 7180)
        source_path = os.environ.get('CYNDAQUIL_MESH_SOURCE')
        if source_path:
            source = json.loads(Path(source_path).read_bytes())
            for p, q in zip(model['primitives'], source['primitives']):
                for key, component in (('positions', 5126), ('normals', 5126), ('base_color', 5126), ('indices', 5125)):
                    self.assertEqual(author.packed(p[key], component), author.packed(q[key], component))
        points = sum(self.model.vertices, [])
        self.assertEqual(min(p[1] for p in points), 0.)
        self.assertEqual(struct.pack('<f', max(p[1] for p in points)), struct.pack('<f', .91))

    def test_standard_bounded_schema_one_skin_and_identity_root(self):
        doc = self.model.doc
        self.assertLess(len(self.blob), 300000)
        self.assertEqual(doc['asset']['version'], '2.0')
        self.assertFalse(doc.get('extensionsRequired'))
        self.assertEqual((len(doc['meshes']), len(doc['skins']), len(doc['scenes']), len(doc['buffers'])), (1, 1, 1, 1))
        self.assertEqual(doc['scenes'][0]['nodes'], [0, 10])
        self.assertEqual(doc['nodes'][0]['translation'], [0., 0., 0.])
        self.assertEqual(doc['nodes'][10], {'name': 'cyndaquil', 'mesh': 0, 'skin': 0})
        self.assertEqual(doc['skins'][0]['joints'], list(range(10)))
        self.assertLessEqual(len(author.JOINTS), 32)
        self.assertNotIn('uri', doc['buffers'][0])
        for view in doc['bufferViews']:
            self.assertEqual(view['byteOffset'] % 4, 0)
            self.assertLessEqual(view['byteOffset'] + view['byteLength'], doc['buffers'][0]['byteLength'])
        for part in self.model.parts:
            self.assertEqual(set(part['attributes']), {'POSITION', 'NORMAL', 'JOINTS_0', 'WEIGHTS_0'})
            for key in ('JOINTS_0', 'WEIGHTS_0'):
                accessor = doc['accessors'][part['attributes'][key]]
                self.assertEqual((accessor['type'], accessor['componentType']), ('VEC4', 5121))
                self.assertEqual(accessor.get('normalized', False), key == 'WEIGHTS_0')

    def test_all_influences_valid_normalized_and_unused_slots_zero(self):
        used = set()
        for part, ids, weights in zip(self.model.parts, self.model.ids, self.model.weights):
            raw = raw_accessor(self.model.doc, self.model.binary, part['attributes']['WEIGHTS_0'])
            self.assertTrue(all(sum(raw[i:i+4]) == 255 for i in range(0, len(raw), 4)))
            for js, ws in zip(ids, weights):
                self.assertAlmostEqual(sum(ws), 1.)
                self.assertTrue(all(type(j) is int and 0 <= j < 10 for j in js))
                self.assertTrue(all(math.isfinite(w) and 0. <= w <= 1. for w in ws))
                for j, w in zip(js, ws):
                    if w:
                        used.add(j)
                    else:
                        self.assertEqual(j, 0)
        self.assertEqual(used, set(range(10)))

    def test_exact_neutral_endpoints_and_rotation_only_clip_contract(self):
        self.assertEqual(set(self.model.clips), {'idle', 'attack', 'hit'})
        for clip, (_, duration, samples) in zip(self.model.doc['animations'], author.CLIPS):
            self.assertEqual(clip['extras']['loopSuggested'], clip['name'] == 'cyndaquil.idle')
            self.assertEqual(clip['extras']['cueRelative'], clip['name'] != 'cyndaquil.idle')
            self.assertEqual(len(clip['channels']), 9)
            for channel in clip['channels']:
                self.assertEqual(channel['target']['path'], 'rotation')
                self.assertIn(channel['target']['node'], range(1, 10))
                self.assertEqual(clip['samplers'][channel['sampler']]['interpolation'], 'LINEAR')
            for _, times, keys in self.model.clips[clip['name'].split('.')[-1]]:
                self.assertEqual(len(times), samples)
                self.assertAlmostEqual(times[-1], duration, places=6)
                self.assertEqual(times, sorted(set(times)))
                self.assertEqual(keys[0], [0., 0., 0., 1.])
                self.assertEqual(keys[-1], keys[0])
                self.assertTrue(all(abs(sum(v*v for v in q)-1.) < 2e-7 for q in keys))
            for t in (0., 1.):
                for matrix in self.model.matrices(clip['name'].split('.')[-1], t):
                    self.assertLess(max(abs(a-b) for a, b in zip(matrix, IDENTITY)), 3e-8)
        for i in range(30):
            for a, b in zip(self.model.vertices[i], self.model.posed(i, self.model.matrices())):
                self.assertLess(math.dist(a, b), 3e-8)
            for a, b in zip(self.model.normals[i], self.model.posed(i, self.model.matrices(), normal=True)):
                self.assertLess(math.dist(a, b), 3e-8)

    def test_grounded_rear_paws_and_low_haunches_stay_fixed(self):
        probes = [(i, j) for i, points in enumerate(self.model.vertices)
                  for j, p in enumerate(points) if p[1] <= .105]
        self.assertGreater(len(probes), 250)
        for i, j in probes:
            self.assertEqual(self.model.ids[i][j], [0, 0, 0, 0])
            self.assertEqual(self.model.weights[i][j], [1., 0., 0., 0.])
        for clip in self.model.clips:
            for step in range(33):
                mats = self.model.matrices(clip, step/32)
                posed = {i: self.model.posed(i, mats) for i in {i for i, _ in probes}}
                for i, j in probes:
                    self.assertEqual(posed[i][j], self.model.vertices[i][j])

    def test_body_coat_share_one_field_face_is_rigid_and_forepaws_articulate(self):
        coincident = {}
        for i in (0, 1, 2, 3, 9):
            for p, ids, ws in zip(self.model.vertices[i], self.model.ids[i], self.model.weights[i]):
                js, weights = author.body_weights(p)
                qw = author.quantized_weights(weights)
                self.assertEqual(ids, [j if w else 0 for j, w in zip(js, qw)])
                self.assertEqual(ws, [w/255. for w in qw])
                signature = tuple(ids), tuple(ws)
                if tuple(p) in coincident:
                    self.assertEqual(signature, coincident[tuple(p)])
                coincident[tuple(p)] = signature
                if i in (0, 1, 2):
                    self.assertEqual(signature, ((2, 0, 0, 0), (1., 0., 0., 0.)))
        for joint in (3, 4):
            self.assertGreater(sum(any(j == joint and w > .75 for j, w in zip(js, ws))
                                   for js, ws in zip(self.model.ids[9], self.model.weights[9])), 30)

    def test_five_folded_quills_keep_all_nested_layers_rigid(self):
        for joint, (name, _) in enumerate(author.QUILLS, 5):
            indices = [i for i, p in enumerate(self.model.names) if p.startswith(name + ' / ')]
            self.assertEqual(len(indices), 5)
            for i in indices:
                self.assertTrue(all(ids == [joint, 0, 0, 0] for ids in self.model.ids[i]))
                self.assertTrue(all(ws == [1., 0., 0., 0.] for ws in self.model.weights[i]))

    def test_quantized_poses_stay_below_one_ten_thousandth_unit(self):
        probes = []
        for i, name in enumerate(self.model.names):
            for point, ids, ws in zip(self.model.vertices[i], self.model.ids[i], self.model.weights[i]):
                full_ids, full_ws = author.skin_weights(name, point)
                if max(full_ws) < 1.:
                    probes.append((point, ids, ws, full_ids, full_ws))
        self.assertGreater(len(probes), 500)
        worst = 0.
        for clip in ('idle', 'attack', 'hit'):
            for fraction in (0., .125, .20, .25, .43, .5, .66, .75, 1.):
                matrices = self.model.matrices(clip, fraction)
                for p, ids, ws, full_ids, full_ws in probes:
                    a, b = [0., 0., 0.], [0., 0., 0.]
                    for j, w in zip(ids, ws):
                        v = transform(matrices[j], p)
                        a = [x+w*y for x, y in zip(a, v)]
                    for j, w in zip(full_ids, full_ws):
                        v = transform(matrices[j], p)
                        b = [x+w*y for x, y in zip(b, v)]
                    worst = max(worst, math.dist(a, b))
        self.assertGreater(worst, .00001)
        self.assertLess(worst, .0001)

    def test_curves_close_smoothly_and_are_species_specific(self):
        for clip in ('idle', 'attack', 'hit'):
            eps = 1e-5
            first, last = author.pose_angles(clip, eps), author.pose_angles(clip, 1.-eps)
            # Idle is periodic, reactions settle to rest at both ends.
            for a, b in zip(first, last):
                for va, vb in zip(a, b):
                    if clip == 'idle':
                        self.assertLess(abs((va+vb)/eps), .0001)
                    else:
                        self.assertLess(max(abs(va), abs(vb))/eps, .0003)
        self.assertNotEqual(author.pose_angles('idle', .25)[5], author.pose_angles('idle', .25)[6])
        self.assertEqual(author.pose_angles('attack', .43)[0], (0., 0., 0.))

    def test_deterministic_export_and_wrong_anatomy_rejection(self):
        model = self.model.neutral_model()
        first = author.export_cyndaquil(model)
        self.assertEqual(first, self.blob)
        self.assertEqual(author.export_cyndaquil(author.decode_cyndaquil(self.blob)), self.blob)
        self.assertEqual(first, author.export_cyndaquil(model))
        model['primitives'][0]['part'] = 'Wrong nose'
        with self.assertRaisesRegex(ValueError, 'anatomy name'):
            author.export_cyndaquil(model)
        model['primitives'][0]['part'] = author.PARTS[0]
        model['primitives'][0]['positions'][0] += .0001
        with self.assertRaisesRegex(ValueError, 'unreviewed neutral'):
            author.export_cyndaquil(model)

    def test_neutral_reader_rejects_modified_skin_and_ignored_fields(self):
        import copy
        for edit in (
            lambda d: d['nodes'][1].update(translation=[0., 1., 0.]),
            lambda d: d['nodes'][10].update(scale=[2., 2., 2.]),
            lambda d: d['animations'][0]['samplers'][0].update(interpolation='STEP'),
            lambda d: d['meshes'][0]['primitives'][0].update(targets=[]),
            lambda d: d['buffers'][0].update(uri='external.bin'),
            lambda d: d['skins'][0]['joints'].reverse(),
            lambda d: d['accessors'][1].update(count=100000000),
        ):
            doc = copy.deepcopy(self.model.doc)
            edit(doc)
            with self.assertRaises(ValueError):
                author.decode_cyndaquil(encode(doc, self.model.binary))
        for blob in (self.blob[:100], self.blob+b'\0\0\0\0', b'\0'*(1024*1024+1)):
            with self.assertRaises(ValueError):
                author.decode_cyndaquil(blob)


if __name__ == '__main__':
    unittest.main()
