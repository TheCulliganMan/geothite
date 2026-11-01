#!/usr/bin/env python3
"""Standard eight-joint skin over Raticate's surviving reviewed sculpture.

Neutral f32 geometry, normals, colors, triangles and named parts are preserved.
The canonical GLB is its own neutral input after retiring migration JSON.
Identical index/skin accessors share storage within the existing reader bounds.
Only the standard library and the repository's skin_glb writer are required.
"""
import argparse
import hashlib
import json
import math
import struct
from pathlib import Path

from skin_glb import (COORDINATES, Glb, envelope, packed,
                      quantized_weights, quaternion_xyz, smoothstep)

FILE = 'raticate.glb'
REST_DIGEST = 'f6043d52ed1588580d3e4c8f44480d8d3cb2b8b2f268e6ccd7220287d76edd0d'
MAX_BYTES = 1024 * 1024
PARTS = ('Raticate / continuous pear body head and fur silhouette', 'Broad cream belly', 'Flared cupped rat ear', 'Recessed ear interior', 'Slanted almond eye', 'Focused small rat pupil', 'Natural upper eyelid', 'Flared cupped rat ear.001', 'Recessed ear interior.001', 'Slanted almond eye.001', 'Focused small rat pupil.001', 'Natural upper eyelid.001', 'Dark open mouth cavity', 'Lower cream jaw', 'Lower mouth tongue', 'Cream whisker cheek', 'Broad upper chisel incisor', 'Short lower chisel incisor', 'Short bent forearm', 'Small grasping forepaw', 'Three curled forepaw digits', 'Forepaw claw', 'Three curled forepaw digits.001', 'Forepaw claw.001', 'Three curled forepaw digits.002', 'Forepaw claw.002', 'Long splayed rat foot', 'Long rat toe', 'Rat toe claw', 'Long rat toe.001', 'Rat toe claw.001', 'Long rat toe.002', 'Rat toe claw.002', 'Tapered facial whisker', 'Tapered facial whisker.001', 'Tapered facial whisker.002', 'Cream whisker cheek.001', 'Broad upper chisel incisor.001', 'Short lower chisel incisor.001', 'Short bent forearm.001', 'Small grasping forepaw.001', 'Three curled forepaw digits.003', 'Forepaw claw.003', 'Three curled forepaw digits.004', 'Forepaw claw.004', 'Three curled forepaw digits.005', 'Forepaw claw.005', 'Long splayed rat foot.001', 'Long rat toe.003', 'Rat toe claw.003', 'Long rat toe.004', 'Rat toe claw.004', 'Long rat toe.005', 'Rat toe claw.005', 'Tapered facial whisker.003', 'Tapered facial whisker.004', 'Tapered facial whisker.005', 'Small split rat nose', 'Nose philtrum', 'Long tapered bare rat tail', 'Subtle tail segmentation', 'Subtle tail segmentation.001', 'Subtle tail segmentation.002')

# Rest pivots belong to the surviving floor-centered sculpture. The fused
# head/body and fitted mouth/eyes remain coherent on one torso bone.
JOINTS = (
    ('root', None, (0.,0.,0.)), ('torso',0,(-.021,.43,.056)),
    ('ear_left',1,(-.273,.755,.129)), ('ear_right',1,(.230,.755,.129)),
    ('forepaw_left',1,(-.315,.43,.23)), ('forepaw_right',1,(.272,.43,.23)),
    ('tail_base',1,(-.021,.287,-.209)), ('tail_tip',6,(.35,.36,-.382)),
)
CLIPS = (('idle',3.6,73),('attack',1.,65),('hit',1.,65))
MATERIAL_NAMES = ('fur','cream','ear','ivory','ink','tongue','philtrum','tail_rings')


def geometry_digest(model):
    digest = hashlib.sha256()
    for part in model['primitives']:
        for key, component in (('positions', 5126), ('normals', 5126),
                               ('indices', 5125), ('base_color', 5126)):
            digest.update(packed(part[key], component))
    return digest.hexdigest()


def validate_model(model):
    if (model.get('name') != 'raticate' or model.get('version') != 1
            or model.get('coordinate_system') != COORDINATES
            or len(model.get('primitives', ())) != len(PARTS)):
        raise ValueError('expected reviewed Raticate schema, coordinates and 27 parts')
    for part, name in zip(model['primitives'], PARTS):
        if part.get('part') != name:
            raise ValueError('Raticate anatomy name does not match the reviewed source')
        p, n, ix, c = (part[k] for k in ('positions', 'normals', 'indices', 'base_color'))
        if (not 9 <= len(p) <= 60000 or len(p) % 3 or len(n) != len(p)
                or not 3 <= len(ix) <= 120000 or len(ix) % 3 or len(c) != 4):
            raise ValueError('invalid bounded Raticate geometry arrays')
        if any(type(i) is not int or not 0 <= i < len(p)//3 for i in ix):
            raise ValueError('Raticate triangle index is outside its primitive')
        if any(type(v) not in (float, int) or not math.isfinite(v) for v in p+n+c):
            raise ValueError('non-finite Raticate geometry')
    if geometry_digest(model) != REST_DIGEST:
        raise ValueError('unreviewed neutral geometry; Raticate anatomy must be reviewed again')


def skin_weights(part, position):
    x,y,z=position
    if part.startswith(('Long splayed rat foot','Long rat toe','Rat toe claw')):
        return [0,0,0,0],[1.,0.,0.,0.]
    if part.startswith(('Flared cupped rat ear','Recessed ear interior')):
        return [2 if x<-.021 else 3,0,0,0],[1.,0.,0.,0.]
    if part.startswith(('Short bent forearm','Small grasping forepaw','Three curled forepaw digits','Forepaw claw')):
        return [4 if x<-.021 else 5,0,0,0],[1.,0.,0.,0.]
    if part.startswith(('Long tapered bare rat tail','Subtle tail segmentation')):
        distance=math.sqrt((x+.021)**2+(y-.287)**2+(z+.209)**2)
        base=smoothstep(.025,.12,distance);tip=smoothstep(.34,.62,y)
        return [1,6,7,0],[1.-base,base*(1.-tip),base*tip,0.]
    return [1,0,0,0],[1.,0.,0.,0.]


def pose_angles(clip,t):
    if clip not in ('idle','attack','hit') or not 0.<=t<=1.:
        raise ValueError('unknown clip or normalized time outside [0, 1]')
    if t in (0.,1.):return [(0.,0.,0.)]*len(JOINTS)
    wave=math.sin(math.tau*t);breath=.5-.5*math.cos(math.tau*t)
    def lag(p):return math.sin(math.tau*t-p)+math.sin(p)
    if clip=='idle':return [(0.,0.,0.),(.011*breath,.012*wave,0.),
        (.023*lag(.2),0.,-.013*breath),(.019*lag(.5),0.,.013*breath),
        (.024*lag(.2),.009*wave,-.014*breath),(.021*lag(.5),-.009*wave,.014*breath),
        (.009*wave,.022*wave,0.),(.017*lag(.7),.033*lag(.7),0.)]
    brace=envelope(t,((0.,0.),(.18,1.),(.40,.72),(.65,.22),(1.,0.)))
    trail=envelope(t,((0.,0.),(.28,1.),(.52,.60),(.78,-.12),(1.,0.)))
    if clip=='attack':return [(0.,0.,0.),(.029*brace,0.,0.),
        (.046*trail,0.,-.024*trail),(.042*trail,0.,.024*trail),
        (.077*brace,.018*brace,-.043*brace),(.069*brace,-.018*brace,.043*brace),
        (.021*brace,.037*brace,0.),(.030*trail,.060*trail,0.)]
    recoil=envelope(t,((0.,0.),(.17,1.),(.39,.30),(.66,-.10),(1.,0.)))
    return [(0.,0.,0.),(-.022*recoil,.008*recoil,0.),
        (-.031*trail,0.,.018*trail),(-.027*trail,0.,-.018*trail),
        (-.049*recoil,0.,.031*recoil),(-.045*recoil,0.,-.031*recoil),
        (-.017*recoil,-.028*recoil,0.),(-.025*trail,-.046*trail,0.)]


def export_raticate(model):
    validate_model(model)
    glb = Glb()
    glb.doc['asset']['generator'] = 'Geothite reconstructed Raticate skin authoring v1'
    inverse, nodes = [], glb.doc['nodes']
    for name, parent, pivot in JOINTS:
        origin = JOINTS[parent][2] if parent is not None else (0., 0., 0.)
        nodes.append({'name': f'raticate/{name}', 'translation': [a-b for a, b in zip(pivot, origin)]})
        if parent is not None:
            nodes[parent].setdefault('children', []).append(len(nodes)-1)
        x, y, z = pivot
        inverse.extend([1., 0., 0., 0., 0., 1., 0., 0., 0., 0., 1., 0., -x, -y, -z, 1.])
    nodes.append({'name': 'raticate', 'mesh': 0, 'skin': 0})
    glb.doc['scenes'] = [{'name': 'raticate', 'nodes': [0, len(JOINTS)]}]
    glb.doc['skins'] = [{'name': 'raticate', 'skeleton': 0, 'joints': list(range(len(JOINTS))),
                         'inverseBindMatrices': glb.accessor(inverse, 'MAT4')}]
    primitives, colors = [], {}
    shared = {}
    def shared_accessor(values,kind,component=5126,target=None,normalized=False):
        key=(tuple(values),kind,component,target,normalized)
        if key not in shared:shared[key]=glb.accessor(values,kind,component,target,normalized=normalized)
        return shared[key]
    for part in model['primitives']:
        color = tuple(part['base_color'])
        if color not in colors:
            colors[color] = len(colors)
            glb.doc['materials'].append({'name': f'raticate/{MATERIAL_NAMES[colors[color]]}',
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
            'JOINTS_0': shared_accessor(ids, 'VEC4', 5121, 34962),
            'WEIGHTS_0': shared_accessor(weights, 'VEC4', 5121, 34962, normalized=True)},
            'indices': shared_accessor(part['indices'], 'SCALAR', 5123, 34963),
            'material': colors[color], 'mode': 4, 'extras': {'part': part['part']}})
    glb.doc['meshes'] = [{'name': 'raticate', 'primitives': primitives}]
    for clip, duration, count in CLIPS:
        times = glb.accessor([i*duration/(count-1) for i in range(count)], 'SCALAR', bounds=True)
        samplers, channels = [], []
        for joint in range(1, len(JOINTS)):
            rotations = [v for i in range(count)
                         for v in quaternion_xyz(*pose_angles(clip, i/(count-1))[joint])]
            channels.append({'sampler': len(samplers), 'target': {'node': joint, 'path': 'rotation'}})
            samplers.append({'input': times, 'output': glb.accessor(rotations, 'VEC4'), 'interpolation': 'LINEAR'})
        glb.doc['animations'].append({'name': f'raticate.{clip}', 'channels': channels, 'samplers': samplers,
            'extras': {'loopSuggested': clip == 'idle', 'cueRelative': clip != 'idle',
                'purpose': 'grounded pear-body brace; coherent face and incisors; grasping forepaws; lagged ears and bare tail; game-owned battle cues'}})
    glb.doc['extras'] = {'coordinate_system': COORDINATES, 'source_name': 'raticate',
        'source_sha256': REST_DIGEST, 'reference_span': .96, 'body_reference_height': .96, 'full_silhouette_height': .96,
        'geometry_preservation': 'exact source primitive order, part names, f32 positions/normals/colors and triangle index order',
        'animation_contract': 'surviving Raticate sculpture; fixed splayed feet; identity root; eight named joints; rotation-only clips'}
    assert len(glb.doc['accessors']) <= 256
    return glb.bytes()


def decode_raticate(blob):
    """Bounded canonical neutral read, rejecting any ignored storage/rig change."""
    if not 28 <= len(blob) <= MAX_BYTES:
        raise ValueError('Raticate GLB is truncated or exceeds its bounded size')
    try:
        if struct.unpack_from('<III', blob) != (0x46546c67, 2, len(blob)):
            raise ValueError('invalid Raticate GLB header')
        jslen, jstype = struct.unpack_from('<II', blob, 12)
        if jstype != 0x4e4f534a or jslen % 4 or 20+jslen+8 > len(blob):
            raise ValueError('invalid Raticate JSON chunk')
        doc = json.loads(blob[20:20+jslen])
        binlen, bintype = struct.unpack_from('<II', blob, 20+jslen)
        if bintype != 0x004e4942 or binlen % 4 or 28+jslen+binlen != len(blob):
            raise ValueError('invalid Raticate binary chunk')
        binary = blob[28+jslen:]
        if (len(doc['meshes']) != 1 or len(doc['meshes'][0]['primitives']) != 63
                or not 1 <= len(doc['accessors']) <= 256 or len(doc['materials']) != 8):
            raise ValueError('unexpected Raticate document dimensions')

        def attribute(index, kind, component):
            if type(index) is not int or not 0 <= index < len(doc['accessors']):
                raise ValueError('invalid Raticate accessor reference')
            a = doc['accessors'][index]
            if (a['type'], a['componentType']) != (kind, component):
                raise ValueError('unsupported Raticate accessor')
            count = a['count']
            width, size, code = (3, 4, 'f') if kind == 'VEC3' else (1, 2, 'H')
            if type(count) is not int or not 0 < count <= 120000:
                raise ValueError('invalid Raticate accessor count')
            vi = a['bufferView']
            if type(vi) is not int or not 0 <= vi < len(doc['bufferViews']):
                raise ValueError('invalid Raticate buffer view')
            view = doc['bufferViews'][vi]
            start, length = view['byteOffset'], view['byteLength']
            if (type(start) is not int or start < 0 or start % 4 or length != count*width*size
                    or start+length > len(binary)):
                raise ValueError('Raticate accessor is outside its buffer')
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
        if export_raticate(model) != blob:
            raise ValueError('unsupported Raticate storage, skin, hierarchy, or animation')
        return model
    except (KeyError, IndexError, TypeError, struct.error, OverflowError, RecursionError, UnicodeError) as error:
        raise ValueError('malformed or unsupported Raticate GLB') from error


def read_raticate(path):
    with Path(path).open('rb') as source:
        return decode_raticate(source.read(MAX_BYTES+1))


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--input', type=Path, required=True, help='canonical GLB or original migration JSON')
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    with args.input.open('rb') as source:
        content = source.read(MAX_BYTES+1)
    if len(content) > MAX_BYTES:
        raise ValueError('Raticate authoring input exceeds its bounded size')
    model = decode_raticate(content) if args.input.suffix == '.glb' else json.loads(content)
    blob = export_raticate(model)
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_bytes(blob)
    print(f'{args.output}: {len(blob)} bytes; sha256 {hashlib.sha256(blob).hexdigest()}')


if __name__ == '__main__':
    main()
