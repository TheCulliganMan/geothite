#!/usr/bin/env python3
"""Original raised gym terrace, authored without source artwork or pack exports."""
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
PAINT = {
    "mortar": [0.38, 0.29, 0.12, 1.0],
    "deck_stone": [0.73, 0.65, 0.40, 1.0],
    "deck_stone_light": [0.82, 0.74, 0.48, 1.0],
    "retaining_gold": [0.68, 0.49, 0.14, 1.0],
    "retaining_ochre": [0.56, 0.39, 0.10, 1.0],
    "stair_stone": [0.57, 0.58, 0.53, 1.0],
    "stair_lip": [0.80, 0.79, 0.66, 1.0],
}


def document():
    parts = []

    def prism(name, rect, bottom, top, material, corner=0.0):
        x0, x1, z0, z1 = rect
        assert x0 < x1 and z0 < z1 and bottom < top
        c = min(corner, (x1 - x0) * 0.2, (z1 - z0) * 0.2)
        ring = ([(x0, z0), (x1, z0), (x1, z1), (x0, z1)] if c == 0 else
                [(x0+c, z0), (x1-c, z0), (x1, z0+c), (x1, z1-c),
                 (x1-c, z1), (x0+c, z1), (x0, z1-c), (x0, z0+c)])
        n = len(ring)
        vertices = [(x, y, z) for y in (bottom, top) for x, z in ring]
        faces = [tuple(range(n)), tuple(range(2*n-1, n-1, -1))]
        faces += [(i, i+n, (i+1) % n+n, (i+1) % n) for i in range(n)]
        positions, normals, indices = [], [], []
        for face in faces:
            for j in range(1, len(face)-1):
                triangle = [vertices[face[k]] for k in (0, j, j+1)]
                a, b, d = triangle
                u = [b[k]-a[k] for k in range(3)]
                v = [d[k]-a[k] for k in range(3)]
                normal = [u[1]*v[2]-u[2]*v[1], u[2]*v[0]-u[0]*v[2],
                          u[0]*v[1]-u[1]*v[0]]
                length = sum(q*q for q in normal)**0.5
                assert length > 0
                normal = [q/length for q in normal]
                for point in triangle:
                    indices.append(len(positions)//3)
                    positions.extend(point)
                    normals.extend(normal)
        parts.append(dict(name=name, material=material, positions=positions,
                          normals=normals, indices=indices, base_color=PAINT[material]))

    # The deck, retaining frame and stair opening are one coherent terrace.
    # The two outer north corners stay outside this original authored volume.
    prism("Closed deck foundation", (16, 80, 8, 48), 0, 7.6, "mortar")
    frame = [(8, 88, 0, 8), (0, 16, 8, 48), (80, 96, 8, 48),
             (0, 32, 48, 64), (64, 96, 48, 64)]
    for j, rect in enumerate(frame):
        prism(f"Closed retaining foundation {j+1}", rect, 0, 7.0, "mortar")
        x0, x1, z0, z1 = rect
        for z in range(z0, z1, 8):
            for x in range(x0, x1, 8):
                material = "retaining_gold" if (x//8+z//8) % 3 else "retaining_ochre"
                prism(f"Cut golden retaining stone {x},{z}",
                      (x+.12, x+7.88, z+.12, z+7.88), 0, 8, material, .65)
    for z in range(8, 48, 8):
        for x in range(16, 80, 8):
            material = "deck_stone_light" if (x//8+z//8) % 3 == 0 else "deck_stone"
            prism(f"Terrace paving tile {x},{z}",
                  (x+.12, x+7.88, z+.12, z+7.88), 7.6, 8, material, .12)
    # Closed treads put the north stair at deck height and the south at half
    # height. Actor support must follow these levels in the runtime binding.
    for name, z0, z1, height in [("North", 48, 56, 8), ("South", 56, 64, 4)]:
        prism(name+" closed stair body", (32, 64, z0, z1), 0, height-.2, "stair_stone")
        prism(name+" full tread", (32, 64, z0, z1), height-.2, height, "stair_lip")
    return dict(name="olivine_stage", coordinate_system="right-handed Y-up; front +Z",
                origin=[0, 0, 0], bounds=dict(min=[0, 0, 0], max=[96, 8, 64]),
                triangle_count=sum(len(p["indices"])//3 for p in parts), primitives=parts)


if __name__ == "__main__":
    path = ROOT / "crates/crystal-voxel-view/models/interiors/olivine_stage.mesh.json"
    path.write_text(json.dumps(document(), separators=(",", ":")) + "\n")
    print(path)
