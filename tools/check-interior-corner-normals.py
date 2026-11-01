"""Blender regression: curved sides stay smooth; planar caps/corners stay hard.
Run: blender -b --python tools/check-interior-corner-normals.py
"""
from pathlib import Path
import math, sys
import bpy
from mathutils import Vector
sys.path.insert(0,str(Path(__file__).resolve().parent))
from blender_corner_normals import transformed_corner_normal

bpy.ops.mesh.primitive_cylinder_add(vertices=24,radius=1,depth=2)
obj=bpy.context.object
obj.scale=(2,.5,1.3);obj.rotation_euler=(.2,.1,.3)
bpy.context.view_layer.update()
mesh=obj.data
for face in mesh.polygons:face.use_smooth=len(face.vertices)==4
mesh.update();mesh.calc_loop_triangles()
normal_matrix=obj.matrix_world.to_3x3().inverted().transposed()
curved=flat=0
for tri in mesh.loop_triangles:
    normals=[]
    for vertex,loop in zip(tri.vertices,tri.loops):
        point=mesh.vertices[vertex].co
        actual=transformed_corner_normal(mesh,loop,normal_matrix)
        local=Vector((point.x,point.y,0)) if mesh.polygons[tri.polygon_index].use_smooth else tri.normal
        expected=(normal_matrix@local).normalized()
        assert actual.dot(expected)>.99999,(actual,expected)
        normals.append(actual)
    if mesh.polygons[tri.polygon_index].use_smooth:
        assert any((n-normals[0]).length>.01 for n in normals[1:])
        curved+=1
    else:
        assert all((n-normals[0]).length<1e-6 for n in normals)
        flat+=1
assert curved==48 and flat==44,(curved,flat)
bpy.ops.mesh.primitive_cube_add()
mesh=bpy.context.object.data;mesh.calc_loop_triangles()
splits={}
for tri in mesh.loop_triangles:
    for vertex,loop in zip(tri.vertices,tri.loops):
        normal=transformed_corner_normal(mesh,loop,Vector((1,1,1)).to_track_quat('Z','Y').to_matrix())
        splits.setdefault(vertex,set()).add(tuple(round(v,5) for v in normal))
assert len(splits)==8 and all(len(values)==3 for values in splits.values())
print('PASS: 48 smooth curved triangles, 44 flat cap triangles, 8 hard three-way corners; inverse-transpose normals preserved')
