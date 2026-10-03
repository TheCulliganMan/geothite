"""Original papercraft Gym scenery; no source pixels or game assets imported.
blender -b --threads 2 --python tools/build-gym-scenery-models.py -- target/gym-scenery [--skip-preview]
Each original part is retained as a named, independently editable closed mesh.
"""
import bpy,math,json,sys,struct
from pathlib import Path
from mathutils import Vector
args=sys.argv[sys.argv.index('--')+1:] if '--' in sys.argv else []
out=Path(next((a for a in args if not a.startswith('--')),'target/gym-scenery'));out.mkdir(parents=True,exist_ok=True)
bpy.ops.object.select_all(action='SELECT');bpy.ops.object.delete(use_global=False)
scene=bpy.context.scene
palette={'oak':(.42,.26,.15),'oak_light':(.62,.42,.24),'oak_end':(.31,.19,.13),'soil':(.15,.115,.095),'leaf_dark':(.14,.28,.15),'leaf':(.25,.43,.20),'leaf_light':(.43,.61,.29),'leaf_gold':(.58,.68,.35),'bark':(.34,.22,.15),'bark_light':(.47,.32,.21),'brass':(.74,.60,.32),'wall':(.41,.43,.43),'wall_light':(.56,.59,.55),'wall_shadow':(.27,.31,.32),'coping':(.67,.71,.64),'trim':(.48,.55,.42)}
materials={}
for name,rgb in palette.items():
 m=bpy.data.materials.new(name);m.diffuse_color=(*rgb,1);m.use_nodes=True;bs=m.node_tree.nodes.get('Principled BSDF');bs.inputs['Base Color'].default_value=(*rgb,1);bs.inputs['Roughness'].default_value=.86;materials[name]=m
assets={};col=None

def begin(name):
 global col
 col=bpy.data.collections.new(name);scene.collection.children.link(col);assets[name]=col

def assign(o,name,material):
 o.name=name
 for c in list(o.users_collection):c.objects.unlink(o)
 col.objects.link(o);o.data.materials.append(materials[material]);return o

def box(name,p,size,material,bevel=.018):
 bpy.ops.mesh.primitive_cube_add(size=1,location=p);o=assign(bpy.context.object,name,material);o.dimensions=size;bpy.ops.object.transform_apply(location=False,rotation=False,scale=True)
 # At native Gym scale, tiny edge strips are well below one screen pixel.
 # Preserve substantial base/soil chamfers; keep the entire named cuboid
 # for small staves, feet, bindings and wall ornaments, with closed faces.
 bevel_width=min(bevel,min(size)*.23)
 if bevel_width>=.03:
  mod=o.modifiers.new('Single hand-cut bevel','BEVEL');mod.width=bevel_width;mod.segments=1;bpy.ops.object.modifier_apply(modifier=mod.name)
 return o

def cone(name,p,r1,r2,h,material,vertices=8):
 bpy.ops.mesh.primitive_cone_add(vertices=vertices,radius1=r1,radius2=r2,depth=h,location=p);return assign(bpy.context.object,name,material)

def rod(name,a,b,r,material,tip=None):
 a,b=Vector(a),Vector(b);o=cone(name,(a+b)*.5,r,r if tip is None else tip,(b-a).length,material);o.rotation_euler=(b-a).to_track_quat('Z','Y').to_euler();return o

def ico(name,p,size,material,sub=2):
 bpy.ops.mesh.primitive_ico_sphere_add(subdivisions=sub,radius=1,location=p);o=assign(bpy.context.object,name,material);o.scale=size;bpy.ops.object.transform_apply(location=False,rotation=False,scale=True);return o

def poly(name,vertices,faces,material):
 me=bpy.data.meshes.new(name);me.from_pydata(vertices,[],faces);me.update();o=bpy.data.objects.new(name,me);col.objects.link(o);me.materials.append(materials[material]);return o

def leaf(name,a,b,width,material):
 # A sealed double-sided folded leaf with a physical central ridge.
 a,b=Vector(a),Vector(b);delta=b-a;side=delta.cross(Vector((0,0,1)))
 if side.length<.01:side=Vector((1,0,0))
 side.normalize();mid=a+delta*.52;ridge=Vector((0,0,width*.16))
 vertices=[a,b,mid+side*width,mid-side*width,mid+ridge,mid-ridge]
 return poly(name,vertices,[(0,2,4),(2,1,4),(1,3,4),(3,0,4),(2,0,5),(1,2,5),(3,1,5),(0,3,5)],material)

def planter(name,rounded):
 begin(name)
 box('Recessed closed planter base',(0,0,.14),(1.32,1.16,.24),'oak_end',.035)
 box('Inset soil visible between plants',(0,0,.45),(1.28,1.10,.19),'soil',.035)
 for side in [-1,1]:
  for z in [.20,.39]:
   box('Separate long oak stave',(0,side*.605,z),(1.46,.12,.175),'oak' if z<.3 else 'oak_light',.016)
   box('Separate end-grain stave',(side*.69,0,z),(.12,1.12,.175),'oak',.016)
  box('Long mitered top rim',(0,side*.615,.53),(1.55,.145,.11),'oak_light',.023)
  box('Short mitered top rim',(side*.705,0,.53),(.14,1.09,.11),'oak_light',.023)
  for x in [-.55,.55]:
   box('Corner binding batten',(x,side*.683,.32),(.095,.045,.47),'oak_end',.009)
   for z in [.15,.48]:ico('Small brass joinery pin',(x,side*.712,z),(.025,.012,.025),'brass',1)
 for x in [-.51,.51]:
  for y in [-.43,.43]:box('Planter squared foot',(x,y,.07),(.18,.18,.14),'oak',.023)
 if rounded:
  for i,(x,y,z,s) in enumerate([(-.32,.06,.86,.39),(.30,.05,.88,.40),(0,-.19,.95,.38)]):
   rod('Short shrub stem',(x,y,.49),(x,y,z),.06,'bark')
   ico('Rounded clipped shrub crown',(x,y,z),(s,s*.82,s*.85),'leaf' if i!=1 else 'leaf_light',2)
   for j in range(3):
    a=j*math.tau/3+i*.7
    leaf('Raised folded leaf facet',(x+math.cos(a)*s*.55,y+math.sin(a)*s*.55,z+s*.30),(x+math.cos(a)*s*.20,y+math.sin(a)*s*.20,z+s*.82),.10,'leaf_light' if j else 'leaf_gold')
 else:
  for i,(x,y) in enumerate([(-.41,0),(0,.12),(.41,0),(0,-.29)]):
   rod('Leafy plant central stem',(x,y,.47),(x,y,.90),.026,'leaf_dark')
   for j in range(5):
    a=j*math.tau/5+i*.43;length=.33 if j%2 else .41
    leaf('Broad upright folded leaf',(x,y,.58),(x+math.cos(a)*length,y+math.sin(a)*length,.82+.20*(j%2)),.12,['leaf','leaf_light','leaf_gold'][j%3])
planter('planter_leafy',False);planter('planter_round',True)

begin('azalea_broad_tree')
cone('Tapered faceted trunk',(0,-.11,.79),.23,.15,1.57,'bark',9)
for i in range(6):
 a=i*math.tau/6+.2
 rod('Buttress root',(0,-.11,.25),(.45*math.cos(a),-.11+.34*math.sin(a),.035),.10,'bark',.065)
for i,(x,y,z) in enumerate([(-.55,.04,1.79),(.48,.01,1.83),(0,.39,1.85)]):
 rod('Spreading branch',(0,-.10,.98),(x,y,z),.11,'bark_light',.055)
# A broad low crown with seven distinct hand-cut lobes. The trunk stays legible.
for i,(x,y,z,sx,sy,sz) in enumerate([(0,.03,2.12,.81,.68,.66),(-.64,.01,1.99,.60,.60,.49),(.62,.06,2.04,.59,.60,.50),(0,-.47,1.97,.69,.48,.45),(-.40,.49,2.11,.58,.47,.48),(.42,.49,2.15,.55,.46,.47),(0,.01,2.52,.57,.48,.34)]):
 ico('Broad rounded canopy lobe',(x,y,z),(sx,sy,sz),['leaf','leaf_light','leaf','leaf_dark','leaf','leaf_light','leaf_gold'][i],2)
for i in range(9):
 a=i*math.tau/9
 leaf('Crown folded leaf cluster',(.68*math.cos(a),.49*math.sin(a),2.34),(.53*math.cos(a),.40*math.sin(a),2.59),.13,'leaf_light')

begin('celadon_round_hedge')
cone('Concealed woody hedge foot',(0,0,.16),.19,.12,.32,'bark',8)
ico('Dense low clipped hedge core',(0,0,.71),(.74,.68,.65),'leaf_dark',2)
for i in range(6):
 a=i*math.tau/6
 ico('Rounded trimmed leaf mass',(.37*math.cos(a),.32*math.sin(a),.86),(.40,.37,.37),'leaf' if i%2 else 'leaf_light',2)
ico('Flat rounded crown',(0,0,1.12),(.48,.43,.29),'leaf',2)
for i in range(6):
 a=i*math.tau/6+.4
 leaf('Small crown folded facet',(.48*math.cos(a),.42*math.sin(a),1.11),(.30*math.cos(a),.27*math.sin(a),1.31),.095,'leaf_gold')

def prism(name,z0,z1,material,mask,inset=0):
 # Chamfer only an exposed exterior corner; joined edges remain exactly aligned.
 west=-.5+(inset if mask&8 else 0);east=.5-(inset if mask&2 else 0);south=-.5+(inset if mask&4 else 0);north=.5-(inset if mask&1 else 0)
 ch=.075;corners=[(west,south),(east,south),(east,north),(west,north)];pts=[]
 for i,(x,y) in enumerate(corners):
  open_a=bool(mask & (4 if y<0 else 1));open_b=bool(mask & (8 if x<0 else 2))
  if open_a and open_b:
   prev=corners[(i-1)%4];nex=corners[(i+1)%4]
   a=Vector((x,y));va=(Vector(prev)-a).normalized();vb=(Vector(nex)-a).normalized();pts.extend([tuple(a+va*ch),tuple(a+vb*ch)])
  else:pts.append((x,y))
 n=len(pts);verts=[(x,y,z)for z in [z0,z1]for x,y in pts]
 faces=[tuple(reversed(range(n))),tuple(range(n,2*n))]+[(i,(i+1)%n,(i+1)%n+n,i+n)for i in range(n)]
 return poly(name,verts,faces,material)

for mask in range(16):
 begin(f'maze_wall_{mask:02x}')
 prism('Closed masonry base course',0,.34,'wall',mask,.018)
 prism('Continuous recessed belt course',.32,.42,'wall_shadow',mask,.032)
 prism('Closed dressed-stone upper course',.40,.84,'wall_light',mask,.055)
 prism('Broad hand-cut coping',.82,1.,'coping',mask)
 # Edge ornament is only emitted on exposed faces. Every module retains the
 # exact unit footprint and 1.0 top datum, including an interior four-way join.
 for side,bit in [('north',1),('east',2),('south',4),('west',8)]:
  if not mask&bit:continue
  angle={'south':0,'east':math.pi/2,'north':math.pi,'west':-math.pi/2}[side]
  def face_box(label,x,z,w,h,depth,mat):
   # Front starts on south -Y; rotate into the requested outward face.
   px=x*math.cos(angle)+(.498-depth*.5)*math.sin(angle);py=x*math.sin(angle)-(.498-depth*.5)*math.cos(angle)
   o=box(label,(px,py,z),(w,depth,h),mat,.009);o.rotation_euler.z=angle;return o
  face_box('Inset stone face panel',0,.62,.65,.24,.027,'wall')
  face_box('Raised continuous olive band',0,.40,.86,.065,.045,'trim')
  for x in [-.37,.37]:face_box('Carved end pilaster',x,.61,.082,.42,.055,'coping')
  # Diamond relief stays inside the source footprint, with genuine thickness.
  o=face_box('Small stone diamond relief',0,.62,.115,.115,.039,'trim');o.rotation_euler=(0,math.pi/4,angle)

# Runtime exporter keeps each part's identity for watertight-part validation.
def export(name,collection):
 prim=[];deps=bpy.context.evaluated_depsgraph_get()
 for obj in collection.objects:
  if obj.type!='MESH':continue
  ev=obj.evaluated_get(deps);me=ev.to_mesh();me.calc_loop_triangles();normalmat=obj.matrix_world.to_3x3().inverted().transposed();groups={}
  for tri in me.loop_triangles:
   mat=obj.data.materials[tri.material_index];g=groups.setdefault(mat.name,{'name':obj.name+':'+mat.name,'part':obj.name,'base_color':[round(v,5)for v in mat.diffuse_color],'positions':[],'normals':[],'indices':[]})
   normal=(normalmat@tri.normal).normalized()
   for vid in tri.vertices:
    v=obj.matrix_world@me.vertices[vid].co;g['indices'].append(len(g['positions'])//3);g['positions'].extend(round(c,6)for c in (v.x,v.z,-v.y));g['normals'].extend(round(c,6)for c in (normal.x,normal.z,-normal.y))
  prim.extend(groups.values());ev.to_mesh_clear()
 xyz=[tuple(g['positions'][i:i+3])for g in prim for i in range(0,len(g['positions']),3)]
 data={'name':name,'coordinate_system':'right-handed; +Y up; front +Z','origin':'floor-center','bounds':{'min':[min(p[i]for p in xyz)for i in range(3)],'max':[max(p[i]for p in xyz)for i in range(3)]},'triangle_count':sum(len(g['indices'])//3 for g in prim),'primitives':prim}
 # Keep a raw reference only during explicit offline equivalence verification.
 if '--write-unindexed-reference' in args:
  reference=out/'unindexed-reference';reference.mkdir(exist_ok=True)
  (reference/(name+'.mesh.json')).write_text(json.dumps(data,separators=(',',':'))+'\n')
 # Reuse exactly equal attributes within one named material primitive only.
 # IEEE keys preserve even signed zero; distinct hard-edge normals never weld.
 # Color and the loader's constant UV belong to the primitive, so all indexed
 # triangle attributes and their order are unchanged by this storage reduction.
 for g in prim:
  positions,normals,indices=g['positions'],g['normals'],g['indices'];unique={};new_positions=[];new_normals=[];new_indices=[]
  for old in indices:
   position=positions[old*3:old*3+3];normal=normals[old*3:old*3+3];key=struct.pack('!6d',*(position+normal))
   if key not in unique:
    unique[key]=len(new_positions)//3;new_positions.extend(position);new_normals.extend(normal)
   new_indices.append(unique[key])
  g['positions'],g['normals'],g['indices']=new_positions,new_normals,new_indices
 (out/(name+'.mesh.json')).write_text(json.dumps(data,separators=(',',':'))+'\n');return {'name':name,'triangles':data['triangle_count'],'vertices':sum(len(g['positions'])//3 for g in prim),'parts':len(collection.objects)}
stats=[export(name,col)for name,col in assets.items()]
for i,(name,col)in enumerate(assets.items()):
 offset=Vector(([-4.9,-1.7,1.7,4.9][i],-2.9,0))if i<4 else Vector((((i-4)%4)*2.4-3.6,((i-4)//4)*1.8+.3,0))
 for obj in col.objects:obj.location+=offset
 # Simple studio labels are excluded from runtime models.
 bpy.ops.object.text_add(location=offset+Vector((-.78,-.94,.025)));o=bpy.context.object;o.name='Gallery label '+name;o.data.body=name.replace('_',' ');o.data.size=.15;o.data.extrude=0;o.rotation_euler=(0,0,0)
world=bpy.data.worlds.new('Gym neutral studio');scene.world=world;world.use_nodes=True;world.node_tree.nodes['Background'].inputs['Color'].default_value=(.15,.19,.22,1);world.node_tree.nodes['Background'].inputs['Strength'].default_value=.65
for p,energy,size in [((-5,-7,13),1500,8),((7,6,10),1100,7)]:
 bpy.ops.object.light_add(type='AREA',location=p);o=bpy.context.object;o.data.energy=energy;o.data.size=size;o.rotation_euler=(Vector((0,1,0))-o.location).to_track_quat('-Z','Y').to_euler()
bpy.ops.object.camera_add(location=(9,-17,19));cam=bpy.context.object;cam.rotation_euler=(Vector((0,1,.6))-cam.location).to_track_quat('-Z','Y').to_euler();cam.data.type='ORTHO';cam.data.ortho_scale=15.8;scene.camera=cam
scene.render.engine='CYCLES';scene.cycles.samples=16;scene.cycles.use_denoising=False;scene.render.threads_mode='FIXED';scene.render.threads=2;scene.render.resolution_x=1440;scene.render.resolution_y=1080;scene.render.resolution_percentage=100;scene.render.image_settings.file_format='PNG';scene.render.film_transparent=False;scene.render.filepath=str(out/'gym-scenery-kit.png');scene.view_settings.view_transform='AgX'
scene['Provenance']='Original authored geometric Gym scenery. No pack pixels, textures, ROM data, map scripts, or game meshes imported.'
bpy.ops.wm.save_as_mainfile(filepath=str(out/'gym-scenery-kit.blend'),compress=False)
if '--skip-preview'not in args:bpy.ops.render.render(write_still=True)
print('GYM_SCENERY_STATS='+json.dumps(stats))
