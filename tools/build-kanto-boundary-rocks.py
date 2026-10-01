"""Original closed Kanto boundary stones, without imported artwork or content.

blender -b --threads 2 --python tools/build-kanto-boundary-rocks.py -- \
    target/kanto-boundary-rocks [--skip-preview]

The three named meshes are the editable source. Material regions belong to
the closed rock shell: no sprite cards, texture planes, or overlapping decals.
Runtime output is material-indexed, right-handed Y-up with flat facet normals.
"""
import bpy
import bmesh
import math
import json
import sys
from pathlib import Path
from mathutils import Vector

args = sys.argv[sys.argv.index('--') + 1:] if '--' in sys.argv else []
output = Path(next((a for a in args if not a.startswith('--')), 'target/kanto-boundary-rocks'))
output.mkdir(parents=True, exist_ok=True)
bpy.ops.object.select_all(action='SELECT')
bpy.ops.object.delete(use_global=False)
for material in list(bpy.data.materials):
    bpy.data.materials.remove(material)
scene = bpy.context.scene

PALETTES = {
    'land': [(.27,.30,.26), (.37,.41,.33), (.48,.51,.41), (.59,.62,.51), (.72,.73,.61), (.65,.68,.53)],
    'shore': [(.19,.25,.26), (.28,.35,.35), (.39,.46,.43), (.51,.58,.51), (.67,.72,.61), (.59,.65,.53)],
    'path': [(.39,.38,.32), (.52,.50,.40), (.64,.62,.49), (.75,.72,.58), (.89,.84,.68), (.80,.77,.60)],
}
NAMES = ['Closed underside', 'Weathered lower stone', 'Broad stone faces',
         'Cut upper shoulders', 'Pale crown', 'Mineral facet']
assets = {}
stats = {}

def material(name, color, wet=False):
    m = bpy.data.materials.new(name)
    m.diffuse_color = (*color, 1)
    m.use_nodes = True
    bsdf = m.node_tree.nodes['Principled BSDF']
    bsdf.inputs['Base Color'].default_value = (*color, 1)
    bsdf.inputs['Roughness'].default_value = .67 if wet else .92
    return m

def stone(family, variant):
    name = 'kanto_boundary_' + family
    collection = bpy.data.collections.new(name)
    scene.collection.children.link(collection)
    assets[name] = collection
    mats = [material(family + ' | ' + label, c, family == 'shore' and i < 2)
            for i, (label, c) in enumerate(zip(NAMES, PALETTES[family]))]
    # Seven angular courses give a broad planted foot, a small erosion seam,
    # rounded shoulders, and an off-centre crown. The asymmetry is authored
    # deterministically; each family has its own silhouette and mineral color.
    count = 14
    heights = [0, .115, .25, .43, .83, 1.02, 1.11]
    radii = [.61, .84, .94, 1.0, .92, .72, .44]
    if family == 'path':
        heights = [0, .10, .22, .38, .80, 1.00, 1.10]
        radii = [.70, .88, .96, 1.0, .91, .73, .43]
    if family == 'shore':
        heights = [0, .12, .26, .39, .77, .98, 1.10]
        radii = [.66, .88, .93, 1.0, .92, .75, .44]
    vertices = []
    for ring, (z, radius) in enumerate(zip(heights, radii)):
        for i in range(count):
            angle = 2 * math.pi * i / count + .11 * variant
            uneven = 1 + .037 * math.sin(i * 2.1 + variant) + .035 * math.cos(i * 3.7 + .6 * ring)
            # Two tiny bites form real facets in the rim, not drawn cracks.
            if ring in (4, 5) and i in ((3 + variant) % count, (10 + variant) % count):
                uneven -= .085
            dx = .055 * ring / 6 * math.sin(variant + .4)
            dy = -.08 * ring / 6
            zz = z + (0 if ring == 0 else .027 * math.sin(i * 1.8 + variant) * ring / 6)
            vertices.append((math.cos(angle) * radius * uneven + dx,
                             math.sin(angle) * radius * uneven * .86 + dy, zz))
    bottom = len(vertices)
    vertices.append((0, 0, 0))
    top = len(vertices)
    vertices.append((.11 * math.cos(variant), -.12, 1.155))
    faces, slots = [], []
    for i in range(count):
        j = (i + 1) % count
        faces.append((bottom, j, i)); slots.append(0)
    for ring in range(len(heights) - 1):
        for i in range(count):
            j = (i + 1) % count
            a, b = ring * count + i, ring * count + j
            c, d = (ring + 1) * count + j, (ring + 1) * count + i
            # Alternating diagonals prevent a spiral strip in the light.
            pair = [(a,b,d),(b,c,d)] if (i + ring) % 2 else [(a,b,c),(a,c,d)]
            base = [1, 1, 2, 2, 3, 4][ring]
            for half, face in enumerate(pair):
                faces.append(face)
                slot = 5 if ring >= 4 and (i * 3 + ring + variant + half) % 13 == 0 else base
                slots.append(slot)
    for i in range(count):
        faces.append((6 * count + i, 6 * count + (i + 1) % count, top))
        slots.append(4 if (i + variant) % 4 else 5)
    mesh = bpy.data.meshes.new(name + ' | continuous manifold')
    mesh.from_pydata(vertices, [], faces)
    mesh.update()
    obj = bpy.data.objects.new(name + ' | hand-cut closed stone', mesh)
    collection.objects.link(obj)
    for m in mats:
        mesh.materials.append(m)
    for polygon, slot in zip(mesh.polygons, slots):
        polygon.material_index = slot
        polygon.use_smooth = False
    bm = bmesh.new(); bm.from_mesh(mesh)
    assert all(e.is_manifold for e in bm.edges), name
    bmesh.ops.recalc_face_normals(bm, faces=list(bm.faces))
    assert bm.calc_volume(signed=True) > .5, name
    bm.to_mesh(mesh); bm.free(); mesh.update()
    obj['Authoring'] = 'Original closed, continuous stone with material faces; no image dependencies'
    obj['Family'] = family
    obj['Runtime footprint'] = 'Complete native 2x2 source drawing; original collision and footing unchanged'
    export(name, obj)

def export(name, obj):
    mesh = obj.data
    mesh.calc_loop_triangles()
    groups = {}
    low, high = [float('inf')] * 3, [-float('inf')] * 3
    for triangle in mesh.loop_triangles:
        mat = mesh.materials[triangle.material_index]
        g = groups.setdefault(mat.name, {'name': mat.name, 'base_color': [round(v, 6) for v in mat.diffuse_color],
                                        'positions': [], 'normals': [], 'indices': [], 'lookup': {}})
        for vi in triangle.vertices:
            p, n = mesh.vertices[vi].co, triangle.normal
            pos = (round(p.x,6), round(p.z,6), round(-p.y,6))
            normal = (round(n.x,6), round(n.z,6), round(-n.y,6))
            key = pos + normal
            for k in range(3):
                low[k], high[k] = min(low[k],pos[k]), max(high[k],pos[k])
            if key not in g['lookup']:
                g['lookup'][key] = len(g['positions']) // 3
                g['positions'].extend(pos); g['normals'].extend(normal)
            g['indices'].append(g['lookup'][key])
    primitives = []
    for g in groups.values():
        del g['lookup']; primitives.append(g)
    triangles = sum(len(g['indices']) // 3 for g in primitives)
    assert 100 < triangles < 500, (name, triangles)
    data = {'name': name, 'coordinate_system': 'right-handed; +Y up; front +Z',
            'bounds': {'min': low, 'max': high}, 'primitives': primitives}
    (output / (name + '.mesh.json')).write_text(json.dumps(data, separators=(',',':')) + '\n')
    stats[name] = {'triangles': triangles, 'parts': 1, 'closed_components': 1, 'materials': len(primitives),
                   'bounds': data['bounds']}

for variant, family in enumerate(('land', 'shore', 'path')):
    stone(family, variant)

# Presentation copies show the actual authored meshes and all-side geometry.
presentation = bpy.data.collections.new('PRESENTATION only')
scene.collection.children.link(presentation)
ground_material = material('Presentation | warm neutral', (.19,.22,.20))
for column, (name, collection) in enumerate(assets.items()):
    for obj in collection.objects:
        obj.location = ((column - 1) * 2.9, 0, 0)
        for y, rotation in [(3.0, math.pi), (5.9, math.pi * .45)]:
            duplicate = obj.copy(); duplicate.data = obj.data
            presentation.objects.link(duplicate)
            duplicate.name = name + (' | rear' if y == 3.0 else ' | side')
            duplicate.location = ((column - 1) * 2.9, y, 0)
            duplicate.rotation_euler.z = rotation
bpy.ops.mesh.primitive_plane_add(size=200, location=(0,0,-.025))
floor = bpy.context.object; floor.name = 'PRESENTATION only | ground'
for col in list(floor.users_collection): col.objects.unlink(floor)
presentation.objects.link(floor); floor.data.materials.append(ground_material)
world = bpy.data.worlds.new('Kanto stone workshop daylight'); scene.world = world; world.use_nodes = True
world.node_tree.nodes['Background'].inputs['Color'].default_value = (.43,.48,.51,1)
world.node_tree.nodes['Background'].inputs['Strength'].default_value = .7
for name, location, energy, size in [('Key',(-5,-6,11),1800,7), ('Fill',(6,4,8),1250,6)]:
    bpy.ops.object.light_add(type='AREA', location=location)
    light = bpy.context.object; light.name = name; light.data.energy = energy; light.data.size = size
    light.rotation_euler = (Vector((0,2,0))-light.location).to_track_quat('-Z','Y').to_euler()
bpy.ops.object.camera_add(location=(8,-12,13))
cam = bpy.context.object; cam.data.type = 'ORTHO'; cam.data.ortho_scale = 12.8
cam.rotation_euler = (Vector((0,2.3,.4))-cam.location).to_track_quat('-Z','Y').to_euler(); scene.camera = cam
scene.render.engine = 'CYCLES'; scene.cycles.samples = 48; scene.cycles.use_denoising = False
scene.render.resolution_x = 1600; scene.render.resolution_y = 1250; scene.render.resolution_percentage = 100
scene.render.image_settings.file_format = 'PNG'; scene.view_settings.view_transform = 'AgX'
scene['Source'] = 'Original editable Kanto boundary stones. No game assets or textures imported.'
scene['Columns'] = 'Land / shore / pale path; front / back / side rows'
bpy.ops.wm.save_as_mainfile(filepath=str(output / 'kanto-boundary-rocks.blend'), compress=False)
(output / 'asset-stats.json').write_text(json.dumps(stats, indent=2) + '\n')
if '--skip-preview' not in args:
    scene.render.filepath = str(output / 'kanto-boundary-rocks-preview.png')
    bpy.ops.render.render(write_still=True)
    # Low rear view proves backing and volume instead of only a flattering top view.
    for obj in presentation.objects:
        if obj.type == 'MESH' and obj != floor:
            obj.hide_render = True
    cam.location = (6,8,4.3); cam.data.ortho_scale = 10.0
    cam.rotation_euler = (Vector((0,0,.4))-cam.location).to_track_quat('-Z','Y').to_euler()
    scene.render.filepath = str(output / 'kanto-boundary-rocks-rear.png')
    scene.render.resolution_x = 1500; scene.render.resolution_y = 780
    bpy.ops.render.render(write_still=True)
print('KANTO_BOUNDARY_ROCKS=' + json.dumps(stats))
