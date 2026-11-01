#!/usr/bin/env python3
"""Original closed arcade prize display; no game artwork or layout is exported."""
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
PAINT = {
    "charcoal": [0.045, 0.052, 0.058, 1.0],
    "drawer": [0.10, 0.12, 0.13, 1.0],
    "steel": [0.62, 0.66, 0.66, 1.0],
    "brass": [0.62, 0.40, 0.12, 1.0],
    "coral_enamel": [0.80, 0.30, 0.25, 1.0],
    "violet_enamel": [0.58, 0.37, 0.73, 1.0],
    "cream_enamel": [0.90, 0.79, 0.51, 1.0],
    "teal_enamel": [0.12, 0.46, 0.49, 1.0],
}


def document():
    parts = []

    def prism(name, bounds, material, corner=0.0):
        x0, x1, y0, y1, z0, z1 = bounds
        assert x0 < x1 and y0 < y1 and z0 < z1
        c = min(corner, (x1-x0)*0.2, (z1-z0)*0.2)
        ring = ([(x0,z0),(x1,z0),(x1,z1),(x0,z1)] if c == 0 else
                [(x0+c,z0),(x1-c,z0),(x1,z0+c),(x1,z1-c),
                 (x1-c,z1),(x0+c,z1),(x0,z1-c),(x0,z0+c)])
        n = len(ring)
        vertices = [(x,y,z) for y in (y0,y1) for x,z in ring]
        faces = [tuple(range(n)), tuple(range(2*n-1,n-1,-1))]
        faces += [(i,i+n,(i+1)%n+n,(i+1)%n) for i in range(n)]
        positions, normals, indices = [], [], []
        for face in faces:
            for i in range(1,len(face)-1):
                points = [vertices[face[j]] for j in (0,i,i+1)]
                a,b,d = points
                u=[b[k]-a[k] for k in range(3)];v=[d[k]-a[k] for k in range(3)]
                normal=[u[1]*v[2]-u[2]*v[1],u[2]*v[0]-u[0]*v[2],u[0]*v[1]-u[1]*v[0]]
                length=sum(q*q for q in normal)**0.5
                assert length > 0
                normal=[q/length for q in normal]
                for point in points:
                    indices.append(len(positions)//3)
                    positions.extend(point);normals.extend(normal)
        parts.append(dict(name=name,material=material,base_color=PAINT[material],
                          positions=positions,normals=normals,indices=indices))

    prism("Closed dark lower cabinet", (0,32,0,8,0,9.65), "charcoal", 0.3)
    prism("Closed rear of prize case", (0,32,8,28,0,1.0), "charcoal")
    for x0,x1 in [(0,1),(31,32)]:
        prism("Closed display side", (x0,x1,8,28,0,10), "charcoal")
    for x in [0,8,16,24,31.5]:
        prism("Silver display upright", (x,x+0.5,8,28,9.45,10), "steel")
    for y in [8,15,22,27.5]:
        prism("Warm shelf lip", (0,32,y,y+0.5,0.8,10), "brass")
    for y in [23,25,27]:
        prism("Upper grille crossbar", (0.5,31.5,y,y+0.35,9.55,9.9), "steel")
    for x in range(2,32,2):
        prism("Upper grille picket", (x,x+0.3,22.5,28,9.5,9.85), "steel")
    for x in [0,8,16,24]:
        prism("Inset drawer panel", (x+0.7,x+7.3,1.2,6.8,9.65,9.95), "drawer", 0.1)
        prism("Small brass drawer pull", (x+3,x+5,5.3,5.7,9.95,10), "brass", 0.05)
    colors = ["coral_enamel","cream_enamel","teal_enamel","violet_enamel"]
    for row, floor in enumerate([8.5,15.5]):
        for column,x in enumerate([2,10,18,26]):
            material=colors[(column+row)%4]
            prism("Beveled prize package", (x,x+4,floor,floor+4.4,4.0,7.5), material, 0.55)
            prism("Prize package gold band", (x+1.55,x+2.05,floor+0.4,floor+4.0,7.5,7.6), "brass")
    return dict(name="arcade_prize_rack", coordinate_system="right-handed Y-up",
                origin=[0,0,0], bounds=dict(min=[0,0,0],max=[32,28,10]),
                triangle_count=sum(len(p["indices"])//3 for p in parts), primitives=parts)


if __name__ == "__main__":
    path=ROOT/"crates/crystal-voxel-view/models/interiors/arcade_prize_rack.mesh.json"
    path.write_text(json.dumps(document(),separators=(",",":"))+"\n")
    print(path)
