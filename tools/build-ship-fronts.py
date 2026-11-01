"""Original lower-deck cap/return shells. Canonical face models stay shared.

python3 tools/build-ship-fronts.py OUTPUT --runtime-only
blender -b --threads 2 --python tools/build-ship-fronts.py -- OUTPUT --skip-preview
The source's two empty corner squares and full 32px entrances stay empty.
"""
from pathlib import Path
import importlib.util
import hashlib
import json
import os
import sys

helper = Path(__file__).with_name('build-lighthouse-chamber.py')
if not helper.exists():
    helper = Path(os.environ.get('GEOTHITE_ART_TOOLS', 'tools')) / helper.name
spec = importlib.util.spec_from_file_location('chamber', helper)
m = importlib.util.module_from_spec(spec)
spec.loader.exec_module(m)
m.PALETTE.update({
    'hull_ivory': (.60, .63, .60, 1),
    'hull_trim': (.73, .75, .69, 1),
    'hull_shadow': (.32, .38, .37, 1),
    'navy_panel': (.22, .30, .32, 1),
    'bulkhead_cap': (.27, .36, .39, 1),
})
DIMS = {'room_front_west': (192, 16, 32), 'room_front_east': (192, 16, 32)}
m.DIMS.update(DIMS)
base_document = m.document
def document(asset):
    result = base_document(asset)
    result['format'] = 'geothite-ship-fronts-v1'
    return result
m.document = document

def wing_prism(asset, label, material, length, y0, y1, mirror, recessed=False, crown=False):
    # One closed wing, with a real corner void and a 16px outer return to the
    # existing U. The central facade slot ends at z29; cached faces fill z29–32.
    # A small explicit grid triangulates the concave/hole topology. Only shared
    # internal boundaries are omitted; all external top/back/end faces remain.
    if crown:
        # The dark crown is inset, exposing a narrow ivory perimeter shoulder.
        # At z32 its 1.25..14.65 return exactly meets the existing U crown.
        # Widening the two corner holes here exposes ivory; no geometry moves
        # into a native source void or entrance.
        xs = [1.25, 6.65, 14.65, 17.35, length - .85]
        zs = [1.25, 6.65, 17.35, 28.15, 32]
        occupied = {(x, z) for x in range(4) for z in range(4)
                    if (z < 3 or x < 2) and not (z == 1 and x in [1, 2])}
    else:
        xs = [0, 8, 16, length - (.75 if recessed else 0)]
        zs = [.8 if recessed else 0, 8, 16, 29, 32]
        occupied = {(x, z) for x in range(3) for z in range(4)
                    if (z < 3 or x < 2) and (x, z) != (1, 1)}
    vertices, faces, ids = [], [], {}
    def vertex(x, y, z):
        point = (192 - x if mirror else x, y, z)
        if point not in ids:
            ids[point] = len(vertices)
            vertices.append(point)
        return ids[point]
    def face(points):
        f = tuple(vertex(*p) for p in points)
        faces.append(tuple(reversed(f)) if mirror else f)
    for x, z in sorted(occupied):
        a, b, c, d = xs[x], xs[x + 1], zs[z], zs[z + 1]
        face([(a,y0,c),(b,y0,c),(b,y0,d),(a,y0,d)])
        face([(a,y1,c),(a,y1,d),(b,y1,d),(b,y1,c)])
        if (x - 1, z) not in occupied:
            face([(a,y0,c),(a,y0,d),(a,y1,d),(a,y1,c)])
        if (x + 1, z) not in occupied:
            face([(b,y0,d),(b,y0,c),(b,y1,c),(b,y1,d)])
        if (x, z - 1) not in occupied:
            face([(b,y0,c),(a,y0,c),(a,y1,c),(b,y1,c)])
        if (x, z + 1) not in occupied:
            face([(a,y0,d),(b,y0,d),(b,y1,d),(a,y1,d)])
    asset.part(label, material, vertices, faces)

def wing(asset, length, mirror):
    side = 'Starboard' if mirror else 'Port'
    wing_prism(asset, side+' continuous steel cap and return body', 'hull_ivory', length, 2.8, 14.8, mirror, True)
    wing_prism(asset, side+' continuous dark deck skirt', 'hull_shadow', length, 0, 2.8, mirror)
    wing_prism(asset, side+' continuous ivory cap rail', 'hull_trim', length, 14.8, 15.2, mirror)
    wing_prism(asset, side+' inset cool steel crown', 'bulkhead_cap', length, 15.2, 16, mirror, crown=True)
    def box(label, material, bounds, bevel=.2):
        x0,x1,y0,y1,z0,z1 = bounds
        if mirror: x0,x1 = 192-x1,192-x0
        asset.box(side+' '+label, material, (x0,x1,y0,y1,z0,z1), bevel)
    for number in range((length + 23)//24):
        a,b = 1.2+number*24, min(length-1.2, (number+1)*24-1.2)
        if b <= a: continue
        box(f'recessed corridor enamel panel {number}', 'navy_panel', (a,b,3.2,12.9,.05,.8))
        for x in [a+1.5,b-1.5]:
            box(f'corridor brass rivet {number} {x}', 'brass', (x-.25,x+.25,11.2,11.7,.015,.09), .03)
    box('open doorway return panel', 'hull_shadow', (length-.75,length-.05,3.2,12.9,1.2,27.8))

def assets():
    out = []
    for name,left,right in [('room_front_west',64,96),('room_front_east',96,64)]:
        asset = m.Asset(name)
        wing(asset,left,False)
        wing(asset,right,True)
        out.append(asset)
    return out

def build_blender(models, out, skip_preview):
    import bpy
    from mathutils import Vector
    bpy.ops.object.select_all(action='SELECT')
    bpy.ops.object.delete(use_global=False)
    scene = bpy.context.scene
    scene['kit'] = 'Original source-shaped Fast Ship room-front caps and returns'
    scene['coordinates'] = 'Runtime Y-up, +Z south. All components remain closed and editable.'
    scene['rebuild'] = 'tools/build-ship-fronts.py'
    scene['shared_faces'] = 'Canonical dungeon ShipBulkhead and PortholeBulkhead, fitted by ship_fronts.rs'
    materials = {}
    for name,color in m.PALETTE.items():
        mat = bpy.data.materials.new(name)
        mat.diffuse_color = color
        mat.use_nodes = True
        mat.node_tree.nodes['Principled BSDF'].inputs['Base Color'].default_value = color
        mat.node_tree.nodes['Principled BSDF'].inputs['Roughness'].default_value = .85
        materials[name] = mat
    proof = []
    for i,asset in enumerate(models):
        collection = bpy.data.collections.new('ASSET | '+asset.name)
        scene.collection.children.link(collection)
        collection['dimensions_pixels'] = DIMS[asset.name]
        collection['runtime_sha256'] = hashlib.sha256((out/(asset.name+'.mesh.json')).read_bytes()).hexdigest()
        for label,material,vertices,faces in asset.parts:
            mesh = bpy.data.meshes.new(label)
            mesh.from_pydata([(x/16,-z/16,y/16) for x,y,z in vertices], [], faces)
            mesh.materials.append(materials[material])
            mesh.update()
            obj = bpy.data.objects.new(asset.name+' | '+label,mesh)
            collection.objects.link(obj)
            obj.location = (0,-i*3,0)
            obj['authored_part'] = label
            for face in mesh.polygons: face.use_smooth=False
        proof.append({'asset':asset.name,'editable_mesh_objects':len(collection.objects),
                      'runtime_sha256':collection['runtime_sha256']})
    (out/'blender-source-check.json').write_text(json.dumps(proof,indent=2)+'\n')
    scene.render.engine = 'CYCLES'
    scene.cycles.samples = 16
    scene.cycles.use_denoising = False
    scene.render.resolution_x,scene.render.resolution_y = 1440,900
    scene.render.resolution_percentage = 100
    scene.world.color = (.17,.17,.17)
    for name,pos,power in [('Key',(3,-7,12),1500),('Fill',(8,3,8),1000)]:
        bpy.ops.object.light_add(type='AREA',location=pos)
        obj=bpy.context.object;obj.name='PREVIEW | '+name;obj.data.energy=power;obj.data.size=8
        obj.rotation_euler=(Vector((6,-2,.5))-obj.location).to_track_quat('-Z','Y').to_euler()
    bpy.ops.object.camera_add(location=(16,-13,12))
    scene.camera=bpy.context.object;scene.camera.name='PREVIEW | Camera'
    scene.camera.data.type='ORTHO';scene.camera.data.ortho_scale=15
    def aim(pos):
        scene.camera.location=pos
        scene.camera.rotation_euler=(Vector((6,-2,.5))-scene.camera.location).to_track_quat('-Z','Y').to_euler()
    aim((16,-13,12))
    bpy.ops.wm.save_as_mainfile(filepath=str(out/'ship-fronts.blend'),compress=False)
    if not skip_preview:
        for name,pos in [('front',(16,-13,12)),('back',(-5,9,11))]:
            aim(pos);scene.render.filepath=str(out/('ship-fronts-'+name+'.png'))
            bpy.ops.render.render(write_still=True)

def main():
    args=sys.argv[sys.argv.index('--')+1:] if '--' in sys.argv else sys.argv[1:]
    out=Path(next((a for a in args if not a.startswith('--')),'target/ship-fronts'))
    out.mkdir(parents=True,exist_ok=True)
    models=assets()
    for asset in models:
        m.save_runtime(asset,out)
        print(asset.name,len(asset.parts),'closed editable components')
    if '--runtime-only' not in args: build_blender(models,out,'--skip-preview' in args)
if __name__=='__main__': main()
