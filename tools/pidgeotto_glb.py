#!/usr/bin/env python3
"""Canonical rigid Pidgeotto GLB, with a strict source-equivalent neutral reader.

The ten wing parts inherit shoulder rotations through cancellation nodes. All
34 anatomy parts retain their original POSITION/NORMAL f32 and index bits.
This intentionally supports one small production schema, not arbitrary glTF.
"""
import argparse
import json
import math
import re
import struct
from pathlib import Path

import animated_glb as glb

FILE = 'pidgeotto.glb'
MAX_BYTES = 1024 * 1024
PART_COUNT = 34
CLIP_NAME = 'pidgeotto.idle_wings'
PIVOT_BASIS = 'nearest-centerline source layered-wing vertex'
CLIP_EXTRAS = {'purpose': 'production idle wing articulation',
               'loopSuggested': True, 'durationSeconds': .8}


def fields(value, required, optional=()):
    if not isinstance(value, dict) or not set(required) <= set(value) <= set(required) | set(optional):
        raise ValueError('unsupported or missing Pidgeotto schema fields')


def reference(value, values):
    if type(value) is not int or not 0 <= value < len(values):
        raise ValueError('invalid Pidgeotto reference')
    return values[value]


def exact(actual, expected, message):
    if (actual != expected or isinstance(actual, bool) != isinstance(expected, bool)
            or (type(expected) is int and type(actual) is not int)):
        raise ValueError(message)
    if isinstance(expected, dict):
        for key in expected:
            exact(actual[key], expected[key], message)
    elif isinstance(expected, (tuple, list)):
        for a, b in zip(actual, expected):
            exact(a, b, message)


def same_f32(actual, expected, message):
    if (any(type(v) not in (int, float) or not math.isfinite(v) for v in actual)
            or glb.packed(actual) != glb.packed(expected)):
        raise ValueError(message)


def export_pidgeotto(model):
    """Export production idle motion; demo callers keep their existing API."""
    fields(model, ('name', 'version', 'coordinate_system', 'primitives'))
    if len(model['primitives']) != PART_COUNT:
        raise ValueError('Pidgeotto requires all 34 source anatomy parts')
    names = []
    for primitive in model['primitives']:
        fields(primitive, ('part', 'positions', 'normals', 'indices', 'base_color'))
        name = primitive['part']
        if not isinstance(name, str) or not name or name in names:
            raise ValueError('missing or duplicate source anatomy name')
        names.append(name)
    blob = glb.export_pidgeotto(model, animation='idle')
    decode_pidgeotto(blob)
    return blob


def _unique_object(pairs):
    result = {}
    for key, value in pairs:
        if key in result:
            raise ValueError('duplicate Pidgeotto JSON field')
        result[key] = value
    return result


def decode_pidgeotto(blob):
    """Bounded, fail-closed decode of the neutral model and its canonical clip."""
    try:
        if not 0 < len(blob) <= MAX_BYTES:
            raise ValueError('Pidgeotto GLB exceeds bounded input size')
        _, binary = glb.parse_glb(blob)
        json_length = struct.unpack_from('<I', blob, 12)[0]
        doc = json.loads(blob[20:20 + json_length], object_pairs_hook=_unique_object)
        fields(doc, ('asset', 'scene', 'scenes', 'nodes', 'meshes', 'materials',
                     'accessors', 'bufferViews', 'animations', 'buffers'))
        fields(doc['asset'], ('version', 'generator'))
        exact(doc['asset']['version'], '2.0', 'unsupported Pidgeotto glTF version')
        exact(doc['scene'], 0, 'unsupported Pidgeotto default scene')
        exact(doc['scenes'], [{'name': 'pidgeotto', 'nodes': [0]}], 'unsupported Pidgeotto scenes')
        if len(doc['buffers']) != 1:
            raise ValueError('expected one embedded Pidgeotto buffer')
        fields(doc['buffers'][0], ('byteLength',))
        length = doc['buffers'][0]['byteLength']
        if type(length) is not int or length != len(binary):
            raise ValueError('invalid Pidgeotto embedded buffer length')
        nodes, meshes = doc['nodes'], doc['meshes']
        if len(nodes) != PART_COUNT + 5 or len(meshes) != PART_COUNT:
            raise ValueError('Pidgeotto requires 39 nodes and 34 anatomy meshes')
        if not 1 <= len(doc['materials']) <= PART_COUNT or not 3 <= len(doc['accessors']) <= 3 * PART_COUNT + 3:
            raise ValueError('invalid Pidgeotto material or accessor count')
        if len(doc['bufferViews']) != len(doc['accessors']):
            raise ValueError('unexpected Pidgeotto buffer views')

        used_accessors, used_materials, used_views = set(), set(), set()
        def read_accessor(index, kind, component=5126, target=None, bounds=False):
            a = reference(index, doc['accessors'])
            fields(a, ('bufferView', 'componentType', 'count', 'type'), ('min', 'max'))
            exact((a['type'], a['componentType']), (kind, component), 'unsupported Pidgeotto accessor encoding')
            v = reference(a['bufferView'], doc['bufferViews'])
            fields(v, ('buffer', 'byteOffset', 'byteLength'), ('target',))
            if type(v['buffer']) is not int or v['buffer'] != 0 or v.get('target') != target:
                raise ValueError('unsupported Pidgeotto buffer view')
            if type(a['count']) is not int or not 1 <= a['count'] <= MAX_BYTES // 4:
                raise ValueError('invalid Pidgeotto accessor count')
            width = {'SCALAR': 1, 'VEC3': 3, 'VEC4': 4}[kind]
            if v['byteLength'] != a['count'] * width * 4:
                raise ValueError('Pidgeotto accessor does not fill its view')
            values = glb.accessor_values(doc, binary, index)
            if bounds:
                exact(a.get('min'), [min(values[i::width]) for i in range(width)], 'invalid Pidgeotto accessor minimum')
                exact(a.get('max'), [max(values[i::width]) for i in range(width)], 'invalid Pidgeotto accessor maximum')
            elif 'min' in a or 'max' in a:
                raise ValueError('unexpected Pidgeotto accessor bounds')
            used_accessors.add(index)
            used_views.add(a['bufferView'])
            return values

        colors = []
        for index, material in enumerate(doc['materials']):
            fields(material, ('name', 'pbrMetallicRoughness', 'alphaMode'))
            exact(material['name'], f'rgba_{index}', 'unsupported Pidgeotto material name')
            pbr = material['pbrMetallicRoughness']
            fields(pbr, ('baseColorFactor', 'metallicFactor', 'roughnessFactor'))
            exact((pbr['metallicFactor'], pbr['roughnessFactor']), (0, 1), 'unsupported Pidgeotto material shading')
            color = pbr['baseColorFactor']
            if len(color) != 4 or any(type(v) not in (int, float) or not math.isfinite(v) or not 0 <= v <= 1 for v in color):
                raise ValueError('invalid Pidgeotto RGBA')
            exact(material['alphaMode'], 'BLEND' if color[3] < 1 else 'OPAQUE', 'unsupported Pidgeotto alpha mode')
            colors.append(color)

        root = nodes[0]
        fields(root, ('name', 'extras', 'children'))
        exact(root['name'], 'pidgeotto', 'invalid Pidgeotto root')
        extras = root['extras']
        fields(extras, ('coordinateSystem', 'sourceSchemaVersion', 'sourceMeshSha256', 'rigSchemaVersion'))
        exact(extras['coordinateSystem'], glb.COORDINATES, 'unsupported Pidgeotto coordinate system')
        exact((extras['sourceSchemaVersion'], extras['rigSchemaVersion']), (1, 1), 'unsupported Pidgeotto schema version')
        if not isinstance(extras['sourceMeshSha256'], str) or not re.fullmatch('[0-9a-f]{64}', extras['sourceMeshSha256']):
            raise ValueError('invalid Pidgeotto source provenance')

        primitives, names = [], set()
        for index, node in enumerate(nodes[5:]):
            fields(node, ('name', 'mesh', 'extras'))
            name = node['name']
            if not isinstance(name, str) or not name or name in names:
                raise ValueError('missing or duplicate Pidgeotto anatomy name')
            names.add(name)
            if type(node['mesh']) is not int or node['mesh'] != index:
                raise ValueError('Pidgeotto mesh source order changed')
            exact(node['extras'], {'sourcePrimitive': index}, 'invalid Pidgeotto source primitive identity')
            mesh = meshes[index]
            fields(mesh, ('name', 'primitives'))
            exact(mesh['name'], name, 'Pidgeotto mesh and anatomy name differ')
            if len(mesh['primitives']) != 1:
                raise ValueError('expected one primitive per Pidgeotto anatomy mesh')
            primitive = mesh['primitives'][0]
            fields(primitive, ('attributes', 'indices', 'mode', 'material'))
            exact(primitive['mode'], 4, 'Pidgeotto requires triangle primitives')
            fields(primitive['attributes'], ('POSITION', 'NORMAL'))
            geometry = {
                'positions': read_accessor(primitive['attributes']['POSITION'], 'VEC3', target=34962, bounds=True),
                'normals': read_accessor(primitive['attributes']['NORMAL'], 'VEC3', target=34962),
                'indices': read_accessor(primitive['indices'], 'SCALAR', 5125, 34963)}
            glb.validate_geometry(geometry)
            color = reference(primitive['material'], colors)
            used_materials.add(primitive['material'])
            primitives.append({'part': name, **geometry, 'base_color': color})
        model = {'name': 'pidgeotto', 'version': 1, 'coordinate_system': glb.COORDINATES,
                 'primitives': primitives}
        groups, pivots = glb.pidgeotto_groups(model)
        wing_members = groups['l'] + groups['r']
        exact(root['children'], [1, 3] + [i + 5 for i in range(PART_COUNT) if i not in wing_members],
              'invalid Pidgeotto root hierarchy')
        for side, hinge_index in (('l', 1), ('r', 3)):
            hinge, cancellation = nodes[hinge_index:hinge_index + 2]
            fields(hinge, ('name', 'translation', 'extras', 'children'))
            fields(cancellation, ('name', 'translation', 'children'))
            exact(hinge['name'], f'wing_{side}_hinge', 'invalid Pidgeotto hinge')
            exact(cancellation['name'], f'wing_{side}_source_coordinates', 'invalid Pidgeotto cancellation node')
            same_f32(hinge['translation'], pivots[side], 'Pidgeotto shoulder pivot changed')
            same_f32(cancellation['translation'], [-v for v in pivots[side]], 'Pidgeotto neutral translation does not cancel')
            exact(hinge['children'], [hinge_index + 1], 'invalid Pidgeotto hinge hierarchy')
            exact(cancellation['children'], [i + 5 for i in groups[side]], 'invalid Pidgeotto wing membership')
            exact(hinge['extras'], {'authoredForRuntime': True, 'pivotBasis': PIVOT_BASIS,
                                   'sourcePrimitiveIndices': groups[side]}, 'invalid Pidgeotto hinge metadata')

        if len(doc['animations']) != 1:
            raise ValueError('expected exactly one Pidgeotto idle clip')
        clip = doc['animations'][0]
        fields(clip, ('name', 'channels', 'samplers', 'extras'))
        exact(clip['name'], CLIP_NAME, 'missing canonical Pidgeotto idle clip')
        exact(clip['extras'], CLIP_EXTRAS, 'unsupported Pidgeotto idle metadata')
        exact(clip['channels'], [{'sampler': i, 'target': {'node': node, 'path': 'rotation'}}
                                for i, node in enumerate((1, 3))], 'unsupported Pidgeotto animation target')
        if len(clip['samplers']) != 2:
            raise ValueError('expected two Pidgeotto wing samplers')
        times, tracks = glb.pidgeotto_idle_samples()
        inputs = []
        for sampler, expected in zip(clip['samplers'], tracks):
            fields(sampler, ('input', 'output', 'interpolation'))
            exact(sampler['interpolation'], 'LINEAR', 'unsupported Pidgeotto interpolation')
            same_f32(read_accessor(sampler['input'], 'SCALAR', bounds=True), times, 'invalid Pidgeotto idle sample times')
            values = read_accessor(sampler['output'], 'VEC4')
            same_f32(values, expected, 'invalid Pidgeotto idle quaternion samples')
            same_f32(values[:4], [0, 0, 0, 1], 'non-neutral Pidgeotto first key')
            same_f32(values[-4:], [0, 0, 0, 1], 'non-neutral Pidgeotto closing key')
            inputs.append(sampler['input'])
        exact(inputs[0], inputs[1], 'Pidgeotto wing timelines must share one accessor')
        for used, values in ((used_accessors, doc['accessors']), (used_views, doc['bufferViews']),
                             (used_materials, doc['materials'])):
            exact(used, set(range(len(values))), 'unreferenced Pidgeotto data')
        offset = 0
        for view in doc['bufferViews']:
            exact(view['byteOffset'], offset, 'overlapping or gapped Pidgeotto binary data')
            offset += view['byteLength']
        exact(offset, len(binary), 'unreferenced Pidgeotto binary data')
        return model
    except (KeyError, IndexError, TypeError, OverflowError, struct.error, RecursionError) as error:
        raise ValueError('malformed or unsupported Pidgeotto GLB') from error


def read_pidgeotto(path):
    with Path(path).open('rb') as source:
        return decode_pidgeotto(source.read(MAX_BYTES + 1))


def load_pidgeotto(directory):
    """Read a transient source when authoring, otherwise the canonical GLB."""
    source = Path(directory) / 'pidgeotto.mesh.json'
    if source.exists():
        from model_asset_storage import read_model_json
        return read_model_json(source)
    return read_pidgeotto(Path(directory) / FILE)


def species_paths(directory):
    """Compatibility entry point; canonical selection is shared by all species."""
    from battle_model_assets import species_paths as canonical_paths
    return canonical_paths(directory)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('source', type=Path, help='transient source mesh JSON')
    parser.add_argument('output', type=Path, help='canonical pidgeotto.glb')
    args = parser.parse_args()
    from model_asset_storage import read_model_json
    blob = export_pidgeotto(read_model_json(args.source))
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_bytes(blob)
    print(f'Wrote {args.output}: {len(blob):,} bytes; 34 anatomy parts; 65-key idle wings')


if __name__ == '__main__':
    main()
