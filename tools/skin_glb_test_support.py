"""Independent binary and transform reference helpers for skin regressions."""
import json
import math
import struct

import skin_glb as author

IDENTITY = [1., 0., 0., 0., 0., 1., 0., 0., 0., 0., 1., 0., 0., 0., 0., 1.]


def decode(blob):
    magic, version, length = struct.unpack_from('<III', blob)
    if (magic, version, length) != (0x46546c67, 2, len(blob)):
        raise ValueError('invalid GLB header')
    offset, chunks = 12, []
    while offset < len(blob):
        length, kind = struct.unpack_from('<II', blob, offset)
        offset += 8
        if length % 4 or offset + length > len(blob):
            raise ValueError('invalid GLB chunk bounds')
        chunks.append((kind, blob[offset:offset + length]))
        offset += length
    if [kind for kind, _ in chunks] != [0x4e4f534a, 0x004e4942]:
        raise ValueError('expected JSON and embedded BIN chunks')
    return json.loads(chunks[0][1]), chunks[1][1]


def encode(doc, binary):
    js = author.canonical(doc)
    js += b' ' * (-len(js) % 4)
    return (struct.pack('<III', 0x46546C67, 2, 28 + len(js) + len(binary))
            + struct.pack('<II', len(js), 0x4E4F534A) + js
            + struct.pack('<II', len(binary), 0x004E4942) + binary)


def raw_accessor(doc, binary, index):
    a = doc['accessors'][index]
    view = doc['bufferViews'][a['bufferView']]
    size = author.COMPONENTS[a['componentType']][1] * author.WIDTHS[a['type']] * a['count']
    start = view.get('byteOffset', 0) + a.get('byteOffset', 0)
    if start < 0 or size > view['byteLength'] or start + size > len(binary):
        raise ValueError('invalid accessor bounds')
    return binary[start:start + size]


def values(doc, binary, index):
    a = doc['accessors'][index]
    code = author.COMPONENTS[a['componentType']][0]
    count = a['count'] * author.WIDTHS[a['type']]
    result = list(struct.unpack('<' + code * count, raw_accessor(doc, binary, index)))
    if a.get('normalized'):
        result = [v / (255. if a['componentType'] == 5121 else 65535.) for v in result]
    return result


def rows(values_, width):
    return [values_[i:i + width] for i in range(0, len(values_), width)]


def matmul(a, b):
    return [sum(a[k * 4 + row] * b[col * 4 + k] for k in range(4))
            for col in range(4) for row in range(4)]


def transform(matrix, point, w=1.):
    return [sum(matrix[k * 4 + row] * point[k] for k in range(3)) + matrix[12 + row] * w
            for row in range(3)]


def trs(translation, rotation):
    x, y, z, w = rotation
    return [1 - 2*y*y - 2*z*z, 2*x*y + 2*z*w, 2*x*z - 2*y*w, 0.,
            2*x*y - 2*z*w, 1 - 2*x*x - 2*z*z, 2*y*z + 2*x*w, 0.,
            2*x*z + 2*y*w, 2*y*z - 2*x*w, 1 - 2*x*x - 2*y*y, 0.,
            *translation, 1.]


def slerp(a, b, fraction):
    dot = sum(x * y for x, y in zip(a, b))
    if dot < 0:
        b, dot = [-x for x in b], -dot
    if dot > .9995:
        q = [x + fraction * (y - x) for x, y in zip(a, b)]
    else:
        angle = math.acos(max(-1., min(1., dot)))
        q = [(math.sin((1 - fraction) * angle) * x + math.sin(fraction * angle) * y) / math.sin(angle)
             for x, y in zip(a, b)]
    length = math.sqrt(sum(x*x for x in q))
    return [x / length for x in q]
