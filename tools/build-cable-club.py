#!/usr/bin/env python3
"""Original Cable Club divider kit, authored in dimensions of source pixels.

python3 tools/build-cable-club.py OUTPUT --runtime-only
blender -b --threads 2 --python tools/build-cable-club.py -- OUTPUT [--skip-preview]

No game images, ROMs, imported geometry or copied source artwork are used.
The runtime and editable Blender meshes come from the same named closed parts.
"""
from pathlib import Path
import base64,gzip,hashlib,json,math,sys
PALETTE={
 'powder_blue':(.36,.63,.74,1), 'blue_inset':(.22,.44,.57,1),
 'soft_ivory':(.82,.85,.76,1), 'slate_plinth':(.15,.27,.33,1),
 'edge_highlight':(.58,.76,.78,1), 'brushed_metal':(.53,.60,.59,1),
}
def sub(a,b):return tuple(x-y for x,y in zip(a,b))
def cross(a,b):return (a[1]*b[2]-a[2]*b[1],a[2]*b[0]-a[0]*b[2],a[0]*b[1]-a[1]*b[0])
def normal(a,b,c):
 n=cross(sub(b,a),sub(c,a));l=math.sqrt(sum(v*v for v in n));assert l>1e-10;return tuple(v/l for v in n)
class Asset:
 def __init__(self,name):self.name=name;self.parts=[]
 def prism(self,label,rect,y0,y1,material,corner=.5):
  x0,x1,z0,z1=rect;c=min(corner,(x1-x0)*.2,(z1-z0)*.2)
  # Clockwise in the XZ floor plane, outward faces point away from the body.
  ring=[(x0+c,z0),(x1-c,z0),(x1,z0+c),(x1,z1-c),(x1-c,z1),(x0+c,z1),(x0,z1-c),(x0,z0+c)]
  vs=[(x,y,z) for y in (y0,y1) for x,z in ring]
  faces=[tuple(range(8)),tuple(range(15,7,-1))]
  for i in range(8):j=(i+1)%8;faces.append((i,i+8,j+8,j))
  self.parts.append((label,material,vs,faces))
def make_asset(name,depth):
 a=Asset(name)
 a.prism('Continuous dark recessed toe plinth',(0.4,15.6,.4,depth-.4),0,2.0,'slate_plinth',.65)
 a.prism('Closed powder blue partition body',(.25,15.75,.25,depth-.25),1.5,14.2,'powder_blue',.65)
 a.prism('Continuous ivory crown with softened corners',(0,16,0,depth),13.7,15.7,'soft_ivory',.65)
 a.prism('Inset blue top rail',(1.05,14.95,1.05,depth-1.05),15.7,16,'edge_highlight',.5)
 # Recessed broad panels and slender mullions remain volumes on both sides.
 # Their depth is well below one source pixel, so silhouette stays quiet.
 count=3 if depth==64 else 2
 for side,x0,x1 in [('west',.10,.38),('east',15.62,15.90)]:
  for i in range(count):
   z0=2.1+i*(depth-4.2)/count;z1=2.1+(i+1)*(depth-4.2)/count-.7
   a.prism(f'{side} recessed blue panel {i+1}',(x0,x1,z0,z1),3.0,12.0,'blue_inset',.05)
   a.prism(f'{side} pale lower panel bead {i+1}',(x0-.035,x1+.035,z0,z1),2.7,3.1,'edge_highlight',.05)
  for z in [1.0,depth-1.4]:
   a.prism(f'{side} rounded end stile {z:g}',(x0-.05,x1+.05,z,z+.4),2.4,13.7,'brushed_metal',.06)
 # Both ends have a broad physical recessed panel and a raised picture rim.
 for end,z0,z1 in [('north',.10,.36),('south',depth-.36,depth-.10)]:
  a.prism(end+' end inset',(1.9,14.1,z0,z1),3.0,12.4,'blue_inset',.05)
  for x0,x1 in [(1.0,1.7),(14.3,15.0)]:a.prism(end+' ivory end stile',(x0,x1,z0-.035,z1+.035),2.5,13.2,'soft_ivory',.05)
 return a

def document(a):
 ps=[]
 for label,mat,vs,faces in a.parts:
  p=dict(name=label,base_color=PALETTE[mat],positions=[],normals=[],indices=[])
  for f in faces:
   for k in range(1,len(f)-1):
    points=[vs[f[0]],vs[f[k]],vs[f[k+1]]];n=normal(*points)
    for v in points:
     p['indices'].append(len(p['positions'])//3);p['positions'].extend(v);p['normals'].extend(n)
  ps.append(p)
 coords=[p['positions'][i:i+3] for p in ps for i in range(0,len(p['positions']),3)]
 return dict(name=a.name,coordinate_system='right-handed; +Y up; front +Z',origin='northwest-floor',bounds=dict(min=[min(v[i] for v in coords)for i in range(3)],max=[max(v[i] for v in coords)for i in range(3)]),triangle_count=sum(len(p['indices'])//3 for p in ps),primitives=ps)
def save_runtime(a,out):
 raw=(json.dumps(document(a),separators=(',',':'))+'\n').encode();compressed=bytearray(gzip.compress(raw,compresslevel=9,mtime=0));compressed[9]=255
 payload=dict(storage='geothite-model-gzip-v1',bytes=len(raw),sha256=hashlib.sha256(raw).hexdigest(),data=base64.b64encode(compressed).decode())
 (out/(a.name+'.mesh.json')).write_text(json.dumps(payload,separators=(',',':'))+'\n')
def build_blender(assets,out,skip_preview):
 import bpy
 from mathutils import Vector
 bpy.ops.object.select_all(action='SELECT');bpy.ops.object.delete(use_global=False)
 scene=bpy.context.scene;mats={}
 for name,color in PALETTE.items():
  m=bpy.data.materials.new(name);m.diffuse_color=color;m.use_nodes=True;b=m.node_tree.nodes['Principled BSDF'];b.inputs['Base Color'].default_value=color;b.inputs['Roughness'].default_value=.55;mats[name]=m
 cols={}
 for a in assets:
  col=bpy.data.collections.new('ASSET | '+a.name);scene.collection.children.link(col);cols[a.name]=col
  for label,mat,vs,faces in a.parts:
   me=bpy.data.meshes.new(label);me.from_pydata([(x/16,-z/16,y/16)for x,y,z in vs],[],faces);me.update();me.materials.append(mats[mat]);ob=bpy.data.objects.new(label,me);col.objects.link(ob)
  col.hide_render=True;col.hide_viewport=True;col['runtime_sha256']=hashlib.sha256((out/(a.name+'.mesh.json')).read_bytes()).hexdigest()
 gallery=bpy.data.collections.new('PREVIEW | Open Cable Club bays');scene.collection.children.link(gallery)
 # Same shape and size relationships as runtime. No floor closes the openings.
 def instance(name,x,z):
  for src in cols[name].objects:
   ob=src.copy();ob.data=src.data;gallery.objects.link(ob);ob.location=(x/16,-z/16,0)
 for x in [0,64,128,192]:instance('divider_long',x,0)
 for x in [144,176]:instance('vestibule_return',x,0)
 bpy.ops.mesh.primitive_plane_add(size=200,location=(6,-2,-.02));bpy.context.object.name='PREVIEW | Studio floor';m=bpy.data.materials.new('Preview neutral floor');m.diffuse_color=(.12,.16,.19,1);bpy.context.object.data.materials.append(m)
 scene.world.color=(.16,.18,.22)
 for name,loc,power,size in [('Key',(0,-6,10),1900,10),('Fill',(10,5,8),1300,8)]:
  bpy.ops.object.light_add(type='AREA',location=loc);o=bpy.context.object;o.name='PREVIEW | '+name;o.data.energy=power;o.data.size=size;o.rotation_euler=(Vector((6,-2,.4))-o.location).to_track_quat('-Z','Y').to_euler()
 bpy.ops.object.camera_add(location=(17,-16,12));camera=bpy.context.object;camera.name='PREVIEW | Camera';camera.data.type='ORTHO';camera.data.ortho_scale=15;scene.camera=camera
 camera.rotation_euler=(Vector((6.5,-2,.4))-camera.location).to_track_quat('-Z','Y').to_euler()
 scene.render.engine='CYCLES';scene.cycles.samples=24;scene.cycles.use_denoising=False;scene.render.resolution_x=1500;scene.render.resolution_y=800;scene.render.resolution_percentage=100;scene.render.image_settings.file_format='PNG';scene.view_settings.view_transform='AgX'
 scene['kit']='Original low-poly Cable Club partition and short vestibule returns';scene['provenance']='Pure authored geometry and colors; no copied game artwork, collision, events, ROM or textures';scene['rebuild']='tools/build-cable-club.py'
 bpy.ops.wm.save_as_mainfile(filepath=str(out/'cable-club.blend'),compress=False)
 if not skip_preview:
  for name,pos in [('front',(17,-16,12)),('rear',(-3,10,7))]:
   camera.location=pos;camera.rotation_euler=(Vector((6.5,-2,.4))-camera.location).to_track_quat('-Z','Y').to_euler();scene.render.filepath=str(out/('cable-club-'+name+'.png'));bpy.ops.render.render(write_still=True)
def main():
 args=sys.argv[sys.argv.index('--')+1:]if '--'in sys.argv else sys.argv[1:];out=Path(next((a for a in args if not a.startswith('--')),'target/cable-club'));out.mkdir(parents=True,exist_ok=True)
 assets=[make_asset('divider_long',64),make_asset('vestibule_return',48)]
 for a in assets:save_runtime(a,out);print(a.name,document(a)['triangle_count'],'triangles',document(a)['bounds'])
 if '--runtime-only'not in args:build_blender(assets,out,'--skip-preview'in args)
if __name__=='__main__':main()
