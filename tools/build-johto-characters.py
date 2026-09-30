"""Original articulated Johto character kit, authored entirely as geometry.

blender -b --python tools/build-johto-characters.py -- target/johto-character-kit
Use --skip-preview to export without the studio character lineup.

The checked-in runtime JSON preserves rigid joint pivots and smooth corner
normals. It is not a flattened animation: every limb has its own mesh, parent,
and bind transform. Editable .blend sources are generated in bounded eight-family batches; previews remain development artifacts.
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
 'pink': (.90,.42,.57), 'rose': (.63,.23,.38), 'orange': (.96,.49,.19),
 'blue': (.28,.42,.73), 'cyan': (.32,.69,.84), 'violet': (.55,.38,.70),
 'leaf': (.37,.61,.29), 'sand': (.80,.73,.56), 'dark_teal': (.11,.35,.37),
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

# Each source family is a deliberate art direction, never a fallback alias.
# kind, outfit, shirt, lower, hair, hairstyle, hat, accessories, body width
# Colors are original 3D art direction, not extracted source palette pixels.
DESIGNS = {
 'trainer': ('jacket','navy','navy','hair','short','trainer', 'backpack',1.00),
 'trainer_female': ('jacket','navy','red_dark','teal','pigtails','kris','backpack',.94),
 'rival': ('jacket','purple','ink','red_dark','spikes','none','',.98),
 'youngster': ('tee','red','denim','hair','short','redcap','shorts',.88),
 'teacher': ('cardigan','mint','red_dark','hair','bun','none','skirt',1.02),
 'lass': ('polo','teal','red_dark','brown','ponytail','none','skirt,schoolbag',.90),
 'scientist': ('coat','white','navy','silver','short','none','glasses,pen',1.00),
 'outdoorsman': ('vest','olive','khaki','brown','short','hiking','backpack,beard',1.18),
 'elder': ('robe','tan','brown','silver','bald','none','beard,cane',.96),
 'beauty': ('dress','purple','purple','brown','long','none','necklace,earrings',.91),
 'biker': ('vest','ink','denim','hair','sidecut','none','stubble,chain,gloves',1.22),
 'bill': ('polo','teal','khaki','brown','curls','none','pen',.95),
 'black_belt': ('martial','white','white','hair','spikes','none','headband,blackbelt',1.08),
 'blaine': ('coat','white','red_dark','silver','bald','none','glasses,moustache,tie',1.08),
 'blue': ('tee','purple','navy','brown','spikes','none','pendant',1.01),
 'brock': ('vest','olive','brown','brown','spikes','none','hikingpack',1.08),
 'bruno': ('martial','skin','white','hair','topknot','none','blackbelt,wristwraps',1.25),
 'bug_catcher': ('tee','leaf','khaki','brown','short','straw','shorts,net',.88),
 'bugsy': ('polo','leaf','khaki','purple','bob','none','shorts,netbelt',.88),
 'cal': ('jacket','red','ink','brown','short','bluecap','backpack',1.00),
 'captain': ('suit','navy','navy','silver','short','captain','moustache,goldbuttons',1.18),
 'chuck': ('martial','skin','white','brown','short','none','beard,blackbelt',1.36),
 'clair': ('bodysuit','blue','blue','blue','long','none','cape,gloves,boots',.95),
 'clerk': ('apron','teal','navy','brown','short','none','namebadge',.96),
 'cooltrainer_f': ('jacket','blue','white','brown','ponytail','none','skirt,satchel',.94),
 'cooltrainer_m': ('jacket','red','navy','hair','spikes','none','wristband',1.04),
 'daisy': ('dress','mint','mint','tan','long','none','ribbon,necklace',.93),
 'elm': ('coat','white','brown','brown','spikes','none','glasses,tealtie,pen',1.04),
 'erika': ('kimono','rose','rose','hair','bob','none','flower,obi',.92),
 'falkner': ('kimono','blue','blue','navy','sidepart','none','obi',.98),
 'fisher': ('vest','olive','navy','brown','short','redcap','rod',1.17),
 'fishing_guru': ('vest','tan','brown','silver','short','straw','moustache,rod',1.25),
 'gameboy_kid': ('tee','blue','khaki','hair','short','none','shorts,handheld',.86),
 'gentleman': ('suit','purple','ink','silver','short','top','moustache,cane,bowtie',1.05),
 'gramps': ('cardigan','sand','brown','silver','bald','none','moustache,glasses',1.13),
 'granny': ('dress','violet','violet','silver','bun','none','shawl,glasses',1.15),
 'gym_guide': ('suit','navy','navy','hair','short','none','tie,clipboard',1.08),
 'janine': ('ninja','purple','purple','violet','highpony','none','scarf,gloves',.93),
 'jasmine': ('dress','white','white','brown','long','none','ribbon',.90),
 'karen': ('dress','ink','ink','silver','long','none','necklace,bracelet',.93),
 'kimono_girl': ('kimono','red','red_dark','hair','bun','none','obi,fan,hairpin',.98),
 'koga': ('ninja','navy','navy','hair','spikes','none','scarf,wristwraps',1.07),
 'kurt': ('kimono','navy','navy','silver','sparse','none','apronfront',1.13),
 'kurt_outside': ('kimono','navy','navy','silver','sparse','none','apronfront,apricornbasket',1.13),
 'lance': ('suit','ink','ink','red_dark','spikes','none','redcape,goldbuttons,boots',1.10),
 'link_receptionist': ('dress','pink','pink','brown','bob','reception','namebadge',.96),
 'misty': ('tee','gold','denim','orange','sidepony','none','shorts,suspenders',.91),
 'mom': ('dress','red_dark','red_dark','brown','bob','none','apronfront',1.09),
 'morty': ('tee','purple','sand','tan','sidepart','none','headband,scarf',1.00),
 'nurse': ('dress','white','white','pink','rolls','nurse','apronfront,medallion',1.00),
 'oak': ('coat','cream','tan','silver','spikes','none','redshirt,pen',1.10),
 'officer': ('suit','navy','navy','hair','short','police','badge,dutybelt',1.13),
 'old_link_receptionist': ('dress','mint','mint','hair','highpony','none','namebadge,headset',.94),
 'pharmacist': ('robe','white','white','silver','bald','none','glasses,moustache,medicinebag',1.02),
 'pokefan_f': ('dress','pink','rose','brown','bun','none','apronfront,heartbadge',1.31),
 'pokefan_m': ('polo','teal','brown','brown','short','none','moustache,roundglasses,ballbadge',1.36),
 'pryce': ('cardigan','cyan','purple','silver','sparse','none','cane,scarf',.98),
 'receptionist': ('suit','blue','navy','brown','bob','none','skirt,namebadge,tie',.94),
 'red': ('vest','red','denim','hair','short','redcap','backpack,whiteshirt',1.00),
 'reds_mom': ('dress','mint','mint','brown','bun','none','apronfront,necklace',1.07),
 'rocker': ('tee','ink','denim','orange','mohawk','none','guitar,studdedbelt',1.02),
 'rocket': ('rocket','ink','ink','hair','short','rocket','redr,gloves,boots',1.06),
 'rocket_girl': ('rocket','ink','ink','rose','long','rocket','redr,gloves,boots,skirt',.94),
 'sabrina': ('bodysuit','rose','rose','hair','long','none','shoulderguards,wristband',.94),
 'sage': ('robe','sand','sand','silver','bald','none','beads',1.09),
 'sailor': ('sailor','white','navy','hair','short','sailor','neckerchief,anchor',1.20),
 'standing_youngster': ('tee','red','denim','hair','short','none','shorts,ballheld',.88),
 'super_nerd': ('polo','cream','navy','hair','sidepart','none','roundglasses,suspenders',1.14),
 'surge': ('vest','olive','olive','tan','flattop','none','dogtags,combatboots',1.26),
 'swimmer_girl': ('swimsuit','pink','pink','brown','highpony','none','goggles',.94),
 'swimmer_guy': ('swimsuit','skin','blue','hair','short','none','goggles',1.07),
 'twin': ('dress','pink','pink','brown','pigtails','none','ribbon',.82),
 'unused_guy': ('suit','tan','brown','brown','sidepart','none','tealtie',1.04),
 'whitney': ('tee','white','blue','pink','pigtails','none','shorts,redstripe',.92),
 'will': ('suit','violet','violet','purple','sidepart','none','mask,whitegloves',1.00),
}
# This explicit public contract is generated into Rust below; model names differ
# from source names only for the two established player runtime filenames.
SOURCE_MODELS = {name:name for name in DESIGNS if name not in ('trainer','trainer_female','outdoorsman')}
SOURCE_MODELS.update(chris='trainer',kris='trainer_female')
assert len(SOURCE_MODELS)==74


def hairstyle(style,color):
 part('head')
 if style in ('bald','sparse'):
  # Remove crown/fringe of the common face. Sparse styles retain side/back hair.
  for obj in list(PARTS['head']):
   if any(x in obj.name for x in ('hair crown','Swept fringe')) or (style=='bald' and 'Side lock' in obj.name):
    PARTS['head'].remove(obj);bpy.data.objects.remove(obj,do_unlink=True)
 if style in ('long','bob'):
  depth=.25 if style=='long' else .13
  for side in [-1,1]:
   orb('Sculpted side curtain',(side*.165,.040,1.25-depth*.30),(.061,.109,depth),color)
  orb('Layered back hair',(0,.135,1.24),(.182,.087,depth),color)
  for side in [-1,1]:rod('Tapered long lock',(side*.179,.075,1.27),(side*.159,.07,1.02 if style=='long' else 1.13),.050,.018,color)
 if style in ('bun','topknot'):
  orb('Gathered hair bun',(0,.095,1.495 if style=='bun' else 1.56),(.087,.081,.067),color)
 if style in ('ponytail','highpony','sidepony'):
  side=.20 if style=='sidepony' else 0
  orb('Tied ponytail root',(side,.125,1.44),(.072,.068,.073),color)
  o=orb('Tapered ponytail',(side,.185,1.32),(.069,.072,.145),color);o.rotation_euler.x=-.32
  orb('Ponytail tie',(side,.156,1.445),(.074,.025,.026),'red')
 if style in ('pigtails','rolls'):
  for side in [-1,1]:
   o=orb('Twin hair roll' if style=='rolls' else 'Twin ponytail',(side*.185,.06,1.30),(.079,.070,.095 if style=='rolls' else .143),color)
   orb('Twin ribbon knot',(side*.18,.056,1.41),(.043,.047,.026),'cream')
 if style in ('spikes','flattop','mohawk'):
  for x in [-.13,-.066,0,.066,.13] if style!='mohawk' else [0]:
   for y in ([0,.09] if style=='mohawk' else [0]):
    rod('Sculpted pointed hair',(x,y,1.46),(x-.032,y+.025,1.58 if style!='flattop' else 1.54),.060,.009,color)
 if style=='curls':
  for x in [-.15,-.08,0,.08,.15]:orb('Rounded hair curl',(x,-.125,1.421),(.049,.050,.047),color)
 if style=='sidepart':
  o=orb('Swept side-part fringe',(-.06,-.130,1.437),(.140,.050,.052),color);o.rotation_euler.y=-.28
 if style=='sidecut':
  rings('Close cropped side hair',[(1.35,.187,.167),(1.44,.175,.159),(1.465,.11,.105)],color)


def hat(kind):
 part('head')
 if kind in ('trainer','kris','redcap','bluecap','rocket'):
  cap(*{'trainer':('navy','gold'),'kris':('white','red'),'redcap':('red','white'),'bluecap':('blue','white'),'rocket':('ink','ink')}[kind],badge=kind!='rocket')
 elif kind in ('straw','hiking','top'):
  color='tan' if kind=='straw' else 'khaki' if kind=='hiking' else 'ink'
  rings('Structured hat crown',[(1.43,.201,.175),(1.50,.188,.162),(1.58 if kind=='top' else 1.54,.17,.15)],color)
  orb('Shaped brim',(0,0,1.438),(.283,.238,.023),color,24,8)
  rings('Hat ribbon',[(1.455,.202,.177),(1.492,.199,.174)],'brown' if kind!='top' else 'purple')
 elif kind in ('police','captain','sailor'):
  rings('Uniform cap',[(1.43,.200,.181),(1.50,.212,.187),(1.55,.173,.156)],'white' if kind!='police' else 'navy')
  rings('Uniform cap band',[(1.433,.204,.184),(1.466,.205,.184)],'navy')
  if kind!='sailor':orb('Uniform visor',(0,-.183,1.43),(.209,.11,.023),'ink')
  orb('Cap insignia',(0,-.185,1.503),(.038,.012,.041),'gold')
 elif kind in ('nurse','reception'):
  box('Folded service cap',(0,.012,1.510),(.297,.194,.114),'white',.036)
  box('Cap center stripe',(0,-.090,1.527),(.050,.019,.071),'red' if kind=='nurse' else 'pink',.005)
  if kind=='nurse':box('Cap cross',(0,-.102,1.527),(.093,.014,.022),'red',.004)


def accessory(name,width):
 part('torso')
 if name in ('backpack','hikingpack'):backpack()
 elif name in ('skirt','shorts','boots','combatboots','gloves','whitegloves','blackbelt'):pass # made with limbs/clothes
 elif name in ('glasses','roundglasses'):
  part('head')
  for side in [-1,1]:
   # Open frames have volume and no opaque lens over the eyes.
   for z in [1.248,1.352]:box('Spectacle frame',(side*.076,-.185,z),(.121,.012,.012),'ink',.005)
   for dx in [-.060,.060]:box('Spectacle temple',(side*.076+dx,-.184,1.30),(.012,.014,.103),'ink',.005)
  box('Spectacle bridge',(0,-.187,1.313),(.033,.012,.012),'ink',.003)
 elif name in ('moustache','beard','stubble'):
  part('head')
  for side in [-1,1]:orb('Sculpted moustache',(side*.035,-.169,1.234),(.053,.024,.019),'silver' if name=='moustache' else 'brown')
  if name!='moustache':orb('Sculpted jaw beard',(0,-.049,1.172),(.131,.117,.062 if name=='beard' else .034),'brown')
 elif name in ('tie','tealtie','bowtie'):
  color='teal' if name=='tealtie' else 'red_dark'
  if name=='bowtie':
   for side in [-1,1]:orb('Bow tie',(side*.045,-.135,1.025),(.039,.020,.023),color)
  else:
   rod('Tailored tie',(0,-.140,1.023),(0,-.145,.871),.025,.014,color)
   orb('Tie knot',(0,-.143,1.027),(.027,.021,.025),color)
 elif name in ('namebadge','badge','heartbadge','ballbadge','medallion'):
  x=.100*width
  if name=='namebadge':
   box('Service name badge',(x,-.145,.969),(.099,.022,.045),'cream',.008)
   box('Badge text line',(x,-.158,.970),(.064,.010,.009),'blue',.002)
  elif name in ('badge','medallion'):orb('Uniform metal badge',(x,-.147,.969),(.034,.020,.043),'gold')
  else:
   orb('Collector badge',(x,-.158,.967),(.041,.025,.040),'red')
   if name=='ballbadge':box('Ball badge band',(x,-.181,.961),(.073,.009,.012),'ink',.004)
 elif name in ('pen','redshirt','whiteshirt'):
  if name=='pen':box('Breast-pocket pen',(.11*width,-.150,.943),(.011,.014,.078),'blue',.003)
  else:box('Visible undershirt',(0,-.128,1.001),(.128,.024,.106),'red' if name=='redshirt' else 'white',.012)
 elif name in ('apronfront','shawl'):
  if name=='apronfront':
   rings('Apron waist tie',[(.738,.181*width,.133),(.777,.185*width,.135)],'cream')
   box('Rounded apron bib',(0,-.133,.930),(.198*width,.034,.203),'cream',.025)
   box('Apron front panel',(0,-.150,.678),(.266*width,.047,.333),'cream',.031)
   box('Apron pocket',(0,-.180,.697),(.118,.018,.071),'white',.012)
  else:
   rings('Soft shoulder shawl',[(.93,.20*width,.132),(1.04,.219*width,.142),(1.079,.105,.080)],'cream')
 elif name in ('obi','dutybelt','studdedbelt','netbelt'):
  color='gold' if name=='obi' else 'brown'
  rings('Wide woven belt',[(.723,.183*width,.132),(.803 if name=='obi' else .763,.185*width,.135)],color)
  box('Belt clasp',(0,-.149,.751),(.064,.024,.042),'gold',.008)
  if name=='obi':box('Obi bow',(0,.159,.773),(.271,.089,.115),'rose',.030)
  if name=='dutybelt':box('Duty pouch',(.20,.016,.737),(.076,.109,.14),'ink',.012)
 elif name in ('scarf','neckerchief'):
  rings('Wrapped scarf',[(1.037,.115,.092),(1.097,.100,.078)],'red' if name=='neckerchief' else 'gold')
  box('Scarf tail',(.068,-.141,.965),(.064,.030,.204),'red' if name=='neckerchief' else 'gold',.018)
 elif name in ('cape','redcape'):
  color='red_dark' if name=='redcape' else 'ink'
  # Closed, flared cape shell: a solid garment with side thickness.
  vs=[(-.21,.075,1.045),(.21,.075,1.045),(.32,.22,.39),(-.32,.22,.39),(-.20,.105,1.04),(.20,.105,1.04),(.30,.246,.40),(-.30,.246,.40)]
  panel('Tailored solid cape',vs,[(0,1,2,3),(7,6,5,4),(0,4,5,1),(1,5,6,2),(2,6,7,3),(3,7,4,0)],color)
  for side in [-1,1]:orb('Cape clasp',(side*.125,-.101,1.034),(.034,.019,.027),'gold')
 elif name in ('ribbon','flower','hairpin'):
  part('head')
  if name=='flower':
   for a in range(5):orb('Hair flower petal',(-.157+.038*math.cos(a*math.tau/5),-.068,1.445+.038*math.sin(a*math.tau/5)),(.030,.019,.030),'pink')
   orb('Hair flower center',(-.157,-.091,1.445),(.021,.015,.021),'gold')
  elif name=='hairpin':rod('Decorative hairpin',(-.14,.088,1.47),(.16,.088,1.54),.011,.011,'gold')
  else:
   for side in [-1,1]:orb('Ribbon bow',(side*.057,.174,1.411),(.056,.029,.034),'red')
 elif name in ('headband','goggles','mask','headset'):
  part('head')
  if name=='headband':
   rings('Woven headband',[(1.391,.194,.173),(1.421,.198,.175)],'red')
   for side in [-1,1]:rod('Headband tails',(side*.025,.170,1.402),(side*.04,.239,1.275),.017,.013,'red')
  elif name=='goggles':
   rings('Goggle strap',[(1.370,.192,.169),(1.395,.192,.169)],'blue')
   for side in [-1,1]:orb('Resting swim goggles',(side*.072,-.164,1.395),(.055,.028,.036),'cyan')
  elif name=='mask':
   for side in [-1,1]:box('Eye mask wing',(side*.120,-.169,1.318),(.045,.025,.090),'white',.020)
   box('Mask brow',(0,-.177,1.351),(.242,.026,.023),'white',.008)
  else:
   orb('Headset earpiece',(.211,.001,1.306),(.025,.05,.055),'ink')
   rod('Microphone boom',(.211,-.02,1.29),(.10,-.195,1.23),.008,.008,'ink')
 elif name in ('necklace','pendant','dogtags','beads','chain'):
  for a in range(9):
   x=(a-4)*.024;y=-.126-abs(x)*.1;z=1.01-.075*math.cos(x/.096*math.pi/2)
   orb('Beaded necklace',(x,y,z),(.014,.015,.017),'brown' if name=='beads' else 'gold',10,6)
  if name!='beads':box('Necklace pendant',(0,-.154,.918),(.042,.019,.055),'silver' if name=='dogtags' else 'gold',.009)
 elif name in ('earrings','bracelet','wristband','wristwraps','shoulderguards'):
  for side,suffix in [(-1,'l'),(1,'r')]:
   if name=='earrings':part('head');orb('Gold earring',(side*.202,-.024,1.246),(.020,.023,.028),'gold')
   elif name=='shoulderguards':part('upper_arm_'+suffix);orb('Round shoulder guard',(side*.217,0,1.017),(.097,.106,.052),'silver')
   else:part('forearm_'+suffix);box('Wrist wrap',(side*.248,-.018,.709),(.118,.12,.056),'white' if name=='wristwraps' else 'gold',.021)
 elif name in ('schoolbag','satchel','medicinebag','apricornbasket'):
  color='cream' if name=='medicinebag' else 'tan'
  box('Side satchel',(.236,.057,.732),(.139,.163,.224),color,.035)
  box('Satchel flap',(.234,.057,.83),(.156,.176,.063),'brown',.015)
  rod('Diagonal satchel strap',(-.14,-.12,1.012),(.17,-.154,.726),.014,.014,'tan')
 elif name in ('rod','net','cane'):
  part('hand_r')
  if name=='cane':
   rod('Walking cane',(.28,-.015,.687),(.31,-.035,.022),.017,.019,'brown')
   orb('Curved cane grip',(.264,-.027,.707),(.055,.025,.025),'brown')
  elif name=='rod':
   rod('Fishing rod grip',(.26,-.005,.602),(.31,.055,.898),.020,.014,'brown')
   rod('Flexible fishing rod',(.31,.055,.898),(.40,.12,1.59),.012,.004,'tan')
   rod('Fishing line',(.40,.12,1.59),(.45,.07,.92),.002,.002,'cream')
   orb('Fishing reel',(.29,.036,.762),(.038,.035,.041),'silver')
  else:
   rod('Catching-net pole',(.258,0,.623),(.32,.058,1.35),.017,.012,'tan')
   # Solid rim and sparse strings provide real 3D depth without a sprite plane.
   for a in range(16):
    t=a*math.tau/16;u=(a+1)*math.tau/16
    rod('Net hoop',(.32+.16*math.cos(t),.060,1.50+.16*math.sin(t)),(.32+.16*math.cos(u),.060,1.50+.16*math.sin(u)),.011,.011,'cream')
   for a in [-.08,0,.08]:
    rod('Net cord',(.32+a,.06,1.37),(.32+a,.09,1.63),.003,.003,'white')
    rod('Net cord',(.19,.06,1.50+a),(.45,.09,1.50+a),.003,.003,'white')
 elif name in ('handheld','clipboard','ballheld','fan'):
  part('hand_l')
  if name=='handheld':
   box('Handheld console',(-.250,-.076,.683),(.124,.077,.179),'cream',.014)
   box('Console screen',(-.250,-.119,.72),(.084,.012,.059),'dark_teal',.003)
   box('Console directional pad',(-.276,-.124,.651),(.027,.01,.011),'ink',.002)
   orb('Console button',(-.219,-.124,.649),(.009,.004,.009),'red',10,6)
  elif name=='ballheld':
   orb('Held collecting ball',(-.248,-.068,.688),(.057,.057,.057),'red')
   box('Ball band',(-.248,-.124,.688),(.087,.009,.013),'ink',.004)
  elif name=='clipboard':box('Guide clipboard',(-.26,-.084,.72),(.165,.031,.238),'tan',.012)
  else:
   for i in range(7):rod('Folding fan rib',(-.247,-.061,.669),(-.247+.14*math.cos(i*math.pi/6),-.073,.70+.14*math.sin(i*math.pi/6)),.010,.023,'cream')
 elif name=='guitar':
  orb('Guitar lower bout',(0,-.207,.756),(.108,.043,.111),'red_dark')
  orb('Guitar upper bout',(.04,-.205,.864),(.077,.041,.074),'red_dark')
  rod('Guitar neck',(.04,-.211,.861),(.26,-.217,1.139),.025,.025,'tan')
  orb('Guitar sound hole',(.025,-.252,.817),(.035,.007,.038),'ink')
 elif name=='redr':
  # Solid raised insignia, not a decal or imported texture.
  x=-.046;y=-.146
  box('R vertical',(x,y,.935),(.026,.018,.141),'red',.006)
  box('R top',(x+.035,y,.993),(.073,.018,.025),'red',.006)
  box('R crossbar',(x+.035,y,.932),(.070,.018,.025),'red',.006)
  box('R bowl',(x+.070,y,.963),(.025,.018,.076),'red',.006)
  rod('R diagonal',(x+.027,y,.931),(x+.084,y,.865),.012,.012,'red')
 elif name in ('goldbuttons','anchor','suspenders','redstripe'):
  if name=='goldbuttons':
   for side in [-1,1]:
    for z in [.79,.87,.95]:orb('Uniform button',(side*.080,-.147,z),(.013,.010,.013),'gold',10,6)
  elif name=='suspenders':
   for side in [-1,1]:box('Suspender strap',(side*.096,-.144,.905),(.022,.014,.258),'red',.005)
  elif name=='redstripe':box('Sport chest stripe',(0,-.145,.916),(.34,.018,.047),'red',.009)
  else:
   rod('Anchor stem',(0,-.149,.90),(0,-.149,.982),.009,.009,'blue')
   for side in [-1,1]:rod('Anchor arm',(0,-.149,.905),(side*.04,-.149,.942),.008,.008,'blue')
 else:raise ValueError('Unimplemented character feature: '+name)


def make_character(kind):
 start(kind)
 outfit,shirt,pants,hair,style,headwear,features,width=DESIGNS[kind]
 features=set(filter(None,features.split(',')))
 skirt='skirt' in features or outfit in ('dress','kimono','robe')
 coat=outfit=='coat';bare=outfit in ('swimsuit',) or shirt=='skin'
 part('pelvis')
 rings('Tailored hips',[(.58,.146*width,.102),(.64,.165*width,.115),(.74,.169*width,.117)],pants)
 part('torso')
 if outfit=='swimsuit' and shirt!='skin':
  rings('Tailored one-piece swimwear',[(.656,.154*width,.114),(.76,.147*width,.118),(.91,.170*width,.127),(1.023,.162*width,.114),(1.058,.098,.075)],shirt)
 else:
  rings('Tailored '+outfit,[(.692,.173*width,.122),(.72,.180*width,.130),(.87,.187*width,.139),(.964,.186*width,.132),(1.028,.170*width,.116),(1.066,.098,.075)],shirt)
 if skirt:
  hem=.285 if outfit in ('kimono','robe') else .414 if outfit=='dress' else .465
  rings('Rounded garment hem',[(hem,.237*width,.157),(hem+.026,.244*width,.163),(.67,.174*width,.126),(.73,.173*width,.123)],pants)
  for x in [-.14,-.07,0,.07,.14]:rod('Garment seam',(x*.76,-.134,.665),(x*width,-.162,hem+.033),.003,.004,shirt)
 if outfit in ('coat','suit','vest','jacket','cardigan'):
  if coat:
   rings('Long open coat',[(.481,.208*width,.145),(.519,.211*width,.148),(.76,.179*width,.129)],shirt)
   box('Visible coat opening',(0,-.150,.681),(.061,.026,.360),pants,.011)
  for side in [-1,1]:
   # Thick collar lapels; the former single triangle collar was a 2.5D remnant.
   ob=box('Tailored folded lapel',(side*.075,-.123,1.014),(.079,.040,.142),'cream' if outfit!='suit' else shirt,.012);ob.rotation_euler.y=side*.30
   box('Welt pocket',(side*.112*width,-.145,.812),(.090,.021,.021),pants,.007)
  box('Center placket',(0,-.146,.877),(.023,.020,.231),'cream' if outfit=='jacket' else pants,.006)
 if outfit in ('polo','sailor','tee') and outfit!='tee':
  for side in [-1,1]:
   ob=box('Folded shirt collar',(side*.063,-.100,1.042),(.091,.080,.028),'navy' if outfit=='sailor' else 'cream',.009);ob.rotation_euler.y=side*.20
 if outfit in ('kimono','robe','martial','ninja'):
  for side in [-1,1]:
   rod('Wrapped garment lapel',(side*.069,-.085,1.045),(-side*.04,-.157,.872),.022,.025,'cream' if outfit=='kimono' else pants)
  rings('Wrapped waist sash',[(.724,.187*width,.137),(.778,.187*width,.137)],'ink' if 'blackbelt' in features else 'tan')
 if kind in ('trainer','trainer_female'):
  rings('Gold jacket hem',[(.698,.178*width,.130),(.748,.183*width,.135)],'gold')
  box('Gold chest stripe',(0,-.150,.966),(.328*width,.018,.054),'gold',.011)
 if outfit=='apron':
  box('Store apron',(0,-.150,.842),(.288,.041,.37),'navy',.032)
 if outfit=='cardigan':
  for z in [.795,.87,.945]:orb('Cardigan button',(0,-.156,z),(.012,.009,.012),'gold',10,6)
 for side,suffix in [(-1,'l'),(1,'r')]:
  short=outfit in ('tee','polo','vest','sailor','swimsuit')
  sleeve_color='skin' if bare else shirt
  part('upper_arm_'+suffix)
  orb('Rounded shoulder',(side*.209,0,.997),(.087,.098,.089),sleeve_color)
  rod('Upper sleeve' if not bare else 'Upper arm',(side*.232,-.003,.983),(side*.242,-.008,.805),.069,.073,sleeve_color)
  orb('Elbow',(side*.242,-.008,.812),(.068,.068,.061),'skin' if short else shirt)
  part('forearm_'+suffix)
  rod('Forearm',(side*.242,-.008,.808),(side*.248,-.018,.683),.052,.062,'skin' if short or bare else shirt)
  if not short and not bare:box('Folded cuff',(side*.248,-.018,.697),(.112,.115,.043),'white' if outfit=='suit' else shirt,.015)
  part('hand_'+suffix)
  glove='white' if 'whitegloves' in features else 'ink' if 'gloves' in features else 'skin'
  orb('Mitten hand',(side*.248,-.022,.638),(.057,.058,.068),glove)
  orb('Curled thumb',(side*.208,-.055,.650),(.024,.027,.039),glove,12,8)
  part('thigh_'+suffix)
  upper_skin=outfit=='swimsuit'
  rod('Upper leg',(side*.105,0,.666),(side*.105,0,.39),.069,.081,'skin' if upper_skin else pants)
  orb('Knee',(side*.105,0,.39),(.067,.067,.067),'skin' if upper_skin or 'shorts' in features else pants)
  part('shin_'+suffix)
  shin_skin=(skirt and outfit not in ('kimono','robe')) or upper_skin or 'shorts' in features
  boot='boots' in features or 'combatboots' in features
  rod('Lower leg',(side*.105,0,.398),(side*.105,0,.12),.057,.067,'ink' if boot else 'skin' if shin_skin else pants)
  if boot:box('Boot upper cuff',(side*.105,0,.360),(.131,.137,.054),'ink',.014)
  elif outfit!='swimsuit':box('Sock cuff',(side*.105,0,.178),(.12,.122,.067),'white' if shin_skin else pants,.015)
  part('shoe_'+suffix)
  shoe='skin' if outfit in ('swimsuit','martial') else 'brown' if outfit in ('robe','kimono','cardigan') else 'ink'
  box('Walking sole',(side*.105,-.052,.030),(.166,.266,.056),shoe if shoe=='skin' else 'sole',.024)
  orb('Rounded toe',(side*.105,-.040,.077),(.082,.133,.065),shoe)
  if shoe!='skin':
   box('Shoe tongue',(side*.105,-.065,.125),(.089,.095,.022),'cream',.008)
   for y in [-.044,-.074]:box('Laces',(side*.105,y,.138),(.080,.011,.009),'white',.003)
 face(hair)
 hairstyle(style,hair)
 if headwear!='none':hat(headwear)
 for feature in sorted(features):accessory(feature,width)
 return GROUP,dict(PARTS)

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
 print(f'{name}: {len(joints)} joints, {triangles} triangles',flush=True)

# --only=a,b is useful for fast art iteration; the full catalog is the default.
only=next((a.split('=',1)[1].split(',') for a in args if a.startswith('--only=')),None)
REQUESTED=only or list(DESIGNS)
assert set(REQUESTED)<=set(DESIGNS), 'Unknown character design'
for batch_index,start_index in enumerate(range(0,len(REQUESTED),8)):
 bpy.ops.object.select_all(action='SELECT');bpy.ops.object.delete(use_global=False)
 for collection in list(bpy.data.collections):
  if collection!=bpy.context.collection and collection.name!='Collection':bpy.data.collections.remove(collection)
 bpy.data.orphans_purge(do_recursive=True)
 MATERIALS.clear()
 CHARACTERS=REQUESTED[start_index:start_index+8]
 collections={};source_parts={}
 for name in CHARACTERS:
  col,parts=make_character(name);export_rig(name,parts);collections[name]=col;source_parts[name]=parts

 # Editable meshes remain separate semantic objects, bound to actual joint empties.
 columns=min(10,len(CHARACTERS));rows=math.ceil(len(CHARACTERS)/columns)
 for idx,name in enumerate(CHARACTERS):
  col=collections[name];roots={}
  for joint,parent,pivot in JOINTS:
   empty=bpy.data.objects.new(name+' / '+joint,None);col.objects.link(empty);empty.empty_display_size=.04
   empty.location=Vector(pivot);roots[joint]=empty
   if parent:empty.parent=roots[parent];empty.location-=PIVOTS[parent]
   else:
    empty.location.x+=((idx%columns)-(columns-1)/2)*1.03
    empty.location.y+=(idx//columns)*1.15
   for obj in source_parts[name][joint]:
    obj.parent=empty;obj.location-=Vector(pivot);obj['rig_joint']=joint
  col['source_id']=next((source for source,model in SOURCE_MODELS.items() if model==name),'legacy_outdoorsman')
  col['design']=json.dumps(DESIGNS[name])

 bpy.ops.mesh.primitive_plane_add(size=200);floor=bpy.context.object;floor.location.z=-.013
 floor.data.materials.append(material('studio',(.76,.82,.80)))
 world=bpy.data.worlds.new('Soft overcast studio');scene.world=world;world.use_nodes=True
 world.node_tree.nodes['Background'].inputs['Color'].default_value=(.25,.32,.40,1)
 world.node_tree.nodes['Background'].inputs['Strength'].default_value=.7
 for name,p,energy,size in [('Key',(-5,-6,12),2100,8),('Fill',(8,-2,7),1300,7),('Rim',(0,8,11),2400,7)]:
  bpy.ops.object.light_add(type='AREA',location=p);o=bpy.context.object;o.name=name;o.data.energy=energy;o.data.shape='DISK';o.data.size=size
  o.rotation_euler=(Vector((0,rows*.45,.8))-o.location).to_track_quat('-Z','Y').to_euler()
 target=Vector((0,(rows-1)*.575,.73))
 bpy.ops.object.camera_add(location=target+Vector((3,-14,13)));camera=bpy.context.object
 camera.rotation_euler=(target-camera.location).to_track_quat('-Z','Y').to_euler()
 camera.data.type='ORTHO';camera.data.ortho_scale=max(columns*1.06,rows*1.35,6.8);scene.camera=camera
 scene.render.engine='CYCLES';scene.cycles.samples=16;scene.cycles.use_denoising=False
 scene.render.resolution_x=2200;scene.render.resolution_y=max(700,round(2200*rows/columns*.9));scene.render.resolution_percentage=100
 scene.view_settings.view_transform='AgX';scene.render.image_settings.file_format='PNG';scene.render.filepath=str(OUT/(f'character-lineup-{batch_index+1:02}.png'))
 scene['authorship']='Original modeled geometry; no sprites, imported models, or extracted art.'
 scene['catalog_contract']='74 source-specific human families; unknown source stays unmodeled. Shared 16-joint production gait.'
 bpy.ops.wm.save_as_mainfile(filepath=str(OUT/(f'johto-character-kit-{batch_index+1:02}.blend')),compress=False)
 if '--skip-preview' not in args:bpy.ops.render.render(write_still=True)

# Runtime packing is lossless: every source primitive and its material remains
# independently editable; only identical exported geometry arrays are shared.
sys.path.insert(0,str(Path(__file__).resolve().parent))
from johto_character_geometry import pack_directory
pack_directory(OUT,OUT)
