#!/usr/bin/env python3
"""Original paper-and-glass department store kit. No imported game artwork.

python3 tools/build-department-store.py OUTPUT --runtime-only
blender -b --threads 2 --python tools/build-department-store.py -- OUTPUT
Runtime triangles and editable Blender objects derive from the same closed parts.
"""
from pathlib import Path
import hashlib,json,math,sys
sys.path.insert(0,str(Path(__file__).resolve().parent))
from model_asset_storage import validate_model
PALETTE={
 'warm_ivory':(.81,.80,.67,1),'paper_edge':(.96,.92,.78,1),
 'ochre_trim':(.64,.49,.27,1),'deep_teal':(.12,.29,.30,1),
 'smoked_glass':(.29,.48,.45,1),'glass_light':(.43,.60,.53,1),
 'walnut_plinth':(.26,.25,.20,1),'inset_shadow':(.10,.18,.18,1),
 'copper':(.60,.38,.22,1),'drawer_face':(.67,.68,.56,1),
 'blue_glass':(.20,.34,.76,1),'blue_reflection':(.62,.79,.97,1),
 'blue_glint':(.86,.93,1,1),'glazing_shadow':(.10,.14,.24,1),
 'countertop_enamel':(.89,.82,.63,1),
}
def sub(a,b):return tuple(x-y for x,y in zip(a,b))
def cross(a,b):return(a[1]*b[2]-a[2]*b[1],a[2]*b[0]-a[0]*b[2],a[0]*b[1]-a[1]*b[0])
def normal(a,b,c):
 n=cross(sub(b,a),sub(c,a));l=math.sqrt(sum(v*v for v in n));assert l>1e-10;return tuple(v/l for v in n)
class Asset:
 def __init__(self,name):self.name=name;self.parts=[]
 def prism(self,label,rect,y0,y1,mat,corner=.2):
  x0,x1,z0,z1=rect;c=min(corner,(x1-x0)*.2,(z1-z0)*.2)
  assert x0<x1 and z0<z1 and y0<y1
  ring=[(x0+c,z0),(x1-c,z0),(x1,z0+c),(x1,z1-c),(x1-c,z1),(x0+c,z1),(x0,z1-c),(x0,z0+c)]
  vs=[(x,y,z)for y in(y0,y1)for x,z in ring];faces=[tuple(range(8)),tuple(range(15,7,-1))]
  for i in range(8):j=(i+1)%8;faces.append((i,i+8,j+8,j))
  self.parts.append((label,mat,vs,faces))
 def facet(self,label,polygon,z0,z1,mat):
  """Closed reflection relief on the recessed glass, below the frame front."""
  assert z0<z1 and len(polygon)>=3
  n=len(polygon);vs=[(x,y,z)for z in(z0,z1)for x,y in polygon]
  faces=[tuple(reversed(range(n))),tuple(range(n,2*n))]
  for i in range(n):j=(i+1)%n;faces.append((i,j,j+n,i+n))
  self.parts.append((label,mat,vs,faces))
 def horizontal_facet(self,label,polygon,y0,y1,mat):
  """Closed glazing relief on an opaque recessed horizontal display lid."""
  assert y0<y1 and len(polygon)>=3
  n=len(polygon);vs=[(x,y,z)for y in(y0,y1)for x,z in polygon]
  faces=[tuple(range(n)),tuple(reversed(range(n,2*n)))]
  for i in range(n):j=(i+1)%n;faces.append((i,i+n,j+n,j))
  self.parts.append((label,mat,vs,faces))
def wall(bays):
 a=Asset(f'window_course_{bays}');w=16*bays
 a.prism('Continuous closed rear cabinet',(.2,w-.2,.1,2.45),1.8,14.6,'glazing_shadow',.15)
 a.prism('One joined toe course',(0,w,0,3),0,1.3,'walnut_plinth',.15)
 a.prism('One joined bottom bead',(0,w,0,3),1.3,2.1,'ochre_trim',.15)
 a.prism('One joined cornice',(0,w,0,3),14.5,16,'warm_ivory',.15)
 a.prism('Continuous folded cornice edge',(0,w,2.6,3),14.1,14.5,'paper_edge',.06)
 for i in range(bays):
  x=i*16
  a.prism(f'Bay {i+1} recessed blue glass',(x+1.1,x+14.9,2.4,2.62),2.4,14.1,'blue_glass',.18)
  a.facet(f'Bay {i+1} diagonal blue reflection',
          [(x+1.8,2.8),(x+3.8,2.8),(x+14.2,13.7),(x+12.2,13.7)],
          2.63,2.67,'blue_reflection')
  a.facet(f'Bay {i+1} narrow sky glint',
          [(x+1.8,6.4),(x+2.6,6.4),(x+9.3,13.7),(x+8.5,13.7)],
          2.68,2.70,'blue_glint')
  for px in ([x,x+15.15] if i==bays-1 else [x]):
   a.prism(f'Bay {i+1} upright frame {px:g}',(px,px+.85,0,3),2.1,14.1,'warm_ivory',.12)
 return a
def directory():
 a=Asset('directory');a.prism('Directory closed ivory housing',(0,16,0,2.6),0,16,'warm_ivory',.4)
 a.prism('Directory dark inset backing',(1.1,14.9,2.6,2.9),1.2,14.8,'inset_shadow',.3)
 for x0,x1 in[(.5,1.3),(14.7,15.5)]:a.prism('Copper vertical directory rim',(x0,x1,2.85,3),.6,15.4,'copper',.05)
 for y0,y1 in[(.6,1.3),(14.7,15.4)]:a.prism('Copper horizontal directory rim',(1.3,14.7,2.85,3),y0,y1,'copper',.05)
 return a
def lift_side():
 a=Asset('lift_side');a.prism('Lift side closed fluted ivory panel',(0,16,0,2.5),0,16,'warm_ivory',.2)
 a.prism('Lift side call panel recess',(1,7.5,2.3,2.65),5.4,14.6,'deep_teal',.35)
 for x in[9,11,13,15]:a.prism('Lift side folded vertical rib '+str(x),(x,x+.6,2.5,3),1.2,14.8,'paper_edge',.07)
 a.prism('Lift side copper foot',(0,16,0,3),0,1.2,'ochre_trim',.15)
 return a
def u_display():
 a=Asset('closed_u_display')
 # Staff enclosure: three counter runs and a thin glass wall toward +Z.
 # Keep original map claims/collision; the native open staff inset is unchanged.
 for label,r in [('North sales rail',(0,64,0,16)),('West return',(0,16,16,64)),('East return',(48,64,16,64))]:
  x0,x1,z0,z1=r
  a.prism(label+' sealed body',(x0,x1,z0,z1),0,7.6,'warm_ivory',.08)
  a.prism(label+' flat countertop',(x0,x1,z0,z1),7.6,8,'countertop_enamel',.08)
  a.prism(label+' side inset',(x0+.1,x1-.1,z1-.2,z1),1,6.8,'drawer_face',.04)
  a.prism(label+' copper edge',(x0+.2,x1-.2,z0+.1,z0+.3),7.8,8,'copper',.02)
 # Glazing is a vertical wall, never an opaque display lid or lower body.
 for i in range(4):
  x=i*16
  a.prism(f'Display {i+1} glass pane',(x+.6,x+15.4,63.6,63.8),.8,7.6,'blue_glass',.02)
  a.facet(f'Display {i+1} diagonal sky reflection',
   [(x+1.2,1.3),(x+2.0,1.3),(x+14.8,7.1),(x+14.0,7.1)],63.81,63.83,'blue_reflection')
  a.facet(f'Display {i+1} narrow glazing glint',
   [(x+3,1.3),(x+3.3,1.3),(x+14.8,6.6),(x+14.8,6.9)],63.84,63.86,'blue_glint')
  a.prism(f'Display {i+1} flat upright',(x,x+.6,63.5,64),.8,7.6,'ochre_trim',.02)
 a.prism('Display east end upright',(63.4,64,63.5,64),.8,7.6,'ochre_trim',.02)
 a.prism('Display continuous top rail',(0,64,63.5,64),7.6,8,'deep_teal',.02)
 a.prism('Continuous closed front toe',(0,64,63.5,64),0,.8,'walnut_plinth',.02)
 return a

def assets():return [wall(n)for n in[1,2,4,8,10]]+[lift_side(),directory(),u_display()]
def document(a):
 ps=[]
 for label,mat,vs,faces in a.parts:
  p=dict(name=label,material=mat,base_color=PALETTE[mat],positions=[],normals=[],indices=[])
  for f in faces:
   for k in range(1,len(f)-1):
    points=[vs[f[0]],vs[f[k]],vs[f[k+1]]];n=normal(*points)
    for v in points:p['indices'].append(len(p['positions'])//3);p['positions'].extend(v);p['normals'].extend(n)
  ps.append(p)
 coords=[p['positions'][i:i+3]for p in ps for i in range(0,len(p['positions']),3)]
 return dict(name=a.name,coordinate_system='right-handed; +Y up; front +Z',origin='northwest-floor',bounds=dict(min=[min(v[i]for v in coords)for i in range(3)],max=[max(v[i]for v in coords)for i in range(3)]),triangle_count=sum(len(p['indices'])//3 for p in ps),primitives=ps)
def save_runtime(a,out):
 raw=(json.dumps(document(a),separators=(',',':'))+'\n').encode()
 path=out/(a.name+'.mesh.json');path.write_bytes(raw);validate_model(path)
def build_blender(aa,out,skip_preview):
 import bpy
 from mathutils import Vector
 bpy.ops.object.select_all(action='SELECT');bpy.ops.object.delete(use_global=False)
 scene=bpy.context.scene;mats={}
 for name,color in PALETTE.items():
  m=bpy.data.materials.new(name);m.diffuse_color=color;m.use_nodes=True;b=m.node_tree.nodes['Principled BSDF'];b.inputs['Base Color'].default_value=color;b.inputs['Roughness'].default_value=.7;mats[name]=m
 cols={}
 for a in aa:
  col=bpy.data.collections.new('ASSET | '+a.name);scene.collection.children.link(col);cols[a.name]=col
  for label,mat,vs,faces in a.parts:
   me=bpy.data.meshes.new(label);me.from_pydata([(x/16,-z/16,y/16)for x,y,z in vs],[],faces);me.update();me.materials.append(mats[mat]);ob=bpy.data.objects.new(label,me);col.objects.link(ob)
  col.hide_render=True;col.hide_viewport=True;col['runtime_sha256']=hashlib.sha256((out/(a.name+'.mesh.json')).read_bytes()).hexdigest()
 gallery=bpy.data.collections.new('PREVIEW | Department store kit');scene.collection.children.link(gallery)
 def instance(name,x,z):
  for src in cols[name].objects:
   ob=src.copy();ob.data=src.data;gallery.objects.link(ob);ob.location=(x/16,-z/16,0)
 instance('window_course_2',0,0);instance('lift_side',48,0);instance('window_course_8',64,0);instance('directory',208,0);instance('closed_u_display',80,56)
 bpy.ops.mesh.primitive_plane_add(size=200,location=(7,-3,-.01));bpy.context.object.name='PREVIEW | Neutral studio floor';m=bpy.data.materials.new('Neutral warm grey');m.diffuse_color=(.24,.26,.24,1);bpy.context.object.data.materials.append(m)
 scene.world.color=(.20,.21,.23)
 for name,loc,power,size in[('Key',(0,-5,12),2100,10),('Fill',(15,5,8),1500,10)]:
  bpy.ops.object.light_add(type='AREA',location=loc);o=bpy.context.object;o.name='PREVIEW | '+name;o.data.energy=power;o.data.size=size;o.rotation_euler=(Vector((7,-2,.4))-o.location).to_track_quat('-Z','Y').to_euler()
 bpy.ops.object.camera_add(location=(19,-20,15));camera=bpy.context.object;camera.name='PREVIEW | Camera';camera.data.type='ORTHO';camera.data.ortho_scale=18;scene.camera=camera
 scene.render.engine='CYCLES';scene.cycles.samples=16;scene.cycles.use_denoising=False;scene.render.resolution_x=1400;scene.render.resolution_y=800;scene.render.resolution_percentage=100;scene.render.image_settings.file_format='PNG';scene.view_settings.view_transform='AgX'
 scene['provenance']='Original closed geometry; no game textures, ROMs, events or collision data';scene['rebuild']='tools/build-department-store.py'
 camera.rotation_euler=(Vector((7,-2,.4))-camera.location).to_track_quat('-Z','Y').to_euler();bpy.ops.wm.save_as_mainfile(filepath=str(out/'department-store.blend'),compress=False)
 if not skip_preview:
  for name,pos in[('front',(19,-20,15)),('rear',(-5,13,16))]:
   camera.location=pos;camera.rotation_euler=(Vector((7,-2,.4))-camera.location).to_track_quat('-Z','Y').to_euler();scene.render.filepath=str(out/('department-store-'+name+'.png'));bpy.ops.render.render(write_still=True)
def main():
 args=sys.argv[sys.argv.index('--')+1:]if '--'in sys.argv else sys.argv[1:];out=Path(next((a for a in args if not a.startswith('--')),'target/department-store'));out.mkdir(parents=True,exist_ok=True);aa=assets()
 for a in aa:save_runtime(a,out);print(a.name,document(a)['triangle_count'],'triangles',document(a)['bounds'])
 if '--runtime-only'not in args:build_blender(aa,out,'--skip-preview'in args)
if __name__=='__main__':main()
