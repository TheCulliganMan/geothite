#!/usr/bin/env python3
"""Original closed leafy terrace planter; stores geometry and color anchors only."""
import json
import math
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
MATERIALS = {
    'terrace_orange': ([1., .23, .003, 1.], [2, 12]),
    'terrace_cream': ([.93, .80, .67, 1.], [2, 9]),
    'terrace_yellow': ([1., 1., .04, 1.], [4, 13]),
    'terrace_earth': ([.04, .04, .04, 1.], [0, 0]),
    'terrace_leaf': ([.20, .38, .003, 1.], [4, 2]),
    'terrace_leaf_light': ([.30, .56, .07, 1.], [2, 1]),
    'terrace_leaf_dark': ([.07, .15, 0., 1.], [9, 2]),
}


def document():
    parts = []

    def poly(name, vertices, faces, material):
        positions, normals, indices = [], [], []
        center = [sum(v[k] for v in vertices)/len(vertices) for k in range(3)]
        for face in faces:
            for j in range(1, len(face)-1):
                points = [vertices[face[k]] for k in (0, j, j+1)]
                a, b, c = points
                u = [b[k]-a[k] for k in range(3)]
                v = [c[k]-a[k] for k in range(3)]
                normal = [u[1]*v[2]-u[2]*v[1], u[2]*v[0]-u[0]*v[2], u[0]*v[1]-u[1]*v[0]]
                length = sum(n*n for n in normal)**.5
                assert length > 1e-8
                if sum(normal[k]*(sum(p[k] for p in points)/3-center[k]) for k in range(3)) < 0:
                    points.reverse()
                    normal = [-n for n in normal]
                normal = [n/length for n in normal]
                for point in points:
                    indices.append(len(positions)//3)
                    positions.extend(point)
                    normals.extend(normal)
        color, anchor = MATERIALS[material]
        parts.append(dict(name=name, material=material, base_color=color,
                          source_pixel=anchor, positions=positions, normals=normals, indices=indices))

    def box(name, bounds, material):
        x0,x1,y0,y1,z0,z1 = bounds
        vertices = [(x,y,z) for y in (y0,y1) for z in (z0,z1) for x in (x0,x1)]
        poly(name, vertices, [(0,1,3,2),(4,6,7,5),(0,4,5,1),
                              (2,3,7,6),(0,2,6,4),(1,5,7,3)], material)

    box('Closed orange ceramic trough', (0,32,0,4.8,0,12), 'terrace_orange')
    box('Recessed dark planting earth', (.8,31.2,4.8,5.05,.8,11.2), 'terrace_earth')
    for name, bounds in [('Front', (0,32,4.8,5.6,10.9,12)),
                         ('Rear', (0,32,4.8,5.6,0,1.1)),
                         ('Left', (0,1.1,4.8,5.6,1.1,10.9)),
                         ('Right', (30.9,32,4.8,5.6,1.1,10.9))]:
        box(name+' cream rolled trough rim', bounds, 'terrace_cream')
    # Front ornaments are shallow closed octagonal studs, not painted cards.
    for x in (4.,12.,20.,28.):
        for name, radius, z0,z1,material in [('Cream surround',1.05,12,12.17,'terrace_cream'),
                                            ('Yellow center',.61,12.17,12.28,'terrace_yellow')]:
            vertices = [(x+radius*math.cos(i*math.tau/8),2.4+radius*math.sin(i*math.tau/8),z)
                        for z in (z0,z1) for i in range(8)]
            poly(name,vertices,[tuple(range(8)),tuple(range(8,16))]+
                 [(i,(i+1)%8,(i+1)%8+8,i+8) for i in range(8)], material)
    # Each leaf is a sealed pointed volume with a raised midrib and a lower keel.
    # Layered radial fans form broad crowns while leaving distinct leaf silhouettes.
    for plant,cx in enumerate((5.4,16.,26.6)):
        for tier in range(2):
            for j in range(8):
                angle = j*math.tau/8 + tier*.37 + plant*.21
                dx,dz = math.cos(angle),math.sin(angle)
                length = 4.7 if tier == 0 else 3.5
                root = (cx,5.2+tier*1.2,6.)
                tip = (cx+dx*length,8.0+tier*3.2+(.6 if j%3==0 else 0),6.+dz*length*.75)
                center = [(root[k]+tip[k])*.5 for k in range(3)]
                width = 1.35 if tier == 0 else 1.2
                left = (center[0]-dz*width,center[1],center[2]+dx*width)
                right = (center[0]+dz*width,center[1],center[2]-dx*width)
                ridge = (center[0],center[1]+.8,center[2])
                keel = (center[0],center[1]-.25,center[2])
                poly(f'Bush {plant+1} layered leaf {tier*8+j+1}',
                     [root,left,tip,right,ridge,keel],
                     [(4,i,(i+1)%4) for i in range(4)]+
                     [(5,(i+1)%4,i) for i in range(4)],
                     ['terrace_leaf','terrace_leaf_light','terrace_leaf_dark'][(j+plant+tier)%3])
    vertices = [v for p in parts for v in zip(*[iter(p['positions'])]*3)]
    return dict(name='roof_planter',coordinate_system='right-handed; +Y up; front +Z',
                origin='floor-center',bounds=dict(min=[min(v[k] for v in vertices) for k in range(3)],
                max=[max(v[k] for v in vertices) for k in range(3)]),
                triangle_count=sum(len(p['indices'])//3 for p in parts),primitives=parts)


if __name__ == '__main__':
    path = ROOT/'crates/crystal-voxel-view/models/special_rooms/roof_planter.mesh.json'
    path.write_text(json.dumps(document(),separators=(',',':'))+'\n')
    print(path)
