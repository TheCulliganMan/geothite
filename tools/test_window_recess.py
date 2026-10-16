"""Geometry checks for actual wall openings and recessed exterior glazing."""
import json
from pathlib import Path
import unittest

ROOT = Path(__file__).resolve().parents[1] / 'crates/crystal-voxel-view/models'


def triangles(model):
    for primitive in model['primitives']:
        points = list(zip(*[iter(primitive['positions'])] * 3))
        normals = list(zip(*[iter(primitive['normals'])] * 3))
        for a, b, c in zip(*[iter(primitive['indices'])] * 3):
            yield primitive['material'], [points[a], points[b], points[c]], normals[a]


def depth_at(points, x, y):
    a, b, c = points
    determinant = (b[0]-a[0])*(c[1]-a[1])-(b[1]-a[1])*(c[0]-a[0])
    if abs(determinant) < 1e-9:
        return None
    u = ((x-a[0])*(c[1]-a[1])-(y-a[1])*(c[0]-a[0]))/determinant
    v = ((b[0]-a[0])*(y-a[1])-(b[1]-a[1])*(x-a[0]))/determinant
    if u < -1e-6 or v < -1e-6 or u+v > 1.000001:
        return None
    return a[2]+u*(b[2]-a[2])+v*(c[2]-a[2])


class WindowRecessTests(unittest.TestCase):
    def test_glass_is_behind_wall_plane_and_apertures_are_open(self):
        for asset in ['world_exteriors/modern_house', 'world_exteriors/modern_blank',
                      'world_exteriors/modern_shop', 'new_bark/house']:
            model = json.loads((ROOT / (asset+'.mesh.json')).read_text())
            surfaces = list(triangles(model))
            checked = 0
            for material, points, normal in surfaces:
                if material != 'glass' or normal[2] < .999:
                    continue
                x, y, z = [sum(p[i] for p in points)/3 for i in range(3)]
                if z <= .5 or abs(x) < .5:
                    continue  # Reverse panes and central door glazing.
                if max(p[0] for p in points)-min(p[0] for p in points) < .08:
                    continue  # The narrow closed edges of side windows.
                # A real opening has no opaque wall between the pane and air.
                blockers = [depth_at(p, x, y) for m, p, n in surfaces
                            if not m.startswith('glass') and n[2] > .99]
                self.assertFalse(any(d is not None and d > z+.002 for d in blockers),
                                 (asset, 'opaque wall still covers inset glass', x, y))
                walls = [depth_at(p, xx, y) for m, p, n in surfaces
                         if m.startswith('plaster') and n[2] > .99
                         for xx in [x-.7, x+.7]]
                walls = [d for d in walls if d is not None]
                self.assertTrue(walls, (asset, 'nearby facade must remain'))
                self.assertGreater(max(walls)-z, .025, (asset, 'pane projects outside wall'))
                checked += 1
            self.assertGreaterEqual(checked, 4, (asset, 'both front window apertures'))


if __name__ == '__main__':
    unittest.main()
