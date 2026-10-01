#!/usr/bin/env python3
"""Exact source-data and evaluated articulation checks; standard library only."""
import copy
import json
import math
import os
import struct
import unittest
from pathlib import Path

import animated_glb as e

REPO = Path(os.environ.get('GEOTHITE_REPO', Path(__file__).resolve().parents[1]))


def parents(doc):
    result = {}
    for parent, node in enumerate(doc['nodes']):
        for child in node.get('children', []):
            assert child not in result, 'node has multiple parents'
            assert 0 <= child < len(doc['nodes']), 'invalid child'
            result[child] = parent
    for node in range(len(doc['nodes'])):
        seen = set()
        while node in result:
            assert node not in seen, 'node cycle'
            seen.add(node)
            node = result[node]
    return result


def qmul(a, b):
    x, y, z, w = a
    X, Y, Z, W = b
    return [w * X + x * W + y * Z - z * Y,
            w * Y - x * Z + y * W + z * X,
            w * Z + x * Y - y * X + z * W,
            w * W - x * X - y * Y - z * Z]


def rotate(q, point):
    return qmul(qmul(q, [*point, 0]), [-q[0], -q[1], -q[2], q[3]])[:3]


def normalized(q):
    length = math.sqrt(sum(v * v for v in q))
    return [v / length for v in q]


def slerp(a, b, t):
    a, b = normalized(a), normalized(b)
    dot = sum(x * y for x, y in zip(a, b))
    if dot < 0:
        b, dot = [-v for v in b], -dot
    if dot > .9995:
        return normalized([x * (1 - t) + y * t for x, y in zip(a, b)])
    theta = math.acos(max(-1, min(1, dot)))
    return [(x * math.sin((1 - t) * theta) + y * math.sin(t * theta)) / math.sin(theta)
            for x, y in zip(a, b)]


def transforms(doc, binary, time=None):
    rotations = {}
    if time is not None:
        for channel in doc['animations'][0]['channels']:
            assert channel['target']['path'] == 'rotation'
            sampler = doc['animations'][0]['samplers'][channel['sampler']]
            times = e.accessor_values(doc, binary, sampler['input'])
            values = e.accessor_values(doc, binary, sampler['output'])
            assert times == sorted(set(times))
            assert sampler['interpolation'] == 'LINEAR'
            q = [values[i:i + 4] for i in range(0, len(values), 4)]
            if time <= times[0]:
                value = q[0]
            elif time >= times[-1]:
                value = q[-1]
            else:
                index = next(i for i in range(len(times) - 1) if times[i] <= time <= times[i + 1])
                value = slerp(q[index], q[index + 1], (time - times[index]) / (times[index + 1] - times[index]))
            rotations[channel['target']['node']] = value
    p = parents(doc)
    world = {}
    def get(index):
        if index not in world:
            node = doc['nodes'][index]
            t = node.get('translation', [0, 0, 0])
            q = rotations.get(index, node.get('rotation', [0, 0, 0, 1]))
            if index in p:
                pt, pq = get(p[index])
                t = [a + b for a, b in zip(pt, rotate(pq, t))]
                q = qmul(pq, q)
            world[index] = (t, q)
        return world[index]
    for index in range(len(doc['nodes'])):
        get(index)
    return world


def world_points(doc, binary, time=None):
    world = transforms(doc, binary, time)
    points = {}
    for index, node in enumerate(doc['nodes']):
        if 'mesh' not in node:
            continue
        t, q = world[index]
        result = []
        for primitive in doc['meshes'][node['mesh']]['primitives']:
            pos = e.accessor_values(doc, binary, primitive['attributes']['POSITION'])
            for i in range(0, len(pos), 3):
                result.append([a + b for a, b in zip(t, rotate(q, pos[i:i + 3]))])
        points[index] = result
    return points


def primitive_contract(test, source, primitive, doc, binary):
    for source_key, accessor in [('positions', primitive['attributes']['POSITION']),
                                 ('normals', primitive['attributes']['NORMAL']),
                                 ('indices', primitive['indices'])]:
        code = 'I' if source_key == 'indices' else 'f'
        actual = e.accessor_values(doc, binary, accessor)
        test.assertEqual(e.packed(source[source_key], code), e.packed(actual, code), source_key)
    color = doc['materials'][primitive['material']]['pbrMetallicRoughness']['baseColorFactor']
    test.assertEqual(source['base_color'], color)
    test.assertEqual(e.packed(source['base_color']), e.packed(color))
    test.assertEqual(primitive['mode'], 4)


class AnimatedGlbProof(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        models = REPO / e.MODEL_ROOT
        from johto_character_geometry import compact_models
        cls.catalog = e.load_catalog(models / 'johto_characters')
        cls.library, cls.rigs = compact_models({name + '.rig.json': rig for name, rig in cls.catalog.items()})
        cls.rig = cls.rigs['trainer.rig.json']
        cls.pokemon = e.read_json(models / 'battle_species/pidgeotto.mesh.json')
        cls.human_blob = e.export_humans([cls.rig], cls.library)
        cls.pokemon_blob = e.export_pidgeotto(cls.pokemon)

    def test_human_exact_geometry_material_joint_contract(self):
        doc, binary = e.parse_glb(self.human_blob)
        expanded = e.resolve_rig(self.rig, self.library)
        self.assertEqual(len(doc['nodes']), 16)
        self.assertNotIn('skins', doc)
        ancestry = parents(doc)
        self.assertEqual(doc['scenes'][0]['nodes'], [0])
        for i, joint in enumerate(expanded['joints']):
            node = doc['nodes'][i]
            self.assertEqual(node['name'], joint['name'])
            self.assertEqual(ancestry.get(i), joint['parent'])
            self.assertEqual(e.packed(node['translation']), e.packed(joint['translation']))
            primitives = doc['meshes'][node['mesh']]['primitives']
            self.assertEqual(len(primitives), len(joint['primitives']))
            for source, actual in zip(joint['primitives'], primitives):
                primitive_contract(self, source, actual, doc, binary)

    def test_pokemon_exact_all_anatomy_and_neutral_world_geometry(self):
        doc, binary = e.parse_glb(self.pokemon_blob)
        self.assertNotIn('skins', doc)
        mesh_nodes = [(i, n) for i, n in enumerate(doc['nodes']) if 'mesh' in n]
        self.assertEqual(len(mesh_nodes), len(self.pokemon['primitives']))
        world = world_points(doc, binary)
        for (index, node), source in zip(mesh_nodes, self.pokemon['primitives']):
            self.assertEqual(node['name'], source['part'])
            primitives = doc['meshes'][node['mesh']]['primitives']
            self.assertEqual(len(primitives), 1)
            primitive_contract(self, source, primitives[0], doc, binary)
            # Ignore the sign of zero in the transform result, but not in the
            # stored accessor contract checked above.
            self.assertEqual([v for p in world[index] for v in p], [e.f32(v) for v in source['positions']])
        groups, pivots = e.pidgeotto_groups(self.pokemon)
        ancestry = parents(doc)
        for side, members in groups.items():
            hinge = next(i for i, n in enumerate(doc['nodes']) if n['name'] == f'wing_{side}_hinge')
            self.assertEqual(doc['nodes'][hinge]['translation'], pivots[side])
            for member in members:
                self.assertEqual(ancestry[ancestry[mesh_nodes[member][0]]], hinge)

    def test_human_nonroot_articulation_moves_only_intended_subtree(self):
        doc, binary = e.parse_glb(self.human_blob)
        self.assertEqual(doc['animations'][0]['name'], 'trainer.rig_wave_demo')
        names = [doc['nodes'][c['target']['node']]['name'] for c in doc['animations'][0]['channels']]
        self.assertEqual(names, ['upper_arm_r', 'forearm_r', 'hand_r'])
        neutral = world_points(doc, binary)
        animated = world_points(doc, binary, .6)  # Between keyframes, exercise slerp.
        moved = [doc['nodes'][i]['name'] for i in neutral if neutral[i] != animated[i]]
        self.assertEqual(moved, ['upper_arm_r', 'forearm_r', 'hand_r'])
        self.assertEqual(neutral, world_points(doc, binary, 0))
        self.assertEqual(neutral, world_points(doc, binary, 2))
        self.assertGreater(max(math.dist(a, b) for a, b in zip(neutral[9], animated[9])), .2)

    def test_pokemon_real_wing_motion_preserves_body_and_pivots(self):
        doc, binary = e.parse_glb(self.pokemon_blob)
        self.assertEqual(doc['animations'][0]['name'], 'pidgeotto.wing_flap_demo')
        names = [doc['nodes'][c['target']['node']]['name'] for c in doc['animations'][0]['channels']]
        self.assertEqual(names, ['wing_l_hinge', 'wing_r_hinge'])
        neutral = world_points(doc, binary)
        animated = world_points(doc, binary, .1)
        changed = [i for i in neutral if neutral[i] != animated[i]]
        self.assertEqual(len(changed), 10)
        self.assertTrue(all(doc['nodes'][i]['name'].startswith(('Layered wing', 'Flight feather')) for i in changed))
        self.assertEqual(neutral, world_points(doc, binary, 0))
        self.assertEqual(neutral, world_points(doc, binary, 1))
        world = transforms(doc, binary, .1)
        for side in ('l', 'r'):
            index = next(i for i, n in enumerate(doc['nodes']) if n['name'] == f'wing_{side}_hinge')
            self.assertEqual(world[index][0], doc['nodes'][index]['translation'])
        self.assertGreater(max(math.dist(a, b) for i in changed for a, b in zip(neutral[i], animated[i])), .05)

    def test_deterministic_bytes_and_disk_outputs(self):
        self.assertEqual(self.human_blob, e.export_humans([self.rig], self.library))
        self.assertEqual(self.pokemon_blob, e.export_pidgeotto(self.pokemon))

    def test_shared_geometry_reuses_accessors_even_across_scenes(self):
        duplicate = copy.deepcopy(self.rig)
        duplicate['name'] = 'trainer_second_instance'
        doc, _ = e.parse_glb(e.export_humans([self.rig, duplicate], self.library))
        self.assertEqual(len(doc['scenes']), 2)
        for left, right in zip(doc['meshes'][:16], doc['meshes'][16:]):
            self.assertEqual(left['primitives'], right['primitives'])
        seen, references = {}, 0
        for mesh in doc['meshes']:
            for primitive in mesh['primitives']:
                references += 1
                key = (primitive['attributes']['POSITION'], primitive['attributes']['NORMAL'], primitive['indices'])
                seen[key] = True
        self.assertLess(len(seen), references / 2)

    def test_complete_human_catalog_preserves_all_75_rigs_and_sharing(self):
        rigs = list(self.rigs.values())
        self.assertEqual(len(rigs), 75)
        doc, binary = e.parse_glb(e.export_humans(rigs, self.library))
        self.assertEqual(len(doc['scenes']), 75)
        self.assertEqual(len(doc['nodes']), 75 * 16)
        self.assertEqual(len(doc['animations']), 75)
        geometry_accessors = {}
        ancestry = parents(doc)
        for model_index, rig in enumerate(sorted(rigs, key=lambda r: r['name'])):
            offset = model_index * 16
            self.assertEqual(doc['scenes'][model_index]['name'], rig['name'])
            self.assertEqual(doc['scenes'][model_index]['nodes'], [offset])
            expanded = e.resolve_rig(rig, self.library)
            for i, joint in enumerate(expanded['joints']):
                node = doc['nodes'][offset + i]
                self.assertEqual(node['name'], joint['name'])
                self.assertEqual(ancestry.get(offset + i), None if joint['parent'] is None else offset + joint['parent'])
                self.assertEqual(e.packed(node['translation']), e.packed(joint['translation']))
                primitives = doc['meshes'][node['mesh']]['primitives']
                for j, (source, actual) in enumerate(zip(joint['primitives'], primitives)):
                    primitive_contract(self, source, actual, doc, binary)
                    reference = rig['joints'][i]['primitives'][j]['geometry']
                    ids = (actual['attributes']['POSITION'], actual['attributes']['NORMAL'], actual['indices'])
                    if reference in geometry_accessors:
                        self.assertEqual(geometry_accessors[reference], ids)
                    geometry_accessors[reference] = ids
        self.assertEqual(len(geometry_accessors), 977)

    def test_canonical_catalog_reads_without_json_sidecars(self):
        import tempfile
        directory = REPO / e.MODEL_ROOT / 'johto_characters'
        with tempfile.TemporaryDirectory() as temporary:
            path = Path(temporary) / e.CATALOG_FILE
            path.write_bytes((directory / e.CATALOG_FILE).read_bytes())
            catalog = e.load_catalog(path.parent)
            self.assertEqual(len(catalog), 75)
            self.assertEqual(catalog['trainer'], self.catalog['trainer'])
            doc, binary = e.parse_glb(path.read_bytes())
            geometry_ids = {(p['attributes']['POSITION'], p['attributes']['NORMAL'], p['indices'])
                            for mesh in doc['meshes'] for p in mesh['primitives']}
            self.assertEqual(len(geometry_ids), 977)
            self.assertEqual(len(doc['nodes']), 75 * 16)
            self.assertEqual({scene['name'] for scene in doc['scenes']}, set(catalog))

    def test_canonical_production_clips_are_complete_looped_and_shared(self):
        directory = REPO / e.MODEL_ROOT / 'johto_characters'
        doc, binary = e.parse_glb((directory / e.CATALOG_FILE).read_bytes())
        self.assertEqual(len(doc['animations']), 150)
        by_name = {clip['name']: clip for clip in doc['animations']}
        self.assertEqual(set(by_name), {name + suffix for name in self.catalog for suffix in ('.walk', '.run')})
        first_samplers = {}
        for name in self.catalog:
            for motion in ('walk', 'run'):
                clip = by_name[name + '.' + motion]
                self.assertEqual(len(clip['channels']), 15)
                self.assertEqual(clip['extras']['purpose'], 'production source-distance locomotion')
                self.assertTrue(clip['extras']['completeLegAnimation'])
                self.assertTrue(clip['extras']['loopSuggested'])
                # Every character has the same bind skeleton; sampler accessors
                # must reuse one physical copy of each authored motion curve.
                if motion in first_samplers:
                    self.assertEqual(clip['samplers'], first_samplers[motion])
                else:
                    first_samplers[motion] = clip['samplers']
                for channel in clip['channels']:
                    sampler = clip['samplers'][channel['sampler']]
                    times = e.accessor_values(doc, binary, sampler['input'])
                    values = e.accessor_values(doc, binary, sampler['output'])
                    width = 4 if channel['target']['path'] == 'rotation' else 3
                    self.assertEqual((times[0], times[-1]), (0, 1))
                    self.assertEqual(e.packed(values[:width]), e.packed(values[-width:]))
                    self.assertEqual(sampler['interpolation'], 'LINEAR')

    def test_migration_original_json_matches_canonical_catalog(self):
        # One-time gate while migrating; ongoing tests above use only GLB.
        directory = REPO / e.MODEL_ROOT / 'johto_characters'
        files = sorted(directory.glob('*.rig.json'))
        if not files:
            self.skipTest('original migration inputs retired; canonical GLB tests remain active')
        self.assertEqual(len(files), 75)
        library = e.read_json(directory / 'shared.geometry.json')
        for file in files:
            original = e.resolve_rig(e.read_json(file), library)
            decoded = self.catalog[original['name']]
            for expected, actual in zip(original['joints'], decoded['joints']):
                self.assertEqual(expected['name'], actual['name'])
                self.assertEqual(expected['parent'], actual['parent'])
                self.assertEqual(e.packed(expected['translation']), e.packed(actual['translation']))
                self.assertEqual(len(expected['primitives']), len(actual['primitives']))
                for before, after in zip(expected['primitives'], actual['primitives']):
                    for key in (*e.GEOMETRY_KEYS, 'base_color'):
                        code = 'I' if key == 'indices' else 'f'
                        self.assertEqual(e.packed(before[key], code), e.packed(after[key], code), (file.name, key))

    def test_layout_rejects_catalog_and_retired_json_family(self):
        import subprocess
        import sys
        import tempfile
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            script = root / 'tools/check-model-layout.py'
            script.parent.mkdir()
            script.write_bytes((REPO / 'tools/check-model-layout.py').read_bytes())
            directory = root / e.MODEL_ROOT / 'johto_characters'
            directory.mkdir(parents=True)
            (directory / e.CATALOG_FILE).write_bytes(b'layout fixture')
            result = subprocess.run([sys.executable, str(script)], capture_output=True)
            self.assertEqual(result.returncode, 0, result.stderr)
            for filename in ('trainer.rig.json', 'shared.geometry.json'):
                duplicate = directory / filename
                duplicate.write_text('{}')
                result = subprocess.run([sys.executable, str(script)], capture_output=True)
                self.assertNotEqual(result.returncode, 0)
                self.assertIn(b'old human JSON family', result.stderr)
                duplicate.unlink()

    def test_reject_accessor_overrun_and_unsupported_encoding(self):
        doc, binary = e.parse_glb(self.human_blob)
        bad = copy.deepcopy(doc)
        bad['accessors'][0]['count'] += len(binary)
        with self.assertRaisesRegex(ValueError, 'outside'):
            e.accessor_values(bad, binary, 0)
        bad = copy.deepcopy(doc)
        bad['accessors'][0]['normalized'] = True
        with self.assertRaisesRegex(ValueError, 'encoding'):
            e.accessor_values(bad, binary, 0)

    def test_reject_bad_library_reference_parent_and_missing_anatomy(self):
        invalid = copy.deepcopy(self.rig)
        invalid['geometry_library'] = 'wrong'
        with self.assertRaisesRegex(ValueError, 'identity'):
            e.export_humans([invalid], self.library)
        invalid = copy.deepcopy(self.rig)
        invalid['joints'][0]['primitives'][0]['geometry'] = len(self.library['geometries'])
        with self.assertRaisesRegex(ValueError, 'reference'):
            e.export_humans([invalid], self.library)
        invalid = copy.deepcopy(self.rig)
        invalid['joints'][0]['parent'] = 1
        with self.assertRaisesRegex(ValueError, 'parent'):
            e.export_humans([invalid], self.library)
        invalid = copy.deepcopy(self.pokemon)
        invalid['primitives'] = [p for p in invalid['primitives'] if not p['part'].startswith('Flight feather')]
        with self.assertRaisesRegex(ValueError, 'four flight feathers'):
            e.export_pidgeotto(invalid)


if __name__ == '__main__':
    unittest.main(verbosity=2)
