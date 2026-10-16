#!/usr/bin/env python3
"""Validate the authored fruit tree's geometry and orchard scale."""
import argparse
import math
from pathlib import Path

from model_asset_storage import read_model_json

ROOT = Path(__file__).resolve().parents[1]


def verify(path):
    model = read_model_json(path)
    assert model['name'] == 'fruit_tree' and model['version'] == 1
    assert model['coordinate_system'] == '+Y up; front +Z; floor-centered root'
    points = []
    parts = set()
    triangles = 0
    for primitive in model['primitives']:
        label = primitive.get('part', '')
        parts.add(label)
        positions, normals, indices = (primitive[k] for k in ('positions', 'normals', 'indices'))
        assert len(positions) >= 9 and len(positions) % 3 == 0, label
        assert len(normals) == len(positions) and len(indices) % 3 == 0, label
        assert all(math.isfinite(v) for v in positions + normals), label
        assert all(0 <= v <= 1 for v in primitive['base_color']), label
        ps = list(zip(*[iter(positions)] * 3))
        ns = list(zip(*[iter(normals)] * 3))
        assert all(type(i) is int and 0 <= i < len(ps) for i in indices), label
        assert all(abs(sum(v*v for v in n) - 1) < 0.00002 for n in ns), label
        volume = 0
        for i in range(0, len(indices), 3):
            ids = indices[i:i+3]
            a, b, c = (ps[j] for j in ids)
            u = [b[k]-a[k] for k in range(3)]
            v = [c[k]-a[k] for k in range(3)]
            cross = [u[1]*v[2]-u[2]*v[1], u[2]*v[0]-u[0]*v[2], u[0]*v[1]-u[1]*v[0]]
            average = [sum(ns[j][k] for j in ids) for k in range(3)]
            assert sum(cross[k]*average[k] for k in range(3)) >= -1e-7, (label, 'winding')
            volume += a[0]*(b[1]*c[2]-b[2]*c[1])+a[1]*(b[2]*c[0]-b[0]*c[2])+a[2]*(b[0]*c[1]-b[1]*c[0])
        assert volume >= -1e-5, (label, 'inverted volume')
        points.extend(ps)
        triangles += len(indices)//3
    low = [min(p[a] for p in points) for a in range(3)]
    high = [max(p[a] for p in points) for a in range(3)]
    assert abs(low[1]) < 1e-6 and abs(high[1]-2.4) < 1e-6, ('floor/height', low, high)
    # Broad crown overhangs the fixed interaction root, without spanning a path.
    assert max(abs(low[0]), abs(high[0])) <= 1.050001, ('width', low, high)
    assert max(abs(low[2]), abs(high[2])) <= 0.950001, ('depth', low, high)
    assert 0 < triangles < 18000, triangles
    assert 'Continuous fluted orchard trunk' in parts
    assert sum(p.startswith('Orchard root flare /') for p in parts) == 5
    assert sum(p.startswith('Orchard leaf /') for p in parts) == 800
    assert sum(p.startswith('Orchard twig /') for p in parts) == 50
    assert not any(p.startswith('Recessed foliage /') for p in parts)
    berries = [p for p in parts if p.startswith('Ripe berry /')]
    assert 6 <= len(berries) <= 16
    assert sum(p.startswith('Hanging berry stem /') for p in parts) == len(berries)
    print(f'PASS: fruit tree; {triangles:,} triangles, {len(berries)} hanging berries; finite attributes, outward winding, floor root, height and footprint')


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--model', type=Path, default=ROOT/'crates/crystal-voxel-view/models/actor_props/fruit_tree.mesh.json')
    verify(parser.parse_args().model)
