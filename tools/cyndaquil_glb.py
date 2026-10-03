#!/usr/bin/env python3
"""Articulate the refined Cyndaquil sculpture without changing its neutral mesh.

The common binary writer and math helpers are imported from gengar_glb; anatomy,
weights, hinges, and motion here are specific to Cyndaquil. No Blender or third
party Python package is required. Named source primitives may be supplied by the
recipe. The retired nameless JSON can only be mapped when its geometry digest
matches the individually reviewed sculpture.
"""
import argparse
import copy
import hashlib
import json
import math
import struct
from pathlib import Path

from animated_glb import parse_glb
from skin_glb import (COORDINATES, Glb, envelope, packed,
                        quantized_weights, quaternion_xyz, smoothstep)

REST_DIGEST = '10b1c66e9e91a1f3b590a914278f4330cd44d77797c74c12706eab6a0f14202e'
FILE = 'cyndaquil.glb'
PARTS = (
    'Small soft charcoal nose',
    'Tapered sleepy closed eyelid',
    'Tapered sleepy closed eyelid.001',
    'Fitted midnight brow nape and back',
    'Upper right flame quill / ember silhouette',
    'Lower right flame quill / ember silhouette',
    'Central rear flame quill / ember silhouette',
    'Lower left flame quill / ember silhouette',
    'Upper left flame quill / ember silhouette',
    'Cyndaquil / continuous muzzle head torso and limbs',
    'Lower left flame quill / back orange fold',
    'Upper left flame quill / back orange fold',
    'Upper right flame quill / front orange fold',
    'Lower right flame quill / front orange fold',
    'Central rear flame quill / front orange fold',
    'Central rear flame quill / back orange fold',
    'Upper right flame quill / back orange fold',
    'Lower right flame quill / back orange fold',
    'Lower left flame quill / front orange fold',
    'Upper left flame quill / front orange fold',
    'Lower left flame quill / back golden heart',
    'Upper left flame quill / back golden heart',
    'Upper right flame quill / front golden heart',
    'Lower right flame quill / front golden heart',
    'Central rear flame quill / front golden heart',
    'Central rear flame quill / back golden heart',
    'Lower right flame quill / back golden heart',
    'Upper right flame quill / back golden heart',
    'Lower left flame quill / front golden heart',
    'Upper left flame quill / front golden heart',
)

# These pivots are in the existing runtime coordinate system. The five flame
# roots are the actual recipe roots transformed by the source object's bind.
SOURCE_SCALE = .8989177942276001
SOURCE_OFFSET = (.002550057601183653, -.001327162142843008, -.008687515743076801)


def source_point(x, y, z):
    return tuple(round(v, 6) for v in (SOURCE_OFFSET[0] + SOURCE_SCALE * x,
                                     SOURCE_OFFSET[1] + SOURCE_SCALE * z,
                                     SOURCE_OFFSET[2] - SOURCE_SCALE * y))


QUILLS = (
    ('Upper left flame quill', (-.071, .060, .604)),
    ('Upper right flame quill', (.071, .060, .604)),
    ('Lower left flame quill', (-.138, .186, .450)),
    ('Lower right flame quill', (.138, .186, .450)),
    ('Central rear flame quill', (0., .252, .432)),
)
JOINTS = (
    ('root', None, (0., 0., 0.)),
    ('torso', 0, (.002550, .175, -.025)),
    ('head', 1, (.002550, .435, .075)),
    ('foreleg_left', 1, source_point(-.181, -.085, .385)),
    ('foreleg_right', 1, source_point(.181, -.085, .385)),
) + tuple((name.lower().replace(' ', '_'), 1, source_point(*root))
          for name, root in QUILLS)
CLIPS = (('idle', 3.2, 65), ('attack', 1., 49), ('hit', 1., 49))


def geometry_digest(model):
    digest = hashlib.sha256()
    for part in model['primitives']:
        for key, component in (('positions', 5126), ('normals', 5126),
                               ('indices', 5125), ('base_color', 5126)):
            digest.update(packed(part[key], component))
    return digest.hexdigest()


def named_model(model):
    """Names came from geometric matching to the real editable Blender objects."""
    if model.get('name') not in ('battle_cyndaquil', 'cyndaquil') or model.get('version') != 1:
        raise ValueError('expected Cyndaquil source schema version 1')
    if model.get('coordinate_system') != COORDINATES or len(model.get('primitives', [])) != len(PARTS):
        raise ValueError('expected refined Cyndaquil coordinates and 30 anatomy parts')
    if geometry_digest(model) != REST_DIGEST:
        raise ValueError('unreviewed neutral geometry; anatomy mapping must be reviewed again')
    result = copy.deepcopy(model)
    for source, name in zip(result['primitives'], PARTS):
        if 'part' in source and source['part'] != name:
            raise ValueError('anatomy name does not match reviewed source geometry')
        source['part'] = name
    return result


def body_weights(position):
    """One continuous field on skin, fitted coat, nose, and closed eyelids.

    Grounded rear paws stay on the identity root. Raised forepaws have their own
    shoulder hinges. The head field is rigid over both facial patches and their
    underlying cheeks, then smoothly meets the nape. Coincident corner vertices
    receive identical influences regardless of primitive or normal splits.
    """
    x, y, z = position
    side = 3 if x < .002550 else 4
    arm = (smoothstep(.130, .182, abs(x - .002550))
           * smoothstep(.040, .110, z) * smoothstep(.130, .178, y)
           * (1. - smoothstep(.265, .360, y)))
    upper = max(smoothstep(.105, .245, y), arm)
    head = smoothstep(.025, .170, z) * smoothstep(.310, .430, y)
    active = [(j, w) for j, w in ((0, 1. - upper),
               (1, upper * (1. - head) * (1. - arm)),
               (2, upper * head * (1. - arm)), (side, upper * arm)) if w > 0.]
    ids = [j for j, _ in active]
    values = [w for _, w in active]
    return ids + [0] * (4 - len(ids)), values + [0.] * (4 - len(values))


def skin_weights(part, position):
    # Each five-layer folded quill is a single rigid paper assembly. A root
    # embedded in the torso prevents an opening seam at these tiny amplitudes.
    for i, (name, _) in enumerate(QUILLS, 5):
        if part.startswith(name + ' / '):
            return [i, 0, 0, 0], [1., 0., 0., 0.]
    return body_weights(position)


def pose_angles(clip, t):
    """Local articulation only; the source move/HP clocks choose clip progress."""
    if clip not in ('idle', 'attack', 'hit') or not 0. <= t <= 1.:
        raise ValueError('unknown clip or normalized time outside [0, 1]')
    if t in (0., 1.):
        return [(0., 0., 0.)] * len(JOINTS)
    a, b = math.sin(math.tau * t), math.sin(2. * math.tau * t)
    if clip == 'idle':
        nod = .5 - .5 * math.cos(math.tau * t)
        pose = [(0., 0., 0.), (.006 * a, 0., .004 * b),
                (-.022 * nod, .032 * a, .014 * a),
                (-.035 * nod, .010 * b, -.017 * a),
                (-.026 * nod, -.008 * b, .013 * a)]
        pose += [(.010 * a + .003 * b, .003 * b, -.010 * a),
                 (.008 * a - .003 * b, -.003 * b, .010 * a),
                 (.009 * b, .004 * a, -.008 * b),
                 (-.008 * b, -.004 * a, .008 * b),
                 (.011 * a, .003 * b, .005 * b)]
        return pose
    if clip == 'attack':
        brace = envelope(t, ((0., 0.), (.18, -.16), (.43, 1.), (.66, .56), (1., 0.)))
        pose = [(0., 0., 0.), (.040 * brace, 0., 0.),
                (.060 * brace, .012 * brace, 0.),
                (.105 * brace, -.014 * brace, -.040 * brace),
                (.090 * brace, .014 * brace, .035 * brace)]
        pose += [(-.020 * brace, 0., -.018 * brace),
                 (-.018 * brace, 0., .018 * brace),
                 (-.023 * brace, 0., -.014 * brace),
                 (-.021 * brace, 0., .014 * brace),
                 (-.026 * brace, .006 * brace, 0.)]
        return pose
    recoil = envelope(t, ((0., 0.), (.20, 1.), (.48, .28), (.72, -.10), (1., 0.)))
    pose = [(0., 0., 0.), (-.045 * recoil, .006 * recoil, .010 * recoil),
            (-.052 * recoil, -.018 * recoil, -.015 * recoil),
            (-.070 * recoil, .015 * recoil, .025 * recoil),
            (-.061 * recoil, -.015 * recoil, -.023 * recoil)]
    pose += [(.024 * recoil, .003 * recoil, .012 * recoil),
             (.021 * recoil, -.003 * recoil, -.012 * recoil),
             (.018 * recoil, .004 * recoil, .010 * recoil),
             (.021 * recoil, -.004 * recoil, -.010 * recoil),
             (.028 * recoil, 0., .005 * recoil)]
    return pose


def export_cyndaquil(model):
    model = named_model(model)
    glb = Glb()
    glb.doc['asset']['generator'] = 'Geothite Cyndaquil skin authoring v1'
    nodes = glb.doc['nodes']
    inverse = []
    for name, parent, pivot in JOINTS:
        origin = JOINTS[parent][2] if parent is not None else (0., 0., 0.)
        nodes.append({'name': f'cyndaquil/{name}', 'translation': [a-b for a, b in zip(pivot, origin)]})
        if parent is not None:
            nodes[parent].setdefault('children', []).append(len(nodes)-1)
        x, y, z = pivot
        inverse.extend([1., 0., 0., 0., 0., 1., 0., 0., 0., 0., 1., 0., -x, -y, -z, 1.])
    nodes.append({'name': 'cyndaquil', 'mesh': 0, 'skin': 0})
    glb.doc['scenes'] = [{'name': 'cyndaquil', 'nodes': [0, len(JOINTS)]}]
    glb.doc['skins'] = [{'name': 'cyndaquil', 'skeleton': 0, 'joints': list(range(len(JOINTS))),
                          'inverseBindMatrices': glb.accessor(inverse, 'MAT4')}]
    primitives, materials = [], {}
    for part in model['primitives']:
        color = tuple(part['base_color'])
        if color not in materials:
            materials[color] = len(materials)
            glb.doc['materials'].append({'name': f'cyndaquil/color_{materials[color]}',
                'pbrMetallicRoughness': {'baseColorFactor': list(color), 'metallicFactor': 0., 'roughnessFactor': 1.},
                'alphaMode': 'BLEND' if color[3] < 1. else 'OPAQUE'})
        ids, weights = [], []
        for i in range(0, len(part['positions']), 3):
            joints, ws = skin_weights(part['part'], part['positions'][i:i+3])
            quantized = quantized_weights(ws)
            ids.extend(j if w else 0 for j, w in zip(joints, quantized))
            weights.extend(quantized)
        primitives.append({'attributes': {
            'POSITION': glb.accessor(part['positions'], 'VEC3', target=34962, bounds=True),
            'NORMAL': glb.accessor(part['normals'], 'VEC3', target=34962),
            'JOINTS_0': glb.accessor(ids, 'VEC4', 5121, 34962),
            'WEIGHTS_0': glb.accessor(weights, 'VEC4', 5121, 34962, normalized=True)},
            'indices': glb.accessor(part['indices'], 'SCALAR', 5123, 34963),
            'material': materials[color], 'mode': 4, 'extras': {'part': part['part']}})
    glb.doc['meshes'] = [{'name': 'cyndaquil', 'primitives': primitives}]
    for clip, duration, samples in CLIPS:
        times = glb.accessor([i*duration/(samples-1) for i in range(samples)], 'SCALAR', bounds=True)
        samplers, channels = [], []
        for joint in range(1, len(JOINTS)):
            rotations = [value for i in range(samples)
                         for value in quaternion_xyz(*pose_angles(clip, i/(samples-1))[joint])]
            channels.append({'sampler': len(samplers), 'target': {'node': joint, 'path': 'rotation'}})
            samplers.append({'input': times, 'output': glb.accessor(rotations, 'VEC4'), 'interpolation': 'LINEAR'})
        glb.doc['animations'].append({'name': f'cyndaquil.{clip}', 'channels': channels, 'samplers': samplers,
            'extras': {'loopSuggested': clip == 'idle', 'cueRelative': clip != 'idle',
                'purpose': 'curious head and raised forepaws, grounded rear paws, restrained paper quills; battle cues remain external'}})
    glb.doc['extras'] = {'coordinate_system': COORDINATES, 'source_name': model['name'],
        'source_sha256': geometry_digest(model),
        'geometry_preservation': 'exact original primitive order, f32 geometry/colors and index order; names matched from Blender source',
        'animation_contract': 'identity root, planted rear feet, attached face/coat, five rigid folded quills, rotation-only joints'}
    return glb.bytes()


def decode_cyndaquil(blob):
    """Read the reviewed neutral sculpture and validate the complete authored GLB.

    Byte-identical regeneration validates skin, binds, clips and all storage
    fields, rather than silently ignoring unsupported transformations. This
    fixed production schema is deliberately not a general glTF importer. Its
    bounded neutral read is also the source for reproducible animation edits.
    """
    if not 28 <= len(blob) <= 1024*1024:
        raise ValueError('Cyndaquil GLB is truncated or exceeds its bounded size')
    try:
        doc, binary = parse_glb(blob)
        if len(doc['meshes']) != 1 or len(doc['meshes'][0]['primitives']) != len(PARTS):
            raise ValueError('expected one mesh containing all Cyndaquil anatomy')

        def attribute(index, kind, component):
            if type(index) is not int or not 0 <= index < len(doc['accessors']):
                raise ValueError('invalid Cyndaquil accessor reference')
            a = doc['accessors'][index]
            if (a['type'], a['componentType']) != (kind, component):
                raise ValueError('unsupported Cyndaquil accessor')
            count = a['count']
            width, size, code = (3, 4, 'f') if kind == 'VEC3' else (1, 2, 'H')
            if type(count) is not int or not 0 < count <= 65536:
                raise ValueError('invalid Cyndaquil accessor count')
            view_index = a['bufferView']
            if type(view_index) is not int or not 0 <= view_index < len(doc['bufferViews']):
                raise ValueError('invalid Cyndaquil buffer view')
            view = doc['bufferViews'][view_index]
            start, length = view['byteOffset'], view['byteLength']
            if (type(start) is not int or start < 0 or start % 4 or length != count*width*size
                    or start+length > len(binary)):
                raise ValueError('Cyndaquil accessor is outside its buffer')
            return list(struct.unpack_from('<'+code*(count*width), binary, start))

        primitives = []
        for part in doc['meshes'][0]['primitives']:
            material = doc['materials'][part['material']]
            primitives.append({'part': part['extras']['part'],
                'positions': attribute(part['attributes']['POSITION'], 'VEC3', 5126),
                'normals': attribute(part['attributes']['NORMAL'], 'VEC3', 5126),
                'indices': attribute(part['indices'], 'SCALAR', 5123),
                'base_color': material['pbrMetallicRoughness']['baseColorFactor']})
        model = {'name': doc['extras']['source_name'], 'version': 1,
                 'coordinate_system': COORDINATES, 'primitives': primitives}
        if export_cyndaquil(model) != blob:
            raise ValueError('unsupported Cyndaquil storage, skin, hierarchy, or animation')
        return model
    except (KeyError, IndexError, TypeError, struct.error, OverflowError, RecursionError, UnicodeError) as error:
        raise ValueError('malformed or unsupported Cyndaquil GLB') from error


def read_cyndaquil(path):
    with Path(path).open('rb') as source:
        return decode_cyndaquil(source.read(1024*1024+1))


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--input', type=Path, required=True, help='canonical GLB, or the original migration JSON')
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    model = read_cyndaquil(args.input) if args.input.suffix == '.glb' else json.loads(args.input.read_bytes())
    blob = export_cyndaquil(model)
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_bytes(blob)
    print(f'{args.output}: {len(blob)} bytes; sha256 {hashlib.sha256(blob).hexdigest()}')


if __name__ == '__main__':
    main()
