#!/usr/bin/env python3
"""Original folded-paper institutional desk kit. No source art is imported.
python tools/build-facility-radio.py OUTPUT --runtime-only
blender -b --threads 2 --python tools/build-facility-radio.py -- OUTPUT
Both editable objects and compact runtime triangles come from the same parts.
"""
from pathlib import Path
import base64,gzip,hashlib,json,math,sys
PALETTE={
 'cream':(.73,.77,.65,1),'paper':(.92,.92,.79,1),'rim':(.62,.67,.55,1),
 'teal':(.22,.41,.39,1),'deep_teal':(.10,.24,.25,1),'ink':(.075,.13,.15,1),
 'screen':(.28,.53,.46,1),'glint':(.57,.76,.63,1),'walnut':(.34,.27,.20,1),
 'tan':(.59,.48,.32,1),'red':(.70,.32,.23,1),'steel':(.42,.50,.46,1),
}
def sub(a,b):return tuple(x-y for x,y in zip(a,b))
def cross(a,b):return(a[1]*b[2]-a[2]*b[1],a[2]*b[0]-a[0]*b[2],a[0]*b[1]-a[1]*b[0])
def normal(a,b,c):
 n=cross(sub(b,a),sub(c,a));l=math.sqrt(sum(v*v for v in n));assert l>1e-10;return tuple(v/l for v in n)
class Asset:
 def __init__(self,name):self.name=name;self.parts=[]
 def prism(self,label,rect,y0,y1,mat,corner=.15):
  x0,x1,z0,z1=rect;c=min(corner,(x1-x0)*.2,(z1-z0)*.2)
  assert x0<x1 and z0<z1 and y0<y1
  ring=[(x0+c,z0),(x1-c,z0),(x1,z0+c),(x1,z1-c),(x1-c,z1),(x0+c,z1),(x0,z1-c),(x0,z0+c)]
  vs=[(x,y,z)for y in(y0,y1)for x,z in ring];faces=[tuple(range(8)),tuple(range(15,7,-1))]
  for i in range(8):j=(i+1)%8;faces.append((i,i+8,j+8,j))
  self.parts.append((label,mat,vs,faces))
 def cylinder(self,label,x,y0,y1,z,r,mat,n=10):
  ring=[(x+r*math.cos(i*math.tau/n),z+r*math.sin(i*math.tau/n))for i in range(n)]
  vs=[(x,y,z)for y in(y0,y1)for x,z in ring];fs=[tuple(range(n)),tuple(range(n*2-1,n-1,-1))]
  for i in range(n):j=(i+1)%n;fs.append((i,i+n,j+n,j))
  self.parts.append((label,mat,vs,fs))
def desk_body(a,x0,x1,z0,z1,label='Desk',top=10,open_knee=False):
 # Separate supports leave a true lower knee recess. Counter modules use a
 # closed cabinet shell; all are beveled solids with finished rear/side faces.
 if open_knee:
  for x in [x0+.65,x1-2.05]:
   a.prism(label+' folded leg',(x,x+1.4,z0+.9,z1-.9),0,top-1,'teal',.18)
   a.prism(label+' dark foot',(x-.25,x+1.65,z0+.5,z1-.5),0,.55,'ink',.16)
  a.prism(label+' recessed back modesty panel',(x0+1.5,x1-1.5,z0+1.4,z0+2.1),2.5,top-1,'deep_teal',.10)
  a.prism(label+' lower rear cross rail',(x0+1.5,x1-1.5,z0+1,z0+2.4),1.3,2.2,'steel',.12)
 else:
  a.prism(label+' continuous recessed toe',(x0+.6,x1-.6,z0+.6,z1-.6),0,1.3,'deep_teal',.18)
  a.prism(label+' closed folded cabinet',(x0+.28,x1-.28,z0+.28,z1-.28),1.1,top-.7,'cream',.28)
  a.prism(label+' inset south fascia',(x0+1.1,x1-1.1,z1-.43,z1-.23),2.0,top-2,'teal',.09)
  a.prism(label+' narrow south return bead',(x0+.65,x1-.65,z1-.36,z1-.11),top-2,top-1.4,'tan',.06)
  a.prism(label+' finished rear inset',(x0+1.1,x1-1.1,z0+.12,z0+.34),2,top-2,'rim',.06)
 a.prism(label+' overhung paper laminate',(x0,x1,z0,z1),top-.9,top,'paper',.30)
 a.prism(label+' folded front edge',(x0+.3,x1-.3,z1-.34,z1-.08),top-.85,top-.32,'rim',.06)

def phone(a,x,z,base=10):
 # Source telephone: pale left keypad, dark short handset, square low case.
 a.prism('Telephone dark foot',(x,x+6,z,z+6),base,base+.6,'ink',.3)
 a.prism('Telephone faceted pale casing',(x+.2,x+5.8,z+.15,z+5.8),base+.4,base+2.2,'cream',.45)
 a.prism('Telephone black keypad well',(x+.7,x+3.7,z+2.5,z+5.2),base+2.16,base+2.34,'deep_teal',.1)
 for row in range(3):
  for col in range(3):
   a.prism('Telephone physical key',(x+.9+col*.85,x+1.5+col*.85,z+2.7+row*.72,z+3.17+row*.72),base+2.3,base+2.55,'paper',.05)
 for xx in[x+.35,x+4.25]:a.prism('Telephone receiver rounded end',(xx,xx+1.4,z+.05,z+1.8),base+2.4,base+4.1,'ink',.28)
 a.prism('Telephone handset bridge',(x+1.15,x+4.95,z+.28,z+1.3),base+2.65,base+3.5,'deep_teal',.18)
 a.prism('Telephone receiver rim',(x+1.5,x+4.5,z+.3,z+.56),base+3.48,base+3.65,'steel',.04)

def memo(a,x,z,base=10):
 a.prism('Small original square memo support',(x,x+4.5,z,z+4.8),base,base+.15,'paper',.03)
 # Runtime lays only the live source memo face here; no copied pixels in asset.
 a.prism('Memo folded corner',(x+3.35,x+4.45,z+.12,z+1.2),base+.14,base+.25,'rim',.04)

def workbench():
 a=Asset('facility_workbench');desk_body(a,0,32,0,18,'Laboratory bench',top=9,open_knee=True)
 a.prism('Continuous bench apron',(1,31,16.2,17.0),5.3,8.1,'teal',.15)
 for x in[2,18]:
  a.prism('Inset bench drawer',(x,x+11,17,17.35),5.9,7.7,'cream',.12)
  a.prism('Slim drawer pull',(x+4,x+7,17.33,17.6),6.55,6.95,'deep_teal',.05)
 # Observed offset monitor to the left, tall auxiliary housing to the right.
 a.prism('Left display pedestal',(4,11,4.4,7.6),9,10.2,'steel',.2)
 a.prism('Left terminal rear casing',(1.1,14.6,1.0,7.2),10,18,'cream',.65)
 a.prism('Left terminal deep dark bezel',(1.6,14.0,7.13,8.05),10.6,17.5,'deep_teal',.34)
 a.prism('Left terminal live display recess',(2.4,13.2,8.02,8.18),11.35,16.85,'screen',.18)
 a.prism('Left terminal narrow pale lip',(2,13.7,8.03,8.3),10.8,11.2,'paper',.10)
 a.prism('Keyboard broad shallow case',(2,14.5,10.2,15.9),9,9.85,'steel',.32)
 for r in range(3):
  for c in range(7):a.prism('Keyboard raised cap',(2.7+c*1.55,3.8+c*1.55,10.8+r*1.23,11.7+r*1.23),9.8,10.15,'paper',.06)
 a.prism('Keyboard spacebar',(5.2,11.7,14.7,15.35),9.8,10.2,'cream',.05)
 a.prism('Right instrument raised foot',(20.1,30.4,3,13.5),9,9.75,'steel',.25)
 a.prism('Right instrument faceted upright',(20.8,30,3.4,12.7),9.6,17.3,'cream',.8)
 a.prism('Right instrument dark front inset',(21.5,27.9,12.55,13),10.4,16.2,'deep_teal',.22)
 a.prism('Right instrument upper face',(22.3,27,12.95,13.2),13.3,15.3,'screen',.16)
 a.prism('Right instrument lower bay',(22.2,27.1,12.96,13.15),11.0,12.7,'ink',.12)
 a.prism('Right instrument pale side facet',(29.3,30.15,5,11.3),10.9,16.0,'rim',.13)
 a.prism('Right instrument narrow status marker',(27.7,28.3,13.02,13.22),11.8,12.5,'paper',.05)
 return a

def phone_desk(name,with_memo=False):
 a=Asset(name);desk_body(a,0,32,0,16,'Office desk',top=10,open_knee=True)
 a.prism('Offset right stationery drawer',(23,30,2.4,14.2),4.4,8.9,'cream',.25)
 a.prism('Drawer recessed front',(23.5,29.5,14.1,14.55),5.0,8.1,'teal',.12)
 a.prism('Drawer pull',(25.0,28,14.5,14.8),6.3,6.7,'paper',.06)
 phone(a,25,2.0)
 if with_memo:memo(a,17.2,2.3)
 return a

def broadcast_terminal(a,x,z):
 a.prism('Broadcast terminal narrow floor foot',(x+.8,x+15.2,z+1,z+11),0,1,'ink',.25)
 a.prism('Broadcast terminal pedestal',(x+2,x+14,z+1.7,z+10.5),.8,9.8,'teal',.38)
 a.prism('Broadcast terminal drawer apron',(x+2.4,x+13.6,z+10.4,z+11),2.7,7.7,'cream',.25)
 a.prism('Broadcast terminal upper drawer',(x+3.2,x+12.8,z+10.95,z+11.25),5.5,7.1,'deep_teal',.1)
 a.prism('Broadcast terminal key shelf',(x+.5,x+15.5,z+3,z+12),9.8,11.0,'paper',.28)
 for row in range(2):
  for col in range(8):a.prism('Broadcast terminal raised key',(x+2+col*1.45,x+3.1+col*1.45,z+8.0+row*1.3,z+8.9+row*1.3),11,11.34,'steel',.05)
 a.prism('Broadcast terminal rear casing',(x+1,x+15,z+1,z+7.3),11,18,'cream',.50)
 a.prism('Broadcast terminal dark bezel',(x+1.8,x+14.2,z+7.25,z+7.8),12.2,17.1,'deep_teal',.24)
 a.prism('Broadcast terminal live display recess',(x+2.8,x+13.2,z+7.78,z+8.0),13.2,16.2,'screen',.13)

def reception(name,width,right_return=False):
 a=Asset(name)
 # These closed parts meet at shared edges. No pedestal, trim or top crosses
 # the source's open staff space under or behind the counter rail.
 desk_body(a,0,width,0,16,'Continuous service rail',top=12)
 desk_body(a,0,16,16,48,'West service return',top=12)
 if right_return:desk_body(a,width-16,width,16,48,'East service return',top=12)
 # Narrow joined folded edge strips conceal the physical corner seams.
 a.prism('West corner continuous folded stile',(.1,.65,.7,47.3),1.0,11.1,'tan',.07)
 if right_return:a.prism('East corner continuous folded stile',(width-.65,width-.1,.7,47.3),1.0,11.1,'tan',.07)
 phone(a,width-(24 if right_return else 13),2.3,12)
 if right_return:broadcast_terminal(a,80,12)
 else:memo(a,32.8,2.3,12)
 return a

def assets():return[workbench(),phone_desk('radio_phone_desk'),phone_desk('radio_memo_desk',True),reception('radio_reception_u',160,True),reception('radio_reception_l',64),phone_desk('radio_counter_extension')]

def document(a):
 ps=[]
 for label,mat,vs,faces in a.parts:
  p=dict(name=label,base_color=PALETTE[mat],positions=[],normals=[],indices=[])
  for f in faces:
   for k in range(1,len(f)-1):
    points=[vs[f[0]],vs[f[k]],vs[f[k+1]]];n=normal(*points)
    for v in points:p['indices'].append(len(p['positions'])//3);p['positions'].extend(v);p['normals'].extend(n)
  ps.append(p)
 coords=[p['positions'][i:i+3]for p in ps for i in range(0,len(p['positions']),3)]
 return dict(name=a.name,coordinate_system='right-handed; +Y up; front +Z',origin='northwest-floor',bounds=dict(min=[min(v[i]for v in coords)for i in range(3)],max=[max(v[i]for v in coords)for i in range(3)]),triangle_count=sum(len(p['indices'])//3 for p in ps),primitives=ps)
def save_runtime(a,out):
 raw=(json.dumps(document(a),separators=(',',':'))+'\n').encode();compressed=bytearray(gzip.compress(raw,compresslevel=9,mtime=0));compressed[9]=255
 payload=dict(storage='geothite-model-gzip-v1',bytes=len(raw),sha256=hashlib.sha256(raw).hexdigest(),data=base64.b64encode(compressed).decode());(out/(a.name+'.mesh.json')).write_text(json.dumps(payload,separators=(',',':'))+'\n')
def build_blender(aa,out,skip_preview):
 import bpy
 from mathutils import Vector
 # Two editable chunks keep source review quick without duplicating any game art.
 for chunk,names in [('facility-workbench',['facility_workbench']),('radio-desks',[a.name for a in aa if a.name!='facility_workbench'])]:
  bpy.ops.object.select_all(action='SELECT');bpy.ops.object.delete(use_global=False)
  for col in list(bpy.data.collections):bpy.data.collections.remove(col)
  scene=bpy.context.scene;mats={}
  for name,color in PALETTE.items():
   m=bpy.data.materials.new(name);m.diffuse_color=color;m.use_nodes=True;b=m.node_tree.nodes['Principled BSDF'];b.inputs['Base Color'].default_value=color;b.inputs['Roughness'].default_value=.78;mats[name]=m
  aa_chunk=[a for a in aa if a.name in names];gallery=bpy.data.collections.new('PREVIEW | '+chunk);scene.collection.children.link(gallery)
  for index,a in enumerate(aa_chunk):
   col=bpy.data.collections.new('ASSET | '+a.name);scene.collection.children.link(col)
   for label,mat,vs,faces in a.parts:
    me=bpy.data.meshes.new(label);me.from_pydata([(x/16,-z/16,y/16)for x,y,z in vs],[],faces);me.update();me.materials.append(mats[mat]);ob=bpy.data.objects.new(label,me);col.objects.link(ob)
    display=ob.copy();display.data=me;gallery.objects.link(display);locations={'facility_workbench':(0,0,0),'radio_phone_desk':(0,0,0),'radio_memo_desk':(4,0,0),'radio_counter_extension':(8,0,0),'radio_reception_u':(0,-4,0),'radio_reception_l':(12,-4,0)};display.location=locations[a.name]
   col.hide_render=True;col.hide_viewport=True;col['runtime_sha256']=hashlib.sha256((out/(a.name+'.mesh.json')).read_bytes()).hexdigest()
  center=(1,-.6,.5)if len(aa_chunk)==1 else(8,-3.5,.5);distance=3.5 if len(aa_chunk)==1 else 18
  bpy.ops.mesh.primitive_plane_add(size=100,location=(center[0],center[1],-.012));bpy.context.object.name='PREVIEW | Neutral studio floor';m=bpy.data.materials.new('Neutral warm grey');m.diffuse_color=(.23,.27,.25,1);bpy.context.object.data.materials.append(m)
  scene.world.color=(.22,.24,.23)
  for name,offset,power,size in[('Key',(-5,-8,12),1600,9),('Fill',(10,8,7),1100,9)]:
   bpy.ops.object.light_add(type='AREA',location=tuple(c+o for c,o in zip(center,offset)));o=bpy.context.object;o.name='PREVIEW | '+name;o.data.energy=power;o.data.size=size;o.rotation_euler=(Vector(center)-o.location).to_track_quat('-Z','Y').to_euler()
  bpy.ops.object.camera_add(location=(center[0]+distance,center[1]-distance,center[2]+distance*.8));camera=bpy.context.object;camera.name='PREVIEW | Camera';camera.data.type='ORTHO';camera.data.ortho_scale=distance*1.15;scene.camera=camera;camera.rotation_euler=(Vector(center)-camera.location).to_track_quat('-Z','Y').to_euler()
  scene.render.engine='CYCLES';scene.cycles.samples=8;scene.cycles.use_denoising=False;scene.render.resolution_x=1200;scene.render.resolution_y=800;scene.render.resolution_percentage=100;scene.render.image_settings.file_format='PNG';scene.view_settings.view_transform='AgX';scene['provenance']='Original geometric sculpture; no imported game art';scene['rebuild']='tools/build-facility-radio.py'
  bpy.ops.wm.save_as_mainfile(filepath=str(out/(chunk+'.blend')),compress=False)
  if not skip_preview:
   for view,offset in [('front',(distance,-distance,distance*.8)),('rear',(-distance,distance,distance*.8))]:
    camera.location=tuple(c+o for c,o in zip(center,offset));camera.rotation_euler=(Vector(center)-camera.location).to_track_quat('-Z','Y').to_euler();scene.render.filepath=str(out/(chunk+'-'+view+'.png'));bpy.ops.render.render(write_still=True)
def main():
 args=sys.argv[sys.argv.index('--')+1:]if '--'in sys.argv else sys.argv[1:];out=Path(next((a for a in args if not a.startswith('--')),'target/facility-radio'));out.mkdir(parents=True,exist_ok=True);aa=assets()
 for a in aa:save_runtime(a,out);print(a.name,document(a)['triangle_count'],'triangles',document(a)['bounds'])
 if '--runtime-only'not in args:build_blender(aa,out,'--skip-preview'in args)
if __name__=='__main__':main()
