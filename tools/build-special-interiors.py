"""Original special-room kit, built only from geometric primitives.

blender -b --python tools/build-special-interiors.py -- target/special-interiors
The editable scene retains named objects and collections. Runtime JSON is Y-up,
front +Z. All source units are arbitrary; runtime fits exact verified source plots.
"""
import bpy, bmesh, math, json, sys
from pathlib import Path
from mathutils import Vector
args=sys.argv[sys.argv.index('--')+1:] if '--' in sys.argv else []
out=Path(next((a for a in args if not a.startswith('--')),'target/special-interiors'));out.mkdir(parents=True,exist_ok=True)
bpy.ops.object.select_all(action='SELECT');bpy.ops.object.delete(use_global=False)
scene=bpy.context.scene
PALETTE={'stone':(.43,.43,.36),'stone_light':(.63,.63,.52),'stone_dark':(.28,.30,.28),'sandstone':(.64,.60,.42),'sand_light':(.79,.74,.52),'dark_rock':(.29,.34,.37),'dark_light':(.42,.47,.48),'ice':(.29,.64,.76),'ice_light':(.66,.89,.93),'ice_dark':(.15,.40,.59),'wood':(.39,.24,.13),'wood_light':(.60,.39,.20),'wood_dark':(.21,.14,.11),'steel':(.30,.39,.41),'metal_light':(.53,.62,.59),'brass':(.75,.59,.29),'red':(.70,.22,.19),'cloth':(.73,.76,.66),'cloth_light':(.91,.91,.77),'blue':(.28,.42,.52),'glass':(.19,.49,.60),'ink':(.11,.16,.17),'leaf':(.24,.40,.22)}
M={}
for name,rgb in PALETTE.items():
 m=bpy.data.materials.new(name);m.diffuse_color=(*rgb,1);m.use_nodes=True;bs=m.node_tree.nodes.get('Principled BSDF');bs.inputs['Base Color'].default_value=(*rgb,1);bs.inputs['Roughness'].default_value=.87;M[name]=m
assets={};COL=None
# A coordinate is only a semantic color anchor into the verified local drawing.
# Runtime samples the current tile/palette; no game pixels are exported here.
SOURCE_PIXELS={
 'roof_binoculars':{'steel':[4,13],'metal_light':[3,11],'ink':[4,12],'cloth_light':[3,2],'blue':[9,1]},
 'facility_instrument_pair':{'steel':[1,0],'metal_light':[3,7],'ink':[2,1],'cloth_light':[4,2]},
 'facility_instrument_bank':{'steel':[1,8],'metal_light':[3,15],'ink':[2,9],'cloth_light':[4,10]},
 'link_round_stool':{'steel':[6,12],'ink':[6,15],'cloth_light':[7,7]},
}
def begin(name):
 global COL
 COL=bpy.data.collections.new(name);scene.collection.children.link(COL);assets[name]=COL

def assign(o,n,mat):
 o.name=n
 for c in list(o.users_collection):c.objects.unlink(o)
 COL.objects.link(o);o.data.materials.append(M[mat]);return o

def box(n,p,s,mat,bevel=.02):
 bpy.ops.mesh.primitive_cube_add(size=1,location=p);o=assign(bpy.context.object,n,mat);o.dimensions=s;bpy.ops.object.transform_apply(location=False,rotation=False,scale=True)
 if bevel:
  mod=o.modifiers.new('Hand-cut bevel','BEVEL');mod.width=min(bevel,min(s)*.25);mod.segments=1;bpy.ops.object.modifier_apply(modifier=mod.name)
 return o

def cone(n,p,r1,r2,h,mat,verts=8):
 bpy.ops.mesh.primitive_cone_add(vertices=verts,radius1=r1,radius2=r2,depth=h,location=p);return assign(bpy.context.object,n,mat)

def ico(n,p,s,mat,sub=1):
 bpy.ops.mesh.primitive_ico_sphere_add(subdivisions=sub,radius=1,location=p);o=assign(bpy.context.object,n,mat);o.scale=s;bpy.ops.object.transform_apply(location=False,rotation=False,scale=True);return o

def rod(n,a,b,r,mat,verts=6):
 a,b=Vector(a),Vector(b);o=cone(n,(a+b)*.5,r,r,(b-a).length,mat,verts);o.rotation_euler=(b-a).to_track_quat('Z','Y').to_euler();return o

def poly(n,verts,faces,mat):
 me=bpy.data.meshes.new(n);me.from_pydata(verts,[],faces);me.update()
 bm=bmesh.new();bm.from_mesh(me);bmesh.ops.recalc_face_normals(bm,faces=list(bm.faces));bm.to_mesh(me);bm.free()
 o=bpy.data.objects.new(n,me);COL.objects.link(o);me.materials.append(M[mat]);return o

def ring(n,p,outer,inner,h,mat,verts=12):
 x,y,z=p;vs=[]
 for zz,r in [(z-h/2,outer),(z+h/2,outer),(z-h/2,inner),(z+h/2,inner)]:
  vs.extend((x+r*math.cos(i*math.tau/verts),y+r*math.sin(i*math.tau/verts),zz) for i in range(verts))
 fs=[]
 for i in range(verts):
  j=(i+1)%verts;fs += [(i,j,j+verts,i+verts),(i+verts,j+verts,j+3*verts,i+3*verts),(i+2*verts,i+3*verts,j+3*verts,j+2*verts),(i,j,j+2*verts,i+2*verts)]
 return poly(n,vs,fs,mat)
# All forms below are original polygonal constructions, never source-art extrusions.
def bike(name,angle=0):
 begin(name)
 for x in [-.71,.71]:
  for mat,outer,inner in [('ink',.45,.355),('metal_light',.365,.329)]:
   o=ring('Wheel tire' if mat=='ink' else 'Polished wheel rim',(0,0,0),outer,inner,.065 if mat=='ink' else .077,mat,24);o.rotation_euler.x=math.pi/2;o.location=(x,0,.46)
  rod('Wheel axle',(x,-.12,.46),(x,.12,.46),.038,'steel',10)
  for i in range(12):
   a=i*math.tau/12;rod('Laced wheel spoke',(x,-.045,.46),(x+.334*math.cos(a),-.045,.46+.334*math.sin(a)),.008,'metal_light',4)
 pts={'rear':(-.71,0,.46),'crank':(-.12,0,.42),'seat':(-.29,0,1.08),'head':(.47,0,1.10),'front':(.71,0,.46)}
 for a,b in [('rear','crank'),('rear','seat'),('seat','crank'),('seat','head'),('head','crank')]:rod('Brazed diamond frame',pts[a],pts[b],.040,'red',8)
 for y in [-.075,.075]:
  rod('Tapered front fork',(.47,y,1.10),(.71,y,.46),.025,'metal_light',8)
  rod('Rear triangle stay',(-.29,y,1.08),(-.71,y,.46),.019,'red',6)
 rod('Adjustable seat post',(-.29,0,1.02),(-.35,0,1.24),.028,'metal_light',8)
 box('Padded leather saddle',(-.35,-.02,1.25),(.31,.20,.08),'ink',.035)
 rod('Handlebar riser',(.47,0,1.07),(.43,0,1.34),.029,'metal_light',8)
 rod('Swept bicycle handlebar',(.43,-.31,1.34),(.43,.31,1.34),.027,'metal_light',8)
 for y in [-.29,.29]:rod('Rubber handlebar grip',(.43,y,1.34),(.56,y,1.33),.043,'ink',8)
 o=ring('Chainring',(0,0,0),.16,.11,.04,'steel',12);o.rotation_euler.x=math.pi/2;o.location=(-.12,-.10,.42)
 for a,b in [((-.12,-.135,.56),(-.71,-.135,.50)),((-.12,-.135,.28),(-.71,-.135,.41))]:rod('Drive chain',a,b,.012,'steel',5)
 for sign in [-1,1]:
  rod('Pedal crank',(-.12,sign*.13,.42),(-.12+sign*.15,sign*.13,.42-sign*.10),.022,'metal_light',6)
  box('Platform pedal',(-.12+sign*.15,sign*.19,.42-sign*.10),(.16,.15,.035),'ink',.008)
 rod('Display support stand',(-.30,.16,.08),(-.30,.16,.45),.025,'steel',6)
 box('Display stand foot',(-.30,.16,.035),(.45,.32,.055),'steel',.015)
 if angle:
  for o in list(COL.objects):
   o.location.rotate(__import__('mathutils').Euler((0,0,angle)));o.rotation_euler.rotate_axis('Z',angle)
bike('bicycle_side')
bike('bicycle_front',math.pi/2)
bike('bicycle_diagonal',math.pi/4)

begin('bike_service_rack')
box('Workshop solid timber cabinet',(0,.09,.50),(2.15,.67,1.0),'wood_dark',.035)
box('Lipped worktop',(0,-.01,1.06),(2.30,.88,.12),'steel',.045)
for z in [.20,.47,.73]:
 box('Drawer face',(0,-.27,z),(2.02,.065,.21),'wood',.02)
 for x in [-.63,.12,.63]:box('Brass drawer pull',(x,-.32,z),(.20,.055,.045),'brass',.012)
for x in [-.75,-.35,.15,.60]:
 o=ring('Spare wheel stock',(0,0,0),.14,.092,.08,'ink',12);o.rotation_euler.x=math.pi/2;o.location=(x,-.04,1.22)
box('Pegboard rear panel',(0,.33,1.50),(2.18,.08,.83),'wood_light',.018)
for x in [-.83,-.5,-.15,.2,.57,.89]:
 rod('Hanging repair tool',(x,.269,1.30),(x,.269,1.70),.022,'steel',6)
 box('Tool grip',(x,.25,1.34),(.06,.045,.16),'red',.014)

# Bespoke mobile apparatus: a left CRT and a distinct right mode plate.
# Geometry is modeled from observed object structure, not extruded source pixels.
def sign_stroke(n,points,r,mat):
 for a,b in zip(points,points[1:]):rod(n,a,b,r,mat,6)
def arrow_plate(n,x,z,direction):
 # Reversible raised transfer arrows, built as closed polygonal solids.
 points=[(-.28,-.055),(.05,-.055),(.05,-.14),(.28,0),(.05,.14),(.05,.055),(-.28,.055)]
 points=[(x+u*direction,z+v) for u,v in points]
 vs=[(xx,y,zz) for y in [-.596,-.624] for xx,zz in points];nverts=len(points)
 fs=[tuple(range(nverts-1,-1,-1)),tuple(range(nverts,nverts*2))]
 fs += [(i,(i+1)%nverts,(i+1)%nverts+nverts,i+nverts) for i in range(nverts)]
 poly(n,vs,fs,'cloth_light')
for trade in [False,True]:
 begin('mobile_trade_terminal' if trade else 'mobile_battle_terminal')
 for x in [-1.22,1.22]:
  box('Full-height armored console upright',(x,.035,1.08),(.28,.64,2.16),'metal_light',.045)
  box('Continuous narrow inset channel',(x,-.304,1.09),(.135,.034,1.86),'steel',.012)
  for z in [.16,.37,.58,.79,1.09,1.48]:
   box('Raised upright collar',(x,-.34,z),(.235,.060,.10),'cloth',.013)
  for z in [.20,.32,.44,.56]:box('Pillar lower vent',(x,-.378,z),(.09,.016,.025),'glass',.004)
 box('Broad upper bridge casing',(0,.01,2.08),(2.73,.68,.56),'metal_light',.06)
 box('Recessed overhead black fascia',(0,-.350,2.09),(2.48,.052,.40),'steel',.024)
 for x in [-1.045,1.045]:
  o=ring('Circular upper receiver socket',(0,0,0),.225,.172,.057,'metal_light',16);o.rotation_euler.x=math.pi/2;o.location=(x,-.402,2.09)
  o=cone('Dark data socket recess',(0,0,0),.172,.172,.041,'ink',16);o.rotation_euler.x=math.pi/2;o.location=(x,-.405,2.09)
  sign_stroke('Diagonal socket conductor',[(x-.08,-.435,2.00),(x+.08,-.435,2.18)],.033,'brass')
 for z in [1.97,2.19]:box('Upper transfer conduit',(0,-.409,z),(1.41 if not trade else .91,.065,.054),'cloth_light',.012)
 if trade:
  for x in [-.66,.66]:
   box('Recessed vertical transfer status bank',(x,-.410,2.085),(.21,.052,.37),'ink',.009)
   for z in [1.96,2.05,2.14,2.23]:
    o=ring('Round transfer sequence lamp',(0,0,0),.039,.022,.015,'glass',8);o.rotation_euler.x=math.pi/2;o.location=(x,-.45,z)
 else:
  box('Battle bridge center spine',(0,-.416,2.083),(1.43,.053,.083),'blue',.014)
 box('Mode panel armored surround',(0,-.01,1.32),(2.11,.75,1.01),'steel',.05)
 box('Overhung sloped receiver eyebrow',(0,-.14,1.80),(2.28,.91,.17),'metal_light',.036)
 for x in [-.79,-.56,-.33,-.1,.13,.36,.59,.82]:
  box('Eyebrow louver',(x,-.612,1.795),(.125,.032,.045),'steel',.008)
 box('Left CRT heavy frame',(-.51,-.429,1.42),(.81,.24,.70),'metal_light',.035)
 box('Left CRT recessed shadow',(-.51,-.563,1.44),(.64,.03,.52),'ink',.025)
 box('Left CRT curved display',(-.51,-.582,1.44),(.52,.021,.41),'glass',.022)
 box('Display corner reflection',(-.668,-.598,1.573),(.11,.013,.033),'ice_light',.005)
 box('Display corner reflection',(-.707,-.598,1.521),(.033,.013,.105),'ice_light',.005)
 box('Right mode plate rim',(.46,-.444,1.47),(.85,.25,.71),'metal_light',.033)
 box('Recessed mode plate field',(.46,-.583,1.49),(.71,.027,.55),'blue' if trade else 'ink',.015)
 if trade:
  arrow_plate('Upper leftward transfer arrow',.46,1.64,-1)
  arrow_plate('Lower rightward transfer arrow',.46,1.37,1)
 else:
  sign_stroke('Raised V match identifier',[(.17,-.620,1.68),(.24,-.620,1.47),(.37,-.620,1.68)],.021,'cloth_light')
  sign_stroke('Raised S match identifier',[(.69,-.620,1.65),(.60,-.620,1.70),(.49,-.620,1.63),(.58,-.620,1.58),(.63,-.620,1.54),(.54,-.620,1.47),(.45,-.620,1.50)],.020,'cloth_light')
  sign_stroke('Battle wave indicator',[(.16,-.620,1.31),(.28,-.620,1.28),(.39,-.620,1.36),(.49,-.620,1.39),(.62,-.620,1.31),(.75,-.620,1.31)],.019,'glass')
 # The source has one wide operator shelf and an open lower footwell.
 box('Recessed horizontal key shelf',(0,-.48,.995),(1.84,.51,.115),'metal_light',.026)
 for i in range(10):
  box('Physical operator key',(-.72+i*.16,-.61,1.065),(.09,.083,.026),'red' if i in [0,9] else 'cloth_light',.006)
 for x in [-.83,.83]:
  box('Square side diagnostic recess',(x,-.426,.82),(.33,.11,.31),'ink',.018)
  o=cone('Control rotary dial',(0,0,0),.091,.091,.048,'steel',12);o.rotation_euler.x=math.pi/2;o.location=(x,-.503,.82)
  box('Narrow outer instrument leg',(x,.03,.37),(.235,.63,.74),'metal_light',.026)
  for z in [.15,.27,.39,.51]:box('Instrument leg slot',(x,-.310,z),(.107,.019,.035),'steel',.004)
 box('Central offset equipment pedestal',(.29,.04,.39),(.35,.62,.78),'steel',.026)
 box('Pedestal inset channel',(.29,-.286,.38),(.18,.025,.61),'ink',.012)
 if trade:
  for z in [.21,.34,.47,.60]:box('Transfer pulse indicator',(.29,-.307,z),(.10,.018,.035),'glass',.004)
 box('Low rear connecting beam',(0,.265,.11),(2.31,.17,.20),'metal_light',.016)

begin('link_battle_console')
box('Shared battle apparatus plinth',(0,0,.09),(1.90,1.14,.18),'metal_light',.045)
box('Dark recessed plinth band',(0,-.06,.20),(1.78,1.0,.10),'steel',.012)
# Twin tall swept receivers create a physical central opening.
for sign in [-1,1]:
 x=sign*.25
 verts=[(x-sign*.14,-.43,.22),(x+sign*.17,-.43,.22),(x+sign*.09,.25,1.32),(x-sign*.065,.25,1.45),(x-sign*.14,.43,.22),(x+sign*.17,.43,.22)]
 poly('Swept receiver buttress',verts,[(0,1,2,3),(1,5,2),(5,4,3,2),(4,0,3),(0,4,5,1)],'cloth_light')
 rod('Curved receiver crest',(x,.23,1.32),(sign*.12,.30,1.44),.075,'metal_light',10)
box('Deep central cradle', (0,.13,.75),(.15,.43,.93),'ink',.012)
box('Front receiver inset ramp',(0,-.22,.44),(.36,.34,.42),'steel',.018)
for x in [-.73,.73]:
 box('Side control wing',(x,-.04,.48),(.43,.70,.48),'cloth',.04)
 box('Wing dark cooling recess',(x,-.402,.47),(.29,.023,.35),'steel',.009)
 for z in [.36,.48,.60]:box('Raised horizontal receiver vent',(x,-.425,z),(.29,.033,.035),'metal_light',.006)
 for y in [-.24,-.07,.10]:box('Wing top status segment',(x,y,.727),(.24,.055,.018),'glass',.005)

begin('link_trade_console')
box('Link console base',(0,0,.13),(1.92,.85,.26),'steel',.05)
for x in [-.51,.51]:
 box('Link console pedestal',(x,0,.55),(.77,.67,.84),'cloth',.05)
 for j in range(5):box('Pedestal cooling slot',(x-.23+j*.115,-.351,.45),(.045,.02,.42),'steel',.004)
 box('Compact monitor bezel',(x,-.055,1.08),(.72,.59,.26),'metal_light',.045)
 box('Glass status surface',(x,-.37,1.10),(.55,.03,.14),'glass',.015)
 box('Status light',(x+.20,-.389,1.10),(.06,.009,.04),'red',.006)
rod('Shared data cable',(-.13,.12,.32),(.13,.12,.32),.07,'ink',8)

begin('shrine_dragon_rail')
box('Carved rail lower timber sill',(0,0,.10),(.59,5.40,.20),'wood_dark',.025)
for y in [-2.48,-1.24,0,1.24,2.48]:
 box('Dark carved rail supporting post',(-.18,y,.91),(.20,.24,1.82),'wood_dark',.018)
 box('Supporting post cap',(-.18,y,1.80),(.35,.37,.13),'wood',.023)
box('Slim upper side beam',(-.14,0,1.68),(.24,5.38,.15),'wood_dark',.019)
# Thick serpentine body is visibly carved, with raised scales and a dorsal crest.
prev=None
for i in range(65):
 y=-2.45+i*4.9/64;z=.95+.51*math.sin(i*math.tau/21.333);p=(.04,y,z)
 if prev:rod('Sweeping carved dragon body',prev,p,.145,'wood_light',10)
 if i%3==0:
  ico('Raised overlapping dragon scale',(.178,y,z+.023),(.043,.104,.096),'wood',1)
 if i%4==0:
  crest=(.025,y,z+.17)
  poly('Serrated dragon dorsal crest',[(crest[0]-.05,crest[1]-.1,crest[2]-.065),(crest[0]-.05,crest[1]+.1,crest[2]-.065),(crest[0]-.05,crest[1]+.035,crest[2]+.15),(crest[0]+.05,crest[1]-.1,crest[2]-.065),(crest[0]+.05,crest[1]+.1,crest[2]-.065),(crest[0]+.05,crest[1]+.035,crest[2]+.15)],[(0,1,2),(5,4,3),(0,3,4,1),(1,4,5,2),(2,5,3,0)],'wood')
 prev=p
# Faces turn out across the rail, so the sculpted eyes and jaws read in play.
for y in [-2.40,2.40]:
 ico('Outward-facing dragon head',(.065,y,1.12),(.28,.29,.30),'wood_light',2)
 ico('Long angular dragon muzzle',(.30,y,1.045),(.23,.19,.14),'wood',1)
 box('Open sculpted dragon mouth',(.40,y,1.00),(.16,.25,.075),'wood_dark',.018)
 for sign in [-1,1]:
  ico('Outward dragon bright eye',(.282,y+sign*.17,1.21),(.028,.052,.040),'cloth_light',1)
  ico('Outward dragon dark pupil',(.306,y+sign*.172,1.217),(.016,.023,.023),'ink',1)
  rod('Rail dragon tapered horn',(.02,y+sign*.18,1.30),(-.075,y+sign*.32,1.60),.055,'brass',7)
  rod('Rail dragon ivory tusk',(.40,y+sign*.080,1.02),(.41,y+sign*.093,.89),.023,'cloth_light',6)
  rod('Trailing dragon whisker',(.38,y+sign*.13,1.08),(.32,y+sign*.40,1.00),.018,'wood_light',6)


begin('shrine_dragon_mask')
# A long-horned dragon face is the entire object; neighboring drapery stays architectural.
box('Shaped dark mask mounting plaque',(0,.17,.96),(.86,.17,1.90),'wood_dark',.04)
ico('Long carved dragon face',(0,-.005,1.08),(.42,.26,.69),'wood_light',2)
ico('Angular dragon forehead ridge',(0,-.19,1.42),(.25,.13,.36),'wood',1)
for sign in [-1,1]:
 ico('Winged dragon cheek',(sign*.27,-.15,.97),(.20,.18,.26),'wood',1)
 rod('Long swept mask horn',(sign*.26,.05,1.46),(sign*.42,.09,2.04),.085,'brass',8)
 rod('Horn tapered upper tip',(sign*.42,.09,2.04),(sign*.47,.12,2.21),.040,'brass',7)
 brow=box('Heavy angled brow',(sign*.19,-.282,1.32),(.33,.125,.105),'wood_dark',.018);brow.rotation_euler.y=sign*-.30
 ico('Pale recessed dragon eye',(sign*.185,-.330,1.235),(.080,.045,.064),'cloth_light',1)
 ico('Dark narrow vertical pupil',(sign*.185,-.373,1.238),(.017,.012,.041),'ink',1)
 ico('Flared dragon nostril',(sign*.13,-.362,.92),(.09,.071,.057),'wood_dark',1)
 rod('Upper dragon fang',(sign*.16,-.365,.75),(sign*.12,-.399,.55),.046,'cloth_light',7)
 rod('Lower dragon tooth',(sign*.08,-.354,.43),(sign*.06,-.396,.54),.032,'cloth_light',7)
ico('Broad projecting dragon muzzle',(0,-.255,.86),(.29,.19,.18),'wood_light',1)
box('Open dark dragon mouth',(0,-.285,.61),(.31,.145,.25),'ink',.045)
ico('Underhung dragon jaw',(0,-.23,.42),(.22,.18,.13),'wood',1)
poly('Long pointed dragon beard',[(-.20,-.09,.44),(.20,-.09,.44),(0,-.12,.14),(-.20,.075,.44),(.20,.075,.44),(0,.035,.14)],[(0,1,2),(5,4,3),(0,3,4,1),(1,4,5,2),(2,5,3,0)],'wood_dark')

begin('shrine_paper_lantern')
box('Lantern timber foot',(0,0,.06),(.57,.56,.12),'wood_dark',.028)
rod('Lantern suspension stem',(0,0,.08),(0,0,1.39),.035,'wood',8)
for z in [.79,1.35]:cone('Lantern wooden end cap',(0,0,z),.40,.40,.09,'wood_dark',16)
# Many lathed bands produce a rounded paper lantern with physical ribs.
for i in range(9):
 z=.84+i*.056;r=.38+.07*math.sin(i*math.pi/8)
 cone('Opaque ivory paper shade',(0,0,z),r,r,.058,'cloth_light',16)
 if i%2==0:ring('Paper lantern raised hoop',(0,0,z),r+.006,r-.018,.009,'cloth',16)
rod('Lantern finial',(0,0,1.37),(0,0,1.57),.03,'brass',8)

begin('tower_emblem_panel')
box('Tower panel closed casing',(0,0,.62),(1.0,.20,1.24),'steel',.04)
box('Chamfered silver surround',(0,-.08,.64),(.94,.20,1.08),'metal_light',.06)
box('Recessed emblem field',(0,-.19,.64),(.76,.055,.83),'blue',.05)
for sign in [-1,1]:
 rod('Crossed geometric insignia',(-.24,-.231,.64+sign*.24),(.24,-.231,.64-sign*.24),.064,'brass',6)
ico('Insignia center boss',(0,-.251,.64),(.14,.043,.14),'metal_light',1)

begin('tower_round_fixture')
cone('Arena fixture foot',(0,0,.08),.48,.44,.16,'steel',16)
cone('Rounded arena end pedestal',(0,0,.36),.44,.36,.50,'metal_light',16)
ring('Dark arena end ring',(0,0,.63),.38,.28,.11,'steel',20)
ico('Illuminated inset end cap',(0,0,.65),(.27,.27,.13),'glass',2)
for x,y in [(-.41,-.41),(-.41,.41),(.41,-.41),(.41,.41)]:
 cone('Four pedestal bolt feet',(x,y,.08),.09,.07,.15,'metal_light',8)

begin('elevator_controls')
box('Lift-control steel backplate',(0,0,.61),(.55,.13,1.22),'metal_light',.035)
box('Floor readout inset',(0,-.076,.94),(.36,.03,.25),'ink',.025)
for x in [-.08,.08]:box('Readout amber segment',(x,-.097,.94),(.05,.013,.12),'brass',.005)
for i in range(3):
 for x in [-.11,.11]:
  o=cone('Lift tactile floor button',(0,0,0),.048,.048,.025,'red' if i==2 and x>.0 else 'brass',12);o.rotation_euler.x=math.pi/2;o.location=(x,-.095,.63-i*.17)

begin('prize_counter')
box('Prize counter solid carcass',(0,0,.49),(2.0,.67,.98),'wood',.04)
box('Durable overhung counter top',(0,-.025,1.03),(2.11,.85,.13),'wood_light',.045)
box('Counter recessed front panel',(0,-.352,.53),(1.80,.055,.59),'wood_dark',.03)
for x in [-.84,-.42,0,.42,.84]:box('Counter vertical trim',(x,-.391,.52),(.052,.035,.64),'brass',.01)
box('Counter toe rail',(0,-.38,.10),(2.07,.11,.12),'wood_light',.02)

begin('roof_access_hut')
box('Stucco rooftop access enclosure',(0,0,.69),(1.68,1.33,1.38),'cloth',.035)
box('Front timber double door',(0,-.683,.62),(.64,.075,1.15),'wood',.028)
for x in [-.30,.30]:box('Door frame jamb',(x,-.728,.65),(.065,.09,1.30),'wood_dark',.008)
box('Door lintel',(0,-.727,1.30),(.72,.09,.09),'wood_dark',.01)
box('Door inset panel',(0,-.729,.69),(.44,.04,.87),'wood_light',.02)
ico('Door brass knob',(.16,-.77,.62),(.035,.025,.035),'brass',1)
box('Coping slab flat roof',(0,0,1.43),(1.89,1.52,.17),'metal_light',.035)
for z in [.17,.33]:box('Access hut horizontal masonry string',(0,-.700,z),(1.66,.065,.06),'cloth_light',.012)

begin('roof_planter')
box('Long terrace planting trough',(0,0,.24),(1.98,.65,.48),'stone',.07)
box('Terrace trough lip',(0,0,.51),(2.10,.74,.13),'stone_light',.045)
box('Dark planting earth',(0,0,.581),(1.87,.53,.026),'wood_dark',.01)
for x in [-.70,-.33,.08,.44,.77]:
 for j in range(3):
  a=j*2.1+x;rod('Stiff ornamental leaf',(x,0,.59),(x+.22*math.cos(a),.20*math.sin(a),.91+(j%2)*.21),.045,'leaf',5)

begin('gym_stone_partition')
box('Solid squared stone partition',(0,0,.59),(1.0,.92,1.18),'stone',.06)
box('Recessed squared stone face',(0,-.473,.64),(.72,.06,.75),'stone_dark',.025)
box('Inset beveled relief panel',(0,-.516,.65),(.55,.045,.57),'stone_light',.028)
box('Partition coping',(0,0,1.16),(1.07,.98,.16),'stone_light',.04)
box('Partition foundation course',(0,0,.08),(1.08,1.02,.16),'stone_dark',.03)

# Only the observed circular instruments, panel bands, housing and foot are
# modeled. The equipment's function is unknown; there are no invented shelves.
def facility_instruments(name,rows):
 begin(name)
 height=.94 if rows==1 else 1.90
 box('Instrument closed housing',(0,.035,height*.5),(1.,.63,height),'steel',0)
 box('Narrow supporting foot',(0,-.005,.055),(.78,.58,.11),'ink',0)
 for row in range(rows):
  z=.65 if rows==1 else .50+row*.57
  box('Dark paired instrument recess',(0,-.288,z),(.86,.04,.42),'ink',0)
  for x in [-.225,.225]:
   o=ring('Circular instrument rim',(0,0,0),.137,.087,.035,'metal_light',12)
   o.rotation_euler.x=math.pi/2;o.location=(x,-.33,z)
   o=cone('Recessed instrument center',(0,0,0),.083,.083,.018,'steel',12)
   o.rotation_euler.x=math.pi/2;o.location=(x,-.335,z)
   box('Pale upper instrument marker',(x-.052,-.354,z+.091),(.043,.025,.043),'cloth_light',0)
  box('Horizontal lower instrument band',(0,-.329,z-.255),(.86,.045,.055),'metal_light',0)
 if rows==2:box('Plain upper housing panel',(0,-.299,height-.26),(.86,.025,.39),'metal_light',0)
 box('Low horizontal inset',(0,-.293,.145),(.67,.045,.045),'ink',0)
facility_instruments('facility_instrument_pair',1)
facility_instruments('facility_instrument_bank',2)

begin('roof_binoculars')
# The source shows two short west-facing barrels, a fork, a narrow pedestal,
# and a rectangular foot. Keep that side-facing silhouette without accessories.
box('Binocular rectangular foot',(0,0,.06),(.75,.62,.12),'steel',0)
box('Binocular pedestal',(0,0,.32),(.14,.14,.46),'metal_light',0)
box('Binocular fork crosspiece',(0,0,.54),(.18,.60,.10),'steel',0)
box('Blue rear binocular housing',(.31,0,.88),(.32,.69,.38),'blue',0)
for y in [-.25,.25]:
 box('Binocular fork upright',(.06,y,.68),(.13,.10,.32),'steel',0)
 o=cone('West-facing binocular barrel',(-.10,y,.88),.16,.16,.78,'cloth_light',12)
 o.rotation_euler.y=math.pi/2
 o=ring('West binocular lens rim',(0,0,0),.17,.12,.055,'ink',12)
 o.rotation_euler.y=math.pi/2;o.location=(-.505,y,.88)
 o=cone('Pale inset binocular lens',(0,0,0),.115,.115,.022,'cloth_light',12)
 o.rotation_euler.y=math.pi/2;o.location=(-.530,y,.88)
box('Binocular barrel bridge',(.13,0,.88),(.21,.35,.18),'steel',0)

begin('link_round_stool')
# A low round seat on a narrower pedestal is the complete observed stool.
# There is no back, armrest, upholstery seam, or chair leg assembly.
cone('Stool round foot',(0,0,.055),.28,.28,.11,'ink',12)
cone('Stool narrow pedestal',(0,0,.19),.23,.23,.24,'steel',12)
cone('Stool dark seat rim',(0,0,.37),.46,.46,.14,'steel',16)
cone('Stool pale round seat',(0,0,.46),.42,.40,.10,'cloth_light',16)

begin('link_room_receiver')
box('Compact link-room receiver base',(0,0,.075),(.62,.47,.15),'steel',.025)
box('Narrow upright data receiver',(0,.035,.56),(.37,.28,1.01),'metal_light',.035)
box('Receiver shadowed front slot',(0,-.116,.72),(.19,.035,.45),'ink',.011)
box('Receiver pale screen glass',(0,-.139,.85),(.12,.016,.13),'glass',.005)
box('Receiver lower button',(0,-.144,.59),(.068,.027,.073),'cloth_light',.005)
box('Wide shoulder housing',(0,.036,1.09),(.72,.47,.22),'cloth',.035)
box('Shoulder top dark inlay',(0,-.206,1.10),(.54,.031,.075),'steel',.008)
for sign in [-1,1]:
 box('Small receiver side handgrip',(sign*.285,-.102,.92),(.095,.25,.25),'metal_light',.018)
 box('Upper paired lamp',(sign*.16,-.225,1.10),(.066,.019,.039),'glass',.004)

# Non-destructive export preserves individual authored objects in .blend.
def export(name,col):
 groups={};deps=bpy.context.evaluated_depsgraph_get()
 for obj in col.objects:
  if obj.type!='MESH':continue
  obj_eval=obj.evaluated_get(deps);me=obj_eval.to_mesh();me.calc_loop_triangles();normal_matrix=obj.matrix_world.to_3x3().inverted().transposed()
  for tri in me.loop_triangles:
   mat=obj.data.materials[tri.material_index];g=groups.setdefault(mat.name,{'name':mat.name,'base_color':[round(v,5) for v in mat.diffuse_color],'positions':[],'normals':[],'indices':[]})
   if name in SOURCE_PIXELS:g['source_pixel']=SOURCE_PIXELS[name][mat.name]
   normal=(normal_matrix@tri.normal).normalized()
   if normal.length_squared<.9:continue
   for vid in tri.vertices:
    v=obj.matrix_world@me.vertices[vid].co;g['indices'].append(len(g['positions'])//3);g['positions'].extend(round(c,6) for c in (v.x,v.z,-v.y));g['normals'].extend(round(c,6) for c in (normal.x,normal.z,-normal.y))
  obj_eval.to_mesh_clear()
 prim=[groups[n] for n in sorted(groups)];allp=[p['positions'] for p in prim];mins=[min(v[i] for p in allp for v in zip(*[iter(p)]*3)) for i in range(3)];maxs=[max(v[i] for p in allp for v in zip(*[iter(p)]*3)) for i in range(3)]
 data={'name':name,'coordinate_system':'right-handed; +Y up; front +Z','origin':'floor-center','bounds':{'min':mins,'max':maxs},'triangle_count':sum(len(p['indices'])//3 for p in prim),'primitives':prim}
 (out/(name+'.mesh.json')).write_text(json.dumps(data,separators=(',',':'))+'\n')
 return {'name':name,'triangles':data['triangle_count'],'dimensions':[round(b-a,3) for a,b in zip(mins,maxs)]}
stats=[export(name,col) for name,col in assets.items()]
# Editable source is the unjoined, named-object scene plus an inspectable contact sheet.
# This supplement preserves every original asset and adds the exact gate-room receiver.
for i,(name,col) in enumerate(assets.items()):
 for obj in col.objects:obj.location+=Vector(((i%5)*4.2-8.4,(i//5)*4.3-8.6,0))
world=bpy.data.worlds.new('Dungeon studio');scene.world=world;world.use_nodes=True;world.node_tree.nodes['Background'].inputs['Color'].default_value=(.17,.23,.27,1);world.node_tree.nodes['Background'].inputs['Strength'].default_value=.65
for p,energy,size in [((-6,-8,14),1900,10),((8,4,10),1700,8)]:
 bpy.ops.object.light_add(type='AREA',location=p);o=bpy.context.object;o.data.energy=energy;o.data.size=size;o.rotation_euler=(Vector((0,0,.6))-o.location).to_track_quat('-Z','Y').to_euler()
bpy.ops.object.camera_add(location=(14,-23,21));cam=bpy.context.object;cam.rotation_euler=(Vector((0,.4,.4))-cam.location).to_track_quat('-Z','Y').to_euler();cam.data.type='ORTHO';cam.data.ortho_scale=30;scene.camera=cam
scene.render.engine='CYCLES';scene.cycles.samples=24;scene.cycles.use_denoising=False;scene.render.resolution_x=1800;scene.render.resolution_y=1600;scene.render.resolution_percentage=100;scene.render.image_settings.file_format='PNG';scene.render.film_transparent=True;scene.render.filepath=str(out/'special-interiors.png');scene.view_settings.view_transform='AgX'
scene['Provenance']='Original hand-authored geometric kit. No pack art, textures, pixels, scripts, ROM data or game models imported.'
bpy.ops.wm.save_as_mainfile(filepath=str(out/'special-interiors.blend'),compress=False)
if '--skip-preview' not in args:bpy.ops.render.render(write_still=True)
print('SPECIAL_ENVIRONMENT_STATS='+json.dumps(stats))
