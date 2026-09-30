"""Original articulated Johto character kit, authored entirely as geometry.

blender -b --python tools/build-johto-characters.py -- target/johto-character-kit
Use --skip-preview to export without the studio character lineup.

The checked-in runtime JSON preserves rigid joint pivots and smooth corner
normals. It is not a flattened animation: every limb has its own mesh, parent,
and bind transform. Source .blend and preview PNG are development artifacts.
Blender source: +Z up, front -Y. Runtime: +Y up, front +Z.
"""
import bpy, json, math, sys
from pathlib import Path
from mathutils import Vector

args = sys.argv[sys.argv.index('--') + 1:] if '--' in sys.argv else []
OUT = Path(next((a for a in args if not a.startswith('--')), 'target/johto-character-kit')).resolve()
OUT.mkdir(parents=True, exist_ok=True)
bpy.ops.object.select_all(action='SELECT'); bpy.ops.object.delete(use_global=False)
scene = bpy.context.scene

# All joints have identity bind rotation. World pivots become parent-relative
# translations on export. Ankle and knee positions match the runtime IK solver.
JOINTS = [
 ('pelvis', None, (0, 0, .66)),
 ('torso', 'pelvis', (0, 0, .77)),
 ('head', 'torso', (0, 0, 1.105)),
 ('eyes', 'head', (0, -.168, 1.295)),
 ('upper_arm_l', 'torso', (-.225, 0, 1.00)),
 ('forearm_l', 'upper_arm_l', (-.242, -.008, .805)),
 ('hand_l', 'forearm_l', (-.248, -.02, .672)),
 ('upper_arm_r', 'torso', (.225, 0, 1.00)),
 ('forearm_r', 'upper_arm_r', (.242, -.008, .805)),
 ('hand_r', 'forearm_r', (.248, -.02, .672)),
 ('thigh_l', 'pelvis', (-.105, 0, .66)),
 ('shin_l', 'thigh_l', (-.105, 0, .39)),
 ('shoe_l', 'shin_l', (-.105, 0, .12)),
 ('thigh_r', 'pelvis', (.105, 0, .66)),
 ('shin_r', 'thigh_r', (.105, 0, .39)),
 ('shoe_r', 'shin_r', (.105, 0, .12)),
]
PIVOTS = {n: Vector(p) for n, _, p in JOINTS}
MATERIALS = {}
PARTS = {}
CURRENT = None
GROUP = None

# Input is sRGB, converted once to linear for physically based runtime shading.
def linear(v):
 return v / 12.92 if v <= .04045 else ((v + .055) / 1.055) ** 2.4

def material(name, color):
 key=(name, tuple(color))
 if key not in MATERIALS:
  m=bpy.data.materials.new(name); m.diffuse_color=tuple(linear(c) for c in color)+(1,)
  m.use_nodes=True; bs=m.node_tree.nodes.get('Principled BSDF')
  bs.inputs['Base Color'].default_value=m.diffuse_color; bs.inputs['Roughness'].default_value=.83
  MATERIALS[key]=m
 return MATERIALS[key]

PAL = {
 'skin': (.99,.77,.57), 'skin_shadow': (.88,.58,.40), 'skin_blush': (.95,.60,.49),
 'ink': (.13,.18,.25), 'eye': (.13,.12,.15), 'white': (.97,.97,.89),
 'cream': (.96,.89,.66), 'gold': (.96,.69,.21), 'sole': (.89,.84,.70),
 'navy': (.17,.28,.42), 'denim': (.28,.43,.54), 'red': (.82,.22,.23),
 'red_dark': (.57,.12,.18), 'teal': (.21,.61,.60), 'mint': (.48,.75,.67),
 'brown': (.30,.17,.12), 'hair': (.19,.13,.11), 'silver': (.68,.72,.71),
 'khaki': (.65,.65,.40), 'olive': (.35,.48,.30), 'tan': (.79,.61,.35),
 'purple': (.39,.25,.49), 'sock': (.77,.78,.75),
}

def start(name):
 global GROUP, PARTS
 GROUP=bpy.data.collections.new(name);scene.collection.children.link(GROUP)
 PARTS={n: [] for n,_,_ in JOINTS}

def part(name):
 global CURRENT
 CURRENT=name

def finish(o, name, color, smooth=True):
 o.name=name
 for col in list(o.users_collection): col.objects.unlink(o)
 GROUP.objects.link(o); PARTS[CURRENT].append(o)
 o.data.materials.append(material(color, PAL[color]))
 if smooth:
  for f in o.data.polygons:f.use_smooth=True
 return o

def orb(name, p, s, color, segments=16, rings=8):
 bpy.ops.mesh.primitive_uv_sphere_add(segments=segments, ring_count=rings, radius=1, location=p)
 o=finish(bpy.context.object,name,color);o.scale=s
 bpy.ops.object.transform_apply(location=False,rotation=False,scale=True)
 return o

def box(name,p,s,color,r=.025):
 bpy.ops.mesh.primitive_cube_add(size=1, location=p)
 o=finish(bpy.context.object,name,color);o.dimensions=s
 bpy.ops.object.transform_apply(location=False,rotation=False,scale=True)
 if r:
  m=o.modifiers.new('Soft tailored edge','BEVEL');m.width=min(r,min(s)*.45);m.segments=3
  bpy.ops.object.modifier_apply(modifier=m.name)
  m=o.modifiers.new('Weighted corner normals','WEIGHTED_NORMAL');m.keep_sharp=True;m.weight=30
  bpy.ops.object.modifier_apply(modifier=m.name)
 return o

def rod(name,a,b,r1,r2,color):
 a,b=Vector(a),Vector(b)
 bpy.ops.mesh.primitive_cone_add(vertices=12,radius1=r1,radius2=r2,depth=(b-a).length,location=(a+b)*.5)
 o=finish(bpy.context.object,name,color);o.rotation_euler=(b-a).to_track_quat('Z','Y').to_euler()
 return o

def panel(name,verts,faces,color):
 me=bpy.data.meshes.new(name);me.from_pydata(verts,[],faces);me.update()
 o=bpy.data.objects.new(name,me);GROUP.objects.link(o);PARTS[CURRENT].append(o);me.materials.append(material(color,PAL[color]))
 return o

def rings(name, specs, color, segments=20):
 # Smooth elliptical tailored volume, with deliberate taper and rounded hems.
 vs=[]
 for z,rx,ry in specs:
  vs.extend([(rx*math.cos(i*math.tau/segments),ry*math.sin(i*math.tau/segments),z) for i in range(segments)])
 fs=[tuple(reversed(range(segments)))]
 for j in range(len(specs)-1):
  for i in range(segments):
   k=j*segments+i;n=j*segments+(i+1)%segments;fs.append((k,n,n+segments,k+segments))
 fs.append(tuple(range((len(specs)-1)*segments,len(specs)*segments)))
 o=panel(name,vs,fs,color)
 for poly in o.data.polygons:poly.use_smooth=len(poly.vertices)==4
 return o

def face(hair, female=False, elder=False, glasses=False):
 part('head')
 orb('Soft neck',(0,0,1.103),(.075,.074,.091),'skin_shadow')
 orb('Rounded cheek and jaw',(0,-.005,1.297),(.187,.165,.211),'skin',24,12)
 for s in [-1,1]:
  orb('Ear',(s*.184,0,1.29),(.039,.037,.059),'skin',12,8)
  orb('Ear inner',(s*.201,-.026,1.294),(.014,.013,.027),'skin_shadow',12,6)
  orb('Cheek',(s*.115,-.140,1.248),(.035,.012,.019),'skin_blush',12,6)
  box('Soft eyebrow',(s*.078,-.155,1.355),(.061,.017,.013),hair,.006)
 orb('Button nose',(0,-.164,1.267),(.028,.032,.033),'skin',12,8)
 box('Quiet smile',(0,-.162,1.223),(.046,.014,.010),'skin_shadow',.004)
 # Eyes are a separate rigid part, so a blink closes real eye geometry.
 part('eyes')
 for s in [-1,1]:
  orb('Eye white',(s*.075,-.155,1.300),(.036,.019,.047),'white',16,8)
  orb('Iris',(s*.075,-.171,1.297),(.023,.010,.034),'eye',12,8)
  orb('Eye catchlight',(s*.068,-.180,1.310),(.009,.004,.011),'white',10,6)
 part('head')
 # Hair cap is a fitted upper hemisphere rather than a ball through the face.
 specs=[(1.354,.186,.166),(1.40,.188,.167),(1.452,.168,.146),(1.49,.112,.103),(1.509,.025,.025)]
 rings('Sculpted hair crown',specs,hair)
 orb('Hair at nape',(0,.102,1.315),(.163,.088,.141),hair)
 for s in [-1,1]:
  orb('Side lock',(s*.163,.0,1.353),(.040,.076,.091),hair)
 # Deliberate asymmetrical fringe, readable at the gameplay camera.
 for x,z,r in [(-.106,1.388,.065),(-.029,1.405,.070),(.060,1.425,.065),(.12,1.405,.043)]:
  o=orb('Swept fringe',(x,-.118,z),(r,.057,.059),hair);o.rotation_euler.y=-.32
 if female:
  for s in [-1,1]:
   orb('Low ponytail',(s*.162,.130,1.281),(.077,.063,.138),hair)
   orb('Hair ribbon',(s*.159,.143,1.383),(.055,.040,.025),'gold')
 if elder:
  orb('Sculpted moustache',(-.035,-.17,1.233),(.052,.025,.019),'silver')
  orb('Sculpted moustache',(.035,-.17,1.233),(.052,.025,.019),'silver')
 if glasses:
  for s in [-1,1]:
   for z in [1.25,1.351]:box('Spectacle frame',(s*.076,-.181,z),(.120,.012,.012),'ink',.004)
   for dx in [-.060,.060]:box('Spectacle frame',(s*.076+dx,-.18,1.30),(.012,.014,.105),'ink',.004)
  box('Spectacle bridge',(0,-.182,1.311),(.040,.016,.013),'ink',.004)

def cap(primary, peak, badge=True):
 part('head')
 rings('Six panel cap',[(1.428,.203,.18),(1.45,.207,.181),(1.507,.186,.157),(1.558,.130,.113),(1.578,.026,.024)],primary)
 rings('Woven cap band',[(1.415,.200,.179),(1.435,.208,.184)],'cream' if primary=='navy' else 'red_dark')
 orb('Curved visor',(0,-.190,1.429),(.217,.126,.027),peak,24,8)
 orb('Cap top button',(0,0,1.579),(.031,.030,.018),primary,12,6)
 if badge:
  orb('Cap badge',(0,-.165,1.505),(.053,.016,.047),'cream')
  box('Cap badge stripe',(0,-.181,1.505),(.047,.009,.012),primary,.004)

def backpack():
 part('torso')
 box('Canvas backpack',(0,.162,.86),(.289,.154,.351),'tan',.046)
 box('Backpack flap',(0,.180,1.007),(.307,.167,.083),'gold',.030)
 box('Outer pocket',(0,.251,.817),(.195,.048,.156),'khaki',.025)
 box('Backpack clasp',(0,.277,.868),(.042,.020,.069),'cream',.009)
 for s in [-1,1]:
  box('Front shoulder strap',(s*.125,-.117,.956),(.047,.034,.180),'tan',.010)
  orb('Shoulder strap turn',(s*.135,.021,1.057),(.032,.142,.025),'tan')
  box('Brass strap slider',(s*.125,-.137,.913),(.050,.012,.032),'gold',.005)

def make_character(kind):
 start(kind)
 female=kind in ['trainer_female','teacher','lass']
 outdoor=kind=='outdoorsman'; elder=kind=='elder'; scientist=kind=='scientist'
 rival=kind=='rival'; teacher=kind=='teacher'; lass=kind=='lass'
 skin='skin'; hair='teal' if kind=='trainer_female' else 'red_dark' if rival else 'silver' if elder or scientist else 'hair'
 shirt='mint' if teacher else 'teal' if lass else 'white' if scientist else 'olive' if outdoor else 'tan' if elder else 'navy' if kind.startswith('trainer') else 'purple' if rival else 'red'
 pants='red_dark' if female else 'denim' if kind=='youngster' else 'ink' if rival else 'navy'
 part('pelvis')
 rings('Tailored hips',[(.58,.146,.102),(.64,.165,.115),(.74,.169,.117)],pants)
 part('torso')
 rings('Fitted jacket',[(.697,.171,.12),(.72,.18,.127),(.93,.183,.13),(1.022,.170,.116),(1.066,.098,.075)],shirt)
 if teacher or lass:
  rings('Pleated skirt',[(.43,.234,.150),(.447,.240,.155),(.70,.164,.12),(.72,.162,.118)],pants)
  for x in [-.14,-.07,0,.07,.14]:
   rod('Skirt pleat',(x*.78,-.128,.65),(x,-.153,.457),.005,.006,'red')
 if scientist:
  box('Laboratory shirt',(0,-.126,1.004),(.131,.019,.116),'teal',.008)
  for s in [-1,1]:
   panel('Folded lapel',[(s*.024,-.148,1.065),(s*.14,-.13,1.014),(s*.071,-.158,.922)],[(0,1,2)],'cream')
   box('Coat pocket',(s*.115,-.130,.824),(.087,.02,.092),'cream',.012)
  box('Pocket pen',(.115,-.146,.879),(.012,.014,.061),'teal',.003)
 else:
  box('Jacket center placket',(0,-.130,.887),(.024,.018,.266),'cream' if kind.startswith('trainer') else pants,.006)
  for s in [-1,1]:
   box('Jacket welt pocket',(s*.11,-.132,.784),(.071,.014,.018),pants,.006)
  if kind.startswith('trainer'):
   rings('Gold jacket hem',[(.698,.174,.124),(.748,.183,.129)],'gold')
   box('Gold chest stripe',(0,-.137,.966),(.326,.017,.058),'gold',.012)
  if outdoor:
   for s in [-1,1]:box('Vest cargo pocket',(s*.112,-.143,.867),(.113,.032,.119),'khaki',.015)
  if teacher or elder:
   for z in [.805,.872,.939]:orb('Cardigan button',(0,-.147,z),(.014,.009,.014),'gold',10,6)
 # Collar is modeled around the neck, not painted onto the face.
 for s in [-1,1]:
  o=box('Folded collar',(s*.060,-.091,1.042),(.084,.075,.032),'cream',.010);o.rotation_euler.y=s*.23
 for s,suffix in [(-1,'l'),(1,'r')]:
  part('upper_arm_'+suffix)
  orb('Rounded shoulder',(s*.208,0,.997),(.086,.100,.093),shirt)
  rod('Sleeve',(s*.232,-.003,.983),(s*.242,-.008,.805),.072,.073,shirt)
  orb('Cloth elbow',(s*.242,-.008,.817),(.072,.072,.063),shirt)
  part('forearm_'+suffix)
  rod('Forearm sleeve',(s*.242,-.008,.808),(s*.248,-.018,.683),.055,.067,shirt)
  box('Jacket cuff',(s*.248,-.018,.697),(.115,.12,.042),'gold' if kind.startswith('trainer') else shirt,.013)
  part('hand_'+suffix)
  orb('Mitten hand',(s*.248,-.022,.638),(.058,.061,.069),skin)
  orb('Curled thumb',(s*.208,-.055,.650),(.024,.027,.039),skin,12,8)
  part('thigh_'+suffix)
  rod('Upper trouser leg',(s*.105,0,.666),(s*.105,0,.39),.070,.083,pants)
  orb('Knee joint',(s*.105,0,.39),(.069,.070,.068),pants)
  part('shin_'+suffix)
  rod('Lower trouser leg',(s*.105,0,.398),(s*.105,0,.12),.058,.069,skin if teacher or lass else pants)
  box('Sock cuff',(s*.105,0,.177),(.12,.124,.067),'sock' if female else pants,.014)
  part('shoe_'+suffix)
  box('Rubber sole',(s*.105,-.054,.030),(.168,.270,.056),'sole',.024)
  orb('Rounded walking shoe',(s*.105,-.04,.077),(.083,.135,.065),'brown' if teacher or elder else 'ink')
  box('Shoe tongue',(s*.105,-.065,.124),(.091,.097,.023),'cream',.008)
  for y in [-.044,-.074]:box('Laces',(s*.105,y,.138),(.082,.012,.009),'white',.003)
 face(hair,female,elder,scientist or elder)
 if kind.startswith('trainer'):
  cap('white' if kind=='trainer_female' else 'navy','red' if kind=='trainer_female' else 'gold');backpack()
 elif kind=='youngster':cap('red','cream')
 elif outdoor:
  part('head');rings('Hiking hat crown',[(1.433,.21,.183),(1.51,.185,.16),(1.55,.140,.13)],'khaki')
  orb('Broad hiking brim',(0,0,1.431),(.283,.241,.024),'tan',24,8)
  box('Hat ribbon',(0,-.182,1.468),(.276,.016,.039),'olive',.012)
 elif teacher:
  part('head');orb('Twisted hair bun',(0,.120,1.50),(.091,.081,.066),hair)
 elif rival:
  part('head')
  for x,z in [(-.11,1.45),(-.03,1.50),(.06,1.48)]:
   rod('Swept pointed hair',(x,.02,z),(x-.04,.07,z+.082),.058,.005,hair)
 return GROUP, dict(PARTS)

def convert(v):return [round(v.x,6),round(v.z,6),round(-v.y,6)]

def export_rig(name,parts):
 bpy.context.view_layer.update()
 joints=[];triangles=0
 for index,(joint,parent,pivot) in enumerate(JOINTS):
  p=Vector(pivot);parent_p=PIVOTS[parent] if parent else Vector()
  primitives=[]
  for obj in parts[joint]:
   me=obj.data;me.calc_loop_triangles();positions=[];normals=[];indices=[];mapping={}
   normal_matrix=obj.matrix_world.to_3x3().inverted().transposed()
   for tri in me.loop_triangles:
    for loop_index in tri.loops:
     vertex=me.vertices[me.loops[loop_index].vertex_index]
     pos=convert(obj.matrix_world@vertex.co-p)
     normal=convert((normal_matrix@me.corner_normals[loop_index].vector).normalized())
     key=tuple(pos+normal)
     if key not in mapping:
      mapping[key]=len(positions)//3;positions.extend(pos);normals.extend(normal)
     indices.append(mapping[key])
   color=[round(c,6) for c in obj.data.materials[0].diffuse_color]
   primitives.append({'positions':positions,'normals':normals,'indices':indices,'base_color':color})
   triangles+=len(indices)//3
  joints.append({'name':joint,'parent':next((i for i,j in enumerate(JOINTS) if j[0]==parent),None),'translation':convert(p-parent_p),'primitives':primitives})
 data={'name':name,'version':1,'coordinate_system':'+Y up; front +Z; floor-centered root','joints':joints}
 (OUT/(name+'.rig.json')).write_text(json.dumps(data,separators=(',',':')))
 print(f'{name}: {len(joints)} joints, {triangles} triangles')

CHARACTERS=['trainer','trainer_female','rival','youngster','teacher','lass','scientist','outdoorsman','elder']
collections={}; source_parts={}
for name in CHARACTERS:
 col,parts=make_character(name);export_rig(name,parts);collections[name]=col;source_parts[name]=parts

# Editable source includes actual joint empties and parented local geometry.
# The studio pose is deliberately kept neutral for comparing silhouettes.
for idx,name in enumerate(CHARACTERS):
 col=collections[name]; roots={}
 for joint,parent,pivot in JOINTS:
  empty=bpy.data.objects.new(name+' / '+joint,None);col.objects.link(empty);empty.empty_display_size=.04
  empty.location=Vector(pivot);roots[joint]=empty
  if parent:empty.parent=roots[parent];empty.location-=PIVOTS[parent]
  else:empty.location.x+=(idx-4)*.93
  for obj in source_parts[name][joint]:
   obj.parent=empty
   obj.location-=Vector(pivot)
   obj['rig_joint']=joint

# Studio floor and broad soft lighting make geometry and colors easy to inspect.
bpy.ops.mesh.primitive_plane_add(size=200);floor=bpy.context.object;floor.location.z=-.013
floor.data.materials.append(material('studio',(.76,.82,.80)))
world=bpy.data.worlds.new('Soft overcast studio');scene.world=world;world.use_nodes=True
world.node_tree.nodes['Background'].inputs['Color'].default_value=(.25,.32,.40,1)
world.node_tree.nodes['Background'].inputs['Strength'].default_value=.6
for name,p,energy,size in [('Key',(-3,-4,7),900,5),('Fill',(5,-2,4),500,4),('Rim',(0,4,6),1100,4)]:
 bpy.ops.object.light_add(type='AREA',location=p);o=bpy.context.object;o.name=name;o.data.energy=energy;o.data.shape='DISK';o.data.size=size
 o.rotation_euler=(Vector((0,0,.8))-o.location).to_track_quat('-Z','Y').to_euler()
bpy.ops.object.camera_add(location=(3,-11,6));camera=bpy.context.object
camera.rotation_euler=(Vector((0,0,.83))-camera.location).to_track_quat('-Z','Y').to_euler()
camera.data.type='ORTHO';camera.data.ortho_scale=9.2;scene.camera=camera
scene.render.engine='CYCLES';scene.cycles.samples=48;scene.cycles.use_denoising=False
scene.render.resolution_x=2200;scene.render.resolution_y=700;scene.render.resolution_percentage=100
scene.view_settings.view_transform='AgX';scene.render.image_settings.file_format='PNG';scene.render.filepath=str(OUT/'character-lineup.png')
scene['authorship']='Original modeled geometry, no sprites, imported models, or extracted art.'
bpy.ops.wm.save_as_mainfile(filepath=str(OUT/'johto-character-kit.blend'))
if '--skip-preview' not in args:bpy.ops.render.render(write_still=True)
