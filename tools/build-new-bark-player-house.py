"""Original two-storey New Bark player home refinement. Run: blender -b --python build_player_house.py.
Blender source is Z-up, front -Y; exports are glTF-standard Y-up, front +Z.
No imported sprites, textures, or Nintendo models. All meshes authored here.
"""
import bpy, math, random, json, sys
from pathlib import Path
from mathutils import Vector, Matrix
args = sys.argv[sys.argv.index('--') + 1:] if '--' in sys.argv else []
P=Path(next((a for a in args if not a.startswith('--')), 'target/new-bark-player-house')); P.mkdir(parents=True, exist_ok=True)
random.seed(2017)
bpy.ops.object.select_all(action='SELECT'); bpy.ops.object.delete(use_global=False)
for m in list(bpy.data.materials): bpy.data.materials.remove(m)
scene=bpy.context.scene
COLORS={
 'shutter_blue':(0.12,0.33,0.43), 'shutter_edge':(0.23,0.48,0.55), 'soil':(.16,.12,.09),
 'plaster':(0.87,0.79,0.59), 'plaster_light':(0.98,0.91,0.73), 'plaster_shadow':(0.70,0.60,0.41),
 'timber':(0.28,0.18,0.14), 'timber_light':(0.48,0.31,0.21), 'timber_gold':(0.64,0.43,0.26),
 'roof_red':(0.62,0.20,0.15), 'roof_red_light':(0.76,0.29,0.20), 'roof_red_dark':(0.43,0.14,0.12),
 'roof_red_warm':(0.69,0.25,0.16), 'roof_teal':(0.16,0.40,0.39), 'roof_teal_light':(0.24,0.51,0.46),
 'roof_teal_dark':(0.12,0.28,0.29), 'roof_teal_warm':(0.22,0.44,0.39),
 'stone':(0.43,0.47,0.41), 'stone_light':(0.62,0.65,0.54), 'stone_dark':(0.32,0.35,0.31),
 'glass':(0.27,0.52,0.58), 'glass_light':(0.63,0.82,0.79), 'glass_dark':(0.17,0.34,0.40),
 'leaf':(0.18,0.38,0.23), 'leaf_light':(0.35,0.53,0.29), 'leaf_dark':(0.11,0.28,0.20), 'leaf_gold':(0.48,0.61,0.33),
 'coral':(0.89,0.34,0.25), 'pink':(0.94,0.61,0.60), 'yellow':(0.97,0.73,0.29),
 'cream':(0.97,0.88,0.67), 'ink':(0.12,0.17,0.20), 'skin':(0.87,0.64,0.43),
 'skin_light':(0.98,0.75,0.51), 'trouser':(0.20,0.32,0.38), 'pack':(0.68,0.48,0.24),
 'white':(0.91,0.91,0.80)
}
MATS={}
for n,c in COLORS.items():
 m=bpy.data.materials.new(n); m.diffuse_color=(*c,1); m.use_nodes=True
 bs=m.node_tree.nodes.get('Principled BSDF'); bs.inputs['Base Color'].default_value=(*c,1); bs.inputs['Roughness'].default_value=.91
 MATS[n]=m
COL=None
ASSETS={}
def begin(name):
 global COL
 COL=bpy.data.collections.new(name);scene.collection.children.link(COL);ASSETS[name]=COL
 return COL

def move(o,name,mat):
 o.name=name
 for c in list(o.users_collection):c.objects.unlink(o)
 COL.objects.link(o)
 if mat:o.data.materials.append(MATS[mat])
 return o

def cube(n,loc,dim,mat,bevel=0):
 bpy.ops.mesh.primitive_cube_add(size=1,location=loc)
 o=move(bpy.context.object,n,mat);o.dimensions=dim;bpy.ops.object.transform_apply(location=False,rotation=False,scale=True)
 if bevel:
  b=o.modifiers.new('Hand-cut edges','BEVEL');b.width=min(bevel,min(dim)*.4);b.segments=1
  bpy.ops.object.modifier_apply(modifier=b.name)
 return o

def mesh(n,verts,faces,mat):
 me=bpy.data.meshes.new(n);me.from_pydata(verts,[],faces);me.update()
 o=bpy.data.objects.new(n,me);COL.objects.link(o);me.materials.append(MATS[mat]);return o

def ico(n,loc,scale,mat,sub=1):
 bpy.ops.mesh.primitive_ico_sphere_add(subdivisions=sub,radius=1,location=loc);o=move(bpy.context.object,n,mat);o.scale=scale;bpy.ops.object.transform_apply(location=False,rotation=False,scale=True);return o

def cone(n,loc,r1,r2,depth,mat,vertices=8):
 bpy.ops.mesh.primitive_cone_add(vertices=vertices,radius1=r1,radius2=r2,depth=depth,location=loc);return move(bpy.context.object,n,mat)

def rod(n,a,b,r,mat,vertices=6):
 a=Vector(a);b=Vector(b);o=cone(n,(a+b)/2,r,r*.80,(b-a).length,mat,vertices);o.rotation_euler=(b-a).to_track_quat('Z','Y').to_euler();return o

# Local facade widgets face -Y, rotated about building center by optional angle.
def facade_cube(n,x,z,w,h,y,depth,mat,angle=0,bevel=0):
 a=math.radians(angle);o=cube(n,(x*math.cos(a)-y*math.sin(a),x*math.sin(a)+y*math.cos(a),z),(w,depth,h),mat,bevel);o.rotation_euler.z=a;return o

def window(x,z,y,w=.68,h=.73,angle=0,shutters=False):
 f=lambda n,dx,dz,ww,hh,off,dd,mat,bevel=0:facade_cube(n,x+dx,z+dz,ww,hh,y+off,dd,mat,angle,bevel)
 f('Recessed window surround',0,0,w+.15,h+.15,-.015,.13,'timber')
 f('Painted window border',0,0,w+.09,h+.09,-.09,.035,'plaster_light')
 f('Cool glass',0,0,w-.07,h-.07,-.115,.025,'glass')
 f('Glass upper reflection',-.12*w,.19*h,w*.46,.06,-.132,.014,'glass_light')
 f('Window vertical bar',0,0,.055,h-.05,-.143,.04,'plaster_light')
 f('Window horizontal bar',0,0,w-.05,.052,-.144,.04,'plaster_light')
 f('Window ledge',0,-h/2-.055,w+.28,.10,-.10,.32,'timber_light')
 if shutters:
  for s in [-1,1]:
   f('Open wooden shutter',s*(w/2+.17),0,.17,h+.08,-.045,.045,'timber_light')
   for zz in [-.2,0,.2]:f('Shutter crosspiece',s*(w/2+.17),zz,.18,.025,-.075,.025,'timber_gold')

def door(y,z=1.02,w=.82,h=1.55,lab=False):
 facade_cube('Deep door frame',0,z,w+.23,h+.16,y,.16,'timber',bevel=.018)
 facade_cube('Cream inner door jamb',0,z,w+.13,h+.08,y-.105,.045,'plaster_light')
 facade_cube('Door leaf',0,z,w,h,y-.135,.04,'roof_teal_dark' if lab else 'timber_light',bevel=.017)
 if lab:
  for dx in [-.23,.23]:facade_cube('Lab door glazing',dx,z+.25,.37,.69,y-.162,.022,'glass')
  facade_cube('Door center seam',0,z,.045,h,y-.177,.02,'plaster_light')
  for dx in [-.08,.08]:facade_cube('Brass door grip',dx,z-.14,.035,.23,y-.191,.06,'yellow',bevel=.01)
 else:
  facade_cube('Door upper inset',0,z+.27,w-.22,.50,y-.162,.025,'timber')
  facade_cube('Door upper glass',0,z+.30,w-.32,.33,y-.177,.025,'glass')
  facade_cube('Door lower panel',0,z-.40,w-.24,.42,y-.162,.025,'timber_gold',bevel=.01)
  facade_cube('Brass door knob',w*.31,z-.1,.07,.07,y-.184,.07,'yellow',bevel=.023)

def roof(width,depth,eave,ridge,palette,rows):
 # Closed gable with rich fascia; independent roof panels overlap like handmade clay courses.
 x=width/2;d=depth/2
 mesh('Roof closed gable', [(-x,-d,eave),(x,-d,eave),(x,0,ridge),(-x,0,ridge),(-x,d,eave),(x,d,eave)],[(0,1,2,3),(3,2,5,4),(0,4,5,1)],palette+'_dark')
 for side in [-1,1]:
  for r in range(rows):
   t0=r/rows;t1=(r+1)/rows
   y0=side*d*(1-t0);y1=side*d*(1-t1)
   z0=eave+(ridge-eave)*t0+.015;z1=eave+(ridge-eave)*t1+.028
   count=5 if width<5 else 7
   for j in range(count):
    xx0=-x+j*width/count+.007;xx1=-x+(j+1)*width/count-.007
    # wedge strip is thin, genuinely modeled on both slopes
    vs=[(xx0,y0,z0),(xx1,y0,z0),(xx1,y1,z1),(xx0,y1,z1),(xx0,y0,z0-.048),(xx1,y0,z0-.048),(xx1,y1,z1-.038),(xx0,y1,z1-.038)]
    faces=[(0,1,2,3),(7,6,5,4),(4,5,1,0),(1,5,6,2),(2,6,7,3),(3,7,4,0)]
    if side>0:faces=[tuple(reversed(f)) for f in faces]
    color=[palette,palette,palette+'_light',palette+'_warm'][(j+2*r+(1 if side>0 else 0))%4]
    mesh('Individual roof course',vs,faces,color)
  cube('Eave shadow board',(0,side*(d-.025),eave-.025),(width,.12,.14),'timber')
 # gable bargeboards as rectangular beams along slope
 for xx in [-x+.015,x-.015]:
  for side in [-1,1]:
   a=Vector((xx,side*d,eave-.015));b=Vector((xx,0,ridge+.015));o=cube('Gable edge timber',(a+b)/2,(.095,.115,(b-a).length),'timber_light');o.rotation_euler=(b-a).to_track_quat('Z','Y').to_euler()
 cube('Ridge cap',(0,0,ridge+.038),(width,.18,.12),palette+'_light',.032)

def foundation(w,d):
 cube('Solid stone plinth',(0,0,.13),(w,d,.26),'stone',.035)
 # substantial corner footings and front blocks give the stonework a crafted rhythm
 for x in [-w/2+.12,w/2-.12]:
  for y in [-d/2+.12,d/2-.12]:cube('Dressed corner stone',(x,y,.18),(.28,.28,.28),'stone_light',.025)
 for x in [-w*.27,w*.27]:
  for y in [-d/2-.002,d/2+.002]:cube('Plinth block accent',(x,y,.135),(.68,.03,.15),'stone_light')

def building_trim(w,d,z,h):
 for x in [-w/2+.04,w/2-.04]:
  for y in [-d/2+.04,d/2-.04]:cube('Timber corner upright',(x,y,z),(.13,.13,h),'timber_light')
 cube('Foundation sill',(0,0,.35),(w+.07,d+.07,.14),'plaster_light',.016)
 for y in [-d/2-.02,d/2+.02]:cube('Long timber header',(0,y,z+h/2-.06),(w,.075,.11),'timber')
 for x in [-w/2-.02,w/2+.02]:cube('Side timber header',(x,0,z+h/2-.06),(.075,d,.11),'timber')

def blue_window(x,z,y,w=.63,h=.70,angle=0,shutters=True):
 # Shadows, cream reveals, subtly reflected blue glazing, painted shutters.
 window(x,z,y,w,h,angle,False)
 if shutters:
  for s in [-1,1]:
   cx=x+s*(w/2+.15)
   facade_cube('Lake-blue shutter',cx,z,.19,h+.09,y-.045,.066,'shutter_blue',angle)
   for dz in [-h*.36,0,h*.36]:
    facade_cube('Shutter pale crossrail',cx,z+dz,.18,.028,y-.087,.027,'shutter_edge',angle)

def boxbeam(n,a,b,width,depth,mat):
 a,b=Vector(a),Vector(b)
 o=cube(n,(a+b)/2,(width,depth,(b-a).length),mat)
 o.rotation_euler=(b-a).to_track_quat('Z','Y').to_euler()
 return o

def windowbox(x,y,z):
 # A modest living accent; every petal stays well clear of the front door.
 cube('Timber flowerbox body',(x,y,z),(.71,.21,.18),'timber_light',.012)
 cube('Flowerbox shadow soil',(x,y,z+.093),(.61,.16,.014),'soil')
 for yy in [y-.11,y+.11]:cube('Flowerbox rim',(x,yy,z+.098),(.75,.04,.05),'timber_gold',.005)
 for xx in [-.31,0,.31]:cube('Flowerbox strap',(x+xx,y-.116,z),(.035,.02,.19),'timber')
 for dx in [-.23,0,.23]:
  ico('Flowerbox leaf cluster',(x+dx,y,z+.17),(.145,.14,.125),'leaf',1)
  ico('Flowerbox light leaves',(x+dx-.035,y-.035,z+.225),(.095,.09,.075),'leaf_light',1)
  for off,cc in [(-.045,'coral'),(.044,'yellow')]:
   ico('Little flowering accent',(x+dx+off,y-.052,z+.28),(.055,.043,.046),cc,1)

begin('player_house')
foundation(3.65,3.53)
# A full second storey rather than an enlarged cottage. Clear floor line.
cube('Lower warm plaster storey',(0,0,1.09),(3.46,3.34,1.74),'plaster',.023)
cube('Upper pale plaster storey',(0,0,2.565),(3.46,3.34,1.21),'plaster_light',.023)
for x in [-1.70,1.70]:
 for y in [-1.64,1.64]:
  cube('Continuous oak corner post',(x,y,1.715),(.12,.12,2.92),'timber_light')
# Wraparound bands separate floors and tie the four facades together.
for z,h,mat in [(.35,.13,'plaster_light'),(1.945,.11,'timber'),(2.04,.09,'timber_light'),(3.12,.12,'timber')]:
 for y in [-1.68,1.68]:cube('Horizontal facade belt',(0,y,z),(3.49,.075,h),mat)
 for x in [-1.75,1.75]:cube('Horizontal side belt',(x,0,z),(.075,3.37,h),mat)
fy=-1.675
# Center door: threshold remains unobstructed, and door visually anchors the facade.
door(fy-.004,z=1.005,w=.80,h=1.49)
cube('Wide stone threshold',(0,-1.815,.12),(1.02,.34,.24),'stone_light',.021)
cube('Threshold front inset',(0,-1.988,.097),(.85,.024,.08),'stone')
for x in [-1.07,1.07]:
 blue_window(x,1.16,fy,w=.57,h=.64,shutters=True)
 blue_window(x,2.57,fy,w=.61,h=.70,shutters=True)
 windowbox(x,-1.852,.68)
# Full-height narrow door canopy ties entrance to its upper floor, not a separate porch.
cube('Door rain hood',(0,-1.78,1.865),(1.06,.35,.095),'roof_red_dark',.016)
cube('Door hood terracotta lip',(0,-1.947,1.887),(1.08,.05,.068),'roof_red_light',.012)
for x in [-.44,.44]:boxbeam('Door hood angled bracket',(x,-1.70,1.66),(x,-1.865,1.82),.05,.05,'timber_light')
# Side windows: intentionally offset bays make the home readable from the game camera.
for a in [-90,90]:
 for x in [-.80,.80]:
  blue_window(x,1.17,-1.737,w=.66,h=.69,angle=a,shutters=False)
  blue_window(x,2.56,-1.737,w=.66,h=.70,angle=a,shutters=False)
for x in [-.92,.92]:
 blue_window(x,1.17,fy,w=.66,h=.70,angle=180,shutters=False)
 blue_window(x,2.56,fy,w=.66,h=.70,angle=180,shutters=True)
# Front- and rear-facing gables make the silhouette distinct from the cottages.
for y in [-1.671,1.671]:
 mesh('Upper triangular gable',[(-1.73,y,3.16),(1.73,y,3.16),(0,y,3.986)],[(0,1,2)] if y<0 else [(2,1,0)],'plaster_light')
 for s in [-1,1]:
  boxbeam('Gable oak diagonal',(s*1.71,y-.027 if y<0 else y+.027,3.18),(0,y-.027 if y<0 else y+.027,3.97),.074,.058,'timber_light')
 cube('Gable central kingpost',(0,y,3.58),(.07,.09,.74),'timber_light')
# Small wood-framed attic vent is deliberately understated.
for a in [0,180]:
 facade_cube('Attic vent border',0,3.48,.29,.25,fy-.02,.10,'timber',a,.008)
 for z in [3.405,3.475,3.545]:facade_cube('Attic painted louvers',0,z,.225,.034,fy-.08,.04,'shutter_blue',a)
# Rotate the roof's ridge to run front-to-back; roof tiles remain modeled geometry.
roof_start=set(COL.objects)
roof(3.94,3.94,3.17,4.015,'roof_red',5)
bpy.context.view_layer.update()
roof_rotation=Matrix.Rotation(math.pi/2,4,'Z')
for obj in set(COL.objects)-roof_start:
 obj.matrix_world=roof_rotation @ obj.matrix_world
bpy.context.view_layer.update()
# A slim pale chimney sits on rear quarter of the roof, entirely within the silhouette bounds.
cube('Pale masonry chimney',(.98,.74,3.715),(.34,.39,.74),'plaster_shadow',.014)
for z in [3.46,3.66,3.86,4.04]:cube('Chimney light mortar',(.98,.74,z),(.36,.41,.033),'plaster_light')
cube('Chimney crown',(.98,.74,4.105),(.46,.49,.11),'roof_red_dark',.015)
cube('Chimney opening',(.98,.74,4.165),(.255,.29,.016),'ink')
# A small front entry light and brass number plate, both attached to the wall.
facade_cube('Entry lantern backing',.53,1.42,.075,.26,fy-.035,.05,'timber')
facade_cube('Amber entry lantern',.53,1.435,.105,.15,fy-.095,.11,'yellow',bevel=.014)
facade_cube('Entry lantern cap',.53,1.535,.145,.05,fy-.10,.145,'timber')

# Transform all authoring meshes into world coordinates, then normalize the occupied
# horizontal bounds to exactly 4 × 4. This includes roof boards, door step and boxes.
objects=[o for o in COL.objects if o.type=='MESH']
for o in objects:
 bpy.context.view_layer.objects.active=o;o.select_set(True)
 bpy.ops.object.transform_apply(location=True,rotation=True,scale=True);o.select_set(False)
minx=min(v.co.x for o in objects for v in o.data.vertices);maxx=max(v.co.x for o in objects for v in o.data.vertices)
miny=min(v.co.y for o in objects for v in o.data.vertices);maxy=max(v.co.y for o in objects for v in o.data.vertices)
sx=4/(maxx-minx);sy=4/(maxy-miny);cx=(maxx+minx)/2;cy=(maxy+miny)/2
for o in objects:
 for v in o.data.vertices:v.co.x=(v.co.x-cx)*sx;v.co.y=(v.co.y-cy)*sy
 o.data.update()
 o['Original artist mesh']=True
 bpy.context.view_layer.objects.active=o;o.select_set(True)
 bpy.ops.object.mode_set(mode='EDIT');bpy.ops.mesh.select_all(action='SELECT');bpy.ops.mesh.normals_make_consistent(inside=False);bpy.ops.object.mode_set(mode='OBJECT');o.select_set(False)

# Export with one primitive per material while retaining separate editable source pieces.
def export_asset(name,source):
 export=bpy.data.collections.new('TEMP_EXPORT');scene.collection.children.link(export)
 groups={}
 for src in source.objects:
  if src.type!='MESH':continue
  ob=src.copy();ob.data=src.data.copy();export.objects.link(ob)
  groups.setdefault(ob.data.materials[0].name,[]).append(ob)
 for mat,parts in groups.items():
  bpy.ops.object.select_all(action='DESELECT')
  for ob in parts:ob.select_set(True)
  bpy.context.view_layer.objects.active=parts[0];bpy.ops.object.join();bpy.context.object.name=f'{name}__{mat}'
  scene.cursor.location=(0,0,0);bpy.ops.object.origin_set(type='ORIGIN_CURSOR')
 primitives=[];minv=[float('inf')]*3;maxv=[-float('inf')]*3;tricount=0
 for ob in sorted(export.objects,key=lambda o:o.name):
  me=ob.data;me.calc_loop_triangles();positions=[];normals=[];indices=[];mapping={}
  for tri in me.loop_triangles:
   if tri.normal.length_squared<.9:continue
   for vid in tri.vertices:
    v=ob.matrix_world@me.vertices[vid].co;n=tri.normal
    pos=(round(v.x,6),round(v.z,6),round(-v.y,6));norm=(round(n.x,6),round(n.z,6),round(-n.y,6))
    for i in range(3):minv[i]=min(minv[i],pos[i]);maxv[i]=max(maxv[i],pos[i])
    key=pos+norm
    if key not in mapping:mapping[key]=len(positions)//3;positions.extend(pos);normals.extend(norm)
    indices.append(mapping[key])
  mat=ob.data.materials[0]
  primitives.append({'name':ob.name,'material':mat.name,'base_color':[round(v,5) for v in mat.diffuse_color],'positions':positions,'normals':normals,'indices':indices})
  tricount+=len(indices)//3
 data={'name':name,'coordinate_system':'right-handed; +Y up; front +Z','origin':'floor-center','bounds':{'min':minv,'max':maxv},'dimensions':[round(maxv[i]-minv[i],6) for i in range(3)],'triangle_count':tricount,'primitive_count':len(primitives),'door_anchor':[0,0,2.0],'door_width':round(.80*sx,6),'primitives':primitives}
 (P/f'{name}.mesh.json').write_text(json.dumps(data,separators=(',',':')))
 bpy.ops.object.select_all(action='DESELECT')
 for ob in export.objects:ob.select_set(True)
 bpy.context.view_layer.objects.active=list(export.objects)[0]
 bpy.ops.export_scene.gltf(filepath=str(P/f'{name}.glb'),use_selection=True,export_format='GLB',export_yup=True,export_apply=True,export_materials='EXPORT',export_animations=False)
 for ob in list(export.objects):bpy.data.objects.remove(ob,do_unlink=True)
 bpy.data.collections.remove(export)
 return {k:v for k,v in data.items() if k!='primitives'}
STATS=export_asset('player_house',ASSETS['player_house'])
assert STATS['dimensions'][0]==4 and STATS['dimensions'][2]==4
assert STATS['dimensions'][1]<=4.2
assert STATS['triangle_count']<5000,STATS['triangle_count']
(P/'asset-stats.json').write_text(json.dumps(STATS,indent=2))
# Presentation only: never included in exports. The editable home stays floor-centered.
begin('PRESENTATION only')
cube('Warm studio floor',(0,0,-.105),(200,200,.20),'cream')
world=bpy.data.worlds.new('Soft daylight');scene.world=world;world.use_nodes=True
world.node_tree.nodes['Background'].inputs['Color'].default_value=(.30,.39,.48,1);world.node_tree.nodes['Background'].inputs['Strength'].default_value=.5
for name,loc,energy,size,color in [('Key',(-6,-8,12),1250,7,(1,.87,.73)),('Fill',(7,-2,8),600,6,(.77,.88,1)),('Rim',(0,6,11),1100,5,(1,.94,.81))]:
 bpy.ops.object.light_add(type='AREA',location=loc);o=bpy.context.object;o.name=name;o.data.energy=energy;o.data.shape='DISK';o.data.size=size;o.data.color=color;o.rotation_euler=(Vector((0,0,1.6))-o.location).to_track_quat('-Z','Y').to_euler()
bpy.ops.object.camera_add(location=(7.5,-11,8));cam=bpy.context.object;cam.name='Player home beauty camera';cam.rotation_euler=(Vector((0,0,1.92))-cam.location).to_track_quat('-Z','Y').to_euler();cam.data.type='ORTHO';cam.data.ortho_scale=7.0;scene.camera=cam
scene.render.engine='CYCLES';scene.cycles.samples=96;scene.cycles.use_denoising=False
scene.render.resolution_x=1400;scene.render.resolution_y=1400;scene.render.resolution_percentage=100
scene.view_settings.view_transform='AgX';scene.render.image_settings.file_format='PNG';scene.render.filepath=str(P/'player-house-preview.png')
scene['Asset source note']='Original hand-crafted geometry; no textures or image planes. Editable pieces retained in player_house collection. Presentation excluded from export.'
scene['Source coordinates']='Z-up, front -Y; exports +Y-up, front +Z, floor-center. Exact occupied footprint 4 × 4.'
scene['door_anchor_export']=[0,0,2.0]
for screen in bpy.data.screens:
 for area in screen.areas:
  if area.type=='VIEW_3D':area.spaces.active.region_3d.view_perspective='CAMERA'
bpy.ops.wm.save_as_mainfile(filepath=str(P/'player-house-editable.blend'))
bpy.ops.render.render(write_still=True)
print('FINAL_STATS='+json.dumps(STATS))
