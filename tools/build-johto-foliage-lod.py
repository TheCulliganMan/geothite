"""Reduced original Johto foliage; no downloaded geometry, textures, or game art.

Run from the repository root:
    blender -b --python tools/build-johto-foliage-lod.py -- target/johto-foliage-lod
Add --skip-preview to skip the inspection render. Generated JSON/GLB, editable
Blender source, and comparison/stats are written only to that ignored directory.
The runtime assets are the two JSON exports copied to models/johto by the author.

The tree starts with the original six authored crown lobes. Exact unions remove
interior overlap before collapse reduction; named trunk/root parts are rebuilt
with only their readable silhouettes. Palette and full original bounds survive.
Grass retains six solid folded blades with a closed triangular cross section.
"""
from pathlib import Path
import json
import math
import sys

ROOT = Path(__file__).resolve().parents[1]
# Reuse the original modeling vocabulary and authored make_tree() definition,
# stopping before the kit's generation/export/presentation side effects.
source = (ROOT / 'tools/build-new-bark-models.py').read_text()
exec(compile(source.split('\nmake_house();')[0], 'new_bark_modeling_vocabulary', 'exec'))
import bmesh

P = P.resolve()
TREE_SOURCE = ROOT / 'crates/crystal-voxel-view/models/new_bark/tree.mesh.json'
GRASS_SOURCE = ROOT / 'crates/crystal-voxel-view/models/johto/grass.mesh.json'
references = {'tree': json.loads(TREE_SOURCE.read_text()),
              'grass': json.loads(GRASS_SOURCE.read_text())}


def active(obj):
    bpy.ops.object.select_all(action='DESELECT')
    obj.select_set(True)
    bpy.context.view_layer.objects.active = obj


def apply_transform(obj):
    active(obj)
    bpy.ops.object.transform_apply(location=True, rotation=True, scale=True)


def consistent_normals(obj):
    bm = bmesh.new()
    bm.from_mesh(obj.data)
    bmesh.ops.recalc_face_normals(bm, faces=list(bm.faces))
    bm.to_mesh(obj.data)
    bm.free()
    obj.data.update()


def source_bounds(data):
    lo, hi = data['bounds']['min'], data['bounds']['max']
    return [lo[0], -hi[2], lo[1]], [hi[0], -lo[2], hi[1]]


def normalize(objects, lower, upper):
    """Match all six original extrema without changing the original ground plane."""
    for obj in objects:
        apply_transform(obj)
    lo = [min(v.co[k] for obj in objects for v in obj.data.vertices) for k in range(3)]
    hi = [max(v.co[k] for obj in objects for v in obj.data.vertices) for k in range(3)]
    for obj in objects:
        for v in obj.data.vertices:
            for k in range(3):
                v.co[k] = lower[k] + (v.co[k] - lo[k]) * (upper[k] - lower[k]) / (hi[k] - lo[k])
        obj.data.update()
        consistent_normals(obj)


def make_tree_lod():
    make_tree()
    original = ASSETS.pop('tree')
    original.name = 'Authored crown construction'
    # Keep the procedural source lobes editable in the inspection blend, but
    # hidden so that neither export nor comparison accidentally includes them.
    original.hide_render = True
    original.hide_viewport = True
    begin('tree_lod')
    lobes = []
    for src in original.objects:
        if src.name.startswith('Sculpted broadleaf crown'):
            obj = src.copy()
            obj.data = src.data.copy()
            COL.objects.link(obj)
            apply_transform(obj)
            lobes.append(obj)
    crown = lobes[0]
    crown.name = 'Continuous six-lobe crown, hidden faces removed'
    for other in lobes[1:]:
        active(crown)
        modifier = crown.modifiers.new('Remove overlapped lobe interior', 'BOOLEAN')
        modifier.operation = 'UNION'
        modifier.solver = 'EXACT'
        modifier.object = other
        bpy.ops.object.modifier_apply(modifier=modifier.name)
        bpy.data.objects.remove(other, do_unlink=True)
    crown.data.calc_loop_triangles()
    before = len(crown.data.loop_triangles)
    # 212 crown triangles leave room for a readable trunk and three broad roots.
    active(crown)
    modifier = crown.modifiers.new('Silhouette-preserving crown LOD', 'DECIMATE')
    modifier.decimate_type = 'COLLAPSE'
    modifier.ratio = 212 / before
    modifier.use_collapse_triangulate = True
    bpy.ops.object.modifier_apply(modifier=modifier.name)
    consistent_normals(crown)
    # Preserve the exact canopy envelope, independent of the trunk and roots.
    lo, hi = source_bounds(references['tree'])
    canopy_low = min((o.matrix_world @ v.co).z for o in original.objects
                     if o.name.startswith('Sculpted broadleaf crown') for v in o.data.vertices)
    normalize([crown], [lo[0], lo[1], canopy_low], hi)
    cone('Seven-sided warm trunk', (0, 0, .78), .20, .145, 1.56, 'timber_light', 7)
    for i in range(3):
        angle = i * math.tau / 3 + .2
        ca, sa = math.cos(angle), math.sin(angle)
        def p(r, w, z):
            return (ca*r-sa*w, sa*r+ca*w, z)
        # A closed tetrahedral root: broad at the ground, tapered into the trunk.
        mesh('Broad grounded root', [p(.04,-.095,0),p(.04,.095,0),p(.43,0,.025),p(.07,0,.25)],
             [(0,2,1),(0,1,3),(1,2,3),(2,0,3)], 'timber')
    # Each original extreme is matched again after adding the grounded detail.
    objects = [o for o in COL.objects if o.type == 'MESH']
    normalize(objects, lo, hi)
    return COL


def make_grass_lod():
    begin('grass_lod')
    # Spread six golden-angle blades across the old tuft's footprint rather
    # than making one dense central fan. Heights/leans come from its authoring.
    for i in [0, 2, 3, 5, 6, 9]:
        a = i * 2.399963
        r = .10 + .25 * ((i * 7) % 11) / 10
        x, y = math.cos(a) * r, math.sin(a) * r
        h = .29 + .30 * ((i * 3) % 11) / 10
        lean = .10 + .07 * (i % 3)
        width = (.045 + .012 * (i % 2)) * 1.23
        dx, dy = math.cos(a) * lean, math.sin(a) * lean
        px, py = -math.sin(a) * width, math.cos(a) * width
        # Two bent triangular side panels (four tris), one back (one tri), and
        # a basal cap (one tri). A folded solid, never a camera-facing billboard.
        verts = [(x-px,y-py,0), (x+px,y+py,0),
                 (x+dx*.42,y+dy*.42,h*.63), (x+dx,y+dy,h),
                 (x+dx*.14,y+dy*.14,0)]
        mesh('Six-triangle folded meadow blade', verts,
             [(0,4,2),(0,2,3),(4,1,2),(1,3,2),(0,3,1),(0,1,4)],
             ['leaf','leaf_light','leaf_gold'][(i//2)%3])
    normalize(list(COL.objects), *source_bounds(references['grass']))
    return COL


def export_asset(name, col):
    objects = [o for o in col.objects if o.type == 'MESH']
    groups = {}
    for obj in objects:
        apply_transform(obj)
        consistent_normals(obj)
        obj.data.calc_loop_triangles()
        for tri in obj.data.loop_triangles:
            # Face-local vertices preserve the original flat-shaded visual style.
            if tri.normal.length_squared < .99:
                raise ValueError(f'{name}: degenerate triangle')
            material = obj.data.materials[tri.material_index]
            group = groups.setdefault(material.name, {
                'name':name+'__'+material.name, 'material':material.name,
                'base_color':[round(c,5) for c in material.diffuse_color],
                'positions':[], 'normals':[], 'indices':[], 'lookup':{}})
            n = tri.normal.normalized()
            norm = (round(n.x,6),round(n.z,6),round(-n.y,6))
            for vi in tri.vertices:
                v = obj.data.vertices[vi].co
                pos = (round(v.x,6),round(v.z,6),round(-v.y,6))
                key = pos + norm
                if key not in group['lookup']:
                    group['lookup'][key] = len(group['positions']) // 3
                    group['positions'].extend(pos)
                    group['normals'].extend(norm)
                group['indices'].append(group['lookup'][key])
    primitives = [groups[k] for k in sorted(groups)]
    for group in primitives:
        del group['lookup']
    positions = [g['positions'][i:i+3] for g in primitives for i in range(0,len(g['positions']),3)]
    lo = [min(p[k] for p in positions) for k in range(3)]
    hi = [max(p[k] for p in positions) for k in range(3)]
    reference = references[name.removesuffix('_lod')]
    assert {'min':lo,'max':hi} == reference['bounds'], (name,lo,hi,reference['bounds'])
    triangles = sum(len(g['indices']) // 3 for g in primitives)
    vertices = sum(len(g['positions']) // 3 for g in primitives)
    assert triangles <= (300 if name == 'tree_lod' else 40)
    assert vertices <= (900 if name == 'tree_lod' else 120)
    for group in primitives:
        assert all(math.isfinite(v) for v in group['positions'] + group['normals'])
        for i in range(0,len(group['normals']),3):
            assert abs(sum(v*v for v in group['normals'][i:i+3])-1) < .000003
    data = {'name':name,'coordinate_system':'right-handed; +Y up; front +Z',
            'origin':'floor-center','bounds':{'min':lo,'max':hi},
            'dimensions':reference['dimensions'],'triangle_count':triangles,
            'primitive_count':len(primitives),'primitives':primitives}
    (P / f'{name}.mesh.json').write_text(json.dumps(data,separators=(',',':')))
    bpy.ops.object.select_all(action='DESELECT')
    for obj in objects:
        obj.select_set(True)
    bpy.context.view_layer.objects.active = objects[0]
    bpy.ops.export_scene.gltf(filepath=str(P / f'{name}.glb'),use_selection=True,
        export_format='GLB',export_yup=True,export_apply=True,
        export_materials='EXPORT',export_animations=False)
    return {'triangles':triangles,'vertices':vertices,'bounds':data['bounds'],
            'triangle_reduction_percent':round(100*(1-triangles/reference['triangle_count']),2)}


def import_reference(name, data):
    begin(name)
    for group in data['primitives']:
        p = group['positions']
        verts = [(p[i],-p[i+2],p[i+1]) for i in range(0,len(p),3)]
        ix = group['indices']
        faces = [ix[i:i+3] for i in range(0,len(ix),3)]
        mesh(group['name']+' reference',verts,faces,group['material'])
    return COL


tree_col = make_tree_lod()
grass_col = make_grass_lod()
stats = {name:export_asset(name,ASSETS[name]) for name in ['tree_lod','grass_lod']}
(P/'asset-stats.json').write_text(json.dumps(stats,indent=2))
original_tree = import_reference('Original tree reference', references['tree'])
original_grass = import_reference('Original grass reference', references['grass'])
for col, offset in [(original_tree,(-1.7,.4,0)),(tree_col,(1.7,.4,0)),
                    (original_grass,(-1.7,-1.55,0)),(grass_col,(1.7,-1.55,0))]:
    for obj in col.objects:
        obj.location += Vector(offset)
begin('PRESENTATION only')
cube('Warm studio floor',(0,0,-.115),(200,200,.20),'cream')
world = bpy.data.worlds.new('Foliage comparison daylight')
scene.world = world
world.use_nodes = True
world.node_tree.nodes['Background'].inputs['Color'].default_value = (.30,.39,.48,1)
world.node_tree.nodes['Background'].inputs['Strength'].default_value = .5
for name,loc,energy,size in [('Key',(-7,-9,12),1600,8),('Fill',(7,-3,10),1100,7),('Rim',(1,10,12),1500,6)]:
    bpy.ops.object.light_add(type='AREA',location=loc)
    obj = bpy.context.object
    obj.name = name
    obj.data.energy = energy
    obj.data.shape = 'DISK'
    obj.data.size = size
    obj.rotation_euler = (Vector((0,0,1.5))-obj.location).to_track_quat('-Z','Y').to_euler()
bpy.ops.object.camera_add(location=(0,-13,8.5))
cam = bpy.context.object
cam.rotation_euler = (Vector((0,-.1,1.5))-cam.location).to_track_quat('-Z','Y').to_euler()
cam.data.type = 'ORTHO'
cam.data.ortho_scale = 8.4
scene.camera = cam
for x,title,detail in [(-1.7,'ORIGINAL','Tree 724 tris | Grass 88 tris'),
                       (1.7,'FOLIAGE LOD',f'Tree {stats["tree_lod"]["triangles"]} tris | Grass {stats["grass_lod"]["triangles"]} tris')]:
    for text,z,size in [(title,3.95,.24),(detail,3.65,.13)]:
        curve = bpy.data.curves.new('Comparison label','FONT')
        curve.body = text
        curve.align_x = 'CENTER'
        curve.size = size
        obj = bpy.data.objects.new(text,curve)
        COL.objects.link(obj)
        obj.location = (x,.4,z)
        obj.rotation_euler = cam.rotation_euler
        obj.data.materials.append(MATS['ink'])
scene.render.engine = 'CYCLES'
scene.cycles.samples = 32
scene.cycles.use_denoising = False
scene.render.resolution_x = 1500
scene.render.resolution_y = 1050
scene.render.resolution_percentage = 100
scene.view_settings.view_transform = 'AgX'
scene.render.image_settings.file_format = 'PNG'
scene.render.filepath = str(P/'foliage-original-vs-lod.png')
scene['Foliage source'] = 'Original authored Johto kit; full original meshes left, silhouette-preserving reduced meshes right.'
scene['Editing note'] = 'Authored crown construction retains editable source lobes; unhide only for editing. Presentation offsets are applied after exports.'
bpy.ops.wm.save_as_mainfile(filepath=str(P/'johto-foliage-lod-editable.blend'))
if '--skip-preview' not in args:
    bpy.ops.render.render(write_still=True)
print('FINAL_STATS='+json.dumps(stats))
