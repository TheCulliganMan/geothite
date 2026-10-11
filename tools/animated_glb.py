#!/usr/bin/env python3
"""Shared GLB primitives and the authored Pidgeotto exporter.

Legacy human catalogs and their exporter have been removed.
"""
import argparse
import hashlib
import json
import math
import struct
from pathlib import Path

COORDINATES = '+Y up; front +Z; floor-centered root'
MODEL_ROOT = Path('crates/crystal-voxel-view/models')
GEOMETRY_KEYS = ('positions', 'normals', 'indices')


def canonical(value):
    return json.dumps(value, sort_keys=True, separators=(',', ':'), allow_nan=False).encode()


def sha(data):
    return hashlib.sha256(data).hexdigest()


def packed(values, code='f'):
    return struct.pack('<' + code * len(values), *values)


def f32(value):
    return struct.unpack('<f', struct.pack('<f', value))[0]


def read_json(path):
    return json.loads(Path(path).read_bytes())


def validate_geometry(geometry):
    positions, normals, indices = (geometry[k] for k in GEOMETRY_KEYS)
    if not positions or len(positions) % 3 or len(normals) != len(positions):
        raise ValueError('invalid vertex/normal array lengths')
    if not indices or len(indices) % 3:
        raise ValueError('triangle indices required')
    if any(type(i) is not int or not 0 <= i < len(positions) // 3 for i in indices):
        raise ValueError('index outside vertex array')
    if any(not math.isfinite(v) for v in positions + normals):
        raise ValueError('non-finite geometry')
    for i in range(0, len(normals), 3):
        length = sum(v * v for v in normals[i:i + 3]) ** .5
        if abs(length - 1) > 1e-4:
            raise ValueError('source normal is not unit length')


class Glb:
    def __init__(self):
        self.doc = {'asset': {'version': '2.0', 'generator': 'Geothite deterministic animated GLB v1'},
                    'scene': 0, 'scenes': [], 'nodes': [], 'meshes': [], 'materials': [],
                    'accessors': [], 'bufferViews': [], 'animations': []}
        self.data = bytearray()
        self.geometries = {}
        self.materials = {}
        self.animation_accessors = {}

    def accessor(self, values, kind, component=5126, target=None, bounds=False):
        width = {'SCALAR': 1, 'VEC3': 3, 'VEC4': 4}[kind]
        if len(values) % width or not values:
            raise ValueError('invalid accessor size')
        blob = packed(values, 'f' if component == 5126 else 'I')
        self.data.extend(b'\0' * (-len(self.data) % 4))
        view = {'buffer': 0, 'byteOffset': len(self.data), 'byteLength': len(blob)}
        if target is not None:
            view['target'] = target
        view_id = len(self.doc['bufferViews'])
        self.doc['bufferViews'].append(view)
        self.data.extend(blob)
        accessor = {'bufferView': view_id, 'componentType': component,
                    'count': len(values) // width, 'type': kind}
        if bounds:
            vals = [f32(v) for v in values] if component == 5126 else values
            accessor['min'] = [min(vals[j::width]) for j in range(width)]
            accessor['max'] = [max(vals[j::width]) for j in range(width)]
        index = len(self.doc['accessors'])
        self.doc['accessors'].append(accessor)
        return index

    def geometry(self, primitive):
        # Source-equivalent f32 bits plus exact u32 indices are the cache key.
        key = tuple(packed(primitive[k], 'I' if k == 'indices' else 'f') for k in GEOMETRY_KEYS)
        if key not in self.geometries:
            self.geometries[key] = {
                'attributes': {'POSITION': self.accessor(primitive['positions'], 'VEC3', target=34962, bounds=True),
                               'NORMAL': self.accessor(primitive['normals'], 'VEC3', target=34962)},
                'indices': self.accessor(primitive['indices'], 'SCALAR', 5125, 34963), 'mode': 4}
        return self.geometries[key]

    def material(self, color):
        if len(color) != 4 or any(not math.isfinite(v) or not 0 <= v <= 1 for v in color):
            raise ValueError('invalid RGBA')
        key = tuple(color)
        if key not in self.materials:
            index = len(self.doc['materials'])
            self.materials[key] = index
            self.doc['materials'].append({'name': f'rgba_{index}',
                'pbrMetallicRoughness': {'baseColorFactor': color, 'metallicFactor': 0, 'roughnessFactor': 1},
                'alphaMode': 'BLEND' if color[3] < 1 else 'OPAQUE'})
        return self.materials[key]

    def mesh(self, name, primitives):
        converted = []
        for primitive in primitives:
            converted.append({**self.geometry(primitive), 'material': self.material(primitive['base_color'])})
        index = len(self.doc['meshes'])
        self.doc['meshes'].append({'name': name, 'primitives': converted})
        return index

    def node(self, value, parent=None):
        index = len(self.doc['nodes'])
        self.doc['nodes'].append(value)
        if parent is not None:
            self.doc['nodes'][parent].setdefault('children', []).append(index)
        return index

    def animation_accessor(self, values, kind, bounds=False):
        key = (kind, bounds, packed(values))
        if key not in self.animation_accessors:
            self.animation_accessors[key] = self.accessor(values, kind, bounds=bounds)
        return self.animation_accessors[key]

    def sampled_clip(self, name, times, tracks, extras):
        time_accessor = self.animation_accessor(times, 'SCALAR', bounds=True)
        channels, samplers = [], []
        for node, path, values in tracks:
            width = 4 if path == 'rotation' else 3
            if path not in ('rotation', 'translation', 'scale') or len(values) != len(times) * width:
                raise ValueError('invalid animation track')
            output = self.animation_accessor(values, 'VEC4' if width == 4 else 'VEC3')
            channels.append({'sampler': len(samplers), 'target': {'node': node, 'path': path}})
            samplers.append({'input': time_accessor, 'output': output, 'interpolation': 'LINEAR'})
        self.doc['animations'].append({'name': name, 'channels': channels, 'samplers': samplers, 'extras': extras})

    def clip(self, name, times, tracks):
        time_accessor = self.animation_accessor(times, 'SCALAR', bounds=True)
        channels, samplers = [], []
        for node, axis, angles in tracks:
            if len(angles) != len(times):
                raise ValueError('animation sample count mismatch')
            values = []
            for angle in angles:
                values.extend([v * math.sin(angle / 2) for v in axis] + [math.cos(angle / 2)])
            output = self.animation_accessor(values, 'VEC4')
            channels.append({'sampler': len(samplers), 'target': {'node': node, 'path': 'rotation'}})
            samplers.append({'input': time_accessor, 'output': output, 'interpolation': 'LINEAR'})
        self.doc['animations'].append({'name': name, 'channels': channels, 'samplers': samplers,
            'extras': {'purpose': 'format and articulation test only; not original attack choreography',
                       'loopSuggested': True}})

    def bytes(self):
        self.doc['buffers'] = [{'byteLength': len(self.data)}]
        js = canonical(self.doc)
        js += b' ' * (-len(js) % 4)
        binary = bytes(self.data) + b'\0' * (-len(self.data) % 4)
        return (struct.pack('<III', 0x46546C67, 2, 12 + 8 + len(js) + 8 + len(binary)) +
                struct.pack('<II', len(js), 0x4E4F534A) + js +
                struct.pack('<II', len(binary), 0x004E4942) + binary)


def pidgeotto_groups(model):
    if model['name'] != 'pidgeotto' or model['coordinate_system'] != COORDINATES or model['version'] != 1:
        raise ValueError('this proof requires the named pidgeotto source')
    groups = {'l': [], 'r': []}
    pivots = {}
    for index, primitive in enumerate(model['primitives']):
        validate_geometry(primitive)
        name = primitive['part']
        if name.startswith(('Layered wing', 'Flight feather')):
            x = primitive['positions'][0::3]
            side = 'l' if sum(x) < 0 else 'r'
            if any(v * (-1 if side == 'l' else 1) <= 0 for v in x):
                raise ValueError('wing primitive crosses the centerline')
            groups[side].append(index)
            if name.startswith('Layered wing'):
                points = [primitive['positions'][i:i + 3] for i in range(0, len(primitive['positions']), 3)]
                # The generator's first outlined shoulder corner is the vertex
                # nearest the centerline. This exact exported vertex is the new
                # hinge anchor. No source object origin survives in this JSON.
                pivots[side] = [f32(v) for v in min(points, key=lambda p: (abs(p[0]), -p[1], p[2]))]
    if set(pivots) != {'l', 'r'} or any(len(v) != 5 for v in groups.values()):
        raise ValueError('expected one layered wing and four flight feathers per side')
    return groups, pivots


def pidgeotto_idle_samples():
    """A smooth 0.8-second cycle; duplicate seam keys are exact identities."""
    times = [.8 * i / 64 for i in range(65)]
    tracks = []
    for sign in (1, -1):
        values = []
        for i, time in enumerate(times):
            if i in (0, 64):
                values.extend([0, 0, 0, 1])
            else:
                angle = sign * .42 * math.sin(2 * math.pi * time / .8)
                values.extend([0, 0, math.sin(angle / 2), math.cos(angle / 2)])
        tracks.append(values)
    return times, tracks


def export_pidgeotto(model, animation='demo'):
    if animation not in ('demo', 'idle'):
        raise ValueError('unsupported Pidgeotto animation')
    groups, pivots = pidgeotto_groups(model)
    glb = Glb()
    root = glb.node({'name': 'pidgeotto', 'extras': {'coordinateSystem': COORDINATES,
        'sourceSchemaVersion': 1, 'sourceMeshSha256': sha(canonical(model))}})
    if animation == 'idle':
        glb.doc['nodes'][root]['extras']['rigSchemaVersion'] = 1
    wing_parents, hinges = {}, {}
    for side in ('l', 'r'):
        pivot = pivots[side]
        hinge = glb.node({'name': f'wing_{side}_hinge', 'translation': pivot,
            'extras': {('authoredForRuntime' if animation == 'idle' else 'authoredForProof'): True,
                       'pivotBasis': 'nearest-centerline source layered-wing vertex',
                       'sourcePrimitiveIndices': groups[side]}}, root)
        hinges[side] = hinge
        # Cancellation node keeps every stored POSITION f32 bit untouched and
        # makes the neutral combined transform exactly identity.
        wing_parents[side] = glb.node({'name': f'wing_{side}_source_coordinates',
                                    'translation': [-v for v in pivot]}, hinge)
    for index, primitive in enumerate(model['primitives']):
        side = next((s for s in ('l', 'r') if index in groups[s]), None)
        glb.node({'name': primitive['part'], 'mesh': glb.mesh(primitive['part'], [primitive]),
                  'extras': {'sourcePrimitive': index}}, wing_parents[side] if side else root)
    glb.doc['scenes'] = [{'name': 'pidgeotto', 'nodes': [root]}]
    if animation == 'idle':
        times, tracks = pidgeotto_idle_samples()
        glb.sampled_clip('pidgeotto.idle_wings', times,
                         [(hinges[side], 'rotation', values) for side, values in zip(('l', 'r'), tracks)],
                         {'purpose': 'production idle wing articulation',
                          'loopSuggested': True, 'durationSeconds': .8})
    else:
        angles = [0, .42, 0, -.42, 0]
        glb.clip('pidgeotto.wing_flap_demo', [0, .2, .4, .6, .8], [
            (hinges['l'], [0, 0, 1], angles),
            (hinges['r'], [0, 0, 1], [-v for v in angles])])
    return glb.bytes()


def parse_glb(blob):
    if len(blob) < 28 or struct.unpack_from('<III', blob) != (0x46546C67, 2, len(blob)):
        raise ValueError('bad GLB header')
    offset, chunks = 12, []
    while offset < len(blob):
        if offset + 8 > len(blob):
            raise ValueError('truncated GLB chunk header')
        length, kind = struct.unpack_from('<II', blob, offset)
        if length % 4 or offset + 8 + length > len(blob):
            raise ValueError('bad GLB chunk')
        chunks.append((kind, blob[offset + 8:offset + 8 + length]))
        offset += 8 + length
    if [k for k, _ in chunks] != [0x4E4F534A, 0x004E4942]:
        raise ValueError('expected one JSON and one BIN chunk')
    return json.loads(chunks[0][1]), chunks[1][1]


def accessor_values(doc, binary, index):
    """Read our tightly packed, embedded f32/u32 accessor subset with bounds checks."""
    try:
        if type(index) is not int or not 0 <= index < len(doc['accessors']):
            raise ValueError('invalid accessor reference')
        a = doc['accessors'][index]
        if 'sparse' in a or a.get('normalized', False):
            raise ValueError('unsupported accessor encoding')
        view_index = a['bufferView']
        if type(view_index) is not int or not 0 <= view_index < len(doc['bufferViews']):
            raise ValueError('invalid buffer view reference')
        view = doc['bufferViews'][view_index]
        if view.get('buffer') != 0 or 'byteStride' in view:
            raise ValueError('expected tightly packed embedded buffer')
        width = {'SCALAR': 1, 'VEC3': 3, 'VEC4': 4}[a['type']]
        code = {5126: 'f', 5125: 'I'}[a['componentType']]
        count = a['count']
        start, size, relative = view.get('byteOffset', 0), view['byteLength'], a.get('byteOffset', 0)
        if any(type(v) is not int or v < 0 for v in (start, size, relative, count)) or not count:
            raise ValueError('invalid accessor bounds')
        length = count * width * 4
        if start % 4 or relative % 4 or relative + length > size or start + size > len(binary):
            raise ValueError('accessor outside buffer view')
        values = list(struct.unpack_from('<' + code * count * width, binary, start + relative))
        if code == 'f' and any(not math.isfinite(v) for v in values):
            raise ValueError('non-finite accessor')
        return values
    except (KeyError, IndexError, TypeError, struct.error) as error:
        raise ValueError('unsupported or malformed accessor') from error
