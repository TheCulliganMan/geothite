#!/usr/bin/env python3
"""Validate original interior runtime exports without a content pack or Blender."""
import json
import math
from pathlib import Path
from model_asset_storage import read_model_bytes, read_model_json, read_model_text, validate_storage

ROOT = Path(__file__).resolve().parent.parent
MODELS = ROOT / 'crates/crystal-voxel-view/models/interiors'
IMPORTER = ROOT / 'crates/crystal-voxel-view/src/interior_models.rs'

def validate():
    paths = sorted(MODELS.glob('*.mesh.json'))
    assert len(paths) == 68, f'Expected 68 original assets, got {len(paths)}'
    importer = IMPORTER.read_text()
    total_triangles = 0
    total_vertices = 0
    for path in paths:
        data = read_model_json(path)
        assert f'/interiors/{path.name}' in importer, path
        assert data['coordinate_system'] == 'right-handed; +Y up; front +Z', path
        lo = [float('inf')] * 3
        hi = [-float('inf')] * 3
        triangles = 0
        materials = set()
        for p in data['primitives']:
            n = len(p['positions']) // 3
            assert n > 0 and len(p['positions']) % 3 == 0, path
            assert len(p['normals']) == n * 3, path
            assert p['indices'] and len(p['indices']) % 3 == 0, path
            assert all(isinstance(i, int) and 0 <= i < n for i in p['indices']), path
            assert all(math.isfinite(v) for v in p['positions'] + p['normals']), path
            assert len(p['base_color']) == 4 and all(0 <= c <= 1 for c in p['base_color']), path
            materials.add(tuple(p['base_color']))
            for i in range(0, len(p['positions']), 3):
                for axis in range(3):
                    lo[axis] = min(lo[axis], p['positions'][i + axis])
                    hi[axis] = max(hi[axis], p['positions'][i + axis])
                normal = p['normals'][i:i + 3]
                assert abs(sum(v * v for v in normal) - 1) < 0.0001, path
            triangles += len(p['indices']) // 3
            total_vertices += n
        assert triangles == data['triangle_count'] and triangles >= 50, path
        assert len(materials) >= 2, f'{path}: no material differentiation'
        assert all(hi[a] > lo[a] for a in range(3)), f'{path}: flat card'
        assert lo == data['bounds']['min'] and hi == data['bounds']['max'], path
        total_triangles += triangles
    print(f'Validated {len(paths)} materialed full-volume interior assets; {total_triangles:,} triangles, {total_vertices:,} exported vertices')

if __name__ == '__main__':
    validate()
