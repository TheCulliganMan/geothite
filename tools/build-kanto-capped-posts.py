"""Original capped Kanto post with continuous shaft, collar, dome and underside.

blender -b --threads 2 --python tools/build-kanto-capped-posts.py -- \
    target/kanto-capped-posts [--skip-preview]

No source images or pack data are imported. The editable mesh exports to the
material-indexed Y-up runtime format. Preview copies use the same actual mesh.
"""
import bpy
import bmesh
import json
import math
import sys
from pathlib import Path
from mathutils import Vector

args = sys.argv[sys.argv.index('--') + 1:] if '--' in sys.argv else []
output = Path(next((a for a in args if not a.startswith('--')), 'target/kanto-capped-posts'))
output.mkdir(parents=True, exist_ok=True)
bpy.ops.object.select_all(action='SELECT')
bpy.ops.object.delete(use_global=False)
for m in list(bpy.data.materials):
    bpy.data.materials.remove(m)
scene = bpy.context.scene
scene.render.engine = 'CYCLES'
scene.cycles.samples = 40
scene.cycles.use_denoising = False
scene.render.resolution_x = 1450
scene.render.resolution_y = 1000
scene.render.resolution_percentage = 100
scene.render.image_settings.file_format = 'PNG'
scene.view_settings.view_transform = 'AgX'


def material(name, color):
    m = bpy.data.materials.new(name)
    m.diffuse_color = (*color, 1)
    m.use_nodes = True
    bsdf = m.node_tree.nodes['Principled BSDF']
    bsdf.inputs['Base Color'].default_value = (*color, 1)
    bsdf.inputs['Roughness'].default_value = .84
    return m


materials = [material(label, color) for label, color in [
    ('Post | closed underside', (.16,.18,.18)),
    ('Post | bevelled dark foot', (.24,.27,.27)),
    ('Post | cool stone shaft', (.40,.45,.45)),
    ('Post | shaft facet', (.46,.50,.49)),
    ('Post | collar shadow', (.28,.33,.33)),
    ('Post | cap rim', (.63,.67,.63)),
    ('Post | pale rounded cap', (.79,.82,.74)),
    ('Post | cap crown', (.88,.89,.80)),
]]

# The radial rings are a single watertight solid. A slim tall shaft, neck
# undercut, overhanging cap and bevelled toe are distinct silhouettes at any
# angle. There are no floating overlays, image cards or inaccessible backs.
profile = [
    (0.000,.190), (.035,.236), (.095,.245), (.150,.214),
    (.805,.214), (.845,.211), (.872,.257), (.930,.292),
    (1.002,.287), (1.090,.247), (1.166,.168), (1.202,.083),
]
segment_materials = [1,1,2,2,4,4,5,5,6,6,7]
radial = 12
vertices = []
for z, radius in profile:
    for i in range(radial):
        angle = 2 * math.pi * i / radial + math.pi / radial
        vertices.append((radius * math.cos(angle), radius * math.sin(angle), z))
bottom = len(vertices)
vertices.append((0,0,0))
top = len(vertices)
vertices.append((0,0,1.212))
faces, slots = [], []
for i in range(radial):
    j = (i + 1) % radial
    faces.append((bottom,j,i)); slots.append(0)
for ring, slot in enumerate(segment_materials):
    for i in range(radial):
        j = (i + 1) % radial
        faces.append((ring*radial+i, ring*radial+j, (ring+1)*radial+j, (ring+1)*radial+i))
        slots.append(3 if ring == 3 and i % 3 == 0 else slot)
for i in range(radial):
    faces.append(((len(profile)-1)*radial+i, (len(profile)-1)*radial+(i+1)%radial, top))
    slots.append(7)
mesh = bpy.data.meshes.new('Kanto capped post | continuous closed sculpt')
mesh.from_pydata(vertices,[],faces)
mesh.update()
for m in materials:
    mesh.materials.append(m)
for face, slot in zip(mesh.polygons, slots):
    face.material_index = slot
    face.use_smooth = False
bm = bmesh.new()
bm.from_mesh(mesh)
assert all(e.is_manifold for e in bm.edges)
bmesh.ops.recalc_face_normals(bm, faces=list(bm.faces))
volume = bm.calc_volume(signed=True)
assert volume > .17
bm.to_mesh(mesh)
bm.free()
mesh.update()
collection = bpy.data.collections.new('ASSET | kanto_capped_post')
scene.collection.children.link(collection)
obj = bpy.data.objects.new('kanto_capped_post | complete editable solid',mesh)
collection.objects.link(obj)
obj['Source ownership'] = 'Exactly one native column and two rows; cap and shaft are one guarded object'
obj['Footprint'] = 'Independent post; no rails or shared wall; original route gaps remain open'
obj['Authoring'] = 'Original closed radial sculpture, with eight material regions and no textures'

mesh.calc_loop_triangles()
groups = {}
low, high = [float('inf')]*3, [-float('inf')]*3
for tri in mesh.loop_triangles:
    m = mesh.materials[tri.material_index]
    group = groups.setdefault(m.name, {'name':m.name, 'base_color':[round(c,6) for c in m.diffuse_color],
                                     'positions':[], 'normals':[], 'indices':[], 'lookup':{}})
    for vi in tri.vertices:
        v, n = mesh.vertices[vi].co, tri.normal
        p = (round(v.x,6),round(v.z,6),round(-v.y,6))
        normal = (round(n.x,6),round(n.z,6),round(-n.y,6))
        for axis in range(3):
            low[axis], high[axis] = min(low[axis],p[axis]), max(high[axis],p[axis])
        key = p + normal
        if key not in group['lookup']:
            group['lookup'][key] = len(group['positions'])//3
            group['positions'].extend(p)
            group['normals'].extend(normal)
        group['indices'].append(group['lookup'][key])
primitives = []
for group in groups.values():
    del group['lookup']
    primitives.append(group)
data = {'name':'kanto_capped_post', 'coordinate_system':'right-handed; +Y up; front +Z',
        'bounds':{'min':low,'max':high}, 'primitives':primitives}
(output/'kanto_capped_post.mesh.json').write_text(json.dumps(data,separators=(',',':'))+'\n')
stats = {'triangles':len(mesh.loop_triangles), 'closed_components':1, 'materials':len(primitives),
         'volume':volume, 'bounds':data['bounds']}
assert stats['triangles'] == 288
(output/'asset-stats.json').write_text(json.dumps(stats,indent=2)+'\n')

# Separate presentation collection: a hero and an independently spaced course
# show the exact asset at its runtime proportions. No course is a shared wall.
presentation = bpy.data.collections.new('PRESENTATION only')
scene.collection.children.link(presentation)
obj.location = (-2.45,-.4,0)
obj.scale = (1.55,1.55,1.55)
for y in [1.10,3.12]:
    for x in [-1.4,-.58,.24,1.88,2.70]:
        duplicate = obj.copy()
        duplicate.data = mesh
        duplicate.name = 'PRESENTATION | independently spaced post'
        duplicate.location = (x,y,0)
        duplicate.scale = (1, .916667, 1)
        presentation.objects.link(duplicate)
bpy.ops.mesh.primitive_plane_add(size=200, location=(0,0,-.008))
floor = bpy.context.object
floor.name = 'PRESENTATION only | pale paving'
for col in list(floor.users_collection):
    col.objects.unlink(floor)
presentation.objects.link(floor)
floor.data.materials.append(material('Presentation | pale paving',(.45,.47,.42)))
world = bpy.data.worlds.new('Kanto post workshop daylight')
scene.world = world
world.use_nodes = True
world.node_tree.nodes['Background'].inputs['Color'].default_value = (.52,.57,.60,1)
world.node_tree.nodes['Background'].inputs['Strength'].default_value = .65
for name,location,energy,size in [('Key',(-3,-6,9),950,6),('Fill',(5,5,6),700,5)]:
    bpy.ops.object.light_add(type='AREA',location=location)
    light = bpy.context.object
    light.name = name
    light.data.energy = energy
    light.data.size = size
    light.rotation_euler = (Vector((0,1,.5))-light.location).to_track_quat('-Z','Y').to_euler()
bpy.ops.object.camera_add(location=(7,-10,8))
camera = bpy.context.object
camera.data.type = 'ORTHO'
camera.data.ortho_scale = 8.1
camera.rotation_euler = (Vector((0,1.0,.65))-camera.location).to_track_quat('-Z','Y').to_euler()
scene.camera = camera
scene['Source'] = 'Original capped post; no imported game assets; front and rear proof views'
bpy.ops.wm.save_as_mainfile(filepath=str(output/'kanto-capped-posts.blend'),compress=False)
if '--skip-preview' not in args:
    scene.render.filepath = str(output/'kanto-capped-posts-front.png')
    bpy.ops.render.render(write_still=True)
    camera.location = (-7,9,3.8)
    camera.rotation_euler = (Vector((0,1,.65))-camera.location).to_track_quat('-Z','Y').to_euler()
    scene.render.filepath = str(output/'kanto-capped-posts-rear.png')
    bpy.ops.render.render(write_still=True)
print('KANTO_CAPPED_POST=' + json.dumps(stats))
