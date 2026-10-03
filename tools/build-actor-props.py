"""Original volumetric actor-prop and creature kit, Blender 4.3+.

blender -b --python tools/build-actor-props.py -- target/actor-props [--skip-preview]
Focused rebuild: append --only=sudowoodo,icon_sudowoodo; other assets are not emitted.
No game images are read or embedded: silhouettes, markings, joins and surfaces
are authored geometry. Collections remain editable and each asset has a floor
root, +Z up/front -Y; exports use +Y up/front +Z. Exact source IDs, rather than
species guesses, select these models in johto_actor_props.rs.
"""
import bpy, json, math, sys
from pathlib import Path
from mathutils import Vector, Matrix

ARGS = sys.argv[sys.argv.index('--') + 1:] if '--' in sys.argv else []
OUT = Path(next((a for a in ARGS if not a.startswith('--')), 'target/actor-props')).resolve()
OUT.mkdir(parents=True, exist_ok=True)
bpy.ops.object.select_all(action='SELECT'); bpy.ops.object.delete(use_global=False)
SCENE = bpy.context.scene
COL = None
MATERIALS = {}
PALETTE = {
 'ink':(.10,.14,.19), 'black':(.16,.18,.20), 'white':(.96,.95,.85),
 'cream':(.94,.85,.61), 'red':(.82,.19,.22), 'coral':(.94,.39,.35),
 'pink':(.96,.62,.66), 'rose':(.71,.36,.46), 'yellow':(.96,.75,.22),
 'gold':(.80,.54,.15), 'orange':(.90,.43,.17), 'brown':(.44,.29,.17),
 'tan':(.73,.58,.36), 'green':(.28,.56,.32), 'leaf':(.44,.69,.36),
 'darkgreen':(.16,.37,.29), 'teal':(.22,.57,.59), 'blue':(.32,.57,.78),
 'navy':(.18,.32,.47), 'purple':(.49,.35,.64), 'lilac':(.65,.57,.76),
 'silver':(.67,.72,.74), 'stone':(.53,.55,.51), 'stone_light':(.69,.69,.62),
 'stone_dark':(.34,.38,.38), 'wood':(.55,.36,.21), 'screen':(.30,.45,.39),
}
def linear(v): return v/12.92 if v <= .04045 else ((v+.055)/1.055)**2.4
def material(key):
 if key not in MATERIALS:
  m=bpy.data.materials.new(key); m.diffuse_color=tuple(linear(c) for c in PALETTE[key])+(1,)
  m.use_nodes=True; bs=m.node_tree.nodes.get('Principled BSDF');bs.inputs['Base Color'].default_value=m.diffuse_color
  bs.inputs['Roughness'].default_value=.76; bs.inputs['Metallic'].default_value=.25 if key in ('gold','silver') else 0
  MATERIALS[key]=m
 return MATERIALS[key]
def finish(o,name,color,smooth=True):
 o.name=name
 for c in list(o.users_collection): c.objects.unlink(o)
 COL.objects.link(o);o.data.materials.append(material(color))
 for p in o.data.polygons:p.use_smooth=smooth
 return o
def sphere(name,p,s,color,segments=16,rings=8):
 bpy.ops.mesh.primitive_uv_sphere_add(segments=segments,ring_count=rings,radius=1,location=p)
 o=finish(bpy.context.object,name,color);o.scale=s
 bpy.ops.object.transform_apply(location=False,rotation=False,scale=True)
 return o
def ico(name,p,s,color,sub=1):
 bpy.ops.mesh.primitive_ico_sphere_add(subdivisions=sub,radius=1,location=p)
 o=finish(bpy.context.object,name,color,False);o.scale=s
 bpy.ops.object.transform_apply(location=False,rotation=False,scale=True)
 return o
def box(name,p,s,color,bevel=.025):
 bpy.ops.mesh.primitive_cube_add(size=1,location=p);o=finish(bpy.context.object,name,color,False);o.dimensions=s
 bpy.ops.object.transform_apply(location=False,rotation=False,scale=True)
 if bevel:
  m=o.modifiers.new('Rounded manufactured edge','BEVEL');m.width=min(bevel,min(s)*.40);m.segments=2
  bpy.ops.object.modifier_apply(modifier=m.name)
  m=o.modifiers.new('Weighted corner normals','WEIGHTED_NORMAL');m.keep_sharp=True
  bpy.ops.object.modifier_apply(modifier=m.name)
 return o
def rod(name,a,b,r1,r2,color,vertices=12):
 a,b=Vector(a),Vector(b)
 bpy.ops.mesh.primitive_cone_add(vertices=vertices,radius1=r1,radius2=r2,depth=(b-a).length,location=(a+b)*.5)
 o=finish(bpy.context.object,name,color);o.rotation_euler=(b-a).to_track_quat('Z','Y').to_euler();return o
def tube(name,points,r,color):
 # Bevelled polyline is actual round geometry, never a camera-facing stripe.
 if 3 <= len(points) <= 20 and name not in ('Curled tail','Curled turtle tail','Curled ear') and not (name == 'Coiled serpent body' and r < .08) and any(k in name.lower() for k in ('tail','serpent','neck','whisker','tentacle','curled','smile','cable')):
  controls=[Vector(p) for p in points];sampled=[]
  for i in range(len(controls)-1):
   a=controls[max(i-1,0)];b=controls[i];c=controls[i+1];d=controls[min(i+2,len(controls)-1)]
   for j in range(4):
    t=j/4;sampled.append(.5*((2*b)+(-a+c)*t+(2*a-5*b+4*c-d)*t*t+(-a+3*b-3*c+d)*t*t*t))
  points=sampled+[controls[-1]]
 cu=bpy.data.curves.new(name,'CURVE');cu.dimensions='3D';cu.resolution_u=1;cu.bevel_depth=r;cu.bevel_resolution=2;cu.resolution_u=1;cu.use_fill_caps=True
 sp=cu.splines.new('POLY');sp.points.add(len(points)-1)
 for p,co in zip(sp.points,points):p.co=(*co,1)
 o=bpy.data.objects.new(name,cu);COL.objects.link(o);bpy.context.view_layer.objects.active=o;o.select_set(True)
 bpy.ops.object.convert(target='MESH');o=bpy.context.object;o.select_set(False);o.data.materials.append(material(color))
 for p in o.data.polygons:p.use_smooth=True
 return o
def torus(name,p,major,minor,color,rotation=None,scale=None):
 bpy.ops.mesh.primitive_torus_add(major_segments=20,minor_segments=8,location=p,major_radius=major,minor_radius=minor)
 o=finish(bpy.context.object,name,color)
 if rotation:o.rotation_euler=rotation
 if scale:o.scale=scale
 return o
def prism(name,outline,y,thickness,color):
 # Closed, extruded outlined shape in the X/Z plane (wings, fins, stars, tails).
 n=len(outline);vs=[(x,y-thickness/2,z) for x,z in outline]+[(x,y+thickness/2,z) for x,z in outline]
 faces=[tuple(range(n-1,-1,-1)),tuple(range(n,n*2))]+[(i,(i+1)%n,(i+1)%n+n,i+n) for i in range(n)]
 me=bpy.data.meshes.new(name);me.from_pydata(vs,[],faces);me.update();o=bpy.data.objects.new(name,me);COL.objects.link(o);me.materials.append(material(color))
 bpy.context.view_layer.objects.active=o;o.select_set(True);bpy.ops.object.mode_set(mode='EDIT');bpy.ops.mesh.select_all(action='SELECT');bpy.ops.mesh.normals_make_consistent(inside=False);bpy.ops.object.mode_set(mode='OBJECT');o.select_set(False)
 return o
def eyes(x,y,z,size=.025,spacing=.10,white=False):
 for side in (-1,1):
  if white:sphere('Eye sclera',(x+side*spacing,y+.003,z),(size*1.55,size*.52,size*1.7),'white',12,6)
  sphere('Eye pupil',(x+side*spacing,y-.01,z),(size,size*.48,size*1.25),'ink',12,6)
  sphere('Eye glint',(x+side*spacing-size*.26,y-size*.48-.012,z+size*.36),(size*.23,size*.16,size*.23),'white',8,4)
def feet(color='cream',spacing=.16,y=-.025,z=.055,s=(.105,.16,.055)):
 for side in (-1,1):sphere('Splayed foot',(side*spacing,y,z),s,color)
def ear(side,p,s,color,tip=None):
 o=sphere('Tapered ear',(side*p[0],p[1],p[2]),s,color);o.rotation_euler[1]=side*.27
 if tip:
  sphere('Dark ear tip',(side*(p[0]+.018),p[1],p[2]+s[2]*.69),(s[0]*.8,s[1]*.88,s[2]*.35),tip)
def belly(p,s,color='cream'):sphere('Belly inset',p,s,color)
def muzzle(p,s,color='cream'):sphere('Muzzle',p,s,color)
def smile(y,z,width=.11):
 tube('Smile seam',[(-width,y,z+.018),(0,y-.008,z-.012),(width,y,z+.018)],.008,'ink')
def leaf(name,p,tip,width,color='leaf'):
 a=Vector(p);b=Vector(tip);mid=(a+b)/2;v=(b-a).cross(Vector((0,1,.1))).normalized()*width
 me=bpy.data.meshes.new(name);verts=[a,mid+v,b,mid-v,mid+Vector((0,-.035,0)),mid+Vector((0,.025,0))]
 faces=[(0,1,4),(1,2,4),(2,3,4),(3,0,4),(1,0,5),(2,1,5),(3,2,5),(0,3,5)]
 me.from_pydata(verts,[],[tuple(reversed(f)) for f in faces]);me.update();o=bpy.data.objects.new(name,me);COL.objects.link(o);me.materials.append(material(color));return o

def ball(voltorb=False):
 # Hemisphere material separation is a genuine equatorial seam.
 for upper,col in ((True,'red'),(False,'white')):
  seg=24;nr=7;vs=[]
  for r in range(nr+1):
   t=(r/nr)*(math.pi/2)+(0 if upper else math.pi/2)
   for k in range(seg):
    a=k/seg*math.tau;vs.append((.32*math.sin(t)*math.cos(a),.32*math.sin(t)*math.sin(a),.32+.32*math.cos(t)))
  fs=[]
  for r in range(nr):
   for k in range(seg):
    a=r*seg+k;b=r*seg+(k+1)%seg;c=(r+1)*seg+(k+1)%seg;d=(r+1)*seg+k
    if r==0 and upper:fs.append((a,c,d))
    elif r==nr-1 and not upper:fs.append((a,b,c))
    else:fs.append((a,b,c,d))
  me=bpy.data.meshes.new('Molded hemisphere');me.from_pydata(vs,[],[tuple(reversed(f)) for f in fs]);me.update();o=bpy.data.objects.new('Upper shell' if upper else 'Lower shell',me);COL.objects.link(o);me.materials.append(material(col))
  for f in me.polygons:f.use_smooth=True
 if voltorb:
  for s in (-1,1):
   prism('Angry eye',[(s*.05,.44),(s*.23,.48),(s*.19,.35),(s*.08,.36)],-.283,.018,'white')
   sphere('Angry pupil',(s*.125,-.30,.397),(.016,.012,.044),'ink')
 else:
  torus('Dark joining seam',(0,0,.32),.322,.022,'ink')
  rod('Button surround',(0,-.32,.32),(0,-.37,.32),.082,.082,'ink',20)
  rod('White release button',(0,-.373,.32),(0,-.39,.32),.052,.052,'white',20)

def stone(boulder=False):
 if boulder:
  o=ico('Broad fractured strength boulder',(0,0,.38),(.43,.39,.40),'stone',2)
  for i,v in enumerate(o.data.vertices):v.co*=1+math.sin(i*7.37)*.09
  for i,p in enumerate(o.data.polygons):
   if i%7==0:p.material_index=1
  o.data.materials.append(material('stone_light'))
  for i in range(4):
   a=i*math.tau/4;ico('Embedded low face',(.23*math.cos(a),.23*math.sin(a),.12),(.17,.16,.15),'stone_dark',1)
 else:
  for i,(p,s) in enumerate([((0,0,.22),(.34,.29,.25)),((-.16,.06,.40),(.20,.18,.23)),((.19,-.05,.16),(.17,.19,.17)),((.02,-.20,.10),(.20,.12,.10))]):ico('Broken rock shard',p,s,'stone_light' if i==1 else 'stone',1)

def fruit_tree():
 rod('Main tapered trunk',(0,0,0),(0,0,.64),.09,.052,'wood')
 for x,y,z,r in [(-.25,.0,.78,.28),(.24,.06,.80,.29),(.0,.02,1.00,.27),(0,-.21,.75,.30),(0,.23,.78,.25)]:
  rod('Branch fork',(0,0,.39),(x,y,z),.035,.016,'wood');ico('Orchard leaf lobe',(x,y,z),(r,r*.93,r*.86),'green',2)
 for x,y,z in [(-.20,-.435,.79),(.18,-.42,.82),(.02,-.41,.95),(-.46,-.08,.76),(.47,-.08,.84),(-.12,.435,.81),(.14,.405,.88),(.035,-.15,1.215),(.02,.40,1.00)]:
  sphere('Berry fruit',(x,y,z),(.048,.048,.053),'red',12,6);rod('Fruit stem',(x,y,z+.045),(x+.008,y,z+.079),.009,.006,'wood')
  leaf('Fruit leaf',(x,y,z+.066),(x+.073,y-.015,z+.083),.019)

def paper():
 o=box('Thick paper folio',(0,0,.055),(.46,.62,.06),'cream',.009)
 for row in range(5):box('Written line',(-.025,-.19+row*.075,.09),(.30-(row%2)*.07,.014,.009),'stone_dark',.003)
 prism('Turned paper corner',[(.15,.085),(.23,.085),(.23,.15)],.28,.06,'white')
def pokedex():
 for s in (-1,1):
  box('Pokedex hinged cover',(s*.18,0,.065),(.34,.50,.11),'red',.035)
  box('Inner panel',(s*.18,-.015,.124),(.285,.43,.014),'cream',.006)
 box('Glass screen',(-.18,-.075,.139),(.215,.21,.012),'screen',.004)
 for j in range(2):
  for k in range(2):box('Directory page block',(.12+j*.115,-.125+k*.16,.14),(.075,.105,.015),'stone',.003)
 rod('Book spine',(0,-.24,.07),(0,.24,.07),.023,.023,'gold')
 for x in (-.24,-.13):sphere('Button',(x,.14,.148),(.025,.025,.01),'ink',10,4)

def controller(p,style='snes'):
 x,y,z=p;sphere('Controller grip',(x,y,z),(.19,.09,.045),'silver' if style=='snes' else 'black')
 if style=='n64':
  for side in (-1,0,1):sphere('Three-prong grip',(x+side*.11,y+.075,z-.01),(.045,.10,.05),'black')
 for dx,dy in ((-.09,0),):
  box('D-pad horizontal',(x+dx,y+dy,z+.044),(.069,.022,.012),'ink',.004);box('D-pad vertical',(x+dx,y+dy,z+.044),(.022,.069,.012),'ink',.004)
 for dx,dy,col in ((.075,-.025,'red'),(.115,.015,'blue')):sphere('Controller button',(x+dx,y+dy,z+.042),(.018,.018,.014),col,10,4)
def console(kind):
 if kind=='famicom':
  box('Console lower housing',(0,.07,.08),(.58,.42,.15),'red',.04);box('Cream upper housing',(0,.07,.18),(.52,.40,.13),'cream',.03)
  box('Cartridge rail',(0,.04,.253),(.32,.07,.015),'ink',.004)
  box('Cartridge',(0,.04,.32),(.28,.045,.15),'cream',.008)
  for s in (-1,1):box('Red side controller',(s*.32,.06,.16),(.105,.29,.10),'red',.02)
  box('Power switch',(-.14,-.075,.25),(.06,.06,.025),'red',.005)
 elif kind=='snes':
  box('SNES lower case',(0,.06,.085),(.55,.44,.16),'silver',.055);box('SNES upper shell',(0,.08,.17),(.48,.36,.13),'white',.045)
  box('Cartridge slot',(0,.08,.242),(.30,.04,.014),'ink',.005)
  for s in (-1,1):box('Purple slide switch',(s*.15,-.04,.245),(.08,.08,.023),'lilac',.007)
  for i in range(5):box('Case cooling vent',(-.10+i*.05,.17,.238),(.02,.09,.014),'stone',.003)
  controller((0,-.32,.055));tube('Controller cable',[(0,-.25,.06),(.20,-.23,.04),(.23,-.17,.04),(.14,-.13,.07)],.012,'ink')
 elif kind=='n64':
  box('Low dark N64 body',(0,.06,.10),(.56,.39,.18),'black',.065);sphere('Rounded console shoulders',(0,.09,.18),(.28,.17,.10),'black')
  box('Cartridge inserted',(0,.13,.28),(.30,.055,.21),'stone',.02);box('Cartridge label',(0,.097,.30),(.20,.013,.095),'gold',.004)
  for i in range(4):rod('Front controller socket',(-.18+i*.12,-.138,.10),(-.18+i*.12,-.155,.10),.023,.023,'stone_dark')
  controller((0,-.36,.07),'n64');tube('Controller wire',[(0,-.30,.055),(-.23,-.26,.045),(-.25,-.17,.04),(-.18,-.15,.10)],.010,'ink')
 elif kind=='virtual_boy':
  box('Red binocular housing',(0,0,.42),(.62,.24,.26),'red',.06)
  for s in (-1,1):
   box('Black eye socket',(s*.15,-.14,.42),(.235,.07,.19),'ink',.06);box('Recessed lens',(s*.15,-.179,.42),(.15,.015,.105),'purple',.035)
   rod('Tripod front leg',(0,.02,.29),(s*.24,-.19,.025),.028,.018,'stone_dark')
  rod('Tripod rear leg',(0,.02,.29),(0,.22,.025),.028,.018,'stone_dark');rod('Stand',(0,0,.18),(0,0,.34),.04,.04,'stone_dark')
def trophy(silver=False):
 col='silver' if silver else 'gold';box('Trophy plinth',(0,0,.045),(.36,.28,.09),'wood',.015)
 box('Engraved plaque',(0,-.146,.05),(.19,.012,.035),col,.005);rod('Stem',(0,0,.075),(0,0,.32),.07,.045,col)
 rod('Cup foot',(0,0,.13),(0,0,.17),.14,.10,col);rod('Flared trophy bowl',(0,0,.31),(0,0,.53),.10,.24,col,24)
 torus('Cup rolled rim',(0,0,.54),.24,.028,col)
 # Dark recessed interior makes the trophy visibly a vessel.
 sphere('Cup inner hollow',(0,0,.526),(.204,.204,.020),'stone_dark')
 for side in (-1,1):torus('Loop handle',(side*.23,0,.41),.10,.024,col,(math.pi/2,0,0),(.75,1,1))

def bird():
 feet('orange',.105,s=(.06,.12,.042));sphere('Bird breast',(0,0,.28),(.21,.17,.24),'tan');belly((0,-.135,.25),(.15,.045,.17),'cream')
 sphere('Rounded head',(0,-.025,.51),(.17,.16,.16),'brown');eyes(0,-.167,.53,.022,.080)
 rod('Conical beak',(0,-.15,.47),(0,-.30,.45),.072,.007,'orange')
 for side in (-1,1):
  o=sphere('Folded feathered wing',(side*.19,.015,.29),(.072,.15,.16),'brown');o.rotation_euler[1]=side*.20
  for j in range(3):tube('Wing feather rib',[(side*.20,-.02+j*.05,.34),(side*.22,.075+j*.025,.21)],.012,'cream')
  rod('Crest feather',(side*.04,-.01,.60),(side*.065,.12,.72),.035,.004,'brown')
 for s in (-1,0,1):rod('Fan tail feather',(0,.11,.18),(s*.10,.35,.25),.052,.015,'brown')

def fairy(jiggly=False):
 col='pink';feet('rose',.15,s=(.095,.12,.055));sphere('Round fairy body',(0,0,.34),(.30,.23,.29),col)
 for side in (-1,1):
  if jiggly:
   prism('Pointed ear',[(side*.15,.51),(side*.27,.72),(side*.29,.44)],.015,.12,col)
   prism('Dark ear inset',[(side*.195,.535),(side*.252,.646),(side*.262,.493)],-.052,.018,'ink')
  else:
   ear(side,(.235,0,.59),(.115,.09,.18),col)
   sphere('Brown ear tip',(side*.263,-.005,.70),(.071,.075,.073),'brown')
  sphere('Short arm',(side*.30,-.02,.32),(.11,.09,.075),col)
 eyes(0,-.226,.39,.045 if jiggly else .025,.103,white=jiggly)
 smile(-.229,.27,.055)
 if jiggly:
  tube('Forehead curl',[(.08,-.145,.59),(.10,-.215,.65),(.015,-.236,.68),(-.08,-.232,.63),(-.07,-.246,.57),(0,-.250,.58)],.035,'pink')
 else:
  tube('Forehead spiral',[(.08,-.095,.59),(0,-.14,.67),(-.095,-.17,.64),(-.11,-.185,.59),(-.045,-.20,.58)],.031,'pink')
  for side in (-1,1):prism('Small back wing',[(side*.16,.32),(side*.39,.42),(side*.37,.27),(side*.23,.21)],.14,.075,'cream')
  tube('Curled tail',[(0,.15,.24),(.24,.27,.25),(.28,.26,.38),(.20,.24,.42),(.17,.23,.35)],.036,'pink')

def monster(big=False):
 color='purple' if not big else 'orange';feet(color,.17,s=(.11,.17,.065));sphere('Kaiju pear torso',(0,0,.36),(.29,.22,.31),color)
 belly((0,-.187,.34),(.20,.063,.23),'cream');sphere('Wide reptile head',(0,-.02,.67),(.25,.21,.20),color)
 muzzle((0,-.193,.62),(.19,.10,.085),color);eyes(0,-.184,.73,.030,.112,True)
 for side in (-1,1):
  rod('Head horn',(side*.16,.0,.78),(side*.24,.0,.97),.075,.003,'cream')
  sphere('Arm',(side*.27,-.005,.44),(.095,.11,.19),color)
  for i in range(3):rod('Toe claw',(side*.17+(i-1)*.045,-.14,.06),(side*.17+(i-1)*.045,-.215,.035),.021,.001,'white')
 tube('Heavy sweeping tail',[(0,.15,.22),(.05,.38,.13),(.21,.46,.19),(.30,.44,.32)],.085,color)
 for z,y in ((.26,.27),(.47,.22),(.70,.15)):rod('Dorsal spine',(0,y,z),(0,y+.14,z+.09),.06,.002,'cream')
 if big:
  for side in (-1,1):prism('Dragon wing',[(side*.16,.62),(side*.52,.87),(side*.55,.49),(side*.36,.57),(side*.27,.41)],.10,.045,'teal')
 smile(-.288,.59,.09)

def dragon():
 monster(True)
 for side in (-1,1):
  rod('Wing leading edge',(side*.17,.10,.56),(side*.53,.10,.88),.025,.014,'orange')
  rod('Wing spar',(side*.30,.095,.66),(side*.55,.095,.49),.020,.009,'orange')
 sphere('Tail flame heart',(.30,.44,.35),(.075,.065,.14),'yellow')
 rod('Tail flame tongue',(.30,.44,.40),(.28,.43,.59),.06,.001,'orange')

def slowpoke():
 sphere('Long low body',(0,.06,.24),(.28,.39,.23),'pink')
 for x in (-.21,.21):
  for y in (-.16,.28):sphere('Stubby planted leg',(x,y,.105),(.105,.13,.105),'pink')
 sphere('Broad sleepy head',(0,-.25,.39),(.28,.23,.23),'pink');muzzle((0,-.453,.30),(.27,.105,.13),'cream')
 eyes(0,-.438,.43,.017,.14)
 for s in (-1,1):sphere('Round little ear',(s*.23,-.17,.56),(.078,.078,.072),'pink')
 tube('Thick curved tail',[(0,.33,.28),(.10,.52,.26),(.22,.62,.37),(.25,.61,.53)],.085,'pink');sphere('Pale tail tip',(.25,.61,.55),(.086,.086,.11),'cream')
 smile(-.553,.25,.12)

def sudowoodo():
 # One sculpted silhouette carries the face, shoulders, branch crown and roots.
 # Broad low-poly planes and inset details stay legible at battle-camera scale.
 # The authoring cage is original geometry; no images or textures are embedded.
 import bmesh
 PALETTE.update({'sudowoodo_bark':(.66,.445,.295),
                 'sudowoodo_cut':(.77,.555,.36),
                 'sudowoodo_groove':(.36,.225,.145),
                 'sudowoodo_ochre':(.96,.755,.29),
                 'sudowoodo_leaf':(.30,.60,.205),
                 'sudowoodo_leaf_light':(.395,.685,.265),
                 'sudowoodo_mouth':(.82,.45,.335),
                 'sudowoodo_eye':(.08,.055,.035)})
 bark='sudowoodo_bark';groove='sudowoodo_groove'
 def mesh_part(name,verts,faces,color,smooth=True):
  me=bpy.data.meshes.new(name+' / authored cage');me.from_pydata(verts,[],faces);me.update()
  o=bpy.data.objects.new(name,me);COL.objects.link(o);me.materials.append(material(color))
  bm=bmesh.new();bm.from_mesh(me);bmesh.ops.recalc_face_normals(bm,faces=bm.faces)
  if bm.calc_volume(signed=True)<0:bmesh.ops.reverse_faces(bm,faces=bm.faces)
  bm.to_mesh(me);bm.free()
  for face in me.polygons:face.use_smooth=smooth
  return o
 def sampled(nodes,steps=3):
  controls=[Vector(n) for n in nodes];values=[]
  for i in range(len(controls)-1):
   a,b,c,d=controls[max(i-1,0)],controls[i],controls[i+1],controls[min(i+2,len(controls)-1)]
   for j in range(steps):
    t=j/steps;values.append(.5*((2*b)+(-a+c)*t+(2*a-5*b+4*c-d)*t*t+(-a+3*b-3*c+d)*t*t*t))
  return values+[controls[-1]]
 def branch(name,nodes,seg=14,steps=3):
  nodes=sampled(nodes,steps);verts=[]
  reference=Vector((0,1,0))
  for i,n in enumerate(nodes):
   p=Vector(n[:3]);t=(Vector(nodes[min(i+1,len(nodes)-1)][:3])-Vector(nodes[max(i-1,0)][:3])).normalized()
   axis_x=reference.cross(t).normalized();axis_y=t.cross(axis_x).normalized()
   for j in range(seg):
    a=math.tau*j/seg;verts.append(p+(axis_x*math.cos(a)+axis_y*math.sin(a))*max(.003,n[3]))
  faces=[tuple(range(seg-1,-1,-1))]
  for i in range(len(nodes)-1):
   for j in range(seg):
    a=i*seg+j;b=i*seg+(j+1)%seg;faces.append((a,b,b+seg,a+seg))
  faces.append(tuple(range((len(nodes)-1)*seg,len(nodes)*seg)))
  return mesh_part(name,verts,faces,bark)
 profiles=sampled([
  (-.039,.018,.155,.047,.054),(-.043,.015,.198,.108,.095),
  (-.043,.012,.270,.158,.128),(-.030,.016,.385,.174,.137),
  (-.005,.017,.530,.164,.132),(.015,.013,.690,.160,.137),
  (.019,.003,.845,.182,.151),(.010,0,.997,.181,.151),
  (.003,.003,1.065,.153,.133),(-.003,.007,1.095,.099,.095),
  (-.007,.009,1.108,.024,.028)],3)
 seg=24;verts=[]
 for cx,cy,z,rx,ry in profiles:
  for j in range(seg):
   a=math.tau*j/seg;c=math.cos(a);s=math.sin(a);exponent=.72 if z>.78 else .88
   # The broad forehead and flattened cheek planes are part of the trunk.
   x=rx*math.copysign(abs(c)**exponent,c);y=ry*math.copysign(abs(s)**exponent,s)
   verts.append((cx+x,cy+y,z))
 faces=[tuple(range(seg-1,-1,-1))]
 for i in range(len(profiles)-1):
  for j in range(seg):
   a=i*seg+j;b=i*seg+(j+1)%seg;faces.append((a,b,b+seg,a+seg))
 faces.append(tuple(range((len(profiles)-1)*seg,len(profiles)*seg)))
 clay=[mesh_part('Broad gently leaning trunk and cheek planes',verts,faces,bark)]
 clay.append(branch('Crown stalk',[(-.020,.015,1.055,.046),(-.025,.014,1.115,.040),(-.037,.015,1.177,.038)]))
 clay.append(branch('Left blunt crown fork',[(-.037,.015,1.165,.039),(-.105,.017,1.189,.043),(-.172,.018,1.232,.039),(-.190,.018,1.250,.035)]))
 clay.append(branch('Right blunt crown fork',[(-.039,.015,1.166,.040),(.027,.016,1.213,.042),(.085,.017,1.267,.041),(.108,.017,1.302,.035)]))
 hands=[]
 for side in (-1,1):
  label='Left' if side<0 else 'Right';dz=.035 if side<0 else -.013
  wrist=(side*.438,-.014,.872+dz)
  clay.append(branch(label+' continuous shoulder and bent branch arm',[
   (side*.115,.002,.740,.074),(side*.209,.004,.735+dz,.056),
   (side*.313,-.003,.764+dz,.045),(side*.386,-.010,.812+dz,.045),(*wrist,.052)]))
  buds=[(side*.458,-.018,.997+dz),(side*.537,-.048,.888+dz),(side*.365,-.068,.926+dz)]
  for k,(x,y,z) in enumerate(buds):
   clay.append(branch(label+' hand finger '+str(k+1),[
    (*wrist,.029),(wrist[0]*.4+x*.6,wrist[1]*.4+y*.6,wrist[2]*.4+z*.6,.024),(x,y,z,.018)],seg=12,steps=2))
   hands.append((label,k,(x,y,z)))
  clay.append(branch(label+' connected root leg',[
   (side*.079,.020,.242,.070),(side*.127,.016,.181,.063),
   (side*.178,-.008,.108,.047),(side*.176,-.043,.063,.047)],seg=16))
  foot=sphere(label+' broad planted root foot',(side*.187,-.073,.053),(.089,.143,.055),bark,20,10)
  foot.rotation_euler[2]=side*.20;clay.append(foot)
  for toe in (-1,1):
   clay.append(branch(label+' blunt root toe '+str(toe),[
    (side*.18+toe*.038,-.127,.046,.034),(side*.19+toe*.038,-.180,.035,.027),
    (side*.197+toe*.037,-.202,.031,.008)],seg=12,steps=2))
 # Fuse the anatomical cage so roots, shoulders and crown grow out of the trunk.
 bpy.ops.object.select_all(action='DESELECT')
 for o in clay:o.select_set(True)
 bpy.context.view_layer.objects.active=clay[0];bpy.ops.object.join();core=bpy.context.object
 core.name='Continuous leaning trunk / integrated crown arms and roots'
 bpy.ops.object.transform_apply(location=False,rotation=False,scale=True)
 m=core.modifiers.new('Continuous wood sculpture','REMESH');m.mode='VOXEL';m.voxel_size=.0105;m.use_smooth_shade=True;bpy.ops.object.modifier_apply(modifier=m.name)
 m=core.modifiers.new('Soften sculpt junctions','SMOOTH');m.factor=.62;m.iterations=3;bpy.ops.object.modifier_apply(modifier=m.name)
 core.select_set(False)
 # Round low-poly green finger orbs remain distinct, with broad planned facets.
 for label,k,p in hands:
  o=ico(label+' green ball '+str(k+1),p,(.082,.078,.083),'sudowoodo_leaf_light' if k==0 else 'sudowoodo_leaf',3)
  for f in o.data.polygons:f.use_smooth=True
 # Blunt cut faces are narrow closed volumes fitted to the actual fork ends.
 for label,p,n in [('Left',(-.189,.018,1.249),(-.71,0,.70)),('Right',(.107,.017,1.300),(.55,0,.84))]:
  o=sphere(label+' crown end grain',p,(.031,.030,.003),'sudowoodo_cut',16,6);o.rotation_euler=Vector(n).to_track_quat('Z','Y').to_euler()
 def front_y(x,z):
  hit,point,normal,index=core.ray_cast(Vector((x,-1,z)),Vector((0,1,0)))
  if hit:return point.y
  lower=profiles[0];upper=profiles[-1]
  for a,b in zip(profiles,profiles[1:]):
   if a[2]<=z<=b[2]:lower,upper=a,b;break
  t=max(0,min(1,(z-lower[2])/max(.000001,upper[2]-lower[2])))
  cx,cy,zz,rx,ry=lower.lerp(upper,t)
  exponent=.72 if z>.78 else .88;u=min(.997,abs((x-cx)/rx))
  return cy-ry*max(.001,1-u**(2/exponent))**(exponent*.5)
 def patch(name,outline,color,offset=.008,depth=.008):
  # Closed radial surface follows the cheek volume, never a floating flat card.
  k=len(outline);cx=sum(p[0] for p in outline)/k;cz=sum(p[1] for p in outline)/k;vs=[]
  for layer in (0,depth):
   vs.append((cx,front_y(cx,cz)-offset+layer,cz))
   for t in (.50,1):
    for x,z in outline:
     px=cx+(x-cx)*t;pz=cz+(z-cz)*t;vs.append((px,front_y(px,pz)-offset+layer,pz))
  n=1+k*2;fs=[]
  for base in (0,n):
   for j in range(k):
    fs.append((base,base+1+j,base+1+(j+1)%k))
    a=base+1+j;b=base+1+(j+1)%k;fs.append((a,b,b+k,a+k))
  for j in range(k):a=1+k+j;b=1+k+(j+1)%k;fs.append((a,b,b+n,a+n))
  return mesh_part(name,vs,fs,color,False)
 def oval(cx,cz,rx,rz,tilt=0,n=16):
  return [(cx+rx*math.cos(a)*math.cos(tilt)+rz*math.sin(a)*math.sin(tilt),
           cz-rx*math.cos(a)*math.sin(tilt)+rz*math.sin(a)*math.cos(tilt)) for a in (math.tau*j/n for j in range(n))]
 for side in (-1,1):
  label='Left' if side<0 else 'Right';x=.009+side*.073;z=.997
  patch(label+' carved oval eye socket',oval(x,z,.030,.043),'sudowoodo_groove',.007,.011)
  patch(label+' inset oval eye',oval(x,z,.0235,.0355),'sudowoodo_eye',.009,.006)
  patch(label+' eye catchlight',oval(x-.005,z+.014,.0051,.0082,n=10),'white',.012,.004)
  # A shallow upper lid shades the socket and makes the expression intentional.
  outline=[(x-.032,z+.015),(x-.023,z+.043),(x+.016,z+.048),(x+.031,z+.030),
           (x+.025,z+.027),(x+.012,z+.039),(x-.018,z+.036),(x-.026,z+.013)]
  patch(label+' sculpted upper eyelid',outline,bark,.012,.010)
 mouth=[]
 for i in range(17):t=-1+2*i/16;mouth.append((.009+.117*t,.927+.023*t*t))
 for i in range(16,-1,-1):t=-1+2*i/16;mouth.append((.009+.117*t,.891+.059*t*t))
 patch('Inset broad smiling mouth',mouth,'sudowoodo_eye',.010,.010)
 lower=[]
 for i in range(13):t=-.86+1.72*i/12;lower.append((.009+.117*t,.902+.047*t*t))
 for i in range(12,-1,-1):t=-.86+1.72*i/12;lower.append((.009+.117*t,.895+.054*t*t))
 patch('Warm lower inner smile',lower,'sudowoodo_mouth',.013,.004)
 # Ocher bark inlays wrap the trunk. Larger quiet shapes survive battle scale.
 for j,(x,z,rx,rz,tilt) in enumerate([(-.087,.312,.031,.059,-.20),(.051,.414,.028,.057,.19),
   (-.058,.563,.034,.060,.22),(.096,.570,.020,.036,.20),(.040,.726,.031,.063,.18)]):
  inlay=patch('Ochre bark inlay '+str(j+1),oval(x,z,rx,rz,tilt),'sudowoodo_ochre',.0015,.005)
  inlay['trunk_surface_inlay']=True
 # Restrained rear grain reinforces the curved, fully volumetric back.
 for side in (-1,1):
  pts=[(side*.092+.004*t,.125+.007*math.sin(t*math.pi),.42+.23*t) for t in (0,.33,.67,1)]
  tube(('Left' if side<0 else 'Right')+' rear growth crease',pts,.0024,groove)

def pikachu(surf=False):
 feet('yellow',.125,s=(.083,.13,.054));sphere('Pear shaped body',(0,0,.31),(.205,.15,.23),'yellow');sphere('Mouse head',(0,-.015,.56),(.23,.18,.20),'yellow')
 for s in (-1,1):
  ear(s,(.155,.01,.82),(.063,.049,.25),'yellow','ink')
  sphere('Red cheek',(s*.182,-.159,.50),(.044,.022,.042),'red')
  sphere('Small reaching arm',(s*.20,-.04,.34),(.064,.066,.14),'yellow')
 eyes(0,-.183,.59,.027,.091);sphere('Nose',(0,-.198,.535),(.022,.012,.015),'ink');smile(-.178,.46,.044)
 for z in (.25,.39):box('Back stripe',(0,.143,z),(.25,.02,.043),'brown',.01)
 prism('Lightning bolt tail',[(.10,.23),(.22,.23),(.26,.41),(.40,.39),(.34,.66),(.49,.70),(.51,.89),(.29,.84),(.31,.64),(.20,.58)],.11,.06,'yellow')
 prism('Brown tail root',[(.10,.23),(.22,.23),(.25,.38),(.18,.36)],.11,.066,'brown')
 if surf:
  for o in list(COL.objects):o.location.z+=.10
  sphere('Surfboard',(0,0,.065),(.31,.61,.060),'blue',24,8);box('Surfboard center stripe',(0,0,.115),(.08,.85,.009),'white',.003)
  rod('Board fin',(0,.34,.035),(0,.38,-.045),.065,.015,'navy')

def snorlax(big=False):
 sphere('Broad seated body',(0,0,.39),(.40,.30,.39),'navy');belly((0,-.254,.41),(.31,.065,.28),'cream')
 sphere('Sleepy head',(0,-.025,.77),(.28,.23,.24),'navy');muzzle((0,-.21,.76),(.23,.045,.17),'cream')
 for s in (-1,1):
  rod('Pointed ear',(s*.19,.0,.90),(s*.245,.0,1.08),.10,.005,'navy')
  sphere('Foot sole',(s*.25,-.21,.09),(.16,.18,.08),'cream');sphere('Foot pad',(s*.25,-.29,.12),(.081,.055,.026),'tan')
  sphere('Resting arm',(s*.38,-.01,.43),(.12,.15,.24),'navy')
  tube('Closed sleepy eye',[(s*.055,-.256,.80),(s*.12,-.257,.811),(s*.17,-.241,.799)],.010,'ink')
  rod('Small fang',(s*.080,-.255,.675),(s*.080,-.26,.642),.016,.001,'white')
 smile(-.255,.67,.092)

def lapras():
 sphere('Long swimming body',(0,.09,.25),(.36,.45,.20),'blue')
 for s in (-1,1):
  o=sphere('Front paddle',(s*.32,-.12,.12),(.25,.14,.065),'blue');o.rotation_euler[2]=s*-.40
  o=sphere('Rear flipper',(s*.25,.40,.13),(.20,.11,.045),'blue');o.rotation_euler[2]=s*.30
 sphere('Shell saddle',(0,.15,.38),(.31,.34,.14),'stone');torus('Shell rim',(0,.15,.36),.31,.027,'cream',scale=(1,1.09,1))
 for x,y in ((-.14,.05),(.13,.04),(0,.26),(-.14,.32),(.15,.30)):rod('Shell boss',(x,y,.45),(x,y,.58),.068,.012,'stone_light')
 tube('Long graceful neck',[(0,-.21,.27),(0,-.37,.45),(0,-.40,.73),(0,-.35,.89)],.12,'blue')
 sphere('Gentle head',(0,-.37,.92),(.18,.22,.145),'blue');muzzle((0,-.535,.86),(.135,.08,.063),'cream');eyes(0,-.520,.962,.019,.104)
 rod('Forehead horn',(0,-.39,1.03),(0,-.40,1.20),.043,.001,'cream')
 for s in (-1,1):tube('Curled ear',[(s*.14,-.28,.96),(s*.25,-.25,1.04),(s*.26,-.27,1.12),(s*.20,-.29,1.12)],.020,'blue')

def onix():
 chain=[((.20,.25,.12),(.11,.13,.11)),((.18,.30,.29),(.16,.16,.16)),((.12,.24,.49),(.19,.18,.18)),((-.12,.16,.64),(.22,.21,.22)),((-.32,.06,.82),(.24,.22,.23)),((-.27,-.03,1.12),(.28,.25,.27)),((0,-.08,1.38),(.31,.27,.29))]
 for p,s in chain:ico('Segmented angular rock body',p,s,'stone',1)
 prism('Angular stone jaw',[(-.24,1.28),(.24,1.28),(.17,1.08),(-.18,1.08)],-.16,.29,'stone_dark')
 eyes(0,-.312,1.42,.027,.12,True);rod('Rock head horn',(0,-.06,1.58),(0,-.09,1.91),.12,.002,'stone_light',5)
 tube('Mouth crack',[(-.18,-.317,1.24),(0,-.327,1.21),(.18,-.317,1.24)],.012,'ink')

def beast(kind):
 color={'entei':'brown','raikou':'yellow','suicune':'blue'}[kind]
 sphere('Powerful feline body',(0,.10,.44),(.25,.37,.25),color)
 for s in (-1,1):
  for y in (-.12,.35):
   rod('Beast planted leg',(s*.185,y,.40),(s*.205,y-.025,.08),.090,.065,color)
   sphere('Broad paw',(s*.205,y-.09,.065),(.095,.14,.067),'cream' if kind=='entei' else color)
   for dx in (-.03,.03):tube('Toe division',[(s*.205+dx,y-.21,.056),(s*.205+dx,y-.16,.083)],.007,'ink')
 sphere('Neck',(0,-.18,.64),(.22,.21,.24),color);sphere('Legendary beast head',(0,-.30,.78),(.22,.19,.20),color)
 muzzle((0,-.448,.70),(.16,.075,.08),'cream');eyes(0,-.451,.827,.028,.090,True);sphere('Dark nose',(0,-.514,.747),(.055,.02,.030),'ink')
 if kind=='entei':
  for i in range(9):
   a=i*math.tau/9;ico('Lion mane lobe',(.27*math.cos(a),-.115,.69+.28*math.sin(a)),(.15,.16,.16),'cream' if i<5 else 'tan',1)
  prism('Red forehead crest',[(-.16,.88),(-.25,1.02),(-.10,1.02),(0,.94),(.10,1.02),(.25,1.02),(.16,.88)],-.43,.07,'red')
  prism('Golden muzzle mask',[(-.15,.79),(-.08,.85),(0,.77),(.08,.85),(.15,.79),(.12,.68),(0,.72),(-.12,.68)],-.475,.035,'gold')
  for side in (-1,1):rod('White cheek blade',(side*.17,-.32,.82),(side*.35,-.29,.91),.080,.002,'white')
  for i in range(4):sphere('Rolling smoke plume',((i-1.5)*.085,.24+i*.09,.71),(.13,.19,.105),'stone_light')
  tube('Heavy tail',[(0,.38,.40),(0,.58,.42),(.08,.61,.70)],.06,'brown')
 elif kind=='raikou':
  for s in (-1,1):
   ear(s,(.17,-.25,.96),(.09,.07,.13),'yellow','ink')
   rod('Sabre tooth',(s*.095,-.492,.69),(s*.094,-.486,.49),.037,.008,'white')
   for y in (-.02,.18,.36):prism('Jagged tiger flank stripe',[(s*.17,.63),(s*.24,.52),(s*.17,.43),(s*.26,.40),(s*.28,.52),(s*.23,.63)],y,.040,'ink')
  for i in range(5):sphere('Purple storm mane',((i%2-.5)*.15,.10+i*.07,.77),(.16,.17,.11),'purple')
  prism('Black face mask',[(-.20,.90),(-.06,.91),(0,.84),(.06,.91),(.20,.90),(.14,.76),(.05,.79),(0,.75),(-.05,.79),(-.14,.76)],-.445,.022,'ink')
  prism('Lightning tail',[(.03,.4),(.16,.42),(.20,.66),(.38,.62),(.33,.90),(.23,.87),(.27,.73),(.12,.75)],.48,.045,'blue')
 else:
  for s in (-1,1):
   prism('Diamond crown',[(s*.04,.92),(s*.19,1.13),(s*.08,1.34),(s*.04,1.24),(s*.12,1.12),(s*.01,.98)],-.30,.07,'teal')
   for j in range(2):prism('Diamond flank spot',[(s*.23,.48+j*.12),(s*.265,.54+j*.12),(s*.23,.60+j*.12),(s*.20,.54+j*.12)],.15+j*.13,.028,'white')
   tube('Streaming ribbon tail',[(s*.14,.39,.46),(s*.43,.50,.43),(s*.48,.68,.61),(s*.34,.74,.76),(s*.36,.91,.66)],.027,'white')
  for i in range(6):sphere('Flowing purple mane',(math.sin(i)*.10,.04+i*.105,.79-i*.04),(.21,.17,.16),'purple')

def bat():
 sphere('Bat pear body',(0,0,.30),(.10,.11,.17),'blue');sphere('Bat head',(0,-.012,.47),(.12,.12,.12),'blue')
 for s in (-1,1):
  prism('Scalloped bat wing',[(s*.08,.41),(s*.30,.66),(s*.52,.58),(s*.54,.31),(s*.41,.37),(s*.34,.23),(s*.24,.32),(s*.13,.21)],.012,.035,'purple')
  tube('Blue leading wing bone',[(s*.08,0,.41),(s*.30,0,.66),(s*.52,0,.58),(s*.54,0,.31)],.020,'blue')
  rod('Wing finger',(s*.30,.005,.62),(s*.34,.005,.23),.012,.006,'blue')
  rod('Tall bat ear',(s*.065,0,.54),(s*.10,0,.73),.048,.007,'blue')
  rod('Bat foot',(s*.045,0,.19),(s*.065,-.02,.08),.015,.010,'blue')
  rod('Fang',(s*.035,-.118,.44),(s*.035,-.12,.37),.017,.002,'white')
 eyes(0,-.124,.49,.018,.053)

def blob():
 for p,s in [((0,0,.12),(.35,.29,.12)),((-.03,0,.27),(.28,.22,.19)),((-.04,-.02,.43),(.22,.18,.17))]:sphere('Mounded sludge fold',p,s,'purple')
 for s in (-1,1):
  tube('Raised sludge arm',[(s*.20,0,.24),(s*.34,-.03,.35),(s*.37,-.03,.48)],.075,'purple')
  for k in range(3):rod('Sludge finger',(s*.36,-.03,.45),(s*(.29+k*.07),-.05,.54+k*.02),.030,.015,'purple')
 eyes(-.04,-.193,.47,.022,.09,True);box('Wide sludge mouth',(-.035,-.202,.37),(.20,.025,.06),'ink',.02)

def bug():
 sphere('Wing case',(0,.10,.31),(.23,.28,.21),'green');sphere('Thorax',(0,-.15,.27),(.15,.16,.16),'darkgreen');sphere('Beetle head',(0,-.25,.36),(.17,.14,.14),'green')
 tube('Wing case seam',[(0,-.11,.43),(0,.05,.51),(0,.24,.47),(0,.37,.36)],.012,'ink');eyes(0,-.38,.40,.02,.08)
 for s in (-1,1):
  for j in range(3):tube('Jointed insect leg',[(s*.14,-.09+j*.14,.25),(s*.33,-.15+j*.17,.19),(s*.39,-.18+j*.18,.035)],.020,'ink')
  tube('Clubbed antenna',[(s*.10,-.25,.43),(s*.17,-.34,.58),(s*.23,-.34,.62)],.016,'darkgreen');sphere('Antenna club',(s*.23,-.34,.62),(.037,.03,.037),'yellow')

def bulbasaur():
 sphere('Stocky low body',(0,.07,.23),(.25,.30,.19),'teal')
 for x in (-.17,.17):
  for y in (-.14,.24):sphere('Short square limb',(x,y,.075),(.095,.11,.085),'teal')
 sphere('Wide frog head',(0,-.23,.29),(.28,.22,.21),'teal')
 for s in (-1,1):
  rod('Pointed ear',(s*.20,-.15,.39),(s*.22,-.13,.55),.083,.004,'teal')
  prism('Red eye',[(s*.07,.38),(s*.20,.40),(s*.20,.29),(s*.10,.28)],-.428,.022,'red')
  sphere('Eye pupil',(s*.145,-.448,.34),(.018,.013,.041),'white')
  for x,z in ((s*.225,.24),(s*.11,.44)):sphere('Dark skin spot',(x,-.414,z),(.036,.012,.028),'darkgreen')
 smile(-.447,.20,.12)
 sphere('Bulb heart',(0,.17,.47),(.235,.24,.23),'green')
 for i in range(6):
  a=i*math.tau/6;leaf('Overlapping pointed bulb leaf',(.20*math.cos(a),.17+.20*math.sin(a),.37),(0,.17,.77),.105,'leaf' if i%2 else 'green')

def caterpillar():
 for j in range(5):
  r=.14-j*.012;sphere('Caterpillar segment',(0,.10+j*.15,r), (r,r,r),'green' if j%2 else 'leaf')
  for s in (-1,1):sphere('Tiny caterpillar foot',(s*.10,.10+j*.15,.04),(.045,.055,.038),'yellow')
 sphere('Large caterpillar face',(0,-.065,.22),(.19,.18,.20),'green');muzzle((0,-.228,.17),(.12,.045,.065),'yellow');eyes(0,-.23,.285,.027,.087,True)
 for s in (-1,1):rod('Red forked feeler',(s*.035,-.055,.38),(s*.11,-.05,.57),.036,.015,'red')

def charmander():
 feet('orange',.13,s=(.09,.13,.055));sphere('Lizard body',(0,0,.30),(.20,.15,.25),'orange');belly((0,-.13,.28),(.13,.035,.19),'cream')
 sphere('Lizard head',(0,-.02,.59),(.21,.18,.20),'orange');muzzle((0,-.176,.52),(.16,.08,.073),'orange');eyes(0,-.179,.64,.028,.09,True)
 for s in (-1,1):sphere('Short lizard arm',(s*.19,-.01,.32),(.062,.067,.14),'orange')
 tube('Curved fire tail',[(0,.11,.18),(.21,.28,.15),(.35,.27,.29),(.36,.22,.45)],.065,'orange')
 sphere('Flame',( .36,.22,.54),(.080,.060,.15),'orange');sphere('Inner flame',(.36,.175,.53),(.045,.025,.10),'yellow');rod('Flame tip',(.36,.22,.62),(.33,.22,.76),.052,.002,'yellow')

def diglett():
 for i in range(8):
  a=i*math.tau/8;ico('Disturbed soil clod',(.23*math.cos(a),.20*math.sin(a),.055),(.11,.095,.065),'wood',1)
 sphere('Emerging capsule body',(0,0,.27),(.19,.17,.27),'brown');eyes(0,-.162,.38,.017,.068);sphere('Oval pink nose',(0,-.184,.28),(.068,.033,.040),'pink')

def egg():
 o=sphere('Egg shell',(0,0,.34),(.25,.25,.34),'cream',24,12)
 for v in o.data.vertices:
  t=(v.co.z+.34)/.68;v.co.x*=1-.22*t;v.co.y*=1-.22*t
 for x,y,z,s in [(-.11,-.197,.28,.065),(.095,-.16,.48,.055),(.19,-.03,.23,.065),(-.13,.17,.37,.053),(0,.19,.16,.045)]:sphere('Raised green egg marking',(x,y,z),(s,s*.30,s*.83),'green',12,6)

def equine():
 sphere('Pony barrel',(0,.05,.39),(.19,.30,.20),'cream')
 for s in (-1,1):
  for y in (-.12,.25):rod('Pony leg',(s*.13,y,.38),(s*.14,y,.055),.052,.037,'cream');box('Hoof',(s*.14,y-.015,.04),(.086,.105,.065),'brown',.016)
 rod('Upright neck',(0,-.17,.39),(0,-.29,.71),.13,.09,'cream');sphere('Pony head',(0,-.34,.75),(.14,.20,.16),'cream');muzzle((0,-.50,.68),(.125,.09,.085),'tan');eyes(0,-.46,.80,.021,.090)
 for s in (-1,1):ear(s,(.09,-.27,.91),(.045,.052,.14),'cream')
 for i in range(5):ico('Flame mane',(0,-.15+i*.065,.73-i*.053),(.09,.10,.14),'orange',1)
 tube('Flame tail',[(0,.30,.43),(0,.50,.42),(.10,.57,.59)],.067,'orange');rod('Tail flame tongue',(.10,.57,.58),(.12,.54,.76),.065,.001,'yellow')

def fighter(humanshape=False):
 col='pink' if humanshape else 'stone';feet(col,.135,s=(.082,.14,.055));sphere('Muscular torso',(0,0,.34),(.23,.15,.21),col);sphere('Head',(0,0,.62),(.19,.16,.19),col)
 for s in (-1,1):
  rod('Bent upper arm',(s*.19,0,.42),(s*.31,0,.35),.065,.066,col);rod('Raised forearm',(s*.31,0,.35),(s*.36,-.03,.53),.060,.075,col);sphere('Clenched fist',(s*.36,-.03,.55),(.09,.08,.08),col)
  rod('Strong leg',(s*.10,0,.23),(s*.135,-.02,.06),.071,.056,col)
 eyes(0,-.157,.66,.023,.083,True);smile(-.156,.55,.069)
 if humanshape:
  for s in (-1,1):ear(s,(.16,0,.79),(.075,.05,.12),'pink','brown')
  belly((0,-.144,.33),(.14,.025,.125),'cream')
 else:
  for x in (-.075,0,.075):prism('Cranial ridge',[(x-.025,.75),(x-.020,.92),(x+.025,.87),(x+.025,.75)],.025,.10,'stone_light')
  for z in (.34,.40):tube('Chest definition',[(-.13,-.152,z),(0,-.17,z-.01),(.13,-.152,z)],.009,'stone_dark')

def fish():
 o=sphere('Deep fish body',(0,0,.30),(.24,.14,.26),'orange');sphere('Puckered mouth',(0,-.136,.26),(.080,.048,.072),'pink');torus('Mouth rim',(0,-.18,.26),.065,.017,'cream',(math.pi/2,0,0))
 eyes(0,-.137,.38,.026,.127,True)
 prism('Dorsal fin',[(-.13,.47),(-.05,.69),(.025,.59),(.10,.66),(.14,.44)],.02,.04,'yellow')
 for s in (-1,1):
  prism('Side fan fin',[(s*.18,.32),(s*.40,.45),(s*.39,.18),(s*.20,.24)],0,.038,'cream')
  tube('Barbel',[(s*.08,-.18,.25),(s*.16,-.19,.18),(s*.22,-.18,.22)],.012,'yellow')
 prism('Forked tail',[(-.15,.22),(-.21,.02),(0,.10),(.21,.02),(.15,.22)],.13,.055,'yellow')
 for side in (-1,1):
  for z in (.24,.34,.44):tube('Scale arc',[(side*.16,-.105,z-.025),(side*.20,-.09,z),(side*.17,-.07,z+.02)],.008,'gold')

def fox():
 sphere('Fox body',(0,.02,.25),(.21,.26,.20),'brown')
 for x in (-.14,.14):
  for y in (-.14,.19):rod('Fox leg',(x,y,.25),(x,y,.045),.055,.045,'brown')
 sphere('Fox head',(0,-.16,.49),(.20,.18,.19),'brown');muzzle((0,-.316,.42),(.14,.09,.080),'cream');eyes(0,-.305,.53,.025,.08)
 sphere('Black nose',(0,-.40,.45),(.029,.022,.024),'ink')
 for s in (-1,1):
  prism('Tall fox ear',[(s*.08,.59),(s*.20,.90),(s*.24,.57)],-.06,.095,'brown');prism('Cream ear inset',[(s*.13,.63),(s*.195,.79),(s*.208,.61)],-.115,.015,'cream')
 for i in range(5):
  a=(i-2)*.31;end=(.36*math.sin(a),.38+.1*math.cos(a),.46+abs(i-2)*.02)
  tube('Fluffy fan tail',[(0,.18,.22),(.28*math.sin(a),.36,.28),end],.080,'brown');sphere('Cream tail tip',end,(.085,.085,.10),'cream')

def geodude():
 o=ico('Floating rocky head',(0,0,.34),(.29,.22,.24),'stone',2)
 eyes(0,-.212,.40,.027,.11,True);box('Rock mouth',(0,-.231,.25),(.16,.025,.024),'ink',.004)
 for s in (-1,1):
  tube('Bent rock arm',[(s*.21,0,.33),(s*.37,0,.31),(s*.45,0,.48)],.058,'stone');ico('Raised stone fist',(s*.45,-.01,.52),(.10,.10,.10),'stone_light',1)
  for dx in (-.035,.025):tube('Fist knuckle seam',[(s*.45+dx,-.101,.51),(s*.45+dx,-.096,.56)],.006,'stone_dark')

def ghost():
 sphere('Ghost floating body',(0,0,.37),(.29,.20,.28),'purple')
 for s in (-1,1):
  prism('Ghost ear spike',[(s*.12,.55),(s*.28,.79),(s*.28,.44)],.025,.09,'purple')
  prism('Ghost claw arm',[(s*.22,.42),(s*.43,.45),(s*.35,.34),(s*.42,.28),(s*.23,.28)],-.015,.09,'purple')
  prism('Red slanted eye',[(s*.055,.46),(s*.23,.50),(s*.18,.39),(s*.08,.40)],-.201,.02,'red')
 prism('Toothy grin',[(-.19,.31),(0,.26),(.19,.31),(.12,.20),(-.11,.20)],-.211,.025,'white')
 for x in (-.09,-.03,.03,.09):box('Tooth separation',(x,-.228,.248),(.008,.010,.085),'purple',0)
 for x,z in ((-.13,.10),(.04,.04),(.17,.11)):rod('Tattered ghost base',(x,0,.22),(x,-.02,z),.10,.002,'purple')

def serpent(gyarados=False):
 col='blue' if gyarados else 'purple'
 pts=[(.14,.26,.07),(-.17,.22,.08),(-.25,.0,.11),(0,-.03,.18),(.21,.08,.25),(.15,.10,.46),(0,.025,.69),(0,-.02,.84)]
 tube('Coiled serpent body',pts,.071 if gyarados else .085,col)
 if gyarados:
  for i,p in enumerate(pts[2:-1]):
   sphere('Armored serpent segment',p,(.125,.12,.13),'blue');rod('Dorsal white spike',p,(p[0],p[1]+.14,p[2]+.12),.065,.002,'white')
  sphere('Dragon serpent head',(0,-.075,.85),(.18,.18,.19),'blue');muzzle((0,-.237,.76),(.13,.06,.14),'cream');box('Gaping mouth',(0,-.300,.75),(.16,.018,.17),'ink',.035)
  for s in (-1,1):
   rod('Jaw fang',(s*.057,-.315,.82),(s*.057,-.315,.75),.021,.002,'white')
   rod('Head trident horn',(s*.10,-.01,.97),(s*.20,.015,1.20),.059,.002,'blue')
   tube('Long whisker',[(s*.15,-.16,.79),(s*.31,-.22,.72),(s*.35,-.20,.87)],.018,'yellow')
  rod('Center horn',(0,-.02,1.0),(0,-.02,1.27),.056,.002,'blue');eyes(0,-.233,.94,.024,.094,True)
 else:
  sphere('Rounded snake head',(0,-.06,.86),(.14,.17,.15),col);eyes(0,-.210,.90,.024,.07);sphere('Pale jaw',(0,-.156,.78),(.11,.075,.045),'cream')
  tube('Forked tongue',[(0,-.22,.785),(0,-.30,.765),(-.035,-.33,.765)],.009,'red');tube('Tongue second fork',[(0,-.30,.765),(.035,-.33,.765)],.009,'red')
  for z in (.39,.49,.60):sphere('Ventral neck plate',(.12 if z<.5 else .05,-.010,z),(.07,.018,.032),'cream')

def legendary_bird(lugia=False):
 col='white' if lugia else 'red';sphere('Large bird breast',(0,0,.42),(.22,.19,.30),col);belly((0,-.161,.37),(.15,.035,.23),'cream')
 rod('Long bird neck',(0,0,.57),(0,-.09,.85),.11,.073,col);sphere('Bird head',(0,-.09,.89),(.14,.15,.15),col)
 rod('Beak',(0,-.213,.86),(0,-.34,.84),.049,.005,'navy' if lugia else 'gold');eyes(0,-.205,.94,.022,.071)
 feet('navy' if lugia else 'gold',.11,s=(.06,.13,.045))
 for s in (-1,1):
  tube('Wing leading arm',[(s*.14,0,.59),(s*.35,-.015,.73),(s*.54,-.02,.69)],.055,col)
  for i in range(5):
   x=s*(.33+i*.08);z=.64-i*.045
   if lugia:rod('Finger feather',(s*.30,0,.69),(x,-.01,z-.31),.046,.019,'white')
   else:
    rod('Golden flight feather',(s*.30,0,.70),(x,0,z-.28),.052,.020,'gold');rod('Green feather tip',(s*(.30+i*.057),0,.50-i*.035),(x,0,z-.30),.037,.012,'green')
  if lugia:
   prism('Blue eye mask',[(s*.055,.965),(s*.16,1.04),(s*.11,.90)],-.18,.025,'navy')
  else:
   for i in range(3):rod('Head crown plume',(s*.03,-.03,.98),(s*(.10+i*.04),.02,1.12+i*.04),.025,.002,'gold')
 for i in range(3 if lugia else 5):
  x=(i-(1 if lugia else 2))*.07;rod('Long tail feather',(0,.13,.25),(x,.52,.10),.060,.026,'navy' if lugia else 'gold')
 if lugia:
  for z,y in ((.34,.21),(.49,.18),(.65,.09)):prism('Blue back plate',[(-.085,z-.06),(0,z+.07),(.085,z-.06)],y,.045,'navy')

def jellyfish():
 sphere('Blue jellyfish bell',(0,0,.52),(.26,.22,.22),'blue')
 for s in (-1,1):sphere('Ruby bell jewel',(s*.135,-.16,.61),(.079,.050,.094),'red')
 sphere('Center jewel',(0,-.218,.53),(.052,.025,.05),'red');sphere('Black lower face',(0,-.02,.355),(.15,.135,.11),'ink');eyes(0,-.139,.36,.020,.074,True)
 for s in (-1,1):tube('Long supple tentacle',[(s*.12,.04,.37),(s*.20,.06,.23),(s*.22,-.02,.08),(s*.35,-.10,.07)],.035,'stone_light')

def moth():
 sphere('Moth thorax',(0,0,.33),(.080,.095,.22),'purple');sphere('Moth head',(0,-.03,.57),(.11,.10,.105),'purple');eyes(0,-.115,.60,.035,.052,True)
 for s in (-1,1):
  for upper in (True,False):
   outline=[(s*.055,.47),(s*.30,.76),(s*.50,.71),(s*.46,.42),(s*.15,.29)] if upper else [(s*.08,.32),(s*.37,.40),(s*.42,.14),(s*.30,.065),(s*.12,.16)]
   prism('Cream patterned wing',outline,.035,.035,'cream')
   tube('Dark wing perimeter',[(x,.010,z) for x,z in outline+[outline[0]]],.019,'ink')
   sphere('Wing eye spot',(s*(.30 if upper else .28),.008,.52 if upper else .24),(.08,.018,.093),'blue')
   sphere('Wing spot center',(s*(.30 if upper else .28),-.008,.52 if upper else .24),(.042,.012,.050),'ink')
  tube('Curved antenna',[(s*.04,-.01,.64),(s*.10,-.02,.77),(s*.18,-.02,.79)],.014,'ink')
  rod('Moth leg',(s*.05,-.025,.20),(s*.12,-.07,.09),.014,.008,'ink')

def oddish():
 feet('blue',.11,s=(.10,.13,.045));sphere('Round root',(0,0,.25),(.22,.18,.22),'navy');eyes(0,-.18,.28,.019,.068);smile(-.182,.20,.038)
 for i,(x,y,z) in enumerate([(-.38,0,.75),(-.23,.15,.89),(0,.05,.93),(.23,.14,.83),(.39,-.01,.71)]):leaf('Broad radish leaf',(0,0,.43),(x,y,z),.10,'leaf' if i%2 else 'green')

def poliwag():
 feet('blue',.11,s=(.065,.12,.04));sphere('Tadpole round body',(0,0,.28),(.26,.20,.25),'blue');belly((0,-.193,.27),(.205,.025,.18),'white')
 pts=[]
 for i in range(75):
  a=i/74*math.pi*4.8;r=.14*(1-i/84);pts.append((r*math.cos(a),-.222,.27+r*math.sin(a)))
 tube('Continuous belly spiral',pts,.011,'ink');eyes(0,-.157,.47,.024,.085,True);sphere('Pink round mouth',(0,-.205,.45),(.038,.026,.025),'pink')
 leaf('Transparent-look broad tail',(0,.14,.23),(.22,.65,.29),.21,'cream');tube('Tail central vein',[(0,.14,.23),(.12,.41,.25),(.22,.65,.29)],.016,'blue')

def shell():
 for s in (-1,1):
  o=sphere('Clam shell valve',(0,s*.05,.29+s*.06),(.29,.21,.14),'lilac');o.rotation_euler[0]=s*-.36
  for i in range(5):
   x=(i-2)*.085;tube('Shell growth rib',[(x*.65,s*.015,.41+s*.055),(x,s*.16,.36+s*.055),(x*.8,s*.22,.26+s*.07)],.012,'purple')
 box('Shadow between valves',(0,-.18,.26),(.41,.07,.075),'ink',.025);eyes(0,-.227,.275,.022,.09,True)
 sphere('Protruding tongue',(0,-.24,.175),(.083,.16,.045),'pink')
 for s in (-1,1):rod('Shell pointed rim',(s*.25,0,.26),(s*.35,-.07,.20),.05,.003,'lilac')

def squirtle():
 feet('blue',.13,s=(.09,.14,.05));sphere('Turtle body',(0,0,.28),(.22,.17,.24),'blue');sphere('Shell dome',(0,.10,.29),(.245,.135,.25),'brown');torus('Shell cream rim',(0,.055,.29),.226,.022,'cream',(math.pi/2,0,0),(.98,1.12,1))
 belly((0,-.146,.28),(.16,.032,.20),'cream')
 for z in (.18,.29,.40):tube('Belly shell panel seam',[(-.13,-.164,z),(0,-.181,z-.01),(.13,-.164,z)],.008,'tan')
 sphere('Round turtle head',(0,-.015,.61),(.225,.19,.20),'blue');eyes(0,-.19,.66,.032,.091,True);smile(-.18,.51,.08)
 for s in (-1,1):sphere('Little turtle arm',(s*.21,-.02,.32),(.075,.095,.145),'blue')
 tube('Curled turtle tail',[(0,.19,.16),(.17,.26,.15),(.26,.25,.23),(.21,.24,.32),(.15,.23,.29)],.030,'blue')

def staryu():
 outline=[]
 for i in range(10):
  a=math.pi/2+i*math.pi/5;r=.40 if i%2==0 else .175;outline.append((math.cos(a)*r,.39+math.sin(a)*r))
 prism('Five pointed star body',outline,0,.12,'tan')
 for i in range(5):
  a=math.pi/2+i*math.tau/5;rod('Star raised radial ridge',(0,-.069,.39),(.31*math.cos(a),-.069,.39+.31*math.sin(a)),.045,.011,'gold',6)
 rod('Gem gold surround',(0,-.065,.39),(0,-.12,.39),.15,.15,'gold',10);ico('Ruby central gem',(0,-.14,.39),(.10,.045,.10),'red',2)

def unown():
 torus('Unown eye ring',(0,0,.43),.165,.045,'ink',(math.pi/2,0,0));sphere('Large white eye',(0,-.010,.43),(.13,.047,.13),'white');sphere('Single pupil',(0,-.06,.43),(.055,.018,.061),'ink')
 rod('Upper stalk',(0,0,.61),(0,0,.83),.029,.029,'ink');rod('Lower stalk',(0,0,.25),(0,0,.08),.029,.029,'ink');rod('Letter foot',(-.12,0,.06),(.12,0,.06),.03,.03,'ink')

def surf_mount():
 # The source is the generic aquatic mount, not an inferred species or rider.
 sphere('Broad floating mount',(0,.08,.18),(.37,.45,.18),'blue');tube('Upright aquatic neck',[(0,-.19,.20),(0,-.35,.40),(0,-.37,.65)],.115,'blue');sphere('Mount head',(0,-.38,.70),(.20,.22,.155),'blue');muzzle((0,-.565,.65),(.15,.064,.068),'cream');eyes(0,-.546,.755,.022,.10)
 for s in (-1,1):
  rod('Head horn',(s*.145,-.31,.79),(s*.19,-.27,.94),.052,.005,'cream');sphere('Swim paddle',(s*.35,.05,.10),(.20,.16,.060),'blue')
 sphere('Recessed saddle',(0,.06,.355),(.20,.23,.04),'navy');tube('Crescent tail',[(0,.37,.18),(0,.59,.19),(.12,.67,.28)],.075,'blue')

def chikorita():
 sphere('Chikorita pear body',(0,.025,.28),(.23,.29,.24),'leaf')
 for x in (-.16,.16):
  for y in (-.17,.21):sphere('Chikorita rounded foot',(x,y,.075),(.075,.095,.077),'leaf')
 sphere('Chikorita round head',(0,-.22,.49),(.21,.18,.21),'leaf');eyes(0,-.390,.55,.025,.091,True);smile(-.396,.42,.056)
 for i in range(7):
  a=i*math.tau/7;sphere('Neck seed bud',(.19*math.cos(a),-.135+.115*math.sin(a),.35),(.038,.034,.049),'darkgreen')
 tube('Head leaf stem',[(0,-.19,.65),(.01,-.16,.75),(.07,-.08,.86)],.028,'darkgreen')
 leaf('Sweeping head leaf',(.04,-.12,.77),(.40,.20,1.12),.20,'green');tube('Leaf central vein',[(.04,-.13,.78),(.22,.04,.96),(.40,.20,1.12)],.014,'leaf')

def cyndaquil():
 # A single connected shrew silhouette, with a fitted midnight dorsal coat
 # and folded flame quills. All surfaces are original full 360-degree geometry.
 import bmesh
 PALETTE.update({'cyndaquil_cream':(.94,.83,.56),'cyndaquil_back':(.14,.30,.34),
  'cyndaquil_ink':(.095,.12,.15),'cyndaquil_ember':(.86,.18,.12),
  'cyndaquil_flame':(.98,.47,.075),'cyndaquil_gold':(1.,.78,.22)})
 cream='cyndaquil_cream';ink='cyndaquil_ink'
 def mesh_part(name,vertices,faces,color,smooth=True):
  me=bpy.data.meshes.new(name);me.from_pydata(vertices,[],faces);me.update()
  bm=bmesh.new();bm.from_mesh(me);bmesh.ops.recalc_face_normals(bm,faces=list(bm.faces));bm.to_mesh(me);bm.free()
  o=bpy.data.objects.new(name,me);COL.objects.link(o);me.materials.append(material(color))
  for f in me.polygons:f.use_smooth=smooth
  return o
 def canonical(o):
  me=o.data;coordinates=[tuple(round(v,7) for v in vert.co) for vert in me.vertices]
  ordered=sorted(set(coordinates));lookup={v:i for i,v in enumerate(ordered)};remap=[lookup[v] for v in coordinates];faces=[]
  for f in me.polygons:
   indices=tuple(remap[i] for i in f.vertices)
   if len(set(indices))<3:continue
   faces.append((min(indices[i:]+indices[:i] for i in range(len(indices))),f.use_smooth))
  faces.sort();new=bpy.data.meshes.new(me.name+' / stable topology');new.from_pydata(ordered,[],[f[0] for f in faces]);new.update()
  for m in me.materials:new.materials.append(m)
  for f,source in zip(new.polygons,faces):f.use_smooth=source[1]
  o.data=new
 def loft(name,profiles,color,seg=24):
  vertices=[]
  for x,y,z,w,h in profiles:
   for i in range(seg):
    a=math.tau*i/seg;vertices.append((x+w*math.cos(a),y,z+h*math.sin(a)))
  faces=[tuple(range(seg-1,-1,-1)),tuple(range((len(profiles)-1)*seg,len(profiles)*seg))]
  for j in range(len(profiles)-1):
   for i in range(seg):a=j*seg+i;b=j*seg+(i+1)%seg;faces.append((a,b,b+seg,a+seg))
  return mesh_part(name,vertices,faces,color)
 # The pointed muzzle is lofted into the cheeks; the neck falls into a pear
 # torso, without a bead joint or a separate cylindrical snout.
 clay=[loft('Long continuous tapered shrew muzzle',[
  (0,-.592,.520,.024,.027),(0,-.550,.523,.047,.040),
  (0,-.455,.540,.084,.069),(0,-.349,.568,.136,.109),
  (0,-.243,.601,.198,.150),(0,-.144,.613,.220,.180),
  (0,-.045,.589,.198,.170),(0,.034,.532,.158,.138)],cream),
  sphere('Broad connected pear torso',(0,.075,.340),(.226,.230,.293),cream,32,16),
  sphere('Flowing nape and chest',(0,-.047,.491),(.189,.182,.212),cream,24,12)]
 for s in (-1,1):
  clay.extend([sphere('Small connected haunch',(s*.141,.101,.166),(.117,.141,.146),cream,20,10),
   sphere('Grounded long rear paw',(s*.160,.013,.054),(.089,.155,.053),cream,20,10),
   rod('Short descending forelimb',(s*.181,-.085,.385),(s*.223,-.170,.247),.052,.045,cream,16),
   sphere('Soft forelimb elbow',(s*.214,-.141,.289),(.052,.052,.084),cream,16,8),
   sphere('Small rounded forepaw',(s*.223,-.188,.230),(.051,.076,.043),cream,16,8)])
 bpy.ops.object.select_all(action='DESELECT')
 for o in clay:o.select_set(True)
 bpy.context.view_layer.objects.active=clay[0];bpy.ops.object.join();body=bpy.context.object;body.name='Cyndaquil / continuous muzzle head torso and limbs'
 bpy.ops.object.transform_apply(location=False,rotation=False,scale=True)
 mod=body.modifiers.new('Connected clay anatomy','REMESH');mod.mode='VOXEL';mod.voxel_size=.009;mod.use_smooth_shade=True;bpy.ops.object.modifier_apply(modifier=mod.name)
 mod=body.modifiers.new('Soft sculpted anatomical transitions','SMOOTH');mod.factor=.52;mod.iterations=4;bpy.ops.object.modifier_apply(modifier=mod.name)
 canonical(body);high_body=body.data.copy();body.data.calc_loop_triangles()
 mod=body.modifiers.new('Bounded sculpture silhouette','DECIMATE');mod.ratio=min(1,3300/len(body.data.loop_triangles));mod.use_collapse_triangulate=True;bpy.ops.object.modifier_apply(modifier=mod.name);canonical(body);body.select_set(False)
 # The dorsal coat is a closed thin shell fitted to the actual final body.
 # Ring widths carry the continuous dark cap from brow through rounded rump.
 profiles=[(-.404,.051),(-.345,.106),(-.265,.158),(-.175,.198),(-.075,.173),
  (.015,.180),(.105,.212),(.192,.189),(.251,.132),(.281,.061)]
 profiles=[(ay+(by-ay)*t,aw+(bw-aw)*t) for (ay,aw),(by,bw) in zip(profiles,profiles[1:]) for t in (0,1/3,2/3)]+[profiles[-1]]
 vertices=[];inside=[];across=28
 for y,w in profiles:
  for j in range(across+1):
   x=w*(2*j/across-1);hit,p,n,face=body.ray_cast(Vector((x,y,2)),Vector((0,0,-1)))
   if not hit:raise ValueError('Cyndaquil dorsal coat missed continuous body')
   vertices.append(tuple(p+n*.008));inside.append(tuple(p-n*.006))
 n=len(vertices);vertices += inside;faces=[]
 for row in range(len(profiles)-1):
  for j in range(across):
   a=row*(across+1)+j;b=a+1;c=b+across+1;d=a+across+1
   faces.extend([(a,b,c,d),(d+n,c+n,b+n,a+n)])
 perimeter=list(range(across+1))+[r*(across+1)+across for r in range(1,len(profiles))]+list(range(n-2,n-across-2,-1))+[r*(across+1) for r in range(len(profiles)-2,0,-1)]
 for a,b in zip(perimeter,perimeter[1:]+perimeter[:1]):faces.append((a,a+n,b+n,b))
 mesh_part('Fitted midnight brow nape and back',vertices,faces,'cyndaquil_back')
 # Slender, tapered closed eyes rest in the two side cheeks. Their expressive
 # upward sweep reads in front and three-quarter views without raised eyeballs.
 for s in (-1,1):
  points=[]
  for x,z in [(.098,.635),(.135,.641),(.170,.653),(.188,.663)]:
   hit,p,n,face=body.ray_cast(Vector((s*x,-2,z)),Vector((0,1,0)))
   if not hit:raise ValueError('Cyndaquil eyelid missed cheek')
   points.append(tuple(p+n*.004))
  # A closed four-sided ribbon keeps a crisp paper-cut expression and depth.
  v=[]
  for i,p in enumerate(points):
   p=Vector(p);w=[.002,.010,.009,.002][i]
   v.extend([tuple(p+Vector((0,-.003,w))),tuple(p+Vector((0,-.003,-w))),tuple(p+Vector((0,.006,-w))),tuple(p+Vector((0,.006,w)))])
  f=[(3,2,1,0),(12,13,14,15)]
  for j in range(3):
   for k in range(4):a=j*4+k;b=j*4+(k+1)%4;f.append((a,b,b+4,a+4))
  mesh_part('Tapered sleepy closed eyelid',v,f,ink,False)
 nose=ico('Small soft charcoal nose',(0,-.595,.521),(.025,.012,.017),ink,2)
 # Five broad folded quills share the back. Each is a closed, asymmetric
 # flame silhouette with an actual central ridge and a tapered outer edge.
 def quill(name,root,tip,width,depth,bend):
  root=Vector(root);tip=Vector(tip);axis=tip-root;side=Vector((1,0,0));side=(side-axis.normalized()*side.dot(axis.normalized())).normalized()
  forward=side.cross(axis.normalized()).normalized()
  if forward.y>0:forward=-forward
  # Coordinates deliberately articulate a lick, a short shoulder and a long
  # sweeping tip, so the flame is neither a cone nor a row of equal spikes.
  outline=[(-.38,.0),(-.94,.24),(-.78,.49),(-.43,.43),(-.36,.72),
   (bend,1.),(.19,.68),(.63,.53),(.40,.31),(.48,.07)]
  ring=[root+axis*t+side*(x*width) for x,t in outline]
  center=root+axis*.41+forward*depth
  back=root+axis*.41-forward*depth*.58
  vs=[tuple(p) for p in ring]+[tuple(center),tuple(back)];n=len(ring)
  fs=[(i,(i+1)%n,n) for i in range(n)]+[((i+1)%n,i,n+1) for i in range(n)]
  mesh_part(name+' / ember silhouette',vs,fs,'cyndaquil_ember',False)
  # Smaller closed nested fold volumes sit on the front ridge. These are
  # sculptural color layers, not billboards or emissive attack effects.
  for face_label,ridge,outward,thick in [('front',center,forward,depth),('back',back,-forward,depth*.58)]:
   for label,scale,offset,color in [('orange fold',.82,.002,'cyndaquil_flame'),('golden heart',.51,.004,'cyndaquil_gold')]:
    nested=[ridge+(p-ridge)*scale+outward*offset for p in ring]
    peak=ridge+outward*offset
    inner=ridge-outward*(thick*scale+.002)+outward*offset
    nv=[tuple(p) for p in nested]+[tuple(peak),tuple(inner)]
    mesh_part(name+' / '+face_label+' '+label,nv,fs,color,False)
 quill('Upper left flame quill',(-.071,.060,.604),(-.247,.304,1.018),.118,.034,-.12)
 quill('Upper right flame quill',(.071,.060,.604),(.237,.332,.976),.117,.035,.21)
 quill('Lower left flame quill',(-.138,.186,.450),(-.339,.465,.776),.140,.040,-.22)
 quill('Lower right flame quill',(.138,.186,.450),(.333,.488,.752),.141,.040,.22)
 quill('Central rear flame quill',(0,.252,.432),(.024,.587,.891),.142,.041,-.19)
 # Preserve editable detailed anatomy. The high resolution copy and all detail
 # parts get their own true floor root in the same dimensions as runtime.
 detailed=bpy.data.collections.new('battle_cyndaquil / detailed authoring source (hidden)');SCENE.collection.children.link(detailed)
 root=bpy.data.objects.new('battle_cyndaquil / detailed floor root',None);detailed.objects.link(root);root['authoring_only']=True;root['source_family']='battle_cyndaquil'
 copies=[]
 for o in list(COL.objects):
  if o.type!='MESH':continue
  canonical(o);source=o.copy();source.data=high_body if o==body else o.data.copy();source.name='AUTHORING / '+o.name;detailed.objects.link(source);source.parent=root;copies.append(source)
  bm=bmesh.new();bm.from_mesh(o.data);bmesh.ops.triangulate(bm,faces=list(bm.faces),quad_method='FIXED',ngon_method='EAR_CLIP');bmesh.ops.recalc_face_normals(bm,faces=list(bm.faces))
  if bm.calc_volume(signed=True)<0:bmesh.ops.reverse_faces(bm,faces=list(bm.faces))
  bm.to_mesh(o.data);bm.free();o.data.update()
  for attempt in range(3):
   bad=[f for f in o.data.polygons if f.use_smooth and f.normal.dot(sum((o.data.corner_normals[i].vector for i in f.loop_indices),Vector()))<.000001]
   if not bad:break
   for f in bad:f.use_smooth=False
   o.data.update()
  o['authoring']='Original connected Cyndaquil sculpture with fitted dorsal coat and folded volumetric flame quills'
 normalize('battle_cyndaquil',copies)
 detailed.hide_render=True;detailed.hide_viewport=True

def totodile():
 # One full, floor-rooted sculpture. Broad connected anatomy, an authored
 # crocodile muzzle and inset facial planes follow the hero-sculpture kit.
 # These original surfaces read no game pixels, packs or external meshes.
 import bmesh
 PALETTE.update({'totodile_blue':(.25,.64,.76),'totodile_throat':(.22,.58,.70),
  'totodile_red':(.86,.27,.23),'totodile_eye':(.15,.18,.23),
  'totodile_iris':(.62,.19,.15),'totodile_ochre':(.96,.80,.37)})
 blue='totodile_blue'
 def mesh_part(name,vertices,faces,color,smooth=True):
  me=bpy.data.meshes.new(name);me.from_pydata(vertices,[],faces);me.update()
  bm=bmesh.new();bm.from_mesh(me);bmesh.ops.recalc_face_normals(bm,faces=list(bm.faces));bm.to_mesh(me);bm.free()
  o=bpy.data.objects.new(name,me);COL.objects.link(o);me.materials.append(material(color))
  for f in me.polygons:f.use_smooth=smooth
  return o
 def loft(name,profiles,color,axis='Z',seg=20,exponent=.78):
  vertices=[]
  for x,y,z,w,d in profiles:
   for i in range(seg):
    a=math.tau*i/seg;c=math.copysign(abs(math.cos(a))**exponent,math.cos(a));s=math.copysign(abs(math.sin(a))**exponent,math.sin(a))
    vertices.append((x+w*c,y+d*s,z) if axis=='Z' else (x+w*c,y,z+d*s))
  faces=[tuple(range(seg-1,-1,-1)),tuple(range((len(profiles)-1)*seg,len(profiles)*seg))]
  for j in range(len(profiles)-1):
   for i in range(seg):a=j*seg+i;b=j*seg+(i+1)%seg;faces.append((a,b,b+seg,a+seg))
  return mesh_part(name,vertices,faces,color)
 def canonical(o):
  me=o.data;coordinates=[tuple(round(v,7) for v in vert.co) for vert in me.vertices]
  ordered=sorted(set(coordinates));lookup={v:i for i,v in enumerate(ordered)};remap=[lookup[v] for v in coordinates];faces=[]
  for f in me.polygons:
   indices=tuple(remap[i] for i in f.vertices)
   if len(set(indices))<3:continue
   faces.append((min(indices[i:]+indices[:i] for i in range(len(indices))),f.use_smooth))
  faces.sort();new=bpy.data.meshes.new(me.name+' / stable topology');new.from_pydata(ordered,[],[f[0] for f in faces]);new.update()
  for m in me.materials:new.materials.append(m)
  for f,source in zip(new.polygons,faces):f.use_smooth=source[1]
  o.data=new
 def joined(name,parts,voxel=.012):
  bpy.ops.object.select_all(action='DESELECT')
  for o in parts:o.select_set(True)
  bpy.context.view_layer.objects.active=parts[0];bpy.ops.object.join();o=bpy.context.object;o.name=name
  bpy.ops.object.transform_apply(location=False,rotation=False,scale=True)
  mod=o.modifiers.new('Connected clay silhouette','REMESH');mod.mode='VOXEL';mod.voxel_size=voxel;mod.use_smooth_shade=True;bpy.ops.object.modifier_apply(modifier=mod.name)
  mod=o.modifiers.new('Sculpted transitions','SMOOTH');mod.factor=.55;mod.iterations=3;bpy.ops.object.modifier_apply(modifier=mod.name)
  canonical(o);o.select_set(False);return o
 def patch(name,outline,surface,color,depth=.006):
  # Each closed inset follows the real head/body surface; no flat billboards.
  outline=[(a[0]*(1-t)+b[0]*t,a[1]*(1-t)+b[1]*t) for a,b in zip(outline,outline[1:]+outline[:1]) for t in (0,.5)]
  n=len(outline);cx=sum(x for x,z in outline)/n;cz=sum(z for x,z in outline)/n;v=[]
  for offset in (0,depth):
   v.append((cx,surface(cx,cz)+offset,cz))
   for t in (.25,.5,.75,1):
    for x,z in outline:
     px=cx+(x-cx)*t;pz=cz+(z-cz)*t;v.append((px,surface(px,pz)+offset,pz))
  layer=1+4*n;f=[]
  for base,flip in ((0,False),(layer,True)):
   for i in range(n):
    tri=(base,base+1+i,base+1+(i+1)%n);f.append(tuple(reversed(tri)) if flip else tri)
   for ring in range(3):
    for i in range(n):
     a=base+1+ring*n+i;b=base+1+ring*n+(i+1)%n;quad=(a,b,b+n,a+n);f.append(tuple(reversed(quad)) if flip else quad)
  for i in range(n):a=1+3*n+i;b=1+3*n+(i+1)%n;f.append((a,b,b+layer,a+layer))
  return mesh_part(name,v,f,color,False)
 head=[(0,.005,.590,.160,.130),(0,.005,.660,.247,.179),
  (0,.005,.750,.292,.209),(0,.005,.850,.291,.201),
  (0,.010,.943,.249,.165),(0,.016,1.015,.167,.119),
  (0,.018,1.042,.058,.052),(0,.018,1.045,.010,.012)]
 clay=[loft('Broad shaped cranium and cheeks',head,blue),
  loft('Compact pear shaped torso',[(0,.025,.13,.10,.10),(0,.018,.20,.19,.16),
    (0,.008,.34,.211,.173),(0,0,.46,.181,.147),(0,.018,.59,.155,.132),(0,.02,.66,.13,.115)],blue),
  loft('Continuous flattened upper crocodile muzzle',[
   (0,.02,.735,.245,.105),(0,-.14,.736,.254,.105),
   (0,-.29,.727,.255,.088),(0,-.43,.718,.247,.071),
   (0,-.515,.704,.199,.050),(0,-.534,.704,.144,.035)],blue,'Y',24,.56)]
 # Splayed, connected thighs and long planted feet, with a strong three-toe edge.
 for s in (-1,1):
  clay.extend([sphere('Connected haunch',(s*.155,.030,.215),(.117,.136,.162),blue,24,12),
   sphere('Broad planted crocodile foot',(s*.165,-.085,.072),(.126,.191,.071),blue,24,12)])
  for j in (-1,0,1):
   clay.append(sphere('Integrated foot digit',(s*.165+j*.069,-.201-.016*(j==0),.047),(.043,.092,.043),blue,16,8))
  # Elbows turn outward and wrists turn toward the opponent, rather than
  # dropping separate cylinders vertically against the belly.
  clay.extend([rod('Upper reaching arm',(s*.160,.005,.490),(s*.286,-.042,.405),.068,.060,blue,16),
   sphere('Rounded elbow',(s*.284,-.041,.413),(.065,.067,.068),blue,16,8),
   rod('Forearm',(s*.282,-.040,.419),(s*.345,-.134,.453),.062,.065,blue,16),
   sphere('Broad small palm',(s*.351,-.138,.459),(.071,.070,.059),blue,16,8)])
  for j in (-1,0,1):
   clay.append(rod('Three tapered hand digits',(s*(.349+j*.021),-.160,.446+j*.025),(s*(.390+j*.019),-.223,.438+j*.033),.026,.010,blue,10))
 # A tapered rear tail is deliberately visible from both battle perspectives.
 clay.append(loft('Broad curved crocodile tail',[
  (0,.100,.198,.124,.102),(0,.239,.169,.113,.085),
  (.035,.359,.155,.093,.068),(.089,.485,.184,.068,.061),
  (.125,.584,.243,.038,.043),(.123,.640,.296,.009,.010)],blue,'Y',18,.82))
 body=joined('Totodile / continuous head muzzle torso limbs and tail',clay)
 high_body_mesh=body.data.copy();body.data.calc_loop_triangles()
 mod=body.modifiers.new('Bounded connected sculpture topology','DECIMATE');mod.ratio=min(1,3600/len(body.data.loop_triangles));mod.use_collapse_triangulate=True;bpy.context.view_layer.objects.active=body;bpy.ops.object.modifier_apply(modifier=mod.name)
 canonical(body)

 # The lower jaw shares the body blue. A shaped charcoal cavity gives the
 # two ivory fangs a shallow crocodile grin instead of a rectangular box mouth.
 loft('Sculpted rounded lower jaw',[(0,.012,.610,.226,.077),(0,-.165,.601,.239,.064),
  (0,-.332,.559,.245,.041),(0,-.447,.576,.215,.030),(0,-.511,.598,.143,.019)],blue,'Y',24,.64)
 loft('Recessed shallow mouth cavity',[(0,-.087,.661,.194,.039),(0,-.261,.632,.246,.046),
  (0,-.426,.637,.239,.041),(0,-.516,.641,.150,.030)],'totodile_eye','Y',24,.63)
 for s in (-1,1):
  rod('Upper ivory crocodile fang',(s*.184,-.475,.678),(s*.184,-.486,.601),.023,.002,'white',10)
  hit,point,normal,index=body.ray_cast(Vector((s*.147,-.487,2)),Vector((0,0,-1)))
  if not hit:raise ValueError('Totodile nostril missed the muzzle')
  nostril=sphere('Shallow inset nostril',point+normal*.003,(.013,.020,.0035),'totodile_eye',12,6);nostril.rotation_euler=normal.to_track_quat('Z','Y').to_euler()
  # Restrained claws retain the three-digit silhouette at gameplay scale.
  for j in (-1,0,1):rod('Ivory toe claw',(s*.165+j*.069,-.255-.016*(j==0),.052),(s*.165+j*.069,-.295-.016*(j==0),.037),.018,.002,'white',10)
 def head_surface(x,z,offset=.0):
  # Fit the inset to the connected sculpt instead of an approximate sphere.
  hit,point,normal,index=body.ray_cast(Vector((x,-2,z)),Vector((0,1,0)))
  if not hit:raise ValueError('Totodile facial detail missed the head')
  return point.y-offset
 for s in (-1,1):
  # A tapered eye polygon sits inside the upper cheek. Blue lids meet the
  # sclera edges; nothing is perched on the head as a separate eyeball.
  outline=[(s*.095,.847),(s*.104,.916),(s*.146,.970),(s*.190,.979),(s*.238,.939),(s*.263,.867),(s*.227,.837),(s*.160,.831)]
  patch('Inset ivory eye',outline,lambda x,z:head_surface(x,z,.008),'white',.007)
  pupil=[(s*.176,.934),(s*.192,.948),(s*.209,.926),(s*.218,.874),(s*.204,.855),(s*.184,.862)]
  patch('Warm narrow iris',pupil,lambda x,z:head_surface(x,z,.015),'totodile_iris',.005)
  pupil=[(s*.184,.931),(s*.195,.938),(s*.207,.913),(s*.210,.872),(s*.199,.861),(s*.188,.872)]
  patch('Tapered dark pupil',pupil,lambda x,z:head_surface(x,z,.022),'totodile_eye',.004)
  patch('Small eye highlight',[(s*.192,.919),(s*.198,.924),(s*.204,.917),(s*.200,.904)],lambda x,z:head_surface(x,z,.027),'white',.003)
 # The pointed chest marking follows the belly, rather than floating as a slab.
 chest=[(-.137,.461),(-.094,.486),(0,.418),(.094,.486),(.137,.461),(.105,.300),(0,.265),(-.105,.300)]
 def chest_surface(x,z):return -.170*math.sqrt(max(.04,1-(x/.218)**2-((z-.335)/.315)**2))-.008
 patch('Golden chest chevron',chest,chest_surface,'totodile_ochre',.009)
 # A single centered row of thick red dorsal plates is legible from the rear.
 # Their swept triangle silhouettes are fins, with a raised central ridge.
 for i,(outline,thick) in enumerate([
  ([( .100,1.016),(.327,.973),(.191,.861)],.045),
  ([(.187,.850),(.389,.754),(.187,.655)],.048),
  ([(.150,.624),(.355,.510),(.163,.407)],.047),
  ([(.243,.211),(.361,.336),(.422,.168)],.037),
  ([(.446,.226),(.552,.330),(.596,.253)],.025)]):
  # Dorsal fin outline lies in the Y/Z plane and tapers to a crisp outer edge.
  cy=sum(y for y,z in outline)/3;cz=sum(z for y,z in outline)/3
  x=0 if i<3 else (.049 if i==3 else .119)
  vertices=[(x,y,z) for y,z in outline]+[(x-thick,cy,cz),(x+thick,cy,cz)]
  mesh_part('Swept red dorsal plate '+str(i+1),vertices,[(0,1,3),(1,2,3),(2,0,3),(1,0,4),(2,1,4),(0,2,4)],'totodile_red',False)
 # Retain editable detailed anatomy, and simplify only the runtime copy.
 detailed=bpy.data.collections.new('battle_totodile / detailed authoring source (hidden)');SCENE.collection.children.link(detailed)
 root=bpy.data.objects.new('battle_totodile / detailed source root',None);detailed.objects.link(root);root['authoring_only']=True
 for o in list(COL.objects):
  if o.type!='MESH':continue
  canonical(o);source=o.copy();source.data=high_body_mesh if o==body else o.data.copy();source.name='AUTHORING / '+o.name;detailed.objects.link(source);source.parent=root
  canonical(o)
  bm=bmesh.new();bm.from_mesh(o.data);bmesh.ops.triangulate(bm,faces=list(bm.faces),quad_method='FIXED',ngon_method='EAR_CLIP');bmesh.ops.recalc_face_normals(bm,faces=list(bm.faces))
  if bm.calc_volume(signed=True)<0:bmesh.ops.reverse_faces(bm,faces=list(bm.faces))
  bm.to_mesh(o.data);bm.free();o.data.update()
  for attempt in range(3):
   bad=[f for f in o.data.polygons if f.use_smooth and f.normal.dot(sum((o.data.corner_normals[i].vector for i in f.loop_indices),Vector()))<.000001]
   if not bad:break
   for f in bad:f.use_smooth=False
   o.data.update()
  o['authoring']='Original full 360-degree Totodile sculpture; connected clay anatomy and closed inset details'
 detailed.hide_render=True;detailed.hide_viewport=True

def pidgey():
 bird()
 for obj in list(COL.objects):
  if obj.name.startswith('Crest feather'):bpy.data.objects.remove(obj,do_unlink=True)
 for s in (-1,1):
  tube('Cream eyebrow',[(s*.028,-.177,.576),(s*.083,-.16,.605),(s*.135,-.12,.594)],.025,'cream')
  sphere('Dark cheek marking',(s*.115,-.142,.481),(.048,.018,.054),'brown')
 for i in range(3):rod('Distinct swept crown',(0,.01,.61),((i-1)*.055,.155,.735-i*.023),.036,.005,'tan')

def rattata():
 sphere('Purple mouse body',(0,.055,.23),(.21,.28,.20),'purple')
 for x in (-.145,.145):
  for y in (-.13,.24):sphere('Mouse paw',(x,y,.047),(.065,.095,.05),'cream')
 sphere('Rat head',(0,-.20,.39),(.22,.20,.20),'purple');muzzle((0,-.358,.31),(.16,.08,.094),'cream');eyes(0,-.371,.47,.028,.105,True)
 for s in (-1,1):
  sphere('Large round rat ear',(s*.164,-.085,.58),(.11,.065,.12),'purple');sphere('Pink ear inset',(s*.164,-.143,.59),(.072,.012,.081),'pink')
  box('Incisor',(s*.032,-.427,.275),(.051,.020,.065),'white',.007)
  for j in range(2):tube('Whisker',[(s*.12,-.40,.33),(s*.29,-.46,.34+j*.06)],.006,'cream')
 sphere('Rat nose',(0,-.438,.362),(.038,.021,.024),'rose');tube('Thin curled rat tail',[(0,.29,.24),(.16,.41,.22),(.33,.38,.35),(.34,.27,.48),(.27,.23,.49)],.032,'pink')

def sentret():
 feet('brown',.12,s=(.09,.14,.05));sphere('Upright scout body',(0,0,.35),(.25,.19,.30),'brown');belly((0,-.174,.34),(.16,.031,.18),'cream');sphere('Belly ring center',(0,-.207,.34),(.095,.013,.105),'brown')
 sphere('Sentret head',(0,-.025,.67),(.22,.18,.18),'brown');muzzle((0,-.186,.60),(.11,.038,.048),'cream');eyes(0,-.192,.715,.025,.095)
 for s in (-1,1):
  sphere('Tall round scout ear',(s*.165,0,.88),(.089,.064,.17),'brown');sphere('Cream ear center',(s*.17,-.055,.89),(.050,.016,.111),'cream');sphere('Short scout arm',(s*.24,-.008,.40),(.071,.079,.13),'brown')
 tail=[(0,.11,.17),(.16,.29,.16),(.28,.37,.31),(.27,.39,.53),(.19,.39,.69)]
 tube('Large balancing tail',tail,.09,'tan')
 for p in ((.16,.29,.16),(.28,.37,.31),(.25,.39,.54)):sphere('Tail dark ring',p,(.093,.091,.054),'brown')

def hoothoot():
 sphere('Round owl body',(0,0,.37),(.29,.25,.35),'brown');belly((0,-.204,.31),(.21,.05,.22),'cream')
 for s in (-1,1):
  sphere('Cream owl face disk',(s*.126,-.208,.56),(.125,.042,.14),'cream');sphere('Huge red eye',(s*.126,-.248,.58),(.072,.020,.091),'red');sphere('Black watch pupil',(s*.126,-.267,.58),(.028,.008,.060),'ink')
  prism('Clock-hand brow',[(s*.03,.68),(s*.08,.83),(s*.16,.91),(s*.17,.73),(s*.23,.68)],-.16,.048,'ink')
  sphere('Folded owl wing',(s*.267,.03,.34),(.072,.16,.21),'tan')
  for z in (.25,.35,.45):tube('Wing feather edge',[(s*.29,-.07,z),(s*.315,.07,z-.07)],.010,'brown')
 rod('Tiny owl beak',(0,-.26,.49),(0,-.34,.445),.037,.002,'ink')
 rod('Single standing leg',(0,0,.14),(0,-.025,.042),.035,.025,'rose')
 for s in (-1,0,1):rod('Owl spreading toe',(0,-.025,.04),(s*.08,-.15,.025),.018,.011,'rose')

def bicycle_rider(female=False):
 # Import only this repository's original editable character geometry. Runtime
 # skeleton pivots are posed hierarchically before conversion into Blender.
 directory=Path(__file__).resolve().parents[1]/'crates/crystal-voxel-view/models/johto_characters'
 sys.path.insert(0,str(Path(__file__).resolve().parent))
 from animated_glb import load_rig
 rig=load_rig(directory,'trainer_female' if female else 'trainer');transforms=[]
 rotations={'torso':.17,'upper_arm_l':-1.02,'upper_arm_r':-1.02,'forearm_l':-.22,'forearm_r':-.22,'thigh_l':-1.09,'shin_l':1.92,'thigh_r':-1.30,'shin_r':1.70}
 for joint in rig['joints']:
  local=Matrix.Translation(Vector(joint['translation']))@Matrix.Rotation(rotations.get(joint['name'],0),4,'X')
  parent=joint['parent'];world=(transforms[parent]@local) if parent is not None else Matrix.Translation((0,0,-.12))@local;transforms.append(world)
  for index,prim in enumerate(joint['primitives']):
   coords=[]
   for i in range(0,len(prim['positions']),3):
    p=world@Vector(prim['positions'][i:i+3]);coords.append((p.x,-p.z,p.y))
   me=bpy.data.meshes.new('Posed rider '+joint['name']);me.from_pydata(coords,[],[prim['indices'][i:i+3] for i in range(0,len(prim['indices']),3)]);me.update()
   o=bpy.data.objects.new('Rider / '+joint['name']+' / '+str(index),me);COL.objects.link(o)
   key=('rider',tuple(prim['base_color']))
   if key not in MATERIALS:
    m=bpy.data.materials.new('Original rider material');m.diffuse_color=prim['base_color'];m.use_nodes=True;m.node_tree.nodes['Principled BSDF'].inputs['Base Color'].default_value=m.diffuse_color;m.node_tree.nodes['Principled BSDF'].inputs['Roughness'].default_value=.83;MATERIALS[key]=m
   me.materials.append(MATERIALS[key])
   for f in me.polygons:f.use_smooth=True
   custom=[]
   for i in range(0,len(prim['normals']),3):
    n=(world.to_3x3()@Vector(prim['normals'][i:i+3])).normalized();custom.append((n.x,-n.z,n.y))
   me.normals_split_custom_set_from_vertices(custom)
 # Bicycle lies in forward -Y; axle runs X. The fixed source pose has shoes on
 # pedals and hands at the handlebar grips; root translation remains authoritative.
 frame='teal' if female else 'red'
 for y in (-.40,.37):
  torus('Rubber tire',(0,y,.225),.205,.025,'ink',(0,math.pi/2,0));torus('Wheel metal rim',(0,y,.225),.182,.011,'silver',(0,math.pi/2,0))
  rod('Wheel axle',(-.065,y,.225),(.065,y,.225),.027,.027,'silver')
  for i in range(10):
   a=i*math.tau/10;rod('Wheel spoke',(0,y,.225),(0,y+math.cos(a)*.178,.225+math.sin(a)*.178),.005,.005,'silver',6)
 seat=(0,.12,.61);crank=(0,.015,.265);steer=(0,-.33,.67);back=(0,.37,.225);front=(0,-.40,.225)
 for a,b in ((seat,crank),(crank,steer),(steer,seat),(back,seat),(back,crank),(steer,front)):rod('Bicycle frame tube',a,b,.022,.022,frame)
 rod('Saddle post',(0,.12,.52),(0,.12,.655),.016,.016,'silver');sphere('Padded saddle',(0,.12,.665),(.10,.115,.026),'ink')
 rod('Handlebar stem',steer,(0,-.37,.855),.018,.018,'silver');tube('Swept handlebar',[(-.25,-.35,.865),(-.21,-.41,.885),(0,-.42,.887),(.21,-.41,.885),(.25,-.35,.865)],.016,'silver')
 for s in (-1,1):rod('Rubber bar grip',(s*.19,-.37,.875),(s*.27,-.32,.855),.023,.023,'ink')
 torus('Chain ring',(.048,.015,.265),.080,.013,'silver',(0,math.pi/2,0));tube('Chain loop',[(.049,.012,.345),(.049,.37,.255),(.049,.395,.225),(.049,.36,.205),(.049,.015,.19),(.049,-.06,.245),(.049,.012,.345)],.008,'ink')
 for s in (-1,1):
  rod('Pedal crank arm',(s*.058,.015,.265),(s*.075,-s*.060,.265+s*.050),.011,.011,'silver');box('Foot pedal',(s*.115,-s*.060,.265+s*.050),(.12,.07,.023),'ink',.009)

BUILDERS={
 'poke_ball':lambda:ball(), 'boulder':lambda:stone(True), 'rock':lambda:stone(), 'fruit_tree':fruit_tree,
 'paper':paper, 'pokedex':pokedex, 'famicom':lambda:console('famicom'), 'snes':lambda:console('snes'), 'n64':lambda:console('n64'), 'virtual_boy':lambda:console('virtual_boy'),
 'gold_trophy':lambda:trophy(), 'silver_trophy':lambda:trophy(True), 'bird':bird, 'fairy':fairy, 'monster':monster, 'dragon':dragon,
 'slowpoke':slowpoke, 'sudowoodo':sudowoodo, 'entei':lambda:beast('entei'), 'raikou':lambda:beast('raikou'), 'suicune':lambda:beast('suicune'),
 'big_snorlax':snorlax, 'big_lapras':lapras, 'big_onix':onix, 'surf':surf_mount, 'surfing_pikachu':lambda:pikachu(True),
 'icon_bat':bat, 'icon_bigmon':lambda:monster(True), 'icon_bird':bird, 'icon_blob':blob, 'icon_bug':bug, 'icon_bulbasaur':bulbasaur,
 'icon_caterpillar':caterpillar, 'icon_charmander':charmander, 'icon_clefairy':fairy, 'icon_diglett':diglett, 'icon_egg':egg, 'icon_equine':equine,
 'icon_fighter':fighter, 'icon_fish':fish, 'icon_fox':fox, 'icon_geodude':geodude, 'icon_ghost':ghost, 'icon_gyarados':lambda:serpent(True),
 'icon_ho_oh':legendary_bird, 'icon_humanshape':lambda:fighter(True), 'icon_jellyfish':jellyfish, 'icon_jigglypuff':lambda:fairy(True),
 'icon_lapras':lapras, 'icon_lugia':lambda:legendary_bird(True), 'icon_monster':monster, 'icon_moth':moth, 'icon_oddish':oddish,
 'icon_pikachu':pikachu, 'icon_poliwag':poliwag, 'icon_serpent':serpent, 'icon_shell':shell, 'icon_slowpoke':slowpoke, 'icon_snorlax':snorlax,
 'chris_bike':lambda:bicycle_rider(), 'kris_bike':lambda:bicycle_rider(True),
 'battle_chikorita':chikorita, 'battle_cyndaquil':cyndaquil, 'battle_totodile':totodile, 'battle_pidgey':pidgey, 'battle_rattata':rattata, 'battle_sentret':sentret, 'battle_hoothoot':hoothoot,
 'icon_squirtle':squirtle, 'icon_staryu':staryu, 'icon_sudowoodo':sudowoodo, 'icon_unown':unown, 'icon_voltorb':lambda:ball(True),
}
# These source families intentionally share a recognizable subject but differ in
# presentation scale. Source identity is never collapsed in the Rust registry.
HEIGHTS={'chris_bike':1.48,'kris_bike':1.48,'battle_chikorita':1.00,'battle_cyndaquil':.91,'battle_totodile':1.02,'battle_pidgey':.84,'battle_rattata':.75,'battle_sentret':1.07,'battle_hoothoot':.93,'big_snorlax':1.48,'big_lapras':1.52,'big_onix':1.72,'fruit_tree':1.08,'poke_ball':.46,'boulder':.68,'rock':.43,
 'paper':.13,'pokedex':.16,'famicom':.40,'snes':.30,'n64':.43,'virtual_boy':.59,'gold_trophy':.63,'silver_trophy':.56,
 'surf':.98,'surfing_pikachu':1.00,'bird':.72,'fairy':.78,'monster':1.00,'dragon':1.12,'slowpoke':.70,'sudowoodo':.96,'entei':1.12,'raikou':1.14,'suicune':1.25}

def convert(v):return [round(v.x,6),round(v.z,6),round(-v.y,6)]
def canonical_primitive(primitive):
 # Blender's edit-mesh allocation order is not a source identity. Canonical
 # attribute and cyclic triangle order avoids allocation-order changes.
 # Quantized-and-renormalized normals also suppress sub-ULP Blender noise.
 p=primitive['positions'];n=primitive['normals']
 vertices=[]
 for i in range(0,len(p),3):
  normal=[round(v,3) for v in n[i:i+3]];length=math.sqrt(sum(v*v for v in normal));normal=[round(v/length,6) for v in normal]
  vertices.append(tuple(0.0 if abs(v)<.0000005 else v for v in p[i:i+3]+normal))
 unique=sorted(set(vertices));lookup={v:i for i,v in enumerate(unique)};remap=[lookup[v] for v in vertices];triangles=[]
 for i in range(0,len(primitive['indices']),3):
  t=tuple(remap[v] for v in primitive['indices'][i:i+3]);triangles.append(min(t,t[1:]+t[:1],t[2:]+t[:2]))
 return {'positions':[v for p in unique for v in p[:3]],'normals':[v for p in unique for v in p[3:]],'indices':[v for t in sorted(triangles) for v in t],'base_color':primitive['base_color']}

def canonical_primitives(primitives):
 return sorted((canonical_primitive(p) for p in primitives),key=lambda p:json.dumps(p,sort_keys=True,separators=(',',':')))

def export(name,objects):
 bpy.context.view_layer.update();primitives=[];tri_count=0
 for o in objects:
  if o.type!='MESH':continue
  me=o.data;me.calc_loop_triangles();pos=[];nor=[];idx=[];lookup={};nm=o.matrix_world.to_3x3().inverted().transposed()
  # Preserve optional fracture face colors as separate material primitives.
  for mi,mat in enumerate(me.materials):
   pos=[];nor=[];idx=[];lookup={}
   for tri in me.loop_triangles:
    if tri.material_index!=mi:continue
    for li in tri.loops:
     p=convert(o.matrix_world@me.vertices[me.loops[li].vertex_index].co);n=convert((nm@me.corner_normals[li].vector).normalized());key=tuple(p+n)
     if key not in lookup:lookup[key]=len(pos)//3;pos.extend(p);nor.extend(n)
     idx.append(lookup[key])
   if idx:primitives.append({'positions':pos,'normals':nor,'indices':idx,'base_color':[round(v,6) for v in mat.diffuse_color]});tri_count+=len(idx)//3
 data={'name':name,'version':1,'coordinate_system':'+Y up; front +Z; floor-centered root','primitives':canonical_primitives(primitives)}
 if name in ('battle_cyndaquil','battle_totodile'):
  sys.path.insert(0,str(Path(__file__).resolve().parent))
  from cyndaquil_glb import export_cyndaquil
  from totodile_glb import export_totodile
  blob=(export_cyndaquil if name=='battle_cyndaquil' else export_totodile)(data)
  (OUT/(name+'.glb')).write_bytes(blob)
 else:(OUT/(name+'.mesh.json')).write_text(json.dumps(data,separators=(',',':')))
 print(f'{name}: {tri_count} triangles, {len(primitives)} primitives')

def normalize(name,objects):
 bpy.context.view_layer.update();corners=[o.matrix_world@v.co for o in objects for v in o.data.vertices]
 lo=Vector(tuple(min(c[a] for c in corners) for a in range(3)));hi=Vector(tuple(max(c[a] for c in corners) for a in range(3)))
 height=HEIGHTS.get(name,.82);scale=height/(hi.z-lo.z)
 # Paper/book retain sensible full width instead of height-dependent magnification.
 if name in ('paper','pokedex'):scale=1
 center=Vector(((lo.x+hi.x)/2,(lo.y+hi.y)/2,lo.z))
 for o in objects:o.location=(o.location-center)*scale;o.scale*=scale

def sudowoodo_runtime_lod(name,objects):
 # Preserve editable detailed anatomy, then produce bounded game geometry.
 # Stable topology removes allocator-order ties before QEM simplification.
 import bmesh
 detailed=bpy.data.collections.new(name+' / detailed authoring source (hidden)')
 SCENE.collection.children.link(detailed);copies=[]
 for o in objects:
  source=o.copy();source.data=o.data.copy();source.name='AUTHORING / '+o.name
  detailed.objects.link(source);copies.append(source)
  if o.type!='MESH':continue
  if o.get('trunk_surface_inlay'):
   # Refit to the final runtime trunk, so color rests flush on its facets.
   core=next(p for p in objects if 'Continuous leaning trunk' in p.name)
   bpy.context.view_layer.update();layer=len(o.data.vertices)//2
   for i,v in enumerate(o.data.vertices):
    hit,point,normal,index=core.ray_cast(Vector((v.co.x,-1,v.co.z)),Vector((0,1,0)))
    if not hit:raise ValueError('Bark inlay missed the runtime trunk')
    v.co.y=point.y-.0015+(.005 if i>=layer else 0)
  me=o.data;coords=[tuple(round(v,7) for v in vert.co) for vert in me.vertices]
  ordered=sorted(set(coords));lookup={v:i for i,v in enumerate(ordered)};remap=[lookup[v] for v in coords];faces=[]
  for f in me.polygons:
   indices=tuple(remap[i] for i in f.vertices)
   if len(set(indices))<3:continue
   indices=min(indices[i:]+indices[:i] for i in range(len(indices)))
   faces.append((indices,f.use_smooth,f.material_index))
  faces.sort();stable=bpy.data.meshes.new(me.name+' / stable topology');stable.from_pydata(ordered,[],[f[0] for f in faces]);stable.update()
  for mat in me.materials:stable.materials.append(mat)
  for f,old in zip(stable.polygons,faces):f.use_smooth=old[1];f.material_index=old[2]
  o.data=stable;stable.calc_loop_triangles();count=len(stable.loop_triangles)
  ratio=.095 if 'Continuous leaning trunk' in o.name else (.52 if 'green ball' in o.name else 1)
  if ratio<1 and count>=100:
   bpy.context.view_layer.objects.active=o
   m=o.modifiers.new('Game resolution / sculpted silhouette','DECIMATE');m.ratio=ratio;m.use_collapse_triangulate=True;bpy.ops.object.modifier_apply(modifier=m.name)
  # Explicit winding and stable flat normals on delicate inset edges prevent
  # a smoothed corner from pointing back into the face after triangulation.
  me=o.data;bm=bmesh.new();bm.from_mesh(me)
  bmesh.ops.triangulate(bm,faces=list(bm.faces),quad_method='FIXED',ngon_method='EAR_CLIP')
  bmesh.ops.recalc_face_normals(bm,faces=list(bm.faces))
  if bm.calc_volume(signed=True)<0:bmesh.ops.reverse_faces(bm,faces=list(bm.faces))
  bm.to_mesh(me);bm.free();me.update()
  for attempt in range(3):
   bad=[]
   for f in me.polygons:
    if f.use_smooth and f.normal.dot(sum((me.corner_normals[i].vector for i in f.loop_indices),Vector()))<.000001:bad.append(f)
   if not bad:break
   for f in bad:f.use_smooth=False
   me.update()
  o['runtime_lod']='Bounded continuous sculpture; detailed source in hidden collection'
 detailed.hide_render=True;detailed.hide_viewport=True
 normalize(name,objects)
 return copies

# Keep editable scenes bounded. Operators and dependency evaluation otherwise
# become quadratic for thousands of independently editable small creature parts.
# Each batch is self-contained, original geometry; runtime exports are unchanged.
BATCH_SIZE=12
ONLY=next((set(a.split('=',1)[1].split(',')) for a in ARGS if a.startswith('--only=')),None)
if ONLY and not ONLY.issubset(BUILDERS):raise ValueError('Unknown --only asset identity')
MODELS=[(name,builder) for name,builder in BUILDERS.items() if ONLY is None or name in ONLY]
for batch_start in range(0,len(MODELS),BATCH_SIZE):
 selected=next((int(a.split('=',1)[1]) for a in ARGS if a.startswith('--batch=')),None)
 if selected is not None and selected != batch_start//BATCH_SIZE+1:continue
 bpy.ops.object.select_all(action='SELECT');bpy.ops.object.delete(use_global=False)
 for c in list(bpy.data.collections):bpy.data.collections.remove(c)
 COLLECTIONS=[]
 for name,builder in MODELS[batch_start:batch_start+BATCH_SIZE]:
  COL=bpy.data.collections.new(name);SCENE.collection.children.link(COL);bpy.ops.object.select_all(action='DESELECT');builder();objects=list(COL.objects);normalize(name,objects)
  detailed=sudowoodo_runtime_lod(name,objects) if name in ('sudowoodo','icon_sudowoodo') else []
  export(name,objects)
  root=bpy.data.objects.new(name+' / floor root',None);COL.objects.link(root);root.empty_display_size=.05;root['source_family']=name
  for o in objects+detailed:o.parent=root
  COLLECTIONS.append((name,root))
 COL=bpy.data.collections.new('Studio');SCENE.collection.children.link(COL)
 for i,(name,root) in enumerate(COLLECTIONS):root.location=((i%4-1.5)*1.85,(i//4)*2.05,0)
 if SCENE.world is None:SCENE.world=bpy.data.worlds.new('Soft studio')
 world=SCENE.world;world.use_nodes=True;world.node_tree.nodes['Background'].inputs['Color'].default_value=(.45,.52,.58,1);world.node_tree.nodes['Background'].inputs['Strength'].default_value=.7
 box('Studio floor',(0,2.0,-.075),(11,10,.10),'white',0)
 for p,power,size in [((-4,-5,9),1100,7),((6,3,8),900,6),((-5,8,8),1200,6)]:
  bpy.ops.object.light_add(type='AREA',location=p);o=bpy.context.object;o.data.energy=power;o.data.shape='DISK';o.data.size=size;o.rotation_euler=(Vector((0,2,0))-o.location).to_track_quat('-Z','Y').to_euler()
 bpy.ops.object.camera_add(location=(5,-9,12));cam=bpy.context.object;cam.rotation_euler=(Vector((0,2,.25))-cam.location).to_track_quat('-Z','Y').to_euler();cam.data.type='ORTHO';cam.data.ortho_scale=10.2;SCENE.camera=cam
 SCENE.render.engine='CYCLES';SCENE.cycles.samples=24;SCENE.cycles.use_denoising=True;SCENE.render.resolution_x=1600;SCENE.render.resolution_y=1400;SCENE.render.resolution_percentage=100
 SCENE.view_settings.view_transform='AgX';SCENE.render.image_settings.file_format='PNG'
 batch=batch_start//BATCH_SIZE+1
 suffix=f'{batch:02d}' if ONLY is None else '-'.join(name for name,_ in COLLECTIONS)
 SCENE.render.filepath=str(OUT/f'actor-props-{suffix}.png');SCENE['authorship']='Original editable authored geometry. No extracted textures, pixels, ROM data, imported third-party models, or runtime state.'
 bpy.ops.wm.save_as_mainfile(filepath=str(OUT/f'actor-props-{suffix}.blend'),compress=True)
 if '--skip-preview' not in ARGS:bpy.ops.render.render(write_still=True)
 print(f'Finished bounded source batch {batch}',flush=True)
