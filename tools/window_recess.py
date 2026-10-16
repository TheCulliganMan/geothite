"""Closed, shallow window pockets for the authored Blender exterior kits."""
import math
import bpy
from mathutils import Vector


def carve_window_pocket(collection, x, z, width, height, y, angle=0):
    """Cut the opaque wall before installing the hollow frame and rear glass."""
    rotation = math.radians(angle)
    local = Vector((x, y + .16, z))
    center = Vector((local.x * math.cos(rotation) - local.y * math.sin(rotation),
                     local.x * math.sin(rotation) + local.y * math.cos(rotation), z))
    bpy.ops.mesh.primitive_cube_add(size=1, location=center)
    cutter = bpy.context.object
    cutter.name = 'Temporary window pocket cutter'
    cutter.dimensions = (width + .002, .44, height + .002)
    cutter.rotation_euler.z = rotation
    bpy.context.view_layer.update()

    def bounds(obj):
        points = [obj.matrix_world @ Vector(corner) for corner in obj.bound_box]
        return ([min(p[i] for p in points) for i in range(3)],
                [max(p[i] for p in points) for i in range(3)])

    low, high = bounds(cutter)
    for obj in list(collection.objects):
        if obj == cutter or obj.type != 'MESH' or len(obj.data.polygons) < 6:
            continue
        if obj.data.materials and obj.data.materials[0].name.startswith(('glass', 'leaf')):
            continue
        lo, hi = bounds(obj)
        if any(hi[i] <= low[i] or lo[i] >= high[i] for i in range(3)):
            continue
        modifier = obj.modifiers.new('True recessed window opening', 'BOOLEAN')
        modifier.operation = 'DIFFERENCE'
        modifier.solver = 'EXACT'
        modifier.object = cutter
        bpy.context.view_layer.objects.active = obj
        bpy.ops.object.modifier_apply(modifier=modifier.name)
    data = cutter.data
    bpy.data.objects.remove(cutter, do_unlink=True)
    if data.users == 0:
        bpy.data.meshes.remove(data)


def build_window_ring(collection, material, name, x, z, width, height, y, depth, thickness, angle=0):
    """One closed annular frame; no hidden overlapping box faces."""
    outer=[(-width/2,-height/2),(width/2,-height/2),(width/2,height/2),(-width/2,height/2)]
    inner=[(-width/2+thickness,-height/2+thickness),
           (width/2-thickness,-height/2+thickness),
           (width/2-thickness,height/2-thickness),(-width/2+thickness,height/2-thickness)]
    vertices=[(u,v,w) for v in [-depth/2,depth/2] for ring in [outer,inner] for u,w in ring]
    faces=[]
    for i in range(4):
        j=(i+1)%4
        faces.extend([(i,j,4+j,4+i),(8+j,8+i,12+i,12+j),
                      (j,i,8+i,8+j),(4+i,4+j,12+j,12+i)])
    mesh=bpy.data.meshes.new(name);mesh.from_pydata(vertices,[],faces);mesh.materials.append(material);mesh.update()
    obj=bpy.data.objects.new(name,mesh);collection.objects.link(obj)
    a=math.radians(angle);obj.location=(x*math.cos(a)-y*math.sin(a),x*math.sin(a)+y*math.cos(a),z)
    obj.rotation_euler.z=a
    return obj
