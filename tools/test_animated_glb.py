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
        from pidgeotto_glb import load_pidgeotto
        cls.pokemon = load_pidgeotto(models / 'battle_species')
        cls.pokemon_blob = e.export_pidgeotto(cls.pokemon)

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

if __name__ == '__main__':
    unittest.main(verbosity=2)
