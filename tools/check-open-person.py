#!/usr/bin/env python3
"""Check an ignored converted person's real skin, texture and animation payload.

This checks the exported data, not in-game appearance or locomotion quality.
Uses the repository's existing GLB reader and writes no asset catalog.
"""
import argparse
import math
import struct
from pathlib import Path

from animated_glb import parse_glb


def accessor_values(doc, binary, index):
    accessor = doc['accessors'][index]
    assert 'sparse' not in accessor
    view = doc['bufferViews'][accessor['bufferView']]
    assert view.get('buffer') == 0
    width = {'SCALAR': 1, 'VEC2': 2, 'VEC3': 3, 'VEC4': 4, 'MAT4': 16}[accessor['type']]
    code, size = {5121: ('B', 1), 5123: ('H', 2), 5125: ('I', 4), 5126: ('f', 4)}[accessor['componentType']]
    count = accessor['count']
    assert 0 < count <= 1000000
    start = view.get('byteOffset', 0) + accessor.get('byteOffset', 0)
    stride = view.get('byteStride', width * size)
    assert stride >= width * size
    assert start >= view.get('byteOffset', 0)
    assert start + (count-1)*stride + width*size <= view.get('byteOffset', 0) + view['byteLength'] <= len(binary)
    values = [v for i in range(count) for v in struct.unpack_from('<'+code*width, binary, start+i*stride)]
    if accessor.get('normalized', False):
        assert code in ('B', 'H')
        denominator = 255 if code == 'B' else 65535
        values = [v/denominator for v in values]
    return values


def check(path):
    doc, binary = parse_glb(path.read_bytes())
    assert len(doc.get('skins', [])) == 1, 'expected one retained skin'
    skin = doc['skins'][0]
    assert len(skin['joints']) >= 16, 'missing humanoid deform skeleton'
    assert len(set(skin['joints'])) == len(skin['joints']), 'duplicate skin joints'
    inverse = accessor_values(doc, binary, skin['inverseBindMatrices'])
    assert len(inverse) == len(skin['joints']) * 16
    assert all(math.isfinite(v) for v in inverse)
    assert doc.get('images') and all('bufferView' in im for im in doc['images']), 'missing embedded skin texture'
    vertices = triangles = 0
    for node in doc['nodes']:
        if 'mesh' not in node:
            continue
        assert node.get('skin') == 0, 'body was flattened or detached from its rig'
        for primitive in doc['meshes'][node['mesh']]['primitives']:
            attr = primitive['attributes']
            assert {'POSITION', 'NORMAL', 'TEXCOORD_0', 'JOINTS_0', 'WEIGHTS_0'} <= attr.keys()
            positions = accessor_values(doc, binary, attr['POSITION'])
            indices = accessor_values(doc, binary, primitive['indices'])
            count = len(positions) // 3
            assert count > 0 and all(math.isfinite(v) for v in positions)
            assert len(indices) % 3 == 0 and all(0 <= i < count for i in indices)
            normals = accessor_values(doc, binary, attr['NORMAL'])
            assert len(normals) == count * 3 and all(math.isfinite(v) for v in normals)
            assert all(abs(sum(n*n for n in normals[i:i+3]) - 1) < 0.005 for i in range(0, len(normals), 3)), 'invalid exported normals'
            uv = accessor_values(doc, binary, attr['TEXCOORD_0'])
            assert len(uv) == count * 2 and all(math.isfinite(v) for v in uv)
            sets = []
            for suffix in ('0', '1'):
                if 'WEIGHTS_' + suffix in attr:
                    weights = accessor_values(doc, binary, attr['WEIGHTS_' + suffix])
                    joints = accessor_values(doc, binary, attr['JOINTS_' + suffix])
                    assert len(weights) == len(joints) == count * 4
                    assert all(math.isfinite(w) and 0 <= w <= 1 for w in weights)
                    assert all(0 <= j < len(skin['joints']) for j in joints)
                    sets.append(weights)
            assert sets, 'missing skin weights'
            assert all(abs(sum(sum(w[i*4:i*4+4]) for w in sets) - 1) < 0.001 for i in range(count)), 'weights no longer sum to one'
            material = doc['materials'][primitive['material']]
            assert 'baseColorTexture' in material['pbrMetallicRoughness']
            vertices += count
            triangles += len(indices) // 3
    assert vertices, 'missing character mesh'
    clips = {}
    for animation in doc.get('animations', []):
        name = animation.get('name')
        assert name and name not in clips, 'unnamed or duplicate animation'
        duration = 0
        changes = 0
        for channel in animation['channels']:
            target = channel['target']
            assert target['path'] in ('rotation', 'translation', 'scale')
            sampler = animation['samplers'][channel['sampler']]
            times = accessor_values(doc, binary, sampler['input'])
            values = accessor_values(doc, binary, sampler['output'])
            assert times and all(math.isfinite(t) and t >= 0 for t in times)
            assert all(a < b for a, b in zip(times, times[1:])), 'nonincreasing animation keys'
            size = 4 if target['path'] == 'rotation' else 3
            assert len(values) == len(times) * size and all(math.isfinite(v) for v in values)
            duration = max(duration, times[-1] - times[0])
            changes += int(any(abs(values[i] - values[i % size]) > 0.00001 for i in range(len(values))))
        assert duration > 0 and changes, 'animation is static or empty'
        clips[name] = round(duration, 4)
    assert {'Idle', 'Run', 'Jump'} <= clips.keys(), f'missing source clips: {clips}'
    print(f'{path.name}: {len(skin["joints"])} skin joints, {vertices} weighted vertices, {triangles} triangles, embedded texture, clips {clips}')


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('model', type=Path)
    check(parser.parse_args().model)
