#!/usr/bin/env python3
"""Small dependency-free regression tests for lossless human geometry storage."""
import copy
import tempfile
import unittest
from pathlib import Path

from johto_character_geometry import (
    canonical, compact_models, compare_directories, expand_rig, f32_contract_digest,
    load_library, pack_directory, read_models, validate_library, write_snapshot,
)


def fixture():
    geometry = {'positions': [-0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 0.0],
                'normals': [0.0, 0.0, 1.0] * 3, 'indices': [0, 1, 2]}
    return {'name': 'fixture', 'version': 1, 'coordinate_system': '+Y up; front +Z',
            'joints': [{'name': 'pelvis', 'parent': None, 'translation': [-0.0, .66, 0.0],
                        'primitives': [{**copy.deepcopy(geometry), 'base_color': color}
                                       for color in ([1.0, 0.0, .25, 1.0], [0.0, 1.0, .5, 1.0])]}]}


class SharedCharacterGeometryTests(unittest.TestCase):
    def test_material_instances_and_signed_zero_round_trip_exactly(self):
        original = fixture()
        library, rigs = compact_models({'fixture.rig.json': original})
        self.assertEqual(len(library['geometries']), 1)
        self.assertEqual(len(rigs['fixture.rig.json']['joints'][0]['primitives']), 2)
        expanded = expand_rig(rigs['fixture.rig.json'], library)
        self.assertEqual(canonical(original), canonical(expanded))
        self.assertEqual(f32_contract_digest(original), f32_contract_digest(expanded))
        changed = copy.deepcopy(original)
        changed['joints'][0]['primitives'][0]['positions'][0] = 0.0
        self.assertNotEqual(f32_contract_digest(original), f32_contract_digest(changed))

    def test_geometry_deduplication_is_not_float_quantization(self):
        original = fixture()
        original['joints'][0]['primitives'][1]['positions'][0] = 0.0
        library, _ = compact_models({'fixture.rig.json': original})
        self.assertEqual(len(library['geometries']), 2)

    def test_input_order_does_not_change_library_or_rig_bytes(self):
        first, second = fixture(), fixture()
        second['name'] = 'other'
        second['joints'][0]['primitives'][0]['positions'][3] = 1.000001
        a, ar = compact_models({'a.rig.json': first, 'b.rig.json': second})
        b, br = compact_models({'b.rig.json': second, 'a.rig.json': first})
        self.assertEqual(canonical(a), canonical(b))
        self.assertEqual(canonical(ar), canonical(br))

    def test_bad_indices_hybrid_fields_and_library_identity_are_rejected(self):
        library, rigs = compact_models({'fixture.rig.json': fixture()})
        for index in (-1, 1, True, 1.5):
            bad = copy.deepcopy(rigs['fixture.rig.json'])
            bad['joints'][0]['primitives'][0]['geometry'] = index
            with self.assertRaises(ValueError): expand_rig(bad, library)
        bad = copy.deepcopy(rigs['fixture.rig.json'])
        bad['geometry_library'] = 'not-the-library'
        with self.assertRaises(ValueError): expand_rig(bad, library)
        bad = copy.deepcopy(rigs['fixture.rig.json'])
        bad['joints'][0]['primitives'][0]['positions'] = []
        with self.assertRaises(ValueError): expand_rig(bad, library)
        bad_library = copy.deepcopy(library)
        bad_library['geometries'][0]['positions'][0] = 0.5
        with self.assertRaises(ValueError): validate_library(bad_library)

    def test_directory_pack_expand_and_repack_are_byte_deterministic(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp); before, after, again, expanded = [root / n for n in ('before', 'after', 'again', 'expanded')]
            write_snapshot(before, {'fixture.rig.json': fixture()})
            pack_directory(before, after)
            pack_directory(after, again)
            self.assertEqual({p.name: p.read_bytes() for p in after.iterdir()},
                             {p.name: p.read_bytes() for p in again.iterdir()})
            write_snapshot(expanded, read_models(after))
            self.assertEqual((before / 'fixture.rig.json').read_bytes(), (expanded / 'fixture.rig.json').read_bytes())
            self.assertEqual(compare_directories(before, after)['rigs'], 1)
            self.assertIsNotNone(load_library(after))


if __name__ == '__main__':
    unittest.main()
