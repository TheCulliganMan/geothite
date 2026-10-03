#!/usr/bin/env python3
"""Articulate the reviewed Totodile sculpture without changing neutral geometry.

Requires the existing tools/animated_glb.py and tools/gengar_glb.py on PYTHONPATH.
Those provide storage/math only. Anatomy, attachment fields, and poses here are
specific to the editable Totodile source; no Blender is needed for regeneration.
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

REST_DIGEST = '3e3f8de35f5d7cabcc889604baff4103f050f6abf2a24f6336f9ea3fea9a47d0'
FILE = 'totodile.glb'
PARTS = (
    'Shallow inset nostril.001', 'Shallow inset nostril',
    'Recessed shallow mouth cavity',
    'Tapered dark pupil', 'Tapered dark pupil.001',
    'Sculpted rounded lower jaw',
    'Totodile / continuous head muzzle torso limbs and tail',
    'Warm narrow iris', 'Warm narrow iris.001',
    'Swept red dorsal plate 5', 'Swept red dorsal plate 2',
    'Swept red dorsal plate 3', 'Swept red dorsal plate 4',
    'Swept red dorsal plate 1', 'Golden chest chevron',
    'Upper ivory crocodile fang', 'Upper ivory crocodile fang.001',
    'Ivory toe claw.002', 'Ivory toe claw.001', 'Ivory toe claw',
    'Ivory toe claw.003', 'Ivory toe claw.004', 'Ivory toe claw.005',
    'Small eye highlight.001', 'Inset ivory eye', 'Inset ivory eye.001',
    'Small eye highlight',
)
# Measured from the real Blender runtime body's object transform. The staged
# scene's presentation root is excluded. No source coordinate is written back
# into the immutable neutral mesh; these values only locate anatomical pivots.
SOURCE_SCALE = .9770957827568054
SOURCE_OFFSET = (.000048862897529033944, -.0013535707257688046, .050494443625211716)


def source_point(x, y, z):
    return tuple(round(v, 6) for v in (SOURCE_OFFSET[0] + SOURCE_SCALE*x,
                                     SOURCE_OFFSET[1] + SOURCE_SCALE*z,
                                     SOURCE_OFFSET[2] - SOURCE_SCALE*y))


JOINTS = (
    ('root', None, (0., 0., 0.)),
    ('torso', 0, source_point(0., .025, .225)),
    ('head', 1, source_point(0., .015, .605)),
    ('lower_jaw', 2, source_point(0., .006, .630)),
    ('forepaw_left', 1, source_point(-.160, .005, .490)),
    ('forepaw_right', 1, source_point(.160, .005, .490)),
    ('tail_base', 0, source_point(0., .100, .198)),
    ('tail_tip', 6, source_point(.069, .435, .175)),
)
CLIPS = (('idle', 3.0, 65), ('attack', 1., 49), ('hit', 1., 49))
MATERIAL_NAMES = ('charcoal_insets', 'blue_paper_body', 'warm_iris',
                  'red_dorsal_plates', 'golden_chest', 'ivory_details')


def geometry_digest(model):
    digest = hashlib.sha256()
    for part in model['primitives']:
        for key, component in (('positions', 5126), ('normals', 5126),
                               ('indices', 5125), ('base_color', 5126)):
            digest.update(packed(part[key], component))
    return digest.hexdigest()


def named_model(model):
    if model.get('name') not in ('battle_totodile', 'totodile') or model.get('version') != 1:
        raise ValueError('expected Totodile source schema version 1')
    if model.get('coordinate_system') != COORDINATES or len(model.get('primitives', [])) != len(PARTS):
        raise ValueError('expected refined Totodile coordinates and 27 anatomy parts')
    if geometry_digest(model) != REST_DIGEST:
        raise ValueError('unreviewed neutral geometry; anatomy mapping must be reviewed again')
    result = copy.deepcopy(model)
    for source, name in zip(result['primitives'], PARTS):
        if 'part' in source and source['part'] != name:
            raise ValueError('anatomy name does not match reviewed source geometry')
        source['part'] = name
    return result


def influences(pairs):
    active = [(j, w) for j, w in pairs if w > 0.]
    if not 1 <= len(active) <= 4 or abs(sum(w for _, w in active)-1.) > 1e-10:
        raise ValueError('expected one to four normalized influences')
    return ([j for j, _ in active] + [0]*(4-len(active)),
            [w for _, w in active] + [0.]*(4-len(active)))


def body_weights(position):
    """Continuous field shared by the body, chest inlay, and dorsal plates.

    The planted feet and low haunches remain identity-rooted. Raised reaching
    forepaws hinge at the actual shoulder locations. The tail is behind the
    haunches, rotates only laterally, and cannot entrain the feet. Sharing this
    field on dorsal-fin roots prevents layer drift at the attached surfaces.
    """
    x, y, z = position
    tail = smoothstep(.075, .245, -z) * (1.-smoothstep(.335, .430, y))
    tip = smoothstep(.280, .490, -z)
    upper = smoothstep(.145, .350, y)
    head = smoothstep(.480, .630, y)
    arm = (smoothstep(.177, .280, abs(x-SOURCE_OFFSET[0]))
           * smoothstep(.280, .345, y) * (1.-smoothstep(.505, .595, y))
           * smoothstep(-.005, .070, z))
    side = 4 if x < SOURCE_OFFSET[0] else 5
    return influences(((0, (1.-tail)*(1.-upper)),
        (1, (1.-tail)*upper*(1.-head)*(1.-arm)),
        (2, (1.-tail)*upper*head*(1.-arm)),
        (side, (1.-tail)*upper*arm),
        (6, tail*(1.-tip)), (7, tail*tip)))


MOUTH_PROFILES = tuple((source_point(0., y, z)[2],
                       source_point(0., y, z)[1], SOURCE_SCALE*radius)
                      for y, z, radius in ((-.087, .661, .039),
                          (-.261, .632, .046), (-.426, .637, .041),
                          (-.516, .641, .030)))


def cavity_weights(position):
    # The actual four-ring mouth loft has a sloping centerline. Its top stays
    # with the upper muzzle; its lower inner surface follows the separate jaw.
    _, y, z = position
    center, radius = MOUTH_PROFILES[-1][1:]
    for (za, ya, ra), (zb, yb, rb) in zip(MOUTH_PROFILES, MOUTH_PROFILES[1:]):
        if z <= zb:
            t = max(0., min(1., (z-za)/(zb-za)))
            center, radius = ya+(yb-ya)*t, ra+(rb-ra)*t
            break
    lower = 1.-smoothstep(-.60, .60, (y-center)/radius)
    return influences(((2, 1.-lower), (3, lower)))


def skin_weights(part, position):
    if part == 'Sculpted rounded lower jaw':
        return [3, 0, 0, 0], [1., 0., 0., 0.]
    if part == 'Recessed shallow mouth cavity':
        return cavity_weights(position)
    if part.startswith('Ivory toe claw'):
        return [0, 0, 0, 0], [1., 0., 0., 0.]
    if part.startswith(('Shallow inset nostril', 'Tapered dark pupil',
                        'Warm narrow iris', 'Upper ivory crocodile fang',
                        'Small eye highlight', 'Inset ivory eye')):
        return [2, 0, 0, 0], [1., 0., 0., 0.]
    return body_weights(position)


def pose_angles(clip, t):
    """Restrained articulation; source move and HP clocks choose progress.

    No translations, scale tracks, attack displacement, projectiles, or effects
    are baked here. The tail only yaws so its low rear silhouette never lifts
    or sweeps the feet. Jaw amplitudes are deliberately small over its real hinge.
    """
    if clip not in ('idle', 'attack', 'hit') or not 0. <= t <= 1.:
        raise ValueError('unknown clip or normalized time outside [0, 1]')
    if t in (0., 1.):
        return [(0., 0., 0.)]*len(JOINTS)
    if clip == 'idle':
        a, b = math.sin(math.tau*t), math.sin(2.*math.tau*t)
        breathe = .5-.5*math.cos(math.tau*t)
        return [(0., 0., 0.), (.005*a, 0., .004*b),
            (-.010*breathe, .024*a, .006*b), (.028*breathe, 0., 0.),
            (.026*breathe, -.009*b, -.010*a),
            (.021*breathe, .008*b, .010*a),
            (0., -.026*a, 0.), (0., -.034*a+.008*b, 0.)]
    if clip == 'attack':
        brace = envelope(t, ((0., 0.), (.18, -.14), (.43, 1.), (.66, .55), (1., 0.)))
        jaw = envelope(t, ((0., 0.), (.24, .2), (.43, 1.), (.64, .38), (1., 0.)))
        return [(0., 0., 0.), (.033*brace, 0., 0.),
            (.036*brace, .010*brace, 0.), (.052*jaw, 0., 0.),
            (.087*brace, -.018*brace, -.033*brace),
            (.077*brace, .016*brace, .029*brace),
            (0., .025*brace, 0.), (0., -.046*brace, 0.)]
    recoil = envelope(t, ((0., 0.), (.19, 1.), (.47, .26), (.73, -.09), (1., 0.)))
    jaw = envelope(t, ((0., 0.), (.22, 1.), (.50, .3), (1., 0.)))
    return [(0., 0., 0.), (-.034*recoil, .006*recoil, .009*recoil),
        (-.043*recoil, -.015*recoil, -.009*recoil), (.039*jaw, 0., 0.),
        (-.060*recoil, .012*recoil, .021*recoil),
        (-.054*recoil, -.013*recoil, -.023*recoil),
        (0., -.030*recoil, 0.), (0., .045*recoil, 0.)]


def export_totodile(model):
    model = named_model(model)
    glb = Glb()
    glb.doc['asset']['generator'] = 'Geothite Totodile skin authoring v1'
    nodes, inverse = glb.doc['nodes'], []
    for name, parent, pivot in JOINTS:
        origin = JOINTS[parent][2] if parent is not None else (0., 0., 0.)
        nodes.append({'name': f'totodile/{name}', 'translation': [a-b for a, b in zip(pivot, origin)]})
        if parent is not None:
            nodes[parent].setdefault('children', []).append(len(nodes)-1)
        x, y, z = pivot
        inverse.extend([1., 0., 0., 0., 0., 1., 0., 0., 0., 0., 1., 0., -x, -y, -z, 1.])
    nodes.append({'name': 'totodile', 'mesh': 0, 'skin': 0})
    glb.doc['scenes'] = [{'name': 'totodile', 'nodes': [0, len(JOINTS)]}]
    glb.doc['skins'] = [{'name': 'totodile', 'skeleton': 0, 'joints': list(range(len(JOINTS))),
                          'inverseBindMatrices': glb.accessor(inverse, 'MAT4')}]
    primitives, materials = [], {}
    for part in model['primitives']:
        color = tuple(part['base_color'])
        if color not in materials:
            materials[color] = len(materials)
            glb.doc['materials'].append({'name': f'totodile/{MATERIAL_NAMES[materials[color]]}',
                'pbrMetallicRoughness': {'baseColorFactor': list(color), 'metallicFactor': 0., 'roughnessFactor': 1.},
                'alphaMode': 'BLEND' if color[3] < 1. else 'OPAQUE'})
        ids, weights = [], []
        # Weight from the same f32 values decoded from the canonical GLB.
        # Decimal migration JSON must not change a largest-remainder tie.
        positions = struct.unpack('<'+'f'*len(part['positions']), packed(part['positions']))
        for i in range(0, len(positions), 3):
            joints, ws = skin_weights(part['part'], positions[i:i+3])
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
    glb.doc['meshes'] = [{'name': 'totodile', 'primitives': primitives}]
    for clip, duration, samples in CLIPS:
        times = glb.accessor([i*duration/(samples-1) for i in range(samples)], 'SCALAR', bounds=True)
        samplers, channels = [], []
        for joint in range(1, len(JOINTS)):
            rotations = [v for i in range(samples)
                         for v in quaternion_xyz(*pose_angles(clip, i/(samples-1))[joint])]
            channels.append({'sampler': len(samplers), 'target': {'node': joint, 'path': 'rotation'}})
            samplers.append({'input': times, 'output': glb.accessor(rotations, 'VEC4'), 'interpolation': 'LINEAR'})
        glb.doc['animations'].append({'name': f'totodile.{clip}', 'channels': channels, 'samplers': samplers,
            'extras': {'loopSuggested': clip == 'idle', 'cueRelative': clip != 'idle',
                'purpose': 'grounded crocodile brace, articulated jaw and raised forepaws, low lateral tail; battle cues remain external'}})
    glb.doc['extras'] = {'coordinate_system': COORDINATES, 'source_name': model['name'],
        'source_sha256': geometry_digest(model),
        'geometry_preservation': 'exact original primitive order, f32 geometry/colors and index order; names matched from Blender source',
        'animation_contract': 'identity root, planted feet and claws, rigid face and jaw, attached chest and dorsal plates, rotation-only joints'}
    return glb.bytes()


def decode_totodile(blob):
    """Bounded neutral reader; exact regeneration validates all authored fields."""
    if not 28 <= len(blob) <= 1024*1024:
        raise ValueError('Totodile GLB is truncated or exceeds its bounded size')
    try:
        doc, binary = parse_glb(blob)
        if len(doc['meshes']) != 1 or len(doc['meshes'][0]['primitives']) != len(PARTS):
            raise ValueError('expected one mesh containing all Totodile anatomy')

        def attribute(index, kind, component):
            if type(index) is not int or not 0 <= index < len(doc['accessors']):
                raise ValueError('invalid Totodile accessor reference')
            a = doc['accessors'][index]
            if (a['type'], a['componentType']) != (kind, component):
                raise ValueError('unsupported Totodile accessor')
            count = a['count']
            width, size, code = (3, 4, 'f') if kind == 'VEC3' else (1, 2, 'H')
            if type(count) is not int or not 0 < count <= 65536:
                raise ValueError('invalid Totodile accessor count')
            view_index = a['bufferView']
            if type(view_index) is not int or not 0 <= view_index < len(doc['bufferViews']):
                raise ValueError('invalid Totodile buffer view')
            view = doc['bufferViews'][view_index]
            start, length = view['byteOffset'], view['byteLength']
            if (type(start) is not int or start < 0 or start % 4 or length != count*width*size
                    or start+length > len(binary)):
                raise ValueError('Totodile accessor is outside its buffer')
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
        if export_totodile(model) != blob:
            raise ValueError('unsupported Totodile storage, skin, hierarchy, or animation')
        return model
    except (KeyError, IndexError, TypeError, struct.error, OverflowError, RecursionError, UnicodeError) as error:
        raise ValueError('malformed or unsupported Totodile GLB') from error


def read_totodile(path):
    with Path(path).open('rb') as source:
        return decode_totodile(source.read(1024*1024+1))


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--input', type=Path, required=True, help='canonical GLB or original migration JSON')
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    model = read_totodile(args.input) if args.input.suffix == '.glb' else json.loads(args.input.read_bytes())
    blob = export_totodile(model)
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_bytes(blob)
    print(f'{args.output}: {len(blob)} bytes; sha256 {hashlib.sha256(blob).hexdigest()}')


if __name__ == '__main__':
    main()
