#!/usr/bin/env python3
"""Export the authored folded-paper Cyndaquil with its ten-joint skin.

Geometry is reproducible from cyndaquil_sculpt.py without Blender, inputs or
third-party packages. The bounded canonical reader rejects unreviewed geometry
and altered skin storage; all cue clocks remain owned by the runtime.
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

REST_DIGEST = '551051e53987f2461c2e5ed22187b44b336be64f300b469950d381b5b174b397'
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

# Keep the original source-to-runtime rig datum. The three lower flame roots
# are refitted into the new folded back; the rest of the rig pivots are stable.
SOURCE_SCALE = .8989177942276001
SOURCE_OFFSET = (.002550057601183653, -.001327162142843008, -.008687515743076801)


def source_point(x, y, z):
    return tuple(round(v, 6) for v in (SOURCE_OFFSET[0] + SOURCE_SCALE * x,
                                     SOURCE_OFFSET[1] + SOURCE_SCALE * z,
                                     SOURCE_OFFSET[2] - SOURCE_SCALE * y))


QUILLS = (
    ('Upper left flame quill', (-.071, .060, .604)),
    ('Upper right flame quill', (.071, .060, .604)),
    ('Lower left flame quill', (-.138, .123, .450)),
    ('Lower right flame quill', (.138, .123, .450)),
    ('Central rear flame quill', (0., .183, .432)),
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
    """Stable anatomy identities match the reviewed procedural sculpture."""
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
    upper = max(smoothstep(.105, .200, y), arm)
    head = smoothstep(-.140, .030, z) * smoothstep(.310, .430, y)
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
        breathe = .5 - .5 * math.cos(math.tau * t)
        # A quiet shrew sniff: nose leads the turn, one forepaw curls, then the
        # other. Root and rear contact never bob to simulate breathing.
        pose = [(0., 0., 0.), (.014 * a, 0., .009 * b),
                (-.065 * breathe, .085 * a, .020 * a),
                (-.075 * breathe, .018 * b, -.033 * a),
                (-.055 * breathe, -.015 * b, .029 * a)]
        pose += [(.035 * a + .012 * b, .008 * b, -.031 * a),
                 (.026 * a - .012 * b, -.008 * b, .037 * a),
                 (.030 * b, .016 * a, -.025 * b),
                 (-.027 * b, -.016 * a, .025 * b),
                 (.040 * a, .015 * b, .016 * b)]
        return pose
    if clip == 'attack':
        # Immediate planted chest brace and settle. This curve is sampled
        # only by normalized source cue progress; it schedules no effect.
        brace = envelope(t, ((0., 0.), (.06, .92), (.20, 1.), (.62, .54), (1., 0.)))
        flare = envelope(t, ((0., 0.), (.07, .45), (.18, 1.), (.62, .34), (1., 0.)))
        pose = [(0., 0., 0.), (.110 * brace, 0., 0.),
                (.120 * brace, .023 * brace, 0.),
                (.218 * brace, -.025 * brace, -.080 * brace),
                (.198 * brace, .025 * brace, .069 * brace)]
        pose += [(-.095 * flare, -.030 * flare, -.082 * flare),
                 (-.078 * flare, .030 * flare, .076 * flare),
                 (-.104 * flare, -.025 * flare, -.061 * flare),
                 (-.087 * flare, .025 * flare, .063 * flare),
                 (-.129 * flare, .025 * flare, 0.)]
        return pose
    recoil = envelope(t, ((0., 0.), (.17, 1.), (.40, .32), (.65, -.16), (1., 0.)))
    trail = envelope(t, ((0., 0.), (.24, 1.), (.49, .17), (.72, -.09), (1., 0.)))
    pose = [(0., 0., 0.), (-.098 * recoil, .014 * recoil, .023 * recoil),
            (-.120 * recoil, -.045 * recoil, -.035 * recoil),
            (-.194 * recoil, .039 * recoil, .067 * recoil),
            (-.161 * recoil, -.033 * recoil, -.060 * recoil)]
    pose += [(.116 * trail, .018 * trail, .051 * trail),
             (.094 * trail, -.018 * trail, -.046 * trail),
             (.080 * trail, .020 * trail, .045 * trail),
             (.099 * trail, -.020 * trail, -.045 * trail),
             (.138 * trail, 0., .027 * trail)]
    return pose


def export_cyndaquil(model):
    model = named_model(model)
    glb = Glb()
    glb.doc['asset']['generator'] = 'Geothite Cyndaquil folded-paper authoring v2'
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
                'purpose': 'nose-led sniff, planted chest brace and delayed folded-quill recoil; battle cue clocks remain external'}})
    glb.doc['extras'] = {'coordinate_system': COORDINATES, 'source_name': model['name'],
        'source_sha256': geometry_digest(model),
        'body_reference_height': .716729, 'full_silhouette_height': .91,
        'geometry_source': 'tools/cyndaquil_sculpt.py',
        'geometry_preservation': 'explicit authored panels regenerated by tools/cyndaquil_sculpt.py; retained anatomy identities and physical height datum',
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
    parser.add_argument('--input', type=Path, help='optional canonical GLB to validate and regenerate; omit to build from sculpture recipe')
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    if args.input is None:
        from cyndaquil_sculpt import sculpt
        model = sculpt()
    else:
        model = read_cyndaquil(args.input) if args.input.suffix == '.glb' else json.loads(args.input.read_bytes())
    blob = export_cyndaquil(model)
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_bytes(blob)
    print(f'{args.output}: {len(blob)} bytes; sha256 {hashlib.sha256(blob).hexdigest()}')


if __name__ == '__main__':
    main()
