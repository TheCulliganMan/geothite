"""Original editable cloth backing for the four live bedroom carpet states.

blender -b --python tools/build-bedroom-cloth.py -- target/bedroom-cloth
No source artwork is embedded. Runtime overlays each exact live atlas cell.
"""
import bpy, json, math, sys
from pathlib import Path
from mathutils import Vector
args=sys.argv[sys.argv.index('--')+1:] if '--' in sys.argv else []
out=Path(args[0] if args else 'target/bedroom-cloth');out.mkdir(parents=True,exist_ok=True)
bpy.ops.object.select_all(action='SELECT');bpy.ops.object.delete(use_global=False)
scene=bpy.context.scene
col=bpy.data.collections.new('carpet_cloth');scene.collection.children.link(col)
def material(name,color):
 m=bpy.data.materials.new(name);m.diffuse_color=(*color,1);m.use_nodes=True
 bs=m.node_tree.nodes['Principled BSDF'];bs.inputs['Base Color'].default_value=(*color,1);bs.inputs['Roughness'].default_value=.98
 return m
back=material('Natural linen reverse',(.44,.40,.32));binding=material('Neutral woven edge',(.63,.60,.50))
def box(name,loc,size,mat,bevel):
 bpy.ops.mesh.primitive_cube_add(size=1,location=loc);o=bpy.context.object;o.name=name;o.dimensions=size
 bpy.ops.object.transform_apply(location=False,rotation=False,scale=True)
 for c in list(o.users_collection):c.objects.unlink(o)
 col.objects.link(o);o.data.materials.append(mat)
 if bevel:
  m=o.modifiers.new('Soft fiber edge','BEVEL');m.width=bevel;m.segments=1;bpy.ops.object.modifier_apply(modifier=m.name)
 return o
# The cap is deliberately neutral: none of this material replaces source art.
box('Closed flexible textile backing',(0,0,.008),(2,2,.016),back,.006)
box('Dense woven cloth face',(0,0,.018),(1.997,1.997,.012),binding,.005)
# A low rolled binding gives a readable profile without invented top markings.
for axis in range(2):
 for side in [-1,1]:
  loc=[0,0,.015];loc[axis]=side*.994
  size=[1.99,1.99,.014];size[axis]=.012
  box('Rolled side binding',loc,size,binding,.005)
  for i in range(8):
   loc=[-.94+i*.2685,side*.999,.012]
   if axis==0:loc[0],loc[1]=loc[1],loc[0]
   size=[.028,.007,.010] if axis==1 else [.007,.028,.010]
   box('Editable side overcast stitch',loc,size,back,.002)
bpy.context.view_layer.update();groups={}
for obj in col.objects:
 me=obj.data;me.calc_loop_triangles();normal_matrix=obj.matrix_world.to_3x3().inverted().transposed()
 for tri in me.loop_triangles:
  mat=me.materials[tri.material_index]
  g=groups.setdefault(mat.name,{'positions':[],'normals':[],'indices':[],'base_color':list(mat.diffuse_color),'material':mat.name,'lookup':{}})
  n=(normal_matrix@tri.normal).normalized();normal=(round(n.x,6),round(n.z,6),round(-n.y,6))
  for vid in tri.vertices:
   v=obj.matrix_world@me.vertices[vid].co;position=(round(v.x,6),round(v.z,6),round(-v.y,6));key=position+normal
   if key not in g['lookup']:
    g['lookup'][key]=len(g['positions'])//3;g['positions'].extend(position);g['normals'].extend(normal)
   g['indices'].append(g['lookup'][key])
primitives=[]
for _,g in sorted(groups.items()):del g['lookup'];primitives.append(g)
pts=[g['positions'][i:i+3] for g in primitives for i in range(0,len(g['positions']),3)]
lo=[min(p[a] for p in pts) for a in range(3)];hi=[max(p[a] for p in pts) for a in range(3)]
data={'name':'carpet_cloth','coordinate_system':'right-handed; +Y up; front +Z','origin':'floor-center','bounds':{'min':lo,'max':hi},'triangle_count':sum(len(g['indices'])//3 for g in primitives),'primitives':primitives}
(out/'carpet_cloth.mesh.json').write_text(json.dumps(data,separators=(',',':')))
scene['Art provenance']='Original closed woven cloth backing, editable seams and overcast stitches. No imported source pixels. Live pattern is mounted by the renderer.'
# This light setup demonstrates actual low textile thickness in an oblique view.
world=bpy.data.worlds.new('Textile studio');scene.world=world;world.use_nodes=True;world.node_tree.nodes['Background'].inputs['Strength'].default_value=.55
for name,loc,power,size in [('Softbox',(-3,-4,6),550,5),('Rim',(3,3,3),300,4)]:
 bpy.ops.object.light_add(type='AREA',location=loc);o=bpy.context.object;o.name=name;o.data.energy=power;o.data.size=size;o.rotation_euler=(Vector((0,0,0))-o.location).to_track_quat('-Z','Y').to_euler()
bpy.ops.object.camera_add(location=(2.8,-3.5,2.3));o=bpy.context.object;o.rotation_euler=(Vector((0,0,0))-o.location).to_track_quat('-Z','Y').to_euler();o.data.type='ORTHO';o.data.ortho_scale=3.3;scene.camera=o
scene.render.engine='CYCLES';scene.cycles.samples=16;scene.cycles.use_denoising=False
scene.render.resolution_x=1000;scene.render.resolution_y=760;scene.render.resolution_percentage=100
scene.render.image_settings.file_format='PNG';scene.render.filepath=str(out/'bedroom-cloth.png')
bpy.ops.wm.save_as_mainfile(filepath=str(out/'bedroom-cloth.blend'),compress=True)
if '--skip-preview' not in args:bpy.ops.render.render(write_still=True)
print(json.dumps({'vertices':len(pts),'triangles':data['triangle_count'],'bounds':data['bounds']}))
