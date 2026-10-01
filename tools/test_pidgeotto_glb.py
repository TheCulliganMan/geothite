#!/usr/bin/env python3
"""Production Pidgeotto geometry, articulation, strict decoding and migration gates."""
import copy
import hashlib
import importlib.util
import json
import math
import os
from pathlib import Path
import struct
import subprocess
import sys
import tempfile
import unittest

import animated_glb as glb
import pidgeotto_glb as pidgeotto
from test_animated_glb import primitive_contract, transforms, world_points

REPO = Path(os.environ.get('GEOTHITE_REPO', Path(__file__).resolve().parents[1]))
MODELS = REPO / glb.MODEL_ROOT
# One migration fingerprint, not a second copy of the authored geometry. This
# keeps the original name/order/f32/u32 gate meaningful after JSON retirement.
ORIGINAL_ANATOMY_SHA256 = 'e2718a2a888eb6c3b70672ea6378aa260d16561f5ccadd2436c7e62488cc86a8'


def encode(doc, binary):
    data = glb.canonical(doc)
    data += b' ' * (-len(data) % 4)
    return (struct.pack('<III', 0x46546C67, 2, 28 + len(data) + len(binary)) +
            struct.pack('<II', len(data), 0x4E4F534A) + data +
            struct.pack('<II', len(binary), 0x004E4942) + binary)


class PidgeottoGlbTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.path = MODELS / 'battle_species/pidgeotto.glb'
        cls.blob = cls.path.read_bytes()
        cls.source = pidgeotto.load_pidgeotto(cls.path.parent)
        cls.model = pidgeotto.read_pidgeotto(cls.path)
        cls.doc, cls.binary = glb.parse_glb(cls.blob)

    def test_preserves_all_source_anatomy_geometry_normals_indices_and_colors(self):
        self.assertEqual(len(self.model['primitives']), 34)
        self.assertEqual(len(self.doc['nodes']), 39)
        self.assertEqual(len(self.doc['meshes']), 34)
        fingerprint = hashlib.sha256()
        for i, (source, decoded) in enumerate(zip(self.source['primitives'], self.model['primitives'])):
            self.assertEqual(source['part'], decoded['part'])
            self.assertEqual(self.doc['nodes'][i + 5]['extras']['sourcePrimitive'], i)
            primitive_contract(self, source, self.doc['meshes'][i]['primitives'][0], self.doc, self.binary)
            for key in (*glb.GEOMETRY_KEYS, 'base_color'):
                code = 'I' if key == 'indices' else 'f'
                self.assertEqual(glb.packed(source[key], code), glb.packed(decoded[key], code), (i, key))
            name = decoded['part'].encode()
            fingerprint.update(struct.pack('<I', len(name)))
            fingerprint.update(name)
            for key in (*glb.GEOMETRY_KEYS, 'base_color'):
                fingerprint.update(struct.pack('<I', len(decoded[key])))
                fingerprint.update(glb.packed(decoded[key], 'I' if key == 'indices' else 'f'))
        self.assertEqual(fingerprint.hexdigest(), ORIGINAL_ANATOMY_SHA256)
        neutral = world_points(self.doc, self.binary)
        for i, source in enumerate(self.source['primitives']):
            self.assertEqual([v for point in neutral[i + 5] for v in point], [glb.f32(v) for v in source['positions']])

    def test_migration_original_json_matches_canonical(self):
        path = self.path.with_suffix('.mesh.json')
        if not path.exists():
            self.skipTest('original migration input retired; canonical geometry tests remain active')
        self.assertEqual(pidgeotto.export_pidgeotto(glb.read_json(path)), self.blob)

    def test_production_curve_has_exact_closed_identity_seam(self):
        clip = self.doc['animations'][0]
        self.assertEqual(clip['name'], 'pidgeotto.idle_wings')
        self.assertEqual(clip['extras'], pidgeotto.CLIP_EXTRAS)
        expected_times, expected_tracks = glb.pidgeotto_idle_samples()
        for sampler, expected in zip(clip['samplers'], expected_tracks):
            times = glb.accessor_values(self.doc, self.binary, sampler['input'])
            values = glb.accessor_values(self.doc, self.binary, sampler['output'])
            self.assertEqual(len(times), 65)
            self.assertEqual(times, sorted(set(times)))
            self.assertEqual(times[-1], glb.f32(.8))
            self.assertEqual(glb.packed(times), glb.packed(expected_times))
            self.assertEqual(glb.packed(values), glb.packed(expected))
            self.assertEqual(sampler['interpolation'], 'LINEAR')
            for endpoint in (values[:4], values[-4:]):
                self.assertEqual(glb.packed(endpoint), glb.packed([0, 0, 0, 1]))
            for i in range(0, len(values), 4):
                self.assertAlmostEqual(sum(v * v for v in values[i:i + 4]), 1, places=6)
            # The end and start slopes agree: no sharp reversal at the loop.
            self.assertAlmostEqual(values[6] - values[2], values[-2] - values[-6], places=7)

    def test_motion_moves_only_ten_wing_parts_and_preserves_both_hinges(self):
        neutral = world_points(self.doc, self.binary)
        expected_nodes = {i + 5 for group in glb.pidgeotto_groups(self.model)[0].values() for i in group}
        self.assertEqual(neutral, world_points(self.doc, self.binary, 0))
        self.assertEqual(neutral, world_points(self.doc, self.binary, glb.f32(.8)))
        for time in (.1, .2, .3, .5, .6, .7, .00625):
            animated = world_points(self.doc, self.binary, time)
            changed = {i for i in neutral if neutral[i] != animated[i]}
            self.assertEqual(changed, expected_nodes)
            world = transforms(self.doc, self.binary, time)
            for index in (1, 3):
                self.assertEqual(world[index][0], self.doc['nodes'][index]['translation'])
                sign = 1 if index == 1 else -1
                angle = 2 * math.atan2(world[index][1][2], world[index][1][3])
                self.assertAlmostEqual(angle, sign * .42 * math.sin(2 * math.pi * time / .8), delta=.0006)
        animated = world_points(self.doc, self.binary, .2)
        self.assertGreater(max(math.dist(a, b) for i in expected_nodes for a, b in zip(neutral[i], animated[i])), .08)

    def test_export_is_deterministic_and_demo_works_from_glb_only(self):
        self.assertEqual(pidgeotto.export_pidgeotto(self.model), pidgeotto.export_pidgeotto(self.model))
        with tempfile.TemporaryDirectory() as temporary:
            path = Path(temporary) / pidgeotto.FILE
            path.write_bytes(self.blob)
            model = pidgeotto.load_pidgeotto(path.parent)
            demo, _ = glb.parse_glb(glb.export_pidgeotto(model))
            self.assertEqual(demo['animations'][0]['name'], 'pidgeotto.wing_flap_demo')
            self.assertEqual(pidgeotto.species_paths(path.parent), {'pidgeotto': path})
            path.with_suffix('.mesh.json').write_text('{}')
            with self.assertRaisesRegex(ValueError, 'duplicate'):
                pidgeotto.species_paths(path.parent)

    def test_rejects_unsupported_hierarchy_materials_animation_and_encoding(self):
        mutations = [
            lambda d: d.update(skins=[]),
            lambda d: d.update(extensionsRequired=['KHR_draco_mesh_compression']),
            lambda d: d['nodes'][0].update(rotation=[0, 0, 0, 1]),
            lambda d: d['nodes'][1].update(scale=[1, 1, 1]),
            lambda d: d['nodes'][2]['translation'].__setitem__(0, 0),
            lambda d: d['nodes'][2]['children'].pop(),
            lambda d: d['nodes'][1]['children'].append(0),
            lambda d: d['nodes'][5]['extras'].update(sourcePrimitive=1),
            lambda d: d['nodes'][1]['extras'].update(sourcePrimitiveIndices=[1]),
            lambda d: d['materials'][0].update(doubleSided=True),
            lambda d: d['materials'][0]['pbrMetallicRoughness'].update(metallicFactor=.5),
            lambda d: d['materials'][0]['pbrMetallicRoughness'].update(baseColorTexture={'index': 0}),
            lambda d: d['animations'][0].update(name='demo'),
            lambda d: d['animations'][0]['channels'][0]['target'].update(node=0),
            lambda d: d['animations'][0]['channels'][0]['target'].update(path='translation'),
            lambda d: d['animations'][0]['samplers'][0].update(interpolation='CUBICSPLINE'),
            lambda d: d['animations'].clear(),
            lambda d: d['accessors'][0].update(count=len(self.binary)),
            lambda d: d['accessors'][0].update(normalized=True),
            lambda d: d['accessors'][0].update(sparse={}),
            lambda d: d['bufferViews'][0].update(byteStride=12),
            lambda d: d['buffers'][0].update(uri='untrusted.bin'),
            lambda d: d['meshes'][0]['primitives'][0].update(targets=[]),
            lambda d: d['meshes'][0]['primitives'][0]['attributes'].update(JOINTS_0=0),
        ]
        for i, mutate in enumerate(mutations):
            with self.subTest(mutation=i):
                doc = copy.deepcopy(self.doc)
                mutate(doc)
                with self.assertRaises(ValueError):
                    pidgeotto.decode_pidgeotto(encode(doc, self.binary))

    def test_rejects_corrupted_curve_and_nonfinite_geometry(self):
        sampler = self.doc['animations'][0]['samplers'][0]
        for accessor_index, offset, value in ((sampler['output'], 8, .1),
                                               (sampler['input'], 4, 0), (0, 0, float('nan'))):
            accessor = self.doc['accessors'][accessor_index]
            view = self.doc['bufferViews'][accessor['bufferView']]
            binary = bytearray(self.binary)
            struct.pack_into('<f', binary, view['byteOffset'] + offset, value)
            with self.assertRaises(ValueError):
                pidgeotto.decode_pidgeotto(encode(copy.deepcopy(self.doc), binary))
        for blob in (self.blob[:-1], self.blob + b'\0', b'bad', b'\0' * (pidgeotto.MAX_BYTES + 1)):
            with self.assertRaises(ValueError):
                pidgeotto.decode_pidgeotto(blob)

    def test_battle_checker_accepts_glb_only_tree_and_rejects_duplicate(self):
        with tempfile.TemporaryDirectory() as temporary:
            models = Path(temporary)
            for source in (MODELS / 'battle_species').iterdir():
                if source.name != 'pidgeotto.mesh.json':
                    (models / source.name).symlink_to(source)
            command = [sys.executable, str(REPO / 'tools/check-battle-species.py'), '--models', str(models)]
            result = subprocess.run(command, capture_output=True, text=True)
            self.assertEqual(result.returncode, 0, result.stderr)
            (models / 'pidgeotto.mesh.json').write_text('{}')
            result = subprocess.run(command, capture_output=True, text=True)
            self.assertNotEqual(result.returncode, 0)
            self.assertIn('duplicate canonical Pidgeotto', result.stderr)

    def test_stl_reads_neutral_glb_with_identical_source_facets(self):
        spec = importlib.util.spec_from_file_location('pidgeotto_model_stl', REPO / 'tools/export-model-stl.py')
        stl = importlib.util.module_from_spec(spec)
        sys.modules[spec.name] = stl
        spec.loader.exec_module(stl)
        canonical = stl.ModelReader(MODELS).read(self.path)
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            source = root / 'pidgeotto.mesh.json'
            source.write_text(json.dumps(self.source))
            original = stl.ModelReader(root).read(source)
            self.assertEqual(stl.encode_stl(original), stl.encode_stl(canonical))
            self.assertEqual(stl.output_path(self.path, MODELS, root), root / 'battle_species/pidgeotto.stl')


if __name__ == '__main__':
    unittest.main(verbosity=2)
