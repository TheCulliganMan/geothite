#!/usr/bin/env python3
"""Deterministic stdlib-only animated GLB exporter for the human catalog.

Rigid anatomy is animated with glTF node rotations. No pretend skin, baked
root motion, vertex welding, decimation, normal repair or geometry recentering.
The human catalog is canonical. JSON rigs are transient authoring inputs;
optional Pokémon articulation proofs belong in ignored build output.
"""
import argparse
import hashlib
import json
import math
import struct
from pathlib import Path
from functools import lru_cache

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


def validate_library(library):
    if set(library) != {'version', 'id', 'geometries'} or library['version'] != 1:
        raise ValueError('unsupported library')
    if sha(canonical({'version': 1, 'geometries': library['geometries']})) != library['id']:
        raise ValueError('library content identity mismatch')
    for geometry in library['geometries']:
        if set(geometry) != set(GEOMETRY_KEYS):
            raise ValueError('unexpected library geometry field')
        validate_geometry(geometry)


def resolve_rig(rig, library):
    if rig['coordinate_system'] != COORDINATES:
        raise ValueError('unsupported coordinate system')
    version = rig['version']
    if version not in (1, 2):
        raise ValueError('unsupported rig version')
    if version == 2 and rig.get('geometry_library') != library['id']:
        raise ValueError('rig/library identity mismatch')
    names = set()
    joints = []
    for i, joint in enumerate(rig['joints']):
        name, parent, translation = (joint[k] for k in ('name', 'parent', 'translation'))
        if name in names or (parent is not None and (type(parent) is not int or not 0 <= parent < i)):
            raise ValueError('duplicate joint or invalid parent order')
        names.add(name)
        if len(translation) != 3 or any(not math.isfinite(v) for v in translation):
            raise ValueError('invalid bind translation')
        primitives = []
        for p in joint['primitives']:
            if version == 2:
                if set(p) != {'geometry', 'base_color'} or type(p['geometry']) is not int or not 0 <= p['geometry'] < len(library['geometries']):
                    raise ValueError('invalid geometry reference')
                geometry = library['geometries'][p['geometry']]
            else:
                if set(p) != {*GEOMETRY_KEYS, 'base_color'}:
                    raise ValueError('invalid inline primitive')
                geometry = {key: p[key] for key in GEOMETRY_KEYS}
            validate_geometry(geometry)
            primitives.append({**geometry, 'base_color': p['base_color']})
        if not primitives:
            raise ValueError('joint without geometry')
        joints.append({**joint, 'primitives': primitives})
    return {**rig, 'joints': joints}


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


def export_humans(rigs, library, animations='demo'):
    """One scene per rig; one shared accessor set per geometry across all scenes."""
    validate_library(library)
    if animations not in ('demo', 'locomotion', 'none'):
        raise ValueError('unknown human animation mode')
    glb = Glb()
    for rig in sorted(rigs, key=lambda r: r['name']):
        expanded = resolve_rig(rig, library)
        roots, node_ids = [], {}
        for i, joint in enumerate(expanded['joints']):
            original = rig['joints'][i]
            parent = node_ids[original['parent']] if original['parent'] is not None else None
            value = {'name': joint['name'], 'translation': joint['translation'],
                'mesh': glb.mesh(f"{rig['name']}/{joint['name']}", joint['primitives']),
                'extras': {'sourceJoint': i, 'sourceParent': joint['parent'],
                           'sourceGeometry': [p.get('geometry') for p in original['primitives']]}}
            node_ids[i] = glb.node(value, parent)
            if parent is None:
                roots.append(node_ids[i])
        glb.doc['scenes'].append({'name': rig['name'], 'nodes': roots,
            'extras': {'sourceSchemaVersion': rig['version'], 'geometryLibrary': rig.get('geometry_library'),
                       'coordinateSystem': rig['coordinate_system'], 'sourceRigSha256': sha(canonical(rig))}})
        by_name = {joint['name']: node_ids[i] for i, joint in enumerate(rig['joints'])}
        required = ('upper_arm_r', 'forearm_r', 'hand_r')
        if animations == 'locomotion':
            from human_locomotion import locomotion_clips
            binds = {joint['name']: joint['translation'] for joint in rig['joints']}
            for clip in locomotion_clips(binds):
                glb.sampled_clip(f"{rig['name']}.{clip['name']}", clip['times'],
                                 [(by_name[t['joint']], t['path'], t['values']) for t in clip['tracks']],
                                 clip['extras'])
        if animations == 'demo' and all(name in by_name for name in required):
            glb.clip(f"{rig['name']}.rig_wave_demo", [0, .4, .8, 1.2, 1.6], [
                (by_name['upper_arm_r'], [0, 0, 1], [0, 1.05, 1.05, 1.05, 0]),
                (by_name['forearm_r'], [0, 0, 1], [0, .45, .9, .45, 0]),
                (by_name['hand_r'], [0, 1, 0], [0, -.35, .35, -.35, 0])])
    return glb.bytes()


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


def export_pidgeotto(model):
    groups, pivots = pidgeotto_groups(model)
    glb = Glb()
    root = glb.node({'name': 'pidgeotto', 'extras': {'coordinateSystem': COORDINATES,
        'sourceSchemaVersion': 1, 'sourceMeshSha256': sha(canonical(model))}})
    wing_parents, hinges = {}, {}
    for side in ('l', 'r'):
        pivot = pivots[side]
        hinge = glb.node({'name': f'wing_{side}_hinge', 'translation': pivot,
            'extras': {'authoredForProof': True, 'pivotBasis': 'nearest-centerline source layered-wing vertex',
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


CATALOG_FILE = 'catalog.glb'


def read_catalog(path):
    """Read canonical GLB scenes as expanded rigs; geometry arrays stay shared.

    Consumers must treat the returned data as read-only. No old JSON file or
    Blender installation is needed. Only the exported rigid-joint subset is
    accepted, so a future unsupported transform cannot silently alter checks.
    """
    doc, binary = parse_glb(Path(path).read_bytes())
    if doc.get('asset', {}).get('version') != '2.0' or doc.get('skins') or doc.get('extensionsRequired'):
        raise ValueError('unsupported character GLB schema')
    buffers = doc.get('buffers', [])
    if len(buffers) != 1 or 'uri' in buffers[0] or not 0 <= len(binary) - buffers[0]['byteLength'] <= 3:
        raise ValueError('expected one complete embedded GLB buffer')
    nodes = doc['nodes']
    parents = {}
    for parent, node in enumerate(nodes):
        if any(k in node for k in ('matrix', 'rotation', 'scale', 'skin', 'weights')):
            raise ValueError('unsupported bind transform or skin')
        for child in node.get('children', []):
            if type(child) is not int or not parent < child < len(nodes) or child in parents:
                raise ValueError('invalid parent order or duplicate child')
            parents[child] = parent
    geometry_cache, materials = {}, doc['materials']
    result, claimed = {}, set()
    node_scene = {}
    for scene in doc['scenes']:
        name = scene['name']
        if not isinstance(name, str) or not name or name in result:
            raise ValueError('duplicate or missing scene name')
        if scene.get('extras', {}).get('coordinateSystem') != COORDINATES:
            raise ValueError('unsupported coordinate system')
        roots = scene['nodes']
        if not roots or any(type(i) is not int or not 0 <= i < len(nodes) or i in parents for i in roots):
            raise ValueError('invalid scene roots')
        pending, members = list(roots), set()
        while pending:
            index = pending.pop()
            if index in members or index in claimed:
                raise ValueError('node reused between scenes or roots')
            members.add(index)
            pending.extend(nodes[index].get('children', []))
        claimed.update(members)
        ordered = sorted(members)
        local_ids = {index: local for local, index in enumerate(ordered)}
        joints, joint_names = [], set()
        for index in ordered:
            node = nodes[index]
            joint_name = node['name']
            if joint_name in joint_names:
                raise ValueError('duplicate joint name')
            joint_names.add(joint_name)
            translation = node.get('translation', [0, 0, 0])
            if len(translation) != 3 or any(not math.isfinite(v) for v in translation):
                raise ValueError('invalid bind translation')
            mesh = doc['meshes'][node['mesh']]
            if 'weights' in mesh or not mesh['primitives']:
                raise ValueError('missing rigid geometry or morph weights')
            primitives = []
            for primitive in mesh['primitives']:
                if primitive.get('mode', 4) != 4 or set(primitive['attributes']) != {'POSITION', 'NORMAL'} or 'targets' in primitive:
                    raise ValueError('unsupported mesh primitive')
                key = (primitive['attributes']['POSITION'], primitive['attributes']['NORMAL'], primitive['indices'])
                if key not in geometry_cache:
                    for ref, kind, component in zip(key, ('VEC3', 'VEC3', 'SCALAR'), (5126, 5126, 5125)):
                        accessor = doc['accessors'][ref]
                        if accessor['type'] != kind or accessor['componentType'] != component:
                            raise ValueError('unsupported geometry accessor')
                    geometry = dict(zip(GEOMETRY_KEYS, (accessor_values(doc, binary, ref) for ref in key)))
                    validate_geometry(geometry)
                    geometry_cache[key] = geometry
                material = materials[primitive['material']]
                color = material['pbrMetallicRoughness']['baseColorFactor']
                if len(color) != 4 or any(not math.isfinite(v) or not 0 <= v <= 1 for v in color):
                    raise ValueError('invalid RGBA')
                primitives.append({**geometry_cache[key], 'base_color': color})
            parent = parents.get(index)
            joints.append({'name': joint_name, 'parent': local_ids[parent] if parent is not None else None,
                           'translation': translation, 'primitives': primitives})
            node_scene[index] = name
        result[name] = {'version': 1, 'name': name, 'coordinate_system': COORDINATES, 'joints': joints}
    if claimed != set(range(len(nodes))):
        raise ValueError('unreferenced character nodes')
    names = set()
    for clip in doc.get('animations', []):
        if clip['name'] in names:
            raise ValueError('duplicate animation name')
        names.add(clip['name'])
        scene_names, targets = set(), set()
        for channel in clip['channels']:
            node = channel['target']['node']
            path = channel['target']['path']
            if node not in node_scene or path not in ('rotation', 'translation', 'scale') or (node, path) in targets:
                raise ValueError('unsupported animation target')
            targets.add((node, path))
            scene_names.add(node_scene[node])
            sampler = clip['samplers'][channel['sampler']]
            if sampler.get('interpolation', 'LINEAR') != 'LINEAR':
                raise ValueError('unsupported animation interpolation')
            times = accessor_values(doc, binary, sampler['input'])
            values = accessor_values(doc, binary, sampler['output'])
            width = 4 if path == 'rotation' else 3
            input_accessor, output_accessor = (doc['accessors'][sampler[key]] for key in ('input', 'output'))
            if input_accessor['type'] != 'SCALAR' or output_accessor['type'] != ('VEC4' if width == 4 else 'VEC3'):
                raise ValueError('invalid animation accessor type')
            if input_accessor['componentType'] != 5126 or output_accessor['componentType'] != 5126:
                raise ValueError('invalid animation component type')
            if times != sorted(set(times)) or times[0] < 0 or len(values) != len(times) * width:
                raise ValueError('invalid animation samples')
            if path == 'rotation' and any(abs(sum(v*v for v in values[i:i+4]) - 1) > 1e-5 for i in range(0, len(values), 4)):
                raise ValueError('invalid animation quaternion')
        if len(scene_names) != 1:
            raise ValueError('animation crosses scene boundary')
    return result


@lru_cache(maxsize=4)
def _cached_catalog(path, modified, size):
    return read_catalog(path)


def load_catalog(directory):
    path = (Path(directory) / CATALOG_FILE).resolve()
    stat = path.stat()
    return _cached_catalog(str(path), stat.st_mtime_ns, stat.st_size)


def load_rig(directory, name):
    return load_catalog(directory)[name]


def export_directory(directory, output):
    """Convert transient authored rigs; no JSON sidecars are retained."""
    from johto_character_geometry import compact_models, read_models
    expanded = read_models(Path(directory))
    if not expanded:
        raise ValueError('no authored human rigs found')
    library, rigs = compact_models(expanded)
    blob = export_humans(list(rigs.values()), library, animations='locomotion')
    output = Path(output)
    output.parent.mkdir(parents=True, exist_ok=True)
    output.write_bytes(blob)
    return blob


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--rig-directory', type=Path, help='transient JSON authoring inputs')
    parser.add_argument('--output', type=Path, help='canonical catalog.glb destination')
    # Compatibility entry point for the optional native proof example.
    parser.add_argument('--repo', type=Path)
    parser.add_argument('--out', type=Path)
    parser.add_argument('--human-catalog', action='store_true')
    args = parser.parse_args()
    if args.rig_directory is not None:
        if args.output is None or args.repo is not None or args.out is not None:
            parser.error('--rig-directory requires --output and cannot combine with --repo/--out')
        output = {args.output: export_directory(args.rig_directory, args.output)}
    elif args.repo is not None and args.out is not None and args.output is None:
        models = args.repo / MODEL_ROOT
        directory = models / 'johto_characters'
        if (directory / CATALOG_FILE).exists():
            from johto_character_geometry import compact_models
            library, packed_rigs = compact_models({name + '.rig.json': rig for name, rig in load_catalog(directory).items()})
            rigs = list(packed_rigs.values())
        else:
            library = read_json(directory / 'shared.geometry.json')
            rigs = [read_json(p) for p in sorted(directory.glob('*.rig.json'))]
        trainer = next(rig for rig in rigs if rig['name'] == 'trainer')
        output = {args.out / 'trainer.glb': export_humans([trainer], library),
                  args.out / 'pidgeotto.glb': export_pidgeotto(read_json(models / 'battle_species/pidgeotto.mesh.json'))}
        if args.human_catalog:
            output[args.out / 'johto_characters.glb'] = export_humans(rigs, library)
    else:
        parser.error('use --rig-directory/--output or --repo/--out')
    for path, blob in output.items():
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_bytes(blob)
        doc, _ = parse_glb(blob)
        print(json.dumps({'file': str(path), 'bytes': len(blob), 'sha256': sha(blob),
                          'scenes': len(doc['scenes']), 'animations': len(doc['animations'])}))


if __name__ == '__main__':
    main()
