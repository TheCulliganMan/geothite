#!/usr/bin/env python3
"""Independent decoded-GLB regressions for Gengar, with no Blender or Cargo.

All playback uses decoded hierarchy, keys, inverse binds and stored U8 weights.
The exporter is invoked only for determinism and invalid-input checks. Optional
GENGAR_MESH_SOURCE compares every field with the retired migration input.
"""
import bisect
import copy
import hashlib
import json
import math
import os
from pathlib import Path
import struct
import tempfile
import unittest

import gengar_glb as author
from skin_glb_test_support import (IDENTITY, decode, encode, matmul, raw_accessor,
                                   rows, slerp, transform, trs, values)

HERE = Path(__file__).resolve().parent
ASSET = Path(os.environ.get('GENGAR_GLB', HERE / 'gengar.glb'))
if not ASSET.exists():
    ASSET = HERE.parent / 'crates/crystal-voxel-view/models/battle_species/gengar.glb'
EXPECTED_NAMES = ['root', 'torso', 'face_core', 'shoulder_left', 'hand_left',
                  'shoulder_right', 'hand_right', 'ear_left', 'ear_right',
                  'dorsal_crown', 'dorsal_left', 'dorsal_right', 'dorsal_lower', 'tail_spine']
EXPECTED_PARENTS = [None, 0, 1, 1, 3, 1, 5, 2, 2, 2, 2, 2, 1, 1]
EXPECTED_DIGEST = 'd458acb7c2853288efef3b83c24ac453bcb409f9db01d24a96008e9a937485aa'


def pack(data, code='f'):
    return struct.pack('<'+code*len(data), *data)


def digest(model):
    h = hashlib.sha256()
    for p in model['primitives']:
        for key, code in (('positions', 'f'), ('normals', 'f'), ('indices', 'I'), ('base_color', 'f')):
            h.update(pack(p[key], code))
    return h.hexdigest()


def winding_alignment(points, normals, triangle):
    a, b, c = (points[i] for i in triangle)
    u, v = [b[k]-a[k] for k in range(3)], [c[k]-a[k] for k in range(3)]
    cross = [u[1]*v[2]-u[2]*v[1], u[2]*v[0]-u[0]*v[2], u[0]*v[1]-u[1]*v[0]]
    normal = [sum(normals[i][k] for i in triangle) for k in range(3)]
    twice_area = math.sqrt(sum(v*v for v in cross))
    normal_length = math.sqrt(sum(v*v for v in normal))
    cosine = sum(a*b for a, b in zip(cross, normal))/(twice_area*normal_length) if twice_area else -1.
    return cosine, twice_area


class Model:
    def __init__(self, blob):
        self.doc, self.binary = decode(blob)
        doc, binary = self.doc, self.binary
        self.parts = doc['meshes'][0]['primitives']
        self.vertices, self.normals, self.ids, self.weights, self.names, self.indices = [], [], [], [], [], []
        self.parents = [None] * len(doc['nodes'])
        for i, node in enumerate(doc['nodes']):
            for child in node.get('children', []):
                if self.parents[child] is not None:
                    raise ValueError('multiple parents')
                self.parents[child] = i
        self.joints = doc['skins'][0]['joints']
        for part in self.parts:
            attrs = part['attributes']
            self.vertices.append(rows(values(doc, binary, attrs['POSITION']), 3))
            self.normals.append(rows(values(doc, binary, attrs['NORMAL']), 3))
            self.ids.append(rows(values(doc, binary, attrs['JOINTS_0']), 4))
            self.weights.append(rows(values(doc, binary, attrs['WEIGHTS_0']), 4))
            self.names.append(part['extras']['part'])
            self.indices.append(values(doc, binary, part['indices']))
        self.inverse = rows(values(doc, binary, doc['skins'][0]['inverseBindMatrices']), 16)
        self.clips = {}
        for clip in doc['animations']:
            tracks = []
            for channel in clip['channels']:
                sampler = clip['samplers'][channel['sampler']]
                tracks.append((channel['target']['node'], values(doc, binary, sampler['input']),
                               rows(values(doc, binary, sampler['output']), 4)))
            self.clips[clip['name'].split('.')[-1]] = tracks

    def rotations(self, clip=None, fraction=0.):
        result = [[0., 0., 0., 1.] for _ in self.doc['nodes']]
        if clip is not None:
            for node, times, keys in self.clips[clip]:
                time = fraction*times[-1]
                lo = max(0, min(len(times)-2, bisect.bisect_right(times, time)-1))
                result[node] = slerp(keys[lo], keys[lo+1], (time-times[lo])/(times[lo+1]-times[lo]))
        return result

    def matrices(self, clip=None, fraction=0., rotations=None):
        rotations = rotations if rotations is not None else self.rotations(clip, fraction)
        globals_ = []
        for i, node in enumerate(self.doc['nodes']):
            local = trs(node.get('translation', [0., 0., 0.]), rotations[i])
            parent = self.parents[i]
            globals_.append(matmul(globals_[parent], local) if parent is not None else local)
        return [matmul(globals_[node], inverse) for node, inverse in zip(self.joints, self.inverse)]

    def point(self, part, index, matrices, normal=False):
        vector = (self.normals if normal else self.vertices)[part][index]
        result = [0., 0., 0.]
        for j, weight in zip(self.ids[part][index], self.weights[part][index]):
            if weight:
                transformed = transform(matrices[j], vector, 0. if normal else 1.)
                for axis in range(3):
                    result[axis] += weight*transformed[axis]
        return result

    def posed(self, part, matrices, normal=False):
        return [self.point(part, i, matrices, normal) for i in range(len(self.vertices[part]))]

    def neutral_model(self):
        return {'name': 'gengar', 'version': 1, 'coordinate_system': '+Y up; front +Z; floor-centered root',
                'primitives': [{'part': self.names[i], 'positions': sum(self.vertices[i], []),
                    'normals': sum(self.normals[i], []), 'indices': self.indices[i],
                    'base_color': self.doc['materials'][part['material']]['pbrMetallicRoughness']['baseColorFactor']}
                    for i, part in enumerate(self.parts)]}


class GengarTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.blob = ASSET.read_bytes()
        cls.model = Model(cls.blob)

    def test_exact_source_geometry_bits_material_identity_and_names(self):
        model = self.model.neutral_model()
        self.assertEqual(digest(model), EXPECTED_DIGEST)
        self.assertEqual(len(model['primitives']), 27)
        self.assertEqual(sum(len(p) for p in self.model.vertices), 6952)
        self.assertEqual(sum(len(p)//3 for p in self.model.indices), 5560)
        self.assertEqual(len(set(self.model.names)), 27)
        self.assertEqual([m['name'] for m in self.model.doc['materials']],
                         ['gengar/violet', 'gengar/mouth', 'gengar/ivory', 'gengar/crimson'])
        source = os.environ.get('GENGAR_MESH_SOURCE')
        if source:
            old = json.loads(Path(source).read_bytes())
            self.assertEqual(len(old['primitives']), 27)
            for a, b in zip(model['primitives'], old['primitives']):
                self.assertEqual(a['part'], b['part'])
                for key, code in (('positions', 'f'), ('normals', 'f'), ('indices', 'I'), ('base_color', 'f')):
                    self.assertEqual(pack(a[key], code), pack(b[key], code))
        points = sum(self.model.vertices, [])
        self.assertEqual(min(v[1] for v in points), 0.)
        self.assertEqual(pack([max(v[1] for v in points)]), pack([1.05]))

    def test_standard_bounded_skin_container(self):
        d = self.model.doc
        self.assertLess(len(self.blob), 400000)
        self.assertEqual(d['asset']['version'], '2.0')
        self.assertFalse(d.get('extensionsRequired'))
        self.assertEqual([len(d[k]) for k in ('meshes', 'skins', 'scenes', 'buffers')], [1, 1, 1, 1])
        self.assertLessEqual(len(d['accessors']), 256)
        self.assertEqual(d['scenes'][0]['nodes'], [0, 14])
        self.assertEqual(d['nodes'][14], {'name': 'gengar', 'mesh': 0, 'skin': 0})
        self.assertNotIn('uri', d['buffers'][0])
        for view in d['bufferViews']:
            self.assertEqual(view['byteOffset'] % 4, 0)
            self.assertLessEqual(view['byteOffset']+view['byteLength'], d['buffers'][0]['byteLength'])
            self.assertNotIn('byteStride', view)
        for p in self.model.parts:
            self.assertEqual(set(p['attributes']), {'POSITION', 'NORMAL', 'JOINTS_0', 'WEIGHTS_0'})
            self.assertEqual(p['mode'], 4)
            self.assertNotIn('targets', p)
            self.assertEqual(d['accessors'][p['indices']]['componentType'], 5123)
            for name in ('JOINTS_0', 'WEIGHTS_0'):
                a = d['accessors'][p['attributes'][name]]
                self.assertEqual((a['type'], a['componentType']), ('VEC4', 5121))
                self.assertEqual(a.get('normalized', False), name == 'WEIGHTS_0')

    def test_anatomical_hierarchy_inverse_binds_and_identity_root(self):
        d = self.model.doc
        self.assertEqual(self.model.joints, list(range(14)))
        self.assertEqual(d['skins'][0]['skeleton'], 0)
        self.assertEqual(self.model.parents[:14], EXPECTED_PARENTS)
        self.assertEqual([n['name'] for n in d['nodes'][:14]], ['gengar/'+n for n in EXPECTED_NAMES])
        self.assertEqual(d['nodes'][0]['translation'], [0., 0., 0.])
        for node in d['nodes'][:14]:
            self.assertFalse(set(node) & {'rotation', 'scale', 'matrix', 'mesh', 'skin'})
        neutral_matrices = self.model.matrices()
        for matrix in neutral_matrices:
            self.assertLess(max(abs(a-b) for a, b in zip(matrix, IDENTITY)), 3e-8)
        for part in range(27):
            for i, point in enumerate(self.model.vertices[part]):
                self.assertLess(math.dist(point, self.model.point(part, i, neutral_matrices)), 3e-8)

    def test_weights_are_real_normalized_multi_joint_influences(self):
        used, blended = set(), 0
        for p, ids, weights in zip(self.model.parts, self.model.ids, self.model.weights):
            raw = raw_accessor(self.model.doc, self.model.binary, p['attributes']['WEIGHTS_0'])
            self.assertTrue(all(sum(raw[i:i+4]) == 255 for i in range(0, len(raw), 4)))
            for js, ws in zip(ids, weights):
                self.assertAlmostEqual(sum(ws), 1.)
                self.assertTrue(all(type(j) is int and 0 <= j < 14 for j in js))
                self.assertTrue(all(0. <= w <= 1. for w in ws))
                blended += sum(w > 0. for w in ws) > 1
                for j, w in zip(js, ws):
                    if w:
                        used.add(j)
                    else:
                        self.assertEqual(j, 0)
        self.assertEqual(used, set(range(14)))
        self.assertGreater(blended, 1000)

    def test_three_rotation_clips_have_shared_clocks_unit_keys_and_neutral_endpoints(self):
        self.assertEqual(list(self.model.clips), ['idle', 'attack', 'hit'])
        for clip, seconds, count in zip(self.model.doc['animations'], (3.6, 1., 1.), (73, 65, 65)):
            suffix = clip['name'].split('.')[-1]
            self.assertEqual(clip['name'], 'gengar.'+suffix)
            self.assertEqual(clip['extras']['loopSuggested'], suffix == 'idle')
            self.assertEqual(clip['extras']['cueRelative'], suffix != 'idle')
            self.assertEqual(len(clip['channels']), 13)
            self.assertEqual({c['target']['node'] for c in clip['channels']}, set(range(1, 14)))
            self.assertEqual({c['target']['path'] for c in clip['channels']}, {'rotation'})
            self.assertEqual({s['interpolation'] for s in clip['samplers']}, {'LINEAR'})
            self.assertEqual(len({s['input'] for s in clip['samplers']}), 1)
            for _, times, keys in self.model.clips[suffix]:
                self.assertEqual(len(times), count)
                self.assertAlmostEqual(times[-1], seconds, places=6)
                self.assertEqual(times, sorted(set(times)))
                self.assertEqual(keys[0], [0., 0., 0., 1.])
                self.assertEqual(keys[-1], keys[0])
                self.assertTrue(all(abs(sum(v*v for v in q)-1.) < 2e-7 for q in keys))
            for t in (0., 1.):
                for matrix in self.model.matrices(suffix, t):
                    self.assertLess(max(abs(a-b) for a, b in zip(matrix, IDENTITY)), 3e-8)

    def test_low_feet_and_all_six_toes_remain_exactly_fixed(self):
        probes = [(p, i) for p, points in enumerate(self.model.vertices)
                  for i, v in enumerate(points) if v[1] <= .120]
        self.assertGreater(len(probes), 330)
        for p in (4, 5, 6, 10, 11, 12):
            self.assertTrue(all((p, i) in probes for i in range(len(self.model.vertices[p]))))
        for p, i in probes:
            self.assertEqual(self.model.ids[p][i], [0, 0, 0, 0])
            self.assertEqual(self.model.weights[p][i], [1., 0., 0., 0.])
        for clip in self.model.clips:
            for step in range(33):
                matrices = self.model.matrices(clip, step/32)
                for p, i in probes:
                    self.assertEqual(self.model.point(p, i, matrices), self.model.vertices[p][i])

    def test_grin_teeth_eyes_eyelids_and_front_core_are_one_coherent_rigid_face(self):
        facial = [(p, i) for p in range(13, 27) for i in range(len(self.model.vertices[p]))]
        under_face = [(0, i) for i, (x, y, z) in enumerate(self.model.vertices[0])
                      if abs(x) <= .3132 and .328 <= y <= .768 and z >= .100]
        self.assertGreater(len(under_face), 75)
        for p, i in facial+under_face:
            self.assertEqual(self.model.ids[p][i], [2, 0, 0, 0])
            self.assertEqual(self.model.weights[p][i], [1., 0., 0., 0.])
        for clip, fraction in (('idle', .25), ('attack', .43), ('hit', .18)):
            matrices = self.model.matrices(clip, fraction)
            for p, i in facial[::19]+under_face:
                expect = transform(matrices[2], self.model.vertices[p][i])
                self.assertEqual(self.model.point(p, i, matrices), expect)
                self.assertLess(abs(math.dist([0., 0., 0.], self.model.point(p, i, matrices, True))-1.), 2e-6)

    def test_duplicate_positions_stay_welded_even_across_material_normal_splits(self):
        seen, repeated = {}, 0
        for p, points in enumerate(self.model.vertices):
            for i, point in enumerate(points):
                key = pack(point)
                signature = self.model.ids[p][i], self.model.weights[p][i]
                if key in seen:
                    self.assertEqual(signature, seen[key])
                    repeated += 1
                seen[key] = signature
        self.assertGreater(repeated, 4000)

    def test_idle_loop_has_no_pose_or_velocity_pop(self):
        start = self.model.matrices('idle', 0.)
        end = self.model.matrices('idle', 1.)
        before = self.model.matrices('idle', 1.-1e-4)
        after = self.model.matrices('idle', 1e-4)
        for p in range(27):
            for i in range(0, len(self.model.vertices[p]), 7):
                a, b = self.model.point(p, i, start), self.model.point(p, i, end)
                self.assertEqual(a, b)
                c, d = self.model.point(p, i, before), self.model.point(p, i, after)
                self.assertLess(math.dist(c, d), .0001)
                left = [(u-v)/.00036 for u, v in zip(a, c)]
                right = [(u-v)/.00036 for u, v in zip(d, b)]
                self.assertLess(math.dist(left, right), .014)

    def test_every_anatomy_region_moves_distinctly_at_readable_amplitude(self):
        # Hinge residuals isolate local motion from inherited torso motion.
        for clip in ('idle', 'attack', 'hit'):
            excursions = [0.] * 14
            for _, _, keys in self.model.clips[clip]:
                self.assertGreater(max(math.dist(q, [0., 0., 0., 1.]) for q in keys), .009)
            for t in (.18, .25, .43, .5, .75):
                matrices = self.model.matrices(clip, t)
                for p, points in enumerate(self.model.vertices):
                    for i, point in enumerate(points):
                        j = max(range(4), key=lambda k: self.model.weights[p][i][k])
                        bone = self.model.ids[p][i][j]
                        moved = self.model.point(p, i, matrices)
                        excursions[bone] = max(excursions[bone], math.dist(point, moved))
            # Root-dominant ankle blend vertices may move; fully root-weighted
            # foot/toe vertices are separately verified bit-exactly fixed.
            self.assertLess(excursions[0], .03)
            threshold = .004 if clip == 'idle' else .010
            for bone in range(1, 14):
                self.assertGreater(excursions[bone], threshold, (clip, EXPECTED_NAMES[bone], excursions[bone]))
            if clip == 'attack':
                self.assertGreater(max(excursions[3:7]), .15)
                self.assertGreater(excursions[2], .04)
        attack = self.model.rotations('attack', .43)
        self.assertNotEqual(attack[3], attack[5])
        self.assertNotEqual(attack[4], attack[6])
        self.assertNotEqual(attack[7], attack[8])
        self.assertNotEqual(attack[10], attack[11])

    def test_dense_playback_and_blends_keep_bounds_normals_and_edges_safe(self):
        # The fused body is the region where skin continuity matters. Facial
        # patches are independently checked rigid, toes are independently fixed.
        edges = set()
        for triangle in rows(self.model.indices[0], 3):
            for a, b in zip(triangle, triangle[1:]+triangle[:1]):
                edges.add(tuple(sorted((a, b))))
        rest_edges = [(a, b, math.dist(self.model.vertices[0][a], self.model.vertices[0][b])) for a, b in sorted(edges)]
        poses = [self.model.rotations(clip, i/32) for clip in self.model.clips for i in range(33)]
        for ca, cb, ta, tb in (('idle', 'attack', .25, .43), ('attack', 'hit', .43, .18), ('hit', 'idle', .24, .75)):
            qa, qb = self.model.rotations(ca, ta), self.model.rotations(cb, tb)
            poses.extend([[slerp(a, b, t) for a, b in zip(qa, qb)] for t in (.25, .5, .75)])
        for rotations in poses:
            matrices = self.model.matrices(rotations=rotations)
            body = self.model.posed(0, matrices)
            for p, points in enumerate(self.model.vertices):
                # Every body vertex and material-part extrema plus strided face
                # samples are enough here; face rigid/ground tests cover all.
                probes = range(len(points)) if p == 0 else range(0, len(points), 11)
                for i in probes:
                    x, y, z = body[i] if p == 0 else self.model.point(p, i, matrices)
                    self.assertTrue(all(math.isfinite(v) for v in (x, y, z)))
                    self.assertTrue(-.75 <= x <= .75 and -1e-6 <= y <= 1.13 and -.55 <= z <= .60, (p, i, x, y, z))
                    normal = self.model.point(p, i, matrices, True)
                    self.assertTrue(.88 <= math.sqrt(sum(v*v for v in normal)) <= 1.00001)
            for a, b, rest in rest_edges:
                if rest > 1e-6:
                    ratio = math.dist(body[a], body[b])/rest
                    self.assertTrue(.55 <= ratio <= 1.65, (a, b, ratio))

    def test_deterministic_glb_only_reexport_and_source_rejection(self):
        neutral = self.model.neutral_model()
        self.assertEqual(author.export_gengar(neutral), self.blob)
        self.assertEqual(author.export_gengar(author.decode_gengar(self.blob)), self.blob)
        self.assertEqual(author.export_gengar(copy.deepcopy(neutral)), self.blob)
        neutral['primitives'][0]['part'] = 'Wrong body'
        with self.assertRaisesRegex(ValueError, 'anatomy name'):
            author.export_gengar(neutral)
        neutral['primitives'][0]['part'] = self.model.names[0]
        neutral['primitives'][0]['positions'][0] += .0001
        with self.assertRaisesRegex(ValueError, 'unreviewed neutral'):
            author.export_gengar(neutral)

    def test_fused_body_winding_stays_positive_against_skinned_source_normals(self):
        # Twice-area < 1e-10 model units squared is the documented tolerance
        # for a numerically degenerate source triangle. The reviewed source has
        # ZERO such triangles, so no geometry is exempted by this tolerance.
        # A newly collapsed posed triangle is always a failure.
        triangles = rows(self.model.indices[0], 3)
        neutral = [winding_alignment(self.model.vertices[0], self.model.normals[0], tr) for tr in triangles]
        self.assertTrue(all(area >= 1e-10 for _, area in neutral))
        self.assertTrue(all(cosine > 0. for cosine, _ in neutral))
        poses = []
        for clip in self.model.clips:
            times = self.model.clips[clip][0][1]
            # Every authored key and the midpoint of every LINEAR interval.
            fractions = sorted([t/times[-1] for t in times]
                               + [(a+b)/2/times[-1] for a, b in zip(times, times[1:])])
            poses.extend((clip, t, self.model.rotations(clip, t)) for t in fractions)
        for ca, cb, ta, tb in (('idle', 'attack', .25, .43), ('attack', 'hit', .43, .18), ('hit', 'idle', .24, .75)):
            qa, qb = self.model.rotations(ca, ta), self.model.rotations(cb, tb)
            poses.extend((ca+'-'+cb, t, [slerp(a, b, t) for a, b in zip(qa, qb)]) for t in (.25, .5, .75))
        for clip, t, rotations in poses:
            matrices = self.model.matrices(rotations=rotations)
            points, normals = self.model.posed(0, matrices), self.model.posed(0, matrices, True)
            for index, triangle in enumerate(triangles):
                cosine, twice_area = winding_alignment(points, normals, triangle)
                self.assertGreaterEqual(twice_area, 1e-10, (clip, t, index, 'collapsed'))
                self.assertGreater(cosine, 0., (clip, t, index, cosine))

    def test_canonical_registry_rejects_duplicate_geometry_and_unknown_glb(self):
        from battle_model_assets import read_species_model, species_paths
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            path = root / 'gengar.glb'
            path.write_bytes(self.blob)
            self.assertEqual(species_paths(root), {'gengar': path})
            self.assertEqual(digest(read_species_model(path)), EXPECTED_DIGEST)
            duplicate = root / 'gengar.mesh.json'
            duplicate.write_text('{}')
            with self.assertRaisesRegex(ValueError, 'duplicate canonical Gengar'):
                species_paths(root)
            duplicate.unlink()
            (root / 'unknown.glb').write_bytes(b'not an authored model')
            with self.assertRaisesRegex(ValueError, 'unsupported battle species GLB'):
                species_paths(root)

    def test_bounded_canonical_reader_rejects_ignored_or_corrupted_data(self):
        edits = (
            lambda d: d['nodes'][1].update(translation=[0., 1., 0.]),
            lambda d: d['nodes'][14].update(scale=[2., 2., 2.]),
            lambda d: d['animations'][0]['samplers'][0].update(interpolation='STEP'),
            lambda d: d['animations'][1]['channels'][0]['target'].update(node=0),
            lambda d: d['meshes'][0]['primitives'][0].update(targets=[]),
            lambda d: d['buffers'][0].update(uri='external.bin'),
            lambda d: d['skins'][0]['joints'].reverse(),
            lambda d: d['accessors'][1].update(count=100000000),
            lambda d: d['materials'][0]['pbrMetallicRoughness'].update(roughnessFactor=.2),
        )
        for edit in edits:
            doc = copy.deepcopy(self.model.doc)
            edit(doc)
            with self.assertRaises(ValueError):
                author.decode_gengar(encode(doc, self.model.binary))
        for blob in (self.blob[:100], self.blob+b'\0\0\0\0', b'\0'*(1024*1024+1)):
            with self.assertRaises(ValueError):
                author.decode_gengar(blob)


if __name__ == '__main__':
    unittest.main()
