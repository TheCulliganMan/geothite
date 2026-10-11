#!/usr/bin/env python3
"""Original arcade wall model. No source artwork or game layout is exported."""
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
PAINT = {
    "shadow": [0.25, 0.22, 0.055, 1.0],
    "timber": [0.42, 0.25, 0.075, 1.0],
    "ochre": [0.64, 0.44, 0.095, 1.0],
    "gold": [0.84, 0.64, 0.22, 1.0],
}


def document():
    parts = []

    def box(name, bounds, material):
        x0, x1, y0, y1, z0, z1 = bounds
        assert x0 < x1 and y0 < y1 and z0 < z1
        faces = [
            ([(x1,y0,z0),(x1,y1,z0),(x1,y1,z1),(x1,y0,z1)], (1,0,0)),
            ([(x0,y0,z1),(x0,y1,z1),(x0,y1,z0),(x0,y0,z0)], (-1,0,0)),
            ([(x0,y1,z0),(x0,y1,z1),(x1,y1,z1),(x1,y1,z0)], (0,1,0)),
            ([(x0,y0,z1),(x0,y0,z0),(x1,y0,z0),(x1,y0,z1)], (0,-1,0)),
            ([(x0,y0,z1),(x1,y0,z1),(x1,y1,z1),(x0,y1,z1)], (0,0,1)),
            ([(x1,y0,z0),(x0,y0,z0),(x0,y1,z0),(x1,y1,z0)], (0,0,-1)),
        ]
        positions, normals, indices = [], [], []
        for points, normal in faces:
            first = len(positions) // 3
            positions.extend(v for point in points for v in point)
            normals.extend(normal * 4)
            indices.extend(first + i for i in (0,1,2,0,2,3))
        parts.append(dict(name=name, positions=positions, normals=normals,
                          indices=indices, base_color=PAINT[material], material=material))

    box("Closed wall substrate", (0,16,0,28,0,1.6), "shadow")
    for left, right in [(0,1),(7.5,8.5),(15,16)]:
        box("Timber panel stile", (left,right,0,28,1.6,3), "timber")
    for bottom, top in [(0,2),(13,15),(26,28)]:
        box("Warm horizontal panel rail", (0,16,bottom,top,1.6,3), "timber")
    for left, right in [(1,7.5),(8.5,15)]:
        for bottom, top in [(2,13),(15,26)]:
            box("Ochre recessed panel", (left,right,bottom,top,1.6,2.15), "ochre")
            box("Panel upper bead", (left+.25,right-.25,top-.65,top-.3,2.15,2.5), "gold")
            box("Panel lower bead", (left+.25,right-.25,bottom+.3,bottom+.65,2.15,2.5), "gold")
    box("Golden coping", (0,16,27.2,28,0,3), "gold")
    return dict(name="arcade_wall", coordinate_system="right-handed Y-up", origin=[0,0,0],
                bounds=dict(min=[0,0,0], max=[16,28,3]),
                triangle_count=sum(len(p["indices"])//3 for p in parts), primitives=parts)


if __name__ == "__main__":
    path = ROOT / "crates/crystal-voxel-view/models/interiors/arcade_wall.mesh.json"
    path.write_text(json.dumps(document(), separators=(",", ":")) + "\n")
    print(path)
