"""Original full-volume outdoor sign frames, with a reserved live-lettering inset.

blender -b --threads 2 --python tools/build-outdoor-sign-frames.py -- target/outdoor-sign-frames [--skip-preview]
The source pack is never imported. Runtime crops its current text into the inset.
"""
import bpy, bmesh, json, math, sys
from pathlib import Path
from mathutils import Vector
args=sys.argv[sys.argv.index('--')+1:] if '--' in sys.argv else []
out=Path(next((a for a in args if not a.startswith('--')),'target/outdoor-sign-frames'));out.mkdir(parents=True,exist_ok=True)
bpy.ops.object.select_all(action='SELECT');bpy.ops.object.delete(use_global=False)
for m in list(bpy.data.materials):bpy.data.materials.remove(m)
scene=bpy.context.scene;scene.render.engine='CYCLES';scene.cycles.samples=24;scene.cycles.use_denoising=False
scene.render.resolution_x=1600;scene.render.resolution_y=650;scene.render.resolution_percentage=100;scene.render.image_settings.file_format='PNG';scene.view_settings.view_transform='AgX'
def mat(name,color,metal=0):
 m=bpy.data.materials.new(name);m.diffuse_color=(*color,1);m.use_nodes=True;bs=m.node_tree.nodes['Principled BSDF'];bs.inputs['Base Color'].default_value=(*color,1);bs.inputs['Roughness'].default_value=.7;bs.inputs['Metallic'].default_value=metal;return m
cream=mat('Inset | ivory paper under live inscription',(.81,.79,.64));bolt=mat('Joinery | aged brass',(.48,.39,.21),.4)
modern=[mat('Wayfinder | midnight blue enamel',(.16,.26,.33)),mat('Wayfinder | blue bevels',(.28,.42,.48)),mat('Wayfinder | dark inset',(.10,.17,.21))]
kanto=[mat('Civic | sage cast metal',(.31,.43,.40)),mat('Civic | pale cap',(.57,.64,.51)),mat('Civic | charcoal backing',(.16,.24,.23))]
park=[mat('Park | deep green enamel',(.19,.32,.22)),mat('Park | green edges',(.38,.48,.29)),mat('Park | inset shadow',(.11,.20,.14))]
forest=[mat('Forest | warm cedar end grain',(.42,.24,.12)),mat('Forest | honey bevels',(.62,.40,.20)),mat('Forest | recessed joinery',(.25,.15,.08))]
assets=[];stats={}
def begin(name):
 global coll,objects
 coll=bpy.data.collections.new('ASSET | '+name);scene.collection.children.link(coll);objects=[]
def keep(o,name,m):
 o.name=name
 for c in list(o.users_collection):c.objects.unlink(o)
 coll.objects.link(o);o.data.materials.append(m);objects.append(o);return o
def box(name,center,size,m,bevel=.10):
 bpy.ops.mesh.primitive_cube_add(size=1,location=(center[0],-center[2],center[1]));o=bpy.context.object;o.scale=(size[0],size[2],size[1]);bpy.ops.object.transform_apply(location=False,rotation=False,scale=True);keep(o,name,m)
 if bevel:
  b=o.modifiers.new('Hand dressed edge','BEVEL');b.width=bevel;b.segments=2
  bpy.context.view_layer.objects.active=o;bpy.ops.object.modifier_apply(modifier=b.name)
 return o
def pin(name,x,y,z,m,r=.17):
 bpy.ops.mesh.primitive_uv_sphere_add(segments=8,ring_count=4,radius=1,location=(x,-z,y));o=bpy.context.object;o.scale=(r,.075,r);bpy.ops.object.transform_apply(location=False,rotation=False,scale=True);return keep(o,name,m)
def beam(name,a,b,width,depth,m):
 a=Vector((a[0],-a[2],a[1]));b=Vector((b[0],-b[2],b[1]));delta=b-a
 bpy.ops.mesh.primitive_cube_add(size=1,location=(a+b)/2);o=bpy.context.object;o.scale=(width,depth,delta.length);o.rotation_euler=delta.to_track_quat('Z','Y').to_euler();bpy.ops.object.transform_apply(location=False,rotation=False,scale=True);keep(o,name,m)
 mod=o.modifiers.new('Brace eased edges','BEVEL');mod.width=.06;mod.segments=1;bpy.context.view_layer.objects.active=o;bpy.ops.object.modifier_apply(modifier=mod.name)
def export(name,face):
 groups={};low=[float('inf')]*3;high=[-float('inf')]*3;components=0;vol=0;triangles=0
 for o in objects:
  mesh=o.data;bm=bmesh.new();bm.from_mesh(mesh);assert all(e.is_manifold for e in bm.edges),o.name;bmesh.ops.recalc_face_normals(bm,faces=list(bm.faces));v=bm.calc_volume(signed=True);assert v>0,o.name;vol+=v;components+=1;bm.to_mesh(mesh);bm.free();mesh.calc_loop_triangles();triangles+=len(mesh.loop_triangles)
  m=mesh.materials[0];g=groups.setdefault(m.name,dict(name=m.name,base_color=[round(v,6)for v in m.diffuse_color],positions=[],normals=[],indices=[],lookup={}))
  for t in mesh.loop_triangles:
   n=o.matrix_world.to_3x3()@t.normal;n.normalize();n=(round(n.x,6),round(n.z,6),round(-n.y,6))
   for vi in t.vertices:
    p=o.matrix_world@mesh.vertices[vi].co;p=(round(p.x,6),round(p.z,6),round(-p.y,6))
    for a in range(3):low[a]=min(low[a],p[a]);high[a]=max(high[a],p[a])
    key=p+n
    if key not in g['lookup']:g['lookup'][key]=len(g['positions'])//3;g['positions'].extend(p);g['normals'].extend(n)
    g['indices'].append(g['lookup'][key])
 for g in groups.values():del g['lookup']
 # Face coordinates normalize against the actual export bounds, so bevels and
 # distinct feet can never displace the live text from its authored recess.
 f=[(face[0]-low[0])/(high[0]-low[0]),(face[1]-low[0])/(high[0]-low[0]),(face[2]-low[1])/(high[1]-low[1]),(face[3]-low[1])/(high[1]-low[1]),(face[4]-low[2])/(high[2]-low[2])]
 data=dict(name=name,coordinate_system='right-handed; +Y up; front +Z',bounds=dict(min=low,max=high),live_face=f,primitives=list(groups.values()))
 (out/(name+'.mesh.json')).write_text(json.dumps(data,separators=(',',':'))+'\n');stats[name]=dict(triangles=triangles,closed_components=components,materials=len(groups),volume=vol,bounds=data['bounds'],live_face=f)
 for o in objects:o['Source ownership']='Complete source 2x2 sign only; native lettering stays live';o['Authoring']='Original closed sculpture; no source textures imported'
 assets.append((name,list(objects)))
# Tall framed modern and park signs share their source silhouette, with distinct
# materials and construction: folded enamel vs cast park uprights and finials.
for name,palette in [('modern_frame',modern),('park_frame',park)]:
 begin(name);a,b,c=palette
 for x in [-6.4,6.4]:
  box('Upright | full-depth post',(x,7.1,0),(1.25,14.2,2),a,.16)
  box('Foot | raised socket',(x,.50,0),(1.85,1,3),c,.15)
  box('Collar | post joint',(x,11.8,0),(1.5,.7,2.2),b,.08)
  if name=='park_frame':pin('Finial | cast dome',x,14.2,0,b,.47)
 box('Crown | chamfered header',(0,14.45,0),(15.9,1.30,2.8),b,.30)
 box('Lower tie | independent rail',(0,2.05,0),(12.4,.85,1.7),a,.12)
 box('Inset | panel backing',(0,9.6,.24),(11.4,5.70,1.35),c,.13)
 box('Inset | ivory surface',(0,10.5,1.0),(10.2,3.1,.16),cream,.04)
 box('Sill | projecting lip',(0,7.00,.56),(12.15,.75,2.35),b,.13)
 for x in [-5.75,5.75]:
  for y in [7.4,12.15]:pin('Fixing | visible pin',x,y,1.16,bolt)
 if name=='modern_frame':
  for x in [-5.9,5.9]:beam('Rear | diagonal folded gusset',(x,5.0,-.75),(x,8.4,-1.12),.5,.45,c)
 else:
  for x in [-5.75,5.75]:beam('Park | cast corner brace',(x,4.05,0),(x*.54,6.72,0),.45,.60,a)
 export(name,[-5.05,5.05,8.98,12.02,1.095])
# Kanto's source is a broad, lower board with two distinctly separate short legs.
begin('kanto_board');a,b,c=kanto
for x in [-4.75,4.75]:
 box('Leg | twin civic post',(x,3.7,-.16),(1.65,7.4,2.25),a,.2)
 box('Foot | chamfered sole',(x,.35,-.16),(2.15,.7,3),c,.18)
 box('Leg | pale collar',(x,3.4,-.16),(1.95,.62,2.5),b,.12)
box('Board | complete back casing',(0,9.45,.10),(15.6,9.1,2),c,.26)
box('Board | enamel fascia',(0,9.50,1.09),(13.9,7.15,.18),cream,.15)
for x in [-7.24,7.24]:box('Rim | vertical folded edge',(x,9.5,1.11),(.57,7.9,.46),a,.12)
box('Rim | lower reveal',(0,5.45,1.1),(14.8,.70,.60),a,.17)
box('Crown | overhanging weather cap',(0,14.0,0),(16,1.55,2.85),b,.32)
box('Crown | runoff bead',(0,13.42,1.36),(15.1,.3,.28),a,.10)
for x in [-6.92,6.92]:
 for y in [6.3,12.5]:pin('Fixing | four civic bolts',x,y,1.42,bolt,.16)
export('kanto_board',[-6.68,6.68,6.12,12.95,1.195])
# Ilex's narrow reading board uses a heavy cedar frame and braced wooden feet.
begin('forest_timber');a,b,c=forest
for x in [-5.6,5.6]:
 box('Leg | square cedar post',(x,4.0,-.1),(2.05,8,2.65),a,.16)
 box('Foot | dark end grain',(x,.42,-.1),(2.1,.84,3.1),c,.10)
 beam('Rear | timber brace',(x,2,-.3),(x,6,-1.2),.6,.65,c)
box('Crown | deep sawn lintel',(0,14.6,0),(16,2.2,3),a,.22)
box('Crown | honey upper edge',(0,15.66,0),(15.1,.28,2.45),b,.08)
box('Panel | recessed cedar back',(0,10.6,0),(13.6,6.1,1.9),c,.16)
box('Panel | pale inset',(0,10.9,1.0),(10.4,3.1,.14),cream,.035)
box('Sill | heavy timber rail',(0,7.40,.10),(15.9,1.6,2.9),a,.20)
box('Sill | worn highlight',(0,8.15,.70),(14.3,.25,1.30),b,.08)
for x in [-6.1,6.1]:
 for y in [7.4,13.9]:pin('Joinery | oak peg',x,y,1.50,b,.22)
export('forest_timber',[-5.12,5.12,9.37,12.42,1.08])
(out/'asset-stats.json').write_text(json.dumps(stats,indent=2)+'\n')
# Preview display is separate from editable assets. Inset blanks are intentional:
# runtime source lettering must never be baked into an original art deliverable.
presentation=bpy.data.collections.new('PRESENTATION only');scene.collection.children.link(presentation)
for i,(name,objs) in enumerate(assets):
 for o in objs:o.hide_render=True
 for o in objs:
  copy=o.copy();copy.data=o.data;copy.location.x+=i*21-31.5;copy.hide_render=False;presentation.objects.link(copy)
bpy.ops.mesh.primitive_plane_add(size=200,location=(0,0,-.05));floor=bpy.context.object;floor.data.materials.append(mat('Presentation | sand',(.47,.46,.39)))
world=bpy.data.worlds.new('Sign workshop daylight');scene.world=world;world.use_nodes=True;world.node_tree.nodes['Background'].inputs['Color'].default_value=(.52,.58,.65,1);world.node_tree.nodes['Background'].inputs['Strength'].default_value=.7
for name,pos,power,size in [('Key',(-22,-35,48),22000,35),('Fill',(32,15,36),13000,30)]:
 bpy.ops.object.light_add(type='AREA',location=pos);o=bpy.context.object;o.name=name;o.data.energy=power;o.data.size=size;o.rotation_euler=(Vector((0,0,8))-o.location).to_track_quat('-Z','Y').to_euler()
bpy.ops.object.camera_add(location=(19,-100,48));cam=bpy.context.object;cam.data.type='ORTHO';cam.data.ortho_scale=84;cam.rotation_euler=(Vector((0,0,8))-cam.location).to_track_quat('-Z','Y').to_euler();scene.camera=cam
scene['Source']='Original editable sign frames. Text surfaces are supplied live by runtime; no imported source images.'
bpy.ops.wm.save_as_mainfile(filepath=str(out/'outdoor-sign-frames.blend'),compress=False)
if '--skip-preview' not in args:
 scene.render.filepath=str(out/'sign-frames-front.png');bpy.ops.render.render(write_still=True)
 cam.location=(-25,95,44);cam.rotation_euler=(Vector((0,0,8))-cam.location).to_track_quat('-Z','Y').to_euler();scene.render.filepath=str(out/'sign-frames-rear.png');bpy.ops.render.render(write_still=True)
print('SIGN_FRAMES='+json.dumps(stats))
