#!/usr/bin/env python3
"""Reconstruct Gengar's skin from its surviving reviewed papercraft sculpture.

This is new anatomical animation, not recovery of the lost rig. Original f32
geometry, normals, colors, triangle order and named parts are immutable. The
canonical GLB is its own neutral authoring input after the migration JSON is
retired. Only Python's standard library and the repository's small skin_glb
writer are used. No Blender, external exporter, or duplicate source is needed.
"""
import argparse
import hashlib
import json
import math
import struct
from pathlib import Path

from skin_glb import (COORDINATES, Glb, envelope, packed,
                      quantized_weights, quaternion_xyz, smoothstep)

FILE = 'gengar.glb'
REST_DIGEST = 'd458acb7c2853288efef3b83c24ac453bcb409f9db01d24a96008e9a937485aa'
MAX_BYTES = 1024 * 1024
PARTS = (
    'Gengar / continuous body ears limbs and dorsal spines',
    'Three tapered hand digits', 'Three tapered hand digits.001',
    'Three tapered hand digits.002',
    'Subtle foot digit', 'Subtle foot digit.001', 'Subtle foot digit.002',
    'Three tapered hand digits.003', 'Three tapered hand digits.004',
    'Three tapered hand digits.005',
    'Subtle foot digit.003', 'Subtle foot digit.004', 'Subtle foot digit.005',
    'Recessed broad crescent smile',
    'Curved ivory grin tooth', 'Curved ivory grin tooth.001',
    'Curved ivory grin tooth.002', 'Curved ivory grin tooth.003',
    'Curved ivory grin tooth.004', 'Curved ivory grin tooth.005',
    'Curved ivory grin tooth.006',
    'Inset angular crimson eye', 'Vertical black eye slit',
    'Heavy expressive upper eyelid', 'Inset angular crimson eye.001',
    'Vertical black eye slit.001', 'Heavy expressive upper eyelid.001',
)

# Global rest pivots in the existing floor-centered sculpture, +Y up/+Z front.
# Gengar has no separable neck or jaw: its fitted grin and eyes move with the
# front core. The fused back has eight tips, grouped into five anatomical hinges.
JOINTS = (
    ('root', None, (0., 0., 0.)),
    ('torso', 0, (0., .245, .020)),
    ('face_core', 1, (0., .485, .045)),
    ('shoulder_left', 1, (-.310, .505, .045)),
    ('hand_left', 3, (-.468, .418, .186)),
    ('shoulder_right', 1, (.310, .505, .045)),
    ('hand_right', 5, (.468, .418, .186)),
    ('ear_left', 2, (-.253, .724, .012)),
    ('ear_right', 2, (.253, .724, .012)),
    ('dorsal_crown', 2, (0., .775, -.135)),
    ('dorsal_left', 2, (-.258, .575, -.168)),
    ('dorsal_right', 2, (.258, .575, -.168)),
    ('dorsal_lower', 1, (0., .414, -.215)),
    ('tail_spine', 1, (0., .258, -.195)),
)
CLIPS = (('idle', 3.6, 73), ('attack', 1., 65), ('hit', 1., 65))
MATERIAL_NAMES = ('violet', 'mouth', 'ivory', 'crimson')


def geometry_digest(model):
    digest = hashlib.sha256()
    for part in model['primitives']:
        for key, component in (('positions', 5126), ('normals', 5126),
                               ('indices', 5125), ('base_color', 5126)):
            digest.update(packed(part[key], component))
    return digest.hexdigest()


def validate_model(model):
    if (model.get('name') != 'gengar' or model.get('version') != 1
            or model.get('coordinate_system') != COORDINATES
            or len(model.get('primitives', ())) != len(PARTS)):
        raise ValueError('expected reviewed Gengar schema, coordinates and 27 parts')
    for part, name in zip(model['primitives'], PARTS):
        if part.get('part') != name:
            raise ValueError('Gengar anatomy name does not match the reviewed source')
        p, n, ix, c = (part[k] for k in ('positions', 'normals', 'indices', 'base_color'))
        if (not 9 <= len(p) <= 60000 or len(p) % 3 or len(n) != len(p)
                or not 3 <= len(ix) <= 120000 or len(ix) % 3 or len(c) != 4):
            raise ValueError('invalid bounded Gengar geometry arrays')
        if any(type(i) is not int or not 0 <= i < len(p)//3 for i in ix):
            raise ValueError('Gengar triangle index is outside its primitive')
        if any(type(v) not in (float, int) or not math.isfinite(v) for v in p+n+c):
            raise ValueError('non-finite Gengar geometry')
    if geometry_digest(model) != REST_DIGEST:
        raise ValueError('unreviewed neutral geometry; Gengar anatomy must be reviewed again')


def skin_weights(part, position):
    """One continuous spatial field on the fused body and all fitted details.

    No part-specific threshold changes, nearest-bone switches, or independent
    facial weights: coincident points always share the exact same weights.
    Non-overlapping hinge domains keep at most four influences without pruning.
    The U8 quantizer is applied only after evaluating this normalized field.
    """
    x, y, z = position
    upper = smoothstep(.120, .310, y)
    front = (smoothstep(.200, .323, y) * smoothstep(-.080, .090, z)
             * (1. - smoothstep(.330, .490, abs(x))))
    head = max(front, smoothstep(.360, .810, y))
    weights = {0: 1.-upper, 1: upper*(1.-head), 2: upper*head}

    arm = (smoothstep(.330, .490, abs(x)) * smoothstep(.235, .310, y)
           * (1.-smoothstep(.495, .610, y)) * smoothstep(-.180, -.080, z))
    hand = smoothstep(.490, .555, abs(x)) * smoothstep(.100, .245, z)
    side = 3 if x < 0. else 5
    weights = {j: w*(1.-arm) for j, w in weights.items()}
    weights[side], weights[side+1] = arm*(1.-hand), arm*hand

    # Facial patches end below .768. Beginning the ear fold above that band
    # keeps the entire fitted face rigid and avoids cutting long ear triangles
    # across a narrow front/back mask.
    ear = (smoothstep(.775, .940, y) * smoothstep(.120, .240, abs(x))
           * smoothstep(-.100, -.025, z))
    weights = {j: w*(1.-ear) for j, w in weights.items()}
    weights[7 if x < 0. else 8] = ear

    # Each height interval overlaps only adjacent dorsal groups. Upper left and
    # right meet smoothly on the centerline, which has no isolated protrusion.
    low = smoothstep(.280, .365, y)
    middle = smoothstep(.485, .590, y)
    crown = smoothstep(.710, .805, y)
    flank = smoothstep(.055, .170, abs(x))
    dorsal = ((1.-crown)*smoothstep(.180, .290, -z)
              + crown*smoothstep(.120, .185, -z))
    dorsal_weights = {13: dorsal*(1.-low), 12: dorsal*low*(1.-middle),
                      10 if x < 0. else 11: dorsal*low*middle*(1.-crown)*flank,
                      9: dorsal*low*middle*crown}
    strength = sum(dorsal_weights.values())
    weights = {j: w*(1.-strength) for j, w in weights.items()}
    weights.update(dorsal_weights)
    active = [(j, w) for j, w in sorted(weights.items()) if w > 0.]
    if len(active) > 4 or abs(sum(w for _, w in active)-1.) > 1e-12:
        raise ValueError('Gengar field requires at most four normalized influences')
    ids, ws = [j for j, _ in active], [w for _, w in active]
    return ids+[0]*(4-len(ids)), ws+[0.]*(4-len(ws))


def pose_angles(clip, t):
    """Expressive local posing; game-owned move/HP clocks select cue progress."""
    if clip not in ('idle', 'attack', 'hit') or not 0. <= t <= 1.:
        raise ValueError('unknown clip or normalized time outside [0, 1]')
    if t in (0., 1.):
        return [(0., 0., 0.)] * len(JOINTS)
    a, b = math.sin(math.tau*t), math.sin(2.*math.tau*t)
    if clip == 'idle':
        breath = .5-.5*math.cos(math.tau*t)
        return [(0., 0., 0.),
                (.021*a, .015*a, .012*b),
                (.020*breath, -.038*a, -.016*b),
                (-.048*breath, .040*a, -.045*a),
                (-.090*breath, .025*b, -.045*a),
                (-.062*breath, -.032*a, -.036*a),
                (-.074*breath, -.021*b, .036*a),
                (.033*a, .020*b, -.069*a),
                (-.026*a+.014*b, -.017*b, .055*a),
                (.044*a+.012*b, .020*b, -.018*a),
                (.025*b, -.035*a, -.025*a),
                (.020*b, .027*a, .030*a),
                (.022*a, .020*b, -.018*a),
                (.016*b, -.020*a, .020*a)]
    if clip == 'attack':
        brace = envelope(t, ((0., 0.), (.17, -.20), (.43, 1.), (.58, .72), (.82, .14), (1., 0.)))
        follow = envelope(t, ((0., 0.), (.20, -.12), (.49, 1.), (.69, .46), (1., 0.)))
        return [(0., 0., 0.),
                (.150*brace, -.025*brace, .014*brace),
                (.058*brace, .085*brace, -.035*brace),
                (-.196*brace, .156*brace, -.204*brace),
                (-.255*follow, .140*follow, -.120*follow),
                (-.174*brace, -.129*brace, .231*brace),
                (-.235*follow, -.115*follow, .150*follow),
                (-.110*follow, .050*follow, -.115*follow),
                (-.145*follow, -.045*follow, .135*follow),
                (-.115*follow, -.050*follow, -.020*follow),
                (-.080*follow, -.110*follow, -.040*follow),
                (-.065*follow, .100*follow, .050*follow),
                (-.085*follow, .025*follow, -.025*follow),
                (-.080*follow, -.060*follow, .030*follow)]
    recoil = envelope(t, ((0., 0.), (.18, 1.), (.39, .42), (.66, -.15), (1., 0.)))
    lag = envelope(t, ((0., 0.), (.24, 1.), (.49, .30), (.76, -.09), (1., 0.)))
    return [(0., 0., 0.),
            (-.135*recoil, .045*recoil, -.042*recoil),
            (-.060*recoil, -.070*recoil, .042*recoil),
            (.180*recoil, -.095*recoil, .125*recoil),
            (.135*lag, -.070*lag, .075*lag),
            (.230*recoil, .085*recoil, -.160*recoil),
            (.165*lag, .055*lag, -.090*lag),
            (.150*lag, -.070*lag, .135*lag),
            (.120*lag, .055*lag, -.165*lag),
            (.125*lag, .045*lag, .040*lag),
            (.090*lag, .085*lag, .055*lag),
            (.110*lag, -.095*lag, -.060*lag),
            (.100*lag, -.035*lag, .030*lag),
            (.075*lag, .065*lag, -.040*lag)]


def export_gengar(model):
    validate_model(model)
    glb = Glb()
    glb.doc['asset']['generator'] = 'Geothite reconstructed Gengar skin authoring v1'
    inverse, nodes = [], glb.doc['nodes']
    for name, parent, pivot in JOINTS:
        origin = JOINTS[parent][2] if parent is not None else (0., 0., 0.)
        nodes.append({'name': f'gengar/{name}', 'translation': [a-b for a, b in zip(pivot, origin)]})
        if parent is not None:
            nodes[parent].setdefault('children', []).append(len(nodes)-1)
        x, y, z = pivot
        inverse.extend([1., 0., 0., 0., 0., 1., 0., 0., 0., 0., 1., 0., -x, -y, -z, 1.])
    nodes.append({'name': 'gengar', 'mesh': 0, 'skin': 0})
    glb.doc['scenes'] = [{'name': 'gengar', 'nodes': [0, len(JOINTS)]}]
    glb.doc['skins'] = [{'name': 'gengar', 'skeleton': 0, 'joints': list(range(len(JOINTS))),
                         'inverseBindMatrices': glb.accessor(inverse, 'MAT4')}]
    primitives, colors = [], {}
    for part in model['primitives']:
        color = tuple(part['base_color'])
        if color not in colors:
            colors[color] = len(colors)
            glb.doc['materials'].append({'name': f'gengar/{MATERIAL_NAMES[colors[color]]}',
                'pbrMetallicRoughness': {'baseColorFactor': list(color), 'metallicFactor': 0., 'roughnessFactor': 1.},
                'alphaMode': 'OPAQUE'})
        ids, weights = [], []
        for offset in range(0, len(part['positions']), 3):
            # Source JSON doubles and decoded GLB f32s must choose identical
            # quantization bins during canonical re-export.
            position = struct.unpack('<3f', packed(part['positions'][offset:offset+3]))
            joints, ws = skin_weights(part['part'], position)
            quantized = quantized_weights(ws)
            ids.extend(j if w else 0 for j, w in zip(joints, quantized))
            weights.extend(quantized)
        primitives.append({'attributes': {
            'POSITION': glb.accessor(part['positions'], 'VEC3', target=34962, bounds=True),
            'NORMAL': glb.accessor(part['normals'], 'VEC3', target=34962),
            'JOINTS_0': glb.accessor(ids, 'VEC4', 5121, 34962),
            'WEIGHTS_0': glb.accessor(weights, 'VEC4', 5121, 34962, normalized=True)},
            'indices': glb.accessor(part['indices'], 'SCALAR', 5123, 34963),
            'material': colors[color], 'mode': 4, 'extras': {'part': part['part']}})
    glb.doc['meshes'] = [{'name': 'gengar', 'primitives': primitives}]
    for clip, duration, count in CLIPS:
        times = glb.accessor([i*duration/(count-1) for i in range(count)], 'SCALAR', bounds=True)
        samplers, channels = [], []
        for joint in range(1, len(JOINTS)):
            rotations = [v for i in range(count)
                         for v in quaternion_xyz(*pose_angles(clip, i/(count-1))[joint])]
            channels.append({'sampler': len(samplers), 'target': {'node': joint, 'path': 'rotation'}})
            samplers.append({'input': times, 'output': glb.accessor(rotations, 'VEC4'), 'interpolation': 'LINEAR'})
        glb.doc['animations'].append({'name': f'gengar.{clip}', 'channels': channels, 'samplers': samplers,
            'extras': {'loopSuggested': clip == 'idle', 'cueRelative': clip != 'idle',
                'purpose': 'grounded pear-body lean, coherent grin, reaching/curling short arms, lagged ears and dorsal tips; game-owned battle cues'}})
    glb.doc['extras'] = {'coordinate_system': COORDINATES, 'source_name': 'gengar',
        'source_sha256': REST_DIGEST,
        'geometry_preservation': 'exact source primitive order, part names, f32 positions/normals/colors and triangle index order',
        'animation_contract': 'new reconstruction from surviving sculpture; fixed feet; identity root; 14 named joints; rotation-only clips'}
    return glb.bytes()


def decode_gengar(blob):
    """Bounded canonical neutral read, rejecting any ignored storage/rig change."""
    if not 28 <= len(blob) <= MAX_BYTES:
        raise ValueError('Gengar GLB is truncated or exceeds its bounded size')
    try:
        if struct.unpack_from('<III', blob) != (0x46546c67, 2, len(blob)):
            raise ValueError('invalid Gengar GLB header')
        jslen, jstype = struct.unpack_from('<II', blob, 12)
        if jstype != 0x4e4f534a or jslen % 4 or 20+jslen+8 > len(blob):
            raise ValueError('invalid Gengar JSON chunk')
        doc = json.loads(blob[20:20+jslen])
        binlen, bintype = struct.unpack_from('<II', blob, 20+jslen)
        if bintype != 0x004e4942 or binlen % 4 or 28+jslen+binlen != len(blob):
            raise ValueError('invalid Gengar binary chunk')
        binary = blob[28+jslen:]
        if (len(doc['meshes']) != 1 or len(doc['meshes'][0]['primitives']) != 27
                or not 1 <= len(doc['accessors']) <= 256 or len(doc['materials']) != 4):
            raise ValueError('unexpected Gengar document dimensions')

        def attribute(index, kind, component):
            if type(index) is not int or not 0 <= index < len(doc['accessors']):
                raise ValueError('invalid Gengar accessor reference')
            a = doc['accessors'][index]
            if (a['type'], a['componentType']) != (kind, component):
                raise ValueError('unsupported Gengar accessor')
            count = a['count']
            width, size, code = (3, 4, 'f') if kind == 'VEC3' else (1, 2, 'H')
            if type(count) is not int or not 0 < count <= 120000:
                raise ValueError('invalid Gengar accessor count')
            vi = a['bufferView']
            if type(vi) is not int or not 0 <= vi < len(doc['bufferViews']):
                raise ValueError('invalid Gengar buffer view')
            view = doc['bufferViews'][vi]
            start, length = view['byteOffset'], view['byteLength']
            if (type(start) is not int or start < 0 or start % 4 or length != count*width*size
                    or start+length > len(binary)):
                raise ValueError('Gengar accessor is outside its buffer')
            return list(struct.unpack_from('<'+code*(count*width), binary, start))

        primitives = []
        for p in doc['meshes'][0]['primitives']:
            material = doc['materials'][p['material']]
            primitives.append({'part': p['extras']['part'],
                'positions': attribute(p['attributes']['POSITION'], 'VEC3', 5126),
                'normals': attribute(p['attributes']['NORMAL'], 'VEC3', 5126),
                'indices': attribute(p['indices'], 'SCALAR', 5123),
                'base_color': material['pbrMetallicRoughness']['baseColorFactor']})
        model = {'name': doc['extras']['source_name'], 'version': 1,
                 'coordinate_system': COORDINATES, 'primitives': primitives}
        if export_gengar(model) != blob:
            raise ValueError('unsupported Gengar storage, skin, hierarchy, or animation')
        return model
    except (KeyError, IndexError, TypeError, struct.error, OverflowError, RecursionError, UnicodeError) as error:
        raise ValueError('malformed or unsupported Gengar GLB') from error


def read_gengar(path):
    with Path(path).open('rb') as source:
        return decode_gengar(source.read(MAX_BYTES+1))


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--input', type=Path, required=True, help='canonical GLB or original migration JSON')
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    with args.input.open('rb') as source:
        content = source.read(MAX_BYTES+1)
    if len(content) > MAX_BYTES:
        raise ValueError('Gengar authoring input exceeds its bounded size')
    model = decode_gengar(content) if args.input.suffix == '.glb' else json.loads(content)
    blob = export_gengar(model)
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_bytes(blob)
    print(f'{args.output}: {len(blob)} bytes; sha256 {hashlib.sha256(blob).hexdigest()}')


if __name__ == '__main__':
    main()
