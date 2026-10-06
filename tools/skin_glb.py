"""Shared deterministic GLB authoring primitives for articulated species."""
import json
import math
import struct

COORDINATES = '+Y up; front +Z; floor-centered root'
COMPONENTS = {5121: ('B', 1), 5123: ('H', 2), 5125: ('I', 4), 5126: ('f', 4)}
WIDTHS = {'SCALAR': 1, 'VEC3': 3, 'VEC4': 4, 'MAT4': 16}


def canonical(value):
    return json.dumps(value, sort_keys=True, separators=(',', ':'), allow_nan=False).encode()


def packed(values, component=5126):
    return struct.pack('<' + COMPONENTS[component][0] * len(values), *values)


def smoothstep(lo, hi, value):
    t = max(0., min(1., (value - lo) / (hi - lo)))
    return t * t * (3. - 2. * t)


def quantized_weights(weights):
    """UNORM8 largest-remainder quantization, with integer sum exactly 255."""
    scaled = [255. * w for w in weights]
    quantized = [math.floor(w) for w in scaled]
    remaining = 255 - sum(quantized)
    order = sorted(range(4), key=lambda i: (-(scaled[i] - quantized[i]), i))
    for i in order[:remaining]:
        quantized[i] += 1
    return quantized


class Glb:
    """A deliberately small independent writer for this authoring proof."""
    def __init__(self):
        self.doc = {'asset': {'version': '2.0', 'generator': 'Geothite Gengar skin authoring v1'},
                    'scene': 0, 'scenes': [], 'nodes': [], 'meshes': [], 'skins': [],
                    'materials': [], 'accessors': [], 'bufferViews': [], 'animations': []}
        self.data = bytearray()

    def accessor(self, values, kind, component=5126, target=None, bounds=False, normalized=False):
        width = WIDTHS[kind]
        if not values or len(values) % width:
            raise ValueError('invalid accessor length')
        self.data.extend(b'\0' * (-len(self.data) % 4))
        blob = packed(values, component)
        view = {'buffer': 0, 'byteOffset': len(self.data), 'byteLength': len(blob)}
        if target is not None:
            view['target'] = target
        a = {'bufferView': len(self.doc['bufferViews']), 'componentType': component,
             'count': len(values) // width, 'type': kind}
        if normalized:
            a['normalized'] = True
        if bounds:
            decoded = struct.unpack('<' + COMPONENTS[component][0] * len(values), blob)
            a.update(min=[min(decoded[j::width]) for j in range(width)],
                     max=[max(decoded[j::width]) for j in range(width)])
        self.doc['bufferViews'].append(view)
        self.data.extend(blob)
        self.doc['accessors'].append(a)
        return len(self.doc['accessors']) - 1

    def bytes(self):
        self.doc['buffers'] = [{'byteLength': len(self.data)}]
        js = canonical(self.doc)
        js += b' ' * (-len(js) % 4)
        binary = bytes(self.data) + b'\0' * (-len(self.data) % 4)
        return (struct.pack('<III', 0x46546C67, 2, 28 + len(js) + len(binary))
                + struct.pack('<II', len(js), 0x4E4F534A) + js
                + struct.pack('<II', len(binary), 0x004E4942) + binary)


def quaternion_xyz(x, y, z):
    """Unit quaternion from local XYZ Euler rotations; glTF order is x,y,z,w."""
    sx, sy, sz = (math.sin(v * .5) for v in (x, y, z))
    cx, cy, cz = (math.cos(v * .5) for v in (x, y, z))
    return [sx * cy * cz - cx * sy * sz,
            cx * sy * cz + sx * cy * sz,
            cx * cy * sz - sx * sy * cz,
            cx * cy * cz + sx * sy * sz]


def envelope(t, keys):
    for (a, va), (b, vb) in zip(keys, keys[1:]):
        if t <= b:
            return va + (vb - va) * smoothstep(a, b, t)
    return keys[-1][1]
