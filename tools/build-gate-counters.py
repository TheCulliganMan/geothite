"""Original joined timber counter and telephone kit; no pixels or game data.
blender -b --threads 2 --python tools/build-gate-counters.py -- OUTPUT [--skip-preview]
NESW edge masks sculpt only exposed joinery; hidden module edges meet flush.
The named mesh pieces and studio assemblies remain editable in the saved blend.
"""
import bpy,bmesh,math,json,sys
from pathlib import Path
sys.path.insert(0,str(Path(__file__).resolve().parent))
from model_asset_storage import validate_model
from mathutils import Vector
args=sys.argv[sys.argv.index('--')+1:] if '--' in sys.argv else []
out=Path(next((a for a in args if not a.startswith('--')),'target/gate-counters'));out.mkdir(parents=True,exist_ok=True)
bpy.ops.object.select_all(action='SELECT');bpy.ops.object.delete(use_global=False)
scene=bpy.context.scene
colors={'timber':(.28,.18,.14),'timber_light':(.48,.31,.21),'timber_gold':(.64,.43,.26),
 'panel':(.37,.24,.17),'wood_edge':(.73,.52,.32),'enamel':(.65,.76,.70),'enamel_light':(.87,.89,.77),
 'teal':(.16,.40,.39),'teal_light':(.27,.53,.48),'brass':(.75,.58,.29),'ink':(.09,.14,.15),
 'rubber':(.12,.17,.18),'screen':(.13,.29,.28),'screen_light':(.45,.73,.54),'key':(.91,.89,.72)}
M={}
for name,c in colors.items():
 m=bpy.data.materials.new(name);m.diffuse_color=(*c,1);m.use_nodes=True
 bs=m.node_tree.nodes['Principled BSDF'];bs.inputs['Base Color'].default_value=(*c,1);bs.inputs['Roughness'].default_value=.55 if name.startswith('timber') else .68
 M[name]=m
assets={};COL=None

def begin(n):
 global COL
 COL=bpy.data.collections.new(n);scene.collection.children.link(COL);assets[n]=COL

def assign(o,n,mat):
 o.name=n
 for c in list(o.users_collection):c.objects.unlink(o)
 COL.objects.link(o);o.data.materials.append(M[mat]);return o

def box(n,p,s,mat,bevel=0):
 bpy.ops.mesh.primitive_cube_add(size=1,location=p);o=assign(bpy.context.object,n,mat);o.dimensions=s;bpy.ops.object.transform_apply(location=False,rotation=False,scale=True)
 if bevel:
  b=o.modifiers.new('Soft cabinet arris','BEVEL');b.width=min(bevel,min(s)*.28);b.segments=1;bpy.ops.object.modifier_apply(modifier=b.name)
 return o

def poly(n,verts,faces,mat):
 me=bpy.data.meshes.new(n);me.from_pydata(verts,[],faces);me.update();bm=bmesh.new();bm.from_mesh(me);bmesh.ops.recalc_face_normals(bm,faces=list(bm.faces));bm.to_mesh(me);bm.free()
 o=bpy.data.objects.new(n,me);COL.objects.link(o);me.materials.append(M[mat]);return o

def rod(n,a,b,r,mat,verts=8):
 a,b=Vector(a),Vector(b);bpy.ops.mesh.primitive_cylinder_add(vertices=verts,radius=r,depth=(b-a).length,location=(a+b)/2);o=assign(bpy.context.object,n,mat);o.rotation_euler=(b-a).to_track_quat('Z','Y').to_euler();return o

def ball(n,p,s,mat):
 bpy.ops.mesh.primitive_ico_sphere_add(subdivisions=2,radius=1,location=p);o=assign(bpy.context.object,n,mat);o.scale=s;bpy.ops.object.transform_apply(location=False,rotation=False,scale=True);return o

def facebox(n,x,z,w,h,d,y,mat,angle=0,bevel=0):
 o=box(n,(x,y,z),(w,d,h),mat,bevel);o.location.rotate(__import__('mathutils').Euler((0,0,angle)));o.rotation_euler.z=angle;return o

def cap(mask):
 # Fixed outer dimensions allow exact flush assembly; only outside edges bevel.
 outer=[(-.5,-.5),(.5,-.5),(.5,.5),(-.5,.5)]
 inset=.035
 def inn(x,y):return (x+(inset if x<0 and mask&8 else -inset if x>0 and mask&2 else 0),y+(inset if y<0 and mask&4 else -inset if y>0 and mask&1 else 0))
 inside=[inn(x,y) for x,y in outer]
 verts=[(x,y,.88) for x,y in outer]+[(x,y,.958) for x,y in outer]+[(x,y,1.) for x,y in inside]
 faces=[(3,2,1,0),(8,9,10,11)]
 for i in range(4):j=(i+1)%4;faces.extend([(i,j,j+4,i+4),(i+4,j+4,j+8,i+8)])
 poly('Continuous solid timber top with outer eased arris',verts,faces,'timber_gold')
 # Subtle inlaid top grain ends at the join; adjacent modules line up exactly.
 for x in [-.24,.22]:
  y0=-.5+(.04 if mask&4 else 0);y1=.5-(.04 if mask&1 else 0)
  poly('Fine continuous timber grain',[(x-.004,y0,1.0004),(x+.004,y0,1.0004),(x+.004,y1,1.0004),(x-.004,y1,1.0004)],[(0,1,2,3)],'timber_light')

for mask in range(16):
 begin(f'counter_{mask:02x}')
 lo_x=-.5+(.045 if mask&8 else 0);hi_x=.5-(.045 if mask&2 else 0)
 lo_y=-.5+(.045 if mask&4 else 0);hi_y=.5-(.045 if mask&1 else 0)
 box('Closed cabinet carcass',((lo_x+hi_x)/2,(lo_y+hi_y)/2,.46),(hi_x-lo_x,hi_y-lo_y,.80),'timber',0)
 box('Recessed continuous toe plinth',((lo_x+hi_x)/2,(lo_y+hi_y)/2,.055),(hi_x-lo_x,hi_y-lo_y,.11),'timber',0)
 cap(mask)
 for bit,a in enumerate([math.pi,math.pi/2,0,-math.pi/2]):
  if not mask&(1<<bit):continue
  # One continuous recessed frame has thickness and bevel shoulders. It
  # replaces tiny bevelled cubes with the visible cabinet silhouette only.
  outer=[(-.48,.13),(.48,.13),(.48,.80),(-.48,.80)]
  inner=[(-.40,.20),(.40,.20),(.40,.73),(-.40,.73)]
  verts=[(x,y,z) for points,y in [(outer,-.455),(outer,-.490),(inner,-.490),(inner,-.465)] for x,z in points]
  faces=[]
  for i in range(4):
   j=(i+1)%4;faces.extend([(i,j,j+4,i+4),(i+4,j+4,j+8,i+8),(i+8,j+8,j+12,i+12)])
  o=poly('Thick recessed frame with chamfered outer shoulder',verts,faces,'timber_light');o.rotation_euler.z=a
  o=poly('Shadowed recessed solid panel',[(-.40,-.465,.20),(.40,-.465,.20),(.40,-.465,.73),(-.40,-.465,.73)],[(0,1,2,3)],'panel');o.rotation_euler.z=a
  # Each flute has a triangular carved cross section: 8 visible triangles.
  for x in [-.24,0,.24]:
   vs=[(x-.012,-.466,.25),(x+.012,-.466,.25),(x,-.485,.25),(x-.012,-.466,.68),(x+.012,-.466,.68),(x,-.485,.68)]
   o=poly('Hand-carved triangular timber flute',vs,[(0,1,2),(3,5,4),(0,3,4,1),(1,4,5,2),(2,5,3,0)],'timber_gold');o.rotation_euler.z=a

begin('phone_console')
box('Solid teal desk console foot',(0,0,.055),(.84,.64,.11),'teal',.035)
box('Raised cream mounting rim',(0,.015,.14),(.76,.57,.09),'enamel_light',.025)
box('Full-depth cast telephone housing',(0,.08,.52),(.71,.41,.77),'enamel',.04)
box('Front deep control recess',(-.06,-.139,.51),(.50,.06,.60),'teal',.022)
box('Small inset status display',(-.12,-.181,.73),(.29,.028,.14),'ink',.008)
box('Green receiver indicator',(-.12,-.198,.736),(.235,.013,.091),'screen',.006)
for i in range(3):box('Subtle display indicator',(-.20+i*.07,-.208,.74),(.036,.008,.012),'screen_light',.002)
box('Recessed keypad well',(-.13,-.181,.415),(.28,.025,.33),'rubber',.007)
for row in range(4):
 for col in range(3):
  box('Individual tactile telephone key',(-.222+col*.093,-.207,.535-row*.078),(.066,.033,.052),'key',0)
box('Coin return recess',(-.13,-.18,.207),(.29,.025,.062),'ink',.007)
box('Brass return lip',(-.13,-.211,.19),(.29,.039,.023),'brass',.005)
# A recognizable side-mounted handset has two bulbous cups and a bowed grip.
for z in [.30,.76]:
 box('Receiver cradle fork',(.26,-.18,z),(.16,.12,.09),'teal',.019)
 ball('Rounded handset acoustic cup',(.29,-.272,z),(.115,.090,.095),'rubber')
 for dx in [-.035,0,.035]:
  for dz in [-.025,.025]:
   xx=.29+dx;zz=z+dz;poly('Inset acoustic perforation',[(xx-.007,-.358,zz-.007),(xx+.007,-.358,zz-.007),(xx+.007,-.358,zz+.007),(xx-.007,-.358,zz+.007)],[(0,1,2,3)],'ink')
rod('Handset lower elbow',(.29,-.273,.35),(.32,-.312,.43),.047,'rubber',10)
rod('Raised handset grip',(.32,-.312,.43),(.32,-.312,.66),.045,'rubber',10)
rod('Handset upper elbow',(.32,-.312,.66),(.29,-.273,.71),.047,'rubber',10)
# The coiled cord remains true solid geometry in the editable source and export.
pts=[]
for i in range(25):
 t=i/24;a=t*math.tau*5;pts.append(Vector((.38+.025*math.cos(a),-.145+.025*math.sin(a),.17+t*.27)))
verts=[]
for i,p in enumerate(pts):
 tangent=(pts[min(i+1,24)]-pts[max(i-1,0)]).normalized();n=tangent.cross(Vector((0,0,1))).normalized();b=tangent.cross(n).normalized()
 for j in range(3):
  a=j*math.tau/3;verts.append(p+.009*(n*math.cos(a)+b*math.sin(a)))
faces=[(2,1,0),(72,73,74)]
for i in range(24):
 for j in range(3):faces.append((i*3+j,i*3+(j+1)%3,(i+1)*3+(j+1)%3,(i+1)*3+j))
poly('Continuous sculpted low-poly coiled handset cord',verts,faces,'rubber')
rod('Cord to handset',(.38,-.145,.44),(.29,-.272,.32),.013,'rubber',5)
for x in [-.29,.29]:
 for z in [.23,.86]:rod('Brass casing screw',(x,-.135,z),(x,-.147,z),.012,'brass',6)

# Export independent assets before the editable studio layout moves any pieces.
def export(name,col):
 groups={};deps=bpy.context.evaluated_depsgraph_get()
 for obj in col.objects:
  if obj.type!='MESH':continue
  ev=obj.evaluated_get(deps);me=ev.to_mesh();me.calc_loop_triangles();norm=obj.matrix_world.to_3x3().inverted().transposed()
  for tri in me.loop_triangles:
   mat=obj.data.materials[tri.material_index];g=groups.setdefault(mat.name,dict(name=mat.name,base_color=[round(v,5) for v in mat.diffuse_color],positions=[],normals=[],indices=[]));n=(norm@tri.normal).normalized()
   if n.length_squared<.9:continue
   for idx in tri.vertices:
    v=obj.matrix_world@me.vertices[idx].co;g['indices'].append(len(g['positions'])//3);g['positions'].extend(round(c,6) for c in [v.x,v.z,-v.y]);g['normals'].extend(round(c,6) for c in [n.x,n.z,-n.y])
  ev.to_mesh_clear()
 ps=list(groups.values());coords=[p['positions'][i:i+3] for p in ps for i in range(0,len(p['positions']),3)];lo=[min(v[i] for v in coords) for i in range(3)];hi=[max(v[i] for v in coords) for i in range(3)]
 d=dict(name=name,coordinate_system='right-handed; +Y up; front +Z',origin='floor-center',bounds=dict(min=lo,max=hi),triangle_count=sum(len(p['indices'])//3 for p in ps),primitives=ps)
 raw=(json.dumps(d,separators=(',',':'))+'\n').encode()
 path=out/(name+'.mesh.json');path.write_bytes(raw);validate_model(path);return dict(name=name,triangles=d['triangle_count'],bounds=d['bounds'],objects=len(col.objects),materials=len(ps))
stats=[export(n,c) for n,c in assets.items()];(out/'asset-stats.json').write_text(json.dumps(stats,indent=2)+'\n')
# A visible U desk, independent L desk, straight return and isolated cap show
# true geometry joins and the intentional service opening. These are artist
# studio assemblies, never game data or replacement gameplay layouts.
demo=bpy.data.collections.new('Joined counter examples');scene.collection.children.link(demo)
def clone(col,offset,scale=1):
 for src in col.objects:
  o=src.copy();o.data=src.data.copy();demo.objects.link(o);o.location=src.location*scale+Vector(offset);o.scale=src.scale*scale

def assembly(points,offset,phone_at=None):
 points=set(points);scale=.52
 for x,y in sorted(points):
  mask=sum(1<<i for i,(dx,dy) in enumerate([(0,1),(1,0),(0,-1),(-1,0)]) if (x+dx,y+dy) not in points)
  clone(assets[f'counter_{mask:02x}'],(offset[0]+x*scale,offset[1]+y*scale,0),scale)
 if phone_at:clone(assets['phone_console'],(offset[0]+phone_at[0]*scale,offset[1]+phone_at[1]*scale,scale),.85)
assembly([(x,y) for y in range(6) for x in range(8) if y<2 or x<2 or x>=6],(-2.2,-1.4),(.5,3.5))
assembly([(x,y) for y in range(5) for x in range(5) if y<2 or x<2],(3.1,-1.4),(0.5,3.2))
# Source modules stay editable off to the rear in a labeled collection row.
for i,(n,c) in enumerate(assets.items()):
 for o in c.objects:o.location+=Vector(((i%8)*1.6-5.6,5+(i//8)*1.8,0));o.hide_render=True
world=bpy.data.worlds.new('Civic workshop');scene.world=world;world.use_nodes=True;world.node_tree.nodes['Background'].inputs['Color'].default_value=(.17,.22,.24,1);world.node_tree.nodes['Background'].inputs['Strength'].default_value=.65
for pos,energy,size in [((-5,-6,12),1600,8),((8,6,10),1200,7)]:
 bpy.ops.object.light_add(type='AREA',location=pos);o=bpy.context.object;o.data.energy=energy;o.data.size=size;o.rotation_euler=(Vector((1,0,.5))-o.location).to_track_quat('-Z','Y').to_euler()
bpy.ops.object.camera_add(location=(12,-16,16));cam=bpy.context.object;cam.rotation_euler=(Vector((1,0,.4))-cam.location).to_track_quat('-Z','Y').to_euler();cam.data.type='ORTHO';cam.data.ortho_scale=10.;scene.camera=cam
scene.render.engine='CYCLES';scene.cycles.samples=20;scene.cycles.use_denoising=False;scene.render.resolution_x=1500;scene.render.resolution_y=1000;scene.render.resolution_percentage=100;scene.render.image_settings.file_format='PNG';scene.render.film_transparent=True;scene.render.filepath=str(out/'gate-counter-joined-studio.png');scene.view_settings.view_transform='AgX'
scene['Provenance']='Original hand-authored cabinet and telephone geometry. No game pixels, textures, ROMs, imported models or scripts.'
scene['Edge convention']='Runtime NESW mask. Exposed frames only; internal timber joins are flush.'
bpy.ops.wm.save_as_mainfile(filepath=str(out/'gate-counters.blend'),compress=False)
if '--skip-preview' not in args:
 bpy.ops.render.render(write_still=True)
 # Front and rear views of the actual exported phone geometry.
 for o in demo.objects:o.hide_render=True
 phone_col=assets['phone_console'];origin=Vector(((-5.6),8.6,0))
 # Phone is the seventeenth source module, at row two, column zero.
 for o in phone_col.objects:
  o.hide_render=False;o.location-=origin;o.location.x-=.64
 rear=bpy.data.collections.new('Telephone rear inspection');scene.collection.children.link(rear)
 for src in phone_col.objects:
  obj=src.copy();obj.data=src.data.copy();rear.objects.link(obj)
  local=src.location+Vector((.64,0,0));local.rotate(__import__('mathutils').Euler((0,0,math.pi)));obj.location=local+Vector((.64,0,0));obj.rotation_euler.rotate_axis('Z',math.pi)
 center=Vector((0,0,.45));cam.location=center+Vector((2.2,-5.7,2.4));cam.rotation_euler=(center-cam.location).to_track_quat('-Z','Y').to_euler();cam.data.ortho_scale=2.8
 for o in scene.objects:
  if o.type=='LIGHT':o.location=center+Vector((-2,-3,5));o.data.energy=450;o.data.size=4;o.rotation_euler=(center-o.location).to_track_quat('-Z','Y').to_euler()
 scene.render.resolution_x=1400;scene.render.resolution_y=900;scene.render.filepath=str(out/'gate-phone-front-rear.png');bpy.ops.render.render(write_still=True)

print('GATE_COUNTER_STATS='+json.dumps(stats))
